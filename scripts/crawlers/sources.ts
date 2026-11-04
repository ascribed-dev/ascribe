// The sources of the crawlers Ascribe knows, `crates/ascribe-model/data/crawlers.toml`:
// each token's operator page, and the day it was read. A test in the model
// fails on an entry without either; what this finds can't fail a commit,
// since a page goes away or a date grows old without one.
//
//   node scripts/crawlers/sources.ts   list each source read over a year ago,
//                                      or that doesn't answer, as Markdown,
//                                      and exit 1 when there's one; and list
//                                      each source that refused to be asked,
//                                      which exits 0
//
// A source that answers 429 when asked again is unchecked, not gone: Apple's
// support pages answer 429 to every request from an Actions runner (#1223),
// so the answer says nothing of the page. The run's summary names it, and
// `stale` still names it once it's a year since it was read.
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

/** How long to wait before asking a source again, in milliseconds. */
const RETRY_MS = 10_000;

/**
 * Whether an answer may be the network's or the server's passing failure: a request that failed,
 * a 429 (the server is limiting how often it's asked, not saying the page moved), or a 5xx.
 */
function passing(status: number | string): boolean {
  return typeof status !== "number" || status === 429 || status >= 500;
}

/** What asking the sources found, each as a sentence. */
export interface Answers {
  /** The sources that don't answer with a success. */
  gone: string[];
  /** The sources that answered 429 when asked again: refused to be asked, so not checked. */
  unchecked: string[];
}

/**
 * Each source that doesn't answer with a success, after redirects; each address asked once, and
 * again after `pause` when the first answer may be a passing failure, so one dropped connection
 * doesn't open an issue. A source that answers 429 when asked again is unchecked instead.
 */
export async function gone(
  entries: readonly Crawler[],
  fetchStatus: (url: string) => Promise<number | string>,
  pause: () => Promise<void> = () => new Promise((done) => setTimeout(done, RETRY_MS)),
): Promise<Answers> {
  const ask = async (url: string): Promise<number | string> => {
    const first = await fetchStatus(url);
    if (!passing(first)) return first;
    await pause();
    return fetchStatus(url);
  };
  const answers: Answers = { gone: [], unchecked: [] };
  const asked = new Map<string, Promise<number | string>>();
  for (const entry of entries) {
    if (!asked.has(entry.source)) asked.set(entry.source, ask(entry.source));
    const status = await asked.get(entry.source);
    if (status === 429)
      answers.unchecked.push(
        `\`${entry.token}\`: ${entry.source} answered 429 twice, so it wasn't checked. Open it by hand before a release.`,
      );
    else if (typeof status !== "number" || status >= 400)
      answers.gone.push(
        `\`${entry.token}\`: ${entry.source} answered ${String(status)}. Find where its operator documents it now, or remove the token.`,
      );
  }
  return answers;
}

/** The status a GET of `url` ends at, or why it failed. */
async function status(url: string): Promise<number | string> {
  try {
    const response = await fetch(url, {
      redirect: "follow",
      headers: {
        "User-Agent": "Mozilla/5.0 (compatible; ascribe-crawler-sources)",
        // Without it, fetch sends `*`, and Google's docs redirect to a language chosen at random.
        "Accept-Language": "en",
      },
      signal: AbortSignal.timeout(30_000),
    });
    return response.status;
  } catch (error) {
    return reason(error);
  }
}

/** An error's message, with its cause's: fetch says only "fetch failed", and the cause says why. */
function reason(error: unknown): string {
  if (!(error instanceof Error)) return String(error);
  return error.cause === undefined ? error.message : `${error.message} (${reason(error.cause)})`;
}

if (import.meta.main) {
  const entries = crawlers(readFileSync(dataFile, "utf8"));
  const answers = await gone(entries, status);
  const problems = [...stale(entries, new Date()), ...answers.gone];
  if (problems.length === 0) {
    const checked = entries.length - answers.unchecked.length;
    console.log(
      answers.unchecked.length === 0
        ? `Every source of the ${entries.length} crawlers answers, and was read within a year.`
        : `Every source of the ${entries.length} crawlers was read within a year, and the ${checked} checked answer.`,
    );
  } else {
    console.log("## Crawler sources\n");
    console.log(
      "The crawlers Ascribe knows, in `crates/ascribe-model/data/crawlers.toml`, need their sources reread:\n",
    );
    for (const problem of problems) console.log(`- ${problem}`);
    process.exitCode = 1;
  }
  if (answers.unchecked.length > 0) {
    console.log("\n## Crawler sources not checked\n");
    console.log("These refused to be asked, so this run can't say whether they still answer:\n");
    for (const sentence of answers.unchecked) console.log(`- ${sentence}`);
  }
}
