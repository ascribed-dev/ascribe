// The JSON Ascribe writes that this package reads, as TypeScript types.
// Generated from the Rust types that write it, through the JSON Schemas in
// schemas/, by crates/ascribe-cli/src/shapes.rs. Change the Rust types, then
// run `ASCRIBE_BLESS=1 cargo test -p ascribe-cli shapes`. Don't edit it.
//
// - DiffReport (`ascribe diff --format json`)
// - ReportData (`ascribe diff --format html`, for the report's script)

/**
 * Where a block's text is written: the README's anchor grammar, the same
 * string the rendered page carries in `data-ascribe-source` and
 * `data-ascribe-via`.
 */
export interface Anchor {
  /**
   * `<path>:<first>-<last>`: the file's content path, percent-encoded by
   * segment, and the block's first and last lines, from 1.
   */
  source: string;
  /**
   * The includes the block came through, outermost first, each
   * `<path>:<line>`. Empty for a block written in the page itself.
   */
  via: string[];
}

/** The base of a comparison, in the report. */
export interface BaseInfo {
  /** The revision asked for, or the default branch used. */
  requested: string;
  /** The commit it names. */
  commit: string;
  /**
   * The merge base of that commit and `HEAD`, which the comparison reads;
   * `null` with `--base-exact`, which reads `commit`.
   */
  merge_base: string | null;
}

/** One build's changed pages. */
export interface BuildData {
  /** The build's name. */
  build: string;
  /** Its changed pages, in path order. */
  pages: PageData[];
}

/** What changed in one build. */
export interface BuildDiff {
  /** The build's name. */
  build: string;
  /** The pages that changed, in path order. */
  pages: PageDiff[];
}

/** One block's change. */
export interface Change {
  /** What happened to it. */
  kind: ChangeKind;
  /**
   * For a removed block, and a moved block's old place: the block of the
   * new version it came after, among its siblings. Absent when it was
   * first.
   */
  after?: Anchor;
  /** Where it's written now: every kind but `removed`. */
  now?: Anchor;
  /**
   * For a removed block, and a moved block's old place: the block of the
   * new version it was inside. Absent at the top of the page.
   */
  parent?: Anchor;
  /**
   * For a removed block, its text, whitespace collapsed, so it can be
   * shown where it was.
   */
  text?: string;
  /** Where it was written: every kind but `added`. */
  was?: Anchor;
  /** For changed prose, the words that differ. */
  words?: Words;
}

/** The kinds of block change. */
export type ChangeKind = "changed" | "added" | "removed" | "moved";

/** How many changes of each kind a page has. */
export interface Counts {
  /** Blocks whose content changed. */
  changed: number;
  /** Blocks only in the new version. */
  added: number;
  /** Blocks only in the old version. */
  removed: number;
  /** Blocks in both, somewhere else. */
  moved: number;
}

/** The whole report, as `ascribe diff --format json` writes it. */
export interface DiffReport {
  /** `SCHEMA_VERSION`. */
  schema_version: number;
  /** The version of Ascribe that wrote it. */
  ascribe_version: string;
  /** What was compared with. */
  base: BaseInfo;
  /** Where the project is. */
  repository: RepositoryInfo;
  /**
   * How many errors `ascribe check` finds in the working tree, for the
   * builds compared. The comparison runs regardless, but a page with an
   * error may not render as it will once it's fixed, so a reviewer should
   * know. `diff_project` sets it; `Report::new` leaves it zero.
   */
  working_tree_errors: number;
  /** What changed, per build, in the order asked for. */
  builds: BuildDiff[];
}

/**
 * A piece of a formatted value, as the JSON output writes it:
 * `{ "type": "text" | "code", "value": … }`. Other JSON that shows a
 * formatted title, such as the review report's, writes it the same way.
 */
export interface FormattedPiece {
  /** What the piece is. */
  type: PieceKind;
  /** Its text: a code span's content without its backticks. */
  value: string;
}

/** An image a page refers to. */
export interface ImageRef {
  /** The image's source file. */
  path: string;
  /** Its size, when it's too large to include. */
  bytes?: number;
  /** Its key in `images`, when it's included. */
  image?: string;
}

/** How many changed pages a report renders, and how many it left out. */
export interface Limit {
  /** The most it renders. */
  pages: number;
  /** How many it left out. */
  omitted: number;
}

/** One changed page: `ascribe diff --format json`'s, and its renderings. */
export interface PageData {
  /** The page's content path. */
  path: string;
  /** Its route: the new one, or for a removed page the old one. */
  route: string;
  /**
   * Whether the build publishes it only now, only before, or in both
   * with a difference.
   */
  status: PageStatus;
  /** Whether the page's own file changed (or exists on one side only). */
  own_file_changed: boolean;
  /**
   * The other changed files the page's change can come from: fragments it
   * includes and pages its links take a title or a heading from, in path
   * order; then the snippets whose code changed, by address
   * (`code:app.py#main`); and `ascribe.toml` (last) when the content model
   * is a cause.
   */
  because: string[];
  /**
   * What changed about the page itself besides its blocks, in this
   * order: `title`, `frontmatter`, `availability` (the page-level one),
   * and `route`. Empty for an added or removed page.
   */
  page_changed: string[];
  /** How many changes of each kind. */
  counts: Counts;
  /**
   * The block-level changes, in the page's order, a removed block where
   * it was. Empty for an added or removed page.
   */
  changes: Change[];
  /** Its title, when the build has one. */
  title: string | null;
  /**
   * Its title formatted, when its field sets `inline = "code"`: for the
   * page list and the page's heading. `title` stays the plain text.
   */
  formatted_title: FormattedPiece[] | null;
  /** The page now, as a key of `pages`; `null` when there's none. */
  now: string | null;
  /** The page before, as a key of `pages`; `null` when there's none. */
  was: string | null;
  /** Whether it's beyond the limit, so not rendered. */
  omitted: boolean;
  /**
   * The agent prompt about the page, which its Copy prompt button copies:
   * what `ascribe diff --format prompt` writes for it. Absent when the
   * report was written without prompts.
   */
  prompt?: string;
}

/** What changed on one page of a build. */
export interface PageDiff {
  /** The page's content path. */
  path: string;
  /** Its route: the new one, or for a removed page the old one. */
  route: string;
  /**
   * Whether the build publishes it only now, only before, or in both
   * with a difference.
   */
  status: PageStatus;
  /** Whether the page's own file changed (or exists on one side only). */
  own_file_changed: boolean;
  /**
   * The other changed files the page's change can come from: fragments it
   * includes and pages its links take a title or a heading from, in path
   * order; then the snippets whose code changed, by address
   * (`code:app.py#main`); and `ascribe.toml` (last) when the content model
   * is a cause.
   */
  because: string[];
  /**
   * What changed about the page itself besides its blocks, in this
   * order: `title`, `frontmatter`, `availability` (the page-level one),
   * and `route`. Empty for an added or removed page.
   */
  page_changed: string[];
  /** How many changes of each kind. */
  counts: Counts;
  /**
   * The block-level changes, in the page's order, a removed block where
   * it was. Empty for an added or removed page.
   */
  changes: Change[];
}

/** Whether a page is new, gone, or different. */
export type PageStatus = "added" | "removed" | "changed";

/** What a piece of a formatted value is. */
export type PieceKind = "text" | "code";

/** A page rendered: its HTML, and what each image reference in it is. */
export interface RenderedPage {
  /** The page's content, rendered with source anchors. */
  html: string;
  /** Each image reference the HTML writes, and what it is. */
  images: Record<string, ImageRef>;
}

/**
 * The data the report's script draws from. Its keys, like the JSON
 * report's, are snake_case.
 */
export interface ReportData {
  /** The version of Ascribe that wrote it. */
  ascribe_version: string;
  /** What was compared with. */
  base: BaseInfo;
  /** How many errors `ascribe check` finds in the working tree. */
  working_tree_errors: number;
  /** What changed, per build. */
  builds: BuildData[];
  /** Each rendered page, once however many builds render it alike. */
  pages: Record<string, RenderedPage>;
  /** Each image, as a `data:` URL, once however many pages use it. */
  images: Record<string, string>;
  /** How many changed pages the report renders, and how many it left out. */
  limit: Limit;
  /** The size, in bytes, above which an image isn't included. */
  image_limit: number;
}

/** The repository, in the report. */
export interface RepositoryInfo {
  /** The repository's top-level directory, as git prints it. */
  root: string;
  /**
   * The project's folder inside it, with a trailing `/`, or empty when
   * the project is at the repository's root.
   */
  project_prefix: string;
}

/**
 * The words that differ inside a changed block of prose. Ranges are
 * `[start, end)` in characters (Unicode scalar values) of each side's
 * `text`, which is the block's text with whitespace collapsed.
 */
export interface Words {
  /** Ranges in `now_text`: words added or replacing others. */
  now: [number, number][];
  /** Ranges in `was_text`: words removed or replaced. */
  was: [number, number][];
  /** The block's text now. */
  now_text: string;
  /** The block's text before. */
  was_text: string;
}
