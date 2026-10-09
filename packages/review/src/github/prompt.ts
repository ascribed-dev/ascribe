// Gathering what a prompt about review threads needs, on a host's Node side:
// the session's threads, the text of a thread's lines, the pages that show
// it, and where the project is. `../prompt` builds the prompt from them.
import { existsSync, realpathSync } from "node:fs";
import path from "node:path";
import {
  openThreadsPrompt,
  threadPrompt,
  type PromptProject,
} from "../prompt/index.js";
import { ReviewError } from "../shared/errors.js";
import type { PromptRequest } from "../shared/types.js";
import { pagesShowing, type ChangedPageRef } from "./requests.js";
import { contentPrefixOf, type ReviewSession } from "./session.js";

/** What building a prompt about threads needs from its host. */
export interface ThreadPromptContext {
  session: ReviewSession;
  /** The changed pages, for which pages show a thread's block. */
  changedPages(): Promise<readonly ChangedPageRef[]>;
  /** Where the project is (`promptProject`). */
  project: PromptProject;
  /**
   * A file's text as saved, by content path, and whether an editor has
   * unsaved changes to it; `undefined` when it can't be read.
   */
  readSource(path: string): Promise<{ text: string; unsaved: boolean } | undefined>;
}

/**
 * The prompt about one thread, or about every open thread. `undefined` when
 * there's no open thread to prompt about; rejects with a `ReviewError` when
 * the thread isn't on the pull request.
 */
export async function buildThreadsPrompt(
  context: ThreadPromptContext,
  request: Extract<PromptRequest, { kind: "thread" | "open-threads" }>,
): Promise<string | undefined> {
  const threads = await context.session.allThreads();
  if (request.kind === "open-threads") {
    return openThreadsPrompt({
      project: context.project,
      pullRequest: context.session.pullRequest.number,
      threads,
    });
  }
  const thread = threads.find((t) => t.id === request.threadId);
  if (thread === undefined) {
    throw new ReviewError("not-found", "That comment isn't on the pull request any more.");
  }
  const onLines =
    thread.lines !== undefined &&
    thread.detached === undefined &&
    !(thread.kind === "review" && thread.side === "LEFT");
  const [file, pages] = await Promise.all([
    context.readSource(thread.path),
    context.changedPages().catch(() => []),
  ]);
  let source: { first: number; last: number; text: string } | undefined;
  if (onLines && thread.lines && file) {
    const lines = file.text.split(/\r?\n/);
    const { first, last } = thread.lines;
    if (last <= lines.length) {
      source = { first, last, text: lines.slice(first - 1, last).join("\n") };
    }
  }
  // A fragment's block shows on the pages that include it; a page's on itself.
  const showing = pagesShowing(thread.path, pages);
  const shownOn = showing.some((p) => p.path === thread.path) ? [] : showing.map((p) => p.path);
  return threadPrompt({
    project: context.project,
    thread,
    source,
    unsaved: file?.unsaved ?? false,
    shownOn,
  });
}

/**
 * Where the project whose `ascribe.toml` is in `projectDir` is, as the
 * binary's prompts find it: its repository is the nearest folder at or above
 * it with a `.git`, and its `AGENTS.md` is the project folder's, or else the
 * repository's.
 */
export function promptProject(projectDir: string): PromptProject {
  const real = (p: string): string => {
    try {
      return realpathSync.native(p);
    } catch {
      return path.resolve(p);
    }
  };
  const dir = real(projectDir);
  let root: string | undefined;
  for (let at = dir; ; at = path.dirname(at)) {
    if (existsSync(path.join(at, ".git"))) {
      root = at;
      break;
    }
    if (path.dirname(at) === at) break;
  }
  const folder = root === undefined ? "" : path.relative(root, dir).split(path.sep).join("/");
  let contentRoot: string;
  try {
    const base = root ?? dir;
    const contentDir = path.join(base, ...contentPrefixOf(base, dir).split("/"));
    contentRoot = path.relative(dir, contentDir).split(path.sep).join("/");
  } catch {
    // No `ascribe.toml` to read: the content root's default.
    contentRoot = "docs";
  }
  const agents = existsSync(path.join(dir, "AGENTS.md"))
    ? folder === ""
      ? "AGENTS.md"
      : `${folder}/AGENTS.md`
    : root !== undefined && existsSync(path.join(root, "AGENTS.md"))
      ? "AGENTS.md"
      : undefined;
  return { folder, contentRoot, agents };
}
