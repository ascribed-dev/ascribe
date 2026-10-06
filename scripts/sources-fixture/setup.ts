// Builds the sources fixture: a code repository, and a docs repository whose
// examples come from it, with the update pull request's workflow
// (examples/docs-repository), for running that workflow for real
// (project-docs/docs/phase-10-update-pull-request.md).
//
//   node scripts/sources-fixture/setup.ts <owner> [--pass] [--dir <dir>] [--force]
//   node scripts/sources-fixture/setup.ts --local [--dir <dir>]
//
// <owner> is a GitHub organization or user with two repositories already
// created, empty or built by this script before: sources-fixture-code and
// sources-fixture-docs. Each run rebuilds both from nothing: it force-pushes
// their `main`, and closes the docs repository's update pull request. It
// refuses a repository whose README it didn't write, unless --force. The docs
// repository's workflow needs a GitHub App that can read the code repository
// and write to the docs repository: docs/content/guides/drift.md says how.
//
// --pass then runs the pass: it makes each scripted change in the code
// repository (scripts/sources-fixture/changes.ts), runs the docs repository's
// update workflow, and checks the pull request it opens, updates, closes, or
// leaves alone, and that pull request's checks, against what the change
// should do. It ends with the code as it was built and no pull request open,
// ready for the schedule.
//
// --local builds both repositories on this machine, each with a bare
// repository beside it as its remote, and runs the same pass with the
// workflow's own steps, a stand-in for `gh` (fake-gh.ts), and no GitHub.
//
// It needs `git`, an `ascribe` binary (ASCRIBE_BIN, or `cargo build -p
// tessera-cli`), and for --local, `bash` and `jq`. For GitHub, it needs the
// GitHub CLI (`gh`), signed in with access to both repositories, and `git`
// able to read and push them (`gh auth setup-git`).
import { execFileSync, spawnSync } from "node:child_process";
import {
  cpSync,
  existsSync,
  mkdirSync,
  mkdtempSync,
  readFileSync,
  symlinkSync,
  writeFileSync,
} from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import process from "node:process";
import { fileURLToPath } from "node:url";
import { parseArgs } from "node:util";

import { CODE, type Change, git, makeChange } from "./changes.ts";
import type { State } from "./fake-gh.ts";

const root = fileURLToPath(new URL("../..", import.meta.url));
const here = join(root, "scripts", "sources-fixture");
const recipe = join(root, "examples", "docs-repository");
const CODE_NAME = "sources-fixture-code";
const DOCS_NAME = "sources-fixture-docs";
/** The branch the update workflow pushes, and its workflow's file. */
const BRANCH = "ascribe/update-sources";
const WORKFLOW = "update-sources.yml";
/** The code repository's address in examples/docs-repository/ascribe.toml. */
const EXAMPLE_URL = `https://github.com/ascribed-dev/${CODE_NAME}.git`;
/** The first line of each fixture repository's README: how a later run knows the repository is its own. */
const MARK = "<!-- ascribe sources fixture: rebuilt by scripts/sources-fixture/setup.ts -->";

const { values: options, positionals } = parseArgs({
  allowPositionals: true,
  options: {
    dir: { type: "string" },
    local: { type: "boolean", default: false },
    pass: { type: "boolean", default: false },
    force: { type: "boolean", default: false },
  },
});
const [owner] = positionals;
if (options.local ? owner !== undefined || options.pass : !/^[\w.-]+$/.test(owner ?? "")) {
  fail(
    "usage: setup.ts <owner> [--pass] [--dir <dir>] [--force]\n       setup.ts --local [--dir <dir>]",
  );
}

const ascribe = binary();
const work = options.dir ?? mkdtempSync(join(tmpdir(), "ascribe-sources-fixture-"));
const code = join(work, "code");
const docs = join(work, "docs");
const cache = join(work, "cache");
for (const dir of [code, docs]) {
  if (existsSync(dir)) fail(`${dir} exists already; choose another --dir, or remove it.`);
}
const repository = (name: string) => `${owner}/${name}`;
const remote = (name: string) =>
  owner === undefined ? join(work, `${name}.git`) : `https://github.com/${repository(name)}.git`;

if (owner !== undefined) {
  for (const name of [CODE_NAME, DOCS_NAME]) checkRepository(repository(name));
}

// --- The code repository: CODE, and the scheduled change ---

mkdirSync(code, { recursive: true });
git(code, "init", "-q", "-b", "main");
for (const [path, text] of Object.entries(CODE)) write(code, path, text);
write(
  code,
  "README.md",
  readme(
    "Ascribe sources fixture: code",
    `A made-up client library for Lantern, whose examples [${DOCS_NAME}](../${DOCS_NAME}) shows. A workflow here makes a scripted change every Monday, Wednesday, and Friday (\`.github/change.ts\`), so the docs repository's update pull request has something to follow.`,
  ),
);
write(code, ".github/change.ts", readFileSync(join(here, "changes.ts"), "utf8"));
write(code, ".github/workflows/change.yml", readFileSync(join(here, "change.yml"), "utf8"));
commit(code, "The Lantern SDK");
publish(code, CODE_NAME);

// --- The docs repository: examples/docs-repository, with its copies fetched ---

cpSync(recipe, docs, {
  recursive: true,
  filter: (source) => !/[\\/](README\.md|\.ascribe|sources|ascribe\.lock)$/.test(source),
});
git(docs, "init", "-q", "-b", "main");
write(
  docs,
  "README.md",
  readme(
    "Ascribe sources fixture: docs",
    `An Ascribe project whose examples come from [${CODE_NAME}](../${CODE_NAME}), copied into \`sources/\` and pinned in \`ascribe.lock\`. \`.github/workflows/${WORKFLOW}\` moves the pins on weekdays and opens a pull request when an example changed.`,
  ),
);
write(docs, ".gitignore", ".ascribe/\n");
const codeUrl = owner === undefined ? fileUrl(remote(CODE_NAME)) : remote(CODE_NAME);
edit(docs, "ascribe.toml", (text) => {
  if (!text.includes(EXAMPLE_URL))
    fail(`examples/docs-repository/ascribe.toml doesn't name ${EXAMPLE_URL}.`);
  return text.replaceAll(EXAMPLE_URL, codeUrl);
});
run(ascribe, ["sources", "fetch"], docs);
run(ascribe, ["check", "--deny-warnings"], docs);
commit(docs, "The Lantern SDK's docs");
if (owner !== undefined) {
  for (const open of ghJson<{ number: number }[]>(
    "pr",
    "list",
    "--repo",
    repository(DOCS_NAME),
    "--head",
    BRANCH,
    "--state",
    "open",
    "--json",
    "number",
  )) {
    gh(
      "pr",
      "close",
      String(open.number),
      "--repo",
      repository(DOCS_NAME),
      "--comment",
      "Closed by the fixture's setup script, which rebuilds this repository.",
    );
  }
}
publish(docs, DOCS_NAME);
deleteBranch();

console.log(`
The fixture is built.

  code  ${owner === undefined ? code : `https://github.com/${repository(CODE_NAME)}`}
  docs  ${owner === undefined ? docs : `https://github.com/${repository(DOCS_NAME)}`}
`);

// --- The pass ---

/** What a run should do to the update pull request. */
type Outcome = "opened" | "updated" | "closed" | "none" | "left";

interface Step {
  /** Someone pushes a commit of their own to the update branch first. */
  edit?: boolean;
  change: Change;
  outcome: Outcome;
  /** The pages the description lists among the examples that changed. */
  changed?: string[];
  /** The pages it lists among the examples that no longer resolve: `ascribe check` fails on them. */
  broken?: string[];
}

const PASS: Step[] = [
  { change: "example", outcome: "opened", changed: ["login.md"], broken: [] },
  { change: "rename", outcome: "updated", changed: ["login.md"], broken: ["connect.md"] },
  { change: "restore", outcome: "closed" },
  { change: "unrelated", outcome: "none" },
  { change: "move", outcome: "opened", changed: [], broken: ["quickstart.md"] },
  { edit: true, change: "example", outcome: "left" },
];

if (options.local || options.pass) {
  const host = owner === undefined ? localHost() : gitHubHost();
  const rows: string[] = [];
  let failed = 0;
  for (const step of PASS) {
    if (step.edit) host.edit();
    makeChange(code, step.change);
    git(code, "push", "-q", "origin", "main");
    const before = host.snapshot();
    const seconds = host.run();
    const after = host.snapshot();
    const outcome = classify(before, after);
    const problems = outcome === step.outcome ? check(step, after) : [`expected ${step.outcome}`];
    if (problems.length === 0 && after.open !== undefined && step.broken !== undefined) {
      const passed = host.checks(after.open.number);
      if (passed !== (step.broken.length === 0)) {
        problems.push(`its checks ${passed ? "passed" : "failed"}`);
      }
    }
    failed += problems.length > 0 ? 1 : 0;
    rows.push(
      `  ${`${step.edit ? "edit, then " : ""}${step.change}`.padEnd(20)} ${outcome.padEnd(8)} ${`${seconds}s`.padStart(5)}  ${
        problems.length > 0 ? `WRONG: ${problems.join("; ")}` : "right"
      }`,
    );
    console.log(rows.at(-1));
    if (after.open !== undefined && (problems.length > 0 || owner === undefined)) {
      console.log(indent(`${after.open.title}\n\n${after.open.body}`));
    }
  }
  // Ready for the schedule: the code as it was built, and no pull request.
  makeChange(code, "restore");
  git(code, "push", "-q", "origin", "main");
  host.close();
  deleteBranch();
  console.log(
    `\nThe pass:\n  ${"change".padEnd(20)} ${"pull".padEnd(8)} ${"run".padStart(5)}  description\n${rows.join("\n")}`,
  );
  if (failed > 0) fail(`\n${failed} of ${PASS.length} steps weren't right.`);
}

interface Snapshot {
  /** The open update pull request, with the commit its branch is at. */
  open?: { number: number; title: string; body: string; head: string };
}

interface Host {
  /** Runs the update workflow; returns how long it took, in seconds. */
  run(): number;
  snapshot(): Snapshot;
  /** Whether the pull request's checks pass. */
  checks(number: number): boolean;
  /** Pushes a commit of someone else's to the update branch. */
  edit(): void;
  /** Closes the open update pull request, if there is one. */
  close(): void;
}

function classify(before: Snapshot, after: Snapshot): Outcome {
  if (before.open === undefined) return after.open === undefined ? "none" : "opened";
  if (after.open === undefined) return "closed";
  if (after.open.number !== before.open.number) return "opened";
  return after.open.head === before.open.head ? "left" : "updated";
}

/** What's wrong with the pull request's description, for a step that opens or updates one. */
function check(step: Step, after: Snapshot): string[] {
  if (after.open === undefined || step.broken === undefined) return [];
  const body = after.open.body;
  const problems: string[] = [];
  const group = (lead: string) => {
    const start = body.indexOf(lead);
    if (start < 0) return [];
    const rest = body.slice(start + lead.length).split(/\n(?=\n[^\n-])/)[0] ?? "";
    return Array.from(rest.matchAll(/^- \[?([^\]\n]+?)\]?(?:\(|$)/gm), (match) => match[1] ?? "");
  };
  for (const [lead, expected] of [
    ["The page shows the new code; check the words around it:", step.changed ?? []],
    ["Examples that no longer resolve:", step.broken],
  ] as const) {
    const found = group(lead);
    if (found.join() !== [...expected].sort().join()) {
      problems.push(
        `"${lead}" lists ${found.join(", ") || "nothing"}, not ${expected.join(", ") || "nothing"}`,
      );
    }
  }
  const fails = body.includes("### `ascribe check` fails");
  if (fails !== step.broken.length > 0) {
    problems.push(`the description ${fails ? "says" : "doesn't say"} \`ascribe check\` fails`);
  }
  if (!/^Update sources: api to [0-9a-f]{7}, examples to [0-9a-f]{7}$/.test(after.open.title)) {
    problems.push(`its title is "${after.open.title}"`);
  }
  return problems;
}

/** The pass on this machine: the workflow's own steps, run with bash, and fake-gh.ts for `gh`. */
function localHost(): Host {
  for (const tool of ["bash", "jq"]) {
    if (spawnSync(tool, ["--version"]).error) fail(`--local needs \`${tool}\`.`);
  }
  const bin = join(work, "bin");
  mkdirSync(bin);
  symlinkSync(ascribe, join(bin, "ascribe"));
  writeFileSync(join(bin, "gh"), `#!/bin/sh\nexec node "${join(here, "fake-gh.ts")}" "$@"\n`, {
    mode: 0o755,
  });
  const state = join(work, "pulls.json");
  const workflow = readFileSync(join(recipe, ".github", "workflows", WORKFLOW), "utf8");
  const steps = ["Move the pins", "Open or update the pull request"].map((name) =>
    runBlock(workflow, name),
  );
  const pulls = (): State =>
    existsSync(state) ? (JSON.parse(readFileSync(state, "utf8")) as State) : { pulls: [], log: [] };
  let runs = 0;
  return {
    run() {
      runs += 1;
      const checkout = join(work, `run-${runs}`);
      execFileSync("git", ["clone", "-q", "--depth", "1", fileUrl(remote(DOCS_NAME)), checkout]);
      const temp = join(checkout, "..", `run-${runs}-temp`);
      mkdirSync(temp);
      const started = Date.now();
      for (const script of steps) {
        const result = spawnSync(
          "bash",
          ["--noprofile", "--norc", "-eo", "pipefail", "-c", script],
          {
            cwd: checkout,
            encoding: "utf8",
            env: {
              ...process.env,
              PATH: `${bin}${process.platform === "win32" ? ";" : ":"}${process.env["PATH"] ?? ""}`,
              ASCRIBE_CACHE_DIR: cache,
              FAKE_GH_STATE: state,
              RUNNER_TEMP: temp,
              GITHUB_STEP_SUMMARY: join(temp, "summary.md"),
              BRANCH,
              BOT: "sources-fixture[bot]",
            },
          },
        );
        if (result.status !== 0) fail(`The workflow failed:\n${result.stdout}${result.stderr}`);
      }
      return Math.round((Date.now() - started) / 1000);
    },
    snapshot() {
      const open = pulls().pulls.find((p) => p.state === "OPEN");
      if (open === undefined) return {};
      return { open: { ...open, head: branchHead() ?? "" } };
    },
    checks(number) {
      const open = pulls().pulls.find((p) => p.number === number);
      return !(open?.body ?? "").includes("### `ascribe check` fails");
    },
    edit: () => pushEdit(),
    close() {
      const open = pulls().pulls.find((p) => p.state === "OPEN");
      if (open === undefined) return;
      const saved = pulls();
      for (const pull of saved.pulls) if (pull.number === open.number) pull.state = "CLOSED";
      writeFileSync(state, JSON.stringify(saved, null, 2));
    },
  };
}

/** The pass on GitHub: the docs repository's workflow, started by hand. */
function gitHubHost(): Host {
  const docsRepo = repository(DOCS_NAME);
  return {
    run() {
      const since = Date.now() - 5000;
      gh("workflow", "run", WORKFLOW, "--repo", docsRepo);
      let id: number | undefined;
      for (let tries = 0; id === undefined; tries++) {
        if (tries > 30) fail(`The ${WORKFLOW} run didn't start.`);
        sleep(2);
        id = ghJson<{ databaseId: number; createdAt: string }[]>(
          "run",
          "list",
          "--repo",
          docsRepo,
          "--workflow",
          WORKFLOW,
          "--event",
          "workflow_dispatch",
          "--limit",
          "1",
          "--json",
          "databaseId,createdAt",
        ).find((r) => Date.parse(r.createdAt) >= since)?.databaseId;
      }
      spawnSync("gh", ["run", "watch", String(id), "--repo", docsRepo, "--exit-status"], {
        stdio: "ignore",
      });
      const done = ghJson<{
        conclusion: string;
        startedAt: string;
        updatedAt: string;
        url: string;
      }>(
        "run",
        "view",
        String(id),
        "--repo",
        docsRepo,
        "--json",
        "conclusion,startedAt,updatedAt,url",
      );
      if (done.conclusion !== "success") fail(`The workflow failed: ${done.url}`);
      return Math.round((Date.parse(done.updatedAt) - Date.parse(done.startedAt)) / 1000);
    },
    snapshot() {
      const [open] = ghJson<{ number: number; title: string; body: string; headRefOid: string }[]>(
        "pr",
        "list",
        "--repo",
        docsRepo,
        "--head",
        BRANCH,
        "--state",
        "open",
        "--json",
        "number,title,body,headRefOid",
      );
      return open === undefined ? {} : { open: { ...open, head: open.headRefOid } };
    },
    checks(number) {
      // The checks start when the push reaches GitHub; wait for them to appear.
      for (let tries = 0; tries < 60; tries++) {
        const listed = spawnSync("gh", ["pr", "checks", String(number), "--repo", docsRepo], {
          encoding: "utf8",
        });
        if (listed.stdout.trim() !== "") break;
        sleep(3);
      }
      const watched = spawnSync(
        "gh",
        ["pr", "checks", String(number), "--repo", docsRepo, "--watch", "--interval", "10"],
        { stdio: "ignore" },
      );
      return watched.status === 0;
    },
    edit: () => pushEdit(),
    close() {
      const open = this.snapshot().open;
      if (open !== undefined) {
        gh(
          "pr",
          "close",
          String(open.number),
          "--repo",
          docsRepo,
          "--comment",
          "Closed by the fixture's pass, which is done.",
        );
      }
    },
  };
}

/** The commit the update branch is at, on the docs repository, or undefined. */
function branchHead(): string | undefined {
  const line = git(docs, "ls-remote", "origin", `refs/heads/${BRANCH}`);
  return line === "" ? undefined : line.split(/\s/)[0];
}

/** Pushes a commit to the update branch as someone else would, fixing a page by hand. */
function pushEdit(): void {
  git(docs, "fetch", "-q", "origin", BRANCH);
  git(docs, "checkout", "-q", "-B", "edit", "FETCH_HEAD");
  edit(docs, "content/quickstart.md", (text) => `${text.trimEnd()}\n\nA line added by hand.\n`);
  git(docs, "-c", "user.name=A Writer", "commit", "-q", "-am", "Add a line by hand");
  git(docs, "push", "-q", "origin", `edit:${BRANCH}`);
  git(docs, "checkout", "-q", "main");
}

/** Deletes the update branch from the docs repository, if it's there. */
function deleteBranch(): void {
  if (branchHead() !== undefined) git(docs, "push", "-q", "origin", "--delete", BRANCH);
}

/** The lines of the step's `run: |` block, without their indentation. */
function runBlock(workflow: string, name: string): string {
  const lines = workflow.split("\n");
  const start = lines.findIndex((line) => line.trim() === `- name: ${name}`);
  const run = lines.findIndex((line, i) => i > start && line.trim() === "run: |");
  if (start < 0 || run < 0) fail(`${WORKFLOW} has no step "${name}" with a \`run: |\` block.`);
  const indent = (lines[run]?.search(/\S/) ?? 0) + 2;
  const block: string[] = [];
  for (const line of lines.slice(run + 1)) {
    if (line.trim() !== "" && line.search(/\S/) < indent) break;
    block.push(line.slice(indent));
  }
  return block.join("\n");
}

/** Pushes a checkout's `main` to its repository: on GitHub, or a bare repository beside it. */
function publish(dir: string, name: string): void {
  if (owner === undefined)
    execFileSync("git", ["init", "-q", "--bare", "-b", "main", remote(name)]);
  git(dir, "remote", "add", "origin", remote(name));
  git(dir, "push", "-q", "--force", "origin", "main");
  git(dir, "branch", "-q", "--set-upstream-to=origin/main");
}

/** Refuses a repository this script didn't build, so a typo can't force-push over real work. */
function checkRepository(name: string): void {
  let readmeText: string;
  try {
    readmeText = gh("api", `repos/${name}/readme`, "--jq", ".content");
  } catch (error) {
    const message = error instanceof Error ? error.message : String(error);
    // An empty repository, or one with no README yet.
    if (/empty|404|Not Found/i.test(message)) {
      try {
        gh("api", `repos/${name}`, "--jq", ".full_name");
      } catch {
        fail(
          `${name} doesn't exist on GitHub, or the GitHub CLI can't see it. Create it first (private, with no README), then run this again.`,
        );
      }
      if (options.force || isEmpty(name)) return;
      fail(
        `${name} has files but no README from this script. Use an empty repository, or --force to overwrite it.`,
      );
    }
    throw error;
  }
  const text = Buffer.from(readmeText, "base64").toString("utf8");
  if (!text.startsWith(MARK) && !options.force) {
    fail(
      `${name} isn't a sources fixture (its README isn't this script's). Use an empty repository, or --force to overwrite it.`,
    );
  }
}

/** Whether a repository has no commits: GitHub answers 409, "Git Repository is empty", for its commits. */
function isEmpty(name: string): boolean {
  try {
    gh("api", `repos/${name}/commits?per_page=1`, "--jq", "length");
    return false;
  } catch (error) {
    return /empty|409/i.test(error instanceof Error ? error.message : String(error));
  }
}

function readme(title: string, text: string): string {
  return [
    MARK,
    "",
    `# ${title}`,
    "",
    text,
    "",
    "Don't edit it: `scripts/sources-fixture/setup.ts` in the Ascribe repository rebuilds both repositories from nothing.",
    "",
  ].join("\n");
}

/** The `ascribe` binary: ASCRIBE_BIN, or this repository's debug build. */
function binary(): string {
  const built = join(
    root,
    "target",
    "debug",
    process.platform === "win32" ? "ascribe.exe" : "ascribe",
  );
  const found = process.env["ASCRIBE_BIN"] ?? (existsSync(built) ? built : undefined);
  if (found === undefined)
    fail("No `ascribe` to run: set ASCRIBE_BIN, or run `cargo build -p tessera-cli`.");
  return found;
}

/** Runs a command in `dir` with the fixture's cache, failing with its output. */
function run(command: string, args: string[], dir: string): void {
  const result = spawnSync(command, args, {
    cwd: dir,
    encoding: "utf8",
    env: { ...process.env, ASCRIBE_CACHE_DIR: cache },
  });
  if (result.status !== 0) {
    fail(`\`${[command, ...args].join(" ")}\` failed in ${dir}:\n${result.stdout}${result.stderr}`);
  }
}

function fileUrl(path: string): string {
  const text = path.replace(/\\/g, "/");
  return text.startsWith("/") ? `file://${text}` : `file:///${text}`;
}

function write(dir: string, file: string, text: string): void {
  const path = join(dir, ...file.split("/"));
  mkdirSync(join(path, ".."), { recursive: true });
  writeFileSync(path, text);
}

function edit(dir: string, file: string, change: (text: string) => string): void {
  const path = join(dir, ...file.split("/"));
  writeFileSync(path, change(readFileSync(path, "utf8").replace(/\r\n/g, "\n")));
}

function commit(dir: string, message: string): void {
  git(dir, "add", "-A");
  git(dir, "commit", "-q", "-m", message);
}

function indent(text: string): string {
  return text.replace(/^/gm, "      ");
}

function sleep(seconds: number): void {
  Atomics.wait(new Int32Array(new SharedArrayBuffer(4)), 0, 0, seconds * 1000);
}

/** Runs the GitHub CLI. */
function gh(...args: string[]): string {
  try {
    return execFileSync("gh", args, { encoding: "utf8", stdio: ["pipe", "pipe", "pipe"] });
  } catch (error) {
    const stderr = (error as { stderr?: string }).stderr ?? "";
    throw new Error(`gh ${args.slice(0, 3).join(" ")} failed: ${stderr.trim() || String(error)}`, {
      cause: error,
    });
  }
}

function ghJson<T>(...args: string[]): T {
  return JSON.parse(gh(...args)) as T;
}

function fail(message: string): never {
  console.error(message);
  process.exit(1);
}
