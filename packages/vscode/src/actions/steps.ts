// What an action asks the writer for, as short steps: pick one, pick
// several, or enter text. The steps are built from the project's targets
// (`ascribe/targets`), so a writer chooses a page, a phrase, or a dimension's
// values instead of writing syntax. This module has no VS Code in it: the
// steps and the flow through them are unit-tested, and `ask.ts` shows them.

import type {
  TargetDimension,
  TargetFeature,
  TargetNote,
  TargetsResult,
  TargetWidget,
} from "../shapes.js";

/** One choice of a pick step. */
export interface Choice {
  label: string;
  description?: string | undefined;
  detail?: string | undefined;
  /** What choosing it answers. */
  value: string;
  /** The name of the run of choices it's in, shown above the run. */
  section?: string | undefined;
}

export type Step =
  | {
      kind: "pick" | "pickMany";
      /** The answer's name. */
      key: string;
      /** The question. */
      prompt: string;
      choices: Choice[];
      /** What to say instead of asking when there's nothing to choose. */
      empty: string;
    }
  | {
      kind: "text";
      key: string;
      prompt: string;
      /** What the box starts with. */
      value?: string | undefined;
      placeholder?: string | undefined;
      /** Why the text can't be used, or `undefined` when it can. */
      validate(value: string): string | undefined;
    };

/** A pick step's answer is the chosen value or values; a text step's, the text. */
export type Answer = string | string[];
export type Answers = Record<string, Answer>;

/** What an action sends `ascribe/edit` (or its own `run`) as `args`. */
export type Args = Record<string, unknown>;

/**
 * An action's questions. `steps` is asked again after each answer, so a
 * step can depend on earlier answers (the values of the chosen dimension),
 * and the count can change (a widget's required attributes).
 */
export interface Wizard {
  steps(answers: Answers): Step[];
  args(answers: Answers): Args;
}

/** Where a step is in its wizard, for the counter, and its earlier answer, after Back. */
export interface Position {
  title: string;
  step: number;
  total: number;
  previous: Answer | undefined;
}

/** A step's outcome: an answer, Back, or `undefined` when the writer gave up. */
export type Shown = { answer: Answer } | "back" | undefined;

/** What shows a step: the quick input, or a script in tests. */
export interface Prompter {
  show(step: Step, position: Position): Promise<Shown>;
}

/** A wizard's outcome: the arguments, why it can't ask, or `undefined` when cancelled. */
export type Asked = { args: Args } | { error: string } | undefined;

/** Runs a wizard's steps in order, with Back, to its arguments. */
export async function runWizard(title: string, wizard: Wizard, prompter: Prompter): Promise<Asked> {
  const answers = new Map<string, Answer>();
  let i = 0;
  for (;;) {
    const steps = wizard.steps(Object.fromEntries(answers));
    const step = steps[i];
    if (!step) return { args: wizard.args(Object.fromEntries(answers)) };
    if (step.kind !== "text" && step.choices.length === 0) return { error: step.empty };
    const shown = await prompter.show(step, {
      title,
      step: i + 1,
      total: steps.length,
      previous: answers.get(step.key),
    });
    if (shown === undefined) return undefined;
    if (shown === "back") {
      i = Math.max(0, i - 1);
      continue;
    }
    answers.set(step.key, shown.answer);
    // A later step can depend on this answer, so its own answer is asked again.
    for (const later of steps.slice(i + 1)) answers.delete(later.key);
    i += 1;
  }
}

/**
 * An answer a test gives a step: what to choose or type, `"back"` for Back,
 * `null` to cancel, or a function that gives one.
 */
export type Scripted = Answer | null | (() => Promise<Answer | null>);

/**
 * Answers steps from a script, for tests. A pick step's answer must be one of
 * its choices' values, and a text step's must pass its check: a script that
 * couldn't be answered in the quick input fails.
 */
export class ScriptedPrompter implements Prompter {
  constructor(private readonly answers: Scripted[]) {}

  async show(step: Step, position: Position): Promise<Shown> {
    if (this.answers.length === 0) {
      throw new Error(`The script has no answer for step ${position.step} (${step.prompt})`);
    }
    const next = this.answers.shift();
    const scripted = typeof next === "function" ? await next() : next;
    if (scripted === null || scripted === undefined) return undefined;
    if (scripted === "back") return "back";
    if (step.kind === "text") {
      if (typeof scripted !== "string") throw new Error(`${step.prompt} takes text`);
      const problem = step.validate(scripted);
      if (problem) throw new Error(`${JSON.stringify(scripted)} for ${step.prompt}: ${problem}`);
      return { answer: scripted };
    }
    const values = [scripted].flat();
    const offered = new Set(step.choices.map((choice) => choice.value));
    const missing = values.filter((value) => !offered.has(value));
    if (missing.length > 0) {
      throw new Error(
        `${missing.join(", ")} isn't a choice of ${step.prompt}: ${[...offered].join(", ")}`,
      );
    }
    return { answer: step.kind === "pickMany" ? values : (values[0] ?? "") };
  }
}

/** The answer to a step as text; a pick-many step's values joined by commas. */
export function text(answers: Answers, key: string): string | undefined {
  const answer = answers[key];
  return Array.isArray(answer) ? answer.join(", ") : answer;
}

/** The answer to a pick-many step. */
export function list(answers: Answers, key: string): string[] {
  const answer = answers[key];
  return answer === undefined ? [] : Array.isArray(answer) ? answer : [answer];
}

// Text checks.

export const required =
  (what: string) =>
  (value: string): string | undefined =>
    value.trim() === "" ? `Enter ${what}.` : undefined;

export const oneLine =
  (what: string) =>
  (value: string): string | undefined =>
    required(what)(value) ?? (/[\r\n]/.test(value) ? `Write ${what} on one line.` : undefined);

/** An id: a word of letters, digits, `-`, and `_`. */
export function validId(value: string): string | undefined {
  if (value.trim() === "") return "Enter an id.";
  return /^[\p{L}\p{N}_-]+$/u.test(value.trim())
    ? undefined
    : "An id is one word: letters, digits, - and _.";
}

/** A whole number from `min` to `max`. */
export const wholeNumber =
  (min: number, max: number) =>
  (value: string): string | undefined => {
    const n = Number(value.trim());
    return value.trim() !== "" && Number.isInteger(n) && n >= min && n <= max
      ? undefined
      : `Enter a whole number from ${min} to ${max}.`;
  };

/** A positive number. */
export function positiveNumber(value: string): string | undefined {
  const n = Number(value.trim());
  return value.trim() !== "" && Number.isFinite(n) && n > 0 ? undefined : "Enter a number.";
}

// Steps from the project's targets.

/** The syntax a note type writes. */
function noteSyntax(type: string): string {
  return type === "note" ? "@note" : `@note {type=${type}}`;
}

/** Which kind of note: the project's note types, with their labels. */
export function noteTypeStep(notes: TargetNote[] = [], except?: string): Step {
  return {
    kind: "pick",
    key: "type",
    prompt: "Which kind of note?",
    choices: notes
      .filter((note) => note.type !== except)
      .map((note) => ({ label: note.label, description: noteSyntax(note.type), value: note.type })),
    empty: "The content model declares no other note types.",
  };
}

/** What a link goes to: the project's pages, then their headings. */
export function destinationStep(targets: TargetsResult): Step {
  const pages: Choice[] = (targets.pages ?? []).map((page) => ({
    label: `$(file) ${page.title ?? page.path}`,
    description: page.path,
    value: page.link,
    section: "Pages",
  }));
  const headings: Choice[] = (targets.headings ?? []).map((heading) => ({
    label: heading.text,
    description: `${heading.page}#${heading.id}`,
    value: heading.link,
    section: "Headings",
  }));
  return {
    kind: "pick",
    key: "destination",
    prompt: "Link to which page or heading?",
    choices: [...pages, ...headings],
    empty: "The project has no pages to link to.",
  };
}

/** Which phrase: each key with its value. */
export function phraseStep(targets: TargetsResult): Step {
  return {
    kind: "pick",
    key: "key",
    prompt: "Which phrase?",
    choices: (targets.phrases ?? []).map((phrase) => ({
      label: `$(symbol-string) ${phrase.key}`,
      description: phrase.value,
      value: phrase.key,
    })),
    empty: "The content model declares no phrases.",
  };
}

/** Which dimension. */
export function dimensionStep(targets: TargetsResult): Step {
  return {
    kind: "pick",
    key: "dimension",
    prompt: "Which dimension?",
    choices: (targets.dimensions ?? []).map((dimension) => ({
      label: `$(symbol-enum) ${dimension.label}`,
      description: dimension.name,
      value: dimension.name,
    })),
    empty: "The content model declares no dimensions.",
  };
}

/** Which value or values of a dimension, leaving out those in `except`. */
export function valueStep(
  dimension: TargetDimension | undefined,
  kind: "pick" | "pickMany",
  except: string[] = [],
): Step {
  return {
    kind,
    key: kind === "pick" ? "value" : "values",
    prompt: kind === "pick" ? "Which value?" : "Which values?",
    choices: (dimension?.values ?? [])
      .filter((value) => !except.includes(value.value))
      .map((value) => ({
        label: `$(symbol-enum-member) ${value.label}`,
        description: value.value,
        value: value.value,
      })),
    empty: dimension
      ? `Every value of ${dimension.label} is already used.`
      : "The content model declares no such dimension.",
  };
}

/** The dimension a `dimension` answer names. */
export function chosenDimension(
  targets: TargetsResult,
  answers: Answers,
): TargetDimension | undefined {
  const name = text(answers, "dimension");
  return targets.dimensions?.find((dimension) => dimension.name === name);
}

/** The value meaning "enter a spec" in the availability step. */
const ENTER_SPEC = "\u0000spec";

/**
 * Where something is available: a feature, or a spec entered as text. With
 * no features, only the text step.
 */
export function availabilitySteps(targets: TargetsResult, answers: Answers): Step[] {
  const features: TargetFeature[] = targets.features ?? [];
  const example = (targets.dimensions ?? [])
    .flatMap((dimension) => dimension.values.map((value) => value.value))
    .slice(0, 2)
    .join(", ");
  const entry: Step = {
    kind: "text",
    key: "spec",
    prompt: "Where is it available? Targets, each with an optional state and version.",
    placeholder: example ? `For example: ${example}` : undefined,
    validate: oneLine("where it's available"),
  };
  if (features.length === 0) return [entry];
  const pick: Step = {
    kind: "pick",
    key: "feature",
    prompt: "Available as which feature, or where?",
    choices: [
      ...features.map((feature) => ({
        label: `$(tag) ${feature.name}`,
        description: feature.key,
        detail: feature.availability,
        value: feature.key,
      })),
      { label: "Enter where it's available…", value: ENTER_SPEC },
    ],
    empty: "",
  };
  return text(answers, "feature") === ENTER_SPEC ? [pick, entry] : [pick];
}

/** The spec the availability steps answered. */
export function availabilitySpec(answers: Answers): string | undefined {
  const feature = text(answers, "feature");
  return feature !== undefined && feature !== ENTER_SPEC ? feature : text(answers, "spec")?.trim();
}

/** What to include: a fragment or a page, then, for a page, the whole of it or one heading. */
export function includeSteps(targets: TargetsResult, answers: Answers): Step[] {
  const fragments: Choice[] = (targets.fragments ?? []).map((fragment) => ({
    label: `$(file-symlink-file) ${fragment.path}`,
    value: fragment.include,
    section: "Fragments",
  }));
  const pages: Choice[] = (targets.pages ?? []).map((page) => ({
    label: `$(file) ${page.title ?? page.path}`,
    description: page.path,
    value: page.link,
    section: "Pages",
  }));
  const file: Step = {
    kind: "pick",
    key: "file",
    prompt: "Include which fragment or page?",
    choices: [...fragments, ...pages],
    empty: "The project has no fragments or pages to include.",
  };
  const chosen = (targets.pages ?? []).find((page) => page.link === text(answers, "file"));
  if (!chosen) return [file];
  const headings = (targets.headings ?? []).filter((heading) => heading.page === chosen.path);
  if (headings.length === 0) return [file];
  return [
    file,
    {
      kind: "pick",
      key: "section",
      prompt: "The whole page, or one section?",
      choices: [
        { label: "The whole page", value: "" },
        ...headings.map((heading) => ({
          label: heading.text,
          description: `#${heading.id}`,
          value: heading.id,
        })),
      ],
      empty: "",
    },
  ];
}

/** The `@include` path the include steps answered. */
export function includePath(answers: Answers): string | undefined {
  const file = text(answers, "file");
  const section = text(answers, "section");
  return file && section ? `${file}#${section}` : file;
}

/** A snippet's source, file, and region (or the whole file). */
export function snippetSteps(targets: TargetsResult, answers: Answers): Step[] {
  const sources = targets.snippets ?? [];
  const steps: Step[] = [
    {
      kind: "pick",
      key: "source",
      prompt: "From which source?",
      choices: sources.map((source) => ({ label: source.name, value: source.name })),
      empty: "The content model declares no sources.",
    },
  ];
  const source = sources.find((s) => s.name === text(answers, "source"));
  if (!source) return steps;
  steps.push({
    kind: "pick",
    key: "file",
    prompt: "Which file?",
    choices: source.files.map((file) => ({ label: `$(file-code) ${file.path}`, value: file.path })),
    empty: `The source ${source.name} has no files a snippet can show.`,
  });
  const file = source.files.find((f) => f.path === text(answers, "file"));
  if (!file || file.regions.length === 0) return steps;
  steps.push({
    kind: "pick",
    key: "address",
    prompt: "The whole file, or one region?",
    choices: [
      { label: "The whole file", value: file.address },
      ...file.regions.map((region) => ({ label: region.name, value: region.address })),
    ],
    empty: "",
  });
  return steps;
}

/** The `@snippet` address the snippet steps answered. */
export function snippetAddress(targets: TargetsResult, answers: Answers): string | undefined {
  const address = text(answers, "address");
  if (address !== undefined) return address;
  return targets.snippets
    ?.find((source) => source.name === text(answers, "source"))
    ?.files.find((file) => file.path === text(answers, "file"))?.address;
}

/** Which image file. */
export function imageStep(targets: TargetsResult): Step {
  return {
    kind: "pick",
    key: "path",
    prompt: "Which image?",
    choices: (targets.images ?? []).map((image) => ({
      label: `$(file-media) ${image.path}`,
      value: image.link,
    })),
    empty: "The project has no image files.",
  };
}

/** Which widget, then its primary and its required attributes. */
export function widgetSteps(targets: TargetsResult, answers: Answers): Step[] {
  const widgets = targets.widgets ?? [];
  const steps: Step[] = [
    {
      kind: "pick",
      key: "name",
      prompt: "Which widget?",
      choices: widgets.map((widget) => ({
        label: `@${widget.name}`,
        description: widget.description ?? undefined,
        value: widget.name,
      })),
      empty: "The content model declares no widgets.",
    },
  ];
  const widget = chosenWidget(targets, answers);
  if (!widget) return steps;
  if (widget.primary !== "none") {
    steps.push({
      kind: "text",
      key: "primary",
      prompt:
        widget.primary === "identifier"
          ? `What goes after @${widget.name}:? One word.`
          : `What goes after @${widget.name}:?`,
      validate: (value) =>
        widget.primary === "identifier"
          ? /\s/.test(value.trim())
            ? "Enter one word."
            : undefined
          : /[\r\n]/.test(value)
            ? "Write it on one line."
            : undefined,
    });
  }
  for (const attribute of widget.attributes.filter((a) => a.required)) {
    const key = `attribute.${attribute.key}`;
    const prompt = attribute.description
      ? `${attribute.key}: ${attribute.description}`
      : `What is its ${attribute.key}?`;
    const values =
      attribute.type === "boolean"
        ? ["true", "false"]
        : attribute.type === "noteType"
          ? (targets.notes ?? []).map((note) => note.type)
          : attribute.values;
    if (values.length > 0) {
      steps.push({
        kind: attribute.type === "set" ? "pickMany" : "pick",
        key,
        prompt,
        choices: values.map((value) => ({ label: value, value })),
        empty: `${attribute.key} has no values to choose from.`,
      });
    } else {
      steps.push({
        kind: "text",
        key,
        prompt,
        value: attribute.default ?? undefined,
        validate: attribute.type === "number" ? aNumber : oneLine(attribute.key),
      });
    }
  }
  return steps;
}

function aNumber(value: string): string | undefined {
  const n = Number(value.trim());
  return value.trim() !== "" && Number.isFinite(n) ? undefined : "Enter a number.";
}

/** The widget a `name` answer names. */
export function chosenWidget(targets: TargetsResult, answers: Answers): TargetWidget | undefined {
  return targets.widgets?.find((widget) => widget.name === text(answers, "name"));
}

/** The arguments the widget steps answered. */
export function widgetArgs(targets: TargetsResult, answers: Answers): Args {
  const attributes: Record<string, string | string[]> = {};
  for (const key of Object.keys(answers)) {
    if (!key.startsWith("attribute.")) continue;
    const name = key.slice("attribute.".length);
    const type = chosenWidget(targets, answers)?.attributes.find((a) => a.key === name)?.type;
    attributes[name] = type === "set" ? list(answers, key).join("|") : (text(answers, key) ?? "");
  }
  const primary = text(answers, "primary")?.trim();
  return {
    name: text(answers, "name"),
    ...(primary ? { primary } : {}),
    attributes,
  };
}
