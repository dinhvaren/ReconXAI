import { chromium } from '../../apps/desktop/node_modules/playwright/index.mjs';
import { createServer } from 'node:net';
import { spawn, execFile, spawnSync } from 'node:child_process';
import { promisify } from 'node:util';
import { mkdir, writeFile, readFile } from 'node:fs/promises';
import { createWriteStream } from 'node:fs';
import { createHash } from 'node:crypto';
import { fileURLToPath } from 'node:url';
import path from 'node:path';
import assert from 'node:assert/strict';
const root=path.resolve(path.dirname(fileURLToPath(import.meta.url)),'../..');
const artifact=path.join(root,'artifacts',`inputs-${Date.now()}`);await mkdir(artifact,{recursive:true});
const desktop=process.env.RECONX_DESKTOP_EXE??path.join(root,'target','debug','reconx-desktop.exe');
const cli=process.env.RECONX_CLI_EXE??path.join(root,'target','debug','reconx.exe');
const env={...process.env,RECONX_DATA_DIR:artifact,RECONX_NATIVE_TEST_HIDDEN:'1',WEBVIEW2_USER_DATA_FOLDER:path.join(artifact,'webview')};
const exec=promisify(execFile);const delay=ms=>new Promise(r=>setTimeout(r,ms));
let child,browser,page;const checks=[];
async function command(args){return JSON.parse((await exec(cli,['--json',...args],{env,windowsHide:true,maxBuffer:16*1024*1024})).stdout);}
async function launch(){
 const listener=createServer();await new Promise(r=>listener.listen(0,'127.0.0.1',r));const port=listener.address().port;await new Promise(r=>listener.close(r));
 child=spawn(desktop,[],{cwd:root,env:{...env,WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS:`--remote-debugging-port=${port}`},windowsHide:true,stdio:['ignore','pipe','pipe']});
 const log=createWriteStream(path.join(artifact,`desktop-${port}.log`));child.stdout.pipe(log);child.stderr.pipe(log);
 let ready=false;for(let i=0;i<120;i++){if(child.exitCode!==null)throw Error('Native application exited');try{const r=await fetch(`http://127.0.0.1:${port}/json/version`);if(r.ok){ready=true;break;}}catch{}await delay(250);}assert(ready);
 browser=await chromium.connectOverCDP(`http://127.0.0.1:${port}`);
 for(let i=0;i<80;i++){page=browser.contexts().flatMap(c=>c.pages())[0];if(page)break;await delay(250);}
 assert(page);await page.getByRole('heading',{name:'Workspace overview',exact:true}).waitFor({timeout:30000});
 return page;
}
async function stop(){if(browser){await browser.close();browser=null;}if(child&&child.exitCode===null){spawnSync('taskkill',['/PID',String(child.pid),'/T','/F'],{windowsHide:true,stdio:'ignore'});await delay(500);}}
try{
 await launch();await page.getByRole('button',{name:'Projects',exact:true}).click();
 await page.getByLabel('Project name',{exact:true}).fill('Input workflow project');await page.getByRole('button',{name:'Create project',exact:true}).click();await page.getByRole('status').filter({hasText:'Project created'}).waitFor();
 const project=(await command(['project','list']))[0];await page.getByLabel('Current project',{exact:true}).selectOption(project.id);
 await page.getByRole('button',{name:'+ Add Target',exact:true}).click();const dialog=page.getByRole('dialog',{name:'Add Target',exact:true});await dialog.waitFor();
 await dialog.getByLabel('Target name',{exact:true}).fill('Owned fixture domain');await dialog.getByLabel('Target value',{exact:true}).fill('example.test');await dialog.getByLabel('Tags / groups').fill('fixture, internal');await dialog.getByLabel('I confirm I have permission').check();
 await dialog.getByRole('button',{name:'Save target',exact:true}).click();await dialog.waitFor({state:'hidden'});
 const targets=await command(['target','list','--project',project.id]);assert.equal(targets[0].value,'example.test');assert.deepEqual(targets[0].tags,['fixture','internal']);checks.push('Desktop Add Target persists the same authorized domain and tags read by CLI');
 await page.getByRole('button',{name:'Targets & Scope',exact:true}).click();await page.getByRole('button',{name:'Owned fixture domain',exact:true}).click();await page.getByRole('status').filter({hasText:'Registered authorization and scope match'}).waitFor();
 await page.getByLabel('Value to validate',{exact:true}).fill('outside.test');await page.getByRole('button',{name:'Validate scope',exact:true}).click();await page.getByRole('status').filter({hasText:'out of scope'}).waitFor();
 await page.getByRole('button',{name:'Edit',exact:true}).click();const edit=page.getByRole('dialog',{name:'Edit target'});await edit.getByLabel('Target name',{exact:true}).fill('Updated domain');await edit.getByRole('button',{name:'Save target',exact:true}).click();await edit.waitFor({state:'hidden'});assert.equal((await command(['target','show',targets[0].id])).name,'Updated domain');checks.push('Target detail scope validation and editing use backend state');
 const exportPath=path.join(artifact,'targets-export.json');
 await page.getByRole('button',{name:'Export targets',exact:true}).click();
 await exec('powershell.exe',['-NoProfile','-ExecutionPolicy','Bypass','-File',path.join(root,'tests','e2e','select-file.ps1'),'-AppProcessId',String(child.pid),'-SamplePath',exportPath],{windowsHide:true,timeout:30000});
 let exported;for(let i=0;i<40;i++){try{exported=JSON.parse(await readFile(exportPath,'utf8'));break;}catch{}await delay(100);}assert.equal(exported.format_version,1);assert.equal(exported.targets[0].name,'Updated domain');
 const importPath=path.join(artifact,'targets-import.json');await writeFile(importPath,JSON.stringify({format_version:1,targets:[{...exported.targets[0],name:'Imported local URL',kind:'url',value:'http://127.0.0.1:8080/',allowed_scope:[],excluded_scope:[],tags:['imported']}]}));
 await page.getByRole('button',{name:'Import targets',exact:true}).click();await exec('powershell.exe',['-NoProfile','-ExecutionPolicy','Bypass','-File',path.join(root,'tests','e2e','select-file.ps1'),'-AppProcessId',String(child.pid),'-SamplePath',importPath],{windowsHide:true,timeout:30000});await page.getByRole('button',{name:'Imported local URL',exact:true}).waitFor();assert.equal((await command(['target','list','--project',project.id])).length,2);checks.push('Native target export/save and list import/open persist real versioned records');
 await page.getByRole('button',{name:'Quick actions',exact:true}).click();await page.getByRole('menuitem',{name:'Import Repository',exact:true}).click();await dialog.waitFor();assert.equal(await dialog.getByLabel('Target type',{exact:true}).inputValue(),'git_repository');await dialog.getByRole('button',{name:'Cancel',exact:true}).click();checks.push('Global quick actions open the shared real target dialog');
 await page.getByRole('button',{name:'Reverse Engineering',exact:true}).click();await page.getByLabel('I have permission to analyze the imported local files.',{exact:true}).check();
 await page.getByRole('button',{name:'Import File',exact:true}).click();
 const picker=await exec('powershell.exe',['-NoProfile','-ExecutionPolicy','Bypass','-File',path.join(root,'tests','e2e','select-file.ps1'),'-AppProcessId',String(child.pid),'-SamplePath',cli],{windowsHide:true,timeout:30000});assert.match(picker.stdout,/native file dialog/);
 await page.getByRole('status').filter({hasText:'1 file(s) imported'}).waitFor({timeout:30000});
 const files=await command(['reverse','list','--project',project.id]);assert.equal(files.length,1);assert.equal(files[0].format,'Pe');assert.equal(files[0].sha256,createHash('sha256').update(await readFile(cli)).digest('hex'));checks.push('Actual Windows native picker imported an owned real PE with matching SHA-256');
 await page.getByRole('button',{name:'Analyze File',exact:true}).click();await page.getByText('Static parsing completed; saving local analysis artifact',{exact:true}).waitFor({state:'hidden',timeout:30000});
 await page.getByText('Analysis saved',{exact:true}).waitFor({timeout:30000});const result=await command(['reverse','metadata',files[0].id]);assert.equal(result.analysis.sha256,files[0].sha256);assert(result.analysis.sections.length>0);assert(result.analysis.imports.length>0);
 await page.getByRole('tab',{name:'Sections',exact:true}).click();await page.getByRole('cell',{name:'.text',exact:true}).waitFor();await page.getByRole('tab',{name:'Strings',exact:true}).click();await page.getByLabel('Search strings',{exact:true}).fill('reconx');await page.getByText(/Showing \d+ of \d+ matches/).waitFor();checks.push('Desktop analysis persists real PE sections, imports, strings and completed job readable by CLI');
 for(const mode of ['Light','Dark','ReconX']){await page.getByRole('button',{name:'Topbar appearance',exact:true}).click();await page.getByRole('menuitemradio',{name:new RegExp('^'+mode)}).click();await page.getByText('Saving appearance…',{exact:true}).waitFor({state:'hidden'});assert.equal(await page.locator('html').getAttribute('data-theme'),mode.toLowerCase());await page.getByRole('button',{name:'Reverse Engineering',exact:true}).click();await page.getByRole('tab',{name:'Metadata',exact:true}).click();await page.getByText('Analysis saved',{exact:true}).waitFor();await page.getByRole('tab',{name:'Sections',exact:true}).click();await page.getByRole('cell',{name:'.text',exact:true}).waitFor();await page.evaluate(()=>document.fonts.ready);await page.screenshot({path:path.join(artifact,`reverse-${mode.toLowerCase()}.png`),fullPage:true});}
 await page.getByRole('button',{name:'Quick actions',exact:true}).click();await page.getByRole('menuitem',{name:'View Jobs',exact:true}).click();await page.getByRole('heading',{name:'Analysis Jobs',exact:true}).waitFor();assert.equal((await command(['jobs','list']))[0].status,'completed');checks.push('Global View Jobs opens real persistent execution history');
 await stop();await launch();await page.getByRole('button',{name:'Targets & Scope',exact:true}).click();await page.getByRole('button',{name:'Updated domain',exact:true}).waitFor();await page.getByRole('button',{name:'Reverse Engineering',exact:true}).click();await page.getByText('Analysis saved',{exact:true}).waitFor({timeout:30000});assert.equal((await command(['reverse','metadata',files[0].id])).id,result.id);checks.push('Native application restart preserves targets, file inventory and analysis artifacts');
 await page.setViewportSize({width:1366,height:768});await page.screenshot({path:path.join(artifact,'reverse-1366.png'),fullPage:true});assert(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth+1));
 await page.getByRole('button',{name:'Targets & Scope',exact:true}).click();await page.screenshot({path:path.join(artifact,'targets-1366.png'),fullPage:true});assert(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth+1));checks.push('Target and reverse pages avoid document clipping at 1366×768');
 await writeFile(path.join(artifact,'results.json'),JSON.stringify({status:'passed',checks},null,2));console.log(JSON.stringify({artifact,passed:checks.length,checks},null,2));
}catch(error){if(page)await page.screenshot({path:path.join(artifact,'failure.png'),fullPage:true}).catch(()=>{});await writeFile(path.join(artifact,'results.json'),JSON.stringify({status:'failed',checks,error:String(error)},null,2));throw error;}finally{await stop();}
