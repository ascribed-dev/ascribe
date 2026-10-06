// Whether the canary on npm (`next`) has what a commit ships, for the checks
// that run Ascribe from npm instead of from the checkout: "Site from npm",
// Drift, and Review.
//
//   node scripts/release/canary.ts paths
//   node scripts/release/canary.ts failed [--canary <commit>]
//
// paths: the files that ship, one per line. canary.yml publishes a canary only
// when one of them has changed since the last.
//
// failed: run when a check fails. When the canary lacks what the checkout
// (HEAD) ships, the failure may only mean "wait for a canary": it says so as a
// warning and in the job's summary, with a link to start one, and exits 0, so
// the check doesn't fail. The run against a canary that has it is the real
// check. Otherwise the failure is real: it says why, and exits 1. The canary
// is `next`'s `gitHead` on npm, or --canary's commit.
//
// The canary has what HEAD ships when HEAD is its commit or an ancestor of it,
// or when nothing that ships differs between the two (a canary publishes
// nothing when nothing that ships has changed). A canary that can't be
// compared (none on npm, or a commit this clone doesn't have) never excuses a
// failure. `next` only moves forward, so reading it after the check ran can't
// excuse one either: a newer canary has more, not less.
import { execFileSync, spawnSync } from "node:child_process";
import { appendFileSync } from "node:fs";
import process from "node:process";
import { parseArgs } from "node:util";

export const shipping = [
  "crates",
  "packages",
  "scripts/release",
  "Cargo.toml",
  "Cargo.lock",
  "pnpm-lock.yaml",
];

export type Verdict =
  | { waiting: true; canary: string; commit: string }
  | { waiting: false; canary: string | undefined; commit: string; why: string };

/** Whether the canary `canary` lacks what `HEAD` ships, in the clone at `cwd`. */
export function verdict(cwd: string, canary: string | undefined): Verdict {
  const commit = git(cwd, "rev-parse", "HEAD").stdout.trim();
  if (canary === undefined || !/^[0-9a-f]{40}$/.test(canary)) {
    return { waiting: false, canary: undefined, commit, why: "There's no canary on npm to compare with." };
  }
  if (git(cwd, "cat-file", "-e", `${canary}^{commit}`).status !== 0) {
    return {
      waiting: false,
      canary,
      commit,
      why: `The canary's commit, ${canary}, isn't in this clone, so it can't be compared.`,
    };
  }
  if (git(cwd, "merge-base", "--is-ancestor", commit, canary).status === 0) {
    return { waiting: false, canary, commit, why: `The canary, ${canary}, includes this commit.` };
  }
  if (git(cwd, "diff", "--quiet", canary, commit, "--", ...shipping).status === 0) {
    return {
      waiting: false,
      canary,
      commit,
      why: `Nothing that ships differs between the canary, ${canary}, and this commit.`,
    };
  }
  return { waiting: true, canary, commit };
}

function git(cwd: string, ...args: string[]): { status: number | null; stdout: string } {
  const result = spawnSync("git", args, { cwd, encoding: "utf8", stdio: ["ignore", "pipe", "ignore"] });
  return { status: result.status, stdout: result.stdout };
}

function npmCanary(): string | undefined {
  try {
    return execFileSync("npm", ["view", "@ascribed/cli@next", "gitHead"], {
      encoding: "utf8",
      stdio: ["ignore", "pipe", "ignore"],
    }).trim();
  } catch {
    return undefined;
  }
}

function failed(canary: string | undefined): never {
  const result = verdict(process.cwd(), canary ?? npmCanary());
  const short = result.commit.slice(0, 12);
  if (!result.waiting) {
    console.log(`::error::This failure is real. ${result.why}`);
    process.exit(1);
  }
  const repository = process.env["GITHUB_REPOSITORY"] ?? "ascribed-dev/ascribe";
  const server = process.env["GITHUB_SERVER_URL"] ?? "https://github.com";
  const start = `${server}/${repository}/actions/workflows/canary.yml`;
  console.log(
    `::warning::Waiting for a canary that includes ${short}. The canary on npm, ${result.canary.slice(0, 12)}, doesn't have what this commit ships, so this failure may only mean "wait"; the check after a canary that has it is the real one.`,
  );
  const summary = process.env["GITHUB_STEP_SUMMARY"];
  if (summary !== undefined && summary !== "") {
    appendFileSync(
      summary,
      [
        "",
        `### Waiting for a canary that includes ${short}`,
        "",
        `This check runs Ascribe from npm, and the canary there (\`next\`, from ${result.canary.slice(0, 12)}) doesn't have what this commit ships yet. It failed, but that may only mean "wait", so it isn't marked failed. ${
          process.env["GITHUB_EVENT_NAME"] === "pull_request"
            ? "Re-run it once a canary has what this pull request ships (for its own changes to Ascribe, that's after it merges): that run is the real check."
            : "The run after a canary that includes this commit is the real check, and the next canary starts it on its own."
        }`,
        "",
        `To not wait a night, [start a canary](${start}) (Run workflow, from \`main\`).`,
        "",
      ].join("\n"),
    );
  }
  process.exit(0);
}

if (import.meta.main) {
  const { values: options, positionals } = parseArgs({
    allowPositionals: true,
    options: { canary: { type: "string" } },
  });
  if (positionals[0] === "paths") console.log(shipping.join("\n"));
  else if (positionals[0] === "failed") failed(options.canary);
  else {
    console.error("usage: canary.ts paths | failed [--canary <commit>]");
    process.exit(1);
  }
}
