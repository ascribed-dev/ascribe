// The JSON Ascribe writes that this package reads, as TypeScript types.
// Generated from the Rust types that write it, through the JSON Schemas in
// schemas/, by crates/ascribe-cli/src/shapes.rs. Change the Rust types, then
// run `ASCRIBE_BLESS=1 cargo test -p ascribe-cli shapes`. Don't edit it.
//
// - PreviewResult (the language server, answering `ascribe/preview`)
// - SetBaseResult (the language server, answering `ascribe/review/setBase`)
// - ChangesResult (the language server, answering `ascribe/review/changes`)

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

/** What a page uses an asset as. */
export type AssetKind = "image" | "link";

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

/** A changed page in the list: `ascribe diff`'s page without its block changes, and its title. */
export interface ChangedPage {
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
  /** The page's title; `null` when it has none. */
  title: string | null;
}

/** The answer to `ascribe/review/changes`. */
export interface ChangesResult {
  /** The build the pages are of. */
  build: string;
  /** The base compared with; `null` when there is none. */
  base: BaseInfo | null;
  /** The content root, as a path, which the pages' paths are relative to. */
  contentRoot: string | null;
  /**
   * The changed pages, in path order: `ascribe diff`'s pages without their
   * `changes`, each with its `title`.
   */
  pages: ChangedPage[];
  /**
   * Why there are no pages to list, when that isn't because nothing
   * changed.
   */
  problem: string | null;
}

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

/** An asset the page uses. */
export interface PreviewAsset {
  /**
   * The reference as the HTML writes it, before any `#fragment`: an
   * `<img src>` for an image, an `<a href>` for a link target. Relative
   * to the page for an image, root-relative for a link target.
   * Percent-encoded as a URL is.
   */
  reference: string;
  /**
   * The source file, as an absolute path: the file the reference names,
   * resolved from the file it is written in.
   */
  path: string;
  /** What the page uses it as. */
  kind: AssetKind;
  /**
   * Whether the preview may read the file: it is in the content root, or
   * in a directory listed in `assetRoots`. A file it may not read is
   * reported in `problems` and isn't shown.
   */
  servable: boolean;
}

/** A build of the content model, for a picker. */
export interface PreviewBuild {
  /** The build's name. */
  name: string;
  /**
   * Whether it is the editor's build (`[editor] build`): the picker's
   * default.
   */
  editor: boolean;
  /**
   * What it does, in the content model's terms: `variants: switch,
   * availability: badge`.
   */
  description: string;
}

/** A link to another page. */
export interface PreviewLink {
  /**
   * The `href` as the HTML writes it: the route, and `#` and the page id
   * when it names a heading.
   */
  href: string;
  /** The target page, as an absolute path. */
  path: string;
  /** The page id of the heading the link names. */
  id: string | null;
}

/** A rendered page. */
export interface PreviewPage {
  /** The page's content path. */
  path: string;
  /** The page's route on the site. */
  route: string;
  /** The page's title (its frontmatter `title`, phrases substituted). */
  title: string | null;
  /**
   * The frontmatter the site output writes, as JSON: `available` is the
   * list of targets a layout passes to `<ascribe-availability>`.
   */
  frontmatter: Record<string, unknown>;
  /**
   * The page's content as HTML: the site markdown, with source anchors,
   * after `render_site_html`, without its frontmatter and without a
   * layout.
   */
  html: string;
  /** Every asset the page uses. */
  assets: PreviewAsset[];
  /** The page links in the content, so a click opens the file. */
  links: PreviewLink[];
  /**
   * The headings written in the previewed file itself, in order. The
   * HTML's source anchors locate every block, headings included.
   */
  sections: PreviewSection[];
}

/** A problem with showing a page. */
export interface PreviewProblem {
  /** How much it matters. */
  severity: ProblemSeverity;
  /** What the author should read. */
  message: string;
}

/**
 * The answer to `ascribe/preview`. It always says which builds exist and
 * where the project's files are, so a client can offer the picker and know
 * which directories the preview may read; `page` is there when there is
 * something to show, and `problems` says why not, or what is missing.
 */
export interface PreviewResult {
  /** The build the answer is for; empty when there is no project. */
  build: string;
  /** Every build of the content model, in the order it declares them. */
  builds: PreviewBuild[];
  /** The project's `ascribe.toml` directory, as a path. */
  projectRoot: string | null;
  /** The content root, as a path. */
  contentRoot: string | null;
  /**
   * The directories outside the content root that the page's assets are
   * in and the preview may read: the directory of each asset that
   * is in the project but not in the content root, and nowhere else. Never
   * the project root, `node_modules`, or the output directory.
   */
  assetRoots: string[];
  /**
   * The version of the open document the answer was computed from, or
   * `null` when the file isn't open (its text is the disk's). A client
   * that sent version *n* and gets an older one has raced its own edit
   * and asks again.
   */
  documentVersion: number | null;
  /** The rendered page. */
  page: PreviewPage | null;
  /**
   * Problems with showing the page: why there is none, or what in it
   * can't be shown.
   */
  problems: PreviewProblem[];
  /**
   * With `review: true`, what changed on the page against the review
   * base; `null` when there's no base or no page, or review wasn't asked
   * for.
   */
  review: PreviewReview | null;
}

/** What changed on the previewed page against the review base. */
export interface PreviewReview {
  /** The base compared with. */
  base: BaseInfo;
  /**
   * The page's changes, as `ascribe diff --format json` reports a page
   * (its keys are snake_case); `null` when the page didn't change.
   */
  changes: PageDiff | null;
  /**
   * The page as it was at the base, rendered as `html` is, with source
   * anchors, for showing removed blocks and changed blocks as they were;
   * `null` when the page didn't change or is new.
   */
  wasHtml: string | null;
}

/** A heading written in the previewed file. */
export interface PreviewSection {
  /** The heading's id on the page (its `id` in the HTML). */
  id: string;
  /** The line it starts on, from 0. */
  line: number;
}

/** How much a problem with showing a page matters. */
export type ProblemSeverity = "error" | "warning" | "info";

/** The answer to `ascribe/review/setBase`. */
export interface SetBaseResult {
  /**
   * What the base resolved to; `null` once it's dropped, or when it
   * couldn't be set.
   */
  base: BaseInfo | null;
  /**
   * Why the base couldn't be set: no project, not a repository, an
   * unknown revision, `git` missing. The base set before, if any, stays.
   */
  problem: string | null;
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
