import * as assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { readFileSync, writeFileSync } from "node:fs";
import * as vscode from "vscode";
import type { AscribeApi } from "../../../src/extension.js";
import type { PreviewApi, RenderRecord } from "../../../src/preview/controller.js";
import type { ChangedPage, FromWebview, PageDiff } from "../../../src/preview/protocol.js";
import { pageDetail } from "../../../src/preview/reviewText.js";
import type { OpenedDiff } from "../../../src/preview/sourceChanges.js";
import type { SidebarItem } from "../../../src/ui/sidebarViews.js";
import { StatefulGitHub } from "./fakeGitHub.js";
import { activated, diagnosticsOf, sleep, uriOf, waitFor, workspace } from "./helpers.js";

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
    ) as { builds: { build: string; pages: PageDiff[] }[] };
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

// What agents in VS Code get, on the same repository: the MCP server the
// extension offers, and its tools, called through VS Code's API as an agent's
// host calls them.
describe("agents' tools, with the real language server on a quill repository", () => {
  const install = uriOf("docs", "install-agent.md");
  const keys = uriOf("docs", "keys.md");
  let api: AscribeApi;

  /** Calls a tool through VS Code, as an agent's host does, and returns its text. */
  const invoke = async (name: string, input: Record<string, unknown> = {}): Promise<string> => {
    const call = vscode.lm.invokeTool(name, { input, toolInvocationToken: undefined });
    const result = await Promise.race([
      call,
      sleep(20_000).then(() => assert.fail(`${name} didn't answer: a confirmation, or a hang`)),
    ]);
    return result.content
      .map((part) => (part instanceof vscode.LanguageModelTextPart ? part.value : ""))
      .join("");
  };

  before(async () => {
    api = await activated();
    await api.whenSettled();
    await vscode.window.showTextDocument(await vscode.workspace.openTextDocument(install));
  });

  after(async () => {
    await api.preview.review.stop(workspace());
    await vscode.commands.executeCommand("workbench.action.closeAllEditors");
  });

  it("offers `ascribe mcp` with the project's binary, in the workspace folder", async () => {
    assert.equal(api.agents.mcp.registered(), true);
    const spec = await api.agents.mcp.spec();
    assert.equal(spec?.label, "Ascribe");
    assert.equal(spec?.command, api.binary()?.path);
    assert.deepEqual(spec?.args, ["mcp"]);
    assert.equal(spec?.cwd, workspace());
  });

  it("registers its three tools with VS Code", () => {
    const names = vscode.lm.tools.map((tool) => tool.name).filter((n) => n.startsWith("ascribe_"));
    assert.deepEqual(names.sort(), [
      "ascribe_editor_problems",
      "ascribe_review_changes",
      "ascribe_review_threads",
    ]);
  });

  it("says in a line that review is off, and doesn't turn it on", async () => {
    assert.equal(api.preview.review.base(workspace()), undefined);
    const threads = await invoke("ascribe_review_threads");
    const changes = await invoke("ascribe_review_changes");
    assert.match(threads, /^Review is off for [^\n]+\.$/);
    assert.equal(changes, threads);
    assert.equal(api.preview.review.base(workspace()), undefined);
  });

  it("reports a problem typed a moment before, unsaved, for that version", async () => {
    const editor = await vscode.window.showTextDocument(install);
    const end = editor.document.lineCount;
    await editor.edit((edit) =>
      edit.insert(new vscode.Position(end, 0), "\nSee [nothing](no-such-page.md).\n"),
    );
    // No wait: the tool waits for the server.
    const answer = JSON.parse(
      await invoke("ascribe_editor_problems", { path: "docs/install-agent.md" }),
    ) as {
      diagnostics: { slug: string; file: string; range: { start: { line: number } } }[];
      document_version: number;
      current: boolean;
      unsaved: string[];
      builds_checked: string[];
      schema_version: number;
    };
    assert.equal(answer.schema_version, 1);
    assert.equal(answer.current, true);
    assert.equal(answer.document_version, editor.document.version);
    assert.deepEqual(answer.unsaved, ["docs/install-agent.md"]);
    assert.equal(answer.builds_checked.length, 1);
    const broken = answer.diagnostics.find((d) => d.slug === "link-target-missing");
    assert.ok(broken, JSON.stringify(answer.diagnostics));
    assert.equal(broken.file, "docs/install-agent.md");
    assert.ok(broken.range.start.line > end - 1, JSON.stringify(broken.range));
    await vscode.commands.executeCommand("undo");
  });

  it("reports a problem written on disk a moment before, in a file that isn't open", async () => {
    const original = readFileSync(keys.fsPath, "utf8");
    try {
      writeFileSync(keys.fsPath, `${original}\nSee [nothing](no-such-page.md).\n`);
      const answer = JSON.parse(await invoke("ascribe_editor_problems", { path: keys.fsPath })) as {
        diagnostics: { slug: string }[];
        document_version: number | null;
        current: boolean;
      };
      assert.equal(answer.document_version, null);
      assert.equal(answer.current, true);
      assert.ok(answer.diagnostics.some((d) => d.slug === "link-target-missing"));
    } finally {
      writeFileSync(keys.fsPath, original);
      await diagnosticsOf(keys, (d) => d.length === 0);
    }
  });

  it("answers at once for a file written on disk that stays clean", async () => {
    // The usual case for an agent's edit: the server publishes for the file
    // even though its diagnostics didn't change, so the tool needn't wait out
    // its second.
    const original = readFileSync(keys.fsPath, "utf8");
    try {
      writeFileSync(keys.fsPath, `${original}\nOne more sentence.\n`);
      const began = Date.now();
      const answer = JSON.parse(await invoke("ascribe_editor_problems", { path: keys.fsPath })) as {
        diagnostics: unknown[];
        current: boolean;
      };
      const took = Date.now() - began;
      assert.equal(answer.current, true);
      assert.deepEqual(answer.diagnostics, []);
      assert.ok(took < 900, `answered in ${took} ms`);
    } finally {
      writeFileSync(keys.fsPath, original);
    }
  });

  it("publishes a change's diagnostics well under a second, typed or written on disk", async () => {
    // Copilot reads the Problems panel a second after its own edit.
    const editor = await vscode.window.showTextDocument(install);
    const end = editor.document.lineCount;
    let began = Date.now();
    await editor.edit((edit) =>
      edit.insert(new vscode.Position(end, 0), "\nSee [typed](typed-missing.md).\n"),
    );
    await diagnosticsOf(install, (d) => d.some((x) => x.message.includes("typed-missing")), 5_000);
    const typed = Date.now() - began;
    await vscode.commands.executeCommand("undo");

    const original = readFileSync(keys.fsPath, "utf8");
    began = Date.now();
    writeFileSync(keys.fsPath, `${original}\nSee [written](written-missing.md).\n`);
    try {
      await diagnosticsOf(keys, (d) => d.some((x) => x.message.includes("written-missing")), 5_000);
    } finally {
      const written = Date.now() - began;
      writeFileSync(keys.fsPath, original);
      console.log(`      change to diagnostics: typed ${typed} ms, written on disk ${written} ms`);
      assert.ok(typed < 500, `typed: ${typed} ms`);
      assert.ok(written < 1_000, `written on disk: ${written} ms`);
    }
    await diagnosticsOf(keys, (d) => d.length === 0);
  });

  it("lists the changed pages review shows, with their causes, once review is on", async () => {
    const started = await api.preview.review.start(workspace());
    assert.equal(started.problem, null);
    const answer = JSON.parse(await invoke("ascribe_review_changes")) as {
      base: { requested: string };
      pages: { file: string; path: string; because: string[]; counts: Record<string, number> }[];
    };
    assert.equal(answer.base.requested, "main");
    const listed = await api.preview.review.changedPages(workspace());
    assert.deepEqual(
      answer.pages.map((p) => [p.path, p.because]),
      listed.map((p) => [p.path, p.because]),
    );
    assert.equal(answer.pages[0]?.file, "docs/install-agent.md");
    // Without a pull request on GitHub, the threads tool says so.
    assert.match(
      await invoke("ascribe_review_threads"),
      /no open pull request|isn't signed in|hasn't been read/,
    );
  });
});

// The Review view on the same repository, its branch's pull request on a
// fake GitHub: the remote is added for the suite and taken away after it.
describe("the Review view, with a fake GitHub", () => {
  const install = uriOf("docs", "install-agent.md");
  const keys = uriOf("docs", "keys.md");
  const git = (...args: string[]) =>
    execFileSync("git", args, { cwd: workspace(), encoding: "utf8" }).trim();
  let api: AscribeApi;
  let github: StatefulGitHub;
  let seq = 2000;
  let firstThread: string;

  const settled = async () => {
    await api.ui.review.whenSettled();
    return api.ui.review;
  };
  /** The view's items as `[label, description, children]`, problems' children left out. */
  const shape = (items: SidebarItem[]): unknown[] =>
    items.map((item) => [
      item.label,
      item.description,
      item.label === "Problems on these pages" ? [] : shape(item.children),
    ]);
  const problems = (items: SidebarItem[]) =>
    items.find((item) => item.label === "Problems on these pages");

  before(async () => {
    api = await activated();
    await api.whenSettled();
    git("remote", "add", "origin", "https://github.com/acme/quill.git");
    github = new StatefulGitHub({
      number: 7,
      baseOid: git("rev-parse", "HEAD"),
      headOid: git("rev-parse", "HEAD"),
      files: ["docs/install-agent.md"],
    });
    firstThread = github.addThread("docs/install-agent.md", 8, "Which changes does it sync?").id;
    api.preview.review.threads.useTransport(() => github);
    await vscode.window.showTextDocument(await vscode.workspace.openTextDocument(install));
    await vscode.commands.executeCommand("ascribe.openPreview");
  });

  after(async () => {
    await api.preview.review.stop(workspace());
    api.preview.review.threads.useTransport(undefined);
    api.ui.review.sourceChanges.useGitExtension(true);
    git("remote", "remove", "origin");
    await vscode.commands.executeCommand("workbench.action.closeAllEditors");
  });

  it("isn't there while review is off", async () => {
    const view = await settled();
    assert.equal(view.project(), undefined);
    assert.deepEqual(view.items(), []);
  });

  it("lists the pages `ascribe diff` lists, with their counts and open threads", async () => {
    const started = await api.preview.review.start(workspace(), "main");
    assert.equal(started.problem, null);
    const view = await waitFor("the view's pages", async () => {
      const shown = await settled();
      return shown.items().length > 0 ? shown : undefined;
    });
    assert.equal(view.project(), workspace());
    const report = JSON.parse(
      execFileSync(api.binary()?.path ?? "ascribe", ["diff", "--format", "json"], {
        cwd: workspace(),
        encoding: "utf8",
      }),
    ) as { builds: { build: string; pages: PageDiff[] }[] };
    const listed = await api.preview.review.changedPages(workspace());
    const fromCli = report.builds.find((b) => b.pages.length > 0)?.pages ?? [];
    assert.deepEqual(
      listed.map((p) => [p.path, p.status, p.because, p.counts]),
      fromCli.map((p) => [p.path, p.status, p.because, p.counts]),
    );
    assert.deepEqual(shape(view.items()), [
      [
        "Changed",
        "1",
        [
          [
            "Install the Quill agent",
            `${pageDetail(listed[0] as ChangedPage)} · 1 open thread`,
            [],
          ],
        ],
      ],
      ["Problems on these pages", "none", []],
    ]);
    assert.equal(view.items()[0]?.children[0]?.opens, install.toString());
  });

  it("opens a page and its preview", async () => {
    await vscode.window.showTextDocument(await vscode.workspace.openTextDocument(keys));
    await api.preview.whenDrawn("keys.md", (r) => r.result.page?.path === "keys.md");
    const chosen = Date.now();
    const [page] = await api.preview.review.changedPages(workspace());
    await vscode.commands.executeCommand("ascribe.review.openPage", {
      kind: "page",
      page: { page, openThreads: 1 },
    });
    await waitFor(
      "the page in the editor",
      () => vscode.window.activeTextEditor?.document.uri.fsPath === install.fsPath,
    );
    await api.preview.whenDrawn(
      "the page in the preview",
      (r) => r.result.page?.path === "install-agent.md" && r.sentAt >= chosen,
    );
  });

  it("counts an unsent comment on its badge", async () => {
    assert.equal((await settled()).badge(), undefined);
    await api.preview.receive({
      type: "threads",
      id: ++seq,
      method: "comment",
      params: {
        anchor: { source: "install-agent.md:12-12", via: [] },
        body: "Say which repository.",
      },
    } as FromWebview);
    await vscode.commands.executeCommand("ascribe.refreshComments");
    const badge = await waitFor("the badge", async () => (await settled()).badge());
    assert.deepEqual(badge, { value: 1, tooltip: "1 comment you haven't submitted" });
  });

  it("shows the page's review in This page, and opens a thread in the preview", async () => {
    await vscode.commands.executeCommand("ascribe.thisPage.focus");
    const editor = await vscode.window.showTextDocument(install);
    editor.selection = new vscode.Selection(0, 0, 0, 0);
    const [page] = await api.preview.review.changedPages(workspace());
    const review = await waitFor("This page's Review section", async () => {
      await api.ui.thisPage.whenSettled();
      return api.ui.thisPage.items().find((item) => item.label === "Review");
    });
    assert.equal(review.description, "against main");
    assert.deepEqual(
      review.children[0] && [review.children[0].label, review.children[0].description],
      ["Changed", pageDetail(page as ChangedPage)],
    );
    const threads = review.children[1];
    assert.equal(threads?.label, "Open threads");
    assert.ok(
      threads.children.some(
        (t) =>
          t.label === "Which changes does it sync?" && t.description === "ana · install-agent.md:8",
      ),
      JSON.stringify(threads),
    );
    // Connections and Memos still follow, so the view's other sections don't move.
    assert.deepEqual(
      api.ui.thisPage.items().map((item) => item.label),
      ["About", "Review", "Connections", "Memos"],
    );

    await api.ui.thisPage.openThread(firstThread);
    await waitFor("the thread in the preview", () =>
      api.preview.threadsOpened().includes(firstThread),
    );
  });

  it("lists the problems on the changed pages", async () => {
    const editor = await vscode.window.showTextDocument(install);
    const end = editor.document.lineCount;
    await editor.edit((edit) =>
      edit.insert(new vscode.Position(end, 0), "\nSee [nothing](no-such-page.md).\n"),
    );
    try {
      await diagnosticsOf(install, (d) => d.some((x) => x.message.includes("no-such-page")));
      const listed = await waitFor("the problem in the view", async () => {
        const group = problems((await settled()).items());
        return group && group.children.length > 0 ? group : undefined;
      });
      assert.ok(
        listed.children.some((c) => c.label.includes("no-such-page")),
        JSON.stringify(listed),
      );
      assert.ok(listed.children.every((c) => c.description.startsWith("install-agent.md:")));
    } finally {
      // Undo acts on what has focus, which the preview can take as it goes to a thread.
      await vscode.window.showTextDocument(install);
      await vscode.commands.executeCommand("undo");
    }
    await diagnosticsOf(install, (d) => !d.some((x) => x.message.includes("no-such-page")));
  });

  it("opens the same source files with the Git extension and without it", async () => {
    const opened = async () => {
      const before = api.ui.review.sourceChanges.opened().length;
      await vscode.commands.executeCommand("ascribe.openSourceChanges");
      return waitFor("the diff", () => api.ui.review.sourceChanges.opened()[before]);
    };
    const baseTexts = async (diff: OpenedDiff) =>
      Promise.all(
        diff.files.map(async (file) =>
          file.original === null
            ? null
            : (await vscode.workspace.openTextDocument(vscode.Uri.parse(file.original))).getText(),
        ),
      );
    const withGit = await opened();
    assert.match(withGit.title, /^Source changes against main/);
    assert.deepEqual(
      withGit.files.map((f) => f.label),
      ["install-agent.md", "_fragments/prerequisites.md"],
    );
    assert.ok(
      withGit.files.every((f) => f.original?.startsWith("git:")),
      JSON.stringify(withGit.files),
    );
    const committed = withGit.files.map((f) => git("show", `HEAD:docs/${f.label}`));

    api.ui.review.sourceChanges.useGitExtension(false);
    const withoutGit = await opened();
    assert.deepEqual(
      withoutGit.files.map((f) => [f.label, f.modified]),
      withGit.files.map((f) => [f.label, f.modified]),
    );
    assert.ok(
      withoutGit.files.every((f) => f.original?.startsWith("ascribe-base:")),
      JSON.stringify(withoutGit.files),
    );
    assert.deepEqual(
      (await baseTexts(withoutGit)).map((t) => t?.replace(/\r\n/g, "\n")),
      committed.map((t) => `${t}\n`),
    );
    assert.deepEqual(await baseTexts(withGit), await baseTexts(withoutGit));
  });

  it("is gone once review stops", async () => {
    await vscode.commands.executeCommand("workbench.action.closeAllEditors");
    await vscode.window.showTextDocument(install);
    await vscode.commands.executeCommand("ascribe.stopReview");
    await waitFor("review to stop", () => api.preview.review.base(workspace()) === undefined);
    const view = await settled();
    assert.equal(view.project(), undefined);
    assert.deepEqual(view.items(), []);
    await waitFor("This page without its Review section", async () => {
      await api.ui.thisPage.whenSettled();
      const labels = api.ui.thisPage.items().map((item) => item.label);
      return labels.length > 0 && !labels.includes("Review");
    });
  });
});
