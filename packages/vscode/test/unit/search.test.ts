import { describe, expect, it } from "vitest";
import type { SearchPlace, SearchResult } from "../../src/shapes.js";
import { excerpt, searchItem, searchItems, searchPlaceholder } from "../../src/ui/searchItems.js";

const LONG =
  "Sign in to Quill Cloud and copy an API key from Settings → Keys, then add it to the agent's configuration file under the server key.";

function place(extra: Partial<SearchPlace> = {}): SearchPlace {
  return {
    uri: "file:///work/quill/docs/install-agent.md",
    range: { start: { line: 61, character: 0 }, end: { line: 61, character: 90 } },
    file: "install-agent.md",
    in: "text",
    text: LONG,
    match: [0, 22],
    through: [{ phrase: "cloud" }],
    arm: "deployment=cloud",
    pages: [{ page: "install-agent.md", heading: "Connect to Quill", id: "connect", builds: [] }],
    pagesTotal: 1,
    builds: [],
    ...extra,
  };
}

function result(extra: Partial<SearchResult> = {}): SearchResult {
  return {
    builds: ["site"],
    places: [],
    total: 0,
    truncated: false,
    nothing: null,
    problem: null,
    ...extra,
  };
}

describe("an item of Search Pages as Readers See Them", () => {
  it("is the reader's text cut around the match, and the command's second line", () => {
    const item = searchItem(place());
    expect(item.label).toBe(
      "Sign in to Quill Cloud and copy an API key from Settings → Keys, then add it to…",
    );
    expect(item.description).toBe("install-agent.md:62 · Connect to Quill · through phrase cloud");
    expect(item.detail).toBe("deployment=cloud arm");
  });

  it("is a phrase's value, with how many places use it", () => {
    const item = searchItem(
      place({
        uri: "file:///work/quill/ascribe.toml",
        range: { start: { line: 33, character: 0 }, end: { line: 33, character: 21 } },
        file: "ascribe.toml",
        in: "phrase",
        text: "Quill Cloud",
        match: [0, 11],
        arm: null,
        pages: [],
        pagesTotal: 5,
      }),
    );
    expect(item.label).toBe("Quill Cloud");
    expect(item.description).toBe("ascribe.toml:34 · phrase cloud");
    expect(item.detail).toBe("5 places use it");
  });

  it("says how many pages show a fragment's place, and not one page's heading", () => {
    const item = searchItem(
      place({
        file: "_fragments/prerequisites.md",
        text: "A docs repository.",
        match: [2, 6],
        through: [],
        arm: null,
        pagesTotal: 30,
      }),
    );
    expect(item.description).toBe("_fragments/prerequisites.md:62");
    expect(item.detail).toBe("on 30 pages");
  });

  it("names the builds a place is in when it isn't in all of them", () => {
    const item = searchItem(place({ arm: null, through: [], builds: ["site", "site@3.4"] }));
    expect(item.detail).toBe("in builds site, site@3.4");
  });

  it("says what a heading, a borrowed title, and a snippet's code read through", () => {
    expect(
      searchItem(place({ in: "heading", text: "Connect", match: [0, 7], through: [], arm: null }))
        .description,
    ).toBe("install-agent.md:62");
    expect(
      searchItem(place({ through: [{ title: "keys.md#create-key" }], arm: null })).description,
    ).toBe("install-agent.md:62 · Connect to Quill · through keys.md#create-key's title");
    expect(
      searchItem(
        place({ file: "examples/app.py", through: [{ code: "code:app.py#main" }], arm: null }),
      ).description,
    ).toBe("examples/app.py:62 · Connect to Quill · through code:app.py#main");
  });
});

describe("the items of a search", () => {
  it("are one per place, then how many when they're cut", () => {
    const items = searchItems(result({ places: [place(), place()], total: 120, truncated: true }));
    expect(items).toHaveLength(3);
    expect(items[2]).toEqual({
      label: "Showing 2 of 120 places. Type more to narrow them.",
      description: "",
      detail: "",
    });
    expect(items[2]?.place).toBeUndefined();
  });

  it("are the command's last line when nothing matched", () => {
    const items = searchItems(
      result({ nothing: "Builds not searched publish them: `site`, `site@3.4`." }),
    );
    expect(items.map((i) => i.label)).toEqual([
      "Builds not searched publish them: `site`, `site@3.4`.",
    ]);
  });

  it("are the problem when the query can't be read", () => {
    const items = searchItems(
      result({ problem: "The query can't be read: a quote in `\"sign in` isn't closed." }),
    );
    expect(items).toHaveLength(1);
    expect(items[0]?.place).toBeUndefined();
  });
});

describe("the placeholder", () => {
  it("names the build searched, or every build", () => {
    expect(searchPlaceholder("cloud")).toBe(
      "Words, or a phrase in quotes, as readers read them in the build cloud",
    );
    expect(searchPlaceholder(undefined)).toBe(
      "Words, or a phrase in quotes, as readers read them in every build",
    );
  });
});

describe("an excerpt", () => {
  it("is the whole text when it's short", () => {
    expect(excerpt("Sign in to Quill Cloud.", [0, 4])).toBe("Sign in to Quill Cloud.");
  });

  it("starts at a word before a match far into the text", () => {
    const text = `${"word ".repeat(40)}target ${"more ".repeat(40)}`.trim();
    const at = text.indexOf("target");
    const cut = excerpt(text, [at, at + 6]);
    expect(cut.startsWith("…word")).toBe(true);
    expect(cut.endsWith("…")).toBe(true);
    expect(cut).toContain("target");
    expect(Array.from(cut).length).toBeLessThanOrEqual(82);
  });

  it("keeps to the match's first word when the match spans a long block", () => {
    const text = `${"word ".repeat(40)}first ${"more ".repeat(3000)}last`;
    const at = text.indexOf("first");
    const cut = excerpt(text, [at, Array.from(text).length]);
    expect(cut).toContain("first");
    expect(cut.endsWith("…")).toBe(true);
    expect(Array.from(cut).length).toBeLessThanOrEqual(82);
  });

  it("counts characters, not UTF-16 units", () => {
    const text = `${"😀 ".repeat(50)}match`;
    const chars = Array.from(text);
    const cut = excerpt(text, [chars.length - 5, chars.length]);
    expect(cut.endsWith("match")).toBe(true);
  });
});
