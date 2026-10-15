// The decisions recorded one file each, in project-docs/decisions/: each new
// decision is `<number>-<slug>.md`, numbered by the issue it was made for (or
// its pull request's, with no issue), so two branches never claim the same
// number or edit the same lines. project-docs/decisions.md keeps the
// numbered decisions made before, up to `lastNumbered`, and takes no new ones.
//
//   node scripts/repo-docs/decisions.ts check             say what's wrong with any file
//   node scripts/repo-docs/decisions.ts list [section]    the files' decisions, under their sections
//
// A file is frontmatter naming its section, one of decisions.md's `##`
// headings, then the decision as decisions.md writes one: a bold sentence,
// the rest, `_Why:_`, and `_From:_`. Its links are relative to the file.
//
// The list is printed on demand rather than written into decisions.md, since
// an index every pull request rewrites would bring back the conflict the
// files remove.
import { existsSync, readFileSync, readdirSync } from "node:fs";
import { dirname, join, posix, relative, resolve } from "node:path";
import { fileURLToPath } from "node:url";

/** The repository's root. */
const repository = resolve(fileURLToPath(import.meta.url), "..", "..", "..");

/** The folder of decisions, relative to the repository's root. */
export const decisionsDir = "project-docs/decisions";

/** The file of the numbered decisions before the folder, relative to the root. */
export const decisionsLog = "project-docs/decisions.md";

/**
 * The last number decisions.md gives a decision. It takes no new ones, so a
 * higher number there is a decision that belongs in a file of the folder,
 * such as one from a branch opened before the folder was.
 */
export const lastNumbered = 128;

/** The file in the folder that says what it is, which isn't a decision. */
const readme = "README.md";

/**
 * A decision's file name: its issue's number, a letter when the issue made
 * more than one, then lowercase words joined by `-`, such as
 * `381-decision-files.md` or `381a-decision-files.md`.
 */
const NAME = /^(\d+)([a-z]?)-[a-z0-9]+(?:-[a-z0-9]+)*\.md$/;

/** One decision's file. */
export interface Decision {
  /** Its file name, in the folder. */
  name: string;
  /** Its text, as the file has it. */
  text: string;
}

/** What the log contributes to checking a file: its sections and its last number. */
export interface Log {
  /** The `##` headings of decisions.md, which are the sections a file may name. */
  sections: string[];
  /** The highest number decisions.md gives a decision. */
  last: number;
}

/** The sections and the last number of the log under `root`. */
export function readLog(root: string = repository): Log {
  const text = readFileSync(join(root, decisionsLog), "utf8");
  const sections = Array.from(text.matchAll(/^## (.+?)\s*$/gm), (m) => m[1] ?? "");
  const numbers = Array.from(text.matchAll(/^(\d+)\. /gm), (m) => Number(m[1]));
  return { sections, last: Math.max(0, ...numbers) };
}

/** The decisions in the folder under `root`, in the order of their names. */
export function readDecisions(root: string = repository): Decision[] {
  const dir = join(root, decisionsDir);
  if (!existsSync(dir)) return [];
  return readdirSync(dir)
    .filter((name) => name !== readme && !name.startsWith("."))
    .sort((a, b) => parseInt(a, 10) - parseInt(b, 10) || a.localeCompare(b))
    .map((name) => ({ name, text: readFileSync(join(dir, name), "utf8") }));
}

/** A decision's id as it's cited, such as `381` or `381a`, or `undefined` for a bad name. */
export function idOf(name: string): string | undefined {
  const match = NAME.exec(name);
  return match === null ? undefined : `${match[1]}${match[2]}`;
}

/** A file's frontmatter's fields and the text after it, or `undefined` without frontmatter. */
function split(text: string): { fields: Map<string, string>; body: string } | undefined {
  const match = /^---\n([\s\S]*?)\n---\n/.exec(text);
  if (match === null) return undefined;
  const fields = new Map<string, string>();
  for (const line of (match[1] ?? "").split("\n")) {
    const field = /^([a-z-]+):\s*(.*?)\s*$/.exec(line);
    if (field !== null) fields.set(field[1] ?? "", field[2] ?? "");
    else fields.set(line, "");
  }
  return { fields, body: text.slice(match[0].length) };
}

/** A decision's section, as its frontmatter names it. */
export function sectionOf(decision: Decision): string | undefined {
  return split(decision.text)?.fields.get("section");
}

/** A decision's bold sentence, struck through when it's been replaced. */
export function headlineOf(decision: Decision): string {
  const body = split(decision.text)?.body ?? decision.text;
  return /^\s*(~~\*\*[\s\S]+?\*\*~~|\*\*[\s\S]+?\*\*)/.exec(body)?.[1] ?? "";
}

/**
 * What's wrong with a decision's file, if anything: each problem a sentence.
 * `exists` says whether a path relative to the repository's root is there.
 */
export function checkDecision(
  decision: Decision,
  log: Log,
  exists: (file: string) => boolean,
): string[] {
  const problems: string[] = [];
  const match = NAME.exec(decision.name);
  if (match === null) {
    problems.push(
      "its name isn't its issue's number, a letter if the issue made more than one, and lowercase words joined by `-`, such as `381-decision-files.md`",
    );
  } else if (Number(match[1]) <= lastNumbered) {
    problems.push(
      `its number isn't above ${lastNumbered}, the last in decisions.md, so it would read as one of those`,
    );
  }
  const { text } = decision;
  if (text.includes("\r")) problems.push("it has a carriage return; end its lines with `\\n` only");
  if (!text.endsWith("\n") || text.endsWith("\n\n")) {
    problems.push("it doesn't end with exactly one line break");
  }
  const parts = split(text);
  if (parts === undefined) {
    return [
      ...problems,
      "it doesn't start with frontmatter naming its section, between `---` lines",
    ];
  }
  const section = parts.fields.get("section");
  const others = [...parts.fields.keys()].filter((key) => key !== "section");
  if (section === undefined || section === "") {
    problems.push("its frontmatter doesn't name its `section`");
  } else if (!log.sections.includes(section)) {
    problems.push(
      `its section "${section}" isn't one of decisions.md's: ${log.sections.map((s) => `"${s}"`).join(", ")}`,
    );
  }
  if (others.length > 0) {
    problems.push(`its frontmatter has more than \`section\`: ${others.join(", ")}`);
  }
  if (headlineOf(decision) === "") {
    problems.push("it doesn't start with the decision in a bold sentence, `**...**`");
  }
  if (!parts.body.includes("_Why:_")) problems.push("it has no `_Why:_`");
  if (!parts.body.includes("_From:_")) problems.push("it has no `_From:_`");
  const from = posix.join(decisionsDir, decision.name);
  for (const [, href = ""] of parts.body.matchAll(/\]\(([^)\s]+)(?:\s+"[^"]*")?\)/g)) {
    // An address, or a heading of the file.
    if (/^[a-z][a-z\d+.-]*:/i.test(href) || href.startsWith("//") || href.startsWith("#")) continue;
    const file = posix.normalize(
      posix.join(posix.dirname(from), decodeURIComponent(href.split("#")[0] ?? "")),
    );
    if (href.startsWith("/") || file === ".." || file.startsWith("../")) {
      problems.push(`the link ${href} leads outside the repository`);
    } else if (!exists(file.replace(/\/$/, ""))) {
      problems.push(`the link ${href} isn't a file in the repository`);
    }
  }
  return problems;
}

/** Every problem with the decisions under `root`, each prefixed with its file. */
export function checkDecisions(root: string = repository): string[] {
  const exists = (file: string): boolean => existsSync(join(root, ...file.split("/")));
  const log = readLog(root);
  const decisions = readDecisions(root);
  const problems = decisions.flatMap((decision) =>
    checkDecision(decision, log, exists).map(
      (problem) => `${decisionsDir}/${decision.name}: ${problem}`,
    ),
  );
  if (log.last > lastNumbered) {
    problems.push(
      `${decisionsLog}: decision ${log.last} is above ${lastNumbered}, and decisions.md takes no new decisions; move it into a file of its own in ${decisionsDir}/, named by its issue (see its README.md)`,
    );
  }
  // Two files can't share an id, and an issue that made more than one
  // decision letters all of them, so "decision #381" names one file.
  const byNumber = new Map<string, string[]>();
  for (const { name } of decisions) {
    const id = idOf(name);
    if (id === undefined) continue;
    const number = id.replace(/[a-z]$/, "");
    byNumber.set(number, [...(byNumber.get(number) ?? []), id]);
  }
  for (const [number, ids] of byNumber) {
    if (ids.length > 1 && (new Set(ids).size < ids.length || ids.includes(number))) {
      problems.push(
        `${decisionsDir}: issue ${number} has more than one decision, so each needs its own letter, such as ${number}a and ${number}b`,
      );
    }
  }
  return problems;
}

/** The decisions under `root` as Markdown, under their sections, in the log's order. */
export function listDecisions(root: string = repository, only?: string): string {
  const { sections } = readLog(root);
  const decisions = readDecisions(root);
  const out: string[] = [];
  for (const section of sections) {
    if (only !== undefined && section.toLowerCase() !== only.toLowerCase()) continue;
    const these = decisions.filter((decision) => sectionOf(decision) === section);
    if (these.length === 0) continue;
    out.push(`## ${section}`, "");
    for (const decision of these) {
      const path = relative(
        dirname(join(root, decisionsLog)),
        join(root, decisionsDir, decision.name),
      );
      out.push(
        `- [Decision #${idOf(decision.name) ?? decision.name}](${path.split("\\").join("/")}): ${headlineOf(decision)}`,
      );
    }
    out.push("");
  }
  return out.join("\n");
}

if (import.meta.main) {
  const [command, ...rest] = process.argv.slice(2);
  if (command === "check") {
    const problems = checkDecisions();
    for (const problem of problems) process.stderr.write(`error: ${problem}\n`);
    process.exit(problems.length > 0 ? 1 : 0);
  } else if (command === "list") {
    process.stdout.write(listDecisions(repository, rest.length > 0 ? rest.join(" ") : undefined));
  } else {
    process.stderr.write("usage: node scripts/repo-docs/decisions.ts check | list [section]\n");
    process.exit(2);
  }
}
