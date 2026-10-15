// New Page and Go to Next Placeholder. New Page isn't about the cursor: it's
// offered in the palette, on a folder in the Explorer, and in the Pages
// view. It asks for a type (from `ascribe/targets`, only those a page in
// the folder can be of) and a title, then the server's `newPage` edit
// creates the page from the type's template; the page is saved and opened
// with the first thing to fill in selected. Go to Next Placeholder selects
// the next thing the template left to fill in, from `nextPlaceholder`.

import * as vscode from "vscode";
import type { ProjectServer } from "../client.js";
import type { ProjectRegistry } from "../registry.js";
import type { TargetsResult } from "../shapes.js";
import { QuickInputPrompter } from "./ask.js";
import { whereOf } from "./run.js";
import { newPageWizard, runWizard, ScriptedPrompter, type Scripted } from "./steps.js";

/** What a run of New Page did, for tests. */
export interface NewPageRecord {
  /** The page it made, relative to the workspace. */
  file?: string;
  /** The text it left selected in the page. */
  selected?: string;
  /** What it told the writer, if anything. */
  messages: string[];
}

/** New Page, for tests. */
export interface NewPageApi {
  /** Answers the next New Page's questions from a script instead of the quick input. */
  answerNext(answers: Scripted[]): void;
  /** What each run did, oldest first. */
  readonly runs: readonly NewPageRecord[];
}

const TITLE = "New Page";

export class NewPage implements vscode.Disposable {
  private script: Scripted[] | undefined;
  private readonly runs: NewPageRecord[] = [];
  private readonly disposables: vscode.Disposable[] = [];

  constructor(private readonly projects: ProjectRegistry) {}

  readonly api: NewPageApi = {
    answerNext: (answers) => {
      this.script = [...answers];
    },
    runs: this.runs,
  };

  register(): void {
    this.disposables.push(
      // From the Explorer, the folder it was run on; from the palette or the
      // Pages view, nothing.
      vscode.commands.registerCommand("ascribe.newPage", async (folder?: unknown) => {
        this.runs.push(await this.newPage(folder instanceof vscode.Uri ? folder : undefined));
      }),
      vscode.commands.registerCommand("ascribe.nextPlaceholder", () => this.nextPlaceholder()),
    );
  }

  dispose(): void {
    for (const disposable of this.disposables.splice(0)) disposable.dispose();
  }

  private async newPage(folder: vscode.Uri | undefined): Promise<NewPageRecord> {
    const record: NewPageRecord = { messages: [] };
    const say = (message: string, warn = false): void => {
      record.messages.push(message);
      const text = `Ascribe: ${message}`;
      void (warn
        ? vscode.window.showWarningMessage(text)
        : vscode.window.showInformationMessage(text));
    };
    const script = this.script;
    this.script = undefined;

    const server = folder ? this.projects.serverFor(folder) : this.projects.current();
    if (!server) {
      say(
        folder
          ? "This folder isn't in an Ascribe project."
          : "Open a folder with an ascribe.toml to start a page.",
      );
      return record;
    }
    await server.start();
    if (server.state !== "running") {
      say("The project's language server isn't running. Restart it, then try again.", true);
      return record;
    }
    // The request is about the project, so it's asked through ascribe.toml.
    const model = { uri: vscode.Uri.file(server.project.config).toString() };
    try {
      const targets = (await server.request("ascribe/targets", {
        textDocument: model,
        kinds: ["types"],
        ...(folder ? { folder: folder.toString() } : {}),
      })) as TargetsResult;
      const asked = await runWizard(
        TITLE,
        newPageWizard(targets.types ?? [], folder !== undefined),
        script ? new ScriptedPrompter(script) : new QuickInputPrompter(),
      );
      if (!asked) return record;
      if ("error" in asked) {
        say(asked.error);
        return record;
      }
      const at = { line: 0, character: 0 };
      const result = await server.requestEdit("ascribe/edit", {
        textDocument: model,
        range: { start: at, end: at },
        action: "newPage",
        args: { ...asked.args, ...(folder ? { folder: folder.toString() } : {}) },
      });
      if ("error" in result) {
        say(result.error, true);
        return record;
      }
      const page = result.edit.entries()[0]?.[0];
      if (!page || !(await vscode.workspace.applyEdit(result.edit, { isRefactoring: true }))) {
        say("The page couldn't be made. Try again.", true);
        return record;
      }
      const document = await vscode.workspace.openTextDocument(page);
      if (document.isDirty) await document.save();
      const editor = await vscode.window.showTextDocument(document);
      record.file = vscode.workspace.asRelativePath(page, false);
      if (result.select) {
        editor.selection = new vscode.Selection(result.select.start, result.select.end);
        editor.revealRange(result.select);
        record.selected = document.getText(result.select);
      }
      return record;
    } catch (error) {
      server.reportFeatureError(`"${TITLE}"`, error);
      return record;
    }
  }

  private async nextPlaceholder(): Promise<void> {
    const editor = vscode.window.activeTextEditor;
    const document = editor?.document;
    const server: ProjectServer | undefined =
      document && (await this.projects.ensureStartedFor(document.uri));
    if (!editor || !document || document.languageId !== "markdown" || !server) {
      void vscode.window.showInformationMessage(
        "Ascribe: open a page of an Ascribe project to go to its next placeholder.",
      );
      return;
    }
    if (server.state !== "running") {
      void vscode.window.showWarningMessage(
        "Ascribe: the project's language server isn't running. Restart it, then try again.",
      );
      return;
    }
    const where = whereOf(editor);
    try {
      const result = await server.requestEdit("ascribe/edit", {
        textDocument: { uri: where.uri },
        range: where.range,
        action: "nextPlaceholder",
        args: {},
        version: where.version,
      });
      if ("error" in result) {
        void vscode.window.showInformationMessage(`Ascribe: ${result.error}`);
        return;
      }
      if (result.select && vscode.window.activeTextEditor === editor) {
        editor.selection = new vscode.Selection(result.select.start, result.select.end);
        editor.revealRange(result.select);
      }
    } catch (error) {
      server.reportFeatureError('"Go to Next Placeholder"', error);
    }
  }
}
