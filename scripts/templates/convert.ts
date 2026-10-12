// Ascribe's page templates, converted from The Good Docs Project's, at the
// release pinned in upstream.toml. Nobody edits a converted file: a change to
// one is a change to this script, to a rule in rules.toml, or to their
// repository (project-docs/page-templates/README.md, decision 1).
//
//   node scripts/templates/convert.ts           rewrite the copies
//   node scripts/templates/convert.ts --check   list the copies that are stale
//
// It fetches the pinned commit's files once, into scripts/templates/.upstream/,
// which git ignores, and writes the copies, and set.toml listing them, into
// crates/ascribe-templates/set/. The last step is `ascribe fmt`, so it needs a
// binary: ASCRIBE_BIN, or this repository's debug build.
//
// scripts/templates/convert.test.ts fails when the copies are stale; with
// ASCRIBE_BLESS=1 it rewrites them.
import { spawnSync } from "node:child_process";
import {
  existsSync,
  mkdirSync,
  mkdtempSync,
  readdirSync,
  readFileSync,
  rmSync,
  writeFileSync,
} from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { parse } from "smol-toml";

const root = fileURLToPath(new URL("../..", import.meta.url));
const here = path.join(root, "scripts", "templates");

/** Where the copies go, relative to the root. */
export const SET = "crates/ascribe-templates/set";

/** Their release: upstream.toml. */
export interface Pin {
  repository: string;
  tag: string;
  commit: string;
}

/** One template's rules: its table in rules.toml. */
export interface Rules {
  description: string;
  about: string[];
  directives: { directive: string; before: string }[];
}

/** A step that can't be done as the source is. It names the template and the place. */
export class ConversionError extends Error {}

/** Their repository couldn't be reached. */
export class Unreachable extends Error {}

export function readPin(): Pin {
  const pin = parse(readFileSync(path.join(here, "upstream.toml"), "utf8")) as unknown as Pin;
  for (const key of ["repository", "tag", "commit"] as const)
    if (typeof pin[key] !== "string") throw new Error(`upstream.toml has no \`${key}\``);
  return pin;
}

export function readRules(
  text = readFileSync(path.join(here, "rules.toml"), "utf8"),
): Map<string, Rules> {
  const tables = parse(text) as Record<string, Partial<Rules>>;
  return new Map(
    Object.entries(tables).map(([name, table]) => {
      if (typeof table.description !== "string")
        throw new Error(`rules.toml: [${name}] has no \`description\``);
      return [
        name,
        {
          description: table.description,
          about: table.about ?? [],
          directives: table.directives ?? [],
        },
      ];
    }),
  );
}

// The conversion. Each step is small enough to predict from their source.

const quoteLine = /^ {0,3}>/;
const fenceLine = /^\s*(`{3,}|~{3,})/;
const thematicBreak = /^ {0,3}([-*_])(?:[ \t]*\1){2,}[ \t]*$/;

function blank(line: string): boolean {
  return line.trim() === "";
}

/**
 * A template, as an Ascribe page: what it says about itself removed, its title
 * moved to frontmatter, each structural rule applied, and every opening brace
 * outside code escaped. `ascribe fmt` is the step after this one.
 */
export function convertTemplate(name: string, source: string, rules: Rules): string {
  const lines = source.replace(/\r\n?/g, "\n").split("\n");
  let start = 0;
  let end = lines.length;
  const skipBlank = () => {
    while (start < end && blank(lines[start] ?? "")) start++;
  };
  const trimBlank = () => {
    while (end > start && blank(lines[end - 1] ?? "")) end--;
  };

  // The title, from the level-1 heading their template opens with (decision 21).
  skipBlank();
  const heading = /^# +(.*?)(?: +#+)? *$/.exec(lines[start] ?? "");
  if (heading === null)
    throw new ConversionError(
      `${name}: the template doesn't open with a level-1 heading (line ${start + 1})`,
    );
  const title = heading[1] ?? "";
  start++;

  // What the template says about itself: the block quotes after the title, and
  // the one it closes with, with the thematic break that sets it off (decision 22).
  skipBlank();
  if (!quoteLine.test(lines[start] ?? ""))
    throw new ConversionError(
      `${name}: no block quote about the template after its title (line ${start + 1})`,
    );
  while (start < end && quoteLine.test(lines[start] ?? "")) {
    while (start < end && quoteLine.test(lines[start] ?? "")) start++;
    skipBlank();
  }
  trimBlank();
  if (end <= start || !quoteLine.test(lines[end - 1] ?? ""))
    throw new ConversionError(`${name}: the template doesn't close with a block quote`);
  while (end > start && quoteLine.test(lines[end - 1] ?? "")) end--;
  trimBlank();
  if (end - 2 >= start && thematicBreak.test(lines[end - 1] ?? "") && blank(lines[end - 2] ?? "")) {
    end--;
    trimBlank();
  }

  // What the template says about itself elsewhere: each paragraph a rule names
  // by its first line, removed whole, with the blank lines after it.
  let body = lines.slice(start, end);
  for (const first of rules.about) {
    const code = codeLines(body);
    const found = body.flatMap((line, i) =>
      !code[i] && line.trim() === first && (i === 0 || blank(body[i - 1] ?? "")) ? [i] : [],
    );
    if (found.length !== 1)
      throw new ConversionError(
        `${name}: the rule \`about\` found ${found.length} paragraphs opening \`${first}\`, not one`,
      );
    const from = found[0] ?? 0;
    let to = from;
    while (to < body.length && !blank(body[to] ?? "")) to++;
    while (to < body.length && blank(body[to] ?? "")) to++;
    body = [...body.slice(0, from), ...body.slice(to)];
  }
  const code = codeLines(body);

  // The structural rules, each at the one line of theirs it names.
  const before = new Map<number, string[]>();
  for (const rule of rules.directives) {
    const found = body.flatMap((line, i) => (!code[i] && line.trim() === rule.before ? [i] : []));
    if (found.length !== 1)
      throw new ConversionError(
        `${name}: the rule \`${rule.directive}\` found ${found.length} lines \`${rule.before}\`, not one`,
      );
    const at = found[0] ?? 0;
    const indent = /^\s*/.exec(body[at] ?? "")?.[0] ?? "";
    before.set(at, [...(before.get(at) ?? []), indent + rule.directive]);
  }

  const escaped = escapeBraces(body, code);
  const out = escaped.flatMap((line, i) => [...(before.get(i) ?? []), line]);
  return [
    "---",
    `title: ${JSON.stringify(title)}`,
    `description: ${JSON.stringify(rules.description)}`,
    "---",
    "",
    ...out,
    "",
  ].join("\n");
}

/** Which lines are a fenced code block's, its fences included. */
function codeLines(lines: string[]): boolean[] {
  const code: boolean[] = [];
  let open: string | undefined;
  for (const line of lines) {
    const fence = fenceLine.exec(line)?.[1];
    if (open === undefined) {
      open = fence;
      code.push(open !== undefined);
    } else {
      code.push(true);
      if (
        fence !== undefined &&
        fence[0] === open[0] &&
        fence.length >= open.length &&
        line.trim() === fence
      )
        open = undefined;
    }
  }
  return code;
}

/**
 * Every opening brace escaped (decisions 4 and 17), but in a fenced code block
 * or a code span, and one escaped already. A closing brace is left as it is,
 * so a placeholder that runs across blocks has one brace escaped, where it
 * opens (decision 20). Code spans are found in each run of non-blank lines, so
 * one can cross a line. An indented code block isn't recognized: none of their
 * templates has one, and in a list item four spaces are the item's text.
 */
export function escapeBraces(lines: string[], code: boolean[] = codeLines(lines)): string[] {
  const out = [...lines];
  let i = 0;
  while (i < lines.length) {
    if (code[i] || blank(lines[i] ?? "")) {
      i++;
      continue;
    }
    let j = i;
    while (j < lines.length && !code[j] && !blank(lines[j] ?? "")) j++;
    const text = lines.slice(i, j).join("\n");
    out.splice(i, j - i, ...escapeOutsideSpans(text).split("\n"));
    i = j;
  }
  return out;
}

function escapeOutsideSpans(text: string): string {
  let out = "";
  let at = 0;
  while (at < text.length) {
    const c = text.charAt(at);
    if (c === "\\") {
      // An escape: it and the character it escapes stay as they are.
      out += text.slice(at, at + 2);
      at += 2;
    } else if (c === "`") {
      const run = /^`+/.exec(text.slice(at))?.[0] ?? "`";
      const close = closingRun(text, at + run.length, run.length);
      if (close === -1) {
        out += run;
        at += run.length;
      } else {
        out += text.slice(at, close + run.length);
        at = close + run.length;
      }
    } else {
      out += c === "{" ? "\\{" : c;
      at++;
    }
  }
  return out;
}

/** Where a backtick run of exactly `length` starts at or after `from`, or -1. */
function closingRun(text: string, from: number, length: number): number {
  const runs = /`+/g;
  runs.lastIndex = from;
  for (let found = runs.exec(text); found !== null; found = runs.exec(text))
    if (found[0].length === length) return found.index;
  return -1;
}

/**
 * A guide, as a document to read: a line saying where it's from, then their
 * text, with each relative link made a link to their repository at the tag,
 * since the files it points at aren't copied.
 */
export function convertGuide(
  name: string,
  file: string,
  source: string,
  pin: Pin,
  license: string,
): string {
  const at = `${pin.repository}/-/blob/${pin.tag}/${name}/`;
  const text = source
    .replace(/\r\n?/g, "\n")
    .replace(/\]\(([^)\s]+)\)/g, (link, target: string) =>
      /^([a-z][a-z0-9+.-]*:|#|\/)/i.test(target) ? link : `](${new URL(target, at).href})`,
    )
    .replace(/\n*$/, "\n");
  return `Generated from [The Good Docs Project's guide](${at}${file}), in their repository on GitLab at the tag ${pin.tag}, under the ${license}.\n\n${text}`;
}

// Their repository, and the files written.

interface IndexEntry {
  name: string;
  summary: string;
  files: { template?: string; guide?: string };
}

/** A file of theirs at the pinned commit, fetched once into .upstream/. */
export async function upstream(pin: Pin, file: string): Promise<string> {
  const cached = path.join(here, ".upstream", pin.commit, file);
  if (existsSync(cached)) return readFileSync(cached, "utf8");
  const url = `${pin.repository}/-/raw/${pin.commit}/${file}`;
  let response: Response;
  try {
    response = await fetch(url, { signal: AbortSignal.timeout(30_000) });
  } catch (error) {
    throw new Unreachable(
      `${url}: ${error instanceof Error ? (error.cause instanceof Error ? error.cause.message : error.message) : String(error)}`,
    );
  }
  if (!response.ok) throw new Error(`${url} answered ${response.status}`);
  const text = await response.text();
  mkdirSync(path.dirname(cached), { recursive: true });
  writeFileSync(cached, text);
  return text;
}

/** The `ascribe` binary: ASCRIBE_BIN, or this repository's debug build. */
export function binary(): string | undefined {
  const built = path.join(
    root,
    "target",
    "debug",
    process.platform === "win32" ? "ascribe.exe" : "ascribe",
  );
  return process.env["ASCRIBE_BIN"] ?? (existsSync(built) ? built : undefined);
}

/** `ascribe fmt` on each page, in a project that declares nothing but a title and a description. */
function format(ascribe: string, pages: Map<string, string>): Map<string, string> {
  const dir = mkdtempSync(path.join(tmpdir(), "ascribe-templates-"));
  try {
    writeFileSync(
      path.join(dir, "ascribe.toml"),
      'spec = "0.1"\n\n[types.page]\ndefault = true\n\n[types.page.frontmatter]\ntitle = "string"\ndescription = "string"\n',
    );
    mkdirSync(path.join(dir, "docs"));
    for (const [file, text] of pages) writeFileSync(path.join(dir, "docs", file), text);
    const run = spawnSync(ascribe, ["fmt"], { cwd: dir, encoding: "utf8" });
    if (run.status !== 0) throw new Error(`ascribe fmt failed:\n${run.stdout}${run.stderr}`);
    return new Map(
      [...pages.keys()].map((file) => [file, readFileSync(path.join(dir, "docs", file), "utf8")]),
    );
  } finally {
    rmSync(dir, { recursive: true, force: true });
  }
}

/** Every file of the set, by its name in SET, as the pinned release gives it. */
export async function generate(ascribe: string): Promise<Map<string, string>> {
  const pin = readPin();
  const rules = readRules();
  const index = JSON.parse(await upstream(pin, "index.json")) as IndexEntry[];
  const license = /^#\s*(.+?)\s*$/m.exec(await upstream(pin, "LICENSE"))?.[1];
  if (license === undefined) throw new Error("Their LICENSE has no heading naming it");

  const pages = new Map<string, string>();
  const guides = new Map<string, string>();
  const listed: string[] = [];
  for (const [name, rule] of rules) {
    const entry = index.find((e) => e.files.template === `template_${name}.md`);
    if (entry === undefined || entry.files.guide === undefined)
      throw new ConversionError(
        `${name}: their index.json has no entry with template_${name}.md and a guide`,
      );
    const template = await upstream(pin, `${name}/template_${name}.md`);
    const guide = await upstream(pin, `${name}/${entry.files.guide}`);
    pages.set(`${name}.md`, convertTemplate(name, template, rule));
    guides.set(`${name}.guide.md`, convertGuide(name, entry.files.guide, guide, pin, license));
    listed.push(
      [
        "[[template]]",
        `name = ${JSON.stringify(name)}`,
        `summary = ${JSON.stringify(entry.summary)}`,
        `template = ${JSON.stringify(`${name}.md`)}`,
        `guide = ${JSON.stringify(`${name}.guide.md`)}`,
      ].join("\n"),
    );
  }

  const set = [
    "# Generated by scripts/templates/convert.ts from The Good Docs Project's templates, at the tag below. Don't edit it or the copies beside it: change the script, its rules (scripts/templates/rules.toml), or the pin (scripts/templates/upstream.toml), then run `node scripts/templates/convert.ts`.",
    `repository = ${JSON.stringify(pin.repository)}`,
    `tag = ${JSON.stringify(pin.tag)}`,
    `commit = ${JSON.stringify(pin.commit)}`,
    `license = ${JSON.stringify(license)}`,
    "",
    listed.join("\n\n"),
    "",
  ].join("\n");
  return new Map([["set.toml", set], ...format(ascribe, pages), ...guides]);
}

/** The files of SET that differ from `files`, or are there and shouldn't be. */
export function stale(files: Map<string, string>): string[] {
  const dir = path.join(root, SET);
  const present = existsSync(dir) ? readdirSync(dir) : [];
  return [
    ...[...files].filter(
      ([file, text]) =>
        !present.includes(file) || readFileSync(path.join(dir, file), "utf8") !== text,
    ),
    ...present.filter((file) => !files.has(file)).map((file) => [file, ""] as const),
  ]
    .map(([file]) => `${SET}/${file}`)
    .sort();
}

/** Writes `files` as SET, removing any other file there. */
export function write(files: Map<string, string>): void {
  const dir = path.join(root, SET);
  mkdirSync(dir, { recursive: true });
  for (const file of readdirSync(dir)) if (!files.has(file)) rmSync(path.join(dir, file));
  for (const [file, text] of files) writeFileSync(path.join(dir, file), text);
}

if (import.meta.main) {
  const ascribe = binary();
  if (ascribe === undefined) {
    console.error("No `ascribe` to run: set ASCRIBE_BIN, or run `cargo build -p ascribe-cli`.");
    process.exit(2);
  }
  const files = await generate(ascribe);
  if (process.argv.includes("--check")) {
    const found = stale(files);
    if (found.length > 0) {
      console.error(
        `Stale, against ${readPin().tag}: run \`node scripts/templates/convert.ts\`, then read the diff.\n${found.join("\n")}`,
      );
      process.exit(1);
    }
    console.log(`${SET} is what ${readPin().tag} gives.`);
  } else {
    write(files);
    console.log(`wrote ${SET}`);
  }
}
