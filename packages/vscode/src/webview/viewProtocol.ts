// The messages between the extension and a webview view of the sidebar: This
// page, and the views that follow it. A webview view is made late, when the
// view is first shown, and its document is thrown away and made again when
// it's hidden, so a message sent before its script runs is lost. The webview
// says it's ready; only then does the extension send an answer, the newest
// it has, and it answers every ready, so a webview that reloads is drawn
// again. Each answer carries a number that rises, and the webview ignores
// one older than what it has drawn. Once an answer is in its document, the
// webview says so with what it drew, which the tests wait for.
//
// Apart from VS Code and the DOM: the extension and the webview both read it.

/** Extension to webview. */
export interface ToView<Answer> {
  type: "answer";
  /** Rises with each answer the extension makes. */
  seq: number;
  answer: Answer;
}

/** Webview to extension. */
export type FromView<Report> =
  | { type: "ready" }
  /** The answer `seq` is in the webview's document. */
  | { type: "drawn"; seq: number; report: Report }
  /** Choose the item `key` of the answer `seq`: open what it names. */
  | { type: "open"; seq: number; key: string }
  /** Escape: the focus goes back to the page's editor. */
  | { type: "focusEditor" };

// This page.

/** One item This page draws, as the model (`sidebar.ts`) builds it. */
export interface ViewItem {
  /**
   * Where it is among the answer's items: the same for the same item from
   * one answer to the next, so the extension finds what a choice names.
   */
  key: string;
  /**
   * `section`: a top-level section, which opens and closes. `group`: a
   * heading within one, over its items. `row`: a label and a value, drawn as
   * a description list's term and details. `item`: one thing on its own line.
   */
  kind: "section" | "group" | "row" | "item";
  label: string;
  description: string;
  /** Its hover text, on its label. */
  tooltip: string;
  /** A codicon's name. */
  icon: string;
  /** Whether it's dimmed: a section or group with nothing in it, a file that isn't found. */
  dimmed: boolean;
  /** Whether it says something needs doing: overdue, too long, stale. */
  warn: boolean;
  /** Whether choosing it does something, which the extension asks of the model. */
  acts: boolean;
  /** For a section: whether it starts open, until the reader opens or closes it. */
  open: boolean;
  children: ViewItem[];
}

/** What This page draws: a message in place of the sections, or the sections. */
export interface ThisPageAnswer {
  /** No page open, or the server's state; `null` when it has sections to draw. */
  message: string | null;
  sections: ViewItem[];
}

/** What This page's webview drew, for tests. */
export interface ThisPageReport {
  message: string | null;
  /** Each section as drawn: whether it's open, and its items' text, depth first. */
  sections: { key: string; label: string; open: boolean; items: string[] }[];
  /** Whether a row's text wraps, as the stylesheet sets it. */
  wraps: boolean;
  /**
   * The key of what has the keyboard's focus in the view: an item's, or a
   * section's for its header; `null` for nothing.
   */
  focused: string | null;
  /** Whether the view's document has the focus. */
  hasFocus: boolean;
  /** How far the view is scrolled, in pixels. */
  scroll: number;
  /** The newest probe the report answers, or 0 when it answers none. */
  probed: number;
  /** Content security policy violations seen so far. */
  violations: string[];
}

/**
 * Extension to This page's webview, for tests: does what a reader does, and
 * answers with a report of the answer drawn, in this order.
 */
export interface ThisPageProbe {
  type: "probe";
  /** Rises with each probe. */
  probe: number;
  /** Opens every section. */
  open?: boolean;
  /** Puts the keyboard's focus on the item of this key, or on a section's header by its key. */
  focus?: string;
  /** Scrolls the view to this many pixels from its top. */
  scroll?: number;
  /** Chooses what has the focus, as Enter does. */
  press?: boolean;
}

// The Project view.

/** One count of the Project view's Health. */
export interface ProjectCount {
  /** Names the count, and what choosing it opens. */
  key: string;
  label: string;
  /**
   * `current`: a count of something, as the project is now. `none`: current,
   * and zero. `outOfDate`: from a run the project has changed since, or one
   * that couldn't finish. `unknown`: never run. `running`: being run now.
   */
  state: "current" | "none" | "outOfDate" | "unknown" | "running";
  /** The count as it's shown: "2 errors, 5 warnings", "0", or "—" when it isn't current. */
  value: string;
  /** When it's from, or why it isn't shown. */
  note: string;
  /** Whether choosing the count opens its list, which only a current count above zero does. */
  opens: boolean;
  /** The action beside it that brings it current, while it isn't. */
  action: { key: string; label: string } | null;
}

/** One of the Project view's actions. */
export interface ProjectAction {
  key: string;
  label: string;
  /** A codicon's name. */
  icon: string;
  /** How it runs. */
  caption: string;
}

/** What the Project view draws: a message in place of the project, or the project. */
export interface ProjectAnswer {
  /** No project to show; `null` when it has one. */
  message: string | null;
  /** The project's name, its folder, its server's state, and the binary's version. */
  head: { name: string; folder: string; state: string; version: string | null } | null;
  /** How many things there are to do, for Health's header: counts above zero, out of date, or never run. */
  toDo: number;
  health: ProjectCount[];
  actions: ProjectAction[];
}

/** What the Project view's webview drew, for tests. */
export interface ProjectReport {
  message: string | null;
  /** The head's text, line by line. */
  head: string[];
  /** Each section as drawn: its key, its header's text, and whether it's open. */
  sections: { key: string; label: string; open: boolean }[];
  /** Each count as drawn: its value, its note, its state, and whether its value is dimmed. */
  health: { key: string; value: string; note: string; state: string; dimmed: boolean }[];
  /** The buttons' labels, in order: the counts' and their actions', then the actions'. */
  buttons: string[];
  /** Content security policy violations seen so far. */
  violations: string[];
}
