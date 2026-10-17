// Content model's Insert at the Cursor: the action a phrase or a widget
// runs, with the entry chosen, and the last Markdown editor of each project
// it writes into. Running it in VS Code is the integration suite's.

import { describe, expect, it } from "vitest";
import { ACTIONS } from "../../src/actions/registry.js";
import { PresetPrompter, runWizard, ScriptedPrompter } from "../../src/actions/steps.js";
import type {
  ContextNode,
  ContextResult,
  InventoryEntry,
  TargetsResult,
} from "../../src/shapes.js";
import { insertAction, insertPlan, LastEditors, type LastEditor } from "../../src/ui/insertAt.js";

const r = (line: number) => ({
  start: { line, character: 0 },
  end: { line, character: 4 },
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

const paragraph: ContextNode = { kind: "paragraph", range: r(2) };
const link: ContextNode = { kind: "link", range: r(2), destination: "keys.md" } as ContextNode;

const phrase: InventoryEntry = {
  kind: "phrase",
  key: "product",
  label: "Quill",
  uses: 3,
  declaration: r(4),
  page: null,
};
const widget: InventoryEntry = {
  kind: "widget",
  key: "quill-lab",
  label: null,
  uses: 0,
  declaration: r(9),
  page: null,
};
const editor: LastEditor = { uri: "file:///quill/docs/keys.md", selection: r(2) };

describe("Insert at the Cursor", () => {
  it("shows the action's hint and changes nothing when the project has no Markdown editor", () => {
    expect(insertPlan(phrase, undefined, undefined)).toEqual({
      kind: "hint",
      hint: "Put the cursor in a paragraph's text, with nothing selected, to insert a phrase.",
    });
  });

  it("shows the hint when the cursor is where a phrase can't go", () => {
    // In a link's text, and on a blank line between blocks.
    expect(insertPlan(phrase, editor, context([paragraph, link]))).toMatchObject({ kind: "hint" });
    expect(insertPlan(phrase, editor, context([], { insertable: true }))).toMatchObject({
      kind: "hint",
    });
    // With text selected.
    expect(
      insertPlan(phrase, editor, context([paragraph], { selection: { kind: "prose" } } as never)),
    ).toMatchObject({ kind: "hint" });
  });

  it("runs Insert a phrase with the phrase chosen where it can go", () => {
    const plan = insertPlan(phrase, editor, context([paragraph]));
    expect(plan).toMatchObject({ kind: "run", preset: { key: "product" } });
    expect(plan?.kind === "run" && plan.action.id).toBe("insertPhrase");
  });

  it("runs Insert a widget with the widget chosen on a blank line", () => {
    const plan = insertPlan(widget, editor, context([], { insertable: true }));
    expect(plan).toMatchObject({ kind: "run", preset: { name: "quill-lab" } });
    expect(plan?.kind === "run" && plan.action.id).toBe("insertWidget");
    expect(insertPlan(widget, editor, context([paragraph]))).toMatchObject({ kind: "hint" });
  });

  it("is only for phrases and widgets", () => {
    expect(insertAction({ ...phrase, kind: "term" })).toBeUndefined();
    expect(insertPlan({ ...phrase, kind: "feature" }, editor, context([paragraph]))).toBe(
      undefined,
    );
  });

  it("asks what Insert a phrase asks, with the phrase answered: the same arguments as picking it", async () => {
    const action = ACTIONS.find((a) => a.id === "insertPhrase");
    const targets: TargetsResult = {
      phrases: [
        { key: "api", value: "https://api.quill.dev/", range: r(3) },
        { key: "product", value: "Quill", range: r(4) },
      ],
    } as TargetsResult;
    const wizard = () => {
      const asked = action?.ask?.(context([paragraph]), targets);
      if (!asked) throw new Error("Insert a phrase asks nothing");
      return asked;
    };
    const picked = await runWizard("Insert", wizard(), new ScriptedPrompter(["product"]));
    const preset = await runWizard(
      "Insert",
      wizard(),
      new PresetPrompter({ key: "product" }, new ScriptedPrompter([])),
    );
    expect(preset).toEqual(picked);
    expect(preset).toEqual({ args: { key: "product" } });
  });

  it("asks a preset that isn't a choice, and the steps after it", async () => {
    const action = ACTIONS.find((a) => a.id === "insertPhrase");
    const targets = { phrases: [{ key: "api", value: "x", range: r(3) }] } as TargetsResult;
    const wizard = action?.ask?.(context([paragraph]), targets);
    if (!wizard) throw new Error("Insert a phrase asks nothing");
    const asked = await runWizard(
      "Insert",
      wizard,
      new PresetPrompter({ key: "gone" }, new ScriptedPrompter(["api"])),
    );
    expect(asked).toEqual({ args: { key: "api" } });
  });
});

describe("the last Markdown editor of each project", () => {
  it("is the one last noted in the project, with its cursor", () => {
    const editors = new LastEditors();
    expect(editors.get("/quill")).toBeUndefined();
    editors.note("/quill", editor);
    const later = { uri: "file:///quill/docs/quickstart.md", selection: r(7) };
    editors.note("/quill", later);
    editors.note("/other", editor);
    expect(editors.get("/quill")).toEqual(later);
    expect(editors.get("/other")).toEqual(editor);
  });

  it("is forgotten once its document closes", () => {
    const editors = new LastEditors();
    editors.note("/quill", editor);
    editors.note("/other", editor);
    editors.forget(editor.uri);
    expect(editors.get("/quill")).toBeUndefined();
    expect(editors.get("/other")).toBeUndefined();
  });
});
