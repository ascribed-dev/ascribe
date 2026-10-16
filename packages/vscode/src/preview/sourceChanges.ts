// A review's source changes in VS Code's multi-file diff: each changed
// source file at the base against the working tree. The base side comes from
// VS Code's Git extension (its API's `toGitUri`) and, where the Git extension
// is off or has no repository for the file, from the language server, which
// answers a file's text at the base (`ascribe/review/baseFile`) as documents
// of the `ascribe-base:` scheme.

import * as vscode from "vscode";
import type { ProjectServer } from "../client.js";
import type { ProjectRegistry } from "../registry.js";
import { DIFF_FILES_LIMIT, diffTitle, type SourceFile } from "../ui/reviewTree.js";
import type { BaseFileResult, BaseInfo } from "../shapes.js";

/** The scheme of a file's text at the base, as the language server answers it. */
const BASE_SCHEME = "ascribe-base";

/** The language server's request for a file's text at the base (`crates/ascribe-lsp/README.md`). */
const BASE_FILE_REQUEST = "ascribe/review/baseFile";

/** How long to wait for the Git extension to find the workspace's repositories. */
const GIT_READY_MS = 5_000;

/** The part of the Git extension's API (`vscode.git`, version 1) used here. */
interface GitApi {
  readonly state: "uninitialized" | "initialized";
  readonly onDidChangeState: vscode.Event<"uninitialized" | "initialized">;
  getRepository(uri: vscode.Uri): object | null;
  toGitUri(uri: vscode.Uri, ref: string): vscode.Uri;
}

/** The Git extension's exports. */
interface GitExtension {
  readonly enabled: boolean;
  getAPI(version: 1): GitApi;
}

/** A diff opened, for tests. */
export interface OpenedDiff {
  title: string;
  /** Each file: the label, and each side's URI, `null` for a side it doesn't have. */
  files: { label: string; original: string | null; modified: string | null }[];
}

/** Open Source Changes, for tests. */
export interface SourceChangesApi {
  /** The diffs opened so far, in order. */
  opened(): OpenedDiff[];
  /** Whether the base side comes from the Git extension when it can; `false` always asks the server. */
  useGitExtension(use: boolean): void;
}

/** Opens a review's source changes in one diff. */
export class SourceChanges implements vscode.Disposable {
  private readonly log: OpenedDiff[] = [];
  private gitAllowed = true;
  private disposables: vscode.Disposable[] = [];

  constructor(private readonly projects: ProjectRegistry) {}

  register(): void {
    this.disposables.push(
      vscode.workspace.registerTextDocumentContentProvider(BASE_SCHEME, {
        provideTextDocumentContent: (uri) => this.baseText(uri),
      }),
    );
  }

  get api(): SourceChangesApi {
    return {
      opened: () => [...this.log],
      useGitExtension: (use) => {
        this.gitAllowed = use;
      },
    };
  }

  /**
   * Opens `files` at `base` against the working tree in one diff, titled
   * with the base's name. Says so when there's no file to show.
   */
  async open(base: BaseInfo, baseName: string, files: readonly SourceFile[]): Promise<void> {
    if (files.length === 0) {
      void vscode.window.showInformationMessage("Ascribe: no source file changed.");
      return;
    }
    const commit = base.merge_base ?? base.commit;
    const git = await this.git();
    const resources: [vscode.Uri, vscode.Uri | undefined, vscode.Uri | undefined][] = files.map(
      (file) => {
        const uri = vscode.Uri.file(file.path);
        const original = file.before ? baseUri(uri, commit, git) : undefined;
        return [uri, original, file.after ? uri : undefined];
      },
    );
    const title = diffTitle(baseName, files.length);
    this.log.push({
      title,
      files: resources.map(([, original, modified], i) => ({
        label: files[i]?.label ?? "",
        original: original?.toString() ?? null,
        modified: modified?.toString() ?? null,
      })),
    });
    if (files.length > DIFF_FILES_LIMIT) {
      void vscode.window.showInformationMessage(
        `Ascribe: ${files.length} files changed. VS Code's diff may repeat their headers past ${DIFF_FILES_LIMIT}; a page's own diff button shows its files alone.`,
      );
    }
    await vscode.commands.executeCommand("vscode.changes", title, resources);
  }

  /** The Git extension's API, when it's on and allowed; once it has looked for repositories. */
  private async git(): Promise<GitApi | undefined> {
    if (!this.gitAllowed) return undefined;
    const extension = vscode.extensions.getExtension<GitExtension>("vscode.git");
    if (!extension) return undefined;
    try {
      const exports = extension.isActive ? extension.exports : await extension.activate();
      if (!exports.enabled) return undefined;
      const api = exports.getAPI(1);
      if (api.state !== "initialized") {
        await new Promise<void>((resolve) => {
          const timer = setTimeout(done, GIT_READY_MS);
          const listener = api.onDidChangeState((state) => {
            if (state === "initialized") done();
          });
          function done(): void {
            clearTimeout(timer);
            listener.dispose();
            resolve();
          }
        });
      }
      return api;
    } catch {
      // The Git extension failed to start: the server answers instead.
      return undefined;
    }
  }

  /** A file's text at the base, from its project's language server; empty when it can't say. */
  private async baseText(uri: vscode.Uri): Promise<string> {
    const file = fromBaseUri(uri);
    const server: ProjectServer | undefined = this.projects.serverFor(file);
    if (!server || server.state !== "running") return "";
    try {
      const result = (await server.request(BASE_FILE_REQUEST, {
        textDocument: { uri: file.toString() },
      })) as BaseFileResult;
      return result.text ?? "";
    } catch (error) {
      server.reportFeatureError("review", error);
      return "";
    }
  }

  dispose(): void {
    for (const d of this.disposables) d.dispose();
    this.disposables = [];
  }
}

/**
 * The base side of a file: the Git extension's, when it has a repository for
 * the file, else the server's. The commit is in the server's URI so a new
 * base is a new document.
 */
function baseUri(file: vscode.Uri, commit: string, git: GitApi | undefined): vscode.Uri {
  if (git?.getRepository(file)) return git.toGitUri(file, commit);
  return file.with({ scheme: BASE_SCHEME, query: commit });
}

/** The file an `ascribe-base:` URI is of. */
function fromBaseUri(uri: vscode.Uri): vscode.Uri {
  return uri.with({ scheme: "file", query: "" });
}
