import * as assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { existsSync, readFileSync } from "node:fs";
import * as path from "node:path";
import * as vscode from "vscode";
import { KEEP_ADDRESSES } from "../../../src/keepAddresses.js";
import type { SidebarItem } from "../../../src/ui/sidebarViews.js";
import { activated, uriOf, waitFor, workspace } from "./helpers.js";

// The real `ascribe lsp` on a copy of examples/astro-site made into a git
// repository on `main`, its ascribe.addresses taken away: the editor offers to keep
// the addresses after a move, then shows an address lost between checks, the
// old addresses that answer at a page, and what a move's undo leaves. The
// tests run in order, each from where the one before left the project.
describe("the addresses of examples/astro-site between checks", () => {
  const addresses = uriOf("ascribe.addresses");
  const setup = uriOf("content", "Guides", "Setup.md");
  const shared = uriOf("content", "Guides", "Teams", "Share a loom.md");

  function git(...args: string[]): void {
    execFileSync(
      "git",
      [
        "-c",
        "user.name=Test",
        "-c",
        "user.email=test@example.com",
        "-c",
        "commit.gpgsign=false",
        "-c",
        "core.autocrlf=false",
        ...args,
      ],
      { cwd: workspace() },
    );
  }

  /** The problems the panel has at ascribe.addresses, from the server or a check, as `code: message`. */
  async function problemsAtAddresses(): Promise<string[]> {
    const api = await activated();
    return api.ui.everyBuild
      .problems()
      .filter((p) => path.basename(p.file) === "ascribe.addresses")
      .map((p) => `${p.code}: ${p.message}`);
  }

  /** Waits for the panel's problems at ascribe.addresses to satisfy `wanted`. */
  async function problemsThere(
    what: string,
    wanted: (problems: string[]) => boolean,
    timeout?: number,
  ): Promise<string[]> {
    return waitFor(
      what,
      async () => {
        const problems = await problemsAtAddresses();
        return wanted(problems) ? problems : undefined;
      },
      timeout,
    ).catch(() => problemsAtAddresses());
  }

  /** Health's count of addresses lost, as `state: label value`. */
  async function againstTheBase(): Promise<string | undefined> {
    const api = await activated();
    const group = api.ui.project.answer()?.health.find((g) => g.key === "addresses");
    const count = group?.counts[0];
    return count && `${count.state}: ${count.label} ${count.value}`;
  }

  const documentOf = (uri: vscode.Uri): vscode.TextDocument | undefined =>
    vscode.workspace.textDocuments.find((d) => d.uri.toString() === uri.toString());

  /** This page's rows, flattened, as `label: description`. */
  function rows(items: SidebarItem[]): string[] {
    return items.flatMap((item) => [`${item.label}: ${item.description}`, ...rows(item.children)]);
  }

  after(async () => {
    await vscode.commands.executeCommand("workbench.action.closeAllEditors");
  });

  /** Moves a page with Pages' Rename or Move…, and waits for the offer to keep the addresses to be handled. */
  async function move(from: string, to: string): Promise<void> {
    const api = await activated();
    const seen = api.ui.keepAddresses.seen();
    const moved = await api.ui.sidebar.act("renameOrMove", from, [to]);
    assert.equal(moved.done, true, moved.messages.join());
    await waitFor("the move seen", () => api.ui.keepAddresses.seen() > seen);
    await api.ui.keepAddresses.whenDone();
    // The server learns of a move from the file watcher, a second or so later.
    await waitFor(`${to} in Pages`, async () => {
      await api.ui.sidebar.whenSettled();
      const listed = rows(api.ui.sidebar.items("pages"));
      return listed.some((r) => r.endsWith(to)) && !listed.some((r) => r.endsWith(from));
    });
  }

  it("offers to keep the addresses after a page's move, once, and records the move", async () => {
    const api = await activated();
    await vscode.commands.executeCommand("ascribe.project.focus");
    await vscode.window.showTextDocument(uriOf("content", "index.md"));
    await waitFor("the Pages view", async () => {
      await api.ui.sidebar.whenSettled();
      return rows(api.ui.sidebar.items("pages")).some((r) => r.endsWith("Guides/My Setup.md"));
    });
    api.ui.keepAddresses.answerNext(KEEP_ADDRESSES);
    await move("Guides/My Setup.md", "Guides/Setup.md");
    assert.equal(api.ui.keepAddresses.offered().length, 1);
    assert.match(
      readFileSync(addresses.fsPath, "utf8"),
      /"Guides\/My Setup\.md" = "Guides\/Setup\.md"/,
    );

    // The commands wrote the file outside the editor: the next move edits it
    // once the server, and VS Code's file watcher with it, has seen it.
    await waitFor("the addresses compared", () => {
      const compared = api.ui.everyBuild.addresses(workspace());
      return compared.kind === "made" && compared.lost === 0;
    });

    // Another move isn't offered again: the file records it.
    api.ui.keepAddresses.answerNext(undefined);
    await move("Reference/Options.md", "Reference/Settings.md");
    assert.equal(api.ui.keepAddresses.offered().length, 1);
  });

  it("reports nothing lost once the moves are recorded, and Health says so", async () => {
    // The second move, made once the file existed, is the server's edit to
    // it, unsaved.
    const document = documentOf(addresses);
    assert.ok(document?.isDirty, "ascribe.addresses isn't edited");
    assert.match(document.getText(), /"Reference\/Options\.md" = "Reference\/Settings\.md"/);
    assert.ok(await document.save(), "ascribe.addresses wasn't saved");
    await vscode.workspace.saveAll(false);
    assert.equal(readFileSync(addresses.fsPath, "utf8"), document.getText());
    assert.deepEqual(await problemsThere("no problems", (p) => p.length === 0), []);
    const health = await waitFor("Health's count", async () => {
      const said = await againstTheBase();
      return said?.startsWith("none") ? said : undefined;
    }).catch(() => againstTheBase());
    assert.equal(health, "none: Addresses lost 0");
    git("add", "-A");
    git("commit", "-q", "-m", "Keep the addresses");
  });

  it("reports a deleted page's lost address within seconds, with no check run", async () => {
    const edit = new vscode.WorkspaceEdit();
    edit.deleteFile(shared);
    assert.ok(await vscode.workspace.applyEdit(edit));
    const problems = await problemsThere("the lost address", (p) => p.length > 0, 10_000);
    assert.equal(problems.length, 1, problems.join("\n"));
    assert.match(problems[0] ?? "", /Guides\/Teams\/Share a loom\.md/);
    const health = await waitFor("Health's count", async () => {
      const said = await againstTheBase();
      return said?.startsWith("current") ? said : undefined;
    }).catch(() => againstTheBase());
    assert.equal(health, "current: Addresses lost 1 page");
  });

  it("replaces the problems with Check Every Build's, without the same one twice", async () => {
    const api = await activated();
    const before = await problemsAtAddresses();
    await api.ui.project.choose("checkEveryBuild");
    await api.ui.everyBuild.whenDone();
    assert.deepEqual(await problemsAtAddresses(), before);
    git("checkout", "--", "content/Guides/Teams/Share a loom.md");
    assert.deepEqual(await problemsThere("nothing lost again", (p) => p.length === 0), []);
  });

  it("lists the old address that answers at a page in This page", async () => {
    const api = await activated();
    await vscode.commands.executeCommand("ascribe.thisPage.focus");
    await vscode.window.showTextDocument(setup);
    const shown = await waitFor("Also answers at", async () => {
      await api.ui.thisPage.whenSettled();
      const found = rows(api.ui.thisPage.items()).filter((r) => r.startsWith("Also answers at"));
      return found.length > 0 ? found : undefined;
    });
    assert.deepEqual(shown, [
      "Also answers at: /docs/guides/my-setup",
      "Also answers at: /docs/guides/my-setup.md",
    ]);
  });

  it("puts a move's entry in the same edit, which undo takes back before it's saved", async () => {
    const api = await activated();
    const limits = uriOf("content", "Reference", "Limits.md");
    const before = readFileSync(addresses.fsPath, "utf8");
    const moved = await api.ui.sidebar.act("renameOrMove", "Reference/Settings.md", [
      "Reference/Limits.md",
    ]);
    assert.equal(moved.done, true, moved.messages.join());
    const document = await waitFor("ascribe.addresses changed", () => {
      const d = documentOf(addresses);
      return d?.isDirty ? d : undefined;
    });
    assert.match(document.getText(), /"Reference\/Settings\.md" = "Reference\/Limits\.md"/);
    // Undone in ascribe.addresses: the move goes with it, as one edit.
    await vscode.window.showTextDocument(document);
    await vscode.commands.executeCommand("undo");
    await waitFor("the move undone", () => !existsSync(limits.fsPath));
    await waitFor("the entry undone", () => documentOf(addresses)?.getText() === before);
    await vscode.workspace.saveAll(false);
    assert.deepEqual(await problemsThere("no problems", (p) => p.length === 0), []);
  });

  it("reports an entry left after a page moves back once it's saved", async () => {
    const api = await activated();
    const moved = await api.ui.sidebar.act("renameOrMove", "Reference/Settings.md", [
      "Reference/Limits.md",
    ]);
    assert.equal(moved.done, true, moved.messages.join());
    await vscode.workspace.saveAll(false);
    // Moved back without the editor, as `git mv` would: the entry stays.
    await vscode.workspace.fs.rename(
      uriOf("content", "Reference", "Limits.md"),
      uriOf("content", "Reference", "Settings.md"),
    );
    const problems = await problemsThere("the entry left", (p) => p.length > 0, 10_000);
    assert.equal(problems.length, 1, problems.join("\n"));
    assert.match(problems[0] ?? "", /Reference\/Settings\.md/);
  });
});
