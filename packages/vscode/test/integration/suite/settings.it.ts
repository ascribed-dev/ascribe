// Project Settings on a copy of examples/quill whose `[consumer]` sets the
// trailing slash and the base path, against the real `ascribe lsp`: the
// round trip of decision #658. A value chosen in the tab becomes an edit to
// `ascribe.toml`, unsaved, with its comments as they were; the server reads
// the edited text, and the tab draws what it now says; Undo in the file puts
// it back, and the tab follows; and a base path with no leading `/` is set,
// with the server's problem under it.
import * as assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import * as vscode from "vscode";
import type { Drawn } from "../../../src/ui/webviewView.js";
import type { SettingsReport } from "../../../src/webview/settingsProtocol.js";
import { activated, uriOf, waitFor } from "./helpers.js";

const SLASH = "consumer.trailing-slash";
const BASE = "consumer.base-path";

/** A setting as the tab drew it. */
const shown = (drawn: Drawn<SettingsReport>, id: string) => {
  const found = drawn.report.settings.find((s) => s.id === id);
  assert.ok(found, `the tab drew no ${id}`);
  return found;
};

/** The lines of `after` that differ from `before`'s, which have as many. */
function changedLines(before: string, after: string): string[] {
  const a = before.split("\n");
  const b = after.split("\n");
  assert.equal(b.length, a.length, "the edit changed how many lines the file has");
  return b.filter((line, i) => line !== a[i]);
}

describe("Project Settings on examples/quill", () => {
  const model = (): vscode.Uri => uriOf("ascribe.toml");
  let onDisk = "";

  before(() => {
    onDisk = readFileSync(model().fsPath, "utf8");
  });

  after(async () => {
    await vscode.commands.executeCommand("workbench.action.closeAllEditors");
  });

  it("shows the trailing slash and the base path, each with its value, its default, its description, and its line", async () => {
    const api = await activated();
    await vscode.commands.executeCommand("ascribe.openProjectSettings");
    const settings = api.ui.projectSettings;
    const drawn = await settings.whenDrawn(
      "the two settings",
      (d) => d.report.settings.length === 2,
    );
    const line = (text: string): number => onDisk.split("\n").indexOf(text) + 1;
    assert.deepEqual(drawn.report, {
      message: null,
      settings: [
        {
          id: SLASH,
          value: "always",
          where: `ascribe.toml:${line('trailing-slash = "always" # as Astro\'s')}`,
          problems: [],
          editable: true,
        },
        {
          id: BASE,
          value: "/",
          where: `ascribe.toml:${line('base-path = "/"')}`,
          problems: [],
          editable: true,
        },
      ],
      violations: [],
    });
    // Opening it again shows the tab that's open.
    await vscode.commands.executeCommand("ascribe.openProjectSettings");
    assert.equal(settings.tabs(), 1);
  });

  it("sets the trailing slash to never in one line, unsaved, and follows Undo in the file", async () => {
    const settings = (await activated()).ui.projectSettings;
    await settings.set(SLASH, "never");
    const set = await settings.whenDrawn(
      "the trailing slash, never",
      (d) => shown(d, SLASH).value === "never",
    );
    const document = vscode.workspace.textDocuments.find(
      (d) => d.uri.toString() === model().toString(),
    );
    assert.ok(document, "ascribe.toml isn't open");
    assert.equal(document.isDirty, true, "the edit was saved");
    assert.deepEqual(changedLines(onDisk, document.getText()), [
      'trailing-slash = "never" # as Astro\'s',
    ]);
    assert.equal(readFileSync(model().fsPath, "utf8"), onDisk, "the file on disk changed");
    assert.equal(shown(set, BASE).value, "/");

    // Undo in the file puts it back, and the tab follows.
    await vscode.window.showTextDocument(document);
    await vscode.commands.executeCommand("undo");
    await settings.whenDrawn(
      "the trailing slash, always",
      (d) => shown(d, SLASH).value === "always",
    );
    assert.equal(document.getText(), onDisk);
  });

  it("follows typing in the file, and never draws an answer for older text", async () => {
    const settings = (await activated()).ui.projectSettings;
    const editor = await vscode.window.showTextDocument(model(), { preview: false });
    const at = editor.document.getText().indexOf('base-path = "/"') + 'base-path = "/'.length;
    // Three keystrokes, each a version the server reads.
    for (const ch of ["d", "o", "c"]) {
      await editor.edit((edit) =>
        edit.insert(editor.document.positionAt(at + ["d", "o", "c"].indexOf(ch)), ch),
      );
    }
    const drawn = await settings.whenDrawn(
      "the base path, /doc",
      (d) => shown(d, BASE).value === "/doc",
    );
    assert.equal(shown(drawn, BASE).value, "/doc");
    await vscode.commands.executeCommand("undo");
    await vscode.commands.executeCommand("undo");
    await vscode.commands.executeCommand("undo");
    await settings.whenDrawn("the base path, /", (d) => shown(d, BASE).value === "/");
    assert.equal(editor.document.getText(), onDisk);
  });

  it("sets a base path with no leading slash, and shows the server's problem under it", async () => {
    const settings = (await activated()).ui.projectSettings;
    await settings.set(BASE, "docs");
    const drawn = await settings.whenDrawn(
      "the base path's problem",
      (d) => shown(d, BASE).value === "docs" && shown(d, BASE).problems.length > 0,
    );
    const problems = vscode.languages
      .getDiagnostics(model())
      .filter((d) => d.severity === vscode.DiagnosticSeverity.Error)
      .map((d) => d.message);
    assert.deepEqual(shown(drawn, BASE).problems, [
      'base-path must start with "/", such as "/docs/"',
    ]);
    assert.ok(
      problems.includes('base-path must start with "/", such as "/docs/"'),
      problems.join("\n"),
    );
    assert.deepEqual(shown(drawn, SLASH).problems, []);

    // Set back from the tab, from the file as it is, though the model doesn't load.
    await settings.set(BASE, "/");
    await settings.whenDrawn(
      "the base path, / again",
      (d) => shown(d, BASE).value === "/" && shown(d, BASE).problems.length === 0,
    );
    const document = await waitFor("ascribe.toml as it was", () => {
      const doc = vscode.workspace.textDocuments.find(
        (d) => d.uri.toString() === model().toString(),
      );
      return doc?.getText() === onDisk ? doc : undefined;
    });
    assert.equal(document.getText(), onDisk);
  });

  it("opens ascribe.toml at a setting's line", async () => {
    const settings = (await activated()).ui.projectSettings;
    await settings.openLine(SLASH);
    const editor = await waitFor("ascribe.toml's editor", () =>
      vscode.window.activeTextEditor?.document.uri.toString() === model().toString()
        ? vscode.window.activeTextEditor
        : undefined,
    );
    assert.equal(
      editor.document.lineAt(editor.selection.start.line).text,
      'trailing-slash = "always" # as Astro\'s',
    );
  });
});
