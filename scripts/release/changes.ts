// The changelog's unreleased entries, one file per change: each pull request
// that a user would notice adds `changes/<area>/<name>.md`, so two pull
// requests never edit the same lines of CHANGELOG.md. A release gathers them.
//
//   node scripts/release/changes.ts gather   move every entry into CHANGELOG.md's
//                                            unreleased section, and delete its file
//   node scripts/release/changes.ts check    say what's wrong with any entry
//
// An entry is one or more of the changelog's list items, as CHANGELOG.md
// writes them: each starts with `- ` and its other lines are indented under
// it, and a link is written relative to the repository's root. Its folder is
// its area, and an area's entries go at the top of the area's `###` heading in
// the unreleased section, in the order of their names. The site
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

/**
 * The areas a change can be in, in the order the changelog's sections list
 * them: each is a folder of `changes/`, and the `###` heading its entries are
 * gathered under.
 */
export const AREAS: readonly { folder: string; heading: string }[] = [
  { folder: "language", heading: "The language" },
  { folder: "command", heading: "The `ascribe` command" },
  { folder: "editor", heading: "The editor" },
  { folder: "language-server", heading: "The language server" },
  { folder: "astro", heading: "Astro and the elements" },
  { folder: "agents", heading: "Agents" },
  { folder: "review", heading: "Review" },
  { folder: "docs", heading: "The docs and the site" },
  { folder: "installing", heading: "Installing" },
  { folder: "fixes", heading: "Fixes" },
];

/**
 * The most characters an entry's summary has, as a reader sees it, with no
 * Markdown: about two lines of the changelog page, where an entry shows its
 * summary until it's opened (decision #382b).
 */
export const SUMMARY_LIMIT = 100;

/** One change's entry. */
export interface Change {
  /** The folder it's in, in `changes/`: its area's, or `""` when it's in none. */
  area: string;
  /** Its file name. */
  name: string;
  /** Its text, as the file has it. */
  text: string;
}

/** Where an entry is, relative to the repository's root. */
export function changePath(change: Change): string {
  return [changesDir, change.area, change.name].filter((part) => part !== "").join("/");
}

/**
 * The entries in `changes/` under `root`: each area's, in the areas' order,
 * then any other folder's, each in the order of their names. An entry in no
 * area, or in a folder that isn't one, is read too, so the check can name it.
 */
export function readChanges(root: string = repository): Change[] {
  const dir = join(root, changesDir);
  if (!existsSync(dir)) return [];
  const order = (folder: string): number => {
    const at = AREAS.findIndex((area) => area.folder === folder);
    return at === -1 ? AREAS.length : at;
  };
  const changes: Change[] = [];
  const entries = readdirSync(dir, { withFileTypes: true })
    .filter((entry) => entry.name !== readme && !entry.name.startsWith("."))
    .sort(
      (a, b) =>
        order(a.isDirectory() ? a.name : "") - order(b.isDirectory() ? b.name : "") ||
        compare(a.name, b.name),
    );
  for (const entry of entries) {
    if (!entry.isDirectory()) {
      changes.push({
        area: "",
        name: entry.name,
        text: readFileSync(join(dir, entry.name), "utf8"),
      });
      continue;
    }
    const folder = join(dir, entry.name);
    for (const name of readdirSync(folder)
      .filter((name) => !name.startsWith("."))
      .sort(compare)) {
      changes.push({ area: entry.name, name, text: readFileSync(join(folder, name), "utf8") });
    }
  }
  return changes;
}

function compare(a: string, b: string): number {
  return a < b ? -1 : a > b ? 1 : 0;
}

/**
 * The line that heads the changelog's unreleased section: `## Unreleased`, or
 * `## x.y.z (unreleased)`, which names the version it will be released as.
 */
export const UNRELEASED = /^## (?:Unreleased|(\d+\.\d+\.\d+) \(unreleased\))\s*$/i;

/**
 * The changelog with the entries in its unreleased section: each area's at
 * the top of the area's `###` heading, before the entries already there, with
 * the heading added, in the areas' order, when the section hasn't one yet. An
 * entry in no area goes at the top of the section, as before there were
 * areas, so the site still renders one the check would refuse. Throws when
 * there's no unreleased section to put them in.
 */
export function withChanges(changelog: string, changes: readonly Change[]): string {
  if (changes.length === 0) return changelog;
  const lines = changelog.split("\n");
  const heading = lines.findIndex((line) => UNRELEASED.test(line));
  if (heading === -1) throw new Error(`CHANGELOG.md has no "## Unreleased" section for them`);
  const groups = [
    ...AREAS.map((area) => ({
      heading: area.heading as string | undefined,
      changes: changes.filter((change) => change.area === area.folder),
    })),
    {
      heading: undefined,
      changes: changes.filter((change) => !AREAS.some((area) => area.folder === change.area)),
    },
  ];
  for (const group of groups) {
    if (group.changes.length === 0) continue;
    const entries = group.changes.flatMap((change) => change.text.replace(/\s+$/, "").split("\n"));
    const end = sectionEnd(lines, heading);
    const at =
      group.heading === undefined
        ? heading
        : lines.findIndex((line, i) => i > heading && i < end && line === `### ${group.heading}`);
    if (at !== -1) {
      insertUnder(lines, at, entries);
      continue;
    }
    // A new heading goes before the first heading of a later area, or at the
    // end of the section.
    const later = AREAS.slice(AREAS.findIndex((area) => area.heading === group.heading) + 1).map(
      (area) => `### ${area.heading}`,
    );
    let before = lines.findIndex((line, i) => i > heading && i < end && later.includes(line));
    if (before === -1) {
      before = end;
      while (before - 1 > heading && lines[before - 1]?.trim() === "") before--;
    }
    const block = [`### ${group.heading}`, "", ...entries];
    const gapBefore = lines[before - 1]?.trim() === "" ? [] : [""];
    const gapAfter = lines[before]?.trim() === "" ? [] : [""];
    lines.splice(before, 0, ...gapBefore, ...block, ...gapAfter);
  }
  return lines.join("\n");
}

/** The line after the section `heading` heads: the next `## ` heading, or the end. */
function sectionEnd(lines: readonly string[], heading: number): number {
  const next = lines.findIndex((line, i) => i > heading && line.startsWith("## "));
  return next === -1 ? lines.length : next;
}

/**
 * Puts `entries` under the heading at `at`, after the blank line that follows
 * it and before the entries already there. The entries already there continue
 * the list; anything else (the next heading, or the end of the file) is kept
 * apart by a blank line.
 */
function insertUnder(lines: string[], at: number, entries: readonly string[]): void {
  let first = at + 1;
  while (first < lines.length && lines[first]?.trim() === "") first++;
  const gap = lines[first]?.startsWith("- ") === true ? [] : [""];
  lines.splice(at + 1, first - (at + 1), "", ...entries, ...gap);
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
  if (!AREAS.some((area) => area.folder === change.area)) {
    const folders = AREAS.map((area) => `\`${area.folder}/\``).join(", ");
    problems.push(
      change.area === ""
        ? `it isn't in an area's folder: move it into the one its change is in, one of ${folders}`
        : `\`${change.area}/\` isn't an area: move it into one of ${folders}`,
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
  for (const item of text.split(/^(?=- )/m).filter((item) => item.startsWith("- "))) {
    const summary = summaryOf(item);
    const start = item.split("\n")[0]?.slice(0, 40) ?? "";
    if (!item.startsWith("- **")) {
      problems.push(`the item \`${start}…\` doesn't start with a bold sentence, \`- **…**\``);
    } else if (summary.length > SUMMARY_LIMIT) {
      problems.push(
        `the summary of \`${start}…\` (its bold sentence, or a fix's first sentence) is ${summary.length} characters, more than ${SUMMARY_LIMIT}: say the change in fewer words, and the rest after it`,
      );
    }
  }
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

/**
 * What an entry shows while it's closed on the changelog page, as plain text:
 * its bold text, when that's a sentence of its own, and otherwise its first
 * sentence, as for an entry whose bold text is only `Fixed:` or starts a
 * sentence (`**`@snippet`** puts…`), or that has none (decision #382c). `item`
 * is one list item, from its `- `. The page finds the same in the rendered
 * HTML (`site/src/changelog.ts`).
 */
export function summaryOf(item: string): string {
  const text = item.replace(/^- /, "");
  const bold = /^\*\*([\s\S]+?)\*\*/.exec(text)?.[1];
  return plainText(bold !== undefined && endsSentence(bold) ? bold : firstSentence(text));
}

/** Whether a bold text is a sentence of its own: it ends as one, and doesn't only label a fix. */
export function endsSentence(bold: string): boolean {
  return /[.:!?]\s*$/.test(bold) && !FIXED.test(bold);
}

/** Bold text that only labels an entry a fix, and says nothing of it. */
export const FIXED = /^\s*Fixed[:.]?\s*$/;

/** Markdown's text up to the end of its first sentence, outside code spans. */
function firstSentence(markdown: string): string {
  let code = false;
  for (let i = 0; i < markdown.length; i++) {
    const char = markdown[i];
    if (char === "`") code = !code;
    if (
      !code &&
      (char === "." || char === "!" || char === "?") &&
      /^(\s|$)/.test(markdown.slice(i + 1, i + 2))
    ) {
      return markdown.slice(0, i + 1);
    }
  }
  return markdown;
}

/** Markdown's text as a reader sees it: no link targets, code marks, or emphasis. */
function plainText(markdown: string): string {
  return markdown
    .replace(/\[([^\]]*)\]\([^)]*\)/g, "$1")
    .replace(/<(https?:[^>]*)>/g, "$1")
    .replace(/[`*]/g, "")
    .replace(/\s+/g, " ")
    .trim();
}

/** Every problem with the entries under `root`, each prefixed with its file. */
export function checkChanges(root: string = repository): string[] {
  const exists = (file: string): boolean => existsSync(join(root, ...file.split("/")));
  return readChanges(root).flatMap((change) =>
    checkChange(change, exists).map((problem) => `${changePath(change)}: ${problem}`),
  );
}

/**
 * Moves the entries under `root` into its CHANGELOG.md and deletes their
 * files. Throws, changing nothing, when an entry has a problem or the
 * changelog has no unreleased section. Returns the paths it moved.
 */
export function gatherChanges(root: string = repository): string[] {
  const problems = checkChanges(root);
  if (problems.length > 0) throw new Error(problems.join("\n"));
  const changes = readChanges(root);
  const file = join(root, "CHANGELOG.md");
  writeFileSync(file, withChanges(readFileSync(file, "utf8"), changes));
  for (const change of changes) rmSync(join(root, ...changePath(change).split("/")));
  return changes.map(changePath);
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
