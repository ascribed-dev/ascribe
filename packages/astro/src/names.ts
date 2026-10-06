// The names Ascribe puts on a page: elements, attributes, classes, and ids.
// Generated from crates/tessera-core/src/names.rs by
// `ASCRIBE_BLESS=1 cargo test -p tessera-core --test names`. Don't edit it.

/** `<ascribe-note>`: a note (packages/elements/CONTRACT.md §1). */
export const ELEMENT_NOTE = "ascribe-note";

/** `<ascribe-steps>`: a procedure (packages/elements/CONTRACT.md §2). */
export const ELEMENT_STEPS = "ascribe-steps";

/** `<ascribe-tabs>`: a variant group of several arms (packages/elements/CONTRACT.md §3). */
export const ELEMENT_TABS = "ascribe-tabs";

/** `<ascribe-tab>`: one arm of `<ascribe-tabs>` (packages/elements/CONTRACT.md §3). */
export const ELEMENT_TAB = "ascribe-tab";

/** `<ascribe-availability>`: a badge or filter (packages/elements/CONTRACT.md §4). */
export const ELEMENT_AVAILABILITY = "ascribe-availability";

/**
 * `<ascribe-availability-target>`: one target of `<ascribe-availability>`
 * (packages/elements/CONTRACT.md §4).
 */
export const ELEMENT_AVAILABILITY_TARGET = "ascribe-availability-target";

/** `<ascribe-group>`: a project widget's group (packages/elements/CONTRACT.md §6). */
export const ELEMENT_GROUP = "ascribe-group";

/** `<ascribe-attributes>`: the attribute marker (site-render contract §1). */
export const ELEMENT_ATTRIBUTES = "ascribe-attributes";

/** The name an anchor comment starts with, `<!--ascribe-anchor` (site-render contract §7.3). */
export const COMMENT_ANCHOR = "ascribe-anchor";

/** A block's source anchor (site-render contract §7.1). */
export const DATA_SOURCE = "data-ascribe-source";

/** The includes a block came through (site-render contract §7.1). */
export const DATA_VIA = "data-ascribe-via";

/** A glossary term's id, on its link (packages/elements/CONTRACT.md §7). */
export const DATA_TERM = "data-ascribe-term";

/** The element the report draws into. */
export const ID_REPORT = "ascribe-review";

/** The `<script type="application/json">` holding the report's data. */
export const ID_REPORT_DATA = "ascribe-review-data";

/**
 * The page's light or dark scheme, `light` or `dark`, on the root
 * element (docs/content/guides/astro.md).
 */
export const DATA_SCHEME = "data-ascribe-scheme";

/** On a marked page's root: what it shows, `changes`, `will`, or `was`. */
export const DATA_SHOW = "data-ascribe-show";

/**
 * On a marked block: its change, `added`, `changed`, `removed`, `moved`,
 * or `moved-from`.
 */
export const DATA_CHANGE = "data-ascribe-change";

/** On every element review adds to the page, so it can be taken away again. */
export const DATA_UI = "data-ascribe-ui";

/** On a change's label: the change. */
export const DATA_LABEL = "data-ascribe-label";

/** On a tab's hint: `added` or `changed`. */
export const DATA_HINT = "data-ascribe-hint";

/** On a moved block: the id its "moved from" link points at. */
export const DATA_MOVE = "data-ascribe-move";

/** On a changed block whose copy as it was follows it. */
export const DATA_HAS_WAS = "data-ascribe-has-was";

/** On a copy of a block as it was, shown only with the page as it was. */
export const DATA_WAS_ONLY = "data-ascribe-was-only";

/** On a changed, removed, or moved block: its source anchor as it was. */
export const DATA_WAS_SOURCE = "data-ascribe-was-source";

/** On the elements that hold the threads' overlay. */
export const DATA_OVERLAY = "data-ascribe-overlay";

/** On the style element the Astro dev toolbar adds for the marks. */
export const DATA_REVIEW = "data-ascribe-review";

/** On the root of a page whose changes are marked. */
export const CLASS_MARKS = "ascribe-marks";

/** A change's label. */
export const CLASS_LABEL = "ascribe-label";

/** A label placed before its block rather than over it. */
export const CLASS_LABEL_BEFORE = "ascribe-label-before";

/** Inserted text. */
export const CLASS_INS = "ascribe-ins";

/** Deleted text. */
export const CLASS_DEL = "ascribe-del";

/** A removed block. */
export const CLASS_REMOVED = "ascribe-removed";

/** A removed block's content. */
export const CLASS_REMOVED_BODY = "ascribe-removed-body";

/** The button that shows or collapses a removed block. */
export const CLASS_TOGGLE = "ascribe-toggle";

/** On a removed block that's shown. */
export const CLASS_OPEN = "ascribe-open";

/** What stands where a moved block was. */
export const CLASS_MOVED_FROM = "ascribe-moved-from";

/** The note with a moved block's link. */
export const CLASS_MOVE_NOTE = "ascribe-move-note";

/** A move note after its block, not inside it. */
export const CLASS_MOVE_AFTER = "ascribe-move-after";

/** The link between a moved block and where it was. */
export const CLASS_MOVE_LINK = "ascribe-move-link";

/** A moved block's copy as it was, where it was. */
export const CLASS_WAS_COPY = "ascribe-was-copy";

/** A tab's hint that it holds changes. */
export const CLASS_HINT = "ascribe-hint";

/** A block flashed after going to it. */
export const CLASS_FLASH = "ascribe-flash";

/** The tip that shows a block's source on hover and focus. */
export const CLASS_WHERE = "ascribe-where";
