import * as assert from "node:assert/strict";
import { mkdirSync, readdirSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import * as path from "node:path";
import * as vscode from "vscode";
import { activated, diagnosticsOf, EXTENSION_ID, uriOf, waitFor, workspace } from "./helpers.js";

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

  it("kills a server that doesn't stop when asked, with the process it started", async () => {
    const api = await activated();
    const hangs = path.join(workspace(), ".stub-hangs");
    const processes = path.join(workspace(), ".stub-processes");
    const running = (pid: number): boolean => {
      try {
        process.kill(pid, 0);
      } catch {
        return false;
      }
      // Killed, and waiting to be reaped: the stub's own process is handed
      // to the system's first process, which can take its time.
      try {
        return readFileSync(`/proc/${pid}/stat`, "utf8").split(") ")[1]?.[0] !== "Z";
      } catch {
        return true;
      }
    };
    writeFileSync(hangs, "");
    try {
      // Started again, the stub reads the marker: it starts a process of its
      // own, and won't stop.
      await vscode.commands.executeCommand("ascribe.restartServer");
      await api.whenSettled();
      const pids = await waitFor("the stub's process ids", () => {
        try {
          return readFileSync(processes, "utf8").trim().split(" ").map(Number);
        } catch {
          return undefined;
        }
      });
      assert.equal(pids.length, 2);
      assert.ok(pids.every(running));

      // Restarting asks it to stop, which times out.
      await vscode.commands.executeCommand("ascribe.restartServer");
      await api.whenSettled();
      assert.equal(api.state(), "running");
      // Killed before the new one started: the language client's own check
      // would come two seconds after the stop timed out, and not at all once
      // the host has exited.
      await waitFor("the stub's processes to be gone", () => !pids.some(running), 1_000);
      await diagnosticsOf(uriOf("docs", "todo.md"), (all) => all.length > 0);
    } finally {
      rmSync(hangs, { force: true });
      // The server running now read the marker too.
      await vscode.commands.executeCommand("ascribe.restartServer");
      await api.whenSettled();
      rmSync(processes, { force: true });
    }
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

  it("ships chat the skill, byte for byte the one `@ascribed/cli` ships", () => {
    const extension = vscode.extensions.getExtension(EXTENSION_ID);
    assert.ok(extension);
    const shipped = path.join(extension.extensionPath, "dist", "skills", "ascribe");
    const home = path.join(extension.extensionPath, "..", "cli", "skills", "ascribe");
    const files = (root: string) =>
      readdirSync(root, { recursive: true, withFileTypes: true })
        .filter((entry) => entry.isFile())
        .map((entry) => path.relative(root, path.join(entry.parentPath, entry.name)))
        .sort();
    assert.deepEqual(files(shipped), files(home));
    assert.ok(files(home).includes("SKILL.md"));
    for (const file of files(home)) {
      assert.ok(
        readFileSync(path.join(shipped, file)).equals(readFileSync(path.join(home, file))),
        `dist/skills/ascribe/${file} differs from packages/cli/skills/ascribe/${file}`,
      );
    }
  });

  it("tells chat whether the workspace has a synced copy of the skill", async () => {
    const api = await activated();
    // The run made `.agents/skills/ascribe/` before VS Code started, empty: a
    // folder made while the watcher runs can go unreported on Linux.
    const copy = path.join(workspace(), ".agents", "skills", "ascribe", "SKILL.md");
    await api.agents.skill.whenSettled();
    assert.equal(api.agents.skill.synced(), false);

    writeFileSync(copy, "---\nname: ascribe\n---\n");
    await waitFor("the synced copy to be seen", async () => {
      await api.agents.skill.whenSettled();
      return api.agents.skill.synced();
    });

    rmSync(copy);
    await waitFor("the synced copy to be gone", async () => {
      await api.agents.skill.whenSettled();
      return !api.agents.skill.synced();
    });
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
    // This page says so, in place of the page, with what helps.
    await vscode.window.showTextDocument(
      await vscode.workspace.openTextDocument(uriOf("docs", "page.md")),
    );
    await vscode.commands.executeCommand("ascribe.thisPage.focus");
    const failed = await api.ui.thisPage.whenDrawn("the failure", (drawn) =>
      (drawn.report.message ?? "").endsWith(": the language server failed to start."),
    );
    assert.deepEqual(failed.report.actions, ["Show Output", "Restart"]);
    assert.deepEqual(failed.report.head, []);

    await config.update("path", good, vscode.ConfigurationTarget.Workspace);
    await waitFor("the server to run again", () => api.state() === "running");
  });
});
