import { execFile } from "node:child_process";
import { stat } from "node:fs/promises";
import type { BinaryEnvironment } from "./binary.js";

/** The real machine: the file system, child processes, and this platform. */
export const nodeEnvironment: BinaryEnvironment = {
  async fileExists(file) {
    try {
      return (await stat(file)).isFile();
    } catch {
      return false;
    }
  },
  runVersion(file) {
    return new Promise((resolve, reject) => {
      execFile(
        shellCommand(file),
        ["--version"],
        // A `.cmd` shim, which is what npm links on Windows, needs a shell.
        { timeout: 10_000, windowsHide: true, shell: usesShell(file) },
        (error, stdout) => (error ? reject(error) : resolve(stdout)),
      );
    });
  },
  platform: process.platform,
  arch: process.arch,
};

/** Whether a binary has to be started through a shell (a Windows `.cmd` shim). */
export function usesShell(file: string, platform: NodeJS.Platform = process.platform): boolean {
  return platform === "win32" && /\.(cmd|bat)$/i.test(file);
}

/**
 * The command to give a process API for a binary. With a shell, Node joins the
 * command and its arguments into one line without quoting, so a path with
 * spaces (`C:\Users\Jane Doe\...`) has to be quoted here.
 */
export function shellCommand(file: string, platform: NodeJS.Platform = process.platform): string {
  return usesShell(file, platform) ? `"${file}"` : file;
}
