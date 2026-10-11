import * as vscode from "vscode";
import { formatterOffer, otherFormatter } from "../formatterChoice.js";
import type { ProjectRegistry } from "../registry.js";

/** `workspaceState` key: the workspace folders whose writer kept their own formatter. */
const KEPT = "ascribe.formatter.kept";

/**
 * Offers to make Ascribe the Markdown formatter of a workspace folder when a
 * page of a project is open there and another extension is the formatter. It
 * asks once a session for each folder, and never again for a folder whose
 * writer kept their formatter.
 */
export class FormatterOffer implements vscode.Disposable {
  private readonly disposables: vscode.Disposable[] = [];
  /** The workspace folders asked about this session. */
  private readonly asked = new Set<string>();

  constructor(
    private readonly context: vscode.ExtensionContext,
    private readonly projects: ProjectRegistry,
  ) {}

  register(): void {
    const check = () => void this.check(vscode.window.activeTextEditor?.document);
    check();
    this.disposables.push(
      vscode.window.onDidChangeActiveTextEditor((editor) => void this.check(editor?.document)),
      vscode.workspace.onDidChangeConfiguration((event) => {
        if (event.affectsConfiguration("editor.defaultFormatter")) check();
      }),
      this.projects.onDidChangeProjects(check),
    );
  }

  dispose(): void {
    for (const disposable of this.disposables.splice(0)) disposable.dispose();
  }

  private async check(document: vscode.TextDocument | undefined): Promise<void> {
    if (
      document?.languageId !== "markdown" ||
      document.uri.scheme !== "file" ||
      this.projects.serverFor(document.uri) === undefined
    ) {
      return;
    }
    const folder = vscode.workspace.getWorkspaceFolder(document.uri);
    if (!folder) return;
    const key = folder.uri.toString();
    const kept = this.context.workspaceState.get<string[]>(KEPT, []);
    if (this.asked.has(key) || kept.includes(key)) return;
    const scope = { uri: document.uri, languageId: "markdown" };
    const other = otherFormatter(
      vscode.workspace.getConfiguration("editor", scope).get("defaultFormatter"),
      this.context.extension.id,
    );
    if (!other) return;
    this.asked.add(key);

    const extension = vscode.extensions.getExtension(other);
    const displayName: unknown = (extension?.packageJSON as { displayName?: unknown } | undefined)
      ?.displayName;
    const name = typeof displayName === "string" && displayName !== "" ? displayName : other;
    const use = "Use Ascribe";
    const keep = `Keep ${name}`;
    const picked = await vscode.window.showWarningMessage(formatterOffer(name), use, keep);
    if (picked === use) {
      try {
        await vscode.workspace
          .getConfiguration("editor", { uri: folder.uri, languageId: "markdown" })
          .update(
            "defaultFormatter",
            this.context.extension.id,
            vscode.ConfigurationTarget.WorkspaceFolder,
            true,
          );
      } catch (error) {
        void vscode.window.showErrorMessage(
          `Ascribe: couldn't make Ascribe the Markdown formatter for ${folder.name}: ${error instanceof Error ? error.message : String(error)}`,
        );
      }
    } else if (picked === keep) {
      await this.context.workspaceState.update(KEPT, [...kept, key]);
    }
  }
}
