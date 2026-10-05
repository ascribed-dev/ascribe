// Moving a line number between a commit and the working tree, through the
// line changes `git diff --unified=0` reports.
import { existsSync } from "node:fs";
import { readFile } from "node:fs/promises";
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

/** The text of both sides of a diff, line by line, for telling replaced lines from deleted ones. */
export interface Sides {
  old: readonly string[];
  new: readonly string[];
}

/**
 * Where line `line` of the old side is on the new side. A line the diff
 * replaced maps onto the line that replaced it, with `replaced` set; a line it
 * only deleted maps to `undefined`. Hunks come from a diff with no context
 * lines.
 *
 * A hunk can delete some lines and reword others, so with `sides` each old
 * line of a hunk is paired with the new line most like it, in order, and an
 * old line like none of them was deleted. Without `sides`, or for a hunk too
 * large to compare, old lines pair with new ones by position, clamped to the
 * last.
 */
export function shiftLine(
  hunks: readonly Hunk[],
  line: number,
  sides?: Sides,
): Shifted | undefined {
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
      const pairs = sides && pairLines(hunk, sides);
      const offset =
        pairs === undefined
          ? Math.min(line - hunk.oldStart, hunk.newCount - 1)
          : pairs[line - hunk.oldStart];
      return offset === undefined ? undefined : { line: hunk.newStart + offset, replaced: true };
    }
    if (line >= end) delta += hunk.newCount - hunk.oldCount;
  }
  return { line: line + delta, replaced: false };
}

/** How alike two lines must be, by their words, for one to be the other reworded. */
const ALIKE = 0.3;

const pairings = new WeakMap<Hunk, (number | undefined)[] | undefined>();

/**
 * For each old line of a replacing hunk, the offset of the new line it became,
 * or `undefined` if it was deleted: the pairing, in order, that makes the
 * paired lines most alike. A hunk of one line for one line keeps its pair
 * however much it changed, since nothing else could have replaced it.
 * `undefined` when the hunk is too large to compare.
 */
function pairLines(hunk: Hunk, sides: Sides): (number | undefined)[] | undefined {
  if (pairings.has(hunk)) return pairings.get(hunk);
  const n = hunk.oldCount;
  const m = hunk.newCount;
  let pairs: (number | undefined)[] | undefined;
  if (n === 1 && m === 1) pairs = [0];
  else if (n * m <= 250_000) {
    const oldWords = sides.old.slice(hunk.oldStart - 1, hunk.oldStart - 1 + n).map(words);
    const newWords = sides.new.slice(hunk.newStart - 1, hunk.newStart - 1 + m).map(words);
    // `best(i, j)`: the most likeness pairing old lines from `i` on with new lines from `j` on.
    const width = m + 1;
    const table = new Float64Array((n + 1) * width);
    const best = (i: number, j: number): number => table[i * width + j] ?? 0;
    const like = (i: number, j: number): number => {
      const score = likeness(oldWords[i] ?? [], newWords[j] ?? []);
      return score >= ALIKE ? score : 0;
    };
    for (let i = n - 1; i >= 0; i--) {
      for (let j = m - 1; j >= 0; j--) {
        const score = like(i, j);
        table[i * width + j] = Math.max(
          score > 0 ? best(i + 1, j + 1) + score : 0,
          best(i + 1, j),
          best(i, j + 1),
        );
      }
    }
    pairs = [];
    let i = 0;
    let j = 0;
    while (i < n) {
      const score = j < m ? like(i, j) : 0;
      if (score > 0 && best(i, j) === best(i + 1, j + 1) + score) {
        pairs.push(j);
        i++;
        j++;
      } else if (j >= m || best(i, j) === best(i + 1, j)) {
        pairs.push(undefined);
        i++;
      } else {
        j++;
      }
    }
  }
  pairings.set(hunk, pairs);
  return pairs;
}

/** A line's words, in lower case. */
function words(line: string | undefined): string[] {
  return (line ?? "").toLowerCase().match(/[\p{L}\p{N}]+/gu) ?? [];
}

/** How many words two lines share, from 0 to 1: twice the shared count over the total. */
function likeness(a: readonly string[], b: readonly string[]): number {
  if (a.length === 0 || b.length === 0) return 0;
  const counts = new Map<string, number>();
  for (const word of a) counts.set(word, (counts.get(word) ?? 0) + 1);
  let shared = 0;
  for (const word of b) {
    const count = counts.get(word) ?? 0;
    if (count > 0) {
      shared++;
      counts.set(word, count - 1);
    }
  }
  return (2 * shared) / (a.length + b.length);
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
  let sides: Sides | undefined;
  if (hunks.some((hunk) => hunk.oldCount > 0 && hunk.newCount > 0)) {
    const atCommitText = (await gitMaybe(root, ["cat-file", "blob", `${commit}:${file}`])) ?? "";
    const inWorktree = await readFile(path.join(root, ...file.split("/")), "utf8").catch(() => "");
    const [from, to] =
      direction === "to-worktree" ? [atCommitText, inWorktree] : [inWorktree, atCommitText];
    sides = { old: from.split(/\r?\n/), new: to.split(/\r?\n/) };
  }
  return { map: (line) => shiftLine(hunks, line, sides), identity: false };
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

/**
 * The hunks that turn `before` into `after`, line by line, as
 * `git diff --unified=0` would report them: for text in hand, such as a file
 * and an editor's unsaved copy of it. Lines common to both ends are skipped
 * first; a middle too large to compare line by line is one hunk.
 */
export function lineHunks(before: string, after: string): Hunk[] {
  const a = before.split(/\r?\n/);
  const b = after.split(/\r?\n/);
  let start = 0;
  while (start < a.length && start < b.length && a[start] === b[start]) start++;
  let endA = a.length;
  let endB = b.length;
  while (endA > start && endB > start && a[endA - 1] === b[endB - 1]) {
    endA--;
    endB--;
  }
  const n = endA - start;
  const m = endB - start;
  if (n === 0 && m === 0) return [];
  if (n * m > 4_000_000 || n === 0 || m === 0) return [hunk(start, n, start, m)];
  // The longest common subsequence of the middles, by dynamic programming:
  // `lcs(i, j)` is its length for the lines from `start + i` and `start + j` on.
  const width = m + 1;
  const table = new Uint32Array((n + 1) * width);
  const lcs = (i: number, j: number): number => table[i * width + j] ?? 0;
  for (let i = n - 1; i >= 0; i--) {
    for (let j = m - 1; j >= 0; j--) {
      table[i * width + j] =
        a[start + i] === b[start + j]
          ? lcs(i + 1, j + 1) + 1
          : Math.max(lcs(i + 1, j), lcs(i, j + 1));
    }
  }
  const hunks: Hunk[] = [];
  let i = 0;
  let j = 0;
  let fromA = 0;
  let fromB = 0;
  const flush = (): void => {
    if (i > fromA || j > fromB)
      hunks.push(hunk(start + fromA, i - fromA, start + fromB, j - fromB));
  };
  while (i < n || j < m) {
    if (i < n && j < m && a[start + i] === b[start + j]) {
      flush();
      i++;
      j++;
      fromA = i;
      fromB = j;
    } else if (j >= m || (i < n && lcs(i + 1, j) >= lcs(i, j + 1))) {
      i++;
    } else {
      j++;
    }
  }
  flush();
  return hunks;
}

/**
 * A hunk of `count` lines after `at` lines (counted from 0) on each side,
 * numbered as `git diff --unified=0` does: an empty side names the line
 * before it.
 */
function hunk(atA: number, countA: number, atB: number, countB: number): Hunk {
  return {
    oldStart: countA === 0 ? atA : atA + 1,
    oldCount: countA,
    newStart: countB === 0 ? atB : atB + 1,
    newCount: countB,
  };
}
