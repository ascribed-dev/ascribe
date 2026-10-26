// Every check CI runs on a pull request, here, in one command: `pnpm check:all`.
//
// The steps are the ones rust.yml and js.yml run, job by job, each named as
// its workflow names it, so scripts/check.test.ts fails when the two drift
// apart. A step that can't run here, or needn't, says why, and the jobs this
// leaves to CI whole are in `ciOnly`. It runs the jobs one after another, in
// an order that works from a fresh checkout, and stops at the first failure.
//
// cargo-deny and cargo-machete aren't part of a Rust toolchain. Without them,
// it says how to install the versions rust.yml pins, before running anything.
import { spawnSync } from "node:child_process";
import { readFileSync } from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";
import process from "node:process";
import { fileURLToPath } from "node:url";

const root = fileURLToPath(new URL("..", import.meta.url));
const exe = process.platform === "win32" ? "ascribe.exe" : "ascribe";

export interface Step {
  workflow: "rust.yml" | "js.yml";
  job: string;
  // The step as the workflow names it: its `name:`, or its `run:` line when it
  // has no name. A step that only `uses:` an action isn't listed.
  ci: string;
  // What runs here, when it isn't `ci` itself: `ascribe` is this checkout's
  // debug build, which the Rust job's `cargo build` makes.
  run?: string[];
  cwd?: string;
  // Why it doesn't run here.
  skip?: string;
  // A failure is reported and doesn't stop the run, as CI's
  // `continue-on-error` doesn't fail the job.
  warn?: string;
}

// With no arguments, rustup installs the toolchain and components
// rust-toolchain.toml pins; it runs once.
const rustup = "rustup toolchain install";

export const steps: Step[] = [
  { workflow: "rust.yml", job: "fmt", ci: rustup },
  { workflow: "rust.yml", job: "fmt", ci: "cargo fmt --all --check" },

  { workflow: "rust.yml", job: "dependencies", ci: rustup },
  {
    workflow: "rust.yml",
    job: "dependencies",
    ci: "cargo deny --locked check bans licenses sources",
  },
  {
    workflow: "rust.yml",
    job: "dependencies",
    ci: "cargo deny --locked check advisories",
    warn: "a new advisory comes from outside, not from this change: file an issue for it (CONTRIBUTING.md)",
  },
  { workflow: "rust.yml", job: "dependencies", ci: "cargo machete" },

  {
    workflow: "js.yml",
    job: "check",
    ci: 'git config --global --add safe.directory "$GITHUB_WORKSPACE"',
    skip: "only Actions' container needs it, where the checkout and the job have different owners",
  },
  { workflow: "js.yml", job: "check", ci: "pnpm install --frozen-lockfile" },
  { workflow: "js.yml", job: "check", ci: "pnpm format:check" },
  { workflow: "js.yml", job: "check", ci: "pnpm lint" },
  { workflow: "js.yml", job: "check", ci: "pnpm exec knip" },
  { workflow: "js.yml", job: "check", ci: "pnpm --filter @ascribed/cli build" },
  { workflow: "js.yml", job: "check", ci: "pnpm --filter @ascribed/elements build" },
  { workflow: "js.yml", job: "check", ci: "pnpm typecheck" },
  { workflow: "js.yml", job: "check", ci: "pnpm test" },
  { workflow: "js.yml", job: "check", ci: "pnpm --filter @ascribed/review build" },
  { workflow: "js.yml", job: "check", ci: "pnpm --filter @ascribed/astro build" },
  { workflow: "js.yml", job: "check", ci: "pnpm check:packages" },

  { workflow: "rust.yml", job: "check", ci: rustup },
  {
    workflow: "rust.yml",
    job: "check",
    ci: "cargo clippy --workspace --all-targets --locked -- -D warnings",
  },
  { workflow: "rust.yml", job: "check", ci: "cargo build --workspace --locked" },
  {
    workflow: "rust.yml",
    job: "check",
    ci: "Check the repository's examples are observed",
    run: ["ascribe", "observe", "--check"],
  },
  {
    workflow: "rust.yml",
    job: "check",
    ci: "Archive examples/quill's 3.4 line",
    run: [
      "ascribe",
      "versions",
      "archive",
      "self-managed 3.4",
      "--to",
      path.join(tmpdir(), `ascribe-check-${process.pid}`, "quill-archive"),
      "--config",
      "examples/quill",
    ],
  },
  {
    workflow: "rust.yml",
    job: "check",
    ci: "Install Vale",
    skip: "outside CI, Vale's tests skip without one; set ASCRIBE_TEST_VALE to a Vale to run them",
  },
  {
    workflow: "rust.yml",
    job: "check",
    ci: "cargo test --workspace --locked -- --skip every_output_is_the_same_twice",
  },
  // Observing again needs the Node each observation names, so here only the
  // check that the committed observations match runs, which runs nothing.
  {
    workflow: "rust.yml",
    job: "check",
    ci: "Observe examples/tested with Node 22",
    run: ["ascribe", "observe", "--check", "--name", "node-22"],
    cwd: "examples/tested/code",
  },
  {
    workflow: "rust.yml",
    job: "check",
    ci: "Observe examples/tested",
    run: ["ascribe", "observe", "--check"],
    cwd: "examples/tested/code",
  },
  { workflow: "rust.yml", job: "check", ci: "node scripts/sources-fixture/setup.ts --local" },
  { workflow: "rust.yml", job: "check", ci: "pnpm install --frozen-lockfile" },
  { workflow: "rust.yml", job: "check", ci: "node scripts/templates/convert.ts --check" },

  { workflow: "rust.yml", job: "determinism", ci: rustup },
  {
    workflow: "rust.yml",
    job: "determinism",
    ci: "cargo test -p ascribe-cli --test determinism --locked",
  },
];

// The jobs left to CI, and why.
export const ciOnly: { workflow: Step["workflow"]; job: string; why: string }[] = [
  { workflow: "js.yml", job: "vscode", why: "it needs VS Code and a display" },
  {
    workflow: "js.yml",
    job: "vscode-platforms",
    why: "it's the VS Code tests on Windows and macOS",
  },
  {
    workflow: "js.yml",
    job: "astro",
    why: "it packs and installs a release build of each platform's binary",
  },
];

// The versions of cargo-deny and cargo-machete rust.yml installs.
export function pinnedTools(): Map<string, string> {
  const workflow = readFileSync(path.join(root, ".github/workflows/rust.yml"), "utf8");
  const tools = /^\s+tool: (.+)$/m.exec(workflow)?.[1] ?? "";
  return new Map(tools.split(",").map((tool) => tool.trim().split("@") as [string, string]));
}

function missingTools(): string[] {
  const missing: string[] = [];
  for (const [tool, version] of pinnedTools()) {
    const found = spawnSync("cargo", [tool.replace(/^cargo-/, ""), "--version"], {
      cwd: root,
      encoding: "utf8",
    });
    if (found.status !== 0) missing.push(`${tool}@${version}`);
    else if (!found.stdout.includes(version)) {
      process.stderr.write(
        `${tool} ${found.stdout.trim()} isn't the ${version} rust.yml pins; CI may disagree with it.\n`,
      );
    }
  }
  return missing;
}

function targetDir(): string {
  // Where cargo puts the build: `target/` unless CARGO_TARGET_DIR or
  // `build.target-dir` says otherwise.
  const metadata = spawnSync("cargo", ["metadata", "--format-version", "1", "--no-deps"], {
    cwd: root,
    encoding: "utf8",
    maxBuffer: 64 * 1024 * 1024,
  });
  if (metadata.status !== 0) {
    process.stderr.write(metadata.stderr);
    process.exit(metadata.status ?? 1);
  }
  return (JSON.parse(metadata.stdout) as { target_directory: string }).target_directory;
}

function main(): void {
  const missing = missingTools();
  if (missing.length > 0) {
    process.stderr.write(
      `CI's dependencies job needs ${missing.join(" and ")}, the versions rust.yml pins. Install them with:\n\n` +
        `  cargo install --locked ${missing.join(" ")}\n`,
    );
    process.exit(1);
  }
  const ascribe = path.join(targetDir(), "debug", exe);
  const ran = new Set<string>();
  const warnings: string[] = [];
  for (const step of steps) {
    const where = `${step.workflow} ${step.job}`;
    if (step.skip !== undefined) {
      process.stdout.write(`\n# skipped: ${step.ci} (${where}): ${step.skip}\n`);
      continue;
    }
    const [command, ...args] = (step.run ?? step.ci.split(" ")).map((word) =>
      word === "ascribe" ? ascribe : word,
    ) as [string, ...string[]];
    const line = [command, ...args].join(" ");
    const cwd = path.join(root, step.cwd ?? "");
    // Two jobs install the same packages; once is enough here.
    if (ran.has(`${cwd}\0${line}`)) continue;
    ran.add(`${cwd}\0${line}`);
    process.stdout.write(`\n$ ${line}${step.cwd ? `  (in ${step.cwd})` : ""}\n`);
    // pnpm is a .cmd script on Windows, which only a shell can start.
    const result = spawnSync(command, args, {
      cwd,
      stdio: "inherit",
      shell: process.platform === "win32",
    });
    if (result.status === 0) continue;
    if (step.warn !== undefined) {
      warnings.push(`${step.ci} (${where}) failed: ${step.warn}`);
      continue;
    }
    process.stderr.write(`\nFailed: ${step.ci}, as CI's ${where} job runs it.\n`);
    process.exit(result.status ?? 1);
  }
  for (const warning of warnings) process.stderr.write(`\nWarning: ${warning}\n`);
  process.stdout.write(
    `\nEvery check passed. CI also runs ${ciOnly.map((j) => `js.yml's ${j.job}`).join(", ")}.\n`,
  );
}

if (import.meta.main) main();
