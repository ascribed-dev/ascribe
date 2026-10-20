import { execFile } from "node:child_process";
import { readFile, stat } from "node:fs/promises";
import * as path from "node:path";
import * as vscode from "vscode";
import { CHECK_SCHEMA_VERSION } from "../agents/problems.js";
import type { ProjectServer } from "../client.js";
import { changesCheck, readLayout, type Layout } from "../diskChanges.js";
import { shellCommand, usesShell } from "../environment.js";
import { checkedText, panelProblems, type EditorRange, type PanelProblem } from "../everyBuild.js";
import { comparable, owningProject, type Project } from "../projects.js";
import type { ProjectRegistry } from "../registry.js";
import type { CheckReport, Summary } from "../shapes.js";
import { STATE_NAMES } from "./describe.js";

export const CHECK_EVERY_BUILD_COMMAND = "ascribe.checkEveryBuild";
const SHOW_PROBLEMS = "Show Problems";
/** The most JSON `ascribe check` may write: a project with a great many problems. */
const MAX_OUTPUT = 256 * 1024 * 1024;

/** What a check of every build left in the Problems panel, for tests. */
export interface EveryBuildApi {
  /** The problems it added, for every project, in file order. */
  problems(): { file: string; code: string; message: string }[];
  /** Whether a project's check is current: nothing in the project has changed since. */
  isCurrent(folder: string): boolean;
  /** Settles when the check under way, if any, has finished. */
  whenDone(): Promise<void>;
}

/** A project's check of every build, under way or made. */
interface Checked {
  project: Project;
  layout: Layout;
  /** When it started, in milliseconds since the epoch. */
  started: number;
  /** The files the check added problems to, as URIs. */
  files: Set<string>;
  /** Whether nothing in the project has changed since it started. */
  current: boolean;
  /** How many problems it found in every build, once it's made. */
  summary?: Summary;
  /** Why the project couldn't be checked, once it's made, or `null`. */
  error?: string | null;
  /** How many of the project's files had unsaved changes, which it read as saved. */
  unsaved?: number;
}

/** A project's check of every build, as the Project view counts it. */
export type EveryBuildResult =
  | { kind: "none" }
  | { kind: "running" }
  | {
      kind: "made";
      /** When it started, in milliseconds since the epoch. */
      at: number;
      summary: Summary;
      /** Why the project couldn't be checked, or `null`. */
      error: string | null;
      /** Whether nothing in the project has changed since it started (`isCurrent`). */
      current: boolean;
      /** How many of the project's files had unsaved changes, which it read as saved. */
      unsaved: number;
    };

/**
 * **Check Every Build**: runs `ascribe check` over every build of the active
 * file's project, and adds the problems the language server doesn't show
 * (those of builds other than the editor's) to the Problems panel, as their
 * own collection. A problem in a file that changes is taken away, since its
 * place may have moved, and the project's check is no longer current until
 * it runs again.
 */
export class EveryBuildChecks implements vscode.Disposable {
  private readonly collection = vscode.languages.createDiagnosticCollection("ascribe-every-build");
  private readonly checked = new Map<string, Checked>();
  /** The check under way. */
  private checking: Checked | undefined;
  private readonly changed = new vscode.EventEmitter<void>();
  private readonly disposables: vscode.Disposable[] = [];
  private running: Promise<void> = Promise.resolve();

  constructor(private readonly projects: ProjectRegistry) {}

  /** Fires when a project's check is made, or stops being current. */
  get onDidChange(): vscode.Event<void> {
    return this.changed.event;
  }

  get api(): EveryBuildApi {
    return {
      problems: () => {
        const all: { file: string; code: string; message: string }[] = [];
        this.collection.forEach((uri, diagnostics) => {
          for (const d of diagnostics) {
            all.push({ file: uri.fsPath, code: codeOf(d) ?? "", message: d.message });
          }
        });
        return all.sort((a, b) => (a.file < b.file ? -1 : a.file > b.file ? 1 : 0));
      },
      isCurrent: (folder) => this.isCurrent(folder),
      whenDone: () => this.running,
    };
  }

  register(): void {
    const watcher = vscode.workspace.createFileSystemWatcher("**/*");
    const changedUri = (uri: vscode.Uri) => {
      if (uri.scheme === "file") void this.diskChanged(uri.fsPath);
    };
    this.disposables.push(
      this.collection,
      watcher,
      watcher.onDidChange(changedUri),
      watcher.onDidCreate(changedUri),
      watcher.onDidDelete(changedUri),
      // Unsaved edits too: the check read the file as it was saved.
      vscode.workspace.onDidChangeTextDocument(({ document, contentChanges }) => {
        if (contentChanges.length > 0 && document.uri.scheme === "file") {
          this.fileChanged(document.uri.fsPath);
        }
      }),
      this.projects.onDidChangeProjects(() => {
        const known = new Set(this.projects.projects.map((p) => comparable(p.config)));
        for (const [key, checked] of this.checked) {
          if (!known.has(comparable(checked.project.config))) this.forget(key);
        }
      }),
      vscode.commands.registerCommand(CHECK_EVERY_BUILD_COMMAND, () => this.queue()),
    );
  }

  /**
   * Checks every build of `server`'s project, as the command does for the
   * active file's: the Project view's, which follows its own project.
   */
  checkProject(server: ProjectServer): Promise<void> {
    return this.queue(server);
  }

  /** One check at a time: a second waits for the first. */
  private queue(server?: ProjectServer): Promise<void> {
    this.running = this.running.then(() => this.check(server)).catch(() => undefined);
    return this.running;
  }

  /** The project in `folder`'s check of every build: under way, made, or none since the window opened. */
  resultOf(folder: string): EveryBuildResult {
    const key = comparable(folder);
    if (this.checking && comparable(this.checking.project.folder) === key) {
      return { kind: "running" };
    }
    const checked = this.checked.get(key);
    if (!checked?.summary) return { kind: "none" };
    return {
      kind: "made",
      at: checked.started,
      summary: checked.summary,
      error: checked.error ?? null,
      current: checked.current,
      unsaved: checked.unsaved ?? 0,
    };
  }

  /** Whether the Problems panel holds a check of every build of the project in `folder` that is still current. */
  isCurrent(folder: string): boolean {
    return this.checked.get(comparable(folder))?.current ?? false;
  }

  dispose(): void {
    for (const disposable of this.disposables.splice(0)) disposable.dispose();
    this.changed.dispose();
  }

  private async check(chosen?: ProjectServer): Promise<void> {
    const active = vscode.window.activeTextEditor?.document.uri;
    const server = chosen ?? (active && this.projects.serverFor(active)) ?? (await this.pick());
    if (!server) {
      if (this.projects.servers.length === 0) {
        void vscode.window.showInformationMessage(
          "Ascribe: there's no Ascribe project in this workspace.",
        );
      }
      return;
    }
    // The binary is found as the server starts; a server that can't find
    // one has said why.
    await server.start();
    await server.whenSettled();
    const binary = server.binary;
    if (!binary) return;
    const project = server.project;
    const name = this.projects.name(project);

    const run: Checked = {
      project,
      layout: await readLayout(project),
      started: Date.now(),
      files: new Set(),
      current: true,
    };
    this.checking = run;
    this.changed.fire();
    let report: CheckReport | undefined;
    try {
      report = await vscode.window.withProgress(
        {
          location: vscode.ProgressLocation.Notification,
          title: `Ascribe: checking every build of ${name}`,
          cancellable: true,
        },
        (_progress, token) => runCheck(binary.path, project, token),
      );
    } catch (error) {
      const message = error instanceof Error ? error.message : String(error);
      server.log(`Check Every Build failed: ${message}`);
      void vscode.window.showErrorMessage(`Ascribe couldn't check ${name}: ${message}`);
      return;
    } finally {
      this.checking = undefined;
      // The Project view stops saying it's checking, whatever came of it.
      this.changed.fire();
    }
    if (!report) return;
    if (report.schema_version !== CHECK_SCHEMA_VERSION) {
      void vscode.window.showErrorMessage(
        `Ascribe couldn't read the check of ${name}: ${binary.path} writes version ${report.schema_version} of the report, and the extension reads version ${CHECK_SCHEMA_VERSION}.`,
      );
      return;
    }

    const key = comparable(project.folder);
    this.forget(key);
    const texts = await readTexts(project.folder, report);
    const problems = panelProblems(
      report,
      project.folder,
      (file) => texts.get(file),
      (file, code, start) =>
        vscode.languages
          .getDiagnostics(vscode.Uri.file(file))
          .some(
            (d) =>
              d.source === "ascribe" &&
              codeOf(d) === code &&
              d.range.start.line === start.line &&
              d.range.start.character === start.character,
          ),
    );
    const byFile = new Map<string, vscode.Diagnostic[]>();
    for (const problem of problems) {
      const uri = vscode.Uri.file(problem.file).toString();
      byFile.set(uri, [...(byFile.get(uri) ?? []), toDiagnostic(problem)]);
    }
    for (const [uri, diagnostics] of byFile)
      this.collection.set(vscode.Uri.parse(uri), diagnostics);
    for (const uri of byFile.keys()) run.files.add(uri);
    run.current &&= report.error === null;
    const unsaved = vscode.workspace.textDocuments.filter(
      (d) =>
        d.isDirty &&
        d.uri.scheme === "file" &&
        owningProject(d.uri.fsPath, this.projects.projects)?.config === project.config,
    ).length;
    run.summary = report.summary;
    run.error = report.error;
    run.unsaved = unsaved;
    this.checked.set(key, run);
    this.changed.fire();

    const text = checkedText(report, name, problems.length, unsaved);
    const total = report.summary.errors + report.summary.warnings + report.summary.advice;
    const shown =
      report.error !== null
        ? vscode.window.showErrorMessage(text, SHOW_PROBLEMS)
        : vscode.window.showInformationMessage(text, ...(total > 0 ? [SHOW_PROBLEMS] : []));
    void shown.then((choice) => {
      if (choice === SHOW_PROBLEMS) {
        void vscode.commands.executeCommand("workbench.actions.view.problems");
      }
    });
  }

  /**
   * A file changed on disk. A change the check already read, made before it
   * started, is passed over: the watcher can report one late.
   */
  private async diskChanged(file: string): Promise<void> {
    const runs = [...this.checked.values(), ...(this.checking ? [this.checking] : [])];
    if (runs.length === 0) return;
    // Outside the language server: when the file was last written, if it still exists.
    const written = await stat(file).then(
      (s) => s.mtimeMs,
      () => undefined,
    );
    this.fileChanged(file, written);
  }

  /**
   * A file changed: the problems the check added to it go, and its
   * project's check isn't current. `written` is when it was written to
   * disk, for a change on disk to a file that still exists.
   */
  private fileChanged(file: string, written?: number): void {
    if (this.checking && this.counts(file, written, this.checking)) this.checking.current = false;
    let changed = false;
    for (const checked of this.checked.values()) {
      if (!this.counts(file, written, checked)) continue;
      const uri = vscode.Uri.file(file);
      if (checked.files.delete(uri.toString())) this.collection.delete(uri);
      if (checked.current) {
        checked.current = false;
        changed = true;
      }
    }
    if (changed) this.changed.fire();
  }

  /** Whether a change to a file makes a check out of date. */
  private counts(file: string, written: number | undefined, checked: Checked): boolean {
    if (written !== undefined && written < checked.started) return false;
    return changesCheck(file, checked.project, checked.layout, this.projects.projects);
  }

  private forget(key: string): void {
    const checked = this.checked.get(key);
    if (!checked) return;
    for (const uri of checked.files) this.collection.delete(vscode.Uri.parse(uri));
    this.checked.delete(key);
    this.changed.fire();
  }

  /** The only project's server, or the one picked from a list when there are several. */
  private async pick(): Promise<ProjectServer | undefined> {
    const servers = this.projects.servers;
    if (servers.length <= 1) return servers[0];
    const picked = await vscode.window.showQuickPick(
      servers.map((server) => ({
        label: this.projects.name(server.project),
        description: STATE_NAMES[server.state],
        server,
      })),
      { title: "Check Every Build", placeHolder: "The project to check" },
    );
    return picked?.server;
  }
}

/**
 * Runs `ascribe check --format json` in the project's folder, which finds
 * its `ascribe.toml` there. Resolves to its report,
 * whatever the exit code (`1` when there are problems), or to `undefined`
 * when it's cancelled.
 */
function runCheck(
  binary: string,
  project: Project,
  token: vscode.CancellationToken,
): Promise<CheckReport | undefined> {
  return new Promise((resolve, reject) => {
    const child = execFile(
      shellCommand(binary),
      ["check", "--format", "json"],
      {
        cwd: project.folder,
        maxBuffer: MAX_OUTPUT,
        windowsHide: true,
        // A `.cmd` shim, which is what npm links on Windows, needs a shell.
        shell: usesShell(binary),
      },
      (error, stdout, stderr) => {
        if (token.isCancellationRequested) {
          resolve(undefined);
          return;
        }
        try {
          resolve(JSON.parse(stdout) as CheckReport);
        } catch {
          const why = stderr.trim() || (error ? error.message : "it wrote no report");
          reject(new Error(why));
        }
      },
    );
    token.onCancellationRequested(() => child.kill());
  });
}

/**
 * The text of each file the report places a problem in, as the check read
 * it from disk, to place the problems by. Outside the language server, so
 * the extension reads them itself; a file it can't read gives nothing.
 */
async function readTexts(root: string, report: CheckReport): Promise<Map<string, string>> {
  const files = new Set<string>();
  for (const entry of report.diagnostics) {
    files.add(entry.file);
    for (const related of entry.related) files.add(related.file);
  }
  const texts = new Map<string, string>();
  await Promise.all(
    [...files].map(async (file) => {
      // As `panelProblems` names it.
      const absolute = path.resolve(root, file);
      const text = await readFile(absolute, "utf8").catch(() => undefined);
      if (text !== undefined) texts.set(absolute, text);
    }),
  );
  return texts;
}

function toDiagnostic(problem: PanelProblem): vscode.Diagnostic {
  const { entry } = problem;
  const diagnostic = new vscode.Diagnostic(
    toRange(problem.range),
    entry.message,
    entry.severity === "error"
      ? vscode.DiagnosticSeverity.Error
      : entry.severity === "warning"
        ? vscode.DiagnosticSeverity.Warning
        : vscode.DiagnosticSeverity.Information,
  );
  diagnostic.source = "ascribe";
  diagnostic.code = entry.docs
    ? { value: entry.code, target: vscode.Uri.parse(entry.docs) }
    : entry.code;
  if (problem.related.length > 0) {
    diagnostic.relatedInformation = problem.related.map(
      (r) =>
        new vscode.DiagnosticRelatedInformation(
          new vscode.Location(vscode.Uri.file(r.file), toRange(r.range)),
          r.message,
        ),
    );
  }
  return diagnostic;
}

function toRange(range: EditorRange): vscode.Range {
  return new vscode.Range(
    range.start.line,
    range.start.character,
    range.end.line,
    range.end.character,
  );
}

/** A diagnostic's code, such as `ASC036`. */
function codeOf(diagnostic: vscode.Diagnostic): string | undefined {
  const code = diagnostic.code;
  if (code === undefined) return undefined;
  return typeof code === "object" ? String(code.value) : String(code);
}
