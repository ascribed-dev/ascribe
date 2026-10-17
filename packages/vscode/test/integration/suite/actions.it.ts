import * as assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { rmSync } from "node:fs";
import * as vscode from "vscode";
import type { RunRecord } from "../../../src/actions/run.js";
import type { Scripted } from "../../../src/actions/steps.js";
import type { AscribeApi } from "../../../src/extension.js";
import { activated, diagnosticsOf, uriOf, waitFor, workspace } from "./helpers.js";

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

  /** Opens the actions bar and gives the rows it lists, once they're in. */
  async function openBar(): Promise<string[]> {
    const before = api.actions.bars.length;
    await vscode.commands.executeCommand("ascribe.actions");
    const record = api.actions.bars[before];
    assert.ok(record, "the bar didn't open");
    return record.rows;
  }

  /** Chooses a row of the open bar, as Enter on it would. */
  async function choose(label: string): Promise<void> {
    assert.ok(api.actions.selectInBar(label), `the bar has no row ${label}`);
    await vscode.commands.executeCommand("workbench.action.acceptSelectedQuickOpenItem");
  }

  it("is in a project in a page of it", async () => {
    await open("keys.md", 9, 0);
    await waitFor("ascribe.inProject", () => api.actions.inProject());
  });

  it("wraps a list item's paragraph in a note, indented to the item, as one undo step", async () => {
    // "   The command prints the installed version, {version}." in step 2.
    const editor = await open("install-agent.md", 44, 10);
    const before = editor.document.getText();
    const record = await run("wrapNote", ["tip"]);
    assert.deepEqual(record, { action: "wrapNote", edits: 1, done: true, messages: [] });
    assert.deepEqual(lines(editor).slice(44, 46), [
      "   @note {type=tip}",
      "   The command prints the installed version, {version}.",
    ]);
    await vscode.commands.executeCommand("undo");
    assert.equal(editor.document.getText(), before);
  });

  it("inserts a note and leaves its placeholder selected for typing", async () => {
    // The blank line between "Sign in to {cloud} …" and "## Rotate keys".
    const editor = await open("keys.md", 10, 0);
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
    const editor = await open("install-agent.md", 11, 4);
    const record = await run("setNoteType", [
      async () => {
        // An edit while the wizard is open: the context's version is stale.
        const end = editor.document.lineAt(editor.document.lineCount - 1).range.end;
        assert.ok(await editor.edit((e) => e.insert(end, "\nOne more paragraph.\n")));
        return "warning";
      },
    ]);
    assert.deepEqual(record, { action: "setNoteType", edits: 2, done: true, messages: [] });
    assert.equal(lines(editor)[10], "@note {type=warning}");
    assert.ok(editor.document.getText().includes("One more paragraph."));
  });

  it("copies a link to the section the cursor is in, from the content root", async () => {
    await open("keys.md", 11, 5);
    await vscode.env.clipboard.writeText("");
    const record = await run("copyLinkToSection");
    assert.equal(record.done, true, record.messages.join());
    assert.equal(await vscode.env.clipboard.readText(), "/keys.md#rotate-keys");
  });

  it("links the selected text to the page picked", async () => {
    // "the playground" in "Open the playground at play.quill.dev …".
    const editor = await open("quickstart.md", 9, 5, 9, 19);
    const record = await run("linkSelection", ["keys.md"]);
    assert.equal(record.done, true, record.messages.join());
    assert.equal(
      lines(editor)[9],
      "Open [the playground](keys.md) at play.quill.dev and paste a page of your docs.",
    );
  });

  it("inserts the phrase picked from the project's", async () => {
    // After "Sign" in "Sign in to {cloud} and open …".
    const editor = await open("keys.md", 9, 4);
    const record = await run("insertPhrase", ["product"]);
    assert.equal(record.done, true, record.messages.join());
    assert.ok(lines(editor)[9]?.startsWith("Sign{product} in to {cloud}"), lines(editor)[9]);
  });

  it("says where an action applies, and changes nothing, where it doesn't", async () => {
    const editor = await open("quickstart.md", 9, 4);
    const record = await run("unwrapNote");
    assert.deepEqual(record, {
      action: "unwrapNote",
      edits: 0,
      done: false,
      messages: ["Put the cursor in a note to take its text out of it."],
    });
    assert.equal(editor.document.isDirty, false);
  });

  describe("the actions bar", () => {
    afterEach(async () => {
      await vscode.commands.executeCommand("workbench.action.closeQuickOpen");
    });

    it("lists the actions for a note, by group", async () => {
      // In the tip's paragraph, "You can run {product} in the browser …".
      await open("install-agent.md", 11, 4);
      assert.deepEqual(await openBar(), [
        "-- Write",
        "Insert a phrase",
        "-- Structure",
        "Wrap in a note",
        "Change the note's kind",
        "Remove the note, keeping its text",
        "Turn the note into collapsible details",
        "Wrap in collapsible details",
        "Mark where it's available",
        "Make the page one variant",
        "Set where the page is available",
        "-- Link",
        "Insert a link",
        "-- Content model",
        "Change a feature's availability",
        "Rename this phrase everywhere",
        "Rename a dimension value everywhere",
        "-- Memos",
        "Add a memo",
        "Add a personal memo",
      ]);
    });

    it("lists the actions for a heading", async () => {
      // In "## Rotate keys", which has an @id.
      await open("keys.md", 11, 5);
      assert.deepEqual(await openBar(), [
        "-- Write",
        "Insert a phrase",
        "-- Structure",
        "Mark where it's available",
        "Make the page one variant",
        "Set where the page is available",
        "-- Link",
        "Copy a link to this section",
        "Insert a link",
        "-- Content model",
        "Change a feature's availability",
        "Rename this phrase everywhere",
        "Rename a dimension value everywhere",
        "-- Memos",
        "Add a memo",
        "Add a personal memo",
      ]);
    });

    it("lists the actions for a selection of prose", async () => {
      // "the playground" in "Open the playground at play.quill.dev …".
      await open("quickstart.md", 9, 5, 9, 19);
      assert.deepEqual(await openBar(), [
        "-- Structure",
        "Wrap in a note",
        "Wrap in collapsible details",
        "Mark where it's available",
        "Make the page one variant",
        "Set where the page is available",
        "-- Link",
        "Link the selected text",
        "-- Content model",
        "Make this a phrase",
        "Add to the glossary",
        "Change a feature's availability",
        "Rename this phrase everywhere",
        "Rename a dimension value everywhere",
        "-- Memos",
        "Add a memo",
        "Add a personal memo",
      ]);
    });

    it("lists the inserts for a blank line", async () => {
      // The blank line between "Sign in to {cloud} …" and "## Rotate keys".
      await open("keys.md", 10, 0);
      assert.deepEqual(await openBar(), [
        "-- Write",
        "Insert a note",
        "Insert steps",
        "Insert collapsible details",
        "-- Structure",
        "Insert content that varies",
        "Make the page one variant",
        "Set where the page is available",
        "-- Media",
        "Insert an image",
        "Include a fragment",
        "Insert a code snippet",
        "Insert a widget",
        "-- Content model",
        "Change a feature's availability",
        "Rename this phrase everywhere",
        "Rename a dimension value everywhere",
        "-- Memos",
        "Add a memo",
        "Add a personal memo",
      ]);
    });

    it("lists the fixes for a problem at the cursor first", async () => {
      const editor = await open("keys.md", 9, 0);
      const end = editor.document.lineAt(editor.document.lineCount - 1).range.end;
      assert.ok(await editor.edit((e) => e.insert(end, "\nA {flush} phrase.\n")));
      const [problem] = await diagnosticsOf(editor.document.uri, (all) =>
        all.some((d) => d.message.includes("flush")),
      );
      assert.ok(problem);
      const inside = problem.range.start.translate(0, 2);
      editor.selection = new vscode.Selection(inside, inside);
      const rows = await openBar();
      assert.equal(rows[0], "-- Fix");
      assert.deepEqual(rows.slice(1, rows.indexOf("-- Structure")).sort(), [
        "Declare phrase in ascribe.toml",
        "Escape this phrase as literal text",
      ]);
      assert.equal(rows.filter((row) => row === "Mark where it's available").length, 1);
    });

    it("runs an action without a wizard", async () => {
      const editor = await open("install-agent.md", 11, 4);
      await openBar();
      const before = api.actions.runs.length;
      await choose("Remove the note, keeping its text");
      const record = await waitFor("the action", () => api.actions.runs[before]);
      assert.equal(record.done, true, record.messages.join());
      const text = editor.document.getText();
      assert.ok(!text.includes("@note {type=tip}"), text);
      assert.ok(text.includes("\nYou can run {product} in the browser"), text);
    });

    it("asks an action's questions in the same quick input", async () => {
      const editor = await open("quickstart.md", 9, 4);
      await openBar();
      const before = api.actions.runs.length;
      const bar = api.actions.bars.at(-1);
      await choose("Wrap in a note");
      // The wizard's first step took the bar's place before the bar closed.
      await waitFor("the wizard's first step", () => bar?.handedOver);
      await vscode.commands.executeCommand("workbench.action.acceptSelectedQuickOpenItem");
      const record = await waitFor("the action", () => api.actions.runs[before]);
      assert.equal(record.done, true, record.messages.join());
      assert.match(lines(editor)[9] ?? "", /^@note( \{type=[a-z-]+\})?$/);
      assert.equal(
        lines(editor)[10],
        "Open the playground at play.quill.dev and paste a page of your docs.",
      );
    });
  });

  describe("the content model", () => {
    /**
     * Runs an action's command with answers for its wizard and, once VS
     * Code's refactor preview lists the changes, discards them there.
     */
    async function runAndDiscard(id: string, answers: Scripted[]): Promise<RunRecord> {
      const before = api.actions.runs.length;
      api.actions.answerNext(answers);
      const running = vscode.commands.executeCommand(`ascribe.action.${id}`);
      // Until the preview is open, the command does nothing.
      const record = await waitFor("the refactor preview", async () => {
        await vscode.commands
          .executeCommand("refactorPreview.discard")
          .then(undefined, () => undefined);
        return api.actions.runs[before];
      });
      await running;
      return record;
    }

    it("makes the selected text a phrase, in ascribe.toml and the page", async () => {
      // "the agent" in "… and restart the agent. To install …".
      const editor = await open("keys.md", 20, 0);
      const line = lines(editor)[20] ?? "";
      const start = line.indexOf("the agent.");
      editor.selection = new vscode.Selection(20, start, 20, start + "the agent".length);
      // A refactoring saves the files it changes: put them back afterwards.
      const files = [uriOf("docs", "keys.md"), uriOf("ascribe.toml")];
      const saved = await Promise.all(files.map((uri) => vscode.workspace.fs.readFile(uri)));
      const record = await run("makePhrase", ["agent", "no"]);
      assert.equal(record.done, true, record.messages.join());
      assert.ok(lines(editor)[20]?.includes("and restart {agent}. To install the agent"));
      const model = await vscode.workspace.openTextDocument(uriOf("ascribe.toml"));
      assert.ok(
        model.getText().includes('\napi = "https://api.quill.dev/v3/"\nagent = "the agent"\n'),
      );
      await vscode.commands.executeCommand("undo");
      assert.equal(lines(editor)[20], line);
      for (const [i, uri] of files.entries()) {
        const bytes = saved[i];
        if (bytes) await vscode.workspace.fs.writeFile(uri, bytes);
      }
    });

    it("renames a phrase picked from the palette, far from its key, asking first", async () => {
      // A paragraph of the quickstart, with no phrase at the cursor.
      await open("quickstart.md", 9, 4);
      const record = await runAndDiscard("renamePhrase", ["cloud", "hosted"]);
      // The preview listed the key in ascribe.toml and the pages that use it.
      assert.equal(record.previewed, true, record.messages.join());
      assert.deepEqual(record.files, ["ascribe.toml", "docs/install-agent.md", "docs/keys.md"]);
    });

    it("changes nothing when the rename's preview is discarded", async () => {
      // In "{cloud}" of "Sign in to {cloud} and open …".
      await open("keys.md", 9, 13);
      const record = await runAndDiscard("renamePhrase", ["hosted"]);
      assert.equal(record.previewed, true);
      assert.equal(record.done, false);
      for (const document of vscode.workspace.textDocuments) {
        assert.equal(document.isDirty, false, document.uri.toString());
      }
    });
  });

  describe("the page's safe fixes", () => {
    /** keys.md with two problems a safe fix fixes, and one only an unsafe one does. */
    async function withProblems(): Promise<vscode.TextEditor> {
      const editor = await open("keys.md", 9, 0);
      const end = editor.document.lineAt(editor.document.lineCount - 1).range.end;
      assert.ok(
        await editor.edit((e) =>
          e.insert(
            end,
            "\nSee [Install](/install-agent/).\n\n@memo:\n@end\n\nSee [Gone](keyz.md).\n",
          ),
        ),
      );
      await diagnosticsOf(editor.document.uri, (all) => {
        const has = (text: string) => all.some((d) => d.message.includes(text));
        return has("`/install-agent.md`") && has("this memo is empty");
      });
      return editor;
    }

    function assertFixed(text: string): void {
      assert.ok(text.includes("See [Install](/install-agent.md)."), text);
      assert.ok(!text.includes("@memo:\n@end"), text);
      // An unsafe fix is left for the writer.
      assert.ok(text.includes("See [Gone](keyz.md)."), text);
    }

    it("applies them with Source Action, and leaves the rest", async () => {
      const editor = await withProblems();
      await vscode.commands.executeCommand("editor.action.sourceAction", {
        kind: "source.fixAll.ascribe",
        apply: "first",
      });
      await waitFor("the fixes", () => editor.document.getText().includes("/install-agent.md"));
      assertFixed(editor.document.getText());
    });

    it("applies them on save where editor.codeActionsOnSave asks", async () => {
      const settings = vscode.workspace.getConfiguration("editor", { languageId: "markdown" });
      await settings.update(
        "codeActionsOnSave",
        { "source.fixAll.ascribe": "explicit" },
        vscode.ConfigurationTarget.Workspace,
        true,
      );
      try {
        const editor = await withProblems();
        assert.ok(await editor.document.save());
        assertFixed(editor.document.getText());
      } finally {
        await settings.update(
          "codeActionsOnSave",
          undefined,
          vscode.ConfigurationTarget.Workspace,
          true,
        );
        // Back to the page as it was, on disk too.
        const document = await vscode.workspace.openTextDocument(uriOf("docs", "keys.md"));
        const text = document.getText();
        const at = text.indexOf("\nSee [Install]");
        if (at >= 0) {
          const edit = new vscode.WorkspaceEdit();
          edit.delete(
            document.uri,
            new vscode.Range(document.positionAt(at), document.positionAt(text.length)),
          );
          await vscode.workspace.applyEdit(edit);
          await document.save();
        }
      }
    });

    it("lists them first among the actions bar's fixes", async () => {
      await withProblems();
      const rows = await openBar();
      assert.deepEqual(rows.slice(0, 2), ["-- Fix", "Apply the safe fixes in this page"]);
      await vscode.commands.executeCommand("workbench.action.closeQuickOpen");
    });
  });

  it("opens the bar on the last page from the walkthrough, where no text editor is active", async () => {
    await open("keys.md", 9, 0);
    await vscode.commands.executeCommand(
      "workbench.action.openWalkthrough",
      "Ascribe.ascribe-vscode#start",
      false,
    );
    await waitFor("the walkthrough", () => vscode.window.activeTextEditor === undefined);
    const before = api.actions.bars.length;
    await vscode.commands.executeCommand("ascribe.walkthrough.actions");
    assert.ok(api.actions.bars[before], "the bar didn't open");
    assert.equal(
      vscode.window.activeTextEditor?.document.uri.toString(),
      uriOf("docs", "keys.md").toString(),
    );
    await vscode.commands.executeCommand("workbench.action.closeQuickOpen");
  });

  it("offers its rewrites in the lightbulb once the cursor's context is known", async () => {
    const editor = await open("quickstart.md", 9, 4);
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

  describe("memo files", () => {
    const made: string[] = [];
    afterEach(() => {
      for (const file of made.splice(0)) rmSync(uriOf(file).fsPath, { force: true });
    });

    /** The lens on a line of a page with `title`, once the server shows it. */
    async function lensAt(uri: vscode.Uri, line: number, title: string): Promise<vscode.CodeLens> {
      return waitFor(`a lens "${title}" on line ${line + 1}`, async () => {
        const lenses = await vscode.commands.executeCommand<vscode.CodeLens[]>(
          "vscode.executeCodeLensProvider",
          uri,
        );
        return lenses.find(
          (lens) => lens.range.start.line === line && lens.command?.title === title,
        );
      });
    }

    /** Runs a lens's command, and the file it shows. */
    async function follow(lens: vscode.CodeLens): Promise<string> {
      const command = lens.command;
      assert.ok(command);
      await vscode.commands.executeCommand(command.command, ...(command.arguments ?? []));
      return waitFor("the memo", () => {
        const shown = vscode.window.activeTextEditor?.document.uri;
        return shown?.path.includes("/memos/") && vscode.workspace.asRelativePath(shown, false);
      });
    }

    /** The memos `ascribe outline` lists on a page's heading. */
    function outlined(page: string, id: string): { file: string; state: string }[] {
      const binary = api.binary()?.path;
      assert.ok(binary);
      const json = execFileSync(binary, ["outline", "--format", "json", page], {
        cwd: workspace(),
        encoding: "utf8",
      });
      const outline = JSON.parse(json) as {
        headings: { id: string; memos: { file: string; state: string }[] }[];
      };
      return outline.headings.find((h) => h.id === id)?.memos ?? [];
    }

    it("counts the memos on a block in a lens that opens them", async () => {
      // memos/agent-first.md quotes "Install the agent package", in step 1.
      const editor = await open("install-agent.md", 0, 0);
      const lens = await lensAt(editor.document.uri, 22, "1 memo");
      assert.equal(await follow(lens), "memos/agent-first.md");
    });

    it("adds a memo from a paragraph, about its section, that outline lists as placed there", async () => {
      // "The command prints the installed version, {version}." in step 2.
      await open("install-agent.md", 44, 10);
      const record = await run("addMemo");
      assert.ok(record.file, JSON.stringify(record));
      made.push(record.file);
      assert.equal(record.file, "memos/the-command-prints-the-installed.md");
      const editor = vscode.window.activeTextEditor;
      assert.equal(editor?.document.isDirty, false);
      assert.equal(
        editor?.document.getText(),
        '---\nabout: install-agent.md#install-agent\nquote: "The command prints the installed version, {version}."\n---\n\n',
      );
      // At its text, to write it.
      assert.equal(editor?.selection.active.line, 5);
      assert.deepEqual(
        outlined("install-agent.md", "install-agent").map((m) => [m.file, m.state]),
        [
          ["memos/agent-first.md", "placed"],
          ["memos/the-command-prints-the-installed.md", "placed"],
        ],
      );
    });

    it("adds a memo about a heading's section, which its lens then opens", async () => {
      // "## Connect to {product}".
      const page = await open("install-agent.md", 57, 4);
      const record = await run("addPersonalMemo");
      assert.equal(record.file, ".ascribe/memos/connect.md");
      made.push(".ascribe/memos/connect.md");
      const lens = await lensAt(page.document.uri, 57, "1 memo");
      assert.equal(await follow(lens), ".ascribe/memos/connect.md");
    });

    it("rewrites a memo's about when F2 renames its heading's @id", async () => {
      // "@id: install-agent", under "## Install the agent".
      const page = await open("install-agent.md", 19, 6);
      const edit = await vscode.commands.executeCommand<vscode.WorkspaceEdit>(
        "vscode.executeDocumentRenameProvider",
        page.document.uri,
        new vscode.Position(19, 6),
        "install",
      );
      const memo = edit.get(uriOf("memos", "agent-first.md"));
      assert.deepEqual(
        memo.map((e) => e.newText),
        ["install-agent.md#install"],
      );
    });
  });
});
