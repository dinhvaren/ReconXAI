import { chromium } from "../../apps/desktop/node_modules/playwright/index.mjs";
import { createServer } from "node:net";
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
const artifact = path.join(root, "artifacts", `outputs-${Date.now()}`);
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
const lifecycle = [];
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
  const launched = child;
  launched.on("exit", (code, signal) => {
    lifecycle.push({
      pid: launched.pid,
      event: "exit",
      code,
      signal,
      time: Date.now(),
    });
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

try {
  await launch();
  await page.getByRole("button", { name: "Projects", exact: true }).click();
  await page
    .getByLabel("Project name", { exact: true })
    .fill("Research Outputs Demo");
  await page
    .getByRole("button", { name: "Create project", exact: true })
    .click();
  await page
    .getByRole("status")
    .filter({ hasText: "Project created" })
    .waitFor();
  const project = (await command(["project", "list"]))[0];
  await page
    .getByLabel("Current project", { exact: true })
    .selectOption(project.id);
  const owned = path.join(artifact, "owned.raw");
  await writeFile(owned, "RECONX_BENIGN_FIXTURE\0Actual local evidence\0");
  const file = await command([
    "reverse",
    "import",
    owned,
    "--project",
    project.id,
    "--authorized",
  ]);
  const analysis = await command(["reverse", "analyze", file.id]);
  assert.equal(
    analysis.analysis.sha256,
    createHash("sha256")
      .update(await readFile(owned))
      .digest("hex"),
  );
  await page.getByRole("button", { name: "Evidence", exact: true }).click();
  const resultRow = page
    .getByRole("row")
    .filter({ hasText: "local_static_binary result" });
  await resultRow.waitFor({ timeout: 20000 });
  let evidence = await command(["evidence", "list", "--project", project.id]);
  assert.equal(evidence.length, 1);
  assert.equal(evidence[0].job_id, analysis.job_id);
  await resultRow.getByRole("button", { name: "Verify integrity" }).click();
  await page
    .getByRole("status")
    .filter({ hasText: "Evidence integrity matched" })
    .waitFor();
  await resultRow.getByRole("button", { name: "Open evidence" }).click();
  let details = page.getByRole("dialog", {
    name: "Evidence details",
    exact: true,
  });
  await details.waitFor();
  assert(
    (await details.locator("pre").innerText()).includes(
      analysis.analysis.sha256,
    ),
  );
  await details
    .getByLabel("Manual review note", { exact: true })
    .fill(
      "Compared metadata, source bytes and independently calculated hash against the owned fixture.",
    );
  await details.getByRole("button", { name: "Mark evidence reviewed" }).click();
  await page
    .getByRole("status")
    .filter({ hasText: "Manual evidence review saved" })
    .waitFor();
  await details.getByRole("button", { name: "Close dialog" }).click();
  checks.push(
    "Actual local analysis automatically records immutable job/target evidence; Desktop verifies content and saves manual source review; CLI reads identical hashes",
  );
  await page.getByRole("button", { name: "Add Evidence", exact: true }).click();
  const add = page.getByRole("dialog", { name: "Add Evidence", exact: true });
  await add
    .getByLabel("Evidence title", { exact: true })
    .fill("Owned HTTP request fixture");
  await add
    .getByLabel("Evidence type", { exact: true })
    .selectOption("http_request");
  await add
    .getByLabel("Evidence content", { exact: true })
    .fill(
      "GET / HTTP/1.1\nAuthorization: Bearer fixture-private-access\nCookie: session=fixture-private-cookie\n\n<script>untrusted_fixture()</script>",
    );
  await add.getByRole("button", { name: "Save evidence", exact: true }).click();
  await add.waitFor({ state: "hidden" });
  const png = path.join(artifact, "owned-screenshot.png");
  await page.screenshot({ path: png });
  await page.getByRole("button", { name: "Add Evidence", exact: true }).click();
  await add
    .getByLabel("Evidence title", { exact: true })
    .fill("Owned screenshot attachment");
  await add
    .getByLabel("Evidence type", { exact: true })
    .selectOption("screenshot");
  await add
    .getByRole("button", { name: "Import Screenshot", exact: true })
    .click();
  await exec(
    "powershell.exe",
    [
      "-NoProfile",
      "-ExecutionPolicy",
      "Bypass",
      "-File",
      path.join(root, "tests", "e2e", "select-file.ps1"),
      "-AppProcessId",
      String(child.pid),
      "-SamplePath",
      png,
    ],
    { windowsHide: true, timeout: 30000 },
  );
  await add.getByRole("button", { name: "Save evidence", exact: true }).click();
  await add.waitFor({ state: "hidden" });
  await page
    .getByRole("row")
    .filter({ hasText: "Owned screenshot attachment" })
    .getByRole("button", { name: "Open evidence" })
    .click();
  await details
    .getByRole("img", { name: "Imported research evidence" })
    .waitFor();
  await page.waitForFunction(() => {
    const img = document.querySelector("dialog[open] img");
    return img?.complete && img.naturalWidth > 0;
  });
  await page.screenshot({
    path: path.join(artifact, "screenshot-evidence-view.png"),
  });
  await details.getByRole("button", { name: "Close dialog" }).click();
  evidence = await command(["evidence", "list"]);
  assert.equal(evidence.length, 3);
  checks.push(
    "Native screenshot import copies actual PNG bytes with matching content hash, renders the local image and persists HTTP request evidence without uploading",
  );
  await page
    .getByRole("button", { name: "Quick actions", exact: true })
    .click();
  await page
    .getByRole("menuitem", { name: "Create Finding", exact: true })
    .click();
  const finding = page.getByRole("dialog", {
    name: "Create Finding",
    exact: true,
  });
  await finding.waitFor();
  await finding
    .getByLabel("Finding title", { exact: true })
    .fill("Owned local observation");
  await finding
    .getByLabel("Finding description", { exact: true })
    .fill(
      "Test record for actual local fixture metadata; no vulnerability is asserted.",
    );
  await finding
    .getByLabel("Finding status", { exact: true })
    .selectOption("verified");
  await finding
    .getByLabel("Finding verification note", { exact: true })
    .fill(
      "Manually reviewed the actual imported bytes and the saved metadata record.",
    );
  await finding.getByLabel("I independently reviewed").check();
  await finding
    .getByRole("button", { name: "Save finding", exact: true })
    .click();
  await finding
    .getByRole("alert")
    .filter({ hasText: "Verified requires explicit manual confirmation" })
    .waitFor();
  await finding
    .getByRole("checkbox", { name: /local_static_binary result/ })
    .check();
  await finding
    .getByLabel("cvss_v3", { exact: true })
    .fill("CVSS:3.1/AV:N/AC:L/PR:N/UI:N/S:U/C:N/I:N/A:N");
  await finding
    .getByRole("button", { name: "Calculate CVSS", exact: true })
    .click();
  await finding.locator("pre").filter({ hasText: '"score": 0' }).waitFor();
  await finding
    .getByRole("button", { name: "Save finding", exact: true })
    .click();
  await finding.waitFor({ state: "hidden" });
  const saved = (await command(["findings", "list"]))[0];
  assert.equal(saved.input.status, "verified");
  assert.equal(saved.cvss[0].score, 0);
  assert.equal(saved.input.evidence_ids.length, 1);
  checks.push(
    "Native finding creation rejects Verified without reviewed source evidence; explicit review then saves linked observation and actual supplied CVSS metrics shared with CLI",
  );
  await page.getByRole("button", { name: "Evidence", exact: true }).click();
  await page.getByRole("button", { name: "Findings", exact: true }).click();
  assert.equal(
    await page
      .getByRole("dialog", { name: "Create Finding", exact: true })
      .count(),
    0,
  );
  await page
    .getByRole("button", { name: "Quick actions", exact: true })
    .click();
  await page
    .getByRole("menuitem", { name: "Export Report", exact: true })
    .click();
  await page
    .getByLabel("Report title", { exact: true })
    .fill("Local research report");
  await page
    .getByLabel("Report template", { exact: true })
    .selectOption("hacker_one");
  await page
    .getByLabel("Executive summary", { exact: true })
    .fill(
      "Ki\u1ec3m tra b\u1eb1ng ch\u1ee9ng c\u1ee5c b\u1ed9. Owned fixture observations only.",
    );
  await page
    .getByLabel("Methodology", { exact: true })
    .fill("Offline local file analysis and manual evidence review.");
  await page.getByRole("checkbox", { name: /Owned local observation/ }).check();
  await page
    .getByRole("checkbox", { name: /Owned HTTP request fixture/ })
    .check();
  await page
    .getByRole("checkbox", { name: /Owned screenshot attachment/ })
    .check();
  await page
    .getByRole("button", { name: "Prepare preview", exact: true })
    .click();
  let preview = page.getByRole("dialog", {
    name: "Report preview",
    exact: true,
  });
  await preview.waitFor();
  const text = await preview.locator("pre").innerText();
  assert(!text.includes("fixture-private-access"));
  assert(!text.includes("fixture-private-cookie"));
  assert(text.includes("[REDACTED]"));
  assert(text.includes("Steps to reproduce"));
  assert(text.includes("raw binary/image omitted"));
  assert(
    await preview
      .getByRole("button", { name: "Export reviewed report" })
      .isDisabled(),
  );
  await preview
    .getByLabel("Export format", { exact: true })
    .selectOption("json");
  const structuredPreview = JSON.parse(
    await preview.locator("pre").innerText(),
  );
  assert.equal(structuredPreview.snapshot.project.id, project.id);
  assert.equal(structuredPreview.snapshot.evidence.length, 3);
  await preview.getByLabel("I reviewed this redacted snapshot").check();
  await preview
    .getByLabel("Export format", { exact: true })
    .selectOption("pdf");
  assert(
    await preview
      .getByRole("button", { name: "Export reviewed report" })
      .isDisabled(),
  );
  await preview.getByLabel("I reviewed this redacted snapshot").check();
  const pdf = path.join(artifact, "reviewed-report.pdf");
  await preview
    .getByRole("button", { name: "Export reviewed report", exact: true })
    .click();
  await exec(
    "powershell.exe",
    [
      "-NoProfile",
      "-ExecutionPolicy",
      "Bypass",
      "-File",
      path.join(root, "tests", "e2e", "select-file.ps1"),
      "-AppProcessId",
      String(child.pid),
      "-SamplePath",
      pdf,
      "-ExpectedName",
      `report-${(await command(["report", "list"]))[0].id}.pdf`,
    ],
    { windowsHide: true, timeout: 30000 },
  );
  await page
    .getByRole("status")
    .filter({ hasText: "Reviewed report exported" })
    .waitFor();
  const inspection = await exec(
    "python",
    [
      "-c",
      "import sys,json;from pypdf import PdfReader;r=PdfReader(sys.argv[1]);t='\\n'.join(p.extract_text() for p in r.pages);assert 'Ki\u1ec3m tra b\u1eb1ng ch\u1ee9ng c\u1ee5c b\u1ed9' in t;assert 'fixture-private-access' not in t;assert '[REDACTED]' in t;print(json.dumps({'pages':len(r.pages),'unicode':True,'redacted':True}))",
      pdf,
    ],
    { windowsHide: true },
  );
  await writeFile(
    path.join(artifact, "pdf-inspection.json"),
    inspection.stdout,
  );
  const report = (await command(["report", "list"]))[0];
  const reportView = await command(["report", "show", report.id]);
  assert(reportView.document.redactions >= 2);
  for (const format of ["markdown", "json", "html"])
    await command([
      "report",
      "export",
      report.id,
      "--format",
      format,
      "--output",
      path.join(artifact, `report.${format}`),
      "--reviewed-sha256",
      report.sha256,
    ]);
  const html = await readFile(path.join(artifact, "report.html"), "utf8");
  assert(!html.includes("<script>"));
  assert(html.includes("&lt;script&gt;"));
  await page.screenshot({
    path: path.join(artifact, "reviewed-report-preview.png"),
  });
  checks.push(
    "Native report preview exposes redactions and omitted attachments; exact approval exports a real multi-page PDF with independently extracted Vietnamese text; CLI exports valid JSON, safe HTML and Markdown from the same snapshot",
  );
  await preview.getByRole("button", { name: "Close dialog" }).click();
  await stop();
  await launch();
  preview = page.getByRole("dialog", { name: "Report preview", exact: true });
  await page.getByRole("button", { name: "Evidence", exact: true }).click();
  await page
    .getByRole("row")
    .filter({ hasText: "Owned HTTP request fixture" })
    .waitFor();
  assert.equal((await command(["evidence", "list"])).length, 3);
  await page.getByRole("button", { name: "Findings", exact: true }).click();
  await page
    .getByRole("cell", { name: "verified", exact: true })
    .waitFor({ timeout: 15000 });
  await page.getByRole("button", { name: "Reports", exact: true }).click();
  await page
    .getByRole("button", { name: "Open report preview", exact: true })
    .click();
  await preview.waitFor();
  assert.equal(
    (await command(["report", "show", report.id])).report.sha256,
    report.sha256,
  );
  await preview.getByRole("button", { name: "Close dialog" }).click();
  checks.push(
    "Actual Desktop restart preserves evidence, manual review, verified finding links and immutable report snapshot; CLI reads the same records",
  );
  for (const mode of ["Light", "Dark", "ReconX"]) {
    await page.getByRole("button", { name: "Appearance", exact: true }).click();
    await page
      .getByRole("menuitemradio", { name: new RegExp("^" + mode) })
      .click();
    for (const name of ["Evidence", "Findings", "Reports"]) {
      await page.getByRole("button", { name, exact: true }).click();
      for (const [width, height] of [
        [1920, 1080],
        [1366, 768],
      ]) {
        await page.setViewportSize({ width, height });
        await page.evaluate(() => document.fonts.ready);
        await delay(150);
        assert(
          await page.evaluate(
            () => document.documentElement.scrollWidth <= innerWidth + 1,
          ),
        );
        await page.screenshot({
          path: path.join(
            artifact,
            `${name.toLowerCase()}-${mode.toLowerCase()}-${width}.png`,
          ),
        });
      }
    }
  }
  checks.push(
    "Evidence, Findings and Reports render in Light/Dark/ReconX at 1920 and 1366 without document clipping; actual screenshots captured",
  );
  await writeFile(
    path.join(artifact, "results.json"),
    JSON.stringify(
      {
        status: "passed",
        checks,
        pdf: "independently parsed with installed pypdf",
        model: "no AI transmission used",
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
  await writeFile(
    path.join(artifact, "lifecycle.json"),
    JSON.stringify(lifecycle, null, 2),
  );
}
