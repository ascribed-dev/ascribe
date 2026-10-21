import { describe, expect, it } from "vitest";
import type { SettingEntry, SettingsResult } from "../../src/shapes.js";
import {
  checkSettingsMessage,
  checkedValue,
  inStep,
  SHOWN,
  setInStep,
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
    ...more,
  };
}

describe("Project Settings' answer", () => {
  it("shows the settings it shows, in its order, each with its value, default, and place", () => {
    const answer = settingsAnswer(result());
    expect(answer.message).toBeNull();
    expect(answer.settings.map((s) => s.id)).toEqual([...SHOWN]);
    expect(answer.settings[0]).toMatchObject({
      key: "[consumer] trailing-slash",
      value: "never",
      defaultText: '"always"',
      where: "ascribe.toml:11",
      set: true,
      editable: true,
    });
  });

  it("shows a setting the file leaves out with its default, which it doesn't change yet", () => {
    const answer = settingsAnswer(
      result({ settings: [entry("trailing-slash", { default: "always" })] }),
    );
    expect(answer.settings).toHaveLength(1);
    expect(answer.settings[0]).toMatchObject({
      value: "always",
      where: "Not set in ascribe.toml",
      set: false,
      editable: false,
    });
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
      settings: [],
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
    const [slash, base] = settingsAnswer(result()).settings as [SettingView, SettingView];
    expect(checkedValue(slash, "never")).toBe("never");
    expect(checkedValue(slash, "sometimes")).toBeUndefined();
    expect(checkedValue(base, "/docs/")).toBe("/docs/");
    expect(checkedValue(base, "two\nlines")).toBeUndefined();
    expect(checkedValue({ ...base, kind: "number" }, 4)).toBe(4);
    expect(checkedValue({ ...base, kind: "number" }, 4.5)).toBeUndefined();
    expect(checkedValue({ ...base, kind: "switch" }, "true")).toBeUndefined();
  });

  it("takes the webview's messages, and nothing else", () => {
    expect(
      checkSettingsMessage({ type: "set", seq: 2, id: "consumer.base-path", value: "/" }),
    ).toEqual({ type: "set", seq: 2, id: "consumer.base-path", value: "/" });
    expect(checkSettingsMessage({ type: "set", seq: 2, id: "x", value: {} })).toBeUndefined();
    expect(checkSettingsMessage({ type: "openLine", seq: -1, id: "x" })).toBeUndefined();
    expect(checkSettingsMessage({ type: "ready" })).toEqual({ type: "ready" });
    expect(checkSettingsMessage({ type: "drawn", seq: 1, report: {} })).toBeUndefined();
    expect(checkSettingsMessage({ type: "run" })).toBeUndefined();
  });
});
