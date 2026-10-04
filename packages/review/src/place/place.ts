// Placing review threads on rendered blocks. A thread is on a repository path
// and a line at a commit; a block's anchor is a content path and lines in the
// working tree. Locating moves each thread into the working tree's terms, once
// per pull request; placing matches the result to one page's anchors.
import type { Side, Thread } from "../shared/types.js";
import { anchorKey, parseSource, type Anchor } from "./anchor.js";
import { lineMap, linesAt, type LineMap } from "./lines.js";

/** A rendered page, as placing needs it. */
export interface PageRef {
  /** The build the page is rendered for. */
  build: string;
  /** The page's own content path. */
  path: string;
  /** Every anchored block on the page, in the page's order. */
  anchors: Anchor[];
  /**
   * Where the diff's removed blocks on this page were (each removed or moved
   * change's `was`), for threads on removed text.
   */
  removed?: Anchor[] | undefined;
}

/** Why a thread isn't on a block. */
export type DetachReason =
  /** The line it's on isn't in the working tree any more. */
  | "line-gone"
  /** It's on a whole file, not a line. */
  | "file"
  /** No block on the page holds its line. */
  | "no-block";

/** A thread, located in the working tree. */
export interface LocatedThread extends Thread {
  /** The file's content path. */
  path: string;
  /**
   * The thread's lines: in the working tree for the right side, at the diff's
   * base for the left side. `undefined` when it's detached.
   */
  lines: { first: number; last: number } | undefined;
  /** Why it's detached, or `undefined` when it's on lines. */
  detached: DetachReason | undefined;
}

/** A page's threads. */
export interface PlacedThreads {
  /** Threads on the page's blocks, by anchor, in the page's order. */
  blocks: { anchor: Anchor; threads: LocatedThread[] }[];
  /** Threads on removed text, by the removed block's anchor. */
  removed: { anchor: Anchor; threads: LocatedThread[] }[];
  /** Threads on the page's files that aren't on any block. */
  detached: LocatedThread[];
}

/** What locating needs to know about the checkout and the pull request. */
export interface LocateContext {
  /** The repository's top-level directory. */
  root: string;
  /** The content root's path in the repository, with a trailing `/`, or `""` at the root. */
  contentPrefix: string;
  /** The pull request's head commit, which current threads' lines are counted at. */
  headOid: string;
}

/**
 * Locates threads in the working tree. Threads on files outside the content
 * root are left out; every other thread is returned, on lines or detached.
 */
export async function locateThreads(
  threads: readonly Thread[],
  context: LocateContext,
): Promise<LocatedThread[]> {
  const maps = new Map<string, Promise<LineMap>>();
  const mapFor = (commit: string, file: string): Promise<LineMap> => {
    const key = `${commit}\0${file}`;
    let map = maps.get(key);
    if (map === undefined) {
      map = lineMap(context.root, commit, file, "to-worktree");
      maps.set(key, map);
    }
    return map;
  };
  const located: LocatedThread[] = [];
  for (const thread of threads) {
    if (!thread.repositoryPath.startsWith(context.contentPrefix)) continue;
    const path = thread.repositoryPath.slice(context.contentPrefix.length);
    const result = (lines: LocatedThread["lines"], detached?: DetachReason, quote?: string) =>
      located.push({
        ...thread,
        path,
        lines,
        detached,
        quote: quote ?? thread.quote,
      });

    if (thread.kind === "conversation" || thread.side === "LEFT") {
      const last = thread.kind === "review" && thread.outdated ? undefined : thread.line;
      if (thread.subject === "file") result(undefined, "file");
      else if (last === null || last === undefined) {
        result(undefined, "line-gone", hunkQuote(thread, thread.side));
      } else {
        const first = thread.startLine ?? last;
        result({ first, last }, undefined, thread.quote ?? hunkQuote(thread, thread.side));
      }
      continue;
    }
    if (thread.subject === "file") {
      result(undefined, "file");
      continue;
    }
    const commit = thread.outdated ? thread.originalCommit : context.headOid;
    const last = thread.outdated ? thread.originalLine : thread.line;
    const start = (thread.outdated ? thread.originalStartLine : thread.startLine) ?? last;
    if (commit === undefined || last === null || start === null) {
      result(undefined, "line-gone", hunkQuote(thread, "RIGHT"));
      continue;
    }
    const original = async () =>
      (await linesAt(context.root, commit, thread.repositoryPath, start, last)) ??
      hunkQuote(thread, "RIGHT");
    const map = await mapFor(commit, thread.repositoryPath);
    const mappedLast = map.map(last);
    if (mappedLast === undefined) {
      result(undefined, "line-gone", await original());
      continue;
    }
    // When the first line went but the last stayed, the thread keeps what's left.
    let first = mappedLast.line;
    let replaced = mappedLast.replaced;
    for (let line = start; line < last; line++) {
      const mapped = map.map(line);
      if (mapped === undefined) {
        replaced = true;
        continue;
      }
      replaced ||= mapped.replaced;
      if (first === mappedLast.line) first = Math.min(mapped.line, mappedLast.line);
    }
    // Reworded text keeps its place, marked outdated, with what it said.
    const outdated = thread.outdated || replaced;
    located.push({
      ...thread,
      path,
      lines: { first, last: mappedLast.line },
      detached: undefined,
      outdated,
      quote: outdated ? await original() : thread.quote,
    });
  }
  return located;
}

/** The thread's lines, from the end of GitHub's diff hunk. */
function hunkQuote(thread: Thread, side: Side): string | undefined {
  if (thread.diffHunk === undefined) return undefined;
  const skip = side === "LEFT" ? "+" : "-";
  const lines = thread.diffHunk
    .split(/\r?\n/)
    .filter((line) => !line.startsWith("@@") && !line.startsWith(skip))
    .map((line) => line.slice(1));
  const last = thread.outdated ? thread.originalLine : thread.line;
  const first = (thread.outdated ? thread.originalStartLine : thread.startLine) ?? last;
  const count = first !== null && last !== null ? last - first + 1 : 1;
  return lines.slice(-Math.max(1, count)).join("\n");
}

/**
 * The threads on one page. A thread goes on the smallest block that holds all
 * its lines (or, failing that, its last line), on every place that block is
 * shown: a fragment included twice is two blocks with the same source. A
 * thread on one of the page's files that no block holds is detached, as are
 * detached threads on those files. Conversation comments marked for another
 * build are left out.
 */
export function placeOnPage(page: PageRef, threads: readonly LocatedThread[]): PlacedThreads {
  const anchors = parseAnchors(page.anchors);
  const removedAnchors = parseAnchors(page.removed ?? []);
  const files = new Set([page.path, ...anchors.map((a) => a.range.path)]);
  const onBlocks = new Map<string, LocatedThread[]>();
  const onRemoved = new Map<string, LocatedThread[]>();
  const detached: LocatedThread[] = [];

  for (const thread of threads) {
    const build = thread.marker?.build;
    if (build !== undefined && build !== page.build) continue;
    if (thread.lines === undefined) {
      if (files.has(thread.path)) detached.push(thread);
      continue;
    }
    const left = thread.kind === "review" && thread.side === "LEFT";
    const matches = holding(left ? removedAnchors : anchors, thread.path, thread.lines);
    if (matches.length === 0) {
      if (files.has(thread.path)) detached.push({ ...thread, detached: "no-block" });
      continue;
    }
    const into = left ? onRemoved : onBlocks;
    for (const match of matches) {
      const key = anchorKey(match.anchor);
      const list = into.get(key) ?? [];
      list.push(thread);
      into.set(key, list);
    }
  }

  const collect = (list: ParsedAnchor[], by: Map<string, LocatedThread[]>) => {
    const out: PlacedThreads["blocks"] = [];
    const seen = new Set<string>();
    for (const { anchor } of list) {
      const key = anchorKey(anchor);
      const found = by.get(key);
      if (found === undefined || seen.has(key)) continue;
      seen.add(key);
      out.push({ anchor, threads: found });
    }
    return out;
  };
  return {
    blocks: collect(anchors, onBlocks),
    removed: collect(removedAnchors, onRemoved),
    detached,
  };
}

interface ParsedAnchor {
  anchor: Anchor;
  range: { path: string; first: number; last: number };
}

function parseAnchors(anchors: readonly Anchor[]): ParsedAnchor[] {
  const parsed: ParsedAnchor[] = [];
  for (const anchor of anchors) {
    const range = parseSource(anchor.source);
    if (range !== undefined) parsed.push({ anchor, range });
  }
  return parsed;
}

/** The smallest anchors on `path` holding `lines`, else the smallest holding its last line. */
function holding(
  anchors: readonly ParsedAnchor[],
  path: string,
  lines: { first: number; last: number },
): ParsedAnchor[] {
  const onFile = anchors.filter((a) => a.range.path === path);
  for (const [first, last] of [
    [lines.first, lines.last],
    [lines.last, lines.last],
  ] as const) {
    const containing = onFile.filter((a) => a.range.first <= first && a.range.last >= last);
    if (containing.length === 0) continue;
    const size = Math.min(...containing.map((a) => a.range.last - a.range.first));
    return containing.filter((a) => a.range.last - a.range.first === size);
  }
  return [];
}
