// What the preview's page may ask of the extension. The page draws a page's
// raw HTML, and in review the change under review, so what it posts is
// checked before anything is done with it: each message's type is a known
// one, each value has its type and a sane size, and a choice is one of its
// list. `MESSAGES` has a check for every `FromWebview`, and its type makes
// adding a message without one an error. Apart from VS Code, so it can be
// tested by itself. The sidebar's webview views check theirs with the same
// checks (`../ui/webviewView.ts`).

import type { PromptRequest } from "@ascribed/review/github";
import { within } from "../projects.js";
import type {
  FromWebview,
  ImageReport,
  RenderReport,
  ThreadsMethod,
  ThreadsReport,
} from "./protocol.js";

/** A check of an untyped value, which says what it is when it passes. */
export type Check<T> = (value: unknown) => value is T;

/** The longest string a message carries: a path, a link, a name, an anchor, a sentence. */
export const MAX_TEXT = 8 * 1024;

/** The most items in a list or a record: a page's images or elements, a report's blocks. */
const MAX_ITEMS = 10_000;

export const text: Check<string> = (value): value is string =>
  typeof value === "string" && value.length <= MAX_TEXT;

/** A count, an id, or a line: a whole number from 0. */
export const whole: Check<number> = (value): value is number =>
  typeof value === "number" && Number.isSafeInteger(value) && value >= 0;

export const boolean: Check<boolean> = (value): value is boolean => typeof value === "boolean";

function oneOf<const T extends readonly string[]>(...values: T): Check<T[number]> {
  return (value): value is T[number] => typeof value === "string" && values.includes(value);
}

export function nullable<T>(check: Check<T>): Check<T | null> {
  return (value): value is T | null => value === null || check(value);
}

export function isObject(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}

export function listOf<T>(item: Check<T>): Check<T[]> {
  return (value): value is T[] =>
    Array.isArray(value) && value.length <= MAX_ITEMS && value.every(item);
}

function recordOf<T>(item: Check<T>): Check<Record<string, T>> {
  return (value): value is Record<string, T> => {
    if (!isObject(value)) return false;
    const entries = Object.entries(value);
    return entries.length <= MAX_ITEMS && entries.every(([key, v]) => text(key) && item(v));
  };
}

/** An object with each of these fields, each passing its check. Other fields are never read. */
export function shape<T>(fields: { [K in keyof T]-?: Check<T[K]> }): Check<T> {
  return (value): value is T =>
    isObject(value) &&
    Object.entries(fields).every(([key, check]) => (check as Check<unknown>)(value[key]));
}

const THREADS_METHODS = [
  "load",
  "commentTarget",
  "comment",
  "reply",
  "allThreads",
  "resolve",
  "submit",
  "discard",
  "promptAgent",
] as const satisfies readonly ThreadsMethod[];

const threadsMethod: Check<ThreadsMethod> = oneOf(...THREADS_METHODS);

const promptRequest: Check<PromptRequest> = (value): value is PromptRequest => {
  if (!isObject(value)) return false;
  switch (value["kind"]) {
    case "thread":
      return text(value["threadId"]);
    case "fragment-reach":
      return text(value["fragment"]);
    case "open-threads":
    case "page-changes":
      return true;
    default:
      return false;
  }
};

/**
 * An overlay request's parameters, which `answerRequest` (`@ascribed/review`)
 * checks one by one; `promptAgent`'s request, which the preview answers
 * itself, is checked here.
 */
const threadsRequest = (
  value: unknown,
): value is Omit<Extract<FromWebview, { type: "threads" }>, "type"> =>
  shape<{ id: number; method: ThreadsMethod; params: Record<string, unknown> }>({
    id: whole,
    method: threadsMethod,
    params: isObject,
  })(value) &&
  (value.method !== "promptAgent" || promptRequest(value.params["request"]));

const renderReport: Check<RenderReport> = shape<RenderReport>({
  headings: listOf(text),
  anchored: whole,
  elements: recordOf(whole),
  elementsDefined: boolean,
  violations: listOf(text),
  marks: recordOf(whole),
  reviewHeader: nullable(text),
});

const imageReport: Check<ImageReport> = shape<ImageReport>({
  src: text,
  loaded: boolean,
  width: whole,
});

const threadsReport: Check<ThreadsReport> = shape<ThreadsReport>({
  blocks: recordOf(listOf(text)),
  detached: listOf(text),
  unsent: whole,
});

const none = (value: unknown): value is Record<never, never> => isObject(value);

/** The fields of each message, apart from `type`. */
type Fields<T extends FromWebview["type"]> = Omit<Extract<FromWebview, { type: T }>, "type">;

/** A check for every message the page sends, by type. */
const MESSAGES: { [T in FromWebview["type"]]: Check<Fields<T>> } = {
  ready: none,
  build: shape<Fields<"build">>({ name: text }),
  open: shape<Fields<"open">>({ href: text }),
  showOutput: none,
  showProblems: none,
  rendered: shape<Fields<"rendered">>({ seq: whole, report: renderReport }),
  revealedLine: shape<Fields<"revealedLine">>({ line: whole, source: text }),
  scrolled: shape<Fields<"scrolled">>({ line: whole }),
  openLine: shape<Fields<"openLine">>({ line: whole }),
  images: shape<Fields<"images">>({ seq: whole, images: listOf(imageReport) }),
  openSource: shape<Fields<"openSource">>({ source: text }),
  openFile: shape<Fields<"openFile">>({ path: text }),
  atEnd: none,
  openPage: shape<Fields<"openPage">>({ path: text }),
  threads: threadsRequest,
  openThread: shape<Fields<"openThread">>({ threadId: text, path: text }),
  notify: shape<Fields<"notify">>({ message: text }),
  signIn: none,
  useGh: none,
  git: shape<Fields<"git">>({ command: oneOf("pull", "push", "fetch") }),
  refreshThreads: none,
  threadsDrawn: shape<Fields<"threadsDrawn">>({ report: threadsReport }),
  surface: shape<Fields<"surface">>({ surface: oneOf("page", "site") }),
  siteShown: shape<Fields<"siteShown">>({ url: text }),
};

/** A message from the page, when it is one the extension takes; `undefined` otherwise. */
export function checkMessage(value: unknown): FromWebview | undefined {
  if (!isObject(value)) return undefined;
  const type = value["type"];
  if (typeof type !== "string" || !Object.hasOwn(MESSAGES, type)) return undefined;
  const check = MESSAGES[type as FromWebview["type"]] as Check<unknown>;
  return check(value) ? (value as FromWebview) : undefined;
}

/** The VS Code command each of the preview's git buttons runs. */
export const GIT_COMMANDS: Readonly<
  Record<Extract<FromWebview, { type: "git" }>["command"], string>
> = {
  pull: "git.pull",
  push: "git.push",
  fetch: "git.fetch",
};

/**
 * Whether the page may have a file opened: one the extension gave it
 * (`given`), or one inside the project's folder. A path the page builds from
 * a root and a content path is checked against that root the same way.
 */
export function mayOpen(
  file: string,
  roots: { given?: readonly string[] | undefined; folder: string | undefined },
): boolean {
  if (roots.given?.includes(file)) return true;
  return roots.folder !== undefined && within(file, roots.folder);
}
