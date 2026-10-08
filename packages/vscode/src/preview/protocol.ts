// The types of the `ascribe/preview` request (`crates/ascribe-lsp/src/preview.rs`),
// the review requests (`crates/ascribe-lsp/src/review.rs`), and of the
// messages between the extension and the preview's webview. What the server
// answers is generated from its Rust types, in `../shapes.ts`.

import type { FormattedPiece, PageDiff, PreviewBuild, PreviewProblem } from "../shapes.js";

export type {
  BaseInfo,
  Change,
  ChangedPage,
  ChangesResult,
  Counts,
  PageDiff,
  PreviewBuild,
  PreviewResult,
  PreviewSection,
  SetBaseResult,
} from "../shapes.js";

/** The parameters of `ascribe/preview`. */
export interface PreviewParams {
  textDocument: { uri: string };
  /** A build name; without one the server uses the editor's build. */
  build?: string;
  /** Include what changed on the page against the review base. */
  review?: boolean;
}

/**
 * The site output's frontmatter, which a preview's page `frontmatter` is:
 * `available` is a list of targets. `tests/zod` checks its schema.
 */
export interface PageFrontmatter {
  available?: AvailabilityTarget[];
  [key: string]: unknown;
}

export interface AvailabilityTarget {
  target: string;
  dimension: string;
  states: string[];
  versions?: string[];
  text: string;
}

/** What the preview shows about review, while it's on: the page's changes. */
export interface ReviewView {
  /** The base's name: "main". */
  base: string;
  /** The commit compared with, shortened: "1a2b3c4". */
  commit: string;
  /** `null` when the page didn't change. */
  page: PageDiff | null;
  wasHtml: string | null;
  /** The files the page changed through, when its own file didn't change. */
  causes: { label: string; path: string }[];
  /** Go to the page's first change once it's drawn (after "Next changed page"). */
  goToFirst: boolean;
  /** The pull request's review threads, or `null` when the checkout has no pull request on GitHub. */
  threads: ThreadsView | null;
  /** How many errors the project has (the Problems panel's): a page may not show as it will. */
  errors: number;
}

/** How the local `HEAD` relates to the pull request's head commit (`@ascribed/review/github`). */
export type LocalState = "same" | "behind" | "ahead" | "diverged" | "missing";

/** What the preview shows about the pull request's threads. */
export interface ThreadsView {
  /**
   * `on`: the overlay shows the threads. `signed-out`: there's no GitHub
   * sign-in, so only the changes show. `error`: the threads couldn't be read.
   */
  state: "on" | "signed-out" | "error";
  /** `null` until the pull request is found (signed out, or an error before). */
  pullRequest: { number: number; url: string; baseRefName: string } | null;
  /** How the checkout relates to the pull request's head, with commit counts. */
  local: { state: LocalState; behind: number; ahead: number } | null;
  /** Signed out: whether `gh` is signed in, so it can be used instead. */
  gh: boolean;
  /** For `error`: what went wrong, as a sentence. */
  message: string | null;
  /** A thread to go to once the overlay has read the threads: one opened from another page. */
  goTo: string | null;
}

/** The overlay's requests to its host, over the webview's messages (`OverlayHost`). */
export type ThreadsMethod =
  "load" | "commentTarget" | "comment" | "reply" | "allThreads" | "resolve" | "submit" | "discard";

/**
 * A problem as the preview shows it: one from the server, or one of the
 * extension's own, which may offer an action.
 */
export interface ShownProblem extends PreviewProblem {
  /** A button after the message: `showOutput` opens the output of the previewed project's server. */
  action?: "showOutput";
}

/** An asset the webview should show: the reference and the URL that reaches its source file. */
export interface WebviewAsset {
  reference: string;
  uri: string;
}

/** Extension to webview. */
export type ToWebview =
  | {
      type: "render";
      /** Counts renders; the webview echoes it. */
      seq: number;
      /** The page's content path, which the HTML's source anchors name; `null` with no page. */
      path: string | null;
      build: string;
      builds: PreviewBuild[];
      title: string | null;
      /** The title formatted, when its field sets `inline = "code"`: the heading shows it. */
      formattedTitle: FormattedPiece[] | null;
      available: AvailabilityTarget[];
      html: string | null;
      assets: WebviewAsset[];
      problems: ShownProblem[];
      /** `null` when there's no page, or no project. */
      review: ReviewView | null;
    }
  | { type: "reveal"; id: string }
  /** The changed page after the previewed one, past whose last change the reader stepped. */
  | { type: "nextPage"; page: { path: string; title: string } | null; first: boolean }
  /**
   * Scroll to the block that stands for a line of the previewed file (from
   * 0), to the top; with `ifHidden`, only when no part of it is in view.
   */
  | { type: "revealLine"; line: number; ifHidden: boolean }
  /** The answer to a `threads` request: its result, or a failure to show. */
  | {
      type: "threadsResult";
      id: number;
      result?: unknown;
      error?: { message: string; code?: string };
    }
  /** The threads changed outside the preview (in the source editor): read them again. */
  | { type: "threadsChanged" }
  /** Go to a thread on the page. */
  | { type: "goToThread"; threadId: string }
  /**
   * Which view the panel shows: the page (Ascribe's render), or the site (the
   * dev server's page in a frame, at `url`), or why the site can't show.
   */
  | { type: "surface"; surface: "page" }
  | { type: "surface"; surface: "site"; url: string }
  | { type: "surface"; surface: "site"; problem: string };

/** Webview to extension. */
export type FromWebview =
  | { type: "ready" }
  | { type: "build"; name: string }
  | { type: "open"; href: string }
  | { type: "showOutput" }
  /** Show the Problems panel: the project's errors. */
  | { type: "showProblems" }
  | { type: "rendered"; seq: number; report: RenderReport }
  /** The preview scrolled to the block for `line`, whose anchor is `source`. */
  | { type: "revealedLine"; line: number; source: string }
  /** The reader scrolled the preview: the block at the top stands for `line`. */
  | { type: "scrolled"; line: number }
  /** The reader double-clicked a block that stands for `line`: show it in the editor. */
  | { type: "openLine"; line: number }
  | { type: "images"; seq: number; images: ImageReport[] }
  /** Open the block whose anchor is `source` in the editor, at its lines. */
  | { type: "openSource"; source: string }
  /** Open a file of the project: a cause of the page's change. */
  | { type: "openFile"; path: string }
  /** The reader stepped past the last change: which changed page is next? */
  | { type: "atEnd" }
  /** Open a changed page, and go to its first change. */
  | { type: "openPage"; path: string }
  /** A request of the overlay's (`ThreadsMethod`); answered with `threadsResult`. */
  | { type: "threads"; id: number; method: ThreadsMethod; params: Record<string, unknown> }
  /** Open the page a thread is on (a content path), and go to the thread. */
  | { type: "openThread"; threadId: string; path: string }
  /** Something the overlay told the reviewer: "Reply sent to GitHub." */
  | { type: "notify"; message: string }
  /** Sign in to GitHub in VS Code, to see the comments. */
  | { type: "signIn" }
  /** Read the comments with the GitHub CLI's sign-in. */
  | { type: "useGh" }
  /** Bring the checkout and the pull request together with VS Code's git. */
  | { type: "git"; command: "pull" | "push" | "fetch" }
  /** Read the threads from GitHub again. */
  | { type: "refreshThreads" }
  /** The overlay drew the threads (for tests): how many on blocks, detached, and unsent. */
  | { type: "threadsDrawn"; report: ThreadsReport }
  /** The author chose Page or Site, or Try again on the site's problem. */
  | { type: "surface"; surface: "page" | "site" }
  /** The site's frame loaded an address (for tests). */
  | { type: "siteShown"; url: string };

/** What the overlay drew, for tests. */
export interface ThreadsReport {
  /** The thread ids beside each block, by its anchor's source. */
  blocks: Record<string, string[]>;
  detached: string[];
  unsent: number;
}

/** What the webview found in the page it just rendered, for tests and diagnostics. */
export interface RenderReport {
  headings: string[];
  /** How many elements carry a source anchor that names the previewed file. */
  anchored: number;
  elements: Record<string, number>;
  /** Whether the element library's custom elements are defined. */
  elementsDefined: boolean;
  /** Content security policy violations seen so far. */
  violations: string[];
  /** Review's marks on the page, by kind (`added`, `changed`, `removed`, `moved`). */
  marks: Record<string, number>;
  /** The review header's text, or `null` when it isn't shown. */
  reviewHeader: string | null;
}

export interface ImageReport {
  /** The `src` the page's image ended up with. */
  src: string;
  loaded: boolean;
  width: number;
}
