// Taking The Good Docs Project's newest release (update.ts): which tag is
// newest, moving the pin, what changed between two tags, and the pull
// request's description. The update itself, against their repository, was
// tried by hand with the pin set back and with a rule broken (#411); the
// workflow can try it again with its `from` input.
import { expect, test } from "vitest";
import {
  compareLink,
  describe,
  type Found,
  type IndexEntry,
  movePin,
  newest,
  title,
  whatChanged,
} from "./update.ts";

const repository = "https://gitlab.com/tgdp/templates";
const from = { repository, tag: "v1.5.0", commit: "a".repeat(40) };
const to = { repository, tag: "v1.6.0", commit: "b".repeat(40) };

test("the newest tag is the newest release's, by version, not by name", () => {
  expect(
    newest([
      { name: "v1.9.0", commit: "9" },
      { name: "v1.10.0", commit: "10" },
      { name: "v0.1", commit: "0" },
      { name: "v2.0.0-rc1", commit: "rc" },
    ]),
  ).toEqual({ name: "v1.10.0", commit: "10" });
  expect(newest([{ name: "latest", commit: "x" }])).toBeUndefined();
});

test("moving the pin changes its tag and commit and nothing else", () => {
  const text = `# Comment.\nrepository = "${repository}"\ntag = "v1.5.0"\ncommit = "${from.commit}"\n`;
  expect(movePin(text, { name: to.tag, commit: to.commit })).toBe(
    `# Comment.\nrepository = "${repository}"\ntag = "v1.6.0"\ncommit = "${to.commit}"\n`,
  );
  expect(() => movePin("tag = 1\n", { name: to.tag, commit: to.commit })).toThrow(
    'upstream.toml has no `tag = "..."` line',
  );
});

function entry(name: string, summary: string, packs = ["core"]): IndexEntry {
  return {
    name,
    summary,
    files: { template: `template_${name}.md`, guide: `guide_${name}.md` },
    packs,
  };
}

test("what changed is from the files the diff names and the index's summaries", () => {
  const before = [entry("how-to", "Steps."), entry("concept", "Ideas."), entry("readme", "R.")];
  const after = [
    entry("how-to", "Steps."),
    entry("concept", "Ideas, explained."),
    entry("readme", "R."),
    entry("faq", "Questions."),
    entry("sdk-overview", "SDKs.", ["misc"]),
  ];
  expect(
    whatChanged(
      ["how-to", "concept", "tutorial"],
      ["how-to/template_how-to.md", "how-to/guide_how-to.md", "concept/example_concept.md"],
      before,
      after,
    ),
  ).toEqual({
    changed: [
      { name: "how-to", what: ["the template", "the guide"] },
      { name: "concept", what: ["its summary in their index"] },
    ],
    added: ["faq (template_faq.md)"],
  });
});

const found: Found = {
  from,
  to,
  changed: [{ name: "reference", what: ["the template", "the guide"] }],
  added: [],
};

test("the description names what changed and links the comparison", () => {
  expect(compareLink(from, to)).toBe(`${repository}/-/compare/v1.5.0...v1.6.0`);
  expect(title(found)).toBe("Take The Good Docs Project's templates at v1.6.0");
  const text = describe({ ...found, added: ["faq (template_faq.md)"] });
  expect(text).toContain("moves the pin from v1.5.0");
  expect(text).toContain(`[v1.5.0...v1.6.0](${repository}/-/compare/v1.5.0...v1.6.0)`);
  expect(text).toContain("- **reference**: the template, the guide\n");
  expect(text).toContain("isn't added");
  expect(text).toContain("- faq (template_faq.md)\n");
  expect(describe({ ...found, changed: [] })).toContain(
    "- Nothing of theirs in the six templates or their guides",
  );
});

test("a failure's description is the failure, and says no template changed", () => {
  const failure = "how-to: the rule `@steps` found 0 lines `1. {Do it.}`, not one";
  const text = describe({ ...found, failure });
  expect(title({ ...found, failure })).toBe(
    "The Good Docs Project's v1.6.0 stops the templates' conversion",
  );
  expect(text).toContain(`\`\`\`text\n${failure}\n\`\`\``);
  expect(text).toContain("changes no template");
  expect(text).not.toContain("**reference**");
});
