// Which files a branch changes are generated, and what writes each, from the
// tables under ARCHITECTURE.md's `ASCRIBE_BLESS=1`: a reviewer accounts for
// each one, since rewriting a generated file takes one variable and a large
// regenerated diff is easy to skim past.
//
//   node scripts/repo-docs/generated.ts [base]
//
// `base` defaults to `origin/main`. The files are those changed since the
// branch left it, committed or not, and the new ones git doesn't ignore.
//
// Each table row is a writer, its first cell (a test, or a command), and its
// paths, the code spans in its Paths cell: a path ending in `/` is everything
// under it, `*` is any part of one name, `**` any number of folders, and a
// path after `!` is left out of that row.
import { execFileSync } from "node:child_process";
import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import { fileURLToPath } from "node:url";

/** The repository's root. */
const repository = resolve(fileURLToPath(import.meta.url), "..", "..", "..");

/** One row of the tables: what writes the files, and which they are. */
export interface Writer {
  /** The test or command, as the row's first cell names it. */
  name: string;
  /** The paths it writes, as the Paths cell writes them. */
  paths: string[];
  /** The paths after `!`, without it. */
  except: string[];
}

/** The section of ARCHITECTURE.md the tables are in, up to the next heading. */
function section(markdown: string): string {
  const start = markdown.indexOf("### `ASCRIBE_BLESS=1`");
  if (start < 0) throw new Error("ARCHITECTURE.md has no `ASCRIBE_BLESS=1` section");
  const body = markdown.indexOf("\n", start);
  const end = markdown.slice(body).search(/^#{2,3} /m);
  return end < 0 ? markdown.slice(start) : markdown.slice(start, body + end);
}

/** The writers in the section's tables with a Paths column, in order. */
export function readWriters(markdown: string): Writer[] {
  const writers: Writer[] = [];
  let column = -1;
  for (const line of section(markdown).split("\n")) {
    if (!line.startsWith("|")) {
      column = -1;
      continue;
    }
    const cells = line
      .slice(1, line.lastIndexOf("|"))
      .split(" | ")
      .map((cell) => cell.trim());
    if (column < 0) {
      column = cells.indexOf("Paths");
      continue;
    }
    if (/^[-:|]+$/.test(line.slice(1))) continue;
    const spans = Array.from((cells[column] ?? "").matchAll(/`([^`]+)`/g), (m) => m[1] ?? "");
    writers.push({
      name: (cells[0] ?? "").replace(/`/g, ""),
      paths: spans.filter((span) => !span.startsWith("!")),
      except: spans.filter((span) => span.startsWith("!")).map((span) => span.slice(1)),
    });
  }
  return writers;
}

/** Whether a path, relative to the root with `/`, is one a table path names. */
export function matches(pattern: string, file: string): boolean {
  const source = pattern
    .split(/(\*\*\/|\*)/)
    .map((part) =>
      part === "**/"
        ? "(?:[^/]+/)*"
        : part === "*"
          ? "[^/]*"
          : part.replace(/[.+?^${}()|[\]\\]/g, "\\$&"),
    )
    .join("");
  return new RegExp(`^${source}${pattern.endsWith("/") ? ".*" : ""}$`).test(file);
}

/** The writers whose paths take the file. */
export function writersOf(file: string, writers: Writer[]): Writer[] {
  return writers.filter(
    (writer) =>
      writer.paths.some((pattern) => matches(pattern, file)) &&
      !writer.except.some((pattern) => matches(pattern, file)),
  );
}

/** The writers ARCHITECTURE.md's tables name. */
export function writers(root: string = repository): Writer[] {
  return readWriters(readFileSync(resolve(root, "ARCHITECTURE.md"), "utf8"));
}

/** The files changed since the branch left `base`, committed or not, and new ones. */
function changed(base: string): string[] {
  const git = (...args: string[]): string[] =>
    execFileSync("git", args, { cwd: repository, encoding: "utf8" })
      .split("\n")
      .filter((line) => line !== "");
  const from = git("merge-base", base, "HEAD")[0] ?? base;
  const files = [
    ...git("diff", "--name-only", from),
    ...git("ls-files", "--others", "--exclude-standard"),
  ];
  return Array.from(new Set(files)).sort();
}

if (import.meta.main) {
  const [base = "origin/main", ...rest] = process.argv.slice(2);
  if (rest.length > 0 || base.startsWith("-")) {
    process.stderr.write("usage: node scripts/repo-docs/generated.ts [base]\n");
    process.exit(2);
  }
  const all = writers();
  let count = 0;
  for (const file of changed(base)) {
    const by = writersOf(file, all);
    if (by.length === 0) continue;
    count++;
    process.stdout.write(`${file}\t${by.map((writer) => writer.name).join(", ")}\n`);
  }
  if (count === 0) process.stdout.write(`No generated file changed since ${base}.\n`);
}
