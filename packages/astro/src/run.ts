// Running `ascribe build` for the configured build: its site output, and
// its JSON output while review is on.
import { execFile } from "node:child_process";

/** What a successful build printed. */
export interface BuildResult {
  /** The checks' summary line, after each error in full when there are any. */
  diagnostics: string;
  /** The `built <build>: …` lines, one per build. */
  summary: string;
}

/**
 * The checks' summary line, the report's last: `checked 14 files in build \`site\`: no problems`,
 * or the counts, with the command that lists the warnings and advice when there are any:
 * `checked 14 files in build \`site\`: 0 errors, 1 warning, 3 advice: ascribe check --build site
 * for each`.
 */
const SUMMARY =
  /^checked [^:\n]*: (?:no problems|(\d+) errors?, (\d+) warnings?, \d+ advice)(?:, \d+ acknowledged)?(?:: .* for each)?$/;

/**
 * Whether a build's report is worth a warning: anything but a summary line
 * that counts no errors and no warnings, which is logged at info level,
 * advice and all.
 */
export function hasWarnings(diagnostics: string): boolean {
  if (diagnostics === "") return false;
  const summary = SUMMARY.exec(diagnostics.trimEnd().split("\n").at(-1) ?? "");
  if (summary === null) return true;
  return (summary[1] ?? "0") !== "0" || (summary[2] ?? "0") !== "0";
}

/**
 * Whether an Astro command builds with source anchors, given the integration's
 * `anchors` option: `"dev"` means in `astro dev` only, so `astro build` has
 * them only when it is `true`. Review in the site preview (`review`) needs
 * them in `astro dev`.
 */
export function anchorsFor(
  anchors: boolean | "dev" | undefined,
  command: string,
  review = false,
): boolean {
  return anchors === true || (command === "dev" && (anchors === "dev" || review));
}

/** Whether an Astro command has review in the site preview, given the `review` option (an entry's name is on). */
export function reviewFor(review: boolean | string | undefined, command: string): boolean {
  return command === "dev" && review !== false;
}

/**
 * Builds one build's outputs, or several builds', the site output by default. Rejects, with the compiler's report as the
 * message, if the build has errors: the Astro build fails with them.
 */
export function runBuild(options: {
  binary: string;
  configPath: string;
  /** The build, or several, each a `--build`. */
  build: string | readonly string[];
  cwd: string;
  /** Write source anchors (`--anchors`). */
  anchors?: boolean;
  /** The outputs to write. Default: `["site"]`. */
  outputs?: readonly ("site" | "plain" | "json")[];
}): Promise<BuildResult> {
  const builds = typeof options.build === "string" ? [options.build] : options.build;
  const args = [
    "build",
    "--emit",
    (options.outputs ?? ["site"]).join(","),
    ...builds.flatMap((build) => ["--build", build]),
    "--config",
    options.configPath,
    "--color",
    "never",
  ];
  if (options.anchors === true) args.push("--anchors");
  return new Promise((resolve, reject) => {
    execFile(
      options.binary,
      args,
      { cwd: options.cwd, maxBuffer: 64 * 1024 * 1024 },
      (error, stdout, stderr) => {
        if (error !== null) {
          const report = `${stdout}${stderr}`.trim();
          reject(
            new Error(
              `\`ascribe build ${builds.map((build) => `--build ${build}`).join(" ")}\` failed${report === "" ? `: ${error.message}` : `:\n${report}`}`,
              { cause: error },
            ),
          );
          return;
        }
        resolve({ diagnostics: stdout.trim(), summary: stderr.trim() });
      },
    );
  });
}
