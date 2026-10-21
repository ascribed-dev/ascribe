// Apply Every Safe Fix and Format Every Page: one edit each to the whole
// project, which the language server makes (`ascribe/projectEdit`) and the
// extension applies, from the Project view or the Command Palette.
//
// The server makes the edit first, so the extension can ask with its counts
// ("Apply 6 safe fixes in 4 files?") before applying it. The edit is a
// workspace edit, as one page's safe fixes and formatting are: the files
// change unsaved, Undo puts them back, and an open file's unsaved text is
// what's edited, never the disk under it. On a large project that can be
// hundreds of files, so they're left unsaved on purpose, to read before
// they're saved. The safe fixes are the editor build's, the problems the
// editor shows; every build's are `ascribe check --fix`'s (decision #544e).

import * as vscode from "vscode";
import type { ProjectServer } from "../client.js";
import type { ProjectRegistry } from "../registry.js";
import { STATE_NAMES } from "./describe.js";
import {
  nothingToEdit,
  PROJECT_EDITS,
  projectEditQuestion,
  type ProjectEditKind,
} from "./projectHealth.js";

export const APPLY_EVERY_SAFE_FIX_COMMAND = "ascribe.applyEverySafeFix";
export const FORMAT_EVERY_PAGE_COMMAND = "ascribe.formatEveryPage";

/** Asks the question before an edit is applied, and says whether to apply it. */
export type Confirm = (question: string, kind: ProjectEditKind) => Promise<boolean>;

/** What a project edit came to, for tests. */
export interface ProjectEditOutcome {
  /** What it asked before applying, naming the counts; `undefined` when it didn't ask. */
  question?: string;
  /** Whether the edit was applied. */
  applied: boolean;
  /** What it said instead of asking, or after: nothing to do, or why it couldn't. */
  said?: string;
}

/** The project edits, for tests. */
export interface ProjectEditsApi {
  /**
   * Makes an edit to the project in `folder`, as the Project view's action
   * does, answering its question with `confirm`.
   */
  run(folder: string, kind: ProjectEditKind, confirm: Confirm): Promise<ProjectEditOutcome>;
}

export class ProjectEdits implements vscode.Disposable {
  private readonly disposables: vscode.Disposable[] = [];

  constructor(private readonly projects: ProjectRegistry) {}

  get api(): ProjectEditsApi {
    return {
      run: (folder, kind, confirm) => {
        const server = this.projects.serverAt(folder);
        if (!server) return Promise.reject(new Error(`no project in ${folder}`));
        return this.run(server, kind, confirm);
      },
    };
  }

  register(): void {
    this.disposables.push(
      vscode.commands.registerCommand(APPLY_EVERY_SAFE_FIX_COMMAND, () =>
        this.fromPalette("fixAll"),
      ),
      vscode.commands.registerCommand(FORMAT_EVERY_PAGE_COMMAND, () => this.fromPalette("format")),
    );
  }

  dispose(): void {
    for (const disposable of this.disposables.splice(0)) disposable.dispose();
  }

  /**
   * Makes an edit to `server`'s project: asks the server for it, says how
   * much it changes, and applies it, unsaved, when the answer is yes.
   */
  async run(
    server: ProjectServer,
    kind: ProjectEditKind,
    confirm: Confirm = ask,
  ): Promise<ProjectEditOutcome> {
    const name = this.projects.name(server.project);
    const say = (said: string, error = false): ProjectEditOutcome => {
      if (error) void vscode.window.showErrorMessage(said);
      else void vscode.window.showInformationMessage(said);
      return { applied: false, said };
    };
    await server.start();
    await server.whenSettled();
    if (server.state !== "running") {
      return say(
        `Ascribe couldn't reach the language server of ${name}. Show Output says why.`,
        true,
      );
    }
    let answer;
    try {
      answer = await vscode.window.withProgress(
        {
          location: vscode.ProgressLocation.Notification,
          title:
            kind === "fixAll"
              ? `Ascribe: finding the safe fixes in ${name}`
              : `Ascribe: formatting ${name}`,
        },
        () => server.requestProjectEdit(kind),
      );
    } catch (error) {
      const message = error instanceof Error ? error.message : String(error);
      server.log(`${PROJECT_EDITS[kind].label} failed: ${message}`);
      return say(`Ascribe couldn't make the edit to ${name}: ${message}`, true);
    }
    if ("error" in answer) return say(`Ascribe: ${answer.error}`, true);
    if (answer.files === 0) return say(nothingToEdit(kind, name));

    const question = projectEditQuestion(answer.files, answer.fixes);
    if (!(await confirm(question, kind))) return { question, applied: false };
    // The answer is for the documents as they were: one that changed while
    // the question was up would get edits meant for other text.
    if (changedSince(answer.versions)) {
      return {
        question,
        ...say("Ascribe: a file changed after the edit was made. Choose it again.", true),
      };
    }
    const applied = await vscode.workspace.applyEdit(answer.edit);
    if (!applied) {
      return { question, ...say(`Ascribe couldn't apply the edit to ${name}.`, true) };
    }
    return { question, applied };
  }

  /** The command: for the active file's project, or the one chosen when no file of a project is active. */
  private async fromPalette(kind: ProjectEditKind): Promise<void> {
    const active = vscode.window.activeTextEditor?.document.uri;
    const server = (active && this.projects.serverFor(active)) ?? (await this.pick(kind));
    if (!server) {
      if (this.projects.servers.length === 0) {
        void vscode.window.showInformationMessage(
          "Ascribe: there's no Ascribe project in this workspace.",
        );
      }
      return;
    }
    await this.run(server, kind);
  }

  private async pick(kind: ProjectEditKind): Promise<ProjectServer | undefined> {
    const servers = this.projects.servers;
    if (servers.length <= 1) return servers[0];
    const picked = await vscode.window.showQuickPick(
      servers.map((server) => ({
        label: this.projects.name(server.project),
        description: STATE_NAMES[server.state],
        server,
      })),
      { title: PROJECT_EDITS[kind].label, placeHolder: "The project to edit" },
    );
    return picked?.server;
  }
}

/** Asks, in a modal dialog, so nothing is typed while it's up. */
async function ask(question: string, kind: ProjectEditKind): Promise<boolean> {
  const yes = kind === "fixAll" ? "Apply" : "Format";
  const detail =
    kind === "fixAll"
      ? "The fixes are for the problems the editor shows, in the editor's build. Every build's are ascribe check --fix's."
      : "Pages are formatted as ascribe fmt formats them.";
  return (
    (await vscode.window.showInformationMessage(question, { modal: true, detail }, yes)) === yes
  );
}

/**
 * Whether a document changed since the edit was made: an open one whose
 * version isn't the one its edit is for, or one that wasn't open then and
 * now has unsaved changes.
 */
function changedSince(versions: Map<string, number | null>): boolean {
  const open = new Map(vscode.workspace.textDocuments.map((d) => [d.uri.toString(), d]));
  for (const [uri, version] of versions) {
    const document = open.get(vscode.Uri.parse(uri).toString());
    if (!document) continue;
    if (version === null ? document.isDirty : document.version !== version) return true;
  }
  return false;
}
