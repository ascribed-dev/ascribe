//! The files a source file names outside its links and images: in raw HTML
//! and in its frontmatter. Nothing resolves or rewrites these; they're read
//! only so that a check asking whether anything uses a file
//! (`image-unused`) doesn't report one a page shows through `<img>` or names
//! in a field such as `cover`, and so that `ascribe refs` lists them.

use ascribe_core::{Destination, RelPath, Span, classify_destination};
use ascribe_syntax::{BlockKind, InlineKind};

use super::FileIndex;
use super::walk::{own_inlines, walk_blocks, walk_inlines};

/// The HTML attributes whose values name a file.
const ATTRIBUTES: &[&str] = &["src", "href", "poster", "srcset"];

/// A file a source file names outside its links and images.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Mention {
    /// The file it names, as a content path.
    pub path: RelPath,
    /// Where it's named: the value as written, when it's found in the
    /// source, or else the start of the HTML or the frontmatter it's in.
    pub span: Span,
    /// Where it's written.
    pub kind: MentionKind,
}

/// Where a file is named outside a link or an image.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MentionKind {
    /// An attribute in raw HTML.
    Html,
    /// A frontmatter field.
    Frontmatter,
}

impl FileIndex {
    /// The files this one names outside its links and images, as content
    /// paths, in no particular order and possibly repeated: the paths of
    /// [`FileIndex::mentions`].
    pub fn mentioned_paths(&self) -> Vec<RelPath> {
        self.mentions().into_iter().map(|m| m.path).collect()
    }

    /// The files this one names outside its links and images, in document
    /// order and possibly repeated:
    ///
    /// - each `src`, `href`, `poster`, and `srcset` attribute value in its
    ///   raw HTML, resolved as a link written in this file is;
    /// - each string in its frontmatter, resolved the same way and also
    ///   from the content root, since a field such as `cover: images/a.png`
    ///   is often written from there.
    ///
    /// A value with a scheme, or that can't be a path, names nothing.
    pub fn mentions(&self) -> Vec<Mention> {
        let mut out = Vec::new();
        if let Some(frontmatter) = &self.frontmatter {
            let within = self
                .document
                .frontmatter
                .as_ref()
                .map_or(Span::new(0, 0), |f| f.content);
            let mut from = within.start();
            let mut strings = Vec::new();
            yaml_strings(frontmatter, &mut strings);
            for text in strings {
                let mut paths: Vec<RelPath> = self.resolve(text).into_iter().collect();
                if !text.starts_with(['/', '.']) {
                    paths.extend(RelPath::parse(text).ok());
                }
                if paths.is_empty() {
                    continue;
                }
                let span = self.find(text, within, &mut from);
                out.extend(paths.into_iter().map(|path| Mention {
                    path,
                    span,
                    kind: MentionKind::Frontmatter,
                }));
            }
        }
        let html = |text: &str, within: Span, out: &mut Vec<Mention>| {
            let mut from = within.start();
            for value in attribute_values(text) {
                if let Some(path) = self.resolve(value) {
                    let span = self.find(value, within, &mut from);
                    out.push(Mention {
                        path,
                        span,
                        kind: MentionKind::Html,
                    });
                }
            }
        };
        walk_blocks(&self.document.blocks, &mut |block| {
            if let BlockKind::HtmlBlock(b) = &block.kind {
                html(&b.literal, block.span, &mut out);
            }
            for inlines in own_inlines(block) {
                walk_inlines(inlines, &mut |inline| {
                    if let InlineKind::Html(text) = &inline.kind {
                        html(text, inline.span, &mut out);
                    }
                });
            }
        });
        out
    }

    /// The file a destination written in this file names, if it's local.
    fn resolve(&self, text: &str) -> Option<RelPath> {
        match classify_destination(text.trim()) {
            Destination::Local(local) if !local.path.is_empty() => local.resolve(&self.path).ok(),
            _ => None,
        }
    }

    /// Where `text` is written in the source within `within`, looking from
    /// `from` on and moving `from` past it; the start of `within` when it
    /// isn't there as written (a YAML string with escapes).
    fn find(&self, text: &str, within: Span, from: &mut usize) -> Span {
        let found = self
            .source
            .get(*from..within.end())
            .and_then(|rest| rest.find(text))
            .map(|at| *from + at);
        match found {
            Some(at) if !text.is_empty() => {
                *from = at + text.len();
                Span::new(at, at + text.len())
            }
            _ => Span::new(within.start(), within.start()),
        }
    }
}

/// The values of [`ATTRIBUTES`] in a run of HTML, each `srcset` candidate on
/// its own. Quoted with `"` or `'`, or unquoted up to a space or `>`.
fn attribute_values(html: &str) -> Vec<&str> {
    let mut out = Vec::new();
    let lower = html.to_ascii_lowercase();
    for name in ATTRIBUTES {
        let mut from = 0;
        while let Some(found) = lower[from..].find(name) {
            let start = from + found;
            from = start + name.len();
            // A whole attribute name: preceded by space, followed by `=`.
            let before = lower[..start].chars().next_back();
            if !before.is_some_and(char::is_whitespace) {
                continue;
            }
            let rest = html[from..].trim_start();
            let Some(rest) = rest.strip_prefix('=') else {
                continue;
            };
            let rest = rest.trim_start();
            let value = match rest.chars().next() {
                Some(quote @ ('"' | '\'')) => rest[1..].split(quote).next().unwrap_or_default(),
                _ => rest
                    .split(|c: char| c.is_whitespace() || c == '>')
                    .next()
                    .unwrap_or_default(),
            };
            if *name == "srcset" {
                out.extend(
                    value
                        .split(',')
                        .filter_map(|candidate| candidate.split_whitespace().next()),
                );
            } else {
                out.push(value);
            }
        }
    }
    out
}

/// Every string in a YAML value, keys aside.
fn yaml_strings<'a>(value: &'a serde_yaml_ng::Value, out: &mut Vec<&'a str>) {
    match value {
        serde_yaml_ng::Value::String(s) => out.push(s),
        serde_yaml_ng::Value::Sequence(items) => {
            for item in items {
                yaml_strings(item, out);
            }
        }
        serde_yaml_ng::Value::Mapping(map) => {
            for item in map.values() {
                yaml_strings(item, out);
            }
        }
        serde_yaml_ng::Value::Tagged(tagged) => yaml_strings(&tagged.value, out),
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::attribute_values;

    #[test]
    fn attribute_values_read_each_quoting_and_srcset_candidates() {
        let html = r#"<img src="a.png" alt=x><IMG SRC='b.png'><video poster=c.png>
<img srcset="d.png 1x, e.png 2x"><a href=f.pdf>f</a><img data-src="g.png">"#;
        let mut values = attribute_values(html);
        values.sort_unstable();
        assert_eq!(
            values,
            ["a.png", "b.png", "c.png", "d.png", "e.png", "f.pdf"]
        );
    }
}
