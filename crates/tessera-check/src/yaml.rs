//! Where things are in a frontmatter block.
//!
//! `serde_yaml_ng` reads values but forgets where they were, and the checks
//! report a frontmatter problem at the line of the key or value that causes
//! it (SPEC §8.1). [`YamlIndex`] reads the same text with an event parser
//! that keeps positions, and answers "where is `author.name`?" or "where is
//! `tags[1]`?".

use std::collections::HashMap;

use tessera_core::Span;
use yaml_rust2::parser::{Event, MarkedEventReceiver, Parser};
use yaml_rust2::scanner::{Marker, TScalarStyle};

/// One located node.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Node {
    /// The key it's the value of, when it's a mapping value.
    pub key: Option<Span>,
    /// The node's own text; for a mapping or a sequence, the empty span at
    /// its start.
    pub value: Span,
    /// Whether the node is a plain (unquoted, single-line) scalar, written
    /// as it appears in `text`.
    pub plain: bool,
    /// Whether the node is a scalar.
    pub scalar: bool,
}

/// The positions of the nodes of one YAML document.
#[derive(Debug, Default)]
pub struct YamlIndex {
    nodes: HashMap<String, Node>,
}

impl YamlIndex {
    /// Indexes `text`, whose first byte is at `base` in its file. Text that
    /// doesn't parse gives an empty index.
    pub fn build(text: &str, base: usize) -> YamlIndex {
        let mut recv = Receiver {
            text,
            base,
            starts: line_starts(text),
            stack: Vec::new(),
            nodes: HashMap::new(),
        };
        let mut parser = Parser::new_from_str(text);
        if parser.load(&mut recv, false).is_err() {
            return YamlIndex::default();
        }
        YamlIndex { nodes: recv.nodes }
    }

    /// The node at `path`: keys joined by `.`, sequence items as `[i]`
    /// (`author.name`, `tags[1]`). The empty path is the document.
    pub fn get(&self, path: &str) -> Option<&Node> {
        self.nodes.get(path)
    }
}

enum Frame {
    Map {
        path: String,
        /// The key just read, waiting for its value.
        key: Option<(String, Span)>,
    },
    Seq {
        path: String,
        next: usize,
    },
}

struct Receiver<'a> {
    text: &'a str,
    base: usize,
    starts: Vec<usize>,
    stack: Vec<Frame>,
    nodes: HashMap<String, Node>,
}

impl Receiver<'_> {
    /// The byte offset (in `text`) of a marker. yaml-rust2 counts lines from
    /// 1 and columns in characters; a column is read as 0- or 1-based
    /// depending on what lands on the scalar, so callers verify by text.
    fn offset(&self, mark: &Marker) -> usize {
        let line = mark.line().saturating_sub(1);
        let start = self.starts.get(line).copied().unwrap_or(self.text.len());
        let rest = &self.text[start..];
        let col = mark.col();
        rest.char_indices()
            .nth(col)
            .map_or(self.text.len(), |(i, _)| start + i)
    }

    fn span(&self, from: usize, to: usize) -> Span {
        Span::new(self.base + from, self.base + to)
    }

    /// Where a scalar's text ends, given where it starts.
    fn scalar_end(&self, start: usize, value: &str, style: TScalarStyle) -> (usize, bool) {
        let rest = self.text.get(start..).unwrap_or("");
        match style {
            TScalarStyle::Plain if rest.starts_with(value) => (start + value.len(), true),
            TScalarStyle::DoubleQuoted | TScalarStyle::SingleQuoted => {
                let quote = if style == TScalarStyle::DoubleQuoted {
                    '"'
                } else {
                    '\''
                };
                let mut chars = rest.char_indices().skip(1);
                let mut end = None;
                while let Some((i, c)) = chars.next() {
                    if c == '\n' {
                        break;
                    }
                    if c == '\\' && quote == '"' {
                        chars.next();
                    } else if c == quote {
                        end = Some(start + i + 1);
                        break;
                    }
                }
                (end.unwrap_or_else(|| line_end(self.text, start)), false)
            }
            _ => (line_end(self.text, start), false),
        }
    }

    fn node(
        &mut self,
        start: usize,
        scalar: Option<(&str, TScalarStyle)>,
    ) -> Option<(String, Node)> {
        let (end, plain) = match scalar {
            Some((value, style)) => self.scalar_end(start, value, style),
            None => (start, false),
        };
        let value = self.span(start, end);
        let is_scalar = scalar.is_some();
        // A key waiting for its value, or a sequence item, or the root.
        match self.stack.last_mut() {
            None => Some((
                String::new(),
                Node {
                    key: None,
                    value,
                    plain,
                    scalar: is_scalar,
                },
            )),
            Some(Frame::Seq { path, next }) => {
                let p = format!("{path}[{next}]");
                *next += 1;
                Some((
                    p,
                    Node {
                        key: None,
                        value,
                        plain,
                        scalar: is_scalar,
                    },
                ))
            }
            Some(Frame::Map { path, key }) => match key.take() {
                None => {
                    // This node is a key.
                    let name =
                        scalar.map_or_else(|| "(complex key)".to_owned(), |(v, _)| v.to_owned());
                    *key = Some((name, value));
                    None
                }
                Some((name, key_span)) => {
                    let p = if path.is_empty() {
                        name
                    } else {
                        format!("{path}.{name}")
                    };
                    Some((
                        p,
                        Node {
                            key: Some(key_span),
                            value,
                            plain,
                            scalar: is_scalar,
                        },
                    ))
                }
            },
        }
    }
}

impl MarkedEventReceiver for Receiver<'_> {
    fn on_event(&mut self, ev: Event, mark: Marker) {
        let start = self.offset(&mark);
        match ev {
            Event::Scalar(value, style, ..) => {
                // yaml-rust2 marks some scalars one column late; back up when
                // the text isn't there but is one character earlier.
                let start = fix_start(self.text, start, &value, style);
                if let Some((path, node)) = self.node(start, Some((&value, style))) {
                    self.nodes.insert(path, node);
                }
            }
            Event::Alias(_) => {
                if let Some((path, node)) = self.node(start, None) {
                    self.nodes.insert(path, node);
                }
            }
            Event::MappingStart(..) | Event::SequenceStart(..) => {
                let is_map = matches!(ev, Event::MappingStart(..));
                let entry = self.node(start, None);
                let path = match entry {
                    Some((path, node)) => {
                        self.nodes.insert(path.clone(), node);
                        path
                    }
                    // A mapping used as a key: index nothing under it.
                    None => "(complex key)".to_owned(),
                };
                self.stack.push(if is_map {
                    Frame::Map { path, key: None }
                } else {
                    Frame::Seq { path, next: 0 }
                });
            }
            Event::MappingEnd | Event::SequenceEnd => {
                self.stack.pop();
            }
            _ => {}
        }
    }
}

/// The offset where a scalar's text really starts, given the marker's guess.
fn fix_start(text: &str, guess: usize, value: &str, style: TScalarStyle) -> usize {
    let at = |i: usize| text.get(i..).unwrap_or("");
    let matches_at = |i: usize| match style {
        TScalarStyle::Plain => at(i).starts_with(value),
        TScalarStyle::DoubleQuoted => at(i).starts_with('"'),
        TScalarStyle::SingleQuoted => at(i).starts_with('\''),
        _ => true,
    };
    if matches_at(guess) {
        return guess;
    }
    // One character back, then one forward.
    let back = text[..guess].char_indices().next_back().map(|(i, _)| i);
    if let Some(b) = back
        && matches_at(b)
    {
        return b;
    }
    guess
}

fn line_starts(text: &str) -> Vec<usize> {
    let mut starts = vec![0];
    for (i, b) in text.bytes().enumerate() {
        if b == b'\n' {
            starts.push(i + 1);
        }
    }
    starts
}

fn line_end(text: &str, from: usize) -> usize {
    text.get(from..)
        .and_then(|r| r.find(['\n', '\r']))
        .map_or(text.len(), |i| from + i)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn text_of(text: &str, base: usize, span: Span) -> &str {
        &text[span.start() - base..span.end() - base]
    }

    #[test]
    fn finds_keys_values_and_items() {
        let yaml = "title: T\nvariant:\n  deployment: [cloud, hybrid]\ntags:\n  - a\n  - \"b c\"\nauthor: {name: K}\n";
        let index = YamlIndex::build(yaml, 10);
        let get = |p: &str| index.get(p).unwrap_or_else(|| panic!("no {p}"));
        assert_eq!(text_of(yaml, 10, get("title").key.unwrap()), "title");
        assert_eq!(text_of(yaml, 10, get("title").value), "T");
        assert!(get("title").plain);
        assert_eq!(
            text_of(yaml, 10, get("variant.deployment").key.unwrap()),
            "deployment"
        );
        assert_eq!(
            text_of(yaml, 10, get("variant.deployment[1]").value),
            "hybrid"
        );
        assert_eq!(text_of(yaml, 10, get("tags[1]").value), "\"b c\"");
        assert!(!get("tags[1]").plain);
        assert_eq!(text_of(yaml, 10, get("author.name").value), "K");
        assert!(index.get("nope").is_none());
    }

    #[test]
    fn unparsable_text_gives_an_empty_index() {
        assert!(YamlIndex::build("a: [b", 0).get("a").is_none());
    }
}
