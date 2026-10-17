//! "Shown in N pages": a CodeLens in a code file a `@snippet` shows, on
//! each region a snippet takes and on the first line of a file a snippet
//! takes whole, that counts the pages showing it and lists where.
//!
//! The client asks for these lenses in the files of the content model's
//! sources, which the server registers for ([`selector`]), without sending
//! their text: the regions are read from the file on disk, as a build reads
//! them, so lines added above a region leave its lens where it was until
//! the file is saved.

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use ascribe_core::RelPath;
use ascribe_core::path::{normalize, relative_path};
use ascribe_model::ContentModel;
use ascribe_resolve::snippet::ReadError;
use ascribe_resolve::{CodeFiles, FileKind, FileSystem};
use lsp_types::{CodeLens, Command, DocumentFilter, Location, Position, Range};
use serde_json::json;

use crate::nav::{Ctx, Lines};
use crate::uri::path_to_uri;

/// The client's command a lens carries: it shows its arguments, a file, a
/// position in it, and locations, as Find All References lists them. The
/// extension registers it (`packages/vscode/src/extension.ts`); the server
/// never runs it.
pub(crate) const SHOW_REFERENCES: &str = "ascribe.showReferences";

/// The document filters of the files the server answers lenses for: the
/// project's pages, for the lens on `@include`, and the files of each of
/// its sources, a folder with its `include` patterns, or the whole folder.
/// Patterns are absolute globs, with the folder's glob characters escaped.
pub(crate) fn selector(root: &Path, model: &ContentModel) -> Vec<DocumentFilter> {
    let mut filters = vec![DocumentFilter {
        language: Some("markdown".to_owned()),
        scheme: Some("file".to_owned()),
        pattern: Some(format!("{}/**", glob_folder(root))),
    }];
    for source in &model.sources {
        let folder = glob_folder(&normalize(&root.join(&source.path)));
        let patterns: Vec<String> = if source.include.is_empty()
            // A pattern's escapes aren't the editor's.
            || source.include.iter().any(|p| p.as_str().contains('\\'))
        {
            vec![format!("{folder}/**")]
        } else {
            source
                .include
                .iter()
                .map(|p| format!("{folder}/{}", p.as_str()))
                .collect()
        };
        for pattern in patterns {
            let filter = DocumentFilter {
                language: None,
                scheme: Some("file".to_owned()),
                pattern: Some(pattern),
            };
            if !filters.contains(&filter) {
                filters.push(filter);
            }
        }
    }
    filters
}

/// A folder as a glob: `/` between segments, and glob characters escaped,
/// as the extension writes a project's folder.
fn glob_folder(folder: &Path) -> String {
    let text = folder.to_string_lossy().replace('\\', "/");
    let mut out = String::with_capacity(text.len());
    for c in text.chars() {
        if matches!(c, '*' | '?' | '[' | '{') {
            out.push('[');
            out.push(c);
            out.push(']');
        } else {
            out.push(c);
        }
    }
    out.trim_end_matches('/').to_owned()
}

/// Whether a file on disk is in one of the model's sources' folders.
pub(crate) fn in_a_source(root: &Path, model: &ContentModel, path: &Path) -> bool {
    let Some(project) = relative_path(root, path) else {
        return false;
    };
    model.sources.iter().any(|source| {
        RelPath::parse(&source.path).is_ok_and(|folder| project.relative_to(&folder).is_some())
    })
}

/// The lenses of the code file at `path`: one for each region a `@snippet`
/// shows, on its start tag line, and one on the first line when a snippet
/// shows the whole file. None for a file in no source, or one the server
/// can't read, which it logs.
pub(crate) fn lenses(ctx: &Ctx, path: &Path) -> Vec<CodeLens> {
    let Some(root) = ctx.config.parent() else {
        return Vec::new();
    };
    let Some(project) = relative_path(root, &normalize(path)) else {
        return Vec::new();
    };
    let Some(uri) = path_to_uri(path) else {
        return Vec::new();
    };
    let fs: &dyn FileSystem = &*ctx.fs;
    let code = CodeFiles::new();
    let mut lines = Lines::new(ctx);
    let mut out = Vec::new();
    for source in &ctx.model.sources {
        let Ok(folder) = RelPath::parse(&source.path) else {
            continue;
        };
        let Some(rest) = project.relative_to(&folder) else {
            continue;
        };
        if !source.reads(rest.as_str()) {
            continue;
        }
        let shown = places(ctx, &source.name, rest.as_str());
        if shown.is_empty() {
            continue;
        }
        let file = match code.read(fs, &project) {
            Ok(file) => file,
            Err(why) => {
                crate::log::line(format_args!(
                    "no lenses in {}: {}",
                    path.display(),
                    match why {
                        ReadError::Missing => "it isn't there".to_owned(),
                        ReadError::Case(actual) => format!("its name on disk is {actual}"),
                        ReadError::NotText(reason) => reason,
                    }
                ));
                continue;
            }
        };
        for (region, shown) in shown {
            let line = match &region {
                None => 0,
                Some(name) => match file.tags.region(name) {
                    Some(region) => u32::try_from(region.start).unwrap_or(u32::MAX),
                    // The snippet's problem says the region isn't there.
                    None => continue,
                },
            };
            let locations: Vec<Location> = shown
                .places
                .iter()
                .filter_map(|(file, span)| {
                    Some(Location {
                        uri: ctx.uri_of(file)?,
                        range: lines.range(file, *span)?,
                    })
                })
                .collect();
            let at = Position::new(line, 0);
            out.push(CodeLens {
                range: Range::new(at, at),
                command: Some(Command {
                    title: title(shown.pages.len()),
                    command: SHOW_REFERENCES.to_owned(),
                    arguments: Some(vec![
                        json!(uri.as_str()),
                        json!(at),
                        serde_json::to_value(locations).unwrap_or_default(),
                    ]),
                }),
                data: None,
            });
        }
    }
    out.sort_by_key(|lens| lens.range.start.line);
    out
}

/// `Shown in 3 pages`.
fn title(pages: usize) -> String {
    match pages {
        0 => "Shown in no pages".to_owned(),
        1 => "Shown in 1 page".to_owned(),
        n => format!("Shown in {n} pages"),
    }
}

/// Where one region of a code file, or the whole file, is shown.
#[derive(Default)]
struct Shown {
    /// The pages that show it: a page with the `@snippet`, and each page
    /// that includes a file with one.
    pages: BTreeSet<RelPath>,
    /// The `@snippet` lines, in content path order and in document order.
    places: Vec<(RelPath, ascribe_core::Span)>,
}

/// Each region of the file at `path` in the source `source` that a
/// `@snippet` shows, by name, `None` for the whole file.
fn places(ctx: &Ctx, source: &str, path: &str) -> BTreeMap<Option<String>, Shown> {
    let mut out: BTreeMap<Option<String>, Shown> = BTreeMap::new();
    for file in ctx.snapshot.files() {
        for snippet in &file.snippets {
            let Some(Ok(address)) = &snippet.address else {
                continue;
            };
            if address.source != source || address.path != path {
                continue;
            }
            let shown = out.entry(address.region.clone()).or_default();
            shown.places.push((file.path.clone(), snippet.span));
            if file.kind == FileKind::Page {
                shown.pages.insert(file.path.clone());
            }
            shown.pages.extend(ctx.snapshot.including_pages(&file.path));
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_folder_is_a_glob_with_its_glob_characters_escaped() {
        assert_eq!(glob_folder(Path::new("/a/b[1]/{c}")), "/a/b[[]1]/[{]c}");
        assert_eq!(glob_folder(Path::new("/a/")), "/a");
    }

    #[test]
    fn a_title_counts_pages() {
        assert_eq!(title(0), "Shown in no pages");
        assert_eq!(title(1), "Shown in 1 page");
        assert_eq!(title(3), "Shown in 3 pages");
    }
}
