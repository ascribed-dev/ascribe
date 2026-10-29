// What review says and where it points (`src/preview/reviewText.ts`).
import * as path from "node:path";
import { describe, expect, it } from "vitest";
import { errorsText } from "../../src/preview/counts.js";
import type { ChangedPage } from "../../src/preview/protocol.js";
import {
  baseCommit,
  baseName,
  causes,
  nextChangedPage,
  operationSpec,
  pageDetail,
  pageLabel,
  parseSource,
  removedPage,
  sameBase,
} from "../../src/preview/reviewText.js";

function page(over: Partial<ChangedPage> = {}): ChangedPage {
  return {
    path: "guides/install.md",
    route: "/guides/install/",
    status: "changed",
    own_file_changed: true,
    because: [],
    page_changed: [],
    counts: { changed: 5, added: 3, removed: 1, moved: 1 },
    title: "Install",
    formatted_title: null,
    ...over,
  };
}

describe("pageLabel", () => {
  it("writes a formatted title's code spans between backticks", () => {
    expect(pageLabel(page())).toBe("Install");
    expect(pageLabel(page({ title: null }))).toBe("guides/install.md");
    expect(
      pageLabel(
        page({
          title: "ascribe.toml reference",
          formatted_title: [
            { type: "code", value: "ascribe.toml" },
            { type: "text", value: " reference" },
          ],
        }),
      ),
    ).toBe("`ascribe.toml` reference");
    // A code span with backticks in it takes a longer fence.
    expect(pageLabel(page({ formatted_title: [{ type: "code", value: "`a` b" }] }))).toBe(
      "`` `a` b ``",
    );
  });
});

describe("pageDetail", () => {
  it("counts each kind of change, leaving out kinds with none", () => {
    expect(pageDetail(page())).toBe("5 changed · 3 added · 1 removed · 1 moved");
    expect(pageDetail(page({ counts: { changed: 1, added: 1, removed: 0, moved: 0 } }))).toBe(
      "1 changed · 1 added",
    );
  });

  it("names what the page changed through when its own file didn't", () => {
    expect(
      pageDetail(
        page({
          own_file_changed: false,
          because: ["_fragments/prereqs.md"],
          counts: { changed: 1, added: 0, removed: 0, moved: 0 },
        }),
      ),
    ).toBe("1 changed · via _fragments/prereqs.md");
  });

  it("says where a page moved from, and that a link changed because a page moved", () => {
    const none = { changed: 0, added: 0, removed: 0, moved: 0 };
    expect(
      pageDetail(
        page({ moved_from: "guides/configure.md", own_file_changed: false, counts: none }),
      ),
    ).toBe("Moved from guides/configure.md · no change to the page");
    expect(
      pageDetail(
        page({
          moved_from: "guides/configure.md",
          counts: { changed: 2, added: 0, removed: 0, moved: 0 },
        }),
      ),
    ).toBe("Moved from guides/configure.md · 2 changed");
    const moved_links = [{ from: "guides/configure.md", to: "guides/setup.md" }];
    const linking = {
      because: ["guides/setup.md"],
      moved_links,
      counts: { changed: 1, added: 0, removed: 0, moved: 0 },
    };
    expect(pageDetail(page({ ...linking, own_file_changed: false }))).toBe(
      "1 changed · a link changed because guides/configure.md moved",
    );
    // Its own file changed too: the page that moved is listed itself.
    expect(pageDetail(page(linking))).toBe("1 changed");
  });

  it("says a page is new or removed, and what about the page itself changed", () => {
    expect(pageDetail(page({ status: "added" }))).toBe("New page");
    expect(pageDetail(page({ status: "removed" }))).toBe("Removed");
    expect(
      pageDetail(
        page({ counts: { changed: 0, added: 0, removed: 0, moved: 0 }, page_changed: ["title"] }),
      ),
    ).toBe("title changed");
  });
});

describe("operationSpec", () => {
  it("is the spec of an operation, and null for any other cause", () => {
    expect(operationSpec("api/openapi.yaml#createWidget")).toBe("api/openapi.yaml");
    expect(operationSpec("../api/openapi.yaml#post/widgets")).toBe("../api/openapi.yaml");
    expect(operationSpec("_fragments/prereqs.md")).toBeNull();
    expect(operationSpec("notes/c#.md")).toBeNull();
    expect(operationSpec("code:app.py#main")).toBeNull();
    expect(operationSpec("ascribe.toml")).toBeNull();
  });
});

describe("causes", () => {
  const roots = {
    projectRoot: path.join("/w", "docs-project"),
    contentRoot: path.join("/w", "docs-project", "docs"),
  };

  it("points each cause at its file: a content path in the content root, ascribe.toml in the project", () => {
    expect(
      causes(
        page({
          own_file_changed: false,
          because: ["_fragments/prereqs.md", "api/openapi.yaml#createWidget", "ascribe.toml"],
        }),
        roots,
      ),
    ).toEqual([
      {
        label: "_fragments/prereqs.md",
        path: path.join("/w", "docs-project", "docs", "_fragments", "prereqs.md"),
      },
      {
        label: "api/openapi.yaml#createWidget",
        path: path.join("/w", "docs-project", "api", "openapi.yaml"),
      },
      { label: "ascribe.toml", path: path.join("/w", "docs-project", "ascribe.toml") },
    ]);
  });

  it("is empty when the page's own file changed", () => {
    expect(causes(page({ because: ["_fragments/prereqs.md"] }), roots)).toEqual([]);
    expect(causes(null, roots)).toEqual([]);
  });
});

describe("removedPage", () => {
  const base = { requested: "main", commit: "1a2b3c4d", merge_base: null };
  const was = {
    base,
    wasHtml: "<p>Gone.</p>",
    wasTitle: "Old",
    wasFormattedTitle: [{ type: "code" as const, value: "old" }],
  };
  const { title: _, formatted_title: __, ...diff } = page({ path: "old.md", status: "removed" });

  it("is the page as it was, for a removed page", () => {
    expect(removedPage({ ...was, changes: { ...diff, changes: [] } })).toEqual({
      path: "old.md",
      title: "Old",
      formattedTitle: [{ type: "code", value: "old" }],
      html: "<p>Gone.</p>",
    });
  });

  it("is null for a page that's still there, and without a review", () => {
    expect(
      removedPage({ ...was, changes: { ...diff, status: "changed", changes: [] } }),
    ).toBeNull();
    expect(removedPage({ ...was, changes: null })).toBeNull();
    expect(removedPage(null)).toBeNull();
  });
});

describe("nextChangedPage", () => {
  const pages = [
    page({ path: "a.md", title: "A" }),
    page({ path: "b.md", title: "B", status: "removed" }),
    page({ path: "c.md", title: "C" }),
  ];

  it("is the next page in path order, skipping removed ones", () => {
    expect(nextChangedPage(pages, "a.md")).toMatchObject({ page: { path: "c.md" }, first: false });
    expect(nextChangedPage(pages, "b2.md")).toMatchObject({ page: { path: "c.md" }, first: false });
  });

  it("goes back to the first past the last, and is null when no other page changed", () => {
    expect(nextChangedPage(pages, "c.md")).toMatchObject({ page: { path: "a.md" }, first: true });
    expect(nextChangedPage([page({ path: "a.md" })], "a.md")).toBeNull();
  });
});

describe("parseSource", () => {
  it("reads a block's anchor as a content path and lines from 0", () => {
    expect(parseSource("guides/my%20install.md:12-14")).toEqual({
      path: "guides/my install.md",
      first: 11,
      last: 13,
    });
  });

  it("has nothing for a malformed anchor", () => {
    expect(parseSource("guides/install.md:12")).toBeUndefined();
    expect(parseSource("guides/install.md:0-1")).toBeUndefined();
    expect(parseSource("guides/install.md:5-4")).toBeUndefined();
    expect(parseSource("%E0%A4%A:1-1")).toBeUndefined();
  });
});

describe("the base's commit", () => {
  const base = { requested: "main", commit: "c".repeat(40), merge_base: "1a2b3c4d5e" };

  it("is where the branch left the base, shortened", () => {
    expect(baseCommit(base)).toBe("1a2b3c4");
    expect(baseCommit({ ...base, merge_base: null })).toBe("ccccccc");
  });

  it("is named as GitHub names the pull request's base", () => {
    expect(baseName({ ...base, requested: "origin/main" }, "main")).toBe("main");
    expect(baseName({ ...base, requested: "origin/main" })).toBe("origin/main");
    expect(baseName({ ...base, requested: "v1.2" }, "main")).toBe("v1.2");
  });

  it("tells a base that moved from one that didn't", () => {
    expect(sameBase(base, { ...base })).toBe(true);
    expect(sameBase(base, { ...base, merge_base: "9f8e7d6" })).toBe(false);
    expect(sameBase(base, { ...base, commit: "d".repeat(40) })).toBe(false);
  });
});

describe("the project's errors", () => {
  it("are mentioned only when there are some", () => {
    expect(errorsText(0)).toBeUndefined();
    expect(errorsText(1)).toBe(
      "This project has 1 error, so a page may not show as it will once it's fixed.",
    );
    expect(errorsText(2)).toContain("has 2 errors");
  });
});
