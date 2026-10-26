import type { Element } from "hast";
import { describe, expect, it } from "vitest";
import { codeLanguages } from "../src/code-languages.js";

/** Shiki's <code>, through the transformer, for a block Shiki highlighted as `lang`. */
function transform(lang: string): Element {
  const code: Element = { type: "element", tagName: "code", properties: {}, children: [] };
  const context = {
    options: { lang },
    addClassToHast(node: Element, className: string) {
      const classes = Array.isArray(node.properties["className"])
        ? node.properties["className"]
        : [];
      node.properties["className"] = [...classes, className];
    },
  };
  codeLanguages.code?.call(context as never, code);
  return code;
}

describe("codeLanguages", () => {
  it("gives the <code> its language as a class, as CommonMark renders it", () => {
    expect(transform("sh").properties).toEqual({ className: ["language-sh"] });
    expect(transform("ts").properties).toEqual({ className: ["language-ts"] });
  });

  it("gives a block highlighted as plaintext none, as a block with no language has none", () => {
    expect(transform("plaintext").properties).toEqual({});
  });
});
