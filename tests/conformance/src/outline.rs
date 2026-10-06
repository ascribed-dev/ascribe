//! The `outline`: an abstract, implementation-independent description of a
//! parsed document.
//!
//! Test authors write outlines by hand in `expect.yaml`; adapters produce them
//! from the real parser. [`compare`] checks an actual outline against an
//! expected one. The YAML form is documented in `tests/conformance/README.md`.
//!
//! Some fields are *structural* and always compared, with a default when the
//! expectation omits them (`form`, `attributes`, `primary`, `title`,
//! `children`, a list's kind). Others are *content* fields, compared only
//! when the expectation writes them (`text`, `binding`, `info`, `start`,
//! `alt`, `fenced`). Adapters should always fill in every field they can.

use std::collections::BTreeMap;
use std::fmt;

use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_yaml_ng::{Mapping, Value};

/// A sequence of blocks: a document, or the children of a container.
pub type Outline = Vec<Node>;

/// Parsed attributes: keys to values, in key order.
pub type Attributes = BTreeMap<String, AttrValue>;

/// One block in an outline.
#[derive(Debug, Clone, PartialEq)]
pub enum Node {
    /// An ATX or setext heading.
    Heading {
        /// 1 to 6.
        level: u8,
        /// The heading's inline source text. Content field.
        text: Option<String>,
    },
    /// A paragraph.
    Paragraph {
        /// The paragraph's inline source text. Content field.
        text: Option<String>,
    },
    /// A paragraph that consists of a single image, with its attribute block if any.
    Image {
        /// The image source, as written.
        src: String,
        /// The alt text. Content field.
        alt: Option<String>,
        /// The image title. Structural: `None` means the image has none.
        title: Option<String>,
        /// The attribute block after the image.
        attributes: Attributes,
    },
    /// A fenced or indented code block.
    Code {
        /// The code block's content. Content field.
        text: Option<String>,
        /// The fence's info string. Content field.
        info: Option<String>,
        /// Whether the block is fenced (as opposed to indented). Content field.
        fenced: Option<bool>,
    },
    /// A block quote.
    Blockquote {
        /// The blocks inside it.
        children: Outline,
    },
    /// A list. Its children are [`Node::Item`]s.
    List {
        /// Ordered or bullet.
        ordered: bool,
        /// An ordered list's start number. Content field.
        start: Option<u64>,
        /// The items.
        children: Outline,
    },
    /// A list item.
    Item {
        /// The blocks inside it.
        children: Outline,
    },
    /// A thematic break.
    ThematicBreak,
    /// A raw HTML block.
    Html {
        /// The block's source text. Content field.
        text: Option<String>,
    },
    /// A table, where the parser supports them. Content field: its rows,
    /// each described by its first cell and that cell's attribute block.
    Table {
        /// The rows, the header row first. Content field.
        rows: Option<Vec<Row>>,
    },
    /// A directive in line or container form, other than an arm of a group.
    Directive(Directive),
    /// A group of arms of one groupable directive (SPEC §3.6).
    Group(Group),
}

/// A table row, described by its first cell (SPEC §4.4).
#[derive(Debug, Clone, PartialEq)]
pub struct Row {
    /// The first cell's inline source text, without its attribute block.
    pub text: String,
    /// The attribute block at the end of the first cell.
    pub attributes: Attributes,
}

/// A directive (SPEC §3).
#[derive(Debug, Clone, PartialEq)]
pub struct Directive {
    /// The keyword, without `@`.
    pub name: String,
    /// Line or container form.
    pub form: Form,
    /// The directive's attributes.
    pub attributes: Attributes,
    /// The primary, with whitespace normalized. `None` when there is none.
    pub primary: Option<String>,
    /// The title from a title line, with whitespace normalized.
    pub title: Option<String>,
    /// What a line-form directive applies to. Content field.
    pub binding: Option<Binding>,
    /// A container's blocks. Always empty in line form.
    pub children: Outline,
}

/// A directive's form (SPEC §3.5).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Form {
    /// A single line.
    Line,
    /// Opened by a trailing colon and closed by `@end`.
    Container,
}

/// What a line-form directive applies to (SPEC §3.8).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Binding {
    /// Its own primary, or nothing: `self`.
    SelfBinding,
    /// The heading at the start of its section: `heading`.
    Heading,
    /// The next block in the same children list: `following-block`.
    FollowingBlock,
    /// Nothing, because binding failed (an error case): `none`.
    Unbound,
}

/// A group of arms (SPEC §3.6).
#[derive(Debug, Clone, PartialEq)]
pub struct Group {
    /// The groupable directive's keyword, for example `variant`.
    pub name: String,
    /// The arms, in source order.
    pub arms: Vec<Arm>,
}

/// One arm of a group.
#[derive(Debug, Clone, PartialEq)]
pub struct Arm {
    /// The opener's attributes.
    pub attributes: Attributes,
    /// The opener's title, with whitespace normalized.
    pub title: Option<String>,
    /// The blocks in the arm.
    pub children: Outline,
}

/// An attribute value. Tokens and quoted strings are both [`AttrValue::Single`];
/// the outline describes meaning, not spelling.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AttrValue {
    /// A token or quoted string, unescaped.
    Single(String),
    /// A value set (`a|b`), in source order.
    Set(Vec<String>),
}

/// A problem in a hand-written outline.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("{path}: {message}")]
pub struct OutlineError {
    /// Where the problem is, for example `outline[2].children[0]`.
    pub path: String,
    /// What's wrong.
    pub message: String,
}

fn err<T>(path: &str, message: impl Into<String>) -> Result<T, OutlineError> {
    Err(OutlineError {
        path: path.to_owned(),
        message: message.into(),
    })
}

/// Collapses runs of whitespace to one space and trims the ends. Text, title,
/// and primary fields are compared after this.
pub fn normalize_ws(s: &str) -> String {
    s.split_whitespace().collect::<Vec<_>>().join(" ")
}

// ---------------------------------------------------------------------------
// YAML to outline

const KINDS: &[&str] = &[
    "heading",
    "paragraph",
    "image",
    "code",
    "blockquote",
    "list",
    "item",
    "thematic-break",
    "html",
    "table",
    "directive",
    "group",
];

/// Parses an outline from its YAML form.
pub fn outline_from_value(value: &Value, path: &str) -> Result<Outline, OutlineError> {
    match value {
        Value::Null => Ok(Vec::new()),
        Value::Sequence(items) => items
            .iter()
            .enumerate()
            .map(|(i, v)| node_from_value(v, &format!("{path}[{i}]")))
            .collect(),
        _ => err(path, "expected a list of blocks"),
    }
}

fn node_from_value(value: &Value, path: &str) -> Result<Node, OutlineError> {
    let (kind, main, fields) = match value {
        Value::String(kind) => (kind.as_str(), &Value::Null, Mapping::new()),
        Value::Mapping(map) => {
            let kinds: Vec<&str> = map
                .keys()
                .filter_map(Value::as_str)
                .filter(|k| KINDS.contains(k))
                .collect();
            let kind = match kinds.as_slice() {
                [kind] => *kind,
                [] => return err(path, format!("no block kind; expected one of {KINDS:?}")),
                _ => return err(path, format!("more than one block kind: {kinds:?}")),
            };
            let main = map.get(kind).unwrap_or(&Value::Null);
            let mut fields = map.clone();
            fields.remove(kind);
            (kind, main, fields)
        }
        _ => return err(path, "expected a block kind or a mapping"),
    };
    let mut f = Fields::new(fields, path)?;
    let node = match kind {
        "heading" => Node::Heading {
            level: match main {
                Value::Number(n) => match n.as_u64() {
                    Some(l @ 1..=6) => l as u8,
                    _ => return err(path, "heading level must be 1 to 6"),
                },
                _ => return err(path, "write `heading: <level>`, for example `heading: 2`"),
            },
            text: f.opt_string("text")?,
        },
        "paragraph" => Node::Paragraph {
            text: opt_scalar_string(main, path)?,
        },
        "image" => Node::Image {
            src: match opt_scalar_string(main, path)? {
                Some(src) => src,
                None => return err(path, "write `image: <src>`"),
            },
            alt: f.opt_string("alt")?,
            title: f.opt_string("title")?,
            attributes: f.attributes()?,
        },
        "code" => Node::Code {
            text: opt_scalar_string(main, path)?,
            info: f.opt_string("info")?,
            fenced: f.opt_bool("fenced")?,
        },
        "blockquote" => Node::Blockquote {
            children: outline_from_value(main, path)?,
        },
        "list" => Node::List {
            ordered: match main.as_str() {
                Some("ordered") => true,
                Some("bullet") => false,
                _ => return err(path, "write `list: ordered` or `list: bullet`"),
            },
            start: f.opt_u64("start")?,
            children: f.children()?,
        },
        "item" => Node::Item {
            children: outline_from_value(main, path)?,
        },
        "thematic-break" => {
            expect_null(main, path, kind)?;
            Node::ThematicBreak
        }
        "html" => Node::Html {
            text: opt_scalar_string(main, path)?,
        },
        "table" => Node::Table {
            rows: match main {
                Value::Null => None,
                Value::Sequence(rows) => Some(
                    rows.iter()
                        .enumerate()
                        .map(|(i, v)| row_from_value(v, &format!("{path}.table[{i}]")))
                        .collect::<Result<_, _>>()?,
                ),
                _ => return err(path, "write `table:` alone, or with a list of rows"),
            },
        },
        "directive" => {
            let name = match main.as_str() {
                Some(name) => name.to_owned(),
                None => return err(path, "write `directive: <name>`"),
            };
            let form = match f.opt_string("form")?.as_deref() {
                None | Some("line") => Form::Line,
                Some("container") => Form::Container,
                Some(other) => return err(path, format!("unknown form `{other}`")),
            };
            let binding = match f.opt_string("binding")?.as_deref() {
                None => None,
                Some("self") => Some(Binding::SelfBinding),
                Some("heading") => Some(Binding::Heading),
                Some("following-block") => Some(Binding::FollowingBlock),
                Some("none") => Some(Binding::Unbound),
                Some(other) => return err(path, format!("unknown binding `{other}`")),
            };
            let children = f.children()?;
            if form == Form::Line && !children.is_empty() {
                return err(path, "a line-form directive has no children");
            }
            if form == Form::Container && binding.is_some() {
                return err(path, "a container has no binding");
            }
            Node::Directive(Directive {
                name,
                form,
                attributes: f.attributes()?,
                primary: f.opt_string("primary")?,
                title: f.opt_string("title")?,
                binding,
                children,
            })
        }
        "group" => {
            let name = match main.as_str() {
                Some(name) => name.to_owned(),
                None => return err(path, "write `group: <name>`"),
            };
            let arms = match f.take("arms") {
                Some(Value::Sequence(arms)) => arms
                    .iter()
                    .enumerate()
                    .map(|(i, v)| arm_from_value(v, &format!("{path}.arms[{i}]")))
                    .collect::<Result<Vec<_>, _>>()?,
                _ => return err(path, "a group needs `arms`, a list"),
            };
            Node::Group(Group { name, arms })
        }
        other => return err(path, format!("unknown block kind `{other}`")),
    };
    f.finish()?;
    Ok(node)
}

/// A row: its first cell's text, or a mapping with `row` (the text) and
/// `attributes`.
fn row_from_value(value: &Value, path: &str) -> Result<Row, OutlineError> {
    let Value::Mapping(map) = value else {
        return Ok(Row {
            text: scalar_string(value, path)?,
            attributes: Attributes::new(),
        });
    };
    let mut f = Fields::new(map.clone(), path)?;
    let row = Row {
        text: match f.opt_string("row")? {
            Some(text) => text,
            None => return err(path, "a row is its first cell's text, or `row: <text>`"),
        },
        attributes: f.attributes()?,
    };
    f.finish()?;
    Ok(row)
}

fn arm_from_value(value: &Value, path: &str) -> Result<Arm, OutlineError> {
    let Value::Mapping(map) = value else {
        return err(path, "an arm is a mapping");
    };
    let mut f = Fields::new(map.clone(), path)?;
    let arm = Arm {
        attributes: f.attributes()?,
        title: f.opt_string("title")?,
        children: f.children()?,
    };
    f.finish()?;
    Ok(arm)
}

fn expect_null(value: &Value, path: &str, kind: &str) -> Result<(), OutlineError> {
    if value.is_null() {
        Ok(())
    } else {
        err(path, format!("`{kind}` takes no value"))
    }
}

fn opt_scalar_string(value: &Value, path: &str) -> Result<Option<String>, OutlineError> {
    match value {
        Value::Null => Ok(None),
        other => scalar_string(other, path).map(Some),
    }
}

/// Converts a YAML scalar to the string an author meant. Floats are rejected,
/// because YAML reads `3.10` as `3.1`.
fn scalar_string(value: &Value, path: &str) -> Result<String, OutlineError> {
    match value {
        Value::String(s) => Ok(s.clone()),
        Value::Bool(b) => Ok(b.to_string()),
        Value::Number(n) if n.is_i64() || n.is_u64() => Ok(n.to_string()),
        Value::Number(n) => err(
            path,
            format!("`{n}` is a YAML float; quote it so it stays exactly as written"),
        ),
        _ => err(path, "expected a string"),
    }
}

/// The non-kind keys of a node's mapping, consumed field by field so that
/// leftovers can be reported as unknown.
struct Fields {
    map: BTreeMap<String, Value>,
    path: String,
}

impl Fields {
    fn new(map: Mapping, path: &str) -> Result<Self, OutlineError> {
        let mut out = BTreeMap::new();
        for (k, v) in map {
            match k {
                Value::String(k) => {
                    out.insert(k, v);
                }
                _ => return err(path, "field names must be strings"),
            }
        }
        Ok(Self {
            map: out,
            path: path.to_owned(),
        })
    }

    fn take(&mut self, key: &str) -> Option<Value> {
        self.map.remove(key)
    }

    fn field_path(&self, key: &str) -> String {
        format!("{}.{key}", self.path)
    }

    fn opt_string(&mut self, key: &str) -> Result<Option<String>, OutlineError> {
        let path = self.field_path(key);
        match self.take(key) {
            None | Some(Value::Null) => Ok(None),
            Some(v) => scalar_string(&v, &path).map(Some),
        }
    }

    fn opt_bool(&mut self, key: &str) -> Result<Option<bool>, OutlineError> {
        let path = self.field_path(key);
        match self.take(key) {
            None | Some(Value::Null) => Ok(None),
            Some(Value::Bool(b)) => Ok(Some(b)),
            Some(_) => err(&path, "expected true or false"),
        }
    }

    fn opt_u64(&mut self, key: &str) -> Result<Option<u64>, OutlineError> {
        let path = self.field_path(key);
        match self.take(key) {
            None | Some(Value::Null) => Ok(None),
            Some(Value::Number(n)) if n.is_u64() => Ok(n.as_u64()),
            Some(_) => err(&path, "expected a non-negative integer"),
        }
    }

    fn children(&mut self) -> Result<Outline, OutlineError> {
        let path = self.field_path("children");
        match self.take("children") {
            None => Ok(Vec::new()),
            Some(v) => outline_from_value(&v, &path),
        }
    }

    fn attributes(&mut self) -> Result<Attributes, OutlineError> {
        let path = self.field_path("attributes");
        let map = match self.take("attributes") {
            None | Some(Value::Null) => return Ok(Attributes::new()),
            Some(Value::Mapping(map)) => map,
            Some(_) => return err(&path, "expected a mapping of keys to values"),
        };
        let mut out = Attributes::new();
        for (k, v) in map {
            let Value::String(key) = k else {
                return err(&path, "attribute keys must be strings");
            };
            let vpath = format!("{path}.{key}");
            let value = match &v {
                Value::Sequence(items) => AttrValue::Set(
                    items
                        .iter()
                        .map(|item| scalar_string(item, &vpath))
                        .collect::<Result<_, _>>()?,
                ),
                Value::Null => {
                    return err(&vpath, "a bare key has no value in the outline; give one");
                }
                other => AttrValue::Single(scalar_string(other, &vpath)?),
            };
            out.insert(key, value);
        }
        Ok(out)
    }

    fn finish(self) -> Result<(), OutlineError> {
        match self.map.keys().next() {
            None => Ok(()),
            Some(key) => err(&self.path, format!("unknown field `{key}`")),
        }
    }
}

// ---------------------------------------------------------------------------
// Outline to YAML, for printing actual outlines

fn s(v: &str) -> Value {
    Value::String(v.to_owned())
}

fn put(map: &mut Mapping, key: &str, value: Value) {
    map.insert(s(key), value);
}

fn put_opt(map: &mut Mapping, key: &str, value: &Option<String>) {
    if let Some(v) = value {
        put(map, key, s(v));
    }
}

fn put_children(map: &mut Mapping, children: &Outline) {
    if !children.is_empty() {
        put(map, "children", outline_to_value(children));
    }
}

fn put_attributes(map: &mut Mapping, attributes: &Attributes) {
    if attributes.is_empty() {
        return;
    }
    let mut m = Mapping::new();
    for (k, v) in attributes {
        let value = match v {
            AttrValue::Single(v) => s(v),
            AttrValue::Set(vs) => Value::Sequence(vs.iter().map(|v| s(v)).collect()),
        };
        m.insert(s(k), value);
    }
    put(map, "attributes", Value::Mapping(m));
}

/// Converts an outline to its YAML form.
pub fn outline_to_value(outline: &Outline) -> Value {
    Value::Sequence(outline.iter().map(node_to_value).collect())
}

fn opt_value(v: &Option<String>) -> Value {
    v.as_deref().map(s).unwrap_or(Value::Null)
}

fn node_to_value(node: &Node) -> Value {
    let mut m = Mapping::new();
    match node {
        Node::Heading { level, text } => {
            put(&mut m, "heading", Value::from(*level));
            put_opt(&mut m, "text", text);
        }
        Node::Paragraph { text } => put(&mut m, "paragraph", opt_value(text)),
        Node::Image {
            src,
            alt,
            title,
            attributes,
        } => {
            put(&mut m, "image", s(src));
            put_opt(&mut m, "alt", alt);
            put_opt(&mut m, "title", title);
            put_attributes(&mut m, attributes);
        }
        Node::Code { text, info, fenced } => {
            put(&mut m, "code", opt_value(text));
            put_opt(&mut m, "info", info);
            if let Some(fenced) = fenced {
                put(&mut m, "fenced", Value::Bool(*fenced));
            }
        }
        Node::Blockquote { children } => put(&mut m, "blockquote", outline_to_value(children)),
        Node::List {
            ordered,
            start,
            children,
        } => {
            put(
                &mut m,
                "list",
                s(if *ordered { "ordered" } else { "bullet" }),
            );
            if let Some(start) = start {
                put(&mut m, "start", Value::from(*start));
            }
            put_children(&mut m, children);
        }
        Node::Item { children } => put(&mut m, "item", outline_to_value(children)),
        Node::ThematicBreak => return s("thematic-break"),
        Node::Html { text } => put(&mut m, "html", opt_value(text)),
        Node::Table { rows: None } => return s("table"),
        Node::Table { rows: Some(rows) } => {
            let rows = rows
                .iter()
                .map(|row| {
                    if row.attributes.is_empty() {
                        return s(&row.text);
                    }
                    let mut r = Mapping::new();
                    put(&mut r, "row", s(&row.text));
                    put_attributes(&mut r, &row.attributes);
                    Value::Mapping(r)
                })
                .collect();
            put(&mut m, "table", Value::Sequence(rows));
        }
        Node::Directive(d) => {
            put(&mut m, "directive", s(&d.name));
            if d.form == Form::Container {
                put(&mut m, "form", s("container"));
            }
            put_attributes(&mut m, &d.attributes);
            put_opt(&mut m, "primary", &d.primary);
            put_opt(&mut m, "title", &d.title);
            if let Some(b) = d.binding {
                put(&mut m, "binding", s(&b.to_string()));
            }
            put_children(&mut m, &d.children);
        }
        Node::Group(g) => {
            put(&mut m, "group", s(&g.name));
            let arms = g
                .arms
                .iter()
                .map(|arm| {
                    let mut a = Mapping::new();
                    put_attributes(&mut a, &arm.attributes);
                    put_opt(&mut a, "title", &arm.title);
                    put_children(&mut a, &arm.children);
                    Value::Mapping(a)
                })
                .collect();
            put(&mut m, "arms", Value::Sequence(arms));
        }
    }
    Value::Mapping(m)
}

/// Renders an outline as YAML, in the form test authors write.
pub fn outline_to_yaml(outline: &Outline) -> String {
    serde_yaml_ng::to_string(&outline_to_value(outline))
        .unwrap_or_else(|e| format!("<couldn't render outline: {e}>"))
}

impl fmt::Display for Binding {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Binding::SelfBinding => "self",
            Binding::Heading => "heading",
            Binding::FollowingBlock => "following-block",
            Binding::Unbound => "none",
        })
    }
}

/// Deserializes an outline field in `expect.yaml`.
pub(crate) fn deserialize_outline<'de, D: Deserializer<'de>>(
    d: D,
) -> Result<Option<Outline>, D::Error> {
    let value = Option::<Value>::deserialize(d)?;
    match value {
        None => Ok(None),
        Some(v) => outline_from_value(&v, "outline")
            .map(Some)
            .map_err(serde::de::Error::custom),
    }
}

/// Serializes an outline field, for round trips.
pub(crate) fn serialize_outline<S: Serializer>(
    outline: &Option<Outline>,
    s: S,
) -> Result<S::Ok, S::Error> {
    outline.as_ref().map(outline_to_value).serialize(s)
}

// ---------------------------------------------------------------------------
// Comparison

/// Compares an actual outline with an expected one. Returns every difference
/// found, each prefixed with its path, or an empty list when they match.
pub fn compare(expected: &Outline, actual: &Outline) -> Vec<String> {
    let mut diffs = Vec::new();
    compare_lists(expected, actual, "outline", &mut diffs);
    diffs
}

fn kind_name(node: &Node) -> String {
    match node {
        Node::Heading { level, .. } => format!("heading {level}"),
        Node::Paragraph { .. } => "paragraph".into(),
        Node::Image { .. } => "image".into(),
        Node::Code { .. } => "code".into(),
        Node::Blockquote { .. } => "blockquote".into(),
        Node::List { ordered, .. } => {
            format!("{} list", if *ordered { "ordered" } else { "bullet" })
        }
        Node::Item { .. } => "item".into(),
        Node::ThematicBreak => "thematic-break".into(),
        Node::Html { .. } => "html".into(),
        Node::Table { .. } => "table".into(),
        Node::Directive(d) => format!("directive `{}`", d.name),
        Node::Group(g) => format!("group `{}`", g.name),
    }
}

fn compare_lists(expected: &Outline, actual: &Outline, path: &str, diffs: &mut Vec<String>) {
    for (i, (e, a)) in expected.iter().zip(actual).enumerate() {
        compare_nodes(e, a, &format!("{path}[{i}]"), diffs);
    }
    if expected.len() != actual.len() {
        let extra = |nodes: &[Node]| nodes.iter().map(kind_name).collect::<Vec<_>>().join(", ");
        if expected.len() > actual.len() {
            diffs.push(format!(
                "{path}: {} missing block(s) at the end: {}",
                expected.len() - actual.len(),
                extra(&expected[actual.len()..])
            ));
        } else {
            diffs.push(format!(
                "{path}: {} unexpected block(s) at the end: {}",
                actual.len() - expected.len(),
                extra(&actual[expected.len()..])
            ));
        }
    }
}

fn text_eq(expected: &Option<String>, actual: &Option<String>) -> bool {
    match (expected, actual) {
        (None, _) => true,
        (Some(e), Some(a)) => normalize_ws(e) == normalize_ws(a),
        (Some(_), None) => false,
    }
}

fn strict_text_eq(expected: &Option<String>, actual: &Option<String>) -> bool {
    expected.as_deref().map(normalize_ws) == actual.as_deref().map(normalize_ws)
}

fn show(v: &Option<String>) -> String {
    match v {
        Some(v) => format!("{v:?}"),
        None => "none".into(),
    }
}

fn check(ok: bool, path: &str, field: &str, e: String, a: String, diffs: &mut Vec<String>) {
    if !ok {
        diffs.push(format!("{path}.{field}: expected {e}, got {a}"));
    }
}

fn show_attrs(attrs: &Attributes) -> String {
    let parts: Vec<String> = attrs
        .iter()
        .map(|(k, v)| match v {
            AttrValue::Single(v) => format!("{k}={v:?}"),
            AttrValue::Set(vs) => format!("{k}={}", vs.join("|")),
        })
        .collect();
    format!("{{{}}}", parts.join(", "))
}

fn compare_nodes(e: &Node, a: &Node, path: &str, diffs: &mut Vec<String>) {
    use Node::*;
    match (e, a) {
        (
            Heading {
                level: el,
                text: et,
            },
            Heading {
                level: al,
                text: at,
            },
        ) => {
            check(
                el == al,
                path,
                "level",
                el.to_string(),
                al.to_string(),
                diffs,
            );
            check(text_eq(et, at), path, "text", show(et), show(at), diffs);
        }
        (Paragraph { text: et }, Paragraph { text: at })
        | (Html { text: et }, Html { text: at }) => {
            check(text_eq(et, at), path, "text", show(et), show(at), diffs);
        }
        (
            Image {
                src: es,
                alt: ea,
                title: et,
                attributes: eattr,
            },
            Image {
                src: as_,
                alt: aa,
                title: at,
                attributes: aattr,
            },
        ) => {
            check(
                es == as_,
                path,
                "src",
                format!("{es:?}"),
                format!("{as_:?}"),
                diffs,
            );
            check(text_eq(ea, aa), path, "alt", show(ea), show(aa), diffs);
            check(
                strict_text_eq(et, at),
                path,
                "title",
                show(et),
                show(at),
                diffs,
            );
            check(
                eattr == aattr,
                path,
                "attributes",
                show_attrs(eattr),
                show_attrs(aattr),
                diffs,
            );
        }
        (
            Code {
                text: et,
                info: ei,
                fenced: ef,
            },
            Code {
                text: at,
                info: ai,
                fenced: af,
            },
        ) => {
            let code_eq = match (et, at) {
                (None, _) => true,
                (Some(e), Some(a)) => e.trim_end_matches('\n') == a.trim_end_matches('\n'),
                (Some(_), None) => false,
            };
            check(code_eq, path, "text", show(et), show(at), diffs);
            check(text_eq(ei, ai), path, "info", show(ei), show(ai), diffs);
            if ef.is_some() && ef != af {
                check(
                    false,
                    path,
                    "fenced",
                    format!("{ef:?}"),
                    format!("{af:?}"),
                    diffs,
                );
            }
        }
        (Blockquote { children: ec }, Blockquote { children: ac })
        | (Item { children: ec }, Item { children: ac }) => {
            compare_lists(ec, ac, path, diffs);
        }
        (
            List {
                ordered: eo,
                start: es,
                children: ec,
            },
            List {
                ordered: ao,
                start: as_,
                children: ac,
            },
        ) => {
            check(eo == ao, path, "list", kind_name(e), kind_name(a), diffs);
            if es.is_some() && es != as_ {
                check(
                    false,
                    path,
                    "start",
                    format!("{es:?}"),
                    format!("{as_:?}"),
                    diffs,
                );
            }
            compare_lists(ec, ac, &format!("{path}.children"), diffs);
        }
        (ThematicBreak, ThematicBreak) => {}
        (Table { rows: er }, Table { rows: ar }) => {
            let Some(er) = er else {
                return;
            };
            let Some(ar) = ar else {
                diffs.push(format!("{path}.table: expected rows, got none"));
                return;
            };
            let shown = |rows: &[Row]| {
                let rows: Vec<String> = rows
                    .iter()
                    .map(|r| format!("{:?} {}", r.text, show_attrs(&r.attributes)))
                    .collect();
                format!("[{}]", rows.join(", "))
            };
            let same = er.len() == ar.len()
                && er.iter().zip(ar).all(|(e, a)| {
                    normalize_ws(&e.text) == normalize_ws(&a.text) && e.attributes == a.attributes
                });
            check(same, path, "table", shown(er), shown(ar), diffs);
        }
        (Directive(ed), Directive(ad)) => {
            check(
                ed.name == ad.name,
                path,
                "directive",
                ed.name.clone(),
                ad.name.clone(),
                diffs,
            );
            check(
                ed.form == ad.form,
                path,
                "form",
                format!("{:?}", ed.form).to_lowercase(),
                format!("{:?}", ad.form).to_lowercase(),
                diffs,
            );
            check(
                ed.attributes == ad.attributes,
                path,
                "attributes",
                show_attrs(&ed.attributes),
                show_attrs(&ad.attributes),
                diffs,
            );
            check(
                strict_text_eq(&ed.primary, &ad.primary),
                path,
                "primary",
                show(&ed.primary),
                show(&ad.primary),
                diffs,
            );
            check(
                strict_text_eq(&ed.title, &ad.title),
                path,
                "title",
                show(&ed.title),
                show(&ad.title),
                diffs,
            );
            if ed.binding.is_some() && ed.binding != ad.binding {
                let show_b = |b: &Option<Binding>| match b {
                    Some(b) => b.to_string(),
                    None => "nothing".into(),
                };
                check(
                    false,
                    path,
                    "binding",
                    show_b(&ed.binding),
                    show_b(&ad.binding),
                    diffs,
                );
            }
            compare_lists(
                &ed.children,
                &ad.children,
                &format!("{path}.children"),
                diffs,
            );
        }
        (Group(eg), Group(ag)) => {
            check(
                eg.name == ag.name,
                path,
                "group",
                eg.name.clone(),
                ag.name.clone(),
                diffs,
            );
            for (i, (ea, aa)) in eg.arms.iter().zip(&ag.arms).enumerate() {
                let apath = format!("{path}.arms[{i}]");
                check(
                    ea.attributes == aa.attributes,
                    &apath,
                    "attributes",
                    show_attrs(&ea.attributes),
                    show_attrs(&aa.attributes),
                    diffs,
                );
                check(
                    strict_text_eq(&ea.title, &aa.title),
                    &apath,
                    "title",
                    show(&ea.title),
                    show(&aa.title),
                    diffs,
                );
                compare_lists(
                    &ea.children,
                    &aa.children,
                    &format!("{apath}.children"),
                    diffs,
                );
            }
            check(
                eg.arms.len() == ag.arms.len(),
                path,
                "arms",
                format!("{} arm(s)", eg.arms.len()),
                format!("{} arm(s)", ag.arms.len()),
                diffs,
            );
        }
        _ => diffs.push(format!(
            "{path}: expected {}, got {}",
            kind_name(e),
            kind_name(a)
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(yaml: &str) -> Result<Outline, OutlineError> {
        let v: Value = serde_yaml_ng::from_str(yaml).unwrap();
        outline_from_value(&v, "outline")
    }

    #[test]
    fn parses_every_kind() {
        let outline = parse(
            r#"
- heading: 2
  text: Install
- paragraph: Some text.
- paragraph
- image: settings.png
  alt: The settings page
  attributes: {width: 600}
- code: "npm install\n"
  info: shell
- blockquote:
    - paragraph
- list: ordered
  start: 1
  children:
    - item:
        - paragraph: One
- thematic-break
- html: <div>
- table
- directive: note
  attributes: {type: tip}
  title: Try it
  binding: following-block
- directive: note
  form: container
  children:
    - paragraph
- group: variant
  arms:
    - attributes: {pm: npm, platform: [cloud, on-prem]}
      children:
        - code
    - title: Other
"#,
        )
        .unwrap();
        assert_eq!(outline.len(), 13);
        // Round trip through the YAML form.
        let again = parse(&outline_to_yaml(&outline)).unwrap();
        assert_eq!(outline, again);
    }

    #[test]
    fn rejects_typos_and_ambiguity() {
        let e = parse("- directive: note\n  attribute: {type: tip}\n").unwrap_err();
        assert_eq!(e.path, "outline[0]");
        assert!(e.message.contains("unknown field `attribute`"));

        let e = parse("- paragraph: a\n  heading: 2\n").unwrap_err();
        assert!(e.message.contains("more than one block kind"));

        let e = parse("- directive: available\n  attributes: {since: 3.10}\n").unwrap_err();
        assert!(e.message.contains("YAML float"), "{e}");

        let e = parse("- directive: note\n  children: [paragraph]\n").unwrap_err();
        assert!(e.message.contains("line-form directive has no children"));
    }

    #[test]
    fn content_fields_are_optional_structural_fields_are_not() {
        let expected = parse("- paragraph\n- directive: note\n").unwrap();
        let actual = parse(
            "- paragraph: Hello\n- directive: note\n  binding: following-block\n  title: A title\n",
        )
        .unwrap();
        let diffs = compare(&expected, &actual);
        assert_eq!(
            diffs,
            vec![r#"outline[1].title: expected none, got "A title""#.to_owned()]
        );
    }

    #[test]
    fn text_compares_with_whitespace_normalized() {
        let expected = parse("- paragraph: Back up your database before you upgrade.\n").unwrap();
        let actual =
            parse("- paragraph: \"Back up your database\\nbefore you upgrade.\"\n").unwrap();
        assert!(compare(&expected, &actual).is_empty());
    }

    #[test]
    fn reports_nested_differences_with_paths() {
        let expected = parse(
            "- group: variant\n  arms:\n    - attributes: {pm: npm}\n      children: [code]\n",
        )
        .unwrap();
        let actual = parse(
            "- group: variant\n  arms:\n    - attributes: {pm: pnpm}\n      children: [code, paragraph]\n",
        )
        .unwrap();
        let diffs = compare(&expected, &actual);
        assert_eq!(diffs.len(), 2, "{diffs:?}");
        assert!(diffs[0].starts_with("outline[0].arms[0].attributes"));
        assert!(diffs[1].contains("unexpected block(s) at the end: paragraph"));
    }
}
