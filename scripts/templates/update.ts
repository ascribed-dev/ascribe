// Takes The Good Docs Project's newest release: when their repository has a
// release tag newer than the pin in upstream.toml, it moves the pin, writes
// the copies again with convert.ts, makes examples/templates again with
// `ascribe templates eject`, and writes the description of a pull request
// saying what changed (https://github.com/ascribed-dev/ascribe/blob/180eee83/project-docs/page-templates/phase-6-upstream.md).
// .github/workflows/templates.yml runs it every week and opens the pull
// request; run here, it changes the working tree and nothing else.
//
//   node scripts/templates/update.ts --body FILE   take the newest release
//   node scripts/templates/update.ts --pin TAG     set the pin to TAG and write
//                                                  the copies, for trying the
//                                                  update from an older release
//
// It prints one line of JSON: `status` is "current" (nothing newer, nothing
// changed), "updated" (the pin moved and the copies are written), or "failed"
// (the pin moved, a rule no longer finds its place, and no copy changed), with
// `from`, `to`, and `title`. With "updated" or "failed", the description is in
// FILE. A failure of anything else exits 1.
//
// It reads their tags, and the diff between two of them, from GitLab's API,
// which answers without signing in (README decision 25). It builds this
// repository's `ascribe` with cargo, since the binary carries the set.
import { spawnSync } from "node:child_process";
import { cpSync, mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";
import {
  ConversionError,
  generate,
  type Pin,
  readPin,
  readRules,
  SET,
  upstream,
  write,
} from "./convert.ts";

const root = fileURLToPath(new URL("../..", import.meta.url));
const pinFile = path.join(root, "scripts", "templates", "upstream.toml");
const example = path.join(root, "examples", "templates");

/** A tag of theirs and the commit it names, as GitLab lists it. */
export interface Tag {
  name: string;
  commit: string;
}

/** An entry of their index.json, as much of it as this reads. */
export interface IndexEntry {
  name: string;
  summary: string;
  files: { template?: string; guide?: string };
  packs?: string[];
}

/** What the update found, for the description. */
export interface Found {
  from: Pin;
  to: Pin;
  /** Each template of the set, with what changed in it between the two tags. */
  changed: { name: string; what: string[] }[];
  /** Templates of their Core pack that the older tag didn't have. */
  added: string[];
  /** Why the copies weren't written, when they weren't. */
  failure?: string;
}

/** A release tag's version, or undefined for any other tag. */
export function version(tag: string): [number, number, number] | undefined {
  const found = /^v(\d+)\.(\d+)\.(\d+)$/.exec(tag);
  return found === null ? undefined : [Number(found[1]), Number(found[2]), Number(found[3])];
}

/** Which of two release tags is newer: negative when `a` is older. */
export function compareTags(a: string, b: string): number {
  const [x, y] = [version(a), version(b)];
  if (x === undefined || y === undefined) throw new Error(`not a release tag: ${a} or ${b}`);
  return x[0] - y[0] || x[1] - y[1] || x[2] - y[2];
}

/** The newest release tag, or undefined when none is a release's. */
export function newest(tags: Tag[]): Tag | undefined {
  return tags
    .filter((tag) => version(tag.name) !== undefined)
    .sort((a, b) => compareTags(b.name, a.name))[0];
}

/** upstream.toml's text with the pin moved, its comments kept. */
export function movePin(text: string, to: Tag): string {
  let moved = text;
  for (const [key, value] of [
    ["tag", to.name],
    ["commit", to.commit],
  ] as const) {
    const line = new RegExp(`^${key} = ".*"$`, "m");
    if (!line.test(moved)) throw new Error(`upstream.toml has no \`${key} = "..."\` line`);
    moved = moved.replace(line, `${key} = ${JSON.stringify(value)}`);
  }
  return moved;
}

/**
 * Which templates of the set changed between two releases, by the files the
 * diff between their tags names, and which templates their Core pack gained.
 * Their index's `last_updated` isn't read: it doesn't follow releases.
 */
export function whatChanged(
  names: string[],
  paths: string[],
  before: IndexEntry[],
  after: IndexEntry[],
): Pick<Found, "changed" | "added"> {
  const changed = names.flatMap((name) => {
    const file = `template_${name}.md`;
    const old = before.find((e) => e.files.template === file);
    const now = after.find((e) => e.files.template === file);
    const what: string[] = [];
    if (paths.includes(`${name}/${file}`)) what.push("the template");
    const guides = new Set([old?.files.guide, now?.files.guide].filter((g) => g !== undefined));
    if ([...guides].some((guide) => paths.includes(`${name}/${guide}`))) what.push("the guide");
    if (old?.summary !== now?.summary) what.push("its summary in their index");
    return what.length > 0 ? [{ name, what }] : [];
  });
  const core = (index: IndexEntry[]) =>
    index.filter((e) => e.packs?.includes("core")).map((e) => e.files.template ?? e.name);
  const had = new Set(core(before));
  const added = after
    .filter((e) => e.packs?.includes("core") && !had.has(e.files.template ?? e.name))
    .map((e) => (e.files.template === undefined ? e.name : `${e.name} (${e.files.template})`));
  return { changed, added };
}

/** Their repository's page on what changed between two tags. */
export function compareLink(from: Pin, to: Pin): string {
  return `${from.repository}/-/compare/${from.tag}...${to.tag}`;
}

/** The pull request's title. */
export function title(found: Found): string {
  return found.failure === undefined
    ? `Take The Good Docs Project's templates at ${found.to.tag}`
    : `The Good Docs Project's ${found.to.tag} stops the templates' conversion`;
}

/** The pull request's description, before the workflow adds what it ran. */
export function describe(found: Found): string {
  const { from, to } = found;
  const link = `[${from.tag}...${to.tag}](${compareLink(from, to)})`;
  const lines: string[] = [];
  if (found.failure !== undefined) {
    lines.push(
      `The Good Docs Project released ${to.tag}, and converting it stops: a rule in \`scripts/templates/rules.toml\` no longer finds its place. This pull request moves the pin from ${from.tag} and changes no template.`,
      "",
      "```text",
      found.failure,
      "```",
      "",
      `Read what they changed, ${link}, decide what the rule should now say, and change it on this branch. Then \`node scripts/templates/update.ts --pin ${to.tag}\` writes the copies and makes \`examples/templates\` again.`,
    );
  } else {
    lines.push(
      `The Good Docs Project released ${to.tag}. This moves the pin from ${from.tag} and writes Ascribe's copies of their templates again, and \`examples/templates\` with them. Their words ship in Ascribe's binary, so read the diff before merging.`,
      "",
      `What changed in the set, from the diff between their tags, ${link}:`,
      "",
      ...(found.changed.length > 0
        ? found.changed.map(({ name, what }) => `- **${name}**: ${what.join(", ")}`)
        : [
            "- Nothing of theirs in the six templates or their guides: the copies change only where they name the release.",
          ]),
    );
  }
  if (found.added.length > 0)
    lines.push(
      "",
      "Their Core pack has a template it didn't have at the pin, which isn't added: the set is decision 10's six, and a seventh is a decision.",
      "",
      ...found.added.map((name) => `- ${name}`),
    );
  return `${lines.join("\n")}\n`;
}

// Their repository, through GitLab's API.

function api(pin: Pin): string {
  const url = new URL(pin.repository);
  if (url.host !== "gitlab.com") throw new Error(`not a GitLab repository: ${pin.repository}`);
  return `https://gitlab.com/api/v4/projects/${encodeURIComponent(url.pathname.slice(1))}`;
}

async function get(url: string): Promise<unknown> {
  const response = await fetch(url, { signal: AbortSignal.timeout(30_000) });
  if (!response.ok) throw new Error(`${url} answered ${response.status}`);
  return response.json();
}

/** Their tags, newest release first among the first hundred GitLab lists. */
export async function tags(pin: Pin): Promise<Tag[]> {
  const listed = (await get(`${api(pin)}/repository/tags?per_page=100`)) as {
    name: string;
    commit: { id: string };
  }[];
  return listed.map((tag) => ({ name: tag.name, commit: tag.commit.id }));
}

/** The paths the diff between two tags names, old and new. */
export async function changedPaths(from: Pin, to: Pin): Promise<string[]> {
  const query = new URLSearchParams({ from: from.tag, to: to.tag, straight: "true" });
  const compared = (await get(`${api(from)}/repository/compare?${query}`)) as {
    diffs: { old_path: string; new_path: string }[];
    compare_timeout?: boolean;
  };
  if (compared.compare_timeout === true)
    throw new Error(`GitLab timed out comparing ${from.tag} with ${to.tag}`);
  return [...new Set(compared.diffs.flatMap((d) => [d.old_path, d.new_path]))];
}

// The working tree.

function run(command: string, args: string[], cwd = root): string {
  const done = spawnSync(command, args, { cwd, encoding: "utf8" });
  if (done.status !== 0)
    throw new Error(`${command} ${args.join(" ")} failed:\n${done.stdout}${done.stderr}`);
  return done.stdout;
}

/** This repository's `ascribe`, built now, so it carries the set as written. */
function build(): string {
  run("cargo", ["build", "--quiet", "--locked", "-p", "ascribe-cli"]);
  return path.join(
    root,
    "target",
    "debug",
    process.platform === "win32" ? "ascribe.exe" : "ascribe",
  );
}

/**
 * Writes the copies, then makes examples/templates' templates again as
 * `ascribe templates eject` with no name writes them, with the binary built
 * from those copies.
 */
function writeCopies(files: Map<string, string>): void {
  write(files);
  const ascribe = build();
  const dir = mkdtempSync(path.join(tmpdir(), "ascribe-example-"));
  try {
    mkdirSync(path.join(dir, "docs"));
    writeFileSync(
      path.join(dir, "ascribe.toml"),
      'spec = "0.1"\n\n[project]\ncontent-root = "docs"\n',
    );
    // A folder of its own, in no repository: `--force` gets eject past the
    // git guard, which has nothing there to protect.
    run(ascribe, ["templates", "eject", "--force"], dir);
    rmSync(path.join(example, "templates"), { recursive: true, force: true });
    cpSync(path.join(dir, "templates"), path.join(example, "templates"), { recursive: true });
    cpSync(path.join(dir, "ascribe.toml"), path.join(example, "ascribe.toml"));
  } finally {
    rmSync(dir, { recursive: true, force: true });
  }
}

function setPin(to: Tag): void {
  writeFileSync(pinFile, movePin(readFileSync(pinFile, "utf8"), to));
}

/** Takes the newest release, if it's newer than the pin. */
export async function update(): Promise<{ found?: Found }> {
  const from = readPin();
  const latest = newest(await tags(from));
  if (latest === undefined || compareTags(latest.name, from.tag) <= 0) return {};
  const to: Pin = { repository: from.repository, tag: latest.name, commit: latest.commit };
  const [paths, before, after] = await Promise.all([
    changedPaths(from, to),
    upstream(from, "index.json").then((text) => JSON.parse(text) as IndexEntry[]),
    upstream(to, "index.json").then((text) => JSON.parse(text) as IndexEntry[]),
  ]);
  const found: Found = {
    from,
    to,
    ...whatChanged([...readRules().keys()], paths, before, after),
  };
  setPin(latest);
  let files: Map<string, string>;
  try {
    // Every template converts before anything is written, so a rule that no
    // longer finds its place changes no copy.
    files = await generate(build());
  } catch (error) {
    if (!(error instanceof ConversionError)) throw error;
    return { found: { ...found, failure: error.message } };
  }
  writeCopies(files);
  return { found };
}

function option(name: string): string | undefined {
  const at = process.argv.indexOf(name);
  return at === -1 ? undefined : process.argv[at + 1];
}

if (import.meta.main) {
  const pinTo = option("--pin");
  if (pinTo !== undefined) {
    const tag = (await tags(readPin())).find((t) => t.name === pinTo);
    if (tag === undefined) {
      console.error(`Their repository has no tag ${pinTo}.`);
      process.exit(2);
    }
    setPin(tag);
    writeCopies(await generate(build()));
    console.log(`The pin is ${tag.name}, and ${SET} and examples/templates are what it gives.`);
  } else {
    const body = option("--body");
    if (body === undefined) {
      console.error("Say where the description goes: --body FILE.");
      process.exit(2);
    }
    const { found } = await update();
    if (found === undefined) {
      console.log(JSON.stringify({ status: "current", from: readPin().tag }));
    } else {
      writeFileSync(body, describe(found));
      console.log(
        JSON.stringify({
          status: found.failure === undefined ? "updated" : "failed",
          from: found.from.tag,
          to: found.to.tag,
          title: title(found),
        }),
      );
    }
  }
}
