import * as assert from "node:assert/strict";
import { existsSync, readFileSync } from "node:fs";
import * as vscode from "vscode";
import type { SidebarItem } from "../../../src/ui/sidebarViews.js";
import { activated, diagnosticsOf, uriOf, waitFor } from "./helpers.js";

// The real `ascribe lsp` on a copy of examples/astro-site, which has no
// ascribe.nav, with Reference/Limits.md added: the Pages view is the
// navigation, and dragging in it writes ascribe.nav. The tests run in order,
// each from where the one before left the file.
describe("the Pages view as the navigation of examples/astro-site", () => {
  const nav = uriOf("ascribe.nav");

  /** The view's sections as `Label`, each with its pages as `  path`, a page not placed marked. */
  function outline(items: SidebarItem[]): string[] {
    return items
      .filter((item) => item.label !== "Fragments")
      .flatMap((section) => [
        section.label,
        ...section.children.map((page) =>
          page.description === "not placed"
            ? `  ${page.label} (not placed)`
            : `  ${page.description}`,
        ),
      ]);
  }

  /** Waits for the view to show `expected`. */
  async function shows(description: string, expected: string[]): Promise<void> {
    const api = await activated();
    const seen = await waitFor(description, async () => {
      await api.ui.sidebar.whenSettled();
      const lines = outline(api.ui.sidebar.items("pages"));
      return lines.join("\n") === expected.join("\n") ? lines : undefined;
    }).catch(() => outline(api.ui.sidebar.items("pages")));
    assert.deepEqual(seen, expected);
  }

  const navDocument = (): vscode.TextDocument | undefined =>
    vscode.workspace.textDocuments.find((d) => d.uri.toString() === nav.toString());

  async function saveNav(): Promise<void> {
    const document = navDocument();
    assert.ok(document, "ascribe.nav isn't open");
    assert.ok(await document.save());
  }

  const start = ["Start", "  index.md"];
  const guides = ["Guides", "  Guides/My Setup.md"];

  after(async () => {
    await vscode.commands.executeCommand("workbench.action.closeAllEditors");
  });

  it("shows Start, then each section with its pages, before the project has ascribe.nav", async () => {
    const api = await activated();
    await vscode.window.showTextDocument(uriOf("content", "index.md"));
    await shows("the default navigation", [
      ...start,
      ...guides,
      "Reference",
      "  Reference/Options.md",
      "  Reference/Limits.md",
    ]);
    const fragments = api.ui.sidebar.items("pages").find((item) => item.label === "Fragments");
    assert.deepEqual(
      fragments?.children.map((folder) => folder.label),
      ["_fragments/"],
    );
  });

  it("writes ascribe.nav, unsaved, when a section is dragged, and undo takes the file away", async () => {
    const api = await activated();
    assert.equal(await api.ui.sidebar.drop("Reference/", "Guides/"), undefined);
    const document = await waitFor("ascribe.nav", () => navDocument());
    assert.ok(document.isDirty);
    assert.equal(
      document.getText(),
      'version = 1\n\n[[sections]]\nfolder = "Reference/"\n\n[[sections]]\nfolder = "Guides/"\n',
    );
    await shows("Reference above Guides", [
      ...start,
      "Reference",
      "  Reference/Options.md",
      "  Reference/Limits.md",
      ...guides,
    ]);
    await vscode.window.showTextDocument(document);
    await vscode.commands.executeCommand("undo");
    await waitFor("ascribe.nav taken away", () => !existsSync(nav.fsPath));
    await vscode.commands.executeCommand("workbench.action.revertAndCloseActiveEditor");
    await shows("the default order again", [
      ...start,
      ...guides,
      "Reference",
      "  Reference/Options.md",
      "  Reference/Limits.md",
    ]);
  });

  it("lists a section's pages as they stood, then the one moved, when a page is dragged", async () => {
    const api = await activated();
    assert.equal(
      await api.ui.sidebar.drop("Reference/Options.md", "Reference/Limits.md"),
      undefined,
    );
    const document = await waitFor("ascribe.nav", () => navDocument());
    assert.equal(
      document.getText(),
      'version = 1\n\n[[sections]]\nfolder = "Guides/"\n\n[[sections]]\nfolder = "Reference/"\npages = ["Limits.md", "Options.md"]\n',
    );
    await saveNav();
    await shows("Options below Limits", [
      ...start,
      ...guides,
      "Reference",
      "  Reference/Limits.md",
      "  Reference/Options.md",
    ]);
  });

  it("shows a page the file leaves out at the end of its section, and Place Here lists it", async () => {
    const api = await activated();
    await vscode.workspace.fs.writeFile(
      uriOf("content", "Reference", "Defaults.md"),
      Buffer.from(
        "---\ntype: page\ntitle: Defaults\n---\n\nWhat Loom starts with. See [the options](Options.md).\n",
      ),
    );
    await shows("Defaults not placed", [
      ...start,
      ...guides,
      "Reference",
      "  Reference/Limits.md",
      "  Reference/Options.md",
      "  Defaults (not placed)",
    ]);
    assert.equal(await api.ui.sidebar.command("placeHere", "Reference/Defaults.md"), undefined);
    await shows("Defaults placed", [
      ...start,
      ...guides,
      "Reference",
      "  Reference/Limits.md",
      "  Reference/Options.md",
      "  Reference/Defaults.md",
    ]);
    assert.match(
      navDocument()?.getText() ?? "",
      /pages = \["Limits.md", "Options.md", "Defaults.md"\]/,
    );
    await saveNav();
  });

  it("renames a section's label in ascribe.nav", async () => {
    const api = await activated();
    assert.equal(
      await api.ui.sidebar.command("renameLabel", "Reference/", "API reference"),
      undefined,
    );
    await shows("the new label", [
      ...start,
      ...guides,
      "API reference",
      "  Reference/Limits.md",
      "  Reference/Options.md",
      "  Reference/Defaults.md",
    ]);
    assert.match(navDocument()?.getText() ?? "", /label = "API reference"/);
    await saveNav();
  });

  it("renames a page's label, its nav-label, in its frontmatter", async () => {
    const api = await activated();
    const options = uriOf("content", "Reference", "Options.md");
    const labelOf = () =>
      api.ui.sidebar
        .items("pages")
        .flatMap((item) => item.children)
        .find((item) => item.description === "Reference/Options.md")?.label;
    assert.equal(
      await api.ui.sidebar.command("renameLabel", "Reference/Options.md", "`loom.yaml`"),
      undefined,
    );
    const document = await waitFor("Options.md changed", () =>
      vscode.workspace.textDocuments.find(
        (d) => d.uri.toString() === options.toString() && d.isDirty,
      ),
    );
    assert.match(document.getText(), /\nnav-label: "`loom.yaml`"\n/);
    assert.ok(await document.save());
    // Read as the title is, its code span as plain text.
    await waitFor("the page's new label", async () => {
      await api.ui.sidebar.whenSettled();
      return labelOf() === "loom.yaml";
    });
  });

  it("moves a page's file, asked first, when it's dropped on another section, and ascribe.nav follows", async () => {
    const api = await activated();
    assert.equal(await api.ui.sidebar.drop("Reference/Defaults.md", "Guides/", false), undefined);
    assert.ok(existsSync(uriOf("content", "Reference", "Defaults.md").fsPath), "moved unasked");
    assert.equal(await api.ui.sidebar.drop("Reference/Defaults.md", "Guides/", true), undefined);
    await waitFor("Defaults.md in Guides", () =>
      existsSync(uriOf("content", "Guides", "Defaults.md").fsPath),
    );
    await shows("Defaults in Guides", [
      ...start,
      "Guides",
      "  Guides/Defaults.md",
      "  Guides/My Setup.md",
      "API reference",
      "  Reference/Limits.md",
      "  Reference/Options.md",
    ]);
    assert.match(navDocument()?.getText() ?? "", /pages = \["Limits.md", "Options.md"\]/);
    await vscode.workspace.saveAll(false);
  });

  it("reveals the open page", async () => {
    const api = await activated();
    await vscode.commands.executeCommand("ascribe.pages.focus");
    await vscode.window.showTextDocument(uriOf("content", "Reference", "Limits.md"));
    await waitFor("Limits selected", () => api.ui.sidebar.selected() === "Reference/Limits.md");
    await vscode.window.showTextDocument(uriOf("content", "Guides", "My Setup.md"));
    await waitFor("My Setup selected", () => api.ui.sidebar.selected() === "Guides/My Setup.md");
  });

  it("leaves nothing for the checks to report in ascribe.nav", async () => {
    assert.ok(readFileSync(nav.fsPath, "utf8").includes('folder = "Reference/"'));
    await vscode.window.showTextDocument(nav);
    await diagnosticsOf(nav, (diagnostics) => diagnostics.length === 0);
  });

  it("checks and completes ascribe.nav as it's typed, its language TOML", async () => {
    await activated();
    const editor = await vscode.window.showTextDocument(nav);
    const document = editor.document;
    // No TOML extension runs here: the language is the one the extension gives the file.
    assert.equal(document.languageId, "toml");
    const listed = 'pages = ["Limits.md", "Options.md"]';
    const at = document.getText().indexOf(listed);
    assert.notEqual(at, -1, "Reference's pages aren't as the steps before left them");
    /** Types `text` over the `length` characters of Reference's `pages`. */
    async function type(text: string, length: number): Promise<void> {
      const range = new vscode.Range(document.positionAt(at), document.positionAt(at + length));
      assert.ok(await editor.edit((e) => e.replace(range, text)));
    }
    const misspelled = 'pages = ["Limits.md", "Optoins.md"]';
    await type(misspelled, listed.length);
    const problems = await diagnosticsOf(nav, (diagnostics) => diagnostics.length > 0);
    assert.deepEqual(
      problems.map((diagnostic) => {
        const code = diagnostic.code;
        return String(typeof code === "object" ? code.value : code);
      }),
      ["ASC231"],
      "nav-page-unknown",
    );
    // Inside a new, empty string: the section's pages not written yet.
    const started = 'pages = ["Limits.md", "Optoins.md", ""]';
    await type(started, misspelled.length);
    const list = await vscode.commands.executeCommand<vscode.CompletionList>(
      "vscode.executeCompletionItemProvider",
      nav,
      document.positionAt(at + started.length - 2),
    );
    assert.deepEqual(
      list.items.map((item) => (typeof item.label === "string" ? item.label : item.label.label)),
      ["Options.md"],
    );
    await vscode.commands.executeCommand("workbench.action.revertAndCloseActiveEditor");
  });
});
