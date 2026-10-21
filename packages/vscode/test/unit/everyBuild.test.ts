import * as path from "node:path";
import { describe, expect, it } from "vitest";
import { checkedText, editorPosition, panelProblems } from "../../src/everyBuild.js";
import type { CheckReport, Entry, Range } from "../../src/shapes.js";

const root = path.resolve("/work/docs");

const at = (line: number, column: number, endColumn = column + 1): Range => ({
  start: { line, column, offset: 0 },
  end: { line, column: endColumn, offset: 0 },
});

function entry(code: string, file: string, range: Range, builds: string[]): Entry {
  return {
    code,
    slug: "link-target-missing",
    severity: "error",
    next: "choose",
    message: `a problem${builds.length === 1 ? ` (only in build ${builds[0]})` : ""}`,
    file,
    range,
    related: [],
    fixes: [],
    builds,
    unpublished: false,
    help: "",
    docs: `https://ascribed-dev.com/go/${code}/`,
    repeats: 0,
  };
}

function report(diagnostics: Entry[], extra: Partial<CheckReport> = {}): CheckReport {
  return {
    schema_version: 1,
    ascribe_version: "0.2.0",
    error: null,
    files_checked: 3,
    files_reported: 3,
    builds_checked: ["site", "cloud"],
    diagnostics,
    truncated: false,
    shown: diagnostics.length,
    total: diagnostics.length,
    next_command: null,
    summary: {
      errors: diagnostics.filter((d) => d.severity === "error").length,
      warnings: 0,
      advice: 0,
    },
    ...extra,
  };
}

describe("a position of ascribe check's, in the editor", () => {
  it("counts lines and columns from 0, and columns in UTF-16 units", () => {
    expect(editorPosition("one\ntwo", { line: 2, column: 2, offset: 5 })).toEqual({
      line: 1,
      character: 1,
    });
    // An emoji is one character to `ascribe check`, and two units to the editor.
    expect(editorPosition("😀 x", { line: 1, column: 3, offset: 5 })).toEqual({
      line: 0,
      character: 3,
    });
  });

  it("clamps a position past the end", () => {
    expect(editorPosition("one\n", { line: 5, column: 9, offset: 0 })).toEqual({
      line: 1,
      character: 0,
    });
    expect(editorPosition("", { line: 1, column: 1, offset: 0 })).toEqual({
      line: 0,
      character: 0,
    });
  });
});

describe("the problems a check of every build adds to the Problems panel", () => {
  const page = path.join(root, "docs", "page.md");
  const text = (file: string) => (file === page ? "# Page\n\nSee [it](/gone/).\n" : undefined);

  it("leaves out the ones the panel shows, at the same place with the same code", () => {
    const shown = entry("ASC036", "docs/page.md", at(3, 5, 17), ["site", "cloud"]);
    const cloud = entry("ASC037", "docs/page.md", at(3, 5, 17), ["cloud"]);
    const problems = panelProblems(
      report([shown, cloud]),
      root,
      text,
      (file, code, start) =>
        file === page && code === "ASC036" && start.line === 2 && start.character === 4,
    );
    expect(problems.map((p) => p.entry.code)).toEqual(["ASC037"]);
    expect(problems[0]?.file).toBe(page);
    expect(problems[0]?.range).toEqual({
      start: { line: 2, character: 4 },
      end: { line: 2, character: 16 },
    });
  });

  it("places related places, and a problem in a file it can't read at the file's start", () => {
    const related = {
      ...entry("ASC037", "docs/missing.md", at(4, 2), ["cloud"]),
      related: [{ file: "docs/page.md", range: at(1, 3), message: "linked from here" }],
    };
    const [problem] = panelProblems(report([related]), root, text, () => false);
    expect(problem?.file).toBe(path.join(root, "docs", "missing.md"));
    expect(problem?.range.start).toEqual({ line: 0, character: 0 });
    expect(problem?.related).toEqual([
      {
        file: page,
        range: { start: { line: 0, character: 2 }, end: { line: 0, character: 3 } },
        message: "linked from here",
      },
    ]);
  });
});

describe("what the check says when it's done", () => {
  const cloud = entry("ASC037", "docs/page.md", at(1, 1), ["cloud"]);

  it("names the builds, counts the problems, and says how many it added", () => {
    expect(checkedText(report([cloud]), "docs", 1, 0)).toBe(
      "Ascribe checked every build of docs (site, cloud): 1 error. 1 wasn't in the Problems panel, and is now.",
    );
    expect(checkedText(report([cloud, cloud]), "docs", 0, 2)).toBe(
      "Ascribe checked every build of docs (site, cloud): 2 errors. The Problems panel already showed them. 2 files with unsaved changes were checked as saved.",
    );
  });

  it("counts advice by its lines, as the views' tooltips do", () => {
    const advice = { ...cloud, severity: "advice" as const };
    const warning = { ...cloud, severity: "warning" as const };
    expect(
      checkedText(
        report([cloud, warning, advice, advice], {
          summary: { errors: 1, warnings: 1, advice: 2 },
        }),
        "docs",
        0,
        0,
      ),
    ).toBe(
      "Ascribe checked every build of docs (site, cloud): 1 error, 1 warning, and advice on 2 lines. The Problems panel already showed them.",
    );
  });

  it("says when there are no problems, and when the project couldn't be checked", () => {
    expect(checkedText(report([]), "docs", 0, 1)).toBe(
      "Ascribe checked every build of docs (site, cloud): no problems. 1 file with unsaved changes was checked as saved.",
    );
    expect(checkedText(report([], { error: "ascribe.toml doesn't parse" }), "docs", 0, 0)).toBe(
      "Ascribe couldn't check docs: ascribe.toml doesn't parse",
    );
  });
});
