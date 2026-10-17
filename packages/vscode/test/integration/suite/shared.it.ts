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
    const greet = uriOf("code", "greet.ts");
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
});
