import * as vscode from "vscode";
import type { ProjectServer } from "../client.js";
import { samePath } from "../projects.js";
import type { ProjectRegistry } from "../registry.js";
import type { BuildViewResult } from "../shapes.js";
import type { ChosenBuilds } from "./chosenBuild.js";
import { lensView, type BuildLenses } from "./lens.js";
import type { ProjectBuilds } from "./projectBuilds.js";

export const TOGGLE_LENS_COMMAND = "ascribe.toggleBuildLens";

/** How long after the last edit to a page the lens asks again. */
const EDIT_DEBOUNCE_MS = 300;

/** What the lens shows in an editor, for tests. */
export interface LensShown {
  /** The build it dims by. */
  build: string;
  /** Each dimmed range, as `line:character-line:character`, with its hover. */
  dimmed: { range: string; hover: string }[];
  /** The line at the top of the page, when the build leaves out the whole page. */
  banner: string | undefined;
  /** The document version it's for. */
  version: number;
}

/** The build lens, for tests. */
export interface BuildLensApi {
  /** Whether the lens is on in the project of `folder`. */
  isOn(folder: string): boolean;
  /** What the lens shows on the page at `uri`; `undefined` when it shows nothing there. */
  shown(uri: vscode.Uri): LensShown | undefined;
  /** Settles when every update under way, or waiting for typing to stop, is done. */
  whenSettled(): Promise<void>;
}

/**
 * The build lens: the editor dims what the build you're looking at leaves
 * out of each visible page of a project whose lens is on. It asks the
 * project's server (`ascribe/buildView`) for the project's chosen build, the
 * one the preview renders, again on edits, when the choice or the content
 * model changes, and when an editor is shown. Dimming is a decoration: it
 * changes nothing in the text, its folding, or its diagnostics.
 */
export class BuildLens implements vscode.Disposable {
  private readonly dim = vscode.window.createTextEditorDecorationType({ opacity: "0.45" });
  private readonly banner = vscode.window.createTextEditorDecorationType({
    after: {
      color: new vscode.ThemeColor("editorCodeLens.foreground"),
      fontStyle: "italic",
      margin: "0 0 0 2em",
    },
  });
  /** What each page shows now, by URI. */
  private readonly shownBy = new Map<string, LensShown>();
  /** The latest request for each page, so an older answer is dropped. */
  private readonly asked = new Map<string, number>();
  private readonly timers = new Map<string, ReturnType<typeof setTimeout>>();
  private readonly pending = new Set<Promise<void>>();
  private sequence = 0;
  private readonly disposables: vscode.Disposable[] = [];

  constructor(
    private readonly projects: ProjectRegistry,
    private readonly builds: ProjectBuilds,
    private readonly chosen: ChosenBuilds,
    private readonly lenses: BuildLenses,
  ) {}

  get api(): BuildLensApi {
    return {
      isOn: (folder) => this.lenses.isOn(folder),
      shown: (uri) => this.shownBy.get(uri.toString()),
      whenSettled: () => this.whenSettled(),
    };
  }

  register(): void {
    this.disposables.push(
      this.dim,
      this.banner,
      vscode.commands.registerCommand(TOGGLE_LENS_COMMAND, () => this.toggle()),
      vscode.window.onDidChangeVisibleTextEditors(() => this.updateAll()),
      vscode.workspace.onDidChangeTextDocument(({ document }) => {
        if (document.languageId === "markdown" && this.visible(document)) this.updateSoon(document);
      }),
      // The project's choice of build, from the status bar or the preview's picker.
      this.chosen.onDidChange((folder) => this.updateProject(folder)),
      this.lenses.onDidChange((folder) => this.updateProject(folder)),
      // A server started or stopped, or its builds were read again after the
      // content model changed.
      this.builds.onDidChange(() => this.updateAll()),
    );
  }

  dispose(): void {
    for (const timer of this.timers.values()) clearTimeout(timer);
    this.timers.clear();
    for (const disposable of this.disposables.splice(0)) disposable.dispose();
  }

  /** Turns the lens on or off for the active file's project. */
  private toggle(): void {
    const uri = vscode.window.activeTextEditor?.document.uri;
    const server = uri && this.projects.serverFor(uri);
    if (!server) {
      void vscode.window.showInformationMessage(
        "Ascribe: the active file isn't part of an Ascribe project.",
      );
      return;
    }
    this.lenses.toggle(server.project.folder);
  }

  private visible(document: vscode.TextDocument): boolean {
    return vscode.window.visibleTextEditors.some((editor) => editor.document === document);
  }

  private documents(): vscode.TextDocument[] {
    return [...new Set(vscode.window.visibleTextEditors.map((editor) => editor.document))];
  }

  private updateAll(): void {
    for (const document of this.documents()) this.update(document);
    // A page no longer visible keeps nothing.
    const visible = new Set(this.documents().map((d) => d.uri.toString()));
    for (const uri of this.shownBy.keys()) if (!visible.has(uri)) this.shownBy.delete(uri);
  }

  private updateProject(folder: string): void {
    for (const document of this.documents()) {
      const owner = this.projects.serverFor(document.uri)?.project.folder;
      if (owner !== undefined && samePath(owner, folder)) this.update(document);
    }
  }

  private updateSoon(document: vscode.TextDocument): void {
    const key = document.uri.toString();
    clearTimeout(this.timers.get(key));
    this.timers.set(
      key,
      setTimeout(() => {
        this.timers.delete(key);
        this.update(document);
      }, EDIT_DEBOUNCE_MS),
    );
  }

  /** The server whose lens covers a page, when it's on and the server is running. */
  private serverFor(document: vscode.TextDocument): ProjectServer | undefined {
    if (document.languageId !== "markdown" || document.uri.scheme !== "file") return undefined;
    const server = this.projects.serverFor(document.uri);
    if (!server || server.state !== "running") return undefined;
    return this.lenses.isOn(server.project.folder) ? server : undefined;
  }

  private update(document: vscode.TextDocument): void {
    const key = document.uri.toString();
    const server = this.serverFor(document);
    if (!server) {
      this.asked.delete(key);
      this.show(document, undefined);
      return;
    }
    const seq = ++this.sequence;
    this.asked.set(key, seq);
    const folder = server.project.folder;
    const build = this.chosen.get(folder);
    const params: Record<string, unknown> = { textDocument: { uri: key } };
    if (build !== undefined) params.build = build;
    const asking = (async () => {
      let result: BuildViewResult;
      try {
        result = (await server.request("ascribe/buildView", params)) as BuildViewResult;
      } catch {
        return;
      }
      if (this.asked.get(key) !== seq) return;
      // A build the content model no longer has: back to the editor's, which
      // asks again through the choice's change.
      if (result.build === "" && build !== undefined) {
        this.chosen.set(folder, undefined);
        return;
      }
      // An answer for an older text: the edit that changed it asks again.
      if (result.documentVersion !== null && result.documentVersion !== document.version) return;
      this.show(document, result);
    })();
    this.pending.add(asking);
    void asking.finally(() => this.pending.delete(asking));
  }

  /** Draws an answer in every editor showing the page, or clears them. */
  private show(document: vscode.TextDocument, result: BuildViewResult | undefined): void {
    const view = result && lensView(result, document.lineCount);
    const dimmed: vscode.DecorationOptions[] = (view?.dimmed ?? []).map((d) => ({
      range: new vscode.Range(
        d.range.start.line,
        d.range.start.character,
        d.range.end.line,
        d.range.end.character,
      ),
      hoverMessage: new vscode.MarkdownString(d.hover),
    }));
    const banner: vscode.DecorationOptions[] =
      view?.banner === undefined
        ? []
        : [
            {
              range: document.lineAt(0).range,
              renderOptions: { after: { contentText: view.banner } },
            },
          ];
    for (const editor of vscode.window.visibleTextEditors) {
      if (editor.document !== document) continue;
      editor.setDecorations(this.dim, dimmed);
      editor.setDecorations(this.banner, banner);
    }
    const key = document.uri.toString();
    if (!result || !view) {
      this.shownBy.delete(key);
      return;
    }
    this.shownBy.set(key, {
      build: result.build,
      dimmed: view.dimmed.map((d) => ({
        range: `${d.range.start.line}:${d.range.start.character}-${d.range.end.line}:${d.range.end.character}`,
        hover: d.hover,
      })),
      banner: view.banner,
      version: document.version,
    });
  }

  private async whenSettled(): Promise<void> {
    for (;;) {
      if (this.timers.size > 0) {
        await new Promise((resolve) => setTimeout(resolve, EDIT_DEBOUNCE_MS));
        continue;
      }
      if (this.pending.size === 0) return;
      await Promise.all(this.pending);
    }
  }
}
