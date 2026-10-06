import type { Root } from "hast";
import { describe, expect, it } from "vitest";
import { codeTitle, codeTitles } from "../src/code-titles.js";

/** Shiki's output for a code block: a root holding its <pre>. */
function highlighted(): Root {
  return {
    type: "root",
    children: [
      {
        type: "element",
        tagName: "pre",
        properties: { className: ["shiki"] },
        children: [{ type: "text", value: "export default {};" }],
      },
    ],
  };
}

function transform(meta: string | undefined, root: Root): Root {
  const context = { options: { meta: meta === undefined ? undefined : { __raw: meta } } };
  codeTitles.root?.call(context as never, root);
  return root;
}

describe("codeTitle", () => {
  it('reads title="…" from the meta, among other words', () => {
    expect(codeTitle('title="astro.config.mjs"')).toBe("astro.config.mjs");
    expect(codeTitle('{1-3} title="a b.ts" wrap')).toBe("a b.ts");
    expect(codeTitle("")).toBeUndefined();
    expect(codeTitle('subtitle="x"')).toBeUndefined();
    expect(codeTitle('title=""')).toBeUndefined();
    // A quote in the title, which arrives with its backslash taken off, and a backslash.
    expect(codeTitle('title="say "hi""')).toBe('say "hi"');
    expect(codeTitle('title="say "hi"" wrap')).toBe('say "hi"');
    expect(codeTitle('title="C:\\dir"')).toBe("C:\\dir");
  });
});

describe("codeTitles", () => {
  it("puts a titled block in a figure with the title as its caption", () => {
    const root = transform('title="astro.config.mjs"', highlighted());
    expect(root.children).toEqual([
      {
        type: "element",
        tagName: "figure",
        properties: { className: ["code-title"] },
        children: [
          {
            type: "element",
            tagName: "figcaption",
            properties: {},
            children: [{ type: "text", value: "astro.config.mjs" }],
          },
          highlighted().children[0],
        ],
      },
    ]);
  });

  it("leaves a block without a title as it is", () => {
    expect(transform(undefined, highlighted())).toEqual(highlighted());
    expect(transform("wrap", highlighted())).toEqual(highlighted());
  });

  it("doesn't wrap a block a site's own transformer already put in a figure", () => {
    const once = transform('title="a.ts"', highlighted());
    const before = structuredClone(once);
    expect(transform('title="a.ts"', once)).toEqual(before);
  });
});
