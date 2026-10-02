import * as assert from "node:assert/strict";
import { copyFileSync, mkdirSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import * as path from "node:path";
import * as vscode from "vscode";
import type { ServerState } from "../../../src/client.js";
import type { AscribeApi } from "../../../src/extension.js";
import type { PreviewApi, RenderRecord } from "../../../src/preview/controller.js";
import { comparable, samePath } from "../../../src/projects.js";
import { activated, diagnosticsOf, uriOf, waitFor, workspace } from "./helpers.js";

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
    const deadline = Date.now() + 30_000;
    for (;;) {
      rmSync(config, { force: true });
      copyFileSync(uriOf("docs", "ascribe.toml").fsPath, config);
      try {
        await waitFor(`the project in ${path.basename(folder)}`, () => known(folder), 2_000);
        return;
      } catch (error) {
        if (Date.now() > deadline) throw error;
      }
    }
  }

  const known = (folder: string): boolean =>
    api.projects().some((project) => samePath(project.folder, folder));

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

    it("starts exactly the server of the project whose file is opened", async () => {
      await open(handbookPage);
      await waitFor("the handbook's server", () => api.state(folder.handbook()) === "running");
      await api.whenSettled();
      assertRunning([folder.handbook()]);
      assert.equal(api.binary(folder.handbook())?.source, "setting");
    });

    it("diagnoses a file only through the project that owns it", async () => {
      await open(docsPage);
      await diagnosticsOf(docsPage, (all) => all.length > 0);
      await api.whenSettled();
      assert.deepEqual(codes(docsPage), ["ASC036"]);
      assertRunning([folder.docs(), folder.handbook()]);
      // The handbook's server reads the nested project's page too (it is in the
      // handbook's content root), and finds two problems in it; neither is shown,
      // and the nested project's server hasn't started.
      await throughHandbook();
      assert.deepEqual(vscode.languages.getDiagnostics(nestedPage), []);
      assert.deepEqual(vscode.languages.getDiagnostics(handbookPage), []);
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

    it("diagnoses a nested project's file through the nested project alone", async () => {
      await open(nestedPage);
      await diagnosticsOf(nestedPage, (all) => all.length > 0);
      await api.whenSettled();
      assertRunning([folder.docs(), folder.handbook(), folder.nested()]);
      // `{edition}` is declared only in the nested project: the handbook's
      // server reports it (ASC044), and the ASC001 a second time.
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

  describe("projects that come and go", () => {
    it("adds a project when its ascribe.toml appears, and stops its server when it goes", async () => {
      const guides = path.join(workspace(), "guides");
      const page = vscode.Uri.file(path.join(guides, "docs", "index.md"));
      mkdirSync(path.dirname(page.fsPath), { recursive: true });
      writeFileSync(page.fsPath, "---\ntitle: Guides\n---\n\nSee [](missing.md).\n");
      try {
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
        // Its server is still running. On Windows this fails if the server
        // runs in the project's folder, which a running process locks.
        rmSync(idle, { recursive: true, force: true });
      }
    });
  });
});
