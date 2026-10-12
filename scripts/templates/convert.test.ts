// The page templates' conversion (convert.ts). The copies in
// crates/ascribe-templates/set/ are what the pinned release gives: a stale copy
// fails here, and ASCRIBE_BLESS=1 rewrites them, then read the diff. That test
// needs their repository and an `ascribe` binary (ASCRIBE_BIN, or this
// repository's debug build), and is skipped, saying so, without either;
// rust.yml runs it after its build (`node scripts/templates/convert.ts
// --check`). The copies passing `ascribe check` and `ascribe fmt` is
// crates/ascribe-templates/tests/all/set.rs, which needs neither.
import { expect, test } from "vitest";
import {
  binary,
  ConversionError,
  convertGuide,
  convertTemplate,
  escapeBraces,
  generate,
  readPin,
  readRules,
  stale,
  Unreachable,
  write,
} from "./convert.ts";

const rules = { description: "{Say what it's for.}", directives: [] };

function template(body: string): string {
  return `# {Product} title\n\n> Read the [guide](./guide.md).\n>\n> Text in {curly brackets}.\n\n${body}\n\n---\n\n> Explore other templates.\n`;
}

test("the title goes to frontmatter, and what the template says about itself goes", () => {
  expect(convertTemplate("t", template("## Overview\n\nThis explains {the task}."), rules)).toBe(
    [
      "---",
      'title: "{Product} title"',
      `description: "{Say what it's for.}"`,
      "---",
      "",
      "## Overview",
      "",
      "This explains \\{the task}.",
      "",
    ].join("\n"),
  );
});

test("a brace in a code span or a fenced block isn't escaped", () => {
  const lines = [
    "Run `echo {a}` and ``x ` {b}`` with {c}.",
    "",
    "```json",
    '{ "d": 1 }',
    "```",
    "",
    "~~~~",
    "```",
    "{e}",
    "~~~~",
    "A span `across",
    "{f}` lines, and \\{g} escaped already, and an unclosed ` {h}.",
  ];
  expect(escapeBraces(lines)).toEqual([
    "Run `echo {a}` and ``x ` {b}`` with \\{c}.",
    "",
    "```json",
    '{ "d": 1 }',
    "```",
    "",
    "~~~~",
    "```",
    "{e}",
    "~~~~",
    "A span `across",
    "{f}` lines, and \\{g} escaped already, and an unclosed ` \\{h}.",
  ]);
});

test("a placeholder across blocks has only its opening brace escaped", () => {
  expect(escapeBraces(["{Include links.", "", "* One", "* Two}"])).toEqual([
    "\\{Include links.",
    "",
    "* One",
    "* Two}",
  ]);
});

test("a directive goes before the line its rule names, at its indentation", () => {
  const page = convertTemplate("t", template("Steps:\n\n  1. {Do it.}\n  2. Then."), {
    ...rules,
    directives: [{ directive: "@steps", before: "1. {Do it.}" }],
  });
  expect(page).toContain("Steps:\n\n  @steps\n  1. \\{Do it.}\n  2. Then.\n");
});

test("a rule that doesn't find its place stops the conversion, naming the template and the place", () => {
  const convert = (body: string) => () =>
    convertTemplate("how-to", template(body), {
      ...rules,
      directives: [{ directive: "@steps", before: "1. {Do it.}" }],
    });
  expect(convert("1. {Do something else.}")).toThrow(
    new ConversionError("how-to: the rule `@steps` found 0 lines `1. {Do it.}`, not one"),
  );
  expect(convert("1. {Do it.}\n\n1. {Do it.}")).toThrow(
    new ConversionError("how-to: the rule `@steps` found 2 lines `1. {Do it.}`, not one"),
  );
  expect(convert("```\n1. {Do it.}\n```")).toThrow(ConversionError);
});

test("a template without the shape the steps expect stops the conversion", () => {
  expect(() => convertTemplate("t", "Text.\n", rules)).toThrow(
    new ConversionError("t: the template doesn't open with a level-1 heading (line 1)"),
  );
  expect(() => convertTemplate("t", "# T\n\nText.\n", rules)).toThrow(
    new ConversionError("t: no block quote about the template after its title (line 3)"),
  );
  expect(() => convertTemplate("t", "# T\n\n> About.\n\nText.\n", rules)).toThrow(
    new ConversionError("t: the template doesn't close with a block quote"),
  );
});

test("a guide says where it's from, and its relative links go to their repository", () => {
  const pin = { repository: "https://gitlab.com/tgdp/templates", tag: "v1.6.0", commit: "c" };
  expect(
    convertGuide(
      "how-to",
      "guide_how-to.md",
      "# Guide\n\nSee [tips](../writing-tips.md#steps), [this](#here), and [them](https://example.com/).\n",
      pin,
      "MIT No Attribution License",
    ),
  ).toBe(
    "Generated from [The Good Docs Project's guide](https://gitlab.com/tgdp/templates/-/blob/v1.6.0/how-to/guide_how-to.md), in their repository on GitLab at the tag v1.6.0, under the MIT No Attribution License.\n\n# Guide\n\nSee [tips](https://gitlab.com/tgdp/templates/-/blob/v1.6.0/writing-tips.md#steps), [this](#here), and [them](https://example.com/).\n",
  );
});

test("the pin is a tag and its commit, and every template converted has its rules", () => {
  const pin = readPin();
  expect(pin.tag).toMatch(/^v\d+\.\d+\.\d+$/);
  expect(pin.commit).toMatch(/^[0-9a-f]{40}$/);
  expect([...readRules().keys()]).toEqual(["how-to", "concept"]);
});

const ascribe = binary();

test.skipIf(ascribe === undefined)(
  "the committed copies are what the pinned release gives",
  { timeout: 120_000 },
  async (context) => {
    if (ascribe === undefined) return;
    let files: Map<string, string>;
    try {
      files = await generate(ascribe);
    } catch (error) {
      if (!(error instanceof Unreachable)) throw error;
      console.warn(`Skipped: their repository can't be reached (${error.message}).`);
      context.skip(`their repository can't be reached: ${error.message}`);
      return;
    }
    if (process.env["ASCRIBE_BLESS"] === "1") write(files);
    expect(
      stale(files),
      "Stale: run `node scripts/templates/convert.ts`, then read the diff",
    ).toEqual([]);
  },
);
