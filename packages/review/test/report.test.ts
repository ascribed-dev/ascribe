// @vitest-environment jsdom
import { beforeEach, describe, expect, it } from "vitest";
import {
  changedPages,
  errorsNotice,
  pageFragment,
  start,
  type BuildData,
  type ReportData,
} from "../src/report/index.js";

const page = (path: string, extra: Partial<ReportData["builds"][0]["pages"][0]> = {}) => ({
  path,
  route: `/${path.replace(/\.md$/, "")}/`,
  status: "changed" as const,
  own_file_changed: false,
  because: ["_fragments/prereqs.md"],
  page_changed: [],
  counts: { changed: 1, added: 0, removed: 0, moved: 0 },
  changes: [
    {
      kind: "changed" as const,
      now: { source: "_fragments/prereqs.md:1-1", via: [`${path}:3`] },
      was: { source: "_fragments/prereqs.md:1-1", via: [`${path}:3`] },
    },
  ],
  title: null,
  formatted_title: null,
  now: "p1",
  was: "p2",
  omitted: false,
  ...extra,
});

function data(): ReportData {
  return {
    ascribe_version: "0.0.0",
    base: { requested: "main", commit: "0123456789", merge_base: "0123456789" },
    working_tree_errors: 0,
    builds: [
      {
        build: "site",
        pages: [page("guide.md"), page("other.md", { omitted: true, now: null, was: null })],
      },
      { build: "cloud", pages: [] },
    ],
    pages: {
      p1: {
        html: '<p data-ascribe-source="_fragments/prereqs.md:1-1" data-ascribe-via="guide.md:3">Agent 2.4.</p><p data-ascribe-source="guide.md:5-5"><img src="./logo.png" alt="Logo"> <img src="https://example.com/a.png" alt="Remote"> <img src="./big.png" alt="Big"></p>',
        images: {
          "./logo.png": { path: "logo.png", image: "i1" },
          "./big.png": { path: "big.png", bytes: 3 * 1024 * 1024 },
        },
      },
      p2: {
        html: '<p data-ascribe-source="_fragments/prereqs.md:1-1" data-ascribe-via="guide.md:3">Agent 2.2.</p>',
        images: {},
      },
    },
    images: { i1: "data:image/png;base64,AAAA" },
    limit: { pages: 1, omitted: 1 },
    image_limit: 1024 * 1024,
  };
}

beforeEach(() => {
  document.body.innerHTML = "";
  location.hash = "";
});

describe("the report", () => {
  it("takes out of a page what could run script", () => {
    const report = data();
    report.pages.p1 = {
      html: [
        `<details open ontoggle="document.title='ran'"><summary onclick="x()">s</summary>x</details>`,
        `<a href=" JaVa\tScript:document.title='ran'">link</a>`,
        `<a href="https://example.com/">fine</a>`,
        `<form action="javascript:x()"><button formaction="javascript:x()">b</button></form>`,
        `<svg><a xlink:href="javascript:x()"><text>t</text></a></svg>`,
        `<script>document.title='ran'</script>`,
        `<meta http-equiv="refresh" content="0;url=https://example.com/">`,
        `<base href="https://example.com/">`,
        `<iframe srcdoc="<script>x()</script>"></iframe>`,
      ].join(""),
      images: {},
    };
    const holder = document.createElement("div");
    holder.append(pageFragment(report, "p1"));
    const html = holder.innerHTML;
    expect(html).not.toMatch(/\son\w+=/i);
    expect(html).not.toMatch(/javascript/i);
    expect(html).not.toMatch(/<(script|meta|base)\b/);
    expect(html).not.toMatch(/srcdoc/);
    expect(holder.querySelector("a[href='https://example.com/']")?.textContent).toBe("fine");
    expect(holder.querySelector("details")?.hasAttribute("open")).toBe(true);
  });

  it("inlines the images it holds and loads no others", () => {
    const fragment = pageFragment(data(), "p1");
    const imgs = Array.from(fragment.querySelectorAll("img"));
    expect(imgs.map((i) => i.getAttribute("src"))).toEqual(["data:image/png;base64,AAAA"]);
    const placeholders = Array.from(fragment.querySelectorAll(".r-image")).map(
      (e) => e.textContent,
    );
    expect(placeholders).toEqual([
      "Image not loaded: https://example.com/a.png (the report makes no network requests)",
      "Image not included: big.png (3.0 MB, over the report's 1.0 MB limit)",
    ]);
  });

  it("lists the changed pages, with what they change through, and marks the page", () => {
    const root = document.createElement("div");
    document.body.append(root);
    start(root, data());
    expect(root.querySelector(".r-title")?.textContent).toContain(
      "2 changed pages, compared with main (0123456)",
    );
    // One build has changes, so there's no picker.
    expect(root.querySelector("select")).toBeNull();
    expect(root.querySelector(".r-page-via")?.textContent).toBe("via _fragments/prereqs.md");
    expect(root.querySelector(".r-omitted")?.textContent).toContain("other.md");
    expect(root.querySelector(".r-head .r-notice")?.textContent).toContain("first 1 changed pages");
    const marked = root.querySelector('[data-ascribe-change="changed"]');
    expect(marked?.textContent).toContain("Agent 2.4.");
    expect(root.querySelector(".r-pos")?.textContent).toBe("1 change on this page");
  });

  it("shows a formatted title's code spans in the list and the heading", () => {
    const root = document.createElement("div");
    document.body.append(root);
    const report = data();
    const titled = page("guide.md", {
      title: "ascribe.toml reference",
      formatted_title: [
        { type: "code", value: "ascribe.toml" },
        { type: "text", value: " reference" },
      ],
    });
    start(root, {
      ...report,
      builds: report.builds.map((b, i) => (i === 0 ? { ...b, pages: [titled] } : b)),
    });
    for (const selector of [".r-page-title", ".r-page-heading"]) {
      const title = root.querySelector(selector);
      expect(title?.textContent).toBe("ascribe.toml reference");
      expect(title?.querySelector("code")?.textContent).toBe("ascribe.toml");
    }
  });

  it("says when the working tree has errors", () => {
    const root = document.createElement("div");
    document.body.append(root);
    start(root, { ...data(), working_tree_errors: 3, limit: { pages: 300, omitted: 0 } });
    const notice = root.querySelector(".r-head .r-notice");
    expect(notice?.textContent).toBe(
      "The working tree has 3 errors, so a page here may not render as it will once they're fixed. ascribe check lists them.",
    );
    expect(notice?.querySelector("code")?.textContent).toBe("ascribe check");
    expect(errorsNotice(1)).toContain("has 1 error, so");
    expect(errorsNotice(0)).toBeNull();
    // An older report has no count.
    expect(errorsNotice(undefined)).toBeNull();
  });

  it("says a page changed only in its frontmatter has no content changes", () => {
    const report = data();
    report.builds = [
      {
        build: "site",
        pages: [
          page("guide.md", {
            changes: [],
            counts: { changed: 0, added: 0, removed: 0, moved: 0 },
            page_changed: ["frontmatter"],
            because: [],
            own_file_changed: true,
          }),
        ],
      },
    ];
    report.limit = { pages: 300, omitted: 0 };
    const root = document.createElement("div");
    document.body.append(root);
    start(root, report);
    expect(root.querySelector(".r-quiet")?.textContent).toBe("No changes to the page's content");
    expect(root.querySelector(".r-pos")).toBeNull();
    expect(root.querySelector('[aria-label="Next change"]')).toBeNull();
    expect(root.querySelector(".r-controls .r-legend")?.textContent).toBe(
      "The page's frontmatter changed, which this render doesn't show.",
    );
  });

  it("switches what the page shows, and steps to the change", () => {
    const root = document.createElement("div");
    document.body.append(root);
    start(root, data());
    const show = (name: string) =>
      Array.from(root.querySelectorAll<HTMLButtonElement>(".r-seg button")).find(
        (b) => b.textContent === name,
      );
    show("As it was")?.click();
    expect(root.querySelector(".r-page")?.getAttribute("data-ascribe-show")).toBe("was");
    expect(show("As it was")?.getAttribute("aria-pressed")).toBe("true");
    root.querySelector<HTMLButtonElement>('[aria-label="Next change"]')?.click();
    expect(root.querySelector(".r-page")?.getAttribute("data-ascribe-show")).toBe("changes");
    expect(root.querySelector(".r-pos")?.textContent).toBe("1 of 1 on this page");
    root.querySelector<HTMLButtonElement>('[aria-label="Next change"]')?.click();
    expect(root.querySelector(".r-controls .r-notice")?.textContent).toContain(
      "That was the last change on this page.",
    );
  });

  it("copies a page's prompt, and offers none when the page has none", async () => {
    const report = data();
    const prompt = "Review what this change does to `docs/guide.md`.\n";
    report.builds[0] = { build: "site", pages: [page("guide.md", { prompt })] };
    report.limit = { pages: 300, omitted: 0 };
    const copied: string[] = [];
    Object.defineProperty(navigator, "clipboard", {
      configurable: true,
      value: { writeText: async (text: string) => void copied.push(text) },
    });
    try {
      const root = document.createElement("div");
      document.body.append(root);
      start(root, report);
      // Let the hash the report set settle first: a page change clears what it says.
      await new Promise((resolve) => setTimeout(resolve, 0));
      const copy = Array.from(root.querySelectorAll("button")).find(
        (b) => b.textContent === "Copy prompt",
      );
      copy?.click();
      await new Promise((resolve) => setTimeout(resolve, 0));
      expect(copied).toEqual([prompt]);
      expect(root.querySelector(".r-status")?.textContent).toBe(
        "Prompt copied. Paste it into your agent.",
      );
      // It only copies: nothing in the report opens an agent.
      expect(root.querySelector('a[href^="vscode:"], a[href^="cursor:"], a[href*="claude"]')).toBeNull();

      document.body.innerHTML = "";
      const without = document.createElement("div");
      document.body.append(without);
      start(without, data());
      expect(
        Array.from(without.querySelectorAll("button")).some((b) => b.textContent === "Copy prompt"),
      ).toBe(false);
    } finally {
      Reflect.deleteProperty(navigator, "clipboard");
    }
  });

  it("counts pages once across builds", () => {
    const d = data();
    expect(
      changedPages([d.builds[0] as BuildData, { build: "cloud", pages: [page("guide.md")] }]),
    ).toBe("2 changed pages in 2 builds");
  });
});
