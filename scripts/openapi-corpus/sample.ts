// The corpus OpenAPI's phase 1 reads: a seeded sample of APIs.guru's
// directory, each API's preferred version, downloaded as JSON into a folder
// for the reader to run over. It's downloaded, so it's kept out of the test
// suite; the pull request that changes the reader records the run.
//
//   node scripts/openapi-corpus/sample.ts <folder> [count] [seed]
//   cargo run --release -p ascribe-resolve --example openapi_corpus -- <folder>
//
// APIs.guru's own host is mirrored on its repository's gh-pages branch,
// which is read when the host can't be reached.
import { mkdir, writeFile } from "node:fs/promises";
import path from "node:path";

const LIST = "https://api.apis.guru/v2/list.json";
const MIRROR = "https://raw.githubusercontent.com/APIs-guru/openapi-directory/gh-pages/v2/";

interface Version {
  swaggerUrl: string;
}
interface Api {
  preferred: string;
  versions: Record<string, Version>;
}

/** The text at `url`, or at its mirror when APIs.guru's host fails. */
async function fetchText(url: string): Promise<string> {
  for (const candidate of [url, url.replace("https://api.apis.guru/v2/", MIRROR)]) {
    try {
      const response = await fetch(candidate);
      if (response.ok) return await response.text();
    } catch {
      // The next candidate.
    }
  }
  throw new Error(`couldn't download ${url}`);
}

/** A seeded random number generator, mulberry32, so a sample can be drawn again. */
function random(seed: number): () => number {
  let state = seed >>> 0;
  return () => {
    state = (state + 0x6d2b79f5) >>> 0;
    let t = state;
    t = Math.imul(t ^ (t >>> 15), t | 1);
    t ^= t + Math.imul(t ^ (t >>> 7), t | 61);
    return ((t ^ (t >>> 14)) >>> 0) / 4294967296;
  };
}

const [folder, count = "300", seed = "917"] = process.argv.slice(2);
if (folder === undefined) {
  console.error("usage: node scripts/openapi-corpus/sample.ts <folder> [count] [seed]");
  process.exit(2);
}
const list = JSON.parse(await fetchText(LIST)) as Record<string, Api>;
const next = random(Number(seed));
const names = Object.keys(list).sort();
// A Fisher-Yates shuffle, then the first `count`.
for (let i = names.length - 1; i > 0; i--) {
  const j = Math.floor(next() * (i + 1));
  [names[i], names[j]] = [names[j] as string, names[i] as string];
}
const sample = names.slice(0, Number(count));
await mkdir(folder, { recursive: true });
let downloaded = 0;
for (const name of sample) {
  const api = list[name];
  const version = api?.versions[api.preferred];
  if (version === undefined) continue;
  try {
    const text = await fetchText(version.swaggerUrl);
    await writeFile(path.join(folder, `${name.replaceAll(/[^\w.-]+/g, "_")}.json`), text);
    downloaded++;
  } catch (error) {
    console.error(String(error));
  }
}
console.log(`downloaded ${downloaded} of ${sample.length} specs into ${folder}`);
