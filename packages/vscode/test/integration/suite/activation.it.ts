import * as assert from "node:assert/strict";
import * as vscode from "vscode";
import type { TesseraApi } from "../../../src/extension.js";
import { EXTENSION_ID, sleep, uriOf } from "./helpers.js";

describe("in a workspace without ascribe.toml", () => {
  it("doesn't activate the extension", async () => {
    // Opening a markdown file must not activate it either: only `ascribe.toml` does.
    await vscode.window.showTextDocument(
      await vscode.workspace.openTextDocument(uriOf("readme.md")),
    );
    await sleep(3000);
    const extension = vscode.extensions.getExtension(EXTENSION_ID);
    assert.ok(extension, "the extension isn't installed");
    assert.equal(extension.isActive, false);
  });

  it("starts no server when a command activates it anyway", async () => {
    // Contributed commands add their own activation events (VS Code 1.74+).
    await vscode.commands.executeCommand("ascribe.showOutput");
    const extension = vscode.extensions.getExtension<TesseraApi>(EXTENSION_ID);
    assert.ok(extension?.isActive, "the command should have activated the extension");
    await vscode.commands.executeCommand("ascribe.restartServer");
    const api = extension.exports;
    await api.whenSettled();
    assert.equal(api.state(), "stopped");
    assert.equal(api.binary(), undefined);
  });
});
