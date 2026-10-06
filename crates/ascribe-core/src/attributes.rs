//! The attribute-block parser (SPEC §3.3, Appendix A `attributes`).
//!
//! One grammar serves directives and images, so one parser does too:
//! [`parse_attribute_block`] reads `{key=value, key=value}` from the text of
//! a single line and returns the [`AttributeBlock`] with a span for every
//! part, plus the [`Issue`]s for anything malformed. It never panics, and it
//! recovers from every error so the rest of the block is still read: the
//! language server needs the keys of a half-typed block, and the checks
//! still see the valid pairs.
//!
//! What it reports, by registry slug:
//!
//! | Slug | When |
//! |---|---|
//! | `attribute-syntax` | The block doesn't parse: no closing `}`, an unclosed quote, `=` with no value, a missing `,` or key, a bad key, an invalid escape, a malformed value set |
//! | `attribute-bare-key` | A key with no `=` and value |
//! | `attribute-unquoted-reserved` | An unquoted value with whitespace, `=`, `{`, or `"` in it |
//! | `attribute-duplicate-key` | A key given twice (reported at the second) |
//!
//! Keys and values are checked against a schema later, by the checks: this
//! parser records only the form of a value (token, quoted string, or set).
//!
//! # Where the block ends
//!
//! The block ends at the first `}` that isn't inside a double-quoted string.
//! Inside a string, a backslash hides the next character. The extent is
//! found first, then the inside is parsed, so the end is the same however
//! malformed the inside is. The block parser in `comrak-ascribe` finds the
//! end of an attribute block with the same rule, so the two agree on where a
//! text primary starts.
//!
//! ```
//! use ascribe_core::{FileId, attributes::parse_attribute_block};
//!
//! let text = r#"{type=caution, label="A \"b\""}: rest"#;
//! let parsed = parse_attribute_block(text, 10, FileId::new(0)).expect("starts with a brace");
//! assert!(parsed.issues.is_empty());
//! assert_eq!(&text[..parsed.len], r#"{type=caution, label="A \"b\""}"#);
//! assert_eq!(parsed.block.get("label").and_then(|a| a.value.as_ref()?.as_text()), Some("A \"b\""));
//! ```

use crate::{
    Attribute, AttributeBlock, AttributeValue, FileId, Issue, Location, Span, Token, diagnostics,
};

/// The result of [`parse_attribute_block`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ParsedAttributes {
    /// The block. When it isn't closed, its span runs to the end of the text.
    pub block: AttributeBlock,
    /// How many bytes of the text the block covers, from the `{` through the
    /// closing `}` (or the whole text, when the block isn't closed).
    pub len: usize,
    /// Whether the block has its closing `}`.
    pub closed: bool,
    /// What's wrong with it, in source order.
    pub issues: Vec<Issue>,
}

/// Parses the attribute block at the start of `text`.
///
/// `text` is the rest of one line, and must not contain a line ending (a
/// line ending ends the text). It returns `None` when `text` doesn't start
/// with `{`. `offset` is where `text` starts in its file, so every span and
/// issue location is in file coordinates, and `file` is the file's id for
/// those locations.
pub fn parse_attribute_block(text: &str, offset: usize, file: FileId) -> Option<ParsedAttributes> {
    if !text.starts_with('{') {
        return None;
    }
    let text = text.split(['\n', '\r']).next().unwrap_or(text);
    let (len, closed) = block_extent(text.as_bytes());
    let inner_end = if closed { len - 1 } else { len };
    let mut parser = Parser {
        text,
        bytes: text.as_bytes(),
        end: inner_end,
        i: 1,
        offset,
        file,
        issues: Vec::new(),
    };
    if !closed {
        // Reported at the opening brace: the end of the line is a poor place
        // to point at.
        parser.issue_syntax(
            Span::new(0, 1),
            "the block has no closing `}` (write `}` at the end)",
        );
    }
    let attributes = parser.attributes();
    let mut issues = parser.issues;
    report_duplicates(&attributes, file, &mut issues);
    if !closed {
        // Text after the last value is most likely what follows a forgotten
        // `}`, so calling it an unquoted value would only add noise.
        issues.retain(|i| i.slug != diagnostics::ATTRIBUTE_UNQUOTED_RESERVED);
        // An unclosed quote swallows the rest of the line, including the `}`,
        // so the quote is the one problem to report: adding the missing
        // brace would say the same thing twice (SPEC §3.3).
        let says = |i: &Issue, text: &str| i.arg("detail").is_some_and(|d| d.contains(text));
        if issues.iter().any(|i| says(i, "no closing quote")) {
            issues.retain(|i| !says(i, "no closing `}`"));
        }
    }
    issues.sort_by_key(|i| i.location.span.start());
    Some(ParsedAttributes {
        block: AttributeBlock {
            span: Span::new(offset, offset + len),
            attributes,
        },
        len,
        closed,
        issues,
    })
}

/// The extent of the block at the start of `bytes`: its length, and whether
/// it's closed. The first `}` outside a quoted string closes it.
fn block_extent(bytes: &[u8]) -> (usize, bool) {
    let mut i = 1;
    let mut quoted = false;
    while let Some(&b) = bytes.get(i) {
        match (quoted, b) {
            (true, b'\\') => i += 1,
            (_, b'"') => quoted = !quoted,
            (false, b'}') => return (i + 1, true),
            _ => {}
        }
        i += 1;
    }
    (bytes.len(), false)
}

fn report_duplicates(attributes: &[Attribute], file: FileId, issues: &mut Vec<Issue>) {
    for (n, attribute) in attributes.iter().enumerate() {
        if attributes[..n].iter().any(|a| a.key == attribute.key) {
            issues.push(
                Issue::new(
                    diagnostics::ATTRIBUTE_DUPLICATE_KEY,
                    Location::new(file, attribute.key_span),
                )
                .with_arg("key", attribute.key.clone()),
            );
        }
    }
}

struct Parser<'a> {
    text: &'a str,
    bytes: &'a [u8],
    /// Where the block's contents end: at the closing `}`, or the end of the
    /// text when there isn't one.
    end: usize,
    i: usize,
    offset: usize,
    file: FileId,
    issues: Vec<Issue>,
}

fn is_ows(b: u8) -> bool {
    b == b' ' || b == b'\t'
}

fn is_key_start(b: u8) -> bool {
    b.is_ascii_lowercase()
}

fn is_key_char(b: u8) -> bool {
    b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-'
}

impl Parser<'_> {
    fn span(&self, from: usize, to: usize) -> Span {
        Span::new(self.offset + from, self.offset + to)
    }

    fn peek(&self) -> Option<u8> {
        if self.i < self.end {
            self.bytes.get(self.i).copied()
        } else {
            None
        }
    }

    fn skip_ows(&mut self) {
        while self.peek().is_some_and(is_ows) {
            self.i += 1;
        }
    }

    /// Reports `attribute-syntax` at `span`, given relative to the text.
    fn issue_syntax(&mut self, span: Span, detail: &str) {
        let span = Span::new(self.offset + span.start(), self.offset + span.end());
        self.issues.push(
            Issue::new(
                diagnostics::ATTRIBUTE_SYNTAX,
                Location::new(self.file, span),
            )
            .with_arg("detail", detail),
        );
    }

    /// The end of the character starting at `i`.
    fn char_end(&self, i: usize) -> usize {
        let mut j = i + 1;
        while j < self.bytes.len() && !self.text.is_char_boundary(j) {
            j += 1;
        }
        j
    }

    /// Skips to the next `,` outside a quoted string, or the end of the
    /// contents.
    fn skip_to_boundary(&mut self) {
        let mut quoted = false;
        while let Some(b) = self.peek() {
            match (quoted, b) {
                (true, b'\\') => self.i += 1,
                (_, b'"') => quoted = !quoted,
                (false, b',') => return,
                _ => {}
            }
            self.i += 1;
        }
        self.i = self.i.min(self.end);
    }

    /// Whether a key and `=` start at `at`, meaning a missing comma.
    fn key_then_equals(&self, at: usize) -> bool {
        let mut j = at;
        if !self.bytes.get(j).copied().is_some_and(is_key_start) {
            return false;
        }
        while j < self.end && self.bytes.get(j).copied().is_some_and(is_key_char) {
            j += 1;
        }
        while j < self.end && self.bytes.get(j).copied().is_some_and(is_ows) {
            j += 1;
        }
        j < self.end && self.bytes.get(j) == Some(&b'=')
    }

    fn attributes(&mut self) -> Vec<Attribute> {
        let mut attributes = Vec::new();
        // True after a comma, when an attribute is required.
        let mut expect = false;
        loop {
            self.skip_ows();
            match self.peek() {
                None => {
                    if expect {
                        let at = self.i;
                        self.issue_syntax(Span::new(at, at), "expected an attribute after `,`");
                    }
                    break;
                }
                Some(b',') => {
                    let at = self.i;
                    self.issue_syntax(
                        Span::new(at, at + 1),
                        "expected an attribute before `,`; remove the extra comma",
                    );
                    self.i += 1;
                    expect = true;
                    continue;
                }
                Some(_) => {}
            }
            let before = self.i;
            attributes.push(self.attribute());
            expect = false;
            self.skip_ows();
            match self.peek() {
                None => break,
                Some(b',') => {
                    self.i += 1;
                    expect = true;
                }
                Some(_) => {
                    let at = self.i;
                    if self.key_then_equals(at) {
                        self.issue_syntax(Span::new(at, at), "expected `,` between attributes");
                    } else {
                        let to = self.char_end(at);
                        self.issue_syntax(
                            Span::new(at, to),
                            "expected `,` or `}` after the value; quote a value that contains spaces",
                        );
                        self.skip_to_boundary();
                    }
                }
            }
            // Guarantees progress however the input is shaped.
            if self.i == before {
                self.i = self.char_end(self.i).min(self.end);
            }
        }
        attributes
    }

    fn attribute(&mut self) -> Attribute {
        let key_start = self.i;
        let (key, key_end) = self.key();
        let key_span = self.span(key_start, key_end);
        if key_end == key_start {
            let to = self.char_end(key_start).min(self.end);
            self.issue_syntax(
                Span::new(key_start, to),
                "expected an attribute key: lowercase letters, digits, and hyphens, starting with a letter",
            );
            self.skip_to_boundary();
            return Attribute {
                key,
                key_span,
                value: None,
                span: key_span,
            };
        }
        self.skip_ows();
        if self.peek() != Some(b'=') {
            match self.peek() {
                None | Some(b',') => self.issues.push(
                    Issue::new(
                        diagnostics::ATTRIBUTE_BARE_KEY,
                        Location::new(self.file, key_span),
                    )
                    .with_arg("key", key.clone()),
                ),
                Some(_) => {
                    let at = self.i;
                    let to = self.char_end(at);
                    self.issue_syntax(
                        Span::new(at, to),
                        &format!("expected `=` after `{key}`, as in `{key}=<value>`"),
                    );
                    self.skip_to_boundary();
                }
            }
            return Attribute {
                key,
                key_span,
                value: None,
                span: key_span,
            };
        }
        let equals = self.i;
        self.i += 1;
        self.skip_ows();
        let value = self.value(&key, equals);
        let end = value
            .as_ref()
            .map_or(equals + 1, |v| v.span().end() - self.offset);
        Attribute {
            span: self.span(key_start, end),
            key,
            key_span,
            value,
        }
    }

    /// Reads a key. An uppercase or otherwise invalid key is reported and
    /// kept, so its value is still read.
    fn key(&mut self) -> (String, usize) {
        let start = self.i;
        let valid = self.peek().is_some_and(is_key_start);
        if valid {
            while self.peek().is_some_and(is_key_char) {
                self.i += 1;
            }
            // A key that continues with letters the grammar doesn't allow,
            // such as `Type` or `my_key`, is one bad key, not two tokens.
            let good_end = self.i;
            while self
                .peek()
                .is_some_and(|b| !is_ows(b) && !matches!(b, b'=' | b',' | b'"' | b'|'))
            {
                self.i += 1;
            }
            if self.i > good_end {
                let key = &self.text[start..self.i];
                self.issue_syntax(
                    Span::new(start, self.i),
                    &format!(
                        "`{key}` isn't a valid key: keys are lowercase letters, digits, and hyphens"
                    ),
                );
            }
        } else if self
            .peek()
            .is_some_and(|b| b.is_ascii_alphabetic() || b == b'_')
        {
            while self
                .peek()
                .is_some_and(|b| !is_ows(b) && !matches!(b, b'=' | b',' | b'"' | b'|'))
            {
                self.i += 1;
            }
            let key = &self.text[start..self.i];
            self.issue_syntax(
                Span::new(start, self.i),
                &format!(
                    "`{key}` isn't a valid key: keys are lowercase letters, digits, and hyphens, starting with a letter"
                ),
            );
        }
        (self.text[start..self.i].to_owned(), self.i)
    }

    fn value(&mut self, key: &str, equals: usize) -> Option<AttributeValue> {
        match self.peek() {
            None | Some(b',') => {
                self.issue_syntax(
                    Span::new(equals, equals + 1),
                    &format!("`{key}=` needs a value after the `=`"),
                );
                None
            }
            Some(b'|') => {
                let at = self.i;
                self.issue_syntax(
                    Span::new(at, at + 1),
                    &format!("`{key}=` needs a value before `|`"),
                );
                self.skip_to_boundary();
                None
            }
            Some(b'"') => Some(self.quoted()),
            Some(_) => Some(self.token_or_set(key)),
        }
    }

    fn quoted(&mut self) -> AttributeValue {
        let start = self.i;
        self.i += 1;
        let mut text = String::new();
        let mut closed = false;
        while self.i < self.end {
            let Some(rest) = self.text.get(self.i..) else {
                break;
            };
            let Some(c) = rest.chars().next() else { break };
            match c {
                '"' => {
                    self.i += 1;
                    closed = true;
                    break;
                }
                '\\' => {
                    let next = self.text.get(self.i + 1..).and_then(|r| r.chars().next());
                    match next {
                        Some(n @ ('"' | '\\')) if self.i + 1 < self.end => {
                            text.push(n);
                            self.i += 2;
                        }
                        other => {
                            let to = match other {
                                Some(n) if self.i + 1 < self.end => self.i + 1 + n.len_utf8(),
                                _ => self.i + 1,
                            };
                            self.issue_syntax(
                                Span::new(self.i, to),
                                "`\\` can only escape `\"` or `\\` inside a quoted value; write `\\\\` for a backslash",
                            );
                            text.push('\\');
                            self.i += 1;
                        }
                    }
                }
                c => {
                    text.push(c);
                    self.i += c.len_utf8();
                }
            }
        }
        if !closed {
            self.issue_syntax(
                Span::new(start, self.i),
                "this quoted value has no closing quote",
            );
        }
        AttributeValue::Quoted {
            text,
            span: self.span(start, self.i),
        }
    }

    /// Reads a token run, up to whitespace, `,`, `|`, or the end. Returns the
    /// end and the first reserved character in it, if any.
    fn token_run(&mut self) -> (usize, Option<char>) {
        let mut reserved = None;
        while let Some(b) = self.peek() {
            if is_ows(b) || b == b',' || b == b'|' {
                break;
            }
            if reserved.is_none() && matches!(b, b'{' | b'}' | b'=' | b'"') {
                reserved = Some(b as char);
            }
            self.i += 1;
        }
        (self.i, reserved)
    }

    fn token_or_set(&mut self, key: &str) -> AttributeValue {
        let start = self.i;
        let (end, reserved) = self.token_run();
        let first = Token {
            text: self.text[start..end].to_owned(),
            span: self.span(start, end),
        };
        // A value set: tokens joined by `|`.
        let mut probe = self.i;
        while probe < self.end && is_ows(self.bytes[probe]) {
            probe += 1;
        }
        if self.bytes.get(probe) == Some(&b'|') && probe < self.end {
            return self.set(key, first, reserved, probe);
        }

        if let Some(c) = reserved {
            self.reserved_issue(key, start, end, &c.to_string());
            return AttributeValue::Token(first);
        }
        // Whitespace inside an unquoted value: `label=Using other images`.
        // The value is everything up to the next comma, unless what follows
        // is another `key=`, which means the comma is missing.
        if probe > self.i
            && probe < self.end
            && self.bytes[probe] != b','
            && !self.key_then_equals(probe)
        {
            self.i = probe;
            self.skip_to_boundary();
            let mut to = self.i;
            while to > start && is_ows(self.bytes[to - 1]) {
                to -= 1;
            }
            self.i = to;
            self.reserved_issue(key, start, to, "whitespace");
            return AttributeValue::Token(Token {
                text: self.text[start..to].to_owned(),
                span: self.span(start, to),
            });
        }
        AttributeValue::Token(first)
    }

    fn reserved_issue(&mut self, key: &str, from: usize, to: usize, character: &str) {
        let value = self.text[from..to].to_owned();
        self.issues.push(
            Issue::new(
                diagnostics::ATTRIBUTE_UNQUOTED_RESERVED,
                Location::new(self.file, self.span(from, to)),
            )
            .with_arg("key", key)
            .with_arg("character", character)
            .with_arg("value", value),
        );
    }

    fn set(
        &mut self,
        key: &str,
        first: Token,
        first_reserved: Option<char>,
        mut probe: usize,
    ) -> AttributeValue {
        let start = first.span.start() - self.offset;
        let mut members = vec![first];
        let mut end = members[0].span.end() - self.offset;
        if let Some(c) = first_reserved {
            self.member_issue(key, members[0].span, c);
        }
        let mut dangling = None;
        while self.bytes.get(probe) == Some(&b'|') && probe < self.end {
            let bar = probe;
            self.i = probe + 1;
            self.skip_ows();
            let starts_member = self.peek().is_some_and(|b| b != b',' && b != b'|');
            if !starts_member {
                self.issue_syntax(
                    Span::new(bar, bar + 1),
                    &format!("`{key}` has a `|` with no value after it"),
                );
                dangling = Some(bar + 1);
                break;
            }
            let member_start = self.i;
            let (member_end, reserved) = self.token_run();
            let member = Token {
                text: self.text[member_start..member_end].to_owned(),
                span: self.span(member_start, member_end),
            };
            if let Some(c) = reserved {
                self.member_issue(key, member.span, c);
            }
            end = member_end;
            members.push(member);
            probe = self.i;
            while probe < self.end && is_ows(self.bytes[probe]) {
                probe += 1;
            }
        }
        self.i = dangling.unwrap_or(end);
        if members.len() == 1 {
            // `a|` with nothing after the bar: keep the single token.
            return AttributeValue::Token(members.remove(0));
        }
        AttributeValue::Set {
            members,
            span: self.span(start, end),
        }
    }

    fn member_issue(&mut self, key: &str, span: Span, c: char) {
        let detail = if c == '"' {
            format!("the members of a value set for `{key}` are never quoted")
        } else {
            format!("a member of the value set for `{key}` can't contain `{c}`")
        };
        self.issues.push(
            Issue::new(
                diagnostics::ATTRIBUTE_SYNTAX,
                Location::new(self.file, span),
            )
            .with_arg("detail", detail),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(text: &str) -> ParsedAttributes {
        parse_attribute_block(text, 0, FileId::new(0)).expect("starts with a brace")
    }

    fn slugs(p: &ParsedAttributes) -> Vec<&'static str> {
        p.issues.iter().map(|i| i.slug.as_str()).collect()
    }

    fn pairs(p: &ParsedAttributes) -> Vec<(String, Option<String>)> {
        p.block
            .attributes
            .iter()
            .map(|a| {
                (
                    a.key.clone(),
                    a.value.as_ref().map(|v| match v {
                        AttributeValue::Set { members, .. } => members
                            .iter()
                            .map(|m| m.text.as_str())
                            .collect::<Vec<_>>()
                            .join("|"),
                        v => v.as_text().unwrap_or_default().to_owned(),
                    }),
                )
            })
            .collect()
    }

    fn kv(k: &str, v: &str) -> (String, Option<String>) {
        (k.to_owned(), Some(v.to_owned()))
    }

    #[test]
    fn requires_an_opening_brace() {
        assert!(parse_attribute_block("type=tip", 0, FileId::new(0)).is_none());
        assert!(parse_attribute_block("", 0, FileId::new(0)).is_none());
    }

    #[test]
    fn reads_empty_blocks_with_any_spacing() {
        for text in ["{}", "{ }", "{ \t }"] {
            let p = parse(text);
            assert!(p.issues.is_empty(), "{text}");
            assert!(p.block.attributes.is_empty());
            assert_eq!(p.len, text.len());
            assert!(p.closed);
        }
    }

    #[test]
    fn reads_tokens_quoted_strings_and_sets() {
        let p = parse(
            r#"{type=caution, since=3.4, label="Using other images", platform=cloud|on-prem}"#,
        );
        assert!(p.issues.is_empty(), "{:?}", p.issues);
        assert_eq!(
            pairs(&p),
            vec![
                kv("type", "caution"),
                kv("since", "3.4"),
                kv("label", "Using other images"),
                kv("platform", "cloud|on-prem"),
            ]
        );
    }

    #[test]
    fn spacing_is_ignored_everywhere() {
        let a = parse("{a=b,c=d|e}");
        let b = parse("{ a = b , c = d | e }");
        let c = parse("{\ta\t=\tb\t,\tc\t=\td\t|\te\t}");
        assert!(b.issues.is_empty() && c.issues.is_empty());
        assert_eq!(pairs(&a), pairs(&b));
        assert_eq!(pairs(&a), pairs(&c));
    }

    #[test]
    fn quoted_escapes() {
        let p = parse(r#"{a="x \"y\" \\ z"}"#);
        assert!(p.issues.is_empty());
        assert_eq!(pairs(&p), vec![kv("a", r#"x "y" \ z"#)]);
        // Reserved characters are fine inside quotes.
        let p = parse(r#"{a="1, 2 | {3} = \"4\""}"#);
        assert!(p.issues.is_empty(), "{:?}", p.issues);
        assert_eq!(p.len, r#"{a="1, 2 | {3} = \"4\""}"#.len());
        // Single quotes mean nothing: the value is a token with a quote in it.
        let p = parse("{a='b'}");
        assert!(p.issues.is_empty());
        assert_eq!(pairs(&p), vec![kv("a", "'b'")]);
    }

    #[test]
    fn spans_cover_exactly_their_text() {
        let text = r#"{ type = caution , label="x y", sets=a | b }"#;
        let p = parse_attribute_block(text, 100, FileId::new(0)).unwrap();
        let cut = |s: Span| &text[s.start() - 100..s.end() - 100];
        assert_eq!(cut(p.block.span), text);
        let a = &p.block.attributes;
        assert_eq!(cut(a[0].key_span), "type");
        assert_eq!(cut(a[0].value.as_ref().unwrap().span()), "caution");
        assert_eq!(cut(a[0].span), "type = caution");
        assert_eq!(cut(a[1].value.as_ref().unwrap().span()), "\"x y\"");
        let AttributeValue::Set { members, span } = a[2].value.as_ref().unwrap() else {
            panic!("a set");
        };
        assert_eq!(cut(*span), "a | b");
        assert_eq!(cut(members[1].span), "b");
    }

    #[test]
    fn reports_bare_keys() {
        for text in ["{heading}", "{heading, x=y}", "{ heading }"] {
            let p = parse(text);
            assert_eq!(slugs(&p), vec!["attribute-bare-key"], "{text}");
            assert_eq!(p.issues[0].arg("key"), Some("heading"));
            assert!(p.block.attributes[0].value.is_none());
        }
    }

    #[test]
    fn reports_unclosed_blocks_and_quotes() {
        let p = parse("{type=tip");
        assert_eq!(slugs(&p), vec!["attribute-syntax"]);
        assert!(!p.closed);
        assert_eq!(pairs(&p), vec![kv("type", "tip")]);
        let p = parse(r#"{label="oops}"#);
        assert!(!p.closed);
        assert_eq!(slugs(&p), vec!["attribute-syntax"]);
        assert_eq!(p.issues[0].location.span.start(), 7);
        let p = parse("{");
        assert_eq!(slugs(&p), vec!["attribute-syntax"]);
    }

    #[test]
    fn reports_missing_values() {
        for text in ["{type=}", "{type=,a=b}", "{type= }", "{type=|a}"] {
            let p = parse(text);
            assert!(slugs(&p).contains(&"attribute-syntax"), "{text}");
            assert!(p.block.attributes[0].value.is_none(), "{text}");
        }
    }

    #[test]
    fn reports_comma_problems() {
        for text in ["{a=b,}", "{,a=b}", "{a=b,,c=d}", "{,}"] {
            let p = parse(text);
            assert!(slugs(&p).contains(&"attribute-syntax"), "{text}");
        }
        let p = parse("{a=b c=d}");
        assert_eq!(slugs(&p), vec!["attribute-syntax"]);
        assert_eq!(pairs(&p), vec![kv("a", "b"), kv("c", "d")]);
    }

    #[test]
    fn reports_unquoted_reserved_characters() {
        let p = parse("{label=Using other images}");
        assert_eq!(slugs(&p), vec!["attribute-unquoted-reserved"]);
        assert_eq!(p.issues[0].arg("value"), Some("Using other images"));
        assert_eq!(p.issues[0].arg("character"), Some("whitespace"));
        let p = parse("{a=b=c}");
        assert_eq!(slugs(&p), vec!["attribute-unquoted-reserved"]);
        assert_eq!(p.issues[0].arg("character"), Some("="));
        assert_eq!(pairs(&p), vec![kv("a", "b=c")]);
        let p = parse("{a=x{y}");
        assert_eq!(slugs(&p), vec!["attribute-unquoted-reserved"]);
    }

    #[test]
    fn reports_duplicate_keys_at_the_second() {
        let p = parse("{type=tip, type=note}");
        assert_eq!(slugs(&p), vec!["attribute-duplicate-key"]);
        assert_eq!(p.issues[0].location.span, Span::new(11, 15));
        assert_eq!(p.block.attributes.len(), 2);
    }

    #[test]
    fn reports_bad_keys_and_escapes() {
        let p = parse("{Type=tip}");
        assert_eq!(slugs(&p), vec!["attribute-syntax"]);
        assert_eq!(pairs(&p), vec![kv("Type", "tip")]);
        let p = parse("{my_key=1}");
        assert_eq!(slugs(&p), vec!["attribute-syntax"]);
        let p = parse("{=x}");
        assert_eq!(slugs(&p), vec!["attribute-syntax"]);
        let p = parse(r#"{a="\n"}"#);
        assert_eq!(slugs(&p), vec!["attribute-syntax"]);
        let p = parse("{a=b|}");
        assert_eq!(slugs(&p), vec!["attribute-syntax"]);
        let p = parse(r#"{a="b"|c}"#);
        assert!(slugs(&p).contains(&"attribute-syntax"));
    }

    #[test]
    fn stops_at_the_first_closing_brace_outside_quotes() {
        let p = parse(r#"{a="}"}: text}"#);
        assert_eq!(p.len, 7);
        assert!(p.closed);
        let p = parse("{a=b}}");
        assert_eq!(p.len, 5);
    }

    #[test]
    fn line_endings_end_the_text() {
        let p = parse("{a=b\nc=d}");
        assert!(!p.closed);
        assert_eq!(p.len, 4);
    }

    #[test]
    fn handles_non_ascii() {
        let p = parse("{label=\"héllo → wörld\", x=ü}");
        assert!(p.issues.is_empty(), "{:?}", p.issues);
        assert_eq!(pairs(&p)[0], kv("label", "héllo → wörld"));
        assert_eq!(pairs(&p)[1], kv("x", "ü"));
        let p = parse("{é=1}");
        assert!(!p.issues.is_empty());
        let p = parse("{a=\"\\é\"}");
        assert!(!p.issues.is_empty());
    }
}
