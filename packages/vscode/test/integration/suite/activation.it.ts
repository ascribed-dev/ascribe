import * as assert from "node:assert/strict";
import * as vscode from "vscode";
import { EXTENSION_ID, sleep, uriOf } from "./helpers.js";

describe("in a workspace without tessera.toml", () => {
  it("doesn't activate the extension", async () => {
    // Opening a markdown file must not activate it either: only `tessera.toml` does.
    await vscode.window.showTextDocument(
      await vscode.workspace.openTextDocument(uriOf("readme.md")),
    );
    await sleep(3000);
    const extension = vscode.extensions.getExtension(EXTENSION_ID);
    assert.ok(extension, "the extension isn't installed");
    assert.equal(extension.isActive, false);
  });
});
