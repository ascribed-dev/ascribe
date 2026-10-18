//! The checks of memo files (SPEC §7.7), each reported on the memo file, at
//! its `about` or `quote` line: `memo-invalid`, a file that isn't one;
//! `memo-detached`, an `about` that names nothing; `memo-outdated`, a quote
//! no block holds; and `memo-long`, a body over the limit. They run for
//! every build, over the source index, on the memos the project's checks
//! read: the committed ones for `ascribe check`, and the personal ones too
//! in the editor.

use ascribe_core::{DiagnosticSlug, FileId, Issue, Location, Span, diagnostics};
use ascribe_model::suggest;
use ascribe_resolve::{
    EntryKind, Invalid, MemoFile, MemoState, Named, PREFIXES, Usable, folder_in_content, place,
};

use crate::Project;
use crate::project::memo_id;

/// The length over which `memo-long` reports a memo when the project sets no
/// `limit`, in characters.
pub const MEMO_LIMIT: u64 = 2_000;

/// The checks of this module.
pub(crate) const MEMO_CHECKS: &[DiagnosticSlug] = &[
    diagnostics::MEMO_INVALID,
    diagnostics::MEMO_DETACHED,
    diagnostics::MEMO_OUTDATED,
    diagnostics::MEMO_LONG,
];

/// Whether there's anything for the memo checks to look at: a memo file, or
/// a `memos/` folder inside the content root.
pub(crate) fn any(project: &Project) -> bool {
    !project.checked_memos().is_empty() || in_content(project).is_some()
}

/// The first source file in a `memos/` folder inside the content root.
fn in_content(project: &Project) -> Option<FileId> {
    let folder = folder_in_content(project.layout())?;
    project
        .sources()
        .iter()
        .filter(|s| s.path.starts_with(&folder))
        .min_by(|a, b| a.path.cmp(&b.path))
        .map(|s| s.id)
}

/// What the memo checks find, for each check `on` says runs.
pub(crate) fn check_memos(
    project: &Project,
    index: &ascribe_resolve::Project,
    on: &dyn Fn(DiagnosticSlug) -> bool,
    issues: &mut Vec<Issue>,
) {
    if on(diagnostics::MEMO_INVALID)
        && let Some(first) = in_content(project)
        && let Some(file) = project.sources().iter().find(|s| s.id == first)
    {
        let line = file.text.find('\n').unwrap_or(file.text.len());
        issues.push(
            Issue::new(
                diagnostics::MEMO_INVALID,
                Location::new(first, Span::new(0, line)),
            )
            .with_variant("content-root"),
        );
    }
    let limit = project
        .model()
        .checks
        .limit(diagnostics::MEMO_LONG)
        .unwrap_or(MEMO_LIMIT);
    for (i, memo) in project.checked_memos().iter().enumerate() {
        let at = |span: Span| Location::new(memo_id(i), span);
        if on(diagnostics::MEMO_INVALID) {
            for problem in &memo.problems {
                issues.push(invalid(memo, &problem.kind, at(problem.span)));
            }
        }
        let about_line = memo.about.as_ref().map_or(Span::new(0, 0), |a| a.line);
        if on(diagnostics::MEMO_LONG) {
            let length = memo.body_text().chars().count() as u64;
            if length > limit {
                issues.push(
                    Issue::new(diagnostics::MEMO_LONG, at(about_line))
                        .with_arg("length", length.to_string())
                        .with_arg("limit", limit.to_string()),
                );
            }
        }
        let Some(placement) = place(memo, index) else {
            continue;
        };
        let about = memo.about.as_ref().map_or("", |a| a.value.as_str());
        match placement.state {
            MemoState::Placed => {}
            MemoState::Detached if on(diagnostics::MEMO_DETACHED) => {
                issues.push(detached(index, &placement.named, about, at(about_line)));
            }
            MemoState::Outdated if on(diagnostics::MEMO_OUTDATED) => {
                let place = match &placement.target {
                    Some(Usable::Heading { id, .. }) => format!("section `#{id}`"),
                    _ => "page".to_owned(),
                };
                let quote_line = memo.quote.as_ref().map_or(about_line, |q| q.line);
                issues.push(
                    Issue::new(diagnostics::MEMO_OUTDATED, at(quote_line))
                        .with_arg("about", about.to_owned())
                        .with_arg("place", place),
                );
            }
            MemoState::Detached | MemoState::Outdated => {}
        }
    }
}

/// `memo-invalid` for one of a memo file's problems.
fn invalid(memo: &MemoFile, kind: &Invalid, at: Location) -> Issue {
    let issue = Issue::new(diagnostics::MEMO_INVALID, at);
    let about = || {
        memo.about
            .as_ref()
            .map_or("", |a| a.value.as_str())
            .to_owned()
    };
    match kind {
        Invalid::Unreadable(reason) => issue
            .with_variant("unreadable")
            .with_arg("reason", reason.clone()),
        Invalid::NoFrontmatter => issue,
        Invalid::Yaml(detail) => issue
            .with_variant("yaml")
            .with_arg("detail", detail.clone()),
        Invalid::NoAbout => issue.with_variant("about-missing"),
        Invalid::UnknownKey(key) => issue.with_variant("key").with_arg("key", key.clone()),
        Invalid::NotText(key) => issue
            .with_variant("not-text")
            .with_arg("key", (*key).to_owned()),
        Invalid::BadAbout(reason) => issue
            .with_variant("about")
            .with_arg("about", about())
            .with_arg("reason", reason.clone()),
        Invalid::QuoteWithoutText => issue.with_variant("quote").with_arg("about", about()),
    }
}

/// `memo-detached` for a memo whose `about`, `named`, names nothing.
fn detached(index: &ascribe_resolve::Project, named: &Named, about: &str, at: Location) -> Issue {
    let issue = Issue::new(diagnostics::MEMO_DETACHED, at).with_arg("about", about.to_owned());
    match named {
        Named::Path { path, id } => match (index.file(path), id) {
            (Some(file), Some(id)) => {
                let ids: Vec<String> = index
                    .link_headings(file)
                    .into_iter()
                    .map(|(_, h)| format!("`{}`", h.source_id))
                    .collect();
                let issue = issue
                    .with_arg("page", path.to_string())
                    .with_arg("id", id.clone());
                if ids.is_empty() {
                    issue.with_variant("heading-none")
                } else {
                    issue
                        .with_variant("heading")
                        .with_arg("ids", ids.join(", "))
                }
            }
            _ => {
                let paths: Vec<String> = index.files().map(|f| f.path.to_string()).collect();
                match suggest(path.as_str(), paths.iter().map(String::as_str)) {
                    Some(found) => issue
                        .with_variant("page-suggestion")
                        .with_arg("suggestion", found.to_owned()),
                    None => issue,
                }
            }
        },
        Named::Entry { kind, name } => {
            let model = index.model();
            let glossary = index.glossary();
            let names: Vec<&str> = match kind {
                EntryKind::Phrase => model.phrases.iter().map(|p| p.key.as_str()).collect(),
                EntryKind::Feature => model.features.iter().map(|f| f.key.as_str()).collect(),
                EntryKind::Term => glossary.terms.iter().map(|t| t.id.as_str()).collect(),
                EntryKind::Dimension => model.dimensions.iter().map(|d| d.name.as_str()).collect(),
                EntryKind::Note => model.notes.iter().map(|n| n.name.as_str()).collect(),
                EntryKind::Widget => model
                    .widgets
                    .iter()
                    .map(|w| w.schema.name.as_str())
                    .collect(),
            };
            let prefix = PREFIXES
                .iter()
                .find(|(_, k)| k == kind)
                .map_or("", |(p, _)| *p);
            // A term is a heading on a glossary page, not an entry of
            // ascribe.toml.
            let (plain, suggestion) = match kind {
                EntryKind::Term => ("term", "term-suggestion"),
                _ => ("entry", "entry-suggestion"),
            };
            match suggest(name, names) {
                Some(found) => issue
                    .with_variant(suggestion)
                    .with_arg("suggestion", format!("{prefix}:{found}")),
                None => issue.with_variant(plain),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_registry_states_the_default_limit() {
        let fix = crate::Registry::global()
            .get(diagnostics::MEMO_LONG)
            .and_then(|e| e.fix.clone())
            .unwrap_or_default();
        assert!(
            fix.contains("The limit is 2,000 characters") && MEMO_LIMIT == 2_000,
            "memo-long's fix names the default limit, {MEMO_LIMIT}"
        );
    }
}
