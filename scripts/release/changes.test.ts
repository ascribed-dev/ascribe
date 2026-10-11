// The changelog's entries in changes/: each one in the repository is well
// formed, and gathering puts them where a release needs them.
import { mkdirSync, mkdtempSync, readFileSync, readdirSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";
import { afterEach, describe, expect, test } from "vitest";
import {
  checkChange,
  checkChanges,
  gatherChanges,
  readChanges,
  withChanges,
  type Change,
} from "./changes.ts";

test("every entry in changes/ is well formed", () => {
  expect(checkChanges(), "see changes/README.md").toEqual([]);
});

const changelog = (unreleased: string): string =>
  `# Changelog\n\nIntro.\n\n${unreleased}## 0.2.0 (2026-10-06)\n\n- Earlier.\n`;

describe("withChanges", () => {
  const a: Change = { name: "a.md", text: "- **A.** First.\n" };
  const b: Change = { name: "b.md", text: "- **B.** Second.\n\n  More about B.\n" };

  test("puts the entries at the top of the unreleased section, before what's there", () => {
    expect(withChanges(changelog("## Unreleased\n\n- **Old.**\n\n"), [a, b])).toBe(
      changelog(
        "## Unreleased\n\n- **A.** First.\n- **B.** Second.\n\n  More about B.\n- **Old.**\n\n",
      ),
    );
  });

  test("fills an empty unreleased section, and one that names its version", () => {
    expect(withChanges(changelog("## 0.3.0 (unreleased)\n\n"), [a])).toBe(
      changelog("## 0.3.0 (unreleased)\n\n- **A.** First.\n\n"),
    );
    expect(withChanges("# Changelog\n\n## Unreleased\n", [a])).toBe(
      "# Changelog\n\n## Unreleased\n\n- **A.** First.\n",
    );
  });

  test("leaves the changelog alone with no entries, and refuses one without an unreleased section", () => {
    const text = changelog("");
    expect(withChanges(text, [])).toBe(text);
    expect(() => withChanges(text, [a])).toThrow(/Unreleased/);
  });
});

describe("checkChange", () => {
  const exists = (file: string): boolean =>
    ["SPEC.md", "docs/content/index.md", "examples"].includes(file);
  const check = (text: string, name = "a-change.md"): string[] =>
    checkChange({ name, text }, exists);

  test("takes a list item with indented lines, a fenced block, and links that land", () => {
    expect(
      check(
        "- **A change.** See [the spec](SPEC.md#links), [the docs](docs/content/index.md), [examples](examples/), and [GitHub](https://github.com).\n\n  ```css\n:root {}\n  ```\n- A second item.\n",
      ),
    ).toEqual([]);
  });

  test("names a bad file name, a missing list item, and a line that ends the item", () => {
    expect(check("- Fine.\n", "Check_Fix.md")).toEqual([expect.stringMatching(/lowercase words/)]);
    expect(check("A paragraph.\n")).toEqual([
      expect.stringMatching(/start with `- `/),
      expect.stringMatching(/^line 1 /),
    ]);
    expect(check("- Item.\n\nA paragraph.\n")).toEqual([expect.stringMatching(/^line 3 /)]);
    expect(check("- Item.\n## Heading\n")).toEqual([expect.stringMatching(/^line 2 /)]);
  });

  test("names an empty entry and a bad ending", () => {
    expect(check("\n")).toEqual([expect.stringMatching(/empty/)]);
    expect(check("- Item.")).toEqual([expect.stringMatching(/one line break/)]);
    expect(check("- Item.\n\n")).toEqual([expect.stringMatching(/one line break/)]);
    expect(check("- Item.\r\n")).toEqual([expect.stringMatching(/carriage return/)]);
  });

  test("names a link that leads nowhere or outside the repository", () => {
    expect(check("- [a](missing.md) [b](../SPEC.md) [c](/SPEC.md)\n")).toEqual([
      expect.stringMatching(/missing\.md isn't a file/),
      expect.stringMatching(/\.\.\/SPEC\.md isn't relative/),
      expect.stringMatching(/\/SPEC\.md isn't relative/),
    ]);
  });
});

describe("gatherChanges", () => {
  const dirs: string[] = [];
  afterEach(() => {
    for (const dir of dirs.splice(0)) rmSync(dir, { recursive: true, force: true });
  });

  function project(files: Record<string, string>): string {
    const dir = mkdtempSync(path.join(tmpdir(), "ascribe-changes-"));
    dirs.push(dir);
    mkdirSync(path.join(dir, "changes"));
    for (const [file, text] of Object.entries(files)) writeFileSync(path.join(dir, file), text);
    return dir;
  }

  test("moves the entries into CHANGELOG.md, in the order of their names, and deletes them", () => {
    const dir = project({
      "CHANGELOG.md": changelog("## Unreleased\n\n"),
      "changes/README.md": "What this is.\n",
      "changes/zebra.md": "- Z.\n",
      "changes/apple.md": "- A.\n",
    });
    expect(gatherChanges(dir)).toEqual(["apple.md", "zebra.md"]);
    expect(readFileSync(path.join(dir, "CHANGELOG.md"), "utf8")).toBe(
      changelog("## Unreleased\n\n- A.\n- Z.\n\n"),
    );
    expect(readdirSync(path.join(dir, "changes"))).toEqual(["README.md"]);
    expect(readChanges(dir)).toEqual([]);
  });

  test("changes nothing when an entry has a problem", () => {
    const before = changelog("## Unreleased\n\n");
    const dir = project({ "CHANGELOG.md": before, "changes/bad.md": "Not a list item.\n" });
    expect(() => gatherChanges(dir)).toThrow(/changes\/bad\.md: /);
    expect(readFileSync(path.join(dir, "CHANGELOG.md"), "utf8")).toBe(before);
    expect(readdirSync(path.join(dir, "changes"))).toEqual(["bad.md"]);
  });
});
