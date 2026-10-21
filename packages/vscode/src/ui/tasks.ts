// The `ascribe` task type's provider: Build and Sync Agent Files for each
// project, in **Tasks: Run Task** and the Project view, and either of them
// `tasks.json` names. What each runs is `../tasks.ts`'s.
//
// Each runs the binary the project's server runs, in the terminal, with no
// problem matcher: the Problems panel stays Check Every Build's and the
// editor's. None is offered in Restricted Mode, where the extension starts
// nothing.

import * as vscode from "vscode";
import { resolveProjectBinary, workspaceFolderOf } from "../client.js";
import { usesShell } from "../environment.js";
import type { Project } from "../projects.js";
import type { ProjectRegistry } from "../registry.js";
import {
  definedProject,
  isAscribeTask,
  projectCommand,
  projectProperty,
  TASK_TYPE,
  taskLabel,
  TASKS,
  type AscribeTaskDefinition,
  type TaskCommand,
  type TaskName,
} from "../tasks.js";

export class ProjectTasks implements vscode.Disposable {
  private readonly disposables: vscode.Disposable[] = [];

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
  }

  dispose(): void {
    for (const disposable of this.disposables.splice(0)) disposable.dispose();
  }

  /** Runs one of a project's tasks, as the Project view's action does. */
  async run(project: Project, name: TaskName): Promise<void> {
    const [task] = await this.tasksOf(project, name);
    if (!task) {
      void vscode.window.showErrorMessage(
        `Ascribe couldn't find a binary to run for ${this.projects.name(project)}. Show Output says where it looked.`,
      );
      return;
    }
    await vscode.tasks.executeTask(task);
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
    const [made] = await this.tasksOf(project, definition.task);
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

  /** A project's tasks, or only the one named `only`; none when no binary is found for it. */
  private async tasksOf(project: Project, only?: TaskName): Promise<vscode.Task[]> {
    const binary = await this.binaryOf(project);
    if (binary === undefined) return [];
    const workspaceFolder = workspaceFolderOf(project);
    const scope =
      vscode.workspace.getWorkspaceFolder(vscode.Uri.file(project.folder)) ??
      vscode.TaskScope.Workspace;
    // Named after the project only when there's more than one to tell apart.
    const projectName = this.projects.projects.length > 1 ? this.projects.name(project) : undefined;
    const property = projectProperty(project, workspaceFolder);
    const names = only === undefined ? (Object.keys(TASKS) as TaskName[]) : [only];
    return names.map((name) => {
      const definition: AscribeTaskDefinition = {
        type: TASK_TYPE,
        task: name,
        ...(property !== undefined && { project: property }),
      };
      const task = new vscode.Task(
        definition,
        scope,
        taskLabel(name, projectName),
        TASK_TYPE,
        execution(binary, projectCommand(name, project, workspaceFolder)),
        [],
      );
      if (name === "build") task.group = vscode.TaskGroup.Build;
      return task;
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
