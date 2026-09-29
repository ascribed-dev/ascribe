//! The `ascribe/preview` request: a page rendered as the site renders it, for
//! the editor's preview (phase 25).
//!
//! The answer comes from the current snapshot, so it includes unsaved edits,
//! and goes through the same steps as `ascribe build --emit site`: the page is
//! resolved for the build (phase 12), written as site markdown by
//! [`SiteEmitter`] (phase 20), and rendered by [`render_site_html`], which
//! implements the site-render contract the Astro plugin implements. Nothing
//! is written to disk, and no third renderer exists.
//!
//! The site output refers to assets by paths relative to the page (asset
//! contract §3), which a webview can't load. So the answer lists every asset
//! the page uses, each with the reference exactly as the HTML writes it and
//! the source file it resolves to, resolved from the file the reference is
//! written in, so a fragment's image is found (contract §7). The client turns
//! those files into webview URLs.

use std::path::{Path, PathBuf};

use lsp_types::{TextDocumentIdentifier, Uri};
use serde::{Deserialize, Serialize};
use serde_json::Value as Json;
use tessera_core::{AssetUse, LineIndex, RelPath, WideEncoding};
use tessera_emit::assets::encode_path;
use tessera_emit::{EmitContext, SiteEmitter, emit_page, render_site_html};
use tessera_model::{AvailabilityMode, Build, ContentModel, VariantMode};
use tessera_resolve::{AstroRouter, DropReason, FileKind, LinkTarget, ResolvedBlock, Snapshot};

use crate::core::Core;
use crate::uri::{normalize, relative_to, uri_to_path};

/// The request's method name.
pub const METHOD: &str = "ascribe/preview";

/// The parameters of `ascribe/preview`.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PreviewParams {
    /// The document to preview: a page of the project.
    pub text_document: TextDocumentIdentifier,
    /// The build to render for. Without one, the editor's build (`[editor]
    /// build`).
    #[serde(default)]
    pub build: Option<String>,
}

/// The answer to `ascribe/preview`. It always says which builds exist and
/// where the project's files are, so a client can offer the picker and know
/// which directories the preview may read; `page` is there when there is
/// something to show, and `problems` says why not, or what is missing.
#[derive(Debug, Default, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct PreviewResult {
    /// The build the answer is for; empty when there is no project.
    pub build: String,
    /// Every build of the content model, in the order it declares them.
    pub builds: Vec<PreviewBuild>,
    /// The project's `ascribe.toml` directory, as a path.
    pub project_root: Option<String>,
    /// The content root, as a path. Nothing outside it is served to the
    /// preview.
    pub content_root: Option<String>,
    /// The version of the open document the answer was computed from, or
    /// `null` when the file isn't open (its text is the disk's). A client
    /// that sent version *n* and gets an older one has raced its own edit
    /// and asks again.
    pub document_version: Option<i32>,
    /// The rendered page.
    pub page: Option<PreviewPage>,
    /// Problems with showing the page: why there is none, or what in it
    /// can't be shown.
    pub problems: Vec<PreviewProblem>,
}

/// A build of the content model, for a picker.
#[derive(Debug, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct PreviewBuild {
    /// The build's name.
    pub name: String,
    /// Whether it is the editor's build (`[editor] build`): the picker's
    /// default.
    pub editor: bool,
    /// What it does, in the content model's terms: `variants: switch,
    /// availability: badge`.
    pub description: String,
}

/// A rendered page.
#[derive(Debug, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct PreviewPage {
    /// The page's content path.
    pub path: String,
    /// The page's route on the site.
    pub route: String,
    /// The page's title (its frontmatter `title`, phrases substituted).
    pub title: Option<String>,
    /// The frontmatter the site output writes, as JSON: `available` is the
    /// list of targets a layout passes to `<ascribe-availability>` (Q142).
    pub frontmatter: Json,
    /// The page's content as HTML: the site markdown after
    /// [`render_site_html`], without its frontmatter and without a layout.
    pub html: String,
    /// Every asset the page uses.
    pub assets: Vec<PreviewAsset>,
    /// The page links in the content, so a click opens the file.
    pub links: Vec<PreviewLink>,
    /// The headings written in the previewed file itself, in order, for
    /// following the cursor.
    pub sections: Vec<PreviewSection>,
}

/// An asset the page uses.
#[derive(Debug, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct PreviewAsset {
    /// The reference as the HTML writes it, before any `#fragment`: an
    /// `<img src>` for an image, an `<a href>` for a link target. Relative
    /// to the page for an image, root-relative for a link target (asset
    /// contract §3.2). Percent-encoded as a URL is.
    pub reference: String,
    /// The source file, as an absolute path: the file the reference names,
    /// resolved from the file it is written in.
    pub path: String,
    /// `image` or `link`.
    pub kind: &'static str,
    /// Whether the file is inside the content root, which is all the
    /// preview may read. A file outside it (asset contract §2, step 5) is
    /// reported in `problems` and isn't shown.
    pub servable: bool,
}

/// A link to another page.
#[derive(Debug, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct PreviewLink {
    /// The `href` as the HTML writes it: the route, and `#` and the page id
    /// when it names a heading.
    pub href: String,
    /// The target page, as an absolute path.
    pub path: String,
    /// The page id of the heading the link names.
    pub id: Option<String>,
}

/// A heading written in the previewed file.
#[derive(Debug, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct PreviewSection {
    /// The heading's id on the page (its `id` in the HTML).
    pub id: String,
    /// The line it starts on, from 0.
    pub line: u32,
}

/// A problem with showing a page.
#[derive(Debug, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct PreviewProblem {
    /// `error`, `warning`, or `info`.
    pub severity: &'static str,
    /// What the author should read.
    pub message: String,
}

impl PreviewProblem {
    fn info(message: impl Into<String>) -> PreviewProblem {
        PreviewProblem {
            severity: "info",
            message: message.into(),
        }
    }

    fn warning(message: impl Into<String>) -> PreviewProblem {
        PreviewProblem {
            severity: "warning",
            message: message.into(),
        }
    }

    fn error(message: impl Into<String>) -> PreviewProblem {
        PreviewProblem {
            severity: "error",
            message: message.into(),
        }
    }
}

/// What the request needs from the server's state, cloned under the lock so
/// the rendering happens without it.
pub(crate) struct Target {
    snapshot: Snapshot,
    model: std::sync::Arc<ContentModel>,
    root: PathBuf,
    content_root: PathBuf,
    /// The document's path on disk.
    document: PathBuf,
    document_version: Option<i32>,
    /// A project has loaded, and the document is inside its content root
    /// or at least its project root.
    content_path: Option<RelPath>,
    /// Why there is no project, when there isn't one.
    unavailable: Option<String>,
}

impl Core {
    /// Takes what `ascribe/preview` needs for a document.
    pub(crate) fn preview_target(&self, uri: &Uri) -> Result<Target, PreviewResult> {
        let document = uri_to_path(uri).map(|p| normalize(&p));
        let Some(loaded) = self.loaded.as_ref() else {
            let reason = match &self.model_problem {
                Some(_) => {
                    "ascribe.toml has errors, so there is no project to preview. Fix them and the preview follows."
                }
                None => "This workspace has no ascribe.toml, so there is no project to preview.",
            };
            return Err(PreviewResult {
                problems: vec![PreviewProblem::error(reason)],
                ..PreviewResult::default()
            });
        };
        let document = document.unwrap_or_default();
        let content_root = normalize(&loaded.root.join(loaded.layout.content_root.as_str()));
        let content_path = relative_to(&content_root, &document)
            .and_then(|rel| RelPath::parse(&rel).ok())
            .filter(RelPath::is_inside);
        Ok(Target {
            snapshot: loaded.inc.snapshot(),
            model: loaded.model.clone(),
            root: loaded.root.clone(),
            content_root,
            document_version: self.docs.get(&document).map(|d| d.version),
            document,
            content_path,
            unavailable: self.model_problem.as_ref().map(|_| {
                "ascribe.toml currently has errors; this is the last version of the project that loaded."
                    .to_owned()
            }),
        })
    }
}

/// Answers `ascribe/preview` for a target.
pub(crate) fn preview(target: &Target, build_name: Option<&str>) -> PreviewResult {
    let model = &*target.model;
    let editor = model.editor_default_build();
    let mut result = PreviewResult {
        build: editor.name.clone(),
        builds: model
            .builds
            .iter()
            .map(|b| PreviewBuild {
                name: b.name.clone(),
                editor: b.name == editor.name,
                description: describe(b),
            })
            .collect(),
        project_root: Some(path_text(&target.root)),
        content_root: Some(path_text(&target.content_root)),
        document_version: target.document_version,
        page: None,
        problems: Vec::new(),
    };
    if let Some(note) = &target.unavailable {
        result.problems.push(PreviewProblem::warning(note.clone()));
    }
    let build = match build_name {
        None => editor,
        Some(name) => match model.build(name) {
            Some(build) => build,
            None => {
                let names: Vec<&str> = model.builds.iter().map(|b| b.name.as_str()).collect();
                result.problems.push(PreviewProblem::error(format!(
                    "The content model has no build named {name}. Its builds are {}.",
                    names.join(", ")
                )));
                return result;
            }
        },
    };
    result.build = build.name.clone();

    let Some(path) = &target.content_path else {
        result.problems.push(PreviewProblem::info(format!(
            "{} isn't a source of the project (it is outside the content root), so there is no page to preview.",
            target.document.display()
        )));
        return result;
    };
    let snapshot = &target.snapshot;
    let Some(index) = snapshot.file(path) else {
        result.problems.push(PreviewProblem::info(format!(
            "{path} isn't a page of the project."
        )));
        return result;
    };
    if index.kind == FileKind::Fragment {
        let pages = snapshot.including_pages(path);
        let message = if pages.is_empty() {
            format!("{path} is a fragment that no page includes, so it is not part of any page.")
        } else {
            let names: Vec<String> = pages.iter().map(ToString::to_string).collect();
            format!(
                "{path} is a fragment. Preview a page that includes it: {}.",
                names.join(", ")
            )
        };
        result.problems.push(PreviewProblem::info(message));
        return result;
    }
    if let Some(reason) = snapshot.dropped(path, build) {
        let why = match reason {
            DropReason::Variant => {
                "its variant frontmatter names a dimension the build selects, and none of the selected values"
            }
            DropReason::Unavailable => {
                "its available frontmatter makes it unavailable for the build's target and version"
            }
        };
        result.problems.push(PreviewProblem::info(format!(
            "The build {} doesn't publish {path}: {why}. Pick another build to preview it.",
            build.name
        )));
        return result;
    }

    let router = AstroRouter::from_consumer(&model.consumer);
    let Some(page) = snapshot.resolve_page(path, build, &router) else {
        result.problems.push(PreviewProblem::info(format!(
            "The build {} doesn't publish {path}.",
            build.name
        )));
        return result;
    };
    let emitter = SiteEmitter::new(model);
    let cx = EmitContext::new(snapshot.project(), &target.root, build);
    let emitted = match emit_page(&emitter, &cx, &page) {
        Ok(emitted) => emitted,
        Err(e) => {
            result.problems.push(PreviewProblem::error(e.to_string()));
            return result;
        }
    };
    // A page is also unreadable to the router when another one has its route
    // (Q143); the preview shows the page anyway and says so.
    let collisions = emitter.profile().astro_router().collisions(
        snapshot
            .pages()
            .filter(|p| snapshot.dropped(&p.path, build).is_none())
            .map(|p| &p.path),
    );
    for (route, pages) in collisions {
        if pages.iter().any(|p| *p == path) {
            let names: Vec<String> = pages.iter().map(ToString::to_string).collect();
            result.problems.push(PreviewProblem::warning(format!(
                "{route} is the route of {}, so the site output can't publish them together.",
                names.join(" and ")
            )));
        }
    }

    let (frontmatter, body) = split_frontmatter(&emitted.text);
    let frontmatter = serde_yaml::from_str::<serde_yaml::Value>(frontmatter)
        .ok()
        .and_then(|v| serde_json::to_value(v).ok())
        .filter(Json::is_object)
        .unwrap_or_else(|| Json::Object(Default::default()));

    let mut assets: Vec<PreviewAsset> = Vec::new();
    for placed in &emitted.assets {
        let reference = match placed.usage {
            AssetUse::Image => encode_path(&placed.placement.reference),
            AssetUse::Link => placed
                .placement
                .url
                .clone()
                .unwrap_or_else(|| encode_path(&placed.placement.reference)),
        };
        if assets.iter().any(|a| a.reference == reference) {
            continue;
        }
        let file = normalize(&cx.asset_source(&placed.source));
        let servable = relative_to(&target.content_root, &file)
            .and_then(|rel| RelPath::parse(&rel).ok())
            .is_some_and(|rel| rel.is_inside());
        if !servable {
            result.problems.push(PreviewProblem::warning(format!(
                "{} is outside the content root, so the preview can't show it. The published site does.",
                placed.source
            )));
        }
        assets.push(PreviewAsset {
            reference,
            path: path_text(&file),
            kind: match placed.usage {
                AssetUse::Image => "image",
                AssetUse::Link => "link",
            },
            servable,
        });
    }

    let mut links: Vec<PreviewLink> = Vec::new();
    let mut sections: Vec<PreviewSection> = Vec::new();
    let lines = LineIndex::new(&index.source);
    let content_root = &target.content_root;
    page.visit(&mut |block: &ResolvedBlock| {
        for link in &block.links {
            if let LinkTarget::Page {
                page: to, id, url, ..
            } = &link.target
                && !links.iter().any(|l| l.href == *url)
            {
                links.push(PreviewLink {
                    href: url.clone(),
                    path: path_text(&normalize(&content_root.join(to.as_str()))),
                    id: id.clone(),
                });
            }
        }
        if let Some(heading) = &block.heading
            && block.file == page.file
            && block.via.is_empty()
            && !heading.page_id.is_empty()
            && let Some(at) = lines.wide_line_col(WideEncoding::Utf32, block.span.start())
        {
            sections.push(PreviewSection {
                id: heading.page_id.clone(),
                line: at.line,
            });
        }
    });

    result.page = Some(PreviewPage {
        path: path.to_string(),
        route: page.route.clone(),
        title: page.title.clone(),
        frontmatter,
        html: render_site_html(body),
        assets,
        links,
        sections,
    });
    result
}

/// Splits the site output's frontmatter (`---`, YAML, `---`, a blank line)
/// from the markdown after it.
fn split_frontmatter(text: &str) -> (&str, &str) {
    let Some(rest) = text.strip_prefix("---\n") else {
        return ("", text);
    };
    // The YAML serializer indents every continuation line, so a line that is
    // exactly `---` closes the block.
    let close = if rest.starts_with("---\n") {
        Some(0)
    } else {
        rest.find("\n---\n").map(|i| i + 1)
    };
    match close {
        Some(at) => (&rest[..at], rest[at + 4..].trim_start_matches('\n')),
        None => ("", text),
    }
}

/// A build in the content model's terms.
fn describe(build: &Build) -> String {
    let variants = match &build.variants {
        VariantMode::Switch => "switch".to_owned(),
        VariantMode::Select(selection) => selection
            .iter()
            .map(|(dimension, values)| format!("{dimension}={}", values.join("|")))
            .collect::<Vec<_>>()
            .join(", "),
    };
    let availability = match &build.availability {
        AvailabilityMode::Badge => "badge".to_owned(),
        AvailabilityMode::Filter { target, version } => match version {
            Some(version) => format!("filter {target} {}", version.text),
            None => format!("filter {target}"),
        },
    };
    format!("variants: {variants}; availability: {availability}")
}

fn path_text(path: &Path) -> String {
    path.to_string_lossy().into_owned()
}

#[cfg(test)]
mod tests {
    use super::split_frontmatter;

    #[test]
    fn frontmatter_is_split_from_the_body() {
        assert_eq!(
            split_frontmatter("---\ntitle: A\n---\n\n# Body\n"),
            ("title: A\n", "# Body\n")
        );
        assert_eq!(split_frontmatter("# Body\n"), ("", "# Body\n"));
        assert_eq!(split_frontmatter("---\n---\n\ntext\n"), ("", "text\n"));
    }
}
