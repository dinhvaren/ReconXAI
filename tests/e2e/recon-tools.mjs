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
  `recon-tools-native-${Date.now()}`,
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

const calls = [],
  aiCalls = [];
const owned = httpServer((req, res) => {
  calls.push({ method: req.method, url: req.url });
  res.writeHead(200, {
    Server: "owned-research-fixture",
    "Content-Type": "text/html",
  });
  res.end("<html><title>Owned native HTTPX</title></html>");
});
const ai = httpServer(async (req, res) => {
  let raw = "";
  for await (const chunk of req) raw += chunk;
  const body = JSON.parse(raw);
  aiCalls.push(body);
  const prompt = body.messages?.map((m) => m.content).join("\n") ?? "";
  const planning = prompt.includes("Return strict JSON only");
  const content = planning
    ? JSON.stringify({
        steps: prompt.includes('"local.metadata"')
          ? [
              {
                capability_id: "local.metadata",
                reason: "Read metadata from the selected benign file",
              },
              {
                capability_id: "local.strings",
                reason: "Read bounded static strings",
              },
            ]
          : [
              {
                capability_id: "local.scope_review",
                reason: "Review the registered scope",
              },
              {
                capability_id: "httpx.probe",
                reason: "Observe the separately approved local fixture",
              },
            ],
      })
    : "Local mock interpretation: the cited evidence records an observed result. No vulnerability has been verified.";
  res.setHeader("Content-Type", "application/json");
  res.end(
    JSON.stringify({
      choices: [{ message: { content } }],
      usage: { prompt_tokens: 37, completion_tokens: 21 },
    }),
  );
});
await new Promise((r) => owned.listen(0, "127.0.0.1", r));
await new Promise((r) => ai.listen(0, "127.0.0.1", r));
const origin = `http://127.0.0.1:${owned.address().port}`;
const gateway = `http://127.0.0.1:${ai.address().port}/v1`;
async function service(r) {
  return page.evaluate(
    (r) => window.__TAURI_INTERNALS__.invoke("service", { request: r }),
    r,
  );
}
async function waitSession(id) {
  for (let i = 0; i < 100; i++) {
    const s = await command(["research", "show", id]);
    if (s.status === "completed") return s;
    if (s.status === "failed") throw Error(JSON.stringify(s));
    await delay(150);
  }
  throw Error("Research session did not finish");
}
async function planAndRun(network) {
  await page
    .getByRole("button", { name: "Review research context", exact: true })
    .click();
  await page
    .getByText("I approve sending exactly this planning context")
    .check();
  assert.equal(calls.length, 0);
  await page
    .getByRole("button", { name: "Generate research plan", exact: true })
    .click();
  await page
    .getByRole("button", { name: "Execute approved plan", exact: true })
    .waitFor();
  assert.equal(
    await page
      .getByRole("heading", { name: "Available capabilities", exact: true })
      .count(),
    0,
    "Planning context closes after the persisted plan is created",
  );
  assert.equal(
    await page
      .getByRole("button", { name: "Execute approved plan", exact: true })
      .isDisabled(),
    true,
  );
  assert.equal(calls.length, 0);
  if (network) {
    const review = page.getByLabel("Recon execution review", { exact: true });
    const sessionId = (
      await command(["research", "list", "--project", projectId])
    )[0];
    const planned = await command(["research", "show", sessionId]);
    const destination = await command([
      "target",
      "show",
      planned.plan.preview.request.target_id,
    ]);
    assert((await review.innerText()).includes(destination.value));
    assert((await review.innerText()).includes("probe"));
  }
  await page
    .getByText("I approve the listed capabilities and resource limits")
    .check();
  if (network)
    await page
      .getByText("I separately approve the displayed recon network")
      .check();
  const id = (await command(["research", "list", "--project", projectId]))[0];
  await page
    .getByRole("button", { name: "Execute approved plan", exact: true })
    .click();
  const s = await waitSession(id);
  await page
    .getByRole("button", { name: "Prepare research report", exact: true })
    .waitFor();
  return s;
}
let projectId;
try {
  await launch();
  await page.getByRole("button", { name: "Projects", exact: true }).click();
  await page.getByLabel("Project name", { exact: true }).fill("Research Demo");
  await page
    .getByRole("button", { name: "Create project", exact: true })
    .click();
  await page
    .getByRole("status")
    .filter({ hasText: "Project created" })
    .waitFor();
  projectId = (await command(["project", "list"]))[0].id;
  assert.equal((await command(["workspace", "show"])).project_id, projectId);
  await service({
    op: "save_provider",
    provider: {
      id: "da1f803a-c5d2-4663-914b-b9f1a4ad4abc",
      name: "Deterministic local research fixture",
      kind: "compatible",
      base_url: gateway,
      model: "fixture-model",
      enabled: true,
      requires_key: false,
      timeout_secs: 10,
      max_output_tokens: 1024,
    },
  });
  const snapshot = await service({ op: "snapshot" });
  await service({
    op: "save_settings",
    settings: {
      ...snapshot.settings,
      security_mode: "lab_research",
      default_provider_id: "da1f803a-c5d2-4663-914b-b9f1a4ad4abc",
    },
  });
  await page
    .getByRole("button", { name: "Quick actions", exact: true })
    .click();
  await page.getByRole("menuitem", { name: "Add Target", exact: true }).click();
  const dialog = page.getByRole("dialog", { name: "Add Target", exact: true });
  await dialog
    .getByLabel("Target name", { exact: true })
    .fill("Owned local research API");
  await dialog.getByLabel("Target type", { exact: true }).selectOption("url");
  await dialog.getByLabel("Target value", { exact: true }).fill(`${origin}/`);
  await dialog
    .getByLabel("Preferred analysis mode", { exact: true })
    .selectOption("lab_research");
  await dialog.getByLabel("I confirm I have permission").check();
  await dialog
    .getByRole("button", { name: "Save target", exact: true })
    .click();
  await dialog.waitFor({ state: "hidden" });
  const target = (await command(["target", "list"])).find(
    (t) => t.name === "Owned local research API",
  );
  assert.equal((await command(["workspace", "show"])).target_id, target.id);
  checks.push(
    "Native project/target creation selects persisted IDs immediately; CLI reads the same selection",
  );
  await page.getByRole("button", { name: "Extensions", exact: true }).click();
  const search = page.getByRole("searchbox", {
    name: "Search extensions",
    exact: true,
  });
  for (const id of ["httpx", "subfinder"]) {
    await search.fill(id);
    await page
      .getByRole("button", { name: "Install extension", exact: true })
      .click();
    const review = page.getByRole("dialog", {
      name: "Review extension installation",
      exact: true,
    });
    await review.waitFor();
    assert(
      await review
        .getByRole("button", { name: "Approve and install", exact: true })
        .isDisabled(),
    );
    await review
      .getByRole("checkbox", {
        name: "I approve these dependencies and extension permissions",
        exact: true,
      })
      .check();
    await review
      .getByRole("button", { name: "Approve and install", exact: true })
      .click();
    await review.waitFor({ state: "hidden", timeout: 180000 });
    assert.equal((await command(["extensions", "info", id])).status, "ready");
  }
  assert.equal(
    calls.length,
    0,
    "Health checks use their own owned-loopback fixtures",
  );
  checks.push(
    "Native Marketplace approval installs exact hash-verified HTTPX/Subfinder and passes real local-GET/source-inventory health; no live domain query",
  );
  await page
    .getByRole("button", { name: "Reconnaissance", exact: true })
    .click();
  assert.equal(
    await page.getByLabel("Inventory target", { exact: true }).inputValue(),
    target.id,
  );
  await page
    .getByRole("button", { name: "Start AI Research", exact: true })
    .click();
  await page
    .getByLabel("Research model")
    .selectOption("da1f803a-c5d2-4663-914b-b9f1a4ad4abc");
  const session = await planAndRun(true);
  assert.equal(session.jobs.length, 2);
  assert.equal(calls.length, 1);
  assert.equal(calls[0].method, "GET");
  assert(session.evidence.length >= 2);
  assert(
    session.jobs.every(
      (j) => j.project_id === projectId && j.target_id === target.id,
    ),
  );
  const inventory = page.getByLabel("Recon asset inventory", { exact: true });
  await inventory.getByText(`${origin}/`, { exact: false }).first().waitFor();
  assert((await inventory.innerText()).includes("200"));
  assert((await inventory.innerText()).includes("httpx"));
  const observed = (
    await command(["recon", "assets", "--project", projectId])
  ).find((a) => a.target_id === target.id);
  assert.equal(observed.facts.title, "Owned native HTTPX");
  for (const item of session.evidence)
    await command(["evidence", "verify", item.id]);
  checks.push(
    "Start AI Research generates a typed MOCK AI plan and executes actual approved scope/managed HTTPX GET jobs; no request before separate approval",
  );
  await page
    .getByRole("button", { name: "Preview AI evidence context", exact: true })
    .click();
  await page
    .getByText("Approve this evidence context for the selected model.", {
      exact: true,
    })
    .check();
  await page
    .getByRole("button", { name: "Correlate evidence", exact: true })
    .click();
  await page
    .getByRole("heading", {
      name: "AI interpretation · requires review",
      exact: true,
    })
    .waitFor();
  assert.equal(aiCalls.length, 2);
  await page
    .getByLabel("Research finding title")
    .fill("Manual fixture review note");
  await page
    .getByLabel("Research finding details")
    .fill(
      "The actual fixture returned metadata. This is a review note, not a confirmed vulnerability.",
    );
  await page
    .getByRole("button", { name: "Save reviewable finding", exact: true })
    .click();
  await page
    .getByText("Manual fixture review note · needs_verification", {
      exact: true,
    })
    .waitFor();
  await page
    .getByRole("button", { name: "Prepare research report", exact: true })
    .click();
  const reportDialog = page.getByRole("dialog", {
    name: "Research report preview",
    exact: true,
  });
  await reportDialog.waitFor();
  const current = await command(["research", "show", session.plan.id]);
  assert.equal(current.findings[0].input.status, "needs_verification");
  const reportId = current.reports[0].id;
  const rv = await service({ op: "report_show", id: reportId });
  for (const format of ["markdown", "json", "html", "pdf"]) {
    const destination = path.join(
      artifact,
      `session-report.${format === "markdown" ? "md" : format}`,
    );
    await service({
      op: "report_export",
      id: reportId,
      format,
      path: destination,
      reviewed_sha256: rv.report.sha256,
    });
    const bytes = await readFile(destination);
    assert(bytes.length > 100);
    if (format === "pdf")
      assert(bytes.subarray(0, 5).equals(Buffer.from("%PDF-")));
    if (format === "json") JSON.parse(bytes.toString());
  }
  checks.push(
    "Actual evidence is interpreted only after context consent; manual finding stays Needs Verification; real Markdown/JSON/HTML/PDF exports validate",
  );
  await reportDialog
    .getByRole("button", { name: "Close dialog", exact: true })
    .click();
  const beforeGraphHttp=calls.length, beforeGraphAi=aiCalls.length;
  await page.getByText("Explore stored provenance",{exact:true}).click();
  const provenance=page.getByLabel("Research provenance",{exact:true});
  await provenance.getByRole("group",{name:"Provenance graph canvas",exact:true}).waitFor();
  const graph=await command(["graph","nodes",session.plan.id]);
  assert.equal(graph.nodes.filter(n=>n.kind==="url").length,1);
  assert(graph.edges.some(e=>e.relation==="SUPPORTED_BY"));
  assert(graph.nodes.some(n=>n.kind==="evidence"&&session.evidence.some(e=>e.id===n.source_id)));
  await provenance.getByRole("button",{name:/^evidence:/}).first().click();
  await provenance.getByRole("button",{name:"Inspect linked record",exact:true}).click();
  const linked=page.getByRole("dialog",{name:"Linked provenance record",exact:true});
  await linked.waitFor();assert((await linked.innerText()).includes("source_id"));
  await linked.getByRole("button",{name:"Close dialog",exact:true}).click();
  await provenance.getByLabel("Graph type",{exact:true}).selectOption("asset");
  await provenance.getByRole("button",{name:/^url:/}).waitFor();
  await provenance.getByLabel("Graph type",{exact:true}).selectOption("");
  await provenance.getByLabel("Graph node type",{exact:true}).selectOption("evidence");
  await provenance.getByRole("button",{name:/^evidence:/}).first().waitFor();
  await provenance.getByLabel("Search graph nodes",{exact:true}).fill(session.evidence[0].id);
  await page.waitForFunction(()=>document.querySelectorAll('[aria-label="Research provenance"] [data-graph-node]').length===1);
  await provenance.getByLabel("Search graph nodes",{exact:true}).fill("");
  await provenance.getByLabel("Graph node type",{exact:true}).selectOption("");
  await provenance.getByRole("button",{name:/^task:/}).first().waitFor();
  const canvas=provenance.getByRole("group",{name:"Provenance graph canvas",exact:true});
  await provenance.getByRole("button",{name:"Zoom in graph",exact:true}).click();
  assert((await provenance.innerText()).includes("125%"));
  const priorView=await canvas.getAttribute("viewBox");await canvas.focus();await canvas.press("ArrowRight");assert.notEqual(await canvas.getAttribute("viewBox"),priorView);
  await provenance.getByRole("button",{name:"Reset graph view",exact:true}).click();
  await provenance.getByRole("button",{name:"Focus connected nodes",exact:true}).click();
  assert(await provenance.locator("[data-graph-node]").count()<graph.nodes.length);
  await provenance.getByRole("button",{name:"Show whole graph page",exact:true}).click();
  const graphFile=path.join(artifact,"research-graph.json");
  await command(["graph","export",session.plan.id,"--output",graphFile]);
  const exported=JSON.parse(await readFile(graphFile,"utf8"));const ids=new Set(exported.nodes.map(n=>n.id));assert(exported.edges.every(e=>ids.has(e.source)&&ids.has(e.target)));
  assert.equal(calls.length,beforeGraphHttp);assert.equal(aiCalls.length,beforeGraphAi);
  for(const theme of ["Light","Dark","ReconX"]){
    await page.getByRole("button",{name:"Appearance",exact:true}).click();await page.getByRole("menuitemradio",{name:new RegExp("^"+theme)}).click();
    for(const [width,height] of [[1366,768],[1440,900],[1920,1080]]){
      await page.setViewportSize({width,height});await canvas.scrollIntoViewIfNeeded();
      assert(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth+1));
      const fill=await provenance.locator("rect").first().evaluate(el=>getComputedStyle(el).fill);assert(fill&&fill!=="rgb(0, 0, 0)","All graph themes must resolve existing color tokens");
      await page.screenshot({path:path.join(artifact,`graph-${theme.toLowerCase()}-${width}.png`)});
    }
  }
  checks.push("Persistent graph uses actual jobs/assets/evidence; native filtering, node inspection, keyboard pan/zoom, neighborhood focus, CLI JSON export and 9 themed captures pass without any new HTTP or AI invocation");
  await page.getByText("Explore stored provenance",{exact:true}).click();
  await page
    .getByRole("button", { name: "Reconnaissance", exact: true })
    .click();
  await page
    .getByLabel("Recon asset inventory", { exact: true })
    .getByText(`${origin}/`, { exact: false })
    .first()
    .waitFor();
  await page
    .getByText("ProjectDiscovery httpx: ready", { exact: false })
    .waitFor();
  const primaryAction = page.getByRole("button", {
    name: "Start AI Research",
    exact: true,
  });
  await primaryAction.focus();
  assert(await primaryAction.evaluate((el) => el === document.activeElement));
  await primaryAction.press("Enter");
  await page
    .getByRole("button", { name: "Prepare research report", exact: true })
    .waitFor();
  await page
    .getByRole("button", { name: "Reconnaissance", exact: true })
    .click();
  await page
    .getByText("ProjectDiscovery httpx: ready", { exact: false })
    .waitFor();
  for (const theme of ["Light", "Dark", "ReconX"]) {
    await page.getByRole("button", { name: "Appearance", exact: true }).click();
    await page
      .getByRole("menuitemradio", { name: new RegExp("^" + theme) })
      .click();
    for (const [width, height] of [
      [1280, 720],
      [1366, 768],
      [1440, 900],
      [1920, 1080],
    ]) {
      await page.setViewportSize({ width, height });
      await page.evaluate(() => {
        document.querySelector(".workspace-content").scrollTop = 0;
      });
      await page.evaluate(() => document.fonts.ready);
      assert(
        await page.evaluate(
          () => document.documentElement.scrollWidth <= innerWidth + 1,
        ),
      );
      await page.screenshot({
        path: path.join(artifact, `recon-${theme.toLowerCase()}-${width}.png`),
      });
    }
  }
  checks.push(
    "Populated Recon preserves target/inventory, supports keyboard primary action and renders across 3 themes and 4 resolutions (12 captures)",
  );
  await page
    .getByRole("button", { name: "Start AI Research", exact: true })
    .click();
  await page
    .getByRole("button", { name: "Prepare research report", exact: true })
    .waitFor();
  await page
    .getByRole("button", { name: "Quick actions", exact: true })
    .click();
  await page
    .getByRole("menuitem", { name: "Create Finding", exact: true })
    .click();
  const quickFinding = page.getByRole("dialog", {
    name: "Create Finding",
    exact: true,
  });
  await quickFinding.waitFor();
  await quickFinding
    .getByRole("button", { name: "Close dialog", exact: true })
    .click();
  checks.push(
    "Global Create Finding opens a real form after context synchronization, preserving manual review workflow",
  );
  await page.getByRole("button", { name: "AI Workspace", exact: true }).click();
  assert.equal(
    await page.getByLabel("Current target", { exact: true }).inputValue(),
    target.id,
  );
  await page
    .getByRole("button", { name: "Start AI Research", exact: true })
    .click();
  await page
    .getByRole("button", { name: "Prepare research report", exact: true })
    .waitFor();
  await stop();
  await launch();
  assert.equal(
    await page.getByLabel("Current target", { exact: true }).inputValue(),
    target.id,
  );
  await page
    .getByRole("button", { name: "Start AI Research", exact: true })
    .click();
  await page
    .getByRole("button", { name: "Prepare research report", exact: true })
    .waitFor();
  checks.push(
    "Navigation through Recon/AI Workspace and native restart preserve the active target and reopen persisted session/evidence/findings/report",
  );
  const otherProject = await command(["project", "create", "Research Demo 2"]);
  const otherTarget = await command([
    "target",
    "add",
    "--project",
    otherProject.id,
    "--type",
    "url",
    "--name",
    "Owned local research API",
    "--value",
    `${origin}/`,
    "--authorized",
    "--mode",
    "lab_research",
  ]);
  await page
    .getByLabel("Current project", { exact: true })
    .selectOption(otherProject.id, { timeout: 10000 });
  await page
    .getByLabel("Current target", { exact: true })
    .selectOption(otherTarget.id);
  await page
    .getByRole("button", { name: "Reconnaissance", exact: true })
    .click();
  assert.equal(
    await page.getByLabel("Inventory target", { exact: true }).inputValue(),
    otherTarget.id,
  );
  await page
    .getByLabel("Current project", { exact: true })
    .selectOption(projectId);
  await page
    .getByLabel("Current target", { exact: true })
    .selectOption(target.id);
  await page.getByRole("button", { name: "AI Workspace", exact: true }).click();
  assert.equal(
    await page.getByLabel("Current target", { exact: true }).inputValue(),
    target.id,
  );
  checks.push(
    "Two projects with identically named targets stay isolated by persistent IDs across CLI-created records, Recon and AI Workspace",
  );
  // Imported benign PE is never executed. The CLI shares the same Core/database.
  const sample = path.join(artifact, "benign-pe.exe");
  const sampleSource = path.join(artifact, "benign-sample.rs");
  await writeFile(
    sampleSource,
    'fn main() { println!("BENIGN_RESEARCH_STATIC_FIXTURE"); }\n',
  );
  await exec("rustc", [sampleSource, "-o", sample], { windowsHide: true });
  const imported = await command([
    "reverse",
    "import",
    sample,
    "--project",
    projectId,
    "--authorized",
  ]);
  await page
    .getByRole("button", { name: "Quick actions", exact: true })
    .click();
  await page.getByRole("menuitem", { name: "View Jobs", exact: true }).click();
  await page
    .getByLabel("Job file", { exact: true })
    .selectOption(imported.id, { timeout: 10000 });
  // CLI imports and the global snapshot refresh independently. Wait for the
  // actual shared selection, including the newly imported target option.
  await page.waitForFunction(
    (id) =>
      document.querySelector('select[aria-label="Current target"]')?.value ===
      id,
    imported.target_id,
  );
  assert.equal(
    await page.getByLabel("Current target", { exact: true }).inputValue(),
    imported.target_id,
  );
  assert.equal(
    (await command(["workspace", "show"])).target_id,
    imported.target_id,
  );
  await page.getByRole("button", { name: "Workflows", exact: true }).click();
  await page.waitForFunction(
    (id) =>
      document.querySelector('select[aria-label="Workflow file"]')?.value ===
      id,
    imported.id,
  );
  checks.push(
    "Manual Jobs and Agent Workflows file selectors share the same active target ID without cross-project stale selections",
  );
  await command([
    "workspace",
    "select",
    "--project",
    projectId,
    "--target",
    imported.target_id,
  ]);
  await page
    .getByLabel("Current target", { exact: true })
    .selectOption(imported.target_id, { timeout: 10000 });
  await page.getByRole("button", { name: "AI Research", exact: true }).click();
  await page
    .getByLabel("Research model")
    .selectOption("da1f803a-c5d2-4663-914b-b9f1a4ad4abc");
  // The file route runs metadata/strings only; previous HTTP count remains one.
  calls.length = 0;
  const binary = await planAndRun(false);
  assert(
    binary.jobs.every((j) =>
      ["binary_metadata", "strings_extraction"].includes(j.kind),
    ),
  );
  assert.equal(calls.length, 0);
  assert.equal(binary.evidence.length, 2);
  checks.push(
    "Benign PE route produces real metadata/strings evidence using the same Job Engine, without executing the file or network requests",
  );
  for (const theme of ["Light", "Dark", "ReconX"]) {
    await page.getByRole("button", { name: "Appearance", exact: true }).click();
    await page
      .getByRole("menuitemradio", { name: new RegExp("^" + theme) })
      .click();
    for (const [width, height] of [
      [1280, 720],
      [1366, 768],
      [1440, 900],
      [1920, 1080],
    ]) {
      await page.setViewportSize({ width, height });
      await page.evaluate(() => {
        document.querySelector(".workspace-content").scrollTop = 0;
      });
      await page.evaluate(() => document.fonts.ready);
      await delay(100);
      assert(
        await page.evaluate(
          () => document.documentElement.scrollWidth <= innerWidth + 1,
        ),
      );
      await page.screenshot({
        path: path.join(
          artifact,
          `research-${theme.toLowerCase()}-${width}.png`,
        ),
      });
    }
  }
  checks.push(
    "Research view rendered with no document overflow in 3 themes and 4 resolutions (12 captures)",
  );
  const beforeHistoryHttp = calls.length, beforeHistoryAi = aiCalls.length;
  const beforeHistory = await service({op:"research_graph",id:binary.plan.id,query:{}});
  await command(["target","remove",imported.target_id]);
  const history = await service({op:"research_graph",id:binary.plan.id,query:{}});
  assert.deepEqual(history.nodes.map(n=>n.id).sort(),beforeHistory.nodes.map(n=>n.id).sort());
  assert.equal(history.nodes.find(n=>n.kind==="target").attributes.source_state,"removed");
  const fileNode=history.nodes.find(n=>n.kind==="file");
  assert.equal(fileNode.attributes.source_state,"present");
  assert.equal(calls.length,beforeHistoryHttp);assert.equal(aiCalls.length,beforeHistoryAi);
  const historicalEvidence=history.nodes.find(n=>n.kind==="evidence");
  const inspectedHistory=await service({op:"research_graph_record",id:binary.plan.id,node:historicalEvidence.id});
  assert.equal(inspectedHistory.source_id,historicalEvidence.source_id);
  assert.equal(inspectedHistory.source_table,"evidence");
  if(inspectedHistory.omitted) assert.match(inspectedHistory.reason,/32 KiB/);
  else assert.equal(inspectedHistory.record.evidence.id,historicalEvidence.source_id);
  assert.equal((await command(["evidence","verify",historicalEvidence.source_id])).integrity,"matched");
  assert.equal((await command(["graph","nodes",binary.plan.id])).revision,history.revision);
  await page.waitForFunction(()=>document.querySelector('select[aria-label="Current target"]')?.value==="");
  await page.getByLabel("Research history",{exact:true}).selectOption(binary.plan.id);
  await page.getByText(/This target no longer exists\. Session history is read-only/).waitFor();
  assert(await page.getByRole("button",{name:"Prepare research report",exact:true}).isDisabled());
  assert(await page.getByRole("button",{name:"Save reviewable finding",exact:true}).isDisabled());
  await page.getByText(/This target no longer exists\. Session history is read-only/).scrollIntoViewIfNeeded();
  await page.screenshot({path:path.join(artifact,"history-read-only.png")});
  await page.getByText("Explore stored provenance",{exact:true}).click();
  // The earlier provenance locator belonged to the pre-restart WebView.
  const historyProvenance=page.getByLabel("Research provenance",{exact:true});
  await historyProvenance.getByRole("button",{name:/^target:/}).click();
  await page.waitForFunction(()=>document.querySelector('.graph-node-detail pre')?.textContent?.includes('"source_state": "removed"'));
  await page.setViewportSize({width:1440,height:900});
  await historyProvenance.locator('.graph-node-detail').scrollIntoViewIfNeeded();
  await page.screenshot({path:path.join(artifact,"history-removed-target.png")});
  assert.equal(calls.length,beforeHistoryHttp);assert.equal(aiCalls.length,beforeHistoryAi);
  checks.push("Actual native history selector, read-only graph/evidence inspector, CLI and screenshot retain a removed-target session without any HTTP/model execution or new finding/report writes");
  await writeFile(
    path.join(artifact, "results.json"),
    JSON.stringify(
      {
        status: "passed",
        checks,
        session_id: session.plan.id,
        binary_session_id: binary.plan.id,
        cloud_ai: "NOT_TESTED; deterministic owned-loopback mock only",
        network: "owned-loopback fixtures only",
        live_subfinder:
          "NOT_TESTED; source inventory and deterministic parser only",
      },
      null,
      2,
    ),
  );
  console.log(
    JSON.stringify({ artifact, passed: checks.length, checks }, null, 2),
  );
} catch (error) {
  if (page)
    await page
      .screenshot({ path: path.join(artifact, "failure.png") })
      .catch(() => {});
  await writeFile(
    path.join(artifact, "results.json"),
    JSON.stringify(
      { status: "failed", checks, error: String(error), stack: error.stack },
      null,
      2,
    ),
  );
  throw error;
} finally {
  await stop();
  for (const server of [owned, ai]) {
    server.closeAllConnections();
    if (server.listening) await new Promise((r) => server.close(r));
  }
}
