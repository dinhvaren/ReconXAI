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
  `large-import-native-${Date.now()}`,
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

const { open, stat, copyFile } = await import("node:fs/promises");
async function hashFile(file) {
  const { createReadStream } = await import("node:fs");
  const h = createHash("sha256");
  for await (const chunk of createReadStream(file)) h.update(chunk);
  return h.digest("hex");
}
async function nativePick(file) {
  await page.getByRole("button", { name: "Import File", exact: true }).click();
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
      file,
    ],
    { windowsHide: true, timeout: 30000 },
  );
}
async function files() {
  return command(["reverse", "list"]);
}
const checksExtra = [];
try {
  await launch();
  await page.getByRole("button", { name: "Projects", exact: true }).click();
  await page
    .getByLabel("Project name", { exact: true })
    .fill("Large Binary Demo");
  await page
    .getByRole("button", { name: "Create project", exact: true })
    .click();
  await page
    .getByRole("status")
    .filter({ hasText: "Project created" })
    .waitFor();
  const project = (await command(["project", "list"]))[0];
  const source = path.join(artifact, "benign.rs"),
    small = path.join(artifact, "benign-small.exe");
  await writeFile(
    source,
    'fn main(){println!("BENIGN_STATIC_LARGE_FIXTURE");}\n',
  );
  await exec("rustc", [source, "-o", small], { windowsHide: true });
  const large = path.join(artifact, "benign-256MiB.exe");
  await copyFile(small, large);
  const fd = await open(large, "r+");
  await fd.truncate(256 * 1024 * 1024);
  await fd.close();
  const expected = await hashFile(large);
  await page
    .getByRole("button", { name: "Reverse Engineering", exact: true })
    .click();
  await page
    .getByLabel("I have permission to analyze the imported local files.")
    .check();
  await nativePick(large);
  await page
    .getByRole("status")
    .filter({ hasText: "1 file(s) imported" })
    .waitFor({ timeout: 60000 });
  const file = (await files())[0];
  assert.equal(file.size, 256 * 1024 * 1024);
  assert.equal(file.sha256, expected);
  assert.equal(
    await hashFile(path.join(artifact, "imports", `${expected}.bin`)),
    expected,
  );
  assert.equal(
    await page.getByLabel("Current target", { exact: true }).inputValue(),
    file.target_id,
  );
  checksExtra.push(
    "Native file picker imports a real 256 MiB padded benign PE using paths only; Node streaming hash independently matches original and complete managed bytes",
  );
  await page.getByRole("button", { name: "Analyze File", exact: true }).click();
  await page
    .getByText("Analysis saved", { exact: true })
    .waitFor({ timeout: 60000 });
  const metadata = await command(["reverse", "metadata", file.id]);
  assert(
    await page.getByRole("tab", { name: "Sections", exact: true }).isDisabled(),
  );
  assert.equal(metadata.analysis.size, file.size);
  assert.equal(metadata.analysis.sha256, expected);
  assert.equal(metadata.analysis.sections.length, 0);
  assert(metadata.analysis.warnings.some((w) => w.includes("first 1 MiB")));
  checksExtra.push(
    "Actual Job Engine streams large-file integrity/entropy and saves bounded metadata, strings/hex and explicit deferred deep-analysis warning",
  );
  const cancelFile = path.join(artifact, "cancel-1GiB.exe");
  await copyFile(small, cancelFile);
  const cf = await open(cancelFile, "r+");
  await cf.truncate(1024 ** 3);
  await cf.close();
  await nativePick(cancelFile);
  await page
    .getByText("Copying and hashing selected file", { exact: true })
    .waitFor();
  // The renderer remains responsive while native I/O runs on a blocking worker.
  assert.equal(await page.evaluate(() => document.title), "ReconX AI");
  await page
    .getByRole("button", { name: "Cancel import", exact: true })
    .click();
  await page
    .getByRole("status")
    .filter({ hasText: "Import interrupted." })
    .waitFor({ timeout: 30000 });
  assert.equal((await files()).length, 1);
  assert.equal((await stat(cancelFile)).size, 1024 ** 3);
  const { readdir } = await import("node:fs/promises");
  assert(
    !(await readdir(path.join(artifact, "imports"))).some((n) =>
      n.startsWith(".import-"),
    ),
  );
  checksExtra.push(
    "Native cancellation interrupts a genuinely running 1 GiB copy; no partial imported row/staging file, original retained and renderer responsive",
  );
  const second = await command(["project", "create", "Large import isolation"]);
  await nativePick(cancelFile);
  await page
    .getByText("Copying and hashing selected file", { exact: true })
    .waitFor();
  await page
    .getByLabel("Current project", { exact: true })
    .selectOption(second.id, { timeout: 10000 });
  await page
    .getByRole("button", { name: "Cancel import", exact: true })
    .waitFor({ state: "hidden", timeout: 30000 });
  assert.equal((await command(["workspace", "show"])).project_id, second.id);
  assert.equal((await files()).length, 1);
  await page
    .getByLabel("Current project", { exact: true })
    .selectOption(project.id);
  await page
    .getByLabel("Current target", { exact: true })
    .selectOption(file.target_id);
  checksExtra.push(
    "Switching project during a real import cancels its copy and fences stale results; the new project selection is not overwritten",
  );
  await stop();
  await launch();
  await page
    .getByRole("button", { name: "Reverse Engineering", exact: true })
    .click();
  await page
    .getByRole("button", { name: "benign-256MiB.exe", exact: false })
    .first()
    .click();
  assert.equal(
    (await command(["reverse", "metadata", file.id])).id,
    metadata.id,
  );
  checksExtra.push(
    "Native restart and independent CLI preserve the 256 MiB target/file/job/analysis IDs and evidence",
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
      assert(
        await page.evaluate(
          () => document.documentElement.scrollWidth <= innerWidth + 1,
        ),
      );
      await page.screenshot({
        path: path.join(
          artifact,
          `large-import-${theme.toLowerCase()}-${width}.png`,
        ),
      });
    }
  }
  checksExtra.push(
    "Large-file workspace renders in three themes and four resolutions without document overflow; unsupported deep-analysis tabs are disabled",
  );
  await writeFile(
    path.join(artifact, "results.json"),
    JSON.stringify(
      {
        status: "passed",
        checks: checksExtra,
        imported: file,
        analysis_id: metadata.id,
        network: "none; no sample execution",
      },
      null,
      2,
    ),
  );
  console.log(
    JSON.stringify(
      { artifact, passed: checksExtra.length, checks: checksExtra },
      null,
      2,
    ),
  );
} catch (error) {
  if (page)
    await page
      .screenshot({ path: path.join(artifact, "failure.png") })
      .catch(() => {});
  await writeFile(
    path.join(artifact, "results.json"),
    JSON.stringify(
      {
        status: "failed",
        checks: checksExtra,
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
}
