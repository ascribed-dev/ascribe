// The JSON Ascribe writes that this package reads, as TypeScript types.
// Generated from the Rust types that write it, through the JSON Schemas in
// schemas/, by crates/tessera-cli/src/shapes.rs. Change the Rust types, then
// run `ASCRIBE_BLESS=1 cargo test -p tessera-cli shapes`. Don't edit it.
//
// - DiffReport (`ascribe diff --format json`)

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
