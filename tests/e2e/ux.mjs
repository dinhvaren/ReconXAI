import { chromium } from "../../apps/desktop/node_modules/playwright/index.mjs";
import { createServer } from "node:net";
import { spawn, execFile, spawnSync } from "node:child_process";
import { promisify } from "node:util";
import { mkdir, writeFile, readFile } from "node:fs/promises";
import { createWriteStream } from "node:fs";
import path from "node:path";
import assert from "node:assert/strict";
import { fileURLToPath } from "node:url";

const root = path.resolve(
  path.dirname(fileURLToPath(import.meta.url)),
  "../..",
);
const phase = process.argv[2] ?? "after";
const artifact = path.join(root, "artifacts", `ux-${phase}-${Date.now()}`);
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
  if (process.env.RECONX_VERIFY_ASSET === "1") {
    const html = await readFile(
      path.join(root, "apps/desktop/dist/index.html"),
      "utf8",
    );
    assert.equal(
      await page.locator('script[type="module"][src]').getAttribute("src"),
      html.match(/src="([^"]+\.js)"/)[1],
      "Native UX verification must use the current embedded frontend",
    );
  }
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
try {
  const project = await command([
    "project",
    "create",
    "UX research fixture",
    "--description",
    "Owned local workspace. No active research or cloud requests.",
  ]);
  await command([
    "target",
    "add",
    "--project",
    project.id,
    "--type",
    "url",
    "--value",
    "http://127.0.0.1:8080",
    "--name",
    "Owned local fixture",
    "--authorized",
  ]);
  await launch();
  if (phase !== "before") {
    assert.equal(await page.locator(".theme-trigger").count(), 1);
    assert.equal(await page.locator(".activity-sidebar").count(), 0);
    await page
      .getByRole("button", { name: "Show activity panel", exact: true })
      .click();
    await page
      .getByRole("complementary", { name: "Workspace activity", exact: true })
      .waitFor();
    await page
      .getByRole("button", { name: "Hide activity panel", exact: true })
      .last()
      .click();
    await page
      .getByRole("button", { name: "Collapse navigation", exact: true })
      .click();
    await page
      .getByRole("button", { name: "Expand navigation", exact: true })
      .click();
    checks.push(
      "One rail appearance control; optional activity and navigation toggle correctly",
    );
  }
  for (const theme of ["Light", "Dark", "ReconX"]) {
    await (await appearance()).click();
    await page
      .getByRole("menuitemradio", { name: new RegExp(`^${theme}`) })
      .click();
    await page.waitForFunction(
      (value) => document.documentElement.dataset.theme === value,
      theme.toLowerCase(),
    );
    await page.waitForFunction(
      () => !document.body.textContent.includes("Saving appearance"),
    );
    assert.equal(
      (await command(["settings", "show"])).theme,
      theme.toLowerCase(),
    );
    const modules = [
      "Dashboard",
      "Reconnaissance",
      "Web Security",
      "API Security",
      "Reverse Engineering",
      "MCP Hub",
      "Tool Manager",
      "Settings",
    ];
    if (phase !== "before")
      modules.push(
        "Projects",
        "Targets & Scope",
        "AI Research",
        "AI Providers",
        "AI Workspace",
        "Workflows",
        "Findings",
        "Evidence",
        "Reports",
      );
    if (
      phase !== "before" &&
      (await page
        .getByRole("button", { name: "Extensions", exact: true })
        .count())
    )
      modules.push("Extensions");
    for (const module of modules) {
      await (
        module === "Settings"
          ? page.getByRole("button", { name: "Settings", exact: true })
          : page
              .getByRole("navigation", { name: "Primary navigation" })
              .getByRole("button", { name: module, exact: true })
      ).click();
      await page.locator("main h1").waitFor();
      if (
        phase !== "before" &&
        ["Web Security", "API Security"].includes(module)
      ) {
        assert.equal(
          await page
            .locator("main")
            .getByLabel("HTTP project", { exact: true })
            .inputValue(),
          project.id,
          "Workspace project must agree with the top bar default selection",
        );
      }
      for (const [width, height] of [
        [1280, 720],
        [1366, 768],
        [1440, 900],
        [1920, 1080],
      ]) {
        await page.setViewportSize({ width, height });
        await page.evaluate(() => document.fonts.ready);
        await delay(220);
        const measured = await page.evaluate(() => {
          const main = document.querySelector("main");
          const visible = (el) => !!el.getClientRects().length;
          return {
            width: innerWidth,
            documentWidth: document.documentElement.scrollWidth,
            mainWidth: main.clientWidth,
            mainScrollWidth: main.scrollWidth,
            railWidth: document
              .querySelector(".icon-rail")
              .getBoundingClientRect().width,
            themeButtons: document.querySelectorAll(".theme-trigger").length,
            buttons: [
              ...main.querySelectorAll(".button,.text-button,.icon-button"),
            ]
              .filter(visible)
              .map((el) => ({
                label: el.textContent.trim() || el.getAttribute("aria-label"),
                height: el.getBoundingClientRect().height,
                font: getComputedStyle(el).fontSize,
                disabled: el.disabled,
                reason:
                  el.getAttribute("title") ||
                  el.getAttribute("aria-describedby"),
              })),
          };
        });
        assert(measured.documentWidth <= width + 1);
        if (phase !== "before") {
          assert(
            measured.mainScrollWidth <= measured.mainWidth + 1,
            `${module} main overflow at ${width}`,
          );
          assert.equal(measured.themeButtons, 1);
          assert.equal(measured.railWidth, 48, `Compact rail in ${theme}`);
          for (const button of measured.buttons)
            if (button.disabled)
              assert(
                button.reason,
                `${module}: disabled action has no explanation: ${button.label}`,
              );
          for (const button of measured.buttons)
            assert(
              [30, 34, 38].some((size) => Math.abs(button.height - size) < 1),
              `${module}: ${JSON.stringify(button)}`,
            );
        }
        observations.push({ module, theme, width, height, ...measured });
        await page.screenshot({
          path: path.join(
            artifact,
            `${module.replaceAll(/[^a-z0-9]/gi, "-")}-${theme.toLowerCase()}-${width}.png`,
          ),
        });
      }
      if (module === "Settings") {
        await page
          .locator("main")
          .getByRole("button", { name: "AI Providers", exact: true })
          .click();
        await delay(100);
        await page.screenshot({
          path: path.join(artifact, `AI-Providers-${theme.toLowerCase()}.png`),
        });
      }
    }
  }
  if (phase !== "before") {
    await page
      .getByRole("navigation", { name: "Primary navigation" })
      .getByRole("button", { name: "Dashboard", exact: true })
      .click();
    await (await appearance()).focus();
    await page.keyboard.press("ArrowDown");
    await page.keyboard.press("Home");
    await page.keyboard.press("Enter");
    assert.equal(
      await page.locator("html").getAttribute("data-theme"),
      "light",
    );
    await page.waitForFunction(
      () => !document.body.textContent.includes("Saving appearance"),
    );
    await stop();
    await launch();
    assert.equal(
      await page.locator("html").getAttribute("data-theme"),
      "light",
    );
    checks.push("Keyboard theme selection persists to CLI and native restart");
    const cdp = await page.context().newCDPSession(page);
    for (const scale of [1.25, 1.5, 2]) {
      await cdp.send("Emulation.setDeviceMetricsOverride", {
        width: 1280,
        height: 720,
        deviceScaleFactor: scale,
        mobile: false,
      });
      const shot = await cdp.send("Page.captureScreenshot", {
        format: "png",
        fromSurface: true,
        clip: { x: 0, y: 0, width: 1280, height: 720, scale: 1 },
      });
      const pixels = Buffer.from(shot.data, "base64");
      assert.equal(pixels.readUInt32BE(16), Math.round(1280 * scale));
      assert.equal(pixels.readUInt32BE(20), Math.round(720 * scale));
      await writeFile(path.join(artifact, `DPI-${scale}.png`), pixels);
    }
    await cdp.send("Emulation.clearDeviceMetricsOverride");
    checks.push(
      "125/150/200% raster scaling via CDP; physical Windows DPI and Linux GUI not claimed",
    );
  }
  await writeFile(
    path.join(artifact, "measurements.json"),
    JSON.stringify(observations, null, 2),
  );
  await writeFile(
    path.join(artifact, "results.json"),
    JSON.stringify(
      {
        status: "passed",
        phase,
        checks,
        observations: observations.length,
        desktop,
        cli,
        scope:
          "Actual Windows Tauri/WebView2; isolated fixture DB, no research traffic, installations or cloud calls",
      },
      null,
      2,
    ),
  );
  console.log(
    JSON.stringify(
      { artifact, phase, observations: observations.length, checks },
      null,
      2,
    ),
  );
} catch (error) {
  if (page)
    await page
      .screenshot({ path: path.join(artifact, "failure.png"), fullPage: true })
      .catch(() => {});
  await writeFile(
    path.join(artifact, "results.json"),
    JSON.stringify(
      { status: "failed", phase, error: String(error), observations },
      null,
      2,
    ),
  );
  throw error;
} finally {
  await stop();
}
