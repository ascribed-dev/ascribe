import { readFile, stat } from "node:fs/promises";
import * as path from "node:path";
import * as vscode from "vscode";
import type { ProjectServer } from "../client.js";
import { comparable, samePath, within } from "../projects.js";
import type { ProjectRegistry } from "../registry.js";
import type { ReviewController } from "../preview/review.js";
import type { TargetsResult } from "../shapes.js";
import { formatVersion } from "../version.js";
import { changesReport } from "./changes.js";
import { problemsReport, relativePath, shellWord } from "./problems.js";
import type { Publication } from "./published.js";

/**
 * The tools, as `contributes.languageModelTools` declares them. Anything an
 * agent can learn from the files on disk is `ascribe mcp`'s; these need the
 * running extension.
 */
const TOOLS = {
  problems: "ascribe_editor_problems",
  threads: "ascribe_review_threads",
  changes: "ascribe_review_changes",
} as const;

/**
 * How long the problems tool waits for the server to publish for a file that
 * changed since it last did. The server takes a few milliseconds; the rest is
 * the editor's file watcher, for a file written on disk.
 */
const WAIT_MS = 1_000;

/** What every tool takes: a file or a project's folder, or neither. */
interface ToolInput {
  path?: string;
}

/** What the extension returns for tests. */
export interface ToolsApi {
  /** Whether the tools are registered: VS Code has the API. */
  registered(): boolean;
  /** Runs a tool as an agent would, and returns its text. */
  run(name: string, input: ToolInput): Promise<string>;
}

/**
 * The language model tools: what's in the editor before it's saved, and the
 * review that's open. They only read: none starts a server, turns review on,
 * or changes a file.
 */
export class AgentTools implements vscode.Disposable {
  private readonly disposables: vscode.Disposable[] = [];
  private isRegistered = false;

  constructor(
    private readonly projects: ProjectRegistry,
    private readonly review: ReviewController,
  ) {}

  /** Registers the tools, when this VS Code has the API; without it, does nothing. */
  register(): void {
    const lm = (vscode as { lm?: Partial<typeof vscode.lm> }).lm;
    if (typeof lm?.registerTool !== "function") return;
    for (const name of Object.values(TOOLS)) {
      this.disposables.push(
        lm.registerTool<ToolInput>(name, {
          invoke: async (options) =>
            new vscode.LanguageModelToolResult([
              new vscode.LanguageModelTextPart(await this.run(name, options.input)),
            ]),
        }),
      );
    }
    this.isRegistered = true;
  }

  get api(): ToolsApi {
    return { registered: () => this.isRegistered, run: (name, input) => this.run(name, input) };
  }

  /** A tool's answer: its JSON or text, or a line saying why there's none. */
  async run(name: string, input: ToolInput): Promise<string> {
    const target = this.target(input.path);
    if (typeof target === "string") return target;
    switch (name) {
      case TOOLS.problems:
        return this.problems(target.server, target.file);
      case TOOLS.threads:
        return this.threads(target.server);
      case TOOLS.changes:
        return this.changes(target.server);
      default:
        return `Ascribe has no tool named ${name}.`;
    }
  }

  /**
   * The project and file a path names: a file in a project, a project's
   * folder, or, with no path, the project of the active editor, else the
   * first. A relative path is from a workspace folder. A string says why
   * there's none.
   */
  private target(given: string | undefined): { server: ProjectServer; file?: string } | string {
    if (this.projects.servers.length === 0) return "This workspace has no Ascribe project.";
    if (given === undefined || given.trim() === "") {
      const server = this.projects.current();
      return server ? { server } : "This workspace has no Ascribe project.";
    }
    for (const candidate of candidates(given.trim())) {
      const project = this.projects.servers.find((s) => samePath(s.project.folder, candidate));
      if (project) return { server: project };
      const server = this.projects.serverFor(vscode.Uri.file(candidate));
      if (server) return { server, file: candidate };
    }
    return `${given} isn't in an Ascribe project of this workspace.`;
  }

  private async problems(server: ProjectServer, file: string | undefined): Promise<string> {
    const name = this.projects.name(server.project);
    if (server.state !== "running") {
      const check = file ? ` ${shellWord(relativePath(server.project.folder, file))}` : "";
      return (
        `The language server of ${name} isn't running, so the editor has no problems for it. ` +
        `For the saved files, use the ascribe_check tool or run \`ascribe check --editor-build${check}\` in ${name}.`
      );
    }
    let publications: Publication[];
    let current: boolean;
    let version: number | null = null;
    if (file) {
      const waited = await this.waitForFile(server, file);
      if (typeof waited === "string") return waited;
      publications = waited.publication ? [waited.publication] : [];
      current = waited.current;
      version = openDocument(file)?.version ?? null;
    } else {
      current = await this.waitForProject(server);
      publications = server.published.all();
    }
    const scope = file ? [file] : undefined;
    const unsaved = vscode.workspace.textDocuments
      .filter((d) => d.isDirty && d.uri.scheme === "file")
      .map((d) => d.uri.fsPath)
      .filter((f) => (scope ? scope.some((s) => samePath(s, f)) : owns(server, this.projects, f)));
    const texts = await readTexts(server, publications);
    const known = await this.projectFacts(server);
    const relative = file ? ` ${shellWord(relativePath(server.project.folder, file))}` : "";
    const report = problemsReport({
      root: server.project.folder,
      publications,
      text: (f) => texts.get(comparable(f)),
      toPath: (uri) => fileOf(uri),
      ascribeVersion: server.binary ? formatVersion(server.binary.version) : "",
      editorBuild: known.editorBuild,
      filesChecked: known.files,
      filesReported: file ? 1 : known.files,
      documentVersion: version,
      current,
      unsaved,
      nextCommand: `ascribe check --editor-build --format json${relative}`,
    });
    return JSON.stringify(report);
  }

  /**
   * Waits for the server to publish for a file that changed since it last
   * did: an open document whose version it hasn't published, or a file on
   * disk written since. A string says why there's nothing to wait for.
   */
  private async waitForFile(
    server: ProjectServer,
    file: string,
  ): Promise<{ publication: Publication | undefined; current: boolean } | string> {
    if (openDocument(file)) {
      return server.published.waitFor(
        file,
        (p) => p !== undefined && p.version === openDocument(file)?.version,
        WAIT_MS,
      );
    }
    let written: number;
    try {
      written = (await stat(file)).mtimeMs;
    } catch {
      return `There's no file at ${file}.`;
    }
    // The server read the file when it started, and again at each change it
    // was told of; it publishes only a file with problems, or one that had
    // some and has none now.
    const since = server.published.get(file)?.at ?? server.since ?? 0;
    if (written <= since) return { publication: server.published.get(file), current: true };
    return server.published.waitFor(file, (p) => p !== undefined && p.at >= written, WAIT_MS);
  }

  /** Waits for the server to publish for each of the project's open documents it's behind on. */
  private async waitForProject(server: ProjectServer): Promise<boolean> {
    const open = vscode.workspace.textDocuments.filter(
      (d) =>
        d.uri.scheme === "file" &&
        owns(server, this.projects, d.uri.fsPath) &&
        (d.languageId === "markdown" || path.basename(d.uri.fsPath) === "ascribe.toml"),
    );
    const waited = await Promise.all(
      open.map((d) =>
        server.published.waitFor(
          d.uri.fsPath,
          // The server publishes for every open document, with its version.
          (p) => p !== undefined && p.version === openDocument(d.uri.fsPath)?.version,
          WAIT_MS,
        ),
      ),
    );
    return waited.every((w) => w.current);
  }

  /** The editor's build and how many source files the project has, from `ascribe/targets`. */
  private async projectFacts(
    server: ProjectServer,
  ): Promise<{ editorBuild: string | undefined; files: number }> {
    try {
      const result = (await server.request("ascribe/targets", {
        textDocument: { uri: vscode.Uri.file(server.project.config).toString() },
        kinds: ["builds", "pages", "fragments"],
      })) as TargetsResult;
      return {
        editorBuild: result.builds?.find((b) => b.editor)?.name,
        files: (result.pages?.length ?? 0) + (result.fragments?.length ?? 0),
      };
    } catch {
      return { editorBuild: undefined, files: 0 };
    }
  }

  private async threads(server: ProjectServer): Promise<string> {
    const name = this.projects.name(server.project);
    if (!this.review.baseOf(server)) return reviewOff(name);
    const connection = this.review.threads.connection(server);
    switch (connection?.state) {
      case undefined:
        return `Review of ${name} is on, but its pull request hasn't been read yet. Try again in a moment.`;
      case "none":
        return `Review of ${name} is on, but this branch has no open pull request on GitHub, so there are no review threads.`;
      case "signed-out":
        return `Review of ${name} is on, but VS Code isn't signed in to GitHub, so the threads can't be read. The person can sign in from the preview.`;
      case "error":
        return `The review threads of ${name} couldn't be read: ${connection.message}`;
      case "on":
        try {
          return await this.review.threads.list(server);
        } catch (error) {
          return `The review threads of ${name} couldn't be read: ${messageOf(error)}`;
        }
    }
  }

  private async changes(server: ProjectServer): Promise<string> {
    const name = this.projects.name(server.project);
    if (!this.review.baseOf(server)) return reviewOff(name);
    if (server.state !== "running") {
      return `Review of ${name} is on, but its language server isn't running, so the changes can't be compared.`;
    }
    try {
      const result = await this.review.changesOf(server);
      if (result.problem !== null)
        return `The changes of ${name} couldn't be compared: ${result.problem}`;
      const connection = this.review.threads.connection(server);
      const pullRequest =
        connection?.state === "on" ? connection.session.pullRequest.number : undefined;
      return JSON.stringify(changesReport(result, server.project.folder, pullRequest));
    } catch (error) {
      return `The changes of ${name} couldn't be compared: ${messageOf(error)}`;
    }
  }

  dispose(): void {
    for (const disposable of this.disposables.splice(0)) disposable.dispose();
  }
}

/** Why a review tool has nothing to say: review is off, and a tool never turns it on. */
function reviewOff(name: string): string {
  return `Review is off for ${name}. The person turns it on with Ascribe: Start Review; these tools don't.`;
}

/** The absolute paths a path the agent gave could mean: itself, or from each workspace folder. */
function candidates(given: string): string[] {
  if (path.isAbsolute(given)) return [given];
  const folders = vscode.workspace.workspaceFolders ?? [];
  return folders.map((folder) => path.join(folder.uri.fsPath, given));
}

/** The open document of a file, if there is one. */
function openDocument(file: string): vscode.TextDocument | undefined {
  return vscode.workspace.textDocuments.find(
    (d) => d.uri.scheme === "file" && samePath(d.uri.fsPath, file),
  );
}

/** Whether a file is one of the server's project's, and not a nested project's. */
function owns(server: ProjectServer, projects: ProjectRegistry, file: string): boolean {
  return (
    within(file, server.project.folder) &&
    !projects.ownedElsewhere(server.project, vscode.Uri.file(file))
  );
}

/**
 * The text of each file the publications name, theirs and their related
 * places', to count positions in: an open document's when the publication
 * is for its version, else the file on disk.
 */
async function readTexts(
  server: ProjectServer,
  publications: readonly Publication[],
): Promise<Map<string, string>> {
  const files = new Map<string, number | null>();
  for (const publication of publications) {
    files.set(publication.file, publication.version);
    for (const d of publication.diagnostics) {
      for (const r of d.relatedInformation ?? []) {
        const at = fileOf(r.location.uri);
        if (at !== undefined && !files.has(at))
          files.set(at, server.published.get(at)?.version ?? null);
      }
    }
  }
  const texts = new Map<string, string>();
  await Promise.all(
    [...files].map(async ([file, version]) => {
      const document = openDocument(file);
      const same = version === null ? !document?.isDirty : document?.version === version;
      if (document && same) {
        texts.set(comparable(file), document.getText());
        return;
      }
      try {
        texts.set(comparable(file), await readFile(file, "utf8"));
      } catch {
        // Gone: its positions are the protocol's.
      }
    }),
  );
  return texts;
}

/** A `file:` URI's path. */
function fileOf(uri: string): string | undefined {
  try {
    const parsed = vscode.Uri.parse(uri, true);
    return parsed.scheme === "file" ? parsed.fsPath : undefined;
  } catch {
    return undefined;
  }
}

function messageOf(error: unknown): string {
  return error instanceof Error ? error.message : String(error);
}
