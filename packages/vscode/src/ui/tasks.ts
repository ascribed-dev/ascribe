// The `ascribe` task type's provider: Build, Sync Agent Files, and Observe
// Examples for each project, in **Tasks: Run Task** and the Project view, and
// any of them `tasks.json` names. What each runs is `../tasks.ts`'s.
//
// Each runs the binary the project's server runs, in the terminal, with no
// problem matcher: the Problems panel stays Check Every Build's and the
// editor's. Build and Sync Agent Files are plain processes. Observe Examples
// runs the commands a source's `ascribe.examples.toml` names, so it shows
// them and asks before each run, and runs `ascribe observe` itself in the
// terminal once it's answered. None is offered in Restricted Mode, where the
// extension starts nothing.

import { spawn, type ChildProcess } from "node:child_process";
import { readFile, stat } from "node:fs/promises";
import * as path from "node:path";
import * as vscode from "vscode";
import { resolveProjectBinary, workspaceFolderOf } from "../client.js";
import { shellCommand, usesShell } from "../environment.js";
import type { Project } from "../projects.js";
import type { ProjectRegistry } from "../registry.js";
import {
  definedProject,
  EXAMPLES_FILE,
  examplesCommands,
  isAscribeTask,
  localSources,
  observeCommand,
  observeQuestion,
  projectCommand,
  projectProperty,
  TASK_TYPE,
  taskLabel,
  type AscribeTaskDefinition,
  type ExamplesSource,
  type ObserveQuestion,
  type TaskCommand,
  type TaskName,
} from "../tasks.js";

const RUN_THEM = "Run Them";

/** A question Observe Examples asked, and its answer. */
export interface AskedRecord extends ObserveQuestion {
  answered: boolean;
}

/** A task the provider ran itself: Observe Examples, once it's answered. */
export interface ObserveRecord {
  cwd: string;
  args: string[];
  /** Its exit code, or `undefined` when it was declined or couldn't start. */
  exitCode: number | undefined;
}

/** The tasks, for tests. */
export interface TasksApi {
  /** Answers Observe Examples' next question with `run` instead of asking. */
  answerNext(run: boolean): void;
  /** Each question Observe Examples asked, oldest first. */
  readonly asked: readonly AskedRecord[];
  /** Each Observe Examples that ended, oldest first. */
  readonly observed: readonly ObserveRecord[];
  /** The project's sources with tested examples. */
  examplesOf(project: Project): Promise<ExamplesSource[]>;
}

export class ProjectTasks implements vscode.Disposable {
  private readonly disposables: vscode.Disposable[] = [];
  private readonly asked: AskedRecord[] = [];
  private readonly observed: ObserveRecord[] = [];
  private answers: boolean[] = [];

  constructor(
    private readonly context: vscode.ExtensionContext,
    private readonly projects: ProjectRegistry,
  ) {}

  get api(): TasksApi {
    return {
      answerNext: (run) => this.answers.push(run),
      asked: this.asked,
      observed: this.observed,
      examplesOf: (project) => this.examplesOf(project),
    };
  }

  register(): void {
    this.disposables.push(
      vscode.tasks.registerTaskProvider(TASK_TYPE, {
        provideTasks: () => this.provide(),
        resolveTask: (task) => this.resolve(task),
      }),
    );
  }

  dispose(): void {
    for (const disposable of this.disposables.splice(0)) disposable.dispose();
  }

  /** Runs one of a project's tasks, as the Project view's action does. */
  async run(project: Project, name: TaskName): Promise<void> {
    if ((await this.binaryOf(project)) === undefined) {
      void vscode.window.showErrorMessage(
        `Ascribe couldn't find a binary to run for ${this.projects.name(project)}. Show Output says where it looked.`,
      );
      return;
    }
    const tasks = await this.tasksOf(project, name);
    if (tasks.length === 0) {
      void vscode.window.showInformationMessage(
        `Ascribe: ${this.projects.name(project)} has no tested examples: none of its sources' folders has an ${EXAMPLES_FILE}.`,
      );
      return;
    }
    let task = tasks[0];
    if (tasks.length > 1) {
      const picked = await vscode.window.showQuickPick(
        tasks.map((each) => ({ label: each.name, task: each })),
        { title: taskLabel(name), placeHolder: "The source whose examples to observe" },
      );
      task = picked?.task;
    }
    if (task) await vscode.tasks.executeTask(task);
  }

  /** The project's sources in this repository whose folder has an `ascribe.examples.toml`. */
  async examplesOf(project: Project): Promise<ExamplesSource[]> {
    const text = await readFile(project.config, "utf8").catch(() => "");
    const sources = localSources(project, text);
    const has = await Promise.all(
      sources.map((source) =>
        stat(path.join(source.folder, EXAMPLES_FILE)).then(
          (found) => found.isFile(),
          () => false,
        ),
      ),
    );
    return sources.filter((_, i) => has[i]);
  }

  private async provide(): Promise<vscode.Task[]> {
    if (!vscode.workspace.isTrusted) return [];
    await this.projects.whenSettled();
    const all = await Promise.all(this.projects.projects.map((project) => this.tasksOf(project)));
    return all.flat();
  }

  private async resolve(task: vscode.Task): Promise<vscode.Task | undefined> {
    if (!vscode.workspace.isTrusted || !isAscribeTask(task.definition)) return undefined;
    const definition = task.definition;
    const folder =
      typeof task.scope === "object"
        ? task.scope.uri.fsPath
        : vscode.workspace.workspaceFolders?.[0]?.uri.fsPath;
    if (folder === undefined) return undefined;
    await this.projects.whenSettled();
    const project = definedProject(definition, folder, this.projects.projects);
    if (!project) return undefined;
    const tasks = (await this.tasksOf(project, definition.task)).filter(
      (each) =>
        definition.source === undefined ||
        (each.definition as AscribeTaskDefinition).source === definition.source,
    );
    // Observe Examples without a `source`, in a project with several, names no one folder.
    const made = tasks.length === 1 ? tasks[0] : undefined;
    if (!made) return undefined;
    // VS Code wants the definition it was given back, with the execution filled in.
    return new vscode.Task(
      definition,
      task.scope ?? made.scope ?? vscode.TaskScope.Workspace,
      task.name,
      TASK_TYPE,
      made.execution,
      [],
    );
  }

  /** A project's tasks, or only those named `only`; none when no binary is found for it. */
  private async tasksOf(project: Project, only?: TaskName): Promise<vscode.Task[]> {
    const binary = await this.binaryOf(project);
    if (binary === undefined) return [];
    const workspaceFolder = workspaceFolderOf(project);
    const scope =
      vscode.workspace.getWorkspaceFolder(vscode.Uri.file(project.folder)) ??
      vscode.TaskScope.Workspace;
    // Named after the project only when there's more than one to tell apart.
    const projectName = this.projects.projects.length > 1 ? this.projects.name(project) : undefined;
    const definition = (task: TaskName, source?: string): AscribeTaskDefinition => {
      const property = projectProperty(project, workspaceFolder);
      return {
        type: TASK_TYPE,
        task,
        ...(property !== undefined && { project: property }),
        ...(source !== undefined && { source }),
      };
    };
    const made: vscode.Task[] = [];
    for (const name of ["build", "syncAgentFiles"] as const) {
      if (only !== undefined && only !== name) continue;
      const task = new vscode.Task(
        definition(name),
        scope,
        taskLabel(name, projectName),
        TASK_TYPE,
        execution(binary, projectCommand(name, project, workspaceFolder)),
        [],
      );
      if (name === "build") task.group = vscode.TaskGroup.Build;
      made.push(task);
    }
    if (only === undefined || only === "observeExamples") {
      const sources = await this.examplesOf(project);
      for (const source of sources) {
        made.push(
          new vscode.Task(
            definition("observeExamples", source.name),
            scope,
            taskLabel("observeExamples", projectName, sources.length > 1 ? source.name : undefined),
            TASK_TYPE,
            new vscode.CustomExecution(() => Promise.resolve(this.observe(binary, source))),
            [],
          ),
        );
      }
    }
    return made;
  }

  /** The binary the project's server runs, or would run once started. */
  private async binaryOf(project: Project): Promise<string | undefined> {
    const running = this.projects.serverAt(project.folder)?.binary;
    if (running) return running.path;
    const resolution = await resolveProjectBinary(this.context, project);
    return resolution.kind === "found" ? resolution.binary.path : undefined;
  }

  /** Observe Examples' terminal: it asks, then runs `ascribe observe` in it. */
  private observe(binary: string, source: ExamplesSource): vscode.Pseudoterminal {
    const command = observeCommand(source);
    const written = new vscode.EventEmitter<string>();
    const closed = new vscode.EventEmitter<number>();
    const write = (text: string) => written.fire(text.replace(/\r?\n/g, "\r\n"));
    let child: ChildProcess | undefined;
    let ended = false;
    // A process that can't start reports an error, and may then close too.
    const end = (exitCode: number | undefined) => {
      if (ended) return;
      ended = true;
      this.observed.push({ cwd: command.cwd, args: command.args, exitCode });
      closed.fire(exitCode ?? 0);
    };
    return {
      onDidWrite: written.event,
      onDidClose: closed.event,
      open: () => {
        void (async () => {
          const file = path.join(source.folder, EXAMPLES_FILE);
          const text = await readFile(file, "utf8").catch(() => "");
          const question = observeQuestion(
            vscode.workspace.asRelativePath(file, true),
            examplesCommands(text),
          );
          const run = await this.ask(question);
          if (!run) {
            write("Observe Examples didn't run: you chose not to run the commands.\n");
            end(undefined);
            return;
          }
          write(`> ${[binary, ...command.args].join(" ")}\n`);
          child = spawn(shellCommand(binary), command.args, {
            cwd: command.cwd,
            windowsHide: true,
            // A `.cmd` shim, which is what npm links on Windows, needs a shell.
            shell: usesShell(binary),
          });
          child.stdout?.on("data", (data: Buffer) => write(data.toString()));
          child.stderr?.on("data", (data: Buffer) => write(data.toString()));
          child.on("error", (error) => {
            write(`Ascribe couldn't run ${binary}: ${error.message}\n`);
            end(undefined);
          });
          child.on("close", (code) => end(code ?? 1));
        })();
      },
      close: () => {
        child?.kill();
      },
    };
  }

  /** Asks whether to run the commands, each time, or takes a test's answer. */
  private async ask(question: ObserveQuestion): Promise<boolean> {
    const scripted = this.answers.shift();
    const answered =
      scripted ??
      (await vscode.window.showWarningMessage(
        question.message,
        { modal: true, detail: question.detail },
        RUN_THEM,
      )) === RUN_THEM;
    this.asked.push({ ...question, answered });
    return answered;
  }
}

/** A process, or, for a `.cmd` shim on Windows, a shell running it with its path quoted. */
function execution(
  binary: string,
  command: TaskCommand,
): vscode.ProcessExecution | vscode.ShellExecution {
  if (!usesShell(binary)) {
    return new vscode.ProcessExecution(binary, command.args, { cwd: command.cwd });
  }
  const quoted = (value: string) => ({ value, quoting: vscode.ShellQuoting.Strong });
  return new vscode.ShellExecution(quoted(binary), command.args.map(quoted), {
    cwd: command.cwd,
  });
}
