import { chromium } from "../../apps/desktop/node_modules/playwright/index.mjs";
import { createServer } from "node:net";
import { createServer as createHttpServer } from "node:http";
import { spawn, execFile, spawnSync } from "node:child_process";
import { promisify } from "node:util";
import { mkdir, writeFile, readFile } from "node:fs/promises";
import { createWriteStream } from "node:fs";
import { createHash, randomUUID } from "node:crypto";
import { fileURLToPath } from "node:url";
import path from "node:path";
import assert from "node:assert/strict";
const root = path.resolve(
  path.dirname(fileURLToPath(import.meta.url)),
  "../..",
);
const artifact = path.join(root, "artifacts", `http-workspace-${Date.now()}`);
await mkdir(artifact, { recursive: true });
const fixture = path.join(root, "crates/reconx-core/tests/fixtures/http");
const desktop =
  process.env.RECONX_DESKTOP_EXE ??
  path.join(root, "target/debug/reconx-desktop.exe");
const cli =
  process.env.RECONX_CLI_EXE ?? path.join(root, "target/debug/reconx.exe");
const env = {
  ...process.env,
  RECONX_DATA_DIR: artifact,
  RECONX_NATIVE_TEST_HIDDEN: "1",
  WEBVIEW2_USER_DATA_FOLDER: path.join(artifact, "webview"),
};
const exec = promisify(execFile);
const delay = (ms) => new Promise((resolve) => setTimeout(resolve, ms));
let child, browser, page;
const checks = [];
const aiCalls = [];
const mock = createHttpServer(async (req, res) => {
  let body = "";
  for await (const chunk of req) body += chunk;
  if (req.url !== "/v1/chat/completions") {
    res.statusCode = 404;
    res.end("{}");
    return;
  }
  const value = JSON.parse(body);
  aiCalls.push(value);
  res.setHeader("Content-Type", "application/json");
  res.end(
    JSON.stringify({
      model: "http-fixture-model",
      choices: [
        {
          message: {
            content:
              "Local fixture: captured-header observations only; authorization is not verified.",
          },
        },
      ],
      usage: { prompt_tokens: 20, completion_tokens: 12 },
    }),
  );
});
await new Promise((resolve) => mock.listen(0, "127.0.0.1", resolve));
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
  await new Promise((resolve) => listener.listen(0, "127.0.0.1", resolve));
  const port = listener.address().port;
  await new Promise((resolve) => listener.close(resolve));
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
    if (child.exitCode !== null) throw Error("Desktop exited");
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
  await page
    .getByRole("heading", { name: "Workspace overview", exact: true })
    .waitFor({ timeout: 30000 });
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
async function importFile(file, kind, identity = "") {
  await page
    .getByRole("button", {
      name: kind === "har" ? "Import HAR" : "Import OpenAPI",
      exact: true,
    })
    .click();
  const picker = await exec(
    "powershell.exe",
    [
      "-NoProfile",
      "-ExecutionPolicy",
      "Bypass",
      "-File",
      path.join(root, "tests/e2e/select-file.ps1"),
      "-AppProcessId",
      String(child.pid),
      "-SamplePath",
      file,
    ],
    { windowsHide: true, timeout: 30000 },
  );
  assert.match(picker.stdout, /native file dialog/);
  const dialog = page.getByRole("dialog", {
    name: "Review offline HTTP import",
    exact: true,
  });
  await dialog.waitFor();
  await dialog
    .getByLabel("HTTP import title", { exact: true })
    .fill(`Owned ${kind} ${identity || "API"} fixture`);
  if (kind === "har") {
    await dialog
      .getByLabel("Captured identity label", { exact: true })
      .fill(identity);
    await dialog
      .getByLabel("Captured role label", { exact: true })
      .fill(identity === "alice" ? "reader" : "guest");
  }
  await dialog
    .getByRole("button", { name: "Import selected files", exact: true })
    .click();
  await dialog.waitFor({ state: "hidden", timeout: 30000 });
}
try {
  const provider = {
    id: randomUUID(),
    name: "Owned HTTP interpretation fixture",
    kind: "compatible",
    base_url: `http://127.0.0.1:${mock.address().port}/v1`,
    model: "http-fixture-model",
    enabled: true,
    timeout_secs: 20,
    max_output_tokens: 1024,
    requires_key: false,
  };
  const providerFile = path.join(artifact, "provider.json");
  await writeFile(providerFile, JSON.stringify(provider));
  await command(["ai", "configure", providerFile]);
  await command(["ai", "profiles", "set-default", provider.id]);
  await launch();
  await page.getByRole("button", { name: "Projects", exact: true }).click();
  await page
    .getByLabel("Project name", { exact: true })
    .fill("HTTP Research Demo");
  await page
    .getByRole("button", { name: "Create project", exact: true })
    .click();
  await page
    .getByRole("status")
    .filter({ hasText: "Project created" })
    .waitFor();
  const project = (await command(["project", "list"]))[0];
  const target = await command([
    "target",
    "add",
    "--project",
    project.id,
    "--type",
    "rest_api",
    "--value",
    "https://api.example.test/api",
    "--name",
    "Owned captured API",
    "--authorized",
  ]);
  await page
    .getByLabel("Current project", { exact: true })
    .selectOption(project.id);
  await page.getByRole("button", { name: "Refresh", exact: true }).click();
  await page.getByRole("button", { name: "Web Security", exact: true }).click();
  await page.getByLabel("HTTP target", { exact: true }).selectOption(target.id);
  await importFile(path.join(fixture, "alice.har"), "har", "alice");
  let ws = await command(["web", "traffic", "--project", project.id]);
  assert.equal(ws.total, 2);
  assert.equal(
    ws.imports[0].sha256,
    createHash("sha256")
      .update(await readFile(path.join(fixture, "alice.har")))
      .digest("hex"),
  );
  assert.equal(ws.traffic[0].target_id, target.id);
  await page.locator(".http-explorer button.http-item").first().click();
  await page.getByRole("tab", { name: "Headers", exact: true }).click();
  assert.equal(
    await page.getByText("synthetic-auth-token", { exact: false }).count(),
    0,
  );
  await page.getByRole("tab", { name: "Body", exact: true }).click();
  await page.getByText("Owned offline capture", { exact: false }).waitFor();
  assert.equal(
    await page.getByText("synthetic-body-token", { exact: false }).count(),
    0,
  );
  checks.push(
    "Actual native picker imports HAR; Desktop/CLI share real decoded, redacted requests and SHA-256 original evidence",
  );
  await page
    .getByRole("button", { name: "Analyze captured headers", exact: true })
    .click();
  await page
    .getByText("Captured-header review · completed", { exact: true })
    .waitFor({ timeout: 30000 });
  const analysisJob = (await command(["jobs", "list"])).find(
    (j) => j.kind === "offline_http_review",
  );
  assert.equal(analysisJob.status, "completed");
  assert.equal(analysisJob.result.network_requests, 0);
  await command(["evidence", "from-job", analysisJob.id]);
  await page
    .getByRole("button", { name: "Review result with AI", exact: true })
    .click();
  const review = page.getByRole("dialog", {
    name: "Review captured evidence with AI",
    exact: true,
  });
  await review.waitFor();
  assert.equal(aiCalls.length, 0);
  assert(
    await review
      .getByRole("button", {
        name: "Analyze preview with selected model",
        exact: true,
      })
      .isDisabled(),
  );
  await review
    .getByLabel("Approve HTTP evidence transmission", { exact: true })
    .check();
  await review
    .getByRole("button", {
      name: "Analyze preview with selected model",
      exact: true,
    })
    .click();
  await review
    .getByText(/Local fixture: captured-header observations only/)
    .waitFor();
  assert.equal(aiCalls.length, 1);
  assert(JSON.stringify(aiCalls[0]).includes("content-security-policy"));
  assert(!JSON.stringify(aiCalls[0]).includes("synthetic-auth-token"));
  assert.equal(
    (await command(["workflow", "interpretations", analysisJob.id])).length,
    1,
  );
  await page.keyboard.press("Escape");
  checks.push(
    "Actual captured-header job uses the existing worker/evidence; separately approved local mock AI interpretation persists and never sees raw HAR credentials",
  );
  await importFile(path.join(fixture, "openapi30.json"), "openapi");
  await importFile(path.join(fixture, "openapi31.yaml"), "openapi");
  ws = await command(["web", "traffic", "--project", project.id]);
  assert.equal(ws.operations.length, 2);
  assert.equal(ws.endpoints[0].documented, true);
  assert.equal(ws.operations[0].parameters[0].name, "userId");
  await page.getByRole("tab", { name: "API definitions", exact: true }).click();
  await page.locator(".http-explorer button.http-item").first().click();
  await page.getByRole("heading", { name: /GET.*users/ }).waitFor();
  checks.push(
    "Native JSON/YAML OpenAPI imports expose actual parameters/security/components and correlate templated paths with HAR",
  );
  await importFile(path.join(fixture, "bob.har"), "har", "bob");
  await page.getByRole("tab", { name: "Traffic", exact: true }).click();
  await page
    .locator(".http-explorer button.http-item")
    .filter({ hasText: "alice" })
    .first()
    .click();
  await page.getByRole("tab", { name: "Diff", exact: true }).click();
  ws = await command(["web", "traffic", "--project", project.id]);
  const bob = ws.traffic.find((e) => e.identity === "bob");
  await page
    .getByLabel("Compare HTTP capture", { exact: true })
    .selectOption(bob.id);
  await page.getByLabel("Ownership field", { exact: true }).fill("owner");
  await page
    .getByRole("button", { name: "Compare captured responses", exact: true })
    .click();
  await page
    .getByRole("heading", {
      name: "Comparison · Needs manual review",
      exact: true,
    })
    .waitFor();
  await page
    .getByRole("button", { name: "Create review finding", exact: true })
    .click();
  await page
    .getByRole("status")
    .filter({ hasText: "Finding saved as Needs Verification" })
    .waitFor();
  const findings = await command(["findings", "list", "--project", project.id]);
  assert.equal(findings.length, 1);
  assert.equal(findings[0].input.status, "needs_verification");
  const comparison = (
    await command(["api", "authz-matrix", "--project", project.id])
  ).comparisons[0];
  assert.equal(comparison.network_requests, 0);
  await command(["evidence", "verify", comparison.evidence_id]);
  checks.push(
    "Native captured-role comparison persists actual ownership/status differences as evidence and a Needs Verification finding; no requests replayed",
  );
  const reportInput = path.join(artifact, "report-input.json");
  await writeFile(
    reportInput,
    JSON.stringify({
      project_id: project.id,
      title: "Owned offline HTTP assessment",
      template: "technical",
      summary: "Captured traffic review only",
      methodology:
        "HAR/OpenAPI correlation and offline comparison; no active replay",
      finding_ids: [findings[0].id],
      evidence_ids: [comparison.evidence_id],
    }),
  );
  const report = await command(["report", "prepare", "--input", reportInput]);
  for (const [format, ext] of [
    ["markdown", "md"],
    ["json", "json"],
    ["html", "html"],
    ["pdf", "pdf"],
  ])
    await command([
      "report",
      "export",
      report.report.id,
      "--format",
      format,
      "--output",
      path.join(artifact, `http-report.${ext}`),
      "--reviewed-sha256",
      report.report.sha256,
    ]);
  assert(
    (await readFile(path.join(artifact, "http-report.pdf")))
      .subarray(0, 5)
      .equals(Buffer.from("%PDF-")),
  );
  const exported = path.join(artifact, "redacted-http.json");
  await command(["web", "export", "--project", project.id, exported]);
  const safe = await readFile(exported, "utf8");
  for (const key of [
    "synthetic-auth-token",
    "synthetic-query-token",
    "synthetic-cookie-token",
    "synthetic-response-cookie",
    "synthetic-body-token",
    "synthetic-header-token",
  ])
    assert(!safe.includes(key));
  checks.push(
    "Same Desktop-created finding/evidence produces real Markdown/JSON/HTML/PDF through CLI; default HTTP export removes synthetic credentials",
  );
  await page.getByRole("tab", { name: "Notes", exact: true }).click();
  await page
    .getByLabel("HTTP analysis note", { exact: true })
    .fill("Reviewed ownership field in supplied captures.");
  await page
    .getByRole("button", { name: "Save HTTP note", exact: true })
    .click();
  await page
    .getByRole("status")
    .filter({ hasText: "HTTP analysis note saved" })
    .waitFor();
  const invalid = path.join(artifact, "invalid.har");
  await writeFile(invalid, '{"log":{}}');
  await page.getByRole("button", { name: "Import HAR", exact: true }).click();
  await exec(
    "powershell.exe",
    [
      "-NoProfile",
      "-ExecutionPolicy",
      "Bypass",
      "-File",
      path.join(root, "tests/e2e/select-file.ps1"),
      "-AppProcessId",
      String(child.pid),
      "-SamplePath",
      invalid,
    ],
    { windowsHide: true, timeout: 30000 },
  );
  const dialog = page.getByRole("dialog", {
    name: "Review offline HTTP import",
  });
  await dialog
    .getByRole("button", { name: "Import selected files", exact: true })
    .click();
  await dialog.getByRole("alert").waitFor();
  assert.equal(
    (await command(["web", "traffic", "--project", project.id])).total,
    3,
  );
  await page.keyboard.press("Escape");
  checks.push(
    "Malformed native import shows a recoverable parser error without adding partial records; capture notes persist",
  );
  await page
    .getByRole("button", { name: "Dismiss error", exact: true })
    .click();
  await page
    .getByRole("button", { name: "Refresh HTTP workspace", exact: true })
    .click();
  await page.getByRole("alert").waitFor({ state: "hidden" });
  await page
    .getByRole("tab", { name: "Authorization comparison", exact: true })
    .click();
  const matrix = page.getByRole("table", {
    name: "Captured authorization observations",
    exact: true,
  });
  await matrix.waitFor();
  assert.equal(await matrix.locator("tbody tr").count(), 2);
  assert(await matrix.innerText().then((t) => t.includes("2 capture(s)")));
  const matrixData = (
    await command(["api", "authz-matrix", "--project", project.id])
  ).authorization_matrix;
  assert.equal(matrixData.length, 2);
  assert.equal(matrixData.find((r) => r.identity === "alice").capture_count, 2);
  await page.screenshot({
    path: path.join(artifact, "http-captured-identity-matrix.png"),
    fullPage: true,
  });
  checks.push(
    "Native captured identity/role matrix agrees with shared CLI aggregation; status observations do not establish permissions",
  );
  await page.getByRole("tab", { name: "Traffic", exact: true }).click();
  await page
    .locator(".http-explorer button.http-item")
    .filter({ hasText: "alice" })
    .first()
    .click();
  await page.getByRole("tab", { name: "Body", exact: true }).click();
  for (const theme of ["Light", "Dark", "ReconX"]) {
    await page
      .getByRole("button", { name: "Topbar appearance", exact: true })
      .click();
    await page
      .getByRole("menuitemradio", { name: new RegExp("^" + theme) })
      .click();
    for (const [width, height] of [
      [1920, 1080],
      [1366, 768],
    ]) {
      await page.setViewportSize({ width, height });
      await delay(150);
      assert(
        await page.evaluate(
          () => document.documentElement.scrollWidth <= innerWidth + 1,
        ),
      );
      await page.screenshot({
        path: path.join(artifact, `http-${theme.toLowerCase()}-${width}.png`),
        fullPage: true,
      });
    }
  }
  checks.push(
    "Actual HTTP workspace screenshots in all three themes at 1920 and 1366 have no document overflow",
  );
  await stop();
  await launch();
  await page
    .getByLabel("Current project", { exact: true })
    .selectOption(project.id);
  await page.getByRole("button", { name: "API Security", exact: true }).click();
  await page
    .getByRole("heading", { name: "Traffic captures (3)", exact: true })
    .waitFor();
  ws = await command(["web", "traffic", "--project", project.id]);
  assert.equal(ws.imports.length, 4);
  assert.equal(ws.comparisons.length, 1);
  assert.equal(
    (await command(["report", "show", report.report.id])).report.sha256,
    report.report.sha256,
  );
  checks.push(
    "Full native restart preserves captures, definitions, comparison evidence, finding and report; CLI reads the same records",
  );
  const otherTarget = await command([
    "target",
    "add",
    "--project",
    project.id,
    "--type",
    "rest_api",
    "--value",
    "https://api.example.test/api/users",
    "--name",
    "Second captured context",
    "--authorized",
  ]);
  await command([
    "web",
    "import-har",
    path.join(fixture, "alice.har"),
    "--project",
    project.id,
    "--target",
    otherTarget.id,
    "--identity",
    "alice",
    "--role",
    "reader",
  ]);
  await page.getByRole("button", { name: "Refresh", exact: true }).click();
  await page
    .getByRole("button", { name: "Refresh HTTP workspace", exact: true })
    .click();
  await page.getByRole("tab", { name: "Endpoints", exact: true }).click();
  await page
    .locator(".http-explorer")
    .getByText("Second captured context", { exact: true })
    .waitFor();
  await page
    .locator(".http-explorer")
    .getByText("Undocumented observation", { exact: true })
    .waitFor();
  const partitioned = await command([
    "api",
    "authz-matrix",
    "--project",
    project.id,
  ]);
  assert.equal(partitioned.authorization_matrix.length, 3);
  assert.equal(
    partitioned.endpoints.find((e) => e.target_id === otherTarget.id)
      .documented,
    false,
  );
  assert.equal(
    partitioned.endpoints.find((e) => e.target_id === target.id).documented,
    true,
  );
  await page
    .getByRole("tab", { name: "Authorization comparison", exact: true })
    .click();
  await page
    .getByLabel("HTTP target", { exact: true })
    .selectOption(otherTarget.id);
  await page.waitForFunction(
    () =>
      document.querySelector("table.http-matrix tbody")?.children.length === 1,
  );
  const scopedMatrix = page.getByRole("table", {
    name: "Captured authorization observations",
  });
  assert((await scopedMatrix.innerText()).includes("Second captured context"));
  await scopedMatrix.scrollIntoViewIfNeeded();
  await page.screenshot({
    path: path.join(artifact, "http-target-partition-matrix.png"),
    fullPage: true,
  });
  checks.push(
    "Overlapping targets remain separate in native inventory and identity matrix; target-bound API definitions never label another target as documented",
  );
  await writeFile(
    path.join(artifact, "results.json"),
    JSON.stringify(
      {
        status: "passed",
        checks,
        network:
          "No imported request replay or cloud transmission; only explicit selected-header interpretation to owned loopback mock and WebView2 debugging",
        desktop,
        cli,
      },
      null,
      2,
    ),
  );
  console.log(
    JSON.stringify({ artifact, passed: checks.length, checks }, null, 2),
  );
} catch (error) {
  if (page) {
    await page
      .screenshot({ path: path.join(artifact, "failure.png"), fullPage: true })
      .catch(() => {});
    await writeFile(
      path.join(artifact, "failure.html"),
      await page.content().catch(() => ""),
    );
  }
  await writeFile(
    path.join(artifact, "results.json"),
    JSON.stringify({ status: "failed", checks, error: String(error) }, null, 2),
  );
  throw error;
} finally {
  await stop();
  mock.closeAllConnections();
  await new Promise((resolve) => mock.close(resolve));
}
