// The changelog's unreleased entries, one file per change: each pull request
// that a user would notice adds `changes/<name>.md`, so two pull requests
// never edit the same lines of CHANGELOG.md. A release gathers them.
//
//   node scripts/release/changes.ts gather   move every entry into CHANGELOG.md's
//                                            unreleased section, and delete its file
//   node scripts/release/changes.ts check    say what's wrong with any entry
//
// An entry is one or more of the changelog's list items, as CHANGELOG.md
// writes them: each starts with `- ` and its other lines are indented under
// it, and a link is written relative to the repository's root. The entries go
// at the top of the unreleased section, in the order of their names. The site
// renders the changelog with them gathered (site/src/changelog.ts), so its
// unreleased section is `main`'s.
//
// This file imports nothing but Node's own modules: the site, which installs
// none of the workspace's, imports it too.
import { existsSync, readFileSync, readdirSync, rmSync, writeFileSync } from "node:fs";
import { join, posix, resolve } from "node:path";
import { fileURLToPath } from "node:url";

/** The repository's root. */
const repository = resolve(fileURLToPath(import.meta.url), "..", "..", "..");

/** The folder of entries, relative to the repository's root. */
export const changesDir = "changes";

/** The file in it that says what it is, which isn't an entry. */
const readme = "README.md";

/** An entry's file name: lowercase words joined by `-`, such as `check-fix.md`. */
const NAME = /^[a-z0-9]+(?:-[a-z0-9]+)*\.md$/;

/** One change's entry. */
export interface Change {
  /** Its file name, in `changes/`. */
  name: string;
  /** Its text, as the file has it. */
  text: string;
}

/** The entries in `changes/` under `root`, in the order of their names. */
export function readChanges(root: string = repository): Change[] {
  const dir = join(root, changesDir);
  if (!existsSync(dir)) return [];
  return readdirSync(dir)
    .filter((name) => name !== readme && !name.startsWith("."))
    .sort()
    .map((name) => ({ name, text: readFileSync(join(dir, name), "utf8") }));
}

/**
 * The line that heads the changelog's unreleased section: `## Unreleased`, or
 * `## x.y.z (unreleased)`, which names the version it will be released as.
 */
export const UNRELEASED = /^## (?:Unreleased|(\d+\.\d+\.\d+) \(unreleased\))\s*$/i;

/**
 * The changelog with the entries at the top of its unreleased section, under
 * its heading, before the entries already there. Throws when there's no
 * unreleased section to put them in.
 */
export function withChanges(changelog: string, changes: readonly Change[]): string {
  if (changes.length === 0) return changelog;
  const lines = changelog.split("\n");
  const heading = lines.findIndex((line) => UNRELEASED.test(line));
  if (heading === -1) throw new Error(`CHANGELOG.md has no "## Unreleased" section for them`);
  let at = heading + 1;
  while (at < lines.length && lines[at]?.trim() === "") at++;
  const entries = changes.map((change) => change.text.replace(/\s+$/, "")).join("\n");
  const rest = lines.slice(at);
  // The entries already there continue the list; anything else (the next
  // release's heading, or the end of the file) is kept apart by a blank line.
  const gap = rest[0]?.startsWith("- ") === true ? [] : [""];
  return [...lines.slice(0, heading + 1), "", entries, ...gap, ...rest].join("\n");
}

/**
 * What's wrong with an entry, if anything: each problem a sentence. `exists`
 * says whether a path relative to the repository's root is there.
 */
export function checkChange(change: Change, exists: (file: string) => boolean): string[] {
  const problems: string[] = [];
  if (!NAME.test(change.name)) {
    problems.push(
      "its name isn't lowercase words joined by `-`, ending `.md`, such as `check-fix.md`",
    );
  }
  const { text } = change;
  if (text.trim() === "") return [...problems, "it's empty"];
  if (text.includes("\r")) problems.push("it has a carriage return; end its lines with `\\n` only");
  if (!text.endsWith("\n") || text.endsWith("\n\n")) {
    problems.push("it doesn't end with exactly one line break");
  }
  if (!text.startsWith("- ")) {
    problems.push("it doesn't start with `- `, as a list item of the changelog does");
  }
  let fenced = false;
  text.split("\n").forEach((line, index) => {
    if (/^\s*(```|~~~)/.test(line)) fenced = !fenced;
    if (fenced || line === "" || line.startsWith("- ") || line.startsWith("  ")) return;
    problems.push(
      `line ${index + 1} would end the list item: start an item with \`- \`, and indent what's under it by two spaces`,
    );
  });
  for (const [, href = ""] of text.matchAll(/\]\(([^)\s]+)(?:\s+"[^"]*")?\)/g)) {
    // An address, or a heading of the changelog.
    if (/^[a-z][a-z\d+.-]*:/i.test(href) || href.startsWith("//") || href.startsWith("#")) continue;
    const file = posix.normalize(href.split("#")[0]?.split("?")[0] ?? "");
    if (file.startsWith("/") || file === ".." || file.startsWith("../")) {
      problems.push(
        `the link ${href} isn't relative to the repository's root, as CHANGELOG.md's are`,
      );
    } else if (!exists(file.replace(/\/$/, ""))) {
      problems.push(`the link ${href} isn't a file in the repository`);
    }
  }
  return problems;
}

/** Every problem with the entries under `root`, each prefixed with its file. */
export function checkChanges(root: string = repository): string[] {
  const exists = (file: string): boolean => existsSync(join(root, ...file.split("/")));
  return readChanges(root).flatMap((change) =>
    checkChange(change, exists).map((problem) => `${changesDir}/${change.name}: ${problem}`),
  );
}

/**
 * Moves the entries under `root` into its CHANGELOG.md and deletes their
 * files. Throws, changing nothing, when an entry has a problem or the
 * changelog has no unreleased section. Returns the names it moved.
 */
export function gatherChanges(root: string = repository): string[] {
  const problems = checkChanges(root);
  if (problems.length > 0) throw new Error(problems.join("\n"));
  const changes = readChanges(root);
  const file = join(root, "CHANGELOG.md");
  writeFileSync(file, withChanges(readFileSync(file, "utf8"), changes));
  for (const change of changes) rmSync(join(root, changesDir, change.name));
  return changes.map((change) => change.name);
}

if (import.meta.main) {
  const [command] = process.argv.slice(2);
  if (command === "check") {
    const problems = checkChanges();
    for (const problem of problems) process.stderr.write(`error: ${problem}\n`);
    process.exit(problems.length > 0 ? 1 : 0);
  } else if (command === "gather") {
    try {
      const names = gatherChanges();
      process.stdout.write(
        names.length === 0
          ? `${changesDir}/ has no entries to gather.\n`
          : `Moved ${names.length} ${names.length === 1 ? "entry" : "entries"} into CHANGELOG.md: ${names.join(", ")}.\n`,
      );
    } catch (error) {
      process.stderr.write(`error: ${(error as Error).message}\n`);
      process.exit(1);
    }
  } else {
    process.stderr.write("usage: node scripts/release/changes.ts gather | check\n");
    process.exit(2);
  }
}
