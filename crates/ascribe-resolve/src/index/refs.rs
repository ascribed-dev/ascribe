//! Links and images: what each one names, resolved from the file it's
//! written in.

use ascribe_core::{Destination, LocalDestination, RelPath, Span, classify_destination};
use ascribe_model::ContentModel;
use ascribe_syntax::{Inline, InlineKind, LinkDefinition, LinkForm, Phrase};

use super::walk::walk_inlines;

/// Whether a reference is a link or an image.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum RefKind {
    /// A link (SPEC §5.2).
    Link,
    /// An image (SPEC §5.3).
    Image,
}

/// A link or an image, with what its destination names.
///
/// What a destination names is worked out from the destination and the file's
/// path alone, so it doesn't depend on which files exist. Whether they do is
/// decided when the project is assembled ([`crate::Resolution`]).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Reference {
    /// Link or image.
    pub kind: RefKind,
    /// The node, from the `[` or `![` through its last character (an image's
    /// attribute block included). For a reference form that's the label, not
    /// the definition.
    pub span: Span,
    /// The destination as written, for an inline link or image: where a
    /// diagnostic about the file it names points.
    pub destination_span: Option<Span>,
    /// The destination, with escapes decoded and `<>` removed. For a
    /// reference form, that of the link reference definition (SPEC §5.3).
    pub destination: String,
    /// The destination after substituting the parser's phrase candidates.
    pub expanded_destination: String,
    /// How the reference was written.
    pub form: LinkForm,
    /// Whether the link has no text, so it takes its target's title
    /// (SPEC §5.2). Always `false` for an image.
    pub text_empty: bool,
    /// What the destination names.
    pub target: Target,
}

/// What a destination names.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Target {
    /// A URL with a scheme, or one starting with `//`. Never an asset, and
    /// passed through unchanged (SPEC §5.2).
    External,
    /// A file in the project.
    Local(Local),
}

/// A local destination, resolved.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Local {
    /// The file it names, as a content path: resolved from the file the
    /// reference is written in, so from a fragment's own location for
    /// included content (SPEC §4.2). It starts with `..` when the file is
    /// outside the content root. `None` when the text can't be a path (it
    /// contains a NUL).
    pub path: Option<RelPath>,
    /// The part after `#`, percent-decoded. A page's heading source id, or
    /// (for an asset) a fragment kept after the rewritten reference.
    /// `None` when there is no `#` or nothing after it.
    pub fragment: Option<String>,
    /// The destination as written, phrases substituted and without the `#`
    /// part, still percent-encoded and with its leading `/`: what messages
    /// call the path.
    pub written: String,
    /// Whether it names an Ascribe source file: a link to a `.md` file inside
    /// the content root, which is a page or a fragment and never an asset.
    /// An image is never a source.
    pub source: bool,
    /// Whether a link looks like a published route rather than a file path:
    /// its last segment has no extension, or it ends in `/` (SPEC §5.2).
    /// It only counts if no such file exists.
    pub route_like: bool,
}

/// Collects the links and images in `inlines`, in document order. `written_in`
/// is the file's content path.
pub(crate) fn collect_references(
    inlines: &[Inline],
    source: &str,
    written_in: &RelPath,
    model: &ContentModel,
    definitions: &[LinkDefinition],
    out: &mut Vec<Reference>,
) {
    walk_inlines(inlines, &mut |inline| {
        let (kind, form, destination, own, label, text_empty, children, alt) = match &inline.kind {
            InlineKind::Link(l) => (
                RefKind::Link,
                l.form,
                &l.destination,
                &l.destination_phrases,
                l.label,
                l.children.is_empty(),
                &l.children,
                None,
            ),
            InlineKind::Image(i) => (
                RefKind::Image,
                i.form,
                &i.destination,
                &i.destination_phrases,
                i.label,
                false,
                &i.children,
                Some(i.alt),
            ),
            _ => return,
        };
        // A reference form's destination is in its definition.
        let phrases = crate::references::destination_phrases(
            source,
            form,
            own,
            label,
            children,
            alt,
            destination,
            definitions,
        );
        let expanded_destination = substitute(destination, phrases, model);
        out.push(Reference {
            kind,
            span: inline.span,
            destination_span: crate::references::destination_span(
                source,
                inline.span,
                form,
                children,
                alt,
                destination,
            ),
            destination: destination.clone(),
            expanded_destination,
            form,
            text_empty,
            target: target_of(kind, destination, phrases, written_in, model),
        });
    });
}

pub(crate) fn target_of(
    kind: RefKind,
    destination: &str,
    phrases: &[Phrase],
    written_in: &RelPath,
    model: &ContentModel,
) -> Target {
    // A reference form's `phrases` are its definition's: the caller
    // passes them, so every form is substituted the same way.
    let text = substitute(destination, phrases, model);
    match classify_destination(&text) {
        Destination::External => Target::External,
        Destination::Local(local) => Target::Local(local_of(kind, &text, &local, written_in)),
    }
}

fn local_of(kind: RefKind, text: &str, local: &LocalDestination, written_in: &RelPath) -> Local {
    let path = local.resolve(written_in).ok();
    let source = kind == RefKind::Link
        && path
            .as_ref()
            .is_some_and(|p| p.is_inside() && p.extension() == Some("md"));
    let names_itself = local.path.is_empty() && !local.root_relative;
    let route_like = kind == RefKind::Link
        && !names_itself
        && (local.path.is_empty()
            || local.path.ends_with('/')
            || path.as_ref().is_none_or(|p| p.extension().is_none()));
    Local {
        path,
        fragment: local.fragment.clone().filter(|f| !f.is_empty()),
        written: text.split('#').next().unwrap_or_default().to_owned(),
        source,
        route_like,
    }
}

/// The destination with each declared phrase candidate replaced by its value,
/// in order. The destination is CommonMark's decoded text, and the candidates
/// are the parser's, so an escaped `\{key}` in a destination that also has a
/// real candidate of the same key could be replaced instead; that pairing of
/// decoded text with source is the parser's to expose, and no author writes it.
pub(crate) fn substitute(destination: &str, phrases: &[Phrase], model: &ContentModel) -> String {
    let mut out = String::new();
    let mut rest = destination;
    for phrase in phrases {
        let Some(value) = model.phrase(&phrase.key) else {
            continue;
        };
        let needle = format!("{{{}}}", phrase.key);
        if let Some(at) = rest.find(&needle) {
            out.push_str(&rest[..at]);
            out.push_str(value);
            rest = &rest[at + needle.len()..];
        }
    }
    out.push_str(rest);
    out
}
