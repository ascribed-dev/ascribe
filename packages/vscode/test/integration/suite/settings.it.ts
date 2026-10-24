// Project Settings on a copy of examples/quill whose `[consumer]` sets the
// trailing slash and the base path, against the real `ascribe lsp`: the
// round trip of decision #658. A value chosen in the tab becomes an edit to
// `ascribe.toml`, unsaved, with its comments as they were; the server reads
// the edited text, and the tab draws what it now says; Undo in the file puts
// it back, and the tab follows; and a base path with no leading `/` is set,
// with the server's problem under it. Every simple setting is shown: one the
// file leaves out is added to its table, and Reset to default takes one out
// with its comment. The checks are listed, every configurable one, and a
// check's level is set from the tab. The title bar's button is offered on the
// project's `ascribe.toml` and not on a fixture's that
// `ascribe.projects.exclude` leaves out, and opens the tab already open; and
// when `ascribe.toml` is deleted, the tab says so, and a click closes it.
import * as assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import * as vscode from "vscode";
import type { Drawn } from "../../../src/ui/webviewView.js";
import type { SettingsReport } from "../../../src/webview/settingsProtocol.js";
import { activated, sleep, uriOf, waitFor } from "./helpers.js";

const SLASH = "consumer.trailing-slash";
const BASE = "consumer.base-path";
const HTML = "consumer.html";
const ORPHAN = "checks.page-orphan.level";

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

/**
 * Undo in `document`, `steps` times, each counted only once it changes the
 * document. Undo goes to whatever has the focus, and the workbench can take
 * it as the test runs (VS Code's agent host, starting late, does), so each
 * step shows the file beside the tab, waits for its editor to be the active
 * one, and tries again when the document didn't change.
 */
async function undoIn(document: vscode.TextDocument, steps = 1): Promise<void> {
  for (let step = 0; step < steps; step++) {
    const version = document.version;
    await waitFor(`Undo ${step + 1} of ${steps} in ${document.uri.fsPath}`, async () => {
      // An Undo that landed late counts, and isn't run again.
      if (document.version !== version) return true;
      await vscode.window.showTextDocument(document, {
        viewColumn: vscode.ViewColumn.Two,
        preserveFocus: false,
      });
      if (vscode.window.activeTextEditor?.document !== document) return false;
      await vscode.commands.executeCommand("undo");
      for (let waited = 0; waited < 1_000; waited += 50) {
        if (document.version !== version) return true;
        await sleep(50);
      }
      return false;
    });
  }
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
    const drawn = await settings.whenDrawn("the settings", (d) =>
      d.report.settings.some((s) => s.id === SLASH),
    );
    const line = (text: string): number => onDisk.split("\n").indexOf(text) + 1;
    assert.equal(drawn.report.message, null);
    assert.deepEqual(drawn.report.violations, []);
    assert.deepEqual(shown(drawn, SLASH), {
      id: SLASH,
      value: "always",
      where: `ascribe.toml:${line('trailing-slash = "always" # as Astro\'s')}`,
      problems: [],
      editable: true,
      resettable: true,
    });
    assert.deepEqual(shown(drawn, BASE), {
      id: BASE,
      value: "/",
      where: `ascribe.toml:${line('base-path = "/"')}`,
      problems: [],
      editable: true,
      resettable: true,
    });
    // A setting the file leaves out, with its default, which the tab adds.
    assert.deepEqual(shown(drawn, HTML), {
      id: HTML,
      value: "true",
      where: "Not set in ascribe.toml",
      problems: [],
      editable: true,
      resettable: false,
    });
    // Every configurable check, with its level.
    const levels = drawn.report.settings.filter(
      (s) =>
        s.id.startsWith("checks.") && s.id.endsWith(".level") && s.id !== "checks.vale.max-level",
    );
    assert.equal(levels.length, 51);
    // Opening it again shows the tab that's open, with the focus. The focus
    // lands once the webview has loaded, which can be after the command is
    // done; a test that goes on to Undo in the file waits for it, or the
    // focus is taken from the file just as Undo runs, and goes to the tab.
    await vscode.commands.executeCommand("ascribe.openProjectSettings");
    assert.equal(settings.tabs(), 1);
    await settings.whenFocused();
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
    // Beside the tab, as the file is opened beside it: a hidden tab draws nothing.
    await undoIn(document);
    await settings.whenDrawn(
      "the trailing slash, always",
      (d) => shown(d, SLASH).value === "always",
    );
    assert.equal(document.getText(), onDisk);
  });

  it("follows typing in the file, and never draws an answer for older text", async () => {
    const settings = (await activated()).ui.projectSettings;
    const editor = await vscode.window.showTextDocument(model(), {
      viewColumn: vscode.ViewColumn.Two,
      preview: false,
    });
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
    await undoIn(editor.document, 3);
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

  it("adds a value the file leaves out to its table, and Reset to default takes it out", async () => {
    const settings = (await activated()).ui.projectSettings;
    await settings.set(HTML, false);
    await settings.whenDrawn(
      "html, false",
      (d) => shown(d, HTML).value === "false" && shown(d, HTML).resettable,
    );
    const document = vscode.workspace.textDocuments.find(
      (d) => d.uri.toString() === model().toString(),
    );
    assert.ok(document, "ascribe.toml isn't open");
    assert.equal(
      document.getText(),
      // At the end of [consumer], after its last key.
      onDisk.replace("agents = false\n", "agents = false\nhtml = false\n"),
    );
    await settings.reset(HTML);
    await settings.whenDrawn("html, its default", (d) => !shown(d, HTML).resettable);
    assert.equal(document.getText(), onDisk);

    // The comment that touches the line goes with it; Undo puts both back.
    await settings.reset(SLASH);
    await settings.whenDrawn("the trailing slash, its default", (d) => !shown(d, SLASH).resettable);
    assert.equal(
      document.getText(),
      onDisk.replace('# Pages end in a slash.\ntrailing-slash = "always" # as Astro\'s\n', ""),
    );
    await undoIn(document);
    await settings.whenDrawn("the trailing slash, set again", (d) => shown(d, SLASH).resettable);
    assert.equal(document.getText(), onDisk);
  });

  it("sets a check's level, and lists the check first once the file names it", async () => {
    const settings = (await activated()).ui.projectSettings;
    await settings.set(ORPHAN, "off");
    const drawn = await settings.whenDrawn(
      "page-orphan, off",
      (d) => shown(d, ORPHAN).value === "off" && shown(d, ORPHAN).resettable,
    );
    const firstCheck = drawn.report.settings.find(
      (s) =>
        s.id.startsWith("checks.") && s.id.endsWith(".level") && s.id !== "checks.vale.max-level",
    );
    assert.equal(firstCheck?.id, ORPHAN);
    const document = vscode.workspace.textDocuments.find(
      (d) => d.uri.toString() === model().toString(),
    );
    assert.ok(document, "ascribe.toml isn't open");
    assert.equal(document.getText(), `${onDisk}\n[checks.page-orphan]\nlevel = "off"\n`);
    await undoIn(document);
    await settings.whenDrawn("page-orphan, its default", (d) => !shown(d, ORPHAN).resettable);
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

  it("offers its button on the project's ascribe.toml and not on a fixture's, and opens the tab already open", async () => {
    const settings = (await activated()).ui.projectSettings;
    assert.equal(settings.offeredOn(model()), true);
    assert.equal(settings.offeredOn(uriOf("tests/fixture/ascribe.toml")), false);
    assert.equal(settings.offeredOn(uriOf("docs/index.md")), false);
    // The button passes the file it's on.
    await vscode.commands.executeCommand("ascribe.openProjectSettings", model());
    assert.equal(settings.tabs(), 1);
  });

  it("says the project is gone when ascribe.toml is deleted, and closes on the next click", async () => {
    const settings = (await activated()).ui.projectSettings;
    await vscode.workspace.fs.delete(model());
    await settings.whenDrawn("that the project is gone", (d) =>
      (d.report.message ?? "").startsWith("ascribe.toml was deleted or renamed"),
    );
    assert.equal(settings.tabs(), 1);
    settings.click();
    assert.equal(settings.tabs(), 0);
  });
});
