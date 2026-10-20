// What each of the docs' scenes does in VS Code's extension host, through the
// extension's own API and VS Code's commands: `screenshot.ts` opens VS Code
// with this as its tests' module and the scene's name in ASCRIBE_SCENE.
// A scene is a list of steps. After each, this writes `staged-<n>` into the
// folder in ASCRIBE_SCENE_SIGNALS and waits for `taken-<n>`, which
// `screenshot.ts` writes once it has the picture; after the last, VS Code quits.
// Bundled to out/scenes.cjs by `node esbuild.mjs --scenes`.
import { execFileSync } from "node:child_process";
import { existsSync, writeFileSync } from "node:fs";
import * as path from "node:path";
import * as vscode from "vscode";
import type { AscribeApi } from "../src/extension.js";
import type { FromWebview } from "../src/preview/protocol.js";
import { StatefulGitHub } from "../test/integration/suite/fakeGitHub.js";

type Step = (api: AscribeApi, folder: string) => Promise<void>;

const sleep = (ms: number) => new Promise((resolve) => setTimeout(resolve, ms));

async function waitFor<T>(
  what: string,
  check: () => T | undefined | false | Promise<T | undefined | false>,
): Promise<T> {
  const deadline = Date.now() + 60_000;
  for (;;) {
    const value = await check();
    if (value !== undefined && value !== false) return value;
    if (Date.now() > deadline) throw new Error(`no ${what} after a minute`);
    await sleep(100);
  }
}

async function open(folder: string, file: string): Promise<vscode.TextEditor> {
  const document = await vscode.workspace.openTextDocument(path.join(folder, file));
  return vscode.window.showTextDocument(document, { viewColumn: vscode.ViewColumn.One });
}

/** Puts the cursor at the start of `text`, on the first line that has it. */
function cursorAt(editor: vscode.TextEditor, text: string): vscode.Position {
  for (let line = 0; line < editor.document.lineCount; line++) {
    const column = editor.document.lineAt(line).text.indexOf(text);
    if (column === -1) continue;
    const at = new vscode.Position(line, column);
    editor.selection = new vscode.Selection(at, at);
    return at;
  }
  throw new Error(`${editor.document.uri.fsPath} has no line with ${text}`);
}

async function running(api: AscribeApi, folder: string): Promise<void> {
  await waitFor("the language server", () => api.state(folder) === "running");
  await api.whenSettled();
}

/** The status bar item's text once it satisfies `check`. */
const status = (api: AscribeApi, what: string, check: (text: string) => boolean) =>
  waitFor(what, () => {
    const text = api.ui.statusBar.shown()?.text;
    return text !== undefined && check(text);
  }).catch((error: unknown) => {
    throw new Error(`${String(error)}; the status bar says ${api.ui.statusBar.shown()?.text}`);
  });

export const SCENES: Record<string, Step[]> = {
  // examples/platforms' page with the lens on for the Windows build, the
  // preview beside it, and the hover over a dimmed arm.
  "build-lens": [
    async (api, folder) => {
      await running(api, folder);
      await open(folder, "docs/install.md");
      await vscode.commands.executeCommand("ascribe.openPreview");
      await api.preview.whenDrawn("the page", (r) => r.result.page?.path === "install.md");
      api.preview.selectBuild("windows");
      await api.preview.whenDrawn(
        "the Windows build",
        (r) => api.preview.build() === "windows" && r.result.page?.path === "install.md",
      );
      const editor = await open(folder, "docs/install.md");
      await vscode.commands.executeCommand("ascribe.toggleBuildLens");
      await waitFor("the dimmed arms", () => {
        const shown = api.ui.lens.shown(editor.document.uri);
        return shown?.build === "windows" && shown.dimmed.length > 0;
      });
      await status(api, "the eye", (text) => text.startsWith("$(eye) "));
      await vscode.commands.executeCommand("workbench.action.closeSidebar");
      cursorAt(editor, "brew install");
      await vscode.commands.executeCommand("editor.action.showHover");
    },
  ],

  // examples/quill's quickstart with the cursor in a paragraph, and the bar open.
  "actions-bar": [
    async (api, folder) => {
      await running(api, folder);
      const editor = await open(folder, "docs/quickstart.md");
      await vscode.commands.executeCommand("workbench.action.closeSidebar");
      cursorAt(editor, "paste a page");
      void vscode.commands.executeCommand("ascribe.actions");
    },
  ],

  // examples/quill on a branch with a pull request on a fake GitHub: review
  // on, the Review view with its badge for an unsent comment, and the page's
  // preview with the thread and the comment.
  "review-view": [
    async (api, folder) => {
      await running(api, folder);
      const git = (...args: string[]) =>
        execFileSync("git", args, { cwd: folder, encoding: "utf8" }).trim();
      const github = new StatefulGitHub({
        number: 128,
        baseOid: git("rev-parse", "main"),
        headOid: git("rev-parse", "HEAD"),
        files: ["docs/install-agent.md"],
      });
      github.addThread("docs/install-agent.md", 8, "Which changes does it sync?");
      api.preview.review.threads.useTransport(() => github);
      await open(folder, "docs/install-agent.md");
      const started = await api.preview.review.start(folder);
      if (started.problem !== null) throw new Error(`review didn't start: ${started.problem}`);
      await vscode.commands.executeCommand("ascribe.openPagePreview");
      await api.preview.whenDrawn("the page", (r) => r.result.page?.path === "install-agent.md");
      await api.preview.receive({
        type: "threads",
        id: 1,
        method: "comment",
        params: {
          anchor: { source: "install-agent.md:12-12", via: [] },
          body: "Say which repository.",
        },
      } as FromWebview);
      await vscode.commands.executeCommand("ascribe.refreshComments");
      await waitFor("the badge", async () => {
        await api.ui.review.whenSettled();
        return api.ui.review.badge();
      });
      await vscode.commands.executeCommand("ascribe.review.focus");
      await waitFor("the threads drawn", () =>
        api.preview.threadsDrawn().some((r) => Object.values(r.blocks).flat().length > 1),
      );
    },
  ],

  // A copy of examples/tested whose example three pages show, and the lens's
  // references peek open.
  "shown-in-pages": [
    async (api, folder) => {
      await running(api, folder);
      const greet = vscode.Uri.file(path.join(folder, "code", "examples", "greet.ts"));
      await vscode.window.showTextDocument(greet);
      await vscode.commands.executeCommand("workbench.action.closeSidebar");
      const lens = await waitFor("the lens", async () => {
        const found = await vscode.commands.executeCommand<vscode.CodeLens[]>(
          "vscode.executeCodeLensProvider",
          greet,
        );
        return found.find((l) => l.command?.title === "Shown in 3 pages");
      });
      const command = lens.command as vscode.Command;
      await vscode.commands.executeCommand(command.command, ...(command.arguments ?? []));
    },
  ],

  // A workspace with a README and a copy of examples/platforms in docs/,
  // whose `ascribe` waits for `go` beside it and fails once `fail` is there:
  // the status bar item before the project is checked, while its server
  // starts, running, with the lens on, and failed.
  "status-bar": [
    async (api) => {
      const folder = root();
      await open(folder, "README.md");
      await status(api, "not checked yet", (text) => text.endsWith("docs: not checked yet"));
    },
    async (api) => {
      await open(root(), "docs/docs/install.md");
      await status(api, "the server starting", (text) => text.startsWith("$(sync~spin) "));
    },
    async (api) => {
      const folder = path.join(root(), "docs");
      writeFileSync(path.join(root(), "go"), "");
      await running(api, folder);
      await status(api, "the problems' build", (text) =>
        text.endsWith("problems: this build only"),
      );
    },
    async (api) => {
      // The preview's picker switches the build the lens dims by.
      await vscode.commands.executeCommand("ascribe.openPreview");
      await api.preview.whenDrawn("the page", (r) => r.result.page?.path === "install.md");
      api.preview.selectBuild("windows");
      await api.preview.whenDrawn("the Windows build", () => api.preview.build() === "windows");
      await open(root(), "docs/docs/install.md");
      await vscode.commands.executeCommand("ascribe.toggleBuildLens");
      await status(api, "the eye", (text) => text.startsWith("$(eye) "));
    },
    async (api) => {
      await vscode.commands.executeCommand("ascribe.toggleBuildLens");
      writeFileSync(path.join(root(), "fail"), "");
      await vscode.commands.executeCommand("ascribe.restartServer");
      await status(api, "the warning", (text) => text.startsWith("$(warning) "));
    },
  ],
};

const root = (): string => {
  const folder = vscode.workspace.workspaceFolders?.[0];
  if (!folder) throw new Error("no folder is open");
  return folder.uri.fsPath;
};

/** Run by VS Code as its tests' module: stages the scene, a step at a time. */
export async function run(): Promise<void> {
  const name = process.env["ASCRIBE_SCENE"] ?? "";
  const signals = process.env["ASCRIBE_SCENE_SIGNALS"] ?? "";
  const steps = SCENES[name];
  if (!steps) throw new Error(`no scene named ${name}`);
  const extension = vscode.extensions.getExtension<AscribeApi>("Ascribe.ascribe-vscode");
  if (!extension) throw new Error("the extension isn't installed");
  const api = extension.isActive ? extension.exports : await extension.activate();
  // VS Code's warning against running as root, in a container, comes after the window.
  await sleep(3000);
  for (const [n, step] of steps.entries()) {
    // What went wrong, for screenshot.ts to say: VS Code's own output isn't shown.
    await step(api, root()).catch((error: unknown) => {
      writeFileSync(path.join(signals, "failed"), `step ${n}: ${String(error)}`);
      throw error;
    });
    // Such as VS Code's own warning against running as root, in a container.
    await vscode.commands.executeCommand("notifications.clearAll");
    writeFileSync(path.join(signals, `staged-${n}`), "");
    await waitFor(`picture ${n}`, () => existsSync(path.join(signals, `taken-${n}`)));
  }
}
