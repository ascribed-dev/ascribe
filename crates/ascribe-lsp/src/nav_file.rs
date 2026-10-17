//! Completion in `ascribe.nav` (SPEC §7.8): the keys, a `folder` from the
//! project's sections the file doesn't list yet, and in `pages` the
//! section's pages it doesn't name yet. The file's problems come from the
//! checks across the project, which read it as the editor has it.
//!
//! As in a page, the context is the text before the cursor, since a file
//! being typed rarely reads as TOML.

use std::collections::BTreeMap;

use ascribe_resolve::{Navigation, Section};
use lsp_types::{CompletionItem, CompletionItemKind, CompletionTextEdit, Range, TextEdit};

/// The project's sections, by folder, each with the file names `pages` can
/// name: its pages, its own page not among them.
pub(crate) fn sections(navigation: &Navigation) -> BTreeMap<String, Vec<String>> {
    let mut out = BTreeMap::new();
    let mut stack: Vec<&Section> = vec![&navigation.root];
    while let Some(section) = stack.pop() {
        let names = section
            .pages
            .iter()
            .map(|p| p.as_str().rsplit('/').next().unwrap_or_default().to_owned())
            .collect();
        out.insert(section.folder.clone(), names);
        stack.extend(section.sections.iter());
    }
    out
}

/// What can be written at `offset` of `text`. `range` gives the protocol's
/// range of a span of the text.
pub(crate) fn complete(
    text: &str,
    offset: usize,
    sections: &BTreeMap<String, Vec<String>>,
    range: &dyn Fn(usize, usize) -> Option<Range>,
) -> Vec<CompletionItem> {
    let offset = offset.min(text.len());
    if !text.is_char_boundary(offset) {
        return Vec::new();
    }
    let line_start = text[..offset].rfind('\n').map_or(0, |i| i + 1);
    let before = &text[line_start..offset];
    // The `[[sections]]` table the cursor is in: from its header to the next.
    let is_header = |start: usize| {
        text[start..]
            .lines()
            .next()
            .is_some_and(|line| line.trim_start().starts_with('['))
    };
    let header = text[..line_start]
        .match_indices('\n')
        .map(|(i, _)| i + 1)
        .chain(std::iter::once(0))
        .filter(|&start| start < line_start && is_header(start))
        .max();
    let in_section = header.is_some_and(|start| {
        let line = text[start..].lines().next().unwrap_or_default();
        line.split_whitespace()
            .collect::<String>()
            .starts_with("[[sections]]")
    });
    let table_end = text[offset..]
        .match_indices('\n')
        .map(|(i, _)| offset + i + 1)
        .find(|&start| is_header(start))
        .unwrap_or(text.len());
    let table = &text[header.unwrap_or(0)..table_end];

    // A value: inside a string, or right after `=` or `[` or `,`.
    let value = |key: &str| -> Option<&str> {
        let rest = before.trim_start().strip_prefix(key)?;
        rest.trim_start().strip_prefix('=')
    };
    if in_section && let Some(after) = value("folder") {
        let Some((typed, quoted)) = typed_string(after) else {
            return Vec::new();
        };
        let listed = written_folders(text);
        return sections
            .keys()
            .filter(|folder| !listed.contains(&folder.as_str()))
            .map(|folder| item(folder, "A section", typed, quoted, offset, range))
            .collect();
    }
    if in_section && let Some(inside) = in_pages(table, header.unwrap_or(0), offset, text) {
        let Some(folder) = folder_of(table) else {
            return Vec::new();
        };
        let Some(names) = sections.get(folder.as_str()) else {
            return Vec::new();
        };
        let (typed, quoted) = typed_string(inside).unwrap_or(("", false));
        let named = written_strings(inside_list(table));
        return names
            .iter()
            .filter(|name| !named.contains(&name.as_str()))
            .map(|name| item(name, "A page of the section", typed, quoted, offset, range))
            .collect();
    }
    // A key, at the start of a line.
    let typed = before.trim_start();
    if !typed
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == '[')
    {
        return Vec::new();
    }
    let written = |key: &str| {
        let scope = if in_section {
            table
        } else {
            &text[..header.unwrap_or(text.len())]
        };
        scope.lines().any(|l| {
            l.trim_start()
                .strip_prefix(key)
                .is_some_and(|rest| rest.trim_start().starts_with('='))
        })
    };
    let keys: &[(&str, &str)] = if in_section {
        &[
            (
                "folder",
                "The section's folder, from the content root, ending in `/`",
            ),
            ("pages", "Its pages' file names, in order"),
            ("label", "Its label"),
        ]
    } else {
        &[("version", "The file's format: 1")]
    };
    let mut out: Vec<CompletionItem> = keys
        .iter()
        .filter(|(key, _)| !written(key))
        .map(|(key, detail)| CompletionItem {
            label: (*key).to_owned(),
            kind: Some(CompletionItemKind::PROPERTY),
            detail: Some((*detail).to_owned()),
            ..CompletionItem::default()
        })
        .collect();
    out.push(CompletionItem {
        label: "[[sections]]".to_owned(),
        kind: Some(CompletionItemKind::MODULE),
        detail: Some("A section, in the order it comes".to_owned()),
        insert_text: Some("[[sections]]\nfolder = ".to_owned()),
        ..CompletionItem::default()
    });
    out
}

/// A completion of a string value: `value` replaces what's typed of it,
/// quoted when the quote isn't typed yet.
fn item(
    value: &str,
    detail: &str,
    typed: &str,
    quoted: bool,
    offset: usize,
    range: &dyn Fn(usize, usize) -> Option<Range>,
) -> CompletionItem {
    let new_text = if quoted {
        value.to_owned()
    } else {
        format!("\"{value}\"")
    };
    CompletionItem {
        label: value.to_owned(),
        kind: Some(CompletionItemKind::FILE),
        detail: Some(detail.to_owned()),
        filter_text: Some(if quoted {
            value.to_owned()
        } else {
            new_text.clone()
        }),
        text_edit: range(offset - typed.len(), offset)
            .map(|range| CompletionTextEdit::Edit(TextEdit { range, new_text })),
        ..CompletionItem::default()
    }
}

/// What's typed of a string value after `=` or in a list: the text after
/// its opening quote, and whether there's one. `None` when the cursor is
/// past a closed string.
fn typed_string(after: &str) -> Option<(&str, bool)> {
    let rest = after
        .rsplit([',', '['])
        .next()
        .unwrap_or(after)
        .trim_start();
    match rest.strip_prefix('"') {
        Some(typed) if !typed.contains('"') => Some((typed, true)),
        Some(_) => None,
        None if rest.is_empty() => Some(("", false)),
        None => None,
    }
}

/// The text of a `pages` list before `offset`, when the cursor is in one.
fn in_pages<'a>(table: &str, table_start: usize, offset: usize, text: &'a str) -> Option<&'a str> {
    let key = table.find("pages")?;
    let open = table_start + key + table[key..].find('[')?;
    if open >= offset {
        return None;
    }
    let inside = &text[open + 1..offset];
    (!inside.contains(']')).then_some(inside)
}

/// A table's `pages` list, inside its brackets, as far as it's written.
fn inside_list(table: &str) -> &str {
    let Some(key) = table.find("pages") else {
        return "";
    };
    let Some(open) = table[key..].find('[') else {
        return "";
    };
    let rest = &table[key + open + 1..];
    &rest[..rest.find(']').unwrap_or(rest.len())]
}

/// A table's `folder`, when it's written.
fn folder_of(table: &str) -> Option<String> {
    table.lines().find_map(|line| {
        let rest = line.trim_start().strip_prefix("folder")?;
        let value = rest.trim_start().strip_prefix('=')?.trim();
        let value = value.strip_prefix('"')?;
        Some(value[..value.find('"')?].to_owned())
    })
}

/// The folders the file lists.
fn written_folders(text: &str) -> Vec<&str> {
    text.lines()
        .filter_map(|line| {
            let rest = line.trim_start().strip_prefix("folder")?;
            let value = rest.trim_start().strip_prefix('=')?.trim();
            let value = value.strip_prefix('"')?;
            Some(&value[..value.find('"')?])
        })
        .collect()
}

/// The strings closed in a list's text.
fn written_strings(list: &str) -> Vec<&str> {
    list.split('"').skip(1).step_by(2).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn labels(text: &str) -> Vec<String> {
        let offset = text.find('|').expect("a cursor");
        let text = text.replace('|', "");
        let sections = BTreeMap::from([
            (String::new(), vec!["install.md".to_owned()]),
            (
                "Reference/".to_owned(),
                vec!["Options.md".to_owned(), "Defaults.md".to_owned()],
            ),
            ("Guides/".to_owned(), vec!["Setup.md".to_owned()]),
        ]);
        complete(&text, offset, &sections, &|_, _| None)
            .into_iter()
            .map(|i| i.label)
            .collect()
    }

    #[test]
    fn keys() {
        assert_eq!(labels("|"), ["version", "[[sections]]"]);
        assert_eq!(labels("version = 1\n|"), ["[[sections]]"]);
        assert_eq!(
            labels("version = 1\n[[sections]]\nfolder = \"\"\n|"),
            ["pages", "label", "[[sections]]"]
        );
    }

    #[test]
    fn folders_not_listed_yet() {
        assert_eq!(
            labels("version = 1\n[[sections]]\nfolder = \"\"\n[[sections]]\nfolder = \"|"),
            ["Guides/", "Reference/"]
        );
        assert_eq!(
            labels("version = 1\n[[sections]]\nfolder = |"),
            ["", "Guides/", "Reference/"]
        );
    }

    #[test]
    fn pages_not_named_yet() {
        assert_eq!(
            labels(
                "version = 1\n[[sections]]\nfolder = \"Reference/\"\npages = [\"Options.md\", \"|"
            ),
            ["Defaults.md"]
        );
        assert_eq!(
            labels("version = 1\n[[sections]]\nfolder = \"Reference/\"\npages = [|]"),
            ["Options.md", "Defaults.md"]
        );
        assert_eq!(
            labels("version = 1\n[[sections]]\npages = [\n  \"|\n]\nfolder = \"Guides/\"\n"),
            ["Setup.md"]
        );
    }
}
