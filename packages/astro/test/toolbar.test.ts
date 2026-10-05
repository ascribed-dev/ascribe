// @vitest-environment jsdom
// What the toolbar app reads from the page and says about it, apart from
// the toolbar.
import { afterEach, describe, expect, it } from "vitest";
import type { LocatedThread } from "@ascribed/review/overlay";
import type { DiffPage, ThreadsState } from "../src/review/protocol.js";
import {
  anchorsArrived,
  anchoredBlocks,
  contentRoot,
  pageFile,
  pageScheme,
  sourceLocation,
} from "../src/toolbar/page.js";
import {
  againstText,
  buttonLabel,
  nextChangedPage,
  nextPageText,
  pageDetail,
  position,
  threadLocation,
  threadsNotice,
} from "../src/toolbar/text.js";

function page(html: string): Document {
  document.body.innerHTML = html;
  return document;
}

const SITE = `
<header><nav>Guides</nav></header>
<main>
  <h1>Set up</h1>
  <article>
    <p data-ascribe-source="Guides/My%20Setup.md:7-7">Intro.</p>
    <ul data-ascribe-source="_f/req.md:1-2" data-ascribe-via="Guides/My%20Setup.md:12">
      <li data-ascribe-source="_f/req.md:1-1">Node</li>
    </ul>
    <span data-ascribe-ui=""><p data-ascribe-source="Guides/My%20Setup.md:9-9">a mark</p></span>
  </article>
</main>`;

describe("the page", () => {
  it("finds the anchored blocks, leaving out review's own elements", () => {
    const blocks = anchoredBlocks(page(SITE));
    expect(blocks.map((b) => b.getAttribute("data-ascribe-source"))).toEqual([
      "Guides/My%20Setup.md:7-7",
      "_f/req.md:1-2",
      "_f/req.md:1-1",
    ]);
  });

  it("takes the element holding every block as the content", () => {
    const blocks = anchoredBlocks(page(SITE));
    expect(contentRoot(blocks)?.localName).toBe("article");
    // One block: its parent, not the block.
    const one = anchoredBlocks(page(`<div id="c"><p data-ascribe-source="a.md:1-1">x</p></div>`));
    expect(contentRoot(one)?.id).toBe("c");
    expect(contentRoot([])).toBeUndefined();
  });

  it("names the page's own file from a block written in it", () => {
    expect(pageFile(anchoredBlocks(page(SITE)))).toBe("Guides/My Setup.md");
    // Only included blocks: no file of its own to name.
    const included = page(`<p data-ascribe-source="_f/a.md:1-1" data-ascribe-via="p.md:3">x</p>`);
    expect(pageFile(anchoredBlocks(included))).toBeNull();
  });

  it("says the anchors arrived when most changed blocks are found", () => {
    const doc = page(SITE);
    const root = contentRoot(anchoredBlocks(doc)) as HTMLElement;
    const found = {
      kind: "changed" as const,
      now: { source: "Guides/My%20Setup.md:7-7", via: [] },
    };
    const lost = { kind: "added" as const, now: { source: "Guides/My%20Setup.md:30-30", via: [] } };
    expect(anchorsArrived(root, 3, [found])).toBe(true);
    expect(anchorsArrived(root, 3, [found, lost])).toBe(true);
    expect(anchorsArrived(root, 3, [found, lost, lost])).toBe(false);
    expect(anchorsArrived(root, 0, [found])).toBe(false);
    // Nothing to place: nothing missing.
    expect(anchorsArrived(root, 0, [])).toBe(true);
    // A removed block is placed by the block it came after.
    const removed = {
      kind: "removed" as const,
      after: { source: "Guides/My%20Setup.md:7-7", via: [] },
    };
    expect(anchorsArrived(root, 3, [removed])).toBe(true);
  });

  it("reads where a source is", () => {
    expect(sourceLocation("Guides/My%20Setup.md:7-9")).toEqual({
      path: "Guides/My Setup.md",
      line: 7,
    });
    expect(sourceLocation("nonsense")).toBeUndefined();
  });
});

function changed(path: string, init: Partial<Omit<DiffPage, "changes">> = {}) {
  return {
    path,
    route: `/docs/${path.replace(/\.md$/, "")}`,
    status: "changed" as const,
    own_file_changed: true,
    because: [],
    page_changed: [],
    counts: { changed: 2, added: 1, removed: 0, moved: 0 },
    ...init,
  };
}

describe("the page's color scheme", () => {
  afterEach(() => {
    document.documentElement.removeAttribute("style");
    document.body.removeAttribute("style");
  });

  it("follows the page's background, not the reader's system", () => {
    page(`<main id="m"><article id="a"><p>x</p></article></main>`);
    const article = document.getElementById("a") as HTMLElement;
    // A light-only site: no background, no color-scheme. Light even when the system is dark.
    expect(pageScheme(article, true)).toBe("light");
    document.body.style.backgroundColor = "rgb(20, 22, 28)";
    expect(pageScheme(article, false)).toBe("dark");
    // The nearest background that isn't see-through decides.
    (document.getElementById("m") as HTMLElement).style.backgroundColor =
      "rgba(255, 255, 255, 0.9)";
    expect(pageScheme(article, true)).toBe("light");
    (document.getElementById("m") as HTMLElement).style.backgroundColor =
      "rgba(255, 255, 255, 0.1)";
    expect(pageScheme(article, false)).toBe("dark");
  });

  it("takes the canvas from the root's color-scheme when nothing has a background", () => {
    page(`<p id="p">x</p>`);
    const p = document.getElementById("p") as HTMLElement;
    document.documentElement.style.colorScheme = "dark";
    expect(pageScheme(p, false)).toBe("dark");
    document.documentElement.style.colorScheme = "light dark";
    expect(pageScheme(p, false)).toBe("light");
    expect(pageScheme(p, true)).toBe("dark");
  });
});

describe("what the app says", () => {
  it("counts the changes and the place in them", () => {
    expect(position(0, -1)).toBe("No changes on this page");
    expect(position(1, -1)).toBe("1 change on this page");
    expect(position(10, 2)).toBe("3 of 10 on this page");
  });

  it("names the pull request's base as GitHub does", () => {
    const view = (requested: string, threads: ThreadsState) =>
      ({ base: { requested }, threads }) as unknown as Parameters<typeof againstText>[0];
    const on = {
      state: "on",
      pullRequest: { number: 128, url: "https://github.com/a/b/pull/128", baseRefName: "main" },
    } as unknown as ThreadsState;
    const off = { state: "none", message: "" } as unknown as ThreadsState;
    expect(againstText(view("origin/main", on))).toEqual({ pullRequest: "#128", base: "main" });
    expect(againstText(view("origin/main", off))).toEqual({
      pullRequest: null,
      base: "origin/main",
    });
  });

  it("puts the unsent count on the button", () => {
    expect(buttonLabel(0)).toBe("Ascribe review");
    expect(buttonLabel(2)).toBe("Ascribe review · 2 unsent comments");
  });

  it("describes a changed page in the list", () => {
    expect(pageDetail(changed("a.md"))).toBe("2 changed · 1 added");
    expect(pageDetail(changed("a.md", { status: "added" }))).toBe("new page");
    expect(
      pageDetail(
        changed("a.md", {
          counts: { changed: 0, added: 0, removed: 0, moved: 0 },
          page_changed: ["title"],
        }),
      ),
    ).toBe("title changed");
  });

  it("offers the next changed page, then the first, never a removed one", () => {
    const pages = [changed("a.md"), changed("b.md", { status: "removed" }), changed("c.md")];
    expect(nextChangedPage(pages, "a.md")).toEqual({ page: pages[2], first: false });
    expect(nextChangedPage(pages, "c.md")).toEqual({ page: pages[0], first: true });
    expect(nextChangedPage([changed("a.md")], "a.md")).toBeNull();
  });

  it("names the next changed page by its title, or its route without one", () => {
    const page = { ...changed("install.md"), title: "Install Loom" };
    expect(nextPageText({ page, first: false })).toBe("Next changed page: Install Loom");
    expect(nextPageText({ page: { ...page, title: null }, first: true })).toBe(
      "First changed page: /docs/install",
    );
  });

  it("says where a listed thread is by its file's name, and marks removed text", () => {
    const thread = (init: Partial<LocatedThread>) =>
      ({ kind: "review", side: "RIGHT", path: "Reference/Options.md", ...init }) as LocatedThread;
    expect(threadLocation(thread({ lines: { first: 16, last: 16 } }))).toEqual({
      label: "Options.md:16",
      removed: false,
    });
    expect(threadLocation(thread({ lines: { first: 3, last: 5 } })).label).toBe("Options.md:3-5");
    expect(threadLocation(thread({ side: "LEFT", lines: { first: 9, last: 9 } }))).toEqual({
      label: "Options.md",
      removed: true,
    });
    expect(threadLocation(thread({ lines: undefined })).label).toBe("Options.md");
  });

  it("says why there are no comments, and how the checkout differs", () => {
    const off: ThreadsState = { state: "gh-missing", message: "Install gh." };
    expect(threadsNotice(off)).toBe("Install gh.");
    const on = (state: "same" | "behind" | "ahead", behind = 0, ahead = 0): ThreadsState => ({
      state: "on",
      pullRequest: { number: 7, url: "https://github.com/acme/docs/pull/7", baseRefName: "main" },
      local: { state, behind, ahead },
    });
    expect(threadsNotice(on("same"))).toBeUndefined();
    expect(threadsNotice(on("behind", 2))).toContain("2 commits behind #7");
    expect(threadsNotice(on("ahead", 0, 1))).toContain("1 commit that isn't pushed");
  });
});
