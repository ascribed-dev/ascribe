import { describe, expect, it } from "vitest";
import { convertEdit, convertProjectEdit, type EditConverter } from "../../src/edit.js";
import type { LspRange, LspWorkspaceEdit } from "../../src/shapes.js";

const range = (line: number, from: number, to: number): LspRange => ({
  start: { line, character: from },
  end: { line, character: to },
});

/** Records what it converts, and marks the result so the test can tell it was converted. */
function converter(): EditConverter<{ converted: LspWorkspaceEdit }, { converted: LspRange }> & {
  calls: string[];
} {
  const calls: string[] = [];
  return {
    calls,
    asWorkspaceEdit: (edit) => {
      calls.push("edit");
      return Promise.resolve({ converted: edit });
    },
    asRange: (r) => {
      calls.push("range");
      return { converted: r };
    },
  };
}

const edit: LspWorkspaceEdit = {
  changes: { "file:///C:/docs/page.md": [{ range: range(2, 0, 0), newText: "@note\n" }] },
};

describe("converting a server's edit", () => {
  it("converts the edit and the range to select with the language client's converter", async () => {
    const c = converter();
    const result = await convertEdit({ edit, select: range(3, 0, 20) }, c);
    expect(result).toEqual({ edit: { converted: edit }, select: { converted: range(3, 0, 20) } });
    expect(c.calls).toEqual(["edit", "range"]);
  });

  it("has nothing to select when `select` is null", async () => {
    const c = converter();
    expect(await convertEdit({ edit, select: null }, c)).toEqual({
      edit: { converted: edit },
      select: undefined,
    });
    expect(c.calls).toEqual(["edit"]);
  });

  it("passes an error through, converting nothing", async () => {
    const c = converter();
    expect(await convertEdit({ error: "Put the cursor in a note." }, c)).toEqual({
      error: "Put the cursor in a note.",
    });
    expect(c.calls).toEqual([]);
  });

  it("takes a rename's bare edit, and its null as the error given", async () => {
    const c = converter();
    expect(await convertEdit(edit, c, "Can't rename it.")).toEqual({
      edit: { converted: edit },
      select: undefined,
    });
    expect(await convertEdit(null, c, "Can't rename it.")).toEqual({ error: "Can't rename it." });
    await expect(convertEdit(edit, c)).rejects.toThrow(/invalid edit/);
  });

  it("converts a new page's edit: the file to create, then its text", async () => {
    const page = "file:///C:/docs/how-to/rotate-keys.md";
    const created: LspWorkspaceEdit = {
      documentChanges: [
        { kind: "create", uri: page, options: { overwrite: false, ignoreIfExists: false } },
        {
          textDocument: { uri: page, version: null },
          edits: [{ range: range(0, 0, 0), newText: "---\ntitle: x\n---\n" }],
        },
      ],
    };
    const c = converter();
    expect(await convertEdit({ edit: created, select: range(2, 14, 30) }, c)).toEqual({
      edit: { converted: created },
      select: { converted: range(2, 14, 30) },
    });
  });

  it.each([
    ["nothing", null],
    ["a list", []],
    ["an edit without changes", { edit: {}, select: null }],
    ["a text edit without a range", { edit: { changes: { "file:///a.md": [{ newText: "x" }] } } }],
    [
      "a negative position",
      { edit: { changes: { "file:///a.md": [{ range: range(-1, 0, 0), newText: "" }] } } },
    ],
    ["a select that isn't a range", { edit, select: { line: 1 } }],
    [
      "a document change that's neither a file to create nor edits",
      { edit: { documentChanges: [{ kind: "delete", uri: "file:///a.md" }] }, select: null },
    ],
    [
      "a document change whose edits aren't text edits",
      {
        edit: { documentChanges: [{ textDocument: { uri: "file:///a.md" }, edits: [{}] }] },
        select: null,
      },
    ],
  ])("rejects %s", async (_name, value) => {
    await expect(convertEdit(value, converter())).rejects.toThrow(/invalid edit/);
  });
});

describe("converting a server's edit to the whole project", () => {
  const versioned: LspWorkspaceEdit = {
    documentChanges: [
      {
        textDocument: { uri: "file:///docs/open.md", version: 4 },
        edits: [{ range: range(2, 0, 0), newText: "x" }],
      },
      {
        textDocument: { uri: "file:///docs/closed.md", version: null },
        edits: [{ range: range(1, 0, 0), newText: "y" }],
      },
    ],
  };

  it("converts the edit, with its counts and the version each document's edit is for", async () => {
    const c = converter();
    expect(await convertProjectEdit({ edit: versioned, files: 2, fixes: 5 }, c)).toEqual({
      edit: { converted: versioned },
      files: 2,
      fixes: 5,
      versions: new Map([
        ["file:///docs/open.md", 4],
        ["file:///docs/closed.md", null],
      ]),
    });
    expect(c.calls).toEqual(["edit"]);
  });

  it("passes formatting's count of fixes, null, and an error through", async () => {
    const c = converter();
    const formatted = await convertProjectEdit({ edit: versioned, files: 2, fixes: null }, c);
    expect("fixes" in formatted && formatted.fixes).toBeNull();
    expect(await convertProjectEdit({ error: "Still checking." }, c)).toEqual({
      error: "Still checking.",
    });
  });

  it.each([
    ["no counts", { edit: versioned }],
    ["a count that isn't one", { edit: versioned, files: -1, fixes: 0 }],
    ["no edit", { files: 0, fixes: 0 }],
  ])("rejects %s", async (_name, value) => {
    await expect(convertProjectEdit(value, converter())).rejects.toThrow(/invalid edit/);
  });
});
