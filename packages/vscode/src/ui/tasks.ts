// The `ascribe` task type's provider: Build and Sync Agent Files for each
// project, and Observe Examples for each of its sources with tested
// examples, in **Tasks: Run Task** and the Project view, and any of them
// `tasks.json` names. What each runs is `../tasks.ts`'s.
//
// Each runs the binary the project's server runs, in the terminal, with no
// problem matcher: the Problems panel stays Check Every Build's and the
// editor's. None is offered, resolved, or run in Restricted Mode, where the
// extension starts nothing: Observe Examples runs commands the repository
// names, the case Restricted Mode exists for. In a trusted workspace,
// `ascribe observe` asks before it runs them, in the task's terminal.

import { execFile } from "node:child_process";
import * as vscode from "vscode";
import { resolveProjectBinary, workspaceFolderOf } from "../client.js";
import { shellCommand, usesShell } from "../environment.js";
import type { Project } from "../projects.js";
import type { ProjectRegistry } from "../registry.js";
import type { ModelReport } from "../shapes.js";
import {
  definedProject,
  definedSource,
  isAscribeTask,
  projectCommand,
  projectProperty,
  sourceCommand,
  TASK_TYPE,
  taskLabel,
  testedSources,
  type AscribeTaskDefinition,
  type ProjectTaskName,
  type TaskCommand,
  type TaskName,
  type TestedSource,
} from "../tasks.js";

/** The tasks run in the project's folder, in the order they're offered. */
const PROJECT_TASKS: readonly ProjectTaskName[] = ["build", "syncAgentFiles"];

export class ProjectTasks implements vscode.Disposable {
  private readonly disposables: vscode.Disposable[] = [];
  /**
   * Each project's sources with tested examples, by its folder and binary,
   * as the binary last answered: asked once, and again only after a file
   * that can change the answer does, so listing the tasks starts no
   * process most times.
   */
  private readonly sources = new Map<string, Promise<TestedSource[]>>();

  constructor(
    private readonly context: vscode.ExtensionContext,
    private readonly projects: ProjectRegistry,
  ) {}

  register(): void {
    this.disposables.push(
      vscode.tasks.registerTaskProvider(TASK_TYPE, {
        provideTasks: () => this.provide(),
        resolveTask: (task) => this.resolve(task),
      }),
    );
    // `ascribe.toml` names the sources, and `ascribe.examples.toml` makes a
    // source's examples tested: a change to either, or to the projects,
    // asks the binary again.
    const watcher = vscode.workspace.createFileSystemWatcher(
      "**/{ascribe.toml,ascribe.examples.toml}",
    );
    const forget = () => this.sources.clear();
    this.disposables.push(
      watcher,
      watcher.onDidChange(forget),
      watcher.onDidCreate(forget),
      watcher.onDidDelete(forget),
      this.projects.onDidChangeProjects(forget),
    );
  }

  dispose(): void {
    for (const disposable of this.disposables.splice(0)) disposable.dispose();
  }

  /**
   * Runs one of a project's tasks, as the Project view's action does. With
   * more than one source with tested examples, Observe Examples asks which.
   */
  async run(project: Project, name: TaskName): Promise<void> {
    if (!vscode.workspace.isTrusted) return;
    const binary = await this.binaryOf(project);
    if (binary === undefined) {
      void vscode.window.showErrorMessage(
        `Ascribe couldn't find a binary to run for ${this.projects.name(project)}. Show Output says where it looked.`,
      );
      return;
    }
    const tasks = await this.tasksOf(project, name, binary);
    if (tasks.length === 0) {
      void vscode.window.showInformationMessage(
        `No source of ${this.projects.name(project)} has tested examples: an ascribe.examples.toml in its folder. Observe Examples runs ascribe observe in each one's folder.`,
      );
      return;
    }
    const task =
      tasks.length === 1
        ? tasks[0]
        : (
            await vscode.window.showQuickPick(
              tasks.map((task) => ({ label: task.name, task })),
              { title: "Observe Examples", placeHolder: "The source to observe" },
            )
          )?.task;
    if (task) await vscode.tasks.executeTask(task);
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
    const binary = await this.binaryOf(project);
    if (binary === undefined) return undefined;
    const tasks = await this.tasksOf(project, definition.task, binary);
    const sourceOf = (each: vscode.Task) => String(each.definition["source"]);
    let made = tasks[0];
    if (definition.task === "observeExamples") {
      const source = definedSource(definition, tasks.map(sourceOf));
      made = tasks.find((each) => source !== undefined && sourceOf(each) === source);
    }
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

  /**
   * A project's tasks, or only those of the one named `only`: one for each
   * task run in the project's folder, and Observe Examples for each source
   * with tested examples. None when no binary is found for it.
   */
  private async tasksOf(project: Project, only?: TaskName, found?: string): Promise<vscode.Task[]> {
    const binary = found ?? (await this.binaryOf(project));
    if (binary === undefined) return [];
    const workspaceFolder = workspaceFolderOf(project);
    const scope =
      vscode.workspace.getWorkspaceFolder(vscode.Uri.file(project.folder)) ??
      vscode.TaskScope.Workspace;
    // Named after the project only when there's more than one to tell apart.
    const projectName = this.projects.projects.length > 1 ? this.projects.name(project) : undefined;
    const property = projectProperty(project, workspaceFolder);
    const make = (definition: AscribeTaskDefinition, label: string, command: TaskCommand) =>
      new vscode.Task(definition, scope, label, TASK_TYPE, execution(binary, command), []);
    const tasks = PROJECT_TASKS.filter((name) => only === undefined || only === name).map(
      (name) => {
        const definition: AscribeTaskDefinition = {
          type: TASK_TYPE,
          task: name,
          ...(property !== undefined && { project: property }),
        };
        const task = make(
          definition,
          taskLabel(name, projectName),
          projectCommand(name, project, workspaceFolder),
        );
        if (name === "build") task.group = vscode.TaskGroup.Build;
        return task;
      },
    );
    if (only !== undefined && only !== "observeExamples") return tasks;
    const sources = await this.sourcesOf(binary, project);
    for (const source of sources) {
      const definition: AscribeTaskDefinition = {
        type: TASK_TYPE,
        task: "observeExamples",
        ...(property !== undefined && { project: property }),
        source: source.name,
      };
      // Named after the source only when there's more than one to tell apart.
      const label = taskLabel(
        "observeExamples",
        projectName,
        sources.length > 1 ? source.name : undefined,
      );
      tasks.push(make(definition, label, sourceCommand(project, source)));
    }
    return tasks;
  }

  /**
   * The project's sources with tested examples, as the binary answers:
   * `ascribe model --section sources --format json`, in the project's
   * folder. None when it can't answer, as for a content model with errors.
   */
  private sourcesOf(binary: string, project: Project): Promise<TestedSource[]> {
    const key = `${binary}\0${project.folder}`;
    let answer = this.sources.get(key);
    if (!answer) {
      answer = this.askSources(binary, project);
      this.sources.set(key, answer);
    }
    return answer;
  }

  private askSources(binary: string, project: Project): Promise<TestedSource[]> {
    return new Promise((resolve) => {
      execFile(
        shellCommand(binary),
        ["model", "--section", "sources", "--format", "json"],
        {
          cwd: project.folder,
          windowsHide: true,
          // A `.cmd` shim, which is what npm links on Windows, needs a shell.
          shell: usesShell(binary),
        },
        (error, stdout) => {
          if (error) {
            resolve([]);
            return;
          }
          try {
            resolve(testedSources(JSON.parse(stdout) as ModelReport));
          } catch {
            resolve([]);
          }
        },
      );
    });
  }

  /** The binary the project's server runs, or would run once started. */
  private async binaryOf(project: Project): Promise<string | undefined> {
    const running = this.projects.serverAt(project.folder)?.binary;
    if (running) return running.path;
    const resolution = await resolveProjectBinary(this.context, project);
    return resolution.kind === "found" ? resolution.binary.path : undefined;
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
