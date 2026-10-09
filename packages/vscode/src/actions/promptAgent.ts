// **Prompt agent**: a code action on each Ascribe problem, after its quick
// fixes, and palette commands for the active file's problems and the
// project's. Each asks the project's server for the prompt
// (`ascribe/agentPrompt`) and delivers it as `ascribe.agents.promptTarget`
// says (`deliverPrompt`). The palette commands show only when there are
// problems: `ascribe.problems.file` and `ascribe.problems.project` say so.

import * as vscode from "vscode";
import type { ProjectServer } from "../client.js";
import type { ProjectRegistry } from "../registry.js";
import type { AgentPromptResult } from "../shapes.js";
import {
  PROMPT_COMMANDS,
  TARGET_NAMES,
  asTarget,
  availableTargets,
  codeOf,
  deliverPrompt,
  promptActionTitle,
  targetsToOffer,
  type Delivery,
  type DeliveryHost,
  type PromptRequest,
  type PromptTarget,
} from "./prompt.js";

/** The setting that says where a prompt goes. */
const SETTING = "agents.promptTarget";
/** Whether the targets have been offered: once, ever. */
const OFFERED = "ascribe.agents.targetsOffered";

const KIND = vscode.CodeActionKind.QuickFix.append("ascribe").append("agent");

/** Prompt agent, for tests. */
export interface PromptAgentApi {
  /** What each delivery did, oldest first. */
  readonly deliveries: readonly Delivery[];
}

export class PromptAgent implements vscode.Disposable {
  private readonly deliveries: Delivery[] = [];
  private readonly keys = new Map<string, boolean>();
  private readonly disposables: vscode.Disposable[] = [];
  private readonly host: DeliveryHost = {
    uriScheme: vscode.env.uriScheme,
    copy: (text) => Promise.resolve(vscode.env.clipboard.writeText(text)),
    status: (message) => {
      this.disposables.push(vscode.window.setStatusBarMessage(`Ascribe: ${message}`, 8_000));
    },
    notice: (message) => void vscode.window.showWarningMessage(`Ascribe: ${message}`),
    hasCommand: async (id) => (await vscode.commands.getCommands(true)).includes(id),
    hasExtension: (id) => vscode.extensions.getExtension(id) !== undefined,
    executeCommand: (id, argument) => Promise.resolve(vscode.commands.executeCommand(id, argument)),
    openLink: (link) => Promise.resolve(vscode.env.openExternal(vscode.Uri.from(link))),
  };

  constructor(
    private readonly context: vscode.ExtensionContext,
    private readonly projects: ProjectRegistry,
  ) {}

  readonly api: PromptAgentApi = { deliveries: this.deliveries };

  register(): void {
    this.disposables.push(
      vscode.commands.registerCommand(
        PROMPT_COMMANDS.problem,
        (uri: unknown, diagnostic: unknown) => this.promptProblem(uri, diagnostic),
      ),
      vscode.commands.registerCommand(PROMPT_COMMANDS.file, () => this.promptFile()),
      vscode.commands.registerCommand(PROMPT_COMMANDS.project, () => this.promptProject()),
      vscode.languages.registerCodeActionsProvider(
        { language: "markdown", scheme: "file" },
        { provideCodeActions: (document, _range, context) => this.actions(document, context) },
        { providedCodeActionKinds: [KIND] },
      ),
      vscode.languages.onDidChangeDiagnostics(() => this.setKeys()),
      vscode.window.onDidChangeActiveTextEditor(() => this.setKeys()),
      this.projects.onDidChangeState(() => this.setKeys()),
    );
    this.setKeys();
  }

  dispose(): void {
    for (const disposable of this.disposables.splice(0)) disposable.dispose();
  }

  /**
   * One action per Ascribe problem at the cursor. It has no diagnostics of its
   * own, so VS Code lists it after the quick fixes, which do.
   */
  private actions(
    document: vscode.TextDocument,
    context: vscode.CodeActionContext,
  ): vscode.CodeAction[] {
    if (context.only && !context.only.contains(KIND) && !KIND.contains(context.only)) return [];
    const server = this.projects.serverFor(document.uri);
    if (!server) return [];
    const problems = context.diagnostics.filter((d) => d.source === "ascribe");
    return problems.map((diagnostic) => {
      const title = promptActionTitle(codeOf(diagnostic.code), problems.length > 1);
      const action = new vscode.CodeAction(title, KIND);
      action.command = {
        command: PROMPT_COMMANDS.problem,
        title,
        // As the protocol has it, so the arguments stay plain data.
        arguments: [document.uri.toString(), server.protocolDiagnostic(diagnostic)],
      };
      return action;
    });
  }

  private async promptProblem(uri: unknown, diagnostic: unknown): Promise<void> {
    if (typeof uri !== "string" || typeof diagnostic !== "object" || diagnostic === null) return;
    const server = this.projects.serverFor(vscode.Uri.parse(uri));
    if (!server) return;
    await this.prompt(server, { kind: "problem", textDocument: { uri }, diagnostic, unsaved: [] });
  }

  private async promptFile(): Promise<void> {
    const document = vscode.window.activeTextEditor?.document;
    const server = document && this.projects.serverFor(document.uri);
    if (!document || !server) {
      void vscode.window.showInformationMessage("Ascribe: open a file of a project first.");
      return;
    }
    await this.prompt(server, {
      kind: "file",
      textDocument: { uri: document.uri.toString() },
      unsaved: [],
    });
  }

  private async promptProject(): Promise<void> {
    const server = this.projects.current();
    if (!server) {
      void vscode.window.showInformationMessage("Ascribe: this workspace has no ascribe.toml.");
      return;
    }
    await this.prompt(server, { kind: "project", unsaved: [] });
  }

  /** Asks the server for the prompt and delivers it; says so when there's no problem. */
  private async prompt(server: ProjectServer, request: PromptRequest): Promise<void> {
    const unsaved = vscode.workspace.textDocuments
      .filter((d) => d.isDirty && d.uri.scheme === "file")
      .map((d) => d.uri.toString());
    let answer: AgentPromptResult | null;
    try {
      answer = (await server.request("ascribe/agentPrompt", {
        ...request,
        unsaved,
      })) as AgentPromptResult | null;
    } catch (error) {
      server.log(`Building the agent prompt failed: ${String(error)}`);
      void vscode.window.showErrorMessage(
        "Ascribe: the agent prompt couldn't be built. The project's output has the details.",
      );
      return;
    }
    if (!answer) {
      void vscode.window.showInformationMessage("Ascribe: there's no problem to prompt about.");
      return;
    }
    const target = asTarget(vscode.workspace.getConfiguration("ascribe").get(SETTING));
    // Only a file's prompt is about one file; a project's names its unsaved files itself.
    const aboutUnsaved =
      request.textDocument !== undefined && unsaved.includes(request.textDocument.uri);
    const delivery = await deliverPrompt(answer.prompt, target, this.host, aboutUnsaved);
    this.deliveries.push(delivery);
    void this.offerTargets(target);
  }

  /** The first time, offers the targets this editor has besides the clipboard. Never again. */
  private async offerTargets(current: PromptTarget): Promise<void> {
    if (this.context.globalState.get<boolean>(OFFERED)) return;
    await this.context.globalState.update(OFFERED, true);
    const offered = targetsToOffer(await availableTargets(this.host), current);
    if (offered.length === 0) return;
    const names = offered.map((t) => TARGET_NAMES[t]);
    const picked = await vscode.window.showInformationMessage(
      `Ascribe: Prompt agent can also fill the prompt in for you, in ${names.join(" or ")}, without sending it.`,
      ...names,
    );
    const chosen = offered.find((t) => TARGET_NAMES[t] === picked);
    if (chosen) {
      await vscode.workspace
        .getConfiguration("ascribe")
        .update(SETTING, chosen, vscode.ConfigurationTarget.Global);
    }
  }

  /** Whether the active file's project, and the active file, have problems. */
  private setKeys(): void {
    const uri = vscode.window.activeTextEditor?.document.uri;
    const server = this.projects.current();
    const running = server?.state === "running";
    const set = (key: string, value: boolean): void => {
      if (this.keys.get(key) === value) return;
      this.keys.set(key, value);
      void vscode.commands.executeCommand("setContext", key, value);
    };
    const fileServer = uri && this.projects.serverFor(uri);
    set(
      "ascribe.problems.file",
      fileServer !== undefined && fileServer.state === "running" && fileServer.hasProblems(uri),
    );
    set("ascribe.problems.project", running && server.hasProblems());
  }
}
