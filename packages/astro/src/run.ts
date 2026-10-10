// Running `ascribe build` for the configured build: its site output, and
// its JSON output while review is on.
import { execFile } from "node:child_process";

/** What a successful build printed. */
export interface BuildResult {
  /** The checks' report, which is more than a summary line when there is anything to report. */
  diagnostics: string;
  /** The `built <build>/site: …` lines. */
  summary: string;
}

/**
 * A check's summary line, the report's last: `checked 14 files: 0 errors, 0 warnings`,
 * with `, 3 advice` after when there is advice.
 */
const SUMMARY = /^checked [^:\n]*: (\d+) errors?, (\d+) warnings?(?:, \d+ advice)?$/;

/**
 * Whether a build's report is worth a warning: anything but a report whose
 * summary line counts no errors and no warnings, which is logged at info
 * level, advice and all.
 */
export function hasWarnings(diagnostics: string): boolean {
  if (diagnostics === "") return false;
  const summary = SUMMARY.exec(diagnostics.trimEnd().split("\n").at(-1) ?? "");
  return summary === null || summary[1] !== "0" || summary[2] !== "0";
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

/** Whether an Astro command has review in the site preview, given the `review` option. */
export function reviewFor(review: boolean | undefined, command: string): boolean {
  return command === "dev" && review !== false;
}

/**
 * Builds one build's outputs, the site output by default. Rejects, with the compiler's report as the
 * message, if the build has errors: the Astro build fails with them.
 */
export function runBuild(options: {
  binary: string;
  configPath: string;
  build: string;
  cwd: string;
  /** Write source anchors (`--anchors`). */
  anchors?: boolean;
  /** The outputs to write. Default: `["site"]`. */
  outputs?: readonly ("site" | "plain" | "json")[];
}): Promise<BuildResult> {
  const args = [
    "build",
    "--emit",
    (options.outputs ?? ["site"]).join(","),
    "--build",
    options.build,
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
              `\`ascribe build --build ${options.build}\` failed${report === "" ? `: ${error.message}` : `:\n${report}`}`,
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
