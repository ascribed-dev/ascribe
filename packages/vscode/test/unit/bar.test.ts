import { describe, expect, it } from "vitest";
import { entries, NOTHING, shown, type Entry, type Fix } from "../../src/actions/bar.js";
import type { ContextNode, ContextResult, LspRange } from "../../src/shapes.js";

const r = (line: number): LspRange => ({
  start: { line, character: 0 },
  end: { line, character: 10 },
});

function context(at: ContextNode[], more: Partial<ContextResult> = {}): ContextResult {
  return {
    project: { root: "/quill", editorBuild: "site" },
    at,
    selection: null,
    token: null,
    insertable: false,
    ...more,
  };
}

const section: ContextNode = { kind: "section", range: r(1), headingId: "install" };
const paragraph: ContextNode = { kind: "paragraph", range: r(2) };
const note: ContextNode = { kind: "note", range: r(3), type: "tip", form: "block" };
const heading: ContextNode = {
  kind: "heading",
  range: r(1),
  level: 2,
  id: "install-cli",
  explicitId: false,
};
const numbered: ContextNode = { kind: "list", range: r(4), ordered: true, steps: false };

const fix = (title: string, more: Partial<Fix> = {}): Fix => ({
  title,
  kind: "quickfix",
  problem: undefined,
  disabled: false,
  ...more,
});

/** The rows as the bar shows them: a separator as `-- Label`, any other as its label. */
const rows = (list: Entry[]): string[] =>
  list.map((entry) => (entry.kind === "separator" ? `-- ${entry.label}` : shown(entry).label));

describe("the actions bar", () => {
  it("lists the actions that apply, by group, each group under a separator", () => {
    expect(rows(entries(context([paragraph, note, section]), []))).toEqual([
      "-- Write",
      "Insert a phrase",
      "-- Structure",
      "Wrap in a note",
      "Change the note's kind",
      "Remove the note, keeping its text",
      "Turn the note into collapsible details",
      "Wrap in collapsible details",
      "Mark where it's available",
      "Make the page one variant",
      "Set where the page is available",
      "-- Link",
      "Insert a link",
    ]);
  });

  it("lists the fixes for problems at the cursor first, with the problem as the detail", () => {
    const list = entries(context([paragraph, section], { selection: null }), [
      fix("Declare phrase in ascribe.toml", { problem: "The phrase {flush} isn't declared." }),
      fix("Escape this phrase as literal text", { problem: "The phrase {flush} isn't declared." }),
    ]);
    expect(rows(list).slice(0, 4)).toEqual([
      "-- Fix",
      "Declare phrase in ascribe.toml",
      "Escape this phrase as literal text",
      "-- Write",
    ]);
    expect(shown(list[1] as Entry)).toEqual({
      label: "Declare phrase in ascribe.toml",
      detail: "The phrase {flush} isn't declared.",
    });
  });

  it("lists an action once when the lightbulb offers it as a quick fix too", () => {
    const list = entries(context([paragraph, section]), [
      fix("Wrap in a note", { kind: "quickfix.ascribe" }),
      fix("Remove the stray colon"),
    ]);
    expect(rows(list).filter((label) => label === "Wrap in a note")).toHaveLength(1);
    expect(rows(list).slice(0, 2)).toEqual(["-- Fix", "Remove the stray colon"]);
    const titles = list.flatMap((entry) => (entry.kind === "action" ? [entry.action.id] : []));
    expect(new Set(titles).size).toBe(titles.length);
  });

  it("leaves out a fix its provider disabled", () => {
    const list = entries(context([paragraph, section]), [fix("Not here", { disabled: true })]);
    expect(rows(list)[0]).toBe("-- Write");
  });

  it("shows each action's description plainly, and what it writes when that's known", () => {
    const list = entries(context([heading, section]), []);
    const addId = list.find(
      (entry) => entry.kind === "action" && entry.action.id === "addHeadingId",
    );
    expect(shown(addId as Entry)).toEqual({
      label: "Give the heading a stable id",
      description: "Add @id: under the heading, so links to it survive rewording it",
      detail: "@id: install-cli",
    });
    const steps = entries(
      context([paragraph, { kind: "listItem", range: r(4) }, numbered, section]),
      [],
    );
    const makeSteps = steps.find(
      (entry) => entry.kind === "action" && entry.action.id === "makeSteps",
    );
    expect(shown(makeSteps as Entry).detail).toBe("@steps");
  });

  it("says what to try when nothing applies", () => {
    const outside: ContextResult = {
      project: null,
      at: [],
      selection: null,
      token: null,
      insertable: false,
    };
    expect(entries(outside, [])).toEqual([{ kind: "empty", label: NOTHING }]);
  });

  it("lists only the fixes when no action applies", () => {
    const outside: ContextResult = {
      project: null,
      at: [],
      selection: null,
      token: null,
      insertable: false,
    };
    expect(rows(entries(outside, [fix("Remove")]))).toEqual(["-- Fix", "Remove"]);
  });

  it("lists what a blank line takes, inserts first", () => {
    expect(rows(entries(context([section], { insertable: true }), []))).toEqual([
      "-- Write",
      "Insert a note",
      "Insert steps",
      "Insert collapsible details",
      "-- Structure",
      "Insert content that varies",
      "Make the page one variant",
      "Set where the page is available",
      "-- Media",
      "Insert an image",
      "Include a fragment",
      "Insert a code snippet",
      "Insert a widget",
    ]);
  });
});
