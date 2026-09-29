import * as assert from "node:assert/strict";
import { mkdirSync, rmSync, writeFileSync } from "node:fs";
import * as path from "node:path";
import * as vscode from "vscode";
import { activated, diagnosticsOf, uriOf, waitFor, workspace } from "./helpers.js";

const stubDiagnostics = (diagnostics: vscode.Diagnostic[]) =>
  diagnostics.filter((diagnostic) => diagnostic.source === "ascribe-stub");

describe("with ascribe.toml and the stub server", () => {
  it("activates, finds the binary named by ascribe.path, and starts the server", async () => {
    const api = await activated();
    await api.whenSettled();
    assert.equal(api.state(), "running");
    assert.equal(api.binary()?.source, "setting");
    assert.match(api.binary()?.path ?? "", /stub-server[\\/]ascribe$/);
  });

  it("shows diagnostics for a file that isn't open, from the project scan", async () => {
    const diagnostics = await diagnosticsOf(uriOf("docs", "todo.md"), (all) => all.length > 0);
    assert.equal(diagnostics[0]?.message, "stub: TODO found");
    assert.equal(diagnostics[0]?.range.start.line, 2);
  });

  it("shows diagnostics for an open document, and updates them as it's edited", async () => {
    const uri = uriOf("docs", "page.md");
    const document = await vscode.workspace.openTextDocument(uri);
    const editor = await vscode.window.showTextDocument(document);
    assert.equal(stubDiagnostics(vscode.languages.getDiagnostics(uri)).length, 0);

    await editor.edit((edit) => edit.insert(new vscode.Position(2, 0), "TODO: unsaved\n"));
    await diagnosticsOf(uri, (all) => stubDiagnostics(all).length === 1);

    await editor.edit((edit) => edit.delete(new vscode.Range(2, 0, 3, 0)));
    await diagnosticsOf(uri, (all) => stubDiagnostics(all).length === 0);
  });

  it("follows a file created, changed, and deleted on disk while it's not open", async () => {
    const file = path.join(workspace(), "docs", "on-disk.md");
    const uri = vscode.Uri.file(file);
    mkdirSync(path.dirname(file), { recursive: true });

    writeFileSync(file, "# On disk\n\nTODO one.\n");
    await diagnosticsOf(uri, (all) => stubDiagnostics(all).length === 1);

    writeFileSync(file, "# On disk\n\nTODO one.\n\nTODO two.\n");
    await diagnosticsOf(uri, (all) => stubDiagnostics(all).length === 2);

    writeFileSync(file, "# On disk\n\nAll done.\n");
    await diagnosticsOf(uri, (all) => stubDiagnostics(all).length === 0);

    writeFileSync(file, "# On disk\n\nTODO again.\n");
    await diagnosticsOf(uri, (all) => stubDiagnostics(all).length === 1);
    rmSync(file);
    await diagnosticsOf(uri, (all) => all.length === 0);
  });

  it("restarts the server on request", async () => {
    const api = await activated();
    await vscode.commands.executeCommand("ascribe.restartServer");
    await api.whenSettled();
    assert.equal(api.state(), "running");
    // The new server reads the project again.
    await diagnosticsOf(uriOf("docs", "todo.md"), (all) => all.length > 0);
  });

  it("registers its commands", async () => {
    const commands = await vscode.commands.getCommands(true);
    assert.ok(commands.includes("ascribe.restartServer"));
    assert.ok(commands.includes("ascribe.showOutput"));
    await vscode.commands.executeCommand("ascribe.showOutput");
  });

  it("stops after ascribe.maxCrashes crashes", async () => {
    const api = await activated();
    // ascribe.maxCrashes is 2 in this workspace: the second crash is the last.
    writeFileSync(path.join(workspace(), "docs", "crash.md"), "# Crash\n\nCRASH\n");
    await waitFor("the server to give up", () => api.state() === "failed", 45_000);
    rmSync(path.join(workspace(), "docs", "crash.md"));

    // Restarting works once the cause is gone.
    await vscode.commands.executeCommand("ascribe.restartServer");
    await api.whenSettled();
    assert.equal(api.state(), "running");
  });

  it("reports an ascribe.path that isn't a working binary, and recovers", async () => {
    const api = await activated();
    const config = vscode.workspace.getConfiguration("ascribe");
    const good = config.get<string>("path");
    await config.update(
      "path",
      path.join(workspace(), "no-such-ascribe"),
      vscode.ConfigurationTarget.Workspace,
    );
    await waitFor("the server to fail", () => api.state() === "failed");
    assert.equal(api.binary(), undefined);

    await config.update("path", good, vscode.ConfigurationTarget.Workspace);
    await waitFor("the server to run again", () => api.state() === "running");
  });
});
