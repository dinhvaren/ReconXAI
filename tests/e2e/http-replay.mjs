import { chromium } from "../../apps/desktop/node_modules/playwright/index.mjs";
import { createServer as tcpServer } from "node:net";
import { createServer } from "node:http";
import { spawn, execFile, spawnSync } from "node:child_process";
import { promisify } from "node:util";
import { mkdir, readFile, writeFile } from "node:fs/promises";
import { createWriteStream } from "node:fs";
import { fileURLToPath } from "node:url";
import path from "node:path";
import assert from "node:assert/strict";
const root = path.resolve(
  path.dirname(fileURLToPath(import.meta.url)),
  "../..",
);
const artifact = path.join(root, "artifacts", `http-replay-${Date.now()}`);
await mkdir(artifact, { recursive: true });
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
const exec = promisify(execFile),
  delay = (ms) => new Promise((r) => setTimeout(r, ms));
const aliceKey = "synthetic-native-alice-http-credential",
  bobKey = "synthetic-native-bob-http-credential";
const calls = [],
  checks = [];
let child, browser, page, projectId;
const fixture = createServer(async (req, res) => {
  let body = "";
  for await (const part of req) body += part;
  const raw = req.headers.authorization ?? "";
  const owner =
    raw === `Bearer ${aliceKey}`
      ? "alice"
      : raw === `Bearer ${bobKey}`
        ? "bob"
        : "anonymous";
  calls.push({
    receivedAt: Date.now(),
    method: req.method,
    url: req.url,
    owner,
    body,
    credentialMatched: owner !== "anonymous",
  });
  if (req.url === "/redirect") {
    res.writeHead(302, {
      Location: "http://192.0.2.1/never-follow",
      "Content-Type": "application/json",
    });
    res.end("{}");
    return;
  }
  if (req.url === "/large") {
    res.writeHead(200, {
      "Content-Type": "text/plain",
      "Content-Length": 300000,
    });
    res.end("x".repeat(300000));
    return;
  }
  const timer = setTimeout(
    () => {
      if (!res.destroyed) {
        res.writeHead(req.method === "POST" ? 201 : 200, {
          "Content-Type": "application/json",
          "Set-Cookie": "session=synthetic-native-response-cookie",
        });
        res.end(JSON.stringify({ owner, id: 1, echo: raw, body }));
      }
    },
    req.url === "/slow" ? 5000 : 200,
  );
  timer.unref();
  res.on("close", () => clearTimeout(timer));
});
await new Promise((r) => fixture.listen(0, "127.0.0.1", r));
const url = `http://127.0.0.1:${fixture.address().port}/`;
async function command(args) {
  const output = await exec(cli, ["--json", ...args], {
    env,
    windowsHide: true,
    maxBuffer: 16 * 1024 * 1024,
  });
  return output.stdout.trim() ? JSON.parse(output.stdout) : null;
}
async function launch() {
  const listener = tcpServer();
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
  for (let i = 0; i < 160; i++) {
    if (child.exitCode !== null) throw Error("Desktop exited");
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
async function openWorkspace(project, target) {
  await page
    .getByLabel("Current project", { exact: true })
    .selectOption(project);
  await page.getByRole("button", { name: "Web Security", exact: true }).click();
  await page.getByLabel("HTTP target", { exact: true }).selectOption(target);
}
async function createIdentity(name, key) {
  const dialog = page.getByRole("dialog", {
    name: "HTTP authentication profiles",
    exact: true,
  });
  await dialog.getByLabel("Identity name", { exact: true }).fill(name);
  await dialog.getByLabel("Identity role", { exact: true }).fill("user");
  await dialog
    .getByLabel("Identity authentication type", { exact: true })
    .selectOption("bearer");
  await dialog.getByLabel("Identity credential", { exact: true }).fill(key);
  assert.equal(
    await dialog
      .getByLabel("Identity credential", { exact: true })
      .getAttribute("type"),
    "password",
  );
  await dialog
    .getByRole("button", { name: "Save identity", exact: true })
    .click();
  await dialog.getByText(`${name} · user`, { exact: true }).waitFor();
  assert.equal(
    await dialog.getByLabel("Identity credential", { exact: true }).count(),
    0,
  );
}
async function respectDispatchSpacing() {
  // Wait for the real backend rate budget, never invent execution progress.
  // Fast native runs can otherwise hit the 500 ms project dispatch guard.
  const last = calls.at(-1)?.receivedAt;
  if (last) await delay(Math.max(0, 650 - (Date.now() - last)));
}
async function sendFromEditor(dialog) {
  await respectDispatchSpacing();
  const previous = (await latest(projectId))?.id;
  await dialog
    .getByRole("button", { name: "Review send", exact: true })
    .click();
  const review = dialog.getByRole("region", {
    name: "HTTP execution review",
    exact: true,
  });
  await review.waitFor();
  assert(await review.innerText().then((t) => t.includes("Pinned socket:")));
  assert.equal(
    await review
      .getByRole("button", { name: "Send approved request", exact: true })
      .isEnabled(),
    false,
  );
  await review
    .getByLabel("Approve exact HTTP request", { exact: true })
    .check();
  if (!previous) {
    await dialog.screenshot({
      path: path.join(artifact, "reviewed-request-editor.png"),
    });
  }
  await review
    .getByRole("button", { name: "Send approved request", exact: true })
    .click();
  let completed;
  for (let i = 0; i < 120; i++) {
    const job = await latest(projectId);
    if (
      job &&
      job.id !== previous &&
      ["completed", "failed", "cancelled"].includes(job.status)
    ) {
      assert.equal(
        job.status,
        "completed",
        JSON.stringify({ code: job.error_code, message: job.error_message }),
      );
      completed = job;
      break;
    }
    await delay(250);
  }
  assert(completed, "The newly approved request did not complete");
  await dialog
    .getByRole("button", { name: "Open captured response", exact: true })
    .first()
    .waitFor({ timeout: 30000 });
}
async function latest(project) {
  return (await command(["jobs", "list", "--project", project])).find(
    (j) => j.kind === "manual_http_replay",
  );
}
try {
  const project = await command([
    "project",
    "create",
    "Manual HTTP Research Demo",
  ]);
  projectId = project.id;
  const target = await command([
    "target",
    "add",
    "--project",
    project.id,
    "--type",
    "rest_api",
    "--value",
    url,
    "--name",
    "Owned disposable API",
    "--authorized",
    "--mode",
    "lab_research",
  ]);
  const settings = await command(["settings", "show"]);
  settings.security_mode = "lab_research";
  const settingsFile = path.join(artifact, "settings.json");
  await writeFile(settingsFile, JSON.stringify(settings));
  await command(["settings", "apply", settingsFile]);
  const har = JSON.parse(
    await readFile(
      path.join(root, "crates/reconx-core/tests/fixtures/http/alice.har"),
      "utf8",
    ),
  );
  har.log.entries = har.log.entries.slice(0, 1);
  har.log.entries[0].request.url = url + "json";
  const input = path.join(artifact, "owned.har");
  await writeFile(input, JSON.stringify(har));
  await launch();
  await openWorkspace(project.id, target.id);
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
      input,
    ],
    { windowsHide: true, timeout: 30000 },
  );
  const importer = page.getByRole("dialog", {
    name: "Review offline HTTP import",
    exact: true,
  });
  await importer.waitFor();
  await importer
    .getByRole("button", { name: "Import selected files", exact: true })
    .click();
  await page.locator(".http-explorer button.http-item").first().click();
  const source = (await command(["web", "traffic", "--project", project.id]))
    .traffic[0];
  assert.equal(calls.length, 0);
  await page
    .getByRole("button", {
      name: "Edit request / replay to loopback",
      exact: true,
    })
    .click();
  let editor = page.getByRole("dialog", {
    name: "Manual loopback HTTP request",
    exact: true,
  });
  await editor.getByLabel("HTTP request URL", { exact: true }).waitFor();
  assert.equal(calls.length, 0);
  await editor
    .getByRole("button", { name: "Close dialog", exact: true })
    .click();
  checks.push(
    "Actual native HAR import and editor seed send no request; captured credentials are not replayed",
  );
  await page
    .getByRole("button", { name: "Authentication profiles", exact: true })
    .click();
  await createIdentity("Alice", aliceKey);
  await createIdentity("Bob", bobKey);
  const auth = page.getByRole("dialog", {
    name: "HTTP authentication profiles",
    exact: true,
  });
  assert.equal(
    await auth
      .getByText("No authentication check has run.", { exact: true })
      .count(),
    2,
  );
  await auth.screenshot({ path: path.join(artifact, "auth-profiles.png") });
  await auth.getByRole("button", { name: "Close dialog", exact: true }).click();
  assert.equal(calls.length, 0);
  const profiles = await command(["auth", "profiles", "--project", project.id]);
  const alice = profiles.find((p) => p.name === "Alice"),
    bob = profiles.find((p) => p.name === "Bob");
  assert(alice && bob);
  assert(!JSON.stringify(profiles).includes(aliceKey));
  checks.push(
    "Native Alice/Bob setup uses actual OS credentials; CLI shares masked metadata and saving never marks authenticated",
  );
  await page
    .getByRole("button", {
      name: "Edit request / replay to loopback",
      exact: true,
    })
    .click();
  editor = page.getByRole("dialog", {
    name: "Manual loopback HTTP request",
    exact: true,
  });
  await editor
    .getByLabel("HTTP request title", { exact: true })
    .fill("Reviewed Alice request");
  await editor
    .getByLabel("HTTP request method", { exact: true })
    .selectOption("POST");
  await editor
    .getByLabel("HTTP test identity", { exact: true })
    .selectOption(alice.id);
  await editor
    .getByLabel("HTTP request content type", { exact: true })
    .fill("application/json");
  await editor
    .getByLabel("HTTP request body", { exact: true })
    .fill('{"note":"disposable local fixture"}');
  await sendFromEditor(editor);
  let job = await latest(project.id);
  assert.equal(job.status, "completed");
  assert.equal(job.result.observations.status, 201);
  assert(calls[0].credentialMatched && calls[0].owner === "alice");
  assert.equal(calls[0].body, '{"note":"disposable local fixture"}');
  assert(!JSON.stringify(job).includes(aliceKey));
  const aliceCapture = job.result.exchange_id;
  const draftId = job.result.draft_id;
  await editor
    .getByRole("button", { name: "Open captured response", exact: true })
    .first()
    .click();
  await page.getByRole("tab", { name: "Body", exact: true }).click();
  await page.getByText("[REDACTED]", { exact: false }).first().waitFor();
  await page.screenshot({
    path: path.join(artifact, "actual-response-redacted.png"),
    fullPage: true,
  });
  checks.push(
    "Reviewed native POST sends the stored Alice credential to the pinned fixture; actual status/body/timing and redacted response become persistent traffic/evidence",
  );
  await page
    .getByRole("button", { name: "Saved requests", exact: true })
    .click();
  editor = page.getByRole("dialog", {
    name: "Manual loopback HTTP request",
    exact: true,
  });
  await editor
    .getByLabel("Saved HTTP request", { exact: true })
    .selectOption(draftId);
  await editor
    .getByLabel("HTTP request method", { exact: true })
    .selectOption("GET");
  await editor.getByLabel("HTTP request body", { exact: true }).fill("");
  await editor
    .getByLabel("HTTP test identity", { exact: true })
    .selectOption(bob.id);
  const before = calls.length;
  await sendFromEditor(editor);
  for (let i = 0; i < 80; i++) {
    job = await latest(project.id);
    if (job.status === "completed" && job.result.exchange_id !== aliceCapture)
      break;
    await delay(100);
  }
  assert.equal(job.status, "completed");
  assert.equal(calls.length, before + 1);
  assert(calls.at(-1).owner === "bob");
  const bobCapture = job.result.exchange_id;
  await editor
    .getByRole("button", { name: "Close dialog", exact: true })
    .click();
  const compared = await command(["http", "compare", aliceCapture, bobCapture]);
  assert.equal(compared.status, "needs_manual_review");
  assert.equal(compared.network_requests, 0);
  checks.push(
    "Switching identities changes a real subsequent request; CLI compares actual captures and never verifies authorization from status differences",
  );
  const evidence = await command(["evidence", "from-job", job.id]);
  await command(["evidence", "verify", evidence.id]);
  const finding = await command(["api", "finding", compared.id]);
  assert.equal(finding.input.status, "needs_verification");
  const reportInput = path.join(artifact, "report-input.json");
  await writeFile(
    reportInput,
    JSON.stringify({
      project_id: project.id,
      title: "Owned manual HTTP observations",
      template: "technical",
      summary:
        "Actual local fixture responses; authorization remains unverified",
      methodology: "Each request individually reviewed and approved",
      finding_ids: [finding.id],
      evidence_ids: [evidence.id, compared.evidence_id],
    }),
  );
  const report = await command(["report", "prepare", "--input", reportInput]);
  for (const format of ["json", "markdown", "html", "pdf"]) {
    const ext = format === "markdown" ? "md" : format;
    await command([
      "report",
      "export",
      report.report.id,
      "--format",
      format,
      "--output",
      path.join(artifact, `manual-http-report.${ext}`),
      "--reviewed-sha256",
      report.report.sha256,
    ]);
  }
  const output = path.join(artifact, "redacted-traffic.json");
  await command(["web", "export", "--project", project.id, output]);
  for (const file of [
    output,
    ...["json", "md", "html"].map((ext) =>
      path.join(artifact, `manual-http-report.${ext}`),
    ),
  ]) {
    const text = await readFile(file, "utf8");
    for (const secret of [
      aliceKey,
      bobKey,
      "synthetic-auth-token",
      "synthetic-response-cookie",
    ])
      assert(!text.includes(secret));
  }
  checks.push(
    "Actual replay evidence, Needs Verification review finding and reviewed JSON/Markdown/HTML/PDF exports share the existing pipeline; tested credentials are redacted",
  );
  await stop();
  await launch();
  await openWorkspace(project.id, target.id);
  assert.equal((await command(["http", "history", draftId])).length, 2);
  assert.equal(
    (await command(["auth", "profiles", "--project", project.id])).length,
    2,
  );
  await page
    .getByRole("button", { name: "Saved requests", exact: true })
    .click();
  editor = page.getByRole("dialog", {
    name: "Manual loopback HTTP request",
    exact: true,
  });
  await editor
    .getByLabel("Saved HTTP request", { exact: true })
    .selectOption(draftId);
  await editor
    .getByLabel("HTTP test identity", { exact: true })
    .selectOption(alice.id);
  const restartHits = calls.length;
  await sendFromEditor(editor);
  for (let i = 0; i < 80 && calls.length === restartHits; i++) await delay(100);
  assert(calls.at(-1).owner === "alice");
  await editor
    .getByRole("button", { name: "Close dialog", exact: true })
    .click();
  checks.push(
    "Complete Desktop restart restores draft/history and securely reuses Alice's OS-stored credential without reentry",
  );
  const publicDraft = await command(["http", "inspect", draftId]);
  const { id, revision, updated_at, ...draftInput } = publicDraft;
  draftInput.url = url + "redirect";
  draftInput.method = "GET";
  draftInput.body = "";
  draftInput.auth_profile_id = null;
  const draftFile = path.join(artifact, "request-input.json");
  await writeFile(draftFile, JSON.stringify(draftInput));
  await command([
    "http",
    "save",
    draftFile,
    "--id",
    id,
    "--revision",
    String(revision),
  ]);
  const plan = await command(["http", "preview", id]);
  const redirectBefore = calls.length;
  await respectDispatchSpacing();
  const queued = await command([
    "http",
    "replay",
    id,
    "--fingerprint",
    plan.fingerprint,
    "--approve",
  ]);
  for (let i = 0; i < 100; i++) {
    job = (await command(["jobs", "list", "--project", project.id])).find(
      (j) => j.id === queued.id,
    );
    if (!["queued", "running"].includes(job.status)) break;
    await delay(100);
  }
  assert.equal(job.status, "completed");
  assert.equal(job.result.observations.status, 302);
  assert.equal(calls.length, redirectBefore + 1);
  checks.push(
    "CLI save/preview/approve executes through the same Desktop worker and captures a redirect without following its external destination",
  );
  await page
    .getByRole("button", { name: "Saved requests", exact: true })
    .click();
  editor = page.getByRole("dialog", {
    name: "Manual loopback HTTP request",
    exact: true,
  });
  await editor
    .getByLabel("Saved HTTP request", { exact: true })
    .selectOption(id);
  await editor
    .getByLabel("HTTP request URL", { exact: true })
    .fill(url + "slow");
  await editor.getByLabel("HTTP request timeout", { exact: true }).fill("5000");
  await respectDispatchSpacing();
  await editor
    .getByRole("button", { name: "Review send", exact: true })
    .click();
  await editor
    .getByLabel("Approve exact HTTP request", { exact: true })
    .check();
  await editor
    .getByRole("button", { name: "Send approved request", exact: true })
    .click();
  await editor
    .getByRole("button", { name: "Cancel request", exact: true })
    .waitFor();
  for (let i = 0; i < 100 && !calls.some((c) => c.url === "/slow"); i++)
    await delay(50);
  assert(
    calls.some((c) => c.url === "/slow"),
    "The controlled service must receive the request before cancellation",
  );
  await editor
    .getByRole("button", { name: "Cancel request", exact: true })
    .click();
  await editor
    .getByText("cancelled", { exact: true })
    .waitFor({ timeout: 15000 });
  await editor
    .getByRole("button", { name: "Close dialog", exact: true })
    .click();
  checks.push(
    "Native cancellation stops a genuinely pending HTTP job and persists Cancelled without fabricating a response",
  );
  for (const [theme, width, height] of [
    ["Light", 1920, 1080],
    ["Dark", 1440, 900],
    ["ReconX", 1366, 768],
  ]) {
    await page.setViewportSize({ width, height });
    await page.getByRole("button", { name: "Appearance", exact: true }).click();
    await page
      .getByRole("menuitemradio", { name: new RegExp("^" + theme) })
      .click();
    await page
      .getByRole("button", { name: "Authentication profiles", exact: true })
      .click();
    await page
      .getByRole("dialog", {
        name: "HTTP authentication profiles",
        exact: true,
      })
      .screenshot({
        path: path.join(artifact, `auth-${theme.toLowerCase()}-${width}.png`),
      });
    assert(
      await page.evaluate(
        () => document.documentElement.scrollWidth <= innerWidth + 1,
      ),
    );
    await page
      .getByRole("dialog", {
        name: "HTTP authentication profiles",
        exact: true,
      })
      .getByRole("button", { name: "Close dialog", exact: true })
      .click();
  }
  checks.push(
    "Actual authentication manager renders in Light/Dark/ReconX at 1920/1440/1366 without exposing stored credentials or document overflow",
  );
  const historyCount = (await command(["http", "history", draftId])).length;
  assert(historyCount >= 5);
  await command(["target", "remove", target.id]);
  assert.equal(
    (await command(["http", "history", draftId])).length,
    historyCount,
  );
  const retainedIdentities = await command([
    "auth",
    "profiles",
    "--project",
    project.id,
  ]);
  assert.equal(retainedIdentities.length, 2);
  assert(
    retainedIdentities.every(
      (p) => p.expired && p.credential_state === "target_removed",
    ),
  );
  await page
    .getByRole("button", { name: "Authentication profiles", exact: true })
    .click();
  const retainedDialog = page.getByRole("dialog", {
    name: "HTTP authentication profiles",
    exact: true,
  });
  await retainedDialog
    .getByText(/^Target removed\./)
    .first()
    .waitFor();
  for (const button of await retainedDialog
    .getByRole("button", { name: "Test with reviewed request", exact: true })
    .all())
    assert(await button.isDisabled());
  for (const button of await retainedDialog
    .getByRole("button", { name: "Edit identity", exact: true })
    .all())
    assert(await button.isDisabled());
  await retainedDialog
    .getByRole("button", { name: "Close dialog", exact: true })
    .click();
  checks.push(
    "Deleting an HTTP target preserves captures and job history, disables its retained identities, and does not break the existing target manager",
  );
  for (const p of await command(["auth", "profiles", "--project", project.id]))
    await command(["auth", "remove", p.id, "--revision", String(p.revision)]);
  assert.equal(
    (await command(["auth", "profiles", "--project", project.id])).length,
    0,
  );
  checks.push(
    "CLI removes the native-created identities and their OS credentials; shared project records remain intact",
  );
  await writeFile(
    path.join(artifact, "results.json"),
    JSON.stringify(
      {
        status: "passed",
        checks,
        desktop,
        cli,
        scope:
          "Manual reviewed loopback fixture only; no autonomous authorization testing or external requests",
        requests: calls.map(({ method, url, owner, credentialMatched }) => ({
          method,
          url,
          owner,
          credentialMatched,
        })),
      },
      null,
      2,
    ),
  );
  console.log(
    JSON.stringify({ artifact, passed: checks.length, checks }, null, 2),
  );
} catch (error) {
  const alerts = page
    ? await page
        .getByRole("alert")
        .allTextContents()
        .catch(() => [])
    : [];
  if (page)
    await page
      .screenshot({ path: path.join(artifact, "failure.png"), fullPage: true })
      .catch(() => {});
  await writeFile(
    path.join(artifact, "results.json"),
    JSON.stringify(
      { status: "failed", checks, error: String(error), alerts },
      null,
      2,
    ),
  );
  throw error;
} finally {
  await stop();
  fixture.closeAllConnections();
  await new Promise((r) => fixture.close(r));
}
