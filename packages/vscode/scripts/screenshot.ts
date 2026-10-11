// Takes the site's picture of the editor: a copy of examples/quill in a real
// VS Code, with a link written as a page's address, its warning in the
// Problems view, and its quick fix open. One picture per scheme, written as
// `editor-light.png` and `editor-dark.png` into the folder given.
//
//   node scripts/screenshot.ts <folder>
//
// `node scripts/docs-site/screenshots.ts editor`, at the repository's root,
// builds the extension, runs this, and records the release beside the
// pictures. It needs ASCRIBE_BIN, the path to a built `ascribe`, and Linux
// with Xvfb (`xvfb-run`): the window is drawn on a virtual screen of a fixed
// size, so the picture is the same on every run. VS Code is the version
// below, downloaded on first use into out/vscode-test, as the integration
// tests do.
import { spawn } from "node:child_process";
import { cpSync, mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { createServer } from "node:net";
import { tmpdir } from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { downloadAndUnzipVSCode } from "@vscode/test-electron";
import { chromium, type Browser, type Page } from "playwright-core";

/** The VS Code the pictures show. A new one changes them: retake them when it moves. */
const VSCODE = "1.141.0";

/** The window, in CSS pixels; the picture is twice as many each way. */
const [WIDTH, HEIGHT] = [1100, 700];
const SCALE = 2;

const packageRoot = fileURLToPath(new URL("..", import.meta.url));
const repositoryRoot = path.resolve(packageRoot, "../..");

/** The page that's open, the line that's changed, and what it says instead. */
const PAGE = path.join("docs", "quickstart.md");
const LINE = "Open the playground at play.quill.dev and paste a page of your docs.";
const BROKEN = `${LINE} Its shortcuts are in [the keys](/keys/).`;

function freePort(): Promise<number> {
  return new Promise((resolve, reject) => {
    const server = createServer();
    server.once("error", reject);
    server.listen(0, "127.0.0.1", () => {
      const address = server.address();
      const port = typeof address === "object" && address !== null ? address.port : 0;
      server.close(() => resolve(port));
    });
  });
}

async function poll<T>(what: string, find: () => Promise<T | undefined>): Promise<T> {
  const deadline = Date.now() + 60_000;
  for (;;) {
    const found = await find().catch(() => undefined);
    if (found !== undefined) return found;
    if (Date.now() > deadline) throw new Error(`no ${what} after a minute`);
    await new Promise((resolve) => setTimeout(resolve, 500));
  }
}

/** A copy of examples/quill with the broken link, and where the cursor goes. */
function workspace(scratch: string): { root: string; goto: string } {
  const root = path.join(scratch, "quill");
  cpSync(path.join(repositoryRoot, "examples", "quill"), root, { recursive: true });
  const file = path.join(root, PAGE);
  const text = readFileSync(file, "utf8").replace(/\r\n/g, "\n");
  if (!text.includes(LINE)) throw new Error(`examples/quill/${PAGE} no longer has: ${LINE}`);
  writeFileSync(file, text.replace(LINE, BROKEN));
  const lines = text.replace(LINE, BROKEN).split("\n");
  const line = lines.indexOf(BROKEN) + 1;
  const column = BROKEN.indexOf("/keys/") + 3;
  return { root, goto: `${file}:${line}:${column}` };
}

/** VS Code's settings for a quiet window: the theme, and nothing that moves or invites. */
function settings(scheme: "light" | "dark", server: string): Record<string, unknown> {
  return {
    "workbench.colorTheme": scheme === "dark" ? "Default Dark Modern" : "Default Light Modern",
    "ascribe.path": server,
    "editor.cursorBlinking": "solid",
    "editor.minimap.enabled": false,
    // The menu is open; a light bulb beside it comes and goes with timing.
    "editor.lightbulb.enabled": "off",
    "editor.fontSize": 14,
    "workbench.reduceMotion": "on",
    // A tab's problem count fades in, and the picture would catch it partway.
    "workbench.editor.decorations.badges": false,
    "workbench.startupEditor": "none",
    "workbench.tips.enabled": false,
    "workbench.layoutControl.enabled": false,
    "workbench.secondarySideBar.defaultVisibility": "hidden",
    "window.restoreWindows": "none",
    "window.newWindowDimensions": "maximized",
    "window.commandCenter": false,
    "chat.disableAIFeatures": true,
    "chat.commandCenter.enabled": false,
    "telemetry.telemetryLevel": "off",
    "update.mode": "none",
    "extensions.ignoreRecommendations": true,
  };
}

/** Runs a command from the command palette by its title. */
async function command(window: Page, title: string): Promise<void> {
  await window.keyboard.press("F1");
  const input = window.locator(".quick-input-widget input");
  await input.waitFor();
  await input.fill(`>${title}`);
  await window.locator(".quick-input-list .monaco-list-row", { hasText: title }).first().waitFor();
  await window.keyboard.press("Enter");
  await window.locator(".quick-input-widget").waitFor({ state: "hidden" });
}

async function picture(scheme: "light" | "dark", out: string, server: string): Promise<void> {
  const executable = await downloadAndUnzipVSCode({
    version: VSCODE,
    cachePath: path.join(packageRoot, "out", "vscode-test"),
  });
  const scratch = mkdtempSync(path.join(tmpdir(), "shot-"));
  const { root, goto } = workspace(scratch);
  const user = path.join(scratch, "user");
  mkdirSync(path.join(user, "User"), { recursive: true });
  writeFileSync(path.join(user, "User", "settings.json"), JSON.stringify(settings(scheme, server)));
  const port = await freePort();
  const env = { ...process.env };
  // Set by an extension host, and VS Code would then run as plain Node.
  delete env["ELECTRON_RUN_AS_NODE"];
  const screen = `-screen 0 ${WIDTH * SCALE}x${HEIGHT * SCALE}x24`;
  const child = spawn(
    "xvfb-run",
    [
      "-a",
      "-s",
      screen,
      executable,
      root,
      "--goto",
      goto,
      `--extensionDevelopmentPath=${packageRoot}`,
      "--disable-workspace-trust",
      "--disable-gpu",
      "--disable-updates",
      "--no-sandbox",
      "--skip-welcome",
      "--skip-release-notes",
      `--force-device-scale-factor=${SCALE}`,
      `--user-data-dir=${user}`,
      `--extensions-dir=${path.join(scratch, "extensions")}`,
      `--remote-debugging-port=${port}`,
    ],
    { stdio: "ignore", env, detached: true },
  );
  let browser: Browser | undefined;
  try {
    browser = await poll("VS Code", () => chromium.connectOverCDP(`http://127.0.0.1:${port}`));
    const connected = browser;
    const window = await poll("window", async () =>
      connected
        .contexts()
        .flatMap((context) => context.pages())
        .find((page) => page.url().includes("workbench")),
    );
    // The language server has checked the page.
    await window.locator(".squiggly-warning").first().waitFor({ timeout: 60_000 });
    await command(window, "Notifications: Clear All Notifications");
    await command(window, "View: Close Primary Side Bar");
    await command(window, "View: Focus Problems");
    await window.locator(".markers-panel .monaco-list-row").first().waitFor();
    await command(window, "View: Focus First Editor Group");
    await window.keyboard.press("Control+.");
    await window.locator(".action-widget").first().waitFor();
    // The menu's entrance, and the panel's last layout.
    await window.waitForTimeout(1000);
    // Without the title bar, which names the window as an extension's test host.
    const title = await window.locator(".part.titlebar").boundingBox();
    const top = Math.ceil(title?.height ?? 0);
    await window.screenshot({
      path: path.join(out, `editor-${scheme}.png`),
      clip: { x: 0, y: top, width: WIDTH, height: HEIGHT - top },
      animations: "disabled",
      caret: "initial",
    });
  } finally {
    await browser?.close().catch(() => undefined);
    if (child.pid !== undefined) {
      try {
        process.kill(-child.pid, "SIGTERM");
      } catch {
        // Gone already.
      }
    }
    await new Promise((resolve) => setTimeout(resolve, 1000));
    rmSync(scratch, { recursive: true, force: true, maxRetries: 5 });
  }
}

const out = process.argv[2];
const server = process.env["ASCRIBE_BIN"];
if (out === undefined || server === undefined) {
  console.error("Usage: ASCRIBE_BIN=<a built ascribe> node scripts/screenshot.ts <folder>");
  process.exit(2);
}
if (process.platform !== "linux") {
  console.error("The editor's picture is taken on Linux, with Xvfb: see the comment at the top.");
  process.exit(2);
}
mkdirSync(out, { recursive: true });
for (const scheme of ["light", "dark"] as const) {
  await picture(scheme, path.resolve(out), path.resolve(server));
  console.log(`Wrote ${path.join(out, `editor-${scheme}.png`)}`);
}
