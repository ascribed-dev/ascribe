//! Writing HTML elements the way the element contract asks
//! (`packages/elements/CONTRACT.md` §0): attribute values in double quotes,
//! escaped, in the contract's order, and layout that makes CommonMark parse
//! the markdown inside a wrapping element as markdown.

use ascribe_core::names;

/// `&`, `<`, `>`, and `"` escaped as the contract says (§0), for an
/// attribute value, and safe as element text.
pub(crate) fn escape(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    for ch in value.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            _ => out.push(ch),
        }
    }
    out
}

/// An element's attributes, in the order they're written. An attribute with
/// no value is never written empty (contract §0), so callers pass `None` for
/// an optional one that's absent.
#[derive(Clone, Debug, Default)]
pub(crate) struct Attrs(Vec<(String, String)>);

impl Attrs {
    pub(crate) fn new() -> Attrs {
        Attrs(Vec::new())
    }

    /// Adds an attribute that is always written.
    pub(crate) fn with(mut self, name: &str, value: impl Into<String>) -> Attrs {
        self.0.push((name.to_owned(), value.into()));
        self
    }

    /// Adds an attribute that is left out when it has no value.
    pub(crate) fn with_opt(self, name: &str, value: Option<String>) -> Attrs {
        match value.filter(|v| !v.is_empty()) {
            Some(value) => self.with(name, value),
            None => self,
        }
    }

    /// `name="value" name="value"`, with a leading space when there are any.
    fn write(&self) -> String {
        self.0
            .iter()
            .map(|(name, value)| format!(" {name}=\"{}\"", escape(value)))
            .collect()
    }
}

/// An opening tag: `<ascribe-note type="tip">`.
pub(crate) fn open(name: &str, attrs: &Attrs) -> String {
    format!("<{name}{}>", attrs.write())
}

/// A closing tag.
pub(crate) fn close(name: &str) -> String {
    format!("</{name}>")
}

/// An element that wraps markdown (contract §0): its opening tag alone on a
/// line, a blank line, the blocks, a blank line, and the closing tag alone on
/// a line. Each string in `blocks` is one markdown block (or several, already
/// separated by blank lines).
pub(crate) fn wrap(name: &str, attrs: &Attrs, blocks: &[String]) -> String {
    let mut out = open(name, attrs);
    out.push_str("\n\n");
    for block in blocks {
        out.push_str(block);
        out.push_str("\n\n");
    }
    out.push_str(&close(name));
    out
}

/// An element with no markdown inside, as one line: `<name a="v"></name>`.
pub(crate) fn empty(name: &str, attrs: &Attrs) -> String {
    format!("{}{}", open(name, attrs), close(name))
}

/// The attribute marker: an empty
/// `ascribe-attributes` element holding `attrs`.
pub(crate) fn marker(attrs: &[(String, String)]) -> String {
    let mut out = Attrs::new();
    for (name, value) in attrs {
        out = out.with(name, value.as_str());
    }
    empty(names::ELEMENT_ATTRIBUTES, &out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn values_are_escaped_in_the_contracts_way() {
        let attrs = Attrs::new().with("heading", "A \"b\" & <c>");
        assert_eq!(
            open("ascribe-note", &attrs),
            "<ascribe-note heading=\"A &quot;b&quot; &amp; &lt;c&gt;\">"
        );
    }

    #[test]
    fn optional_attributes_are_left_out_when_empty() {
        let attrs = Attrs::new()
            .with("type", "note")
            .with_opt("heading", Some(String::new()))
            .with_opt("x", None);
        assert_eq!(open("ascribe-note", &attrs), "<ascribe-note type=\"note\">");
    }

    #[test]
    fn a_wrapping_element_has_blank_lines_inside_its_tags() {
        let attrs = Attrs::new();
        assert_eq!(
            wrap("ascribe-steps", &attrs, &["1. One.".to_owned()]),
            "<ascribe-steps>\n\n1. One.\n\n</ascribe-steps>"
        );
    }

    #[test]
    fn a_marker_is_an_empty_element() {
        assert_eq!(
            marker(&[("id".to_owned(), "a-&-b".to_owned())]),
            "<ascribe-attributes id=\"a-&amp;-b\"></ascribe-attributes>"
        );
    }
}
