import * as vscode from "vscode";
import { anySynced, SYNCED_KEY, WATCHED } from "./synced.js";

/** What the extension returns for tests. */
export interface SkillApi {
  /** The key's value as last set. */
  synced(): boolean;
  /** Settles when the key has been worked out again. */
  whenSettled(): Promise<void>;
}

/**
 * Keeps `ascribe.agentsSynced` set: the extension's own copy of the skill is
 * offered to chat only while it's false. Each workspace folder is looked at
 * on its own: a copy `sync` wrote at the repository's root isn't one chat
 * reads in a workspace opened below it.
 */
export class SyncedSkillKey implements vscode.Disposable {
  private readonly disposables: vscode.Disposable[] = [];
  private watchers: vscode.Disposable[] = [];
  private value = false;
  private updating: Promise<void> = Promise.resolve();

  register(): void {
    this.disposables.push(
      vscode.workspace.onDidChangeWorkspaceFolders(() => {
        this.watch();
        this.update();
      }),
    );
    this.watch();
    this.update();
  }

  get api(): SkillApi {
    return { synced: () => this.value, whenSettled: () => this.updating };
  }

  /** One watcher per workspace folder, for the files and folders `WATCHED` names. */
  private watch(): void {
    for (const watcher of this.watchers.splice(0)) watcher.dispose();
    for (const folder of vscode.workspace.workspaceFolders ?? []) {
      const watcher = vscode.workspace.createFileSystemWatcher(
        new vscode.RelativePattern(folder, WATCHED),
        false,
        true,
        false,
      );
      watcher.onDidCreate(() => this.update());
      watcher.onDidDelete(() => this.update());
      this.watchers.push(watcher);
    }
  }

  /** Looks for a copy again, after any look under way, and sets the key. */
  private update(): void {
    this.updating = this.updating
      .then(async () => {
        const folders = (vscode.workspace.workspaceFolders ?? []).map((f) => f.uri.fsPath);
        this.value = await anySynced(folders, isFile);
        await vscode.commands.executeCommand("setContext", SYNCED_KEY, this.value);
      })
      // Setting a context key fails only while the window closes; the next
      // change looks again.
      .catch(() => undefined);
  }

  dispose(): void {
    for (const disposable of [...this.watchers.splice(0), ...this.disposables.splice(0)]) {
      disposable.dispose();
    }
  }
}

async function isFile(file: string): Promise<boolean> {
  try {
    const stat = await vscode.workspace.fs.stat(vscode.Uri.file(file));
    return (stat.type & vscode.FileType.File) !== 0;
  } catch {
    return false;
  }
}
