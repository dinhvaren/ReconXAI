import { chromium } from '../../apps/desktop/node_modules/playwright/index.mjs';
import { createServer as netServer } from 'node:net';
import { createServer } from 'node:http';
import { spawn,execFile,spawnSync } from 'node:child_process';
import { promisify } from 'node:util';
import { mkdir,writeFile,readFile } from 'node:fs/promises';
import { createWriteStream } from 'node:fs';
import { fileURLToPath } from 'node:url';
import path from 'node:path';
import assert from 'node:assert/strict';
const root=path.resolve(path.dirname(fileURLToPath(import.meta.url)),'../..');
const phase=process.argv[2]??'after';assert(['before','after'].includes(phase));
const artifact=process.env.RECONX_TYPOGRAPHY_ARTIFACT??path.join(root,'artifacts',`typography-${Date.now()}`);
const output=path.join(artifact,phase);await mkdir(output,{recursive:true});
const desktop=process.env.RECONX_DESKTOP_EXE??path.join(root,'target','debug','reconx-desktop.exe');
const cli=process.env.RECONX_CLI_EXE??path.join(root,'target','debug','reconx.exe');
const env={...process.env,RECONX_DATA_DIR:path.join(artifact,'data'),RECONX_NATIVE_TEST_HIDDEN:'1',WEBVIEW2_USER_DATA_FOLDER:path.join(artifact,'webview')};
const exec=promisify(execFile);const delay=ms=>new Promise(r=>setTimeout(r,ms));
const reply='Đây là phản hồi từ gateway kiểm thử cục bộ. Evidence is retained locally; this text does not claim a vulnerability.';
const mock=createServer(async(req,res)=>{let text='';for await(const chunk of req)text+=chunk;res.setHeader('Content-Type','application/json');
 if(req.method==='GET'&&req.url==='/v1/models')res.end(JSON.stringify({data:[{id:'typography-fixture'}]}));
 else if(req.method==='POST'&&req.url==='/v1/chat/completions')res.end(JSON.stringify({choices:[{message:{content:reply}}],usage:{prompt_tokens:8,completion_tokens:12}}));
 else {res.statusCode=404;res.end('{}');}});
await new Promise(r=>mock.listen(0,'127.0.0.1',r));
async function command(args){return JSON.parse((await exec(cli,['--json',...args],{env,windowsHide:true,maxBuffer:16*1024*1024})).stdout);}
let child,browser,page,cdp;const observations=[];
async function launch(){
 const portServer=netServer();await new Promise(r=>portServer.listen(0,'127.0.0.1',r));const port=portServer.address().port;await new Promise(r=>portServer.close(r));
 child=spawn(desktop,[],{cwd:root,env:{...env,WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS:`--remote-debugging-port=${port}`},windowsHide:true,stdio:['ignore','pipe','pipe']});
 const log=createWriteStream(path.join(output,`native-${port}.log`));child.stdout.pipe(log);child.stderr.pipe(log);
 let ready=false;for(let i=0;i<120;i++){if(child.exitCode!==null)throw Error('Native instance exited');try{if((await fetch(`http://127.0.0.1:${port}/json/version`)).ok){ready=true;break;}}catch{}await delay(250);}assert(ready);
 browser=await chromium.connectOverCDP(`http://127.0.0.1:${port}`);
 for(let i=0;i<80;i++){page=browser.contexts().flatMap(c=>c.pages())[0];if(page)break;await delay(250);}assert(page);
 await page.getByRole('heading',{name:'Workspace overview',exact:true}).waitFor({timeout:30000});
 cdp=await page.context().newCDPSession(page);await cdp.send('DOM.enable');await cdp.send('CSS.enable');
}
async function actualFonts(selector){const {root:doc}=await cdp.send('DOM.getDocument');const {nodeId}=await cdp.send('DOM.querySelector',{nodeId:doc.nodeId,selector});if(!nodeId)return [];return (await cdp.send('CSS.getPlatformFontsForNode',{nodeId})).fonts;}
async function navigate(name){await page.getByRole('button',{name,exact:true}).click();await page.locator('main h1').waitFor();
 if(name==='Reverse Engineering'){await page.getByRole('tab',{name:'Metadata',exact:true}).waitFor({timeout:30000});}
 if(name==='Targets & Scope'){await page.getByRole('button',{name:'API kiểm thử',exact:true}).waitFor();}
 await page.evaluate(()=>document.fonts.ready);await delay(80);
}
async function measure(){return page.evaluate(()=>{
 const visible=el=>el.getClientRects().length&&getComputedStyle(el).visibility!=='hidden';
 const roles={title:'main h1',section:'.panel-heading h2,.section-title h2',card:'.feature-card h3,.library-card h3',navigation:'.nav-item:not(.active)',activeNavigation:'.nav-item.active',group:'.navigation-group-label',label:'.form label',input:'.form input:not([type="checkbox"])',secondary:'.subtitle,.feature-card-body > p',activity:'.activity-description b',code:'.technical,.storage-detail code',ai:'.chat-message pre'};
 const styles=Object.fromEntries(Object.entries(roles).map(([name,selector])=>{const el=document.querySelector(selector);if(!el)return [name,null];const s=getComputedStyle(el);return [name,{family:s.fontFamily,size:s.fontSize,weight:s.fontWeight,lineHeight:s.lineHeight,tracking:s.letterSpacing,transform:s.textTransform,color:s.color}];}));
 const palette=Object.fromEntries([...getComputedStyle(document.documentElement)].filter(key=>key.startsWith('--rx-')).map(key=>[key,getComputedStyle(document.documentElement).getPropertyValue(key).trim()]));
 const tiny=[...document.querySelectorAll('main p,main label,main input,main small,.nav-item,.activity-sidebar p')].filter(visible).filter(el=>parseFloat(getComputedStyle(el).fontSize)<12).map(el=>el.textContent.slice(0,50));
 const clippedNavigation=[...document.querySelectorAll('.nav-item > span')].filter(visible).filter(el=>el.scrollWidth>el.clientWidth+1).map(el=>el.textContent);
 const fonts=[...document.fonts].map(f=>({family:f.family,weight:f.weight,status:f.status}));
 const text=[...document.querySelectorAll('h1,h2,h3,p,label,small,b,strong,.nav-item')].filter(visible).map(el=>({weight:Number(getComputedStyle(el).fontWeight),size:Number.parseFloat(getComputedStyle(el).fontSize)}));
 return {styles,fonts,palette,tiny,clippedNavigation,text,theme:document.documentElement.dataset.theme,documentWidth:document.documentElement.scrollWidth,viewport:innerWidth,dpr:devicePixelRatio,contentWidth:document.querySelector('.workspace-content').clientWidth,contentScrollWidth:document.querySelector('.workspace-content').scrollWidth};
 });}
try{
 let projects=await command(['project','list']);let project=projects.find(p=>p.name==='Nghiên cứu / Local research');
 if(!project)project=await command(['project','create','Nghiên cứu / Local research','--description','Dữ liệu kiểm thử typography — projects, targets and local analysis.']);
 if(!(await command(['project','list'])).some(p=>p.name==='API review'))await command(['project','create','API review','--description','Endpoint inventory and local fixture notes.']);
 if(!(await command(['target','list','--project',project.id])).length)await command(['target','add','--project',project.id,'--type','rest_api','--value','http://127.0.0.1:8080/api','--name','API kiểm thử','--description','Local registration fixture; no target traffic is sent.','--tags','local,fixture','--authorized']);
 let file=(await command(['reverse','list','--project',project.id]))[0];if(!file){file=await command(['reverse','import',cli,'--project',project.id,'--authorized']);await command(['reverse','analyze',file.id]);}
 const providers=await command(['ai','providers']);const providerPath=path.join(output,'provider-fixture.json');
 await writeFile(providerPath,JSON.stringify({id:providers[0]?.id??'4607c965-4d59-4b5e-9193-267cc2c5a0d3',name:'Local gateway / Phân tích',kind:'compatible',base_url:`http://127.0.0.1:${mock.address().port}/v1`,model:'typography-fixture',enabled:true,timeout_secs:10,max_output_tokens:256,requires_key:false}));await command(['ai','configure',providerPath]);
 await launch();await page.getByLabel('Current project',{exact:true}).selectOption(project.id);
 await navigate('AI Workspace');await page.getByLabel('Stream output',{exact:true}).uncheck();await page.getByLabel('Message',{exact:true}).fill('Giải thích dữ liệu mẫu / Explain fixture evidence');await page.getByRole('button',{name:'Send message ↗',exact:true}).click();await page.getByText(reply,{exact:true}).waitFor();
 // Compare fonts in the native renderer before changing production selectors.
 if(phase==='before'){
  await navigate('Dashboard');await page.setViewportSize({width:1440,height:900});
  const data=await readFile(path.join(root,'apps/desktop/src/assets/fonts/sourcesans3/SourceSans3VF-Upright.ttf'));
  // FontFace accepts owned bytes without a URL fetch or weakening production CSP.
  await page.evaluate(async encoded=>{const bytes=Uint8Array.from(atob(encoded),c=>c.charCodeAt(0));const face=new FontFace('Source Sans 3',bytes.buffer,{weight:'200 900',style:'normal'});await face.load();document.fonts.add(face);window.reconxTrialFont=face;},data.toString('base64'));
  const samples=[];
  for(const family of ['DM Sans','Segoe UI Variable','Segoe UI Variable Text','Segoe UI Variable Display','Segoe UI','Source Sans 3']){
   await page.evaluate(f=>document.documentElement.style.setProperty('--font-ui',`"${f}",sans-serif`),family);await page.evaluate(()=>document.fonts.ready);await delay(100);
   const glyphs=await page.evaluate(()=>{const el=document.createElement('p');el.id='glyph-probe';el.textContent='Nghiên cứu bảo mật — kiểm thử, bằng chứng, cấu hình. Research evidence: 0123456789';el.style.cssText='position:fixed;bottom:45px;left:330px;z-index:999;background:var(--rx-surface);padding:8px;font-size:14px;font-weight:400';document.body.append(el);return el.textContent;});
   samples.push({family,glyphs,platformFonts:await actualFonts('#glyph-probe'),titleFonts:await actualFonts('main h1')});await page.screenshot({path:path.join(output,`font-trial-${family.toLowerCase().replaceAll(' ','-')}.png`)});await page.locator('#glyph-probe').evaluate(el=>el.remove());
  }await page.evaluate(()=>{document.documentElement.style.removeProperty('--font-ui');document.fonts.delete(window.reconxTrialFont);delete window.reconxTrialFont;});await writeFile(path.join(output,'font-trials.json'),JSON.stringify(samples,null,2));
 }
 // Verify the actual self-hosted fallback and Vietnamese glyphs on Windows too.
 if(phase==='after'){
  await navigate('Dashboard');await page.evaluate(()=>document.documentElement.style.setProperty('--font-ui','"Source Sans 3",sans-serif'));
  await page.evaluate(()=>document.fonts.ready);await page.evaluate(async()=>document.fonts.load('14px "Source Sans 3"','Nghiên cứu bảo mật — kiểm thử, bằng chứng, cấu hình.'));
  const fallbackFonts=await actualFonts('.feature-content h2');assert(fallbackFonts.some(f=>/Source.?Sans.?3/.test(f.familyName)),'Bundled fallback resolves from the application origin');
  await writeFile(path.join(output,'fallback-fonts.json'),JSON.stringify(fallbackFonts,null,2));await page.screenshot({path:path.join(output,'fallback-source-sans-3.png')});await page.evaluate(()=>document.documentElement.style.removeProperty('--font-ui'));
 }
 for(const theme of ['ReconX','Light','Dark']){
  await page.getByRole('button',{name:'Topbar appearance',exact:true}).click();await page.getByRole('menuitemradio',{name:new RegExp('^'+theme)}).click();await page.getByText('Saving appearance…',{exact:true}).waitFor({state:'hidden'});
  for(const module of ['Dashboard','Projects','Targets & Scope','Reverse Engineering','AI Workspace','Settings']){
   await navigate(module);
   if(module==='AI Workspace'&&!await page.getByText(reply,{exact:true}).count()){await page.getByLabel('Stream output',{exact:true}).uncheck();await page.getByLabel('Message',{exact:true}).fill('Giải thích dữ liệu mẫu');await page.getByRole('button',{name:'Send message ↗',exact:true}).click();await page.getByText(reply,{exact:true}).waitFor();}
   for(const [width,height] of [[1920,1080],[1440,900],[1366,768]]){
    await page.setViewportSize({width,height});await page.evaluate(()=>document.fonts.ready);await delay(80);
    const measured=await measure();measured.platformTitleFonts=await actualFonts('main h1');measured.platformBodyFonts=await actualFonts('.subtitle,.feature-card-body > p');measured.platformCodeFonts=await actualFonts('.storage-detail code,.technical');
    observations.push({module,width,height,theme,...measured});
    if(phase==='after'){assert.equal(measured.styles.title.size,'24px');assert.equal(measured.styles.title.weight,'600');assert.equal(measured.styles.navigation.weight,'400');assert.equal(measured.styles.activeNavigation.weight,'500');assert.deepEqual(measured.tiny,[]);assert.deepEqual(measured.clippedNavigation,[]);assert(measured.documentWidth<=width+1);assert(measured.contentScrollWidth<=measured.contentWidth+1);assert(measured.text.every(t=>t.weight<=600),'UI copy stays at regular/medium/semibold');assert(measured.platformTitleFonts.some(f=>/Segoe UI Variable|Source.?Sans.?3/.test(f.familyName)));if(measured.styles.code){assert.equal(measured.styles.code.weight,'400');assert.equal(measured.styles.code.size,'13px');}}
    await page.screenshot({path:path.join(output,`${theme.toLowerCase()}-${module.toLowerCase().replaceAll(' ','-').replaceAll('&','and')}-${width}x${height}.png`)});
   }
  }
 }
 // CDP device scale emulation exercises the real WebView2 font rasterizer. This
 // is not a claim that Windows monitor scaling settings were physically changed.
 await page.setViewportSize({width:1440,height:900});
 for(const scale of [1.25,1.5,2]){
  await cdp.send('Emulation.setDeviceMetricsOverride',{width:1440,height:900,deviceScaleFactor:scale,mobile:false});await navigate('Settings');await delay(150);
  // Playwright retains its logical viewport when a CDP scale override is set.
  // Capture directly so pixel dimensions prove the requested raster scale.
  const screenshot=await cdp.send('Page.captureScreenshot',{format:'png',fromSurface:true,captureBeyondViewport:true,clip:{x:0,y:0,width:1440,height:900,scale:1}});
  const pixels=Buffer.from(screenshot.data,'base64');const pixelWidth=pixels.readUInt32BE(16),pixelHeight=pixels.readUInt32BE(20);
  assert.equal(pixelWidth,Math.round(1440*scale));assert.equal(pixelHeight,Math.round(900*scale));
  observations.push({module:'Settings',width:1440,height:900,scale,pixelWidth,pixelHeight,...await measure()});await writeFile(path.join(output,`dpi-settings-${scale}.png`),pixels);
 }await cdp.send('Emulation.clearDeviceMetricsOverride');
 await writeFile(path.join(output,'computed-styles.json'),JSON.stringify(observations,null,2));await writeFile(path.join(output,'results.json'),JSON.stringify({status:'passed',phase,desktop,cli,observations:observations.length,fixture:'Owned local project/target/sample and deterministic local AI gateway'},null,2));console.log(JSON.stringify({artifact,phase,observations:observations.length},null,2));
}catch(error){if(page)await page.screenshot({path:path.join(output,'failure.png')}).catch(()=>{});await writeFile(path.join(output,'computed-styles.json'),JSON.stringify(observations,null,2));await writeFile(path.join(output,'results.json'),JSON.stringify({status:'failed',phase,error:String(error)},null,2));throw error;}
finally{if(browser)await browser.close();if(child&&child.exitCode===null)spawnSync('taskkill',['/PID',String(child.pid),'/T','/F'],{windowsHide:true,stdio:'ignore'});await new Promise(r=>mock.close(r));}
