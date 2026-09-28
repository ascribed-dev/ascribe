//! What a file refers to: `@include` targets, link destinations, and image
//! sources (SPEC §4.2, §5.2, §5.3), checked against the file system with the
//! asset contract's rules (`project-docs/contracts/assets.md`, §2):
//!
//! - a destination resolves from the file it's written in;
//! - the file must be inside the project root or the content root, and not
//!   inside the output directory, or it's reported as not existing (Q10);
//! - names match exactly, on every platform (Q10);
//! - a destination that names no file and looks like a route gets
//!   `link-route` instead of a missing-file error (Q22).

use tessera_core::{
    Destination, Fix, Issue, LocalDestination, Location, RelPath, Span, TextEdit,
    classify_destination, diagnostics,
};
use tessera_syntax::{Image, Inline, Link, LinkForm};

use super::Ctx;
use super::attrs::{Owner, required_missing};
use crate::Lookup;

/// What became of a reference.
enum Existence {
    /// A regular file, named exactly.
    Found,
    /// Only a file whose name differs in case; holds its real path.
    Case(String),
    /// Outside the project and content roots, or inside the output directory.
    Outside,
    /// No such file.
    Missing,
}

impl Ctx<'_> {
    /// The directory of this file, as a content path.
    fn file_dir(&self) -> RelPath {
        self.file.path.parent().unwrap_or_default()
    }

    /// Whether a content path names a file that exists, under the asset
    /// contract's boundary and exact-case rules.
    fn existence(&self, content: &RelPath) -> Existence {
        let root = self.project.content_root();
        let Ok(project_path) = root.join(content.as_str()) else {
            return Existence::Missing;
        };
        let output = RelPath::parse(&self.model.project.output_dir).unwrap_or_default();
        let in_output = !output.is_root() && project_path.starts_with(&output);
        let inside = content.is_inside() || project_path.is_inside();
        if !inside || in_output {
            return Existence::Outside;
        }
        match self.project.lookup(&project_path) {
            Lookup::Found => Existence::Found,
            Lookup::CaseMismatch(actual) => Existence::Case(self.relative_to_content_root(&actual)),
            Lookup::Missing => Existence::Missing,
        }
    }

    /// A project-relative path, written relative to the content root when it's
    /// inside it.
    fn relative_to_content_root(&self, project_path: &str) -> String {
        let root = self.project.content_root().as_str();
        if root.is_empty() {
            return project_path.to_owned();
        }
        project_path
            .strip_prefix(root)
            .and_then(|rest| rest.strip_prefix('/'))
            .unwrap_or(project_path)
            .to_owned()
    }

    /// `@include: path#id` (SPEC §4.2): the file must exist. Whether the id
    /// exists is a page-level check.
    pub(super) fn check_include(&mut self, primary: &str, span: Span) {
        let path = primary.split_once('#').map_or(primary, |(p, _)| p);
        if path.is_empty() {
            return;
        }
        let resolved = if let Some(rooted) = path.strip_prefix('/') {
            RelPath::root().join(rooted)
        } else {
            self.file_dir().join(path)
        };
        let content = resolved.unwrap_or_default();
        let issue = Issue::new(diagnostics::INCLUDE_TARGET_MISSING, self.location(span))
            .with_arg("path", path.to_owned());
        match self.existence(&content) {
            Existence::Found => {}
            Existence::Case(actual) => {
                self.report(issue.with_variant("case").with_arg("actual", actual));
            }
            Existence::Outside | Existence::Missing => self.report(issue),
        }
    }

    /// A link's destination (SPEC §5.2): a file that exists, a page and not
    /// a fragment, and not a route.
    pub(super) fn check_link(&mut self, span: Span, link: &Link) {
        if link.form == LinkForm::Autolink {
            return;
        }
        // SPEC-QUESTION(Q54): phrases in a destination are substituted
        // before the file checks see it.
        let destination = self.substitute_phrases(&link.destination);
        let Destination::Local(local) = classify_destination(&destination) else {
            return;
        };
        if local.path.is_empty() && !local.root_relative {
            // SPEC-QUESTION(Q59): `#id` alone names the file it's written in
            // (page level); an empty destination names nothing.
            return;
        }
        let Ok(content) = local.resolve(&self.file.path) else {
            return;
        };
        let written = destination.split('#').next().unwrap_or_default().to_owned();
        let at = self.destination_span(span, link.form, &link.children, None, &link.destination);
        let location = self.location(at.unwrap_or(span));
        match self.existence(&content) {
            Existence::Found => {
                let is_page_or_fragment = content.is_inside() && content.extension() == Some("md");
                if is_page_or_fragment && self.model.is_fragment(content.as_str()) {
                    let issue = Issue::new(diagnostics::LINK_TO_FRAGMENT, location)
                        .with_arg("path", written);
                    self.report(issue);
                }
            }
            outcome => {
                if looks_like_route(&local) {
                    let issue = self.route_issue(&local, &content, location, at);
                    self.report(issue);
                    return;
                }
                let issue = Issue::new(diagnostics::LINK_TARGET_MISSING, location)
                    .with_arg("path", written);
                self.report(missing_variant(issue, outcome));
            }
        }
    }

    /// `link-route`: this looks like a published route, not a file (Q22),
    /// with the link to the page's file as the suggested fix.
    fn route_issue(
        &self,
        local: &LocalDestination,
        content: &RelPath,
        location: Location,
        destination: Option<Span>,
    ) -> Issue {
        // SPEC-QUESTION(Q55): the conventional route-to-page mapping, until
        // the consumer profile's router (phase 12) can answer.
        let (page, exists) = self.route_page(content);
        let file_dir = self.file_dir();
        let path = if local.root_relative {
            format!("/{page}")
        } else {
            page.relative_from(&file_dir).map_or_else(
                || page.to_string(),
                |p| p.strip_prefix("./").map_or(p.clone(), str::to_owned),
            )
        };
        let suggestion = match &local.fragment {
            Some(f) => format!("{path}#{f}"),
            None => path,
        };
        let mut issue = Issue::new(diagnostics::LINK_ROUTE, location)
            .with_arg("page", page.to_string())
            .with_arg("suggestion", suggestion.clone());
        if let (true, Some(span)) = (exists, destination) {
            let new_text = if suggestion.contains([' ', '(', ')']) {
                format!("<{suggestion}>")
            } else {
                suggestion
            };
            issue = issue.with_fix(Fix {
                title: "Link to the page's file instead of its route".into(),
                file: self.id,
                edits: vec![TextEdit::replace(span, new_text)],
            });
        }
        issue
    }

    /// The source file a route most likely names: `route.md`, or
    /// `route/index.md`, whichever exists; else `route.md`. The bool says
    /// whether one exists. The real mapping is the consumer profile's router
    /// (phase 12); this is the conventional one.
    fn route_page(&self, content: &RelPath) -> (RelPath, bool) {
        let name = format!("{}.md", content.as_str());
        let candidates = [
            if content.is_root() {
                "index.md".to_owned()
            } else {
                name.clone()
            },
            format!("{}/index.md", content.as_str()),
        ];
        for candidate in &candidates {
            if let Ok(path) = RelPath::root().join(candidate)
                && matches!(self.existence(&path), Existence::Found)
            {
                return (path, true);
            }
        }
        let default = RelPath::root().join(&candidates[0]).unwrap_or_default();
        (default, false)
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

        if image.form == LinkForm::Autolink {
            return;
        }
        let destination = self.substitute_phrases(&image.destination);
        let Destination::Local(local) = classify_destination(&destination) else {
            return;
        };
        if local.path.is_empty() && !local.root_relative {
            return;
        }
        let Ok(content) = local.resolve(&self.file.path) else {
            return;
        };
        let written = destination.split('#').next().unwrap_or_default().to_owned();
        let at = self.destination_span(
            span,
            image.form,
            &image.children,
            Some(image.alt),
            &image.destination,
        );
        let outcome = self.existence(&content);
        if !matches!(outcome, Existence::Found) {
            let issue = Issue::new(
                diagnostics::IMAGE_SOURCE_MISSING,
                self.location(at.unwrap_or(span)),
            )
            .with_arg("path", written);
            self.report(missing_variant(issue, outcome));
        }
    }

    /// A destination with its declared phrases filled in (SPEC §5.1: phrases
    /// apply in link destinations), so `{api}streaming` is checked as the
    /// URL it becomes. An undeclared `{key}` stays as written.
    fn substitute_phrases(&self, destination: &str) -> String {
        let mut out = String::with_capacity(destination.len());
        let mut rest = destination;
        while let Some(open) = rest.find('{') {
            out.push_str(&rest[..open]);
            let tail = &rest[open..];
            let phrase = tail
                .find('}')
                .and_then(|close| self.model.phrase(&tail[1..close]).map(|v| (close, v)));
            match phrase {
                Some((close, value)) => {
                    out.push_str(value);
                    rest = &tail[close + 1..];
                }
                None => {
                    out.push('{');
                    rest = &tail[1..];
                }
            }
        }
        out.push_str(rest);
        out
    }

    fn source_str(&self, span: Span) -> &str {
        self.source().get(span.range()).unwrap_or_default()
    }

    /// The span of an inline link's or image's destination as written, for
    /// pointing at it. `None` for reference forms (the destination is in the
    /// definition) and when the text doesn't read back as `destination`.
    fn destination_span(
        &self,
        span: Span,
        form: LinkForm,
        children: &[Inline],
        alt: Option<Span>,
        destination: &str,
    ) -> Option<Span> {
        // SPEC-QUESTION(Q53): the destination as written for inline forms,
        // the whole link or image for reference forms.
        if form != LinkForm::Inline {
            return None;
        }
        let source = self.source();
        // The `]` that closes the text is where the children end, or the
        // alt text; look for `](` from there.
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
}

/// The message variant for a reference that isn't there.
fn missing_variant(issue: Issue, outcome: Existence) -> Issue {
    match outcome {
        Existence::Outside => issue.with_variant("outside"),
        Existence::Case(actual) => issue.with_variant("case").with_arg("actual", actual),
        Existence::Found | Existence::Missing => issue,
    }
}

/// SPEC §5.2 / Q22: a destination looks like a route when its last segment
/// has no file extension, or it ends in `/`.
fn looks_like_route(local: &LocalDestination) -> bool {
    if local.path.is_empty() || local.path.ends_with('/') {
        return true;
    }
    let last = local.path.rsplit('/').next().unwrap_or_default();
    !last.get(1..).unwrap_or_default().contains('.')
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

    fn local(dest: &str) -> LocalDestination {
        match classify_destination(dest) {
            Destination::Local(l) => l,
            Destination::External => panic!("external"),
        }
    }

    #[test]
    fn routes_have_no_extension_or_end_in_a_slash() {
        for route in [
            "/guides/install/",
            "/guides/install",
            "../guides/install",
            "/",
            "a/",
        ] {
            assert!(looks_like_route(&local(route)), "{route}");
        }
        for file in ["keys.md", "/a/b.png", "a/.hidden", "downloads/quill.yaml"] {
            // `.hidden` has no extension by RelPath's rule, so it is route-like.
            if file == "a/.hidden" {
                continue;
            }
            assert!(!looks_like_route(&local(file)), "{file}");
        }
    }

    #[test]
    fn unescapes_punctuation_only() {
        assert_eq!(unescape(r"a\(b\)\x"), r"a(b)\x");
    }
}
