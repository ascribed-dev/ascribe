// The sources of the crawlers Ascribe knows, `crates/ascribe-model/data/crawlers.toml`:
// each token's operator page, and the day it was read. A test in the model
// fails on an entry without either; what this finds can't fail a commit,
// since a page goes away or a date grows old without one.
//
//   node scripts/crawlers/sources.ts   list each source read over a year ago,
//                                      or that doesn't answer, as Markdown,
//                                      and exit 1 when there's one
//
// The daily Report workflow runs it and opens an issue with what it prints.
// RELEASING.md's step to reread the sources runs it too.
import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { parse } from "smol-toml";

/** The data file. */
const dataFile = resolve(
  fileURLToPath(import.meta.url),
  "../../../crates/ascribe-model/data/crawlers.toml",
);

/** How long a source stays fresh after it's read: a year. */
const FRESH_DAYS = 365;

/** One token's entry, as much as this reads. */
export interface Crawler {
  token: string;
  source: string;
  read: string;
}

/** The entries of the data file's text. */
export function crawlers(text: string): Crawler[] {
  const data = parse(text) as { crawler?: Partial<Crawler>[] };
  return (data.crawler ?? []).map((c) => ({
    token: c.token ?? "",
    source: c.source ?? "",
    read: c.read ?? "",
  }));
}

/** Each entry read more than a year before `today`, or with no date that parses, as a sentence. */
export function stale(entries: readonly Crawler[], today: Date): string[] {
  const problems: string[] = [];
  for (const entry of entries) {
    const read = Date.parse(`${entry.read}T00:00:00Z`);
    const days = (today.getTime() - read) / 86_400_000;
    if (Number.isNaN(read) || days > FRESH_DAYS)
      problems.push(
        `\`${entry.token}\`: its source was read on ${entry.read || "no date"}, over a year ago. Reread ${entry.source}, then update \`read\`.`,
      );
  }
  return problems;
}

/** Each source that doesn't answer with a success, after redirects, as a sentence; each address asked once. */
export async function gone(
  entries: readonly Crawler[],
  fetchStatus: (url: string) => Promise<number | string>,
): Promise<string[]> {
  const problems: string[] = [];
  const asked = new Map<string, Promise<number | string>>();
  for (const entry of entries) {
    if (!asked.has(entry.source)) asked.set(entry.source, fetchStatus(entry.source));
    const status = await asked.get(entry.source);
    if (typeof status !== "number" || status >= 400)
      problems.push(
        `\`${entry.token}\`: ${entry.source} answered ${String(status)}. Find where its operator documents it now, or remove the token.`,
      );
  }
  return problems;
}

/** The status a GET of `url` ends at, or why it failed. */
async function status(url: string): Promise<number | string> {
  try {
    const response = await fetch(url, {
      redirect: "follow",
      headers: { "User-Agent": "Mozilla/5.0 (compatible; ascribe-crawler-sources)" },
      signal: AbortSignal.timeout(30_000),
    });
    return response.status;
  } catch (error) {
    return error instanceof Error ? error.message : String(error);
  }
}

if (import.meta.main) {
  const entries = crawlers(readFileSync(dataFile, "utf8"));
  const problems = [...stale(entries, new Date()), ...(await gone(entries, status))];
  if (problems.length === 0) {
    console.log(
      `Every source of the ${entries.length} crawlers answers, and was read within a year.`,
    );
  } else {
    console.log("## Crawler sources\n");
    console.log(
      "The crawlers Ascribe knows, in `crates/ascribe-model/data/crawlers.toml`, need their sources reread:\n",
    );
    for (const problem of problems) console.log(`- ${problem}`);
    process.exitCode = 1;
  }
}
