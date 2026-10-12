// The changelog's entries in changes/: each one in the repository is well
// formed, and gathering puts them where a release needs them.
import { mkdirSync, mkdtempSync, readFileSync, readdirSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";
import { afterEach, describe, expect, test } from "vitest";
import {
  AREAS,
  checkChange,
  checkChanges,
  gatherChanges,
  readChanges,
  summaryOf,
  withChanges,
  type Change,
} from "./changes.ts";

test("every entry in changes/ is well formed", () => {
  expect(checkChanges(), "see changes/README.md").toEqual([]);
});

test("changes/README.md lists the areas, with their folders and headings, in order", () => {
  const readme = readFileSync(path.join(import.meta.dirname, "../../changes/README.md"), "utf8");
  const rows = Array.from(readme.matchAll(/^\| `([a-z-]+)\/` \| ([^|]+) \|/gm), (match) => ({
    folder: match[1],
    heading: match[2]?.trim(),
  }));
  expect(rows).toEqual(AREAS);
});

const changelog = (unreleased: string): string =>
  `# Changelog\n\nIntro.\n\n${unreleased}## 0.2.0 (2026-10-06)\n\n- Earlier.\n`;

describe("withChanges", () => {
  const a: Change = { area: "editor", name: "a.md", text: "- **A.** First.\n" };
  const b: Change = { area: "editor", name: "b.md", text: "- **B.** Second.\n\n  More about B.\n" };
  const fix: Change = { area: "fixes", name: "fix.md", text: "- **Fixed:** a bug.\n" };
  const lang: Change = { area: "language", name: "lang.md", text: "- **L.** A directive.\n" };

  test("puts an area's entries at the top of its heading, before what's there", () => {
    expect(
      withChanges(changelog("## Unreleased\n\n### The editor\n\n- **Old.**\n\n"), [a, b]),
    ).toBe(
      changelog(
        "## Unreleased\n\n### The editor\n\n- **A.** First.\n- **B.** Second.\n\n  More about B.\n- **Old.**\n\n",
      ),
    );
  });

  test("adds an area's heading where the areas' order puts it", () => {
    const unreleased = "## Unreleased\n\n### The editor\n\n- **Old.**\n\n";
    expect(withChanges(changelog(unreleased), [lang, fix])).toBe(
      changelog(
        "## Unreleased\n\n### The language\n\n- **L.** A directive.\n\n### The editor\n\n- **Old.**\n\n### Fixes\n\n- **Fixed:** a bug.\n\n",
      ),
    );
  });

  test("fills an empty unreleased section, and one that names its version", () => {
    expect(withChanges(changelog("## 0.3.0 (unreleased)\n\n"), [a])).toBe(
      changelog("## 0.3.0 (unreleased)\n\n### The editor\n\n- **A.** First.\n\n"),
    );
    expect(withChanges("# Changelog\n\n## Unreleased\n", [a])).toBe(
      "# Changelog\n\n## Unreleased\n\n### The editor\n\n- **A.** First.\n",
    );
  });

  test("puts an entry in no area at the top of the section", () => {
    const loose: Change = { area: "", name: "loose.md", text: "- **Loose.**\n" };
    expect(
      withChanges(changelog("## Unreleased\n\n### The editor\n\n- **Old.**\n\n"), [loose]),
    ).toBe(changelog("## Unreleased\n\n- **Loose.**\n\n### The editor\n\n- **Old.**\n\n"));
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
  const check = (text: string, name = "a-change.md", area = "editor"): string[] =>
    checkChange({ area, name, text }, exists);

  test("takes a list item with indented lines, a fenced block, and links that land", () => {
    expect(
      check(
        "- **A change.** See [the spec](SPEC.md#links), [the docs](docs/content/index.md), [examples](examples/), and [GitHub](https://github.com).\n\n  ```css\n:root {}\n  ```\n- **A second item.**\n",
      ),
    ).toEqual([]);
  });

  test("names a bad file name, a missing list item, and a line that ends the item", () => {
    expect(check("- **Fine.**\n", "Check_Fix.md")).toEqual([
      expect.stringMatching(/lowercase words/),
    ]);
    expect(check("A paragraph.\n")).toEqual([
      expect.stringMatching(/start with `- `/),
      expect.stringMatching(/^line 1 /),
    ]);
    expect(check("- **Item.**\n\nA paragraph.\n")).toEqual([expect.stringMatching(/^line 3 /)]);
    expect(check("- **Item.**\n## Heading\n")).toEqual([expect.stringMatching(/^line 2 /)]);
  });

  test("names an entry in no area, or in a folder that isn't one", () => {
    expect(check("- **Item.**\n", "a.md", "")).toEqual([
      expect.stringMatching(/isn't in an area's folder: .*`language\/`/),
    ]);
    expect(check("- **Item.**\n", "a.md", "misc")).toEqual([
      expect.stringMatching(/^`misc\/` isn't an area/),
    ]);
  });

  test("names an item with no bold sentence, or one too long to scan", () => {
    expect(check("- **One.** Fine.\n- Two isn't bold.\n")).toEqual([
      expect.stringMatching(/`- Two isn't bold\.…` doesn't start with a bold sentence/),
    ]);
    const long = `- **${"word ".repeat(20)}and more.** Rest.\n`;
    expect(check(long)).toEqual([expect.stringMatching(/is 109 characters, more than 100/)]);
    expect(check(`- **Fixed:** ${"a word ".repeat(15)}ends here. Rest.\n`)).toEqual([
      expect.stringMatching(/is 122 characters/),
    ]);
  });

  test("names an empty entry and a bad ending", () => {
    expect(check("\n")).toEqual([expect.stringMatching(/empty/)]);
    expect(check("- **Item.**")).toEqual([expect.stringMatching(/one line break/)]);
    expect(check("- **Item.**\n\n")).toEqual([expect.stringMatching(/one line break/)]);
    expect(check("- **Item.**\r\n")).toEqual([expect.stringMatching(/carriage return/)]);
  });

  test("names a link that leads nowhere or outside the repository", () => {
    expect(check("- **Links.** [a](missing.md) [b](../SPEC.md) [c](/SPEC.md)\n")).toEqual([
      expect.stringMatching(/missing\.md isn't a file/),
      expect.stringMatching(/\.\.\/SPEC\.md isn't relative/),
      expect.stringMatching(/\/SPEC\.md isn't relative/),
    ]);
  });
});

describe("summaryOf", () => {
  test("is the bold sentence as a reader sees it", () => {
    expect(summaryOf("- **`ascribe check --fix` applies [the fixes](SPEC.md).** More.\n")).toBe(
      "ascribe check --fix applies the fixes.",
    );
    expect(summaryOf("- **The extension has an icon:** the mark.\n")).toBe(
      "The extension has an icon:",
    );
  });

  test("of a fix, or of bold that starts a sentence, is the first sentence, outside code", () => {
    expect(summaryOf("- **Fixed:** `a. b` works. It didn't.\n")).toBe("Fixed: a. b works.");
    expect(summaryOf("- **Fixed:** all of it\n")).toBe("Fixed: all of it");
    expect(summaryOf("- **`@snippet`** puts code in a page. More.\n")).toBe(
      "@snippet puts code in a page.",
    );
    expect(summaryOf("- A plain item. More.\n")).toBe("A plain item.");
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
    for (const [file, text] of Object.entries(files)) {
      mkdirSync(path.dirname(path.join(dir, file)), { recursive: true });
      writeFileSync(path.join(dir, file), text);
    }
    return dir;
  }

  test("moves the entries under their areas, in the order of their names, and deletes them", () => {
    const dir = project({
      "CHANGELOG.md": changelog("## Unreleased\n\n"),
      "changes/README.md": "What this is.\n",
      "changes/fixes/zebra.md": "- **Fixed:** Z.\n",
      "changes/editor/zebra.md": "- **Z.**\n",
      "changes/editor/apple.md": "- **A.**\n",
    });
    expect(gatherChanges(dir)).toEqual([
      "changes/editor/apple.md",
      "changes/editor/zebra.md",
      "changes/fixes/zebra.md",
    ]);
    expect(readFileSync(path.join(dir, "CHANGELOG.md"), "utf8")).toBe(
      changelog(
        "## Unreleased\n\n### The editor\n\n- **A.**\n- **Z.**\n\n### Fixes\n\n- **Fixed:** Z.\n\n",
      ),
    );
    expect(readChanges(dir)).toEqual([]);
  });

  test("changes nothing when an entry has a problem", () => {
    const before = changelog("## Unreleased\n\n");
    const dir = project({ "CHANGELOG.md": before, "changes/editor/bad.md": "Not a list item.\n" });
    expect(() => gatherChanges(dir)).toThrow(/changes\/editor\/bad\.md: /);
    expect(readFileSync(path.join(dir, "CHANGELOG.md"), "utf8")).toBe(before);
    expect(readdirSync(path.join(dir, "changes", "editor"))).toEqual(["bad.md"]);
  });
});
