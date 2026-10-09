import * as assert from "node:assert/strict";
import * as vscode from "vscode";
import type { AscribeApi } from "../../../src/extension.js";
import type { LensShown } from "../../../src/ui/buildLens.js";
import { activated, uriOf, waitFor, workspace } from "./helpers.js";

// The build lens with the real `ascribe lsp` on a copy of
// examples/monorepo/docs (Lantern): its `self-hosted` build selects the
// self-hosted edition and filters out what 2.5 doesn't have.
describe("the build lens, with the real language server on the Lantern docs", () => {
  const gettingStarted = uriOf("content", "getting-started.md");
  const rollouts = uriOf("content", "guides", "rollouts.md");
  let api: AscribeApi;

  async function open(uri: vscode.Uri): Promise<vscode.TextEditor> {
    // Beside the preview, which opens to the side.
    return vscode.window.showTextDocument(await vscode.workspace.openTextDocument(uri), {
      viewColumn: vscode.ViewColumn.One,
    });
  }

  /** Waits until the lens shows something on `uri` that satisfies `check`. */
  function shown(uri: vscode.Uri, description: string, check: (s: LensShown) => boolean) {
    return waitFor(`the lens to show ${description}`, () => {
      const now = api.ui.lens.shown(uri);
      return now && check(now) ? now : undefined;
    });
  }

  /** The line of the first line of the document that starts with `text`. */
  async function lineOf(uri: vscode.Uri, text: string): Promise<number> {
    const document = await vscode.workspace.openTextDocument(uri);
    for (let line = 0; line < document.lineCount; line++) {
      if (document.lineAt(line).text.startsWith(text)) return line;
    }
    assert.fail(`${uri.fsPath} has no line starting with ${text}`);
  }

  const startLine = (s: LensShown, i = 0) => Number(s.dimmed[i]?.range.split(":")[0]);

  before(async () => {
    api = await activated();
    await api.whenSettled();
    assert.equal(api.state(), "running");
    await api.ui.whenBuildsKnown();
  });

  after(async () => {
    await vscode.commands.executeCommand("workbench.action.closeAllEditors");
  });

  it("dims nothing in the editor build, which switches and badges", async () => {
    await open(gettingStarted);
    assert.equal(api.ui.lens.isOn(workspace()), false);
    await vscode.commands.executeCommand("ascribe.toggleBuildLens");
    assert.equal(api.ui.lens.isOn(workspace()), true);
    const site = await shown(gettingStarted, "the site build", (s) => s.build === "site");
    assert.deepEqual(site.dimmed, []);
    await waitFor("the status bar to show the eye", () =>
      api.ui.statusBar.shown()?.text.startsWith("$(eye) "),
    );
  });

  it("dims what the build picked in the preview leaves out, and both show that build", async () => {
    await vscode.commands.executeCommand("ascribe.openPreview");
    await api.preview.whenDrawn("the page", (r) => r.result.page?.path === "getting-started.md");
    // What the webview posts when the reader picks a build.
    await api.preview.receive({ type: "build", name: "self-hosted" });
    const self = await shown(
      gettingStarted,
      "self-hosted's cloud arm",
      (s) => s.build === "self-hosted" && s.dimmed.length > 0,
    );
    assert.equal(self.dimmed.length, 1);
    assert.equal(startLine(self), await lineOf(gettingStarted, "@variant {edition=cloud}:"));
    assert.match(
      self.dimmed[0]?.hover ?? "",
      /Left out of self-hosted.*Shows only edition=self-hosted/,
    );
    assert.equal(api.preview.build(), "self-hosted");
    assert.match(api.ui.statusBar.shown()?.text ?? "", /^\$\(eye\) .* · self-hosted$/);

    await open(rollouts);
    const section = await shown(
      rollouts,
      "the scheduled-rollouts section",
      (s) => s.dimmed.length > 0,
    );
    assert.equal(section.build, "self-hosted");
    assert.equal(section.dimmed.length, 1);
    assert.equal(startLine(section), await lineOf(rollouts, "## Scheduled rollouts"));
    assert.match(section.dimmed[0]?.hover ?? "", /Scheduled rollouts: available on Lantern Cloud/);
  });

  it("follows unsaved edits", async () => {
    const editor = await open(rollouts);
    const end = editor.document.lineAt(editor.document.lineCount - 1).range.end;
    assert.ok(await editor.edit((e) => e.insert(end, "\n\n@available: cloud\nCloud only.\n")));
    const edited = await shown(
      rollouts,
      "the new block",
      (s) => s.version === editor.document.version && s.dimmed.length === 2,
    );
    assert.equal(startLine(edited, 1), await lineOf(rollouts, "@available: cloud"));
    assert.equal(editor.document.isDirty, true);
    await vscode.commands.executeCommand("workbench.action.files.revert");
  });

  it("changes with the build picked in the status bar", async () => {
    await open(rollouts);
    api.ui.statusBar.answerNext("Switch build");
    api.ui.statusBar.answerNext("site");
    await vscode.commands.executeCommand("ascribe.projectMenu");
    const site = await shown(rollouts, "the site build", (s) => s.build === "site");
    assert.deepEqual(site.dimmed, []);
    await api.preview.whenDrawn("the site build", (r) => r.result.build === "site");
    assert.equal(api.preview.build(), "site");
  });

  it("removes every decoration when it's turned off", async () => {
    await api.preview.receive({ type: "build", name: "self-hosted" });
    await shown(
      rollouts,
      "self-hosted again",
      (s) => s.build === "self-hosted" && s.dimmed.length > 0,
    );
    api.ui.statusBar.answerNext("Stop dimming");
    await vscode.commands.executeCommand("ascribe.projectMenu");
    await waitFor("the lens to clear", () => api.ui.lens.shown(rollouts) === undefined);
    assert.equal(api.ui.lens.isOn(workspace()), false);
    assert.doesNotMatch(api.ui.statusBar.shown()?.text ?? "", /eye/);
  });
});
