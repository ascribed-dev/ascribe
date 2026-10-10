// The JSON Ascribe writes that this package reads, as TypeScript types.
// Generated from the Rust types that write it, through the JSON Schemas in
// schemas/, by crates/ascribe-cli/src/shapes.rs. Change the Rust types, then
// run `ASCRIBE_BLESS=1 cargo test -p ascribe-cli shapes`. Don't edit it.
//
// - CheckReport (`ascribe check --format json` and `ascribe build --format json`)
// - PreviewResult (the language server, answering `ascribe/preview`)
// - SetBaseResult (the language server, answering `ascribe/review/setBase`)
// - ChangesResult (the language server, answering `ascribe/review/changes`)
// - ContextResult (the language server, answering `ascribe/context`)
// - TargetsResult (the language server, answering `ascribe/targets`)
// - InventoryResult (the language server, answering `ascribe/inventory`)
// - EditResult (the language server, answering `ascribe/edit`)
// - BuildViewResult (the language server, answering `ascribe/buildView`)
// - AgentPromptResult (the language server, answering `ascribe/agentPrompt`)

/** A problem acknowledged as intended. */
export interface AcknowledgedEntry {
  /** The code of the check that found it, such as `ASC036`. */
  code: string;
  /** The check's name. */
  slug: string;
  /** What the check found. */
  message: string;
  /** The file, as a diagnostic's `file` is. */
  file: string;
  /** Where in the file. */
  range: Range;
  /** The builds it appears in, as a diagnostic's `builds` are. */
  builds: string[];
  /** Why it's intended: the acknowledgement's reason. */
  reason: string;
  /** Where the acknowledgement is written. */
  at: Place;
}

/**
 * The answer to `ascribe/agentPrompt`, or `null` when there's nothing to
 * prompt about: the file has no problem, the diagnostic is no longer
 * reported, review is off, or the page or fragment didn't change.
 */
export interface AgentPromptResult {
  /**
   * The prompt: plain text with Markdown, short enough for an agent's
   * link to carry.
   */
  prompt: string;
}

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

/** An attribute's type. */
export type AttributeKind = "string" | "number" | "boolean" | "enum" | "set" | "noteType";

/** An attribute, as written. */
export interface AttributePair {
  /** The key. */
  key: string;
  /**
   * The value, without quotes (a value set's members joined by `|`);
   * `null` for a key with no value.
   */
  value: string | null;
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

/** What a line-form directive applies to. */
export type BindingKind = "self" | "heading" | "block" | "headingOrBlock";

/**
 * The answer to `ascribe/buildView`. A document that isn't a source file of
 * the project, or a build the content model doesn't have, gets an answer
 * with an empty `build` and nothing left out.
 */
export interface BuildViewResult {
  /** The build the answer is for; empty when there is none. */
  build: string;
  /**
   * The version of the open document the answer was computed from, or
   * `null` when the file isn't open.
   */
  documentVersion: number | null;
  /**
   * Whether the build publishes the page. `false` when it drops the whole
   * page (its `variant` or `available` frontmatter); a fragment, which is
   * part of the pages that include it, counts as published.
   */
  pageIncluded: boolean;
  /** Why the build doesn't publish the page, when it doesn't. */
  pageDetail: string | null;
  /** What the build leaves out of the page's own text, in document order. */
  excluded: Excluded[];
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
  /** The title formatted, when its field sets `inline = "code"`; `null` when it doesn't. */
  formatted_title: FormattedPiece[] | null;
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
   * `changes`, each with its `title` and `formatted_title`.
   */
  pages: ChangedPage[];
  /**
   * Why there are no pages to list, when that isn't because nothing
   * changed.
   */
  problem: string | null;
}

/**
 * What `ascribe check --format json` and `ascribe build --format json`
 * write: one document, whatever the outcome. Fields can be added without a
 * new `schema_version`, so a reader ignores fields it doesn't know.
 */
export interface CheckReport {
  /**
   * The version of this schema. It changes only when a field is removed
   * or changes meaning.
   */
  schema_version: number;
  /** The version of Ascribe that wrote it. */
  ascribe_version: string;
  /**
   * Why the project couldn't be checked (exit code 2), or `null`. When it
   * isn't `null`, `diagnostics` holds what was found first: the content
   * model's problems.
   */
  error: string | null;
  /** How many source files were checked: the project's. */
  files_checked: number;
  /**
   * How many of them the report covers: the source files in the paths
   * named, or every one when no path was.
   */
  files_reported: number;
  /**
   * The builds whose page-level checks ran, in `ascribe.toml`'s order:
   * every build, the ones named with `--build`, or the editor's with
   * `--editor-build`. Empty when the project couldn't be checked.
   */
  builds_checked: string[];
  /**
   * Every diagnostic, in file order, and in source order within a file;
   * advice after the errors and warnings, in the same order.
   * With paths, only those that count for them. With `--summary`, none:
   * see `truncated`.
   */
  diagnostics: Entry[];
  /**
   * Whether `diagnostics` leaves some out. It does with `--summary`,
   * which lists none.
   */
  truncated: boolean;
  /** How many diagnostics `diagnostics` lists. */
  shown: number;
  /** How many diagnostics there are. */
  total: number;
  /**
   * The command that lists the ones left out, when `truncated`; `null`
   * otherwise.
   */
  next_command: string | null;
  /** How many errors, warnings, and advice. */
  summary: Summary;
  /**
   * The problems acknowledged as intended, which `diagnostics` leaves
   * out and which don't fail the command, in file order. Left out when
   * there are none, and with `--summary`.
   */
  acknowledged?: AcknowledgedEntry[];
}

/** How many diagnostics have one code. */
export interface CodeCount {
  /** The code, such as `ASC036`. */
  code: string;
  /** The diagnostic's name, such as `link-target-missing`. */
  slug: string;
  /** `error`, `warning`, or `advice`. */
  severity: string;
  /** How many. */
  count: number;
}

/**
 * Something that contains a position: a block, a construct that groups
 * blocks, or an inline node. Every range is in the negotiated position
 * encoding.
 */
export type ContextNode =
  | {
      kind: "frontmatter";
      /** The frontmatter, delimiters included. */
      range: LspRange;
      /** Its `variant` key's value, when it has one. */
      variant: FrontmatterValue | null;
      /** Its `available` key's value, when it has one. */
      available: FrontmatterValue | null;
    }
  | {
      kind: "section";
      /** The section. */
      range: LspRange;
      /** The id of its heading; empty when the heading has none. */
      headingId: string;
    }
  | {
      kind: "heading";
      /** The heading. */
      range: LspRange;
      /** 1 to 6. */
      level: number;
      /** Its id: its `@id`, or its slug; empty when it has none. */
      id: string;
      /** Whether the id is an `@id`. */
      explicitId: boolean;
    }
  | {
      kind: "paragraph";
      /** The paragraph. */
      range: LspRange;
    }
  | {
      kind: "list";
      /** The list. */
      range: LspRange;
      /** Whether it's ordered. */
      ordered: boolean;
      /** Whether a `@steps` binds it. */
      steps: boolean;
    }
  | {
      kind: "listItem";
      /** The item, from its marker. */
      range: LspRange;
    }
  | {
      kind: "blockQuote";
      /** The block quote. */
      range: LspRange;
    }
  | {
      kind: "note";
      /**
       * The note: its line, the block it binds, or the container, with
       * its title line.
       */
      range: LspRange;
      /** Its type (`note` when it gives none). */
      type: string;
      /** Its form. */
      form: Form;
    }
  | {
      kind: "details";
      /**
       * The details: the block it binds, or the container, with its title
       * line.
       */
      range: LspRange;
      /** Its title's text, as written. */
      title: string | null;
      /** Its form. */
      form: Form;
    }
  | {
      kind: "steps";
      /** From the `@steps` through the list. */
      range: LspRange;
    }
  | {
      kind: "variantGroup";
      /** The group, through its `@end`. */
      range: LspRange;
      /** The dimension every arm names; `null` for labeled arms. */
      dimension: string | null;
      /** The arms, in order. */
      arms: VariantArm[];
      /**
       * The index of the arm the position is in; `null` on the group's
       * `@end`.
       */
      arm: number | null;
    }
  | {
      kind: "availability";
      /** The line, or the line and the block it binds. */
      range: LspRange;
      /** The spec or feature key, as written. */
      spec: string;
    }
  | {
      kind: "include";
      /** The directive line. */
      range: LspRange;
      /** The path as written, without the `#id`. */
      path: string;
      /** The id after `#`, when it includes one section. */
      section: string | null;
    }
  | {
      kind: "snippet";
      /** The directive line. */
      range: LspRange;
      /** Its address as written: `<source>:<path>#<region>`. */
      address: string;
    }
  | {
      kind: "widget";
      /**
       * The widget: its line, the block it binds, its container, or its
       * group of arms.
       */
      range: LspRange;
      /** Its name. */
      name: string;
      /**
       * Its attributes, as written: of the group's arm the position is in,
       * for a group.
       */
      attributes: AttributePair[];
      /** Its form. */
      form: Form;
    }
  | {
      kind: "codeBlock";
      /** The block, fences included. */
      range: LspRange;
      /** The fence's info string; empty when it has none. */
      info: string;
      /** Whether it's fenced. */
      fenced: boolean;
    }
  | {
      kind: "table";
      /** The table. */
      range: LspRange;
    }
  | {
      kind: "tableRow";
      /** The row. */
      range: LspRange;
      /** Whether it's the header row. */
      header: boolean;
      /** Its `available` attribute's value, when it has one. */
      available: string | null;
    }
  | {
      kind: "link";
      /** The link, brackets included. */
      range: LspRange;
      /** Its destination, escapes decoded. */
      destination: string;
      /** Whether it has no text, so it takes its target's title. */
      textEmpty: boolean;
    }
  | {
      kind: "image";
      /** The image, its attribute block included. */
      range: LspRange;
      /** Its source, escapes decoded. */
      src: string;
      /** Its alt text, as written. */
      alt: string;
      /** Its attributes (`{width=600}`), as written. */
      attributes: AttributePair[];
    }
  | {
      kind: "phrase";
      /** The candidate, braces included. */
      range: LspRange;
      /** The key. */
      key: string;
      /** Whether the content model declares it. */
      declared: boolean;
    };

/** The project a page is in. */
export interface ContextProject {
  /** The directory of its `ascribe.toml`, as a path. */
  root: string;
  /** The editor's build (`[editor] build`), which decides the diagnostics. */
  editorBuild: string;
}

/**
 * The answer to `ascribe/context`. A document that isn't a source file of
 * the project gets an empty answer: no project, nothing at the range.
 */
export interface ContextResult {
  /** The project the page is in; `null` when it's in none. */
  project: ContextProject | null;
  /** What contains the start of the range, innermost first. */
  at: ContextNode[];
  /** What the selection is; `null` for an empty range. */
  selection: Selection | null;
  /** The token under the start of the range, if there is one. */
  token: ContextToken | null;
  /**
   * Whether the line at the start of the range is blank and between
   * blocks, where a block can be inserted: not in a code block or the
   * frontmatter.
   */
  insertable: boolean;
}

/** The token under a position. */
export type ContextToken =
  | {
      kind: "link";
      /** The link, brackets included. */
      range: LspRange;
      /** Its destination, escapes decoded. */
      destination: string;
      /** Whether it has no text. */
      textEmpty: boolean;
    }
  | {
      kind: "image";
      /** The image, its attribute block included. */
      range: LspRange;
      /** Its source, escapes decoded. */
      src: string;
    }
  | {
      kind: "include";
      /** The path, and the `#id` if it has one. */
      range: LspRange;
      /** The path as written, without the `#id`. */
      path: string;
      /** The id after `#`. */
      section: string | null;
    }
  | {
      kind: "phrase";
      /** The candidate, braces included. */
      range: LspRange;
      /** The key. */
      key: string;
      /** Whether the content model declares it. */
      declared: boolean;
    }
  | {
      kind: "directiveName";
      /** The `@` and the name. */
      range: LspRange;
      /** The name, without `@`. */
      name: string;
    }
  | {
      kind: "attribute";
      /** From the key through the value. */
      range: LspRange;
      /** The directive's name. */
      directive: string;
      /** The key. */
      key: string;
      /** The value, without quotes; `null` for a key with no value. */
      value: string | null;
    };

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

/** One edit of a fix. */
export interface Edit {
  /** The text it replaces. */
  range: Range;
  /** The text that replaces it. */
  new_text: string;
}

/** The answer to `ascribe/edit`: the edit, or why there is none. */
export type EditResult =
  | {
      /**
       * Plain text edits under `changes`, in canonical form: to the
       * requested document, and, for an action on the content model, to
       * `ascribe.toml` and any other pages it changes.
       */
      edit: LspWorkspaceEdit;
      /**
       * The placeholder text the edit wrote, in the document as it is
       * after the edit, for the client to leave selected; `null` when it
       * wrote none.
       */
      select: LspRange | null;
    }
  | {
      /** The message. */
      error: string;
    };

/** A diagnostic. */
export interface Entry {
  /** The code, such as `ASC036`. */
  code: string;
  /** The diagnostic's name, such as `link-target-missing`. */
  slug: string;
  /**
   * `error`, `warning`, or `advice`. Advice never fails the command.
   * More severities may be added; a reader treats one it doesn't know as
   * it treats advice.
   */
  severity: string;
  /**
   * The kind of next step: `fix` when Ascribe can make the edit,
   * `choose` when the author picks among things Ascribe can list,
   * `write` when it needs writing or judgment, `outside` when nothing in
   * the source can fix it, and `review` when it may be fine as it is.
   */
  next: string;
  /** What's wrong, and what to do about it. */
  message: string;
  /**
   * The file, relative to the project root (the directory of
   * `ascribe.toml`), with `/` separators. `ascribe.toml` for a
   * content-model problem.
   */
  file: string;
  /** Where in the file. */
  range: Range;
  /** Other places that explain it. */
  related: Related[];
  /** Edits that would fix it. */
  fixes: Fix[];
  /**
   * The builds a page-level diagnostic appears in, in `ascribe.toml`'s
   * order. Empty for a file-level diagnostic, and for one in content no
   * build publishes. With `--build`, only that build.
   */
  builds: string[];
  /** Whether it's in content that no build publishes. */
  unpublished: boolean;
  /**
   * How to fix it, in general: the diagnostics reference's advice for its
   * code.
   */
  help: string;
  /** The address of its entry in the diagnostics reference. */
  docs: string;
  /**
   * For a problem in included content reported because one of its related
   * places is in a path named: at how many other includes it's reported
   * too, collapsed into this one. `0` otherwise.
   */
  repeats: number;
  /**
   * The rule of the program that found it, such as `Ascribe.Repeated`
   * from Vale, for a `prose` diagnostic. Absent for Ascribe's own.
   */
  rule?: string;
}

/** Text a build leaves out of a page. */
export interface Excluded {
  /**
   * What's left out: whole blocks, arms, or table rows, from the first
   * directive line through the last line. Neighbors left out for the same
   * reason are one range.
   */
  range: LspRange;
  /** Why. */
  reason: ExclusionReason;
  /**
   * The reason, for the author: `Shows only edition=self-hosted`, or
   * `Scheduled rollouts: available on Lantern Cloud (preview), not
   * Self-hosted 2.5`, with the content model's display labels.
   */
  detail: string;
}

/** Why a build leaves text out of a page. */
export type ExclusionReason = "variant" | "availability";

/** How many diagnostics are in one file. */
export interface FileCount {
  /** The file, as a diagnostic's `file` is. */
  file: string;
  /** How many errors. */
  errors: number;
  /** How many warnings. */
  warnings: number;
  /** How many advice. */
  advice: number;
}

/** Edits that would fix a diagnostic. */
export interface Fix {
  /** What the fix does. */
  title: string;
  /** The file the edits are in, as a diagnostic's `file` is. */
  file: string;
  /** The edits, each replacing the text of its range. */
  edits: Edit[];
  /**
   * `safe` when applying the edits as they are can't change what the page
   * says and leaves nothing to decide; `unsafe` otherwise.
   */
  applicability: string;
}

/** How a directive is written (SPEC §3.5, §3.6). */
export type Form = "line" | "block" | "container" | "group";

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

/** A frontmatter key's value. */
export interface FrontmatterValue {
  /** The value, after the key's colon, through its last line. */
  range: LspRange;
  /** The value's YAML, as written; a one-line value without its quotes. */
  value: string;
}

/** An entry of the content model. */
export interface InventoryEntry {
  /** What kind of entry it is. */
  kind: ModelKind;
  /** Its key, id, or name, as pages write it. */
  key: string;
  /** Its label, name, or term, where it has one besides its key. */
  label: string | null;
  /**
   * How many places use it; `null` for a build, which pages don't name,
   * and for a glossary term with `match = "marked"`, whose uses are links
   * to its page.
   */
  uses: number | null;
  /**
   * Where `ascribe.toml` declares it; `null` for a built-in note type,
   * and for an entry that can't be found there.
   */
  declaration: LspRange | null;
}

/** A fragment. */
export interface InventoryFragment {
  /** Its content path. */
  path: string;
  /** The content paths of the files that include it, in order. */
  includedBy: string[];
}

/** A page. */
export interface InventoryPage {
  /** Its content path. */
  path: string;
  /** Its title (frontmatter `title`). */
  title: string | null;
  /** Its content type; `null` when no one type applies. */
  type: string | null;
  /** How many links from other files, and includes, name it. */
  incoming: number;
}

/**
 * The answer to `ascribe/inventory`. A document that isn't one of the
 * project's files gets empty lists.
 */
export interface InventoryResult {
  /** The pages, by content path. */
  pages: InventoryPage[];
  /** The fragments, by content path. */
  fragments: InventoryFragment[];
  /**
   * The content paths of the pages no other file links to or includes,
   * other than index pages (`index.md`, in any folder). Without a
   * navigation file a reader may still reach them, so this is a hint.
   */
  orphans: string[];
  /**
   * The content model's entries: phrases, features, glossary terms,
   * dimensions, note types, widgets, then builds, each kind in
   * declaration order.
   */
  model: InventoryEntry[];
  /**
   * The `file:` URI of the content root, which content paths are
   * relative to.
   */
  contentUri?: string;
  /**
   * The `file:` URI of the project's `ascribe.toml`, which the ranges of
   * declarations are in.
   */
  modelUri?: string;
}

/**
 * A position in a document: a zero-based line, and a zero-based column in
 * the position encoding the client and server agreed on.
 */
export interface LspPosition {
  /** The line, from 0. */
  line: number;
  /** The column, from 0, in the negotiated position encoding. */
  character: number;
}

/** A range in a document, from `start` up to (not including) `end`. */
export interface LspRange {
  /** Where it starts. */
  start: LspPosition;
  /** Where it ends. */
  end: LspPosition;
}

/** A text edit: replace `range` with `newText`. */
export interface LspTextEdit {
  /** The range to replace, in the document as it is before the edit. */
  range: LspRange;
  /** The text that replaces it. */
  newText: string;
}

/** Changes to documents. */
export interface LspWorkspaceEdit {
  /** The edits to each document, by its URI. */
  changes: Record<string, LspTextEdit[]>;
}

/** A kind of content model entry. */
export type ModelKind = "phrase" | "feature" | "term" | "dimension" | "note" | "widget" | "build";

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
   * it was. Empty for an added or removed page, and for every page when
   * the report's `blocks_omitted` is `true`.
   */
  changes: Change[];
}

/** Whether a page is new, gone, or different. */
export type PageStatus = "added" | "removed" | "changed";

/** What a piece of a formatted value is. */
export type PieceKind = "text" | "code";

/** A place in a file. */
export interface Place {
  /** The file, as a diagnostic's `file` is. */
  file: string;
  /** Where in the file. */
  range: Range;
}

/** A position in a file. */
export interface Pos {
  /** The line, from 1. */
  line: number;
  /**
   * The column, from 1, in Unicode characters (not bytes or UTF-16
   * units).
   */
  column: number;
  /** The byte offset from the start of the file. */
  offset: number;
}

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
   * The title formatted, when its field sets `inline = "code"`: for the
   * page's heading. `title` stays the plain text.
   */
  formattedTitle: FormattedPiece[] | null;
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

/** What a directive takes after its colon. */
export type PrimaryKind = "none" | "identifier" | "text" | "availability";

/** How much a problem with showing a page matters. */
export type ProblemSeverity = "error" | "warning" | "info";

/**
 * A span of a file. `end` is just past its last character; an empty range
 * (an insertion) has equal positions.
 */
export interface Range {
  /** Its first character. */
  start: Pos;
  /** Just past its last character. */
  end: Pos;
}

/** Another place that explains a diagnostic. */
export interface Related {
  /** The file, as a diagnostic's `file` is. */
  file: string;
  /** Where in the file. */
  range: Range;
  /** What it has to do with the diagnostic. */
  message: string;
}

/** What a selection is. */
export interface Selection {
  /** Its kind. */
  kind: SelectionKind;
  /** The selected text. */
  text: string;
  /** Whether it's inside one paragraph's or heading's text. */
  inline: boolean;
}

/** The kinds of selection. Whitespace at either end doesn't count. */
export type SelectionKind = "prose" | "blocks" | "code" | "mixed" | "other";

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

/** How many diagnostics of each severity. */
export interface Summary {
  /** How many errors. */
  errors: number;
  /** How many warnings. */
  warnings: number;
  /** How many advice. */
  advice: number;
  /**
   * How many problems are acknowledged as intended. Left out when there
   * are none.
   */
  acknowledged?: number;
  /** With `--summary`: how many diagnostics have each code, most first. */
  by_code?: CodeCount[];
  /** With `--summary`: how many diagnostics are in each file, most first. */
  by_file?: FileCount[];
}

/** An attribute a widget accepts. */
export interface TargetAttribute {
  /** The key. */
  key: string;
  /** The value's type. */
  type: AttributeKind;
  /**
   * The values it allows, for `enum` and a `set` of named values; empty
   * otherwise.
   */
  values: string[];
  /** Whether every use must give it. */
  required: boolean;
  /**
   * The value used when it's left out, as written (a set's members joined
   * by `|`).
   */
  default: string | null;
  /** What it's for. */
  description: string | null;
}

/** A build. */
export interface TargetBuild {
  /** Its name. */
  name: string;
  /** Whether it's the editor's build (`[editor] build`). */
  editor: boolean;
}

/** A dimension. */
export interface TargetDimension {
  /** Its name. */
  name: string;
  /** Its label. */
  label: string;
  /** Its values, in display order. */
  values: TargetDimensionValue[];
}

/** A value of a dimension. */
export interface TargetDimensionValue {
  /** The value. */
  value: string;
  /** Its label. */
  label: string;
  /** Whether it's versionless. */
  versionless: boolean;
  /**
   * The value in its dimension's `values` in `ascribe.toml`, inside its
   * quotes; `null` when it can't be found there.
   */
  range: LspRange | null;
}

/** A feature. */
export interface TargetFeature {
  /** Its key. */
  key: string;
  /** Its name. */
  name: string;
  /** Its availability spec, as written. */
  availability: string;
  /**
   * Its `[features.<key>]` table header in `ascribe.toml`; `null` when it
   * can't be found there.
   */
  range: LspRange | null;
}

/** A fragment. */
export interface TargetFragment {
  /** Its content path. */
  path: string;
  /** The path an `@include` on the requesting page writes for it. */
  include: string;
  /** Whether its first block is a heading. */
  startsWithHeading: boolean;
}

/** A heading a link can name. */
export interface TargetHeading {
  /** The content path of the page a link names it on. */
  page: string;
  /** The heading's text, phrases replaced by their values. */
  text: string;
  /** Its id. */
  id: string;
  /** 1 to 6. */
  level: number;
  /**
   * The destination of a link to it from the requesting page: `page.md#id`,
   * or `#id` on the requesting page itself.
   */
  link: string;
  /**
   * The destination of a link to it from the content root,
   * `/page.md#id`, which any page of the project can use.
   */
  rootLink: string;
}

/** An image file. */
export interface TargetImage {
  /** Its path from the project root. */
  path: string;
  /** The source of an image of it on the requesting page. */
  link: string;
}

/** A note type. */
export interface TargetNote {
  /** The type, as `@note {type=…}` writes it. */
  type: string;
  /** Its label. */
  label: string;
}

/** Where some text occurs. */
export interface TargetOccurrence {
  /** The file's content path. */
  path: string;
  /** The text, in that file. */
  range: LspRange;
}

/** A page. */
export interface TargetPage {
  /** Its content path. */
  path: string;
  /** Its title (frontmatter `title`). */
  title: string | null;
  /** Its content type; `null` when no one type applies. */
  type: string | null;
  /** The destination of a link to it from the requesting page. */
  link: string;
  /**
   * The destination of a link to it from the content root, `/page.md`,
   * which any page of the project can use.
   */
  rootLink: string;
}

/** A phrase. */
export interface TargetPhrase {
  /** The key. */
  key: string;
  /** The value. */
  value: string;
  /** Its key in `ascribe.toml`; `null` when it can't be found there. */
  range: LspRange | null;
}

/** A region of a source file. */
export interface TargetRegion {
  /** Its name. */
  name: string;
  /** The address a `@snippet` writes for it. */
  address: string;
}

/** A source of the content model and its files. */
export interface TargetSource {
  /** The source's name. */
  name: string;
  /** The files a snippet can take code from, by path. */
  files: TargetSourceFile[];
}

/** A file a snippet can take code from. */
export interface TargetSourceFile {
  /** Its path relative to the source's folder. */
  path: string;
  /** The address a `@snippet` writes for the whole file. */
  address: string;
  /** Its regions, in the order they start. */
  regions: TargetRegion[];
}

/** A project widget. */
export interface TargetWidget {
  /** Its name. */
  name: string;
  /** What it's for, as the content model describes it. */
  description: string | null;
  /** Whether it may be one line. */
  line: boolean;
  /** Whether it may be a container. */
  container: boolean;
  /** Whether its openers form groups of arms. */
  groupable: boolean;
  /** What its line form takes after the colon. */
  primary: PrimaryKind;
  /** What its line form applies to; `null` when it has no line form. */
  binding: BindingKind | null;
  /** Its attributes, in canonical order. */
  attributes: TargetAttribute[];
}

/**
 * The answer to `ascribe/targets`: a list for each kind asked for, and no
 * others. A document that isn't one of the project's files gets no lists.
 */
export interface TargetsResult {
  /** Builds, in declaration order. */
  builds?: TargetBuild[];
  /** Dimensions, in declaration order. */
  dimensions?: TargetDimension[];
  /** Features, in declaration order. */
  features?: TargetFeature[];
  /** Fragments, by content path. */
  fragments?: TargetFragment[];
  /**
   * The headings a link can name, page by page in content path order,
   * each page's in document order: a page's own, and those of the
   * fragments it includes.
   */
  headings?: TargetHeading[];
  /** The image files under the content root, by path. */
  images?: TargetImage[];
  /**
   * The `file:` URI of the project's `ascribe.toml`, which the ranges of
   * declarations are in.
   */
  modelUri?: string;
  /** Note types: the built-ins, then the declared ones. */
  notes?: TargetNote[];
  /**
   * The other whole-word occurrences, in the project's prose, of the text
   * the range selects, by content path: those making the selection a
   * phrase everywhere (`makePhrase` with `everywhere`) replaces. Empty
   * when the selection isn't text a phrase can take the place of.
   */
  occurrences?: TargetOccurrence[];
  /** Pages, by content path. */
  pages?: TargetPage[];
  /** Phrases, in declaration order. */
  phrases?: TargetPhrase[];
  /**
   * The content model's sources, in the order it declares them, with the
   * files a snippet can take code from.
   */
  snippets?: TargetSource[];
  /** Project widgets, in declaration order. */
  widgets?: TargetWidget[];
}

/** An arm of a `@variant` group. */
export interface VariantArm {
  /**
   * The arm's values of the group's dimension, as written
   * (`cloud|self-managed`); `null` for a labeled arm.
   */
  value: string | null;
  /** The arm's title, for a labeled arm. */
  label: string | null;
  /** The arm, from its title line or opener through its last block. */
  range: LspRange;
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
