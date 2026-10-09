import * as assert from "node:assert/strict";
import { readFileSync, rmSync, writeFileSync } from "node:fs";
import * as vscode from "vscode";
import type { SidebarItem } from "../../../src/ui/sidebarViews.js";
import { activated, diagnosticsOf, uriOf, waitFor } from "./helpers.js";

// The sidebar's views run first, while the project's pages are the example's own.
describe("the sidebar's Used by, Pages, and Content model views on examples/quill", () => {
  const keys = uriOf("docs", "keys.md");
  const quickstart = uriOf("docs", "quickstart.md");

  /** A view's items as `label (description)`, with their children, depth first. */
  function flat(items: SidebarItem[], depth = 0): string[] {
    return items.flatMap((item) => [
      `${"  ".repeat(depth)}${item.label}${item.description ? ` (${item.description})` : ""}`,
      ...flat(item.children, depth + 1),
    ]);
  }

  async function settled(): Promise<ReturnType<typeof flat>> {
    const api = await activated();
    await api.ui.sidebar.whenSettled();
    return flat(api.ui.sidebar.items("usedBy"));
  }

  /** Opens a page with the cursor at the start of the line that starts with `line`. */
  async function openAt(uri: vscode.Uri, line?: string): Promise<void> {
    const editor = await vscode.window.showTextDocument(
      await vscode.workspace.openTextDocument(uri),
    );
    const at = line === undefined ? 0 : editor.document.getText().split("\n").indexOf(line);
    assert.ok(at >= 0, `no line ${line}`);
    editor.selection = new vscode.Selection(at, 0, at, 0);
  }

  after(async () => {
    await vscode.commands.executeCommand("workbench.action.closeAllEditors");
  });

  it("lists the project's pages by type, its fragments, and what includes each", async () => {
    const api = await activated();
    await openAt(keys);
    const expected = [
      "page (4)",
      "  Broken (broken.md)",
      "  Install the Quill agent (install-agent.md)",
      "  API keys (keys.md)",
      "  Try Quill in the browser (quickstart.md)",
      "Fragments (1)",
      "  _fragments/prerequisites.md (1 file)",
      "    install-agent.md (includes it)",
      "Orphans (1)",
      "  Broken (broken.md)",
    ];
    // These run before the tests that add and rename pages: a page that's
    // still open in the editor stays in the project after it's deleted.
    const pages = await waitFor("the Pages view", async () => {
      await api.ui.sidebar.whenSettled();
      const items = flat(api.ui.sidebar.items("pages"));
      return items.join("\n") === expected.join("\n") ? items : undefined;
    }).catch(() => flat(api.ui.sidebar.items("pages")));
    assert.deepEqual(pages, expected);
    const page = api.ui.sidebar.items("pages")[0]?.children[2];
    assert.ok(page?.opens?.endsWith("/docs/keys.md"), page?.opens);
  });

  it("lists the content model with each entry's uses, and opens its declaration", async () => {
    const api = await activated();
    await api.ui.sidebar.whenSettled();
    const items = api.ui.sidebar.items("model");
    const model = flat(items);
    assert.ok(model.includes("Phrases (4)"), model.join("\n"));
    assert.ok(
      model.some((line) => /^  product \(\d+ uses\)$/.test(line)),
      model.join("\n"),
    );
    assert.ok(model.includes("  api (1 use)"), model.join("\n"));
    assert.ok(model.includes("Dimensions (2)"), model.join("\n"));
    assert.ok(model.includes("Builds (3)"), model.join("\n"));
    const product = items[0]?.children.find((item) => item.label === "product");
    assert.match(product?.opens ?? "", /ascribe\.toml:\d+$/);
  });

  it("follows the active page and narrows to the heading at the cursor", async () => {
    // Used by asks only while it's showing.
    await vscode.commands.executeCommand("ascribe.usedBy.focus");
    await openAt(keys);
    assert.deepEqual(
      await waitFor("Used by for keys.md", async () => {
        const items = await settled();
        return items.length > 0 ? items : undefined;
      }),
      [
        "Linked from (1)",
        "  install-agent.md (docs/install-agent.md)",
        "    A common cause is an expired API key. Generate a new key, then restart the agent. See [](keys.md#rotate-keys). (line 93)",
      ],
    );
    const api = await activated();
    /** Waits for Used by's message, which says what it answered. */
    const answered = (message: string) =>
      waitFor(message, async () => {
        const items = await settled();
        return api.ui.sidebar.message("usedBy") === message ? items : undefined;
      });
    await openAt(keys, "## Rotate keys");
    assert.equal((await answered("What links to “Rotate keys”")).length, 3);
    await openAt(keys, "## Create a key");
    assert.deepEqual(await answered("Nothing links to or includes “Create a key”."), []);
  });

  it("counts again when a page of the project is saved", async () => {
    const api = await activated();
    const original = readFileSync(quickstart.fsPath, "utf8");
    const incoming = (): string | undefined =>
      flat(api.ui.sidebar.items("pages")).find((line) => line.includes("(keys.md)"));
    try {
      await openAt(quickstart);
      const document = await vscode.workspace.openTextDocument(quickstart);
      const edit = new vscode.WorkspaceEdit();
      edit.insert(quickstart, document.positionAt(original.length), "\nSee [keys](keys.md).\n");
      assert.ok(await vscode.workspace.applyEdit(edit));
      assert.ok(await document.save());
      await waitFor("the new link counted", async () => {
        await api.ui.sidebar.whenSettled();
        const page = api.ui.sidebar
          .items("pages")[0]
          ?.children.find((p) => p.description === "keys.md");
        return page?.tooltip.includes("2 links and includes") ? page : undefined;
      });
      assert.ok(incoming());
    } finally {
      writeFileSync(quickstart.fsPath, original);
      await vscode.commands.executeCommand("workbench.action.revertAndCloseActiveEditor");
    }
  });
});

// The real `ascribe lsp` on a copy of examples/quill with a broken
// page, `docs/broken.md`, added (its unknown attribute key is a §8.2 error).
describe("with the real language server on examples/quill", () => {
  const broken = uriOf("docs", "broken.md");
  const original = () => readFileSync(broken.fsPath, "utf8");

  /**
   * Writes `text` to the broken page on disk until its diagnostics match,
   * writing it again every two seconds. The server's file watcher is
   * registered after it starts, and a write that lands before VS Code's
   * watcher is live is never reported: on a busy machine that can take
   * seconds, and a single write then waits forever.
   */
  async function writeUntil(
    text: string,
    predicate: (all: vscode.Diagnostic[]) => boolean,
  ): Promise<vscode.Diagnostic[]> {
    const deadline = Date.now() + 30_000;
    for (;;) {
      writeFileSync(broken.fsPath, text);
      try {
        return await diagnosticsOf(broken, predicate, 2_000);
      } catch (error) {
        if (Date.now() > deadline) throw error;
      }
    }
  }

  it("starts `ascribe lsp` from the ascribe.path setting", async () => {
    const api = await activated();
    await api.whenSettled();
    assert.equal(api.state(), "running");
    assert.equal(api.binary()?.source, "setting");
  });

  it("delivers diagnostics for a page that isn't open", async () => {
    const diagnostics = await diagnosticsOf(broken, (all) => all.length > 0);
    assert.ok(diagnostics.every((diagnostic) => diagnostic.severity !== undefined));
    assert.ok(diagnostics.some((diagnostic) => diagnostic.range.start.line === 7));
  });

  it("reports nothing for the pages of the example that are correct", async () => {
    await diagnosticsOf(broken, (all) => all.length > 0);
    for (const page of ["quickstart.md", "install-agent.md", "keys.md"]) {
      assert.deepEqual(vscode.languages.getDiagnostics(uriOf("docs", page)), [], page);
    }
  });

  it("updates diagnostics after the file changes on disk", async () => {
    const text = original();
    try {
      await writeUntil(text.replace("{colour=red}", "{type=tip}"), (all) => all.length === 0);
      const diagnostics = await writeUntil(
        text.replace("{colour=red}", "{colour=blue, size=big}"),
        (all) => all.length >= 2,
      );
      assert.ok(diagnostics.length >= 2);
    } finally {
      writeFileSync(broken.fsPath, text);
    }
  });

  it("updates diagnostics for an open document as it's edited", async () => {
    const editor = await vscode.window.showTextDocument(
      await vscode.workspace.openTextDocument(broken),
    );
    await diagnosticsOf(broken, (all) => all.length > 0);
    const line = editor.document.lineAt(7);
    await editor.edit((edit) => edit.replace(line.range, "@note {type=tip}: Now it is fine."));
    await diagnosticsOf(broken, (all) => all.length === 0);
  });

  it("formats through ascribe-fmt on save when enabled", async () => {
    const file = uriOf("docs", "format-on-save.md");
    const input = "@note{type = tip}: Save safely.\n";
    writeFileSync(file.fsPath, "placeholder\n");
    const config = vscode.workspace.getConfiguration("ascribe");
    const previous = config.get("formatOnSave", false);
    await config.update("formatOnSave", true, vscode.ConfigurationTarget.Workspace);
    try {
      const document = await vscode.workspace.openTextDocument(file);
      const editor = await vscode.window.showTextDocument(document);
      await editor.edit((edit) =>
        edit.replace(
          new vscode.Range(document.positionAt(0), document.positionAt(document.getText().length)),
          input,
        ),
      );
      await document.save();
      assert.equal(readFileSync(file.fsPath, "utf8"), "@note {type=tip}: Save safely.\n");
    } finally {
      await config.update("formatOnSave", previous, vscode.ConfigurationTarget.Workspace);
      rmSync(file.fsPath, { force: true });
    }
  });

  it("updates file references before a workspace rename", async () => {
    const target = uriOf("docs", "keys.md");
    const moved = uriOf("docs", "guides", "keys.md");
    const linking = uriOf("docs", "install-agent.md");
    const originalTarget = readFileSync(target.fsPath, "utf8");
    const originalLinking = readFileSync(linking.fsPath, "utf8");
    try {
      await vscode.workspace.openTextDocument(target);
      await vscode.workspace.openTextDocument(linking);
      const edit = new vscode.WorkspaceEdit();
      edit.renameFile(target, moved);
      assert.equal(await vscode.workspace.applyEdit(edit), true);
      assert.ok(
        (await vscode.workspace.openTextDocument(linking))
          .getText()
          .includes("See [](guides/keys.md#rotate-keys)."),
      );
      const reverse = new vscode.WorkspaceEdit();
      reverse.renameFile(moved, target);
      assert.equal(await vscode.workspace.applyEdit(reverse), true);
      assert.equal((await vscode.workspace.openTextDocument(linking)).getText(), originalLinking);
    } finally {
      rmSync(moved.fsPath, { force: true });
      writeFileSync(target.fsPath, originalTarget);
      writeFileSync(linking.fsPath, originalLinking);
    }
  });
});
