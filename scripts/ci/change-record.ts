// What a pull request must record besides its code, as far as a script can
// tell from the files it changes (#991). CI runs it on every pull request
// (ci.yml's `record` job):
//
//   node scripts/ci/change-record.ts --base <rev> [--description <file>]
//
// - A contract changed (SPEC.md, docs/content/contracts/,
//   packages/elements/CONTRACT.md) with no decision added or changed in
//   project-docs/decisions/: an error, and it exits 1. A contract changes
//   only by a recorded decision.
// - The source of a crate or package changed (crates/*/src/,
//   packages/*/src/) with no changelog entry added in changes/: a warning,
//   unless the description has a line starting `Not user-visible:`. Plenty
//   of source changes are invisible to a user, so it's for the reviewer to
//   weigh, not a failure.
//
// It prints each as a GitHub annotation, and writes them to the run's summary
// when GITHUB_STEP_SUMMARY is set. Whether a change is a behavior change
// (a conformance case) or a clean-up (the `optimization` label) it can't
// tell; the review skill has the reviewer look for those.
//
// This file imports nothing but Node's own modules, so the job runs it with
// no install.
import { execFileSync } from "node:child_process";
import { appendFileSync, readFileSync } from "node:fs";
import { parseArgs } from "node:util";

/** A file the change touches, as `git diff --name-status --no-renames` says it. */
export interface Changed {
  /** `A` added, `M` modified, `D` deleted, `T` its type changed. */
  status: string;
  path: string;
}

/**
 * The line in a pull request's description that says its change to the
 * source is one no user would notice, followed by why. CONTRIBUTING.md
 * gives the same wording, and a test holds it to this.
 */
export const NOT_USER_VISIBLE = "Not user-visible:";

/** The contracts, which change only with a decision. */
const CONTRACT = /^(SPEC\.md|docs\/content\/contracts\/.+|packages\/elements\/CONTRACT\.md)$/;

/** A file of a crate's or a package's source. */
const SOURCE = /^(crates|packages)\/[^/]+\/src\//;

/** A decision: a file of project-docs/decisions/ other than its README. */
const DECISION = /^project-docs\/decisions\/(?!README\.md$)[^/]+$/;

/** A changelog entry: a file in an area's folder of changes/. */
const ENTRY = /^changes\/[^/]+\/[^/]+\.md$/;

/** What `git diff --name-status --no-renames` printed, as files. */
export function parseNameStatus(text: string): Changed[] {
  return text
    .split("\n")
    .filter((line) => line.trim() !== "")
    .map((line) => {
      const [status = "", path = ""] = line.split("\t");
      return { status: status.charAt(0), path };
    });
}

/** Whether the description says the change isn't one a user would notice. */
export function saysNotUserVisible(description: string): boolean {
  return description
    .split(/\r?\n/)
    .some(
      (line) =>
        line.startsWith(NOT_USER_VISIBLE) && line.slice(NOT_USER_VISIBLE.length).trim() !== "",
    );
}

/** What the change is missing: errors fail the run, warnings are for the reviewer. */
export function review(changed: readonly Changed[], description: string) {
  const errors: string[] = [];
  const warnings: string[] = [];

  const contracts = changed.filter((f) => CONTRACT.test(f.path)).map((f) => f.path);
  const decided = changed.some((f) => f.status !== "D" && DECISION.test(f.path));
  if (contracts.length > 0 && !decided) {
    errors.push(
      `A contract changed (${contracts.join(", ")}) and no decision in project-docs/decisions/ did. ` +
        "A contract changes only by a decision recorded with the change: add its file " +
        "(project-docs/decisions/README.md says how), or undo the change to the contract.",
    );
  }

  const sources = changed.filter((f) => SOURCE.test(f.path));
  const entry = changed.some((f) => f.status === "A" && ENTRY.test(f.path));
  if (sources.length > 0 && !entry && !saysNotUserVisible(description)) {
    warnings.push(
      `The source changed (${sources.length} ${sources.length === 1 ? "file" : "files"} in crates/*/src/ or packages/*/src/) ` +
        "and no changelog entry was added in changes/. A change a user would notice adds one, and " +
        "updates its page under docs/content/; one they wouldn't says so in the description, in a " +
        `line starting \`${NOT_USER_VISIBLE}\` and saying why.`,
    );
  }

  return { errors, warnings };
}

function main() {
  const { values } = parseArgs({
    options: { base: { type: "string" }, description: { type: "string" } },
  });
  if (values.base === undefined) {
    console.error("usage: node scripts/ci/change-record.ts --base <rev> [--description <file>]");
    process.exit(2);
  }
  const changed = parseNameStatus(
    execFileSync("git", ["diff", "--name-status", "--no-renames", values.base, "HEAD"], {
      encoding: "utf8",
    }),
  );
  const description =
    values.description === undefined ? "" : readFileSync(values.description, "utf8");
  const { errors, warnings } = review(changed, description);

  for (const error of errors) console.log(`::error::${error}`);
  for (const warning of warnings) console.log(`::warning::${warning}`);
  if (errors.length + warnings.length === 0) console.log("Nothing missing.");

  const summary = process.env.GITHUB_STEP_SUMMARY;
  if (summary) {
    const lines = [
      "### What the change records",
      "",
      ...errors.map((e) => `- **Error:** ${e}`),
      ...warnings.map((w) => `- **Warning:** ${w}`),
    ];
    if (errors.length + warnings.length === 0) lines.push("Nothing missing.");
    appendFileSync(summary, `${lines.join("\n")}\n`);
  }
  process.exit(errors.length > 0 ? 1 : 0);
}

if (import.meta.main) main();
