import { describe, expect, test } from "vitest";
import { formatSection, joinSections, marker, parseSections } from "../src/github/marker.js";

describe("hidden anchor markers", () => {
  test("write the marker the design names", () => {
    expect(marker("guides/install.md:12-14", "site")).toBe(
      "<!-- ascribe:anchor guides/install.md:12-14 build=site -->",
    );
  });

  test("leave the build out of a comment that's on every build's page", () => {
    expect(marker("a.md:1-1", undefined)).toBe("<!-- ascribe:anchor a.md:1-1 -->");
    expect(marker("a.md:1-1", "")).toBe("<!-- ascribe:anchor a.md:1-1 -->");
  });

  test("round-trip sections, with the quote and link taken off", () => {
    const one = formatSection({
      source: "guides/install.md:12-14",
      build: "site",
      body: "Is this still true?",
      quote: "Install the CLI.\n\nThen run it.",
      link: {
        label: "guides/install.md, lines 12–14",
        url: "https://github.com/x/y/blob/abc/f#L12-L14",
      },
    });
    const two = formatSection({
      source: "a.md:1-1",
      build: undefined,
      body: "Typo.",
      quote: undefined,
      link: undefined,
    });
    const body = joinSections([one, two], "Overall looks good.");
    expect(body).toContain("> Install the CLI.\n>\n> Then run it.");
    const parsed = parseSections(body);
    expect(parsed.rest).toBe("Overall looks good.");
    expect(parsed.sections).toEqual([
      {
        source: "guides/install.md:12-14",
        build: "site",
        body: "Is this still true?",
        quote: "Install the CLI.\n\nThen run it.",
      },
      { source: "a.md:1-1", build: undefined, body: "Typo.", quote: undefined },
    ]);
  });

  test("a body without markers has no sections", () => {
    expect(parseSections("Just a comment.")).toEqual({ sections: [], rest: "Just a comment." });
  });
});
