// The pages that route a reader to a feature, each checked against what it
// routes to: the docs index against the sections in docs/ascribe.nav, the
// command reference's list against the commands in the synopsis the help text
// generates, and Files Ascribe writes against the project files the code
// names. A feature added to one and not the other fails here.
import { execFileSync } from "node:child_process";
import { existsSync, readFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { parse } from "smol-toml";
import { expect, test } from "vitest";

const root = fileURLToPath(new URL("../..", import.meta.url));
const content = "docs/content";

function read(file: string): string {
  return readFileSync(path.join(root, file), "utf8");
}

/** The tracked files matching git's pathspecs. */
function tracked(...pathspecs: string[]): string[] {
  return execFileSync("git", ["ls-files", "--", ...pathspecs], { cwd: root, encoding: "utf8" })
    .split("\n")
    .filter((file) => file !== "");
}

/** A page's text from a `## heading` to the next one, or to its end. */
function section(text: string, heading: string): string {
  const start = text.indexOf(`\n## ${heading}\n`);
  if (start === -1) throw new Error(`no "## ${heading}"`);
  const end = text.indexOf("\n## ", start + 1);
  return text.slice(start, end === -1 ? undefined : end);
}

/** Every code span in a text, without its backticks. */
function codeSpans(text: string): string[] {
  return Array.from(text.matchAll(/`([^`]+)`/g), ([, span = ""]) => span);
}

test("the docs index links every section in ascribe.nav, and the contracts only in their own group", () => {
  const nav = parse(read("docs/ascribe.nav")) as { sections: { folder: string }[] };
  const index = read(`${content}/index.md`);
  /** The pages a text links, from the content root, without a heading's `#`. */
  const linked = (text: string): string[] =>
    Array.from(text.matchAll(/\]\(([^)#{]+\.md)(?:#[^)]*)?\)/g), ([, page = ""]) => page);
  const pages = new Set(linked(index));

  const missing = nav.sections
    .map(({ folder }) => folder)
    // The content root's own page is the index.
    .filter((folder) => folder !== "")
    .filter((folder) => {
      // A section's own page: its index.md, or the page beside its folder.
      const own = [`${folder}index.md`, `${folder.replace(/\/$/, "")}.md`].find((page) =>
        existsSync(path.join(root, content, page)),
      );
      if (own !== undefined) return !pages.has(own);
      // A section with none is linked by one of its pages.
      return ![...pages].some(
        (page) => page.startsWith(folder) && !page.slice(folder.length).includes("/"),
      );
    });
  expect(missing, "sections the docs index doesn't link").toEqual([]);

  const start = index.indexOf("\n## For implementers\n");
  expect(start, "the docs index's contracts group").toBeGreaterThan(-1);
  const contracts = (page: string): boolean => page.startsWith("contracts/");
  expect(linked(index.slice(0, start)).filter(contracts)).toEqual([]);
  expect(linked(index.slice(start)).filter(contracts).length).toBeGreaterThan(5);
});

test("the command reference lists every top-level command in the synopsis, each linked to its section", () => {
  const synopsis = read(`${content}/_generated/cli-synopsis.md`);
  const commands = [
    ...new Set(Array.from(synopsis.matchAll(/^ascribe ([a-z]\S*)/gm), ([, name = ""]) => name)),
  ];
  expect(commands.length).toBeGreaterThan(20);

  const list = section(read(`${content}/command/index.md`), "Commands");
  const rows = Array.from(
    list.matchAll(/^\| \[`ascribe ([a-z]\S*)`\]\(([^)]*)\) \|/gm),
    ([, name = "", target = ""]) => ({ name, anchor: target.split("#")[1] }),
  );
  expect(rows.map(({ name }) => name).sort()).toEqual([...commands].sort());
  expect(rows.filter(({ name, anchor }) => anchor !== `cli-${name}`)).toEqual([]);
});

test("Files Ascribe writes has a row for every project file the code names", () => {
  // The files and folders a project holds are named as constants in the
  // crates that read and write them (`ascribe_model::ADDRESSES_FILE`,
  // `ascribe_check::baseline::BASELINE_FILE`, ...). The outputs' paths, in
  // ascribe-emit, are the output layout's, not the project's.
  const declared = tracked(
    "crates/ascribe-model/src/*.rs",
    "crates/ascribe-check/src/*.rs",
    "crates/ascribe-report/src/*.rs",
    "crates/ascribe-resolve/src/*.rs",
  ).flatMap((file) =>
    Array.from(
      read(file).matchAll(/^pub const \w*(?:FILE|FOLDER|DIR)\w*: &str = "([^"]+)";/gm),
      ([, name = ""]) => `${name} (${file})`,
    ),
  );
  expect(declared.length).toBeGreaterThan(10);

  const rows = Array.from(
    section(read(`${content}/command/project-files.md`), "The files").matchAll(
      /^\| (`.*?) \| (.*?) \| (?:Yes|No) \|$/gm,
    ),
    ([, file = "", what = ""]) => ({ files: codeSpans(file), what: codeSpans(what) }),
  );
  const folders = rows.flatMap(({ files }) => files.filter((file) => file.endsWith("/")));
  const covered = (name: string): boolean =>
    rows.some(({ files }) => files.includes(name) || files.includes(`${name}/`)) ||
    // A file in a folder that has a row, such as `.ascribe/history/links.json`.
    folders.some((folder) => name.startsWith(folder)) ||
    // A file a row's folder holds, which the row names, such as `templates.lock`.
    rows.some(({ files, what }) => files.some((file) => file.endsWith("/")) && what.includes(name));

  expect(
    declared.filter((entry) => !covered(entry.slice(0, entry.indexOf(" (")))),
    "project files Files Ascribe writes has no row for",
  ).toEqual([]);
});
