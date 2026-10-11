// The site's screenshots: the pictures of the editor and of review on the
// landing page and Why Ascribe, each in light and dark, and beside them the
// release each was taken at. This retakes them, one name or every one:
//
//   node scripts/docs-site/screenshots.ts [review | editor]
//
// Each picture's own script takes it (the steps below), and this writes its
// record, `<name>.json`, with the release `[versions] current` in
// docs/ascribe.toml names now. screenshots.test.ts fails when a record is
// more than one minor release behind it. The review picture needs the site
// installed with the Ascribe to show (`npm run build:checkout` in site/); the
// editor's needs ASCRIBE_BIN and Linux with Xvfb. site/README.md has the rest.
import { execFileSync } from "node:child_process";
import { readFileSync, writeFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { parse } from "smol-toml";

const ROOT = fileURLToPath(new URL("../..", import.meta.url));

/** Where the pictures and their records are, under home/'s content root. */
export const FOLDER = "site/home/content/screenshots";

/** A picture: its name, which its files start with, and the commands that take it. */
interface Picture {
  name: string;
  steps: { cwd: string; command: string[] }[];
}

export const PICTURES: Picture[] = [
  {
    name: "review",
    steps: [{ cwd: "site", command: ["node", "scripts/screenshot.ts", path.join(ROOT, FOLDER)] }],
  },
  {
    name: "editor",
    steps: [
      { cwd: "packages/vscode", command: ["node", "esbuild.mjs"] },
      {
        cwd: "packages/vscode",
        command: ["node", "scripts/screenshot.ts", path.join(ROOT, FOLDER)],
      },
    ],
  },
];

export const SCHEMES = ["light", "dark"] as const;

/** The command that retakes the picture `name`. */
export const retake = (name: string): string => `node scripts/docs-site/screenshots.ts ${name}`;

/** What a picture's record says. */
export interface Taken {
  /** The release the picture was taken at: `[versions] current` then. */
  release: string;
}

/** `[versions] current` in docs/ascribe.toml: the release the docs describe. */
export function currentRelease(toml = readFileSync(path.join(ROOT, "docs/ascribe.toml"), "utf8")) {
  const current = (parse(toml) as { versions?: { current?: unknown } }).versions?.current;
  if (typeof current !== "string") throw new Error("docs/ascribe.toml has no [versions] current");
  return current;
}

/** How many minor releases a picture may be behind the current one. */
export const ALLOWED = 1;

function minors(version: string): [major: number, minor: number] {
  const found = /^(\d+)\.(\d+)\.\d+/.exec(version);
  if (found === null) throw new Error(`not a release: ${version}`);
  return [Number(found[1]), Number(found[2])];
}

/**
 * Why the picture `name`, taken at `release`, is out of date for `current`,
 * or undefined when it isn't: when it's more than ALLOWED minor releases
 * behind, or from an earlier major.
 */
export function stale(name: string, release: string, current: string): string | undefined {
  const [major, minor] = minors(release);
  const [currentMajor, currentMinor] = minors(current);
  const behind = currentMajor > major || currentMinor - minor > ALLOWED;
  if (!behind) return undefined;
  return `${FOLDER}/${name}-*.png show ${release}, more than ${ALLOWED} minor release behind ${current}: run \`${retake(name)}\``;
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
    const file = path.join(FOLDER, `${picture.name}.json`);
    writeFileSync(path.join(ROOT, file), `${JSON.stringify(record, null, 2)}\n`);
    console.log(`Wrote ${file}`);
  }
}
