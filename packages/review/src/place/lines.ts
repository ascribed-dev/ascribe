// Moving a line number between a commit and the working tree, through the
// line changes `git diff --unified=0` reports.
import { existsSync } from "node:fs";
import path from "node:path";
import { git, gitMaybe } from "../shared/git.js";

/** One hunk of a unified diff: lines `oldStart`.. on one side became `newStart`.. on the other. */
export interface Hunk {
  oldStart: number;
  oldCount: number;
  newStart: number;
  newCount: number;
}

/** The hunks of a unified diff of one file, in order. */
export function parseHunks(diff: string): Hunk[] {
  const hunks: Hunk[] = [];
  for (const match of diff.matchAll(/^@@ -(\d+)(?:,(\d+))? \+(\d+)(?:,(\d+))? @@/gm)) {
    hunks.push({
      oldStart: Number(match[1]),
      oldCount: match[2] === undefined ? 1 : Number(match[2]),
      newStart: Number(match[3]),
      newCount: match[4] === undefined ? 1 : Number(match[4]),
    });
  }
  return hunks;
}

/** Where a line is on the other side of a diff. */
export interface Shifted {
  line: number;
  /** Whether the diff replaced the line: its text changed, and `line` is where the new text is. */
  replaced: boolean;
}

/**
 * Where line `line` of the old side is on the new side. A line the diff
 * replaced maps onto the lines that replaced it, at the same offset clamped to
 * them, with `replaced` set; a line it only deleted maps to `undefined`. Hunks
 * come from a diff with no context lines.
 */
export function shiftLine(hunks: readonly Hunk[], line: number): Shifted | undefined {
  let delta = 0;
  for (const hunk of hunks) {
    if (hunk.oldCount === 0) {
      // Lines inserted after old line `oldStart`.
      if (line > hunk.oldStart) delta += hunk.newCount;
      continue;
    }
    const end = hunk.oldStart + hunk.oldCount;
    if (line >= hunk.oldStart && line < end) {
      if (hunk.newCount === 0) return undefined;
      const offset = Math.min(line - hunk.oldStart, hunk.newCount - 1);
      return { line: hunk.newStart + offset, replaced: true };
    }
    if (line >= end) delta += hunk.newCount - hunk.oldCount;
  }
  return { line: line + delta, replaced: false };
}

/** Maps line numbers of a file from one side to the other. */
export interface LineMap {
  /** Where the line is on the other side, or `undefined` if it was deleted. */
  map(line: number): Shifted | undefined;
  /** Whether the file is the same on both sides. */
  identity: boolean;
}

const NOTHING: LineMap = { map: () => undefined, identity: false };

/**
 * A map of the lines of the repository-relative `file` between `commit` and
 * the working tree, in the direction given. Every line maps to nothing when
 * the commit isn't in the repository or the file is missing on either side.
 */
export async function lineMap(
  root: string,
  commit: string,
  file: string,
  direction: "to-worktree" | "to-commit",
): Promise<LineMap> {
  const atCommit = await gitMaybe(root, ["cat-file", "-e", `${commit}:${file}`]);
  if (atCommit === undefined) return NOTHING;
  if (!existsSync(path.join(root, ...file.split("/")))) return NOTHING;
  const diff = await git(root, [
    "diff",
    "--no-color",
    "--no-ext-diff",
    "--no-renames",
    "--unified=0",
    ...(direction === "to-commit" ? ["-R"] : []),
    commit,
    "--",
    `:(literal)${file}`,
  ]);
  const hunks = parseHunks(diff);
  if (hunks.length === 0) return { map: (line) => ({ line, replaced: false }), identity: true };
  return { map: (line) => shiftLine(hunks, line), identity: false };
}

/** Lines `first`..`last` of the repository-relative `file` at `commit`, or `undefined`. */
export async function linesAt(
  root: string,
  commit: string,
  file: string,
  first: number,
  last: number,
): Promise<string | undefined> {
  const text = await gitMaybe(root, ["cat-file", "blob", `${commit}:${file}`]);
  if (text === undefined) return undefined;
  const lines = text.split(/\r?\n/);
  if (last > lines.length) return undefined;
  return lines.slice(first - 1, last).join("\n");
}
