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
  upstream,
  write,
} from "./convert.ts";

const rules = { description: "{Say what it's for.}", about: [], memos: [], directives: [] };

function template(body: string): string {
  return `# {Product} title\n\n> Read the [guide](./guide.md).\n>\n> Text in {curly brackets}.\n\n${body}\n\n---\n\n> Explore other templates.\n`;
}

test("the title goes to frontmatter, and what the template says about itself goes", () => {
  expect(convertTemplate("t", template("## Overview\n\nThis explains {the task}."), rules)).toBe(
    [
      "---",
      "type: t",
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

test("every block quote the template opens with goes, however many", () => {
  const source =
    "# T\n\n> Read the [guide](link to the guide).\n\n> Text in {curly brackets}.\n\n## Overview\n\nText.\n\n---\n\n> Explore other templates.\n";
  expect(convertTemplate("t", source, rules)).toBe(
    [
      "---",
      "type: t",
      'title: "T"',
      `description: "{Say what it's for.}"`,
      "---",
      "",
      "## Overview",
      "",
      "Text.",
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

test("a paragraph a rule says is about the template goes whole, and one it doesn't find stops the conversion", () => {
  const about = {
    ...rules,
    about: ["{Read the [process](process_t.md)", "{Text in {curly brackets}.}"],
  };
  expect(
    convertTemplate(
      "t",
      template(
        "{Read the [process](process_t.md)\ndocument.}\n\n{Text in {curly brackets}.}\n\n{Release date}",
      ),
      about,
    ),
  ).toBe(
    [
      "---",
      "type: t",
      'title: "{Product} title"',
      `description: "{Say what it's for.}"`,
      "---",
      "",
      "\\{Release date}",
      "",
    ].join("\n"),
  );
  expect(() => convertTemplate("t", template("{Release date}"), about)).toThrow(
    new ConversionError(
      "t: the rule `about` found 0 paragraphs opening `{Read the [process](process_t.md)`, not one",
    ),
  );
});

function body(page: string): string {
  return page.slice(page.indexOf("---\n\n", 4) + 5);
}

test("an instruction before a block is a memo in its place, touching the block", () => {
  expect(
    body(
      convertTemplate(
        "t",
        template("## Overview\n\n{Optional: Say why.}\n\nThis explains {the task}."),
        {
          ...rules,
          memos: ["{Optional: Say why.}"],
        },
      ),
    ),
  ).toBe("## Overview\n\n@memo: Optional: Say why.\nThis explains \\{the task}.\n");
});

test("an instruction before `@steps` stacks with it, and is about the list", () => {
  expect(
    body(
      convertTemplate("t", template("## Do it\n\n{Use this format:}\n\n1. {Do it.}\n2. Then."), {
        ...rules,
        memos: ["{Use this format:}"],
        directives: [{ directive: "@steps", before: "1. {Do it.}" }],
      }),
    ),
  ).toBe("## Do it\n\n@memo: Use this format:\n@steps\n1. \\{Do it.}\n2. Then.\n");
});

test("an instruction that ends its section goes to the top of the section, under its heading and `@id`", () => {
  const memos = { ...rules, memos: ["{This is optional. Use it only when the task is big.}"] };
  expect(
    body(
      convertTemplate(
        "t",
        template(
          "## Sub-task\n@id: sub\n\n...\n\n{This is optional.\nUse it only when the task is big.}\n\n## See also\n\nLinks.",
        ),
        memos,
      ),
    ),
  ).toBe(
    "## Sub-task\n@id: sub\n@memo: This is optional. Use it only when the task is big.\n\n...\n\n## See also\n\nLinks.\n",
  );
  expect(
    body(
      convertTemplate(
        "t",
        template("## Last\n\n{This is optional. Use it only when the task is big.}"),
        memos,
      ),
    ),
  ).toBe("## Last\n@memo: This is optional. Use it only when the task is big.\n");
});

test("an instruction of several lines too long for one is a container with their lines", () => {
  const long = "a".repeat(100);
  expect(
    body(
      convertTemplate(
        "t",
        template(
          `## Links\n\n{Use this section for ${long}\nand say "Related" ${long}.}\n\nSee these.`,
        ),
        {
          ...rules,
          memos: [`{Use this section for ${long} and say "Related" ${long}.}`],
        },
      ),
    ),
  ).toBe(
    `## Links\n\n@memo:\nUse this section for ${long}\nand say "Related" ${long}.\n@end\nSee these.\n`,
  );
});

test("an instruction that ends a list item, or one the rule doesn't find, stops the conversion", () => {
  const convert = (source: string, memo: string) => () =>
    convertTemplate("how-to", template(source), { ...rules, memos: [memo] });
  expect(
    convert(
      "## Steps\n\n1. {Do it.}\n\n    {Optional: The result.}\n\n2. Then.",
      "{Optional: The result.}",
    ),
  ).toThrow(
    new ConversionError(
      "how-to: the memo `{Optional: The result.}` is the last block of a container with no heading, so it has nothing to be about",
    ),
  );
  expect(convert("{Optional: Say why.}\n\n## Overview", "{Optional: Say why.}")).toThrow(
    ConversionError,
  );
  expect(convert("## Overview\n\n{Say why.}", "{Say something else.}")).toThrow(
    new ConversionError(
      "how-to: the rule `memos` found 0 paragraphs `{Say something else.}`, not one",
    ),
  );
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
  expect([...readRules().keys()]).toEqual([
    "how-to",
    "concept",
    "tutorial",
    "reference",
    "troubleshooting",
    "release-notes",
  ]);
});

test("a template's table without a description stops the script", () => {
  expect(() =>
    readRules('[t]\n\n[[t.directives]]\ndirective = "@steps"\nbefore = "1. Do it."\n'),
  ).toThrow("rules.toml: [t] has no `description`");
});

test(
  "every rule of rules.toml is used, at one place, and the script stops without it",
  { timeout: 60_000 },
  async (context) => {
    // A directive, an `about` paragraph, or a memo finds its place exactly once, or the
    // conversion throws. Taken away, each changes what the script writes, so
    // `convert.ts --check` stops on the committed copies.
    const pin = readPin();
    for (const [name, all] of readRules()) {
      let source: string;
      try {
        source = await upstream(pin, `${name}/template_${name}.md`);
      } catch (error) {
        if (!(error instanceof Unreachable)) throw error;
        context.skip(`their repository can't be reached: ${error.message}`);
        return;
      }
      const converted = convertTemplate(name, source, all);
      const without: [string, typeof all][] = [
        ...all.directives.map((rule, i): [string, typeof all] => [
          `${rule.directive} before \`${rule.before}\``,
          { ...all, directives: all.directives.filter((_, j) => j !== i) },
        ]),
        ...all.about.map((first, i): [string, typeof all] => [
          `about \`${first}\``,
          { ...all, about: all.about.filter((_, j) => j !== i) },
        ]),
        ...all.memos.map((text, i): [string, typeof all] => [
          `memo \`${text}\``,
          { ...all, memos: all.memos.filter((_, j) => j !== i) },
        ]),
      ];
      for (const [rule, rules] of without)
        expect(convertTemplate(name, source, rules), `${name}: ${rule}`).not.toBe(converted);
    }
  },
);

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
