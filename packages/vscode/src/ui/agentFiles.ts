// The agent files out of date, for the Project view: `ascribe agents sync
// --check` for a project, the one count that's no problem `check` reports.
// The agent files are made from the content model and the pages, so most
// saves can change it: it runs again after each save of a file of the
// project, at most once every few seconds, never as you type. A change on
// disk that isn't a save, such as a `git checkout`, makes it out of date
// until it runs again.

import { execFile } from "node:child_process";
import { stat } from "node:fs/promises";
import { tmpdir } from "node:os";
import * as path from "node:path";
import * as vscode from "vscode";
import type { ProjectServer } from "../client.js";
import { changesCheck, readLayout, type Layout } from "../diskChanges.js";
import { shellCommand, usesShell } from "../environment.js";
import { comparable, type Project } from "../projects.js";
import type { ProjectRegistry } from "../registry.js";
import { staleFiles } from "./projectHealth.js";

/** The shortest time between two checks of a project's agent files, after a save. */
const AGENT_FILES_INTERVAL_MS = 5_000;

/** A project's check of its agent files, as the Project view counts it. */
export type AgentFilesResult =
  | { kind: "none" }
  | { kind: "running" }
  | {
      kind: "made";
      /** When it started, in milliseconds since the epoch. */
      at: number;
      /** The files `ascribe agents sync` would change, as it names them from the project's folder. */
      stale: string[];
      /** Why the files couldn't be checked, or `null`. */
      error: string | null;
      /** Whether no file of the project has changed on disk since, other than by a save it ran after. */
      current: boolean;
    };

/** The agent files' checks, for tests. */
export interface AgentFilesApi {
  /** The project in `folder`'s check, under way, made, or none. */
  resultOf(folder: string): AgentFilesResult;
  /** How many checks have started, for every project. */
  runs(): number;
  /** Settles when no check is under way or due. */
  whenSettled(): Promise<void>;
}

interface Made {
  project: Project;
  layout: Layout;
  at: number;
  stale: string[];
  error: string | null;
  current: boolean;
}

/** A project's checks: the one under way, the one due, and when the last started. */
interface Runs {
  running: Promise<void> | undefined;
  due: ReturnType<typeof setTimeout> | undefined;
  /** A save came while one ran: it runs again once that finishes. */
  again: boolean;
  last: number;
}

export class AgentFilesChecks implements vscode.Disposable {
  private readonly made = new Map<string, Made>();
  private readonly runs = new Map<string, Runs>();
  private started = 0;
  private readonly changed = new vscode.EventEmitter<void>();
  private readonly disposables: vscode.Disposable[] = [];

  constructor(private readonly projects: ProjectRegistry) {}

  /** Fires when a project's check starts, is made, or stops being current. */
  get onDidChange(): vscode.Event<void> {
    return this.changed.event;
  }

  get api(): AgentFilesApi {
    return {
      resultOf: (folder) => this.resultOf(folder),
      runs: () => this.started,
      whenSettled: () => this.whenSettled(),
    };
  }

  register(): void {
    const watcher = vscode.workspace.createFileSystemWatcher("**/*");
    const changedUri = (uri: vscode.Uri) => {
      if (uri.scheme === "file") void this.diskChanged(uri.fsPath);
    };
    this.disposables.push(
      watcher,
      watcher.onDidChange(changedUri),
      watcher.onDidCreate(changedUri),
      watcher.onDidDelete(changedUri),
      vscode.workspace.onDidSaveTextDocument((document) => {
        if (document.uri.scheme !== "file") return;
        const server = this.projects.serverFor(document.uri);
        if (server && this.made.has(comparable(server.project.folder))) this.soon(server);
      }),
      this.projects.onDidChangeProjects(() => {
        const known = new Set(this.projects.projects.map((p) => comparable(p.folder)));
        for (const key of this.made.keys()) if (!known.has(key)) this.made.delete(key);
      }),
    );
  }

  dispose(): void {
    for (const runs of this.runs.values()) clearTimeout(runs.due);
    for (const disposable of this.disposables.splice(0)) disposable.dispose();
    this.changed.dispose();
  }

  /** The project in `folder`'s check: running while one is under way or due after a save. */
  resultOf(folder: string): AgentFilesResult {
    const key = comparable(folder);
    const runs = this.runs.get(key);
    if (runs && (runs.running || runs.due)) return { kind: "running" };
    const made = this.made.get(key);
    if (!made) return { kind: "none" };
    const { at, stale, error, current } = made;
    return { kind: "made", at, stale, error, current };
  }

  /** Checks `server`'s project once, the first time the Project view shows it with its server running. */
  ensure(server: ProjectServer): void {
    const key = comparable(server.project.folder);
    const runs = this.runs.get(key);
    if (server.state !== "running" || this.made.has(key) || runs?.running || runs?.due) return;
    void this.checkProject(server);
  }

  /** Checks `server`'s project's agent files now, or once the check under way finishes. */
  checkProject(server: ProjectServer): Promise<void> {
    const runs = this.runsOf(server);
    clearTimeout(runs.due);
    runs.due = undefined;
    if (runs.running) {
      runs.again = true;
      return runs.running;
    }
    return this.run(server, runs);
  }

  /**
   * Runs `ascribe agents sync` for `server`'s project, logging what it wrote
   * in the server's output, then checks the files again.
   */
  async sync(server: ProjectServer): Promise<void> {
    const binary = server.binary;
    if (!binary) return;
    const name = this.projects.name(server.project);
    const { code, stdout, stderr } = await runAgents(binary.path, server.project, []);
    server.log(`ascribe agents sync:\n${stdout}${stderr}`);
    if (code !== 0) {
      void vscode.window.showErrorMessage(
        `Ascribe couldn't sync the agent files of ${name}: ${stderr.trim() || `it exited with ${code}`}`,
      );
    }
    await this.checkProject(server);
  }

  /** After a save: soon, but at most once every `AGENT_FILES_INTERVAL_MS`. */
  private soon(server: ProjectServer): void {
    const runs = this.runsOf(server);
    if (runs.running) {
      runs.again = true;
      return;
    }
    if (runs.due) return;
    const wait = Math.max(0, runs.last + AGENT_FILES_INTERVAL_MS - Date.now());
    runs.due = setTimeout(() => {
      runs.due = undefined;
      void this.run(server, runs);
    }, wait);
    this.changed.fire();
  }

  private runsOf(server: ProjectServer): Runs {
    const key = comparable(server.project.folder);
    let runs = this.runs.get(key);
    if (!runs) {
      runs = { running: undefined, due: undefined, again: false, last: 0 };
      this.runs.set(key, runs);
    }
    return runs;
  }

  private run(server: ProjectServer, runs: Runs): Promise<void> {
    const done = this.check(server, runs).finally(() => {
      runs.running = undefined;
      if (runs.again) {
        runs.again = false;
        this.soon(server);
      }
      this.changed.fire();
    });
    runs.running = done;
    return done;
  }

  private async check(server: ProjectServer, runs: Runs): Promise<void> {
    const binary = server.binary;
    const project = server.project;
    if (!binary || server.state !== "running") return;
    const at = Date.now();
    runs.last = at;
    this.started++;
    const layout = await readLayout(project);
    this.changed.fire();
    const { code, stdout, stderr } = await runAgents(binary.path, project, ["--check"]);
    // 0: up to date; 1: a file is out of date. Anything else couldn't check.
    const error =
      code === 0 || code === 1 ? null : stderr.trim() || `ascribe agents sync exited with ${code}`;
    if (error !== null) server.log(`ascribe agents sync --check: ${error}`);
    this.made.set(comparable(project.folder), {
      project,
      layout,
      at,
      // From the project's folder, as `sync` names them when it's run there.
      stale:
        error === null
          ? staleFiles(stdout).map((file) =>
              path
                .relative(project.folder, path.resolve(AGENTS_CWD, file))
                .split(path.sep)
                .join("/"),
            )
          : [],
      error,
      current: true,
    });
  }

  /**
   * A file changed on disk: a project's check that read it before is out of
   * date. A save's change is followed by a check of its own.
   */
  private async diskChanged(file: string): Promise<void> {
    if (this.made.size === 0) return;
    // Outside the language server: when the file was last written, if it still exists.
    const written = await stat(file).then(
      (s) => s.mtimeMs,
      () => undefined,
    );
    let changed = false;
    for (const made of this.made.values()) {
      if (!made.current) continue;
      if (written !== undefined && written < made.at) continue;
      if (!changesCheck(file, made.project, made.layout, this.projects.projects)) continue;
      made.current = false;
      changed = true;
    }
    if (changed) this.changed.fire();
  }

  private async whenSettled(): Promise<void> {
    for (;;) {
      const busy = [...this.runs.values()].filter((runs) => runs.running || runs.due);
      if (busy.length === 0) return;
      await Promise.all(
        busy.map((runs) => runs.running ?? new Promise((resolve) => setTimeout(resolve, 100))),
      );
    }
  }
}

/**
 * Where `ascribe agents sync` runs: outside every project, since a process's
 * folder can't be deleted on Windows while it runs, and the check runs after
 * saves the writer didn't ask for.
 */
const AGENTS_CWD = tmpdir();

/** Runs `ascribe agents sync` for the project, naming its `ascribe.toml` with `--config`. */
function runAgents(
  binary: string,
  project: Project,
  args: string[],
): Promise<{ code: number; stdout: string; stderr: string }> {
  // A `.cmd` shim, which is what npm links on Windows, needs a shell, which
  // joins the arguments unquoted.
  const shell = usesShell(binary);
  const config = shell ? `"${project.config}"` : project.config;
  return new Promise((resolve) => {
    execFile(
      shellCommand(binary),
      ["--config", config, "agents", "sync", ...args],
      { cwd: AGENTS_CWD, windowsHide: true, shell },
      (error, stdout, stderr) => {
        const code = error === null ? 0 : typeof error.code === "number" ? error.code : Number.NaN;
        resolve({
          code,
          stdout,
          stderr: Number.isNaN(code) && error ? error.message : stderr,
        });
      },
    );
  });
}
