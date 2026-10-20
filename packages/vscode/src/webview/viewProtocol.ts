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
  /** Content security policy violations seen so far. */
  violations: string[];
}
