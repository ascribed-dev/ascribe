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

/**
 * Problems by severity, as the Problems panel counts them: drawn as its
 * icons, each with its count, and said in words (`../problems.ts`).
 */
export interface ProblemCounts {
  errors: number;
  warnings: number;
  /** Information, as the language server sends advice. */
  advice: number;
}

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
  /**
   * A row's state, drawn first, before its description, as a colored label:
   * the warning color when the row warns. Empty when it has none.
   */
  state: string;
  description: string;
  /** Its hover text, on its label. */
  tooltip: string;
  /** A codicon's name. */
  icon: string;
  /**
   * A row's problems, drawn in place of its state as an icon per severity
   * with its count; its state says them in words, for its tooltip and a
   * screen reader. `null` for a row that isn't problems, or has none.
   */
  problems: ProblemCounts | null;
  /** Whether it's dimmed: a section or group with nothing in it, a file that isn't found. */
  dimmed: boolean;
  /** Whether it says something needs doing: overdue, too long, stale. */
  warn: boolean;
  /** Whether choosing it does something, which the extension asks of the model. */
  acts: boolean;
  /**
   * Whether choosing it opens an address outside VS Code, on the published
   * site: it's drawn as a link, and the extension opens it, never the webview.
   */
  link: boolean;
  /** For a section: whether it starts open, until the reader opens or closes it. */
  open: boolean;
  children: ViewItem[];
}

/** A piece of a title: text, or a code span's content, drawn as code. */
interface TitlePiece {
  type: "text" | "code";
  value: string;
}

/** The head of This page: what the page is. */
export interface ThisPageHead {
  /** The title in pieces, each set as text: a code span's is drawn as code. */
  title: TitlePiece[];
  /** Its content type, or "fragment"; `null` when no one type applies. */
  type: string | null;
  /** Its content path. */
  path: string;
}

/** A button under a message, which the extension runs by its key. */
export interface ViewAction {
  key: string;
  label: string;
}

/** What This page draws: a message in place of the page, or its head and sections. */
export interface ThisPageAnswer {
  /**
   * No page open, the server starting or failed, or Restricted Mode; `null`
   * when it has a page to draw.
   */
  message: string | null;
  /** The buttons under the message: Show Output and Restart, Manage Workspace Trust. */
  actions: ViewAction[];
  /** What the page is; `null` with a message. */
  head: ThisPageHead | null;
  sections: ViewItem[];
}

/** What This page's webview drew, for tests. */
export interface ThisPageReport {
  message: string | null;
  /** The labels of the buttons under the message. */
  actions: string[];
  /** The head's text, line by line: the title, then the type and the path. */
  head: string[];
  /** The title's pieces drawn as code, each as its text. */
  code: string[];
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
  /** The problems' icons drawn, each as its codicon and count: "warning 1". */
  severities: string[];
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

/** A choice that brings counts current: Check Every Build, or Check Agent Files. */
export interface ProjectCheck {
  key: string;
  label: string;
}

/**
 * One group of the Project view's Health: the counts that are from the same
 * run, so the group's head says once when they're from and, while they
 * aren't current, the action that brings them current.
 */
export interface ProjectHealthGroup {
  /** `everyBuild`, `addresses`, `asYouType`, or `lastSave`. */
  key: string;
  /** "Every build", "Against the base", "As you type", or "At the last save". */
  label: string;
  /**
   * `current`: its counts are as the project is now. `outOfDate`: from a run
   * the project has changed since, or one that couldn't finish. `unknown`:
   * never run. `running`: being run now.
   */
  state: "current" | "outOfDate" | "unknown" | "running";
  /** When its counts are from, or why they aren't shown: "checked at 3:30 PM", "not checked yet". Empty when there's nothing to say. */
  note: string;
  /** The action at its head that brings it current, while it isn't. */
  action: ProjectCheck | null;
  counts: ProjectCount[];
}

/** One count of the Project view's Health, in its group. */
export interface ProjectCount {
  /** Names the count, and what choosing it opens. */
  key: string;
  label: string;
  /**
   * `current`: a count of something, as the project is now. `none`: current,
   * and zero. `outOfDate`, `unknown`, `running`: as its group, or its own
   * note says.
   */
  state: "current" | "none" | "outOfDate" | "unknown" | "running";
  /** The count in words: "2 errors and 5 warnings", "0", or "—" when it isn't current. */
  value: string;
  /**
   * Problems, drawn as an icon per severity with its count, its value saying
   * them in words for its tooltip and a screen reader; absent for any other
   * count, or none.
   */
  problems?: ProblemCounts;
  /**
   * Its own note, when it isn't current though its group is, as overdue
   * reviews checked on another day; `null` when its group's says it.
   */
  note: string | null;
  /** Whether choosing the count opens its list, which only a current count above zero does. */
  opens: boolean;
  /** Its own action, beside its own note. */
  action: ProjectCheck | null;
}

/** One of the Project view's actions. */
export interface ProjectAction {
  key: string;
  label: string;
  /** A codicon's name. */
  icon: string;
  /** How it runs. */
  caption: string;
  /** Whether it's off: every action is, in Restricted Mode. */
  off: boolean;
}

/** A group of the Project view's actions: Check, Write, or Review and agents. */
export interface ProjectActionGroup {
  label: string;
  actions: ProjectAction[];
}

/** A button of the Project view's head, drawn as its icon, with its label for a screen reader. */
export interface ProjectButton {
  key: string;
  label: string;
  /** A codicon's name. */
  icon: string;
  /** Its hover text. */
  tooltip: string;
}

/** A build the Project view's switch offers. */
interface ProjectBuildChoice {
  /** The build's name, as the switch sends it back: `site`, or a line's, `site@3.4`. */
  name: string;
  /** What else is true of it: "editor build", "line 3.4 of site". Empty when nothing is. */
  description: string;
}

/** The head of the Project view: what the project is, its server, and the build you're looking at. */
export interface ProjectHead {
  name: string;
  folder: string;
  /** The server's state, in words. */
  state: string;
  version: string | null;
  /** Beside the server's state: Show Output and Restart. */
  server: ProjectButton[];
  /** Beside the name: Project Settings and VS Code's Settings. */
  settings: ProjectButton[];
  /**
   * The build you're looking at, among the builds to choose from; `null`
   * until the server has said what they are.
   */
  build: { shown: string; choices: ProjectBuildChoice[] } | null;
}

/** A row of the Project view's About. */
export interface ProjectAboutRow {
  label: string;
  value: string;
}

/** What the Project view draws: a message in place of the project, or the project. */
export interface ProjectAnswer {
  /** No project to show, or Restricted Mode; `null` when it has one. */
  message: string | null;
  /** The buttons under the message: Manage Workspace Trust, in Restricted Mode. */
  notice: { key: string; label: string }[];
  head: ProjectHead | null;
  /** How many things there are to do, for Health's header: the current counts above zero. */
  toDo: number;
  /** Health's counts, in their groups. */
  health: ProjectHealthGroup[];
  /** The actions, in their groups; in Restricted Mode, each off. */
  actions: ProjectActionGroup[];
  /** What the project is made of: its content root, builds, release lines, sources, and content types. */
  about: ProjectAboutRow[];
}

/** What the Project view's webview drew, for tests. */
export interface ProjectReport {
  message: string | null;
  /** The head's text, line by line. */
  head: string[];
  /** The build switch's choices, and the one it shows; `null` when there's no switch. */
  build: { shown: string; choices: string[] } | null;
  /** Each section as drawn: its key, its header's text, and whether it's open. */
  sections: { key: string; label: string; open: boolean }[];
  /**
   * Health's groups as drawn: each head's text, the action at it, and the
   * line that shows its counts of none, "Nothing to do" or "Nothing else to
   * do", or `null` when it has none.
   */
  groups: { key: string; head: string; action: string | null; none: string | null }[];
  /**
   * Each count as drawn: its value, its note (its own, or else its group's),
   * its state, whether it's shown or hidden behind its group's line of none,
   * and whether its value is dimmed.
   */
  health: {
    key: string;
    value: string;
    note: string;
    state: string;
    shown: boolean;
    dimmed: boolean;
    /** The icons drawn in place of its value's text, each as its codicon and count: "error 2". */
    severities: string[];
  }[];
  /**
   * The buttons' labels, in order: the head's, the actions', then Health's
   * groups' actions and their counts'. An icon's button by its label.
   */
  buttons: string[];
  /** The labels of the buttons that are off. */
  off: string[];
  /** About's rows, as "label: value". */
  about: string[];
  /** Content security policy violations seen so far. */
  violations: string[];
}
