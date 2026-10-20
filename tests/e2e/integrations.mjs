// Actual Tauri/WebView2 + native picker + one isolated shared SQLite workspace.
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
const artifact = path.join(root, "artifacts", `integrations-${Date.now()}`);
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
  RECONX_TEST_PROVIDER_SECRET: "fixture-marker-not-a-real-credential",
  WEBVIEW2_USER_DATA_FOLDER: path.join(artifact, "webview"),
};
const exec = promisify(execFile),
  delay = (ms) => new Promise((r) => setTimeout(r, ms));
const checks = [],
  observed = [],
  calls = [];
let child, browser, page;
const mock = httpServer(async (req, res) => {
  let body = "";
  for await (const part of req) body += part;
  calls.push({ url: req.url, body: JSON.parse(body || "{}") });
  res.setHeader("Content-Type", "application/json");
  res.end(
    JSON.stringify({
      choices: [
        {
          message: {
            content: body.includes(
              "Choose 1 to 5 suitable offline file capabilities",
            )
              ? JSON.stringify({
                  steps: [
                    {
                      capability_id: "local.metadata",
                      reason: "Inspect real metadata",
                    },
                    {
                      capability_id: "yara-x.signatures",
                      reason: "Match fixed offline file signatures",
                    },
                  ],
                })
              : "Deterministic local model interpretation of actual tool evidence. No vulnerability is asserted.",
          },
        },
      ],
    }),
  );
});
await new Promise((r) => mock.listen(0, "127.0.0.1", r));
async function command(args) {
  return JSON.parse(
    (
      await exec(cli, ["--json", ...args], {
        env,
        windowsHide: true,
        maxBuffer: 32 * 1024 * 1024,
      })
    ).stdout,
  );
}
async function service(op, args = {}) {
  return page.evaluate(
    async ({ op, args }) =>
      window.__TAURI_INTERNALS__.invoke("service", {
        request: { op, ...args },
      }),
    { op, args },
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
      if ((await fetch(`http://127.0.0.1:${port}/json/version`)).ok) {
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
  await page
    .getByRole("heading", { name: "Workspace overview", exact: true })
    .waitFor({ timeout: 30000 });
  await page.exposeFunction("recordNativeJob", (job) => observed.push(job));
  await page.evaluate(async () => {
    const handler = window.__TAURI_INTERNALS__.transformCallback((event) => {
      void window.recordNativeJob(event.payload);
    });
    await window.__TAURI_INTERNALS__.invoke("plugin:event|listen", {
      event: "research-job",
      target: { kind: "Any" },
      handler,
    });
  });
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
    await delay(300);
  }
}

async function nav(name) {
  await page
    .getByRole("button", {
      name: name === "Agent Workflows" ? "Workflows" : name,
      exact: true,
    })
    .click();
  await page.getByRole("heading", { name, exact: true }).waitFor();
}
async function waitJob(id) {
  const end = Date.now() + 30000;
  while (Date.now() < end) {
    const j = await command(["jobs", "show", id]);
    if (!["queued", "running"].includes(j.status)) return j;
    await delay(100);
  }
  throw Error("Job did not finish: " + id);
}
async function theme(name) {
  await page.getByRole("button", { name: "Appearance", exact: true }).click();
  await page
    .getByRole("menuitemradio", { name: new RegExp("^" + name) })
    .click();
  await page.waitForFunction(
    (n) => document.documentElement.dataset.theme === n,
    name.toLowerCase(),
  );
}
try {
  await launch();
  await page.setViewportSize({ width: 1600, height: 900 });
  const project = await service("create_project", {
    name: "Milestone 2 Demo",
    description: "Owned integration fixture",
  });
  const sample = path.join(artifact, "owned-fixture.bin");
  await writeFile(sample, "RECONX_BENIGN_FIXTURE\0owned local analysis");
  const file = await service("file_import", {
    project: project.id,
    path: sample,
    authorized: true,
    folder: false,
  });
  const provider = {
    id: "5fec4949-d9a4-48a9-a364-45fc0cfd5e8e",
    name: "Deterministic local fixture",
    kind: "compatible",
    base_url: `http://127.0.0.1:${mock.address().port}/v1`,
    model: "fixture",
    enabled: true,
    timeout_secs: 10,
    max_output_tokens: 1024,
    requires_key: false,
  };
  await service("save_provider", { provider });
  await nav("Tool Manager");
  const row = page
    .locator("article.provider-row")
    .filter({ hasText: "YARA-X" });
  await row
    .getByRole("button", { name: "Review install plan", exact: true })
    .click();
  await page
    .getByRole("heading", { name: /Installation plan.*YARA-X/ })
    .waitFor();
  await page
    .getByRole("button", { name: "Install verified release", exact: true })
    .click();
  await row.getByText("tested", { exact: true }).waitFor({ timeout: 150000 });
  const installed = (await command(["tools", "list"])).find(
    (t) => t.manifest.id === "yara-x",
  );
  assert(installed.integrity_verified);
  assert.equal(installed.status, "tested");
  assert.equal(installed.manifest.version, "1.21.0");
  assert.equal((await command(["tools", "verify", "yara-x"])).status, "tested");
  assert.equal(
    (await command(["tools", "info", "yara-x"])).integration,
    "implemented",
  );
  assert(
    (await command(["tools", "search", "semgrep"])).some(
      (t) => t.integration === "pending",
    ),
  );
  await page.screenshot({ path: path.join(artifact, "tool-installed.png") });
  checks.push(
    "Native UI downloads official YARA-X, verifies both hashes, runs real version check; CLI shares receipt",
  );
  await nav("MCP Hub");
  await page
    .getByRole("button", { name: "Add MCP Server", exact: true })
    .click();
  await page
    .getByLabel("Server name", { exact: true })
    .fill("Owned local MCP fixture");
  await page
    .getByLabel("MCP executable", { exact: true })
    .fill(process.execPath);
  await page
    .getByLabel("MCP arguments", { exact: true })
    .fill(JSON.stringify([path.join(root, "tests/e2e/mcp-fixture.mjs")]));
  await page.getByLabel("MCP allowed tools", { exact: true }).fill("echo");
  await page.getByLabel("Trust MCP server", { exact: true }).check();
  await page.getByRole("button", { name: "Save server", exact: true }).click();
  await page
    .getByRole("button", { name: "Connect / Discover", exact: true })
    .click();
  await page.getByLabel("MCP tool", { exact: true }).selectOption("echo");
  const server = (await command(["mcp", "list"]))[0];
  assert.equal(server.tools.length, 2);
  assert.equal(
    (await command(["mcp", "inspect", server.config.id, "echo"])).name,
    "echo",
  );
  await page
    .getByLabel("MCP input", { exact: true })
    .fill(JSON.stringify({ text: "real approved MCP response" }));
  await page
    .getByLabel("MCP project", { exact: true })
    .selectOption(project.id);
  await page
    .getByRole("button", { name: "Review invocation", exact: true })
    .click();
  await page
    .getByRole("dialog", { name: "Approve MCP invocation", exact: true })
    .waitFor();
  await page.screenshot({ path: path.join(artifact, "mcp-approval.png") });
  await page
    .getByRole("button", { name: "Approve and queue", exact: true })
    .click();
  let history = [];
  for (let i = 0; i < 100; i++) {
    history = await command(["mcp", "history", server.config.id]);
    if (history.length) break;
    await delay(100);
  }
  assert(history.length);
  const mcpJob = await waitJob(history[0].id);
  assert.equal(mcpJob.status, "completed", JSON.stringify(mcpJob));
  assert(
    mcpJob.result.response.structuredContent.echo.includes("real approved"),
  );
  assert.equal(
    mcpJob.result.response.structuredContent.host_secret_inherited,
    false,
  );
  await page.getByText("Completed", { exact: true }).first().waitFor();
  await page.screenshot({ path: path.join(artifact, "mcp-history.png") });
  await page.getByLabel("MCP tool", { exact: true }).selectOption("denied");
  assert(
    await page
      .getByRole("button", { name: "Review invocation", exact: true })
      .isDisabled(),
  );
  checks.push(
    "Native MCP configuration, actual stdio discovery/schema inspection, explicit approval, real Job Engine response/history; denied tools blocked and host secret environment excluded",
  );
  await nav("Agent Workflows");
  await page
    .getByLabel("Workflow project", { exact: true })
    .selectOption(project.id);
  await page.getByLabel("Workflow file", { exact: true }).selectOption(file.id);
  await page
    .getByLabel("Workflow provider", { exact: true })
    .selectOption(provider.id);
  await page
    .getByRole("button", { name: "Preview planning context", exact: true })
    .click();
  await page
    .getByLabel("Approve planning metadata transmission", { exact: true })
    .waitFor();
  assert(
    await page
      .getByRole("button", { name: "Generate structured plan", exact: true })
      .isDisabled(),
  );
  assert.equal(calls.length, 0);
  await page
    .getByLabel("Approve planning metadata transmission", { exact: true })
    .check();
  await page
    .getByRole("button", { name: "Generate structured plan", exact: true })
    .click();
  await page
    .getByRole("button", { name: "Approve local plan and queue", exact: true })
    .waitFor();
  assert.equal(calls.length, 1);
  assert(JSON.stringify(calls[0]).includes("yara-x.signatures"));
  assert(!JSON.stringify(calls[0]).includes(sample));
  await page.screenshot({
    path: path.join(artifact, "ai-plan-awaiting-approval.png"),
  });
  await page
    .getByRole("button", { name: "Approve local plan and queue", exact: true })
    .click();
  const plans = await command(["workflow", "list", "--project", project.id]);
  assert.equal(plans.length, 1);
  assert.equal(plans[0].jobs.length, 2);
  const results = [];
  for (const id of plans[0].jobs) {
    const j = await waitJob(id);
    assert.equal(j.status, "completed", JSON.stringify(j));
    results.push(j);
  }
  const yara = results.find((j) => j.kind === "yara_signatures");
  assert(yara.result.observations);
  assert(JSON.stringify(yara.result).includes("ReconX_Test_Marker"));
  assert.equal(
    yara.result.sample_sha256,
    createHash("sha256")
      .update(await readFile(sample))
      .digest("hex"),
  );
  const eventDeadline = Date.now() + 5000;
  while (
    !observed.some((j) => j.id === yara.id && j.status === "completed") &&
    Date.now() < eventDeadline
  )
    await delay(30);
  assert(
    observed.some(
      (j) => j.kind === "yara_signatures" && j.status === "running",
    ),
  );
  assert(
    observed.some(
      (j) => j.kind === "yara_signatures" && j.status === "completed",
    ),
  );
  checks.push(
    "Deterministic mock model selects registered capabilities; native consent and approval cause real local metadata/YARA-X jobs with independent SHA-256, actual signatures and native events",
  );
  const evidenceRow = page
    .locator(".integration-evidence")
    .filter({ hasText: "yara signatures" });
  await evidenceRow
    .getByRole("button", { name: "Review evidence with AI", exact: true })
    .click();
  await page
    .getByLabel("Approve evidence transmission", { exact: true })
    .check();
  await page
    .getByRole("button", { name: "Interpret evidence", exact: true })
    .click();
  await page
    .getByText(
      "Deterministic local model interpretation of actual tool evidence. No vulnerability is asserted.",
      { exact: true },
    )
    .waitFor();
  assert.equal(calls.length, 2);
  await page.screenshot({
    path: path.join(artifact, "ai-evidence-interpretation.png"),
  });
  await page
    .getByRole("dialog", { name: "Review selected job evidence", exact: true })
    .getByRole("button", { name: "Close dialog", exact: true })
    .click();
  checks.push(
    "Explicit evidence consent sends actual selected local tool result to local mock provider; interpretation saved as unverified",
  );
  for (const t of ["Light", "Dark", "ReconX"]) {
    await theme(t);
    await page.screenshot({
      path: path.join(artifact, `workflows-${t.toLowerCase()}.png`),
    });
    await nav("MCP Hub");
    await page.getByRole("button", { name: /Owned local MCP fixture/ }).click();
    await page.getByLabel("MCP tool", { exact: true }).selectOption("echo");
    await delay(300);
    await page.screenshot({
      path: path.join(artifact, `mcp-${t.toLowerCase()}.png`),
    });
    await nav("Agent Workflows");
    await page
      .getByLabel("Workflow project", { exact: true })
      .selectOption(project.id);
    await page
      .getByRole("button", { name: "Approval consumed", exact: true })
      .waitFor();
    await delay(300);
  }
  await page.setViewportSize({ width: 1366, height: 768 });
  assert(
    await page.evaluate(
      () => document.documentElement.scrollWidth <= innerWidth + 1,
    ),
  );
  await page.screenshot({ path: path.join(artifact, "workflows-1366.png") });
  await stop();
  await launch();
  assert.equal(
    (await command(["tools", "list"])).find((t) => t.manifest.id === "yara-x")
      .status,
    "tested",
  );
  assert.equal(
    (await command(["mcp", "history", server.config.id]))[0].status,
    "completed",
  );
  assert.equal(
    (await command(["workflow", "list", "--project", project.id]))[0].jobs
      .length,
    2,
  );
  await nav("MCP Hub");
  await page.getByRole("button", { name: /Owned local MCP fixture/ }).click();
  await page
    .getByRole("button", { name: "Connect / Discover", exact: true })
    .click();
  await page.getByLabel("MCP tool", { exact: true }).selectOption("echo");
  await page.getByRole("button", { name: "Disconnect", exact: true }).click();
  checks.push(
    "Restart preserves managed tools, MCP permissions/discovery/history, plans and tool jobs; actual reconnect/disconnect works; three themes retained at desktop sizes",
  );
  assert.equal(
    (await command(["workflow", "interpretations", yara.id])).length,
    1,
  );
  await command(["tools", "update", "yara-x"]);
  await command(["tools", "remove", "yara-x"]);
  await nav("Tool Manager");
  const repairedRow = page
    .locator("article.provider-row")
    .filter({ hasText: "YARA-X" });
  await repairedRow.getByText("not installed", { exact: true }).waitFor();
  await repairedRow
    .getByRole("button", { name: "Repair", exact: true })
    .click();
  await repairedRow
    .getByText("tested", { exact: true })
    .waitFor({ timeout: 150000 });
  await repairedRow
    .getByRole("button", { name: "Open logs", exact: true })
    .click();
  await page
    .getByRole("dialog", { name: "Tool operation history", exact: true })
    .waitFor();
  await page.screenshot({
    path: path.join(artifact, "tool-operation-history.png"),
  });
  await page
    .getByRole("dialog", { name: "Tool operation history", exact: true })
    .getByRole("button", { name: "Close dialog", exact: true })
    .click();
  await page
    .getByRole("button", { name: "Review profile plan", exact: true })
    .click();
  await page
    .getByRole("button", { name: "Approve profile installation", exact: true })
    .click();
  for (let i = 0; i < 100; i++) {
    const tools = await command(["tools", "list"]);
    if (tools.every((t) => t.status === "tested")) break;
    await delay(100);
  }
  assert(
    (await command(["tools", "list"])).every((t) => t.status === "tested"),
  );
  checks.push(
    "Pinned update, managed removal and native Repair reinstall/verify succeed; operation logs and reviewed Reverse profile install share CLI state",
  );
  await writeFile(
    path.join(artifact, "events.json"),
    JSON.stringify(observed, null, 2),
  );
  await writeFile(
    path.join(artifact, "results.json"),
    JSON.stringify(
      {
        platform: process.platform,
        model: "deterministic local HTTP mock; no cloud credentials used",
        passed: checks.length,
        checks,
      },
      null,
      2,
    ),
  );
  console.log(
    JSON.stringify({ artifact, passed: checks.length, checks }, null, 2),
  );
} catch (error) {
  await writeFile(
    path.join(artifact, "events.json"),
    JSON.stringify(observed, null, 2),
  );
  if (page)
    await page
      .screenshot({ path: path.join(artifact, "failure.png") })
      .catch(() => {});
  await writeFile(
    path.join(artifact, "results.json"),
    JSON.stringify(
      {
        passed: checks.length,
        checks,
        error: String(error),
        stack: error.stack,
      },
      null,
      2,
    ),
  );
  throw error;
} finally {
  await stop();
  await new Promise((r) => mock.close(r));
}
