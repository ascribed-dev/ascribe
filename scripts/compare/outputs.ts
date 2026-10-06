// Compares what two builds of Ascribe write, to show that a change meant to
// change nothing (a clean-up, project-docs/optimization/) didn't.
//
//   node scripts/compare/outputs.ts --base <revision>
//   node scripts/compare/outputs.ts --before <ascribe> --after <ascribe>
//
// --base builds the `before` binary from that revision, in a temporary git
// worktree, and the `after` binary from this checkout (or takes --after).
//
// The projects are every folder under examples/ with an ascribe.toml, and
// docs/. They're copied, with what docs/ reads from the rest of the
// repository, into a temporary git repository set up as the determinism test
// sets it up (crates/tessera-cli/tests/determinism.rs): its one commit has
// "the" made "a" in every code file and every second page, so `diff` and
// `drift` always have the same changes to report. In that copy both binaries
// run `check`, `build` (every output, then the site again with anchors),
// `diff`, and `drift`, in each of their formats, and the script compares
// their reports and output files byte for byte. Paths to the copy, the
// checkout, or the worktree are replaced with `<root>` first.
//
// With --base, it also builds examples/astro-site with the packages at both
// revisions (each with its own binary) and compares `dist/`. Astro names
// assets by a hash of their content, which is the same in both checkouts, so
// the files are compared as they are. --skip-site leaves this out; it needs
// `pnpm install` to have run.
//
// It prints each file that differs with a short diff, then one line per
// project: `same (N files)` or `N of M files differ`. It exits 1 when anything
// differs, and 2 when it couldn't compare, which includes a `before` binary
// that fails on a project that should pass or builds nothing: two identical
// failures would otherwise count as `same`. examples/getting-started (a broken
// link, on purpose) and examples/docs-repository (its sources need
// `ascribe sources fetch`) are expected to fail, so only their reports are
// compared, not built outputs.
//
// The outputs stay in --out (a new temporary directory by default). The
// checkout isn't written to, except for the Astro example's `dist/` and the
// packages' builds.
import { execFileSync, spawnSync } from "node:child_process";
import {
  cpSync,
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
import process from "node:process";
import { fileURLToPath } from "node:url";
import { parseArgs } from "node:util";

const root = fileURLToPath(new URL("../..", import.meta.url));
const exe = process.platform === "win32" ? "ascribe.exe" : "ascribe";
/** Stands for the checkout or worktree an output came from. */
export const ROOT = "<root>";
/** Lines of diff shown per file. */
const DIFF_LINES = 40;

/** What's copied: the projects, and what docs/ascribe.toml's `[sources.code]` reads besides them. */
const COPIED = ["examples", "docs", ".github/workflows", "crates/tessera-cli/tests/output"];

/** Projects whose `check` and `build` stop with errors, by design. */
export const FAILING = ["examples/docs-repository", "examples/getting-started"];

/** Each command run on each project, as in the determinism test. */
const COMMANDS = [
  ["check"],
  ["check", "--format", "json"],
  ["diff", "--base", "HEAD"],
  ["diff", "--base", "HEAD", "--format", "json"],
  ["diff", "--base", "HEAD", "--format", "html"],
  ["drift", "--base", "HEAD"],
  ["drift", "--base", "HEAD", "--format", "json"],
  ["drift", "--base", "HEAD", "--format", "summary"],
  ["build"],
  ["build", "--emit", "site", "--anchors"],
];

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
      "skip-site": { type: "boolean", default: false },
      out: { type: "string" },
    },
  });
  if ((options.base === undefined) === (options.before === undefined)) {
    fail(
      "usage: outputs.ts --base <revision> [--after <ascribe>] [--skip-site] [--out <dir>]\n" +
        "       outputs.ts --before <ascribe> --after <ascribe> [--out <dir>]",
    );
  }
  if (options.before !== undefined && options.after === undefined) {
    fail("--before needs --after");
  }
  const out = path.resolve(options.out ?? mkdtempSync(path.join(tmpdir(), "ascribe-compare-")));
  mkdirSync(out, { recursive: true });

  let worktree: string | undefined;
  try {
    const binaries = {} as Record<Side, string>;
    if (options.base !== undefined) {
      const base = revision(options.base);
      const at = path.join(out, "worktree");
      console.error(`Checking out ${options.base} (${base.slice(0, 12)}) in ${at}`);
      git(root, "worktree", "add", "--detach", "--force", at, base);
      worktree = at;
      binaries.before = binary(worktree, path.join(out, "bin", "before"));
    } else if (options.before !== undefined) {
      binaries.before = path.resolve(options.before);
    }
    binaries.after =
      options.after === undefined
        ? binary(root, path.join(out, "bin", "after"))
        : path.resolve(options.after);

    const copy = path.join(out, "copy");
    const files = copyProjects(copy);
    const spellings = rootSpellings([copy, root, ...(worktree ? [worktree] : [])]);
    const results: [string, number, number][] = [];
    for (const project of projects(files)) {
      const runs = {} as Record<Side, Run>;
      for (const side of SIDES) {
        console.error(`Running ${side} on ${project}`);
        runs[side] = runProject(binaries[side], copy, project, spellings, side === "before");
      }
      results.push([project, ...compare(runs, project, path.join(out, "projects", project))]);
    }
    if (worktree !== undefined && !options["skip-site"]) {
      const sites = {} as Record<Side, Run>;
      for (const side of SIDES) {
        const checkout = side === "before" ? worktree : root;
        console.error(`Building examples/astro-site ${side}`);
        sites[side] = buildSite(checkout, binaries[side], spellings);
      }
      const name = "examples/astro-site/dist";
      results.push([`${name} (Astro)`, ...compare(sites, name, path.join(out, "site"))]);
    }

    console.log();
    for (const [name, differ, total] of results) {
      console.log(
        `${name}: ${differ === 0 ? `same (${total} files)` : `${differ} of ${total} files differ`}`,
      );
    }
    console.error(`\nThe outputs are in ${out}`);
    process.exitCode = results.some(([, differ]) => differ > 0) ? 1 : 0;
  } finally {
    if (worktree !== undefined) {
      // Cleaning up mustn't hide what stopped the comparison.
      try {
        git(root, "worktree", "remove", "--force", worktree);
      } catch (e) {
        console.error(`warning: couldn't remove the worktree ${worktree}: ${String(e)}`);
      }
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

/** `text` with every whole word "the" made "a": a change that keeps every directive, region, and link working. */
export function reword(text: string): string {
  return text.replace(/\bthe\b/g, "a");
}

/**
 * Copies the projects into `to`, as a repository whose one commit has the
 * words changed and whose working tree is the projects as they are here.
 * Returns the files copied, with `/`.
 */
function copyProjects(to: string): string[] {
  // Tracked files, and new ones not yet added, but nothing ignored, such as a build.
  const listed = git(
    root,
    "ls-files",
    "-z",
    "--cached",
    "--others",
    "--exclude-standard",
    "--",
    ...COPIED,
  );
  const files = [...new Set(listed.split("\0").filter((f) => f !== ""))]
    .filter((f) => existsSync(path.join(root, f)))
    .sort();
  let pages = 0;
  const originals: [string, Buffer][] = [];
  for (const file of files) {
    const bytes = readFileSync(path.join(root, file));
    const name = path.posix.basename(file);
    const target = path.join(to, file);
    mkdirSync(path.dirname(target), { recursive: true });
    let reworded: string | undefined;
    if (!isBinary(bytes) && name !== "ascribe.toml" && name !== "ascribe.lock") {
      const text = bytes.toString("utf8");
      if (Buffer.from(text, "utf8").equals(bytes)) {
        if (!name.endsWith(".md") || ++pages % 2 === 0) {
          reworded = reword(text);
        }
      }
    }
    if (reworded !== undefined && reworded !== bytes.toString("utf8")) {
      writeFileSync(target, reworded);
      originals.push([target, bytes]);
    } else {
      writeFileSync(target, bytes);
    }
  }
  const commit = (...args: string[]): void => {
    execFileSync(
      "git",
      [
        "-c",
        "user.name=Mira Okafor",
        "-c",
        "user.email=mira@example.com",
        "-c",
        "commit.gpgsign=false",
        "-c",
        "core.autocrlf=false",
        "-c",
        "init.defaultBranch=main",
        ...args,
      ],
      {
        cwd: to,
        stdio: ["ignore", "ignore", "inherit"],
        env: {
          ...process.env,
          GIT_AUTHOR_DATE: "2026-10-01T12:00:00+00:00",
          GIT_COMMITTER_DATE: "2026-10-01T12:00:00+00:00",
        },
      },
    );
  };
  commit("init", "-q");
  commit("add", "-A");
  commit("commit", "-q", "-m", "The base");
  for (const [target, bytes] of originals) {
    writeFileSync(target, bytes);
  }
  return files;
}

/** The projects among `files`: each folder with an ascribe.toml, with `/`. */
export function projects(files: string[]): string[] {
  return files
    .filter((f) => f.endsWith("/ascribe.toml"))
    .map((f) => f.slice(0, -"/ascribe.toml".length))
    .sort();
}

/** The project's `[project] output-dir`, or the default, relative to the project. */
export function outputDir(toml: string): string {
  for (const line of toml.split(/\r?\n/)) {
    const value = /^\s*output-dir\s*=\s*(?:"([^"]*)"|'([^']*)')/.exec(line);
    if (value) {
      return value[1] ?? value[2] ?? "";
    }
  }
  return ".ascribe/build";
}

/**
 * Runs every command on `project`, in the copy at `copy`, with `ascribe`:
 * each one's report, and each output file. With `strict`, a command that
 * exits other than as expected, or a build that writes nothing, means the
 * comparison can't be trusted, and fails it.
 */
function runProject(
  ascribe: string,
  copy: string,
  project: string,
  spellings: string[],
  strict: boolean,
): Run {
  const dir = path.join(copy, project);
  const outputs = path.join(dir, outputDir(readFileSync(path.join(dir, "ascribe.toml"), "utf8")));
  const failing = FAILING.includes(project);
  const found: Run = new Map();
  for (const args of COMMANDS) {
    const name = `ascribe ${args.join(" ")}`;
    rmSync(outputs, { recursive: true, force: true });
    const result = spawnSync(ascribe, ["--color", "never", ...args], {
      cwd: dir,
      maxBuffer: 1 << 30,
    });
    if (result.error) {
      fail(`can't run ${ascribe}: ${result.error.message}`);
    }
    const expected = failing && (args[0] === "check" || args[0] === "build") ? 1 : 0;
    if (strict && result.status !== expected) {
      fail(
        `${project}: \`${name}\` with the before binary exited ${result.status}, not ${expected}, so there's nothing to compare\n${result.stderr.toString()}`,
      );
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
    if (args[0] === "build") {
      const written = existsSync(outputs) ? filesUnder(outputs).filter((f) => f !== ".lock") : [];
      if (strict && !failing && written.length === 0) {
        fail(`${project}: \`${name}\` with the before binary wrote nothing to ${outputs}`);
      }
      for (const file of written) {
        found.set(`${name}/${file}`, normalize(readFileSync(path.join(outputs, file)), spellings));
      }
    }
  }
  rmSync(outputs, { recursive: true, force: true });
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
 * a short diff of each file that differs, and returns how many do, of how many.
 */
function compare(runs: Record<Side, Run>, label: string, out: string): [number, number] {
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
  return [differ, names.length];
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
    // Anything that stops the comparison is a 2, not the 1 that means "they differ".
    console.error(e instanceof Failure ? `error: ${e.message}` : e);
    process.exitCode = 2;
  }
}
