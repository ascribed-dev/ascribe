import * as assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import * as vscode from "vscode";
import type { PreviewApi, RenderRecord } from "../../../src/preview/controller.js";
import type { PageChanges } from "../../../src/preview/protocol.js";
import { activated, uriOf, waitFor, workspace } from "./helpers.js";

// Review in the preview with the real `ascribe lsp`, on a copy of
// examples/quill made into a git repository with one commit and a change in
// the working tree (`run.ts` prepares it): install-agent.md's first paragraph
// is reworded, and a paragraph is added to the prerequisites fragment.
describe("review, with the real language server on a quill repository", () => {
  const install = uriOf("docs", "install-agent.md");
  let preview: PreviewApi;
  let binary: string;

  const drawn = (description: string, predicate: (render: RenderRecord) => boolean) =>
    preview.whenDrawn(description, predicate);

  before(async () => {
    const api = await activated();
    await api.whenSettled();
    assert.equal(api.state(), "running");
    preview = api.preview;
    const resolved = api.binary();
    assert.ok(resolved);
    binary = resolved.path;
    await vscode.window.showTextDocument(await vscode.workspace.openTextDocument(install));
    await vscode.commands.executeCommand("ascribe.openPreview");
    await drawn("the page", (r) => r.result.page?.path === "install-agent.md");
  });

  after(async () => {
    await vscode.commands.executeCommand("workbench.action.closeAllEditors");
  });

  it("shows no review header while it's off", async () => {
    const render = await drawn("the page", (r) => r.report?.reviewHeader !== undefined);
    assert.equal(render.report?.reviewHeader, null);
    assert.deepEqual(render.report?.marks, {});
    assert.equal(preview.review.status().text, "$(git-compare) Review: off");
  });

  it("marks what `ascribe diff` reports for the same base", async () => {
    const started = await preview.review.start(workspace());
    assert.equal(started.problem, null);
    assert.equal(started.base?.requested, "main");
    const render = await drawn(
      "the marks",
      (r) => (r.report?.marks["changed"] ?? 0) > 0 && r.result.review?.changes !== null,
    );
    const report = JSON.parse(
      execFileSync(binary, ["diff", "--format", "json"], { cwd: workspace(), encoding: "utf8" }),
    ) as { builds: { build: string; pages: PageChanges[] }[] };
    const site = report.builds.find((b) => b.build === render.result.build);
    const fromCli = site?.pages.find((p) => p.path === "install-agent.md");
    assert.ok(fromCli, "ascribe diff reports the page");
    assert.deepEqual(render.result.review?.changes, fromCli);
    // Every change is marked on the page.
    const kinds: Record<string, number> = {};
    for (const change of fromCli.changes) kinds[change.kind] = (kinds[change.kind] ?? 0) + 1;
    assert.deepEqual(render.report?.marks, kinds);
    assert.match(render.report?.reviewHeader ?? "", /^Against main/);
    assert.equal(preview.review.status().text, "$(git-compare) Review: main");
  });

  it("updates the marks as the buffer changes, before it's saved", async () => {
    const editor = await vscode.window.showTextDocument(install);
    const before = preview.renders().at(-1)?.report?.marks["added"] ?? 0;
    const end = editor.document.lineCount;
    await editor.edit((edit) =>
      edit.insert(new vscode.Position(end, 0), "\nA paragraph added in the editor.\n"),
    );
    await drawn("one more added block", (r) => (r.report?.marks["added"] ?? 0) === before + 1);
    await vscode.commands.executeCommand("undo");
    await drawn("the added block gone", (r) => (r.report?.marks["added"] ?? 0) === before);
  });

  it("lists the changed pages", async () => {
    const pages = await preview.review.changedPages(workspace());
    assert.deepEqual(
      pages.map((p) => [p.path, p.title, p.own_file_changed]),
      [["install-agent.md", "Install the Quill agent", true]],
    );
  });

  it("removes the marks when review stops", async () => {
    await preview.review.stop(workspace());
    const render = await drawn("no marks", (r) => r.report?.reviewHeader === null);
    assert.deepEqual(render.report?.marks, {});
    assert.equal(render.result.review, null);
    assert.equal(preview.review.base(workspace()), undefined);
  });

  it("says why a base that isn't a revision can't be used", async () => {
    const result = await preview.review.start(workspace(), "no-such-branch");
    assert.equal(result.base, null);
    assert.equal(
      result.problem,
      "`no-such-branch` isn't a branch, tag, or commit of this repository.",
    );
    await waitFor("review to stay off", () => preview.review.base(workspace()) === undefined);
  });
});
