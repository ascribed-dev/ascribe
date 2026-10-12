// The decisions in project-docs/decisions/: each one in the repository is
// well formed, and the list prints them under their sections.
import { mkdirSync, mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";
import { afterEach, describe, expect, test } from "vitest";
import { checkDecision, checkDecisions, listDecisions, type Log } from "./decisions.ts";

test("every decision in project-docs/decisions/ is well formed", () => {
  expect(checkDecisions(), "see project-docs/decisions/README.md").toEqual([]);
});

const good =
  "---\nsection: The language\n---\n\n**A thing.** More. _Why:_ a reason. _From:_ the maintainer.\n";

describe("checkDecision", () => {
  const log: Log = { sections: ["The binary", "The language"], last: 123 };
  const exists = (file: string): boolean =>
    ["SPEC.md", "project-docs/decisions.md", "project-docs/decisions/400-other.md"].includes(file);
  const check = (text: string, name = "381-a-thing.md"): string[] =>
    checkDecision({ name, text }, log, exists);

  test("takes a decision with its section, Why, From, and links that land", () => {
    expect(check(good)).toEqual([]);
    expect(
      check(
        "---\nsection: The binary\n---\n\n~~**Old.**~~ Replaced by [decision #400](400-other.md). See [the spec](../../SPEC.md#links) and [the log](../decisions.md). _Why:_ a. _From:_ b.\n",
        "381b-old.md",
      ),
    ).toEqual([]);
  });

  test("names a bad file name, and a number that would read as one of decisions.md's", () => {
    expect(check(good, "decision-files.md")).toEqual([expect.stringMatching(/issue's number/)]);
    expect(check(good, "381-Decision_Files.md")).toEqual([expect.stringMatching(/issue's number/)]);
    expect(check(good, "381ab-x.md")).toEqual([expect.stringMatching(/issue's number/)]);
    expect(check(good, "74-old.md")).toEqual([expect.stringMatching(/isn't above 123/)]);
  });

  test("names missing frontmatter, an unknown section, and another field", () => {
    expect(check("**A.** _Why:_ a. _From:_ b.\n")).toEqual([
      expect.stringMatching(/frontmatter naming its section/),
    ]);
    expect(check(good.replace("The language", "Language"))).toEqual([
      expect.stringMatching(/"Language" isn't one of decisions\.md's/),
    ]);
    expect(check(good.replace("section: The language", "number: 3"))).toEqual([
      expect.stringMatching(/doesn't name its `section`/),
      expect.stringMatching(/more than `section`: number/),
    ]);
  });

  test("names a missing bold sentence, Why, and From, and a bad ending", () => {
    expect(check("---\nsection: The binary\n---\n\nA thing.\n")).toEqual([
      expect.stringMatching(/bold sentence/),
      expect.stringMatching(/_Why:_/),
      expect.stringMatching(/_From:_/),
    ]);
    expect(check(good.trimEnd())).toEqual([expect.stringMatching(/one line break/)]);
    expect(check(good.replace(/\n/g, "\r\n"))).toContainEqual(
      expect.stringMatching(/carriage return/),
    );
  });

  test("names a link that leads nowhere or outside the repository", () => {
    expect(
      check(good.replace("More.", "[a](missing.md) [b](../../../x.md) [c](/SPEC.md)")),
    ).toEqual([
      expect.stringMatching(/missing\.md isn't a file/),
      expect.stringMatching(/\.\.\/\.\.\/\.\.\/x\.md leads outside/),
      expect.stringMatching(/\/SPEC\.md leads outside/),
    ]);
  });
});

describe("a folder of decisions", () => {
  const dirs: string[] = [];
  afterEach(() => {
    for (const dir of dirs.splice(0)) rmSync(dir, { recursive: true, force: true });
  });

  function project(files: Record<string, string>): string {
    const dir = mkdtempSync(path.join(tmpdir(), "ascribe-decisions-"));
    dirs.push(dir);
    mkdirSync(path.join(dir, "project-docs", "decisions"), { recursive: true });
    writeFileSync(
      path.join(dir, "project-docs", "decisions.md"),
      "# Decisions\n\n## The binary\n\n1. **One.**\n\n## The language\n\n2. **Two.**\n",
    );
    for (const [file, text] of Object.entries(files)) {
      writeFileSync(path.join(dir, "project-docs", "decisions", file), text);
    }
    return dir;
  }

  test("asks for a letter on each of an issue's decisions", () => {
    const dir = project({ "381-a.md": good, "381b-b.md": good, "382a-c.md": good });
    expect(checkDecisions(dir)).toEqual([expect.stringMatching(/issue 381 .* own letter/)]);
    expect(checkDecisions(project({ "381a-a.md": good, "381a-b.md": good }))).toEqual([
      expect.stringMatching(/issue 381 /),
    ]);
  });

  test("lists the decisions under their sections, in decisions.md's order and by number", () => {
    const dir = project({
      "README.md": "What this is.\n",
      "1000-late.md": good.replace("A thing.", "Late."),
      "999-early.md": good.replace("A thing.", "Early."),
      "500-binary.md": good.replace("The language", "The binary").replace("A thing.", "Binary."),
    });
    expect(checkDecisions(dir)).toEqual([]);
    expect(listDecisions(dir)).toBe(
      [
        "## The binary",
        "",
        "- [Decision #500](decisions/500-binary.md): **Binary.**",
        "",
        "## The language",
        "",
        "- [Decision #999](decisions/999-early.md): **Early.**",
        "- [Decision #1000](decisions/1000-late.md): **Late.**",
        "",
      ].join("\n"),
    );
    expect(listDecisions(dir, "the binary")).toBe(
      "## The binary\n\n- [Decision #500](decisions/500-binary.md): **Binary.**\n",
    );
  });
});
