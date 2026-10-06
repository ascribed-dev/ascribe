//! Display text that both outputs share: availability lines, the label of a
//! group's arm, and inline content as plain text.

use ascribe_core::availability::{AvailabilitySpec, Detail, Entry, parse_availability};
use ascribe_model::ContentModel;
use ascribe_syntax::{DirectiveLine, Inline, InlineKind};

/// An availability spec as a person reads it, with the content model's
/// display labels: `Quill Cloud (GA); self-managed (preview, 3.4+)` (SPEC
/// §9.4).
///
/// Each entry is its target's label, then in parentheses its state and the
/// version where it begins (`3.4+`). A target with no state is generally
/// available (`GA`); a bare version is `GA` from that version; a history
/// lists each state and the version it begins at (`preview 3.3, GA 3.5,
/// deprecated 4.0`). A dimension name standing for all its values shows the
/// dimension's label. This is the element contract's text for each target
/// (`packages/elements/CONTRACT.md` §4).
pub fn availability_display(model: &ContentModel, spec: &AvailabilitySpec) -> String {
    spec.entries
        .iter()
        .map(|entry| availability_target_text(model, entry))
        .collect::<Vec<_>>()
        .join("; ")
}

/// One target of an availability spec as a person reads it: its label, and in
/// parentheses its state and version (`Self-managed (preview, 3.4+)`). The
/// text of a `<ascribe-availability-target>` (element contract §4).
pub fn availability_target_text(model: &ContentModel, entry: &Entry) -> String {
    let ga = model
        .lifecycle_state("ga")
        .map_or("GA", |s| s.label.as_str());
    let state = |name: &str| {
        model
            .lifecycle_state(name)
            .map_or(name.to_owned(), |s| s.label.clone())
    };
    let target = entry.target.text.as_str();
    let label = model
        .value_label(target)
        .or_else(|| model.dimension(target).map(|d| d.label.as_str()))
        .unwrap_or(target);
    let detail = match &entry.detail {
        Detail::None => ga.to_owned(),
        Detail::Version(v) => format!("{ga}, {}+", v.text),
        Detail::State {
            state: s,
            version: None,
        } => state(&s.text),
        Detail::State {
            state: s,
            version: Some(v),
        } => format!("{}, {}+", state(&s.text), v.text),
        Detail::History(steps) => steps
            .iter()
            .map(|step| format!("{} {}", state(&step.state.text), step.version.text))
            .collect::<Vec<_>>()
            .join(", "),
    };
    format!("{label} ({detail})")
}

/// [`availability_display`] for a spec written as text, such as an
/// annotation's. `None` when the text isn't a valid spec.
pub fn availability_display_of_text(model: &ContentModel, text: &str) -> Option<String> {
    let spec = parse_availability(text, 0).ok()?;
    Some(availability_display(model, &spec))
}

/// The label of a dimensional arm, from the display labels of the values its
/// attributes name (SPEC §9.4): the labels of one attribute's values are
/// joined with ` / `, and several attributes with `, `. `None` for an arm
/// with no attributes (a labeled arm's label is its title).
pub fn dimensional_label(model: &ContentModel, opener: &DirectiveLine) -> Option<String> {
    let block = opener.attributes.as_ref()?;
    let parts: Vec<String> = block
        .attributes
        .iter()
        .filter_map(|a| a.value.as_ref())
        .map(|value| {
            value
                .members()
                .into_iter()
                .map(|v| model.value_label(v).unwrap_or(v).to_owned())
                .collect::<Vec<_>>()
                .join(" / ")
        })
        .collect();
    (!parts.is_empty()).then(|| parts.join(", "))
}

/// Inline content as plain text: the text of everything, with no markup.
/// A soft break is a space.
pub fn plain_text(inlines: &[Inline]) -> String {
    let mut out = String::new();
    push_plain(inlines, &mut out);
    out
}

fn push_plain(inlines: &[Inline], out: &mut String) {
    for inline in inlines {
        match &inline.kind {
            InlineKind::Text(t) | InlineKind::Code(t) => out.push_str(t),
            InlineKind::SoftBreak | InlineKind::HardBreak => out.push(' '),
            InlineKind::Html(_) => {}
            InlineKind::Emphasis(c) | InlineKind::Strong(c) => push_plain(c, out),
            InlineKind::Link(l) => push_plain(&l.children, out),
            InlineKind::Image(i) => push_plain(&i.children, out),
            InlineKind::Phrase(p) => {
                out.push('{');
                out.push_str(&p.key);
                out.push('}');
            }
        }
    }
}

/// The values of an attribute block, as `(key, values)` pairs in source order:
/// a token or quoted string has one value, a set several, and a bare key none.
pub fn attribute_values(
    block: Option<&ascribe_core::AttributeBlock>,
) -> Vec<(String, Vec<String>)> {
    block
        .map(|b| {
            b.attributes
                .iter()
                .map(|a| {
                    let values = a
                        .value
                        .as_ref()
                        .map(|v| v.members().into_iter().map(str::to_owned).collect())
                        .unwrap_or_default();
                    (a.key.clone(), values)
                })
                .collect()
        })
        .unwrap_or_default()
}
