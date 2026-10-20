import { chromium } from "../../apps/desktop/node_modules/playwright/index.mjs";
import { createServer } from "node:net";
import { spawn, execFile, spawnSync } from "node:child_process";
import { promisify } from "node:util";
import { mkdir, writeFile, readdir, readFile } from "node:fs/promises";
import { createWriteStream } from "node:fs";
import path from "node:path";
import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { fileURLToPath } from "node:url";

const root = path.resolve(
  path.dirname(fileURLToPath(import.meta.url)),
  "../..",
);
const phase = process.argv[2] ?? "after";
const artifact = path.join(root, "artifacts", `extensions-${Date.now()}`);
await mkdir(artifact, { recursive: true });
const desktop =
  process.env.RECONX_DESKTOP_EXE ??
  path.join(root, "target/release/reconx-desktop.exe");
const cli =
  process.env.RECONX_CLI_EXE ?? path.join(root, "target/release/reconx.exe");
const env = {
  ...process.env,
  RECONX_DATA_DIR: artifact,
  RECONX_NATIVE_TEST_HIDDEN: "1",
  WEBVIEW2_USER_DATA_FOLDER: path.join(artifact, "webview"),
};
const exec = promisify(execFile);
const command = async (args) =>
  JSON.parse(
    (await exec(cli, ["--json", ...args], { env, windowsHide: true })).stdout,
  );
let child, browser, page;
const observations = [];
const checks = [];
const delay = (ms) => new Promise((r) => setTimeout(r, ms));
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
    if (child.exitCode !== null) throw Error("Native Desktop exited");
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
  for (let i = 0; i < 100; i++) {
    page = browser.contexts().flatMap((c) => c.pages())[0];
    if (page) break;
    await delay(100);
  }
  await page
    .getByRole("heading", { name: "Workspace overview", exact: true })
    .waitFor();
  await page.emulateMedia({ reducedMotion: "reduce" });
}
async function stop() {
  if (browser) await browser.close();
  browser = undefined;
  if (child && child.exitCode === null)
    spawnSync("taskkill", ["/PID", String(child.pid), "/T", "/F"], {
      windowsHide: true,
      stdio: "ignore",
    });
}
async function appearance() {
  return page.getByRole("button", {
    name: phase === "before" ? "Sidebar appearance" : "Appearance",
    exact: true,
  });
}

const id = "reconx.yara-signatures";
async function failed(args, code) {
  try {
    await command(args);
    assert.fail("Expected policy rejection");
  } catch (error) {
    assert(
      String(error.stderr ?? error).includes(code),
      String(error.stderr ?? error),
    );
  }
}
async function waitJob(job) {
  for (let i = 0; i < 300; i++) {
    const current = await command(["jobs", "show", job]);
    if (!["queued", "running"].includes(current.status)) return current;
    await delay(100);
  }
  throw Error("Actual extension job did not finish");
}
try {
  const legacy = process.env.RECONX_LEGACY_CLI_EXE;
  const legacyCommand = legacy
    ? async (args) =>
        JSON.parse(
          (await exec(legacy, ["--json", ...args], { env, windowsHide: true }))
            .stdout,
        )
    : command;
  const project = await legacyCommand([
    "project",
    "create",
    "Extension research fixture",
    "--description",
    "Owned offline fixture",
  ]);
  const sample = path.join(artifact, "owned-marker.bin");
  await writeFile(sample, "RECONX_BENIGN_FIXTURE\0owned research input");
  const file = await legacyCommand([
    "reverse",
    "import",
    sample,
    "--project",
    project.id,
    "--authorized",
  ]);
  const queued = await legacyCommand([
    "jobs",
    "enqueue",
    "--kind",
    "artifact_index",
    "--file",
    file.id,
  ]);
  const catalog = await command(["extensions", "search", "yara"]);
  assert.equal(catalog[0].status, "not_installed");
  assert.equal(catalog[0].manifest.id, id);
  await failed(
    ["extensions", "install", id, "--fingerprint", "unreviewed"],
    "APPROVAL_REQUIRED",
  );
  await failed(
    ["extensions", "install", id, "--fingerprint", "unreviewed", "--approve"],
    "PLAN_CHANGED",
  );
  if (legacy) {
    const backups = (await readdir(artifact)).filter(
      (s) => s.includes("pre-v16") && s.endsWith(".bak"),
    );
    assert.equal(backups.length, 1);
    const inspected = JSON.parse(
      (
        await exec(
          "python",
          [
            "-c",
            "import sqlite3,json,sys; c=sqlite3.connect(sys.argv[1]); print(json.dumps([c.execute('PRAGMA user_version').fetchone()[0],c.execute('PRAGMA integrity_check').fetchone()[0],c.execute('SELECT status FROM analysis_jobs WHERE id=?',[sys.argv[2]]).fetchone()[0]]))",
            path.join(artifact, backups[0]),
            queued.id,
          ],
          { windowsHide: true },
        )
      ).stdout,
    );
    assert.deepEqual(inspected, [Number(process.env.RECONX_LEGACY_SCHEMA ?? 14), "ok", "queued"]);
    checks.push(
      "Actual older-schema CLI queued work is retained in an intact pre-v16 backup before the new binary migrates",
    );
  }
  await launch();
  assert.equal((await waitJob(queued.id)).status, "completed");
  await page
    .getByRole("navigation", { name: "Primary navigation" })
    .getByRole("button", { name: "Extensions", exact: true })
    .click();
  await page
    .getByRole("heading", { name: "Extensions", exact: true })
    .waitFor();
  const search = page.getByRole("searchbox", {
    name: "Search extensions",
    exact: true,
  });
  await search.fill("missing-package");
  await page.getByText("No matching extensions", { exact: true }).waitFor();
  await search.fill("yara");
  await page.setViewportSize({ width: 1280, height: 720 });
  await page
    .getByRole("button", { name: "Install extension", exact: true })
    .click();
  let review = page.getByRole("dialog", {
    name: "Review extension installation",
    exact: true,
  });
  await review.waitFor();
  assert(await review.evaluate(el => el.scrollWidth <= el.clientWidth + 1));
  assert(
    await review
      .getByRole("button", { name: "Approve and install", exact: true })
      .isDisabled(),
  );
  assert.equal((await command(["extensions", "list"])).length, 0);
  await review
    .getByRole("checkbox", {
      name: "I approve these dependencies and extension permissions",
      exact: true,
    })
    .check();
  await page.screenshot({
    path: path.join(artifact, "install-plan.png"),
    fullPage: true,
  });
  await review
    .getByRole("button", { name: "Approve and install", exact: true })
    .click();
  await review.waitFor({ state: "hidden", timeout: 180000 });
  assert.equal((await command(["extensions", "info", id])).status, "ready");
  const tools = await command(["tools", "list"]);
  assert.equal(tools.find((t) => t.manifest.id === "yara-x").status, "tested");
  assert(tools.find((t) => t.manifest.id === "yara-x").integrity_verified);
  checks.push(
    "Native approval downloads the real pinned YARA-X dependency, verifies hashes/version, runs its benign signature health check, and shares Ready state with CLI",
  );
  await page
    .getByLabel("Extension input file", { exact: true })
    .selectOption(file.id);
  await page
    .getByRole("button", { name: "Run signatures", exact: true })
    .click();
  await page.getByText(/Signature job queued:/).waitFor();
  const first = (await command(["extensions", "history", id]))[0];
  const done = await waitJob(first.id);
  assert.equal(done.status, "completed", JSON.stringify(done));
  assert(
    JSON.stringify(done.result.observations).includes("ReconX_Test_Marker"),
  );
  assert.equal(
    done.result.sample_sha256,
    createHash("sha256")
      .update(await readFile(sample))
      .digest("hex"),
  );
  const evidence = await command(["evidence", "list", "--project", project.id]);
  assert(evidence.some((e) => e.job_id === done.id));
  checks.push(
    "Marketplace invocation uses the shared Job Engine and actual signature output with matching sample hash and persisted source evidence",
  );
  await page
    .getByRole("button", { name: "Disable extension", exact: true })
    .click();
  await page
    .getByRole("button", { name: "Enable extension", exact: true })
    .waitFor();
  assert.equal((await command(["extensions", "info", id])).status, "disabled");
  await failed(
    ["extensions", "invoke", id, "--file", file.id],
    "EXTENSION_NOT_READY",
  );
  await failed(["tools", "remove", "yara-x"], "TOOL_REQUIRED_BY_EXTENSION");
  await page
    .getByRole("button", { name: "Enable extension", exact: true })
    .click();
  await page
    .getByRole("button", { name: "Disable extension", exact: true })
    .waitFor();
  await stop();
  const toCancel = await command([
    "extensions",
    "invoke",
    id,
    "--file",
    file.id,
  ]);
  await command(["extensions", "disable", id]);
  assert.equal(
    (await command(["jobs", "show", toCancel.id])).status,
    "cancelled",
  );
  await command(["extensions", "enable", id]);
  await launch();
  await page
    .getByRole("navigation", { name: "Primary navigation" })
    .getByRole("button", { name: "Extensions", exact: true })
    .click();
  await page
    .getByRole("button", { name: "Run health check", exact: true })
    .click();
  await page
    .getByText(
      "Executable integrity, version and real benign signature scan passed.",
      { exact: true },
    )
    .waitFor();
  checks.push(
    "Disable rejects new invocations and cancels queued bound work; shared dependencies cannot be removed while registered; state/history and real health checks survive native restart",
  );
  await failed(["extensions", "update", id], "NO_EXTENSION_UPDATE");
  await page
    .getByRole("tab", { name: "Updates available", exact: true })
    .click();
  await page
    .getByText("No compatible extension update is available in this catalog.", {
      exact: true,
    })
    .waitFor();
  await page.getByRole("tab", { name: "Installed", exact: true }).click();
  for (const theme of ["Light", "Dark", "ReconX"]) {
    await page.getByRole("button", { name: "Appearance", exact: true }).click();
    await page
      .getByRole("menuitemradio", { name: new RegExp(`^${theme}`) })
      .click();
    for (const [width, height] of [
      [1280, 720],
      [1366, 768],
      [1440, 900],
      [1920, 1080],
    ]) {
      await page.setViewportSize({ width, height });
      await delay(220);
      assert(
        await page.evaluate(
          () =>
            document.querySelector("main").scrollWidth <=
            document.querySelector("main").clientWidth + 1,
        ),
      );
      await page.screenshot({
        path: path.join(
          artifact,
          `marketplace-${theme.toLowerCase()}-${width}.png`,
        ),
      });
    }
  }
  await page
    .getByRole("button", { name: "Uninstall extension", exact: true })
    .click();
  await page
    .getByRole("button", { name: "Install extension", exact: true })
    .waitFor();
  assert.equal((await command(["extensions", "list"])).length, 0);
  assert.equal(
    (await command(["tools", "list"])).find((t) => t.manifest.id === "yara-x")
      .status,
    "tested",
  );
  assert.equal((await command(["extensions", "history", id])).length, 2);
  review = page.getByRole("dialog", {
    name: "Review extension installation",
    exact: true,
  });
  await page
    .getByRole("button", { name: "Install extension", exact: true })
    .click();
  await review
    .getByRole("checkbox", {
      name: "I approve these dependencies and extension permissions",
      exact: true,
    })
    .check();
  await review
    .getByRole("button", { name: "Approve and install", exact: true })
    .click();
  await review.waitFor({ state: "hidden", timeout: 60000 });
  assert.equal((await command(["extensions", "info", id])).status, "ready");
  checks.push(
    "No fabricated update is offered; uninstall retains shared tools/evidence/history and approved reinstall reuses and verifies the real dependency; three-theme/four-size screenshots captured",
  );
  await writeFile(
    path.join(artifact, "results.json"),
    JSON.stringify(
      {
        status: "passed",
        desktop,
        cli,
        checks,
        fixtureOnly: true,
        scope:
          "Actual Windows native first-party adapter. No cloud calls, arbitrary community code, external active research or OS sandbox claimed.",
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
      .screenshot({ path: path.join(artifact, "failure.png"), fullPage: true })
      .catch(() => {});
  await writeFile(
    path.join(artifact, "results.json"),
    JSON.stringify({ status: "failed", error: String(error), checks }, null, 2),
  );
  throw error;
} finally {
  await stop();
}
