import * as assert from "node:assert/strict";
import { readFileSync, rmSync, writeFileSync } from "node:fs";
import * as path from "node:path";
import { shellWord } from "@ascribed/review/prompt";
import * as vscode from "vscode";
import { ascribeElsewhere } from "../../../src/agents/reach.js";
import { nodePathEnvironment } from "../../../src/environment.js";
import type { ItemRecord, SidebarItem } from "../../../src/ui/sidebarViews.js";
import type { Drawn } from "../../../src/ui/webviewView.js";
import type { ProjectReport, ThisPageReport } from "../../../src/webview/viewProtocol.js";
import { activated, diagnosticsOf, uriOf, waitFor } from "./helpers.js";

// The sidebar's views run first, while the project's pages are the example's own.
describe("the sidebar's This page, Pages, and Content model views on examples/quill", () => {
  const keys = uriOf("docs", "keys.md");
  const quickstart = uriOf("docs", "quickstart.md");
  const installAgent = uriOf("docs", "install-agent.md");

  /** A view's items as `label (description)`, with their children, depth first. */
  function flat(items: SidebarItem[], depth = 0): string[] {
    return items.flatMap((item) => [
      `${"  ".repeat(depth)}${item.label}${item.description ? ` (${item.description})` : ""}`,
      ...flat(item.children, depth + 1),
    ]);
  }

  /**
   * This page's items, without Health, whose Problems row counts what the
   * checks report and is held by its own test.
   */
  async function settled(): Promise<ReturnType<typeof flat>> {
    const api = await activated();
    await api.ui.thisPage.whenSettled();
    return flat(api.ui.thisPage.items().filter((item) => item.label !== "Health"));
  }

  /** Waits for This page to show what `wanted` accepts, and returns what it shows. */
  async function showing(what: string, wanted: (items: string[]) => boolean): Promise<string[]> {
    return waitFor(what, async () => {
      const items = await settled();
      return wanted(items) ? items : undefined;
    }).catch(() => settled());
  }

  /** Accepts exactly `expected`. */
  const exactly = (expected: string[]) => (items: string[]) =>
    items.join("\n") === expected.join("\n");

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

  it("lists the project's pages in the navigation's order, its fragments, and what includes each", async () => {
    const api = await activated();
    await openAt(keys);
    // Quill has no ascribe.nav and one type, so its pages are in title order.
    const expected = [
      "Start",
      "  API keys (keys.md)",
      "  Broken (broken.md)",
      "  Configure the agent (configure.md)",
      // The editor's build, which the status bar shows, publishes 3.5.
      "  Configure the agent (3.4) (Not in site)",
      "  Install the Quill agent (install-agent.md)",
      "  Try Quill in the browser (quickstart.md)",
      "  Upgrade the agent (upgrade.md)",
      "  Webhooks (webhooks.md)",
      "Fragments (1)",
      "  _fragments/ (1)",
      "    prerequisites.md (1 file)",
      "      install-agent.md (includes it)",
    ];
    // These run before the tests that add and rename pages: a page that's
    // still open in the editor stays in the project after it's deleted.
    const pages = await waitFor("the Pages view", async () => {
      await api.ui.sidebar.whenSettled();
      const items = flat(api.ui.sidebar.items("pages"));
      return items.join("\n") === expected.join("\n") ? items : undefined;
    }).catch(() => flat(api.ui.sidebar.items("pages")));
    assert.deepEqual(pages, expected);
    const page = api.ui.sidebar.items("pages")[0]?.children[0];
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
    assert.ok(model.includes("Builds (4)"), model.join("\n"));
    const phrases = items.find((item) => item.label === "Phrases");
    const product = phrases?.children.find((item) => item.label === "product");
    assert.match(product?.opens ?? "", /ascribe\.toml:\d+$/);
    // Page types come first: Quill's one, declared through its frontmatter's table.
    assert.equal(model[0], "Page types (1)");
    assert.match(model[1] ?? "", /^  page \(\d+ pages\)$/);
    assert.match(items[0]?.children[0]?.opens ?? "", /ascribe\.toml:\d+$/);
    // And each dimension's values, under it.
    assert.ok(model.includes("    cloud (Quill Cloud)"), model.join("\n"));
    assert.ok(model.includes("    self-managed"), model.join("\n"));
  });

  /** What This page shows for keys.md, with what links to `heading` (else the page). */
  function keysPage(connections: string[]): string[] {
    return [
      "About",
      "  Title (API keys)",
      "  Type (page)",
      "  Address (/keys/)",
      "  Markdown (not published)",
      "  Description (52 characters)",
      "  Builds (Published by site, site@3.4, cloud, self-managed-3.3)",
      "  Listing (Listed (the default))",
      ...connections,
    ];
  }

  const linkToRotate =
    "      A common cause is an expired API key. Generate a new key, then restart the agent. See [](keys.md#rotate-keys). (line 94)";

  it("says what the page is, and narrows its connections to the heading at the cursor", async () => {
    const api = await activated();
    // This page asks only while it's showing.
    await vscode.commands.executeCommand("ascribe.thisPage.focus");
    await openAt(
      keys,
      "Generate a new key, put it in `quill.yaml`, and restart the agent. To install the agent first, see [](install-agent.md#install-agent).",
    );
    const rotate = keysPage([
      "Connections (to “Rotate keys”)",
      "  Linked from (1)",
      "    install-agent.md (install-agent.md)",
      linkToRotate,
      "  Included by (0)",
      "  Includes (0)",
      "Memos (none)",
    ]);
    assert.deepEqual(await showing("This page under Rotate keys", exactly(rotate)), rotate);
    assert.ok(
      api.ui.thisPage
        .items()
        .find((item) => item.label === "Connections")
        ?.children[0]?.children[0]?.children[0]?.opens?.endsWith("/docs/install-agent.md:94"),
    );
    // A cursor move asks once.
    const asked = api.ui.thisPage.asked();
    await openAt(keys, "## Create a key");
    const create = keysPage([
      "Connections (to “Create a key”)",
      "  Linked from (0)",
      "  Included by (0)",
      "  Includes (0)",
      "Memos (none)",
    ]);
    assert.deepEqual(await showing("This page under Create a key", exactly(create)), create);
    assert.equal(api.ui.thisPage.asked(), asked + 1);
    // Above the first heading, the whole page.
    await openAt(keys);
    const whole = await showing("This page for keys.md", (items) =>
      items.includes("Connections (to the page)"),
    );
    assert.ok(whole.includes("  Linked from (4)"), whole.join("\n"));
  });

  it("says how the page stands in Health, and its rows go where they say", async () => {
    const api = await activated();
    await vscode.commands.executeCommand("ascribe.thisPage.focus");
    await openAt(keys);
    const health = await waitFor("Health", async () => {
      await api.ui.thisPage.whenSettled();
      return api.ui.thisPage.items().find((item) => item.label === "Health");
    });
    const rows = health.children.map((item) => `${item.label} (${item.description})`);
    // The same count the Problems panel has for the page.
    const problems = vscode.languages.getDiagnostics(keys);
    assert.match(
      rows[0] ?? "",
      /^Problems \((none|.*(error|warning|advice).*)\)$/,
      rows.join("\n"),
    );
    assert.equal(rows[0] === "Problems (none)", problems.length === 0, rows.join("\n"));
    assert.equal(health.children[0]?.runs, "workbench.action.problems.focus");
    assert.match(rows[1] ?? "", /^Size \([\d,]+ of 50,000 characters, in site\)$/, rows.join("\n"));
    const about = api.ui.thisPage.items().find((item) => item.label === "About");
    const row = (label: string) => about?.children.find((item) => item.label === label);
    assert.ok(row("Description")?.opens?.endsWith("/docs/keys.md:4"), row("Description")?.opens);
    assert.equal(row("Builds")?.runs, "ascribe.switchBuild");
  });

  it("counts an unsaved edit's link before it's saved", async () => {
    const original = readFileSync(installAgent.fsPath, "utf8");
    try {
      const document = await vscode.workspace.openTextDocument(installAgent);
      const edit = new vscode.WorkspaceEdit();
      edit.insert(
        installAgent,
        document.positionAt(original.length),
        "\nRotate them with [](keys.md#rotate-keys).\n",
      );
      assert.ok(await vscode.workspace.applyEdit(edit));
      await openAt(keys, "## Rotate keys");
      const items = await showing("two links to Rotate keys", (shown) =>
        shown.includes("  Linked from (2)"),
      );
      assert.ok(items.includes("  Linked from (2)"), items.join("\n"));
      assert.ok(document.isDirty);
    } finally {
      await vscode.window.showTextDocument(await vscode.workspace.openTextDocument(installAgent));
      await vscode.commands.executeCommand("workbench.action.revertAndCloseActiveEditor");
      writeFileSync(installAgent.fsPath, original);
    }
  });

  it("asks a file that isn't a page to be opened", async () => {
    const api = await activated();
    await vscode.window.showTextDocument(
      await vscode.workspace.openTextDocument(uriOf("ascribe.toml")),
    );
    await waitFor("the message", async () => {
      await api.ui.thisPage.whenSettled();
      return api.ui.thisPage.message() === "Open a page to see what's known about it.";
    });
    assert.deepEqual(api.ui.thisPage.items(), []);
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
        return page?.tooltip.includes("5 links and includes") ? page : undefined;
      });
      assert.ok(incoming());
    } finally {
      writeFileSync(quickstart.fsPath, original);
      await vscode.commands.executeCommand("workbench.action.revertAndCloseActiveEditor");
    }
  });
});

// This page is a webview: the thread through it, from the extension's answer
// to what its webview drew. Its webview posts what it drew, which these wait for.
describe("This page's webview on examples/quill", () => {
  const installAgent = uriOf("docs", "install-agent.md");
  const quickstart = uriOf("docs", "quickstart.md");

  /** The title About's first row gives, in what the webview drew last. */
  const title = (drawn: Drawn<ThisPageReport>): string | undefined =>
    drawn.report.sections.find((section) => section.key === "about")?.items[0];

  after(async () => {
    await vscode.commands.executeCommand("workbench.action.closeAllEditors");
  });

  it("shows the page with About open and its rows wrapped, stays on it when the view takes the focus, and comes back on another page", async () => {
    const api = await activated();
    const view = api.ui.thisPage;
    await vscode.commands.executeCommand("ascribe.thisPage.focus");
    await vscode.window.showTextDocument(await vscode.workspace.openTextDocument(installAgent));
    const first = await view.whenDrawn(
      "install-agent.md",
      (drawn) => title(drawn) === "Title (Install the Quill agent)",
    );
    assert.deepEqual(
      first.report.sections.map(({ label, open }) => `${label} ${open ? "open" : "closed"}`),
      ["About open", "Health closed", "Connections closed", "Memos closed"],
    );
    assert.equal(first.report.wraps, true);
    assert.deepEqual(first.report.violations, []);

    // Clicking in the view takes the focus from the editor: the view stays on the page.
    await vscode.commands.executeCommand("ascribe.thisPage.focus");
    await view.whenSettled();
    assert.equal(title(view.drawn() ?? first), "Title (Install the Quill agent)");
    // So does focus in a webview editor, the preview's, which leaves VS Code
    // reporting no active editor at all.
    await vscode.commands.executeCommand("ascribe.openPreview");
    await (await activated()).preview.whenDrawn("the preview", () => true);
    await vscode.commands.executeCommand("workbench.action.focusSecondEditorGroup");
    await waitFor("the focus in the preview", async () =>
      vscode.window.activeTextEditor === undefined ? true : undefined,
    );
    const asked = view.asked();
    await view.whenSettled();
    assert.equal(title(view.drawn() ?? first), "Title (Install the Quill agent)");
    assert.equal(view.drawn()?.report.message, null);
    // Escape in the view gives the focus back to the page's editor.
    await view.escape();
    assert.equal(vscode.window.activeTextEditor?.document.uri.toString(), installAgent.toString());

    // Collapsed, or its container hidden, it asks nothing.
    await vscode.commands.executeCommand("workbench.view.explorer");
    await waitFor("This page hidden", async () => (view.visible() ? undefined : true));
    const editor = await vscode.window.showTextDocument(
      await vscode.workspace.openTextDocument(quickstart),
    );
    editor.selection = new vscode.Selection(6, 0, 6, 0);
    await view.whenSettled();
    const hidden = view.asked();
    assert.ok(hidden <= asked + 1, `asked ${hidden - asked} times while hidden`);
    const before = view.drawn()?.seq ?? 0;

    // Shown again, its webview is made again, after the cursor moved: it draws
    // the other page, with About open, and never the page it showed before.
    await vscode.commands.executeCommand("ascribe.thisPage.focus");
    const again = await view.whenDrawn(
      "quickstart.md",
      (drawn) => drawn.seq > before && title(drawn) === "Title (Try Quill in the browser)",
    );
    assert.equal(again.report.sections[0]?.open, true);
    assert.ok(view.asked() > hidden);
  });

  it("stays on its page while a file that isn't a page covers its tab, and moves on once the page's tab is closed", async () => {
    const api = await activated();
    const view = api.ui.thisPage;
    await vscode.commands.executeCommand("workbench.action.closeAllEditors");
    await vscode.commands.executeCommand("ascribe.thisPage.focus");
    await vscode.window.showTextDocument(await vscode.workspace.openTextDocument(installAgent), {
      viewColumn: vscode.ViewColumn.One,
      preview: false,
    });
    await view.whenDrawn(
      "install-agent.md",
      (drawn) => title(drawn) === "Title (Install the Quill agent)",
    );

    // ascribe.toml in the page's group covers its tab: the page's editor
    // leaves the screen, and This page stays on it.
    const model = uriOf("ascribe.toml");
    await vscode.window.showTextDocument(await vscode.workspace.openTextDocument(model), {
      viewColumn: vscode.ViewColumn.One,
      preview: false,
    });
    await waitFor("the page's tab covered", async () =>
      vscode.window.visibleTextEditors.every(
        (editor) => editor.document.uri.toString() !== installAgent.toString(),
      )
        ? true
        : undefined,
    );
    await view.whenSettled();
    const covered = view.drawn();
    assert.ok(covered);
    assert.equal(covered.report.message, null);
    assert.equal(title(covered), "Title (Install the Quill agent)");
    // Escape brings the page's tab back to the top, and the focus to it.
    await view.escape();
    assert.equal(vscode.window.activeTextEditor?.document.uri.toString(), installAgent.toString());

    // Closing the page's tab, with ascribe.toml active, leaves it on no page.
    await vscode.window.showTextDocument(model, { viewColumn: vscode.ViewColumn.One });
    const tab = vscode.window.tabGroups.all
      .flatMap((group) => group.tabs)
      .find(
        (tab) =>
          tab.input instanceof vscode.TabInputText &&
          tab.input.uri.toString() === installAgent.toString(),
      );
    assert.ok(tab);
    await vscode.window.tabGroups.close(tab);
    await view.whenDrawn("no page", (drawn) => drawn.report.message !== null);
  });

  it("opens what a row names when it's chosen", async () => {
    const api = await activated();
    const view = api.ui.thisPage;
    await vscode.commands.executeCommand("ascribe.thisPage.focus");
    await vscode.window.showTextDocument(await vscode.workspace.openTextDocument(installAgent));
    await view.whenDrawn(
      "install-agent.md",
      (drawn) => title(drawn) === "Title (Install the Quill agent)",
    );
    // Its Description row goes to the field, in the page's frontmatter.
    const about = view.items().find((item) => item.label === "About");
    const at = about?.children.findIndex((item) => item.label === "Description") ?? -1;
    assert.ok(at >= 0);
    await vscode.commands.executeCommand("ascribe.thisPage.focus");
    view.choose(`about/${at}`);
    await waitFor("the description's line", async () => {
      const editor = vscode.window.activeTextEditor;
      return editor?.document.uri.toString() === installAgent.toString() &&
        editor.selection.start.line === 3
        ? true
        : undefined;
    });
    // A key no item of the answer has opens nothing.
    view.choose("about/99");
    await view.whenSettled();
    assert.equal(vscode.window.activeTextEditor?.document.uri.toString(), installAgent.toString());
  });

  it("opens an include beside the page, with the focus left in the view and the view on the page", async () => {
    const api = await activated();
    const view = api.ui.thisPage;
    await vscode.commands.executeCommand("workbench.action.closeAllEditors");
    await vscode.commands.executeCommand("ascribe.thisPage.focus");
    await vscode.window.showTextDocument(await vscode.workspace.openTextDocument(installAgent), {
      viewColumn: vscode.ViewColumn.One,
    });
    await view.whenDrawn(
      "install-agent.md",
      (drawn) => title(drawn) === "Title (Install the Quill agent)",
    );
    // Connections' third group, Includes, lists the page's one `@include`.
    const includes = view.items().find((item) => item.label === "Connections")?.children[2];
    assert.equal(includes?.children[0]?.label, "_fragments/prerequisites.md");

    // The reader clicks in the view, and chooses the include with the keyboard.
    await vscode.commands.executeCommand("ascribe.thisPage.focus");
    const pressed = await view.probe({ open: true, focus: "connections/2/0", press: true });
    assert.equal(pressed.report.focused, "connections/2/0");
    const fragment = uriOf("docs", "_fragments", "prerequisites.md");
    const beside = await waitFor("the include beside the page", async () =>
      vscode.window.visibleTextEditors.find(
        (editor) => editor.document.uri.toString() === fragment.toString(),
      ),
    );
    assert.equal(beside.viewColumn, vscode.ViewColumn.Two);
    assert.ok(
      vscode.window.visibleTextEditors.some(
        (editor) =>
          editor.document.uri.toString() === installAgent.toString() &&
          editor.viewColumn === vscode.ViewColumn.One,
      ),
    );
    // The view stays on the page, and the focus where the choice left it.
    await view.whenSettled();
    const after = await view.probe({});
    assert.equal(title(after), "Title (Install the Quill agent)");
    assert.equal(after.report.focused, "connections/2/0");
    assert.equal(after.report.hasFocus, true);
    assert.notEqual(vscode.window.activeTextEditor?.document.uri.toString(), fragment.toString());
  });

  it("keeps its scroll and the keyboard's focus as the cursor moves", async () => {
    const api = await activated();
    const view = api.ui.thisPage;
    const keys = uriOf("docs", "keys.md");
    await vscode.commands.executeCommand("workbench.action.closeAllEditors");
    await vscode.commands.executeCommand("ascribe.thisPage.focus");
    const editor = await vscode.window.showTextDocument(
      await vscode.workspace.openTextDocument(keys),
    );
    const lines = editor.document.getText().split("\n");
    const at = (heading: string) => {
      const line = lines.indexOf(heading);
      assert.ok(line >= 0, `no ${heading}`);
      return new vscode.Selection(line + 1, 0, line + 1, 0);
    };
    editor.selection = at("## Rotate keys");
    await view.whenDrawn("keys.md under Rotate keys", (drawn) =>
      (drawn.report.sections.find((section) => section.key === "connections")?.items ?? []).some(
        (item) => item.startsWith("install-agent.md"),
      ),
    );

    // Every section open, the focus on About's Description, and the view scrolled.
    await vscode.commands.executeCommand("ascribe.thisPage.focus");
    const placed = await view.probe({ open: true, focus: "about/4", scroll: 40 });
    assert.equal(placed.report.focused, "about/4");
    assert.ok(placed.report.scroll > 0, "the view doesn't scroll");

    // The cursor moves to another heading: Connections changes, and the
    // reader's place in the view doesn't.
    const before = view.drawn()?.seq ?? 0;
    editor.selection = at("## Create a key");
    const moved = await view.whenDrawn(
      "keys.md under Create a key",
      (drawn) =>
        drawn.seq > before &&
        !(drawn.report.sections.find((section) => section.key === "connections")?.items ?? []).some(
          (item) => item.startsWith("install-agent.md"),
        ),
    );
    assert.equal(moved.report.focused, "about/4");
    assert.equal(moved.report.scroll, placed.report.scroll);
  });
});

// Pages' and Content model's menus and filters, run through the views' test
// API with scripted answers, against the real `ascribe lsp`.
describe("the menus and filters of Pages and Content model on examples/quill", () => {
  const keys = uriOf("docs", "keys.md");
  const linking = uriOf("docs", "install-agent.md");

  /** A view's items as `label (description)`, with their children, depth first. */
  function flat(items: SidebarItem[], depth = 0): string[] {
    return items.flatMap((item) => [
      `${"  ".repeat(depth)}${item.label}${item.description ? ` (${item.description})` : ""}`,
      ...flat(item.children, depth + 1),
    ]);
  }

  /** Every item of a view, depth first. */
  function all(items: SidebarItem[]): SidebarItem[] {
    return items.flatMap((item) => [item, ...all(item.children)]);
  }

  /** Opens a page with the cursor at `line` and `character`. */
  async function openAt(uri: vscode.Uri, line = 0, character = 0): Promise<vscode.TextEditor> {
    const editor = await vscode.window.showTextDocument(
      await vscode.workspace.openTextDocument(uri),
    );
    editor.selection = new vscode.Selection(line, character, line, character);
    return editor;
  }

  /** Waits for a view's lines to pass `wanted`, and returns them. */
  async function listed(
    view: "pages" | "model",
    what: string,
    wanted: (lines: string[]) => boolean,
  ) {
    const api = await activated();
    return waitFor(what, async () => {
      await api.ui.sidebar.whenSettled();
      const lines = flat(api.ui.sidebar.items(view));
      return wanted(lines) ? lines : undefined;
    }).catch(() => flat(api.ui.sidebar.items(view)));
  }

  before(async () => {
    await openAt(keys);
    await listed("pages", "the Pages view", (lines) => lines.includes("  API keys (keys.md)"));
  });

  after(async () => {
    await vscode.commands.executeCommand("workbench.action.closeAllEditors");
  });

  it("gives each page its file, so its problems color it as in the Explorer", async () => {
    const api = await activated();
    const broken = uriOf("docs", "broken.md");
    await diagnosticsOf(broken, (list) =>
      list.some((d) => d.severity === vscode.DiagnosticSeverity.Error),
    );
    const items = all(api.ui.sidebar.items("pages"));
    const item = items.find((i) => i.description === "broken.md");
    assert.equal(item?.resource, broken.toString());
    assert.equal(items.find((i) => i.description === "keys.md")?.resource, keys.toString());
  });

  it("dims the pages the status bar's build leaves out, and follows it", async () => {
    const api = await activated();
    await openAt(keys);
    try {
      api.ui.statusBar.answerNext("cloud");
      await vscode.commands.executeCommand("ascribe.switchBuild");
      const lines = await listed("pages", "the pages cloud leaves out dimmed", (l) =>
        l.includes("  Upgrade the agent (Not in cloud)"),
      );
      assert.ok(lines.includes("  Upgrade the agent (Not in cloud)"), lines.join("\n"));
      assert.ok(lines.includes("  Configure the agent (3.4) (Not in cloud)"), lines.join("\n"));
      assert.ok(lines.includes("  API keys (keys.md)"), lines.join("\n"));
      const upgrade = all(api.ui.sidebar.items("pages")).find(
        (i) => i.description === "Not in cloud",
      );
      assert.equal(upgrade?.dimmed, true);
    } finally {
      api.ui.statusBar.answerNext("site");
      await vscode.commands.executeCommand("ascribe.switchBuild");
    }
    const lines = await listed("pages", "nothing dimmed in site", (l) =>
      l.every((line) => !line.includes("Not in")),
    );
    assert.ok(lines.includes("  Upgrade the agent (upgrade.md)"), lines.join("\n"));
  });

  it("lists the pages and fragments nothing links to or includes with Orphans Only", async () => {
    const api = await activated();
    const everything = flat(api.ui.sidebar.items("pages"));
    await api.ui.sidebar.filter("pages", true);
    try {
      const lines = await listed(
        "pages",
        "the orphans",
        (l) => !l.includes("  API keys (keys.md)"),
      );
      // The broken page the suite adds is linked from nowhere; keys.md is.
      assert.ok(lines.includes("  Broken (broken.md)"), lines.join("\n"));
      assert.ok(!lines.includes("  API keys (keys.md)"), lines.join("\n"));
      for (const line of lines) assert.ok(everything.includes(line), line);
    } finally {
      await api.ui.sidebar.filter("pages", false);
    }
    assert.deepEqual(flat(api.ui.sidebar.items("pages")), everything);
  });

  it("copies a page's link, and finds its references at its start", async () => {
    const api = await activated();
    const copied = await api.ui.sidebar.act("copyLink", "keys.md");
    assert.equal(copied.copied, "/keys.md");
    const page = all(api.ui.sidebar.items("pages")).find((i) => i.description === "keys.md");
    const incoming = Number(/(\d+) links? and includes?/.exec(page?.tooltip ?? "")?.[1]);
    const found = await api.ui.sidebar.act("findReferences", "keys.md");
    assert.equal(found.done, true, found.messages.join());
    assert.equal(found.references?.length, incoming, found.references?.join("\n"));
    assert.ok(found.references?.includes("docs/install-agent.md:" + lineOf(linking, "keys.md#")));
    assert.equal(vscode.window.activeTextEditor?.document.uri.toString(), keys.toString());
  });

  it("renames or moves a page, and the links to it follow", async () => {
    const api = await activated();
    const moved = uriOf("docs", "guides", "keys.md");
    const originalKeys = readFileSync(keys.fsPath, "utf8");
    const originalLinking = readFileSync(linking.fsPath, "utf8");
    try {
      await vscode.workspace.openTextDocument(linking);
      const there = await api.ui.sidebar.act("renameOrMove", "keys.md", ["guides/keys.md"]);
      assert.equal(there.done, true, there.messages.join());
      assert.ok(
        (await vscode.workspace.openTextDocument(linking))
          .getText()
          .includes("See [](guides/keys.md#rotate-keys)."),
      );
      await listed("pages", "the moved page", (l) =>
        l.some((line) => line.includes("guides/keys.md")),
      );
      const back = await api.ui.sidebar.act("renameOrMove", "guides/keys.md", ["keys.md"]);
      assert.equal(back.done, true, back.messages.join());
      assert.equal((await vscode.workspace.openTextDocument(linking)).getText(), originalLinking);
    } finally {
      rmSync(moved.fsPath, { force: true });
      writeFileSync(keys.fsPath, originalKeys);
      writeFileSync(linking.fsPath, originalLinking);
      await vscode.commands.executeCommand("workbench.action.closeAllEditors");
    }
  });

  it("starts a page in a section's folder with New Page Here", async () => {
    const api = await activated();
    const made = uriOf("docs", "scratch-page.md");
    try {
      api.newPage.answerNext(["page", "Scratch page"]);
      const before = api.newPage.runs.length;
      const record = await api.ui.sidebar.act("newPageHere", "");
      assert.equal(record.done, true);
      const run = await waitFor("New Page", () => api.newPage.runs[before]);
      assert.equal(run.file, "docs/scratch-page.md", run.messages.join());
    } finally {
      await vscode.commands.executeCommand("workbench.action.closeAllEditors");
      rmSync(made.fsPath, { force: true });
    }
  });

  it("finds an entry's references from its declaration, as many as its uses", async () => {
    const api = await activated();
    const product = all(api.ui.sidebar.items("model")).find((i) => i.label === "product");
    const uses = Number(/^(\d+) uses?$/.exec(product?.description ?? "")?.[1]);
    const found = await api.ui.sidebar.act("findReferences", "phrase:product");
    assert.equal(found.done, true, found.messages.join());
    // Its declaration, then each use.
    assert.equal(found.references?.length, uses + 1, found.references?.join("\n"));
    assert.match(found.references?.[0] ?? "", /^ascribe\.toml:\d+$/);
  });

  it("finds a page type's pages from its declaration, at each page's type", async () => {
    const api = await activated();
    const found = await api.ui.sidebar.act("findReferences", "type:page");
    assert.equal(found.done, true, found.messages.join());
    const shown = found.references ?? [];
    // Its declaration, then each page at its `type` line. Other tests add
    // and remove pages, so this checks the example's own, not a count.
    assert.match(shown[0] ?? "", /^ascribe\.toml:\d+$/);
    for (const page of ["configure.md", "keys.md", "quickstart.md", "webhooks.md"]) {
      const at = `docs/${page}:${lineOf(uriOf("docs", page), "type:")}`;
      assert.ok(shown.includes(at), `${at} in\n${shown.join("\n")}`);
    }
  });

  it("lists only what nothing uses with Unused Only", async () => {
    const api = await activated();
    await api.ui.sidebar.filter("model", true);
    try {
      const lines = await listed("model", "the unused entries", (l) => !l.includes("Builds (4)"));
      for (const line of lines.filter((l) => /^  \S/.test(l))) {
        assert.match(line, /\((unused|no pages)\)$/, lines.join("\n"));
      }
    } finally {
      await api.ui.sidebar.filter("model", false);
    }
  });

  /** Runs a command that renames, and discards its refactor preview once it lists the changes. */
  async function discarding(run: () => Promise<ItemRecord>): Promise<ItemRecord> {
    const running = run();
    let done: ItemRecord | undefined;
    void running.then((record) => (done = record));
    await waitFor("the refactor preview", async () => {
      await vscode.commands
        .executeCommand("refactorPreview.discard")
        .then(undefined, () => undefined);
      return done;
    });
    return running;
  }

  it("renames a phrase as F2 on it in a page does, asking first", async () => {
    const api = await activated();
    // F2 on {cloud} in "Sign in to {cloud} and open …", as the action runs it.
    await openAt(keys, 9, 13);
    const before = api.actions.runs.length;
    api.actions.answerNext(["hosted"]);
    const f2 = await discarding(async () => {
      await vscode.commands.executeCommand("ascribe.action.renamePhrase");
      const run = api.actions.runs[before];
      return { messages: run?.messages ?? [], done: run?.done ?? false, files: run?.files ?? [] };
    });
    const record = await discarding(() => api.ui.sidebar.act("rename", "phrase:cloud", ["hosted"]));
    assert.equal(record.previewed, true, record.messages.join());
    assert.deepEqual(record.files, f2.files);
    assert.deepEqual(record.files, ["ascribe.toml", "docs/install-agent.md", "docs/keys.md"]);
    for (const document of vscode.workspace.textDocuments) {
      assert.equal(document.isDirty, false, document.uri.toString());
    }
  });

  it("renames a dimension's value where it's declared", async () => {
    const api = await activated();
    const record = await discarding(() =>
      api.ui.sidebar.act("rename", "value:deployment=cloud", ["hosted"]),
    );
    assert.equal(record.previewed, true, record.messages.join());
    assert.ok(record.files?.includes("ascribe.toml"), record.files?.join());
  });

  it("inserts a phrase at the cursor of the last page, as Insert a phrase does", async () => {
    const api = await activated();
    // After "Sign in to " on line 10.
    const editor = await openAt(keys, 9, "Sign in to ".length);
    const line = editor.document.lineAt(9).text;
    await vscode.commands.executeCommand("ascribe.model.focus");
    const record = await api.ui.sidebar.act("insert", "phrase:product");
    assert.equal(record.done, true, [...record.messages, ...(record.run?.messages ?? [])].join());
    const inserted = editor.document.lineAt(9).text;
    assert.equal(vscode.window.activeTextEditor?.document, editor.document);
    await vscode.commands.executeCommand("undo");
    assert.equal(editor.document.lineAt(9).text, line);
    // The action, at the same place, writes the same.
    editor.selection = new vscode.Selection(9, "Sign in to ".length, 9, "Sign in to ".length);
    const before = api.actions.runs.length;
    api.actions.answerNext(["product"]);
    await vscode.commands.executeCommand("ascribe.action.insertPhrase");
    const run = await waitFor("Insert a phrase", () => api.actions.runs[before]);
    assert.equal(run.done, true, run.messages.join());
    assert.equal(editor.document.lineAt(9).text, inserted);
    assert.ok(inserted.includes("{product}"), inserted);
    await vscode.commands.executeCommand("undo");
    await editor.document.save();
  });

  it("shows the hint and changes nothing where the cursor can't take a phrase", async () => {
    const api = await activated();
    // A blank line between blocks.
    const editor = await openAt(keys, 8, 0);
    await vscode.commands.executeCommand("ascribe.model.focus");
    const record = await api.ui.sidebar.act("insert", "phrase:product");
    assert.equal(record.done, false);
    assert.deepEqual(record.messages, [
      "Put the cursor in a paragraph's text, with nothing selected, to insert a phrase.",
    ]);
    assert.equal(editor.document.isDirty, false);
  });
});

/** The line, from 1, of the first line of a file that has `text`. */
function lineOf(uri: vscode.Uri, text: string): number {
  return (
    readFileSync(uri.fsPath, "utf8")
      .split("\n")
      .findIndex((line) => line.includes(text)) + 1
  );
}

// The Project view's counts, each shown as current only while it is: every
// build's problems, examples, overdue reviews, and the permalinks list from
// the last Check Every Build, while nothing in the project has changed since
// (decision #654a); orphans and unused entries from the server's inventory,
// as you type; and the agent files, after each save.
describe("the Project view on examples/quill", () => {
  const keys = uriOf("docs", "keys.md");
  const memo = uriOf("memos", "product-name.md");

  after(async () => {
    await vscode.commands.executeCommand("workbench.action.closeAllEditors");
  });

  /** The problems count as the webview drew it. */
  const problems = (drawn: Drawn<ProjectReport>) =>
    drawn.report.health.find((count) => count.key === "problems");

  /** Waits for the view to draw the problems count in `state`, and returns it. */
  async function drawnIn(state: string): Promise<NonNullable<ReturnType<typeof problems>>> {
    const api = await activated();
    const drawn = await api.ui.project.whenDrawn(
      `every build's problems ${state}`,
      (d) => problems(d)?.state === state,
      30_000,
    );
    const count = problems(drawn);
    assert.ok(count);
    return count;
  }

  /** Makes an unsaved edit to `uri` in its editor, and takes it back once `then` has run. */
  async function editing(uri: vscode.Uri, then: () => Promise<void>): Promise<void> {
    const editor = await vscode.window.showTextDocument(
      await vscode.workspace.openTextDocument(uri),
    );
    assert.ok(await editor.edit((e) => e.insert(new vscode.Position(0, 0), "x")));
    try {
      await then();
    } finally {
      await vscode.window.showTextDocument(editor.document);
      await vscode.commands.executeCommand("workbench.action.files.revert");
    }
  }

  it("counts every build's problems as unknown, current after Check Every Build, and out of date after an edit", async () => {
    const api = await activated();
    await vscode.window.showTextDocument(await vscode.workspace.openTextDocument(keys));
    await vscode.commands.executeCommand("ascribe.project.focus");
    const folder = await waitFor("the sidebar's project", () => api.ui.sidebar.project());

    const unknown = await drawnIn("unknown");
    assert.deepEqual([unknown.value, unknown.note], ["—", "Needs every build checked"]);
    const head = api.ui.project.drawn()?.report;
    assert.ok(head?.head.includes("Server: running"), head?.head.join("\n"));
    assert.ok(head?.buttons.includes("Check Every Build"), head?.buttons.join("\n"));

    // The view's Check Every Build checks its own project, wherever the focus is.
    await api.ui.project.choose("checkEveryBuild");
    const current = await drawnIn("current");
    // broken.md's unknown attribute key is an error in every build.
    assert.match(current.value, /^\d+ errors?(, \d+ warnings?)?(, \d+ advice)?$/, current.value);
    assert.match(current.note, /^Checked at /, current.note);
    assert.equal(api.ui.everyBuild.isCurrent(folder), true);
    assert.equal(api.ui.project.drawn()?.report.buttons[0], current.value);

    // An edit to a page, unsaved, makes it out of date.
    await editing(keys, async () => {
      const stale = await drawnIn("outOfDate");
      assert.equal(stale.value, "—");
      assert.match(stale.note, /^Out of date: the project changed after the check at /);
      assert.equal(api.ui.everyBuild.isCurrent(folder), false);
    });
  });

  /** Waits for the view to draw the count `key` in one of `states`, and returns it. */
  async function countIn(key: string, ...states: string[]) {
    const api = await activated();
    const drawn = await api.ui.project
      .whenDrawn(
        `${key} ${states.join(" or ")}`,
        (d) => states.includes(d.report.health.find((count) => count.key === key)?.state ?? ""),
        30_000,
      )
      .catch((error: unknown) => {
        // Says what it drew instead.
        const last = api.ui.project.drawn()?.report.health.find((count) => count.key === key);
        throw new Error(`${String(error)}; it drew ${JSON.stringify(last)}`);
      });
    const count = drawn.report.health.find((c) => c.key === key);
    assert.ok(count);
    return count;
  }

  it("counts examples, overdue reviews for the day, and the permalinks list from the same check, out of date with it", async () => {
    const api = await activated();
    await vscode.window.showTextDocument(await vscode.workspace.openTextDocument(keys));
    await vscode.commands.executeCommand("ascribe.project.focus");
    await api.ui.project.choose("checkEveryBuild");
    await drawnIn("current");
    for (const key of ["examples", "reviews", "permalinks"]) {
      const count = await countIn(key, "current", "none");
      assert.match(count.note, /^Checked at /, count.note);
      // A zero is drawn dimmed, and a count above it isn't.
      assert.equal(count.dimmed, count.state === "none", `${key}: ${count.value}`);
    }
    assert.match((await countIn("reviews", "current", "none")).note, /^Checked at .+, for .+/);
    await editing(keys, async () => {
      for (const key of ["examples", "reviews", "permalinks"]) {
        const stale = await countIn(key, "outOfDate");
        assert.equal(stale.value, "—");
        assert.equal(stale.dimmed, false);
      }
    });
  });

  it("counts orphans and unused entries from the inventory, and opens Pages and Content model filtered", async () => {
    const api = await activated();
    await vscode.window.showTextDocument(await vscode.workspace.openTextDocument(keys));
    await vscode.commands.executeCommand("ascribe.project.focus");
    const orphans = await countIn("orphans", "current", "none");
    const unused = await countIn("unused", "current", "none");
    for (const count of [orphans, unused]) {
      assert.match(count.value, /^(0|\d+ (pages?|entry|entries))$/, count.value);
      assert.equal(count.dimmed, count.state === "none");
    }
    try {
      await api.ui.project.choose("orphans");
      await api.ui.sidebar.whenSettled();
      assert.equal(
        api.ui.sidebar.message("pages") ===
          "No orphans: something links to or includes every page and fragment.",
        orphans.state === "none",
        api.ui.sidebar.message("pages"),
      );
      await api.ui.project.choose("unused");
      await api.ui.sidebar.whenSettled();
      assert.equal(
        api.ui.sidebar.message("model") === "Nothing unused: every entry is used.",
        unused.state === "none",
        api.ui.sidebar.message("model"),
      );
    } finally {
      await api.ui.sidebar.filter("pages", false);
      await api.ui.sidebar.filter("model", false);
    }
  });

  it("checks the agent files when it shows the project, again after a save, and goes out of date on a change on disk", async () => {
    const api = await activated();
    await vscode.window.showTextDocument(await vscode.workspace.openTextDocument(keys));
    await vscode.commands.executeCommand("ascribe.project.focus");
    const folder = await waitFor("the sidebar's project", () => api.ui.sidebar.project());
    // Showing the project checked them; an earlier test's change on disk may
    // have put that check out of date since, so the view's action checks again.
    await waitFor("a check of the agent files", () => api.ui.agentFiles.runs() > 0);
    await api.ui.agentFiles.whenSettled();
    await api.ui.project.choose("checkAgentFiles");
    const first = await countIn("agentFiles", "current", "none");
    assert.match(first.value, /^(0|\d+ files?)$/, first.value);
    assert.match(first.note, /^Checked at .+, after the last save$/, first.note);

    // A save of a page runs it again, once.
    const before = api.ui.agentFiles.runs();
    const editor = await vscode.window.showTextDocument(
      await vscode.workspace.openTextDocument(keys),
    );
    assert.ok(await editor.edit((e) => e.insert(new vscode.Position(0, 0), "x")));
    assert.ok(await editor.document.save());
    await waitFor("a check after the save", () => api.ui.agentFiles.runs() > before);
    assert.ok(await editor.edit((e) => e.delete(new vscode.Range(0, 0, 0, 1))));
    assert.ok(await editor.document.save());
    await api.ui.agentFiles.whenSettled();
    assert.equal(api.ui.agentFiles.resultOf(folder).kind, "made");

    // A file written on disk, not saved in the editor, makes it out of date.
    const written = path.join(folder, "written-on-disk.txt");
    writeFileSync(written, "Not a page.\n");
    try {
      const stale = await countIn("agentFiles", "outOfDate");
      assert.match(stale.note, /^Out of date: a file changed on disk after the check at /);
      assert.ok(api.ui.project.drawn()?.report.buttons.includes("Check Agent Files"));
      await api.ui.project.choose("checkAgentFiles");
      await countIn("agentFiles", "current", "none");
    } finally {
      rmSync(written, { force: true });
    }
  });

  it("goes out of date for a file beside ascribe.toml, outside the content root", async () => {
    const api = await activated();
    await vscode.window.showTextDocument(await vscode.workspace.openTextDocument(keys));
    await vscode.commands.executeCommand("ascribe.project.focus");
    await api.ui.project.choose("checkEveryBuild");
    await drawnIn("current");
    // memos/ is the project's, beside docs/, its content root.
    await editing(memo, async () => {
      await drawnIn("outOfDate");
    });
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
    assert.ok(diagnostics.some((diagnostic) => diagnostic.range.start.line === 8));
  });

  it("reports nothing for the pages of the example that are correct", async () => {
    await diagnosticsOf(broken, (all) => all.length > 0);
    for (const page of ["quickstart.md", "install-agent.md", "keys.md"]) {
      assert.deepEqual(vscode.languages.getDiagnostics(uriOf("docs", page)), [], page);
    }
  });

  it("offers Prompt agent on a problem after its quick fixes, and copies the prompt", async () => {
    const api = await activated();
    const [problem] = await diagnosticsOf(broken, (all) => all.length > 0);
    assert.ok(problem);
    await vscode.window.showTextDocument(await vscode.workspace.openTextDocument(broken));
    const actions = await vscode.commands.executeCommand<vscode.CodeAction[]>(
      "vscode.executeCodeActionProvider",
      broken,
      problem.range,
    );
    const titles = actions.map((action) => action.title);
    const at = titles.indexOf("Prompt agent: fix this problem");
    assert.ok(at >= 0, `the actions: ${titles.join(", ")}`);
    const lastFix = actions.findLastIndex((action) => (action.diagnostics?.length ?? 0) > 0);
    assert.ok(lastFix < at, `after the quick fixes: ${titles.join(", ")}`);
    const command = actions[at]?.command;
    assert.ok(command);
    await vscode.env.clipboard.writeText("");
    const before = api.promptAgent.deliveries.length;
    await vscode.commands.executeCommand(command.command, ...(command.arguments ?? []));
    await waitFor("the delivery", () => api.promptAgent.deliveries.length > before);
    assert.equal(api.promptAgent.deliveries[before]?.delivered, "clipboard");
    const prompt = await vscode.env.clipboard.readText();
    assert.ok(
      prompt.startsWith("Fix this problem in `docs/broken.md`.\n\nWhere: docs/broken.md:9\n"),
      prompt,
    );
    // The binary the editor runs, as an agent beside it can run it.
    const binary = api.binary()?.path;
    assert.ok(binary);
    const elsewhere = await ascribeElsewhere(binary, nodePathEnvironment());
    const ascribe = elsewhere ? shellWord(elsewhere.program) : "ascribe";
    assert.ok(prompt.includes(`When you're done, run \`${ascribe} check docs/broken.md\``), prompt);
    await vscode.commands.executeCommand("workbench.action.closeAllEditors");
  });

  it("puts the binary the editor runs on the terminals' path when `ascribe` isn't it", async () => {
    const api = await activated();
    await api.agents.terminals.whenSettled();
    const binary = api.binary()?.path;
    assert.ok(binary);
    const elsewhere = await ascribeElsewhere(binary, nodePathEnvironment());
    const folder = vscode.workspace.workspaceFolders?.[0]?.uri.fsPath;
    assert.ok(folder);
    assert.equal(
      api.agents.terminals.directories().get(folder),
      elsewhere ? path.dirname(binary) : undefined,
    );
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
    const line = editor.document.lineAt(8);
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
