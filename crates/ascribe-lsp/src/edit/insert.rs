//! Blocks inserted on a blank line: notes, steps, variant groups, details,
//! includes, snippets, images, and widgets.
//!
//! The new lines replace the blank line the cursor is on, each indented to
//! the container that line is in (a list item's content, a block quote), and
//! a blank line is added above or below where the block would otherwise touch
//! text.

use ascribe_core::schema::Primary;
use ascribe_core::{
    Binding, Destination, RelPath, Span, TextEdit, TitleRule, classify_destination,
};
use ascribe_resolve::{CodeFiles, source_files};
use ascribe_syntax::{Block, BlockKind};

use super::blocks::{arm_placeholder, note_type_arg};
use super::{Outcome, Page, Placeholder, Plan, is_blank, line_end, line_start, title_line};
use crate::context::is_insertable;

/// A block's lines, and which of them holds placeholder text: its line, and
/// the placeholder's start and length in it.
struct Lines {
    lines: Vec<String>,
    placeholder: Option<(usize, usize, usize)>,
    /// The groupable directive the block is a group of, if it is one.
    group: Option<String>,
}

impl Lines {
    fn new() -> Lines {
        Lines {
            lines: Vec::new(),
            placeholder: None,
            group: None,
        }
    }

    fn push(&mut self, line: impl Into<String>) {
        self.lines.push(line.into());
    }

    /// Adds `before`, then placeholder text, as one line; the first
    /// placeholder is the one selected.
    fn push_placeholder(&mut self, before: &str, text: &str) {
        if self.placeholder.is_none() {
            self.placeholder = Some((self.lines.len(), before.len(), text.len()));
        }
        self.lines.push(format!("{before}{text}"));
    }
}

/// The plan that writes `lines` on the blank line the cursor is on.
fn insert(page: &Page<'_>, lines: Lines) -> Outcome {
    if page.start != page.end || !is_insertable(page.source, page.start, &page.found_outermost()) {
        return Err("Put the cursor on a blank line between blocks.".to_owned());
    }
    if let Some(name) = &lines.group
        && innermost_group(page).as_deref() == Some(name.as_str())
    {
        // A run of openers is one group: the new arms would join the arm's
        // own group.
        return Err(if name == "variant" {
            "A group of `@variant` arms can't go directly in an arm of another. Put both dimensions on one arm instead.".to_owned()
        } else {
            format!("A group of `@{name}` arms can't go directly in an arm of another.")
        });
    }
    let source = page.source;
    let start = line_start(source, page.start);
    let end = line_end(source, page.start);
    let prefix = container_prefix(page, &page.file.document.blocks, start).unwrap_or_default();
    let nl = page.nl;
    let blank = prefix.trim_end();
    let before = start > 0 && {
        let previous = line_start(source, start - 1);
        !is_blank(&source[previous..start])
    };
    let after = end < source.len() && {
        let next = end
            + if source[end..].starts_with("\r\n") {
                2
            } else {
                1
            };
        !is_blank(&source[next..line_end(source, next)])
    };
    let mut text = String::new();
    if before {
        text.push_str(blank);
        text.push_str(nl);
    }
    let mut select = None;
    for (i, line) in lines.lines.iter().enumerate() {
        if i > 0 {
            text.push_str(nl);
        }
        text.push_str(&prefix);
        if let Some((at, from, len)) = lines.placeholder
            && at == i
        {
            select = Some(Placeholder {
                edit: 0,
                from: text.len() + from,
                len,
            });
        }
        text.push_str(line);
    }
    if after {
        text.push_str(nl);
        text.push_str(blank);
    }
    Ok(Plan {
        edits: vec![TextEdit::replace(Span::new(start, end), text)],
        select,
        ..Plan::new(Vec::new())
    })
}

/// What a line at `offset` (a blank line's start) needs before its text to
/// be in the container it's in: the content of a list item, inside a block
/// quote. A blank line between two items of a list, or after its last item
/// and indented to its content, is in the item above it.
fn container_prefix(page: &Page<'_>, blocks: &[Block], offset: usize) -> Option<String> {
    let indent = page.source[offset..]
        .chars()
        .take_while(|c| *c == ' ')
        .count();
    for block in blocks {
        let inside = block.span.start() <= offset && offset <= block.span.end();
        match &block.kind {
            BlockKind::BlockQuote(q) if inside => {
                return container_prefix(page, &q.children, offset)
                    .or_else(|| q.children.first().map(|c| page.prefix(c.span.start())));
            }
            BlockKind::Container(c) if inside => {
                return container_prefix(page, &c.children, offset)
                    .or_else(|| Some(page.prefix(block.span.start())));
            }
            BlockKind::Group(g) if inside => {
                let arm = g.arms.iter().rev().find(|a| a.span.start() <= offset)?;
                return container_prefix(page, &arm.children, offset)
                    .or_else(|| Some(page.prefix(block.span.start())));
            }
            BlockKind::List(list) => {
                let trailing = offset > block.span.end()
                    && page.source[block.span.end()..offset].trim().is_empty();
                if !inside && !trailing {
                    continue;
                }
                let item = list.items.iter().rev().find(|i| i.span.start() <= offset)?;
                let content = item_prefix(page, item);
                if trailing && indent < content.len() {
                    continue;
                }
                return container_prefix(page, &item.children, offset).or(Some(content));
            }
            _ => {}
        }
    }
    None
}

/// The name of the group the cursor is directly in, when the innermost
/// container around it is a group.
fn innermost_group(page: &Page<'_>) -> Option<String> {
    use crate::context::{ContextNode, Form};
    page.found.iter().find_map(|(_, node)| match node {
        ContextNode::VariantGroup { .. } => Some(Some("variant".to_owned())),
        ContextNode::Widget {
            name,
            form: Form::Group,
            ..
        } => Some(Some(name.clone())),
        ContextNode::Note {
            form: Form::Container,
            ..
        }
        | ContextNode::Details {
            form: Form::Container,
            ..
        }
        | ContextNode::Widget {
            form: Form::Container,
            ..
        }
        | ContextNode::List { .. }
        | ContextNode::ListItem { .. }
        | ContextNode::BlockQuote { .. } => Some(None),
        _ => None,
    })?
}

/// The prefix of a list item's content.
fn item_prefix(page: &Page<'_>, item: &ascribe_syntax::ListItem) -> String {
    match item.children.first() {
        Some(first) => page.prefix(first.span.start()),
        None => format!("{} ", page.prefix(item.marker.end())),
    }
}

impl Page<'_> {
    /// What contains the start of the range, outermost first.
    fn found_outermost(&self) -> Vec<(Span, crate::context::ContextNode)> {
        self.found.iter().rev().cloned().collect()
    }
}

pub(crate) fn note(page: &Page<'_>) -> Outcome {
    let note_type = note_type_arg(page)?;
    let pairs = if note_type == "note" {
        Vec::new()
    } else {
        vec![("type".to_owned(), note_type)]
    };
    let head = match ascribe_fmt::directive_block("note", &pairs, &page.options(), &page.ctx.model)
    {
        Some(block) => format!("@note {block}: "),
        None => "@note: ".to_owned(),
    };
    let mut lines = Lines::new();
    match page.opt_str("text")? {
        Some(text) if !text.trim().is_empty() => {
            let mut rows = text.trim().lines();
            lines.push(format!("{head}{}", rows.next().unwrap_or_default()));
            for row in rows {
                lines.push(row.trim_start());
            }
        }
        _ => lines.push_placeholder(&head, "Write the note here."),
    }
    insert(page, lines)
}

pub(crate) fn steps(page: &Page<'_>) -> Outcome {
    let count = match page.args.get("count") {
        None | Some(serde_json::Value::Null) => 3,
        Some(value) => value
            .as_u64()
            .or_else(|| value.as_str().and_then(|s| s.trim().parse().ok()))
            .ok_or_else(|| "`count` must be a number.".to_owned())?,
    };
    if !(1..=50).contains(&count) {
        return Err("`count` must be from 1 to 50.".to_owned());
    }
    let mut lines = Lines::new();
    lines.push("@steps");
    for n in 1..=count {
        lines.push_placeholder(&format!("{n}. "), &format!("Step {n}."));
    }
    insert(page, lines)
}

pub(crate) fn variant_group(page: &Page<'_>) -> Outcome {
    let model = &page.ctx.model;
    let name = page.str_arg("dimension")?;
    let Some(dimension) = model.dimension(&name) else {
        let names: Vec<String> = model
            .dimensions
            .iter()
            .map(|d| format!("`{}`", d.name))
            .collect();
        return Err(if names.is_empty() {
            "The content model declares no dimensions.".to_owned()
        } else {
            format!(
                "`{name}` isn't a dimension. The dimensions are {}.",
                names.join(", ")
            )
        });
    };
    let values = page.strings_arg("values")?;
    if values.is_empty() {
        return Err("Choose at least one value.".to_owned());
    }
    for value in &values {
        if !dimension.values.iter().any(|v| v.value == *value) {
            let known: Vec<String> = dimension
                .values
                .iter()
                .map(|v| format!("`{}`", v.value))
                .collect();
            return Err(format!(
                "`{value}` isn't a value of `{name}`. Its values are {}.",
                known.join(", ")
            ));
        }
    }
    let options = page.options();
    let mut lines = Lines::new();
    lines.group = Some("variant".to_owned());
    // The arms in the order the model declares the values.
    for declared in dimension
        .values
        .iter()
        .filter(|v| values.contains(&v.value))
    {
        let pairs = vec![(name.clone(), declared.value.clone())];
        let block = ascribe_fmt::directive_block("variant", &pairs, &options, model)
            .ok_or_else(|| format!("`{}` can't be an attribute value.", declared.value))?;
        lines.push(format!("@variant {block}:"));
        lines.push_placeholder("", &arm_placeholder(&declared.label));
    }
    lines.push("@end");
    insert(page, lines)
}

pub(crate) fn details(page: &Page<'_>) -> Outcome {
    let title = page.line_arg("title")?;
    let mut lines = Lines::new();
    lines.push(title_line(&title));
    lines.push("@details:");
    lines.push_placeholder("", "Write the details here.");
    lines.push("@end");
    insert(page, lines)
}

pub(crate) fn include(page: &Page<'_>) -> Outcome {
    let path = page.line_arg("path")?;
    if path.contains(char::is_whitespace) {
        return Err("Write spaces in the path as `%20`.".to_owned());
    }
    let Destination::Local(local) = classify_destination(&path) else {
        return Err("Only a file of the project can be included.".to_owned());
    };
    let target = local
        .resolve(&page.ctx.path)
        .ok()
        .filter(|t| page.ctx.snapshot.file(t).is_some())
        .ok_or_else(|| format!("No page or fragment is at `{path}`."))?;
    if target == page.ctx.path || reaches(page, &target, &page.ctx.path) {
        return Err(format!(
            "Including `{path}` would make a cycle: it includes this page."
        ));
    }
    if let Some(id) = local.fragment.as_deref().filter(|id| !id.is_empty())
        && page
            .ctx
            .snapshot
            .file(&target)
            .and_then(|f| f.heading_by_id(id))
            .is_none()
    {
        return Err(format!(
            "`{}` has no heading with the id `{id}`.",
            local.path
        ));
    }
    let mut lines = Lines::new();
    lines.push(format!("@include: {path}"));
    insert(page, lines)
}

/// Whether `from` includes `to`, directly or through other files.
fn reaches(page: &Page<'_>, from: &RelPath, to: &RelPath) -> bool {
    let snapshot = &page.ctx.snapshot;
    let mut seen = std::collections::BTreeSet::new();
    let mut queue = vec![from.clone()];
    while let Some(path) = queue.pop() {
        let Some(file) = snapshot.file(&path) else {
            continue;
        };
        for target in file.includes.iter().filter_map(|i| i.target.clone()) {
            if target == *to {
                return true;
            }
            if seen.insert(target.clone()) {
                queue.push(target);
            }
        }
    }
    false
}

pub(crate) fn snippet(page: &Page<'_>) -> Outcome {
    let address = page.line_arg("address")?;
    let model = &page.ctx.model;
    let (source_name, rest) = address
        .split_once(':')
        .ok_or_else(|| "An address is `<source>:<path>`.".to_owned())?;
    let Some(source) = model.sources.iter().find(|s| s.name == source_name) else {
        let names: Vec<String> = model
            .sources
            .iter()
            .map(|s| format!("`{}`", s.name))
            .collect();
        return Err(if names.is_empty() {
            "The content model declares no sources.".to_owned()
        } else {
            format!(
                "`{source_name}` isn't a source. The sources are {}.",
                names.join(", ")
            )
        });
    };
    let (file, region) = match rest.split_once('#') {
        Some((file, region)) => (file, Some(region)),
        None => (rest, None),
    };
    let files = source_files(source, &*page.ctx.fs, &CodeFiles::new());
    let whole = format!("{source_name}:{file}");
    let Some(found) = files.iter().find(|f| f.address == whole) else {
        return Err(format!(
            "`{source_name}` has no file `{file}` a snippet can use."
        ));
    };
    if let Some(region) = region
        && !found.regions.iter().any(|r| r == region)
    {
        return Err(if found.regions.is_empty() {
            format!("`{file}` has no regions.")
        } else {
            format!(
                "`{file}` has no region `{region}`. Its regions are {}.",
                found
                    .regions
                    .iter()
                    .map(|r| format!("`{r}`"))
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        });
    }
    let mut pairs = Vec::new();
    for key in ["lang", "title"] {
        if let Some(value) = page.opt_str(key)?.filter(|v| !v.trim().is_empty()) {
            if value.contains(['\n', '\r']) {
                return Err(format!("`{key}` must be one line."));
            }
            pairs.push((key.to_owned(), value.trim().to_owned()));
        }
    }
    let head = match ascribe_fmt::directive_block("snippet", &pairs, &page.options(), model) {
        Some(block) => format!("@snippet {block}: "),
        None if pairs.is_empty() => "@snippet: ".to_owned(),
        None => return Err("The attributes can't be written.".to_owned()),
    };
    let mut lines = Lines::new();
    lines.push(format!("{head}{address}"));
    insert(page, lines)
}

/// An image's destination as a link writes it: in `<>` when it has
/// whitespace or parentheses.
pub(super) fn written_destination(path: &str) -> String {
    if path.contains(|c: char| c.is_whitespace() || c == '(' || c == ')') {
        format!("<{path}>")
    } else {
        path.to_owned()
    }
}

/// Text with the characters that would end it early, or start markup,
/// escaped: for link text and alt text.
pub(super) fn escaped_text(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for c in text.chars() {
        if matches!(c, '[' | ']' | '\\' | '`' | '*' | '_' | '<' | '{' | '}') {
            out.push('\\');
        }
        out.push(c);
    }
    out
}

/// The image attribute keys `pairs` uses that the model doesn't declare,
/// as an error naming the declared ones.
pub(super) fn check_image_keys(page: &Page<'_>, pairs: &[(String, String)]) -> Result<(), String> {
    let declared = &page.ctx.model.image_attributes;
    for (key, _) in pairs {
        if !declared.iter().any(|a| a.key == *key) {
            let keys: Vec<String> = declared.iter().map(|a| format!("`{}`", a.key)).collect();
            return Err(if keys.is_empty() {
                "The content model declares no image attributes.".to_owned()
            } else {
                format!(
                    "`{key}` isn't an image attribute. The image attributes are {}.",
                    keys.join(", ")
                )
            });
        }
    }
    Ok(())
}

pub(crate) fn image(page: &Page<'_>) -> Outcome {
    let path = page.line_arg("path")?;
    let alt = page.line_arg("alt")?;
    let pairs = page.attributes_arg("attributes")?;
    check_image_keys(page, &pairs)?;
    if matches!(classify_destination(&path), Destination::External) {
        return Err("An image is a file of the project, not a URL.".to_owned());
    }
    let block = ascribe_fmt::image_block(&pairs, &page.ctx.model);
    if block.is_none() && !pairs.is_empty() {
        return Err("The attributes can't be written.".to_owned());
    }
    let mut lines = Lines::new();
    lines.push(format!(
        "![{}]({}){}",
        escaped_text(&alt),
        written_destination(&path),
        block.unwrap_or_default()
    ));
    insert(page, lines)
}

pub(crate) fn widget(page: &Page<'_>) -> Outcome {
    let model = &page.ctx.model;
    let name = page.str_arg("name")?;
    let Some(widget) = model.widget(&name) else {
        let names: Vec<String> = model
            .widgets
            .iter()
            .map(|w| format!("`{}`", w.schema.name))
            .collect();
        return Err(if names.is_empty() {
            "The content model declares no widgets.".to_owned()
        } else {
            format!(
                "`{name}` isn't a widget. The widgets are {}.",
                names.join(", ")
            )
        });
    };
    let schema = &widget.schema;
    let pairs = page.attributes_arg("attributes")?;
    if let ascribe_core::Attributes::Declared(declared) = &schema.attributes {
        for (key, _) in &pairs {
            if !declared.iter().any(|a| a.key == *key) {
                let keys: Vec<String> = declared.iter().map(|a| format!("`{}`", a.key)).collect();
                return Err(if keys.is_empty() {
                    format!("`{name}` takes no attributes.")
                } else {
                    format!(
                        "`{name}` has no attribute `{key}`. Its attributes are {}.",
                        keys.join(", ")
                    )
                });
            }
        }
        if let Some(missing) = declared
            .iter()
            .find(|a| a.required && !pairs.iter().any(|(k, _)| *k == a.key))
        {
            return Err(format!("`{name}` needs `{}`.", missing.key));
        }
    }
    let primary = page.opt_str("primary")?.filter(|p| !p.trim().is_empty());
    let primary = primary.map(|p| p.trim().to_owned());
    match (schema.primary, &primary) {
        (Primary::None, Some(_)) => return Err(format!("`{name}` takes no primary.")),
        (Primary::Identifier { .. }, Some(p)) if p.contains(char::is_whitespace) => {
            return Err(format!("`{name}`'s primary is one word."));
        }
        (_, Some(p)) if p.contains(['\n', '\r']) => {
            return Err(format!("`{name}`'s primary must be one line."));
        }
        _ => {}
    }
    let block = ascribe_fmt::directive_block(&name, &pairs, &page.options(), model);
    if block.is_none() && !pairs.is_empty() {
        return Err("The attributes can't be written.".to_owned());
    }
    let head = match &block {
        Some(block) => format!("@{name} {block}"),
        None => format!("@{name}"),
    };
    let mut lines = Lines::new();
    if schema.groupable {
        lines.group = Some(name.clone());
    }
    if schema.title == TitleRule::Required {
        lines.push_placeholder(".", "Title");
    }
    let forms = schema.forms;
    let binding = schema.binding;
    let line_stands_alone = forms.line
        && (binding == Some(Binding::SelfBound)
            || (matches!(schema.primary, Primary::Text { .. }) && primary.is_some()));
    if line_stands_alone || (forms.line && !forms.container && primary.is_some()) {
        match &primary {
            Some(p) => lines.push(format!("{head}: {p}")),
            None if schema.primary.is_required() => {
                return Err(format!("`{name}` needs a primary."));
            }
            None => lines.push(head),
        }
    } else if forms.container {
        if primary.is_some() {
            return Err(format!(
                "`{name}` is written here as a container, which takes no primary."
            ));
        }
        lines.push(format!("{head}:"));
        lines.push_placeholder("", "Write the content here.");
        lines.push("@end");
    } else {
        if schema.primary.is_required() {
            return Err(format!("`{name}` needs a primary."));
        }
        lines.push(head);
        if matches!(binding, Some(Binding::Block | Binding::HeadingOrBlock)) {
            lines.push_placeholder("", "Write the content here.");
        }
    }
    insert(page, lines)
}
