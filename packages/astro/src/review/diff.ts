// Running `ascribe diff` for the integration's build: its changes as JSON, and
// its agent prompts.
import { execFile } from "node:child_process";
import type { DiffReport } from "../shapes.js";
import type { BaseInfo, PageDiff } from "./protocol.js";

/** One build's changes against a base. */
export interface DiffResult {
  base: BaseInfo;
  /** The build's changed pages: the first build compared, a versioned build's current line. */
  pages: PageDiff[];
  /**
   * Each build compared, in order, with its changed pages, when more than one
   * was: a versioned build's lines, the current one first.
   */
  lines?: { build: string; pages: PageDiff[] }[];
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
  /** The build, or a versioned build's lines, the current one first. */
  build: string | readonly string[];
  cwd: string;
  base: string | undefined;
}): Promise<DiffResult> {
  const args = [
    "diff",
    "--format",
    "json",
    ...buildArgs(options.build),
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

/** `--build` for each build. */
function buildArgs(build: string | readonly string[]): string[] {
  return (typeof build === "string" ? [build] : build).flatMap((b) => ["--build", b]);
}

/** The build's part of `ascribe diff --format json`'s output, or each of the builds' parts. */
export function parseDiff(json: string, build: string | readonly string[]): DiffResult {
  // Checked as far as telling a diff report from other JSON.
  const report = JSON.parse(json) as Partial<DiffReport>;
  if (report.base === undefined || !Array.isArray(report.builds)) {
    throw new Error("not a diff report");
  }
  const builds = typeof build === "string" ? [build] : build;
  const reported = report.builds;
  const pagesOf = (name: string) => reported.find((b) => b.build === name)?.pages ?? [];
  const pages = pagesOf(builds[0] ?? "");
  const errors = typeof report.working_tree_errors === "number" ? report.working_tree_errors : 0;
  if (builds.length < 2) return { base: report.base, pages, errors };
  const lines = builds.map((name) => ({ build: name, pages: pagesOf(name) }));
  return { base: report.base, pages, errors, lines };
}

/**
 * `ascribe diff --format prompt`'s prompt about a page, or a fragment's reach,
 * at a content path; `undefined` when it didn't change. Rejects with
 * `ascribe diff`'s reason as the message.
 */
export function runPrompt(options: {
  binary: string;
  configPath: string;
  /** The build, or a versioned build's lines: the prompt is about the first the page changed in. */
  build: string | readonly string[];
  cwd: string;
  base: string | undefined;
  path: string;
}): Promise<string | undefined> {
  const args = [
    "diff",
    "--format",
    "prompt",
    ...buildArgs(options.build),
    "--config",
    options.configPath,
    "--color",
    "never",
  ];
  if (options.base !== undefined) args.push("--base", options.base);
  // After `--`: a content path that starts with `-` is still a path.
  args.push("--", options.path);
  return new Promise((resolve, reject) => {
    execFile(
      options.binary,
      args,
      { cwd: options.cwd, maxBuffer: 16 * 1024 * 1024, windowsHide: true },
      (error, stdout, stderr) => {
        if (error !== null) {
          const reason = stderr.trim();
          reject(new Error(reason === "" ? error.message : reason, { cause: error }));
          return;
        }
        resolve(stdout === "" ? undefined : stdout);
      },
    );
  });
}
