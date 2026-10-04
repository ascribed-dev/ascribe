// Running `git` with a fixed argument list, never through a shell.
import { execFile } from "node:child_process";
import { ReviewError } from "./errors.js";

/** Runs `git` in `cwd` and resolves with its standard output. */
export function git(cwd: string, args: readonly string[]): Promise<string> {
  return new Promise((resolve, reject) => {
    execFile(
      "git",
      ["-c", "core.quotepath=off", ...args],
      { cwd, maxBuffer: 256 * 1024 * 1024, encoding: "utf8" },
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
