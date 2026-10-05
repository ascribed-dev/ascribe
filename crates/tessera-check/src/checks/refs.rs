//! What a file refers to: `@include` targets, `@snippet` addresses, link
//! destinations, and image sources (SPEC §4.2, §4.8, §5.2, §5.3).
//!
//! Every rule about what a reference names, and whether it's there, lives in
//! `tessera_resolve::references`: the same code the source index runs, so
//! `ascribe check` and the source index can't disagree. That covers phrase
//! substitution in destinations, the boundary and exact-case rules for
//! local files, routes, and where a diagnostic points.
//! This module runs those rules on each reference of a file, and adds
//! the image checks that need the content model.

use tessera_core::{Issue, Location, Span, diagnostics};
use tessera_resolve::{
    RefKind, SnippetUse, destination_phrases, destination_span, include_issue, include_target,
    reference_issue, reference_target, resolve_reference, snippet_issues,
};
use tessera_syntax::{DirectiveLine, Image, Inline, Link, LinkForm, Phrase};

use super::Ctx;
use super::attrs::{Owner, required_missing};

impl Ctx<'_> {
    /// `@include: path#id` (SPEC §4.2): the target must be a source file of
    /// the project. Whether the id exists is a page-level check.
    pub(super) fn check_include(&mut self, primary: &str, span: Span) {
        let include = include_target(primary, &self.file.path);
        let at = self.location(span);
        if let Some(issue) =
            include_issue(&include.written, include.target.as_ref(), self.project, at)
        {
            self.report(issue);
        }
    }

    /// A `@snippet`'s address names a file of a source, and a region of it
    /// whose tags are sound (SPEC §4.8).
    pub(super) fn check_snippet(&mut self, line: &DirectiveLine) {
        let snippet = SnippetUse::of(line);
        let project = self.project;
        for issue in snippet_issues(
            &snippet,
            self.model,
            project.file_system(),
            project.code_files(),
            self.id,
        ) {
            self.report(issue);
        }
    }

    /// A link's destination (SPEC §5.2): a file that exists, a page and not
    /// a fragment, and not a route.
    pub(super) fn check_link(&mut self, span: Span, link: &Link) {
        self.check_reference(Reference {
            kind: RefKind::Link,
            span,
            form: link.form,
            destination: &link.destination,
            phrases: &link.destination_phrases,
            label: link.label,
            children: &link.children,
            alt: None,
        });
    }

    /// An image: the source exists, it has alt text, and its attributes
    /// match the model's (SPEC §5.3).
    pub(super) fn check_image(&mut self, span: Span, image: &Image) {
        if image.children.is_empty() && self.source_str(image.alt).trim().is_empty() {
            let issue = Issue::new(diagnostics::IMAGE_ALT_MISSING, self.location(span));
            self.report(issue);
        }

        let declared = self.model.image_attributes.clone();
        if let Some(attributes) = &image.attributes {
            self.check_block(&attributes.block, &declared, Owner::Image);
        }
        for missing in required_missing(&declared, image.attributes.as_ref().map(|a| &a.block)) {
            let issue = Issue::new(diagnostics::IMAGE_ATTRIBUTE_MISSING, self.location(span))
                .with_arg("key", missing.key.clone());
            self.report(issue);
        }

        self.check_reference(Reference {
            kind: RefKind::Image,
            span,
            form: image.form,
            destination: &image.destination,
            phrases: &image.destination_phrases,
            label: image.label,
            children: &image.children,
            alt: Some(image.alt),
        });
    }

    /// Runs the shared reference rules on one link or image.
    fn check_reference(&mut self, r: Reference<'_>) {
        // A reference form's phrases are its definition's.
        let phrases = destination_phrases(
            self.source(),
            r.form,
            r.phrases,
            r.label,
            r.children,
            r.alt,
            r.destination,
            &self.definitions,
        );
        let target = reference_target(r.kind, r.destination, phrases, &self.file.path, self.model);
        let resolution = resolve_reference(
            r.kind,
            &target,
            &self.file.path,
            self.model,
            self.project,
            self.project.file_system(),
            self.project.layout(),
        );
        let at = Location::new(self.id, r.span);
        let destination_at = destination_span(
            self.source(),
            r.span,
            r.form,
            r.children,
            r.alt,
            r.destination,
        );
        if let Some(issue) = reference_issue(
            r.kind,
            &target,
            r.destination,
            &resolution,
            at,
            destination_at,
        ) {
            self.report(issue);
        }
    }

    fn source_str(&self, span: Span) -> &str {
        self.source().get(span.range()).unwrap_or_default()
    }
}

/// A link or image, as the shared rules need it.
struct Reference<'a> {
    kind: RefKind,
    span: Span,
    form: LinkForm,
    destination: &'a str,
    phrases: &'a [Phrase],
    label: Option<Span>,
    children: &'a [Inline],
    alt: Option<Span>,
}
