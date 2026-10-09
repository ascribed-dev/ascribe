import { readFileSync } from "node:fs";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { ContextCache, type Where } from "../../src/actions/context.js";
import {
  ACTIONS,
  CONTEXT_KEYS,
  commandId,
  contextKeys,
  copyLinkToSection,
  lightbulbActions,
  plain,
  type Action,
} from "../../src/actions/registry.js";
import {
  availabilitySpec,
  availabilitySteps,
  destinationStep,
  includePath,
  includeSteps,
  noteTypeStep,
  runWizard,
  ScriptedPrompter,
  snippetAddress,
  snippetSteps,
  validId,
  valueStep,
  widgetArgs,
  widgetSteps,
  wholeNumber,
  type Scripted,
  type Step,
} from "../../src/actions/steps.js";
import type { ContextNode, ContextResult, LspRange, TargetsResult } from "../../src/shapes.js";

interface Manifest {
  contributes: {
    commands: { command: string; title: string; category: string }[];
    submenus: { id: string; label: string }[];
    menus: Record<string, { command?: string; submenu?: string; when: string; group?: string }[]>;
  };
}

const manifest = JSON.parse(
  readFileSync(new URL("../../package.json", import.meta.url), "utf8"),
) as Manifest;

const byText = (a: string | undefined, b: string | undefined): number =>
  (a ?? "").localeCompare(b ?? "");

const action = (id: string): Action => {
  const found = ACTIONS.find((a) => a.id === id);
  if (!found) throw new Error(`no action ${id}`);
  return found;
};

// Sample contexts, as `ascribe/context` answers them.

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
const heading = (id: string, explicitId: boolean): ContextNode => ({
  kind: "heading",
  range: r(1),
  level: 2,
  id,
  explicitId,
});
const note: ContextNode = { kind: "note", range: r(3), type: "tip", form: "block" };
const list = (ordered: boolean, steps: boolean): ContextNode => ({
  kind: "list",
  range: r(4),
  ordered,
  steps,
});
const item: ContextNode = { kind: "listItem", range: r(4) };
const group = (dimension: string | null, arms: number, arm: number | null): ContextNode => ({
  kind: "variantGroup",
  range: r(5),
  dimension,
  arms: Array.from({ length: arms }, (_, i) => ({
    value: dimension ? (["cloud", "self-managed", "on-prem"][i] ?? null) : null,
    label: dimension ? null : `Arm ${i}`,
    range: r(5 + i),
  })),
  arm,
});
const row = (header: boolean): ContextNode => ({
  kind: "tableRow",
  range: r(6),
  header,
  available: null,
});
const table: ContextNode = { kind: "table", range: r(6) };
const link = (textEmpty: boolean): ContextNode => ({
  kind: "link",
  range: r(2),
  destination: "keys.md",
  textEmpty,
});
const image: ContextNode = {
  kind: "image",
  range: r(7),
  src: "playground.png",
  alt: "The playground",
  attributes: [{ key: "width", value: "600" }],
};

const SAMPLES: Record<string, ContextResult> = {
  "outside a project": { project: null, at: [], selection: null, token: null, insertable: false },
  "a blank line": context([section], { insertable: true }),
  "a blank line, selected": context([section], {
    insertable: true,
    selection: { kind: "other", text: "\n", inline: false },
  }),
  "the frontmatter": context([
    { kind: "frontmatter", range: r(0), variant: null, available: null },
  ]),
  "a paragraph": context([paragraph, section]),
  "prose selected": context([paragraph, section], {
    selection: { kind: "prose", text: "the agent", inline: true },
  }),
  "blocks selected": context([paragraph, section], {
    selection: { kind: "blocks", text: "One.\n\nTwo.", inline: false },
  }),
  "a mixed selection": context([paragraph, section], {
    selection: { kind: "mixed", text: "One.\n\n@end", inline: false },
  }),
  "a code block": context([{ kind: "codeBlock", range: r(2), info: "sh", fenced: true }, section]),
  "a heading": context([heading("install", false), section]),
  "a heading with @id": context([heading("install", true), section]),
  "a heading with no id": context([heading("", false), section]),
  "a note's paragraph": context([paragraph, note, section]),
  "a details block": context([
    paragraph,
    { kind: "details", range: r(3), title: "More", form: "container" },
    section,
  ]),
  "a numbered list": context([paragraph, item, list(true, false), section]),
  "a bulleted list": context([paragraph, item, list(false, false), section]),
  steps: context([paragraph, item, list(true, true), { kind: "steps", range: r(4) }, section]),
  "an arm of a dimension's group": context([paragraph, group("deployment", 2, 0), section]),
  "the only arm of a group": context([paragraph, group("deployment", 1, 0), section]),
  "a group's @end": context([group("deployment", 2, null), section]),
  "an arm of a labeled group": context([paragraph, group(null, 2, 1), section]),
  "a table's header row": context([row(true), table, section]),
  "a table's body row": context([row(false), table, section]),
  "a link with text": context([link(false), paragraph, section]),
  "a link without text": context([link(true), paragraph, section]),
  "an image": context([image, paragraph, section]),
  "a phrase": context([
    { kind: "phrase", range: r(2), key: "product", declared: true },
    paragraph,
    section,
  ]),
};

/** Which actions apply where; every other action doesn't. */
const EXPECTED: Record<string, string[]> = {
  "outside a project": [],
  "a blank line": [
    "insertNote",
    "insertSteps",
    "insertDetails",
    "insertVariantGroup",
    "setPageVariant",
    "setPageAvailable",
    "insertImage",
    "insertInclude",
    "insertSnippet",
    "insertWidget",
  ],
  "a blank line, selected": ["setPageVariant", "setPageAvailable"],
  "the frontmatter": ["setPageVariant", "setPageAvailable"],
  "a paragraph": [
    "wrapNote",
    "wrapDetails",
    "markAvailable",
    "setPageVariant",
    "setPageAvailable",
    "insertLink",
    "insertPhrase",
  ],
  "prose selected": [
    "wrapNote",
    "wrapDetails",
    "markAvailable",
    "setPageVariant",
    "setPageAvailable",
    "linkSelection",
  ],
  "blocks selected": [
    "wrapNote",
    "wrapDetails",
    "markAvailable",
    "setPageVariant",
    "setPageAvailable",
  ],
  "a mixed selection": ["markAvailable", "setPageVariant", "setPageAvailable"],
};

describe("the registry", () => {
  it("has one action per id, each with a title, description, where, and hint", () => {
    expect(new Set(ACTIONS.map((a) => a.id)).size).toBe(ACTIONS.length);
    for (const a of ACTIONS) {
      for (const field of [a.title, a.description, a.where, a.hint])
        expect(field.trim()).not.toBe("");
      expect(a.ask !== undefined || a.needs === undefined || "run" in a.does).toBe(true);
    }
  });

  it("has an action for every ascribe/edit operation", () => {
    const readme = readFileSync(
      new URL("../../../../crates/ascribe-lsp/README.md", import.meta.url),
      "utf8",
    );
    const table = readme.slice(readme.indexOf("| Action | Applies to |"));
    const operations = [...table.matchAll(/^\| `(\w+)` \|/gm)].map((m) => m[1]);
    expect(operations.length).toBeGreaterThan(20);
    const done = ACTIONS.flatMap((a) => ("operation" in a.does ? [a.does.operation] : []));
    expect(done.sort(byText)).toEqual(operations.sort(byText));
  });

  it("writes descriptions a quick pick shows plainly", () => {
    expect(plain(action("wrapNote").description)).toBe(
      "Put the paragraph or the selected blocks in an @note callout",
    );
  });
});

describe("package.json", () => {
  const { commands, menus, submenus } = manifest.contributes;

  it("has a command for every action, titled as the registry titles it", () => {
    for (const a of ACTIONS) {
      expect(commands.find((c) => c.command === commandId(a))).toEqual({
        command: commandId(a),
        title: a.title,
        category: "Ascribe",
      });
    }
    const stray = commands
      .map((c) => c.command)
      .filter(
        (id) => id.startsWith("ascribe.action.") && !ACTIONS.some((a) => commandId(a) === id),
      );
    expect(stray).toEqual([]);
  });

  it("shows every action in the palette in a Markdown file of a project", () => {
    for (const a of ACTIONS) {
      expect(menus.commandPalette?.find((e) => e.command === commandId(a))?.when).toBe(
        "ascribe.inProject && editorLangId == markdown",
      );
    }
  });

  it("lists every action in the context menu's Ascribe submenu, where it applies", () => {
    expect(submenus).toContainEqual({ id: "ascribe.actions", label: "Ascribe" });
    expect(menus["editor/context"]?.find((e) => e.submenu === "ascribe.actions")?.when).toBe(
      "ascribe.inProject && editorLangId == markdown",
    );
    for (const a of ACTIONS) {
      const entry = menus["ascribe.actions"]?.find((e) => e.command === commandId(a));
      expect(entry?.when, a.id).toBe(a.when);
      expect(entry?.group, a.id).toMatch(new RegExp(`^\\d_${a.group}@\\d+$`));
    }
    // And the actions bar, first.
    expect(menus["ascribe.actions"]?.[0]).toEqual({ command: "ascribe.actions", group: "0_bar@1" });
    expect(menus["ascribe.actions"]).toHaveLength(ACTIONS.length + 1);
  });
});

/**
 * Evaluates a `when` clause as VS Code does, for the forms the registry
 * uses: keys, `!`, `&&` (binding tighter), `||`, and parentheses.
 */
function evaluate(clause: string, keys: Record<string, boolean>): boolean {
  const tokens = clause.match(/\(|\)|&&|\|\||!|[\w.]+/g) ?? [];
  let i = 0;
  const or = (): boolean => {
    let value = and();
    while (tokens[i] === "||") {
      i++;
      value = and() || value;
    }
    return value;
  };
  const and = (): boolean => {
    let value = unary();
    while (tokens[i] === "&&") {
      i++;
      value = unary() && value;
    }
    return value;
  };
  const unary = (): boolean => {
    const token = tokens[i++];
    if (token === "!") return !unary();
    if (token === "(") {
      const value = or();
      i++;
      return value;
    }
    if (token === undefined || !(token in keys))
      throw new Error(`unknown key ${token} in ${clause}`);
    return keys[token] ?? false;
  };
  const value = or();
  if (i !== tokens.length) throw new Error(`can't read ${clause}`);
  return value;
}

describe("where actions apply", () => {
  it.each(Object.entries(SAMPLES))("agrees with its menu's when clause in %s", (_name, sample) => {
    const keys = contextKeys(sample);
    for (const a of ACTIONS) expect(evaluate(a.when, keys), a.id).toBe(a.applies(sample));
  });

  it.each(Object.entries(EXPECTED))("in %s", (name, expected) => {
    const sample = SAMPLES[name];
    if (!sample) throw new Error(name);
    expect(
      ACTIONS.filter((a) => a.applies(sample))
        .map((a) => a.id)
        .sort(),
    ).toEqual([...expected].sort());
  });

  const appliesIn = (id: string): string[] =>
    Object.entries(SAMPLES)
      .filter(([, sample]) => action(id).applies(sample))
      .map(([name]) => name);

  it.each([
    ["setNoteType", ["a note's paragraph"]],
    ["unwrapNote", ["a note's paragraph"]],
    ["noteToDetails", ["a note's paragraph"]],
    ["unwrapDetails", ["a details block"]],
    ["makeSteps", ["a numbered list"]],
    ["removeSteps", ["steps"]],
    ["addHeadingId", ["a heading", "a heading with no id"]],
    ["copyLinkToSection", ["a heading", "a heading with @id"]],
    [
      "addVariantArm",
      ["an arm of a dimension's group", "the only arm of a group", "a group's @end"],
    ],
    ["removeVariantArm", ["an arm of a dimension's group", "an arm of a labeled group"]],
    ["setLinkTarget", ["a link with text", "a link without text"]],
    ["useTargetTitle", ["a link with text"]],
    ["setImageWidth", ["an image"]],
    ["setImageAlt", ["an image"]],
  ])("%s applies only in its own node", (id, expected) => {
    expect(appliesIn(id)).toEqual(expected);
  });

  it("marks a body row, a heading, or a block, but not a header row", () => {
    expect(appliesIn("markAvailable")).toContain("a table's body row");
    expect(appliesIn("markAvailable")).toContain("a heading");
    expect(appliesIn("markAvailable")).toContain("a code block");
    expect(appliesIn("markAvailable")).not.toContain("a table's header row");
  });

  it("wraps in details any block a note can't wrap", () => {
    expect(appliesIn("wrapDetails")).toContain("a code block");
    expect(appliesIn("wrapNote")).not.toContain("a code block");
  });

  it("inserts nothing inside a link or a phrase", () => {
    for (const id of ["insertLink", "insertPhrase"]) {
      expect(appliesIn(id)).not.toContain("a link with text");
      expect(appliesIn(id)).not.toContain("a phrase");
      expect(appliesIn(id)).toContain("a heading");
    }
  });

  it("sets every context key false outside a page", () => {
    const keys = contextKeys(SAMPLES["outside a project"] as ContextResult);
    expect(Object.keys(keys)).toEqual(Object.keys(CONTEXT_KEYS));
    expect(Object.values(keys).every((value) => !value)).toBe(true);
  });
});

// The wizards.

const targets: TargetsResult = {
  notes: [
    { type: "note", label: "Note" },
    { type: "tip", label: "Tip" },
    { type: "warning", label: "Warning" },
  ],
  pages: [
    { path: "keys.md", title: "API keys", type: "page", link: "keys.md" },
    { path: "guides/my page.md", title: null, type: null, link: "guides/my%20page.md" },
  ],
  headings: [
    {
      page: "keys.md",
      text: "Rotate keys",
      id: "rotate-keys",
      level: 2,
      link: "keys.md#rotate-keys",
    },
    { page: "install.md", text: "Install", id: "install", level: 2, link: "#install" },
    {
      page: "guides/my page.md",
      text: "Set up (once)",
      id: "set-up",
      level: 2,
      link: "guides/my%20page.md#set-up",
    },
  ],
  fragments: [
    {
      path: "_fragments/prerequisites.md",
      include: "_fragments/prerequisites.md",
      startsWithHeading: false,
    },
  ],
  phrases: [{ key: "product", value: "Quill", range: null }],
  dimensions: [
    {
      name: "deployment",
      label: "Deployment",
      values: [
        { value: "cloud", label: "Quill Cloud", versionless: true, range: null },
        { value: "self-managed", label: "self-managed", versionless: false, range: null },
      ],
    },
  ],
  features: [{ key: "streaming", name: "Streaming sync", availability: "cloud", range: null }],
  snippets: [
    {
      name: "app",
      files: [
        {
          path: "main.rs",
          address: "app:main.rs",
          regions: [{ name: "setup", address: "app:main.rs#setup" }],
        },
        { path: "lib.rs", address: "app:lib.rs", regions: [] },
      ],
    },
  ],
  widgets: [
    {
      name: "card",
      description: "A card",
      line: true,
      container: false,
      groupable: false,
      primary: "text",
      binding: null,
      attributes: [
        {
          key: "tone",
          type: "enum",
          values: ["calm", "loud"],
          required: true,
          default: null,
          description: null,
        },
        {
          key: "tags",
          type: "set",
          values: ["a", "b"],
          required: true,
          default: null,
          description: null,
        },
        {
          key: "size",
          type: "number",
          values: [],
          required: true,
          default: "2",
          description: "How big",
        },
        {
          key: "hint",
          type: "string",
          values: [],
          required: false,
          default: null,
          description: null,
        },
      ],
    },
  ],
};

const values = (step: Step | undefined): string[] =>
  step && step.kind !== "text" ? step.choices.map((choice) => choice.value) : [];

/** Runs an action's wizard against a script. */
async function ask(id: string, at: ContextResult, script: Scripted[]) {
  const a = action(id);
  if (!a.ask) throw new Error(`${id} asks nothing`);
  return runWizard(a.title, a.ask(at, targets), new ScriptedPrompter(script));
}

describe("the wizard's steps", () => {
  it("offers the note types, without the note's own", () => {
    expect(values(noteTypeStep(targets.notes))).toEqual(["note", "tip", "warning"]);
    expect(values(noteTypeStep(targets.notes, "tip"))).toEqual(["note", "warning"]);
    const step = noteTypeStep(targets.notes);
    expect(step.kind !== "text" && step.choices[1]?.description).toBe("@note {type=tip}");
  });

  it("offers pages, then headings, by their links from the page", () => {
    const step = destinationStep(targets);
    expect(values(step)).toEqual([
      "keys.md",
      "guides/my%20page.md",
      "keys.md#rotate-keys",
      "#install",
      "guides/my%20page.md#set-up",
    ]);
    expect(step.kind !== "text" && step.choices.map((c) => c.section)).toEqual([
      "Pages",
      "Pages",
      "Headings",
      "Headings",
      "Headings",
    ]);
    // A page without a title shows its path.
    expect(step.kind !== "text" && step.choices[1]?.label).toBe("$(file) guides/my page.md");
  });

  it("offers a dimension's values, without those used", () => {
    const deployment = targets.dimensions?.[0];
    expect(values(valueStep(deployment, "pickMany"))).toEqual(["cloud", "self-managed"]);
    expect(values(valueStep(deployment, "pick", ["cloud"]))).toEqual(["self-managed"]);
  });

  it("asks for a spec when there are no features, or when asked to", () => {
    expect(
      availabilitySteps({ dimensions: targets.dimensions ?? [] }, {}).map((s) => s.kind),
    ).toEqual(["text"]);
    expect(availabilitySteps(targets, {}).map((s) => s.key)).toEqual(["feature"]);
    const enter = values(availabilitySteps(targets, {})[0]).at(-1) ?? "";
    expect(availabilitySteps(targets, { feature: enter }).map((s) => s.key)).toEqual([
      "feature",
      "spec",
    ]);
    expect(availabilitySpec({ feature: "streaming" })).toBe("streaming");
    expect(availabilitySpec({ feature: enter, spec: " cloud " })).toBe("cloud");
  });

  it("includes a fragment, or a page or one of its sections", () => {
    expect(values(includeSteps(targets, {})[0])).toEqual([
      "_fragments/prerequisites.md",
      "keys.md",
      "guides/my%20page.md",
    ]);
    expect(includeSteps(targets, { file: "_fragments/prerequisites.md" })).toHaveLength(1);
    expect(values(includeSteps(targets, { file: "keys.md" })[1])).toEqual(["", "rotate-keys"]);
    expect(includePath({ file: "keys.md", section: "rotate-keys" })).toBe("keys.md#rotate-keys");
    expect(includePath({ file: "keys.md", section: "" })).toBe("keys.md");
  });

  it("takes a snippet from a source's file, or one of its regions", () => {
    expect(snippetSteps(targets, { source: "app", file: "main.rs" }).map((s) => s.key)).toEqual([
      "source",
      "file",
      "address",
    ]);
    expect(snippetSteps(targets, { source: "app", file: "lib.rs" })).toHaveLength(2);
    expect(snippetAddress(targets, { source: "app", file: "lib.rs" })).toBe("app:lib.rs");
    expect(
      snippetAddress(targets, { source: "app", file: "main.rs", address: "app:main.rs#setup" }),
    ).toBe("app:main.rs#setup");
  });

  it("asks a widget's primary and its required attributes, each by its type", () => {
    const steps = widgetSteps(targets, { name: "card" });
    expect(steps.map((s) => `${s.key}:${s.kind}`)).toEqual([
      "name:pick",
      "primary:text",
      "attribute.tone:pick",
      "attribute.tags:pickMany",
      "attribute.size:text",
    ]);
    const size = steps[4];
    expect(size?.kind === "text" && size.value).toBe("2");
    expect(size?.kind === "text" && size.validate("big")).toBe("Enter a number.");
    expect(
      widgetArgs(targets, {
        name: "card",
        primary: " Hello ",
        "attribute.tone": "calm",
        "attribute.tags": ["a", "b"],
        "attribute.size": "3",
      }),
    ).toEqual({
      name: "card",
      primary: "Hello",
      attributes: { tone: "calm", tags: "a|b", size: "3" },
    });
  });

  it("checks what's typed", () => {
    expect(validId("install-cli")).toBeUndefined();
    expect(validId("install cli")).toMatch(/one word/);
    expect(wholeNumber(1, 50)("51")).toMatch(/1 to 50/);
    expect(wholeNumber(1, 50)("3")).toBeUndefined();
  });
});

describe("running a wizard", () => {
  const paragraphHere = SAMPLES["a paragraph"] as ContextResult;
  const blank = SAMPLES["a blank line"] as ContextResult;

  it("collects the arguments", async () => {
    expect(await ask("wrapNote", paragraphHere, ["tip"])).toEqual({ args: { type: "tip" } });
    expect(
      await ask("insertVariantGroup", blank, ["deployment", ["cloud", "self-managed"]]),
    ).toEqual({
      args: { dimension: "deployment", values: ["cloud", "self-managed"] },
    });
    expect(await ask("insertSteps", blank, ["4"])).toEqual({ args: { count: 4 } });
  });

  it("goes back a step, and keeps going", async () => {
    expect(
      await ask("insertVariantGroup", blank, ["deployment", "back", "deployment", ["cloud"]]),
    ).toEqual({ args: { dimension: "deployment", values: ["cloud"] } });
  });

  it("stops when the writer gives up", async () => {
    expect(await ask("setPageVariant", blank, ["deployment", null])).toBeUndefined();
  });

  it("says why when there's nothing to choose", async () => {
    const a = action("insertPhrase");
    expect(
      await runWizard(
        a.title,
        a.ask?.(paragraphHere, {}) ?? { steps: () => [], args: () => ({}) },
        new ScriptedPrompter([]),
      ),
    ).toEqual({ error: "The content model declares no phrases." });
  });

  it("adds a variant for a value the group doesn't have", async () => {
    const at = SAMPLES["the only arm of a group"] as ContextResult;
    await expect(ask("addVariantArm", at, ["cloud"])).rejects.toThrow(/isn't a choice/);
    expect(await ask("addVariantArm", at, ["self-managed"])).toEqual({
      args: { value: "self-managed" },
    });
  });

  it("starts the heading's id from its slug", async () => {
    const a = action("addHeadingId");
    const step = a.ask?.(SAMPLES["a heading"] as ContextResult, targets).steps({})[0];
    expect(step?.kind === "text" && step.value).toBe("install");
  });
});

describe("Copy a link to this section", () => {
  async function copy(at: ContextResult) {
    const copied: string[] = [];
    const said: string[] = [];
    await copyLinkToSection(at, targets, undefined, {
      copy: (text) => {
        copied.push(text);
        return Promise.resolve();
      },
      say: (message) => said.push(message),
    });
    return { copied, said };
  }

  it("copies the heading's destination from the content root", async () => {
    expect((await copy(SAMPLES["a heading"] as ContextResult)).copied).toEqual([
      "/install.md#install",
    ]);
  });

  it("encodes the page's path as a destination", async () => {
    const at = context([heading("set-up", true), section]);
    const own: TargetsResult = {
      headings: [
        {
          page: "guides/my page (old).md",
          text: "Set up",
          id: "set-up",
          level: 2,
          link: "#set-up",
        },
      ],
    };
    const copied: string[] = [];
    await copyLinkToSection(at, own, undefined, {
      copy: (text) => {
        copied.push(text);
        return Promise.resolve();
      },
      say: () => undefined,
    });
    expect(copied).toEqual(["/guides/my%20page%20%28old%29.md#set-up"]);
  });

  it("says so for a heading no page of its own lists", async () => {
    const result = await copy(context([heading("elsewhere", true), section]));
    expect(result.copied).toEqual([]);
    expect(result.said[0]).toMatch(/isn't on a page/);
  });
});

describe("the cached context", () => {
  const where = (line: number, version = 1): Where => ({
    uri: "file:///C:/docs/page.md",
    version,
    range: { start: { line, character: 0 }, end: { line, character: 0 } },
  });
  const answer = SAMPLES["a paragraph"] as ContextResult;
  let sent: Where[];
  let answers: (ContextResult | undefined)[];
  let cache: ContextCache;

  beforeEach(() => {
    vi.useFakeTimers();
    sent = [];
    answers = [];
    cache = new ContextCache(
      (w) => {
        sent.push(w);
        return Promise.resolve(answer);
      },
      (_w, result) => answers.push(result),
    );
  });

  afterEach(() => {
    cache.dispose();
    vi.useRealTimers();
  });

  it("asks once the selection settles, whatever the lightbulb asks meanwhile", async () => {
    for (let line = 0; line < 5; line++) {
      cache.schedule(where(line));
      // VS Code asks the lightbulb on every move.
      expect(lightbulbActions(cache.get(where(line)))).toEqual([]);
      await vi.advanceTimersByTimeAsync(50);
    }
    await vi.advanceTimersByTimeAsync(500);
    expect(sent).toEqual([where(4)]);
    expect(answers).toEqual([answer]);
  });

  it("answers the lightbulb from a matching answer, and with nothing for another version", async () => {
    const at = context([paragraph, note, section]);
    cache = new ContextCache(
      (w) => {
        sent.push(w);
        return Promise.resolve(at);
      },
      () => undefined,
    );
    cache.schedule(where(3));
    await vi.advanceTimersByTimeAsync(500);
    expect(lightbulbActions(cache.get(where(3))).map((a) => a.id)).toEqual([
      "wrapNote",
      "unwrapNote",
      "noteToDetails",
      "wrapDetails",
    ]);
    expect(lightbulbActions(cache.get(where(3, 2)))).toEqual([]);
    expect(lightbulbActions(cache.get(where(4)))).toEqual([]);
    expect(sent).toHaveLength(1);
  });

  it("doesn't ask again for what it has, or what's on its way", async () => {
    await cache.request(where(1));
    cache.schedule(where(1));
    await vi.advanceTimersByTimeAsync(500);
    const first = cache.request(where(2));
    const second = cache.request(where(2));
    await Promise.all([first, second]);
    expect(sent).toEqual([where(1), where(2)]);
  });

  it("cancels a request the selection moved on from", async () => {
    const signals: AbortSignal[] = [];
    cache = new ContextCache(
      (w, signal) => {
        sent.push(w);
        signals.push(signal);
        return new Promise((resolve) => setTimeout(() => resolve(answer), 1_000));
      },
      (_w, result) => answers.push(result),
    );
    cache.schedule(where(1));
    await vi.advanceTimersByTimeAsync(200);
    cache.schedule(where(2));
    expect(signals[0]?.aborted).toBe(true);
    await vi.advanceTimersByTimeAsync(2_000);
    expect(sent).toEqual([where(1), where(2)]);
    // Only the answer for where the selection is now is kept.
    expect(answers).toEqual([answer]);
    expect(cache.get(where(1))).toBeUndefined();
    expect(cache.get(where(2))).toBe(answer);
  });

  it("forgets its answer for another editor", async () => {
    await cache.request(where(1));
    cache.clear();
    expect(cache.get(where(1))).toBeUndefined();
    expect(answers).toEqual([answer, undefined]);
  });
});
