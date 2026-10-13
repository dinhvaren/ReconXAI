import { chromium } from '../../apps/desktop/node_modules/playwright/index.mjs';
import { createServer } from 'node:http';
import { createServer as createNetServer } from 'node:net';
import { spawn, execFile, spawnSync } from 'node:child_process';
import { promisify } from 'node:util';
import { mkdir, writeFile } from 'node:fs/promises';
import { createWriteStream } from 'node:fs';
import { fileURLToPath } from 'node:url';
import path from 'node:path';
import assert from 'node:assert/strict';

const root=path.resolve(path.dirname(fileURLToPath(import.meta.url)),'../..');
const artifact=path.join(root,'artifacts',`native-${Date.now()}`);
await mkdir(artifact,{recursive:true});
const exec=promisify(execFile);
const desktop=process.env.RECONX_DESKTOP_EXE??path.join(root,'target','debug','reconx-desktop.exe');
const cli=process.env.RECONX_CLI_EXE??path.join(root,'target','debug','reconx.exe');
const env={...process.env,RECONX_DATA_DIR:artifact};
const calls=[];
const mock=createServer(async(req,res)=>{
  let body='';for await(const chunk of req)body+=chunk;
  calls.push({method:req.method,path:req.url});
  res.setHeader('Content-Type','application/json');
  if(req.method==='GET'&&req.url==='/v1/models')res.end(JSON.stringify({data:[{id:'fixture-model'}]}));
  else if(req.method==='POST'&&req.url==='/v1/chat/completions'){
    const value=JSON.parse(body);assert.equal(value.model,'fixture-model');assert.equal(value.tools,undefined);
    if(value.stream){
      res.setHeader('Content-Type','text/event-stream');
      if(value.messages.at(-1).content==='Cancellation fixture'){
        res.write('data: '+JSON.stringify({choices:[{delta:{content:'Cancellation fixture partial'}}]})+'\n\n');
        await new Promise(resolve=>setTimeout(resolve,1000));
        if(!res.destroyed)res.end('data: [DONE]\n\n'); return;
      }
      res.write('data: '+JSON.stringify({choices:[{delta:{content:'Local fixture response: '}}]})+'\n\n');
      await new Promise(resolve=>setTimeout(resolve,60));
      res.write('data: '+JSON.stringify({choices:[{delta:{content:'evidence explanation.'},finish_reason:'stop'}],usage:{prompt_tokens:8,completion_tokens:6}})+'\n\n');
      res.end('data: [DONE]\n\n'); return;
    }
    res.end(JSON.stringify({choices:[{message:{content:'Local fixture response: evidence explanation.'}}],usage:{prompt_tokens:8,completion_tokens:6}}));
  } else {res.statusCode=404;res.end('{}');}
});
await new Promise(resolve=>mock.listen(0,'127.0.0.1',resolve));
const gateway=`http://127.0.0.1:${mock.address().port}/v1`;
let child,browser,currentPage;
const delay=ms=>new Promise(resolve=>setTimeout(resolve,ms));
async function launch(){
  const listener=createNetServer();await new Promise(resolve=>listener.listen(0,'127.0.0.1',resolve));const port=listener.address().port;await new Promise(resolve=>listener.close(resolve));
  const logfile=createWriteStream(path.join(artifact,`desktop-${port}.log`));
  child=spawn(desktop,[],{cwd:root,env:{...env,WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS:`--remote-debugging-port=${port}`},windowsHide:true,stdio:['ignore','pipe','pipe']});
  child.stdout.pipe(logfile);child.stderr.pipe(logfile);
  let ready=false;
  for(let i=0;i<120;i++){
    if(child.exitCode!==null)throw new Error(`Desktop exited with ${child.exitCode}; inspect logs in ${artifact}`);
    try{const response=await fetch(`http://127.0.0.1:${port}/json/version`);if(response.ok){ready=true;break;}}catch{}
    await delay(250);
  }
  assert(ready,'WebView2 debugging endpoint did not become ready');
  browser=await chromium.connectOverCDP(`http://127.0.0.1:${port}`);
  let page;
  for(let i=0;i<80;i++){page=browser.contexts().flatMap(c=>c.pages())[0];if(page)break;await delay(250);}
  assert(page,'No native WebView2 page');
  await page.getByText('Workspace overview',{exact:true}).waitFor({timeout:30000});
  currentPage=page; return page;
}
async function stop(){
  if(browser){await browser.close();browser=null;}
  if(child&&child.exitCode===null){
    if(process.platform==='win32')spawnSync('taskkill',['/PID',String(child.pid),'/T','/F'],{windowsHide:true,stdio:'ignore'});
    else child.kill();
    await delay(500);
  }
}
const results=[];
try{
  let page=await launch();
  assert.equal(await page.locator('html').getAttribute('data-theme'),'dark');
  await page.getByRole('button',{name:'Switch to light mode',exact:true}).click();
  await page.locator('html[data-theme="light"]').waitFor();
  assert.equal(JSON.parse((await exec(cli,['--json','settings','show'],{env,windowsHide:true})).stdout).theme,'light');
  await page.screenshot({path:path.join(artifact,'dashboard-light.png'),fullPage:true});
  await page.getByRole('button',{name:'Projects',exact:true}).click();
  await page.getByLabel('Project name',{exact:true}).fill('Native desktop project');
  await page.getByLabel('Description',{exact:true}).fill('Created in the real Tauri window');
  await page.getByRole('button',{name:'Create project',exact:true}).click();
  await page.getByRole('status').filter({hasText:'Project created'}).waitFor();
  await page.getByRole('heading',{name:'Native desktop project',exact:true}).waitFor();
  const listed=JSON.parse((await exec(cli,['--json','project','list'],{env,windowsHide:true})).stdout);
  assert.equal(listed[0].name,'Native desktop project');
  results.push('Desktop project is visible to the CLI through the shared database');
  await exec(cli,['project','create','CLI-created project','--json'],{env,windowsHide:true});
  await page.getByRole('button',{name:/Refresh/}).click();
  await page.getByRole('heading',{name:'CLI-created project',exact:true}).waitFor();
  results.push('CLI project is visible to the native desktop');
  await page.getByRole('button',{name:'Settings',exact:true}).click();
  await page.getByLabel('Appearance',{exact:true}).selectOption('system');
  await page.getByRole('button',{name:'Save preferences',exact:true}).click();
  await page.getByRole('status').filter({hasText:'Preferences saved'}).waitFor();
  await page.emulateMedia({colorScheme:'dark'});
  await page.locator('html[data-theme="dark"]').waitFor();
  await page.emulateMedia({colorScheme:'light'});
  await page.locator('html[data-theme="light"]').waitFor();
  await page.getByLabel('Appearance',{exact:true}).selectOption('light');
  await page.getByLabel('Security mode').selectOption('lab_research');
  await page.getByRole('button',{name:'Save preferences',exact:true}).click();
  await page.getByRole('status').filter({hasText:'Preferences saved'}).waitFor();
  assert.equal(JSON.parse((await exec(cli,['--json','settings','show'],{env,windowsHide:true})).stdout).security_mode,'lab_research');
  await page.getByRole('button',{name:'+ Add provider',exact:true}).click();
  await page.getByLabel('Configuration name',{exact:true}).fill('Local fixture gateway');
  await page.getByLabel('Provider protocol').selectOption('compatible');
  await page.getByLabel('Gateway base URL',{exact:true}).fill(gateway);
  await page.getByLabel('Model ID',{exact:true}).fill('fixture-model');
  await page.getByLabel('Requires secure API key',{exact:true}).uncheck();
  await page.getByRole('button',{name:'Save provider',exact:true}).click();
  await page.getByRole('status').filter({hasText:'Provider configuration saved'}).waitFor();
  await page.getByRole('button',{name:'Test connection',exact:true}).click();
  await page.getByText(/Local fixture gateway: connected/).waitFor();
  const provider=JSON.parse((await exec(cli,['--json','ai','providers'],{env,windowsHide:true})).stdout)[0];
  await writeFile(path.join(artifact,'prompt.txt'),'Explain local evidence');
  const reply=JSON.parse((await exec(cli,['--json','ai','chat',provider.id,'--prompt-file',path.join(artifact,'prompt.txt')],{env,windowsHide:true})).stdout);
  assert.equal(reply.text,'Local fixture response: evidence explanation.');
  const streamReply=JSON.parse((await exec(cli,['--json','ai','chat',provider.id,'--prompt-file',path.join(artifact,'prompt.txt'),'--stream'],{env,windowsHide:true})).stdout);
  assert.equal(streamReply.text,'Local fixture response: evidence explanation.');
  await page.getByRole('button',{name:'AI Workspace',exact:true}).click();
  await page.getByLabel('Message',{exact:true}).fill('Explain local evidence');
  await page.getByRole('button',{name:'Send message ↗',exact:true}).click();
  await page.getByText('Local fixture response: evidence explanation.',{exact:true}).waitFor();
  results.push('Provider discovery and text requests work from native desktop and CLI against a local mock HTTP gateway');
  await page.screenshot({path:path.join(artifact,'ai-workspace.png'),fullPage:true});
  await page.getByLabel('Message',{exact:true}).fill('Cancellation fixture');
  await page.getByRole('button',{name:'Send message ↗',exact:true}).click();
  await page.getByText('Cancellation fixture partial',{exact:true}).waitFor();
  await page.getByRole('button',{name:'Cancel request',exact:true}).click();
  await page.getByRole('alert').filter({hasText:'AI request cancelled'}).waitFor();
  const audit=JSON.parse((await exec(cli,['--json','audit'],{env,windowsHide:true})).stdout);
  assert(audit.some(e=>e.action==='ai.stream'&&e.outcome==='cancelled'));
  results.push('Native streaming cancellation stops the request and records a cancelled audit outcome visible to the CLI');
  await stop();
  page=await launch();
  await page.locator('html[data-theme="light"]').waitFor();
  await page.getByRole('button',{name:'Projects',exact:true}).click();
  await page.getByRole('heading',{name:'Native desktop project',exact:true}).waitFor();
  await page.getByRole('heading',{name:'CLI-created project',exact:true}).waitFor();
  await page.screenshot({path:path.join(artifact,'projects.png'),fullPage:true});
  await page.getByRole('button',{name:'Settings',exact:true}).click();
  assert.equal(await page.getByLabel('Appearance',{exact:true}).inputValue(),'light');
  await page.screenshot({path:path.join(artifact,'settings-light.png'),fullPage:true});
  assert.equal(await page.getByLabel('Security mode').inputValue(),'lab_research');
  await page.getByText('Local fixture gateway',{exact:true}).waitFor();
  results.push('Projects, provider configuration and settings survive a complete native application restart');
  await page.getByRole('button',{name:'Dashboard',exact:true}).click();
  await page.getByRole('button',{name:'Switch to dark mode',exact:true}).click();
  await page.locator('html[data-theme="dark"]').waitFor();
  results.push('Dark/Light quick switching persists to CLI and across native restart; System responds to OS color-scheme changes');
  await page.screenshot({path:path.join(artifact,'dashboard.png'),fullPage:true});
  if(process.env.RECONX_E2E_TOOL_INSTALL==='1'){
    await page.getByRole('button',{name:'Tool Manager',exact:true}).click();
    await page.getByRole('button',{name:'Review install plan',exact:true}).click();
    await page.getByRole('heading',{name:'Installation plan · ripgrep',exact:true}).waitFor();
    await page.getByRole('button',{name:'Install verified release',exact:true}).click();
    await page.getByRole('status').filter({hasText:'Official release downloaded, verified and installed'}).waitFor({timeout:120000});
    const tools=JSON.parse((await exec(cli,['--json','tools','list'],{env,windowsHide:true})).stdout);
    assert.equal(tools[0].status,'installed');assert.equal(tools[0].integrity_verified,true);
    await page.getByRole('button',{name:'Health check',exact:true}).click();
    await page.getByRole('status').filter({hasText:'Pinned executable integrity and version verified'}).waitFor();
    const health=JSON.parse((await exec(cli,['--json','tools','doctor'],{env,windowsHide:true})).stdout);
    assert.equal(health.status,'tested');
    await page.screenshot({path:path.join(artifact,'tool-manager.png'),fullPage:true});
    await exec(cli,['--json','tools','uninstall','ripgrep'],{env,windowsHide:true});
    await page.getByRole('button',{name:'Refresh inventory',exact:true}).click();
    await page.getByText('not installed',{exact:true}).waitFor();
    results.push('Native GUI installs the official pinned ripgrep release, verifies archive and executable hashes, checks its version, shares inventory with CLI, and uninstalls it');
  }
  await writeFile(path.join(artifact,'results.json'),JSON.stringify({status:'passed',platform:process.platform,desktop,cli,results,calls},null,2));
  console.log(JSON.stringify({status:'passed',artifact,checks:results},null,2));
}catch(error){
  if(currentPage){await currentPage.screenshot({path:path.join(artifact,'failure.png')}).catch(()=>{}); await writeFile(path.join(artifact,'failure.html'),await currentPage.content().catch(()=>''));}
  await writeFile(path.join(artifact,'results.json'),JSON.stringify({status:'failed',message:String(error),results,calls},null,2));
  throw error;
}finally{await stop();await new Promise(resolve=>mock.close(resolve));}
