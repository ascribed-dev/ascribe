import { describe, expect, it } from "vitest";
import type { CheckEntry, SettingEntry, SettingsResult } from "../../src/shapes.js";
import {
  allSettings,
  checkSettingsMessage,
  checkedValue,
  inStep,
  setInStep,
  settingTarget,
  settingsAnswer,
} from "../../src/ui/settingsModel.js";
import type { SettingView } from "../../src/webview/settingsProtocol.js";

function entry(key: string, more: Partial<SettingEntry> = {}): SettingEntry {
  return {
    table: "consumer",
    key,
    name: key,
    kind: "text",
    choices: [],
    default: "/",
    defaultText: null,
    description: [{ text: "What it does.", code: false }],
    value: null,
    set: null,
    problems: [],
    ...more,
  };
}

const range = { start: { line: 9, character: 0 }, end: { line: 9, character: 24 } };

function check(slug: string, more: Partial<CheckEntry> = {}): CheckEntry {
  return {
    slug,
    code: "ASC149",
    name: "Page size",
    area: "Pages",
    when: [{ text: "A page is long.", code: false }],
    choices: ["off", "advice", "warning", "error"],
    default: "advice",
    named: false,
    level: null,
    set: null,
    problems: [],
    limit: null,
    ...more,
  };
}

function result(more: Partial<SettingsResult> = {}): SettingsResult {
  return {
    modelUri: "file:///p/ascribe.toml",
    version: 3,
    unreadable: null,
    settings: [
      entry("site", { default: null }),
      entry("base-path", { value: "docs", set: { range, line: 10 } }),
      entry("trailing-slash", {
        kind: "choice",
        choices: ["always", "never"],
        default: "always",
        value: "never",
        set: { range, line: 11 },
      }),
    ],
    checks: [],
    parts: [],
    ...more,
  };
}

describe("Project Settings' answer", () => {
  it("shows every setting, in the server's order, each with its value, default, and place", () => {
    const answer = settingsAnswer(result());
    expect(answer.message).toBeNull();
    expect(answer.settings.map((s) => s.id)).toEqual([
      "consumer.site",
      "consumer.base-path",
      "consumer.trailing-slash",
    ]);
    expect(answer.settings[2]).toMatchObject({
      section: "[consumer]",
      key: "[consumer] trailing-slash",
      value: "never",
      defaultText: '"always"',
      where: "ascribe.toml:11",
      set: true,
      editable: true,
      resettable: true,
    });
  });

  it("shows a setting the file leaves out with its default, which a change adds", () => {
    const answer = settingsAnswer(
      result({ settings: [entry("trailing-slash", { default: "always" })] }),
    );
    expect(answer.settings).toHaveLength(1);
    expect(answer.settings[0]).toMatchObject({
      value: "always",
      where: "Not set in ascribe.toml",
      set: false,
      editable: true,
      resettable: false,
    });
    expect(
      settingsAnswer(result({ settings: [entry("spec", { table: "" })] })).settings[0],
    ).toMatchObject({ section: "The top level", key: "spec" });
  });

  it("shows each check's level and limit as settings of its own table", () => {
    const answer = settingsAnswer(
      result({
        checks: [
          check("page-orphan", {
            name: "Page orphan",
            named: true,
            level: "off",
            set: { range, line: 20 },
          }),
          check("page-size", {
            limit: {
              kind: "number",
              default: 50_000,
              defaultText: "50,000 characters",
              value: null,
              set: null,
              problems: [],
            },
          }),
        ],
      }),
    );
    const [orphan, size] = answer.checks;
    expect(orphan).toMatchObject({ slug: "page-orphan", named: true, limit: null });
    expect(orphan?.level).toMatchObject({
      id: "checks.page-orphan.level",
      key: "[checks] page-orphan",
      kind: "level",
      value: "off",
      defaultText: "advice",
      where: "ascribe.toml:20",
      resettable: true,
      description: [{ text: "A page is long.", code: false }],
    });
    // A check the file leaves out shows its own level.
    expect(size?.level).toMatchObject({ value: "advice", set: false, editable: true });
    expect(size?.limit).toMatchObject({
      id: "checks.page-size.limit",
      key: "[checks.page-size] limit",
      kind: "number",
      value: 50_000,
      defaultText: "50,000 characters",
      resettable: false,
    });
    expect(allSettings(answer).map((s) => s.id)).toEqual([
      "consumer.site",
      "consumer.base-path",
      "consumer.trailing-slash",
      "checks.page-orphan.level",
      "checks.page-size.level",
      "checks.page-size.limit",
    ]);
  });

  it("finds the table and key the server sets for each id", () => {
    const answer = result({ checks: [check("page-orphan", { set: { range, line: 20 } })] });
    expect(settingTarget(answer, "consumer.base-path")).toEqual({
      table: "consumer",
      key: "base-path",
      set: { range, line: 10 },
    });
    expect(settingTarget(answer, "checks.page-orphan.level")).toEqual({
      table: "checks.page-orphan",
      key: "level",
      set: { range, line: 20 },
    });
    expect(settingTarget(answer, "checks.page-orphan.limit")).toBeUndefined();
    expect(settingTarget(answer, "consumer.nothing")).toBeUndefined();
  });

  it("shows each part with structure as a card under its section, which opens the file at its table", () => {
    const answer = settingsAnswer(
      result({
        parts: [
          {
            table: "types",
            name: "guide",
            keys: ["purpose", "frontmatter"],
            set: { range, line: 14 },
          },
          { table: "builds", name: "cloud", keys: [], set: null },
        ],
      }),
    );
    expect(answer.parts).toEqual([
      {
        id: "part:types.guide",
        section: "Content types",
        name: "guide",
        key: "[types.guide]",
        summary: "Sets purpose, frontmatter",
        where: "ascribe.toml:14",
      },
      {
        id: "part:builds.cloud",
        section: "Builds",
        name: "cloud",
        key: "[builds.cloud]",
        summary: "Sets nothing yet",
        where: "Not set in ascribe.toml",
      },
    ]);
    // A card's link opens its line; no setting has its id, so none is set.
    const shared = { table: "", name: "frontmatter", keys: ["title"], set: { range, line: 3 } };
    expect(settingsAnswer(result({ parts: [shared] })).parts).toEqual([
      {
        id: "part:frontmatter",
        section: "Content types",
        name: "frontmatter",
        key: "[frontmatter]",
        summary: "Sets title",
        where: "ascribe.toml:3",
      },
    ]);
    expect(settingTarget(result({ parts: [shared] }), "part:frontmatter")).toEqual({
      table: "",
      key: "frontmatter",
      set: { range, line: 3 },
    });
    expect(settingTarget(result({ parts: [] }), "part:types.guide")).toBeUndefined();
    expect(
      settingTarget(
        result({
          parts: [{ table: "types", name: "guide", keys: [], set: { range, line: 14 } }],
        }),
        "part:types.guide",
      ),
    ).toEqual({ table: "types", key: "guide", set: { range, line: 14 } });
    expect(allSettings(answer).map((s) => s.id)).not.toContain("part:types.guide");
  });

  it("doesn't change a value the file writes in a form it can't show", () => {
    const answer = settingsAnswer(
      result({ settings: [entry("base-path", { value: null, set: { range, line: 4 } })] }),
    );
    expect(answer.settings[0]).toMatchObject({ value: null, set: true, editable: false });
  });

  it("says why when the file can't be read", () => {
    expect(settingsAnswer(result({ unreadable: "`ascribe.toml` isn't valid TOML: x" }))).toEqual({
      message: "`ascribe.toml` isn't valid TOML: x Fix it in the file to see its settings.",
      gone: false,
      settings: [],
      checks: [],
      parts: [],
    });
  });
});

describe("the rule the tab keeps", () => {
  it("draws an answer only for the text the editor has", () => {
    expect(inStep(4, 4)).toBe(true);
    expect(inStep(3, 4)).toBe(false);
    expect(inStep(null, undefined)).toBe(true);
    // The file was opened, or closed, while the server read it.
    expect(inStep(null, 1)).toBe(false);
    expect(inStep(5, undefined)).toBe(false);
  });

  it("applies a change only to the text it was planned for", async () => {
    let open: number | undefined = 4;
    const applied: string[] = [];
    const apply = (edit: string) => {
      applied.push(edit);
      return Promise.resolve(true);
    };
    expect(
      await setInStep(
        4,
        () => open,
        () => Promise.resolve({ edit: "a" }),
        apply,
      ),
    ).toEqual({ type: "applied", ok: true });
    // Typed in the file while the server planned it.
    const typed = () => {
      open = 5;
      return Promise.resolve({ edit: "b" });
    };
    expect(await setInStep(4, () => open, typed, apply)).toEqual({ type: "stale" });
    // Closed while the server planned it: the disk isn't the buffer it was shown from.
    const closed = () => {
      open = undefined;
      return Promise.resolve({ edit: "c" });
    };
    expect(await setInStep(5, () => open, closed, apply)).toEqual({ type: "stale" });
    expect(
      await setInStep(
        null,
        () => open,
        () => Promise.resolve({ error: "No." }),
        apply,
      ),
    ).toEqual({ type: "refused", error: "No." });
    expect(applied).toEqual(["a"]);
  });

  it("sends a value only of the setting's kind", () => {
    const [, base, slash] = settingsAnswer(result()).settings as [
      SettingView,
      SettingView,
      SettingView,
    ];
    expect(checkedValue(slash, "never")).toBe("never");
    expect(checkedValue(slash, "sometimes")).toBeUndefined();
    expect(checkedValue(base, "/docs/")).toBe("/docs/");
    expect(checkedValue(base, "two\nlines")).toBeUndefined();
    expect(checkedValue({ ...base, kind: "number" }, 4)).toBe(4);
    expect(checkedValue({ ...base, kind: "number" }, 4.5)).toBeUndefined();
    expect(checkedValue({ ...base, kind: "switch" }, "true")).toBeUndefined();
    expect(checkedValue({ ...base, kind: "size" }, 200_000)).toBe(200_000);
    expect(checkedValue({ ...base, kind: "size" }, "200 KB")).toBe("200 KB");
    expect(checkedValue({ ...base, kind: "size" }, " ")).toBeUndefined();
  });

  it("takes the webview's messages, and nothing else", () => {
    expect(
      checkSettingsMessage({ type: "set", seq: 2, id: "consumer.base-path", value: "/" }),
    ).toEqual({ type: "set", seq: 2, id: "consumer.base-path", value: "/" });
    expect(checkSettingsMessage({ type: "set", seq: 2, id: "x", value: {} })).toBeUndefined();
    expect(checkSettingsMessage({ type: "openLine", seq: -1, id: "x" })).toBeUndefined();
    expect(checkSettingsMessage({ type: "reset", seq: 2, id: "consumer.site" })).toEqual({
      type: "reset",
      seq: 2,
      id: "consumer.site",
    });
    expect(checkSettingsMessage({ type: "reset", seq: 2 })).toBeUndefined();
    expect(checkSettingsMessage({ type: "ready" })).toEqual({ type: "ready" });
    expect(checkSettingsMessage({ type: "contentModel" })).toEqual({ type: "contentModel" });
    expect(checkSettingsMessage({ type: "close" })).toEqual({ type: "close" });
    expect(checkSettingsMessage({ type: "focus", focused: true })).toEqual({
      type: "focus",
      focused: true,
    });
    expect(checkSettingsMessage({ type: "focus" })).toBeUndefined();
    expect(checkSettingsMessage({ type: "drawn", seq: 1, report: {} })).toBeUndefined();
    expect(checkSettingsMessage({ type: "run" })).toBeUndefined();
  });
});
