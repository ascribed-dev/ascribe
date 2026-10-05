// The types of the `ascribe/preview` request (`crates/tessera-lsp/src/preview.rs`),
// the review requests (`crates/tessera-lsp/src/review.rs`), and of the
// messages between the extension and the preview's webview.

/** The parameters of `ascribe/preview`. */
export interface PreviewParams {
  textDocument: { uri: string };
  /** A build name; without one the server uses the editor's build. */
  build?: string;
  /** Include what changed on the page against the review base. */
  review?: boolean;
}

export interface PreviewBuild {
  name: string;
  /** Whether it's the editor's build: the picker's default. */
  editor: boolean;
  description: string;
}

export interface PreviewAsset {
  /** The reference as the page's HTML writes it, without `#fragment`. */
  reference: string;
  /** The source file, an absolute path. */
  path: string;
  kind: "image" | "link";
  /** Whether the preview may read the file: it is in the content root or an `assetRoots` directory. */
  servable: boolean;
}

export interface PreviewLink {
  href: string;
  path: string;
  id: string | null;
}

/** A heading written in the previewed file. The HTML's source anchors locate every block. */
export interface PreviewSection {
  id: string;
  /** From 0. */
  line: number;
}

export interface PreviewPage {
  path: string;
  route: string;
  title: string | null;
  frontmatter: PageFrontmatter;
  html: string;
  assets: PreviewAsset[];
  links: PreviewLink[];
  sections: PreviewSection[];
}

/** The site output's frontmatter: `available` is a list of targets. */
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

export interface PreviewProblem {
  severity: "error" | "warning" | "info";
  message: string;
}

export interface PreviewResult {
  build: string;
  builds: PreviewBuild[];
  projectRoot: string | null;
  contentRoot: string | null;
  /** Directories outside the content root that the page's assets are in. */
  assetRoots: string[];
  documentVersion: number | null;
  page: PreviewPage | null;
  problems: PreviewProblem[];
  /** With `review: true` and a base set: what changed on the page. */
  review?: PreviewReview | null;
}

/** A review base, as `ascribe diff` reports it. */
export interface BaseInfo {
  /** The revision asked for, or the default branch used. */
  requested: string;
  commit: string;
  /** The merge base of `commit` and `HEAD`, which is compared with. */
  merge_base: string | null;
}

/** How many blocks changed, by kind. */
export interface Counts {
  changed: number;
  added: number;
  removed: number;
  moved: number;
}

/** Where a block is written: the anchor grammar's `source` and `via`. */
export interface Anchor {
  source: string;
  via: string[];
}

/** One block's change, as `ascribe diff --format json` writes it (`@ascribed/review/marks` draws it). */
export interface Change {
  kind: "changed" | "added" | "removed" | "moved";
  now?: Anchor;
  was?: Anchor;
  words?: { now: [number, number][]; was: [number, number][]; now_text: string; was_text: string };
  after?: Anchor;
  parent?: Anchor;
  text?: string;
}

/** A changed page, as `ascribe diff --format json` reports it (its keys are snake_case). */
export interface PageChanges {
  path: string;
  route: string;
  status: "added" | "removed" | "changed";
  own_file_changed: boolean;
  /** The other changed files the page's change comes from, content paths; `ascribe.toml` last. */
  because: string[];
  page_changed: string[];
  counts: Counts;
  changes: Change[];
}

/** What changed on the previewed page against the review base. */
export interface PreviewReview {
  base: BaseInfo;
  /** `null` when the page didn't change. */
  changes: PageChanges | null;
  /** The page as it was, rendered with anchors; `null` for a new page or no change. */
  wasHtml: string | null;
}

/** The answer to `ascribe/review/setBase`. */
export interface SetBaseResult {
  base: BaseInfo | null;
  problem: string | null;
}

/** A changed page in the list: `PageChanges` without its `changes`, and its title. */
export type ChangedPage = Omit<PageChanges, "changes"> & { title: string | null };

/** The answer to `ascribe/review/changes`. */
export interface ChangesResult {
  build: string;
  base: BaseInfo | null;
  contentRoot: string | null;
  pages: ChangedPage[];
  problem: string | null;
}

/** What the preview shows about review, while it's on: the page's changes. */
export interface ReviewView {
  /** The base's name: "main". */
  base: string;
  /** The commit compared with, shortened: "1a2b3c4". */
  commit: string;
  /** `null` when the page didn't change. */
  page: PageChanges | null;
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
