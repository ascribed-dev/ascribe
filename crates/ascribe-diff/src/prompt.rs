//! Agent prompts about changes: what **Prompt agent** puts where a reviewer
//! sends it to their agent, and `ascribe diff --format prompt` writes.
//!
//! Three prompts, each built from what a comparison found:
//!
//! - [`page`]: one changed page of one build, each change in a line;
//! - [`fragment_reach`]: a changed fragment and the pages that show it;
//! - [`pages`]: every changed page, one line each.
//!
//! They follow the agent prompt format (`project-docs/agents/README.md`), as
//! `ascribe_check::prompt`'s prompts about problems do, and use its parts:
//! the task, where, what Ascribe knows, how to finish. They ask for a report,
//! not edits: a reviewer hands a change to their agent to read it as readers
//! will. A prompt points rather than dumps: it names the page, each change's
//! lines, and the commands that show the rest (`ascribe render`, `ascribe
//! diff`).

use ascribe_check::prompt::{Context, MAX_LISTED, assemble, code_list, shell_word};
use ascribe_core::{RelPath, percent_decode};

use crate::{Anchor, BaseInfo, BuildDiff, Change, ChangeKind, PageDiff, PageStatus};

/// What the prompts need besides a page: the base, and where the content is.
#[derive(Clone, Copy, Debug)]
pub struct Review<'a> {
    /// What the comparison is with.
    pub base: &'a BaseInfo,
    /// The content root, relative to the project root; empty when it is the
    /// project root. Content paths are shown under it.
    pub content_root: &'a RelPath,
}

impl Review<'_> {
    /// A content path, or another name `because` gives, as a prompt shows
    /// it: a content path relative to the project root; a snippet's address
    /// and `ascribe.toml` as they are.
    fn shown(&self, name: &str) -> String {
        if name == crate::compare::MODEL_FILE || name.starts_with("code:") {
            return name.to_owned();
        }
        match RelPath::parse(name).and_then(|p| self.content_root.join(p.as_str())) {
            Ok(path) => path.to_string(),
            Err(_) => name.to_owned(),
        }
    }

    /// A block's place: `guides/install.md:12-14`, and the include that
    /// shows it on the page when it's written in another file.
    fn place(&self, anchor: &Anchor) -> String {
        let mut text = self.source(&anchor.source);
        if let Some(outer) = anchor.via.first() {
            text.push_str(&format!(", shown through {}", self.source(outer)));
        }
        text
    }

    /// `<path>:<lines>` with its path decoded and shown.
    fn source(&self, source: &str) -> String {
        match source.rsplit_once(':') {
            Some((path, lines)) => {
                // One line is written as one number, as the problem prompts
                // write it.
                let lines = match lines.split_once('-') {
                    Some((first, last)) if first == last => first,
                    _ => lines,
                };
                format!("{}:{lines}", self.shown(&percent_decode(path)))
            }
            None => self.shown(&percent_decode(source)),
        }
    }

    /// The base: `Base: `main`, from where this branch left it (1a2b3c4)`.
    fn base_line(&self) -> String {
        let base = self.base;
        match &base.merge_base {
            Some(merge_base) => format!(
                "Base: `{}`, from where this branch left it ({})",
                base.requested,
                short(merge_base)
            ),
            None => format!("Base: `{}` ({})", base.requested, short(&base.commit)),
        }
    }

    /// `ascribe diff` against the same base, from the repository's root,
    /// with `options` after it.
    fn diff_command(&self, context: &Context, builds: &[&str], options: &[&str]) -> String {
        let mut words = vec!["ascribe".to_owned(), "diff".to_owned()];
        if let Some(folder) = &context.folder {
            words.push("--config".to_owned());
            words.push(shell_word(&format!("{folder}/ascribe.toml")));
        }
        words.push("--base".to_owned());
        words.push(shell_word(&self.base.requested));
        if self.base.merge_base.is_none() {
            words.push("--base-exact".to_owned());
        }
        for build in builds {
            words.push("--build".to_owned());
            words.push(shell_word(build));
        }
        words.extend(options.iter().map(|o| (*o).to_owned()));
        words.join(" ")
    }
}

/// A commit as people write it: its first seven characters.
fn short(commit: &str) -> String {
    commit.chars().take(7).collect()
}

/// `ascribe render` of a page (shown from the project root), as `build`'s
/// reader sees it, from the repository's root.
fn render_command(context: &Context, page: &str, build: &str) -> String {
    format!(
        "ascribe render {} --build {}",
        shell_word(&context.in_repository(page)),
        shell_word(build)
    )
}

/// The prompt about one changed page of `build`: "Review what this change
/// does to `<page>`, as a reader of build `<build>` sees it."
pub fn page(context: &Context, review: &Review<'_>, build: &str, page: &PageDiff) -> String {
    let path = review.shown(&page.path);
    let mut place = vec![format!("Where: {path}")];
    unsaved_lines(context, review, page, &mut place);
    context.project_lines(&mut place);

    let mut known = vec![review.base_line()];
    known.push(match page.status {
        PageStatus::Added => "The page is new: the base doesn't publish it.".to_owned(),
        PageStatus::Removed => "The page is removed: the build doesn't publish it now.".to_owned(),
        PageStatus::Changed => format!("What changed: {}", page.summary()),
    });
    if !page.because.is_empty() {
        let causes: Vec<String> = page.because.iter().map(|c| review.shown(c)).collect();
        known.push(format!(
            "{}: {}",
            if page.own_file_changed {
                "Also changed through"
            } else {
                "Changed through"
            },
            code_list(&causes)
        ));
    }
    if page.status != PageStatus::Removed {
        known.push(format!(
            "To read it: `{}`",
            render_command(context, &path, build)
        ));
    }
    let rest = review.diff_command(context, &[build], &["--format", "json"]);
    if !page.changes.is_empty() {
        known.push("Changes:".to_owned());
        known.extend(
            page.changes
                .iter()
                .take(MAX_LISTED)
                .map(|c| change_line(review, c)),
        );
        if page.changes.len() > MAX_LISTED {
            known.push(format!(
                "and {} more: `{rest}`",
                page.changes.len() - MAX_LISTED
            ));
        }
    }

    assemble(
        &format!(
            "Review what this change does to `{path}`, as a reader of build `{build}` sees it."
        ),
        &place,
        &known,
        &format!("(cut: run `{rest}` for the rest)"),
        &context.report_finish(),
    )
}

/// One change, in a line: its kind and where it is.
fn change_line(review: &Review<'_>, change: &Change) -> String {
    let at = |anchor: &Option<Anchor>| anchor.as_ref().map(|a| review.place(a));
    match (change.kind, at(&change.now), at(&change.was)) {
        (ChangeKind::Moved, Some(now), Some(was)) => {
            format!("- moved: {now}, from {was} in the base")
        }
        (ChangeKind::Removed, _, Some(was)) => format!("- removed: {was} in the base"),
        (kind, Some(now), _) => format!("- {}: {now}", kind_word(kind)),
        (kind, None, Some(was)) => format!("- {}: {was} in the base", kind_word(kind)),
        (kind, None, None) => format!("- {}", kind_word(kind)),
    }
}

fn kind_word(kind: ChangeKind) -> &'static str {
    match kind {
        ChangeKind::Changed => "changed",
        ChangeKind::Added => "added",
        ChangeKind::Removed => "removed",
        ChangeKind::Moved => "moved",
    }
}

/// The lines that say the page, or a file its change comes through, has
/// unsaved changes: the agent reads the files on disk.
fn unsaved_lines(context: &Context, review: &Review<'_>, page: &PageDiff, lines: &mut Vec<String>) {
    let path = review.shown(&page.path);
    let unsaved: Vec<String> = std::iter::once(path.clone())
        .chain(page.because.iter().map(|c| review.shown(c)))
        .filter(|p| context.unsaved.contains(p))
        .collect();
    match unsaved.as_slice() {
        [] => {}
        [only] if *only == path => {
            lines.push("The file has unsaved changes; save it before you start.".to_owned());
        }
        _ => lines.push(format!(
            "Some files have unsaved changes; save them before you start: {}",
            code_list(&unsaved)
        )),
    }
}

/// The prompt about a changed fragment (a content path) and the pages of
/// `build` that changed through it: "`<fragment>` changed, and N pages show
/// it. Check that the new text fits each." `None` when no page did.
pub fn fragment_reach(
    context: &Context,
    review: &Review<'_>,
    build: &BuildDiff,
    fragment: &RelPath,
) -> Option<String> {
    let pages: Vec<&PageDiff> = build
        .pages
        .iter()
        .filter(|p| p.status != PageStatus::Removed)
        .filter(|p| p.because.iter().any(|c| c == fragment.as_str()))
        .collect();
    if pages.is_empty() {
        return None;
    }
    let path = review.shown(fragment.as_str());
    let mut place = vec![format!("Where: {path}")];
    if context.unsaved.contains(&path) {
        place.push("The file has unsaved changes; save it before you start.".to_owned());
    }
    context.project_lines(&mut place);

    let in_repository = context.in_repository(&path);
    let mut known = vec![
        review.base_line(),
        format!(
            "To see what changed in it: `git diff {} -- {}`",
            short(
                review
                    .base
                    .merge_base
                    .as_ref()
                    .unwrap_or(&review.base.commit)
            ),
            shell_word(&in_repository)
        ),
        format!(
            "To read a page: `ascribe render <page> --build {}`",
            shell_word(&build.build)
        ),
        "Pages:".to_owned(),
    ];
    let refs = format!("ascribe refs {}", shell_word(&in_repository));
    known.extend(
        pages
            .iter()
            .take(MAX_LISTED)
            .map(|p| format!("- {}", review.shown(&p.path))),
    );
    if pages.len() > MAX_LISTED {
        known.push(format!("and {} more: `{refs}`", pages.len() - MAX_LISTED));
    }
    let (count, fits) = if pages.len() == 1 {
        ("1 page shows it".to_owned(), "it")
    } else {
        (format!("{} pages show it", pages.len()), "each")
    };
    Some(assemble(
        &format!("`{path}` changed, and {count}. Check that the new text fits {fits}."),
        &place,
        &known,
        &format!("(cut: run `{refs}` for the rest)"),
        &context.report_finish(),
    ))
}

/// The prompt about every changed page of `builds`: "Review what this change
/// does to the N pages it changes." `None` when nothing changed.
pub fn pages(context: &Context, review: &Review<'_>, builds: &[BuildDiff]) -> Option<String> {
    let changed: Vec<&BuildDiff> = builds.iter().filter(|b| !b.pages.is_empty()).collect();
    let mut distinct: Vec<&str> = changed
        .iter()
        .flat_map(|b| b.pages.iter().map(|p| p.path.as_str()))
        .collect();
    distinct.sort_unstable();
    distinct.dedup();
    if distinct.is_empty() {
        return None;
    }
    let names: Vec<&str> = builds.iter().map(|b| b.build.as_str()).collect();
    let mut place = Vec::new();
    context.project_lines(&mut place);

    let rest = review.diff_command(context, &names, &[]);
    let several = changed.len() > 1;
    let build_word = match changed.as_slice() {
        [only] => shell_word(&only.build),
        _ => "<build>".to_owned(),
    };
    let mut known = vec![
        review.base_line(),
        format!("To read a page: `ascribe render <page> --build {build_word}`"),
    ];
    let mut listed = 0;
    let total: usize = changed.iter().map(|b| b.pages.len()).sum();
    for build in &changed {
        if listed == MAX_LISTED {
            break;
        }
        known.push(if several {
            format!("In build `{}`:", build.build)
        } else {
            format!("Changed pages of build `{}`:", build.build)
        });
        for page in build.pages.iter().take(MAX_LISTED - listed) {
            known.push(format!(
                "- {}: {}",
                review.shown(&page.path),
                page.describe(|c| review.shown(c))
            ));
            listed += 1;
        }
    }
    if total > listed {
        known.push(format!("and {} more: `{rest}`", total - listed));
    }
    let task = if distinct.len() == 1 {
        "Review what this change does to the page it changes.".to_owned()
    } else {
        format!(
            "Review what this change does to the {} pages it changes.",
            distinct.len()
        )
    };
    Some(assemble(
        &task,
        &place,
        &known,
        &format!("(cut: run `{rest}` for the rest)"),
        &context.report_finish(),
    ))
}
