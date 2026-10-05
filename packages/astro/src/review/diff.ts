// Running `ascribe diff --format json` for the integration's build.
import { execFile } from "node:child_process";
import type { DiffBase, DiffPage } from "./protocol.js";

/** One build's changes against a base. */
export interface DiffResult {
  base: DiffBase;
  pages: DiffPage[];
  /** How many errors `ascribe check` finds in the working tree for the build. */
  errors: number;
}

/**
 * Compares the build's pages with `base` (the default branch when it's
 * `undefined`). Rejects with `ascribe diff`'s reason as the message.
 */
export function runDiff(options: {
  binary: string;
  configPath: string;
  build: string;
  cwd: string;
  base: string | undefined;
}): Promise<DiffResult> {
  const args = [
    "diff",
    "--format",
    "json",
    "--build",
    options.build,
    "--config",
    options.configPath,
    "--color",
    "never",
  ];
  if (options.base !== undefined) args.push("--base", options.base);
  return new Promise((resolve, reject) => {
    execFile(
      options.binary,
      args,
      { cwd: options.cwd, maxBuffer: 256 * 1024 * 1024, windowsHide: true },
      (error, stdout, stderr) => {
        if (error !== null) {
          const reason = stderr.trim();
          reject(new Error(reason === "" ? error.message : reason, { cause: error }));
          return;
        }
        try {
          resolve(parseDiff(stdout, options.build));
        } catch (cause) {
          reject(new Error("`ascribe diff` wrote output that isn't its JSON.", { cause }));
        }
      },
    );
  });
}

/** The build's part of `ascribe diff --format json`'s output. */
export function parseDiff(json: string, build: string): DiffResult {
  const report = JSON.parse(json) as {
    base?: DiffBase;
    working_tree_errors?: number;
    builds?: { build: string; pages: DiffPage[] }[];
  };
  if (report.base === undefined || !Array.isArray(report.builds)) {
    throw new Error("not a diff report");
  }
  const pages = report.builds.find((b) => b.build === build)?.pages ?? [];
  const errors = typeof report.working_tree_errors === "number" ? report.working_tree_errors : 0;
  return { base: report.base, pages, errors };
}
