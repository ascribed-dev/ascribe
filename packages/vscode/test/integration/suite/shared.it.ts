import * as assert from "node:assert/strict";
import * as vscode from "vscode";
import type { AscribeApi } from "../../../src/extension.js";
import { activated, uriOf, waitFor } from "./helpers.js";

// The real `ascribe lsp` on test/fixtures/shared-sources: two projects, docs/
// and api/, whose sources are the same folder, code/. Each project's server
// answers for the code file, and each lens names its project.
describe("a code file two projects show", () => {
  let api: AscribeApi;

  before(async () => {
    api = await activated();
    await api.whenSettled();
    await waitFor("both servers", () => {
      const projects = api.projects();
      return projects.length === 2 && projects.every((p) => p.state === "running");
    });
  });

  it("has a lens from each project, each naming its project", async () => {
    const greet = uriOf("code", "greet.py");
    await vscode.window.showTextDocument(greet);
    const titles = await waitFor("a lens from each project", async () => {
      const found = await vscode.commands.executeCommand<vscode.CodeLens[]>(
        "vscode.executeCodeLensProvider",
        greet,
        10,
      );
      const titles = found.map((lens) => lens.command?.title ?? "").sort();
      return titles.length === 2 && titles.every((t) => t.includes(":")) && titles;
    });
    assert.deepEqual(titles, ["api: shown in 1 page", "docs: shown in 1 page"]);
  });

  // The code file is outside both projects' folders, so each server hears
  // it was saved only if its file events get through (`scope.ts`).
  it("moves each project's lens when the file is saved with lines above its region", async () => {
    const greet = uriOf("code", "greet.py");
    const editor = await vscode.window.showTextDocument(greet);
    const folders = api.projects().map((p) => p.folder);
    const before = folders.map((folder) => api.lensRefreshes(folder));
    await editor.edit((edit) => edit.insert(new vscode.Position(0, 0), "import os\n\n"));
    try {
      await editor.document.save();
      await waitFor("each server to ask for its lenses again", () =>
        folders.every((folder, i) => api.lensRefreshes(folder) > (before[i] ?? 0)),
      );
      const lines = await waitFor("both lenses on the region's new line", async () => {
        const found = await vscode.commands.executeCommand<vscode.CodeLens[]>(
          "vscode.executeCodeLensProvider",
          greet,
          10,
        );
        const lines = found.map((lens) => lens.range.start.line);
        return lines.length === 2 && lines.every((line) => line === 2) && lines;
      });
      assert.deepEqual(lines, [2, 2]);
    } finally {
      await editor.edit((edit) => edit.delete(new vscode.Range(0, 0, 2, 0)));
      await editor.document.save();
    }
  });
});
