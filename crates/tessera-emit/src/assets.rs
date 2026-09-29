//! Assets: where copies go and how references to them are written
//! (`project-docs/contracts/assets.md`, §3 and §4).

use tessera_core::RelPath;

/// Where an emitter put an asset, and how a page refers to it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Placement {
    /// The copy's path inside the emitter root.
    pub copy_to: RelPath,
    /// How the page writes the reference, before any `#fragment`: relative to
    /// the page's output directory for a mirrored copy, a URL for a published
    /// one. Not yet escaped for markdown (see [`markdown_destination`]).
    pub reference: String,
    /// For a copy the consumer must serve, the URL pages use for it. It's
    /// recorded in the manifest (output-layout contract, §3).
    pub url: Option<String>,
}

/// The mirrored path of an asset (asset contract, §3.1): the source path
/// itself inside the content root, and `_ascribe/up/…` outside it, with each
/// leading `..` replaced by `up`.
pub fn mirrored_path(source: &RelPath) -> RelPath {
    let ups = source.up_count();
    if ups == 0 {
        return source.clone();
    }
    let mut path = String::from("_ascribe");
    for _ in 0..ups {
        path.push_str("/up");
    }
    for segment in source.segments().skip(ups) {
        path.push('/');
        path.push_str(segment);
    }
    RelPath::parse(&path).unwrap_or_else(|_| source.clone())
}

/// The reference from the page written at `page_output` to `copy`: relative
/// to the page's directory, and always starting with `./` or `../` (asset
/// contract, §4).
pub fn relative_reference(page_output: &RelPath, copy: &RelPath) -> String {
    let dir = page_output.parent().unwrap_or_else(RelPath::root);
    copy.relative_from(&dir)
        .unwrap_or_else(|| format!("./{copy}"))
}

/// Percent-encodes the characters of a file path that a reader would take for
/// an escape, a fragment, or a query: `%`, `#`, and `?` (asset contract, §4).
/// Spaces and parentheses stay, and [`markdown_destination`] brackets them.
pub fn encode_path(path: &str) -> String {
    let mut out = String::with_capacity(path.len());
    for ch in path.chars() {
        match ch {
            '%' => out.push_str("%25"),
            '#' => out.push_str("%23"),
            '?' => out.push_str("%3F"),
            _ => out.push(ch),
        }
    }
    out
}

/// Writes `raw` as a markdown link destination that CommonMark reads back as
/// exactly `raw`: in angle brackets when it's empty or has a space, a
/// parenthesis, an angle bracket, a backslash, or a control character, with
/// `<`, `>`, and `\` backslash-escaped; and with a `&` that would start a
/// character reference escaped.
pub fn markdown_destination(raw: &str) -> String {
    let bracket = raw.is_empty()
        || raw.chars().any(|c| {
            c.is_whitespace() || c.is_control() || matches!(c, '(' | ')' | '<' | '>' | '\\')
        });
    let chars: Vec<char> = raw.chars().collect();
    let mut out = String::with_capacity(raw.len() + 2);
    if bracket {
        out.push('<');
    }
    for (i, &ch) in chars.iter().enumerate() {
        match ch {
            '<' | '>' | '\\' if bracket => {
                out.push('\\');
                out.push(ch);
            }
            '&' if starts_reference(&chars[i + 1..]) => out.push_str("\\&"),
            _ => out.push(ch),
        }
    }
    if bracket {
        out.push('>');
    }
    out
}

/// Whether the characters after a `&` look like the rest of a character
/// reference (`amp;`, `#38;`).
pub(crate) fn starts_reference(rest: &[char]) -> bool {
    let name = rest
        .iter()
        .take_while(|c| c.is_ascii_alphanumeric() || **c == '#')
        .count();
    name > 0 && rest.get(name) == Some(&';')
}

#[cfg(test)]
mod tests {
    use super::*;

    fn path(text: &str) -> RelPath {
        RelPath::parse(text).expect("a path")
    }

    #[test]
    fn mirrored_paths() {
        assert_eq!(mirrored_path(&path("img/a.png")).as_str(), "img/a.png");
        assert_eq!(
            mirrored_path(&path("../shared/logo.png")).as_str(),
            "_ascribe/up/shared/logo.png"
        );
        assert_eq!(
            mirrored_path(&path("../../x.png")).as_str(),
            "_ascribe/up/up/x.png"
        );
    }

    #[test]
    fn references_are_relative_to_the_page() {
        let page = path("guides/install.md");
        assert_eq!(
            relative_reference(&page, &path("_fragments/diagram.png")),
            "../_fragments/diagram.png"
        );
        assert_eq!(
            relative_reference(&page, &path("guides/img/settings.png")),
            "./img/settings.png"
        );
        assert_eq!(
            relative_reference(&path("index.md"), &path("playground.png")),
            "./playground.png"
        );
        assert_eq!(
            relative_reference(&path("a/b/c.md"), &path("_ascribe/up/x.png")),
            "../../_ascribe/up/x.png"
        );
    }

    #[test]
    fn file_names_are_encoded_only_where_needed() {
        assert_eq!(encode_path("a b/c%d#e?f.png"), "a b/c%25d%23e%3Ff.png");
    }

    #[test]
    fn destinations_read_back_as_written() {
        assert_eq!(markdown_destination("./a.png"), "./a.png");
        assert_eq!(
            markdown_destination("../My Diagrams/a.png"),
            "<../My Diagrams/a.png>"
        );
        assert_eq!(markdown_destination("./a(1).png"), "<./a(1).png>");
        assert_eq!(markdown_destination("./a<b>.png"), "<./a\\<b\\>.png>");
        assert_eq!(markdown_destination(""), "<>");
        assert_eq!(markdown_destination("./a?x=1&amp;y"), "./a?x=1\\&amp;y");
        assert_eq!(markdown_destination("./a?x=1&y=2"), "./a?x=1&y=2");
    }
}
