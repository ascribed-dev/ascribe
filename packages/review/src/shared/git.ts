// Running `git` with a fixed argument list, never through a shell.
import { execFile } from "node:child_process";
import { ReviewError } from "./errors.js";

/**
 * The options every `git` Ascribe starts runs with, each as `-c <option>`, so
 * that reading a repository runs nothing it supplies: `ascribe_core::git`'s
 * `OPTIONS`, which says why each is there. `test/git.test.ts` compares the two.
 */
export const OPTIONS: readonly string[] = ["safe.bareRepository=explicit", "core.fsmonitor=false"];

/**
 * The environment variables cleared for every `git` Ascribe starts, each of
 * which would point it at another repository: `ascribe_core::git`'s `CLEARED`.
 */
export const CLEARED: readonly string[] = [
  "GIT_DIR",
  "GIT_WORK_TREE",
  "GIT_INDEX_FILE",
  "GIT_OBJECT_DIRECTORY",
  "GIT_ALTERNATE_OBJECT_DIRECTORIES",
  "GIT_NAMESPACE",
  "GIT_COMMON_DIR",
];

/** This process's environment without `CLEARED`. */
function environment(): NodeJS.ProcessEnv {
  return Object.fromEntries(
    Object.entries(process.env).filter(([name]) => !CLEARED.includes(name)),
  );
}

/** Runs `git` in `cwd` and resolves with its standard output. */
export function git(cwd: string, args: readonly string[]): Promise<string> {
  return new Promise((resolve, reject) => {
    execFile(
      "git",
      [...OPTIONS.flatMap((option) => ["-c", option]), "-c", "core.quotepath=off", ...args],
      { cwd, env: environment(), maxBuffer: 256 * 1024 * 1024, encoding: "utf8" },
      (error, stdout, stderr) => {
        if (error === null) {
          resolve(stdout);
          return;
        }
        const missing = (error as NodeJS.ErrnoException).code === "ENOENT";
        const detail = stderr.trim() || error.message;
        reject(
          new ReviewError(
            "git",
            missing ? "`git` isn't on the path." : `\`git ${args.join(" ")}\` failed: ${detail}`,
            { cause: error },
          ),
        );
      },
    );
  });
}

/** Like `git`, but resolves with `undefined` instead of rejecting when git exits non-zero. */
export async function gitMaybe(cwd: string, args: readonly string[]): Promise<string | undefined> {
  try {
    return await git(cwd, args);
  } catch (error) {
    if (error instanceof ReviewError && error.message.startsWith("`git` isn't")) throw error;
    return undefined;
  }
}
