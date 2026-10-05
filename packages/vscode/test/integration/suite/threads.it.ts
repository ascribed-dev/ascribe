import * as assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import * as vscode from "vscode";
import type { PreviewApi, RenderRecord } from "../../../src/preview/controller.js";
import type { FromWebview } from "../../../src/preview/protocol.js";
import { StatefulGitHub } from "./fakeGitHub.js";
import { activated, uriOf, waitFor, workspace } from "./helpers.js";

// A pull request's review threads in the preview and the source editor, with
// the real `ascribe lsp`, on a copy of examples/quill made into a git
// repository whose `feature` branch rewords install-agent.md's first paragraph
// (line 7) and is pushed to github.com/acme/quill (`run.ts` prepares it).
// GitHub is a fake, kept in memory: the extension's sign-in is replaced by
// its transport.
describe("review threads, with a fake GitHub", () => {
  const install = uriOf("docs", "install-agent.md");
  const git = (...args: string[]) =>
    execFileSync("git", args, { cwd: workspace(), encoding: "utf8" }).trim();
  let preview: PreviewApi;
  let github: StatefulGitHub;
  let seq = 1000;

  const drawn = (description: string, predicate: (render: RenderRecord) => boolean) =>
    preview.whenDrawn(description, predicate);

  /** Sends one of the overlay's requests, as the webview would. */
  const request = (method: string, params: Record<string, unknown>) =>
    preview.receive({ type: "threads", id: ++seq, method, params } as FromWebview);

  before(async () => {
    const api = await activated();
    await api.whenSettled();
    preview = api.preview;
    github = new StatefulGitHub({
      number: 7,
      baseOid: git("rev-parse", "main"),
      headOid: git("rev-parse", "HEAD"),
      files: ["docs/install-agent.md"],
    });
    github.addThread("docs/install-agent.md", 7, "Which changes does it sync?");
    preview.review.threads.useTransport(() => github);
    await vscode.window.showTextDocument(await vscode.workspace.openTextDocument(install));
    await vscode.commands.executeCommand("ascribe.openPreview");
    await drawn("the page", (r) => r.result.page?.path === "install-agent.md");
  });

  after(async () => {
    preview.review.threads.useTransport(undefined);
    await vscode.commands.executeCommand("workbench.action.closeAllEditors");
  });

  it("starts review against the pull request's base, and names the pull request", async () => {
    const started = await preview.review.start(workspace());
    assert.equal(started.problem, null);
    assert.equal(started.base?.requested, "origin/main");
    assert.equal(preview.review.status().text, "$(git-compare) Review: #7 against main");
    const render = await drawn("the header", (r) =>
      (r.report?.reviewHeader ?? "").startsWith("#7 against"),
    );
    assert.match(render.report?.reviewHeader ?? "", /^#7 against main/);
  });

  it("shows a thread made on GitHub beside its block", async () => {
    const report = await waitFor("the thread beside its block", () =>
      preview.threadsDrawn().find((r) => Object.values(r.blocks).flat().length > 0),
    );
    const [source, ids] = Object.entries(report.blocks)[0] ?? [];
    assert.match(source ?? "", /^install-agent\.md:7-/);
    assert.deepEqual(ids, [github.threads[0]?.id]);
    assert.deepEqual(report.detached, []);
  });

  it("shows the same thread on its line in the source editor", async () => {
    const thread = await waitFor("the thread on its line", () =>
      preview.sourceThreads().find((t) => t.id === github.threads[0]?.id),
    );
    assert.equal(thread.file, install.fsPath);
    assert.equal(thread.line, 6);
    assert.deepEqual(thread.bodies, ["Which changes does it sync?"]);
  });

  it("holds a comment in the pending review, and sends it on submit", async () => {
    const before = github.calls.length;
    await request("comment", {
      anchor: { source: "install-agent.md:7-7", via: [] },
      body: "Say which repository.",
    });
    const added = github.calls.slice(before).find((c) => c.operation === "AddThread");
    assert.ok(added, "the comment reached GitHub as a review thread");
    assert.equal(added.variables["path"], "docs/install-agent.md");
    assert.equal(added.variables["line"], 7);
    // Pending: only the reviewer sees it until the review is submitted.
    assert.equal(github.reviews[0]?.state, "PENDING");
    assert.equal(
      github.calls.some((c) => c.operation === "SubmitReview"),
      false,
    );
    // The overlay reads the threads again after its own actions; this request
    // came from the test, so read them as Refresh Comments does.
    await vscode.commands.executeCommand("ascribe.refreshComments");
    const unsent = await waitFor("the overlay to count the unsent comment", () =>
      preview.threadsDrawn().find((r) => r.unsent === 1),
    );
    assert.equal(unsent.unsent, 1);
    await waitFor("the unsent comment in the source editor", () =>
      preview.sourceThreads().find((t) => t.bodies.includes("Say which repository.")),
    );

    await request("submit", { event: "COMMENT" });
    const submit = github.calls.find((c) => c.operation === "SubmitReview");
    assert.equal(submit?.variables["event"], "COMMENT");
    assert.equal(github.reviews[0]?.state, "COMMENTED");
  });

  it("sends a reply at once with Reply now", async () => {
    const id = github.threads[0]?.id;
    await request("reply", { threadId: id, body: "Thanks, fixed.", when: "now" });
    const reply = github.calls.find((c) => c.operation === "AddReply");
    assert.equal(reply?.variables["threadId"], id);
    assert.equal(reply?.variables["reviewId"], null);
    await waitFor("the reply in the source editor", () =>
      preview.sourceThreads().find((t) => t.id === id && t.bodies.includes("Thanks, fixed.")),
    );
  });

  it("drops the threads when review stops", async () => {
    await preview.review.stop(workspace());
    await waitFor("no threads in the source editor", () => preview.sourceThreads().length === 0);
    assert.equal(preview.review.status().text, "$(git-compare) Review: off");
  });

  it("offers review of the branch's pull request once, now that review was used here", async () => {
    await preview.review.offer(workspace());
    const offers = preview.review.offers();
    assert.equal(offers.at(-1), "Ascribe: this branch has pull request #7. Start Review?");
    // Once a session: showing the page again offers nothing more.
    await vscode.commands.executeCommand("workbench.action.closeAllEditors");
    await vscode.window.showTextDocument(await vscode.workspace.openTextDocument(install));
    assert.equal(preview.review.offers().length, offers.length);
  });
});
