// Real Tauri/WebView2 + OS credential-store acceptance. All inference is loopback.
import {chromium} from '../../apps/desktop/node_modules/playwright/index.mjs';
import {createServer} from 'node:http';
import {createServer as netServer} from 'node:net';
import {spawn,spawnSync,execFile} from 'node:child_process';
import {promisify} from 'node:util';
import {mkdir,readFile,writeFile} from 'node:fs/promises';
import {createWriteStream} from 'node:fs';
import {fileURLToPath} from 'node:url';
import path from 'node:path';
import assert from 'node:assert/strict';

const root=path.resolve(path.dirname(fileURLToPath(import.meta.url)),'../..');
const artifact=path.join(root,'artifacts',`native-ai-providers-${Date.now()}`);await mkdir(artifact,{recursive:true});
const exec=promisify(execFile);
const cli=process.env.RECONX_CLI_EXE??path.join(root,'target/release/reconx.exe');
const desktop=process.env.RECONX_DESKTOP_EXE??path.join(root,'target/release/reconx-desktop.exe');
const env={...process.env,RECONX_DATA_DIR:artifact,RECONX_NATIVE_TEST_HIDDEN:'1',WEBVIEW2_USER_DATA_FOLDER:path.join(artifact,'webview')};
const delay=ms=>new Promise(resolve=>setTimeout(resolve,ms));
const calls=[];let rejectModel=false;
const mock=createServer(async(req,res)=>{
  let raw='';for await(const chunk of req)raw+=chunk;
  const body=raw?JSON.parse(raw):{};
  calls.push({path:req.url,method:req.method,model:body.model??null,custom_auth:req.headers['x-private-key']==='synthetic-native-fixture-key',tenant:req.headers['x-tenant']==='synthetic-private-header'}); // Never log headers or raw prompts.
  res.setHeader('Content-Type','application/json');
  if(req.url.endsWith('/models'))return res.end(JSON.stringify({data:[{id:'fixture-one'},{id:'fixture-two'}]}));
  if(rejectModel){res.statusCode=401;return res.end(JSON.stringify({error:'synthetic-authentication-failure'}));}
  if(req.url.includes('/responses'))return res.end(JSON.stringify({output:[{content:[{type:'output_text',text:'Loopback inference OK'}]}]}));
  if(req.url.includes('/messages'))return res.end(JSON.stringify({content:[{type:'text',text:'Loopback inference OK'}]}));
  if(req.url.includes(':generateContent'))return res.end(JSON.stringify({candidates:[{content:{parts:[{text:'Loopback inference OK'}]}}]}));
  if(req.url.includes('/chat/completions'))return res.end(JSON.stringify({choices:[{message:{content:`Actual selected model: ${body.model}`}}]}));
  res.statusCode=404;res.end('{}');
});
await new Promise(resolve=>mock.listen(0,'127.0.0.1',resolve));const base=`http://127.0.0.1:${mock.address().port}`;
let child,browser,page;
async function launch(){
  const portServer=netServer();await new Promise(r=>portServer.listen(0,'127.0.0.1',r));const port=portServer.address().port;await new Promise(r=>portServer.close(r));
  const log=createWriteStream(path.join(artifact,`desktop-${port}.log`));
  child=spawn(desktop,[],{cwd:root,env:{...env,WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS:`--remote-debugging-port=${port}`},windowsHide:true,stdio:['ignore','pipe','pipe']});child.stdout.pipe(log);child.stderr.pipe(log);
  let connected=false;
  for(let i=0;i<120;i++){assert.equal(child.exitCode,null,'Native application exited during startup');try{if((await fetch(`http://127.0.0.1:${port}/json/version`)).ok){connected=true;break;}}catch{}await delay(250);}
  assert(connected,'Native WebView2 debugging endpoint unavailable');browser=await chromium.connectOverCDP(`http://127.0.0.1:${port}`);
  for(let i=0;i<80;i++){page=browser.contexts().flatMap(c=>c.pages())[0];if(page)break;await delay(250);}assert(page);await page.getByRole('heading',{name:'Workspace overview',exact:true}).waitFor({timeout:30000});
}
async function stop(){if(browser){await browser.close();browser=null;}if(child&&child.exitCode===null){spawnSync('taskkill',['/PID',String(child.pid),'/T','/F'],{windowsHide:true,stdio:'ignore'});await delay(500);}}
async function command(args){const result=await exec(cli,['--json',...args],{cwd:root,env,windowsHide:true,maxBuffer:4*1024*1024});return JSON.parse(result.stdout);}
async function settings(){await page.getByRole('button',{name:'Settings',exact:true}).click();await page.getByRole('button',{name:'Add AI Provider',exact:true}).waitFor();}
async function setup(vendor,{gateway,name,model,key=true}={}){
  await page.getByRole('button',{name:'Add AI Provider',exact:true}).click();const dialog=page.getByRole('dialog');await dialog.getByLabel('Select Provider',{exact:true}).selectOption(vendor);
  assert(await dialog.getByLabel('Select Model',{exact:true}).isVisible(),'Model dropdown is visible before API key entry');
  if((gateway||name||model)&&!(await dialog.getByLabel('Configuration name',{exact:true}).isVisible())){await dialog.getByText('Advanced Settings',{exact:true}).click();}
  if(gateway)await dialog.getByLabel('Gateway base URL',{exact:true}).fill(gateway);
  if(name)await dialog.getByLabel('Configuration name',{exact:true}).fill(name);
  if(model){await dialog.getByLabel('Enter custom model ID',{exact:true}).check();await dialog.getByLabel('Model ID',{exact:true}).fill(model);}
  if(key)await dialog.getByLabel('API key',{exact:true}).fill('synthetic-native-fixture-key');
  else if(await dialog.getByLabel('Requires secure API key',{exact:true}).isChecked())await dialog.getByLabel('Requires secure API key',{exact:true}).uncheck();
  if(!(await dialog.getByLabel('Select Model',{exact:true}).inputValue())){await dialog.getByRole('button',{name:'Refresh Models',exact:true}).click();await dialog.getByRole('option',{name:'fixture-one',exact:true}).waitFor({state:'attached'});}
  await dialog.getByRole('button',{name:'Save provider',exact:true}).click();await dialog.waitFor({state:'hidden'});await page.getByRole('status').filter({hasText:'AI provider saved securely'}).waitFor();
}
const checkpoints=[];
try{
  await launch();await settings();
  const catalog=await command(['ai','catalog']);assert.equal(catalog.length,24);
  for(const vendor of ['deepseek','anthropic','google','openai','zai','xai']){
    await setup(vendor);const profile=(await command(['ai','profiles','list'])).find(p=>p.vendor_id===vendor);
    assert(profile);assert.equal(profile.credential_state,'stored');assert.equal(profile.connection,null);const preset=catalog.find(p=>p.id===vendor);assert.equal(profile.provider.base_url,preset.base_url);assert.equal(profile.provider.model,preset.models[0].id);
  }
  assert.equal(calls.length,0,'Save must not initiate inference or model discovery');checkpoints.push('Six official providers save through native UI with verified presets and OS-stored keys; no network calls on Save');
  await page.screenshot({path:path.join(artifact,'configured-providers-reconx.png'),fullPage:true});
  for(const vendor of ['openai','anthropic','google']){
    await setup(vendor,{gateway:`${base}/${vendor}`,name:`Loopback ${vendor}`});
    const row=page.locator('article.provider-row').filter({has:page.getByRole('heading',{name:`Loopback ${vendor}`,exact:true})});await row.getByRole('button',{name:'Test connection',exact:true}).click();await row.getByText(/Connected \(last test\)/).waitFor();
  }
  assert(calls.some(c=>c.path==='/openai/responses'));assert(calls.some(c=>c.path==='/anthropic/messages'));assert(calls.some(c=>c.path.includes('/google/models/')&&c.path.endsWith(':generateContent')));checkpoints.push('Actual selected-model inference checks use distinct OpenAI Responses, Anthropic Messages and Gemini generateContent protocols');
  await setup('custom',{gateway:`${base}/custom`,name:'Local native gateway',model:'fixture-one'});
  let custom=(await command(['ai','profiles','list'])).find(p=>p.provider.name==='Local native gateway');assert(custom);const id=custom.provider.id;
  let row=page.locator('article.provider-row').filter({has:page.getByRole('heading',{name:/^Local native gateway/,})});await row.getByRole('button',{name:'Manage',exact:true}).click();
  let dialog=page.getByRole('dialog');assert.equal(await dialog.getByLabel('API key',{exact:true}).inputValue(),'');await dialog.getByRole('button',{name:'Refresh Models',exact:true}).click();await dialog.getByRole('option',{name:'fixture-two',exact:true}).waitFor({state:'attached'});assert.equal(await dialog.getByLabel('Select Model',{exact:true}).inputValue(),'fixture-one');await dialog.getByLabel('Select Model',{exact:true}).selectOption('fixture-two');await dialog.getByRole('button',{name:'Save provider',exact:true}).click();await dialog.waitFor({state:'hidden'});
  custom=(await command(['ai','profiles','list'])).find(p=>p.provider.id===id);assert.equal(custom.provider.model,'fixture-two');assert.equal(custom.connection,null);
  await command(['ai','profiles','set-default',id]);await page.getByRole('button',{name:'AI Workspace',exact:true}).click();await page.getByRole('button',{name:'Refresh',exact:true}).click();
  await page.getByLabel('AI provider',{exact:true}).selectOption(id);await page.getByLabel('Stream output',{exact:true}).uncheck();await page.getByLabel('Message',{exact:true}).fill('Show which model actually received this request');await page.getByRole('button',{name:/Send message/}).click();await page.getByText('Actual selected model: fixture-two',{exact:true}).waitFor();
  assert(calls.some(c=>c.path==='/custom/chat/completions'&&c.model==='fixture-two'));checkpoints.push('Custom gateway discovery preserves selection, cached models refresh, replacing the model changes actual native AI Workspace requests without restart');
  await page.screenshot({path:path.join(artifact,'ai-workspace-selected-model.png'),fullPage:true});
  await settings();row=page.locator('article.provider-row').filter({has:page.getByRole('heading',{name:/^Local native gateway/,})});await row.getByRole('button',{name:'Manage',exact:true}).click();dialog=page.getByRole('dialog');await dialog.getByLabel('Authentication header',{exact:true}).fill('x-private-key');await dialog.getByRole('combobox',{name:/^Authentication prefix/}).selectOption('');await dialog.getByLabel('Replace custom headers',{exact:true}).check();await dialog.getByLabel('Custom headers (JSON)',{exact:true}).fill(JSON.stringify({'X-Tenant':'synthetic-private-header'}));await dialog.getByLabel('Enable streaming',{exact:true}).uncheck();await dialog.getByRole('button',{name:'Save provider',exact:true}).click();await dialog.waitFor({state:'hidden'});await row.getByRole('button',{name:'Test connection',exact:true}).click();await row.getByText(/Connected \(last test\)/).waitFor();assert(calls.some(c=>c.path==='/custom/chat/completions'&&c.custom_auth&&c.tenant));const secure=(await command(['ai','profiles','list'])).find(p=>p.provider.id===id);assert.equal(secure.options.auth_header,'x-private-key');assert.deepEqual(secure.header_names,['X-Tenant']);assert(!JSON.stringify(secure).includes('synthetic-private-header'));checkpoints.push('Native custom authentication and private headers reach the actual loopback server while metadata exposes header names only');

  await settings();row=page.locator('article.provider-row').filter({has:page.getByRole('heading',{name:/^Local native gateway/,})});rejectModel=true;await row.getByRole('button',{name:'Test connection',exact:true}).click();await row.getByText('Connection test failed',{exact:false}).waitFor();assert.equal((await command(['ai','profiles','list'])).find(p=>p.provider.id===id).connection.error_code,'AI_AUTHENTICATION_FAILED');rejectModel=false;
  await row.getByRole('button',{name:'Test connection',exact:true}).click();await row.getByText(/Connected \(last test\)/).waitFor();checkpoints.push('Authentication failure persists as Failed, can be retried explicitly, and never exposes provider response bodies');
  await page.getByText('Agent model routing',{exact:true}).click();await page.getByLabel('Use Default Model for All Agents',{exact:true}).uncheck();await page.getByLabel('orchestrator agent model',{exact:true}).selectOption(id);await page.getByRole('button',{name:'Save agent routing',exact:true}).click();await page.getByRole('status').filter({hasText:'Agent routing saved'}).waitFor();assert.equal((await command(['ai','routing','resolve','orchestrator'])).id,id);checkpoints.push('Native agent routing saves and the CLI resolves the same profile');
  for(const theme of ['Light','Dark','ReconX']){
    await page.getByRole('button',{name:'Topbar appearance',exact:true}).click();await page.getByRole('menuitemradio',{name:new RegExp('^'+theme)}).click();await page.locator(`html[data-theme="${theme.toLowerCase()}"]`).waitFor();await page.getByRole('heading',{name:'AI Providers',exact:true}).evaluate(el=>el.scrollIntoView({block:'start'}));await page.screenshot({path:path.join(artifact,`ai-settings-${theme.toLowerCase()}.png`),fullPage:true});
  }
  const before=await command(['ai','profiles','list']);await stop();await launch();await settings();const after=await command(['ai','profiles','list']);assert.equal(after.length,before.length);assert.equal(after.find(p=>p.provider.id===id).connection.state,'connected');assert.equal(after.find(p=>p.provider.id===id).credential_state,'stored');assert.equal((await command(['ai','routing','resolve','orchestrator'])).id,id);checkpoints.push('Native restart preserves provider profiles, OS credentials, selected model, successful test metadata and agent routes');
  row=page.locator('article.provider-row').filter({has:page.getByRole('heading',{name:/^Local native gateway/,})});await row.getByRole('button',{name:'Test connection',exact:true}).click();await row.getByText(/Connected \(last test\)/).waitFor();assert(calls.at(-1).custom_auth&&calls.at(-1).tenant);
  await setup('lmstudio',{gateway:`${base}/local`,name:'Keyless local fixture',key:false});
  const local=(await command(['ai','profiles','list'])).find(p=>p.provider.name==='Keyless local fixture');assert.equal(local.provider.requires_key,false);assert.equal(local.provider.model,'fixture-one');assert.equal(local.credential_state,'not_required');checkpoints.push('Keyless local inference profile discovers server models and saves without a cloud API key');
  row=page.locator('article.provider-row').filter({has:page.getByRole('heading',{name:'Keyless local fixture',exact:true})});await row.getByRole('button',{name:'Remove',exact:true}).click();dialog=page.getByRole('dialog',{name:'Remove AI model profile',exact:true});await dialog.getByRole('button',{name:'Confirm removal',exact:true}).click();await dialog.waitFor({state:'hidden'});assert(!(await command(['ai','profiles','list'])).some(p=>p.provider.id===local.provider.id));checkpoints.push('Desktop Remove action deletes the selected model profile through the shared service');
  row=page.locator('article.provider-row').filter({has:page.getByRole('heading',{name:/^Local native gateway/})});await row.getByRole('button',{name:'Manage',exact:true}).click();dialog=page.getByRole('dialog');await dialog.getByRole('button',{name:'Remove stored API key',exact:true}).click();await dialog.waitFor({state:'hidden'});const noKey=(await command(['ai','profiles','list'])).find(p=>p.provider.id===id);assert.equal(noKey.credential_state,'missing');assert.equal(noKey.connection,null);await page.getByRole('button',{name:'AI Workspace',exact:true}).click();const unavailable=page.getByLabel('AI provider',{exact:true}).locator(`option[value="${id}"]`);for(let i=0;i<40&&!await unavailable.isDisabled();i++)await delay(50);assert(await unavailable.isDisabled());assert.equal(await page.getByText('Ready for a text request',{exact:true}).count(),0);checkpoints.push('Removing a stored API key invalidates connection status and disables that model in AI Workspace');

}catch(error){
  if(page&&!page.isClosed())await page.screenshot({path:path.join(artifact,'failure.png'),fullPage:true}).catch(()=>{});
  await writeFile(path.join(artifact,'failure.txt'),String(error));throw error;
}finally{
  await stop();
  const profiles=await command(['ai','providers']).catch(()=>[]);
  for(const p of profiles)await command(['ai','delete',p.id]).catch(()=>{});
  await new Promise(resolve=>mock.close(resolve));
  await writeFile(path.join(artifact,'results.json'),JSON.stringify({checkpoints,calls},null,2));
  console.log(JSON.stringify({artifact,checkpoints:checkpoints.length},null,2));
}
