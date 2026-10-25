import { execFile } from "node:child_process";
import { existsSync } from "node:fs";
import { stat } from "node:fs/promises";
import * as path from "node:path";
import * as vscode from "vscode";
import { readLayout } from "../diskChanges.js";
import { shellCommand, usesShell } from "../environment.js";
import { KEEP_ADDRESSES, keepQuestion, pageMoves, type Moved } from "../keepAddresses.js";
import { comparable, owningProject, type Project } from "../projects.js";
import type { ProjectRegistry } from "../registry.js";

/** `workspaceState` key: the folders of the projects the offer was made for, each once. */
const OFFERED = "ascribe.keepAddressesOffered";

/** **Keep the addresses you've published**, for tests. */
export interface KeepAddressesApi {
  /** Answers the next offer with `choice`, the button's title, or `undefined` to dismiss it, instead of showing it. */
  answerNext(choice: string | undefined): void;
  /**
   * How many of the editor's moves it has seen. VS Code tells of a move
   * after the edit that made it settles, so a test waits for this first.
   */
  seen(): number;
  /** Settles when every move seen so far has been handled: offered or not, and its answer acted on. */
  whenDone(): Promise<void>;
  /** The folders of the projects the offer was made for. */
  offered(): string[];
}

/**
 * **Keep the addresses you've published**: after the editor moves a page of
 * a project without `ascribe.addresses`, the extension offers, once for the
 * project, to start the file. Accepted, it runs `ascribe addresses init`, then
 * `ascribe addresses move` for each page the move took, so their old
 * addresses are kept too. Dismissed or not, it isn't offered again for the
 * project: whether to keep them is the project's decision, and a docs page
 * says how.
 */
export class KeepAddresses implements vscode.Disposable {
  private readonly disposables: vscode.Disposable[] = [];
  private readonly answers: (string | undefined)[] = [];
  private handling: Promise<void> = Promise.resolve();
  private seen = 0;

  constructor(
    private readonly projects: ProjectRegistry,
    private readonly storage: vscode.Memento,
  ) {}

  get api(): KeepAddressesApi {
    return {
      answerNext: (choice) => this.answers.push(choice),
      seen: () => this.seen,
      whenDone: () => this.handling,
      offered: () => this.offeredFolders(),
    };
  }

  register(): void {
    this.disposables.push(
      vscode.workspace.onDidRenameFiles(({ files }) => {
        const moved = files
          .filter(({ oldUri, newUri }) => oldUri.scheme === "file" && newUri.scheme === "file")
          .map(({ oldUri, newUri }) => ({ from: oldUri.fsPath, to: newUri.fsPath }));
        this.seen += 1;
        this.handling = this.handling.then(() => this.renamed(moved)).catch(() => undefined);
      }),
    );
  }

  dispose(): void {
    for (const disposable of this.disposables.splice(0)) disposable.dispose();
  }

  private async renamed(moved: Moved[]): Promise<void> {
    const byProject = new Map<string, { project: Project; moved: Moved[] }>();
    for (const move of moved) {
      const project = owningProject(move.to, this.projects.projects);
      if (!project) continue;
      const key = comparable(project.folder);
      const entry = byProject.get(key) ?? { project, moved: [] };
      entry.moved.push(...(await filesOf(move)));
      byProject.set(key, entry);
    }
    for (const { project, moved: files } of byProject.values()) {
      await this.offer(project, files);
    }
  }

  /** Offers to keep `project`'s addresses, after the editor moved `moved`, unless it was offered before. */
  private async offer(project: Project, moved: Moved[]): Promise<void> {
    if (this.offeredFolders().includes(comparable(project.folder))) return;
    // Outside the language server: whether the project keeps its addresses.
    if (existsSync(path.join(project.folder, "ascribe.addresses"))) return;
    const binary = this.projects.serverAt(project.folder)?.binary?.path;
    if (!binary) return;
    const { contentRoot } = await readLayout(project);
    const pages = pageMoves(moved, contentRoot);
    if (pages.length === 0) return;
    await this.storage.update(OFFERED, [...this.offeredFolders(), comparable(project.folder)]);
    const name = this.projects.name(project);
    const choice =
      this.answers.length > 0
        ? this.answers.shift()
        : await vscode.window.showInformationMessage(
            keepQuestion(name, pages.length),
            KEEP_ADDRESSES,
          );
    if (choice !== KEEP_ADDRESSES) return;
    // From the project's folder, which the commands run in: they take no absolute path.
    const root = path.relative(project.folder, contentRoot).split(path.sep).join("/");
    const content = (page: string) => path.posix.join(root, page);
    try {
      await run(binary, project, ["addresses", "init"]);
      for (const page of pages) {
        await run(binary, project, ["addresses", "move", content(page.from), content(page.to)]);
      }
    } catch (error) {
      const message = error instanceof Error ? error.message : String(error);
      void vscode.window.showErrorMessage(
        `Ascribe couldn't keep the addresses of ${name}: ${message}`,
      );
      return;
    }
    void vscode.window.showInformationMessage(
      `Ascribe: ${name} keeps its addresses in ascribe.addresses now, with ${pages.length === 1 ? "the move" : `the ${pages.length} moves`} recorded. Commit it with the move.`,
    );
  }

  private offeredFolders(): string[] {
    const stored = this.storage.get<unknown>(OFFERED);
    return Array.isArray(stored) ? stored.filter((f): f is string => typeof f === "string") : [];
  }
}

/**
 * The files a move took: the file itself, or each Markdown file now in a
 * folder moved, with where it was. Outside the language server, so the
 * extension lists them itself.
 */
async function filesOf(move: Moved): Promise<Moved[]> {
  const folder = await stat(move.to).then(
    (s) => s.isDirectory(),
    () => false,
  );
  if (!folder) return [move];
  const found = await vscode.workspace.findFiles(
    new vscode.RelativePattern(vscode.Uri.file(move.to), "**/*.md"),
  );
  return found.map((uri) => ({
    from: path.join(move.from, path.relative(move.to, uri.fsPath)),
    to: uri.fsPath,
  }));
}

/** Runs the binary in the project's folder; rejects with what it wrote to standard error when it fails. */
function run(binary: string, project: Project, args: string[]): Promise<void> {
  // A `.cmd` shim, which is what npm links on Windows, needs a shell, which
  // joins the arguments unquoted.
  const shell = usesShell(binary);
  return new Promise((resolve, reject) => {
    execFile(
      shellCommand(binary),
      shell ? args.map((arg) => `"${arg}"`) : args,
      { cwd: project.folder, windowsHide: true, shell },
      (error, _stdout, stderr) => {
        if (error) reject(new Error(stderr.trim() || error.message));
        else resolve();
      },
    );
  });
}
