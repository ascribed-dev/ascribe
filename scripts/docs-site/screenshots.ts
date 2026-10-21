// The screenshots: the pictures of the editor and of review on the landing
// page and Why Ascribe, and the docs' scenes of the editor, the site, and the
// report, each in light and dark, and beside them the release each was taken at. This retakes them, one
// name or every one:
//
//   node scripts/docs-site/screenshots.ts [review | editor | build-lens | ...]
//
// Each picture's own script takes it (the steps below), and this writes its
// record, `<name>.json`, with the current line of `[releases.ascribe]` in
// docs/ascribe.toml: the release the docs describe now. screenshots.test.ts fails when a record is
// more than one minor release behind it. The review picture needs the site
// installed with the Ascribe to show (`npm run build:checkout` in site/); the
// editor's and the scenes need ASCRIBE_BIN, the status bar's scene a POSIX
// shell, and the site's and the report's scenes examples/astro-site installed. site/README.md has the rest.
import { execFileSync } from "node:child_process";
import { readFileSync, writeFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { parse } from "smol-toml";

const ROOT = fileURLToPath(new URL("../..", import.meta.url));

/** Where the site's pictures and their records are, under home/'s content root. */
export const FOLDER = "site/home/content/screenshots";

/** Where the docs' scenes and their records are, under the docs' content root. */
export const SCENES = "docs/content/images";

/** A picture: its name, which its files start with, its folder, and the commands that take it. */
interface Picture {
  name: string;
  folder: string;
  steps: { cwd: string; command: string[] }[];
}

/** A picture of the editor, which packages/vscode/scripts/screenshot.ts takes by its name. */
const editor = (name: string, folder: string): Picture => ({
  name,
  folder,
  steps: [
    { cwd: "packages/vscode", command: ["node", "esbuild.mjs", "--scenes"] },
    {
      cwd: "packages/vscode",
      command: ["node", "scripts/screenshot.ts", path.join(ROOT, folder), name],
    },
  ],
});

/** A picture of review in the site, which site/scripts/screenshot.ts takes by its name. */
const site = (name: string, folder: string): Picture => ({
  name,
  folder,
  steps: [
    { cwd: "site", command: ["node", "scripts/screenshot.ts", path.join(ROOT, folder), name] },
  ],
});

/** A scene of an Astro site or the report, which examples/astro-site/scripts/screenshot.ts takes. */
const example = (name: string): Picture => ({
  name,
  folder: SCENES,
  steps: [
    {
      cwd: "examples/astro-site",
      command: ["node", "scripts/screenshot.ts", path.join(ROOT, SCENES), name],
    },
  ],
});

export const PICTURES: Picture[] = [
  site("review", FOLDER),
  editor("editor", FOLDER),
  editor("build-lens", SCENES),
  editor("actions-bar", SCENES),
  editor("review-view", SCENES),
  editor("status-bar", SCENES),
  editor("shown-in-pages", SCENES),
  editor("this-page", SCENES),
  editor("pages", SCENES),
  editor("project", SCENES),
  site("review-panel", SCENES),
  example("html-report"),
  example("elements"),
  example("starter-sidebar"),
  example("versions"),
];

export const SCHEMES = ["light", "dark"] as const;

/** The command that retakes the picture `name`. */
export const retake = (name: string): string => `node scripts/docs-site/screenshots.ts ${name}`;

/** What a picture's record says. */
export interface Taken {
  /** The release the picture was taken at: the current line then. */
  release: string;
}

/**
 * What the docs take code examples from: each pattern of `[sources.code]
 * include` in docs/ascribe.toml, up to its first wildcard, as a path from the
 * repository's root. The review picture's copy of the repository holds these.
 */
export function codeSources(toml = readFileSync(path.join(ROOT, "docs/ascribe.toml"), "utf8")) {
  const include = (parse(toml) as { sources?: { code?: { include?: unknown } } }).sources?.code
    ?.include;
  const patterns = Array.isArray(include) ? include.filter((p) => typeof p === "string") : [];
  return patterns
    .map((pattern) => {
      const segments = pattern.split("/");
      const wild = segments.findIndex((segment) => /[*?[{]/.test(segment));
      return (wild === -1 ? segments : segments.slice(0, wild)).join("/");
    })
    .filter((source) => source !== "");
}

/**
 * The current line of `[releases.ascribe]` in docs/ascribe.toml, the newest
 * live one: the release the docs describe.
 */
export function currentRelease(toml = readFileSync(path.join(ROOT, "docs/ascribe.toml"), "utf8")) {
  const live = (parse(toml) as { releases?: { ascribe?: { live?: unknown } } }).releases?.ascribe
    ?.live;
  const lines = Array.isArray(live) ? live.filter((l) => typeof l === "string") : [];
  if (lines.length === 0)
    throw new Error("docs/ascribe.toml has no live line in [releases.ascribe]");
  return lines.reduce((newest, line) => (newer(line, newest) ? line : newest));
}

/** Whether version `a` is later than `b`, part by part, missing parts as 0. */
function newer(a: string, b: string): boolean {
  const pa = a.split(".").map(Number);
  const pb = b.split(".").map(Number);
  for (let i = 0; i < Math.max(pa.length, pb.length); i++) {
    const d = (pa[i] ?? 0) - (pb[i] ?? 0);
    if (d !== 0) return d > 0;
  }
  return false;
}

/** How many minor releases a picture may be behind the current one. */
export const ALLOWED = 1;

function minors(version: string): [major: number, minor: number] {
  const found = /^(\d+)\.(\d+)\.\d+/.exec(version);
  if (found === null) throw new Error(`not a release: ${version}`);
  return [Number(found[1]), Number(found[2])];
}

/**
 * Why the picture `name` in `folder`, taken at `release`, is out of date for
 * `current`, or undefined when it isn't: when it's more than ALLOWED minor
 * releases behind, or from an earlier major.
 */
export function stale(
  name: string,
  release: string,
  current: string,
  folder = FOLDER,
): string | undefined {
  const [major, minor] = minors(release);
  const [currentMajor, currentMinor] = minors(current);
  const behind = currentMajor > major || currentMinor - minor > ALLOWED;
  if (!behind) return undefined;
  return `${folder}/${name}-*.png show ${release}, more than ${ALLOWED} minor release behind ${current}: run \`${retake(name)}\``;
}

if (import.meta.main) {
  const names = process.argv.slice(2);
  const unknown = names.filter((name) => !PICTURES.some((p) => p.name === name));
  if (unknown.length > 0) {
    console.error(
      `No picture named ${unknown.join(", ")}: ${PICTURES.map((p) => p.name).join(", ")}`,
    );
    process.exit(2);
  }
  const release = currentRelease();
  for (const picture of PICTURES) {
    if (names.length > 0 && !names.includes(picture.name)) continue;
    for (const { cwd, command } of picture.steps) {
      const [program = "node", ...args] = command;
      execFileSync(program, args, { cwd: path.join(ROOT, cwd), stdio: "inherit" });
    }
    const record: Taken = { release };
    const file = path.join(picture.folder, `${picture.name}.json`);
    writeFileSync(path.join(ROOT, file), `${JSON.stringify(record, null, 2)}\n`);
    console.log(`Wrote ${file}`);
  }
}
