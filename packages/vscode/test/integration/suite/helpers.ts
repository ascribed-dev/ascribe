import * as assert from "node:assert/strict";
import { rmSync } from "node:fs";
import * as path from "node:path";
import * as vscode from "vscode";
import type { AscribeApi } from "../../../src/extension.js";

export const workspace = (): string => {
  const folder = vscode.workspace.workspaceFolders?.[0];
  assert.ok(folder, "the test workspace isn't open");
  return folder.uri.fsPath;
};

export const uriOf = (...parts: string[]): vscode.Uri =>
  vscode.Uri.file(path.join(workspace(), ...parts));

export const sleep = (ms: number) => new Promise((resolve) => setTimeout(resolve, ms));

/**
 * Deletes a folder a test made in the workspace. On Windows a process running
 * in the folder keeps it from being deleted: the Project view's check of a
 * project runs in the project's folder, and can still be running when the
 * test is done. Node retries on `EPERM` and `EBUSY`, for at least 2 seconds in
 * all, long enough for a check to finish; a server that's still running holds
 * the folder for longer than that.
 */
export const removeFolder = (folder: string): void =>
  rmSync(folder, { recursive: true, force: true, maxRetries: 10, retryDelay: 200 });

/** Polls until `check` returns a value that isn't `undefined` or `false`, or fails after `timeout` ms. */
export async function waitFor<T>(
  description: string,
  check: () => T | undefined | false | Promise<T | undefined | false>,
  timeout = 30_000,
): Promise<T> {
  const deadline = Date.now() + timeout;
  for (;;) {
    const value = await check();
    if (value !== undefined && value !== false) return value;
    if (Date.now() > deadline) assert.fail(`Timed out waiting for ${description}`);
    await sleep(100);
  }
}

export const EXTENSION_ID = "Ascribe.ascribe-vscode";

/** The extension, activated (it activates on `ascribe.toml`; a command also does). */
export async function activated(): Promise<AscribeApi> {
  const extension = vscode.extensions.getExtension<AscribeApi>(EXTENSION_ID);
  assert.ok(extension, `extension ${EXTENSION_ID} isn't installed`);
  return extension.isActive ? extension.exports : extension.activate();
}

/** Waits for a diagnostic list for `uri` that satisfies `predicate`. */
export function diagnosticsOf(
  uri: vscode.Uri,
  predicate: (diagnostics: vscode.Diagnostic[]) => boolean,
  timeout?: number,
): Promise<vscode.Diagnostic[]> {
  return waitFor(
    `diagnostics for ${path.basename(uri.fsPath)}`,
    () => {
      const diagnostics = vscode.languages.getDiagnostics(uri);
      return predicate(diagnostics) ? diagnostics : undefined;
    },
    timeout,
  );
}
