// No paragraph of a guide or a reference page runs over 150 words, so a page
// isn't a wall of text (#1032): a long one becomes a lead sentence and a list,
// a table, or paragraphs of their own. Measured on each file under
// docs/content/ as it's written, fragments and generated files included, with
// code, tables, lists, headings, and directives left out, and a link counted
// by its text and an image not at all. The contracts are left out: a contract's
// rules are written to be read as one.
import { execFileSync } from "node:child_process";
import { readFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { expect, test } from "vitest";

const root = fileURLToPath(new URL("../..", import.meta.url));
const MAX_WORDS = 150;

/** The tracked files matching git's pathspecs. */
function tracked(...pathspecs: string[]): string[] {
  return execFileSync("git", ["ls-files", "--", ...pathspecs], { cwd: root, encoding: "utf8" })
    .split("\n")
    .filter((file) => file !== "");
}

/** A file's paragraphs, each with the line it starts on. */
function paragraphs(text: string): { line: number; text: string }[] {
  const found: { line: number; text: string }[] = [];
  let current: string[] = [];
  let start = 0;
  let fence: string | null = null;
  const flush = (): void => {
    const first = current[0]?.trimStart() ?? "";
    const other =
      /^(\||#|>|<|@|[-*+] |\d+[.)] )/.test(first) || (current[0] ?? "").startsWith("    ");
    if (current.length > 0 && !other) found.push({ line: start, text: current.join(" ") });
    current = [];
  };
  const lines = text.replace(/^---\n[\s\S]*?\n---\n/, (front) => "\n".repeat(front.split("\n").length - 1)).split("\n");
  lines.forEach((line, i) => {
    const trimmed = line.trim();
    if (fence !== null) {
      if (trimmed.startsWith(fence)) fence = null;
      return;
    }
    const opens = /^(`{3,}|~{3,})/.exec(trimmed);
    if (opens !== null) fence = opens[1] ?? null;
    if (opens !== null || trimmed === "") {
      flush();
      return;
    }
    if (current.length === 0) start = i + 1;
    current.push(trimmed);
  });
  flush();
  return found;
}

function words(paragraph: string): number {
  const read = paragraph.replace(/!\[[^\]]*\]\([^)]*\)/g, "").replace(/\[([^\]]*)\]\([^)]*\)/g, "$1");
  return read.split(/\s+/).filter((word) => word !== "").length;
}

test(`no paragraph of the docs outside the contracts runs over ${MAX_WORDS} words`, () => {
  const long = tracked("docs/content/*.md")
    .filter((file) => !file.startsWith("docs/content/contracts/"))
    .flatMap((file) =>
      paragraphs(readFileSync(path.join(root, file), "utf8"))
        .map(({ line, text }) => ({ at: `${file}:${line}`, words: words(text) }))
        .filter(({ words }) => words > MAX_WORDS)
        .map(({ at, words }) => `${at}: ${words} words`),
    );
  expect(long).toEqual([]);
});
