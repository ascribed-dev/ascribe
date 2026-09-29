//! What an include, link, or image names, and what's wrong with it.
//!
//! This is the one implementation of the rules for references: the source
//! index ([`crate::Project`]) and the file-level checks (`tessera-check`) both
//! call it, so `tessera check`, the build, and the language server can't
//! disagree about whether a file exists. It covers:
//!
//! - what a destination names: phrase substitution (SPEC §5.1, Q54),
//!   external or local, a source file or an asset, and whether it looks like
//!   a route (SPEC §5.2, Q22) ([`reference_target`], [`include_target`]);
//! - whether it's there: the asset contract's boundary (SPEC §9.4, Q10,
//!   [`Layout::is_allowed`]) and exact-case names on every platform
//!   ([`FileSystem::probe`]) ([`resolve_reference`]);
//! - the page a route most likely names (Q55);
//! - the file-level issue, if any ([`reference_issue`], [`include_issue`]),
//!   located at the destination as written (Q53).
//!
//! Page-level problems, such as a link to an id the target page lacks, are
//! the source index's ([`crate::Project::problems`]).

use tessera_core::{Fix, Issue, Location, RelPath, Span, TextEdit, diagnostics, percent_decode};
use tessera_model::ContentModel;
use tessera_syntax::{Inline, LinkDefinition, LinkForm, Phrase};

use crate::astro::AstroRouter;
use crate::fs::{FileSystem, Probe};
use crate::index::{Local, RefKind, Target};
use crate::layout::Layout;
use crate::project::{Missing, Resolution};

/// The source files a reference can name.
pub trait SourceSet {
    /// Whether there is a source file at this content path.
    fn contains(&self, path: &RelPath) -> bool;

    /// A source file whose content path differs from `path` only in case.
    fn case_twin(&self, path: &RelPath) -> Option<RelPath>;

    /// Every source file's content path, in path order. A route names the
    /// page whose route it is (`AstroRouter::page_for_route`), which takes the
    /// list. A set that can't list its files returns none, and routes are
    /// found by the conventional mapping alone.
    fn pages(&self) -> Vec<RelPath> {
        Vec::new()
    }
}

/// What an `@include` primary names.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IncludeTarget {
    /// The path as written, without the `#id`.
    pub written: String,
    /// The file it names, as a content path: relative to the including file,
    /// or to the content root when it starts with `/`. `None` for an empty
    /// path, or one that can't be a path.
    pub target: Option<RelPath>,
    /// The source id after `#`; `None` when there's none, or it's empty.
    pub section: Option<String>,
}

/// What an `@include` primary names, resolved from the file it's written in
/// (SPEC §4.2).
pub fn include_target(primary: &str, written_in: &RelPath) -> IncludeTarget {
    // SPEC §4.2 (resolved Q62): an include path is decoded as a link
    // destination is, so `my%20snippet.md` names `my snippet.md`, and an
    // empty id (`file.md#`) includes the whole file.
    let (path, section) = match primary.split_once('#') {
        Some((path, id)) => (path, Some(percent_decode(id)).filter(|id| !id.is_empty())),
        None => (primary, None),
    };
    let target = if path.is_empty() {
        None
    } else {
        let decoded = percent_decode(path);
        let base = if decoded.starts_with('/') {
            RelPath::root()
        } else {
            written_in.parent().unwrap_or_default()
        };
        base.join(decoded.trim_start_matches('/')).ok()
    };
    IncludeTarget {
        written: path.to_owned(),
        target,
        section,
    }
}

/// The file-level issue with an include, if any: its target isn't a source
/// file of the project. Only a source file can be included (SPEC §4.2,
/// resolved Q63): a file outside the content root, or that isn't Markdown,
/// gets the `not-source` message, whether or not it exists on disk.
///
/// `written` and `target` are an [`IncludeTarget`]'s. `at` is the primary.
pub fn include_issue(
    written: &str,
    target: Option<&RelPath>,
    sources: &dyn SourceSet,
    at: Location,
) -> Option<Issue> {
    let target = target?;
    if sources.contains(target) {
        return None;
    }
    let issue =
        Issue::new(diagnostics::INCLUDE_TARGET_MISSING, at).with_arg("path", written.to_owned());
    if !target.is_inside() || target.extension() != Some("md") {
        return Some(issue.with_variant("not-source"));
    }
    Some(match sources.case_twin(target) {
        Some(actual) => issue
            .with_variant("case")
            .with_arg("actual", actual.to_string()),
        None => issue,
    })
}

/// What a link's or image's destination names, from the destination and the
/// path of the file it's written in alone. `phrases` are the parser's phrase
/// candidates in the destination: for a reference form, its definition's
/// ([`destination_phrases`]).
pub fn reference_target(
    kind: RefKind,
    destination: &str,
    phrases: &[Phrase],
    written_in: &RelPath,
    model: &ContentModel,
) -> Target {
    crate::index::target_of(kind, destination, phrases, written_in, model)
}

/// The phrase candidates in a link's or image's destination.
///
/// An inline form, or an autolink, carries them itself (`own`). A reference
/// form's destination is its link reference definition's, so they are the
/// definition's (SPEC §5.1, resolved Q43): the definition whose normalized
/// label is the reference's label, else, if the label can't be read back from
/// the source, the first definition with the same decoded destination.
///
/// `span` is the whole link or image, `children` its text (a link) or alt
/// text as inlines (an image), `alt` an image's alt text as source, and
/// `label` the label of a full reference. Both `tessera check` and the source
/// index call this, so the two can't disagree.
#[allow(clippy::too_many_arguments)]
pub fn destination_phrases<'a>(
    source: &str,
    form: LinkForm,
    own: &'a [Phrase],
    label: Option<Span>,
    children: &[Inline],
    alt: Option<Span>,
    destination: &str,
    definitions: &'a [LinkDefinition],
) -> &'a [Phrase] {
    if !matches!(
        form,
        LinkForm::Full | LinkForm::Collapsed | LinkForm::Shortcut
    ) {
        return own;
    }
    let raw = match (form, label, alt) {
        (LinkForm::Full, Some(label), _) => source.get(label.range()),
        // An image's alt text is the label of the collapsed and shortcut
        // forms; a link's is its text.
        (_, _, Some(alt)) => source.get(alt.range()),
        _ => children
            .first()
            .zip(children.last())
            .and_then(|(first, last)| source.get(first.span.start()..last.span.end())),
    };
    let by_label = raw.map(normalize_label).and_then(|label| {
        definitions
            .iter()
            .find(|d| !label.is_empty() && d.normalized_label == label)
    });
    let definition = by_label.or_else(|| definitions.iter().find(|d| d.url == destination));
    definition.map_or(&[], |d| d.destination_phrases.as_slice())
}

/// CommonMark's label normalization: whitespace runs collapse to one space,
/// the ends are trimmed, and case is folded (lowercased, here).
fn normalize_label(label: &str) -> String {
    label
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase()
}

/// What a target names, once the project's files are known.
pub fn resolve_reference(
    kind: RefKind,
    target: &Target,
    written_in: &RelPath,
    model: &ContentModel,
    sources: &dyn SourceSet,
    fs: &dyn FileSystem,
    layout: &Layout,
) -> Resolution {
    let local = match target {
        Target::External => return Resolution::External,
        Target::Local(local) => local,
    };
    let Some(path) = &local.path else {
        return Resolution::AssetMissing(Missing::Absent);
    };
    // SPEC §5.3 (resolved Q59): an image with no path (`![a]()`, `![a](#x)`)
    // has no source, which is reported. A link with only a `#id` names the
    // file it's in.
    if kind == RefKind::Image && local.written.is_empty() {
        return Resolution::AssetMissing(Missing::Absent);
    }
    if local.source {
        return if sources.contains(path) {
            Resolution::Source {
                target: path.clone(),
                id: local.fragment.clone(),
                fragment: model.is_fragment(path.as_str()),
            }
        } else {
            Resolution::SourceMissing {
                actual: sources.case_twin(path),
            }
        };
    }
    // A file the build would copy.
    let missing = if !layout.is_allowed(path) {
        Missing::Outside
    } else {
        match fs.probe(&layout.project_path(path)) {
            Probe::File => {
                return Resolution::Asset {
                    path: path.clone(),
                    fragment: local.fragment.clone(),
                };
            }
            Probe::Missing => Missing::Absent,
            Probe::CaseMismatch(actual) => match content_path_of(layout, &actual) {
                Some(actual) => Missing::Case(actual),
                None => Missing::Absent,
            },
        }
    };
    if local.route_like && !matches!(missing, Missing::Case(_)) {
        return route(model, local, path, written_in, sources);
    }
    Resolution::AssetMissing(missing)
}

/// The page a route-like link most likely names, and the file-path link to
/// write instead.
// Resolved Q55, then Resolved Q148: the conventional mapping first (`route.md`, else
// `route/index.md`, whichever is a source file), then the consumer profile's
// router: the page whose route this is, with its base path and the entry ids
// Astro gives files (`guides/my-setup` for `Guides/My Setup.md`). Otherwise
// `route.md`, which doesn't exist.
fn route(
    model: &ContentModel,
    local: &Local,
    path: &RelPath,
    written_in: &RelPath,
    sources: &dyn SourceSet,
) -> Resolution {
    let as_file = if path.is_root() {
        RelPath::parse("index.md").unwrap_or_default()
    } else {
        RelPath::parse(&format!("{path}.md")).unwrap_or_else(|_| path.clone())
    };
    let as_directory = path.join("index.md").unwrap_or_else(|_| path.clone());
    let (page, page_exists) = [&as_file, &as_directory]
        .into_iter()
        .find(|p| sources.contains(p))
        .cloned()
        .or_else(|| page_of_route(model, local, path, sources))
        .map_or((as_file.clone(), false), |p| (p, true));
    let suggestion = if local.written.starts_with('/') {
        format!("/{page}")
    } else {
        let from = written_in.parent().unwrap_or_default();
        if page.is_inside() {
            page.relative_from(&from).map_or_else(
                || format!("/{page}"),
                |p| {
                    p.strip_prefix("./")
                        .map_or_else(|| p.clone(), str::to_owned)
                },
            )
        } else {
            // A page above the content root: up out of the file's directory
            // first, then the path's own `..` segments.
            format!("{}{page}", "../".repeat(from.segments().count()))
        }
    };
    let suggestion = match &local.fragment {
        Some(id) => format!("{suggestion}#{id}"),
        None => suggestion,
    };
    Resolution::Route {
        page: page.to_string(),
        suggestion,
        page_exists,
    }
}

/// The page the consumer profile's router says has this route. A destination
/// that starts with `/` is a route as written, which may include the base
/// path; any other is the route of the content path it names.
fn page_of_route(
    model: &ContentModel,
    local: &Local,
    path: &RelPath,
    sources: &dyn SourceSet,
) -> Option<RelPath> {
    if !path.is_inside() {
        return None;
    }
    let route = if local.written.starts_with('/') {
        local.written.clone()
    } else {
        format!("/{path}")
    };
    let pages: Vec<RelPath> = sources
        .pages()
        .into_iter()
        .filter(|p| !model.is_fragment(p.as_str()))
        .collect();
    AstroRouter::from_consumer(&model.consumer)
        .page_for_route(&route, &pages)
        .cloned()
}

/// A path relative to the project root as a content path, when it's inside
/// the content root or reaches it by `..`.
fn content_path_of(layout: &Layout, project_path: &RelPath) -> Option<RelPath> {
    let root: Vec<&str> = layout.content_root.segments().collect();
    let mine: Vec<&str> = project_path.segments().collect();
    let common = root.iter().zip(&mine).take_while(|(a, b)| a == b).count();
    let ups = "../".repeat(root.len() - common);
    RelPath::parse(&format!("{ups}{}", mine[common..].join("/"))).ok()
}

/// The file-level issue with a link or image, if any: a missing file, a
/// file outside the boundary or named with the wrong case, a link to a
/// fragment, or a link that looks like a route (with a fix, when the page it
/// names exists).
///
/// `at` is the whole link or image; `destination` is the destination as
/// written, when it's known ([`destination_span`]). Diagnostics about the
/// file are reported there (Q53).
pub fn reference_issue(
    kind: RefKind,
    target: &Target,
    written_destination: &str,
    resolution: &Resolution,
    at: Location,
    destination: Option<Span>,
) -> Option<Issue> {
    // SPEC §8.1 (resolved Q53): the destination as written for inline forms,
    // the whole link or image for reference forms.
    let here = Location::new(at.file, destination.unwrap_or(at.span));
    let written = match target {
        Target::Local(l) if l.written.is_empty() && !written_destination.is_empty() => {
            written_destination.to_owned()
        }
        Target::Local(l) if l.written.is_empty() => "(no source)".to_owned(),
        Target::Local(l) => l.written.clone(),
        _ => written_destination.to_owned(),
    };
    let missing_slug = match kind {
        RefKind::Link => diagnostics::LINK_TARGET_MISSING,
        RefKind::Image => diagnostics::IMAGE_SOURCE_MISSING,
    };
    match resolution {
        Resolution::External | Resolution::Asset { .. } => None,
        Resolution::SourceMissing { actual } => {
            let issue = Issue::new(missing_slug, here).with_arg("path", written);
            Some(match actual {
                Some(actual) => issue
                    .with_variant("case")
                    .with_arg("actual", actual.to_string()),
                None => issue,
            })
        }
        Resolution::AssetMissing(_)
            if kind == RefKind::Image
                && matches!(target, Target::Local(l) if l.written.is_empty()) =>
        {
            Some(
                Issue::new(missing_slug, here)
                    .with_variant("empty")
                    .with_arg("path", written),
            )
        }
        Resolution::AssetMissing(why) => {
            let issue = Issue::new(missing_slug, here).with_arg("path", written);
            Some(match why {
                Missing::Absent => issue,
                Missing::Case(actual) => issue
                    .with_variant("case")
                    .with_arg("actual", actual.to_string()),
                Missing::Outside => issue.with_variant("outside"),
            })
        }
        Resolution::Route {
            page,
            suggestion,
            page_exists,
        } => {
            let mut issue = Issue::new(diagnostics::LINK_ROUTE, here)
                .with_arg("page", page.clone())
                .with_arg("suggestion", suggestion.clone());
            if let (true, Some(span)) = (*page_exists, destination) {
                let new_text = if suggestion.contains([' ', '(', ')']) {
                    format!("<{suggestion}>")
                } else {
                    suggestion.clone()
                };
                issue = issue.with_fix(Fix {
                    title: "Link to the page's file instead of its route".into(),
                    file: at.file,
                    edits: vec![TextEdit::replace(span, new_text)],
                });
            }
            Some(issue)
        }
        Resolution::Source { fragment, .. } => {
            // SPEC §5.2 (resolved Q64): a `#id` alone in a fragment names a
            // heading of the fragment itself, which each including page
            // publishes; only a link to a fragment *file* is an error.
            let names_itself = matches!(target, Target::Local(l) if l.written.is_empty());
            (*fragment && !names_itself)
                .then(|| Issue::new(diagnostics::LINK_TO_FRAGMENT, here).with_arg("path", written))
        }
    }
}

/// The span of an inline link's or image's destination as written, for
/// pointing at it. `None` for other forms (a reference form's destination is
/// in its definition) and when the text doesn't read back as `destination`.
///
/// `span` is the whole link or image; `children` its text (a link) and `alt`
/// its alt text (an image).
pub fn destination_span(
    source: &str,
    span: Span,
    form: LinkForm,
    children: &[Inline],
    alt: Option<Span>,
    destination: &str,
) -> Option<Span> {
    if form != LinkForm::Inline {
        return None;
    }
    // The `]` that closes the text is where the children end, or the alt
    // text; look for `](` from there.
    let from = alt
        .map(Span::end)
        .or_else(|| children.last().map(|c| c.span.end()))
        .unwrap_or(span.start() + 1);
    let close = source.get(from..span.end())?.find("](")? + from;
    let mut i = close + 2;
    let bytes = source.as_bytes();
    while i < span.end() && matches!(bytes[i], b' ' | b'\t' | b'\n' | b'\r') {
        i += 1;
    }
    let start = i;
    let raw_end = if bytes.get(i) == Some(&b'<') {
        let rest = source.get(i + 1..span.end())?;
        i + 1 + rest.find('>')? + 1
    } else {
        let mut depth = 0usize;
        while i < span.end() {
            match bytes[i] {
                b'\\' => i += 1,
                b'(' => depth += 1,
                b')' if depth == 0 => break,
                b')' => depth -= 1,
                b' ' | b'\t' | b'\n' | b'\r' => break,
                _ => {}
            }
            i += 1;
        }
        i.min(span.end())
    };
    let raw = source.get(start..raw_end)?;
    let inner = raw
        .strip_prefix('<')
        .and_then(|r| r.strip_suffix('>'))
        .unwrap_or(raw);
    (unescape(inner) == destination).then(|| Span::new(start, raw_end))
}

/// CommonMark's backslash escapes: a backslash before ASCII punctuation.
fn unescape(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut chars = text.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\\'
            && let Some(next) = chars.peek()
            && next.is_ascii_punctuation()
        {
            continue;
        }
        out.push(c);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unescapes_punctuation_only() {
        assert_eq!(unescape(r"a\(b\)\x"), r"a(b)\x");
    }

    #[test]
    fn include_targets() {
        let from = RelPath::parse("guides/setup.md").expect("path");
        let t = include_target("_f.md#install", &from);
        assert_eq!(t.written, "_f.md");
        assert_eq!(t.target.map(|p| p.to_string()), Some("guides/_f.md".into()));
        assert_eq!(t.section.as_deref(), Some("install"));
        let t = include_target("/_f.md#", &from);
        assert_eq!(t.target.map(|p| p.to_string()), Some("_f.md".into()));
        assert_eq!(t.section, None);
        assert_eq!(include_target("#x", &from).target, None);
    }
}
