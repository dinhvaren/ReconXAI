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
const artifact = path.join(root, "artifacts", `jobs-${Date.now()}`);
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
            content:
              "Local fixture interpretation. Static evidence only; no vulnerability is asserted.",
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
async function jobsPage() {
  await page
    .getByRole("button", { name: "Quick actions", exact: true })
    .click();
  await page.getByRole("menuitem", { name: "View Jobs", exact: true }).click();
  await page
    .getByRole("heading", { name: "Analysis Jobs", exact: true })
    .waitFor();
}
async function waitJob(id, status, timeout = 30000) {
  const until = Date.now() + timeout;
  while (Date.now() < until) {
    const job = await command(["jobs", "show", id]);
    if ((Array.isArray(status) ? status : [status]).includes(job.status))
      return job;
    await delay(60);
  }
  throw Error(`Job did not enter ${status}: ${id}`);
}
async function queueUi(file, kind = "local_static_binary", timeout = 60) {
  await page.getByLabel("Job type", { exact: true }).selectOption(kind);
  await page.getByLabel("Job file", { exact: true }).selectOption(file);
  await page
    .getByLabel("Timeout (seconds)", { exact: true })
    .fill(String(timeout));
  const before = new Set((await command(["jobs", "list"])).map((j) => j.id));
  await page.getByRole("button", { name: "Queue job", exact: true }).click();
  for (let i = 0; i < 80; i++) {
    const found = (await command(["jobs", "list"])).find(
      (j) => !before.has(j.id),
    );
    if (found) return found;
    await delay(40);
  }
  throw Error("Queue action did not persist a job");
}
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
  const project = (await command(["project", "list"]))[0];
  await page
    .getByLabel("Current project", { exact: true })
    .selectOption(project.id);
  await page.getByRole("button", { name: "+ Add Target", exact: true }).click();
  const dialog = page.getByRole("dialog", { name: "Add Target", exact: true });
  await dialog.waitFor();
  await dialog.getByLabel("Target name", { exact: true }).fill("Owned domain");
  await dialog.getByLabel("Target value", { exact: true }).fill("example.test");
  await dialog.getByLabel("I confirm I have permission").check();
  await dialog
    .getByRole("button", { name: "Save target", exact: true })
    .click();
  await dialog.waitFor({ state: "hidden" });
  const target = (
    await command(["target", "list", "--project", project.id])
  )[0];
  assert.equal(target.value, "example.test");
  assert.equal((await command(["scope", "show", target.id])).authorized, true);
  await page
    .getByRole("button", { name: "Targets & Scope", exact: true })
    .click();
  await page.getByRole("button", { name: "Owned domain", exact: true }).click();
  await page
    .getByRole("button", { name: "Review target scope", exact: true })
    .click();
  for (let i = 0; i < 80; i++) {
    const jobs = await command(["jobs", "list"]);
    if (
      jobs.some(
        (j) =>
          j.kind === "target_registration_review" && j.status === "completed",
      )
    )
      break;
    await delay(80);
  }
  assert(
    (await command(["jobs", "list"])).some(
      (j) =>
        j.kind === "target_registration_review" &&
        j.result.status === "allowed",
    ),
  );
  await stop();
  await launch();
  await page
    .getByRole("button", { name: "Targets & Scope", exact: true })
    .click();
  await page
    .getByRole("button", { name: "Owned domain", exact: true })
    .waitFor();
  await page.screenshot({
    path: path.join(artifact, "target-after-restart.png"),
  });
  checks.push(
    "Research Demo and Desktop example.test target survive native restart and share scope/records with CLI",
  );

  await page
    .getByRole("button", { name: "Reverse Engineering", exact: true })
    .click();
  await page
    .getByLabel("I have permission to analyze the imported local files.", {
      exact: true,
    })
    .check();
  await page.getByRole("button", { name: "Import File", exact: true }).click();
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
      cli,
    ],
    { windowsHide: true, timeout: 30000 },
  );
  await page
    .getByRole("status")
    .filter({ hasText: "1 file(s) imported" })
    .waitFor({ timeout: 30000 });
  const file = (await command(["reverse", "list", "--project", project.id]))[0];
  assert.equal(file.format, "Pe");
  assert.equal(
    file.sha256,
    createHash("sha256")
      .update(await readFile(cli))
      .digest("hex"),
  );
  assert(file.source_path);
  await page.getByRole("button", { name: "Analyze File", exact: true }).click();
  await page
    .getByText("Analysis saved", { exact: true })
    .waitFor({ timeout: 30000 });
  const record = await command(["reverse", "metadata", file.id]);
  assert(record.analysis.sections.length);
  assert(record.analysis.strings.length);
  assert.equal(record.analysis.sha256, file.sha256);
  await page.getByRole("tab", { name: "Sections", exact: true }).click();
  await page.getByRole("cell", { name: ".text", exact: true }).waitFor();
  await page.getByRole("tab", { name: "Strings", exact: true }).click();
  await page.getByLabel("Search strings", { exact: true }).fill("reconx");
  await page.getByText(/Showing \d+ of \d+ matches/).waitFor();
  await page.getByRole("tab", { name: "Metadata", exact: true }).click();
  await page
    .getByLabel("Local file notes", { exact: true })
    .fill(
      "Owned local research notes — retained, never included in AI preview.",
    );
  await page.getByRole("button", { name: "Save notes", exact: true }).click();
  await page
    .getByRole("status")
    .filter({ hasText: "Research notes saved" })
    .waitFor();
  await page.setViewportSize({ width: 1920, height: 1080 });
  await page.locator("main h1").scrollIntoViewIfNeeded();
  await page.screenshot({ path: path.join(artifact, "reverse-real-pe.png") });
  checks.push(
    "Native picker imports a real owned PE; independent hash, sections, strings, source path, persisted analysis and local notes are real",
  );

  const provider = {
    id: "797fd9d7-479b-48f9-8c0b-891fa99def63",
    name: "Owned local review fixture",
    kind: "compatible",
    base_url: `http://127.0.0.1:${mock.address().port}/v1`,
    model: "fixture",
    enabled: true,
    timeout_secs: 10,
    max_output_tokens: 256,
    requires_key: false,
  };
  const providerFile = path.join(artifact, "provider.json");
  await writeFile(providerFile, JSON.stringify(provider));
  await command(["ai", "configure", providerFile]);
  const preview = await service("file_artifact_preview", {
    id: file.id,
    section: "summary",
  });
  assert(!preview.includes("retained, never included"));
  assert(!preview.includes(file.source_path));
  const review = {
    id: file.id,
    section: "summary",
    provider: provider.id,
    preview,
    gateway: provider.base_url,
    consent: false,
  };
  const denied = await page.evaluate(async (review) => {
    try {
      await window.__TAURI_INTERNALS__.invoke("service", {
        request: { op: "file_explain", review },
      });
      return null;
    } catch (e) {
      return e;
    }
  }, review);
  assert.equal(denied.code, "CONSENT_REQUIRED");
  assert.equal(calls.length, 0);
  await stop();
  await launch();
  await page
    .getByRole("button", { name: "Reverse Engineering", exact: true })
    .click();
  await page.getByText("Analysis saved", { exact: true }).waitFor();
  await page.getByLabel("Local file notes", { exact: true }).waitFor();
  assert.match(
    await page.getByLabel("Local file notes", { exact: true }).inputValue(),
    /Owned local research notes/,
  );
  await page
    .getByLabel("Review provider", { exact: true })
    .selectOption(provider.id);
  await page.getByLabel("I approve sending this preview").check();
  await page
    .getByRole("button", { name: "Explain selected artifacts", exact: true })
    .click();
  await page
    .getByText(
      "Local fixture interpretation. Static evidence only; no vulnerability is asserted.",
      { exact: true },
    )
    .waitFor();
  assert.equal(calls.length, 1);
  assert(!JSON.stringify(calls).includes("retained, never included"));
  assert(!JSON.stringify(calls).includes(file.source_path));
  checks.push(
    "Notes and PE results reopen after restart; backend denies unapproved transmission; explicit native UI consent sends only reviewed artifacts to the local gateway",
  );

  await jobsPage();
  let job = await queueUi(file.id, "binary_metadata");
  assert.equal((await waitJob(job.id, "completed")).progress, 100);
  assert.equal(
    (await command(["jobs", "result", job.id])).analysis.sha256,
    file.sha256,
  );
  await page
    .locator(`[data-job-id="${job.id}"]`)
    .getByRole("button", { name: "View result", exact: true })
    .click();
  await page.getByRole("dialog", { name: "Job result", exact: true }).waitFor();
  await page
    .getByRole("dialog", { name: "Job result" })
    .getByRole("button", { name: "Close dialog" })
    .click();
  job = await queueUi(file.id, "strings_extraction");
  await waitJob(job.id, "completed");
  assert((await command(["jobs", "result", job.id])).analysis.strings.length);
  job = await queueUi(file.id, "artifact_index");
  await waitJob(job.id, "completed");
  assert.equal((await command(["jobs", "result", job.id])).file_id, file.id);
  checks.push(
    "Job Center queues metadata, strings and artifact index jobs, opens exact persisted results, and CLI reads the same job results",
  );

  const controlled = path.join(artifact, "owned-large.raw");
  await writeFile(controlled, Buffer.alloc(64 * 1024 * 1024, 0));
  const large = await command([
    "reverse",
    "import",
    controlled,
    "--project",
    project.id,
    "--authorized",
  ]);
  await page.getByRole("button", { name: "Refresh", exact: true }).click();
  await page
    .getByLabel("Job file", { exact: true })
    .locator(`option[value="${large.id}"]`)
    .waitFor({ state: "attached" });
  job = await queueUi(large.id, "local_static_binary");
  await waitJob(job.id, "running");
  await page
    .locator(`[data-job-id="${job.id}"][data-job-status="running"]`)
    .waitFor();
  await page.locator(`[data-job-id="${job.id}"]`).scrollIntoViewIfNeeded();
  await page.screenshot({ path: path.join(artifact, "job-running.png") });
  await page
    .getByRole("button", { name: `Cancel job ${job.id}`, exact: true })
    .click();
  const cancelled = await waitJob(job.id, "cancelled");
  assert.equal(cancelled.result, null);
  assert.equal(cancelled.error_code, "CANCELLED");
  checks.push(
    "Real 64 MiB controlled-file work enters Running; native Cancel is acknowledged by the worker and prevents result commit",
  );
  job = await queueUi(large.id, "local_static_binary", 0.001);
  const failed = await waitJob(job.id, "failed");
  assert.equal(failed.error_code, "JOB_TIMEOUT");
  assert.equal(failed.result, null);
  await page
    .locator(`[data-job-id="${job.id}"]`)
    .getByText("Execution log", { exact: true })
    .click();
  await page.getByText(/JOB_TIMEOUT:/).waitFor();
  await page.screenshot({
    path: path.join(artifact, "job-failed-and-cancelled.png"),
  });
  checks.push(
    "A genuine configured timeout produces Failed with persisted error/logs and no result",
  );

  job = await queueUi(large.id, "local_static_binary", 60);
  await waitJob(job.id, "running");
  const interruptedId = job.id;
  await stop();
  await launch();
  await jobsPage();
  const recovered = await waitJob(interruptedId, "failed", 30000);
  assert.equal(recovered.error_code, "JOB_INTERRUPTED");
  assert.equal(recovered.result, null);
  await page
    .locator(`[data-job-id="${interruptedId}"]`)
    .getByText("Execution log", { exact: true })
    .click();
  await page.getByText(/JOB_INTERRUPTED:/).waitFor();
  await page.screenshot({
    path: path.join(artifact, "job-interruption-recovered.png"),
  });
  checks.push(
    "Actual native-process termination during work is reconciled after lease expiry on restart; interrupted work is not replayed",
  );
  const validEvents = observed.filter((j) => j.file_id === large.id);
  assert(validEvents.some((j) => j.status === "running" && j.progress > 1));
  assert(validEvents.some((j) => j.status === "cancelled"));
  assert(validEvents.some((j) => j.status === "failed"));
  for (const theme of ["Light", "Dark", "ReconX"]) {
    await page
      .getByRole("button", { name: "Topbar appearance", exact: true })
      .click();
    await page
      .getByRole("menuitemradio", { name: new RegExp("^" + theme) })
      .click();
    for (const [width, height] of [
      [1366, 768],
      [1920, 1080],
    ]) {
      await page.setViewportSize({ width, height });
      const controls = await page.locator("form.job-controls").boundingBox();
      assert(
        controls && controls.height < 260,
        "Queue controls stay compact and usable",
      );
      assert(
        await page.evaluate(
          () => document.documentElement.scrollWidth <= innerWidth + 1,
        ),
      );
      await page.screenshot({
        path: path.join(artifact, `jobs-${theme.toLowerCase()}-${width}.png`),
      });
    }
  }
  checks.push(
    "Real job events and persisted stage progress are observed; Job Center remains usable in Light/Dark/ReconX at 1366 and 1920 without document clipping",
  );
  await writeFile(
    path.join(artifact, "events.json"),
    JSON.stringify(observed, null, 2),
  );
  await writeFile(
    path.join(artifact, "results.json"),
    JSON.stringify(
      {
        status: "passed",
        desktop,
        cli,
        checks,
        event_count: observed.length,
        local_ai_calls: calls.length,
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
    await writeFile(
      path.join(artifact, "dom-state.json"),
      JSON.stringify(
        await page.evaluate(() =>
          [...document.querySelectorAll("label")].map((l) => ({
            text: l.textContent,
            control: l.control?.outerHTML.slice(0, 400),
          })),
        ),
        null,
        2,
      ),
    ).catch(() => {});
  if (page)
    await page
      .screenshot({ path: path.join(artifact, "failure.png") })
      .catch(() => {});
  await writeFile(
    path.join(artifact, "events.json"),
    JSON.stringify(observed, null, 2),
  );
  await writeFile(
    path.join(artifact, "results.json"),
    JSON.stringify({ status: "failed", checks, error: String(error) }, null, 2),
  );
  throw error;
} finally {
  await stop();
  await new Promise((r) => mock.close(r));
}
