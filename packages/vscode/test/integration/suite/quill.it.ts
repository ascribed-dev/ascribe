import * as assert from "node:assert/strict";
import { readFileSync, rmSync, writeFileSync } from "node:fs";
import * as vscode from "vscode";
import { activated, diagnosticsOf, uriOf } from "./helpers.js";

// The real `ascribe lsp` (phase 15) on a copy of examples/quill with a broken
// page, `docs/broken.md`, added (its unknown attribute key is a §8.2 error).
describe("with the real language server on examples/quill", () => {
  const broken = uriOf("docs", "broken.md");
  const original = () => readFileSync(broken.fsPath, "utf8");

  it("starts `ascribe lsp` from the ascribe.path setting", async () => {
    const api = await activated();
    await api.whenSettled();
    assert.equal(api.state(), "running");
    assert.equal(api.binary()?.source, "setting");
  });

  it("delivers diagnostics for a page that isn't open", async () => {
    const diagnostics = await diagnosticsOf(broken, (all) => all.length > 0);
    assert.ok(diagnostics.every((diagnostic) => diagnostic.severity !== undefined));
    assert.ok(diagnostics.some((diagnostic) => diagnostic.range.start.line === 7));
  });

  it("reports nothing for the pages of the example that are correct", async () => {
    await diagnosticsOf(broken, (all) => all.length > 0);
    for (const page of ["quickstart.md", "install-agent.md", "keys.md"]) {
      assert.deepEqual(vscode.languages.getDiagnostics(uriOf("docs", page)), [], page);
    }
  });

  it("updates diagnostics after the file changes on disk", async () => {
    const text = original();
    try {
      writeFileSync(broken.fsPath, text.replace("{colour=red}", "{type=tip}"));
      await diagnosticsOf(broken, (all) => all.length === 0);

      writeFileSync(broken.fsPath, text.replace("{colour=red}", "{colour=blue, size=big}"));
      const diagnostics = await diagnosticsOf(broken, (all) => all.length >= 2);
      assert.ok(diagnostics.length >= 2);
    } finally {
      writeFileSync(broken.fsPath, text);
    }
  });

  it("updates diagnostics for an open document as it's edited", async () => {
    const editor = await vscode.window.showTextDocument(
      await vscode.workspace.openTextDocument(broken),
    );
    await diagnosticsOf(broken, (all) => all.length > 0);
    const line = editor.document.lineAt(7);
    await editor.edit((edit) => edit.replace(line.range, "@note {type=tip}: Now it is fine."));
    await diagnosticsOf(broken, (all) => all.length === 0);
  });

  it("formats through tessera-fmt on save when enabled", async () => {
    const file = uriOf("docs", "format-on-save.md");
    const input = "@note{type = tip}: Save safely.\n";
    writeFileSync(file.fsPath, "placeholder\n");
    const config = vscode.workspace.getConfiguration("ascribe");
    const previous = config.get("formatOnSave", false);
    await config.update("formatOnSave", true, vscode.ConfigurationTarget.Workspace);
    try {
      const document = await vscode.workspace.openTextDocument(file);
      const editor = await vscode.window.showTextDocument(document);
      await editor.edit((edit) =>
        edit.replace(
          new vscode.Range(document.positionAt(0), document.positionAt(document.getText().length)),
          input,
        ),
      );
      await document.save();
      assert.equal(readFileSync(file.fsPath, "utf8"), "@note {type=tip}: Save safely.\n");
    } finally {
      await config.update("formatOnSave", previous, vscode.ConfigurationTarget.Workspace);
      rmSync(file.fsPath, { force: true });
    }
  });

  it("updates file references before a workspace rename", async () => {
    const target = uriOf("docs", "keys.md");
    const moved = uriOf("docs", "guides", "keys.md");
    const linking = uriOf("docs", "install-agent.md");
    const originalTarget = readFileSync(target.fsPath, "utf8");
    const originalLinking = readFileSync(linking.fsPath, "utf8");
    try {
      await vscode.workspace.openTextDocument(target);
      await vscode.workspace.openTextDocument(linking);
      const edit = new vscode.WorkspaceEdit();
      edit.renameFile(target, moved);
      assert.equal(await vscode.workspace.applyEdit(edit), true);
      assert.ok(
        (await vscode.workspace.openTextDocument(linking))
          .getText()
          .includes("See [](guides/keys.md#rotate-keys)."),
      );
      const reverse = new vscode.WorkspaceEdit();
      reverse.renameFile(moved, target);
      assert.equal(await vscode.workspace.applyEdit(reverse), true);
      assert.equal((await vscode.workspace.openTextDocument(linking)).getText(), originalLinking);
    } finally {
      rmSync(moved.fsPath, { force: true });
      writeFileSync(target.fsPath, originalTarget);
      writeFileSync(linking.fsPath, originalLinking);
    }
  });
});
