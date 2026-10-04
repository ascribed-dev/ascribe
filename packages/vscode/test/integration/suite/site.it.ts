import * as assert from "node:assert/strict";
import { mkdirSync, rmSync, writeFileSync } from "node:fs";
import { createServer, type Server } from "node:http";
import type { AddressInfo } from "node:net";
import * as path from "node:path";
import * as vscode from "vscode";
import type { PreviewApi } from "../../../src/preview/controller.js";
import { activated, uriOf, waitFor, workspace } from "./helpers.js";

// The site preview, with the real `ascribe lsp` on a copy of examples/quill
// and a fake dev server: an HTTP server that answers every page and records
// what it was asked for, named in `.ascribe/dev.json` as `astro dev` names
// itself.
describe("the site preview, against a fake dev server", () => {
  const install = uriOf("docs", "install-agent.md");
  const devFile = (): string => path.join(workspace(), ".ascribe", "dev.json");
  let preview: PreviewApi;
  let server: Server;
  let origin: string;
  /** What the fake dev server was asked for, and whether as a frame's page. */
  const requests: { url: string; frame: boolean }[] = [];

  before(async () => {
    server = createServer((request, response) => {
      requests.push({
        url: request.url ?? "",
        frame: request.headers["sec-fetch-dest"] === "iframe",
      });
      response.writeHead(200, { "content-type": "text/html" });
      response.end(`<!doctype html><title>Site</title><h1>${request.url ?? ""}</h1>`);
    });
    await new Promise<void>((resolve) => server.listen(0, "127.0.0.1", resolve));
    origin = `http://127.0.0.1:${(server.address() as AddressInfo).port}`;
    const api = await activated();
    await api.whenSettled();
    preview = api.preview;
    preview.site.captureExternal();
  });

  after(async () => {
    server.closeAllConnections();
    server.close();
    rmSync(devFile(), { force: true });
    await vscode.commands.executeCommand("workbench.action.closeAllEditors");
  });

  const writeDevFile = (url: string): void => {
    mkdirSync(path.dirname(devFile()), { recursive: true });
    writeFileSync(devFile(), JSON.stringify({ url, build: "site", pid: 1 }));
  };

  /** The page's route, from the page preview. */
  async function routeOf(uri: vscode.Uri): Promise<string> {
    await vscode.window.showTextDocument(await vscode.workspace.openTextDocument(uri));
    await vscode.commands.executeCommand("ascribe.openPreview");
    const render = await preview.whenDrawn("the page's render", (r) => r.result.page !== null);
    return render.result.page?.route ?? "";
  }

  it("says to start a dev server when there's none", async () => {
    await vscode.window.showTextDocument(await vscode.workspace.openTextDocument(install));
    rmSync(devFile(), { force: true });
    await vscode.commands.executeCommand("ascribe.openSitePreview");
    assert.match(preview.site.messages().at(-1) ?? "", /Start its dev server \(astro dev\)/);
    assert.deepEqual(preview.site.opened(), []);
  });

  it("ignores a dev.json whose server is gone", async () => {
    const gone = createServer();
    await new Promise<void>((resolve) => gone.listen(0, "127.0.0.1", resolve));
    const port = (gone.address() as AddressInfo).port;
    await new Promise((resolve) => gone.close(resolve));
    writeDevFile(`http://127.0.0.1:${port}/`);
    await vscode.window.showTextDocument(await vscode.workspace.openTextDocument(install));
    await vscode.commands.executeCommand("ascribe.openSitePreview");
    assert.match(preview.site.messages().at(-1) ?? "", /Start its dev server/);
    assert.deepEqual(preview.site.opened(), []);
  });

  it("opens the active page on the dev server, at the heading the editor shows", async () => {
    const route = await routeOf(install);
    assert.ok(route.startsWith("/"), `a route: ${route}`);
    writeDevFile(`${origin}/`);
    const editor = await vscode.window.showTextDocument(
      await vscode.workspace.openTextDocument(install),
      vscode.ViewColumn.One,
    );
    // The editor scrolled to the "Streaming sync" section.
    const line = editor.document
      .getText()
      .split("\n")
      .findIndex((l) => l.startsWith("## Streaming sync"));
    assert.ok(line > 0);
    editor.revealRange(
      new vscode.Range(line + 1, 0, line + 1, 0),
      vscode.TextEditorRevealType.AtTop,
    );
    // How far it scrolls depends on the window's height: expect the section
    // of whatever line ends up at the top, past the first heading.
    const top = await waitFor("the editor to scroll", () => {
      const first = editor.visibleRanges[0]?.start.line ?? 0;
      return first > 20 && first;
    });
    const sections = preview.renders().at(-1)?.result.page?.sections ?? [];
    const expected = sections.filter((s) => s.line <= top).at(-1)?.id;
    assert.ok(expected, `a heading above line ${top}`);
    await vscode.commands.executeCommand("ascribe.openSitePreview");
    const opened = await waitFor("the site preview to open", () => preview.site.opened().at(-1));
    const url = new URL(opened);
    assert.equal(url.origin, origin);
    assert.equal(url.pathname, route);
    assert.equal(url.hash, `#${expected}`);
  });

  it("opens the previewed page when the page preview is active", async () => {
    const route = await routeOf(install);
    writeDevFile(`${origin}/`);
    await vscode.commands.executeCommand("workbench.action.focusSecondEditorGroup");
    await waitFor("the preview to be active", () => vscode.window.activeTextEditor === undefined);
    const before = preview.site.opened().length;
    await vscode.commands.executeCommand("ascribe.openSitePreview");
    const opened = await waitFor(
      "the site preview to open",
      () => preview.site.opened().length > before && preview.site.opened().at(-1),
    );
    assert.equal(new URL(opened).pathname, route);
  });

  describe("in the preview panel", () => {
    const quickstart = uriOf("docs", "quickstart.md");

    it("says to start a dev server, and shows the site once there is one", async () => {
      const route = await routeOf(install);
      rmSync(devFile(), { force: true });
      await preview.site.selectSurface("site");
      const problem = await waitFor("the site's problem", () => {
        const last = preview.site.shown().at(-1);
        return last && "problem" in last ? last.problem : undefined;
      });
      assert.match(problem, /Start its dev server/);
      writeDevFile(`${origin}/`);
      await preview.site.selectSurface("site");
      const framed = await waitFor("the frame to load the page", () =>
        preview.site.framed().find((url) => new URL(url).pathname === route),
      );
      assert.equal(new URL(framed).origin, origin);
      // The webview framed it: the dev server got the frame's request.
      assert.ok(
        requests.some((r) => r.frame && r.url.startsWith(route)),
        `a frame request for ${route}: ${JSON.stringify(requests)}`,
      );
      assert.match(preview.shell() ?? "", new RegExp(`frame-src ${origin}`));
    });

    it("follows the active file", async () => {
      assert.equal(preview.site.surface(), "site");
      await vscode.window.showTextDocument(
        await vscode.workspace.openTextDocument(quickstart),
        vscode.ViewColumn.One,
      );
      const render = await preview.whenDrawn(
        "the quickstart's render",
        (r) => r.result.page?.path === "quickstart.md",
      );
      const route = render.result.page?.route ?? "";
      await waitFor("the frame to load the quickstart", () =>
        preview.site.framed().some((url) => new URL(url).pathname === route),
      );
    });

    it("goes back to the page", async () => {
      await preview.site.selectSurface("page");
      assert.equal(preview.site.surface(), "page");
    });
  });
});
