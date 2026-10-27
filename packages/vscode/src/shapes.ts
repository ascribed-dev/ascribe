// The JSON Ascribe writes that this package reads, as TypeScript types.
// Generated from the Rust types that write it, through the JSON Schemas in
// schemas/, by crates/ascribe-cli/src/shapes.rs. Change the Rust types, then
// run `ASCRIBE_BLESS=1 cargo test -p ascribe-cli shapes`. Don't edit it.
//
// - CheckReport (`ascribe check --format json` and `ascribe build --format json`)
// - PreviewResult (the language server, answering `ascribe/preview`)
// - SetBaseResult (the language server, answering `ascribe/review/setBase`)
// - ChangesResult (the language server, answering `ascribe/review/changes`)
// - BaseFileResult (the language server, answering `ascribe/review/baseFile`)
// - ContextResult (the language server, answering `ascribe/context`)
// - TargetsResult (the language server, answering `ascribe/targets`)
// - InventoryResult (the language server, answering `ascribe/inventory`)
// - PageResult (the language server, answering `ascribe/page`)
// - EditResult (the language server, answering `ascribe/edit`)
// - ProjectEditResult (the language server, answering `ascribe/projectEdit`)
// - SettingsResult (the language server, answering `ascribe/settings`)
// - BuildViewResult (the language server, answering `ascribe/buildView`)
// - SearchResult (the language server, answering `ascribe/search`)
// - AgentPromptResult (the language server, answering `ascribe/agentPrompt`)
// - AddressesParams (the language server, sending `ascribe/addresses`)
// - ModelReport (`ascribe model --format json`)

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
 * The parameters of `ascribe/addresses`: what the last comparison found.
 * Sent once the project loads and after each change that compares again,
 * when it, or a page's old addresses (This page's "Also answers at"),
 * differ from what was sent last.
 */
export interface AddressesParams {
  /**
   * `ascribe.addresses`' `file:` URI; `null` when the project has no
   * file, and nothing is compared.
   */
  uri: string | null;
  /**
   * The base compared with; `null` when there's no file, it doesn't
   * read, or the base couldn't be read.
   */
  base: BaseInfo | null;
  /**
   * How many `address-lost` warnings there are: one for each page whose
   * addresses are lost.
   */
  lost: number;
  /**
   * The problems, all at `ascribe.addresses`, as `ascribe check`
   * reports them: the protocol's diagnostics, each with the data a
   * published diagnostic has, its fixes among it.
   */
  diagnostics: unknown[];
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

/** The answer to `ascribe/review/baseFile`. */
export interface BaseFileResult {
  /**
   * The file's text at the base; `null` when the base doesn't have it, or
   * there's no base.
   */
  text: string | null;
  /**
   * Why there's no text, when that isn't because the base doesn't have
   * the file: review is off, or the file isn't the project's.
   */
  problem: string | null;
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

/** A live line of a versioned build. */
export interface BuildLineEntry {
  /**
   * The build that is the line: the versioned build's own name for the
   * current line, `<build>@<line>` for another.
   */
  name: string;
  /** The line, as `[releases.<target>] live` writes it. */
  line: string;
  /** Whether it's the current line. */
  current: boolean;
}

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
   * (`code:app.py#main`); then the operations of a declared API whose spec
   * changed what the page writes for them, an `@operation`'s method, path,
   * summary, or deprecation, or a link's address, each by its spec's path
   * from the project root, `#`, and the fragment the page names it by
   * (`api/openapi.yaml#createWidget`); and `ascribe.toml` (last) when the
   * content model is a cause. The kinds are told apart by their form: a
   * content file's path ends in `.md`, a snippet's address has its
   * source's name and `:` before its path, and an operation's has a `#`
   * and neither.
   */
  because: string[];
  /**
   * What changed about the page itself besides its blocks, in this
   * order: `title`, `frontmatter`, `availability` (the page-level one),
   * and `route`. Empty for an added or removed page. A moved page's
   * route moves with it, so it isn't listed for one.
   */
  page_changed: string[];
  /** How many changes of each kind. */
  counts: Counts;
  /** The page's title; `null` when it has none. */
  title: string | null;
  /** The title formatted, when its field sets `inline = "code"`; `null` when it doesn't. */
  formatted_title: FormattedPiece[] | null;
  /**
   * For a page that moved, the content path it was at: git sees its
   * file renamed, or `ascribe.addresses` records the move. The page is
   * listed once, at `path`, compared with the page as it was at
   * `moved_from`; `status` is `changed`, and its `counts` and `changes`
   * are what differs besides the move, none for a page that only moved.
   * Absent for a page that didn't move.
   */
  moved_from?: string;
  /**
   * The pages among `because` whose only change is that they moved, each
   * by where it was and where it is: a link on this page leads to its new
   * address. Absent when there's none.
   */
  moved_links?: MovedPage[];
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

/** One configurable check, as `[checks]` sets it. */
export interface CheckEntry {
  /** Its slug, which `[checks]` names it by: `page-size`. */
  slug: string;
  /** Its code: `ASC149`. */
  code: string;
  /** Its name for people: the slug in words, `Page size`. */
  name: string;
  /** The area the diagnostics reference lists it under: `Pages`. */
  area: string;
  /**
   * What it reports: the registry's `when`, as plain text and code spans,
   * as a setting's description is.
   */
  when: SettingPiece[];
  /** The levels `[checks]` takes, in the order a message lists them. */
  choices: string[];
  /**
   * Its own level, which it has when the file sets none: `advice`,
   * `warning`, or `error`.
   */
  default: string;
  /** Whether the file names the check in `[checks]` at all. */
  named: boolean;
  /**
   * The level the file sets; `null` when it sets none, or writes one that
   * isn't a string.
   */
  level: string | null;
  /** Where the file sets its level; `null` when it doesn't. */
  set: SettingPlace | null;
  /** The server's problems on the lines that set its level. */
  problems: SettingProblem[];
  /**
   * Its `limit`, for a check that takes one: `page-size`, `image-large`,
   * `memo-long`, and `summary-long`.
   */
  limit: CheckLimit | null;
}

/** A check's `limit`. */
export interface CheckLimit {
  /**
   * `number`, a whole number above 0; or `size`, a number of bytes or a
   * string with a unit, `"500 KB"`.
   */
  kind: string;
  /** The limit used when the file sets none. */
  default: SettingValue;
  /** The default in words, with its unit: `50,000 characters`, `500 KB`. */
  defaultText: string;
  /** The limit the file sets; `null` when it sets none. */
  value: SettingValue | null;
  /** Where the file sets it; `null` when it doesn't. */
  set: SettingPlace | null;
  /** The server's problems on the lines that set it. */
  problems: SettingProblem[];
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
  /**
   * With `--fix`, the safe fixes it applied, in file order and, within a
   * file, in the order of their diagnostics; the rest of the report is
   * the check after them. Left out when none was applied.
   */
  fixed?: FixedEntry[];
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
   * code, in Markdown, whose links are the docs' permalinks, such as
   * `https://ascribed-dev.com/go/22-checks/`.
   */
  help: string;
  /**
   * The address of its entry in the diagnostics reference: the docs'
   * permalink for its code, such as `https://ascribed-dev.com/go/ASC036/`,
   * which forwards to the entry wherever it is.
   */
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

/** A tested example's state. */
export type ExampleState = "tested" | "stale" | "untested";

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

/** A safe fix `--fix` applied. */
export interface FixedEntry {
  /** The code of the diagnostic it fixed, such as `ASC036`. */
  code: string;
  /** The diagnostic's name. */
  slug: string;
  /** What the fix did. */
  title: string;
  /** The file it changed, as a diagnostic's `file` is. */
  file: string;
}

/** How a directive is written (SPEC §3.5, §3.6). */
export type Form = "line" | "block" | "container" | "group";

/**
 * A piece of a formatted value, as the JSON output writes it:
 * `{ "type": "text" | "code", "value": … }`. Other JSON that shows a
 * formatted title, such as the HTML preview's, writes it the same way.
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

/** What kind of directive includes it. */
export type IncludeKind = "include" | "snippet";

/** An entry of the content model. */
export interface InventoryEntry {
  /** What kind of entry it is. */
  kind: ModelKind;
  /** Its key, id, or name, as pages write it. */
  key: string;
  /** Its label, name, or term, where it has one besides its key. */
  label: string | null;
  /**
   * How many places use it; `null` for a build, which pages don't name.
   * For a content type, how many pages have it.
   */
  uses: number | null;
  /**
   * Where it's declared: in `ascribe.toml`, or, for a glossary term, in
   * `page`. `null` for a built-in note type, the implicit `page` type and
   * `site` build, and an entry that can't be found there.
   */
  declaration: LspRange | null;
  /**
   * For a glossary term, the content path of the file its heading is
   * written in, which `declaration` is a range in: its glossary page, or
   * a fragment the page includes. `null` for an entry of `ascribe.toml`.
   */
  page: string | null;
  /**
   * A dimension's values, in declaration order; left out for every other
   * kind.
   */
  values?: InventoryValue[];
}

/** A fragment. */
export interface InventoryFragment {
  /** Its content path. */
  path: string;
  /** The content paths of the files that include it, in order. */
  includedBy: string[];
}

/** A page in a section of the navigation. */
export interface InventoryNavPage {
  /** Its content path, as `pages` has it. */
  path: string;
  /**
   * Whether it has its place: its section lists no pages, or lists it,
   * or it's `unlisted`. One that isn't comes after those that are, and
   * `nav-page-unplaced` reports it.
   */
  placed: boolean;
  /**
   * Whether it's `unlisted`: no build's sidebar shows it, so it needs no
   * place.
   */
  unlisted: boolean;
}

/** A page. */
export interface InventoryPage {
  /** Its content path. */
  path: string;
  /** Its title (frontmatter `title`). */
  title: string | null;
  /**
   * Its label where it's an entry in a list, as the navigation shows it:
   * as plain text, its `nav-label`, else its title, else its file name
   * without `.md`.
   */
  label: string;
  /**
   * The `nav-label` its frontmatter gives it, as written, when it gives
   * one.
   */
  navLabel: string | null;
  /** Its content type; `null` when no one type applies. */
  type: string | null;
  /** How many links from other files, and includes, name it. */
  incoming: number;
  /**
   * The builds that publish it, in declaration order: the ones This
   * page's Builds row says publish it.
   */
  builds: string[];
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
   * The project's navigation (SPEC §7.8), `ascribe.nav`'s order over the
   * default: its root section. `null` for a document that isn't one of
   * the project's files.
   */
  navigation: InventorySection | null;
  /**
   * The content model's entries: phrases, features, glossary terms,
   * dimensions, note types, widgets, builds, then page content types,
   * each kind in declaration order.
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

/** A section of the navigation: a folder and its pages, in order. */
export interface InventorySection {
  /** Its folder, from the content root, ending in `/`; `""` for the root. */
  folder: string;
  /**
   * Its label: `label` from `ascribe.nav`, else its own page's title as
   * plain text, else its folder's name, and "Start" for the root.
   */
  label: string;
  /** The `label` `ascribe.nav` gives it, when it gives one. */
  labelSet: string | null;
  /**
   * The content path of its own page: its `index.md`, or the page beside
   * the folder with its name, which is then one of its parent's `pages`
   * too, and below the content root the section sits where it does.
   */
  page: string | null;
  /** Its pages in order, its own page not among them. */
  pages: InventoryNavPage[];
  /**
   * Its nested sections: below the content root, each whose own page is
   * among `pages` sits there; the rest, and every top section, come after
   * the pages, in this order.
   */
  sections: InventorySection[];
  /** Whether `ascribe.nav` lists it. */
  listed: boolean;
  /**
   * Whether `ascribe.nav` lists its pages, so a page it leaves out isn't
   * placed.
   */
  listsPages: boolean;
}

/** A value of a dimension. */
export interface InventoryValue {
  /** The value, as pages write it. */
  value: string;
  /** Its label, where it has one besides the value. */
  label: string | null;
  /**
   * Where the dimension's `values` in `ascribe.toml` has it, inside its
   * quotes; `null` when it can't be found there.
   */
  declaration: LspRange | null;
}

/** Why a build leaves a page out (SPEC §9.3). */
export type LeftOut = "variant" | "availability" | "draft";

/** A file to create, empty. */
export interface LspCreateFile {
  /** `create`. */
  kind: LspCreateKind;
  /** The file's URI. */
  uri: string;
  /** What to do when it's there already. */
  options?: LspCreateFileOptions;
}

/** What to do when a file to create is there already. */
export interface LspCreateFileOptions {
  /** Whether to leave it as it is, and go on. */
  ignoreIfExists?: boolean;
  /** Whether to write over it. */
  overwrite?: boolean;
}

/** The kind of a file to create. */
export type LspCreateKind = "create";

/** A file to create, or edits to one document. */
export type LspDocumentChange = LspCreateFile | LspTextDocumentEdit;

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

/** Edits to one document. */
export interface LspTextDocumentEdit {
  /** The document. */
  textDocument: LspVersionedDocument;
  /** Its edits, each to the document as it is before any of them. */
  edits: LspTextEdit[];
}

/** A text edit: replace `range` with `newText`. */
export interface LspTextEdit {
  /** The range to replace, in the document as it is before the edit. */
  range: LspRange;
  /** The text that replaces it. */
  newText: string;
}

/** A document, and the version an edit is for; `null` for any. */
export interface LspVersionedDocument {
  /** Its URI. */
  uri: string;
  /** The version. */
  version: number | null;
}

/**
 * Changes to documents: `changes`, or, for an edit that creates a file,
 * `documentChanges`.
 */
export interface LspWorkspaceEdit {
  /** The edits to each document, by its URI. */
  changes?: Record<string, LspTextEdit[]>;
  /**
   * Files to create and edits to make, in order: each file is created
   * before the edit that gives it its text.
   */
  documentChanges?: LspDocumentChange[];
}

/** An API: an OpenAPI spec that pages link into. */
export interface ModelApi {
  /** Its name, as `@operation` and `api:` addresses write it. */
  name: string;
  /** The spec's file, from the folder `ascribe.toml` is in. */
  spec: string;
  /**
   * Where its reference is: the renderer's page, or an address
   * pattern, as written.
   */
  reference: string;
  /**
   * The renderer its reference is published with, `redoc`; absent when
   * `reference` is an address pattern.
   */
  renderer?: string;
}

/** An attribute a widget accepts. */
export interface ModelAttribute {
  /** The key. */
  key: string;
  /**
   * The value's type: `string`, `number`, `boolean`, `enum`, `set`, or
   * `note-type`.
   */
  type: string;
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
export interface ModelBuild {
  /** Its name. */
  name: string;
  /**
   * Which variant arms it keeps: `switch` for all of them, or the
   * selection, such as `deployment=cloud`.
   */
  variants: string;
  /**
   * How it treats availability: `badge`, or the target it filters for,
   * such as `filter self-managed 3.3`.
   */
  availability: string;
  /** Whether it's the build the editor checks (`[editor] build`). */
  editor: boolean;
  /**
   * For a line of a versioned build, the line, as `[releases.<target>]`
   * writes it.
   */
  line?: string;
  /** For a line of a versioned build, the target it's versioned by. */
  versions?: string;
}

/** A dimension. */
export interface ModelDimension {
  /** Its name. */
  name: string;
  /** Its label. */
  label: string;
  /** Its values, in display order. */
  values: ModelDimensionValue[];
}

/** A value of a dimension. */
export interface ModelDimensionValue {
  /** The value. */
  value: string;
  /** Its label. */
  label: string;
  /**
   * Whether it has no versions, so an availability spec gives it a state
   * but no version.
   */
  versionless: boolean;
}

/** A source's tested examples. */
export interface ModelExamples {
  /**
   * Whether its `ascribe.examples.toml` can be read. One that can't makes
   * no file an example.
   */
  valid: boolean;
  /**
   * How many of the files the source makes readable its config makes
   * examples.
   */
  files: number;
  /** Its observations, in name order: those that can't be read left out. */
  observations: ModelObservation[];
}

/** A feature. */
export interface ModelFeature {
  /** Its key, which an availability spec can name. */
  key: string;
  /** Its name. */
  name: string;
  /** Its availability spec, as written. */
  availability: string;
}

/** A frontmatter field. */
export interface ModelField {
  /** Its name. */
  name: string;
  /** Its type in short form: `string`, `list(string)`, `enum(a, b)`. */
  type: string;
  /** Whether every page of the type must give it. */
  required: boolean;
  /**
   * The values it allows, for an enumeration or a list of one; empty
   * otherwise.
   */
  values: string[];
  /** What it's for, as the content model describes it. */
  description: string | null;
  /** An object's fields; empty for any other type. */
  fields: ModelField[];
  /**
   * Which table declares a page type's field: `frontmatter` for
   * `[frontmatter]`, which every type shares, `type` for the type's own
   * `frontmatter`, or `implicit` for the required string `title` neither
   * declares. Absent on an object's fields.
   */
  declared?: string;
}

/** `[fragments]`. */
export interface ModelFragments {
  /**
   * The patterns of the files that are fragments besides those whose
   * path has a file or folder name starting with `_`, from the content
   * root.
   */
  patterns: string[];
  /** The fields a fragment's frontmatter may give, in declaration order. */
  fields: ModelField[];
}

/** A file `[from]` takes tables or keys from. */
export interface ModelFrom {
  /** The file, from the folder `ascribe.toml` is in. */
  file: string;
  /** What it takes, as `take` names it: `releases`, `consumer.site`. */
  take: string[];
}

/**
 * An acknowledged problem: a check's problem about a content model entry
 * or an image, which is intended.
 */
export interface ModelIntended {
  /** The check, by its slug. */
  check: string;
  /** What it's about: `phrase`, `feature`, or `image`. */
  kind: string;
  /** Its key, or the image's path. */
  name: string;
  /** Why the problem is intended. */
  reason: string;
}

/** A kind of content model entry. */
export type ModelKind =
  "phrase" | "feature" | "term" | "dimension" | "note" | "widget" | "build" | "type";

/** A lifecycle state. */
export interface ModelLifecycleState {
  /** Its name, which an availability spec gives. */
  name: string;
  /** Its label. */
  label: string;
  /** Whether content in it counts as available. */
  available: boolean;
  /** Whether it's built in, rather than declared by the project. */
  builtin: boolean;
}

/** A release line. */
export interface ModelLine {
  /** The line, as the list writes it. */
  line: string;
  /** `current`, `live`, `planned`, or `archived`. */
  status: string;
  /**
   * The builds that are this line, one for each build versioned by the
   * target: `site@3.4`, or `site` for the current line.
   */
  builds: string[];
}

/** A page of the navigation. */
export interface ModelNavPage {
  /** Its path from the content root. */
  source: string;
  /**
   * Its label as plain text: its `nav-label`, else its title, else its
   * file name without `.md`.
   */
  label: string;
  /** Its content type's name, when it has one. */
  type?: string;
}

/** A section of the navigation: a folder and its pages, in order. */
export interface ModelNavSection {
  /** The folder, from the content root, ending in `/`; `""` for the root. */
  folder: string;
  /**
   * Its label: from `ascribe.nav`, else its own page's title as plain
   * text, else its folder's name with `-` and `_` as spaces and its first
   * letter capitalized, and "Start" for the root.
   */
  label: string;
  /** Its pages in order, its own page not among them. */
  pages: ModelNavPage[];
  /**
   * Its nested sections with a page, in order: each one whose own page
   * is among `pages` right after that page, then the rest.
   */
  sections: ModelNavSection[];
  /** Whether `ascribe.nav` lists it. */
  listed: boolean;
  /**
   * Its own page: its `index.md`, or the page beside the folder with its
   * name, which is then one of the parent's `pages` too.
   */
  page?: ModelNavPage;
}

/** A note type. */
export interface ModelNoteType {
  /** Its name, which `@note`'s `type` gives. */
  name: string;
  /** Its label. */
  label: string;
  /** Whether it's built in, rather than declared by the project. */
  builtin: boolean;
}

/** An observation of a source's examples. */
export interface ModelObservation {
  /** Its name: `default`, or the name `ascribe observe --name` gave it. */
  name: string;
  /**
   * Its environment as it has it, by name: `os`, `arch`, and each
   * entry's kept line.
   */
  environment: Record<string, string>;
  /** What a page says it was tested with, by this observation alone. */
  tested: string;
  /**
   * How many files it lists. For a source in another repository, only
   * those it has copies of, since the others weren't copied.
   */
  listed: number;
  /**
   * How many of those are as it found them: the rest changed since, or
   * are gone.
   */
  current: number;
}

/** A phrase. */
export interface ModelPhrase {
  /** Its key, written `{key}` in a page. */
  key: string;
  /** Its value. */
  value: string;
}

/** A target's release lines. */
export interface ModelReleases {
  /** The target: a versioned dimension value. */
  target: string;
  /** Its lines, newest first: planned, live, then archived. */
  lines: ModelLine[];
}

/** The repository a source is in. */
export interface ModelRemote {
  /** Its URL. */
  url: string;
  /** The branch an update follows; `null` for the repository's default. */
  branch: string | null;
}

/**
 * What `ascribe model --format json` answers: the content model's sections,
 * or the one asked for.
 */
export interface ModelReport {
  /**
   * The version of this schema. It changes only when a field is removed
   * or changes meaning.
   */
  schema_version: number;
  /** The version of Ascribe that wrote it. */
  ascribe_version: string;
  /** APIs, in declaration order. */
  apis?: ModelApi[];
  /**
   * Builds, in declaration order, a versioned build as one for each live
   * line, the current line first.
   */
  builds?: ModelBuild[];
  /**
   * `[checks]`: each key of `[checks.vale]` and `[checks.links]`, with its
   * value in effect, and each check the file sets, by its slug.
   */
  checks?: ModelSetting[];
  /** `[consumer]`: each key, with its value in effect. */
  consumer?: ModelSetting[];
  /** `[crawlers]`: each key, with its value in effect. */
  crawlers?: ModelSetting[];
  /** Dimensions, in declaration order. */
  dimensions?: ModelDimension[];
  /** `[editor]`: each key, with its value in effect. */
  editor?: ModelSetting[];
  /** Features, in declaration order. */
  features?: ModelFeature[];
  /**
   * `[fragments]`: the files that are fragments, and the fields a
   * fragment's frontmatter may give.
   */
  fragments?: ModelFragments;
  /**
   * `[from]`: each file the project takes from, in the order `take`
   * names them, with what it takes.
   */
  from?: ModelFrom[];
  /**
   * `[frontmatter]`: the fields every page type has, as written, in
   * declaration order. Each type's `fields` has them already.
   */
  frontmatter?: ModelField[];
  /**
   * Glossary terms, in the order of `[glossary] pages`, and on each page
   * in document order.
   */
  glossary?: ModelTerm[];
  /**
   * `[images.attributes]`: the keys an image's attribute block may give,
   * in declaration order.
   */
  images?: ModelAttribute[];
  /** `[[intended]]`: the acknowledged problems, in declaration order. */
  intended?: ModelIntended[];
  /**
   * Lifecycle states: the built-in ones, as the project adjusts them,
   * then the ones it declares.
   */
  lifecycle?: ModelLifecycleState[];
  /** `[listing]`: each key, with its value in effect. */
  listing?: ModelSetting[];
  /**
   * The project's navigation, its root section: every page that isn't
   * `unlisted`, in order, as each build's `_ascribe/nav.json` has its
   * share of it. Only in the whole report.
   */
  navigation?: ModelNavSection;
  /**
   * Note types, the values of `@note`'s `type`: the built-in ones, as
   * the project labels them, then the ones it declares.
   */
  notes?: ModelNoteType[];
  /** Phrases, in declaration order. */
  phrases?: ModelPhrase[];
  /** `[project]`: each key, with its value in effect. */
  project?: ModelSetting[];
  /**
   * `[releases.<target>]`, in declaration order: each target's lines,
   * with their status.
   */
  releases?: ModelReleases[];
  /** Sources, in declaration order. */
  sources?: ModelSource[];
  /**
   * Page types, in declaration order: the implicit `page` type when none
   * is declared.
   */
  types?: ModelType[];
  /** `[versions]`: each key, with its value in effect. */
  versions?: ModelSetting[];
  /** Project widgets, in declaration order. */
  widgets?: ModelWidget[];
}

/**
 * A key of a section of settings, such as `[consumer]`, as the registry of
 * `ascribe.toml`'s keys has it, with the value in effect.
 */
export interface ModelSetting {
  /**
   * The table it's in, as `ascribe.toml` writes it: `consumer`,
   * `checks.vale`.
   */
  table: string;
  /** The key: in `checks`, a check's slug. */
  key: string;
  /**
   * What its value is: `switch`, `choice`, `number`, `text`, `level`,
   * `list`, or `table`.
   */
  kind: string;
  /** Whether `ascribe.toml` sets it. */
  set: boolean;
  /** The value used when the file leaves it out, when that's one value. */
  default?: unknown;
  /**
   * What's used when the file leaves it out, in the reference's words,
   * when that isn't one value.
   */
  default_text?: string;
  /**
   * The file it's taken from, when `[from]` takes it: a path from the
   * folder `ascribe.toml` is in. Its value is that file's, and it's
   * `set`.
   */
  from?: string;
  /**
   * The value in effect: the file's, else the default. Absent when the
   * file leaves it out and its default isn't one value.
   */
  value?: unknown;
}

/** A source: a folder of files, such as code, that pages can show. */
export interface ModelSource {
  /** Its name, as addresses write it: `code:` for a source named `code`. */
  name: string;
  /**
   * Its folder, relative to the project's folder: the `path` written,
   * or, for a source in another repository, the folder of its copies.
   */
  path: string;
  /**
   * The patterns of the files it makes readable, relative to its folder;
   * empty for every file.
   */
  include: string[];
  /**
   * The patterns of the files it doesn't make readable, though `include`
   * matches them.
   */
  ignore: string[];
  /** The repository it's in, for a source in another repository. */
  git: ModelRemote | null;
  /**
   * Its tested examples, for a source with an `ascribe.examples.toml`:
   * absent for any other. For a source in another repository, they're
   * read from its copies.
   */
  examples?: ModelExamples;
}

/**
 * A glossary term: a heading marked `@term` on a glossary page. Its
 * definition is the paragraph under the heading, which differs by build:
 * the JSON output of a build has it.
 */
export interface ModelTerm {
  /** Its id: its heading's id. */
  id: string;
  /** The term: its heading's text. */
  term: string;
  /** Other ways it's written. */
  aliases: string[];
  /**
   * Which occurrences are linked: `first` on each page, `every` one, or
   * only those an author links (`marked`).
   */
  match: string;
  /** Whether an occurrence must match its case. */
  case_sensitive: boolean;
  /**
   * The file its heading is written in, from the content root: its
   * glossary page, or a fragment the page includes.
   */
  page: string;
  /** The heading's line in that file, from 1. */
  line: number;
}

/** A page type. */
export interface ModelType {
  /**
   * Its name, which each page of the type gives as its `type`; the
   * implicit `page` type's pages give none.
   */
  name: string;
  /** Its frontmatter fields, in declaration order. */
  fields: ModelField[];
  /**
   * What to read before writing a page of the type, in the order to read
   * it, from the folder `ascribe.toml` is in, from the type's `guide` or
   * the lock, as `template` is; absent when it has none.
   */
  guide?: string[];
  /**
   * What a page of the type is for, as the content model says; absent
   * when it says nothing.
   */
  purpose?: string;
  /**
   * The page a page of the type starts from, from the folder
   * `ascribe.toml` is in: the type's `template`, or the one
   * `templates/templates.lock` gives a type that names neither a template
   * nor a guide; absent when it has none.
   */
  template?: string;
}

/** A project widget. */
export interface ModelWidget {
  /** Its name, written `@name`. */
  name: string;
  /** What it's for, as the content model describes it. */
  description: string | null;
  /** Whether it may be one line. */
  line: boolean;
  /** Whether it may be a container, closed by `@end`. */
  container: boolean;
  /** Its attributes, in canonical order. */
  attributes: ModelAttribute[];
}

/** A page that moved: where it was, and where it is, by content path. */
export interface MovedPage {
  /** Where it was, in the base. */
  from: string;
  /** Where it is now. */
  to: string;
}

/** What a page or fragment is. */
export interface PageAbout {
  /** Its content path. */
  path: string;
  /** Whether it's a fragment, which is published only where it's included. */
  fragment: boolean;
  /** Its title (frontmatter `title`), as written. */
  title: string | null;
  /**
   * The title formatted, when its type's `title` sets `inline`: its
   * pieces of text and code, as the JSON output writes a formatted title,
   * with phrases substituted when the field takes them; `null` otherwise.
   */
  formattedTitle: FormattedPiece[] | null;
  /**
   * Its content type; `null` for a fragment, and when no one type
   * applies.
   */
  type: string | null;
  /**
   * Its address, from the host's root, as the site output's links write
   * it; `null` for a fragment.
   */
  address: string | null;
  /**
   * Its Markdown version's address, from the host's root, when the
   * project publishes one (`[consumer] agents = true`); `null` otherwise,
   * and for a fragment.
   */
  markdownAddress: string | null;
  /**
   * The project's published site, its origin (`[consumer] site`), which
   * the addresses are on: `null` when the project names none, and for a
   * fragment.
   */
  site: string | null;
  /**
   * Its description, when its type gives a field the description role;
   * `null` for a fragment.
   */
  description: PageDescription | null;
  /**
   * When it's due to be checked again, when its type gives a field the
   * recheck-date role and the page has a date in it.
   */
  recheck: PageRecheck | null;
  /** Its own `available`, as shown; `null` when it has none. */
  availability: string | null;
  /**
   * Whether each build publishes it, in declaration order; empty for a
   * fragment, which is published only where it's included.
   */
  builds: PageBuild[];
  /**
   * Whether it's listed, and what sets that; `null` for a fragment and
   * for a page no build publishes.
   */
  listing: PageListing | null;
  /** The permalinks the file declares, in document order. */
  permalinks: PagePermalink[];
  /**
   * The old addresses that lead to it, each once, in address order: each
   * `from` of a redirect a build writes to it in `redirects.json`, with
   * its entry, as the server last compared the addresses, at the last
   * save or change on disk. Empty for a fragment, and for a project
   * without `ascribe.addresses`.
   */
  alsoAnswersAt: PageOldAddress[];
  /**
   * What its type's template still leaves to fill in; `null` when its
   * type has no template or nothing of it is left.
   */
  template: PageTemplate | null;
}

/** Whether a build publishes the page. */
export interface PageBuild {
  /** The build's name. */
  name: string;
  /** Why the build leaves the page out; `null` when it publishes it. */
  leftOut: LeftOut | null;
}

/** A page's description, measured against its line in `llms.txt`. */
export interface PageDescription {
  /** The frontmatter field the page's type gives the description role. */
  field: string;
  /**
   * Its length in characters, as plain text on one line, as
   * `description-too-long` measures it: the page's own, or its field's
   * default when it leaves the field out or empty. `null` when it has
   * none, which is where `page-description-missing` reports it.
   */
  length: number | null;
  /**
   * The most characters it has before `description-too-long` reports it;
   * `null` when the project publishes no `llms.txt`
   * (`[consumer] agents`), which is the only place it's too long for.
   */
  limit: number | null;
  /**
   * Whether `description-too-long` reports it: it's over `limit`, and the
   * project hasn't turned the check off. A default is reported where
   * `ascribe.toml` writes it.
   */
  reported: boolean;
  /**
   * The field's line, or the page's first line when it has none, in the
   * negotiated position encoding.
   */
  range: LspRange;
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
   * (`code:app.py#main`); then the operations of a declared API whose spec
   * changed what the page writes for them, an `@operation`'s method, path,
   * summary, or deprecation, or a link's address, each by its spec's path
   * from the project root, `#`, and the fragment the page names it by
   * (`api/openapi.yaml#createWidget`); and `ascribe.toml` (last) when the
   * content model is a cause. The kinds are told apart by their form: a
   * content file's path ends in `.md`, a snippet's address has its
   * source's name and `:` before its path, and an operation's has a `#`
   * and neither.
   */
  because: string[];
  /**
   * What changed about the page itself besides its blocks, in this
   * order: `title`, `frontmatter`, `availability` (the page-level one),
   * and `route`. Empty for an added or removed page. A moved page's
   * route moves with it, so it isn't listed for one.
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
  /**
   * For a page that moved, the content path it was at: git sees its
   * file renamed, or `ascribe.addresses` records the move. The page is
   * listed once, at `path`, compared with the page as it was at
   * `moved_from`; `status` is `changed`, and its `counts` and `changes`
   * are what differs besides the move, none for a page that only moved.
   * Absent for a page that didn't move.
   */
  moved_from?: string;
  /**
   * The pages among `because` whose only change is that they moved, each
   * by where it was and where it is: a link on this page leads to its new
   * address. Absent when there's none.
   */
  moved_links?: MovedPage[];
}

/** A tested example a `@snippet` of the file shows. */
export interface PageExample {
  /** The `@snippet`'s address, as written. */
  address: string;
  /** Whether it's tested as it is now. */
  state: ExampleState;
  /** What it was tested with, when it's tested. */
  tested: string | null;
  /**
   * The heading it's under: the innermost whose section holds it, as
   * plain text; `null` before the first heading.
   */
  heading: string | null;
  /** The `@snippet` line, in the negotiated position encoding. */
  range: LspRange;
}

/** A heading of the page. */
export interface PageHeading {
  /** Its source id: its `@id`, or its slug. */
  id: string;
  /**
   * Its text as plain text, with each declared phrase replaced by its
   * value.
   */
  text: string;
  /** The heading line, in the negotiated position encoding. */
  range: LspRange;
}

/** How a page stands: its size, and its tested examples. */
export interface PageHealth {
  /**
   * Its plain Markdown's size in the build where it's largest; `null`
   * for a fragment, and for a page no build publishes.
   */
  size: PageSize | null;
  /**
   * Each `@snippet` of the file that shows a tested example, in
   * document order.
   */
  examples: PageExample[];
}

/** What a file includes: a fragment, or a snippet of a code file. */
export interface PageInclude {
  /** `include` for an `@include`, `snippet` for a `@snippet`. */
  kind: IncludeKind;
  /** What it names, as written. */
  written: string;
  /**
   * The file it shows: a fragment's content path, or a code file's path
   * from the project root; `null` when the address names no file.
   */
  path: string | null;
  /** That file's `file:` URI. */
  uri: string | null;
  /** The directive line, in the negotiated position encoding. */
  range: LspRange;
}

/** Whether search engines list a page, and what sets that. */
export interface PageListing {
  /**
   * The build it's read from: the editor's build when it publishes the
   * page, and otherwise the first that does.
   */
  build: string;
  /** `listed`, `unquoted`, or `unlisted`: `pages.json`'s `listing`. */
  listing: string;
  /**
   * What sets it, as `pages.json`'s `listing_from.level` says: `default`,
   * `page`, `site`, `older-lines`, `folder`, or `type`.
   */
  from: string;
  /** The rule of `[listing]` that sets it, as written; `null` otherwise. */
  rule: string | null;
  /**
   * The `file:` URI of the file that sets it: the page for its own
   * `listing`, `ascribe.toml` for a rule; `null` when nothing does.
   */
  uri: string | null;
  /** Where in that file, in the negotiated position encoding. */
  range: LspRange | null;
}

/** A memo This page lists. */
export interface PageMemo {
  /** The memo file's `file:` URI. */
  uri: string;
  /** Its path from the project root. */
  path: string;
  /** The first line of its text that isn't blank. */
  firstLine: string;
  /** `placed`, or `outdated` when no block holds its quote. */
  state: string;
  /**
   * Whether it's a personal memo, kept in `.ascribe/memos/` on this
   * machine only.
   */
  personal: boolean;
  /**
   * Where its text starts, in the memo file, in the negotiated position
   * encoding.
   */
  body: LspRange;
}

/** An old address that leads to a page: This page's "Also answers at". */
export interface PageOldAddress {
  /** The address, from the host's root, as `redirects.json`'s `from`. */
  address: string;
  /**
   * The `ascribe.addresses` key it comes from, as `redirects.json`'s
   * `entry`.
   */
  entry: string;
}

/** A permalink declared in the file (SPEC §7.5). */
export interface PagePermalink {
  /** Its name. */
  name: string;
  /** The text of the heading it leads to; `null` for the page's own. */
  heading: string | null;
  /**
   * The summary of the section it leads to in the editor's build, as
   * `_ascribe/help.json` writes it: the section's `@summary`, or for the
   * page's own, the page's description. `null` when it has none, which
   * is where `summary-missing` reports a heading's.
   */
  summary: string | null;
  /**
   * The page's `permalink` line, or the heading's `@id` line, in the
   * negotiated position encoding.
   */
  range: LspRange;
}

/** A link or an `@include`. */
export interface PagePlace {
  /** The `file:` URI of the file it's written in. */
  uri: string;
  /** That file's content path. */
  path: string;
  /** The link or the directive line, in the negotiated position encoding. */
  range: LspRange;
  /** The text of the line it starts on, trimmed. */
  text: string;
}

/** When a page is due to be checked again. */
export interface PageRecheck {
  /** The frontmatter field with the recheck-date role. */
  field: string;
  /** The date it holds, `YYYY-MM-DD`. */
  date: string;
  /**
   * The days from today to the date: negative once it's passed. `null`
   * when the server has no day to count from.
   */
  days: number | null;
  /** Whether the date has passed. */
  overdue: boolean;
  /**
   * Whether `recheck-overdue` reports the page: the date has passed, and
   * the project hasn't turned the check off.
   */
  reported: boolean;
  /** The field's line, in the negotiated position encoding. */
  range: LspRange;
}

/**
 * The answer to `ascribe/page`. A document that isn't a page or a fragment
 * of the project gets an empty answer: no `about`, no heading, and empty
 * lists.
 */
export interface PageResult {
  /**
   * What the page is; `null` when the document isn't a page or a
   * fragment.
   */
  about: PageAbout | null;
  /**
   * The heading whose section holds the cursor, the innermost one with an
   * id; `null` when the cursor is in none, and the lists are then about
   * the whole file.
   */
  heading: PageHeading | null;
  /**
   * The links to the heading, or to the file when there's no heading:
   * from other files, and, for a heading, from its own page and from a
   * page that includes it. File by file in content path order, and in
   * document order within a file.
   */
  linkedFrom: PagePlace[];
  /**
   * The `@include`s of the file, or of the heading's section, in the
   * same order.
   */
  includedBy: PagePlace[];
  /**
   * The memo files about the heading, or about the file when there's no
   * heading, or about a block in it (SPEC §7.7): the committed ones, then
   * the personal ones, each in path order. A detached memo isn't listed.
   */
  memos: PageMemo[];
  /**
   * How the page stands: its size and its tested examples. Empty when
   * the document isn't a page or a fragment.
   */
  health: PageHealth;
  /** The file's `@include`s and `@snippet`s, in document order. */
  includes: PageInclude[];
}

/** A page's size, as `page-size` measures it. */
export interface PageSize {
  /** Characters of plain Markdown. */
  characters: number;
  /** The build it's that size in: the first of the largest. */
  build: string;
  /** The size `page-size` reports it at; `null` when the check is off. */
  limit: number | null;
}

/** Whether a page is new, gone, or different. */
export type PageStatus = "added" | "removed" | "changed";

/** What a page's template still leaves to fill in. */
export interface PageTemplate {
  /** The content type whose template it is. */
  type: string;
  /**
   * How many placeholders and template values are left, as
   * `template-placeholder-left` finds them.
   */
  left: number;
}

/**
 * A part of the project with structure, which Project Settings shows as a
 * card that opens the file at its table.
 */
export interface PartEntry {
  /**
   * The table it's in: `types`, `dimensions`, `releases`, `builds`, or
   * `sources`; empty for a table at the top level, `[frontmatter]`.
   */
  table: string;
  /** Its name in that table: `guide`, or `frontmatter`. */
  name: string;
  /** The keys the file sets in it, in the file's order. */
  keys: string[];
  /**
   * Where the file names it: its key, in its table's header or its
   * dotted key. `null` when it can't be placed.
   */
  set: SettingPlace | null;
}

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
  /**
   * For a versioned build, its live lines, the current one first, each
   * a build the preview can render. Absent for a build that isn't
   * versioned.
   */
  lines?: BuildLineEntry[];
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
   * anchors, for showing removed blocks and changed blocks as they were,
   * and a removed page whole; `null` when the page didn't change or is
   * new.
   */
  wasHtml: string | null;
  /** The page's title as it was, with `wasHtml`. */
  wasTitle: string | null;
  /** That title formatted, when its field sets `inline = "code"`. */
  wasFormattedTitle: FormattedPiece[] | null;
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
 * The answer to `ascribe/projectEdit`: the edit and how much it changes, or
 * why there's none.
 */
export type ProjectEditResult =
  | {
      /**
       * Edits to each file it changes, under `documentChanges`, in path
       * order: an open document's names the version it's for, and any
       * other's `null`.
       */
      edit: LspWorkspaceEdit;
      /** How many files it changes. */
      files: number;
      /**
       * For the safe fixes, how many problems they fix; `null` for
       * formatting.
       */
      fixes: number | null;
    }
  | {
      /** The message. */
      error: string;
    };

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

/** What a place is. */
export type SearchIn = "title" | "description" | "heading" | "text" | "table" | "code" | "phrase";

/** A page that shows a place. */
export interface SearchPage {
  /** The page's content path. */
  page: string;
  /**
   * The heading the place is under there, as a reader reads it; for a
   * heading, the heading itself.
   */
  heading: string | null;
  /** That heading's id. */
  id: string | null;
  /** The builds that show it there, in the content model's order. */
  builds: string[];
}

/** A place the query matches: a line of the source to edit. */
export interface SearchPlace {
  /**
   * The `file:` URI of the file to edit: a page, a fragment, a snippet's
   * code file, or `ascribe.toml` for a phrase.
   */
  uri: string;
  /**
   * The match's first character to the end of its line, in the file.
   * For a phrase, its key to the end of its line in `ascribe.toml`.
   */
  range: LspRange;
  /**
   * The file as the editor names it: a source's content path, a code
   * file's path from the project root, or `ascribe.toml`.
   */
  file: string;
  /**
   * What it is: `title`, `description`, `heading`, `text`, `table`,
   * `code`, or `phrase`, a phrase's value at its line in `ascribe.toml`.
   */
  in: SearchIn;
  /**
   * The block's text as a reader reads it, whole; for a phrase, its
   * value.
   */
  text: string;
  /**
   * Where the match is in `text`: its first character and the one after
   * its last, in Unicode characters.
   */
  match: [number, number];
  /** What the match reads through that the line doesn't spell, in order. */
  through: SearchThrough[];
  /**
   * The arm of a group the block is written in, as its attributes are
   * written (`deployment=cloud`), or else its title.
   */
  arm: string | null;
  /** The pages that show it, in content path order, at most three. */
  pages: SearchPage[];
  /**
   * How many pages show it; for a phrase, how many places in the source
   * use it.
   */
  pagesTotal: number;
  /**
   * The builds it's in, in the content model's order, when it isn't in
   * every build searched; empty when it is.
   */
  builds: string[];
}

/**
 * The answer to `ascribe/search`. A document that isn't one of the
 * project's files gets an answer with nothing searched.
 */
export interface SearchResult {
  /** The builds searched, in the content model's order. */
  builds: string[];
  /**
   * The places, at most `limit` of them, in the order `ascribe search`
   * lists them: titles, then headings, then the rest; in each, the places
   * written in pages, then those in fragments and code files, each in
   * path order and in order within a file.
   */
  places: SearchPlace[];
  /** How many places match. */
  total: number;
  /** Whether the list was cut: there are more than `places`. */
  truncated: boolean;
  /**
   * When nothing matched, what the source says about the words, as the
   * last line of `ascribe search` says it: that no page has them, where
   * they're written though no build searched publishes them there, or
   * the builds not searched that do. `null` when something matched, or
   * the query couldn't be read.
   */
  nothing: string | null;
  /**
   * Why nothing was searched: the query can't be read (a quote isn't
   * closed, or it has no words), or the build isn't one of the content
   * model's. `null` when it was searched.
   */
  problem: string | null;
}

/** What a match reads through that its line doesn't spell. */
export type SearchThrough =
  | {
      phrase: string;
    }
  | {
      title: string;
    }
  | {
      code: string;
    };

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

/** One setting. */
export interface SettingEntry {
  /** Its table, as a dotted path: `consumer`, or empty for the top level. */
  table: string;
  /** Its key. */
  key: string;
  /** Its name for people: the key in words, `Trailing slash`. */
  name: string;
  /** What its value is: `switch`, `choice`, `number`, `text`, or `level`. */
  kind: string;
  /**
   * For a choice or a level, its values, in the order a message lists
   * them.
   */
  choices: string[];
  /** The value used when the file leaves it out, when it has one. */
  default: SettingValue | null;
  /** The reference's words for a default that isn't one value. */
  defaultText: string | null;
  /**
   * What it does, as plain text: the registry's Markdown with a link's
   * address dropped and its text kept, emphasis taken off, and code spans
   * as pieces of their own.
   */
  description: SettingPiece[];
  /**
   * Its value in the file; `null` when the file leaves it out, or writes
   * it as something other than a string, a boolean, or a whole number.
   */
  value: SettingValue | null;
  /** Where the file sets it; `null` when it leaves it out. */
  set: SettingPlace | null;
  /** The server's problems on the lines that set it, in its words. */
  problems: SettingProblem[];
}

/** A piece of a description. */
export interface SettingPiece {
  /** Its text. */
  text: string;
  /** Whether it's a code span. */
  code: boolean;
}

/** Where the file sets a setting. */
export interface SettingPlace {
  /** From its key to the end of its value. */
  range: LspRange;
  /** The line its key is on, counted from 1, as `ascribe.toml:12` names it. */
  line: number;
}

/** A problem the server reports on a setting. */
export interface SettingProblem {
  /** The message, as the Problems panel shows it. */
  message: string;
  /** `error`, `warning`, or `advice`, at the level `[checks]` sets. */
  severity: string;
}

/** A setting's value. */
export type SettingValue = boolean | number | string;

/**
 * The answer to `ascribe/settings`. A document that isn't one of the
 * project's files gets no settings.
 */
export interface SettingsResult {
  /**
   * The `file:` URI of the project's `ascribe.toml`, which the ranges are
   * in.
   */
  modelUri: string | null;
  /**
   * The version of `ascribe.toml`'s open document the answer was read
   * from; `null` when it isn't open, and the answer was read from the
   * disk.
   */
  version: number | null;
  /**
   * Why the file's values can't be read: it isn't valid TOML. `null` when
   * they can.
   */
  unreadable: string | null;
  /**
   * Every setting of a simple kind in a table a project doesn't name
   * (`[project]`, `[consumer]`, `[editor]`, and the rest, not
   * `[types.<name>]`), in the content model reference's order.
   */
  settings: SettingEntry[];
  /**
   * Every configurable check: the ones the file sets first, then the
   * rest, each in the diagnostics reference's order.
   */
  checks: CheckEntry[];
  /**
   * The parts of the project with structure the file names: the
   * frontmatter every type shares, then each content type, dimension,
   * release target, build, and source, in the content model reference's
   * order and the file's within each.
   */
  parts: PartEntry[];
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
  /**
   * How many problems `ascribe.baseline` holds back, which `diagnostics`
   * leaves out and which are advice. Left out when there are none, and
   * with `--no-baseline`.
   */
  baselined?: number;
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
  /**
   * For a versioned build, its live lines, the current one first: each
   * is a build of its own a client can choose (`site@3.4`). Absent for
   * a build that isn't versioned.
   */
  lines?: BuildLineEntry[];
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

/** A page type. */
export interface TargetType {
  /** Its name. */
  name: string;
  /**
   * What it's for, on one line: its `purpose`, or, for a type ejected
   * from the set, their index's summary of it, as the rules for agents
   * say it; `null` when it says neither.
   */
  purpose: string | null;
  /**
   * Whether it has a template, which a page starts from; without one, a
   * page starts with its frontmatter.
   */
  template: boolean;
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
  /**
   * Page types, in declaration order: with a folder, only those a page
   * started in it would be of.
   */
  types?: TargetType[];
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
