// Answering the overlay's requests (`OverlayHost`) from a review session, for
// a host that keeps GitHub to itself and passes the overlay's requests on as
// messages: the page preview's extension, or the site preview's dev server.
// The parameters arrive as untyped messages, so each is checked here.
import { anchorKey, parseSource, type Anchor } from "../place/anchor.js";
import { lineHunks, shiftLine, type Hunk, type Sides } from "../place/lines.js";
import type { PageRef, PlacedThreads } from "../place/place.js";
import type { FormattedPiece } from "../shapes.js";
import { ReviewError } from "../shared/errors.js";
import { gitMaybe } from "../shared/git.js";
import { parseRemote } from "./repository.js";
import type { PullRequestInfo, ReviewEvent, ReviewSession } from "./session.js";

/** The overlay's requests, one per `OverlayHost` method that reads or writes threads. */
export type OverlayMethod =
  "load" | "commentTarget" | "comment" | "reply" | "allThreads" | "resolve" | "submit" | "discard";

export const OVERLAY_METHODS: readonly OverlayMethod[] = [
  "load",
  "commentTarget",
  "comment",
  "reply",
  "allThreads",
  "resolve",
  "submit",
  "discard",
];

/** A changed page as `ascribe diff` lists it, with its title when the host knows it. */
export interface ChangedPageRef {
  path: string;
  status: "added" | "removed" | "changed";
  because: readonly string[];
  title?: string | null;
  /** The title formatted, when its field sets `inline = "code"`. */
  formatted_title?: readonly FormattedPiece[] | null;
}

/** What answering a request needs. */
export interface RequestContext {
  session: ReviewSession;
  /** The page the overlay is on: its build and content path. */
  page: { build: string; path: string } | undefined;
  /** The page as the overlay last read it, with its anchors: comments are made on it. */
  lastPage: PageRef | undefined;
  /** The changed pages, for which pages show a thread. */
  changedPages(): Promise<readonly ChangedPageRef[]>;
  /**
   * For a host whose pages are rendered from an editor's unsaved text: a
   * file's text on disk (`saved`) and in the editor (`current`), by content
   * path, or `undefined` when they're the same. The session counts lines on
   * disk, so the page's blocks are moved to their lines there, and back.
   */
  unsaved?(path: string): Promise<{ saved: string; current: string } | undefined>;
}

/** An answer: the result, the page read (for `load`), and whether the request changed the threads. */
export interface Answer {
  result: unknown;
  page?: PageRef;
  changed: boolean;
}

/** Answers one of the overlay's requests. Rejects with a `ReviewError`. */
export async function answerRequest(
  context: RequestContext,
  method: OverlayMethod,
  params: Record<string, unknown>,
): Promise<Answer> {
  const { session } = context;
  switch (method) {
    case "load": {
      const page = context.page;
      if (!page) throw new ReviewError("not-found", "There's no page to review here.");
      const shown = anchors(params["anchors"]);
      const edits = await unsavedEdits(context, shown);
      // The page's blocks at their lines on disk, and the way back.
      const back = new Map<string, Anchor>();
      for (const block of shown) {
        const saved = toSaved(edits, block, false);
        if (saved !== undefined && !back.has(anchorKey(saved))) back.set(anchorKey(saved), block);
      }
      const ref: PageRef = {
        build: page.build,
        path: page.path,
        anchors: edits.size === 0 ? shown : [...back.keys()].map((key) => fromKey(key)),
        removed: anchors(params["removed"]),
      };
      const [placed, pending, viewer] = await Promise.all([
        session.threads(ref),
        session.pending(),
        session.viewer(),
      ]);
      const threads: PlacedThreads =
        edits.size === 0
          ? placed
          : {
              ...placed,
              blocks: placed.blocks.map((block) => ({
                ...block,
                anchor: back.get(anchorKey(block.anchor)) ?? block.anchor,
              })),
            };
      const pr = session.pullRequest;
      return {
        result: { pullRequest: { number: pr.number, url: pr.url }, threads, pending, viewer },
        page: ref,
        changed: false,
      };
    }
    case "commentTarget": {
      const shown = anchor(params["anchor"]);
      const saved = toSaved(await unsavedEdits(context, [shown]), shown, true);
      if (saved === undefined)
        return { result: { kind: "push-first", message: UNSAVED }, changed: false };
      return { result: await session.commentTarget(saved), changed: false };
    }
    case "allThreads": {
      const [threads, pages] = await Promise.all([
        session.allThreads(),
        context.changedPages().catch(() => []),
      ]);
      return {
        result: threads.map((thread) => ({ thread, pages: pagesShowing(thread.path, pages) })),
        changed: false,
      };
    }
    case "comment": {
      const shown = anchor(params["anchor"]);
      const target = toSaved(await unsavedEdits(context, [shown]), shown, true);
      if (target === undefined) throw new ReviewError("push-first", UNSAVED);
      const page = context.lastPage ?? {
        build: context.page?.build ?? "",
        path: context.page?.path ?? "",
        anchors: [target],
      };
      const quote = typeof params["quote"] === "string" ? params["quote"] : undefined;
      return {
        result: await session.comment(target, text(params["body"]), page, quote),
        changed: true,
      };
    }
    case "reply":
      return {
        result: await session.reply(
          text(params["threadId"]),
          text(params["body"]),
          params["when"] === "now" ? "now" : "withReview",
        ),
        changed: true,
      };
    case "resolve":
      await session.resolve(text(params["threadId"]), params["resolved"] === true);
      return { result: null, changed: true };
    case "submit": {
      const body = typeof params["body"] === "string" ? params["body"] : undefined;
      await session.submit(event(params["event"]), body);
      return { result: null, changed: true };
    }
    case "discard":
      await session.discard();
      return { result: null, changed: true };
  }
}

/** A file's unsaved edits: the line changes from the editor's text to the text on disk. */
interface Edit {
  hunks: Hunk[];
  sides: Sides;
}

const UNSAVED = "This block has changes that aren't saved. Save the file, then comment.";

/** The line changes from each shown file's text in the editor to its text on disk, where they differ. */
async function unsavedEdits(
  context: RequestContext,
  shown: readonly Anchor[],
): Promise<Map<string, Edit>> {
  const edits = new Map<string, Edit>();
  if (!context.unsaved) return edits;
  const files = new Set<string>();
  for (const block of shown) {
    const range = parseSource(block.source);
    if (range) files.add(range.path);
    for (const include of block.via) {
      const at = parseInclude(include);
      if (at) files.add(at.path);
    }
  }
  for (const file of files) {
    const text = await context.unsaved(file);
    if (text === undefined) continue;
    const hunks = lineHunks(text.current, text.saved);
    if (hunks.length > 0) {
      edits.set(file, {
        hunks,
        sides: { old: text.current.split(/\r?\n/), new: text.saved.split(/\r?\n/) },
      });
    }
  }
  return edits;
}

/**
 * A block's anchor with its lines on disk. `strict` (for commenting) wants
 * every line of the block saved as it is; otherwise an edited block takes
 * the lines its text has on disk. `undefined` when nothing of it is saved.
 */
function toSaved(
  edits: ReadonlyMap<string, Edit>,
  block: Anchor,
  strict: boolean,
): Anchor | undefined {
  if (edits.size === 0) return block;
  const range = parseSource(block.source);
  if (range === undefined) return block;
  const edit = edits.get(range.path);
  let source = block.source;
  if (edit !== undefined) {
    const lines: number[] = [];
    for (let line = range.first; line <= range.last; line++) {
      const moved = shiftLine(edit.hunks, line, edit.sides);
      if (moved === undefined || moved.replaced) {
        if (strict) return undefined;
        if (moved === undefined) continue;
      }
      lines.push(moved.line);
    }
    if (lines.length === 0) return undefined;
    const colon = block.source.lastIndexOf(":");
    source = `${block.source.slice(0, colon)}:${Math.min(...lines)}-${Math.max(...lines)}`;
  }
  const via: string[] = [];
  for (const include of block.via) {
    const at = parseInclude(include);
    const includeEdit = at && edits.get(at.path);
    if (!at || !includeEdit) {
      via.push(include);
      continue;
    }
    const moved = shiftLine(includeEdit.hunks, at.line, includeEdit.sides);
    if (moved === undefined || (strict && moved.replaced)) return undefined;
    via.push(`${include.slice(0, include.lastIndexOf(":"))}:${moved.line}`);
  }
  return { source, via };
}

/** An include, `<path>:<line>`, parsed, with its path decoded. */
function parseInclude(include: string): { path: string; line: number } | undefined {
  const colon = include.lastIndexOf(":");
  const line = Number(include.slice(colon + 1));
  if (colon <= 0 || !Number.isInteger(line) || line < 1) return undefined;
  const range = parseSource(`${include.slice(0, colon)}:1-1`);
  return range && { path: range.path, line };
}

/** The anchor `anchorKey` wrote. */
function fromKey(key: string): Anchor {
  const [source = "", ...via] = key.split(" ");
  return { source, via };
}

/**
 * The changed pages that show a file (a content path): the file's own page,
 * and the pages that changed through it (an include). `[]` when none does.
 */
export function pagesShowing(
  file: string,
  pages: readonly ChangedPageRef[],
): { path: string; title: string | null; formatted_title: FormattedPiece[] | null }[] {
  return pages
    .filter((page) => page.status !== "removed")
    .filter((page) => page.path === file || page.because.includes(file))
    .sort((a, b) => Number(b.path === file) - Number(a.path === file))
    .map((page) => ({
      path: page.path,
      title: page.title ?? null,
      formatted_title: page.formatted_title ? [...page.formatted_title] : null,
    }));
}

/**
 * The git revision to compare with for a pull request's base: the remote
 * branch, from the remote whose URL is the pull request's repository, when
 * there is one; else the branch's name.
 */
export function pullRequestBase(
  remotes: readonly { name: string; repository: string | undefined }[],
  repository: string,
  baseRefName: string,
): string {
  const remote =
    remotes.find((r) => r.repository === repository && r.name === "upstream") ??
    remotes.find((r) => r.repository === repository && r.name === "origin") ??
    remotes.find((r) => r.repository === repository);
  return remote ? `${remote.name}/${baseRefName}` : baseRefName;
}

/**
 * The git revision for the pull request's base in the checkout at `root`:
 * `origin/main` when that remote branch exists, else the branch's name.
 */
export async function baseRevision(root: string, pr: PullRequestInfo): Promise<string> {
  const out = (await gitMaybe(root, ["remote", "-v"]).catch(() => undefined)) ?? "";
  const remotes = out
    .split("\n")
    .map((line) => /^(\S+)\s+(\S+)\s+\(fetch\)$/.exec(line))
    .filter((match) => match !== null)
    .map((match) => {
      const repository = parseRemote(match[2] ?? "");
      return { name: match[1] ?? "", repository: repository && repoKey(repository) };
    });
  const revision = pullRequestBase(remotes, repoKey(pr.repository), pr.baseRefName);
  if (revision !== pr.baseRefName) {
    const exists = await gitMaybe(root, ["rev-parse", "--verify", "--quiet", revision]).catch(
      () => undefined,
    );
    if (exists !== undefined) return revision;
  }
  return pr.baseRefName;
}

function repoKey(repository: { host: string; owner: string; name: string }): string {
  return `${repository.host}/${repository.owner}/${repository.name}`.toLowerCase();
}

const EVENTS: readonly ReviewEvent[] = ["COMMENT", "APPROVE", "REQUEST_CHANGES"];

function event(value: unknown): ReviewEvent {
  if (EVENTS.includes(value as ReviewEvent)) return value as ReviewEvent;
  throw new ReviewError("refused", `A review can't be submitted as ${String(value)}.`);
}

function text(value: unknown): string {
  if (typeof value !== "string") throw new ReviewError("refused", "The request is missing text.");
  return value;
}

function anchor(value: unknown): Anchor {
  const record = value as Partial<Anchor> | null;
  if (typeof record?.source !== "string") {
    throw new ReviewError("refused", "The request is missing a block.");
  }
  const via = Array.isArray(record.via) ? record.via.filter((v) => typeof v === "string") : [];
  return { source: record.source, via };
}

function anchors(value: unknown): Anchor[] {
  return Array.isArray(value) ? value.map(anchor) : [];
}
