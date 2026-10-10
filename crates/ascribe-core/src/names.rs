//! The names Ascribe puts on a page: elements, attributes, classes, and ids.
//!
//! This is their one home. The Rust crates use these constants;
//! `crates/ascribe-core/tests/names.rs` writes the same constants, with the
//! same comments, into a generated `src/names.ts` in each package that needs
//! them, and fails when one is stale (`ASCRIBE_BLESS=1` rewrites them). The
//! same test fails on a literal of one of these names in source anywhere
//! else, and on a name in a stylesheet that isn't here. Tests and fixtures
//! keep their literals: they're the independent check.
//!
//! The names belong to the published contracts named on each, and don't
//! change without a decision to change them. Strings that only look like
//! names aren't here: custom properties (`--ascribe-…`), storage keys, the
//! language server's registration ids, and documentation anchors.

/// Declares each name as a constant, and [`ALL`] as every one, with its
/// documentation, for the generated TypeScript.
macro_rules! names {
    ($($(#[doc = $doc:literal])+ $name:ident = $value:literal;)+) => {
        $(
            $(#[doc = $doc])+
            pub const $name: &str = $value;
        )+

        /// Every name: its constant's name, its value, and its documentation,
        /// one line per line of the doc comment.
        pub const ALL: &[Name] = &[
            $(Name { constant: stringify!($name), value: $value, doc: &[$($doc),+] },)+
        ];
    };
}

/// One name, as [`ALL`] lists it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Name {
    /// The constant's name, the same in Rust and TypeScript.
    pub constant: &'static str,
    /// The name itself.
    pub value: &'static str,
    /// The constant's doc comment, one entry per line, each starting with
    /// the space after `///`.
    pub doc: &'static [&'static str],
}

names! {
    // The site output's elements: packages/elements/CONTRACT.md.

    /// `<ascribe-note>`: a note (packages/elements/CONTRACT.md §1).
    ELEMENT_NOTE = "ascribe-note";
    /// `<ascribe-steps>`: a procedure (packages/elements/CONTRACT.md §2).
    ELEMENT_STEPS = "ascribe-steps";
    /// `<ascribe-tabs>`: a variant group of several arms (packages/elements/CONTRACT.md §3).
    ELEMENT_TABS = "ascribe-tabs";
    /// `<ascribe-tab>`: one arm of `<ascribe-tabs>` (packages/elements/CONTRACT.md §3).
    ELEMENT_TAB = "ascribe-tab";
    /// `<ascribe-availability>`: a badge or filter (packages/elements/CONTRACT.md §4).
    ELEMENT_AVAILABILITY = "ascribe-availability";
    /// `<ascribe-availability-target>`: one target of `<ascribe-availability>`
    /// (packages/elements/CONTRACT.md §4).
    ELEMENT_AVAILABILITY_TARGET = "ascribe-availability-target";
    /// `<ascribe-group>`: a project widget's group (packages/elements/CONTRACT.md §6).
    ELEMENT_GROUP = "ascribe-group";
    /// `<ascribe-for-agents>`: the visually hidden pointer to `llms.txt` at the top
    /// of a page, for agents (packages/elements/CONTRACT.md §8).
    ELEMENT_FOR_AGENTS = "ascribe-for-agents";

    // The site output's markers: docs/content/contracts/site-render.md.

    /// `<ascribe-attributes>`: the attribute marker (site-render contract §1).
    ELEMENT_ATTRIBUTES = "ascribe-attributes";
    /// The name an anchor comment starts with, `<!--ascribe-anchor` (site-render contract §7.3).
    COMMENT_ANCHOR = "ascribe-anchor";
    /// A block's source anchor (site-render contract §7.1).
    DATA_SOURCE = "data-ascribe-source";
    /// The includes a block came through (site-render contract §7.1).
    DATA_VIA = "data-ascribe-via";
    /// A glossary term's id, on its link (packages/elements/CONTRACT.md §7).
    DATA_TERM = "data-ascribe-term";

    // The HTML report, `ascribe diff --format html` (crates/ascribe-diff/src/html/),
    // which @ascribed/review's report script draws.

    /// The element the report draws into.
    ID_REPORT = "ascribe-review";
    /// The `<script type="application/json">` holding the report's data.
    ID_REPORT_DATA = "ascribe-review-data";

    // Review's marks and threads (@ascribed/review), which the HTML report,
    // the page preview, and the Astro dev toolbar host. Written by TypeScript;
    // kept here so every package and stylesheet takes them from one place.

    /// The page's light or dark scheme, `light` or `dark`, on the root
    /// element (docs/content/guides/astro.md).
    DATA_SCHEME = "data-ascribe-scheme";
    /// On a marked page's root: what it shows, `changes`, `will`, or `was`.
    DATA_SHOW = "data-ascribe-show";
    /// On a marked block: its change, `added`, `changed`, `removed`, `moved`,
    /// or `moved-from`.
    DATA_CHANGE = "data-ascribe-change";
    /// On every element review adds to the page, so it can be taken away again.
    DATA_UI = "data-ascribe-ui";
    /// On a change's label: the change.
    DATA_LABEL = "data-ascribe-label";
    /// On a tab's hint: `added` or `changed`.
    DATA_HINT = "data-ascribe-hint";
    /// On a moved block: the id its "moved from" link points at.
    DATA_MOVE = "data-ascribe-move";
    /// On a changed block whose copy as it was follows it.
    DATA_HAS_WAS = "data-ascribe-has-was";
    /// On a copy of a block as it was, shown only with the page as it was.
    DATA_WAS_ONLY = "data-ascribe-was-only";
    /// On a changed, removed, or moved block: its source anchor as it was.
    DATA_WAS_SOURCE = "data-ascribe-was-source";
    /// On the elements that hold the threads' overlay.
    DATA_OVERLAY = "data-ascribe-overlay";
    /// On the style element the Astro dev toolbar adds for the marks.
    DATA_REVIEW = "data-ascribe-review";
    /// On the root of a page whose changes are marked.
    CLASS_MARKS = "ascribe-marks";
    /// A change's label.
    CLASS_LABEL = "ascribe-label";
    /// A label placed before its block rather than over it.
    CLASS_LABEL_BEFORE = "ascribe-label-before";
    /// Inserted text.
    CLASS_INS = "ascribe-ins";
    /// Deleted text.
    CLASS_DEL = "ascribe-del";
    /// A removed block.
    CLASS_REMOVED = "ascribe-removed";
    /// A removed block's content.
    CLASS_REMOVED_BODY = "ascribe-removed-body";
    /// The button that shows or collapses a removed block.
    CLASS_TOGGLE = "ascribe-toggle";
    /// On a removed block that's shown.
    CLASS_OPEN = "ascribe-open";
    /// What stands where a moved block was.
    CLASS_MOVED_FROM = "ascribe-moved-from";
    /// The note with a moved block's link.
    CLASS_MOVE_NOTE = "ascribe-move-note";
    /// A move note after its block, not inside it.
    CLASS_MOVE_AFTER = "ascribe-move-after";
    /// The link between a moved block and where it was.
    CLASS_MOVE_LINK = "ascribe-move-link";
    /// A moved block's copy as it was, where it was.
    CLASS_WAS_COPY = "ascribe-was-copy";
    /// A tab's hint that it holds changes.
    CLASS_HINT = "ascribe-hint";
    /// A block flashed after going to it.
    CLASS_FLASH = "ascribe-flash";
    /// The tip that shows a block's source on hover and focus.
    CLASS_WHERE = "ascribe-where";
}
