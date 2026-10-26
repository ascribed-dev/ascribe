// What the Review view shows, and the files Open Source Changes opens (`src/ui/reviewTree.ts`).
import * as path from "node:path";
import { describe, expect, it } from "vitest";
import type { ChangedPage } from "../../src/preview/protocol.js";
import {
  badgeTooltip,
  diffTitle,
  pageDescription,
  reviewChildren,
  reviewLook,
  reviewRoots,
  shownProblems,
  sourceFiles,
  type PageProblem,
  type ReviewNode,
} from "../../src/ui/reviewTree.js";

function page(over: Partial<ChangedPage> = {}): ChangedPage {
  return {
    path: "install.md",
    route: "/install/",
    status: "changed",
    own_file_changed: true,
    because: [],
    page_changed: [],
    counts: { changed: 2, added: 1, removed: 0, moved: 0 },
    title: "Install",
    formatted_title: null,
    ...over,
  };
}

function problem(over: Partial<PageProblem> = {}): PageProblem {
  return {
    path: "install.md",
    severity: "error",
    message: "Broken link",
    line: 4,
    character: 0,
    ...over,
  };
}

const labels = (nodes: ReviewNode[]) => nodes.map((node) => reviewLook(node).label);

describe("reviewRoots", () => {
  const pages = [
    page({ path: "a.md", title: "A", status: "removed" }),
    page({ path: "b.md", title: "B" }),
    page({ path: "c.md", title: "C", status: "added" }),
    page({ path: "d.md", title: "D" }),
  ].map((p) => ({ page: p, openThreads: undefined }));

  it("groups the pages under Added, Changed, and Removed, in path order, then the problems", () => {
    const roots = reviewRoots(pages, [], false);
    expect(labels(roots)).toEqual(["Added", "Changed", "Removed", "Problems on these pages"]);
    const [added, changed, removed] = roots;
    expect(labels(reviewChildren(added as ReviewNode))).toEqual(["C"]);
    expect(labels(reviewChildren(changed as ReviewNode))).toEqual(["B", "D"]);
    expect(labels(reviewChildren(removed as ReviewNode))).toEqual(["A"]);
    expect(reviewLook(changed as ReviewNode).description).toBe("2");
  });

  it("leaves out a group with no pages, and shows nothing when no page changed", () => {
    expect(labels(reviewRoots(pages.slice(1, 2), [], false))).toEqual([
      "Changed",
      "Problems on these pages",
    ]);
    expect(reviewRoots([], [problem()], false)).toEqual([]);
  });

  it("lists errors and warnings, and advice only when it's shown", () => {
    const problems = [
      problem({ severity: "info", message: "Long sentence" }),
      problem({ severity: "warning", message: "Orphan", line: 0 }),
      problem({ path: "b.md", message: "Missing image", line: 9 }),
      problem({ message: "Broken link", line: 2 }),
    ];
    const [, group] = reviewRoots(pages.slice(1, 2), problems, false);
    expect(labels(reviewChildren(group as ReviewNode))).toEqual([
      "Missing image",
      "Broken link",
      "Orphan",
    ]);
    expect(reviewLook(group as ReviewNode).description).toBe("3");
    const [, withAdvice] = reviewRoots(pages.slice(1, 2), problems, true);
    expect(labels(reviewChildren(withAdvice as ReviewNode))).toEqual([
      "Missing image",
      "Broken link",
      "Orphan",
      "Long sentence",
    ]);
  });

  it("says the problems are on the pages, not made by the branch", () => {
    const [, group] = reviewRoots(pages.slice(1, 2), [], false);
    const look = reviewLook(group as ReviewNode);
    expect(look.label).toBe("Problems on these pages");
    expect(look.description).toBe("none");
    expect(look.expandable).toBe(false);
    expect(look.tooltip).toContain("whether this branch made them or they were there before");
  });

  it("places a problem by its page and line", () => {
    const look = reviewLook({ kind: "problem", problem: problem() });
    expect(look.description).toBe("install.md:5");
    expect(look.icon).toBe("error");
  });
});

describe("pageDescription", () => {
  it("says the page's counts, what it changed through, and its open threads", () => {
    expect(pageDescription({ page: page(), openThreads: undefined })).toBe("2 changed · 1 added");
    expect(pageDescription({ page: page(), openThreads: 0 })).toBe("2 changed · 1 added");
    expect(pageDescription({ page: page(), openThreads: 1 })).toBe(
      "2 changed · 1 added · 1 open thread",
    );
    expect(
      pageDescription({
        page: page({ own_file_changed: false, because: ["_fragments/requirements.md"] }),
        openThreads: 3,
      }),
    ).toBe("2 changed · 1 added · via _fragments/requirements.md · 3 open threads");
    expect(pageDescription({ page: page({ status: "added" }), openThreads: undefined })).toBe(
      "New page",
    );
  });
});

describe("shownProblems", () => {
  it("puts errors first, then warnings, then by page and place", () => {
    const shown = shownProblems(
      [
        problem({ severity: "warning", path: "a.md" }),
        problem({ path: "b.md", line: 1 }),
        problem({ path: "a.md", line: 3, character: 4 }),
        problem({ path: "a.md", line: 3, character: 1 }),
      ],
      false,
    );
    expect(shown.map((p) => `${p.severity} ${p.path}:${p.line}:${p.character}`)).toEqual([
      "error a.md:3:1",
      "error a.md:3:4",
      "error b.md:1:0",
      "warning a.md:4:0",
    ]);
  });
});

describe("badgeTooltip", () => {
  it("counts the comments not submitted, and is none at zero", () => {
    expect(badgeTooltip(undefined)).toBeUndefined();
    expect(badgeTooltip(0)).toBeUndefined();
    expect(badgeTooltip(1)).toBe("1 comment you haven't submitted");
    expect(badgeTooltip(4)).toBe("4 comments you haven't submitted");
  });
});

describe("sourceFiles", () => {
  const roots = {
    projectRoot: path.join("/repo", "site"),
    contentRoot: path.join("/repo", "site", "docs"),
  };
  const at = (...parts: string[]) => path.join("/repo", "site", ...parts);

  it("names each page's own file and what it changed through, once each", () => {
    const files = sourceFiles(
      [
        page({ path: "install.md", because: ["_fragments/requirements.md"] }),
        page({
          path: "upgrade.md",
          own_file_changed: false,
          because: ["_fragments/requirements.md", "code:app.py#main", "ascribe.toml"],
        }),
        page({ path: "new.md", status: "added" }),
        page({ path: "old.md", status: "removed" }),
      ],
      roots,
    );
    expect(files).toEqual([
      { label: "install.md", path: at("docs", "install.md"), before: true, after: true },
      {
        label: "_fragments/requirements.md",
        path: at("docs", "_fragments", "requirements.md"),
        before: true,
        after: true,
      },
      { label: "ascribe.toml", path: at("ascribe.toml"), before: true, after: true },
      { label: "new.md", path: at("docs", "new.md"), before: false, after: true },
      { label: "old.md", path: at("docs", "old.md"), before: true, after: false },
    ]);
  });

  it("compares a page that moved with its file where it was", () => {
    const moved_links = [{ from: "guides/configure.md", to: "guides/setup.md" }];
    const files = sourceFiles(
      [
        // Named first as a page a link changed through.
        page({ path: "guides/install.md", because: ["guides/setup.md"], moved_links }),
        page({
          path: "guides/setup.md",
          own_file_changed: false,
          moved_from: "guides/configure.md",
        }),
      ],
      roots,
    );
    expect(files).toEqual([
      {
        label: "guides/install.md",
        path: at("docs", "guides", "install.md"),
        before: true,
        after: true,
      },
      {
        label: "guides/setup.md",
        path: at("docs", "guides", "setup.md"),
        before: true,
        after: true,
        basePath: at("docs", "guides", "configure.md"),
      },
    ]);
  });

  it("names nothing without a content root", () => {
    expect(sourceFiles([page()], { projectRoot: null, contentRoot: null })).toEqual([]);
  });
});

describe("diffTitle", () => {
  it("names the base, and warns past the files VS Code's diff shows well", () => {
    expect(diffTitle("main", 30)).toBe("Source changes against main");
    expect(diffTitle("main", 31)).toBe(
      "Source changes against main (31 files: VS Code's diff may repeat headers past 30)",
    );
  });
});
