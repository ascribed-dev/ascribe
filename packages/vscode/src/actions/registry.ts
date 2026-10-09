// The editor's actions, each defined once. The registry feeds the Command
// Palette (a command per action), the editor's context menu (the Ascribe
// submenu), the lightbulb, and the actions bar. An action says where it
// applies, from the server's `ascribe/context` answer; what it asks the
// writer, from `ascribe/targets`; and what it does: an `ascribe/edit`
// operation, or a function of the extension's own. No VS Code in this
// module, so it's unit-tested as is.
//
// package.json repeats each action's command, title, and context-menu `when`
// clause, since VS Code reads them from there; test/unit/actions.test.ts
// fails when the two differ.

import type { ContextNode, ContextResult, TargetsResult } from "../shapes.js";
import {
  availabilitySpec,
  availabilitySteps,
  chosenDimension,
  destinationStep,
  dimensionStep,
  imageStep,
  includePath,
  includeSteps,
  list,
  noteTypeStep,
  oneLine,
  phraseStep,
  positiveNumber,
  snippetAddress,
  snippetSteps,
  text,
  validId,
  valueStep,
  widgetArgs,
  widgetSteps,
  wholeNumber,
  type Args,
  type Wizard,
} from "./steps.js";

/** What an `ascribe/targets` request can ask for. */
type TargetKind = Exclude<keyof TargetsResult, "modelUri">;

/** What a `run` action may do besides reading: the runner's side of it. */
export interface Effects {
  /** Puts text on the clipboard. */
  copy(text: string): Promise<void>;
  /** Tells the writer something, briefly. */
  say(message: string): void;
}

export interface Action {
  /** `wrapNote`; the command is `ascribe.action.wrapNote`. */
  id: string;
  /** For writers: "Wrap in a note". */
  title: string;
  /** One line that names the syntax, with code between backticks. */
  description: string;
  /** Where it applies, for the docs: "A paragraph, or whole blocks selected". */
  where: string;
  /** What to say when it's run where it doesn't apply. */
  hint: string;
  group: "fix" | "write" | "structure" | "link" | "media";
  /** Whether it applies to the context, from `ascribe/context`. */
  applies(context: ContextResult): boolean;
  /**
   * The context-menu `when` clause: `applies`, over the context keys
   * `contextKeys` sets. A test holds the two to the same answer.
   */
  when: string;
  /** Offered by the lightbulb (`Cmd+.`) too, as this kind of code action. */
  lightbulb?: "refactor" | "quickfix";
  /** What `ascribe/targets` lists the wizard (or `run`) needs. */
  needs?: TargetKind[];
  /** The wizard: what it asks. */
  ask?(context: ContextResult, targets: TargetsResult): Wizard;
  /** What it does: an `ascribe/edit` operation, or the extension's own function. */
  does:
    | { operation: string }
    | {
        run(
          context: ContextResult,
          targets: TargetsResult,
          args: Args | undefined,
          effects: Effects,
        ): Promise<void>;
      };
}

/** The command that runs an action. */
export const commandId = (action: Action): string => `ascribe.action.${action.id}`;

/** A description without its backticks, for a quick pick, which shows them as written. */
export const plain = (description: string): string => description.replaceAll("`", "");

// What the context says.

type Kind = ContextNode["kind"];
type NodeOf<K extends Kind> = Extract<ContextNode, { kind: K }>;

/** The innermost node of a kind that contains the range. */
function innermost<K extends Kind>(context: ContextResult, kind: K): NodeOf<K> | undefined {
  return context.at.find((node): node is NodeOf<K> => node.kind === kind);
}

const inPage = (c: ContextResult): boolean => c.project !== null;
const has = (c: ContextResult, ...kinds: Kind[]): boolean =>
  inPage(c) && c.at.some((node) => kinds.includes(node.kind));
const cursor = (c: ContextResult): boolean => inPage(c) && c.selection === null;
/** A cursor, or a selection of prose: a wrap takes the block it's in. */
const cursorOrProse = (c: ContextResult): boolean =>
  inPage(c) && (c.selection === null || c.selection.kind === "prose");
const blocksSelected = (c: ContextResult): boolean => inPage(c) && c.selection?.kind === "blocks";

/** The kinds `markAvailable` marks: a heading's section, a block, or a table's row. */
const MARKABLE: Kind[] = [
  "tableRow",
  "heading",
  "paragraph",
  "codeBlock",
  "table",
  "list",
  "blockQuote",
  "note",
  "details",
  "steps",
  "variantGroup",
  "widget",
  "include",
  "snippet",
];

/**
 * The context keys the context menu's `when` clauses read, each from the
 * latest `ascribe/context` answer for the cursor. All are false outside a
 * page of a project.
 */
export const CONTEXT_KEYS: Record<string, (c: ContextResult) => boolean> = {
  "ascribe.inPage": inPage,
  "ascribe.cursor": cursor,
  "ascribe.insertable": (c) => cursor(c) && c.insertable,
  "ascribe.selection.prose": (c) =>
    inPage(c) && c.selection?.kind === "prose" && c.selection.inline,
  "ascribe.selection.blocks": blocksSelected,
  "ascribe.selection.proseOrNone": cursorOrProse,
  "ascribe.at.paragraph": (c) => has(c, "paragraph"),
  "ascribe.at.block": (c) => has(c, "paragraph", "codeBlock", "table", "snippet", "include"),
  "ascribe.at.heading": (c) => has(c, "heading"),
  "ascribe.at.headingWithId": (c) => inPage(c) && (innermost(c, "heading")?.id ?? "") !== "",
  "ascribe.at.headingWithoutAtId": (c) => {
    const heading = innermost(c, "heading");
    return inPage(c) && heading !== undefined && !heading.explicitId;
  },
  "ascribe.at.note": (c) => has(c, "note"),
  "ascribe.at.details": (c) => has(c, "details"),
  "ascribe.at.steps": (c) => has(c, "steps"),
  "ascribe.at.numberedList": (c) => {
    const listNode = innermost(c, "list");
    return inPage(c) && listNode !== undefined && listNode.ordered && !listNode.steps;
  },
  "ascribe.at.dimensionGroup": (c) => {
    const group = innermost(c, "variantGroup");
    return inPage(c) && group !== undefined && group.dimension !== null;
  },
  "ascribe.at.removableArm": (c) => {
    const group = innermost(c, "variantGroup");
    return inPage(c) && group !== undefined && group.arm !== null && group.arms.length > 1;
  },
  "ascribe.at.markable": (c) => {
    const node = c.at.find((n) => MARKABLE.includes(n.kind));
    return inPage(c) && node !== undefined && !(node.kind === "tableRow" && node.header);
  },
  "ascribe.at.text": (c) => has(c, "paragraph", "heading") && !has(c, "link", "image", "phrase"),
  "ascribe.at.link": (c) => has(c, "link"),
  "ascribe.at.linkWithText": (c) => {
    const link = innermost(c, "link");
    return inPage(c) && link !== undefined && !link.textEmpty;
  },
  "ascribe.at.image": (c) => has(c, "image"),
};

/** Each context key's value for a context answer. */
export function contextKeys(context: ContextResult): Record<string, boolean> {
  return Object.fromEntries(Object.entries(CONTEXT_KEYS).map(([key, f]) => [key, f(context)]));
}

const key = (name: string) => (c: ContextResult) => CONTEXT_KEYS[name]?.(c) ?? false;

/** `/guides/install.md#install-cli`: the heading's destination from the content root. */
export async function copyLinkToSection(
  context: ContextResult,
  targets: TargetsResult,
  _args: Args | undefined,
  effects: Effects,
): Promise<void> {
  const id = innermost(context, "heading")?.id ?? "";
  // A heading on the requesting page is listed with a link of `#id`.
  const heading = targets.headings?.find((h) => h.id === id && h.link === `#${id}`);
  if (!heading) {
    effects.say(
      "This heading isn't on a page a link can name: copy the link from a page that includes this file.",
    );
    return;
  }
  const link = heading.rootLink;
  await effects.copy(link);
  effects.say(`Copied ${link}`);
}

// The actions.

const insertable = key("ascribe.insertable");

export const ACTIONS: Action[] = [
  // Notes.
  {
    id: "wrapNote",
    title: "Wrap in a note",
    description: "Put the paragraph or the selected blocks in an `@note` callout",
    where: "A paragraph, or whole blocks selected",
    hint: "Put the cursor in a paragraph, or select whole blocks, to wrap them in a note.",
    group: "structure",
    applies: (c) => blocksSelected(c) || (cursorOrProse(c) && has(c, "paragraph")),
    when: "ascribe.selection.blocks || ascribe.selection.proseOrNone && ascribe.at.paragraph",
    lightbulb: "refactor",
    needs: ["notes"],
    ask: (_c, targets) => ({
      steps: () => [noteTypeStep(targets.notes)],
      args: (answers) => ({ type: text(answers, "type") }),
    }),
    does: { operation: "wrapNote" },
  },
  {
    id: "setNoteType",
    title: "Change the note's kind",
    description: "Set the `type` of the `@note` the cursor is in",
    where: "A note",
    hint: "Put the cursor in a note to change its kind.",
    group: "structure",
    applies: (c) => has(c, "note"),
    when: "ascribe.at.note",
    needs: ["notes"],
    ask: (c, targets) => ({
      steps: () => [noteTypeStep(targets.notes, innermost(c, "note")?.type)],
      args: (answers) => ({ type: text(answers, "type") }),
    }),
    does: { operation: "setNoteType" },
  },
  {
    id: "unwrapNote",
    title: "Remove the note, keeping its text",
    description: "Take the content out of its `@note`",
    where: "A note",
    hint: "Put the cursor in a note to take its text out of it.",
    group: "structure",
    applies: (c) => has(c, "note"),
    when: "ascribe.at.note",
    lightbulb: "refactor",
    does: { operation: "unwrapNote" },
  },
  {
    id: "noteToDetails",
    title: "Turn the note into collapsible details",
    description: "Make the `@note` a `@details` block, with a title",
    where: "A note",
    hint: "Put the cursor in a note to turn it into details.",
    group: "structure",
    applies: (c) => has(c, "note"),
    when: "ascribe.at.note",
    lightbulb: "refactor",
    ask: () => ({
      steps: () => [
        {
          kind: "text",
          key: "title",
          prompt: "What's the title readers click to open it?",
          validate: oneLine("a title"),
        },
      ],
      args: (answers) => ({ title: text(answers, "title")?.trim() }),
    }),
    does: { operation: "noteToDetails" },
  },
  // Details.
  {
    id: "wrapDetails",
    title: "Wrap in collapsible details",
    description: "Put the block or the selected blocks in `@details`, with a title",
    where: "A paragraph, code block, table, include, or snippet, or whole blocks selected",
    hint: "Put the cursor in a block, or select whole blocks, to wrap them in details.",
    group: "structure",
    applies: (c) =>
      blocksSelected(c) ||
      (cursorOrProse(c) && has(c, "paragraph", "codeBlock", "table", "snippet", "include")),
    when: "ascribe.selection.blocks || ascribe.selection.proseOrNone && ascribe.at.block",
    lightbulb: "refactor",
    ask: () => ({
      steps: () => [
        {
          kind: "text",
          key: "title",
          prompt: "What's the title readers click to open it?",
          validate: oneLine("a title"),
        },
      ],
      args: (answers) => ({ title: text(answers, "title")?.trim() }),
    }),
    does: { operation: "wrapDetails" },
  },
  {
    id: "unwrapDetails",
    title: "Remove the details, keeping the content",
    description: "Take the content out of its `@details` and its title",
    where: "A details block",
    hint: "Put the cursor in a details block to take its content out of it.",
    group: "structure",
    applies: (c) => has(c, "details"),
    when: "ascribe.at.details",
    lightbulb: "refactor",
    does: { operation: "unwrapDetails" },
  },
  // Steps.
  {
    id: "makeSteps",
    title: "Make the list steps",
    description: "Show the numbered list as `@steps`",
    where: "A numbered list that isn't steps",
    hint: "Put the cursor in a numbered list to make it steps.",
    group: "structure",
    applies: key("ascribe.at.numberedList"),
    when: "ascribe.at.numberedList",
    lightbulb: "refactor",
    does: { operation: "makeSteps" },
  },
  {
    id: "removeSteps",
    title: "Make the steps a plain list",
    description: "Remove the list's `@steps`",
    where: "Steps",
    hint: "Put the cursor in a list of steps to make it a plain list.",
    group: "structure",
    applies: (c) => has(c, "steps"),
    when: "ascribe.at.steps",
    lightbulb: "refactor",
    does: { operation: "removeSteps" },
  },
  // Headings.
  {
    id: "addHeadingId",
    title: "Give the heading a stable id",
    description: "Add `@id:` under the heading, so links to it survive rewording it",
    where: "A heading without `@id`",
    hint: "Put the cursor in a heading without an @id to give it one.",
    group: "link",
    applies: key("ascribe.at.headingWithoutAtId"),
    when: "ascribe.at.headingWithoutAtId",
    lightbulb: "refactor",
    ask: (c) => ({
      steps: () => [
        {
          kind: "text",
          key: "id",
          prompt: "What id? Links name the heading by it.",
          value: innermost(c, "heading")?.id || undefined,
          validate: validId,
        },
      ],
      args: (answers) => ({ id: text(answers, "id")?.trim() }),
    }),
    does: { operation: "addHeadingId" },
  },
  {
    id: "copyLinkToSection",
    title: "Copy a link to this section",
    description:
      "Copy the heading's destination from the content root, such as `/guides/install.md#install-cli`, to paste in any page",
    where: "A heading",
    hint: "Put the cursor in a heading to copy a link to its section.",
    group: "link",
    applies: key("ascribe.at.headingWithId"),
    when: "ascribe.at.headingWithId",
    needs: ["headings"],
    does: { run: copyLinkToSection },
  },
  // Blocks inserted on a blank line.
  {
    id: "insertNote",
    title: "Insert a note",
    description: "Add an `@note` callout, with its text ready to type",
    where: "A blank line between blocks",
    hint: "Put the cursor on a blank line between blocks to insert a note.",
    group: "write",
    applies: insertable,
    when: "ascribe.insertable",
    needs: ["notes"],
    ask: (_c, targets) => ({
      steps: () => [noteTypeStep(targets.notes)],
      args: (answers) => ({ type: text(answers, "type") }),
    }),
    does: { operation: "insertNote" },
  },
  {
    id: "insertSteps",
    title: "Insert steps",
    description: "Add `@steps` and a numbered list",
    where: "A blank line between blocks",
    hint: "Put the cursor on a blank line between blocks to insert steps.",
    group: "write",
    applies: insertable,
    when: "ascribe.insertable",
    ask: () => ({
      steps: () => [
        {
          kind: "text",
          key: "count",
          prompt: "How many steps?",
          value: "3",
          validate: wholeNumber(1, 50),
        },
      ],
      args: (answers) => ({ count: Number(text(answers, "count")) }),
    }),
    does: { operation: "insertSteps" },
  },
  {
    id: "insertDetails",
    title: "Insert collapsible details",
    description: "Add a `@details` block with a title",
    where: "A blank line between blocks",
    hint: "Put the cursor on a blank line between blocks to insert details.",
    group: "write",
    applies: insertable,
    when: "ascribe.insertable",
    ask: () => ({
      steps: () => [
        {
          kind: "text",
          key: "title",
          prompt: "What's the title readers click to open it?",
          validate: oneLine("a title"),
        },
      ],
      args: (answers) => ({ title: text(answers, "title")?.trim() }),
    }),
    does: { operation: "insertDetails" },
  },
  {
    id: "insertVariantGroup",
    title: "Insert content that varies",
    description: "Add `@variant` arms, one for each value of a dimension you choose",
    where: "A blank line between blocks",
    hint: "Put the cursor on a blank line between blocks to insert content that varies.",
    group: "structure",
    applies: insertable,
    when: "ascribe.insertable",
    needs: ["dimensions"],
    ask: (_c, targets) => ({
      steps: (answers) => {
        const dimension = chosenDimension(targets, answers);
        return [dimensionStep(targets), valueStep(dimension, "pickMany")];
      },
      args: (answers) => ({
        dimension: text(answers, "dimension"),
        values: list(answers, "values"),
      }),
    }),
    does: { operation: "insertVariantGroup" },
  },
  {
    id: "addVariantArm",
    title: "Add a variant",
    description: "Add a `@variant` arm for another value of the group's dimension",
    where: "A group of one dimension's `@variant` arms",
    hint: "Put the cursor in a group of @variant arms to add one.",
    group: "structure",
    applies: key("ascribe.at.dimensionGroup"),
    when: "ascribe.at.dimensionGroup",
    needs: ["dimensions"],
    ask: (c, targets) => {
      const group = innermost(c, "variantGroup");
      const dimension = targets.dimensions?.find((d) => d.name === group?.dimension);
      const used = (group?.arms ?? []).flatMap((arm) => arm.value?.split("|") ?? []);
      return {
        steps: () => [valueStep(dimension, "pick", used)],
        args: (answers) => ({ value: text(answers, "value") }),
      };
    },
    does: { operation: "addVariantArm" },
  },
  {
    id: "removeVariantArm",
    title: "Remove this variant",
    description: "Remove the `@variant` arm the cursor is in from its group",
    where: "An arm of a group with others",
    hint: "Put the cursor in an arm of a group of @variant arms to remove it.",
    group: "structure",
    applies: key("ascribe.at.removableArm"),
    when: "ascribe.at.removableArm",
    does: { operation: "removeVariantArm" },
  },
  {
    id: "markAvailable",
    title: "Mark where it's available",
    description:
      "Add `@available:` to the section or block, or `{available=…}` to the table row, with a feature or a spec",
    where: "A heading (its section), a block, or a table's body row",
    hint: "Put the cursor in a heading, a block, or a table's row to mark where it's available.",
    group: "structure",
    applies: key("ascribe.at.markable"),
    when: "ascribe.at.markable",
    needs: ["features", "dimensions"],
    ask: (_c, targets) => ({
      steps: (answers) => availabilitySteps(targets, answers),
      args: (answers) => ({ spec: availabilitySpec(answers) }),
    }),
    does: { operation: "markAvailable" },
  },
  // The page.
  {
    id: "setPageVariant",
    title: "Make the page one variant",
    description: "Set a dimension's value under `variant:` in the page's frontmatter",
    where: "Anywhere in a page",
    hint: "Open a page of the project to set its variant.",
    group: "structure",
    applies: inPage,
    when: "ascribe.inPage",
    needs: ["dimensions"],
    ask: (_c, targets) => ({
      steps: (answers) => [
        dimensionStep(targets),
        valueStep(chosenDimension(targets, answers), "pick"),
      ],
      args: (answers) => ({
        dimension: text(answers, "dimension"),
        value: text(answers, "value"),
      }),
    }),
    does: { operation: "setPageVariant" },
  },
  {
    id: "setPageAvailable",
    title: "Set where the page is available",
    description: "Set `available:` in the page's frontmatter, with a feature or a spec",
    where: "Anywhere in a page",
    hint: "Open a page of the project to set where it's available.",
    group: "structure",
    applies: inPage,
    when: "ascribe.inPage",
    needs: ["features", "dimensions"],
    ask: (_c, targets) => ({
      steps: (answers) => availabilitySteps(targets, answers),
      args: (answers) => ({ spec: availabilitySpec(answers) }),
    }),
    does: { operation: "setPageAvailable" },
  },
  // Links and phrases.
  {
    id: "linkSelection",
    title: "Link the selected text",
    description: "Make the selection a link, `[text](…)`, to a page or heading you choose",
    where: "Text selected in one paragraph or heading",
    hint: "Select text in one paragraph or heading to link it.",
    group: "link",
    applies: key("ascribe.selection.prose"),
    when: "ascribe.selection.prose",
    lightbulb: "refactor",
    needs: ["pages", "headings"],
    ask: (_c, targets) => ({
      steps: () => [destinationStep(targets)],
      args: (answers) => ({ destination: text(answers, "destination") }),
    }),
    does: { operation: "linkSelection" },
  },
  {
    id: "insertLink",
    title: "Insert a link",
    description: "Add a link, `[](…)`, to a page or heading you choose, that shows its title",
    where: "The cursor in a paragraph or heading, outside links and phrases",
    hint: "Put the cursor in a paragraph's text, with nothing selected, to insert a link.",
    group: "link",
    applies: (c) => cursor(c) && key("ascribe.at.text")(c),
    when: "ascribe.cursor && ascribe.at.text",
    needs: ["pages", "headings"],
    ask: (_c, targets) => ({
      steps: () => [destinationStep(targets)],
      args: (answers) => ({ destination: text(answers, "destination") }),
    }),
    does: { operation: "insertLink" },
  },
  {
    id: "setLinkTarget",
    title: "Change where the link goes",
    description: "Set the link's destination to a page or heading you choose",
    where: "A link",
    hint: "Put the cursor in a link to change where it goes.",
    group: "link",
    applies: (c) => has(c, "link"),
    when: "ascribe.at.link",
    needs: ["pages", "headings"],
    ask: (_c, targets) => ({
      steps: () => [destinationStep(targets)],
      args: (answers) => ({ destination: text(answers, "destination") }),
    }),
    does: { operation: "setLinkTarget" },
  },
  {
    id: "useTargetTitle",
    title: "Show the page's title as the link text",
    description: "Empty the link's text, `[](…)`, so it shows its target's title",
    where: "A link to a page, with text",
    hint: "Put the cursor in a link with text to show its target's title instead.",
    group: "link",
    applies: key("ascribe.at.linkWithText"),
    when: "ascribe.at.linkWithText",
    lightbulb: "refactor",
    does: { operation: "useTargetTitle" },
  },
  {
    id: "insertPhrase",
    title: "Insert a phrase",
    description: "Add a phrase the content model declares, `{key}`, which shows its value",
    where: "The cursor in a paragraph or heading, outside links and phrases",
    hint: "Put the cursor in a paragraph's text, with nothing selected, to insert a phrase.",
    group: "write",
    applies: (c) => cursor(c) && key("ascribe.at.text")(c),
    when: "ascribe.cursor && ascribe.at.text",
    needs: ["phrases"],
    ask: (_c, targets) => ({
      steps: () => [phraseStep(targets)],
      args: (answers) => ({ key: text(answers, "key") }),
    }),
    does: { operation: "insertPhrase" },
  },
  // Media, includes, snippets, and widgets.
  {
    id: "insertImage",
    title: "Insert an image",
    description: "Add an image of the project, `![alt](path)`, with its alt text",
    where: "A blank line between blocks",
    hint: "Put the cursor on a blank line between blocks to insert an image.",
    group: "media",
    applies: insertable,
    when: "ascribe.insertable",
    needs: ["images"],
    ask: (_c, targets) => ({
      steps: () => [
        imageStep(targets),
        {
          kind: "text",
          key: "alt",
          prompt: "Describe the image for readers who can't see it.",
          validate: oneLine("a description"),
        },
      ],
      args: (answers) => ({ path: text(answers, "path"), alt: text(answers, "alt")?.trim() }),
    }),
    does: { operation: "insertImage" },
  },
  {
    id: "setImageWidth",
    title: "Set the image's width",
    description: "Set the image's `width` attribute",
    where: "An image",
    hint: "Put the cursor in an image to set its width.",
    group: "media",
    applies: (c) => has(c, "image"),
    when: "ascribe.at.image",
    ask: (c) => ({
      steps: () => [
        {
          kind: "text",
          key: "width",
          prompt: "How wide, in pixels?",
          value: innermost(c, "image")?.attributes.find((a) => a.key === "width")?.value ?? "",
          validate: positiveNumber,
        },
      ],
      args: (answers) => ({ width: text(answers, "width")?.trim() }),
    }),
    does: { operation: "setImageWidth" },
  },
  {
    id: "setImageAlt",
    title: "Change the image's description",
    description: "Set the image's alt text, `![alt]`, which readers who can't see it get",
    where: "An image",
    hint: "Put the cursor in an image to change its description.",
    group: "media",
    applies: (c) => has(c, "image"),
    when: "ascribe.at.image",
    ask: (c) => ({
      steps: () => [
        {
          kind: "text",
          key: "alt",
          prompt: "Describe the image for readers who can't see it.",
          value: innermost(c, "image")?.alt,
          validate: oneLine("a description"),
        },
      ],
      args: (answers) => ({ alt: text(answers, "alt")?.trim() }),
    }),
    does: { operation: "setImageAlt" },
  },
  {
    id: "insertInclude",
    title: "Include a fragment",
    description: "Add `@include:` for a fragment, a page, or one of its sections",
    where: "A blank line between blocks",
    hint: "Put the cursor on a blank line between blocks to include a fragment.",
    group: "media",
    applies: insertable,
    when: "ascribe.insertable",
    needs: ["fragments", "pages", "headings"],
    ask: (_c, targets) => ({
      steps: (answers) => includeSteps(targets, answers),
      args: (answers) => ({ path: includePath(answers) }),
    }),
    does: { operation: "insertInclude" },
  },
  {
    id: "insertSnippet",
    title: "Insert a code snippet",
    description: "Add `@snippet:` with code from one of the project's sources",
    where: "A blank line between blocks",
    hint: "Put the cursor on a blank line between blocks to insert a code snippet.",
    group: "media",
    applies: insertable,
    when: "ascribe.insertable",
    needs: ["snippets"],
    ask: (_c, targets) => ({
      steps: (answers) => snippetSteps(targets, answers),
      args: (answers) => ({ address: snippetAddress(targets, answers) }),
    }),
    does: { operation: "insertSnippet" },
  },
  {
    id: "insertWidget",
    title: "Insert a widget",
    description: "Add one of the project's widgets, `@name`, with the attributes it needs",
    where: "A blank line between blocks",
    hint: "Put the cursor on a blank line between blocks to insert a widget.",
    group: "media",
    applies: insertable,
    when: "ascribe.insertable",
    needs: ["widgets", "notes"],
    ask: (_c, targets) => ({
      steps: (answers) => widgetSteps(targets, answers),
      args: (answers) => widgetArgs(targets, answers),
    }),
    does: { operation: "insertWidget" },
  },
];

/**
 * The actions the lightbulb offers for a context answer: those with a
 * lightbulb kind that apply. Nothing without an answer: the lightbulb never
 * asks for one.
 */
export function lightbulbActions(
  context: ContextResult | undefined,
  actions: Action[] = ACTIONS,
): Action[] {
  if (!context) return [];
  return actions.filter((action) => action.lightbulb !== undefined && action.applies(context));
}
