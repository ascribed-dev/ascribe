// Which files git ignores, asked of git itself: VS Code's `findFiles` doesn't
// read ignore files. No VS Code in this module.
import { execFile } from "node:child_process";
import * as path from "node:path";

/**
 * The files among `files`, all inside `folder`, that the repository around
 * `folder` ignores, as given. One `git check-ignore --stdin` for them all,
 * so a file git tracks counts as not ignored, whatever the rules say. None
 * when git isn't installed, `folder` isn't in a repository, or git fails.
 */
export function gitIgnored(folder: string, files: readonly string[]): Promise<Set<string>> {
  if (files.length === 0) return Promise.resolve(new Set());
  // Relative to the folder, as git reads them from its working directory,
  // and with forward slashes, which git takes on every platform.
  const relative = new Map(
    files.map((file) => [path.relative(folder, file).split(path.sep).join("/"), file]),
  );
  return new Promise((resolve) => {
    const child = execFile(
      "git",
      ["check-ignore", "--stdin", "-z"],
      { cwd: folder, timeout: 15_000, windowsHide: true, encoding: "utf8" },
      (error, stdout) => {
        // Status 1 is none ignored; anything else that fails is no answer.
        if (error && error.code !== 1) {
          resolve(new Set());
          return;
        }
        const ignored = stdout.split("\0").flatMap((line) => {
          const file = relative.get(line);
          return file === undefined ? [] : [file];
        });
        resolve(new Set(ignored));
      },
    );
    // git may exit before reading it all, as when the folder isn't a repository.
    child.stdin?.on("error", () => undefined);
    child.stdin?.end([...relative.keys()].map((file) => `${file}\0`).join(""));
  });
}
