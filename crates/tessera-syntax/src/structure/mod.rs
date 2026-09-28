//! The structure pass (phase 06): turns the flat sequence of directive lines
//! and blocks that tree conversion produces into Tessera's structure.
//!
//! [`crate::parse`] runs it after conversion. It works one *scope* at a time:
//! the document, a list item, or a block quote, each a list of sibling
//! blocks. Containers can't straddle scopes (SPEC §3.9), so each scope is a
//! stack walk with no lookahead across scopes:
//!
//! 1. **Titles** ([`titles`]): a one-line `.Title` paragraph directly above a
//!    directive that accepts a title is attached to it.
//! 2. **Containers and groups** ([`nest`]): the walk keeps a stack of open
//!    containers. An opener pushes one (or, for a groupable directive, joins
//!    the nearest open group of the same name), an end line pops one, and
//!    whatever is still open when the scope ends is reported.
//! 3. **Binding** ([`bind`]) and **list heuristics** ([`lists`]): once a list
//!    of sibling blocks is final (the scope's, a container's, or an arm's),
//!    each line-form directive gets its binding and the checks that need
//!    neighbors run.
//!
//! Everything here reads one file and the directive schemas. What needs
//! another file, or the content model's data, is for later phases.

mod bind;
mod lists;
mod nest;
mod titles;

pub use bind::{bound_block, bound_heading};

use tessera_core::{Issue, LineIndex, Location, Span};

use crate::options::ParseOptions;
use crate::tree::*;

/// Runs the structure pass over the blocks of a converted document.
///
/// Issues it finds are appended to `issues`, which the caller sorts.
pub(crate) fn run(
    source: &str,
    options: &ParseOptions,
    blocks: Vec<Block>,
    issues: &mut Vec<Issue>,
) -> Vec<Block> {
    let mut pass = Pass {
        source,
        options,
        index: LineIndex::new(source),
        issues,
        open: Vec::new(),
        orphans: Vec::new(),
    };
    pass.scope(blocks, Scope::Document)
}

/// What kind of list of sibling blocks the pass is working on.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Scope {
    Document,
    ListItem,
    BlockQuote,
    Container,
    Arm,
}

impl Scope {
    /// How messages name the scope's end: "before {end}".
    fn end(self) -> &'static str {
        match self {
            Scope::Document => "the end of the document",
            Scope::ListItem => "the end of the list item",
            Scope::BlockQuote => "the end of the block quote",
            Scope::Container | Scope::Arm => "the end of the container",
        }
    }

    /// How messages name the scope itself.
    fn noun(self) -> &'static str {
        match self {
            Scope::Document => "document",
            Scope::ListItem => "list item",
            Scope::BlockQuote => "block quote",
            Scope::Container => "container",
            Scope::Arm => "arm",
        }
    }
}

pub(crate) struct Pass<'a> {
    source: &'a str,
    options: &'a ParseOptions,
    index: LineIndex,
    issues: &'a mut Vec<Issue>,
    /// The keywords of the containers open right now, outermost first, across
    /// every scope being worked on. A group counts once.
    open: Vec<String>,
    /// Containers left unclosed when their scope ended, as (keyword, opener),
    /// outermost first. A later end line in another scope may be the one
    /// their author meant (SPEC §3.9, resolved Q19).
    orphans: Vec<(String, Span)>,
}

impl<'a> Pass<'a> {
    fn report(&mut self, issue: Issue) {
        self.issues.push(issue);
    }

    fn location(&self, span: Span) -> Location {
        Location::new(self.options.file, span)
    }

    /// The 0-based line of an offset.
    fn line(&self, offset: usize) -> u32 {
        self.index.line_col(offset).map_or(0, |p| p.line)
    }

    /// Whether the line of `later_start` is the line right after the one
    /// `earlier_end` is on: nothing, not even a blank line, sits between.
    fn touches(&self, earlier_end: usize, later_start: usize) -> bool {
        self.line(later_start) == self.line(earlier_end).saturating_add(1)
    }

    /// Structures one list of sibling blocks.
    fn scope(&mut self, blocks: Vec<Block>, scope: Scope) -> Vec<Block> {
        let blocks = self.attach_titles(blocks);
        let mut frames: Vec<nest::Frame> = Vec::new();
        let mut root: Vec<Block> = Vec::new();
        for block in blocks {
            let Block { span, kind } = block;
            match kind {
                BlockKind::Directive(line) => self.directive(span, line, &mut frames, &mut root),
                BlockKind::End(end) => self.end_line(span, end, &mut frames, &mut root),
                BlockKind::BlockQuote(quote) => {
                    let children = self.scope(quote.children, Scope::BlockQuote);
                    let kind = BlockKind::BlockQuote(BlockQuote { children });
                    nest::push(&mut frames, &mut root, Block { span, kind });
                }
                BlockKind::List(mut list) => {
                    for item in &mut list.items {
                        let children = std::mem::take(&mut item.children);
                        item.children = self.scope(children, Scope::ListItem);
                    }
                    let kind = BlockKind::List(list);
                    nest::push(&mut frames, &mut root, Block { span, kind });
                }
                kind => nest::push(&mut frames, &mut root, Block { span, kind }),
            }
        }
        self.close_unclosed(frames, scope, &mut root);
        self.finish_children(&mut root, scope);
        root
    }

    /// Everything that needs a final list of siblings.
    fn finish_children(&mut self, children: &mut [Block], scope: Scope) {
        self.bind(children, scope);
        self.list_heuristics(children);
    }
}
