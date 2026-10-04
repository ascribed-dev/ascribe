import * as assert from "node:assert/strict";
import { copyFileSync, mkdirSync, writeFileSync } from "node:fs";
import * as path from "node:path";
import * as vscode from "vscode";
import type { PreviewApi, RenderRecord } from "../../../src/preview/controller.js";
import { EXTENSION_ID, activated, sleep, uriOf, waitFor, workspace } from "./helpers.js";

// The preview panel with the real `ascribe lsp` on a copy of
// examples/quill: the webview loads for real, so a render is "drawn" when the
// webview says it has put the page in its document.
describe("the preview, with the real language server on examples/quill", () => {
  const install = uriOf("docs", "install-agent.md");
  let preview: PreviewApi;

  const firstLine = (render: RenderRecord) => render.result.page?.html ?? "";
  const drawnFor = (predicate: (render: RenderRecord) => boolean, timeout?: number) =>
    preview.whenDrawn("a render that satisfies the test", predicate, timeout);

  before(async () => {
    const api = await activated();
    await api.whenSettled();
    assert.equal(api.state(), "running");
    preview = api.preview;
    await vscode.window.showTextDocument(await vscode.workspace.openTextDocument(install));
    await vscode.commands.executeCommand("ascribe.openPreview");
  });

  after(async () => {
    await vscode.commands.executeCommand("workbench.action.closeAllEditors");
  });

  it("renders the Quill page with its tabs, notes, steps, and badges", async () => {
    assert.ok(preview.isOpen());
    const render = await drawnFor((r) => r.result.page?.path === "install-agent.md");
    const report = render.report;
    assert.ok(report);
    assert.equal(report.elementsDefined, true, "the element library registered its elements");
    assert.ok((report.elements["ascribe-tabs"] ?? 0) >= 2, "tabs for the pm and deployment groups");
    assert.ok((report.elements["ascribe-note"] ?? 0) >= 2);
    assert.equal(report.elements["ascribe-steps"], 1);
    assert.ok((report.elements["ascribe-availability"] ?? 0) >= 1);
    assert.ok(report.headings.includes("prerequisites"));
    assert.ok(report.headings.includes("streaming-sync"));
    assert.deepEqual(report.violations, [], "nothing was refused by the content security policy");
    assert.equal(render.result.build, "site");
    assert.equal(preview.build(), "site");
  });

  it("shows the fragment's image, found beside the fragment", async () => {
    const render = await drawnFor((r) => r.images !== undefined && r.result.page !== null);
    assert.ok(render.images);
    assert.equal(render.images.length, 1, "the page has one image, and it comes from a fragment");
    const [image] = render.images;
    assert.equal(image?.loaded, true, `the image at ${image?.src} loaded`);
    assert.ok((image?.width ?? 0) > 0);
    // Its URL is the webview URL of `_fragments/prerequisites.png`, not a path beside the page.
    assert.match(decodeURIComponent(image?.src ?? ""), /_fragments\/prerequisites\.png/);
    assert.ok(render.assets.some((a) => a.reference === "./_fragments/prerequisites.png"));
  });

  it("may read only the extension's webview files and the content root", () => {
    const extensionPath = vscode.extensions.getExtension(EXTENSION_ID)?.extensionPath ?? "";
    const roots = preview.localResourceRoots().map((root) => path.resolve(root));
    assert.deepEqual(roots, [
      path.resolve(extensionPath, "dist", "webview"),
      path.resolve(workspace(), "docs"),
    ]);
    const shell = preview.shell() ?? "";
    assert.match(shell, /Content-Security-Policy/);
    assert.match(shell, /default-src 'none'/);
    assert.doesNotMatch(shell, /unsafe-inline|unsafe-eval|nonce-/);
  });

  it("follows edits within half a second, including unsaved ones", async () => {
    const editor = await vscode.window.showTextDocument(
      await vscode.workspace.openTextDocument(install),
    );
    await drawnFor((r) => r.result.documentVersion === editor.document.version);
    const times: number[] = [];
    const end = editor.document.lineAt(editor.document.lineCount - 1).range.end;
    for (let n = 1; n <= 12; n++) {
      const marker = `MARKER${n}`;
      const started = Date.now();
      await editor.edit((edit) => edit.insert(end, `\n\nAn edit: ${marker}.\n`));
      const render = await drawnFor((r) => firstLine(r).includes(marker), 5_000);
      assert.ok(render.drawnAt !== undefined);
      times.push(render.drawnAt - started);
      // Wait for the render that came from this edit to be the last one, so
      // the next edit starts from quiet, as a person typing in bursts does.
      await sleep(150);
    }
    const sorted = [...times].sort((a, b) => a - b);
    const median = sorted[Math.floor(sorted.length / 2)] ?? 0;
    console.log(`edit to drawn, ms: ${times.join(" ")} (median ${median}, max ${sorted.at(-1)})`);
    // "Within about half a second": the typical edit. One edit
    // may be slower when the machine is busy (a full run starts four VS Code
    // windows in turn), so the slowest has a looser bound that still catches
    // a real regression.
    assert.ok(median < 500, `the median edit was drawn within 500 ms: ${times.join(" ")}`);
    assert.ok(
      (sorted.at(-1) ?? 0) < 1_500,
      `every edit was drawn within 1.5 s: ${times.join(" ")}`,
    );
    // The file on disk hasn't changed: the preview showed the buffer.
    assert.equal(editor.document.isDirty, true);
  });

  it("switches builds from the picker", async () => {
    assert.match(
      firstLine(await drawnFor((r) => r.result.build === "site")),
      /Point the agent at your server/,
    );
    // What the webview posts when the reader picks a build.
    await preview.receive({ type: "build", name: "cloud" });
    const cloud = await drawnFor((r) => r.result.build === "cloud");
    assert.ok(!firstLine(cloud).includes("Point the agent at your server"));
    assert.match(firstLine(cloud), /Sign in to Quill Cloud/);
    assert.equal(preview.build(), "cloud");
    assert.deepEqual(
      cloud.result.builds.map((b) => b.name),
      ["site", "cloud", "self-managed-3.3"],
    );

    await preview.receive({ type: "build", name: "self-managed-3.3" });
    const sm = await drawnFor((r) => r.result.build === "self-managed-3.3");
    assert.ok(!firstLine(sm).includes("Streaming sync"));

    // Back to the editor's build: the picker's default.
    preview.selectBuild("site");
    const site = await drawnFor(
      (r) => r.result.build === "site" && r.seq > sm.seq && firstLine(r).includes("Streaming sync"),
    );
    assert.match(firstLine(site), /Point the agent at your server/);
  });

  it("opens the file a link names when it is clicked", async () => {
    await preview.receive({ type: "open", href: "/keys/#rotate-keys" });
    await waitFor(
      "keys.md to open",
      () => vscode.window.activeTextEditor?.document.uri.fsPath === uriOf("docs", "keys.md").fsPath,
    );
    // The preview follows the editor to the page that is now active.
    const render = await drawnFor((r) => r.result.page?.path === "keys.md");
    assert.equal(render.result.page?.title, "API keys");

    // A link the preview can't open safely does nothing.
    await preview.receive({ type: "open", href: "javascript:alert(1)" });
    await preview.receive({ type: "open", href: "command:workbench.action.closeAllEditors" });
    await sleep(200);
    assert.equal(
      vscode.window.activeTextEditor?.document.uri.fsPath,
      uriOf("docs", "keys.md").fsPath,
    );
  });

  it("says why a fragment has no preview", async () => {
    await vscode.window.showTextDocument(
      await vscode.workspace.openTextDocument(uriOf("docs", "_fragments", "prerequisites.md")),
    );
    const render = await drawnFor((r) => r.result.page === null && r.result.problems.length > 0);
    assert.match(render.result.problems[0]?.message ?? "", /is a fragment.*install-agent\.md/);
  });

  it("scrolls with the editor by block, both ways", async () => {
    const editor = await vscode.window.showTextDocument(
      await vscode.workspace.openTextDocument(install),
    );
    const render = await drawnFor(
      (r) => r.result.page?.path === "install-agent.md" && r.seq === preview.renders().at(-1)?.seq,
    );
    assert.ok((render.report?.anchored ?? 0) > 20, "the page's blocks carry anchors");
    const lines = editor.document.getText().split("\n");
    const lineOf = (start: string) => lines.findIndex((text) => text.startsWith(start));

    // The editor scrolls: the preview shows the block at its top. The line
    // VS Code reports at the top can sit a little above the one revealed (sticky
    // scroll keeps the heading in view), so the block is the one for that line:
    // in the streaming sync section, from its heading to the paragraph.
    const heading = lineOf("## Streaming sync");
    const paragraph = lineOf("Streaming sync pushes changes");
    editor.revealRange(
      new vscode.Range(paragraph, 0, paragraph, 0),
      vscode.TextEditorRevealType.AtTop,
    );
    const top = () => editor.visibleRanges[0]?.start.line;
    const scrolled = await waitFor("a scroll to the streaming sync section", () => {
      const line = top();
      return line !== undefined && line >= heading && line <= paragraph
        ? preview.lineReveals().find((r) => r.line === line)
        : undefined;
    }).catch((error: unknown) => {
      throw new Error(
        `${String(error)}; editor top ${top()}, reveals ${JSON.stringify(preview.lineReveals())}`,
      );
    });
    const [, first, last] = /^install-agent\.md:(\d+)-(\d+)$/.exec(scrolled.source) ?? [];
    assert.ok(
      Number(first) >= heading + 1 && Number(last) <= paragraph + 1,
      `the block at the top is in the streaming sync section: ${scrolled.source}`,
    );

    // The editor shows an include: the preview shows the first block it brought in.
    const include = lineOf("@include: _fragments/prerequisites.md");
    editor.selection = new vscode.Selection(include, 0, include, 0);
    editor.revealRange(new vscode.Range(include, 0, include, 0), vscode.TextEditorRevealType.AtTop);
    await waitFor("a scroll to the fragment's first block", () =>
      preview.lineReveals().some((r) => r.source === "_fragments/prerequisites.md:1-1"),
    );

    // A double-click on a block in the preview puts the cursor on its line.
    const steps = lineOf("@steps");
    await preview.receive({ type: "openLine", line: steps });
    assert.equal(vscode.window.activeTextEditor?.selection.active.line, steps);

    // The preview scrolls: the editor follows.
    const troubleshooting = lineOf("## Troubleshooting");
    await preview.receive({ type: "scrolled", line: troubleshooting });
    await waitFor("the editor at the troubleshooting heading", () =>
      vscode.window.visibleTextEditors.some(
        (e) =>
          e.document.uri.fsPath === install.fsPath &&
          e.visibleRanges.some((r) => r.start.line === troubleshooting),
      ),
    );
  });

  it("shows an image from a directory beside the content root, and serves only that directory", async () => {
    mkdirSync(uriOf("shared").fsPath, { recursive: true });
    copyFileSync(uriOf("docs", "playground.png").fsPath, uriOf("shared", "logo.png").fsPath);
    const page = uriOf("docs", "shared-image.md");
    writeFileSync(
      page.fsPath,
      "---\ntitle: Shared\n---\n\n## Logo\n@id: logo\n\n![The shared logo](../shared/logo.png)\n",
    );
    await vscode.window.showTextDocument(await vscode.workspace.openTextDocument(page));
    const render = await drawnFor(
      (r) => r.result.page?.path === "shared-image.md" && r.images !== undefined,
    );
    assert.deepEqual(render.result.assetRoots, [uriOf("shared").fsPath]);
    assert.equal(render.result.problems.length, 0);
    assert.equal(
      render.images?.[0]?.loaded,
      true,
      `the image at ${render.images?.[0]?.src} loaded`,
    );
    const extensionPath = vscode.extensions.getExtension(EXTENSION_ID)?.extensionPath ?? "";
    assert.deepEqual(
      preview.localResourceRoots().map((root) => path.resolve(root)),
      [
        path.resolve(extensionPath, "dist", "webview"),
        path.resolve(workspace(), "docs"),
        path.resolve(workspace(), "shared"),
      ],
      "the directory of the asset, not the project root",
    );
  });
});
