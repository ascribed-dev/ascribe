//! The `ascribe/buildView` request: what a build leaves out of a page, for
//! the editor's build lens.
//!
//! The answer comes from the current snapshot, so it includes unsaved edits,
//! and from the resolver's own decisions ([`Project::removed`] and
//! [`Project::dropped`]), the ones `ascribe build` makes: the variant arms
//! the build's selection removes, and the content, table rows included, its
//! availability filter removes. Each range covers what the build removes
//! whole, its directive lines and `@end` included. Content of an included
//! fragment isn't in the page's file and isn't listed.
//!
//! [`Project::removed`]: ascribe_resolve::Project::removed
//! [`Project::dropped`]: ascribe_resolve::Project::dropped

use ascribe_core::{LineIndex, Span};
use ascribe_emit::labels::availability_display;
use ascribe_model::{AvailabilityMode, Build, ContentModel, VariantMode};
use ascribe_resolve::{FileKind, Removal, Removed};
use lsp_types::{Range, TextDocumentIdentifier};
use serde::{Deserialize, Serialize};

use crate::nav::Ctx;
use crate::preview::drop_explanation;

/// The request's method name.
pub const METHOD: &str = "ascribe/buildView";

/// The parameters of `ascribe/buildView`.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BuildViewParams {
    /// The page.
    pub text_document: TextDocumentIdentifier,
    /// The build to look at. Without one, the editor's build (`[editor]
    /// build`).
    #[serde(default)]
    pub build: Option<String>,
}

/// The answer to `ascribe/buildView`. A document that isn't a source file of
/// the project, or a build the content model doesn't have, gets an answer
/// with an empty `build` and nothing left out.
#[derive(Debug, Serialize, PartialEq)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct BuildViewResult {
    /// The build the answer is for; empty when there is none.
    pub build: String,
    /// The version of the open document the answer was computed from, or
    /// `null` when the file isn't open.
    pub document_version: Option<i32>,
    /// Whether the build publishes the page. `false` when it drops the whole
    /// page (its `variant` or `available` frontmatter); a fragment, which is
    /// part of the pages that include it, counts as published.
    pub page_included: bool,
    /// Why the build doesn't publish the page, when it doesn't.
    pub page_detail: Option<String>,
    /// What the build leaves out of the page's own text, in document order.
    pub excluded: Vec<Excluded>,
}

impl Default for BuildViewResult {
    fn default() -> Self {
        BuildViewResult {
            build: String::new(),
            document_version: None,
            page_included: true,
            page_detail: None,
            excluded: Vec::new(),
        }
    }
}

/// Text a build leaves out of a page.
#[derive(Debug, Serialize, PartialEq)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct Excluded {
    /// What's left out: whole blocks, arms, or table rows, from the first
    /// directive line through the last line. Neighbors left out for the same
    /// reason are one range.
    #[cfg_attr(feature = "json-schema", schemars(with = "crate::schema::LspRange"))]
    pub range: Range,
    /// Why.
    pub reason: ExclusionReason,
    /// The reason, for the author: `Shows only edition=self-hosted`, or
    /// `Scheduled rollouts: available on Lantern Cloud (preview), not
    /// Self-hosted 2.5`, with the content model's display labels.
    pub detail: String,
}

/// Why a build leaves text out of a page.
#[derive(Clone, Copy, Debug, Serialize, PartialEq, Eq)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "lowercase")]
pub enum ExclusionReason {
    /// A variant arm the build's selection doesn't select.
    Variant,
    /// Content the build's availability filter removes.
    Availability,
}

/// Answers `ascribe/buildView` for a document.
pub(crate) fn build_view(ctx: &Ctx, build: Option<&str>) -> BuildViewResult {
    let model = &*ctx.model;
    let build = match build {
        None => model.editor_default_build(),
        Some(name) => match model.build(name) {
            Some(build) => build,
            None => return BuildViewResult::default(),
        },
    };
    let mut result = BuildViewResult {
        build: build.name.clone(),
        document_version: ctx.version,
        ..BuildViewResult::default()
    };
    let Some(index) = ctx.file() else {
        return result;
    };
    if index.kind != FileKind::Page {
        return result;
    }
    if let Some(reason) = ctx.snapshot.dropped(&ctx.path, build) {
        result.page_included = false;
        result.page_detail = Some(format!(
            "The build {} doesn't publish this page: {}.",
            build.name,
            drop_explanation(&reason)
        ));
        return result;
    }
    let Some(removed) = ctx.snapshot.removed(&ctx.path, build) else {
        return result;
    };
    let mut own: Vec<(Span, ExclusionReason, String)> = removed
        .iter()
        .filter(|r| r.file == index.file && r.via.is_empty())
        .map(|r| {
            let (reason, detail) = explain(model, build, r);
            (r.span, reason, detail)
        })
        .collect();
    own.sort_by_key(|(span, ..)| (span.start(), span.end()));
    // Neighbors left out for the same reason, with only blank space between
    // them, are one range: a section is its heading and each of its blocks.
    let source = &index.source;
    let mut merged: Vec<(Span, ExclusionReason, String)> = Vec::new();
    for (span, reason, detail) in own {
        if let Some((last, last_reason, last_detail)) = merged.last_mut()
            && *last_reason == reason
            && *last_detail == detail
            && span.start() >= last.end()
            && source
                .get(last.end()..span.start())
                .is_some_and(|between| between.trim().is_empty())
        {
            *last = Span::new(last.start(), span.end().max(last.end()));
            continue;
        }
        if merged
            .last()
            .is_some_and(|(last, ..)| span.start() < last.end())
        {
            // Inside one already listed; the resolver lists only the
            // outermost, so this doesn't happen.
            continue;
        }
        merged.push((span, reason, detail));
    }
    let lines = LineIndex::new(source);
    result.excluded = merged
        .into_iter()
        .map(|(span, reason, detail)| Excluded {
            range: ctx.encoding.range(&lines, span),
            reason,
            detail,
        })
        .collect();
    result
}

/// Why a removal happened, for the author.
fn explain(model: &ContentModel, build: &Build, removed: &Removed) -> (ExclusionReason, String) {
    match &removed.cause {
        Removal::Variant { dimensions } => {
            let VariantMode::Select(selection) = &build.variants else {
                return (ExclusionReason::Variant, String::new());
            };
            let shown: Vec<String> = selection
                .iter()
                .filter(|(dimension, _)| dimensions.contains(dimension))
                .map(|(dimension, values)| format!("{dimension}={}", values.join("|")))
                .collect();
            (
                ExclusionReason::Variant,
                format!("Shows only {}", shown.join(", ")),
            )
        }
        Removal::Availability(spec) => {
            // The build's target as a person reads it: `Self-hosted 2.5`.
            let built = match &build.availability {
                AvailabilityMode::Filter { target, version } => {
                    let label = model.value_label(target).unwrap_or(target);
                    match version {
                        Some(version) => format!("{label} {}", version.text),
                        None => label.to_owned(),
                    }
                }
                AvailabilityMode::Badge => String::new(),
            };
            let shown = availability_display(model, &spec.spec);
            let feature = spec
                .feature
                .as_deref()
                .and_then(|key| model.feature(key))
                .map(|f| f.name.clone());
            let detail = match feature {
                Some(name) => format!("{name}: available on {shown}, not {built}"),
                None => format!("Available on {shown}, not {built}"),
            };
            (ExclusionReason::Availability, detail)
        }
    }
}
