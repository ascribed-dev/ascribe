// A pull request's review threads, for the preview's overlay and the source
// editor: one `ReviewSession` per project while review is on, made with VS
// Code's GitHub sign-in (or the GitHub CLI's), and every request the overlay
// makes, answered here. GitHub stays in the extension host: the webview never
// sees a token and makes no requests.

import { execFile } from "node:child_process";
import { readFile, realpath } from "node:fs/promises";
import * as path from "node:path";
import * as vscode from "vscode";
import {
  answerRequest,
  baseRevision,
  contentPrefixOf,
  ghTransport,
  openReview,
  readCheckout,
  ReviewError,
  tokenTransport,
  type GitHubTransport,
  type ReviewSession,
} from "@ascribed/review/github";
import type { PageRef } from "@ascribed/review/place";
import type { ProjectServer } from "../client.js";
import { comparable, throughFolder } from "../projects.js";
import type { ChangedPage, LocalState, ThreadsMethod, ThreadsView } from "./protocol.js";

/** The scope posting review comments needs: `repo` (`public_repo` covers public repositories only). */
export const GITHUB_SCOPES = ["repo"];

/** A project's connection to its pull request. */
export type Connection =
  /** No GitHub remote, a detached `HEAD`, or no open pull request for the branch: nothing to show. */
  | { state: "none" }
  /** No GitHub sign-in: the changes show without threads. */
  | { state: "signed-out"; host: string; gh: boolean }
  | { state: "error"; message: string }
  | {
      state: "on";
      session: ReviewSession;
      via: "vscode" | "gh" | "test";
      local: { state: LocalState; behind: number; ahead: number };
      /**
       * The content root, absolute, through the links the workspace was
       * opened with: where the threads' content paths are.
       */
      contentRoot: string;
      /** The git revision for the pull request's base: `origin/main`. */
      base: string;
    };

/** What the threads need from review. */
export interface ThreadsHost {
  /** The project's changed pages, for which pages show a thread. */
  changedPages(server: ProjectServer): Promise<ChangedPage[]>;
}

/** Where a change to the threads came from: the preview's overlay, the source editor, or a refresh. */
export type ChangeOrigin = "preview" | "source" | "refresh";

/** What the extension returns for tests. */
export interface ThreadsApi {
  /** The connection of the project in `folder`. */
  connection(folder: string): Connection | undefined;
  /**
   * Makes sessions with `transport` instead of signing in: a fake GitHub.
   * `undefined` goes back to signing in.
   */
  useTransport(transport: ((host: string) => GitHubTransport) | undefined): void;
}

/**
 * The review threads of each project with review on. A session is made when
 * review starts (asking for a GitHub sign-in only then, and only when the
 * reviewer started it), read again on Refresh, and dropped when review stops.
 */
export class ThreadsController implements vscode.Disposable {
  private readonly connections = new Map<string, Connection>();
  private readonly connecting = new Map<string, Promise<Connection>>();
  /** The last page the preview showed of each project: comments are made on it. */
  private readonly pages = new Map<string, PageRef>();
  private readonly changed = new vscode.EventEmitter<{
    server: ProjectServer;
    origin: ChangeOrigin;
  }>();
  private transport: ((host: string) => GitHubTransport) | undefined;

  /** Fires when a project's threads changed: connected, refreshed, or acted on. */
  readonly onDidChange = this.changed.event;

  constructor(
    private readonly host: ThreadsHost,
    /** The workspace's state, which remembers the projects whose reviewer chose the GitHub CLI. */
    private readonly state: vscode.Memento,
  ) {}

  get api(): ThreadsApi {
    return {
      connection: (folder) => this.connections.get(comparable(folder)),
      useTransport: (transport) => {
        this.transport = transport;
      },
    };
  }

  /** The project's connection, while review is on. */
  connection(server: ProjectServer): Connection | undefined {
    return this.connections.get(key(server));
  }

  /** The project's session, when it's connected. */
  session(server: ProjectServer): ReviewSession | undefined {
    const connection = this.connection(server);
    return connection?.state === "on" ? connection.session : undefined;
  }

  /**
   * Finds the project's pull request and reads its threads. With
   * `interactive`, asks for a GitHub sign-in when there's none; without, uses
   * one only if VS Code has it already.
   */
  connect(server: ProjectServer, options: { interactive: boolean }): Promise<Connection> {
    const id = key(server);
    const running = this.connecting.get(id);
    if (running) return running;
    const next = this.open(server, options.interactive)
      .catch((error: unknown): Connection => ({ state: "error", message: messageOf(error) }))
      .then((connection) => {
        if (this.connecting.get(id) === next) {
          this.connections.set(id, connection);
          this.changed.fire({ server, origin: "refresh" });
        }
        return connection;
      })
      .finally(() => {
        if (this.connecting.get(id) === next) this.connecting.delete(id);
      });
    this.connecting.set(id, next);
    return next;
  }

  /** Drops the project's session: review stopped. */
  disconnect(server: ProjectServer): void {
    const id = key(server);
    this.connecting.delete(id);
    this.pages.delete(id);
    if (this.connections.delete(id)) this.changed.fire({ server, origin: "refresh" });
  }

  /** Reads the pull request and its threads from GitHub again, without asking to sign in. */
  async refresh(server: ProjectServer): Promise<Connection> {
    return this.connect(server, { interactive: false });
  }

  /** Signs in to GitHub in VS Code, then reads the threads. */
  async signIn(server: ProjectServer): Promise<Connection> {
    await this.chooseGh(server, false);
    return this.connect(server, { interactive: true });
  }

  /** Reads the threads with the GitHub CLI's sign-in. */
  async useGh(server: ProjectServer): Promise<Connection> {
    await this.chooseGh(server, true);
    return this.connect(server, { interactive: false });
  }

  /** What the preview shows about the project's threads; `null` when there's no pull request. */
  view(server: ProjectServer, goTo: string | null): ThreadsView | null {
    const connection = this.connection(server);
    if (!connection || connection.state === "none") return null;
    if (connection.state === "signed-out") {
      return {
        state: "signed-out",
        pullRequest: null,
        local: null,
        gh: connection.gh,
        message: null,
        goTo: null,
      };
    }
    if (connection.state === "error") {
      return {
        state: "error",
        pullRequest: null,
        local: null,
        gh: false,
        message: connection.message,
        goTo: null,
      };
    }
    const pr = connection.session.pullRequest;
    return {
      state: "on",
      pullRequest: { number: pr.number, url: pr.url, baseRefName: pr.baseRefName },
      local: connection.local,
      gh: false,
      message: null,
      goTo,
    };
  }

  /**
   * Answers one of the overlay's requests (or the source editor's) for the
   * page the preview shows (`page`: its build and content path). Rejects with
   * a `ReviewError`, whose message is a sentence to show.
   */
  async handle(
    server: ProjectServer,
    page: { build: string; path: string } | undefined,
    method: ThreadsMethod,
    params: Record<string, unknown>,
    origin: ChangeOrigin,
  ): Promise<unknown> {
    const connection = this.connection(server);
    const session = this.session(server);
    if (!session || connection?.state !== "on") {
      throw new ReviewError("not-found", "Review comments aren't on for this project.");
    }
    const id = key(server);
    const answer = await answerRequest(
      {
        session,
        page,
        // A comment from the source editor is on the file, in every build.
        lastPage: origin === "preview" ? this.pages.get(id) : undefined,
        changedPages: () => this.host.changedPages(server),
        unsaved: (file) => unsavedText(path.join(connection.contentRoot, ...file.split("/"))),
      },
      method,
      params,
    );
    if (answer.page) this.pages.set(id, answer.page);
    if (answer.changed) this.changed.fire({ server, origin });
    return answer.result;
  }

  private async open(server: ProjectServer, interactive: boolean): Promise<Connection> {
    const projectDir = server.project.folder;
    let checkout;
    try {
      checkout = await readCheckout(projectDir);
    } catch {
      // Not a git repository, or one with no commits: nothing to review on GitHub.
      return { state: "none" };
    }
    const host = checkout.bases[0]?.host;
    if (host === undefined) return { state: "none" };
    let transport = this.transport;
    let via: "vscode" | "gh" | "test" = "test";
    if (!transport && this.choseGh(server)) {
      transport = (h) => ghTransport({ host: h });
      via = "gh";
    }
    if (!transport) {
      const signedIn = await githubSession(host, interactive);
      if (!signedIn) return { state: "signed-out", host, gh: await ghSignedIn(host) };
      transport = (h) => tokenTransport(() => githubToken(h), { host: h });
      via = "vscode";
    }
    let session: ReviewSession | undefined;
    try {
      session = await openReview({ projectDir, transport });
    } catch (error) {
      if (error instanceof ReviewError && error.code === "not-signed-in") {
        if (via === "gh") await this.chooseGh(server, false);
        return { state: "signed-out", host, gh: via !== "gh" && (await ghSignedIn(host)) };
      }
      throw error;
    }
    if (!session) return { state: "none" };
    const pr = session.pullRequest;
    const counts = await aheadBehind(checkout.root, pr.headOid);
    return {
      state: "on",
      session,
      via,
      local: { state: pr.local, ...counts },
      contentRoot: await openedContentRoot(
        path.join(checkout.root, ...contentPrefixOf(checkout.root, projectDir).split("/")),
        projectDir,
      ),
      base: await baseRevision(checkout.root, pr),
    };
  }

  /**
   * Whether the project's reviewer chose the GitHub CLI's sign-in: remembered
   * across windows, so Start Review doesn't ask for VS Code's sign-in first.
   */
  private choseGh(server: ProjectServer): boolean {
    return this.state.get<string[]>(VIA_GH, []).includes(key(server));
  }

  private async chooseGh(server: ProjectServer, gh: boolean): Promise<void> {
    const id = key(server);
    const others = this.state.get<string[]>(VIA_GH, []).filter((folder) => folder !== id);
    await this.state.update(VIA_GH, gh ? [...others, id] : others);
  }

  dispose(): void {
    this.changed.dispose();
  }
}

/** The workspace state key for the projects whose reviewer chose the GitHub CLI's sign-in. */
const VIA_GH = "ascribe.review.viaGh";

function key(server: ProjectServer): string {
  return comparable(server.project.folder);
}

/** VS Code's authentication provider for a host. */
function provider(host: string): string {
  return host === "github.com" ? "github" : "github-enterprise";
}

/** Whether there's a GitHub sign-in for the host; with `interactive`, asks for one. */
async function githubSession(host: string, interactive: boolean): Promise<boolean> {
  try {
    const session = await vscode.authentication.getSession(
      provider(host),
      GITHUB_SCOPES,
      interactive ? { createIfNone: true } : { silent: true },
    );
    return session !== undefined;
  } catch {
    // Declined, or no provider for the host (GitHub Enterprise isn't set up).
    return false;
  }
}

/** The host's token, read for each request so VS Code can refresh it. */
async function githubToken(host: string): Promise<string | undefined> {
  const session = await vscode.authentication
    .getSession(provider(host), GITHUB_SCOPES, { silent: true })
    .then(
      (s) => s,
      () => undefined,
    );
  return session?.accessToken;
}

/** Runs a command with a fixed argument list; resolves with its output, or `undefined` if it failed. */
function run(command: string, args: string[], cwd?: string): Promise<string | undefined> {
  return new Promise((resolve) => {
    execFile(
      command,
      args,
      { cwd, timeout: 15_000, windowsHide: true, encoding: "utf8" },
      (error, stdout) => resolve(error ? undefined : stdout),
    );
  });
}

/** Whether `gh` is installed and signed in to the host. */
async function ghSignedIn(host: string): Promise<boolean> {
  return (await run("gh", ["auth", "status", "--hostname", host])) !== undefined;
}

/** How many commits `HEAD` is ahead of and behind the pull request's head. */
async function aheadBehind(root: string, head: string): Promise<{ ahead: number; behind: number }> {
  const out = await run("git", ["rev-list", "--left-right", "--count", `HEAD...${head}`], root);
  const [ahead, behind] = (out ?? "").trim().split(/\s+/).map(Number);
  return { ahead: ahead || 0, behind: behind || 0 };
}

/**
 * A file's text on disk and in its editor, when the editor has unsaved
 * changes: the preview renders the editor's text, and GitHub's lines are
 * the file's on disk.
 */
async function unsavedText(file: string): Promise<{ saved: string; current: string } | undefined> {
  // The file's path comes from git, a real path; the editor's is the path the
  // workspace was opened with, which may go through a link (macOS's /var).
  const real = comparable(await realPath(file));
  let document: vscode.TextDocument | undefined;
  for (const doc of vscode.workspace.textDocuments) {
    if (doc.uri.scheme !== "file" || !doc.isDirty) continue;
    if (comparable(await realPath(doc.uri.fsPath)) === real) {
      document = doc;
      break;
    }
  }
  if (!document) return undefined;
  try {
    return { saved: await readFile(file, "utf8"), current: document.getText() };
  } catch {
    return undefined;
  }
}

/**
 * The content root through the links the workspace was opened with: git's
 * root is a real path, and the source editor's threads and commenting ranges
 * are matched against the editors' URIs. git's path when the rewritten one
 * isn't the same folder.
 */
async function openedContentRoot(fromGit: string, projectDir: string): Promise<string> {
  const opened = throughFolder(fromGit, projectDir, await realPath(projectDir));
  if (opened === fromGit) return fromGit;
  const [a, b] = await Promise.all([realPath(opened), realPath(fromGit)]);
  return comparable(a) === comparable(b) ? opened : fromGit;
}

/** `file` with its links resolved, or as it is when it can't be. */
function realPath(file: string): Promise<string> {
  return realpath(file).catch(() => file);
}

/** A failure as a sentence. */
export function messageOf(error: unknown): string {
  return error instanceof Error ? error.message : String(error);
}
