// Every surface that says what Ascribe is says it in the words of their one
// home, design/words.toml. A surface generated from the home is generated; one
// that can't be is compared with it here, so changing a line in the home
// fails this file until every surface agrees. design/README.md lists them.

import { readFileSync } from "node:fs";
import { join } from "node:path";
import { describe, expect, test } from "vitest";
import { readSources } from "./assets.ts";
import { ROOT } from "./specimen.ts";
import { readWords, SHORT_MAX, words, WORDS } from "./words.ts";

const read = (file: string): string => readFileSync(join(ROOT, file), "utf8");
const home = words();

describe("design/words.toml", () => {
  test("holds the story's headline and sentence, as the story quotes them", () => {
    const story = read("project-docs/brand/story.md");
    expect(story).toContain(`\n> ${home.headline}\n`);
    expect(story).toContain(`\n> ${home.sentence}\n`);
  });

  test("its short form is the sentence's first clause, as a line", () => {
    const clause = home.short.replace(/\.$/, "");
    expect(home.short).toMatch(/\.$/);
    expect(home.sentence.startsWith(`Ascribe is ${clause.toLowerCase()}`)).toBe(true);
    expect(home.short.length).toBeLessThan(SHORT_MAX);
  });

  test("a missing or too long line names the problem", () => {
    const text = read(WORDS);
    expect(() => readWords(text.replace(/^headline = .*$/m, ""))).toThrow(
      `${WORDS} headline: a line of text`,
    );
    expect(() => readWords(text.replace(/^short = .*$/m, `short = "${"a".repeat(120)}"`))).toThrow(
      `${WORDS} short: under ${SHORT_MAX} characters, not 120`,
    );
  });
});

test("the social card draws the headline", () => {
  expect(readSources().line).toBe(home.headline);
});
