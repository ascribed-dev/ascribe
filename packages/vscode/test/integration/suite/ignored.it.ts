import * as assert from "node:assert/strict";
import * as path from "node:path";
import * as vscode from "vscode";
import type { AscribeApi } from "../../../src/extension.js";
import { comparable } from "../../../src/projects.js";
import { activated, sleep, uriOf, workspace } from "./helpers.js";

// test/fixtures/stub-project as a git repository, with servers started at
// once: its own project and `guides/` are projects, and the `ascribe.toml`
// files in `target/`, in `generated/` (which git ignores), and in the root
// project's source copies (`sources/api/`) aren't. Which folders are left
// out by name is held by the unit tests of `projectExclude`.
describe("with ascribe.toml files in build output, ignored folders, and a source's copies", () => {
  let api: AscribeApi;

  const found = (): string[] =>
    api
      .projects()
      .map((project) => comparable(project.folder))
      .sort();

  const expected = (): string[] =>
    [workspace(), path.join(workspace(), "guides")].map((folder) => comparable(folder)).sort();

  before(async () => {
    api = await activated();
    await api.whenSettled();
  });

  it("lists and starts only the projects beside them", async () => {
    assert.deepEqual(found(), expected());
    assert.deepEqual(
      api.projects().map((project) => project.state),
      ["running", "running"],
    );
    const listed = api.ui.switchProject
      .items()
      .map((item) => item.label.replace(/^\$\([\w-]+\) /, ""))
      .sort();
    assert.deepEqual(listed, ["guides", path.basename(workspace())].sort());
  });

  it("starts none of them when one of their pages is opened", async () => {
    for (const folder of [
      ["target", "tmp", "case"],
      ["generated", "site"],
      ["sources", "api"],
    ]) {
      await vscode.window.showTextDocument(
        await vscode.workspace.openTextDocument(uriOf(...folder, "docs", "page.md")),
      );
    }
    // Long enough for the search an unknown `ascribe.toml` asks for.
    await sleep(2000);
    await api.whenSettled();
    assert.deepEqual(found(), expected());
  });
});
