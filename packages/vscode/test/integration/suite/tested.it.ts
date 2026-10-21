import * as assert from "node:assert/strict";
import * as vscode from "vscode";
import type { AscribeApi } from "../../../src/extension.js";
import { activated, uriOf, waitFor } from "./helpers.js";

// The real `ascribe lsp` on a copy of examples/tested: the lens in a code
// file a page shows, which says how many pages show it and lists them.
describe("a code file a page shows", () => {
  let api: AscribeApi;

  before(async () => {
    api = await activated();
    await api.whenSettled();
    await waitFor("the server", () => api.state() === "running");
  });

  afterEach(async () => {
    await vscode.commands.executeCommand("workbench.action.closeAllEditors");
  });

  /** The lenses VS Code shows in a file, once there are some. */
  function lenses(uri: vscode.Uri): Promise<vscode.CodeLens[]> {
    return waitFor(`the lenses in ${uri.path}`, async () => {
      const found = await vscode.commands.executeCommand<vscode.CodeLens[]>(
        "vscode.executeCodeLensProvider",
        uri,
        10,
      );
      return found.length > 0 && found;
    });
  }

  it("says over the region that one page shows it, and lists that page", async () => {
    const greet = uriOf("code", "examples", "greet.ts");
    await vscode.window.showTextDocument(greet);
    const [lens, ...rest] = await lenses(greet);
    assert.ok(lens?.command);
    assert.equal(rest.length, 0);
    assert.equal(lens.range.start.line, 0);
    assert.equal(lens.command.title, "Shown in 1 page");
    assert.equal(lens.command.command, "ascribe.showReferences");
    const [, , places] = lens.command.arguments as [
      string,
      unknown,
      { uri: string; range: { start: { line: number } } }[],
    ];
    assert.deepEqual(
      places.map((place) => [vscode.Uri.parse(place.uri).toString(), place.range.start.line]),
      [[uriOf("docs", "index.md").toString(), 6]],
    );

    // Choosing it shows the place in the references peek; choosing that
    // opens the page at its `@snippet`, as the peek opens any location.
    await vscode.commands.executeCommand(lens.command.command, ...(lens.command.arguments ?? []));
    await vscode.commands.executeCommand("closeReferenceSearch");
    const [place] = places;
    assert.ok(place);
    const page = await vscode.window.showTextDocument(vscode.Uri.parse(place.uri), {
      selection: new vscode.Range(place.range.start.line, 0, place.range.start.line, 0),
    });
    assert.ok(page.document.lineAt(page.selection.start.line).text.startsWith("@snippet:"));
  });

  it("puts a lens on the first line of a file a page shows whole", async () => {
    const output = uriOf("code", "examples", "greet.out.txt");
    await vscode.window.showTextDocument(output);
    const [lens] = await lenses(output);
    assert.equal(lens?.range.start.line, 0);
    assert.equal(lens?.command?.title, "Shown in 1 page");
  });

  it("asks the server nothing for a code file outside every source", async () => {
    const greet = uriOf("code", "examples", "greet.ts");
    const folders = await waitFor("the lens registration", async () => {
      const found = await api.lensesFor(greet);
      return found.length > 0 && found;
    });
    assert.equal(folders.length, 1);
    // In the project's folder, but in no source and not a page.
    const scratch = uriOf("notes", "scratch.ts");
    await vscode.workspace.fs.writeFile(scratch, new TextEncoder().encode("// nothing\n"));
    try {
      assert.deepEqual(await api.lensesFor(scratch), []);
    } finally {
      await vscode.workspace.fs.delete(vscode.Uri.joinPath(scratch, ".."), { recursive: true });
    }
  });

  it("lists the page's tested examples in This page, and the files its snippets show", async () => {
    await vscode.commands.executeCommand("ascribe.thisPage.focus");
    await vscode.window.showTextDocument(uriOf("docs", "index.md"));
    const items = await waitFor("This page's Health", async () => {
      await api.ui.thisPage.whenSettled();
      const shown = api.ui.thisPage.items();
      return shown.some((item) => item.label === "Health") && shown;
    });
    const health = items.find((item) => item.label === "Health");
    const greet = health?.children.find(
      (item) => item.label === "Example" && item.description.startsWith("code:examples/greet.ts"),
    );
    assert.ok(greet, JSON.stringify(health));
    // It leads with its state, then what it was tested with.
    assert.equal(greet.state, "tested", JSON.stringify(greet));
    assert.ok(greet.description.includes(", with Node"), greet.description);
    // It opens the file its `@snippet` shows, beside the page.
    assert.ok(greet.opens?.endsWith("/code/examples/greet.ts"), greet.opens);
    const includes = items
      .find((item) => item.label === "Connections")
      ?.children.find((item) => item.label === "Includes");
    const shown = includes?.children.find((item) => item.description === "code/examples/greet.ts");
    assert.ok(shown?.opens?.endsWith("/code/examples/greet.ts"), JSON.stringify(includes));
  });

  it("opens a snippet's file and a tested example's beside the page, with This page still on it", async () => {
    const view = api.ui.thisPage;
    const page = uriOf("docs", "index.md");
    await vscode.commands.executeCommand("workbench.action.closeAllEditors");
    await vscode.commands.executeCommand("ascribe.thisPage.focus");
    await vscode.window.showTextDocument(page, { viewColumn: vscode.ViewColumn.One });
    const drawn = await view.whenDrawn("index.md", (drawn) =>
      drawn.report.sections.some((section) => section.key === "health"),
    );
    const items = view.items();
    const health = items.find((item) => item.label === "Health")?.children ?? [];
    const example = health.findIndex(
      (item) => item.label === "Example" && item.description.startsWith("code:examples/greet.ts"),
    );
    const includes = items.find((item) => item.label === "Connections")?.children[2]?.children;
    const snippet = (includes ?? []).findIndex(
      (item) => item.description === "code/examples/greet.out.txt",
    );
    assert.ok(example >= 0 && snippet >= 0, JSON.stringify(items));
    const title = drawn.report.sections.find((section) => section.key === "about")?.items[0];

    for (const [key, file] of [
      [`connections/2/${snippet}`, uriOf("code", "examples", "greet.out.txt")],
      [`health/${example}`, uriOf("code", "examples", "greet.ts")],
    ] as const) {
      // The reader clicks in the view, and chooses the row with the keyboard.
      await vscode.commands.executeCommand("ascribe.thisPage.focus");
      await view.probe({ open: true, focus: key, press: true });
      const beside = await waitFor(`${file.path} beside the page`, async () =>
        vscode.window.visibleTextEditors.find(
          (editor) => editor.document.uri.toString() === file.toString(),
        ),
      );
      assert.equal(beside.viewColumn, vscode.ViewColumn.Two);
      await view.whenSettled();
      const after = await view.probe({});
      assert.equal(
        after.report.sections.find((section) => section.key === "about")?.items[0],
        title,
      );
      assert.equal(after.report.focused, key);
      assert.equal(after.report.hasFocus, true);
    }
  });
});
