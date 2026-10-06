//! A page's frontmatter in the site output (SPEC §9.4, §9.6).
//!
//! The page's own frontmatter passes through, with phrases already
//! substituted where the content model says. The one field Ascribe rewrites is
//! the reserved `available`: it is the spec as the author wrote it, which a
//! layout would have to parse, so it becomes a list of targets a layout can
//! pass straight to `<ascribe-availability scope="page">` (element contract
//! §4).
//!
//! ```yaml
//! available:
//!   - target: cloud
//!     dimension: deployment
//!     states: [ga]
//!     text: Quill Cloud (GA)
//!   - target: self-managed
//!     dimension: deployment
//!     states: [preview]
//!     versions: ['3.3']
//!     text: Self-managed (preview, 3.3+)
//! ```
//!
//! Each entry has the attributes of a `<ascribe-availability-target>` and its
//! text. `variant` passes through as written.
//!
//! A field that sets `inline = "code"` is written as its plain text, for a
//! layout's `<title>`, search, and sorting, and its formatted form goes under
//! the reserved `formatted`, as HTML, for its heading and navigation:
//!
//! ```yaml
//! title: ascribe.toml reference
//! formatted:
//!   title: <code>ascribe.toml</code> reference
//! ```

use serde_yaml_ng::{Mapping, Value};
use tessera_model::Segment;
use tessera_resolve::ResolvedPage;

use super::blocks::availability_target;
use super::element::escape;
use crate::emitter::PageContext;
use crate::error::EmitError;

/// The page's frontmatter block, `---` lines included and a blank line after,
/// or an empty string when the page has nothing to put in it.
pub(crate) fn render(cx: &PageContext<'_>, page: &ResolvedPage) -> Result<String, EmitError> {
    let mut map = match &page.frontmatter {
        Some(Value::Mapping(map)) => map.clone(),
        _ => Mapping::new(),
    };
    let key = Value::String("available".to_owned());
    match &page.availability {
        Some(availability) => {
            let targets: Vec<Value> = availability
                .spec
                .entries
                .iter()
                .map(|entry| target_value(&availability_target(cx.emit.model, entry)))
                .collect();
            map.insert(key, Value::Sequence(targets));
        }
        None => {
            map.remove(&key);
        }
    }
    let key = Value::String("formatted".to_owned());
    map.remove(&key);
    if !page.formatted.is_empty() {
        let mut formatted = Mapping::new();
        for field in &page.formatted {
            formatted.insert(
                Value::String(field.name.clone()),
                Value::String(html(&field.segments)),
            );
        }
        map.insert(key, Value::Mapping(formatted));
    }
    if map.is_empty() {
        return Ok(String::new());
    }
    let yaml = serde_yaml_ng::to_string(&Value::Mapping(map)).map_err(|e| EmitError::Render {
        page: page.path.to_string(),
        message: format!("can't write the frontmatter: {e}"),
    })?;
    Ok(format!("---\n{yaml}---\n\n"))
}

/// A formatted value as HTML: its text escaped, and each code span a
/// `<code>`.
pub(crate) fn html(segments: &[Segment]) -> String {
    segments
        .iter()
        .map(|segment| match segment {
            Segment::Text(text) => escape(text),
            Segment::Code(code) => format!("<code>{}</code>", escape(code)),
        })
        .collect()
}

fn target_value(view: &super::blocks::TargetView) -> Value {
    let strings =
        |list: &[String]| Value::Sequence(list.iter().cloned().map(Value::String).collect());
    let mut map = Mapping::new();
    map.insert("target".into(), Value::String(view.target.clone()));
    map.insert("dimension".into(), Value::String(view.dimension.clone()));
    map.insert("states".into(), strings(&view.states));
    if !view.versions.is_empty() {
        map.insert("versions".into(), strings(&view.versions));
    }
    map.insert("text".into(), Value::String(view.text.clone()));
    Value::Mapping(map)
}
