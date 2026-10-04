import { describe, expect, test } from "vitest";
import { encodePath, formatSource, parseSource } from "../src/place/anchor.js";

describe("anchors", () => {
  test("parse a block's source", () => {
    expect(parseSource("guides/install.md:12-14")).toEqual({
      path: "guides/install.md",
      first: 12,
      last: 14,
    });
  });

  test("decode percent-encoded segments and split at the last colon", () => {
    expect(parseSource("guides/a%20b%3Ac%C3%A9.md:3-3")).toEqual({
      path: "guides/a b:cé.md",
      first: 3,
      last: 3,
    });
  });

  test("reject what isn't a block's source", () => {
    for (const bad of [
      "guides/install.md",
      "guides/install.md:12",
      "x.md:5-4",
      "x.md:0-1",
      ":1-2",
    ]) {
      expect(parseSource(bad)).toBeUndefined();
    }
  });

  test("encode only what the grammar requires", () => {
    expect(encodePath("a b/c~d-e_f.g/é")).toBe("a%20b/c~d-e_f.g/%C3%A9");
    expect(formatSource({ path: "a b.md", first: 1, last: 2 })).toBe("a%20b.md:1-2");
  });
});
