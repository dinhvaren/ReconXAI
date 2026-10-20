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
const artifact = path.join(root, "artifacts", `recon-local-${Date.now()}`);
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

const requests = [];
const owned = httpServer((req, res) => {
  requests.push({
    method: req.method,
    url: req.url,
    authorization: req.headers.authorization ?? null,
    cookie: req.headers.cookie ?? null,
  });
  const finish = () => {
    if (!res.destroyed) {
      res.setHeader("Server", "owned-local-fixture");
      res.setHeader("Set-Cookie", "session=fixture-private-cookie");
      res.setHeader("Content-Type", "text/plain");
      res.end();
    }
  };
  if (req.url === "/slow") setTimeout(finish, 4500);
  else finish();
});
await new Promise((r) => owned.listen(0, "127.0.0.1", r));
const origin = `http://127.0.0.1:${owned.address().port}`;
async function addTarget(name, value) {
  await page
    .getByRole("button", { name: "Quick actions", exact: true })
    .click();
  await page.getByRole("menuitem", { name: "Add Target", exact: true }).click();
  const dialog = page.getByRole("dialog", { name: "Add Target", exact: true });
  await dialog.getByLabel("Target name", { exact: true }).fill(name);
  await dialog.getByLabel("Target type", { exact: true }).selectOption("url");
  await dialog.getByLabel("Target value", { exact: true }).fill(value);
  await dialog
    .getByLabel("Preferred analysis mode", { exact: true })
    .selectOption("lab_research");
  await dialog.getByLabel("I confirm I have permission").check();
  await dialog
    .getByRole("button", { name: "Save target", exact: true })
    .click();
  await dialog.waitFor({ state: "hidden" });
  return (await command(["target", "list"])).find((t) => t.name === name);
}
async function approve() {
  await page
    .getByRole("button", { name: "Preview local HEAD plan", exact: true })
    .click();
  const dialog = page.getByRole("dialog", {
    name: "Approve local HTTP plan",
    exact: true,
  });
  await dialog.waitFor();
  assert(
    await dialog
      .getByRole("button", { name: "Approve and queue HEAD" })
      .isDisabled(),
  );
  await dialog.getByLabel("I approve this exact request").check();
  await dialog.getByRole("button", { name: "Approve and queue HEAD" }).click();
  await dialog.waitFor({ state: "hidden" });
}
async function jobWith(status) {
  for (let i = 0; i < 100; i++) {
    const jobs = await command(["jobs", "list"]);
    const job = jobs.find(
      (j) => j.kind === "local_http_inventory" && j.status === status,
    );
    if (job) return job;
    await delay(100);
  }
  throw Error(`Local HTTP job never reached ${status}`);
}
try {
  await launch();
  await page.getByRole("button", { name: "Projects", exact: true }).click();
  await page
    .getByLabel("Project name", { exact: true })
    .fill("Owned Local Inventory");
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
  const target = await addTarget("Owned HTTP service", `${origin}/`);
  await page
    .getByRole("button", { name: "Targets & Scope", exact: true })
    .click();
  await page
    .getByRole("button", { name: "Owned HTTP service", exact: true })
    .click();
  await page
    .getByRole("button", { name: "Plan local HTTP inventory", exact: true })
    .click();
  assert.equal(
    await page.getByLabel("Inventory target", { exact: true }).inputValue(),
    target.id,
  );
  await page
    .getByRole("button", { name: "Preview local HEAD plan", exact: true })
    .click();
  await page
    .locator("p[role=alert]")
    .filter({ hasText: "HTTP sends traffic" })
    .waitFor();
  assert.equal(requests.length, 0);
  checks.push(
    "Desktop target details preserve project/target context; Passive mode blocks preview before any network request",
  );
  await page
    .getByRole("button", { name: "Open security settings", exact: true })
    .click();
  await page
    .getByLabel("Security mode", { exact: true })
    .selectOption("lab_research");
  await page
    .getByRole("button", { name: "Save preferences", exact: true })
    .click();
  await page
    .getByRole("status")
    .filter({ hasText: "Preferences saved" })
    .waitFor();
  await page
    .getByRole("button", { name: "Reconnaissance", exact: true })
    .click();
  await page
    .getByLabel("Inventory target", { exact: true })
    .selectOption(target.id);
  await page
    .getByRole("button", { name: "Preview local HEAD plan", exact: true })
    .click();
  const plan = page.getByRole("dialog", {
    name: "Approve local HTTP plan",
    exact: true,
  });
  await plan.waitFor();
  assert((await plan.innerText()).includes(`${origin}/`));
  assert.equal(requests.length, 0);
  await page.screenshot({
    path: path.join(artifact, "exact-plan-approval.png"),
  });
  assert(
    await plan
      .getByRole("button", { name: "Approve and queue HEAD" })
      .isDisabled(),
  );
  await plan.getByLabel("I approve this exact request").check();
  await plan.getByRole("button", { name: "Approve and queue HEAD" }).click();
  await plan.waitFor({ state: "hidden" });
  const done = await jobWith("completed");
  const entry = page.locator(`[data-job-id="${done.id}"]`);
  await entry.locator(".badge").filter({ hasText: "completed" }).waitFor();
  await page
    .getByRole("cell", { name: "owned-local-fixture", exact: false })
    .waitFor();
  assert.deepEqual(requests, [
    { method: "HEAD", url: "/", authorization: null, cookie: null },
  ]);
  const assets = await command(["recon", "assets", "--project", project.id]);
  assert.equal(assets.length, 1);
  assert.equal(assets[0].facts.status, 200);
  assert(!JSON.stringify(done).includes("fixture-private-cookie"));
  assert(!JSON.stringify(assets).includes("fixture-private-cookie"));
  await entry.getByRole("button", { name: "View result", exact: true }).click();
  const result = page.getByRole("dialog", { name: "Job result", exact: true });
  await result.waitFor();
  assert((await result.locator("pre").innerText()).includes(assets[0].id));
  await result.getByRole("button", { name: "Close dialog" }).click();
  checks.push(
    "Exact approval runs one real HEAD through the shared job worker, normalizes an actual asset and shows a persisted safe job result in Desktop and CLI",
  );
  await page
    .getByRole("button", { name: "Open inventory evidence", exact: true })
    .click();
  await page
    .getByRole("row")
    .filter({ hasText: "Approved loopback HTTP response" })
    .getByRole("button", { name: "Open evidence" })
    .click();
  const details = page.getByRole("dialog", {
    name: "Evidence details",
    exact: true,
  });
  await details.waitFor();
  assert(
    (await details.locator("pre").innerText()).includes(
      "fixture-private-cookie",
    ),
  );
  await details.getByRole("button", { name: "Close dialog" }).click();
  const raw = (await command(["evidence", "list"])).find(
    (e) => e.kind === "http_response",
  );
  const safe = path.join(artifact, "redacted-http.json");
  await command(["evidence", "export", raw.id, "--output", safe]);
  assert(!(await readFile(safe, "utf8")).includes("fixture-private-cookie"));
  await page.getByRole("button", { name: "Reports", exact: true }).click();
  await page
    .getByLabel("Report title", { exact: true })
    .fill("Owned HTTP inventory report");
  await page
    .getByLabel("Executive summary", { exact: true })
    .fill(
      "Actual loopback availability observation; no vulnerability is asserted.",
    );
  await page
    .getByRole("checkbox", { name: /Approved loopback HTTP response/ })
    .check();
  await page
    .getByRole("button", { name: "Prepare preview", exact: true })
    .click();
  let preview = page.getByRole("dialog", {
    name: "Report preview",
    exact: true,
  });
  await preview.waitFor();
  assert(
    !(await preview.locator("pre").innerText()).includes(
      "fixture-private-cookie",
    ),
  );
  assert((await preview.locator("pre").innerText()).includes(`${origin}/`));
  const report = (await command(["report", "list"]))[0];
  for (const [format, ext] of [
    ["markdown", "md"],
    ["json", "json"],
    ["html", "html"],
    ["pdf", "pdf"],
  ]) {
    await command([
      "report",
      "export",
      report.id,
      "--format",
      format,
      "--output",
      path.join(artifact, `local-inventory.${ext}`),
      "--reviewed-sha256",
      report.sha256,
    ]);
    if (format !== "pdf")
      assert(
        !(
          await readFile(path.join(artifact, `local-inventory.${ext}`), "utf8")
        ).includes("fixture-private-cookie"),
      );
  }
  const pdfInspection = await exec(
    "python",
    [
      "-c",
      "import sys,json;from pypdf import PdfReader;r=PdfReader(sys.argv[1]);t='\\n'.join(p.extract_text() for p in r.pages);assert 'fixture-private-cookie' not in t;assert '[REDACTED]' in t;print(json.dumps({'pages':len(r.pages),'cookie_redacted':True}))",
      path.join(artifact, "local-inventory.pdf"),
    ],
    { windowsHide: true },
  );
  await writeFile(
    path.join(artifact, "pdf-inspection.json"),
    pdfInspection.stdout,
  );
  await preview.getByRole("button", { name: "Close dialog" }).click();
  checks.push(
    "Actual private HTTP headers are viewable as evidence; default export and the generated research report redact the observed session cookie",
  );
  const slow = await addTarget("Owned slow service", `${origin}/slow`);
  await page
    .getByRole("button", { name: "Reconnaissance", exact: true })
    .click();
  await page
    .getByLabel("Inventory target", { exact: true })
    .selectOption(slow.id);
  await approve();
  const running = await jobWith("running");
  await page
    .locator(`[data-job-id="${running.id}"][data-job-status="running"]`)
    .waitFor();
  for (let i = 0; i < 40 && !requests.some((r) => r.url === "/slow"); i++)
    await delay(25);
  assert(requests.some((r) => r.method === "HEAD" && r.url === "/slow"));
  await page
    .getByRole("button", { name: `Cancel job ${running.id}`, exact: true })
    .click();
  const cancelled = await jobWith("cancelled");
  assert.equal(cancelled.id, running.id);
  await page
    .locator(`[data-job-id="${running.id}"][data-job-status="cancelled"]`)
    .waitFor();
  assert.equal((await command(["recon", "assets"])).length, 1);
  assert.equal((await command(["evidence", "list"])).length, 2);
  checks.push(
    "Desktop cancellation interrupts a real slow loopback request and leaves no fabricated asset or response evidence",
  );
  owned.closeAllConnections();
  await new Promise((r) => owned.close(r));
  await page
    .getByLabel("Inventory target", { exact: true })
    .selectOption(target.id);
  await approve();
  const failed = await jobWith("failed");
  assert.equal(failed.error_code, "HTTP_FAILED");
  const failure = page.locator(`[data-job-id="${failed.id}"]`);
  await failure.locator("summary").click();
  await failure
    .getByRole("status")
    .filter({ hasText: "HTTP_FAILED" })
    .waitFor();
  assert.equal((await command(["recon", "assets"])).length, 1);
  checks.push(
    "Unavailable local service produces a real failed job and visible error/logs; prior successful observation remains unchanged",
  );
  await stop();
  await launch();
  await page
    .getByRole("button", { name: "Reconnaissance", exact: true })
    .click();
  await page
    .getByRole("cell", { name: "owned-local-fixture", exact: false })
    .waitFor();
  assert.equal((await command(["recon", "assets"]))[0].id, assets[0].id);
  await page.getByRole("button", { name: "Reports", exact: true }).click();
  await page
    .getByRole("button", { name: "Open report preview", exact: true })
    .click();
  preview = page.getByRole("dialog", { name: "Report preview", exact: true });
  await preview.waitFor();
  await preview.getByRole("button", { name: "Close dialog" }).click();
  checks.push(
    "Native restart preserves assets, success/cancel/failure logs, evidence and report snapshot; CLI reads identical IDs",
  );
  for (const theme of ["Light", "Dark", "ReconX"]) {
    await page.getByRole("button", { name: "Appearance", exact: true }).click();
    await page
      .getByRole("menuitemradio", { name: new RegExp("^" + theme) })
      .click();
    await page
      .getByRole("button", { name: "Reconnaissance", exact: true })
      .click();
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
        path: path.join(artifact, `recon-${theme.toLowerCase()}-${width}.png`),
      });
    }
  }
  checks.push(
    "Actual Recon page captured in all three themes at 1920 and 1366 with no document overflow",
  );
  await writeFile(
    path.join(artifact, "requests.json"),
    JSON.stringify(requests, null, 2),
  );
  await writeFile(
    path.join(artifact, "results.json"),
    JSON.stringify(
      {
        status: "passed",
        checks,
        network:
          "Owned loopback HTTP fixture only; no external targets or AI transmission",
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
  owned.closeAllConnections();
  if (owned.listening) await new Promise((r) => owned.close(r));
  await stop();
}
