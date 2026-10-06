// Compares what two builds of Ascribe write, to show that a change meant to
// change nothing (a clean-up, project-docs/optimization/) didn't.
//
//   node scripts/compare/outputs.ts --base <revision>
//   node scripts/compare/outputs.ts --before <ascribe> --after <ascribe>
//
// --base builds the `before` binary from that revision, in a temporary git
// worktree, and the `after` binary from this checkout (or takes --after).
// For every project here (each folder under examples/ with an ascribe.toml,
// and docs/), both binaries run `check` (text and JSON) and `build` (every
// output, then the site again with anchors), and the script compares their
// reports and output files byte for byte. Paths to the checkout or the
// worktree are replaced with `<root>` first, so two checkouts compare equal.
//
// With --base, it also builds examples/astro-site with the packages at both
// revisions (each with its own binary) and compares `dist/`. Astro names
// assets by a hash of their content, which is the same in both checkouts, so
// the files are compared as they are. --skip-site leaves this out; it needs
// `pnpm install` to have run.
//
// --diff-base <revision> also runs `diff` (text, JSON, and HTML) and `drift`
// (text and JSON) against that revision, which must be in the history.
//
// It prints each file that differs with a short diff, then one line per
// project, `same` or `N files differ`. It exits 1 when anything differs, and
// 2 when it couldn't compare.
// The outputs stay in --out (a new temporary directory by default).
//
// Each project's output folder is moved aside while the script runs and put
// back after, so a build you have there is kept.
import { execFileSync, spawnSync } from "node:child_process";
import {
  cpSync,
  existsSync,
  mkdirSync,
  mkdtempSync,
  readdirSync,
  readFileSync,
  renameSync,
  rmSync,
  writeFileSync,
} from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";
import process from "node:process";
import { fileURLToPath } from "node:url";
import { parseArgs } from "node:util";

const root = fileURLToPath(new URL("../..", import.meta.url));
const exe = process.platform === "win32" ? "ascribe.exe" : "ascribe";
/** Stands for the checkout or worktree an output came from. */
export const ROOT = "<root>";
/** Lines of diff shown per file. */
const DIFF_LINES = 40;

type Side = "before" | "after";
const SIDES: Side[] = ["before", "after"];

/** What one command printed, and the files it wrote. */
type Run = Map<string, Buffer>;

function main(): void {
  const { values: options } = parseArgs({
    options: {
      base: { type: "string" },
      before: { type: "string" },
      after: { type: "string" },
      "diff-base": { type: "string" },
      "skip-site": { type: "boolean", default: false },
      out: { type: "string" },
    },
  });
  if ((options.base === undefined) === (options.before === undefined)) {
    fail(
      "usage: outputs.ts --base <revision> [--after <ascribe>] [--diff-base <revision>] [--skip-site] [--out <dir>]\n" +
        "       outputs.ts --before <ascribe> --after <ascribe> [--diff-base <revision>] [--out <dir>]",
    );
  }
  if (options.before !== undefined && options.after === undefined) {
    fail("--before needs --after");
  }
  const out = path.resolve(options.out ?? mkdtempSync(path.join(tmpdir(), "ascribe-compare-")));
  mkdirSync(out, { recursive: true });
  const diffBase = options["diff-base"] && revision(options["diff-base"]);

  let worktree: string | undefined;
  try {
    const binaries = {} as Record<Side, string>;
    if (options.base !== undefined) {
      const base = revision(options.base);
      worktree = path.join(out, "worktree");
      console.error(`Checking out ${options.base} (${base.slice(0, 12)}) in ${worktree}`);
      git(root, "worktree", "add", "--detach", "--force", worktree, base);
      binaries.before = binary(worktree, path.join(out, "bin", "before"));
    } else if (options.before !== undefined) {
      binaries.before = path.resolve(options.before);
    }
    binaries.after =
      options.after === undefined
        ? binary(root, path.join(out, "bin", "after"))
        : path.resolve(options.after);

    const spellings = rootSpellings([root, ...(worktree ? [worktree] : [])]);
    const results: [string, number][] = [];
    for (const project of projects()) {
      const runs = {} as Record<Side, Run>;
      for (const side of SIDES) {
        console.error(`Running ${side} on ${project}`);
        runs[side] = runProject(binaries[side], project, diffBase, spellings);
      }
      results.push([project, compare(runs, project, path.join(out, "projects", project))]);
    }
    if (worktree !== undefined && !options["skip-site"]) {
      const sites = {} as Record<Side, Run>;
      for (const side of SIDES) {
        const checkout = side === "before" ? worktree : root;
        console.error(`Building examples/astro-site ${side}`);
        sites[side] = buildSite(checkout, binaries[side], spellings);
      }
      const name = "examples/astro-site/dist";
      results.push([`${name} (Astro)`, compare(sites, name, path.join(out, "site"))]);
    }

    console.log();
    for (const [name, n] of results) {
      console.log(
        `${name}: ${n === 0 ? "same" : `${n} ${n === 1 ? "file differs" : "files differ"}`}`,
      );
    }
    console.error(`\nThe outputs are in ${out}`);
    process.exitCode = results.some(([, n]) => n > 0) ? 1 : 0;
  } finally {
    if (worktree !== undefined) {
      git(root, "worktree", "remove", "--force", worktree);
    }
  }
}

/** The commit `name` is, or a failure that says it isn't in the history. */
function revision(name: string): string {
  const found = spawnSync("git", ["rev-parse", "--verify", "--quiet", `${name}^{commit}`], {
    cwd: root,
    encoding: "utf8",
  });
  if (found.status !== 0) {
    fail(`${name} isn't a commit in this repository's history (a shallow clone may not have it)`);
  }
  return found.stdout.trim();
}

/** Builds `ascribe` in `checkout` and copies it to `to`, so the next build can't replace it. */
function binary(checkout: string, to: string): string {
  console.error(`Building ascribe in ${checkout}`);
  // One target directory for both, so the dependencies are built once.
  run(
    "cargo",
    ["build", "--locked", "-p", "tessera-cli", "--target-dir", path.join(root, "target")],
    checkout,
  );
  mkdirSync(to, { recursive: true });
  const copy = path.join(to, exe);
  cpSync(path.join(root, "target", "debug", exe), copy);
  return copy;
}

/** Each project: a folder under examples/ with an ascribe.toml, and docs/, relative to the root with `/`. */
export function projects(): string[] {
  const found: string[] = [];
  const walk = (dir: string): void => {
    for (const entry of readdirSync(path.join(root, dir), { withFileTypes: true })) {
      if (
        entry.isDirectory() &&
        !["node_modules", ".ascribe", "dist", ".astro"].includes(entry.name)
      ) {
        walk(`${dir}/${entry.name}`);
      } else if (entry.name === "ascribe.toml") {
        found.push(dir);
      }
    }
  };
  walk("examples");
  found.push("docs");
  return found.sort();
}

/** The project's `[project] output-dir`, or the default, relative to the project. */
export function outputDir(toml: string): string {
  const line = toml.split(/\r?\n/).find((l) => /^\s*output-dir\s*=/.test(l));
  return line?.replace(/^[^=]*=\s*"([^"]*)".*$/, "$1") ?? ".ascribe/build";
}

/** Runs every command on `project` with `ascribe`: each one's report, and each output file. */
function runProject(
  ascribe: string,
  project: string,
  diffBase: string | undefined,
  spellings: string[],
): Run {
  const dir = path.join(root, project);
  const outputs = path.join(dir, outputDir(readFileSync(path.join(dir, "ascribe.toml"), "utf8")));
  const commands: string[][] = [
    ["check"],
    ["check", "--format", "json"],
    ["build"],
    ["build", "--emit", "site", "--anchors"],
  ];
  if (diffBase !== undefined) {
    commands.push(
      ["diff", "--base", diffBase],
      ["diff", "--base", diffBase, "--format", "json"],
      ["diff", "--base", diffBase, "--format", "html"],
      ["drift", "--base", diffBase],
      ["drift", "--base", diffBase, "--format", "json"],
    );
  }
  const found: Run = new Map();
  const saved = `${outputs}.compare-saved`;
  const hadOutputs = existsSync(outputs);
  // The folders above the output folder that a build would make, to remove after.
  const made: string[] = [];
  for (
    let d = path.dirname(outputs);
    d.startsWith(dir + path.sep) && !existsSync(d);
    d = path.dirname(d)
  ) {
    made.push(d);
  }
  if (hadOutputs) {
    renameSync(outputs, saved);
  }
  try {
    for (const args of commands) {
      const name = `ascribe ${args.join(" ")}`;
      const result = spawnSync(ascribe, ["--color", "never", ...args], {
        cwd: dir,
        maxBuffer: 1 << 30,
      });
      if (result.error) {
        fail(`can't run ${ascribe}: ${result.error.message}`);
      }
      found.set(
        `${name}.txt`,
        Buffer.concat([
          Buffer.from(`exit ${result.status}\n--- stdout\n`),
          normalize(result.stdout, spellings),
          Buffer.from("\n--- stderr\n"),
          normalize(result.stderr, spellings),
        ]),
      );
      if (args[0] === "build" && existsSync(outputs)) {
        for (const file of filesUnder(outputs)) {
          if (file !== ".lock") {
            found.set(
              `${name}/${file}`,
              normalize(readFileSync(path.join(outputs, file)), spellings),
            );
          }
        }
        rmSync(outputs, { recursive: true, force: true });
      }
    }
  } finally {
    rmSync(outputs, { recursive: true, force: true });
    if (hadOutputs) {
      renameSync(saved, outputs);
    }
    for (const d of made) {
      rmSync(d, { recursive: true, force: true });
    }
  }
  return found;
}

/** Builds the Astro example in `checkout` with its own packages and `ascribe`: the files of `dist/`. */
function buildSite(checkout: string, ascribe: string, spellings: string[]): Run {
  if (checkout !== root) {
    run("pnpm", ["install", "--frozen-lockfile"], checkout);
  }
  // What the site's build imports, in the order js.yml builds them.
  for (const name of ["cli", "elements", "review", "astro"]) {
    run("pnpm", ["--filter", `@ascribed/${name}`, "build"], checkout);
  }
  const site = path.join(checkout, "examples", "astro-site");
  const dist = path.join(site, "dist");
  rmSync(dist, { recursive: true, force: true });
  run("pnpm", ["--filter", "@ascribed/example-astro-site", "build"], checkout, {
    ASCRIBE_BIN: ascribe,
    ASTRO_TELEMETRY_DISABLED: "1",
  });
  const found: Run = new Map();
  for (const file of filesUnder(dist)) {
    found.set(file, normalize(readFileSync(path.join(dist, file)), spellings));
  }
  return found;
}

/**
 * Compares two runs of `label`. Writes each side's files under `out`, prints
 * a short diff of each file that differs, and returns how many do.
 */
function compare(runs: Record<Side, Run>, label: string, out: string): number {
  const names = [...new Set([...runs.before.keys(), ...runs.after.keys()])].sort();
  let differ = 0;
  for (const side of SIDES) {
    for (const [name, bytes] of runs[side]) {
      const file = path.join(out, side, safeName(name));
      mkdirSync(path.dirname(file), { recursive: true });
      writeFileSync(file, bytes);
    }
  }
  for (const name of names) {
    const [before, after] = [runs.before.get(name), runs.after.get(name)];
    if (before !== undefined && after !== undefined && before.equals(after)) {
      continue;
    }
    differ += 1;
    console.log(`\n${label}: ${name}`);
    if (before === undefined || after === undefined) {
      console.log(`  only ${before === undefined ? "after" : "before"}`);
    } else if (isBinary(before) || isBinary(after)) {
      console.log(`  binary files differ (${before.length} and ${after.length} bytes)`);
    } else {
      console.log(
        shortDiff(
          path.join(out, "before", safeName(name)),
          path.join(out, "after", safeName(name)),
        ),
      );
    }
  }
  return differ;
}

/** A command's name as a path: `ascribe build/site/x.md` stays a path, and spaces and `-`s do no harm. */
function safeName(name: string): string {
  return name.replaceAll(" ", "_");
}

/** At most DIFF_LINES lines of `git diff` between two files, without its header. */
function shortDiff(a: string, b: string): string {
  const result = spawnSync("git", ["diff", "--no-index", "--no-color", "-U2", "--", a, b], {
    encoding: "utf8",
    maxBuffer: 1 << 30,
  });
  const lines = result.stdout.split("\n");
  const start = lines.findIndex((l) => l.startsWith("@@"));
  const body = lines.slice(start === -1 ? 0 : start).filter((l) => l !== "");
  const shown = body.slice(0, DIFF_LINES).map((l) => `  ${l}`);
  if (body.length > DIFF_LINES) {
    shown.push(`  … ${body.length - DIFF_LINES} more lines`);
  }
  return shown.join("\n");
}

export function isBinary(bytes: Buffer): boolean {
  return bytes.includes(0);
}

/**
 * The ways each of `dirs` can appear in an output: as it is, with the other
 * separator, and escaped for JSON. The longest first, so a shorter one doesn't
 * replace part of a longer one.
 */
export function rootSpellings(dirs: string[]): string[] {
  const forms = new Set<string>();
  for (const dir of dirs) {
    const bare = dir.replace(/[\\/]+$/, "");
    for (const form of [bare, bare.replaceAll("\\", "/"), bare.replaceAll("\\", "\\\\")]) {
      forms.add(form);
    }
  }
  return [...forms].sort((a, b) => b.length - a.length);
}

/** `bytes` with each spelling of a root replaced by ROOT, unless it's a binary file. */
export function normalize(bytes: Buffer, spellings: string[]): Buffer {
  if (isBinary(bytes)) {
    return bytes;
  }
  let text = bytes.toString("utf8");
  if (!Buffer.from(text, "utf8").equals(bytes)) {
    return bytes;
  }
  for (const form of spellings) {
    text = text.replaceAll(form, ROOT);
  }
  return Buffer.from(text, "utf8");
}

/** Every file under `dir`, relative to it with `/`. */
function filesUnder(dir: string): string[] {
  return readdirSync(dir, { recursive: true, withFileTypes: true })
    .filter((e) => e.isFile())
    .map((e) => path.relative(dir, path.join(e.parentPath, e.name)).replaceAll("\\", "/"))
    .sort();
}

function run(command: string, args: string[], cwd: string, env: Record<string, string> = {}): void {
  const result = spawnSync(command, args, {
    cwd,
    // Its output is progress, like this script's own: on standard error.
    stdio: ["ignore", 2, 2],
    env: { ...process.env, ...env },
    // pnpm is a script on Windows.
    shell: process.platform === "win32",
  });
  if (result.status !== 0) {
    fail(`${command} ${args.join(" ")} failed in ${cwd}`);
  }
}

function git(cwd: string, ...args: string[]): string {
  return execFileSync("git", args, { cwd, encoding: "utf8", stdio: ["ignore", "pipe", "inherit"] });
}

/** A reason the comparison couldn't be made. */
class Failure extends Error {}

function fail(message: string): never {
  throw new Failure(message);
}

if (
  process.argv[1] !== undefined &&
  path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)
) {
  try {
    main();
  } catch (e) {
    if (!(e instanceof Failure)) {
      throw e;
    }
    console.error(`error: ${e.message}`);
    process.exitCode = 2;
  }
}
