//! Regions in a code file (SPEC §4.8): finding the tags, and taking a
//! snippet out.
//!
//! The tags are Bluehawk's, read from its documentation: `:snippet-start:`
//! and `:snippet-end:`, `:remove-start:` and `:remove-end:`, and a line
//! ending in `:remove:`. A tag counts only in a line comment, in the syntax
//! [`comment_style`] gives the file's extension, so a tag-like string in code
//! isn't one.

use ascribe_core::Span;

/// One way of writing a line comment.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Marker {
    /// What starts a comment: `//`, `#`, `--`, `;`, or `<!--`.
    pub open: &'static str,
    /// What ends one on the same line: `-->` after `<!--`, otherwise none.
    pub close: Option<&'static str>,
}

/// How a file's line comments are written: one marker, or, for a component
/// file with both script and markup (`.astro`, `.svelte`, `.vue`), either of
/// two.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CommentStyle {
    /// The markers a tag line may use.
    pub markers: &'static [Marker],
}

const SLASHES_MARKER: Marker = Marker {
    open: "//",
    close: None,
};
const HTML_MARKER: Marker = Marker {
    open: "<!--",
    close: Some("-->"),
};

const SLASHES: CommentStyle = CommentStyle {
    markers: &[SLASHES_MARKER],
};
const HASH: CommentStyle = CommentStyle {
    markers: &[Marker {
        open: "#",
        close: None,
    }],
};
const DASHES: CommentStyle = CommentStyle {
    markers: &[Marker {
        open: "--",
        close: None,
    }],
};
const SEMICOLON: CommentStyle = CommentStyle {
    markers: &[Marker {
        open: ";",
        close: None,
    }],
};
const HTML: CommentStyle = CommentStyle {
    markers: &[HTML_MARKER],
};
const COMPONENT: CommentStyle = CommentStyle {
    markers: &[SLASHES_MARKER, HTML_MARKER],
};

/// The comment table of SPEC §4.8, by extension.
const TABLE: &[(CommentStyle, &[&str])] = &[
    (
        SLASHES,
        &[
            "c", "cc", "cjs", "cpp", "cs", "cts", "cxx", "dart", "go", "gradle", "groovy", "h",
            "hpp", "java", "js", "jsonc", "jsx", "kt", "kts", "mjs", "mts", "php", "proto", "rs",
            "scala", "swift", "ts", "tsx", "zig",
        ],
    ),
    (
        HASH,
        &[
            "bash", "cfg", "conf", "ex", "exs", "fish", "hcl", "nix", "pl", "pm", "ps1", "py", "r",
            "rb", "sh", "tf", "toml", "yaml", "yml", "zsh",
        ],
    ),
    (DASHES, &["elm", "hs", "lua", "sql"]),
    (
        SEMICOLON,
        &["asm", "clj", "cljs", "el", "ini", "lisp", "scm"],
    ),
    (HTML, &["htm", "html", "md", "mdx", "svg", "xml"]),
    (COMPONENT, &["astro", "svelte", "vue"]),
];

/// The comment syntax of files with this extension, compared without regard
/// to case, or `None` for an extension the table doesn't have: such a file
/// has no tags.
pub fn comment_style(extension: &str) -> Option<CommentStyle> {
    let extension = extension.to_ascii_lowercase();
    TABLE
        .iter()
        .find(|(_, extensions)| extensions.contains(&extension.as_str()))
        .map(|(style, _)| *style)
}

/// Bluehawk's tags that Ascribe reserves for later (SPEC §4.8): each is a
/// `-start` and `-end` tag line.
const RESERVED_BLOCKS: &[&str] = &[
    "state",
    "state-remove",
    "state-uncomment",
    "replace",
    "uncomment",
    "emphasize",
];

/// Bluehawk's one-line tags that Ascribe reserves, written at the end of a
/// line as `:remove:` is.
const RESERVED_LINES: &[&str] = &["emphasize", "uncomment"];

/// A region: the lines between a `:snippet-start:` tag line and the tag line
/// that closes it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Region {
    /// Its name.
    pub name: String,
    /// The start tag line's index (from 0).
    pub start: usize,
    /// The end tag line's index.
    pub end: usize,
    /// The start tag line, as a byte range of the file.
    pub span: Span,
}

/// A problem with a file's tags. Any problem makes every snippet of the file
/// an error.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TagProblem {
    /// A start tag that nothing closes.
    Unclosed {
        /// The tag as written, such as `:snippet-start: setup`.
        tag: String,
        /// Its line's index (from 0).
        line: usize,
        /// The line, as a byte range of the file.
        span: Span,
    },
    /// An end tag with nothing to close.
    Unmatched {
        /// The tag as written.
        tag: String,
        /// Its line's index.
        line: usize,
        /// The line.
        span: Span,
    },
    /// A region name used by a second start tag.
    Duplicate {
        /// The name.
        name: String,
        /// The first start tag's line index.
        first: usize,
        /// The second's.
        line: usize,
        /// The second start tag's line.
        span: Span,
        /// The first's.
        first_span: Span,
    },
    /// A `:snippet-start:` with no name, or a name that isn't letters,
    /// digits, `-`, `_`, and `.`.
    Name {
        /// Its line's index.
        line: usize,
        /// The line.
        span: Span,
    },
    /// A tag Ascribe reserves for later.
    Reserved {
        /// The tag as written, such as `:state-start: start`.
        tag: String,
        /// Its line's index.
        line: usize,
        /// The line.
        span: Span,
    },
}

impl TagProblem {
    /// The line the problem is reported at, as a byte range of the file.
    pub fn span(&self) -> Span {
        match self {
            TagProblem::Unclosed { span, .. }
            | TagProblem::Unmatched { span, .. }
            | TagProblem::Duplicate { span, .. }
            | TagProblem::Name { span, .. }
            | TagProblem::Reserved { span, .. } => *span,
        }
    }
}

/// What a code file's tags say: its regions, the lines no snippet shows, and
/// any problems.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Tags {
    /// The regions, in the order they start.
    pub regions: Vec<Region>,
    /// For each line, whether a snippet leaves it out: a tag line, a line
    /// inside `:remove-start:` and `:remove-end:`, or a line ending in
    /// `:remove:`.
    pub hidden: Vec<bool>,
    /// Problems, in line order.
    pub problems: Vec<TagProblem>,
}

impl Tags {
    /// The region with this name.
    pub fn region(&self, name: &str) -> Option<&Region> {
        self.regions.iter().find(|r| r.name == name)
    }
}

/// One line of a file: its text without the line ending, and its byte range
/// (also without the line ending).
#[derive(Clone, Copy, Debug)]
struct Line<'a> {
    text: &'a str,
    span: Span,
}

/// The file's lines. A file that ends in a line ending has no empty last
/// line; `\r\n`, `\n`, and a lone `\r` each end a line.
fn lines(text: &str) -> Vec<Line<'_>> {
    let bytes = text.as_bytes();
    let mut out = Vec::new();
    let mut start = 0;
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'\n' => {
                out.push(Line {
                    text: &text[start..i],
                    span: Span::new(start, i),
                });
                i += 1;
                start = i;
            }
            b'\r' => {
                out.push(Line {
                    text: &text[start..i],
                    span: Span::new(start, i),
                });
                i += if bytes.get(i + 1) == Some(&b'\n') {
                    2
                } else {
                    1
                };
                start = i;
            }
            _ => i += 1,
        }
    }
    if start < bytes.len() {
        out.push(Line {
            text: &text[start..],
            span: Span::new(start, bytes.len()),
        });
    }
    out
}

/// The tag a whole line holds: what's between a comment marker and the end
/// of the line (or the closing `-->`), trimmed, when it starts with `:`.
fn tag_line(line: &str, style: CommentStyle) -> Option<&str> {
    style.markers.iter().find_map(|marker| {
        let rest = line.trim().strip_prefix(marker.open)?;
        let rest = match marker.close {
            Some(close) => rest.trim_end().strip_suffix(close)?,
            None => rest,
        };
        let tag = rest.trim();
        tag.starts_with(':').then_some(tag)
    })
}

/// The one-line tag at the end of a line, such as `:remove:`, when the line
/// ends with a comment marker, optional whitespace, and the tag.
fn ends_with_tag(line: &str, style: CommentStyle, name: &str) -> bool {
    let tag = format!(":{name}:");
    style.markers.iter().any(|marker| {
        let mut rest = line.trim_end();
        if let Some(close) = marker.close {
            let Some(r) = rest.strip_suffix(close) else {
                return false;
            };
            rest = r.trim_end();
        }
        let Some(rest) = rest.strip_suffix(tag.as_str()) else {
            return false;
        };
        rest.trim_end().ends_with(marker.open)
    })
}

/// A tag line's name and argument: `:snippet-start: setup` is
/// `("snippet-start", "setup")`.
fn split_tag(tag: &str) -> Option<(&str, &str)> {
    let rest = tag.strip_prefix(':')?;
    let colon = rest.find(':')?;
    let name = &rest[..colon];
    let valid = !name.is_empty()
        && name
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-');
    valid.then(|| (name, rest[colon + 1..].trim()))
}

/// SPEC Appendix A `region-name`.
pub(crate) fn is_region_name(name: &str) -> bool {
    !name.is_empty()
        && name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.'))
}

/// An open region: its name, start line, the tag as written, and its span.
/// One that isn't `sound` (its name is wrong or taken) makes no region.
struct Open {
    name: String,
    start: usize,
    tag: String,
    span: Span,
    sound: bool,
}

/// Finds the tags in a file with this comment syntax.
pub fn scan(text: &str, style: CommentStyle) -> Tags {
    let lines = lines(text);
    let mut tags = Tags {
        hidden: vec![false; lines.len()],
        ..Tags::default()
    };
    let mut open: Vec<Open> = Vec::new();
    // Open removal blocks: start line, the tag, its span.
    let mut removing: Vec<(usize, String, Span)> = Vec::new();
    for (i, line) in lines.iter().enumerate() {
        if let Some(tag) = tag_line(line.text, style)
            && let Some((name, argument)) = split_tag(tag)
        {
            match name {
                "snippet-start" => {
                    tags.hidden[i] = true;
                    // A start tag whose name is wrong is still open, so the
                    // end tag that closes it isn't reported as well; it just
                    // makes no region.
                    let mut sound = true;
                    if !is_region_name(argument) {
                        tags.problems.push(TagProblem::Name {
                            line: i,
                            span: line.span,
                        });
                        sound = false;
                    } else if let Some((first, first_span)) = tags
                        .regions
                        .iter()
                        .map(|r| (&r.name, r.start, r.span))
                        .chain(
                            open.iter()
                                .filter(|o| o.sound)
                                .map(|o| (&o.name, o.start, o.span)),
                        )
                        .find(|(n, ..)| *n == argument)
                        .map(|(_, first, span)| (first, span))
                    {
                        tags.problems.push(TagProblem::Duplicate {
                            name: argument.to_owned(),
                            first,
                            line: i,
                            span: line.span,
                            first_span,
                        });
                        sound = false;
                    }
                    open.push(Open {
                        name: argument.to_owned(),
                        start: i,
                        tag: tag.to_owned(),
                        span: line.span,
                        sound,
                    });
                    continue;
                }
                "snippet-end" => {
                    tags.hidden[i] = true;
                    let at = if argument.is_empty() {
                        open.len().checked_sub(1)
                    } else {
                        open.iter().rposition(|o| o.name == argument)
                    };
                    match at {
                        Some(at) => {
                            let o = open.remove(at);
                            if o.sound {
                                tags.regions.push(Region {
                                    name: o.name,
                                    start: o.start,
                                    end: i,
                                    span: o.span,
                                });
                            }
                        }
                        None => tags.problems.push(TagProblem::Unmatched {
                            tag: tag.to_owned(),
                            line: i,
                            span: line.span,
                        }),
                    }
                    continue;
                }
                "remove-start" => {
                    tags.hidden[i] = true;
                    removing.push((i, tag.to_owned(), line.span));
                    continue;
                }
                "remove-end" => {
                    tags.hidden[i] = true;
                    if removing.pop().is_none() {
                        tags.problems.push(TagProblem::Unmatched {
                            tag: tag.to_owned(),
                            line: i,
                            span: line.span,
                        });
                    }
                    continue;
                }
                other => {
                    let base = other
                        .strip_suffix("-start")
                        .or_else(|| other.strip_suffix("-end"));
                    if base.is_some_and(|b| RESERVED_BLOCKS.contains(&b))
                        || RESERVED_LINES.contains(&other)
                    {
                        tags.hidden[i] = true;
                        tags.problems.push(TagProblem::Reserved {
                            tag: tag.to_owned(),
                            line: i,
                            span: line.span,
                        });
                        continue;
                    }
                }
            }
        }
        if !removing.is_empty() || ends_with_tag(line.text, style, "remove") {
            tags.hidden[i] = true;
        }
        if let Some(name) = RESERVED_LINES
            .iter()
            .find(|name| ends_with_tag(line.text, style, name))
        {
            tags.problems.push(TagProblem::Reserved {
                tag: format!(":{name}:"),
                line: i,
                span: line.span,
            });
        }
    }
    for o in open.into_iter().filter(|o| o.sound) {
        tags.problems.push(TagProblem::Unclosed {
            tag: o.tag,
            line: o.start,
            span: o.span,
        });
    }
    for (line, tag, span) in removing {
        tags.problems.push(TagProblem::Unclosed { tag, line, span });
    }
    tags.problems.sort_by_key(|p| p.span());
    tags.regions.sort_by_key(|r| r.start);
    tags
}

/// A snippet taken out of a file.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Extracted {
    /// The code: the lines that remain, dedented, each ending in `\n`.
    pub code: String,
    /// The first and last lines of the file it covers, from 1: a region's
    /// lines between its tags, or the whole file. `None` for an empty file
    /// or region.
    pub lines: Option<(u32, u32)>,
}

/// Takes the snippet out of `text`: `region`'s lines, or every line, without
/// hidden lines or the blank lines at the start and end of what's left,
/// dedented by their common leading whitespace. `tags` is what
/// [`scan`] found in `text`; a file with no comment syntax has
/// [`Tags::default`], so nothing is hidden.
pub fn extract(text: &str, tags: &Tags, region: Option<&Region>) -> Extracted {
    let lines = lines(text);
    let (from, to) = match region {
        Some(r) => (r.start + 1, r.end),
        None => (0, lines.len()),
    };
    let blank = |l: &str| l.trim().is_empty();
    let mut kept: Vec<&str> = (from..to)
        .filter(|i| !tags.hidden.get(*i).copied().unwrap_or(false))
        .filter_map(|i| lines.get(i).map(|l| l.text))
        .collect();
    // Blank lines at the start and end go, so where a formatter puts a tag
    // line doesn't change the snippet.
    while kept.last().is_some_and(|l| blank(l)) {
        kept.pop();
    }
    let lead = kept.iter().take_while(|l| blank(l)).count();
    kept.drain(..lead);
    // The common leading whitespace, character for character, of the lines
    // that aren't blank.
    let mut common: Option<&str> = None;
    for line in kept.iter().filter(|l| !blank(l)) {
        let indent = &line[..line.len() - line.trim_start().len()];
        common = Some(match common {
            None => indent,
            Some(c) => {
                let shared = c
                    .char_indices()
                    .zip(indent.chars())
                    .take_while(|((_, a), b)| a == b)
                    .last()
                    .map_or(0, |((i, a), _)| i + a.len_utf8());
                &c[..shared]
            }
        });
    }
    let cut = common.map_or(0, str::len);
    let mut code = String::new();
    for line in &kept {
        if !blank(line) {
            code.push_str(&line[cut..]);
        }
        code.push('\n');
    }
    let lines = (from < to).then(|| (from as u32 + 1, to as u32));
    Extracted { code, lines }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hash(text: &str) -> Tags {
        scan(text, HASH)
    }

    fn region(text: &str, style: CommentStyle, name: &str) -> String {
        let tags = scan(text, style);
        assert!(tags.problems.is_empty(), "{:?}", tags.problems);
        extract(text, &tags, tags.region(name)).code
    }

    #[test]
    fn a_region_is_the_lines_between_its_tags() {
        let text = "a = 1\n# :snippet-start: two\nb = 2\nc = 3\n# :snippet-end:\nd = 4\n";
        assert_eq!(region(text, HASH, "two"), "b = 2\nc = 3\n");
        let tags = hash(text);
        assert_eq!(extract(text, &tags, tags.region("two")).lines, Some((3, 4)));
    }

    #[test]
    fn the_whole_file_leaves_out_tag_lines() {
        let text = "a = 1\n# :snippet-start: two\nb = 2\n# :snippet-end:\n";
        let tags = hash(text);
        let whole = extract(text, &tags, None);
        assert_eq!(whole.code, "a = 1\nb = 2\n");
        assert_eq!(whole.lines, Some((1, 4)));
    }

    #[test]
    fn regions_nest_and_overlap() {
        let text = "\
// :snippet-start: outer
a();
// :snippet-start: inner
b();
// :snippet-end:
c();
// :snippet-end:
// :snippet-start: one
d();
// :snippet-start: two
e();
// :snippet-end: one
f();
// :snippet-end: two
";
        assert_eq!(region(text, SLASHES, "outer"), "a();\nb();\nc();\n");
        assert_eq!(region(text, SLASHES, "inner"), "b();\n");
        assert_eq!(region(text, SLASHES, "one"), "d();\ne();\n");
        assert_eq!(region(text, SLASHES, "two"), "e();\nf();\n");
    }

    #[test]
    fn removal_leaves_lines_out() {
        let text = "\
# :snippet-start: config
keep = 1
secret = 2  # :remove:
# :remove-start:
hidden = 3
# :remove-start:
deeper = 4
# :remove-end:
hidden = 5
# :remove-end:
also = 6
# :snippet-end:
";
        assert_eq!(region(text, HASH, "config"), "keep = 1\nalso = 6\n");
        // A `:remove:` must follow the comment marker.
        let text = "x = \":remove:\"\n";
        assert_eq!(extract(text, &hash(text), None).code, text);
    }

    #[test]
    fn extraction_is_dedented() {
        let text = "\
fn main() {
    // :snippet-start: body
    let x = 1;

    if x > 0 {
        println!(\"{x}\");
    }
    // :snippet-end:
}
";
        assert_eq!(
            region(text, SLASHES, "body"),
            "let x = 1;\n\nif x > 0 {\n    println!(\"{x}\");\n}\n"
        );
        // Tabs and spaces are compared as characters: only what's shared goes.
        let text = "\t\ta\n\t b\n";
        assert_eq!(extract(text, &Tags::default(), None).code, "\ta\n b\n");
        // Whitespace-only lines are emptied and don't count.
        let text = "    a\n  \n    b\n";
        assert_eq!(extract(text, &Tags::default(), None).code, "a\n\nb\n");
    }

    #[test]
    fn blank_lines_at_the_edges_go() {
        let text = "\
    - name: Report
# :snippet-start: step

  - run: drift
  # :remove-start:
  - run: secret
  # :remove-end:
    \t

# :snippet-end:
";
        assert_eq!(region(text, HASH, "step"), "- run: drift\n");
        // Blank lines between kept lines stay.
        let text = "# :snippet-start: a\nx\n\n\ny\n# :snippet-end:\n";
        assert_eq!(region(text, HASH, "a"), "x\n\n\ny\n");
        // So do a region's lines, for what it covers.
        let tags = hash(text);
        assert_eq!(extract(text, &tags, tags.region("a")).lines, Some((2, 5)));
        // A whole file is trimmed too; a blank one is empty.
        let text = "\n# :remove:\nx\n\n";
        assert_eq!(extract(text, &hash(text), None).code, "x\n");
        assert_eq!(extract("\n\n", &Tags::default(), None).code, "");
    }

    #[test]
    fn each_comment_marker() {
        for (ext, start, end) in [
            ("rs", "// :snippet-start: r", "// :snippet-end:"),
            ("py", "# :snippet-start: r", "# :snippet-end:"),
            ("sql", "-- :snippet-start: r", "-- :snippet-end:"),
            ("ini", "; :snippet-start: r", "; :snippet-end:"),
            (
                "html",
                "<!-- :snippet-start: r -->",
                "<!-- :snippet-end: -->",
            ),
            ("XML", "<!--:snippet-start: r-->", "<!--:snippet-end:-->"),
        ] {
            let style = comment_style(ext).expect("a known extension");
            let text = format!("before\n  {start}  \nbody\n{end}\nafter\n");
            assert_eq!(region(&text, style, "r"), "body\n", "{ext}");
        }
        for ext in ["astro", "svelte", "vue"] {
            let style = comment_style(ext).expect("a known extension");
            let text = "---\n// :snippet-start: script\nconst a = 1; // :remove:\nconst b = 2;\n// :snippet-end:\n---\n<!-- :snippet-start: body -->\n<p>{b}</p>\n<i>x</i> <!-- :remove: -->\n<!-- :snippet-end: -->\n";
            assert_eq!(region(text, style, "script"), "const b = 2;\n", "{ext}");
            assert_eq!(region(text, style, "body"), "<p>{b}</p>\n", "{ext}");
        }
        assert_eq!(comment_style("json"), None);
        assert_eq!(comment_style("TOML"), Some(HASH));
    }

    #[test]
    fn a_tag_needs_its_own_comment_line() {
        // In code, or in a string, it isn't a tag.
        let text = "x = 1 # :snippet-start: a\ns = \"# :snippet-end:\"\n";
        let tags = hash(text);
        assert!(tags.regions.is_empty());
        assert!(tags.problems.is_empty());
        assert_eq!(extract(text, &tags, None).code, text);
        // In another comment syntax, it isn't either.
        let text = "// :snippet-start: a\n";
        assert!(hash(text).problems.is_empty());
    }

    #[test]
    fn crlf_and_no_trailing_newline() {
        let text = "# :snippet-start: a\r\none\r\ntwo\r\n# :snippet-end:\r\nlast";
        assert_eq!(region(text, HASH, "a"), "one\ntwo\n");
        let tags = hash(text);
        let whole = extract(text, &tags, None);
        assert_eq!(whole.code, "one\ntwo\nlast\n");
        assert_eq!(whole.lines, Some((1, 5)));
        // A lone CR ends a line too.
        assert_eq!(extract("a\rb", &Tags::default(), None).code, "a\nb\n");
        assert_eq!(extract("", &Tags::default(), None).lines, None);
    }

    #[test]
    fn unbalanced_tags_are_problems() {
        let tags = hash("# :snippet-start: a\nx\n");
        assert!(matches!(
            &tags.problems[..],
            [TagProblem::Unclosed { tag, line: 0, .. }] if tag == ":snippet-start: a"
        ));
        let tags = hash("x\n# :snippet-end:\n");
        assert!(matches!(
            &tags.problems[..],
            [TagProblem::Unmatched { line: 1, .. }]
        ));
        let tags = hash("# :snippet-start: a\n# :snippet-end: b\n# :snippet-end:\n");
        assert!(matches!(
            &tags.problems[..],
            [TagProblem::Unmatched { tag, line: 1, .. }] if tag == ":snippet-end: b"
        ));
        let tags = hash("# :remove-start:\nx\n");
        assert!(matches!(&tags.problems[..], [TagProblem::Unclosed { .. }]));
        let tags = hash("# :remove-end:\n");
        assert!(matches!(&tags.problems[..], [TagProblem::Unmatched { .. }]));
    }

    #[test]
    fn names_are_unique_and_well_formed() {
        let tags =
            hash("# :snippet-start: a\n# :snippet-end:\n# :snippet-start: a\n# :snippet-end:\n");
        assert!(matches!(
            &tags.problems[..],
            [TagProblem::Duplicate { name, first: 0, line: 2, .. }] if name == "a"
        ));
        let tags = hash("# :snippet-start:\n# :snippet-start: a b\n");
        assert_eq!(tags.problems.len(), 2);
        assert!(
            tags.problems
                .iter()
                .all(|p| matches!(p, TagProblem::Name { .. }))
        );
        assert!(is_region_name("setup.v2_final-1"));
        assert!(!is_region_name("a b"));
    }

    #[test]
    fn reserved_tags_are_problems() {
        let tags = hash("# :state-start: begin\nx\n# :state-end:\ny  # :emphasize:\n");
        assert_eq!(tags.problems.len(), 3);
        assert!(
            tags.problems
                .iter()
                .all(|p| matches!(p, TagProblem::Reserved { .. }))
        );
        // A word that only looks like a tag is code.
        assert!(hash("# :note: this is a comment\n").problems.is_empty());
    }
}
