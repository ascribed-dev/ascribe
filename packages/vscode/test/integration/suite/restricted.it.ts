import * as assert from "node:assert/strict";
import * as vscode from "vscode";
import { sleep } from "./helpers.js";

// A workspace with ascribe.toml, opened in Restricted Mode. The extension
// would run the project's own binary, so it starts nothing until the folder
// is trusted, and offers none of its tasks.
describe("in Restricted Mode", () => {
  it("offers no ascribe task", async () => {
    assert.equal(vscode.workspace.isTrusted, false, "the workspace should be in Restricted Mode");
    // Time for the extension to activate on ascribe.toml, as far as it goes.
    await sleep(3000);
    assert.deepEqual(await vscode.tasks.fetchTasks({ type: "ascribe" }), []);
    const all = await vscode.tasks.fetchTasks();
    assert.deepEqual(
      all.filter((task) => task.definition.type === "ascribe").map((task) => task.name),
      [],
    );
  });
});
