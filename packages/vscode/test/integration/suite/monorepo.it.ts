import * as assert from "node:assert/strict";
import { copyFileSync, existsSync, mkdirSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import * as path from "node:path";
import * as vscode from "vscode";
import type { ServerState } from "../../../src/client.js";
import type { AscribeApi } from "../../../src/extension.js";
import type { PreviewApi, RenderRecord } from "../../../src/preview/controller.js";
import { MAX_WAIT_MS } from "../../../src/diskChanges.js";
import { comparable, samePath } from "../../../src/projects.js";
import { activated, diagnosticsOf, sleep, uriOf, waitFor, workspace } from "./helpers.js";

// The real `ascribe lsp` on test/fixtures/monorepo: a folder with no project
// (code/), a project (docs/), and a project (handbook/) with another nested
// in its content root (handbook/pages/nested/). Servers start on demand.
describe("with several projects, one nested in another", () => {
  const folder = {
    docs: () => path.join(workspace(), "docs"),
    handbook: () => path.join(workspace(), "handbook"),
    nested: () => path.join(workspace(), "handbook", "pages", "nested"),
  };
  const docsPage = uriOf("docs", "docs", "index.md");
  const handbookPage = uriOf("handbook", "pages", "index.md");
  const nestedPage = uriOf("handbook", "pages", "nested", "content", "index.md");
  const codeReadme = uriOf("code", "README.md");
  let api: AscribeApi;

  const codes = (uri: vscode.Uri): string[] =>
    vscode.languages
      .getDiagnostics(uri)
      .map((diagnostic) => {
        const code = diagnostic.code;
        return String(typeof code === "object" ? code.value : code);
      })
      .sort();

  /** The state of each project's server, by folder in comparable form. */
  const states = (): Map<string, ServerState> =>
    new Map(api.projects().map((project) => [comparable(project.folder), project.state]));

  /** Asserts which projects' servers are running; every other one must be stopped. */
  function assertRunning(running: string[]): void {
    const expected = new Map(
      [folder.docs(), folder.handbook(), folder.nested()].map((f) => [
        comparable(f),
        running.some((r) => samePath(r, f)) ? "running" : "stopped",
      ]),
    );
    assert.deepEqual(states(), expected);
  }

  async function open(uri: vscode.Uri): Promise<vscode.TextEditor> {
    return vscode.window.showTextDocument(await vscode.workspace.openTextDocument(uri));
  }

  /** Opens a file and replaces its text in the editor, without saving. */
  async function edit(uri: vscode.Uri, change: (text: string) => string): Promise<void> {
    const editor = await open(uri);
    const document = editor.document;
    const whole = new vscode.Range(
      document.positionAt(0),
      document.positionAt(document.getText().length),
    );
    assert.ok(await editor.edit((e) => e.replace(whole, change(document.getText()))));
  }

  /** Puts back the text on disk, and saves, so the editor isn't left dirty. */
  async function restore(uri: vscode.Uri): Promise<void> {
    await edit(uri, () => readFileSync(uri.fsPath, "utf8"));
    assert.ok(await (await vscode.workspace.openTextDocument(uri)).save());
  }

  /**
   * Edits the handbook's open page to add a problem only its server reports,
   * waits for it, and takes it out again. The server publishes in order, so
   * once this comes back, whatever it published before has arrived too.
   */
  async function throughHandbook(): Promise<void> {
    await edit(handbookPage, (text) => `${text}\nA {flush} phrase.\n`);
    await diagnosticsOf(handbookPage, () => codes(handbookPage).join() === "ASC044");
    await restore(handbookPage);
    await diagnosticsOf(handbookPage, (all) => all.length === 0);
  }

  /**
   * Creates a project by writing its `ascribe.toml` (the docs project's model,
   * whose content root is `docs` too) and waits for the extension to find it.
   * A file created in a directory made a moment before can be missed while
   * the watcher takes in the new directory, so it's written again (deleted
   * and created) every two seconds until it's found.
   */
  async function createProject(folder: string): Promise<void> {
    const config = path.join(folder, "ascribe.toml");
    // Seen independently of the extension, to say why on a timeout.
    const watcher = vscode.workspace.createFileSystemWatcher("**/ascribe.toml");
    let created = 0;
    watcher.onDidCreate((uri) => {
      if (samePath(uri.fsPath, config)) created++;
    });
    const deadline = Date.now() + 30_000;
    try {
      for (;;) {
        rmSync(config, { force: true });
        copyFileSync(uriOf("docs", "ascribe.toml").fsPath, config);
        try {
          await waitFor(`the project in ${path.basename(folder)}`, () => known(folder), 2_000);
          return;
        } catch (error) {
          if (Date.now() > deadline) {
            const found = (await vscode.workspace.findFiles("**/ascribe.toml")).some((uri) =>
              samePath(uri.fsPath, config),
            );
            throw new Error(
              `${String(error)}. Its ascribe.toml was reported created ${created} times, and ` +
                `findFiles ${found ? "finds" : "doesn't find"} it. Known projects: ` +
                api
                  .projects()
                  .map((project) => project.folder)
                  .join(", "),
            );
          }
        }
      }
    } finally {
      watcher.dispose();
    }
  }

  /**
   * Waits until changes just made on disk have been acted on: a project's
   * files written before its ascribe.toml would otherwise start its server
   * as soon as the project is found.
   */
  const afterDiskChanges = () => sleep(MAX_WAIT_MS + 1_000);

  /**
   * Makes a folder and those above it one at a time, each once the file
   * watcher has reported the one before. The watcher never reports anything
   * in a folder made before it took in the folder's parent (#56), so a file
   * written there would start nothing. A folder that isn't reported is made
   * again every two seconds.
   */
  async function mkdirWatched(folder: string): Promise<void> {
    const missing: string[] = [];
    for (let f = folder; !existsSync(f); f = path.dirname(f)) missing.unshift(f);
    // Served by the workspace's own watcher, as the extension's is.
    const watcher = vscode.workspace.createFileSystemWatcher("**/*");
    const reported = new Set<string>();
    watcher.onDidCreate((uri) => reported.add(comparable(uri.fsPath)));
    try {
      for (const dir of missing) {
        const deadline = Date.now() + 30_000;
        for (;;) {
          rmSync(dir, { recursive: true, force: true });
          mkdirSync(dir);
          try {
            await waitFor(
              `the watcher to report ${dir}`,
              () => reported.has(comparable(dir)),
              2_000,
            );
            break;
          } catch (error) {
            if (Date.now() > deadline) throw error;
          }
        }
      }
    } finally {
      watcher.dispose();
    }
  }

  const known = (folder: string): boolean =>
    api.projects().some((project) => samePath(project.folder, folder));

  /** The Projects view's top level, by label: each project's state and its children's labels. */
  async function viewed(): Promise<Map<string, { state: string; children: string[] }>> {
    const items = await api.ui.projects.items();
    return new Map(
      items.map((item) => [
        item.label,
        {
          state: item.contextValue.replace("ascribe.project.", ""),
          children: item.children.map((child) => `${child.label} (${child.description})`),
        },
      ]),
    );
  }

  /** The status bar item's text, once the item is shown and `check` accepts it. */
  const statusText = (description: string, check: (text: string) => boolean) =>
    waitFor(`the status bar to show ${description}`, () => {
      const text = api.ui.statusBar.shown()?.text;
      return text !== undefined && check(text) ? text : undefined;
    });

  before(async () => {
    api = await activated();
    await api.whenSettled();
  });

  after(async () => {
    await vscode.commands.executeCommand("workbench.action.closeAllEditors");
  });

  describe("servers", () => {
    it("finds every project and starts no server until one of its files is opened", () => {
      assertRunning([]);
      // Nothing is running, so nothing has diagnostics, problems or not.
      assert.deepEqual(vscode.languages.getDiagnostics(docsPage), []);
      assert.deepEqual(vscode.languages.getDiagnostics(nestedPage), []);
    });

    it("lists every project in the Projects view, and showing or refreshing it starts none", async () => {
      const started: string[] = [];
      const listener = api.onDidStartServer((f) => started.push(f));
      try {
        await vscode.commands.executeCommand("workbench.view.extension.ascribe");
        const shown = await viewed();
        await api.ui.projects.refresh();
        await api.whenSettled();
        assert.deepEqual(await viewed(), shown);
        // Used by, Pages, and Content model have no running project to show.
        await api.ui.sidebar.whenSettled();
        assert.deepEqual(api.ui.sidebar.items("pages"), []);
        assert.deepEqual(api.ui.sidebar.items("model"), []);
        assert.deepEqual(
          shown,
          new Map(
            ["docs", "handbook", "handbook/pages/nested"].map((name) => [
              name,
              { state: "stopped", children: ["ascribe.toml (content model)"] },
            ]),
          ),
        );
      } finally {
        listener.dispose();
        await vscode.commands.executeCommand("workbench.action.closeSidebar");
      }
      // What the view caused: no server started while it was shown and refreshed.
      assert.deepEqual(started, []);
    });

    it("starts exactly the server of the project whose file is opened", async () => {
      await open(handbookPage);
      await waitFor("the handbook's server", () => api.state(folder.handbook()) === "running");
      await api.whenSettled();
      assertRunning([folder.handbook()]);
      assert.equal(api.binary(folder.handbook())?.source, "setting");
    });

    it("shows the started project running in the Projects view, and in the status bar", async () => {
      await api.ui.whenBuildsKnown();
      const shown = await waitFor("the handbook's editor build in the view", async () => {
        const view = await viewed();
        return view.get("handbook")?.children.length === 3 ? view : undefined;
      });
      assert.deepEqual(shown.get("handbook")?.state, "running");
      assert.deepEqual(shown.get("handbook")?.children.slice(0, 2), [
        "ascribe.toml (content model)",
        "site (editor build)",
      ]);
      assert.match(shown.get("handbook")?.children[2] ?? "", /^ascribe \S+ \(setting\)$/);
      assert.equal(shown.get("docs")?.state, "stopped");
      assert.equal(shown.get("handbook/pages/nested")?.state, "stopped");

      await statusText("the handbook", (text) => text === "$(book) handbook · site");
      const tooltip = api.ui.statusBar.shown()?.tooltip ?? "";
      assert.ok(tooltip.includes(path.join(folder.handbook(), "ascribe.toml")), tooltip);
      assert.match(tooltip, /Server: running/);
    });

    it("lists only the active project's own pages and content model in the sidebar", async () => {
      await open(handbookPage);
      const pages = await waitFor("the handbook's pages", async () => {
        await api.ui.sidebar.whenSettled();
        return samePath(api.ui.sidebar.project() ?? "", folder.handbook())
          ? api.ui.sidebar.items("pages")
          : undefined;
      });
      // The nested project's page is in the handbook's content root, but it's
      // the nested project's.
      assert.deepEqual(
        pages.map((group) => [group.label, group.children.map((page) => page.description)]),
        [["page", ["index.md"]]],
      );
      const phrases = api.ui.sidebar
        .items("model")
        .find((kind) => kind.label === "Phrases")
        ?.children.map((entry) => `${entry.label} (${entry.description})`);
      assert.deepEqual(phrases, ["product (1 use)"]);
      // Filling the views started no other server.
      assertRunning([folder.handbook()]);
    });

    it("diagnoses a file only through the project that owns it", async () => {
      await open(docsPage);
      await diagnosticsOf(docsPage, (all) => all.length > 0);
      await api.whenSettled();
      assert.deepEqual(codes(docsPage), ["ASC036"]);
      assertRunning([folder.docs(), folder.handbook()]);
      // The nested project's page is in the handbook's content root, but in
      // another project's folder: the handbook's server doesn't read it, and the
      // nested project's server hasn't started, so it has no diagnostics.
      await throughHandbook();
      assert.deepEqual(vscode.languages.getDiagnostics(nestedPage), []);
      assert.deepEqual(vscode.languages.getDiagnostics(handbookPage), []);
      // So to the handbook it isn't a source: a link to it names a missing file.
      await edit(handbookPage, (text) => `${text}\nSee [](nested/content/index.md).\n`);
      await diagnosticsOf(handbookPage, () => codes(handbookPage).join() === "ASC036");
      await restore(handbookPage);
      await diagnosticsOf(handbookPage, (all) => all.length === 0);
      assert.deepEqual(vscode.languages.getDiagnostics(nestedPage), []);
    });

    it("restarts the running servers and leaves the others stopped", async () => {
      const started: string[] = [];
      const listener = api.onDidStartServer((f) => started.push(comparable(f)));
      try {
        await vscode.commands.executeCommand("ascribe.restartServer");
        await api.whenSettled();
      } finally {
        listener.dispose();
      }
      assert.deepEqual(started.sort(), [comparable(folder.docs()), comparable(folder.handbook())]);
      assertRunning([folder.docs(), folder.handbook()]);
      // The new server reads its project again.
      await diagnosticsOf(docsPage, () => codes(docsPage).join() === "ASC036");
    });

    it("restarts one project's server from the Projects view", async () => {
      const started: string[] = [];
      const listener = api.onDidStartServer((f) => started.push(comparable(f)));
      try {
        await api.ui.projects.run("restart", folder.docs());
        await api.whenSettled();
      } finally {
        listener.dispose();
      }
      assert.deepEqual(started, [comparable(folder.docs())]);
      assertRunning([folder.docs(), folder.handbook()]);
      await diagnosticsOf(docsPage, () => codes(docsPage).join() === "ASC036");
    });

    it("diagnoses a nested project's file through the nested project alone", async () => {
      await open(nestedPage);
      await diagnosticsOf(nestedPage, (all) => all.length > 0);
      await api.whenSettled();
      assertRunning([folder.docs(), folder.handbook(), folder.nested()]);
      // The status bar names the nested project, not the one around it.
      await statusText("the nested project", (text) =>
        text.startsWith("$(book) handbook/pages/nested"),
      );
      // The sidebar lists the nested project's own pages and content model.
      const pages = await waitFor("the nested project's pages", async () => {
        await api.ui.sidebar.whenSettled();
        return samePath(api.ui.sidebar.project() ?? "", folder.nested())
          ? api.ui.sidebar.items("pages")
          : undefined;
      });
      assert.deepEqual(
        pages.map((group) => [group.label, group.children.map((page) => page.description)]),
        [["page", ["index.md"]]],
      );
      const entries = (label: string) =>
        api.ui.sidebar
          .items("model")
          .find((kind) => kind.label === label)
          ?.children.map((entry) => entry.label);
      assert.deepEqual(entries("Phrases"), ["product", "edition"]);
      assert.deepEqual(entries("Dimensions"), ["tier"]);
      // `{edition}` is declared only in the nested project, so the page has no
      // ASC044, which the handbook's model would give it, and its ASC001 is
      // reported once. Once the handbook's server has answered an edit, it has
      // published whatever it would for the page.
      await throughHandbook();
      assert.deepEqual(codes(nestedPage), ["ASC001"]);

      // The open document goes to the nested project's server as it's edited.
      await edit(nestedPage, (text) => text.replace("{colour=red}", "{type=tip}"));
      await diagnosticsOf(nestedPage, (all) => all.length === 0);
      await restore(nestedPage);
      await diagnosticsOf(nestedPage, () => codes(nestedPage).join() === "ASC001");

      // And the handbook's own page is the handbook's alone.
      await edit(handbookPage, (text) => `${text}\nSee [](missing.md).\n`);
      await diagnosticsOf(handbookPage, (all) => all.length > 0);
      assert.deepEqual(codes(handbookPage), ["ASC036"]);
      await restore(handbookPage);
      await diagnosticsOf(handbookPage, (all) => all.length === 0);
    });
  });

  describe("the preview", () => {
    let preview: PreviewApi;
    const html = (render: RenderRecord) => render.result.page?.html ?? "";
    const drawnFor = (description: string, predicate: (render: RenderRecord) => boolean) =>
      preview.whenDrawn(description, predicate);
    const latestSeq = () => preview.renders().at(-1)?.seq ?? 0;
    const isOf = (render: RenderRecord, uri: vscode.Uri) =>
      samePath(vscode.Uri.parse(render.document).fsPath, uri.fsPath);

    before(async () => {
      preview = api.preview;
      await open(docsPage);
      await vscode.commands.executeCommand("ascribe.openPreview");
    });

    after(async () => {
      await vscode.commands.executeCommand("workbench.action.closeAllEditors");
    });

    it("renders each project's page, with its own build", async () => {
      const docs = await drawnFor(
        "the docs page",
        (r) => isOf(r, docsPage) && r.result.page?.path === "index.md",
      );
      assert.equal(docs.result.build, "site");
      assert.ok(
        docs.result.contentRoot && samePath(docs.result.contentRoot, uriOf("docs", "docs").fsPath),
      );
      assert.match(html(docs), /Point the agent at your server/);

      // Choosing a build in one project.
      await preview.receive({ type: "build", name: "cloud" });
      const cloud = await drawnFor(
        "the docs page in the cloud build",
        (r) => isOf(r, docsPage) && r.result.build === "cloud",
      );
      assert.doesNotMatch(html(cloud), /Point the agent at your server/);
      assert.match(html(cloud), /Sign in to Quill Cloud/);

      // Another project previews its own editor's build.
      let after = latestSeq();
      await open(handbookPage);
      const handbook = await drawnFor(
        "the handbook page",
        (r) => r.seq > after && isOf(r, handbookPage) && r.result.page !== null,
      );
      assert.equal(handbook.result.build, "site");
      assert.match(html(handbook), /handbook/);
      assert.ok(
        handbook.result.contentRoot &&
          samePath(handbook.result.contentRoot, uriOf("handbook", "pages").fsPath),
      );
      assert.ok(
        preview
          .localResourceRoots()
          .some((root) => samePath(root, uriOf("handbook", "pages").fsPath)),
        "the webview may read the handbook's content root",
      );
      assert.ok(
        !preview.localResourceRoots().some((root) => samePath(root, uriOf("docs", "docs").fsPath)),
        "and no longer the docs project's",
      );

      // And the first project kept its choice.
      after = latestSeq();
      await open(docsPage);
      const back = await drawnFor(
        "the docs page again",
        (r) => r.seq > after && isOf(r, docsPage) && r.result.page !== null,
      );
      assert.equal(back.result.build, "cloud");

      // The nested project's page comes from the nested project's server.
      after = latestSeq();
      await open(nestedPage);
      const nested = await drawnFor(
        "the nested page",
        (r) => r.seq > after && isOf(r, nestedPage) && r.result.page !== null,
      );
      assert.equal(nested.result.page?.path, "index.md");
      assert.ok(
        nested.result.contentRoot &&
          samePath(
            nested.result.contentRoot,
            uriOf("handbook", "pages", "nested", "content").fsPath,
          ),
      );
      assert.deepEqual(
        nested.result.builds.map((b) => b.name),
        ["site"],
      );
    });

    it("shares one build with the status bar", async () => {
      // The choice made in the preview's picker above is the status bar's too.
      let after = latestSeq();
      await open(docsPage);
      await drawnFor(
        "the docs page in the cloud build",
        (r) => r.seq > after && isOf(r, docsPage) && r.result.build === "cloud",
      );
      await statusText("the cloud build", (text) => text === "$(book) docs · cloud");

      await preview.receive({ type: "build", name: "site" });
      await statusText("the site build", (text) => text === "$(book) docs · site");

      // Switch build, from the status bar's menu, changes what the preview renders.
      after = latestSeq();
      api.ui.statusBar.answerNext("Switch build");
      api.ui.statusBar.answerNext("cloud");
      await vscode.commands.executeCommand("ascribe.projectMenu");
      const cloud = await drawnFor(
        "the docs page in the cloud build again",
        (r) => r.seq > after && isOf(r, docsPage) && r.result.build === "cloud",
      );
      assert.match(html(cloud), /Sign in to Quill Cloud/);
      assert.equal(preview.build(), "cloud");
      await statusText("the cloud build", (text) => text === "$(book) docs · cloud");

      // And back to the editor's build, with the command.
      after = latestSeq();
      api.ui.statusBar.answerNext("site");
      await vscode.commands.executeCommand("ascribe.switchBuild");
      await drawnFor(
        "the docs page in the site build",
        (r) => r.seq > after && isOf(r, docsPage) && r.result.build === "site",
      );
      await statusText("the site build", (text) => text === "$(book) docs · site");
    });

    it("says when a file isn't part of any project", async () => {
      const after = latestSeq();
      await open(codeReadme);
      const render = await drawnFor("the code README", (r) => r.seq > after && isOf(r, codeReadme));
      assert.equal(render.result.page, null);
      assert.equal(render.result.problems.length, 1);
      assert.match(
        render.result.problems[0]?.message ?? "",
        /isn't part of an Ascribe project \(no ascribe\.toml above it\)/,
      );
      // No server checks it: its broken link has no diagnostic.
      assert.deepEqual(vscode.languages.getDiagnostics(codeReadme), []);
      // And the status bar names no project.
      await waitFor("the status bar to hide", () => api.ui.statusBar.shown() === undefined);
    });

    it("says when a file is in a project but outside its content root", async () => {
      const readme = uriOf("docs", "README.md");
      const after = latestSeq();
      await open(readme);
      const render = await drawnFor(
        "the docs README",
        (r) => r.seq > after && isOf(r, readme) && r.result.problems.length > 0,
      );
      assert.equal(render.result.page, null);
      const messages = render.result.problems.map((problem) => problem.message);
      assert.equal(messages.length, 1, messages.join("\n"));
      assert.match(messages[0] ?? "", /is in the project docs, but outside its content root/);
      assert.doesNotMatch(messages[0] ?? "", /isn't part of an Ascribe project/);
    });
  });

  describe("actions", () => {
    it("are offered in a file of a project, and not in a Markdown file outside every project", async () => {
      await open(codeReadme);
      await waitFor("ascribe.inProject to be false", () => !api.actions.inProject());
      await open(nestedPage);
      await waitFor("ascribe.inProject", () => api.actions.inProject());
    });

    it("go to the server of the project that owns the file", async () => {
      // `edition` is a phrase of the nested project only, and the handbook's
      // server has no page here.
      const editor = await open(nestedPage);
      // Before "edition" in "## The {edition} edition".
      editor.selection = new vscode.Selection(5, 17, 5, 17);
      const before = api.actions.runs.length;
      api.actions.answerNext(["edition"]);
      await vscode.commands.executeCommand("ascribe.action.insertPhrase");
      try {
        assert.deepEqual(api.actions.runs[before]?.messages, []);
        assert.equal(editor.document.lineAt(5).text, "## The {edition} {edition}edition");
      } finally {
        await vscode.commands.executeCommand("workbench.action.files.revert");
      }
    });

    it("open the bar in a page of a project, and nothing in a Markdown file outside every project", async () => {
      const before = api.actions.bars.length;
      await open(codeReadme);
      await vscode.commands.executeCommand("ascribe.actions");
      assert.equal(api.actions.bars.length, before);
      await open(nestedPage);
      await vscode.commands.executeCommand("ascribe.actions");
      assert.equal(api.actions.bars.length, before + 1);
      await vscode.commands.executeCommand("workbench.action.closeQuickOpen");
    });

    it("offer the nested project's note types and dimensions in its page, not the handbook's", async () => {
      const editor = await open(nestedPage);
      // The blank line between "@id: edition" and the note.
      editor.selection = new vscode.Selection(7, 0, 7, 0);
      /** The values the first step of the bar's action offers; the wizard is then cancelled. */
      async function offered(label: string): Promise<string[]> {
        let values: string[] = [];
        api.actions.answerNext([
          (step) => {
            values = step.kind === "text" ? [] : step.choices.map((choice) => choice.value);
            return Promise.resolve(null);
          },
        ]);
        const before = api.actions.runs.length;
        await vscode.commands.executeCommand("ascribe.actions");
        assert.ok(api.actions.selectInBar(label), `the bar has no row ${label}`);
        await vscode.commands.executeCommand("workbench.action.acceptSelectedQuickOpenItem");
        await waitFor(label, () => api.actions.runs[before]);
        return values;
      }
      const notes = await offered("Insert a note");
      assert.ok(notes.includes("edition"), notes.join());
      assert.ok(!notes.includes("policy"), notes.join());
      assert.deepEqual(await offered("Make the page one variant"), ["tier"]);
    });
  });

  describe("files changed on disk", () => {
    // A project none of whose files is open, as an agent that edits files
    // without opening them leaves it.
    const agent = () => path.join(workspace(), "agent");
    const page = (name: string) => vscode.Uri.file(path.join(agent(), "docs", name));

    before(async () => {
      await vscode.commands.executeCommand("workbench.action.closeAllEditors");
      await mkdirWatched(path.join(agent(), "docs"));
      writeFileSync(page("index.md").fsPath, "---\ntitle: Agent\n---\n\nAgent.\n");
      await afterDiskChanges();
      await createProject(agent());
      assert.equal(api.state(agent()), "stopped");
    });

    after(async () => {
      rmSync(path.join(agent(), "ascribe.toml"), { force: true });
      await waitFor("the agent project to go", () => !known(agent()));
      await api.whenSettled();
      rmSync(agent(), { recursive: true, force: true });
    });

    it("starts nothing for a file written into the project's output directory", async () => {
      const output = path.join(agent(), ".ascribe", "build", "site", "plain");
      await mkdirWatched(output);
      writeFileSync(path.join(output, "index.md"), "# Agent\n\nSee [](missing.md).\n");
      await afterDiskChanges();
      assert.equal(api.state(agent()), "stopped");
    });

    it("starts the project's server once for a burst of files, and shows their problems", async () => {
      const started: string[] = [];
      const listener = api.onDidStartServer((folder) => started.push(comparable(folder)));
      try {
        const broken = page("broken.md");
        writeFileSync(broken.fsPath, "---\ntitle: Broken\n---\n\nSee [](missing.md).\n");
        for (let i = 0; i < 30; i++) {
          writeFileSync(page(`page-${i}.md`).fsPath, `---\ntitle: Page ${i}\n---\n\nPage.\n`);
        }
        await diagnosticsOf(broken, () => codes(broken).join() === "ASC036");
        assert.equal(api.state(agent()), "running");
        // Nothing opened the file.
        assert.ok(
          !vscode.workspace.textDocuments.some((document) =>
            samePath(document.uri.fsPath, broken.fsPath),
          ),
        );
        await afterDiskChanges();
        await api.whenSettled();
        assert.deepEqual(started, [comparable(agent())]);
      } finally {
        listener.dispose();
      }
    });
  });

  describe("projects that come and go", () => {
    it("adds a project when its ascribe.toml appears, and stops its server when it goes", async () => {
      const guides = path.join(workspace(), "guides");
      const page = vscode.Uri.file(path.join(guides, "docs", "index.md"));
      mkdirSync(path.dirname(page.fsPath), { recursive: true });
      writeFileSync(page.fsPath, "---\ntitle: Guides\n---\n\nSee [](missing.md).\n");
      try {
        await afterDiskChanges();
        await createProject(guides);
        assert.equal(api.state(guides), "stopped");

        await open(page);
        await diagnosticsOf(page, () => codes(page).join() === "ASC036");
        await api.whenSettled();
        assert.equal(api.state(guides), "running");

        rmSync(path.join(guides, "ascribe.toml"));
        await waitFor("the guides project to go", () => !known(guides));
        await api.whenSettled();
        assert.equal(api.state(guides), "stopped");
        // Its server's diagnostics went with it.
        await diagnosticsOf(page, (all) => all.length === 0);
      } finally {
        await vscode.commands.executeCommand("workbench.action.closeAllEditors");
        rmSync(guides, { recursive: true, force: true });
      }
    });

    it('starts every server when ascribe.startServers is "all"', async () => {
      const settings = vscode.workspace.getConfiguration("ascribe");
      // A project none of whose files has been opened.
      const idle = path.join(workspace(), "idle");
      mkdirSync(path.join(idle, "docs"), { recursive: true });
      writeFileSync(path.join(idle, "docs", "index.md"), "---\ntitle: Idle\n---\n\nIdle.\n");
      try {
        await afterDiskChanges();
        await createProject(idle);
        assert.equal(api.state(idle), "stopped");
        await settings.update("startServers", "all", vscode.ConfigurationTarget.Workspace);
        await waitFor("every server to run", () =>
          api.projects().every((project) => project.state === "running"),
        );
        await api.whenSettled();
        assert.equal(api.projects().length, 4);
        for (const project of api.projects()) {
          assert.equal(api.binary(project.folder)?.source, "setting", project.folder);
        }
      } finally {
        await settings.update("startServers", undefined, vscode.ConfigurationTarget.Workspace);
        // Wait for the watcher to report the deletion, and for the project
        // search it starts: a search that ran after the next test made its
        // unwatched project would find that project before its file is opened.
        rmSync(path.join(idle, "ascribe.toml"), { force: true });
        await waitFor("the idle project to go", () => !known(idle));
        await api.whenSettled();
        // Its server stopped with it. On Windows this fails if the server runs
        // in the project's folder, which a running process locks.
        rmSync(idle, { recursive: true, force: true });
      }
    });

    // Last: the deletion of its ascribe.toml isn't reported either, so the
    // project stays known until projects are looked for again.
    it("finds a project the file watcher didn't report when one of its files is opened", async () => {
      // The workspace's settings exclude unwatched/ from the file watcher.
      const unwatched = path.join(workspace(), "unwatched");
      const page = vscode.Uri.file(path.join(unwatched, "docs", "index.md"));
      mkdirSync(path.dirname(page.fsPath), { recursive: true });
      writeFileSync(page.fsPath, "---\ntitle: Unwatched\n---\n\nSee [](missing.md).\n");
      copyFileSync(uriOf("docs", "ascribe.toml").fsPath, path.join(unwatched, "ascribe.toml"));
      try {
        // Time for a watcher event, if there were going to be one.
        await new Promise((resolve) => setTimeout(resolve, 2_000));
        assert.ok(!known(unwatched), "the watcher reported the project, so this proves nothing");

        await open(page);
        await diagnosticsOf(page, () => codes(page).join() === "ASC036");
        await api.whenSettled();
        assert.ok(known(unwatched));
        assert.equal(api.state(unwatched), "running");
      } finally {
        await vscode.commands.executeCommand("workbench.action.closeAllEditors");
        rmSync(unwatched, { recursive: true, force: true });
      }
    });
  });
});
