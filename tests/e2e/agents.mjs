import { chromium } from "../../apps/desktop/node_modules/playwright/index.mjs";
import { createServer } from "node:net";
import { createServer as httpServer } from "node:http";
import { spawn, execFile, spawnSync } from "node:child_process";
import { promisify } from "node:util";
import { mkdir, writeFile, readFile } from "node:fs/promises";
import { createWriteStream } from "node:fs";
import { createHash } from "node:crypto";
import { fileURLToPath } from "node:url";
import path from "node:path";
import assert from "node:assert/strict";
const root = path.resolve(
  path.dirname(fileURLToPath(import.meta.url)),
  "../..",
);
const artifact = path.join(
  root,
  "artifacts",
  `offline-agents-native-${Date.now()}`,
);
await mkdir(artifact, { recursive: true });
const desktop =
  process.env.RECONX_DESKTOP_EXE ??
  path.join(root, "target", "debug", "reconx-desktop.exe");
const cli =
  process.env.RECONX_CLI_EXE ??
  path.join(root, "target", "debug", "reconx.exe");
const env = {
  ...process.env,
  RECONX_DATA_DIR: artifact,
  RECONX_NATIVE_TEST_HIDDEN: "1",
  WEBVIEW2_USER_DATA_FOLDER: path.join(artifact, "webview"),
};
const exec = promisify(execFile);
const delay = (ms) => new Promise((r) => setTimeout(r, ms));
let child, browser, page;
const checks = [];
async function command(args) {
  return JSON.parse(
    (
      await exec(cli, ["--json", ...args], {
        env,
        windowsHide: true,
        maxBuffer: 16 * 1024 * 1024,
      })
    ).stdout,
  );
}
async function launch() {
  const listener = createServer();
  await new Promise((r) => listener.listen(0, "127.0.0.1", r));
  const port = listener.address().port;
  await new Promise((r) => listener.close(r));
  child = spawn(desktop, [], {
    cwd: root,
    env: {
      ...env,
      WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS: `--remote-debugging-port=${port}`,
    },
    windowsHide: true,
    stdio: ["ignore", "pipe", "pipe"],
  });
  const log = createWriteStream(path.join(artifact, `desktop-${port}.log`));
  child.stdout.pipe(log);
  child.stderr.pipe(log);
  let ready = false;
  for (let i = 0; i < 120; i++) {
    if (child.exitCode !== null) throw Error("Native application exited");
    try {
      const r = await fetch(`http://127.0.0.1:${port}/json/version`);
      if (r.ok) {
        ready = true;
        break;
      }
    } catch {}
    await delay(250);
  }
  assert(ready);
  browser = await chromium.connectOverCDP(`http://127.0.0.1:${port}`);
  for (let i = 0; i < 80; i++) {
    page = browser.contexts().flatMap((c) => c.pages())[0];
    if (page) break;
    await delay(250);
  }
  assert(page);
  if (process.env.RECONX_VERIFY_ASSET === "1") {
    const html = await readFile(
      path.join(root, "apps/desktop/dist/index.html"),
      "utf8",
    );
    const expected = html.match(/src="([^"]+\.js)"/)[1];
    assert.equal(
      await page.locator('script[type="module"][src]').getAttribute("src"),
      expected,
      "Native executable must embed the current built frontend",
    );
  }
  await page
    .getByRole("heading", { name: "Workspace overview", exact: true })
    .waitFor({ timeout: 30000 });
  return page;
}
async function stop() {
  if (browser) {
    await browser.close();
    browser = null;
  }
  if (child && child.exitCode === null) {
    spawnSync("taskkill", ["/PID", String(child.pid), "/T", "/F"], {
      windowsHide: true,
      stdio: "ignore",
    });
    await delay(500);
  }
}

const modelCalls = [];
const ai = httpServer(async (req, res) => {
  let raw = "";
  for await (const chunk of req) raw += chunk;
  const body = JSON.parse(raw);
  modelCalls.push({model: body.model, path: req.url});
  assert.equal(req.url, "/v1/chat/completions");
  const content = JSON.stringify({steps: [
    {capability_id: "local.metadata", reason: "Inspect the benign local file"},
    {capability_id: "local.strings", reason: "Read bounded local strings"},
  ]});
  res.setHeader("Content-Type", "application/json");
  res.end(JSON.stringify({choices: [{message: {content}}], usage: {prompt_tokens: 17, completion_tokens: 11}}));
});
await new Promise(r => ai.listen(0, "127.0.0.1", r));
const gateway = `http://127.0.0.1:${ai.address().port}/v1`;
async function service(request) {
  return page.evaluate(r => window.__TAURI_INTERNALS__.invoke("service", {request:r}), request);
}
async function waitRuntime(id) {
  for (let i=0; i<160; i++) {
    const runtime = await command(["agents", "status", id]);
    if (runtime.status === "completed") return runtime;
    assert(!["failed", "cancelled"].includes(runtime.status), JSON.stringify(runtime));
    await delay(200);
  }
  throw Error("Offline DAG did not complete");
}
try {
  const project = await command(["project", "create", "Native offline agents"]);
  const sample = path.join(artifact, "benign.bin");
  await writeFile(sample, process.env.RECONX_AGENT_SAMPLE
    ? await readFile(process.env.RECONX_AGENT_SAMPLE)
    : "RECONX_BENIGN_NATIVE_AGENT_FIXTURE\0read-only sample");
  const imported = await command(["reverse", "import", sample, "--project", project.id, "--authorized"]);
  if (process.env.RECONX_AGENT_SAMPLE) assert.equal(imported.format, "Pe");
  await command(["workspace", "select", "--project", project.id, "--target", imported.target_id]);
  await launch();
  await service({op:"save_provider", provider:{id:"3b559e3d-f1cf-46ae-ad34-f91a81450aab", name:"Owned MOCK agent planner", kind:"compatible", base_url:gateway, model:"agent-fixture", enabled:true, requires_key:false, timeout_secs:10, max_output_tokens:1024}});
  // Reload the actual native snapshot after seeding the test-only provider.
  await stop(); await launch();
  await page.getByRole("button", {name:"AI Research",exact:true}).click();
  assert.equal(await page.getByLabel("Research project",{exact:true}).inputValue(), project.id);
  assert.equal(await page.getByLabel("Research target",{exact:true}).inputValue(), imported.target_id);
  await page.getByLabel("Research model",{exact:true}).selectOption("3b559e3d-f1cf-46ae-ad34-f91a81450aab");
  await page.getByRole("button",{name:"Review research context",exact:true}).click();
  assert.equal(modelCalls.length,0);
  await page.getByText("I approve sending exactly this planning context").check();
  await page.getByRole("button",{name:"Generate research plan",exact:true}).click();
  await page.getByRole("button",{name:"Review offline agent DAG",exact:true}).waitFor();
  assert.equal(modelCalls.length,1);
  const id = (await command(["research","list","--project",project.id]))[0];
  let panel = page.getByLabel("Offline agent runtime",{exact:true});
  await panel.getByRole("button",{name:"Review offline agent DAG",exact:true}).click();
  await panel.getByLabel("Reviewed agent DAG",{exact:true}).waitFor();
  assert(await panel.getByRole("button",{name:"Start offline agent DAG",exact:true}).isDisabled());
  await panel.getByLabel("Agent invocation budget",{exact:true}).fill("20");
  assert.equal(await panel.getByLabel("Reviewed agent DAG",{exact:true}).count(),0);
  await panel.getByRole("button",{name:"Review offline agent DAG",exact:true}).click();
  await panel.getByLabel("Approve offline agent DAG",{exact:true}).check();
  await panel.getByRole("button",{name:"Start offline agent DAG",exact:true}).click();
  const runtime = await waitRuntime(id);
  assert.equal(runtime.tasks.length,5);
  assert.equal(runtime.invocations,5);
  assert(runtime.tasks.every(t=>t.state==="COMPLETED"&&t.attempt===0));
  await panel.getByRole("table",{name:"Agent task execution",exact:true}).waitFor();
  await page.waitForFunction(()=>document.querySelector('[aria-label="Offline agent runtime"] [role="status"]')?.textContent?.includes("5/5 completed"));
  assert.equal(await panel.getByRole("table").locator("tbody tr").count(),5);
  assert((await panel.innerText()).includes("Cost: not reported"));
  assert.equal(modelCalls.length,1);
  const session = await command(["research","show",id]);
  assert.equal(session.evidence.length,5); assert.equal(session.reports.length,1);
  for(const evidence of session.evidence) assert.equal((await command(["evidence","verify",evidence.id])).integrity,"matched");
  checks.push("Actual native MOCK planning consent, stale-budget invalidation and explicit offline DAG approval execute five real jobs through the shared Job Engine");
  checks.push("Native table and CLI agree on completed dependencies, real evidence integrity and automatic report snapshot; no runtime network/model request");
  await panel.getByText("Structured agent events",{exact:true}).click();
  await panel.getByText(/TASK_COMPLETED/).first().waitFor();
  await panel.getByText("Structured agent events",{exact:true}).click();
  for(const theme of ["Light","Dark","ReconX"]){
    await page.getByRole("button",{name:"Appearance",exact:true}).click();
    await page.getByRole("menuitemradio",{name:new RegExp("^"+theme)}).click();
    for(const [width,height] of [[1366,768],[1440,900],[1920,1080]]){
      await page.setViewportSize({width,height}); await panel.scrollIntoViewIfNeeded();
      await page.evaluate(()=>document.fonts.ready);
      assert(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth+1));
      await page.screenshot({path:path.join(artifact,`agents-${theme.toLowerCase()}-${width}.png`)});
    }
  }
  checks.push("Populated real task table renders at three desktop resolutions in Light/Dark/ReconX with no page overflow (nine captures)");
  await page.getByText("Explore stored provenance",{exact:true}).click();
  let graph = page.getByLabel("Research provenance",{exact:true});
  await graph.getByLabel("Graph node type",{exact:true}).selectOption("agent");
  await graph.getByRole("button",{name:/^agent:/}).first().waitFor();
  assert.equal(await graph.getByRole("button",{name:/^agent:/}).count(),6);
  await graph.getByRole("button",{name:/^agent:/}).first().click();
  await graph.getByRole("button",{name:"Inspect linked record",exact:true}).click();
  const linked = page.getByRole("dialog",{name:"Linked provenance record",exact:true});
  await linked.waitFor(); assert((await linked.innerText()).includes("research_agent_assignments"));
  await linked.getByRole("button",{name:"Close dialog",exact:true}).click();
  await graph.getByLabel("Graph node type",{exact:true}).selectOption("agent_task");
  await graph.getByRole("button",{name:/^agent_task:/}).first().waitFor();
  assert.equal(await graph.getByRole("button",{name:/^agent_task:/}).count(),5);
  await graph.scrollIntoViewIfNeeded();
  await page.screenshot({path:path.join(artifact,"agents-graph.png")});
  checks.push("Native graph filters six persisted assignments and five real tasks; linked-record inspector reads the stored source");
  const report = await command(["report","show",session.reports[0].id]);
  await service({op:"report_export", id:report.report.id, format:"pdf", path:path.join(artifact,"agent-report.pdf"), reviewed_sha256:report.report.sha256});
  assert((await readFile(path.join(artifact,"agent-report.pdf"))).subarray(0,5).equals(Buffer.from("%PDF-")));
  await stop(); await launch();
  await page.getByRole("button",{name:"AI Research",exact:true}).click();
  await page.getByLabel("Research history",{exact:true}).selectOption(id);
  panel = page.getByLabel("Offline agent runtime",{exact:true});
  await panel.getByRole("table",{name:"Agent task execution",exact:true}).waitFor();
  assert.equal((await command(["agents","status",id])).invocations,5);
  assert.equal(modelCalls.length,1);
  checks.push("Native restart restores persisted session, task results and report without repeating jobs or contacting the MOCK planner");
  // Create another reviewed session while Desktop is stopped, so pause is deterministic.
  await stop();
  const preview = await command(["research","preview",imported.target_id,"--mode","manual"]);
  const planFile = path.join(artifact,"second-plan.json");
  await writeFile(planFile,JSON.stringify({request:preview.request,context:preview.context,fingerprint:preview.fingerprint,consent:false}));
  const second = await command(["research","plan",planFile]);
  const sid = second.plan.id;
  const dag = await command(["research","dag-preview",sid]);
  const reviewFile = path.join(artifact,"second-dag.json");
  await writeFile(reviewFile,JSON.stringify({session_id:sid,fingerprint:dag.fingerprint,limits:dag.limits,approved:true}));
  await command(["research","dag-start",reviewFile]);
  assert.equal((await command(["research","pause",sid])).status,"paused");
  await launch();
  await page.getByRole("button",{name:"AI Research",exact:true}).click();
  await page.getByLabel("Research history",{exact:true}).selectOption(sid);
  panel = page.getByLabel("Offline agent runtime",{exact:true});
  await panel.getByRole("button",{name:"Resume offline DAG",exact:true}).waitFor();
  assert.equal((await command(["agents","status",sid])).invocations,0);
  await panel.getByRole("button",{name:"Resume offline DAG",exact:true}).click();
  assert.equal((await waitRuntime(sid)).invocations,5);
  assert.equal(modelCalls.length,1);
  checks.push("Actual native Resume handles a persisted paused DAG after restart; no task executes before resume and no inference is triggered");
  const result={status:"passed",timestamp:new Date().toISOString(),checks,session_id:id,second_session_id:sid,model:"MOCK_VERIFIED owned loopback only",desktop_sha256:createHash("sha256").update(await readFile(desktop)).digest("hex"),cli_sha256:createHash("sha256").update(await readFile(cli)).digest("hex")};
  await writeFile(path.join(artifact,"results.json"),JSON.stringify(result,null,2));
  console.log(JSON.stringify({artifact,...result},null,2));
}catch(error){
  if(page) await page.screenshot({path:path.join(artifact,"failure.png")}).catch(()=>{});
  await writeFile(path.join(artifact,"results.json"),JSON.stringify({status:"failed",checks,error:String(error),stack:error.stack},null,2));
  throw error;
}finally{
  await stop(); ai.closeAllConnections();
  await new Promise(r=>ai.close(r));
}
