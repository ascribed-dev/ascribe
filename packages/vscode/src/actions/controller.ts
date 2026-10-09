// Offers the registry's actions: a command for each, the context keys the
// context menu's `when` clauses read, and the lightbulb. Follows the active
// editor: `ascribe.inProject` says whether its file belongs to a project, and
// its selection's `ascribe/context` (asked for once the selection settles)
// sets the other keys.

import * as vscode from "vscode";
import type { ProjectRegistry } from "../registry.js";
import type { ContextResult } from "../shapes.js";
import { QuickInputPrompter } from "./ask.js";
import { ContextCache, type Where } from "./context.js";
import { ACTIONS, commandId, contextKeys, lightbulbActions, type Action } from "./registry.js";
import { runAction, whereOf, type RunRecord } from "./run.js";
import { ScriptedPrompter, type Prompter, type Scripted } from "./steps.js";

/** The actions, for tests. */
export interface ActionsApi {
  /** Whether the active editor's file belongs to a project: the `ascribe.inProject` key. */
  inProject(): boolean;
  /** Answers the next action's questions from a script instead of the quick input. */
  answerNext(answers: Scripted[]): void;
  /** What each action run did, oldest first. */
  readonly runs: readonly RunRecord[];
}

/** The answer for a file that isn't a page: every key false. */
const NOWHERE: ContextResult = {
  project: null,
  at: [],
  selection: null,
  token: null,
  insertable: false,
};

const REFACTOR = vscode.CodeActionKind.RefactorRewrite.append("ascribe");
const QUICKFIX = vscode.CodeActionKind.QuickFix.append("ascribe");

export class ActionsController implements vscode.Disposable {
  private readonly cache: ContextCache;
  /** The context keys as last set, to set only those that change. */
  private readonly keys = new Map<string, boolean>();
  private inProject = false;
  /** The document the cache's answers are for. */
  private followed: string | undefined;
  private script: Scripted[] | undefined;
  private readonly runs: RunRecord[] = [];
  private readonly disposables: vscode.Disposable[] = [];

  constructor(private readonly projects: ProjectRegistry) {
    this.cache = new ContextCache(
      (where, signal) => this.sendContext(where, signal),
      (_where, result) => this.setKeys(result ?? NOWHERE),
      (error) =>
        this.projects
          .current()
          ?.log(`Asking for the context at the cursor failed: ${String(error)}`),
    );
  }

  readonly api: ActionsApi = {
    inProject: () => this.inProject,
    answerNext: (answers) => {
      this.script = [...answers];
    },
    runs: this.runs,
  };

  register(): void {
    for (const action of ACTIONS) {
      this.disposables.push(
        vscode.commands.registerCommand(commandId(action), () => this.run(action)),
      );
    }
    this.disposables.push(
      vscode.languages.registerCodeActionsProvider(
        { language: "markdown", scheme: "file" },
        {
          provideCodeActions: (document, range, context) =>
            this.lightbulb(document, range, context),
        },
        { providedCodeActionKinds: [REFACTOR, QUICKFIX] },
      ),
      vscode.window.onDidChangeActiveTextEditor(() => this.follow()),
      vscode.window.onDidChangeTextEditorSelection((event) => {
        if (event.textEditor === vscode.window.activeTextEditor) this.follow();
      }),
      vscode.workspace.onDidChangeTextDocument((event) => {
        if (event.document === vscode.window.activeTextEditor?.document) this.follow();
      }),
      this.projects.onDidChangeProjects(() => this.follow()),
      this.projects.onDidStart(() => this.follow()),
    );
    this.setKeys(NOWHERE);
    this.follow();
  }

  dispose(): void {
    this.cache.dispose();
    for (const disposable of this.disposables.splice(0)) disposable.dispose();
  }

  private async run(action: Action): Promise<void> {
    const script = this.script;
    this.script = undefined;
    const prompter: Prompter = script ? new ScriptedPrompter(script) : new QuickInputPrompter();
    this.runs.push(
      await runAction(action, {
        projects: this.projects,
        cache: this.cache,
        prompter: () => prompter,
      }),
    );
  }

  /** Follows the active editor: whether it's in a project, and its selection's context. */
  private follow(): void {
    const editor = vscode.window.activeTextEditor;
    const document = editor?.document;
    const server =
      document?.uri.scheme === "file" ? this.projects.serverFor(document.uri) : undefined;
    const inProject = server !== undefined;
    if (inProject !== this.inProject) {
      this.inProject = inProject;
      void vscode.commands.executeCommand("setContext", "ascribe.inProject", inProject);
    }
    const uri = document?.uri.toString();
    if (uri !== this.followed) {
      this.followed = uri;
      this.cache.clear();
    }
    if (
      !editor ||
      !server ||
      server.state !== "running" ||
      editor.document.languageId !== "markdown"
    ) {
      return;
    }
    this.cache.schedule(whereOf(editor));
  }

  private sendContext(where: Where, signal: AbortSignal): Promise<ContextResult> {
    const server = this.projects.serverFor(vscode.Uri.parse(where.uri));
    if (!server) return Promise.reject(new Error("the file belongs to no project"));
    const cancel = new vscode.CancellationTokenSource();
    signal.addEventListener("abort", () => cancel.cancel());
    return server
      .request(
        "ascribe/context",
        { textDocument: { uri: where.uri }, range: where.range },
        cancel.token,
      )
      .then((value) => value as ContextResult)
      .finally(() => cancel.dispose());
  }

  private setKeys(context: ContextResult): void {
    for (const [key, value] of Object.entries(contextKeys(context))) {
      if (this.keys.get(key) === value) continue;
      this.keys.set(key, value);
      void vscode.commands.executeCommand("setContext", key, value);
    }
  }

  /** The lightbulb's actions, from the cached context only: it never asks the server. */
  private lightbulb(
    document: vscode.TextDocument,
    range: vscode.Range | vscode.Selection,
    context: vscode.CodeActionContext,
  ): vscode.CodeAction[] {
    const cached = this.cache.get({
      uri: document.uri.toString(),
      version: document.version,
      range: {
        start: { line: range.start.line, character: range.start.character },
        end: { line: range.end.line, character: range.end.character },
      },
    });
    return lightbulbActions(cached).flatMap((action) => {
      const kind = action.lightbulb === "quickfix" ? QUICKFIX : REFACTOR;
      if (context.only && !context.only.contains(kind)) return [];
      const codeAction = new vscode.CodeAction(action.title, kind);
      codeAction.command = { command: commandId(action), title: action.title };
      return [codeAction];
    });
  }
}
