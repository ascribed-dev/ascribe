import * as assert from "node:assert/strict";
import { existsSync, readFileSync } from "node:fs";
import * as vscode from "vscode";
import type { NewPageRecord } from "../../../src/actions/newPage.js";
import type { Scripted, Step } from "../../../src/actions/steps.js";
import type { AscribeApi } from "../../../src/extension.js";
import { activated, uriOf, waitFor } from "./helpers.js";

// Starting a page in a project with the six page types and their templates
// (a copy of examples/templates), against the real `ascribe lsp`: New Page
// on a folder, with scripted answers, and Go to Next Placeholder in the page
// it made.
describe("starting a page from a template", () => {
  let api: AscribeApi;

  before(async () => {
    api = await activated();
    await api.whenSettled();
    await waitFor("the server", () => api.state() === "running");
  });

  /** Runs New Page with answers for its questions, and what the run did. */
  async function newPage(answers: Scripted[], folder?: vscode.Uri): Promise<NewPageRecord> {
    const before = api.newPage.runs.length;
    api.newPage.answerNext(answers);
    await vscode.commands.executeCommand("ascribe.newPage", folder);
    const record = api.newPage.runs[before];
    assert.ok(record, "New Page didn't run");
    return record;
  }

  it("on the how-to folder, offers the how-to, makes the page, and selects its first placeholder", async () => {
    let offered: string[] = [];
    const record = await newPage(
      [
        (step: Step) => {
          offered = step.kind === "text" ? [] : step.choices.map((choice) => choice.value);
          return Promise.resolve("how-to");
        },
        "Rotate signing keys",
      ],
      uriOf("docs", "how-to"),
    );
    assert.deepEqual(record.messages, []);
    assert.deepEqual(offered, ["how-to"]);
    assert.equal(record.file, "docs/how-to/rotate-signing-keys.md");

    const file = uriOf("docs", "how-to", "rotate-signing-keys.md");
    const editor = vscode.window.activeTextEditor;
    assert.equal(editor?.document.uri.fsPath, file.fsPath);
    assert.equal(editor.document.isDirty, false, "the page is saved");
    const template = readFileSync(uriOf("templates", "how-to.md").fsPath, "utf8");
    assert.equal(
      readFileSync(file.fsPath, "utf8"),
      template.replace('title: "Title"', 'title: "Rotate signing keys"'),
    );
    const placeholder =
      "{One sentence that says what a reader can do once they've followed this page.}";
    assert.equal(record.selected, placeholder);
    assert.equal(editor.document.getText(editor.selection), placeholder);

    // Typing replaces it, and the next placeholder is in the body.
    await editor.edit((edit) =>
      edit.replace(editor.selection, "Rotate the keys a release is signed with."),
    );
    await vscode.commands.executeCommand("ascribe.nextPlaceholder");
    await waitFor("the next placeholder", () =>
      editor.document.getText(editor.selection) === "\\{insert a brief description of the task}"
        ? true
        : undefined,
    );
    await vscode.commands.executeCommand("workbench.action.files.revert");
    await vscode.commands.executeCommand("workbench.action.closeAllEditors");
  });

  it("never makes a page over a file", async () => {
    const record = await newPage(["how-to", "Rotate keys"], uriOf("docs", "how-to"));
    assert.equal(record.file, undefined);
    assert.equal(record.messages.length, 1);
    assert.match(record.messages[0] ?? "", /is there already/);
    assert.ok(existsSync(uriOf("docs", "how-to", "rotate-keys.md").fsPath));
  });
});
