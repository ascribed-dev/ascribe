// The stylesheets' generated blocks are current with design/tokens.toml, and
// no stylesheet writes a color anywhere else. Run with ASCRIBE_BLESS=1 to
// rewrite the blocks; then `pnpm --filter @ascribed/review embed` for the
// report's copy. design/README.md has the rest.

import { readFileSync, writeFileSync } from "node:fs";
import { format, resolveConfig } from "prettier";
import { describe, expect, test } from "vitest";
import { blank, declarations, sheet } from "./css.ts";
import { blocks, load, references, render, ROOT, source, type Tokens } from "./tokens.ts";

const BLESS = "ASCRIBE_BLESS=1 pnpm exec vitest run scripts/design/tokens.test.ts";

const read = (file: string): string => readFileSync(new URL(file, ROOT), "utf8");

/** `text` with its generated blocks rewritten, and formatted as the repository formats it. */
async function rendered(tokens: Tokens, file: string, text: string): Promise<string> {
  const out = render(tokens, file, text);
  if (!file.endsWith(".css")) return out;
  const path = new URL(file, ROOT).pathname;
  return format(out, { ...(await resolveConfig(path)), filepath: path });
}

// The CSS named colors. `transparent` and `currentColor` aren't among them,
// and neither are the system colors (`Canvas`, `ButtonFace`), which follow the
// reader's settings rather than naming a value.
const NAMED = new Set(
  `aliceblue antiquewhite aqua aquamarine azure beige bisque black blanchedalmond blue
  blueviolet brown burlywood cadetblue chartreuse chocolate coral cornflowerblue cornsilk
  crimson cyan darkblue darkcyan darkgoldenrod darkgray darkgreen darkgrey darkkhaki
  darkmagenta darkolivegreen darkorange darkorchid darkred darksalmon darkseagreen
  darkslateblue darkslategray darkslategrey darkturquoise darkviolet deeppink deepskyblue
  dimgray dimgrey dodgerblue firebrick floralwhite forestgreen fuchsia gainsboro ghostwhite
  gold goldenrod gray green greenyellow grey honeydew hotpink indianred indigo ivory khaki
  lavender lavenderblush lawngreen lemonchiffon lightblue lightcoral lightcyan
  lightgoldenrodyellow lightgray lightgreen lightgrey lightpink lightsalmon lightseagreen
  lightskyblue lightslategray lightslategrey lightsteelblue lightyellow lime limegreen linen
  magenta maroon mediumaquamarine mediumblue mediumorchid mediumpurple mediumseagreen
  mediumslateblue mediumspringgreen mediumturquoise mediumvioletred midnightblue mintcream
  mistyrose moccasin navajowhite navy oldlace olive olivedrab orange orangered orchid
  palegoldenrod palegreen paleturquoise palevioletred papayawhip peachpuff peru pink plum
  powderblue purple rebeccapurple red rosybrown royalblue saddlebrown salmon sandybrown
  seagreen seashell sienna silver skyblue slateblue slategray slategrey snow springgreen
  steelblue tan teal thistle tomato turquoise violet wheat white whitesmoke yellow
  yellowgreen`.split(/\s+/),
);

/** The colors `file` writes outside its generated blocks, as `file:line: …`. */
function literals(file: string, text: string): string[] {
  let outside = text;
  for (const { start, end } of blocks(text)) outside = blank(outside, start, end);
  const found: string[] = [];
  for (const { property, value, line } of declarations(sheet(file, outside))) {
    const bare = value.replace(/"[^"]*"|'[^']*'/g, "");
    const words = bare.match(/[\w-]+/g) ?? [];
    if (
      /#[0-9a-f]{3,8}\b/i.test(bare) ||
      /\b(rgba?|hsla?|hwb|lab|lch|oklab|oklch|color)\(/i.test(bare) ||
      words.some((word) => NAMED.has(word.toLowerCase()))
    ) {
      found.push(`${file}:${line}: ${property}: ${value}`);
    }
  }
  return found;
}

describe("the stylesheets", () => {
  const tokens = source();
  const files = [...tokens.emit.keys()];

  test.each(files)("%s is current with design/tokens.toml", async (file) => {
    const text = read(file);
    const expected = await rendered(tokens, file, text);
    if (process.env.ASCRIBE_BLESS === "1") {
      if (expected !== text) writeFileSync(new URL(file, ROOT), expected);
      return;
    }
    expect(text, `${file} is stale: run \`${BLESS}\``).toBe(expected);
  });

  test("write no color outside their generated blocks", () => {
    const found = files.flatMap((file) => literals(file, read(file)));
    expect(found, "Add the color to design/tokens.toml and the stylesheet's [emit]").toEqual([]);
  });
});

test("a color outside a generated block is found, by file and line", () => {
  const marker = "/* Generated from design/tokens.toml by scripts/design/tokens.ts: b, light. */";
  const css = [
    ":root {",
    `  ${marker}`,
    "  --a: #ffffff;",
    "  /* End of generated b. */",
    "  --b: var(--x, #fff);",
    "}",
    ".c { color: transparent; background: Canvas; border-color: currentColor; }",
    '.d::before { content: "red #abc"; color: rgb(0 0 0 / 0.5); }',
    ".e { outline: 1px solid white; }",
  ].join("\n");
  expect(literals("x.css", css)).toEqual([
    "x.css:5: --b: var(--x, #fff)",
    "x.css:8: color: rgb(0 0 0 / 0.5)",
    "x.css:9: outline: 1px solid white",
  ]);
  const ts = ["// A module.", "export const X_CSS = `", ".a {", "  color: #000;", "}", "`;"];
  expect(literals("x.ts", ts.join("\n"))).toEqual(["x.ts:4: color: #000"]);
});

test("changing a palette value changes the stylesheets that use it, and nothing else", () => {
  const text = read("design/tokens.toml");
  const tokens = load(text);
  const files = [...tokens.emit.keys()].map((file) => [file, read(file)] as const);
  const before = new Map(files.map(([file, css]) => [file, render(tokens, file, css)]));
  // A block writes its colors' light values, dark values, or both.
  const schemes = { light: ["light"], dark: ["dark"], "light-dark": ["light", "dark"] } as const;
  for (const [name, value] of tokens.palette) {
    const changed = { ...tokens, palette: new Map(tokens.palette).set(name, "#123456") };
    for (const [file, css] of files) {
      const uses = blocks(css).some(({ block, mode }) =>
        [...(tokens.emit.get(file)?.get(block)?.values() ?? [])].some((v) =>
          references(v).some((token) => {
            const color = tokens.colors.get(token);
            return color !== undefined && schemes[mode].some((scheme) => color[scheme] === name);
          }),
        ),
      );
      const after = render(changed, file, css);
      const diff = after !== before.get(file);
      if (diff !== uses)
        expect.fail(`${name} (${value}) in ${file}: changed ${diff}, uses ${uses}`);
      if (diff) expect(after.replaceAll("#123456", value)).toBe(before.get(file));
    }
  }
});

test("the source is checked", () => {
  const problems = (text: string): string => {
    try {
      load(text);
      return "";
    } catch (error) {
      return String(error);
    }
  };
  expect(problems('[palette.blue]\n1 = "#FFF"\n')).toContain("palette blue.1: write #rrggbb");
  const unused = problems(
    '[palette.blue]\n1 = "#ffffff"\n2 = "#000000"\n[color.a]\nlight = "blue.1"\ndark = "blue.3"\n' +
      '[color.b]\nlight = "blue.1"\ndark = "blue.1"\n[emit."x.css".b]\n"--a" = "color.a"\n"--b" = "{blue.1}"\n',
  );
  expect(unused).toContain('color.a.dark: "blue.3" isn\'t in the palette');
  expect(unused).toContain('emit x.css b --b: "blue.1" isn\'t a token');
  expect(unused).toContain("color.b isn't used by any stylesheet");
  expect(unused).toContain("palette blue.2 isn't used by any color");
});
