// Running `ascribe build --emit site` for the configured build.
import { execFile } from "node:child_process";

/** What a successful build printed. */
export interface BuildResult {
  /** The checks' report, which is more than a summary line when there are warnings. */
  diagnostics: string;
  /** The `built <build>/site: …` lines. */
  summary: string;
}

/**
 * Whether an Astro command builds with source anchors, given the integration's
 * `anchors` option: `"dev"` means in `astro dev` only, so `astro build` has
 * them only when it is `true`.
 */
export function anchorsFor(anchors: boolean | "dev" | undefined, command: string): boolean {
  return anchors === true || (anchors === "dev" && command === "dev");
}

/**
 * Builds one build's site output. Rejects, with the compiler's report as the
 * message, if the build has errors: the Astro build fails with them.
 */
export function runBuild(options: {
  binary: string;
  configPath: string;
  build: string;
  cwd: string;
  /** Write source anchors (`--anchors`). */
  anchors?: boolean;
}): Promise<BuildResult> {
  const args = [
    "build",
    "--emit",
    "site",
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
