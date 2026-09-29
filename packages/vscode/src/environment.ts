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
        file,
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
export function usesShell(file: string): boolean {
  return process.platform === "win32" && /\.(cmd|bat)$/i.test(file);
}
