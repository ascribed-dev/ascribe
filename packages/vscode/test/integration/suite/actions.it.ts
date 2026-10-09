import * as assert from "node:assert/strict";
import * as vscode from "vscode";
import type { RunRecord } from "../../../src/actions/run.js";
import type { Scripted } from "../../../src/actions/steps.js";
import type { AscribeApi } from "../../../src/extension.js";
import { activated, uriOf, waitFor } from "./helpers.js";

// The editor's actions, run through their commands with scripted answers to
// their wizards, on a copy of examples/quill against the real `ascribe lsp`.
describe("the editor's actions", () => {
  let api: AscribeApi;

  before(async () => {
    api = await activated();
    await api.whenSettled();
    await waitFor("the server", () => api.state() === "running");
  });

  afterEach(async () => {
    // Back to the text on disk, and nothing left open.
    for (const document of vscode.workspace.textDocuments) {
      if (document.isDirty) {
        await vscode.window.showTextDocument(document);
        await vscode.commands.executeCommand("workbench.action.files.revert");
      }
    }
    await vscode.commands.executeCommand("workbench.action.closeAllEditors");
  });

  /** Opens a page of docs/ with the selection given (a cursor when it ends where it starts). */
  async function open(
    file: string,
    line: number,
    character: number,
    endLine = line,
    endCharacter = character,
  ): Promise<vscode.TextEditor> {
    const document = await vscode.workspace.openTextDocument(uriOf("docs", file));
    const editor = await vscode.window.showTextDocument(document);
    editor.selection = new vscode.Selection(line, character, endLine, endCharacter);
    return editor;
  }

  /** Runs an action's command with answers for its wizard, and what the run did. */
  async function run(id: string, answers: Scripted[] = []): Promise<RunRecord> {
    const before = api.actions.runs.length;
    api.actions.answerNext(answers);
    await vscode.commands.executeCommand(`ascribe.action.${id}`);
    const record = api.actions.runs[before];
    assert.ok(record, `${id} didn't run`);
    return record;
  }

  const lines = (editor: vscode.TextEditor): string[] => editor.document.getText().split(/\r?\n/);

  it("is in a project in a page of it", async () => {
    await open("keys.md", 8, 0);
    await waitFor("ascribe.inProject", () => api.actions.inProject());
  });

  it("wraps a list item's paragraph in a note, indented to the item, as one undo step", async () => {
    // "   The command prints the installed version, {version}." in step 2.
    const editor = await open("install-agent.md", 43, 10);
    const before = editor.document.getText();
    const record = await run("wrapNote", ["tip"]);
    assert.deepEqual(record, { action: "wrapNote", edits: 1, done: true, messages: [] });
    assert.deepEqual(lines(editor).slice(43, 45), [
      "   @note {type=tip}",
      "   The command prints the installed version, {version}.",
    ]);
    await vscode.commands.executeCommand("undo");
    assert.equal(editor.document.getText(), before);
  });

  it("inserts a note and leaves its placeholder selected for typing", async () => {
    // The blank line between "Sign in to {cloud} …" and "## Rotate keys".
    const editor = await open("keys.md", 9, 0);
    const record = await run("insertNote", ["warning"]);
    assert.equal(record.done, true, record.messages.join());
    assert.equal(editor.document.getText(editor.selection), "Write the note here.");
    assert.ok(
      lines(editor).includes("@note {type=warning}: Write the note here."),
      editor.document.getText(),
    );
  });

  it("asks again, and tries once more, when the page changes under it", async () => {
    // In the tip's paragraph, "You can run {product} in the browser …".
    const editor = await open("install-agent.md", 10, 4);
    const record = await run("setNoteType", [
      async () => {
        // An edit while the wizard is open: the context's version is stale.
        const end = editor.document.lineAt(editor.document.lineCount - 1).range.end;
        assert.ok(await editor.edit((e) => e.insert(end, "\nOne more paragraph.\n")));
        return "warning";
      },
    ]);
    assert.deepEqual(record, { action: "setNoteType", edits: 2, done: true, messages: [] });
    assert.equal(lines(editor)[9], "@note {type=warning}");
    assert.ok(editor.document.getText().includes("One more paragraph."));
  });

  it("copies a link to the section the cursor is in, from the content root", async () => {
    await open("keys.md", 10, 5);
    await vscode.env.clipboard.writeText("");
    const record = await run("copyLinkToSection");
    assert.equal(record.done, true, record.messages.join());
    assert.equal(await vscode.env.clipboard.readText(), "/keys.md#rotate-keys");
  });

  it("links the selected text to the page picked", async () => {
    // "the playground" in "Open the playground at play.quill.dev …".
    const editor = await open("quickstart.md", 8, 5, 8, 19);
    const record = await run("linkSelection", ["keys.md"]);
    assert.equal(record.done, true, record.messages.join());
    assert.equal(
      lines(editor)[8],
      "Open [the playground](keys.md) at play.quill.dev and paste a page of your docs.",
    );
  });

  it("inserts the phrase picked from the project's", async () => {
    // After "Sign" in "Sign in to {cloud} and open …".
    const editor = await open("keys.md", 8, 4);
    const record = await run("insertPhrase", ["product"]);
    assert.equal(record.done, true, record.messages.join());
    assert.ok(lines(editor)[8]?.startsWith("Sign{product} in to {cloud}"), lines(editor)[8]);
  });

  it("says where an action applies, and changes nothing, where it doesn't", async () => {
    const editor = await open("quickstart.md", 8, 4);
    const record = await run("unwrapNote");
    assert.deepEqual(record, {
      action: "unwrapNote",
      edits: 0,
      done: false,
      messages: ["Put the cursor in a note to take its text out of it."],
    });
    assert.equal(editor.document.isDirty, false);
  });

  it("offers its rewrites in the lightbulb once the cursor's context is known", async () => {
    const editor = await open("quickstart.md", 8, 4);
    const titles = await waitFor("the lightbulb's actions", async () => {
      const actions = await vscode.commands.executeCommand<vscode.CodeAction[]>(
        "vscode.executeCodeActionProvider",
        editor.document.uri,
        editor.selection,
        vscode.CodeActionKind.RefactorRewrite.value,
      );
      const ours = actions.filter((a) => a.kind?.value === "refactor.rewrite.ascribe");
      return ours.length > 0 && ours.map((a) => a.title).sort();
    });
    assert.deepEqual(titles, ["Wrap in a note", "Wrap in collapsible details"]);
  });
});
