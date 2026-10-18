import * as assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { readFileSync, rmSync, writeFileSync } from "node:fs";
import * as path from "node:path";
import * as vscode from "vscode";
import type { AscribeApi } from "../../../src/extension.js";
import { activated, diagnosticsOf, uriOf, waitFor, workspace } from "./helpers.js";

// A glossary's terms in the editor, with the real `ascribe lsp` on a copy of
// examples/monorepo/docs (Lantern), whose glossary page is
// content/reference/glossary.md: going from a term to its heading,
// completing `@term`, a term off a glossary page and its fix, and renaming a
// term with its heading's `@id`.
describe("a glossary's terms, with the real language server on the Lantern docs", () => {
  const glossary = uriOf("content", "reference", "glossary.md");
  const rollouts = uriOf("content", "guides", "rollouts.md");
  const gettingStarted = uriOf("content", "getting-started.md");
  let api: AscribeApi;

  async function open(uri: vscode.Uri): Promise<vscode.TextEditor> {
    return vscode.window.showTextDocument(await vscode.workspace.openTextDocument(uri));
  }

  /** The position of the first `needle` in the document, plus `extra` characters. */
  function positionOf(document: vscode.TextDocument, needle: string, extra = 0): vscode.Position {
    const at = document.getText().indexOf(needle);
    assert.ok(at >= 0, `${path.basename(document.uri.fsPath)} has no ${needle}`);
    return document.positionAt(at + extra);
  }

  /** Replaces the document's text in the editor, without saving. */
  async function replace(editor: vscode.TextEditor, text: string): Promise<void> {
    const document = editor.document;
    const whole = new vscode.Range(
      document.positionAt(0),
      document.positionAt(document.getText().length),
    );
    assert.ok(await editor.edit((e) => e.replace(whole, text)));
  }

  /** The labels completion offers at `position`. */
  async function completions(uri: vscode.Uri, position: vscode.Position): Promise<string[]> {
    const list = await vscode.commands.executeCommand<vscode.CompletionList>(
      "vscode.executeCompletionItemProvider",
      uri,
      position,
    );
    return list.items.map((item) =>
      typeof item.label === "string" ? item.label : item.label.label,
    );
  }

  const codes = (uri: vscode.Uri): string[] =>
    vscode.languages.getDiagnostics(uri).map((diagnostic) => {
      const code = diagnostic.code;
      return String(typeof code === "object" ? code.value : code);
    });

  before(async () => {
    api = await activated();
    await api.whenSettled();
    assert.equal(api.state(), "running");
  });

  after(async () => {
    await vscode.commands.executeCommand("workbench.action.closeAllEditors");
  });

  it("goes from a term in prose to its heading on the glossary page", async () => {
    const editor = await open(rollouts);
    // "A rollout turns a flag on…", the guide's first use of the term.
    const at = positionOf(editor.document, "A rollout turns", 4);
    const found = await waitFor("the term's definition", async () => {
      const locations = await vscode.commands.executeCommand<
        (vscode.Location | vscode.LocationLink)[]
      >("vscode.executeDefinitionProvider", rollouts, at);
      return locations.length > 0 && locations;
    });
    const [first] = found;
    assert.ok(first);
    const uri = "uri" in first ? first.uri : first.targetUri;
    const range = "range" in first ? first.range : first.targetRange;
    assert.equal(uri.toString(), glossary.toString());
    const page = await vscode.workspace.openTextDocument(glossary);
    assert.equal(page.lineAt(range.start.line).text, "## Rollout");
  });

  it("offers @term at the top of a section on the glossary page, and its match values", async () => {
    const editor = await open(glossary);
    const original = editor.document.getText();
    try {
      await replace(editor, `${original}\n## Flag key\n@te\n\nThe name code checks a flag by.\n`);
      const line = editor.document.lineCount - 4;
      const names = await completions(glossary, new vscode.Position(line, 3));
      assert.ok(names.includes("term"), names.join(", "));
      await replace(
        editor,
        `${original}\n## Flag key\n@term {match=}\n\nThe name code checks a flag by.\n`,
      );
      const values = await completions(glossary, new vscode.Position(line, 13));
      for (const value of ["first", "every", "marked"]) {
        assert.ok(values.includes(value), values.join(", "));
      }
    } finally {
      await replace(editor, original);
      await editor.document.save();
    }
    // Not on a page that isn't in `[glossary] pages`.
    const other = await open(gettingStarted);
    const text = other.document.getText();
    try {
      await replace(other, `${text}\n## Flag key\n@te\n`);
      const names = await completions(
        gettingStarted,
        new vscode.Position(other.document.lineCount - 2, 3),
      );
      assert.ok(names.includes("id"), names.join(", "));
      assert.ok(!names.includes("term"), names.join(", "));
    } finally {
      await replace(other, text);
      await other.document.save();
    }
  });

  it("underlines @term off the glossary pages, and lists the page to fix it", async () => {
    const config = path.join(workspace(), "ascribe.toml");
    const model = readFileSync(config, "utf8");
    const terms = uriOf("content", "guides", "terms.md");
    writeFileSync(
      terms.fsPath,
      "---\ntitle: Terms\ntype: guide\n---\n\nSee [the guide](rollouts.md).\n\n## Flag key\n\nThe name code checks a flag by.\n",
    );
    try {
      // The checks across pages run when a page is saved.
      const editor = await open(terms);
      await editor.edit((e) =>
        e.insert(positionOf(editor.document, "## Flag key\n", 12), "@term\n"),
      );
      assert.ok(await editor.document.save());
      const problem = (await diagnosticsOf(terms, () => codes(terms).includes("ASC242"))).find(
        (d) => (typeof d.code === "object" ? d.code.value : d.code) === "ASC242",
      );
      assert.ok(problem);
      const actions = await vscode.commands.executeCommand<vscode.CodeAction[]>(
        "vscode.executeCodeActionProvider",
        terms,
        problem.range,
      );
      const fix = actions.find((a) => a.title === "List `guides/terms.md` in `[glossary] pages`");
      assert.ok(fix?.edit, actions.map((a) => a.title).join(", "));
      assert.ok(await vscode.workspace.applyEdit(fix.edit));
      // The edit leaves `ascribe.toml` changed in the editor, to save.
      assert.ok(await (await vscode.workspace.openTextDocument(vscode.Uri.file(config))).save());
      assert.ok(readFileSync(config, "utf8").includes('"guides/terms.md"'));
      await diagnosticsOf(terms, () => !codes(terms).includes("ASC242"));
    } finally {
      await vscode.commands.executeCommand("workbench.action.closeAllEditors");
      writeFileSync(config, model);
      rmSync(terms.fsPath, { force: true });
    }
  });

  it("renames a term with F2 on its heading's @id, and the build links the new id", async () => {
    const page = await open(gettingStarted);
    const text = page.document.getText();
    try {
      await replace(page, `${text}\nRead about [rollouts](reference/glossary.md#rollout).\n`);
      await page.document.save();
      const editor = await open(glossary);
      const at = positionOf(editor.document, "@id: rollout", 6);
      const edit = await waitFor("the rename", async () => {
        const edit = await vscode.commands.executeCommand<vscode.WorkspaceEdit>(
          "vscode.executeDocumentRenameProvider",
          glossary,
          at,
          "rollouts-explained",
        );
        return edit.get(gettingStarted).length > 0 && edit;
      });
      const changed = edit.get(gettingStarted).map((e) => e.newText);
      assert.ok(
        changed.every((text) => text.includes("rollouts-explained")),
        changed.join(", "),
      );
      assert.ok(await vscode.workspace.applyEdit(edit));
      await vscode.workspace.saveAll();
      const binary = api.binary()?.path;
      assert.ok(binary);
      execFileSync(binary, ["build", "--build", "site", "--emit", "site"], { cwd: workspace() });
      const built = readFileSync(
        path.join(workspace(), ".ascribe", "build", "site", "site", "guides", "rollouts.md"),
        "utf8",
      );
      assert.ok(built.includes('data-ascribe-term="rollouts-explained"'), built);
      assert.ok(!built.includes('data-ascribe-term="rollout"'), built);
    } finally {
      const editor = await open(glossary);
      await replace(
        editor,
        editor.document.getText().replace("@id: rollouts-explained", "@id: rollout"),
      );
      await editor.document.save();
      const restored = await open(gettingStarted);
      await replace(restored, text);
      await restored.document.save();
    }
  });
});
