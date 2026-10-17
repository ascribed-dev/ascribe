//! The memo files about a page (SPEC §7.7), as prompts name them: a line
//! each, with the memo's path, what it's about, and its state when it isn't
//! placed, under a heading of its own. The agent reads the file; nothing
//! here puts a memo's text in a prompt.
//!
//! [`MemoPlace::all`] places each memo against the source index once, as
//! `ascribe outline` does, with the lines of the page, section, or block it's
//! on. A prompt about problems takes the memos [`about_file`] its page; one
//! about changes, those [`about_lines`] each change.

use ascribe_core::{LineIndex, RelPath, Span};
use ascribe_resolve::{MemoFile, MemoState, Named, Usable, place};

use super::{Context, shell_word};

/// The most memos a prompt lists.
pub const MAX_MEMOS: usize = 10;

/// A memo file about a source file, and where in it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MemoPlace {
    /// The memo file, from the project root.
    memo: String,
    /// Its `about`, as written.
    about: String,
    /// Whether it's kept on this machine only, in `.ascribe/memos/`.
    personal: bool,
    /// Placed, outdated, or detached.
    state: MemoState,
    /// The file it's about, by content path.
    file: RelPath,
    /// What it's on in that file.
    on: On,
}

/// What in a file a memo is on: the lines are from 1.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum On {
    /// The file as a whole.
    File,
    /// A heading's section.
    Section(u32, u32),
    /// The block its quote is in.
    Block(u32, u32),
}

impl MemoPlace {
    /// Each memo of `memos` about a source file of `index`, placed against
    /// it: every memo on a page, a heading, or a block, and each detached
    /// one whose `about` names a heading a file doesn't have. A memo about
    /// a content model entry is about no page. In the order of `memos`.
    pub fn all(index: &ascribe_resolve::Project, memos: &[MemoFile]) -> Vec<MemoPlace> {
        memos
            .iter()
            .filter_map(|memo| {
                let placement = place(memo, index)?;
                let (file, on) = match (&placement.target, &placement.named) {
                    (Some(Usable::File(file)), _) => (file.clone(), On::File),
                    (Some(Usable::Heading { file, id }), _) => {
                        let section = index.heading(file, id)?.section;
                        let (first, last) = lines(index, file, section)?;
                        (file.clone(), On::Section(first, last))
                    }
                    (Some(_), _) => return None,
                    (None, Named::Path { path, .. }) if index.file(path).is_some() => {
                        (path.clone(), On::File)
                    }
                    (None, _) => return None,
                };
                let on = match placement.block {
                    Some(block) => {
                        let (first, last) = lines(index, &file, block)?;
                        On::Block(first, last)
                    }
                    None => on,
                };
                Some(MemoPlace {
                    memo: memo.path.to_string(),
                    about: memo.about.as_ref()?.value.clone(),
                    personal: memo.personal(),
                    state: placement.state,
                    file,
                    on,
                })
            })
            .collect()
    }
}

/// The first and last lines of `span` in `file`, from 1.
fn lines(index: &ascribe_resolve::Project, file: &RelPath, span: Span) -> Option<(u32, u32)> {
    let source = &index.file(file)?.source;
    let at = LineIndex::new(source);
    let first = at.line_col(span.start())?.line;
    let last = at
        .line_col(span.end().saturating_sub(1).max(span.start()))?
        .line;
    Some((first + 1, last + 1))
}

/// The memos of `places` about the file at content path `file`, wherever in
/// it they are, detached ones too.
pub fn about_file<'p>(places: &'p [MemoPlace], file: &RelPath) -> Vec<&'p MemoPlace> {
    places.iter().filter(|p| p.file == *file).collect()
}

/// The memos of `places` about a change to lines `first` to `last` (from 1)
/// of the file at content path `file`: one on the file, one on the section
/// the lines are in, and, unless the change took the lines out
/// (`removed`), one on a block they overlap. A detached memo is on nothing.
pub fn about_lines<'p>(
    places: &'p [MemoPlace],
    file: &RelPath,
    (first, last): (u32, u32),
    removed: bool,
) -> Vec<&'p MemoPlace> {
    let overlaps = |a: u32, b: u32| first <= b && a <= last;
    places
        .iter()
        .filter(|p| p.file == *file && p.state != MemoState::Detached)
        .filter(|p| match p.on {
            On::File => true,
            On::Section(a, b) => overlaps(a, b),
            On::Block(a, b) => !removed && overlaps(a, b),
        })
        .collect()
}

/// What a list of memos asks of the agent.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ask {
    /// Read them before changing the page: a prompt about problems in one
    /// page.
    Page,
    /// The same, of several pages.
    Pages,
    /// Read them before judging a change, and check it still agrees with
    /// each: a prompt about changes.
    Change,
}

/// The lines that name `found`: a heading, then a line each, in path order
/// and each memo once, at most [`MAX_MEMOS`]; past that, how many more and
/// the `ascribe outline` command that lists them, about `outline`, the
/// page as the prompt shows it, or for several pages `None`. Empty when
/// there are none.
pub fn memo_lines(
    context: &Context,
    mut found: Vec<&MemoPlace>,
    ask: Ask,
    outline: Option<&str>,
) -> Vec<String> {
    found.sort_by(|a, b| a.memo.cmp(&b.memo));
    found.dedup_by(|a, b| a.memo == b.memo);
    if found.is_empty() {
        return Vec::new();
    }
    let mut lines = vec![
        match ask {
            Ask::Page => "Memos about this page, to read before changing it:",
            Ask::Pages => "Memos about these pages, to read before changing them:",
            Ask::Change => "Memos about what changed, to read before judging it:",
        }
        .to_owned(),
    ];
    for memo in found.iter().take(MAX_MEMOS) {
        let mut notes = Vec::new();
        if memo.personal {
            notes.push("personal, kept on this machine");
        }
        match memo.state {
            MemoState::Placed => {}
            MemoState::Outdated => notes.push("outdated: no block holds its quote now"),
            MemoState::Detached => notes.push("detached: what its `about` names isn't there"),
        }
        let mut line = format!("- {}, about {}", context.shown(&memo.memo), memo.about);
        if !notes.is_empty() {
            line.push_str(&format!(" ({})", notes.join("; ")));
        }
        if ask == Ask::Change {
            line.push_str(": check the change still agrees with it.");
        }
        lines.push(line);
    }
    if found.len() > MAX_MEMOS {
        let more = found.len() - MAX_MEMOS;
        lines.push(match outline {
            Some(page) => format!(
                "and {more} more: `{} outline {}` lists them",
                context.ascribe(),
                shell_word(page)
            ),
            None => format!(
                "and {more} more: `{} outline <page>` lists a page's",
                context.ascribe()
            ),
        });
    }
    lines
}
