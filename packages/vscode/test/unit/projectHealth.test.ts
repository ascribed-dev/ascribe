import { describe, expect, it } from "vitest";
import type { EveryBuildResult } from "../../src/ui/checkEveryBuild.js";
import {
  NO_PROJECT,
  nothingToEdit,
  PROJECT_KEYS,
  problemsCount,
  projectAnswer,
  projectEditQuestion,
  type ProjectFacts,
} from "../../src/ui/projectHealth.js";

const facts = (everyBuild: EveryBuildResult): ProjectFacts => ({
  name: "docs",
  folder: "docs",
  state: "running",
  version: "0.4.0",
  everyBuild,
  time: (at) => `${at / 1000}s`,
});

const made = (
  current: boolean,
  summary = { errors: 2, warnings: 1, advice: 3 },
  more: { error?: string | null; unsaved?: number } = {},
): EveryBuildResult => ({
  kind: "made",
  at: 5000,
  summary,
  error: more.error ?? null,
  current,
  unsaved: more.unsaved ?? 0,
});

describe("projectAnswer", () => {
  it("says to open a page when the sidebar has no project", () => {
    expect(projectAnswer(undefined)).toEqual({
      message: NO_PROJECT,
      head: null,
      toDo: 0,
      health: [],
      actions: [],
    });
  });

  it("heads with the project's name, folder, server state, and version, and offers Check Every Build, the edits, and the tasks", () => {
    const answer = projectAnswer({
      ...facts({ kind: "none" }),
      state: "failed",
      version: undefined,
    });
    expect(answer.message).toBeNull();
    expect(answer.head).toEqual({ name: "docs", folder: "docs", state: "failed", version: null });
    expect(answer.actions.map((action) => action.key)).toEqual([
      PROJECT_KEYS.checkEveryBuild,
      PROJECT_KEYS.applyEverySafeFix,
      PROJECT_KEYS.formatEveryPage,
      PROJECT_KEYS.build,
      PROJECT_KEYS.syncAgentFiles,
    ]);
  });

  it("says each task runs in the terminal", () => {
    const answer = projectAnswer(facts({ kind: "none" }));
    for (const action of answer.actions.slice(3)) expect(action.caption).toMatch(/^A task: /);
  });

  it("says each edit is one edit left unsaved, and that the safe fixes are the editor build's", () => {
    const [, fixAll, format] = projectAnswer(facts({ kind: "none" })).actions;
    expect(fixAll?.caption).toMatch(
      /the editor build's, in every file, as one edit left unsaved\./,
    );
    expect(fixAll?.caption).toMatch(/Every build's are ascribe check --fix's\.$/);
    expect(format?.caption).toMatch(/as one edit left unsaved\.$/);
  });

  it("counts a count to do while it's above zero, out of date, or never run, and not while it's running or none", () => {
    expect(projectAnswer(facts({ kind: "none" })).toDo).toBe(1);
    expect(projectAnswer(facts({ kind: "running" })).toDo).toBe(0);
    expect(projectAnswer(facts(made(true))).toDo).toBe(1);
    expect(projectAnswer(facts(made(false))).toDo).toBe(1);
    expect(projectAnswer(facts(made(true, { errors: 0, warnings: 0, advice: 0 }))).toDo).toBe(0);
  });
});

describe("problemsCount", () => {
  it("is unknown until every build is checked, with the check beside it", () => {
    expect(problemsCount(facts({ kind: "none" }))).toEqual({
      key: PROJECT_KEYS.problems,
      label: "Problems in every build",
      state: "unknown",
      value: "—",
      note: "Needs every build checked",
      opens: false,
      action: { key: PROJECT_KEYS.checkEveryBuild, label: "Check Every Build" },
    });
  });

  it("says it's checking while the check runs", () => {
    expect(problemsCount(facts({ kind: "running" }))).toMatchObject({
      state: "running",
      value: "—",
      note: "Checking every build…",
      action: null,
    });
  });

  it("counts by severity while current, says when it's from, and opens the Problems panel", () => {
    expect(problemsCount(facts(made(true)))).toMatchObject({
      state: "current",
      value: "2 errors, 1 warning, 3 advice",
      note: "Checked at 5s",
      opens: true,
      action: null,
    });
    expect(problemsCount(facts(made(true, { errors: 1, warnings: 0, advice: 0 })))).toMatchObject({
      value: "1 error",
    });
  });

  it("shows none as 0, which opens nothing", () => {
    expect(problemsCount(facts(made(true, { errors: 0, warnings: 0, advice: 0 })))).toMatchObject({
      state: "none",
      value: "0",
      opens: false,
    });
  });

  it("says when files with unsaved changes were read as saved", () => {
    expect(problemsCount(facts(made(true, undefined, { unsaved: 2 }))).note).toBe(
      "Checked at 5s, 2 files with unsaved changes read as saved",
    );
  });

  it("shows a dash once the project has changed, saying why, with the check beside it", () => {
    expect(problemsCount(facts(made(false)))).toMatchObject({
      state: "outOfDate",
      value: "—",
      note: "Out of date: the project changed after the check at 5s",
      opens: false,
      action: { key: PROJECT_KEYS.checkEveryBuild },
    });
  });

  it("shows a dash for a check that couldn't finish, saying why", () => {
    expect(
      problemsCount(facts(made(false, undefined, { error: "ascribe.toml: bad" }))),
    ).toMatchObject({
      state: "outOfDate",
      value: "—",
      note: "The check at 5s couldn't finish: ascribe.toml: bad",
    });
  });
});

describe("projectEditQuestion", () => {
  it("names the fixes and the files, and says they're left unsaved", () => {
    expect(projectEditQuestion(4, 6)).toBe(
      "Apply 6 safe fixes in 4 files? They're left unsaved, to read before you save them.",
    );
    expect(projectEditQuestion(1, 1)).toBe(
      "Apply 1 safe fix in 1 file? It's left unsaved, to read before you save it.",
    );
  });

  it("names the files to format", () => {
    expect(projectEditQuestion(12, null)).toBe(
      "Format 12 files? They're left unsaved, to read before you save them.",
    );
    expect(projectEditQuestion(1, null)).toBe(
      "Format 1 file? It's left unsaved, to read before you save it.",
    );
  });

  it("says when there's nothing to do", () => {
    expect(nothingToEdit("fixAll", "docs")).toBe(
      "Ascribe: none of the problems the editor shows in docs has a safe fix.",
    );
    expect(nothingToEdit("format", "docs")).toBe(
      "Ascribe: every page of docs is formatted already.",
    );
  });
});
