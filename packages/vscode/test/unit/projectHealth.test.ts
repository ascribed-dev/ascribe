import { describe, expect, it } from "vitest";
import type { Counted, Found } from "../../src/everyBuild.js";
import type { InventoryResult } from "../../src/shapes.js";
import type { AgentFilesResult } from "../../src/ui/agentFiles.js";
import type { EveryBuildResult } from "../../src/ui/checkEveryBuild.js";
import {
  asYouTypeGroup,
  everyBuildGroup,
  inventoryCounts,
  lastSaveGroup,
  NO_PROJECT,
  nothingToEdit,
  PROJECT_KEYS,
  projectAnswer,
  projectEditQuestion,
  RESTRICTED,
  restrictedAnswer,
  aboutRows,
  staleFiles,
  syncQuestion,
  type ProjectFacts,
} from "../../src/ui/projectHealth.js";
import type { ProjectHealthGroup } from "../../src/webview/viewProtocol.js";

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
  builds: [],
  build: undefined,
  about: { contentRoot: undefined, types: undefined, sources: "asking" },
  ...more,
});

/** A group's count of `key`. */
const countOf = (group: ProjectHealthGroup, key: string) =>
  group.counts.find((item) => item.key === key);

/** Every action's key, in its group's order. */
const actionKeys = (answer: ReturnType<typeof projectAnswer>) =>
  answer.actions.flatMap((group) => group.actions.map((action) => action.key));

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
      notice: [],
      head: null,
      toDo: 0,
      health: [],
      actions: [],
      about: [],
    });
  });

  it("heads with the project's name, folder, server state with its buttons, version, and settings, and offers the actions in their groups", () => {
    const answer = projectAnswer({
      ...facts({ kind: "none" }),
      state: "failed",
      version: undefined,
    });
    expect(answer.message).toBeNull();
    expect(answer.head).toMatchObject({
      name: "docs",
      folder: "docs",
      state: "failed",
      version: null,
      build: null,
    });
    expect(answer.head?.server.map((button) => button.key)).toEqual([
      PROJECT_KEYS.showOutput,
      PROJECT_KEYS.restart,
    ]);
    expect(answer.head?.settings.map((button) => [button.key, button.icon])).toEqual([
      [PROJECT_KEYS.projectSettings, "settings-gear"],
      [PROJECT_KEYS.editorSettings, "settings"],
    ]);
    expect(answer.actions.map((group) => group.label)).toEqual([
      "Check",
      "Write",
      "Review and agents",
    ]);
    expect(actionKeys(answer)).toEqual([
      PROJECT_KEYS.checkEveryBuild,
      PROJECT_KEYS.applyEverySafeFix,
      PROJECT_KEYS.formatEveryPage,
      PROJECT_KEYS.build,
      PROJECT_KEYS.syncAgentFiles,
      PROJECT_KEYS.observeExamples,
      PROJECT_KEYS.startReview,
      PROJECT_KEYS.promptAgent,
    ]);
    expect(answer.actions.flatMap((group) => group.actions).some((action) => action.off)).toBe(
      false,
    );
  });

  it("offers Restart only once the server has started", () => {
    const stopped = projectAnswer({ ...facts({ kind: "none" }), state: "stopped" });
    expect(stopped.head?.server.map((button) => button.key)).toEqual([PROJECT_KEYS.showOutput]);
  });

  it("switches among the builds the server lists, a versioned build's lines among them, showing the one you're looking at", () => {
    const builds = [
      { name: "site", editor: true },
      {
        name: "api",
        editor: false,
        lines: [
          { name: "api", line: "3.5", current: true },
          { name: "api@3.4", line: "3.4", current: false },
        ],
      },
    ];
    const editor = projectAnswer(facts({ kind: "none" }, { builds }));
    expect(editor.head?.build).toEqual({
      shown: "site",
      choices: [
        { name: "site", description: "editor build" },
        { name: "api", description: "line 3.5, the current line" },
        { name: "api@3.4", description: "line 3.4 of api" },
      ],
    });
    const chosen = projectAnswer(facts({ kind: "none" }, { builds, build: "api@3.4" }));
    expect(chosen.head?.build?.shown).toBe("api@3.4");
  });

  it("says in Restricted Mode that Ascribe isn't running, with every action off and Manage Workspace Trust", () => {
    const answer = restrictedAnswer();
    expect(answer.message).toBe(RESTRICTED);
    expect(answer.notice).toEqual([
      { key: PROJECT_KEYS.manageTrust, label: "Manage Workspace Trust" },
    ]);
    expect(answer.health).toEqual([]);
    expect(answer.head).toBeNull();
    expect(actionKeys(answer)).toEqual(actionKeys(projectAnswer(facts({ kind: "none" }))));
    expect(answer.actions.flatMap((group) => group.actions).every((action) => action.off)).toBe(
      true,
    );
  });

  it("says each task runs in the terminal", () => {
    const answer = projectAnswer(facts({ kind: "none" }));
    const tasks = answer.actions[1]?.actions.slice(2) ?? [];
    expect(tasks.map((action) => action.key)).toEqual([
      PROJECT_KEYS.build,
      PROJECT_KEYS.syncAgentFiles,
      PROJECT_KEYS.observeExamples,
    ]);
    for (const action of tasks) expect(action.caption).toMatch(/^A task: /);
  });

  it("says each edit is one edit left unsaved, and that the safe fixes are the editor build's", () => {
    const [fixAll, format] = projectAnswer(facts({ kind: "none" })).actions[1]?.actions ?? [];
    expect(fixAll?.caption).toMatch(
      /the editor build's, in every file, as one edit left unsaved\./,
    );
    expect(fixAll?.caption).toMatch(/Every build's are ascribe check --fix's\.$/);
    expect(format?.caption).toMatch(/as one edit left unsaved\.$/);
  });

  it("counts a thing to do for each current count above zero, and none for a group that isn't current", () => {
    const clean = { errors: 0, warnings: 0, advice: 0 };
    // Not checked, being checked, or out of date: the group's head says so, with its action.
    expect(projectAnswer(facts({ kind: "none" })).toDo).toBe(0);
    expect(projectAnswer(facts({ kind: "running" })).toDo).toBe(0);
    expect(projectAnswer(facts(made(false))).toDo).toBe(0);
    expect(projectAnswer(facts(made(true))).toDo).toBe(1);
    expect(projectAnswer(facts(made(true, clean))).toDo).toBe(0);
    // The mock's: unused entries and agent files.
    expect(
      projectAnswer(
        facts(
          { kind: "none" },
          {
            inventory: { orphans: 0, unused: 5 },
            agentFiles: { kind: "made", at: 7000, stale: ["a", "b"], error: null, current: true },
          },
        ),
      ).toDo,
    ).toBe(2);
    // Overdue reviews from another day aren't counted, though their group is current.
    expect(
      projectAnswer(facts(made(true, clean, { counted: { reviews: found(2) } }), { now: DAY + 1 }))
        .toDo,
    ).toBe(0);
  });

  it("groups Health's counts by when each is from", () => {
    const health = projectAnswer(facts({ kind: "none" })).health;
    expect(health.map((group) => [group.key, group.label])).toEqual([
      ["everyBuild", "Every build"],
      ["asYouType", "As you type"],
      ["lastSave", "At the last save"],
    ]);
    expect(health.map((group) => group.counts.map((item) => [item.key, item.label]))).toEqual([
      [
        [PROJECT_KEYS.problems, "Problems"],
        [PROJECT_KEYS.examples, "Tested examples"],
        [PROJECT_KEYS.reviews, "Reviews overdue"],
        [PROJECT_KEYS.permalinks, "Permalinks behind"],
      ],
      [
        [PROJECT_KEYS.orphans, "Pages nothing links to"],
        [PROJECT_KEYS.unused, "Unused model entries"],
      ],
      [[PROJECT_KEYS.agentFiles, "Agent files out of date"]],
    ]);
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

describe("everyBuildGroup", () => {
  const keys = [
    PROJECT_KEYS.problems,
    PROJECT_KEYS.examples,
    PROJECT_KEYS.reviews,
    PROJECT_KEYS.permalinks,
  ];
  const check = { key: PROJECT_KEYS.checkEveryBuild, label: "Check Every Build" };

  /** Each of the group's counts is a dash in `state`, with no note or action of its own. */
  const dashes = (group: ProjectHealthGroup, state: string) =>
    expect(group.counts).toEqual(
      keys.map((key) =>
        expect.objectContaining({ key, state, value: "—", note: null, opens: false, action: null }),
      ),
    );

  it("says once that every build isn't checked yet, with Check Every Build at its head", () => {
    const group = everyBuildGroup(facts({ kind: "none" }));
    expect(group).toMatchObject({
      key: "everyBuild",
      label: "Every build",
      state: "unknown",
      note: "not checked yet",
      action: check,
    });
    dashes(group, "unknown");
  });

  it("says it's checking while the check runs, with no action", () => {
    const group = everyBuildGroup(facts({ kind: "running" }));
    expect(group).toMatchObject({ state: "running", note: "checking…", action: null });
    dashes(group, "running");
  });

  it("says it's reading the last check while one kept from an earlier session is compared, with no action", () => {
    const group = everyBuildGroup(facts({ kind: "comparing" }));
    expect(group).toMatchObject({
      state: "running",
      note: "reading the last check…",
      action: null,
    });
    dashes(group, "running");
  });

  it("says which day a check from another day is from: yesterday, or its date", () => {
    const yesterday = facts(made(false), { now: DAY + 6000 });
    expect(everyBuildGroup(yesterday).note).toBe(
      "out of date: the project changed after the check yesterday at 5s",
    );
    const before = facts(made(true), { now: 3 * DAY });
    expect(everyBuildGroup(before).note).toBe("checked on day 0 at 5s");
  });

  it("goes out of date once the project changes, saying why, with the check at its head", () => {
    const group = everyBuildGroup(
      facts(made(false, undefined, { counted: { examples: found(2) } })),
    );
    expect(group).toMatchObject({
      state: "outOfDate",
      note: "out of date: the project changed after the check at 5s",
      action: check,
    });
    dashes(group, "outOfDate");
  });

  it("says why a check couldn't finish", () => {
    const group = everyBuildGroup(facts(made(false, undefined, { error: "ascribe.toml: bad" })));
    expect(group).toMatchObject({
      state: "outOfDate",
      note: "the check at 5s couldn't finish: ascribe.toml: bad",
      action: check,
    });
    dashes(group, "outOfDate");
  });

  it("counts while current, saying when it's from, each count opening its list while above zero", () => {
    const group = everyBuildGroup(
      facts(
        made(true, undefined, {
          counted: { examples: found(3), reviews: found(1), permalinks: found(2) },
        }),
      ),
    );
    expect(group).toMatchObject({ state: "current", note: "checked at 5s", action: null });
    expect(group.counts).toEqual([
      {
        key: PROJECT_KEYS.problems,
        label: "Problems",
        state: "current",
        value: "2 errors, 1 warning, 3 advice",
        note: null,
        opens: true,
        action: null,
      },
      expect.objectContaining({ state: "current", value: "3 examples", opens: true }),
      expect.objectContaining({ state: "current", value: "1 page", opens: true }),
      expect.objectContaining({ state: "current", value: "2 permalinks", opens: true }),
    ]);
    expect(
      countOf(everyBuildGroup(facts(made(true, { errors: 1, warnings: 0, advice: 0 }))), "problems")
        ?.value,
    ).toBe("1 error");
  });

  it("shows none as 0, which opens nothing", () => {
    const group = everyBuildGroup(facts(made(true, { errors: 0, warnings: 0, advice: 0 })));
    for (const item of group.counts) {
      expect(item).toMatchObject({ state: "none", value: "0", opens: false });
    }
  });

  it("says when files with unsaved changes were read as saved", () => {
    expect(everyBuildGroup(facts(made(true, undefined, { unsaved: 2 }))).note).toBe(
      "checked at 5s, 2 files with unsaved changes read as saved",
    );
  });

  it("leaves overdue reviews alone a dash when the day changes, with their own note and the check", () => {
    const group = everyBuildGroup(
      facts(made(true, undefined, { counted: { reviews: found(1) } }), { now: DAY + 1 }),
    );
    expect(group).toMatchObject({
      state: "current",
      note: "checked yesterday at 5s",
      action: null,
    });
    expect(countOf(group, PROJECT_KEYS.reviews)).toEqual({
      key: PROJECT_KEYS.reviews,
      label: "Reviews overdue",
      state: "outOfDate",
      value: "—",
      note: "checked on day 0; reviews fall due by the day",
      opens: false,
      action: check,
    });
    // The other counts from the same check stay current.
    expect(countOf(group, PROJECT_KEYS.examples)?.state).toBe("none");
    expect(countOf(group, PROJECT_KEYS.problems)?.state).toBe("current");
  });
});

describe("asYouTypeGroup", () => {
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

  it("is current as the server answers, with nothing more to say, each count opening its list", () => {
    const group = asYouTypeGroup(facts(made(true), { inventory: { orphans: 1, unused: 4 } }));
    expect(group).toMatchObject({
      key: "asYouType",
      label: "As you type",
      state: "current",
      note: "",
      action: null,
    });
    expect(group.counts).toEqual([
      expect.objectContaining({ key: PROJECT_KEYS.orphans, value: "1 page", opens: true }),
      expect.objectContaining({ key: PROJECT_KEYS.unused, value: "4 entries", opens: true }),
    ]);
    const none = asYouTypeGroup(facts(made(true), { inventory: { orphans: 0, unused: 0 } }));
    for (const item of none.counts) {
      expect(item).toMatchObject({ state: "none", value: "0", opens: false });
    }
  });

  it("shows dashes while the server counts, and while it isn't running", () => {
    expect(asYouTypeGroup(facts(made(true), { inventory: undefined }))).toMatchObject({
      state: "running",
      note: "counting…",
      action: null,
      counts: [{ value: "—" }, { value: "—" }],
    });
    expect(
      asYouTypeGroup(facts(made(true), { inventory: undefined, state: "failed" })),
    ).toMatchObject({
      state: "unknown",
      note: "counted while the language server runs: it's failed",
      action: null,
    });
  });
});

describe("lastSaveGroup", () => {
  const files = (agentFiles: AgentFilesResult) => lastSaveGroup(facts(made(true), { agentFiles }));
  const check = { key: PROJECT_KEYS.checkAgentFiles, label: "Check Agent Files" };

  it("is unknown until checked, with the check at its head, and says when it's checking", () => {
    expect(files({ kind: "none" })).toMatchObject({
      key: "lastSave",
      label: "At the last save",
      state: "unknown",
      note: "not checked yet",
      action: check,
      counts: [{ state: "unknown", value: "—", note: null, action: null }],
    });
    expect(files({ kind: "running" })).toMatchObject({
      state: "running",
      note: "checking…",
      action: null,
    });
  });

  it("counts the stale files from the last save, at its time, opening Sync Agent Files", () => {
    const group = files({
      kind: "made",
      at: 7000,
      stale: ["AGENTS.md", ".claude/skills/ascribe/SKILL.md"],
      error: null,
      current: true,
    });
    expect(group).toMatchObject({ state: "current", note: "7s", action: null });
    expect(group.counts).toEqual([
      expect.objectContaining({
        key: PROJECT_KEYS.agentFiles,
        state: "current",
        value: "2 files",
        opens: true,
      }),
    ]);
    expect(
      files({ kind: "made", at: 7000, stale: [], error: null, current: true }).counts[0],
    ).toMatchObject({ state: "none", value: "0", opens: false });
  });

  it("goes out of date on a change on disk, and says why a check couldn't finish", () => {
    expect(
      files({ kind: "made", at: 7000, stale: ["AGENTS.md"], error: null, current: false }),
    ).toMatchObject({
      state: "outOfDate",
      note: "out of date: a file changed on disk after the check at 7s",
      action: check,
      counts: [{ value: "—", opens: false }],
    });
    expect(
      files({ kind: "made", at: 7000, stale: [], error: "no ascribe.toml", current: true }),
    ).toMatchObject({
      state: "outOfDate",
      note: "the check at 7s couldn't finish: no ascribe.toml",
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

describe("aboutRows", () => {
  it("says what's known only once the server and the binary have said it", () => {
    expect(aboutRows(facts({ kind: "none" }))).toEqual([
      { label: "Content root", value: "Known once the language server runs" },
      { label: "Builds", value: "Known once the language server runs" },
      { label: "Release lines", value: "Known once the language server runs" },
      { label: "Sources", value: "Asking ascribe model…" },
      { label: "Content types", value: "Known once the language server runs" },
    ]);
    expect(
      aboutRows(
        facts(
          { kind: "none" },
          {
            about: { contentRoot: undefined, types: undefined, sources: "failed" },
          },
        ),
      )[3],
    ).toEqual({ label: "Sources", value: "Not known: ascribe model couldn't answer" });
  });

  it("names the content root, the builds, the release lines, the sources, and the content types", () => {
    const rows = aboutRows(
      facts(
        { kind: "none" },
        {
          builds: [
            { name: "site", editor: true },
            {
              name: "api",
              editor: false,
              lines: [
                { name: "api", line: "3.5", current: true },
                { name: "api@3.4", line: "3.4", current: false },
              ],
            },
          ],
          about: {
            contentRoot: "docs/content",
            types: [
              { key: "page", pages: 1 },
              { key: "guide", pages: 12 },
            ],
            sources: [
              { name: "code", path: "examples", tested: true },
              { name: "api", path: "vendor/api", tested: false },
            ],
          },
        },
      ),
    );
    expect(rows.map(({ label, value }) => `${label}: ${value}`)).toEqual([
      "Content root: docs/content",
      "Builds: site (editor build), api",
      "Release lines: api: 3.5 (current), 3.4",
      "Sources: code (examples, tested examples), api (vendor/api)",
      "Content types: page (1 page), guide (12 pages)",
    ]);
  });

  it("says when no build is versioned and there are no sources", () => {
    const rows = aboutRows(
      facts(
        { kind: "none" },
        {
          builds: [{ name: "site", editor: true }],
          about: { contentRoot: ".", types: [], sources: [] },
        },
      ),
    );
    expect(rows[2]).toEqual({ label: "Release lines", value: "None: no build is versioned" });
    expect(rows[3]).toEqual({ label: "Sources", value: "None" });
  });
});
