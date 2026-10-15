import * as vscode from "vscode";
import type { ProjectRegistry } from "../registry.js";
import { PAGE_STEPS } from "./walkthroughSteps.js";

/** Runs the walkthrough's page commands, following the last page the writer had open. */
export class Walkthrough implements vscode.Disposable {
  private readonly disposables: vscode.Disposable[] = [];
  /** The last page that was the active editor. */
  private last: vscode.Uri | undefined;

  constructor(private readonly projects: ProjectRegistry) {}

  register(): void {
    this.follow(vscode.window.activeTextEditor);
    this.disposables.push(
      vscode.window.onDidChangeActiveTextEditor((editor) => this.follow(editor)),
      ...Object.entries(PAGE_STEPS).map(([id, command]) =>
        vscode.commands.registerCommand(id, () => this.onPage(command)),
      ),
    );
  }

  dispose(): void {
    for (const disposable of this.disposables.splice(0)) disposable.dispose();
  }

  private follow(editor: vscode.TextEditor | undefined): void {
    const document = editor?.document;
    if (document?.languageId === "markdown" && this.isPage(document.uri)) this.last = document.uri;
  }

  private isPage(uri: vscode.Uri): boolean {
    return uri.scheme === "file" && this.projects.serverFor(uri) !== undefined;
  }

  /** Shows the page to act on, then runs `command` there. */
  private async onPage(command: string): Promise<void> {
    const page = this.page();
    if (!page) {
      void vscode.window.showInformationMessage(
        "Ascribe: open a page of your project first, such as from the Explorer, then try again.",
      );
      return;
    }
    if (vscode.window.activeTextEditor?.document.uri.toString() !== page.toString()) {
      // A page that's showing in another group is focused there; anything
      // else opens in the active group, in place of the Welcome page.
      const visible = vscode.window.visibleTextEditors.find(
        (editor) => editor.document.uri.toString() === page.toString(),
      );
      try {
        await vscode.window.showTextDocument(page, {
          viewColumn: visible?.viewColumn ?? vscode.ViewColumn.Active,
          preview: false,
        });
      } catch {
        // Deleted or moved since.
        this.last = undefined;
        void vscode.window.showInformationMessage(
          `Ascribe: ${vscode.workspace.asRelativePath(page)} can't be opened. Open a page of your project, then try again.`,
        );
        return;
      }
    }
    await vscode.commands.executeCommand(command);
  }

  /**
   * The active page, else the last page that was active, else a page open in
   * a tab since before the extension started, as one behind the Welcome page
   * is when VS Code restores a window.
   */
  private page(): vscode.Uri | undefined {
    const active = vscode.window.activeTextEditor?.document;
    if (active?.languageId === "markdown" && this.isPage(active.uri)) return active.uri;
    if (this.last) return this.last;
    const groups = vscode.window.tabGroups;
    for (const group of [groups.activeTabGroup, ...groups.all]) {
      for (const tab of group.tabs) {
        const uri = tab.input instanceof vscode.TabInputText ? tab.input.uri : undefined;
        if (uri && /\.md$/i.test(uri.path) && this.isPage(uri)) return uri;
      }
    }
    return undefined;
  }
}
