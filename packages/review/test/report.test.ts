// @vitest-environment jsdom
import { beforeEach, describe, expect, it } from "vitest";
import {
  changedPages,
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
  now: "p1",
  was: "p2",
  omitted: false,
  ...extra,
});

function data(): ReportData {
  return {
    ascribe_version: "0.0.0",
    base: { requested: "main", commit: "0123456789", merge_base: "0123456789" },
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

  it("counts pages once across builds", () => {
    const d = data();
    expect(
      changedPages([d.builds[0] as BuildData, { build: "cloud", pages: [page("guide.md")] }]),
    ).toBe("2 changed pages in 2 builds");
  });
});
