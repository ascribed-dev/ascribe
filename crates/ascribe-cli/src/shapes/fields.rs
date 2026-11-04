//! The tables of fields the docs show for a schema, before the schema itself.
//!
//! One table is the document's, and one is each type it names, in the order
//! the tables first name them. A type named once, that's an object, has no
//! table of its own: its fields are rows of the table that names it, under the
//! field's path (`diagnostics[].file`). A union of objects, such as a block,
//! has a table of the fields every option has, then one for each option, named
//! by its tag (`` `type` is `"heading"` ``); a union of strings has a table of
//! its values.
//!
//! A field is nullable when it's required and can be `null`. A field that
//! isn't required is left out when it has no value, and never `null`, as the
//! JSON reports contract says.

use std::collections::{BTreeMap, VecDeque};
use std::fmt::Write as _;

use serde_json::{Map, Value};

/// The tables of `schema`, whose title is `title`, as Markdown.
pub(super) fn tables(title: &str, schema: &Value) -> String {
    let defs = schema
        .get("$defs")
        .and_then(Value::as_object)
        .cloned()
        .unwrap_or_default();
    let mut uses = BTreeMap::new();
    count_refs(schema, &mut uses);
    let mut tables = Tables {
        defs: &defs,
        uses,
        queue: VecDeque::new(),
        named: Vec::new(),
    };
    let mut out = String::new();
    tables.write_type(&mut out, title, schema);
    while let Some(name) = tables.queue.pop_front() {
        let def = &defs[&name];
        out.push('\n');
        tables.write_type(&mut out, &name, def);
    }
    out
}

struct Tables<'a> {
    defs: &'a Map<String, Value>,
    /// How many times each type is named in the whole schema.
    uses: BTreeMap<String, usize>,
    /// The types named that have a table, and haven't had it written yet.
    queue: VecDeque<String>,
    /// Every type given a table, so far.
    named: Vec<String>,
}

const HEAD: &str = "| Field | Type | Required | Nullable | Description |\n\
                    | --- | --- | --- | --- | --- |\n";

impl Tables<'_> {
    /// The table, or tables, of a type: its name, its description, then its
    /// fields, the options of a union, or the values of an enumeration.
    fn write_type(&mut self, out: &mut String, name: &str, schema: &Value) {
        let options = union(schema);
        if let Some(values) = options.and_then(|options| enumeration(options)) {
            writeln!(out, "{} One of:\n", label(name, schema)).ok();
            out.push_str("| Value | Description |\n| --- | --- |\n");
            for (value, description) in values {
                writeln!(out, "| `{value}` | {} |", cell(description)).ok();
            }
            return;
        }
        let Some(options) = options.filter(|o| o.iter().any(|o| properties(o).is_some())) else {
            if properties(schema).is_none() {
                let ty = self.type_of(schema);
                writeln!(out, "{} Its type is `{}`.", label(name, schema), code(&ty)).ok();
                return;
            }
            writeln!(out, "{}\n", label(name, schema)).ok();
            out.push_str(HEAD);
            self.rows(out, "", schema, &mut Vec::new());
            return;
        };
        if properties(schema).is_some() {
            writeln!(
                out,
                "{} Every option has these fields:\n",
                label(name, schema)
            )
            .ok();
            out.push_str(HEAD);
            self.rows(out, "", schema, &mut Vec::new());
        } else {
            writeln!(out, "{} One of these options:", label(name, schema)).ok();
        }
        for option in options {
            out.push('\n');
            let description = description(option)
                .map(|text| format!(": {}", line(text)))
                .unwrap_or_default();
            writeln!(out, "**`{name}`** {}{description}\n", option_name(option)).ok();
            if properties(option).is_some() {
                out.push_str(HEAD);
                self.rows(out, "", option, &mut Vec::new());
            } else {
                let ty = self.type_of(option);
                writeln!(out, "Its type is `{}`.", code(&ty)).ok();
            }
        }
    }

    /// A row for each property of `schema`, under `prefix`, and the rows of
    /// each object it has, or names once, under the property's path.
    fn rows(&mut self, out: &mut String, prefix: &str, schema: &Value, inlined: &mut Vec<String>) {
        let Some(properties) = properties(schema) else {
            return;
        };
        let required: Vec<&str> = schema
            .get("required")
            .and_then(Value::as_array)
            .map(|names| names.iter().filter_map(Value::as_str).collect())
            .unwrap_or_default();
        // The required properties in the order the type declares them, then
        // the others by name, as the TypeScript types list them.
        let mut names: Vec<&str> = required.clone();
        names.extend(
            properties
                .keys()
                .map(String::as_str)
                .filter(|name| !required.contains(name)),
        );
        for name in names {
            let property = &properties[name];
            let is_required = required.contains(&name);
            let path = format!("{prefix}{name}");
            let (shown, nullable) = without_null(property);
            let nullable = nullable && is_required;
            // The object whose fields are rows under this one's path, and the
            // path they're under: an inline object, or a type named once.
            let (inner, suffix) = match shown.get("items") {
                Some(items) if is_array(&shown) => (items.clone(), "[]."),
                _ => (shown.clone(), "."),
            };
            let nested = self.nested(&inner, inlined);
            let ty = match &nested {
                Some(_) if suffix == "[]." => "object[]".to_owned(),
                Some(_) => "object".to_owned(),
                None => self.type_of(&shown),
            };
            let always = property
                .get("const")
                .map(|value| format!("Always `{}`.", literal(value)));
            let text = description(property)
                .or_else(|| nested.as_ref().and_then(|(_, def)| description(def)))
                .or(always.as_deref())
                .unwrap_or_default();
            writeln!(
                out,
                "| `{path}` | `{}` | {} | {} | {} |",
                code(&ty),
                yes(is_required),
                yes(nullable),
                cell(text)
            )
            .ok();
            if let Some((name, object)) = nested {
                inlined.extend(name.clone());
                self.rows(out, &format!("{path}{suffix}"), &object, inlined);
                if name.is_some() {
                    inlined.pop();
                }
            }
        }
    }

    /// The object whose fields are written in the table that names it, with
    /// its type's name if it has one: an object written inline, or a type
    /// that's an object, named once in the schema, and not already being
    /// written.
    fn nested(&self, schema: &Value, inlined: &[String]) -> Option<(Option<String>, Value)> {
        if let Some(name) = reference(schema) {
            let def = self.defs.get(name)?;
            let once = self.uses.get(name).copied() == Some(1);
            let plain = properties(def).is_some() && union(def).is_none();
            return (once && plain && !inlined.iter().any(|n| n == name))
                .then(|| (Some(name.to_owned()), def.clone()));
        }
        (properties(schema).is_some() && union(schema).is_none()).then(|| (None, schema.clone()))
    }

    /// The type a cell shows, with each type named queued for its table.
    fn type_of(&mut self, schema: &Value) -> String {
        if schema == &Value::Bool(true) || any_value(schema) {
            return "any".to_owned();
        }
        if let Some(name) = reference(schema) {
            if !self.named.iter().any(|n| n == name) {
                self.named.push(name.to_owned());
                self.queue.push_back(name.to_owned());
            }
            return name.to_owned();
        }
        if let Some(value) = schema.get("const") {
            return literal(value);
        }
        if let Some(options) = union(schema) {
            let options: Vec<String> = options.iter().map(|o| self.type_of(o)).collect();
            return options.join(" | ");
        }
        match schema.get("type") {
            Some(Value::String(name)) => self.named_type(name, schema),
            Some(Value::Array(names)) => names
                .iter()
                .map(|name| self.named_type(name.as_str().unwrap_or_default(), schema))
                .collect::<Vec<_>>()
                .join(" | "),
            _ => panic!("a schema with no type: {schema}"),
        }
    }

    fn named_type(&mut self, name: &str, schema: &Value) -> String {
        match name {
            "string" | "integer" | "number" | "boolean" | "null" => name.to_owned(),
            "array" => {
                if let Some(prefix) = schema.get("prefixItems").and_then(Value::as_array) {
                    let items: Vec<String> = prefix.iter().map(|i| self.type_of(i)).collect();
                    return format!("[{}]", items.join(", "));
                }
                let items = schema
                    .get("items")
                    .unwrap_or_else(|| panic!("an array with no items: {schema}"));
                let item = self.type_of(items);
                if item.contains(" | ") {
                    format!("({item})[]")
                } else {
                    format!("{item}[]")
                }
            }
            "object" => match schema.get("additionalProperties") {
                Some(Value::Bool(true)) | None => "object".to_owned(),
                Some(values) => format!("{{ [name]: {} }}", self.type_of(values)),
            },
            _ => panic!("an unknown type {name}: {schema}"),
        }
    }
}

/// How many times each type is named in `value`.
fn count_refs(value: &Value, uses: &mut BTreeMap<String, usize>) {
    match value {
        Value::Object(map) => {
            if let Some(name) = reference(value) {
                *uses.entry(name.to_owned()).or_default() += 1;
            }
            map.values().for_each(|v| count_refs(v, uses));
        }
        Value::Array(items) => items.iter().for_each(|v| count_refs(v, uses)),
        _ => {}
    }
}

/// The name of the type a schema refers to.
fn reference(schema: &Value) -> Option<&str> {
    let reference = schema.get("$ref")?.as_str()?;
    Some(
        reference
            .strip_prefix("#/$defs/")
            .unwrap_or_else(|| panic!("a reference outside $defs: {reference}")),
    )
}

/// The options of a union.
fn union(schema: &Value) -> Option<&Vec<Value>> {
    ["anyOf", "oneOf"]
        .iter()
        .find_map(|key| schema.get(*key).and_then(Value::as_array))
}

/// The values of a union of string constants, with their descriptions.
fn enumeration(options: &[Value]) -> Option<Vec<(String, &str)>> {
    options
        .iter()
        .map(|option| {
            let value = option.get("const")?.as_str()?;
            Some((
                literal(&value.into()),
                description(option).unwrap_or_default(),
            ))
        })
        .collect()
}

/// What names an option of a union of objects: the field whose value is a
/// constant, as `` when `type` is `"heading"` ``, or its required fields.
fn option_name(option: &Value) -> String {
    let tag = properties(option).and_then(|properties| {
        properties
            .iter()
            .find_map(|(name, p)| p.get("const").map(|value| (name, literal(value))))
    });
    if let Some((name, value)) = tag {
        return format!("when `{name}` is `{value}`");
    }
    let required: Vec<String> = option
        .get("required")
        .and_then(Value::as_array)
        .map(|names| {
            names
                .iter()
                .filter_map(Value::as_str)
                .map(|name| format!("`{name}`"))
                .collect()
        })
        .unwrap_or_default();
    if required.is_empty() {
        "of another shape".to_owned()
    } else {
        format!("with {}", required.join(" and "))
    }
}

/// The sentence that opens a type's tables: its name, and its description.
fn label(name: &str, schema: &Value) -> String {
    match description(schema) {
        Some(text) => {
            let text = line(text);
            let end = if text.ends_with(['.', '?', '!']) {
                ""
            } else {
                "."
            };
            format!("**`{name}`**: {text}{end}")
        }
        None => format!("**`{name}`**."),
    }
}

/// The schema without `null` among its types, and whether it had it.
fn without_null(schema: &Value) -> (Value, bool) {
    let mut schema = schema.clone();
    let mut nullable = false;
    if let Some(types) = schema.get_mut("type").and_then(Value::as_array_mut) {
        let before = types.len();
        types.retain(|t| t != "null");
        nullable = types.len() < before;
        if let [only] = types.as_slice() {
            let only = only.clone();
            schema["type"] = only;
        }
    }
    for key in ["anyOf", "oneOf"] {
        if let Some(options) = schema.get_mut(key).and_then(Value::as_array_mut) {
            let before = options.len();
            options.retain(|option| option.get("type").and_then(Value::as_str) != Some("null"));
            nullable |= options.len() < before;
            if let [only] = options.as_slice() {
                let only = only.clone();
                return (only, nullable);
            }
        }
    }
    (schema, nullable)
}

fn properties(schema: &Value) -> Option<&Map<String, Value>> {
    schema.get("properties").and_then(Value::as_object)
}

fn is_array(schema: &Value) -> bool {
    schema.get("type").and_then(Value::as_str) == Some("array")
}

fn description(schema: &Value) -> Option<&str> {
    schema.get("description").and_then(Value::as_str)
}

/// Whether a schema has only annotations, and so takes any value.
fn any_value(schema: &Value) -> bool {
    schema.as_object().is_some_and(|map| {
        map.keys().all(|key| {
            matches!(
                key.as_str(),
                "description" | "title" | "default" | "examples" | "deprecated"
            )
        })
    })
}

fn literal(value: &Value) -> String {
    serde_json::to_string(value).unwrap_or_default()
}

/// A description as one line: its lines joined, as Markdown joins them.
fn line(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// A description as a table's cell, with its pipes escaped.
fn cell(text: &str) -> String {
    line(text).replace('|', "\\|")
}

/// A type as a code span in a table's cell, with its pipes escaped.
fn code(ty: &str) -> String {
    ty.replace('|', "\\|")
}

fn yes(value: bool) -> &'static str {
    if value { "yes" } else { "no" }
}
