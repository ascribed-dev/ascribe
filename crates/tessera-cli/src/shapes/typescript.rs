//! TypeScript types from the JSON Schemas `schemars` derives.
//!
//! It handles what the schemas of Ascribe's JSON use, and panics on anything
//! else, so a new kind of field is noticed here rather than typed wrongly:
//! objects with properties, maps, arrays and fixed-length arrays, strings,
//! numbers, booleans, `null`, string constants and enumerations, unions,
//! references to `$defs`, and `true` (any value).
//!
//! An interface lists the required properties in the order the Rust type
//! declares them, then the optional ones by name.
//!
//! A property the schema doesn't require is optional (`name?: T`) and never
//! `null`: in the serializing schema `schemars` writes, a property is left out
//! of `required` only when serde skips it, which Ascribe does only for `None`.

use std::collections::BTreeMap;
use std::fmt::Write as _;

use serde_json::{Map, Value};

/// The types named in `defs`, each exported, in name order.
pub(super) fn module(defs: &BTreeMap<String, Value>) -> String {
    let mut out = String::new();
    for (name, schema) in defs {
        out.push('\n');
        doc(&mut out, "", schema);
        match properties(schema) {
            Some(properties) => {
                writeln!(out, "export interface {name} {{").ok();
                members(&mut out, schema, properties, "  ");
                out.push_str("}\n");
            }
            None => {
                writeln!(out, "export type {name} = {};", type_of(schema)).ok();
            }
        }
    }
    out
}

/// A schema's `properties`, when it's an object that has them.
fn properties(schema: &Value) -> Option<&Map<String, Value>> {
    schema.get("properties").and_then(Value::as_object)
}

/// Each property of an object, with its description.
fn members(out: &mut String, schema: &Value, properties: &Map<String, Value>, indent: &str) {
    let required: Vec<&str> = schema
        .get("required")
        .and_then(Value::as_array)
        .map(|names| names.iter().filter_map(Value::as_str).collect())
        .unwrap_or_default();
    // The required properties in the order the type declares them, then the
    // others; the schema's own order is the names'.
    let mut names: Vec<&str> = required.clone();
    names.extend(
        properties
            .keys()
            .map(String::as_str)
            .filter(|name| !required.contains(name)),
    );
    for name in names {
        let property = &properties[name];
        doc(out, indent, property);
        if required.contains(&name) {
            writeln!(out, "{indent}{name}: {};", type_of(property)).ok();
        } else {
            writeln!(
                out,
                "{indent}{name}?: {};",
                type_of(&without_null(property))
            )
            .ok();
        }
    }
}

/// A JSDoc comment of the schema's description, if it has one.
fn doc(out: &mut String, indent: &str, schema: &Value) {
    let Some(text) = schema.get("description").and_then(Value::as_str) else {
        return;
    };
    let lines: Vec<&str> = text.lines().collect();
    if let [line] = lines.as_slice() {
        writeln!(out, "{indent}/** {line} */").ok();
        return;
    }
    writeln!(out, "{indent}/**").ok();
    for line in lines {
        if line.is_empty() {
            writeln!(out, "{indent} *").ok();
        } else {
            writeln!(out, "{indent} * {line}").ok();
        }
    }
    writeln!(out, "{indent} */").ok();
}

/// The schema without `null` among its types.
fn without_null(schema: &Value) -> Value {
    let mut schema = schema.clone();
    if let Some(types) = schema.get_mut("type").and_then(Value::as_array_mut) {
        types.retain(|t| t != "null");
        if let [only] = types.as_slice() {
            let only = only.clone();
            schema["type"] = only;
        }
    }
    for key in ["anyOf", "oneOf"] {
        if let Some(options) = schema.get_mut(key).and_then(Value::as_array_mut) {
            options.retain(|option| option.get("type").and_then(Value::as_str) != Some("null"));
            if let [only] = options.as_slice() {
                let only = only.clone();
                return only;
            }
        }
    }
    schema
}

/// The TypeScript type of a schema.
fn type_of(schema: &Value) -> String {
    if schema == &Value::Bool(true) {
        return "unknown".to_owned();
    }
    if let Some(reference) = schema.get("$ref").and_then(Value::as_str) {
        return reference
            .strip_prefix("#/$defs/")
            .unwrap_or_else(|| panic!("a reference outside $defs: {reference}"))
            .to_owned();
    }
    if let Some(value) = schema.get("const") {
        return literal(value);
    }
    if let Some(values) = schema.get("enum").and_then(Value::as_array) {
        return values.iter().map(literal).collect::<Vec<_>>().join(" | ");
    }
    for key in ["anyOf", "oneOf"] {
        if let Some(options) = schema.get(key).and_then(Value::as_array) {
            return options.iter().map(type_of).collect::<Vec<_>>().join(" | ");
        }
    }
    match schema.get("type") {
        Some(Value::String(name)) => named(name, schema),
        Some(Value::Array(names)) => names
            .iter()
            .map(|name| named(name.as_str().unwrap_or_default(), schema))
            .collect::<Vec<_>>()
            .join(" | "),
        _ => panic!("a schema with no type: {schema}"),
    }
}

/// The TypeScript type of one of a schema's JSON types.
fn named(name: &str, schema: &Value) -> String {
    match name {
        "string" => "string".to_owned(),
        "integer" | "number" => "number".to_owned(),
        "boolean" => "boolean".to_owned(),
        "null" => "null".to_owned(),
        "array" => array(schema),
        "object" => object(schema),
        _ => panic!("an unknown type {name}: {schema}"),
    }
}

fn array(schema: &Value) -> String {
    let items = schema
        .get("items")
        .unwrap_or_else(|| panic!("an array with no items: {schema}"));
    let item = type_of(items);
    let length = |key: &str| schema.get(key).and_then(Value::as_u64);
    match (length("minItems"), length("maxItems")) {
        (Some(min), Some(max)) if min == max => {
            let items = vec![item; usize::try_from(min).unwrap_or_default()];
            format!("[{}]", items.join(", "))
        }
        _ if item.contains(" | ") => format!("({item})[]"),
        _ => format!("{item}[]"),
    }
}

fn object(schema: &Value) -> String {
    if let Some(properties) = properties(schema) {
        let mut out = String::from("{\n");
        members(&mut out, schema, properties, "  ");
        out.push('}');
        return out;
    }
    match schema.get("additionalProperties") {
        Some(Value::Bool(true)) | None => "Record<string, unknown>".to_owned(),
        Some(values) => format!("Record<string, {}>", type_of(values)),
    }
}

fn literal(value: &Value) -> String {
    serde_json::to_string(value).unwrap_or_default()
}
