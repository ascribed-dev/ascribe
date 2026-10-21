// Takes a picture of the editor in a real VS Code, one per scheme, written as
// `<name>-light.png` and `<name>-dark.png` into the folder given:
//
//   node scripts/screenshot.ts <folder> <name>
//
// `editor`, the site's picture, is a copy of examples/quill with a link
// written as a page's address, its warning in the Problems view, and its
// quick fix open. The others are the docs' scenes (SCENES below): each opens
// its own workspace, and scenes.ts, run in VS Code's extension host, stages
// it through the extension's API, a step at a time, while this takes the
// picture of each step.
//
// `node scripts/docs-site/screenshots.ts <name>`, at the repository's root,
// builds the extension, runs this, and records the release beside the
// pictures. It needs ASCRIBE_BIN, the path to a built `ascribe`. The page is
// laid out at a fixed size whatever the screen, so a picture is the same on
// every run on one machine: on Linux VS Code is drawn on a virtual screen of
// that size (`xvfb-run`), and elsewhere in a window whose page is held to it.
// VS Code is the version below, downloaded on first use into out/vscode-test,
// as the integration tests do. The status bar's scene runs `ascribe` through a
// shell script, so it needs a POSIX shell: Linux or macOS.
import { execFileSync, spawn, type ChildProcess } from "node:child_process";
import {
  chmodSync,
  cpSync,
  existsSync,
  mkdirSync,
  readFileSync,
  rmSync,
  writeFileSync,
} from "node:fs";
import { createServer } from "node:net";
import { tmpdir } from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { downloadAndUnzipVSCode } from "@vscode/test-electron";
import { chromium, type Browser, type Page } from "playwright-core";

/** The VS Code the pictures show. A new one changes them: retake them when it moves. */
const VSCODE = "1.141.0";

/** The window, in CSS pixels, unless a picture sets its own; the picture is twice as many each way. */
const [WIDTH, HEIGHT] = [1100, 700];
const SCALE = 2;

/** Whether VS Code is drawn on a virtual screen of the picture's size, as on Linux with Xvfb. */
const VIRTUAL = process.platform === "linux";

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

/** A workspace to open: its folder, a place to open in it, and settings beyond the quiet ones. */
interface Workspace {
  root: string;
  goto?: string;
  settings?: Record<string, unknown>;
}

/** Waits for the extension host to stage a scene's step `n`, and says when its picture is taken. */
interface Stage {
  staged(n: number): Promise<void>;
  taken(n: number): void;
}

/** A picture: the workspace it opens, and how it's taken once VS Code is up. */
interface Picture {
  /** Whether scenes.ts stages it, under the picture's name. */
  scene: boolean;
  /** The window's size in CSS pixels, when it isn't WIDTH by HEIGHT. */
  size?: [width: number, height: number];
  workspace(scratch: string, server: string): Workspace;
  take(window: Page, stage: Stage): Promise<Buffer>;
}

/** A copy of an example, as `name` in `scratch`. */
function copy(scratch: string, example: string, name = path.basename(example)): string {
  const root = path.join(scratch, name);
  cpSync(path.join(repositoryRoot, "examples", example), root, { recursive: true });
  return root;
}

/** Runs git in `cwd` as nobody in particular. */
function git(cwd: string, ...args: string[]): void {
  execFileSync(
    "git",
    [
      "-c",
      "user.name=Ana",
      "-c",
      "user.email=ana@example.com",
      "-c",
      "commit.gpgsign=false",
      "-c",
      "core.autocrlf=false",
      ...args,
    ],
    { cwd, stdio: "ignore" },
  );
}

/** Replaces `from` in a file, which must have it. */
function edit(file: string, from: string, to: string): void {
  const text = readFileSync(file, "utf8").replace(/\r\n/g, "\n");
  if (!text.includes(from)) throw new Error(`${file} no longer has: ${from}`);
  writeFileSync(file, text.replace(from, to));
}

/** The window, without the title bar, which names it as an extension's test host. */
async function shot(window: Page): Promise<Buffer> {
  // The last entrance and layout.
  await window.waitForTimeout(1000);
  const title = await window.locator(".part.titlebar").boundingBox();
  const top = Math.ceil(title?.height ?? 0);
  const [width, height] = await window.evaluate<[number, number]>("[innerWidth, innerHeight]");
  return window.screenshot({
    clip: { x: 0, y: top, width, height: height - top },
    animations: "disabled",
    caret: "initial",
  });
}

/** Every language server started as the workspace opens, rather than on a page's. */
const startAll = { "ascribe.startServers": "all" };

/** Takes one step's picture of the window, once `visible` is. */
async function stepShot(window: Page, stage: Stage, visible: string): Promise<Buffer> {
  await stage.staged(0);
  await window.locator(visible).first().waitFor({ timeout: 60_000 });
  const picture = await shot(window);
  stage.taken(0);
  return picture;
}

export const PICTURES: Record<string, Picture> = {
  editor: {
    scene: false,
    workspace(scratch) {
      const root = copy(scratch, "quill");
      const file = path.join(root, PAGE);
      edit(file, LINE, BROKEN);
      const lines = readFileSync(file, "utf8").split("\n");
      const line = lines.indexOf(BROKEN) + 1;
      const column = BROKEN.indexOf("/keys/") + 3;
      return { root, goto: `${file}:${line}:${column}` };
    },
    async take(window) {
      // The language server has checked the page.
      await window.locator(".squiggly-warning").first().waitFor({ timeout: 60_000 });
      await command(window, "Notifications: Clear All Notifications");
      await command(window, "View: Close Primary Side Bar");
      await command(window, "View: Focus Problems");
      await window.locator(".markers-panel .monaco-list-row").first().waitFor();
      await command(window, "View: Focus First Editor Group");
      await window.keyboard.press(process.platform === "darwin" ? "Meta+." : "Control+.");
      await window.locator(".action-widget").first().waitFor();
      return shot(window);
    },
  },
  "build-lens": {
    scene: true,
    workspace: (scratch) => ({ root: copy(scratch, "platforms"), settings: startAll }),
    take: (window, stage) => stepShot(window, stage, ".monaco-hover:has-text('Left out of')"),
  },
  "actions-bar": {
    scene: true,
    workspace: (scratch) => ({ root: copy(scratch, "quill"), settings: startAll }),
    take: (window, stage) => stepShot(window, stage, ".quick-input-widget .monaco-list-row"),
  },
  "review-view": {
    scene: true,
    workspace(scratch) {
      const root = copy(scratch, "quill");
      git(root, "init", "-q", "-b", "main");
      git(root, "add", "-A");
      git(root, "commit", "-q", "-m", "The Quill example");
      git(root, "checkout", "-q", "-b", "feature");
      edit(
        path.join(root, "docs", "install-agent.md"),
        "syncs changes to",
        "syncs every change to",
      );
      git(root, "commit", "-q", "-am", "Reword the agent's introduction");
      // Pushed to GitHub, which scenes.ts fakes.
      git(root, "remote", "add", "origin", "https://github.com/acme/quill.git");
      git(root, "config", "branch.feature.remote", "origin");
      git(root, "config", "branch.feature.merge", "refs/heads/feature");
      git(root, "update-ref", "refs/remotes/origin/main", "main");
      git(root, "update-ref", "refs/remotes/origin/feature", "feature");
      return {
        root,
        settings: { ...startAll, "workbench.view.alwaysShowHeaderActions": true },
      };
    },
    take: (window, stage) => stepShot(window, stage, ".activitybar .badge-content:text-is('1')"),
  },
  "shown-in-pages": {
    scene: true,
    workspace(scratch) {
      const root = copy(scratch, "tested");
      const page = (file: string, title: string, text: string) =>
        writeFileSync(
          path.join(root, "docs", file),
          `---\ntitle: ${title}\n---\n\n${text}\n\n@snippet: code:examples/greet.ts#greet\n`,
        );
      page("quickstart.md", "Quickstart", "Greet your first reader:");
      page("reference.md", "Reference", "`greet` takes a name and returns a greeting:");
      return { root, goto: path.join(root, "code", "examples", "greet.ts"), settings: startAll };
    },
    take: (window, stage) => stepShot(window, stage, ".peekview-widget .monaco-list-row"),
  },
  "this-page": {
    scene: true,
    // Tall enough for every section of the view.
    size: [1100, 1000],
    workspace: (scratch) => ({ root: copy(scratch, "quill"), settings: startAll }),
    async take(window, stage) {
      await stage.staged(0);
      await only(window, "This page", 480);
      const picture = await shot(window);
      stage.taken(0);
      return picture;
    },
  },
  pages: {
    scene: true,
    workspace(scratch) {
      const root = copy(scratch, "quill");
      const docs = path.join(root, "docs");
      const page = (file: string, title: string, text: string) =>
        writeFileSync(path.join(docs, file), `---\ntype: page\ntitle: ${title}\n---\n\n${text}\n`);
      mkdirSync(path.join(docs, "reference"));
      page("reference/index.md", "Reference", "Every command and setting of {product}.");
      page(
        "reference/cli.md",
        "The quill command",
        "`quill init` writes [quill.yaml](config.md) and starts [the agent](../agnet.md).",
      );
      page("reference/config.md", "quill.yaml", "The agent reads `quill.yaml` when it starts.");
      // Keys left out of the Start section's order.
      writeFileSync(
        path.join(root, "ascribe.nav"),
        [
          "version = 1",
          "",
          "[[sections]]",
          'folder = ""',
          'pages = ["quickstart.md", "install-agent.md", "configure.md", "configure-legacy.md", "upgrade.md", "webhooks.md"]',
          "",
          "[[sections]]",
          'folder = "reference/"',
          "",
        ].join("\n"),
      );
      return { root, settings: startAll };
    },
    async take(window, stage) {
      await stage.staged(0);
      await only(window, "Pages");
      // Each section open, as a click on it opens it.
      const pane = window.locator(".pane", {
        has: window.locator(".pane-header h3", { hasText: /^Pages$/ }),
      });
      for (const label of ["Start", "Reference"]) {
        const row = pane.locator(".monaco-list-row", { hasText: label }).first();
        if ((await row.getAttribute("aria-expanded")) === "false")
          await row.locator(".monaco-tl-twistie").click();
      }
      await pane.locator(".monaco-list-row", { hasText: "quill.yaml" }).first().waitFor();
      // The click's selection and focus.
      await command(window, "View: Focus First Editor Group");
      const picture = await shot(window);
      stage.taken(0);
      return picture;
    },
  },
  project: {
    scene: true,
    // Tall enough for Health and Actions.
    size: [1100, 1000],
    workspace: (scratch) => ({ root: copy(scratch, "quill"), settings: startAll }),
    async take(window, stage) {
      await stage.staged(0);
      await only(window, "Project", 480);
      const picture = await shot(window);
      stage.taken(0);
      return picture;
    },
  },
  "status-bar": {
    scene: true,
    workspace(scratch, server) {
      const root = path.join(scratch, "workspace");
      mkdirSync(root);
      copy(root, "platforms", "docs");
      writeFileSync(path.join(root, "README.md"), "# The workspace\n\nIts docs are in docs/.\n");
      // `ascribe`, once `go` is there, and failing once `fail` is.
      const slow = path.join(scratch, "ascribe");
      writeFileSync(
        slow,
        [
          "#!/bin/sh",
          `while [ ! -e '${path.join(root, "go")}' ]; do sleep 0.2; done`,
          `[ -e '${path.join(root, "fail")}' ] && exit 1`,
          `exec '${server}' "$@"`,
          "",
        ].join("\n"),
      );
      chmodSync(slow, 0o755);
      return { root, settings: { "ascribe.path": slow } };
    },
    async take(window, stage) {
      const item = window.locator(`[id="Ascribe.ascribe-vscode.ascribe.project"]`);
      const bar = window.locator(".part.statusbar");
      const crops: string[] = [];
      const STATES = 5;
      for (let n = 0; n < STATES; n++) {
        await stage.staged(n);
        await window.waitForTimeout(500);
        const at = await item.boundingBox();
        const strip = await bar.boundingBox();
        if (!at || !strip) throw new Error("no status bar item");
        const crop = await window.screenshot({
          clip: { x: at.x - 8, y: strip.y, width: at.width + 16, height: strip.height },
          animations: "disabled",
        });
        crops.push(crop.toString("base64"));
        // The last step's is taken once the board is: VS Code quits then.
        if (n < STATES - 1) stage.taken(n);
      }
      // The states one above another, on the status bar's own color.
      // A string, since this script has no DOM types: the page's own code.
      await window.evaluate(`{
        const bar = document.querySelector(".part.statusbar");
        const board = document.createElement("div");
        board.id = "ascribe-states";
        Object.assign(board.style, {
          position: "fixed",
          inset: "0 auto auto 0",
          zIndex: "100000",
          display: "flex",
          flexDirection: "column",
          alignItems: "flex-start",
          gap: "6px",
          padding: "10px 12px",
          background: getComputedStyle(bar).backgroundColor,
        });
        for (const data of ${JSON.stringify(crops)}) {
          const image = document.createElement("img");
          image.src = "data:image/png;base64," + data;
          image.style.zoom = String(1 / devicePixelRatio);
          board.append(image);
        }
        document.body.append(board);
      }`);
      await window.waitForTimeout(500);
      const board = await window.locator("#ascribe-states").screenshot({ animations: "disabled" });
      stage.taken(STATES - 1);
      return board;
    },
  },
};

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

/**
 * Collapses every view of the Ascribe sidebar but `title`, so it has the
 * sidebar's height, and widens the sidebar to `width`.
 */
async function only(window: Page, title: string, width = 400): Promise<void> {
  const sidebar = await window.locator(".part.sidebar").boundingBox();
  if (!sidebar) throw new Error("no sidebar");
  // Its edge, dragged.
  const edge = sidebar.x + sidebar.width;
  const y = sidebar.y + sidebar.height / 2;
  await window.mouse.move(edge, y);
  await window.mouse.down();
  await window.mouse.move(edge + (sidebar.x + width - edge) / 2, y);
  await window.mouse.move(sidebar.x + width, y);
  await window.mouse.up();
  const headers = window.locator(".sidebar .pane-header");
  for (const header of await headers.all()) {
    const name = (await header.locator("h3").textContent())?.trim();
    const expanded = (await header.getAttribute("aria-expanded")) === "true";
    if ((name === title) !== expanded) await header.click();
  }
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

async function take(
  name: string,
  scheme: "light" | "dark",
  out: string,
  server: string,
): Promise<void> {
  const picture = PICTURES[name];
  if (!picture) throw new Error(`no picture named ${name}`);
  const executable = await downloadAndUnzipVSCode({
    version: VSCODE,
    cachePath: path.join(packageRoot, "out", "vscode-test"),
  });
  // One path every time: the references peek shows a page's folder.
  const scratch = path.join(tmpdir(), "ascribe-screenshot");
  rmSync(scratch, { recursive: true, force: true, maxRetries: 5 });
  mkdirSync(scratch);
  const workspace = picture.workspace(scratch, server);
  const user = path.join(scratch, "user");
  mkdirSync(path.join(user, "User"), { recursive: true });
  writeFileSync(
    path.join(user, "User", "settings.json"),
    JSON.stringify({ ...settings(scheme, server), ...workspace.settings }),
  );
  const signals = path.join(scratch, "signals");
  mkdirSync(signals);
  const port = await freePort();
  const env: NodeJS.ProcessEnv = {
    ...process.env,
    ASCRIBE_SCENE: name,
    ASCRIBE_SCENE_SIGNALS: signals,
  };
  // Set by an extension host, and VS Code would then run as plain Node.
  delete env["ELECTRON_RUN_AS_NODE"];
  const args = [
    workspace.root,
    ...(workspace.goto === undefined ? [] : ["--goto", workspace.goto]),
    `--extensionDevelopmentPath=${packageRoot}`,
    ...(picture.scene
      ? [`--extensionTestsPath=${path.join(packageRoot, "out", "scenes.cjs")}`]
      : []),
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
  ];
  const [width, height] = picture.size ?? [WIDTH, HEIGHT];
  const screen = `-screen 0 ${width * SCALE}x${height * SCALE}x24`;
  const child = VIRTUAL
    ? spawn("xvfb-run", ["-a", "-s", screen, executable, ...args], {
        stdio: "ignore",
        env,
        detached: true,
      })
    : spawn(executable, args, { stdio: "ignore", env, detached: true });
  const stage: Stage = {
    staged: async (n) => {
      const failed = path.join(signals, "failed");
      const staged = path.join(signals, `staged-${n}`);
      await poll(`scene step ${n}`, async () =>
        existsSync(failed) || existsSync(staged) ? true : undefined,
      );
      if (existsSync(failed))
        throw new Error(`The scene failed at ${readFileSync(failed, "utf8")}`);
    },
    taken: (n) => writeFileSync(path.join(signals, `taken-${n}`), ""),
  };
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
    if (!VIRTUAL) {
      // A real window is the screen's size: hold the page to the picture's.
      const cdp = await window.context().newCDPSession(window);
      await cdp.send("Emulation.setDeviceMetricsOverride", {
        width,
        height,
        deviceScaleFactor: SCALE,
        mobile: false,
      });
    }
    writeFileSync(path.join(out, `${name}-${scheme}.png`), await picture.take(window, stage));
  } finally {
    await quit(child, browser);
    rmSync(scratch, { recursive: true, force: true, maxRetries: 5 });
  }
}

/**
 * Quits VS Code as its menu does, through the debugging protocol, and stops
 * it only when it hasn't gone after a while. Stopped with a signal, VS Code on
 * macOS leaves a dialog saying its window was killed.
 */
async function quit(child: ChildProcess, browser: Browser | undefined): Promise<void> {
  const gone = (): boolean => child.exitCode !== null || child.signalCode !== null;
  const exited = new Promise<boolean>((resolve) => {
    if (gone()) resolve(true);
    else child.once("exit", () => resolve(true));
  });
  const timeout = new Promise<boolean>((resolve) => setTimeout(() => resolve(false), 10_000));
  if (browser !== undefined) {
    // Not awaited: VS Code quits before it answers, and the answer never comes.
    void browser
      .newBrowserCDPSession()
      .then((session) => session.send("Browser.close"))
      .catch(() => undefined);
  }
  if (await Promise.race([exited, timeout])) return;
  console.error("VS Code didn't quit after 10 seconds; stopping it.");
  if (child.pid !== undefined) {
    try {
      process.kill(-child.pid, "SIGTERM");
    } catch {
      // Gone already.
    }
  }
  await new Promise((resolve) => setTimeout(resolve, 1000));
}

if (import.meta.main) {
  const [out, name] = process.argv.slice(2);
  const server = process.env["ASCRIBE_BIN"];
  if (out === undefined || name === undefined || server === undefined || !(name in PICTURES)) {
    console.error(
      `Usage: ASCRIBE_BIN=<a built ascribe> node scripts/screenshot.ts <folder> <${Object.keys(PICTURES).join(" | ")}>`,
    );
    process.exit(2);
  }
  mkdirSync(out, { recursive: true });
  for (const scheme of ["light", "dark"] as const) {
    await take(name, scheme, path.resolve(out), path.resolve(server));
    console.log(`Wrote ${path.join(out, `${name}-${scheme}.png`)}`);
  }
}
