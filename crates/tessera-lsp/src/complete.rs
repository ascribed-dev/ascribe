//! Completion (SPEC §10), from the cursor's context.
//!
//! The context comes from the text of the line up to the cursor: a file being
//! typed rarely parses into the construct the author is in the middle of, and
//! an unclosed `{` or `](` is exactly what the line holds. What each context
//! completes comes from the content model and the project index; the targets
//! of includes and links are read with `tessera_resolve::references`, the one
//! implementation of the reference rules.
//!
//! Long lists (every page and heading of a large project) are searched by what
//! was typed, ranked, and cut at [`LIMIT`], with the list marked incomplete so
//! the client asks again as the author types.

use lsp_types::{
    CompletionItem, CompletionItemKind, CompletionItemLabelDetails, CompletionList,
    CompletionResponse, CompletionTextEdit, Documentation, MarkupContent, MarkupKind, Position,
    Range, TextEdit,
};
use tessera_core::schema::{AttributeSchema, AttributeType, Attributes, Origin, SetMember};
use tessera_core::{LineIndex, RelPath, percent_decode};
use tessera_resolve::references::{include_target, reference_target};
use tessera_resolve::{FileIndex, RefKind, Target};
use tessera_syntax::{Block, BlockKind, CodeBlock, Inline, InlineKind};

use crate::hover::{attribute_line, describe_directive, type_name};
use crate::nav::{Ctx, encode_destination, line_prefix, relative_path};

/// The most items a search returns.
pub(crate) const LIMIT: usize = 100;

/// The completions at a position of the requested file.
pub(crate) fn complete(ctx: &Ctx, position: Position) -> Option<CompletionResponse> {
    let file = ctx.file()?;
    let index = LineIndex::new(&file.source);
    let offset = ctx.encoding.offset_lenient(&index, &file.source, position);
    let source: &str = &file.source;
    let prefix = line_prefix(source, offset);
    let cx = Cx {
        ctx,
        file,
        index: &index,
        offset,
        position,
    };

    if let Some(frontmatter) = &file.document.frontmatter
        && frontmatter.content.start() <= offset
        && offset <= frontmatter.content.end()
    {
        let value = prefix.strip_prefix("available:")?;
        let typed = value.trim_start().trim_start_matches(['"', '\'']);
        return Some(list(cx.availability(typed), false));
    }

    let code = code_block_at(&file.document.blocks, offset);
    if let Some(code) = code
        && code.phrases.is_none()
    {
        return None;
    }
    let rest = strip_container(prefix);
    if code.is_none() && rest.starts_with('@') {
        return cx.directive_line(&rest[1..]);
    }
    cx.prose(rest, code.is_none())
}

fn list(items: Vec<CompletionItem>, incomplete: bool) -> CompletionResponse {
    CompletionResponse::List(CompletionList {
        is_incomplete: incomplete,
        items,
    })
}

struct Cx<'a> {
    ctx: &'a Ctx,
    file: &'a FileIndex,
    index: &'a LineIndex,
    offset: usize,
    position: Position,
}

impl Cx<'_> {
    /// The range from `typed` bytes before the cursor to the cursor.
    fn back(&self, typed: usize) -> Range {
        let start = self
            .ctx
            .encoding
            .position(self.index, self.offset - typed)
            .unwrap_or(self.position);
        Range {
            start,
            end: self.position,
        }
    }

    fn item(
        &self,
        label: &str,
        kind: CompletionItemKind,
        typed: usize,
        insert: &str,
    ) -> CompletionItem {
        CompletionItem {
            label: label.to_owned(),
            kind: Some(kind),
            text_edit: Some(CompletionTextEdit::Edit(TextEdit {
                range: self.back(typed),
                new_text: insert.to_owned(),
            })),
            ..CompletionItem::default()
        }
    }

    // -- Directive lines ------------------------------------------------------

    /// The line is `@…`, and `after_at` is what follows the `@` up to the cursor.
    fn directive_line(&self, after_at: &str) -> Option<CompletionResponse> {
        let name_len = after_at
            .find(|c: char| !(c.is_ascii_alphanumeric() || c == '_' || c == '-'))
            .unwrap_or(after_at.len());
        if name_len == after_at.len() {
            return Some(list(self.directive_names(after_at), false));
        }
        let name = &after_at[..name_len];
        let tail = &after_at[name_len..];
        match scan_head(tail) {
            Head::Attributes(inner) => {
                let schemas = self.ctx.model.directive_schemas();
                let schema = schemas.iter().find(|s| s.name == name)?;
                let items = match &schema.attributes {
                    Attributes::Declared(keys) => self.attributes(Some(keys), inner),
                    Attributes::Dimensions => self.attributes(None, inner),
                };
                Some(list(items, false))
            }
            Head::Primary(primary) => match name {
                "include" => Some(self.include(primary)),
                "available" => Some(list(self.availability(primary), false)),
                _ => {
                    let schemas = self.ctx.model.directive_schemas();
                    let schema = schemas.iter().find(|s| s.name == name)?;
                    match schema.primary {
                        tessera_core::schema::Primary::Text { .. } => self.prose(primary, true),
                        _ => None,
                    }
                }
            },
            Head::None => None,
        }
    }

    fn directive_names(&self, typed: &str) -> Vec<CompletionItem> {
        let mut items = Vec::new();
        for (i, schema) in self.ctx.model.directive_schemas().iter().enumerate() {
            let widget = matches!(schema.origin, Origin::Widget);
            let mut item = self.item(
                &schema.name,
                if widget {
                    CompletionItemKind::CLASS
                } else {
                    CompletionItemKind::KEYWORD
                },
                typed.len(),
                &schema.name,
            );
            item.detail = Some(
                schema
                    .description
                    .clone()
                    .unwrap_or_else(|| "Project widget".to_owned()),
            );
            item.documentation = Some(markdown(describe_directive(schema)));
            item.sort_text = Some(format!("{:03}", i));
            items.push(item);
        }
        let mut end = self.item("end", CompletionItemKind::KEYWORD, typed.len(), "end");
        end.detail = Some("Close a container.".to_owned());
        end.sort_text = Some("999".to_owned());
        items.push(end);
        items
    }

    // -- Attribute blocks -----------------------------------------------------

    /// Keys or values in `{…}`. `keys` is a schema's declared attributes;
    /// `None` is `@variant`, whose keys are the model's dimensions.
    fn attributes(&self, keys: Option<&[AttributeSchema]>, inner: &str) -> Vec<CompletionItem> {
        let model = &self.ctx.model;
        let (used, current) = split_pairs(inner);
        let Some(current) = current else {
            return Vec::new();
        };
        let Some((key, value)) = current.split_once('=') else {
            // A key.
            let typed = current.trim_start();
            let mut items = Vec::new();
            match keys {
                Some(keys) => {
                    for (i, k) in keys.iter().enumerate() {
                        if used.iter().any(|u| u == &k.key) {
                            continue;
                        }
                        let mut item = self.item(
                            &k.key,
                            CompletionItemKind::PROPERTY,
                            typed.len(),
                            &format!("{}=", k.key),
                        );
                        item.detail = Some(type_name(&k.ty));
                        item.documentation = Some(markdown(attribute_line(k)));
                        item.sort_text = Some(format!("{i:03}"));
                        items.push(item);
                    }
                }
                None => {
                    for (i, d) in model.dimensions.iter().enumerate() {
                        if used.iter().any(|u| u == &d.name) {
                            continue;
                        }
                        let mut item = self.item(
                            &d.name,
                            CompletionItemKind::PROPERTY,
                            typed.len(),
                            &format!("{}=", d.name),
                        );
                        item.detail = Some(d.label.clone());
                        item.sort_text = Some(format!("{i:03}"));
                        items.push(item);
                    }
                }
            }
            return items;
        };
        // A value, or a member of a value set.
        let key = key.trim();
        let value = value.trim_start();
        if value.matches('"').count() % 2 == 1 {
            return Vec::new();
        }
        let (chosen, typed) = match value.rsplit_once('|') {
            Some((before, after)) => (
                before.split('|').map(str::trim).collect::<Vec<_>>(),
                after.trim_start(),
            ),
            None => (Vec::new(), value),
        };
        let is_set = value.contains('|');
        let choices: Vec<(String, Option<String>)> = match keys {
            None => model.dimension_values(key).map_or(Vec::new(), |values| {
                values
                    .iter()
                    .map(|v| (v.value.clone(), Some(v.label.clone())))
                    .collect()
            }),
            Some(keys) => match keys.iter().find(|k| k.key == key).map(|k| &k.ty) {
                Some(AttributeType::Boolean) if !is_set => {
                    vec![("true".to_owned(), None), ("false".to_owned(), None)]
                }
                Some(AttributeType::Enum(values)) if !is_set => {
                    values.iter().map(|v| (v.clone(), None)).collect()
                }
                Some(AttributeType::Set(SetMember::Enum(values))) => {
                    values.iter().map(|v| (v.clone(), None)).collect()
                }
                Some(AttributeType::NoteType) if !is_set => model
                    .notes
                    .iter()
                    .map(|n| (n.name.clone(), Some(n.label.clone())))
                    .collect(),
                _ => Vec::new(),
            },
        };
        choices
            .into_iter()
            .enumerate()
            .filter(|(_, (value, _))| !chosen.contains(&value.as_str()))
            .map(|(i, (value, label))| {
                let mut item =
                    self.item(&value, CompletionItemKind::ENUM_MEMBER, typed.len(), &value);
                item.detail = label;
                item.sort_text = Some(format!("{i:03}"));
                item
            })
            .collect()
    }

    // -- Availability ---------------------------------------------------------

    /// Targets, dimension names, lifecycle states, and feature keys, by where
    /// in the spec the cursor is. `typed` is the spec so far.
    fn availability(&self, typed: &str) -> Vec<CompletionItem> {
        let model = &self.ctx.model;
        let depth = typed.matches('(').count() as i64 - typed.matches(')').count() as i64;
        let entry_start = if depth > 0 {
            // A history: the states inside the parentheses.
            typed.rfind(['(', ',']).map_or(0, |i| i + 1)
        } else {
            typed.rfind(',').map_or(0, |i| i + 1)
        };
        let entry = &typed[entry_start..];
        let ends_blank = entry.ends_with(char::is_whitespace);
        let words: Vec<&str> = entry.split_whitespace().collect();
        let (complete, word) = if ends_blank || words.is_empty() {
            (words.len(), "")
        } else {
            (words.len() - 1, words[words.len() - 1])
        };
        let mut items = Vec::new();
        let states = |items: &mut Vec<CompletionItem>| {
            for (i, s) in model.lifecycle.iter().enumerate() {
                let mut item = self.item(
                    &s.name,
                    CompletionItemKind::ENUM_MEMBER,
                    word.len(),
                    &s.name,
                );
                item.detail = Some(s.label.clone());
                item.sort_text = Some(format!("{i:03}"));
                items.push(item);
            }
        };
        if depth > 0 {
            if complete == 0 {
                states(&mut items);
            }
            return items;
        }
        match complete {
            0 => {
                let first_entry = !typed.contains(',');
                for d in &model.dimensions {
                    let mut item =
                        self.item(&d.name, CompletionItemKind::MODULE, word.len(), &d.name);
                    item.detail = Some(format!("{}: every value", d.label));
                    item.sort_text = Some(format!("1{}", d.name));
                    items.push(item);
                    for v in &d.values {
                        let mut item = self.item(
                            &v.value,
                            CompletionItemKind::ENUM_MEMBER,
                            word.len(),
                            &v.value,
                        );
                        item.detail = Some(if v.versionless {
                            v.label.clone()
                        } else {
                            format!("{} (versioned)", v.label)
                        });
                        item.sort_text = Some(format!("0{}", v.value));
                        items.push(item);
                    }
                }
                if first_entry {
                    for f in &model.features {
                        let mut item =
                            self.item(&f.key, CompletionItemKind::CONSTANT, word.len(), &f.key);
                        item.detail = Some(format!("Feature: {}", f.name));
                        item.documentation = Some(markdown(format!(
                            "Available: {}",
                            tessera_emit::labels::availability_display(model, &f.available)
                        )));
                        item.sort_text = Some(format!("2{}", f.key));
                        items.push(item);
                    }
                }
            }
            1 if !word.starts_with(|c: char| c.is_ascii_digit())
                && words.first().is_some_and(|t| model.is_target(t)) =>
            {
                states(&mut items);
            }
            _ => {}
        }
        items
    }

    // -- Phrases, links --------------------------------------------------------

    /// Prose: `{` starts a phrase (or an image's attribute block), `](` a link
    /// destination. `text` is the line up to the cursor.
    fn prose(&self, text: &str, allow_links: bool) -> Option<CompletionResponse> {
        if code_span_at(&self.file.document.blocks, self.offset) || code_span_open(text) {
            return None;
        }
        let brace = text.rfind('{').filter(|&i| !text[i..].contains('}'));
        let link = if allow_links {
            link_destination(text)
        } else {
            None
        };
        match (brace, link) {
            (Some(b), Some((l, _))) if b > l => Some(self.brace(text, b)?),
            (Some(b), None) => Some(self.brace(text, b)?),
            (_, Some((_, dest))) => Some(self.link(dest)),
            (None, None) => None,
        }
    }

    fn brace(&self, text: &str, at: usize) -> Option<CompletionResponse> {
        let inner = &text[at + 1..];
        if is_image_before(&text[..at]) {
            return Some(list(
                self.attributes(Some(&self.ctx.model.image_attributes), inner),
                false,
            ));
        }
        if !inner
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
        {
            return None;
        }
        let close = !self.file.source[self.offset..].starts_with('}');
        let items = self
            .ctx
            .model
            .phrases
            .iter()
            .enumerate()
            .map(|(i, p)| {
                let insert = if close {
                    format!("{}}}", p.key)
                } else {
                    p.key.clone()
                };
                let mut item =
                    self.item(&p.key, CompletionItemKind::CONSTANT, inner.len(), &insert);
                item.detail = Some(p.value.clone());
                item.documentation = Some(markdown(format!("**{{{}}}**\n\n{}", p.key, p.value)));
                item.sort_text = Some(format!("{i:03}"));
                item
            })
            .collect();
        Some(list(items, false))
    }

    /// A link destination typed so far (`dest` is what follows the `(`).
    fn link(&self, dest: &str) -> CompletionResponse {
        let angle = dest.starts_with('<');
        let raw = dest.strip_prefix('<').unwrap_or(dest);
        if raw.contains('"') {
            return list(Vec::new(), false);
        }
        if let Some((path_part, id)) = raw.split_once('#') {
            let target = match reference_target(
                RefKind::Link,
                path_part,
                &[],
                &self.ctx.path,
                &self.ctx.model,
            ) {
                Target::Local(local) => local.path,
                Target::External => None,
            };
            let (items, incomplete) = self.headings_of(target.as_ref(), id, false, angle);
            return list(items, incomplete);
        }
        self.search(&percent_decode(raw), raw.len(), angle)
    }

    /// The headings of a file, by the id being typed: for `file.md#…` and
    /// `@include: file.md#…`. A link can't name a fragment's ids, except the
    /// fragment's own.
    fn headings_of(
        &self,
        target: Option<&RelPath>,
        typed: &str,
        include: bool,
        raw: bool,
    ) -> (Vec<CompletionItem>, bool) {
        let Some(target) = target else {
            return (Vec::new(), false);
        };
        let Some(file) = self.ctx.snapshot.file(target) else {
            return (Vec::new(), false);
        };
        if !include && *target != self.ctx.path && self.ctx.model.is_fragment(target.as_str()) {
            return (Vec::new(), false);
        }
        let wanted = percent_decode(typed).to_lowercase();
        let mut matches: Vec<_> = file
            .headings
            .iter()
            .filter(|h| !h.source_id.is_empty())
            .filter(|h| {
                wanted.is_empty()
                    || h.source_id.to_lowercase().contains(&wanted)
                    || h.text.to_lowercase().contains(&wanted)
            })
            .take(LIMIT + 1)
            .collect();
        let incomplete = matches.len() > LIMIT;
        matches.truncate(LIMIT);
        let items = matches
            .into_iter()
            .enumerate()
            .map(|(i, h)| {
                let insert = if raw {
                    h.source_id.clone()
                } else {
                    encode_destination(&h.source_id)
                };
                let mut item =
                    self.item(&h.text, CompletionItemKind::REFERENCE, typed.len(), &insert);
                item.detail = Some(format!("#{}", h.source_id));
                item.filter_text = Some(format!("{} {}", h.text, h.source_id));
                item.sort_text = Some(format!("{i:03}"));
                item
            })
            .collect();
        (items, incomplete)
    }

    // SPEC-QUESTION(Q161)
    /// Pages and headings whose title matches what was typed.
    fn search(&self, typed: &str, typed_len: usize, raw: bool) -> CompletionResponse {
        let wanted = typed.trim_start_matches("./").to_lowercase();
        let mut found: Vec<Match<'_>> = Vec::new();
        for file in self.ctx.snapshot.pages() {
            let own = file.path == self.ctx.path;
            let title = file
                .title
                .as_deref()
                .unwrap_or_else(|| file.path.file_name().unwrap_or_default());
            if !own {
                let score = if wanted.is_empty() {
                    Some(0)
                } else {
                    score(&wanted, title).or_else(|| {
                        file.path
                            .as_str()
                            .to_lowercase()
                            .contains(&wanted)
                            .then_some(3)
                    })
                };
                if let Some(score) = score {
                    found.push(Match {
                        score: score * 2,
                        file,
                        heading: None,
                        title,
                    });
                }
            }
            if wanted.is_empty() && !own {
                continue;
            }
            for (i, h) in file.headings.iter().enumerate() {
                if h.source_id.is_empty() {
                    continue;
                }
                let s = if wanted.is_empty() {
                    Some(0)
                } else {
                    score(&wanted, &h.text)
                };
                if let Some(s) = s {
                    found.push(Match {
                        score: s * 2 + 1,
                        file,
                        heading: Some(i),
                        title,
                    });
                }
            }
        }
        found.sort_by(|a, b| {
            (a.score, a.title, &a.file.path, a.heading).cmp(&(
                b.score,
                b.title,
                &b.file.path,
                b.heading,
            ))
        });
        let truncated = found.len() > LIMIT;
        found.truncate(LIMIT);
        let items = found
            .iter()
            .enumerate()
            .map(|(rank, m)| self.match_item(m, rank, typed_len, raw))
            .collect();
        list(items, truncated)
    }

    fn match_item(
        &self,
        m: &Match<'_>,
        rank: usize,
        typed_len: usize,
        raw: bool,
    ) -> CompletionItem {
        let own = m.file.path == self.ctx.path;
        let (label, id) = match m.heading {
            Some(i) => (
                m.file.headings[i].text.as_str(),
                Some(m.file.headings[i].source_id.as_str()),
            ),
            None => (m.title, None),
        };
        let path = if own {
            String::new()
        } else {
            relative_path(&m.file.path, &self.ctx.path)
        };
        let mut insert = if raw { path } else { encode_destination(&path) };
        let mut shown = m.file.path.to_string();
        if let Some(id) = id {
            insert.push('#');
            insert.push_str(&if raw {
                id.to_owned()
            } else {
                encode_destination(id)
            });
            shown.push('#');
            shown.push_str(id);
        }
        let mut item = self.item(
            label,
            if id.is_some() {
                CompletionItemKind::REFERENCE
            } else {
                CompletionItemKind::FILE
            },
            typed_len,
            &insert,
        );
        item.detail = Some(shown.clone());
        if id.is_some() && !own {
            item.label_details = Some(CompletionItemLabelDetails {
                detail: None,
                description: Some(m.title.to_owned()),
            });
        }
        item.filter_text = Some(format!("{label} {shown}"));
        item.sort_text = Some(format!("{rank:04}"));
        item
    }

    // -- Includes --------------------------------------------------------------

    // SPEC-QUESTION(Q161)
    /// The primary of an `@include`: files, then, after `#`, ids.
    fn include(&self, typed: &str) -> CompletionResponse {
        if let Some((path_part, id)) = typed.split_once('#') {
            let target = include_target(path_part, &self.ctx.path).target;
            let (items, incomplete) = self.headings_of(target.as_ref(), id, true, false);
            return list(items, incomplete);
        }
        let wanted = percent_decode(typed).to_lowercase();
        let root = wanted.starts_with('/');
        let mut found: Vec<(u8, String, &FileIndex)> = Vec::new();
        for file in self.ctx.snapshot.files() {
            if file.path == self.ctx.path {
                continue;
            }
            let written = if root {
                format!("/{}", file.path)
            } else {
                relative_path(&file.path, &self.ctx.path)
            };
            let lower = written.to_lowercase();
            let score = if wanted.is_empty() || lower.starts_with(&wanted) {
                0
            } else if lower.contains(&wanted) {
                1
            } else {
                continue;
            };
            found.push((score, written, file));
        }
        found.sort_by(|a, b| (a.0, &a.1).cmp(&(b.0, &b.1)));
        let truncated = found.len() > LIMIT;
        found.truncate(LIMIT);
        let items = found
            .iter()
            .enumerate()
            .map(|(rank, (_, written, file))| {
                let mut item = self.item(
                    written,
                    CompletionItemKind::FILE,
                    typed.len(),
                    &encode_destination(written),
                );
                item.detail = Some(
                    match (&file.title, self.ctx.model.is_fragment(file.path.as_str())) {
                        (Some(title), _) => title.clone(),
                        (None, true) => "Fragment".to_owned(),
                        (None, false) => "Page".to_owned(),
                    },
                );
                item.sort_text = Some(format!("{rank:04}"));
                item
            })
            .collect();
        list(items, truncated)
    }
}

/// A page or heading that matched a search.
struct Match<'a> {
    score: u8,
    file: &'a FileIndex,
    heading: Option<usize>,
    /// The page's title.
    title: &'a str,
}

/// How well `haystack` matches the lowercase `wanted`: 0 starts with it, 1 has
/// a word that does, 2 contains it. `None` when it doesn't.
fn score(wanted: &str, haystack: &str) -> Option<u8> {
    let lower = haystack.to_lowercase();
    if lower.starts_with(wanted) {
        Some(0)
    } else if lower
        .split(|c: char| !c.is_alphanumeric())
        .any(|w| w.starts_with(wanted))
    {
        Some(1)
    } else {
        lower.contains(wanted).then_some(2)
    }
}

fn markdown(text: String) -> Documentation {
    Documentation::MarkupContent(MarkupContent {
        kind: MarkupKind::Markdown,
        value: text,
    })
}

// -- Reading the line --------------------------------------------------------

/// What follows a directive's name on its line, up to the cursor.
enum Head<'a> {
    /// An unclosed attribute block: what is inside it.
    Attributes(&'a str),
    /// After the colon: the primary so far.
    Primary(&'a str),
    /// Anything else.
    None,
}

fn scan_head(tail: &str) -> Head<'_> {
    let mut open: Option<usize> = None;
    let mut quoted = false;
    let mut escaped = false;
    for (i, c) in tail.char_indices() {
        if quoted {
            match c {
                _ if escaped => escaped = false,
                '\\' => escaped = true,
                '"' => quoted = false,
                _ => {}
            }
            continue;
        }
        match c {
            '"' if open.is_some() => quoted = true,
            '{' if open.is_none() => open = Some(i),
            '}' if open.is_some() => open = None,
            ':' if open.is_none() => return Head::Primary(tail[i + 1..].trim_start()),
            _ => {}
        }
    }
    match open {
        Some(i) => Head::Attributes(&tail[i + 1..]),
        None => Head::None,
    }
}

/// Splits an attribute block's text into the keys of the finished pairs and
/// the pair being typed (`None` inside a quoted value).
fn split_pairs(inner: &str) -> (Vec<String>, Option<&str>) {
    let mut used = Vec::new();
    let mut start = 0;
    let mut quoted = false;
    let mut escaped = false;
    for (i, c) in inner.char_indices() {
        if quoted {
            match c {
                _ if escaped => escaped = false,
                '\\' => escaped = true,
                '"' => quoted = false,
                _ => {}
            }
            continue;
        }
        match c {
            '"' => quoted = true,
            ',' => {
                let pair = &inner[start..i];
                used.push(pair.split('=').next().unwrap_or_default().trim().to_owned());
                start = i + 1;
            }
            _ => {}
        }
    }
    (used, (!quoted).then(|| &inner[start..]))
}

/// The line without the block quote and list markers before its content.
fn strip_container(prefix: &str) -> &str {
    let mut rest = prefix;
    loop {
        let trimmed = rest.trim_start();
        if let Some(after) = trimmed.strip_prefix('>') {
            rest = after;
            continue;
        }
        let bullet = trimmed
            .strip_prefix(['-', '*', '+'])
            .filter(|after| after.starts_with([' ', '\t']));
        let ordered = || {
            let digits = trimmed.chars().take_while(char::is_ascii_digit).count();
            let after = &trimmed[digits..];
            (digits > 0 && digits <= 9)
                .then(|| after.strip_prefix(['.', ')']))
                .flatten()
                .filter(|after| after.starts_with([' ', '\t']))
        };
        match bullet.or_else(ordered) {
            Some(after) => rest = after,
            None => return trimmed,
        }
    }
}

/// The `](` or `]:` of a link destination being typed: where it starts, and the
/// text after it. `None` inside an image (whose source isn't a page) or when
/// the destination is closed.
fn link_destination(text: &str) -> Option<(usize, &str)> {
    // A link reference definition: `[label]: dest`.
    if let Some(after) = text.strip_prefix('[')
        && let Some(close) = after.find("]:")
    {
        let dest = after[close + 2..].trim_start();
        return Some((0, dest));
    }
    let at = text.rfind("](")?;
    let dest = &text[at + 2..];
    if dest.contains(')') {
        return None;
    }
    // The `[` that matches the `]`, and a `!` before it.
    let mut depth = 0i32;
    for (i, c) in text[..at].char_indices().rev() {
        match c {
            ']' => depth += 1,
            '[' if depth == 0 => {
                return (!text[..i].ends_with('!')).then_some((at, dest));
            }
            '[' => depth -= 1,
            _ => {}
        }
    }
    Some((at, dest))
}

/// Whether the text ends in an image, `![alt](src)`: what an attribute block
/// after it belongs to.
fn is_image_before(text: &str) -> bool {
    let Some(inner) = text.strip_suffix(')') else {
        return false;
    };
    let Some(open) = inner.rfind("](") else {
        return false;
    };
    let mut depth = 0i32;
    for (i, c) in inner[..open].char_indices().rev() {
        match c {
            ']' => depth += 1,
            '[' if depth == 0 => return inner[..i].ends_with('!'),
            '[' => depth -= 1,
            _ => {}
        }
    }
    false
}

fn code_block_at(blocks: &[Block], offset: usize) -> Option<&CodeBlock> {
    for block in blocks {
        if !(block.span.start() < offset && offset <= block.span.end()) {
            continue;
        }
        match &block.kind {
            BlockKind::CodeBlock(code) => return Some(code),
            BlockKind::BlockQuote(q) => {
                if let Some(c) = code_block_at(&q.children, offset) {
                    return Some(c);
                }
            }
            BlockKind::List(l) => {
                for item in &l.items {
                    if let Some(c) = code_block_at(&item.children, offset) {
                        return Some(c);
                    }
                }
            }
            BlockKind::Container(c) => {
                if let Some(c) = code_block_at(&c.children, offset) {
                    return Some(c);
                }
            }
            BlockKind::Group(g) => {
                for arm in &g.arms {
                    if let Some(c) = code_block_at(&arm.children, offset) {
                        return Some(c);
                    }
                }
            }
            _ => {}
        }
    }
    None
}

fn code_span_at(blocks: &[Block], offset: usize) -> bool {
    blocks.iter().any(|block| {
        if !(block.span.start() < offset && offset < block.span.end()) {
            return false;
        }
        match &block.kind {
            BlockKind::Heading(heading) => code_inline_at(&heading.inlines, offset),
            BlockKind::Paragraph(paragraph) => code_inline_at(&paragraph.inlines, offset),
            BlockKind::BlockQuote(quote) => code_span_at(&quote.children, offset),
            BlockKind::List(list) => list
                .items
                .iter()
                .any(|item| code_span_at(&item.children, offset)),
            BlockKind::Container(container) => code_span_at(&container.children, offset),
            BlockKind::Group(group) => group
                .arms
                .iter()
                .any(|arm| code_span_at(&arm.children, offset)),
            _ => false,
        }
    })
}

fn code_inline_at(inlines: &[Inline], offset: usize) -> bool {
    inlines.iter().any(|inline| match &inline.kind {
        InlineKind::Code(_) => inline.span.start() < offset && offset < inline.span.end(),
        InlineKind::Emphasis(children) | InlineKind::Strong(children) => {
            code_inline_at(children, offset)
        }
        InlineKind::Link(link) => code_inline_at(&link.children, offset),
        InlineKind::Image(image) => code_inline_at(&image.children, offset),
        _ => false,
    })
}

fn code_span_open(text: &str) -> bool {
    let bytes = text.as_bytes();
    let mut index = 0;
    let mut open_length = None;
    while index < bytes.len() {
        if bytes[index] != b'`' {
            index += 1;
            continue;
        }
        let run_start = index;
        while bytes.get(index) == Some(&b'`') {
            index += 1;
        }
        let run_length = index - run_start;
        let escaped = bytes[..run_start]
            .iter()
            .rev()
            .take_while(|byte| **byte == b'\\')
            .count()
            % 2
            == 1;
        if !escaped {
            match open_length {
                None => open_length = Some(run_length),
                Some(length) if length == run_length => open_length = None,
                Some(_) => {}
            }
        }
    }
    open_length.is_some()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn container_markers_are_stripped() {
        assert_eq!(strip_container("@note"), "@note");
        assert_eq!(strip_container("  > @no"), "@no");
        assert_eq!(strip_container("- @ava"), "@ava");
        assert_eq!(strip_container("1. > @ava"), "@ava");
        assert_eq!(strip_container("-text"), "-text");
    }

    #[test]
    fn a_directive_head_is_read() {
        assert!(matches!(scan_head(" {ty"), Head::Attributes("ty")));
        assert!(matches!(
            scan_head(" {a=\"x}\", b"),
            Head::Attributes("a=\"x}\", b")
        ));
        assert!(matches!(scan_head(" {a=b}: pri"), Head::Primary("pri")));
        assert!(matches!(scan_head(": path.md"), Head::Primary("path.md")));
        assert!(matches!(scan_head(" {a=b} "), Head::None));
    }

    #[test]
    fn pairs_are_split() {
        assert_eq!(split_pairs("a=b, c"), (vec!["a".to_owned()], Some(" c")));
        assert_eq!(split_pairs("a=\"x, y"), (vec![], None));
        assert_eq!(split_pairs(""), (vec![], Some("")));
    }

    #[test]
    fn link_destinations_are_found() {
        assert_eq!(link_destination("See [x](ke"), Some((6, "ke")));
        assert_eq!(link_destination("See [x](k.md) and [y]("), Some((20, "")));
        assert_eq!(link_destination("See [x](k.md) and"), None);
        assert_eq!(link_destination("![alt](im"), None);
        assert_eq!(link_destination("[ref]: ke"), Some((0, "ke")));
        assert_eq!(link_destination("[a [b]](c"), Some((6, "c")));
    }

    #[test]
    fn images_are_recognized_before_an_attribute_block() {
        assert!(is_image_before("![alt](a.png)"));
        assert!(!is_image_before("[alt](a.png)"));
        assert!(!is_image_before("text"));
    }

    #[test]
    fn scores_rank_prefix_then_word_then_contains() {
        assert_eq!(score("rot", "Rotate keys"), Some(0));
        assert_eq!(score("keys", "Rotate keys"), Some(1));
        assert_eq!(score("ate", "Rotate keys"), Some(2));
        assert_eq!(score("zzz", "Rotate keys"), None);
    }
}
