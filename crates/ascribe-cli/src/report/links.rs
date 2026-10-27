//! Help written in Markdown, as the diagnostics registry writes it, made
//! readable in a terminal: a link becomes its text, and a link to an
//! address keeps the address after it. A link in a code span is code, so
//! it stays as written.

use std::borrow::Cow;

/// `text` with each Markdown link outside a code span written plainly:
/// ``[`[checks]`](content-model/checks.md#22-checks)`` as `` `[checks]` ``,
/// and `[Vale](https://vale.sh)` as `Vale (https://vale.sh)`.
pub fn plain(text: &str) -> Cow<'_, str> {
    if !text.contains("](") {
        return Cow::Borrowed(text);
    }
    let mut out = String::with_capacity(text.len());
    let mut at = 0;
    while at < text.len() {
        let rest = &text[at..];
        if rest.starts_with('`') {
            let span = code_span(rest);
            out.push_str(&rest[..span]);
            at += span;
        } else if let Some((label, target, len)) = link(rest) {
            out.push_str(label);
            if target.contains("://") && target != label {
                out.push_str(&format!(" ({target})"));
            }
            at += len;
        } else {
            let c = rest.chars().next().map_or(1, char::len_utf8);
            out.push_str(&rest[..c]);
            at += c;
        }
    }
    Cow::Owned(out)
}

/// The length of the code span `text` starts with: its opening backticks to
/// the closing run of as many. Backticks that open nothing are themselves.
fn code_span(text: &str) -> usize {
    let ticks = text.bytes().take_while(|&b| b == b'`').count();
    let mut at = ticks;
    while let Some(found) = text[at..].find('`') {
        let start = at + found;
        let run = text[start..].bytes().take_while(|&b| b == b'`').count();
        if run == ticks {
            return start + run;
        }
        at = start + run;
    }
    ticks
}

/// The link `text` starts with: its label, its destination, and its length.
/// The label may hold code spans and balanced brackets.
fn link(text: &str) -> Option<(&str, &str, usize)> {
    if !text.starts_with('[') {
        return None;
    }
    let mut depth = 0;
    let mut at = 0;
    let close = loop {
        let rest = &text[at..];
        let c = rest.chars().next()?;
        match c {
            '`' => {
                at += code_span(rest);
                continue;
            }
            '[' => depth += 1,
            ']' => {
                depth -= 1;
                if depth == 0 {
                    break at;
                }
            }
            _ => {}
        }
        at += c.len_utf8();
    };
    let after = text[close + 1..].strip_prefix('(')?;
    let end = after.find([')', ' ', '\n'])?;
    if !after[end..].starts_with(')') || end == 0 {
        return None;
    }
    Some((&text[1..close], &after[..end], close + 1 + 1 + end + 1))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_docs_link_becomes_its_text() {
        assert_eq!(
            plain(
                "turn the check off in [`[checks]`](content-model/checks.md#22-checks) (`page-orphan = \"off\"`)."
            ),
            "turn the check off in `[checks]` (`page-orphan = \"off\"`)."
        );
        assert_eq!(
            plain("See the [diagnostics reference](checks/diagnostics.md)."),
            "See the diagnostics reference."
        );
    }

    #[test]
    fn an_address_is_kept() {
        assert_eq!(
            plain("Install [Vale](https://vale.sh) first."),
            "Install Vale (https://vale.sh) first."
        );
    }

    #[test]
    fn code_and_text_that_isnt_a_link_stay() {
        for text in [
            "Write `![Alt](image.png)` instead.",
            "A list `[a, b]` (and more).",
            "No links here.",
            "[not a link] (spaced)",
            "Unclosed [bracket](",
            "A stray ` backtick [x](y.md",
        ] {
            assert_eq!(plain(text), text, "{text:?}");
        }
        assert_eq!(
            plain("``a `[x](y.md)` b`` [z](w.md)"),
            "``a `[x](y.md)` b`` z"
        );
    }
}
