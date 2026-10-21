import { describe, expect, it } from "vitest";
import type { Counted, Found } from "../../src/everyBuild.js";
import type { InventoryResult } from "../../src/shapes.js";
import type { AgentFilesResult } from "../../src/ui/agentFiles.js";
import type { EveryBuildResult } from "../../src/ui/checkEveryBuild.js";
import {
  agentFilesCount,
  examplesCount,
  inventoryCounts,
  NO_PROJECT,
  nothingToEdit,
  orphansCount,
  permalinksCount,
  PROJECT_KEYS,
  problemsCount,
  projectAnswer,
  projectEditQuestion,
  reviewsCount,
  staleFiles,
  syncQuestion,
  unusedCount,
  type ProjectFacts,
} from "../../src/ui/projectHealth.js";

const DAY = 24 * 60 * 60 * 1000;

const facts = (everyBuild: EveryBuildResult, more: Partial<ProjectFacts> = {}): ProjectFacts => ({
  name: "docs",
  folder: "docs",
  state: "running",
  version: "0.4.0",
  everyBuild,
  inventory: { orphans: 0, unused: 0 },
  agentFiles: { kind: "made", at: 7000, stale: [], error: null, current: true },
  now: 6000,
  time: (at) => `${at / 1000}s`,
  day: (at) => `day ${Math.floor(at / DAY)}`,
  ...more,
});

const nothing: Found = { count: 0, first: null };
const found = (count: number): Found => ({
  count,
  first:
    count === 0
      ? null
      : {
          file: "/docs/a.md",
          range: { start: { line: 2, character: 0 }, end: { line: 2, character: 4 } },
        },
});

const made = (
  current: boolean,
  summary = { errors: 2, warnings: 1, advice: 3 },
  more: { error?: string | null; unsaved?: number; counted?: Partial<Record<Counted, Found>> } = {},
): EveryBuildResult => ({
  kind: "made",
  at: 5000,
  summary,
  error: more.error ?? null,
  current,
  unsaved: more.unsaved ?? 0,
  counted: { examples: nothing, reviews: nothing, permalinks: nothing, ...more.counted },
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
      PROJECT_KEYS.observeExamples,
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
    // Every count from the check: problems, examples, reviews, and permalinks.
    expect(projectAnswer(facts({ kind: "none" })).toDo).toBe(4);
    expect(projectAnswer(facts({ kind: "running" })).toDo).toBe(0);
    expect(projectAnswer(facts(made(true))).toDo).toBe(1);
    expect(projectAnswer(facts(made(false))).toDo).toBe(4);
    expect(projectAnswer(facts(made(true, { errors: 0, warnings: 0, advice: 0 }))).toDo).toBe(0);
    expect(
      projectAnswer(
        facts(made(true, { errors: 0, warnings: 0, advice: 0 }), {
          inventory: { orphans: 2, unused: 1 },
          agentFiles: { kind: "none" },
        }),
      ).toDo,
    ).toBe(3);
  });

  it("lists Health's counts in the proposal's order", () => {
    expect(projectAnswer(facts({ kind: "none" })).health.map((count) => count.key)).toEqual([
      PROJECT_KEYS.problems,
      PROJECT_KEYS.examples,
      PROJECT_KEYS.reviews,
      PROJECT_KEYS.permalinks,
      PROJECT_KEYS.orphans,
      PROJECT_KEYS.unused,
      PROJECT_KEYS.agentFiles,
    ]);
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

describe("the counts from the same check", () => {
  const counts = [examplesCount, reviewsCount, permalinksCount];

  it("follow the check: a dash until it runs, while it runs, and once the project changes", () => {
    for (const count of counts) {
      expect(count(facts({ kind: "none" }))).toMatchObject({
        state: "unknown",
        value: "—",
        note: "Needs every build checked",
        action: { key: PROJECT_KEYS.checkEveryBuild },
      });
      expect(count(facts({ kind: "running" }))).toMatchObject({ state: "running", action: null });
      expect(
        count(facts(made(false, undefined, { counted: { examples: found(2) } }))),
      ).toMatchObject({
        state: "outOfDate",
        value: "—",
        note: "Out of date: the project changed after the check at 5s",
        opens: false,
      });
      expect(count(facts(made(false, undefined, { error: "bad" })))).toMatchObject({
        state: "outOfDate",
        note: "The check at 5s couldn't finish: bad",
      });
    }
  });

  it("count their own problems, opening them while above zero, and show none as 0", () => {
    const check = made(true, undefined, {
      counted: { examples: found(3), reviews: found(1), permalinks: found(2) },
    });
    expect(examplesCount(facts(check))).toMatchObject({
      key: PROJECT_KEYS.examples,
      state: "current",
      value: "3 examples",
      note: "Checked at 5s",
      opens: true,
    });
    expect(reviewsCount(facts(check))).toMatchObject({
      state: "current",
      value: "1 page",
      note: "Checked at 5s, for day 0",
      opens: true,
    });
    expect(permalinksCount(facts(check))).toMatchObject({ value: "2 permalinks", opens: true });
    for (const count of counts) {
      expect(count(facts(made(true)))).toMatchObject({ state: "none", value: "0", opens: false });
    }
  });

  it("puts overdue reviews out of date when the day changes, whatever isCurrent says", () => {
    const check = made(true, undefined, { counted: { reviews: found(1) } });
    expect(reviewsCount(facts(check, { now: DAY + 1 }))).toMatchObject({
      state: "outOfDate",
      value: "—",
      note: "Out of date: checked on day 0, and reviews fall due by the day",
      opens: false,
      action: { key: PROJECT_KEYS.checkEveryBuild },
    });
    // The other counts from the same check stay current.
    expect(examplesCount(facts(check, { now: DAY + 1 })).state).toBe("none");
    expect(problemsCount(facts(check, { now: DAY + 1 })).state).toBe("current");
  });
});

describe("the counts from the inventory", () => {
  const inventory = (orphans: string[], uses: (number | null)[]): InventoryResult => ({
    pages: [],
    fragments: [],
    orphans,
    navigation: null,
    model: uses.map(
      (n, i) => ({ uses: n, key: `k${i}` }) as unknown as InventoryResult["model"][number],
    ),
  });

  it("count orphans, and entries nothing uses, leaving out an entry whose uses aren't counted", () => {
    expect(inventoryCounts(inventory(["a.md", "b.md"], [0, 3, null, 0, 0]))).toEqual({
      orphans: 2,
      unused: 3,
    });
  });

  it("are current as the server answers, open their lists, and show none as 0", () => {
    const counted = facts(made(true), { inventory: { orphans: 1, unused: 4 } });
    expect(orphansCount(counted)).toMatchObject({
      key: PROJECT_KEYS.orphans,
      state: "current",
      value: "1 page",
      opens: true,
    });
    expect(unusedCount(counted)).toMatchObject({ value: "4 entries", opens: true });
    const none = facts(made(true), { inventory: { orphans: 0, unused: 0 } });
    expect(orphansCount(none)).toMatchObject({ state: "none", value: "0", opens: false });
    expect(unusedCount(none)).toMatchObject({ state: "none", value: "0", opens: false });
  });

  it("show a dash while the server counts, and while it isn't running", () => {
    expect(orphansCount(facts(made(true), { inventory: undefined }))).toMatchObject({
      state: "running",
      value: "—",
      note: "Counting…",
    });
    expect(unusedCount(facts(made(true), { inventory: undefined, state: "failed" }))).toMatchObject(
      {
        state: "unknown",
        value: "—",
        note: "Counted while the language server runs: it's failed",
      },
    );
  });
});

describe("agentFilesCount", () => {
  const files = (agentFiles: AgentFilesResult) =>
    agentFilesCount(facts(made(true), { agentFiles }));

  it("is unknown until checked, with the check beside it, and says when it's checking", () => {
    expect(files({ kind: "none" })).toMatchObject({
      state: "unknown",
      value: "—",
      note: "Needs the agent files checked",
      action: { key: PROJECT_KEYS.checkAgentFiles, label: "Check Agent Files" },
    });
    expect(files({ kind: "running" })).toMatchObject({
      state: "running",
      note: "Checking the agent files…",
      action: null,
    });
  });

  it("counts the stale files from the last save, opening Sync Agent Files", () => {
    expect(
      files({
        kind: "made",
        at: 7000,
        stale: ["AGENTS.md", ".claude/skills/ascribe/SKILL.md"],
        error: null,
        current: true,
      }),
    ).toMatchObject({
      key: PROJECT_KEYS.agentFiles,
      state: "current",
      value: "2 files",
      note: "Checked at 7s, after the last save",
      opens: true,
    });
    expect(files({ kind: "made", at: 7000, stale: [], error: null, current: true })).toMatchObject({
      state: "none",
      value: "0",
      opens: false,
    });
  });

  it("goes out of date on a change on disk, and says why a check couldn't finish", () => {
    expect(
      files({ kind: "made", at: 7000, stale: ["AGENTS.md"], error: null, current: false }),
    ).toMatchObject({
      state: "outOfDate",
      value: "—",
      note: "Out of date: a file changed on disk after the check at 7s",
      opens: false,
      action: { key: PROJECT_KEYS.checkAgentFiles },
    });
    expect(
      files({ kind: "made", at: 7000, stale: [], error: "no ascribe.toml", current: true }),
    ).toMatchObject({
      state: "outOfDate",
      note: "The check at 7s couldn't finish: no ascribe.toml",
    });
  });

  it("reads the stale files from ascribe agents sync --check", () => {
    expect(
      staleFiles(
        "stale      AGENTS.md\nup to date .claude/skills/ascribe/SKILL.md\r\nstale      ../.github/copilot-instructions.md\n",
      ),
    ).toEqual(["AGENTS.md", "../.github/copilot-instructions.md"]);
  });

  it("asks before Sync Agent Files, naming the files it would change", () => {
    expect(syncQuestion("docs", ["AGENTS.md", "CLAUDE.md"])).toEqual({
      message: "2 agent files are out of date in docs. Sync Agent Files?",
      detail: "ascribe agents sync would change:\nAGENTS.md\nCLAUDE.md",
    });
    expect(syncQuestion("docs", ["AGENTS.md"]).message).toBe(
      "1 agent file is out of date in docs. Sync Agent Files?",
    );
  });
});
