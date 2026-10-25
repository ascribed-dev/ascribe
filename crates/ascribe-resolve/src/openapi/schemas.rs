//! A spec's request and response bodies, and their schemas as JSON Schema.
//!
//! Each operation's bodies are read by media type, with the pointer to each
//! one's schema in the spec; a `$ref` to a request body, a response, or a
//! parameter in the same file is followed. OpenAPI 3.1's and 3.2's schemas
//! are JSON Schema 2020-12. OpenAPI 3.0's and Swagger 2.0's are read as
//! draft 4, after a translation: `nullable` and `x-nullable` add `null`,
//! `type: file` is dropped, and a `$ref`'s siblings are dropped, as both
//! specs say. In every version, a `readOnly` property isn't required of a
//! request, nor a `writeOnly` one of a response, and each `$ref`'s fragment
//! is percent-encoded, since names hold `\` and spaces (decision #920).

use std::collections::BTreeSet;

use serde_json::{Map, Number, Value};

use crate::json_schema::{Dialect, decode, encode, token};

/// One body of an operation: its request body, or one of its responses, in
/// one media type.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Body {
    /// The response's status as the spec writes it (`201`, `2XX`,
    /// `default`), or `None` for the request body.
    pub status: Option<String>,
    /// Its media type, as the spec writes it. `None` when the body is a
    /// `$ref` into another file, whose media types aren't read.
    pub media_type: Option<String>,
    /// The JSON pointer to its schema in the spec, when it has one.
    pub schema: Option<String>,
}

/// Whether a body is the request's or a response's, which decides whether
/// `readOnly` or `writeOnly` properties stop being required.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Direction {
    /// A request body.
    Request,
    /// A response.
    Response,
}

/// Whether a media type is JSON: `application/json`, or any `…+json`.
pub fn is_json(media_type: &str) -> bool {
    let essence = media_type
        .split(';')
        .next()
        .unwrap_or_default()
        .trim()
        .to_ascii_lowercase();
    essence == "application/json" || essence.ends_with("+json")
}

/// The JSON Schema a spec's schemas are written in.
pub fn dialect(document: &Value) -> Dialect {
    match document.get("openapi").and_then(Value::as_str) {
        Some(version) if !version.starts_with("3.0") && !version.starts_with('2') => {
            Dialect::Draft2020
        }
        _ => Dialect::Draft4,
    }
}

/// A YAML document as JSON: each key as text, as a spec's status codes are
/// meant (`200:` is the key `"200"`), and a tag left off its value.
pub(crate) fn json_of(yaml: &serde_yaml_ng::Value) -> Value {
    use serde_yaml_ng::Value as Yaml;
    match yaml {
        Yaml::Null => Value::Null,
        Yaml::Bool(b) => Value::Bool(*b),
        Yaml::Number(n) => {
            if let Some(i) = n.as_i64() {
                Value::from(i)
            } else if let Some(u) = n.as_u64() {
                Value::from(u)
            } else {
                n.as_f64()
                    .and_then(Number::from_f64)
                    .map_or(Value::Null, Value::Number)
            }
        }
        Yaml::String(s) => Value::String(s.clone()),
        Yaml::Sequence(items) => Value::Array(items.iter().map(json_of).collect()),
        Yaml::Mapping(map) => Value::Object(
            map.iter()
                .filter_map(|(k, v)| Some((key_of(k)?, json_of(v))))
                .collect(),
        ),
        Yaml::Tagged(tagged) => json_of(&tagged.value),
    }
}

fn key_of(key: &serde_yaml_ng::Value) -> Option<String> {
    use serde_yaml_ng::Value as Yaml;
    match key {
        Yaml::String(s) => Some(s.clone()),
        Yaml::Number(n) => Some(n.to_string()),
        Yaml::Bool(b) => Some(b.to_string()),
        Yaml::Null => Some("null".to_owned()),
        Yaml::Tagged(tagged) => key_of(&tagged.value),
        Yaml::Sequence(_) | Yaml::Mapping(_) => None,
    }
}

/// The bodies of the operation at `pointer` in a spec, whose path item is at
/// `item`: in Swagger 2.0, its `in: body` parameter's schema, in each of
/// its `consumes`, and each response's schema, in each of its `produces`;
/// in OpenAPI 3, its `requestBody`'s and each response's `content`.
pub(crate) fn bodies(document: &Value, item: &str, pointer: &str) -> Vec<Body> {
    let Some(operation) = document.pointer(pointer) else {
        return Vec::new();
    };
    let mut out = Vec::new();
    if document.get("swagger").is_some() {
        let media = |key: &str| -> Vec<String> {
            let listed = operation
                .get(key)
                .or_else(|| document.get(key))
                .and_then(Value::as_array)
                .map(|types| {
                    types
                        .iter()
                        .filter_map(Value::as_str)
                        .filter(|t| is_json(t))
                        .map(str::to_owned)
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default();
            if listed.is_empty() {
                vec!["application/json".to_owned()]
            } else {
                listed
            }
        };
        let parameters = [
            format!("{item}/parameters"),
            format!("{pointer}/parameters"),
        ];
        let body = parameters
            .iter()
            .rev()
            .flat_map(|list| {
                let count = document
                    .pointer(list)
                    .and_then(Value::as_array)
                    .map_or(0, Vec::len);
                (0..count).map(move |i| format!("{list}/{i}"))
            })
            .filter_map(|p| follow(document, p))
            .find(|p| {
                document
                    .pointer(p)
                    .and_then(|v| v.get("in"))
                    .and_then(Value::as_str)
                    == Some("body")
            });
        if let Some(body) = body {
            for media_type in media("consumes") {
                out.push(Body {
                    status: None,
                    media_type: Some(media_type),
                    schema: schema_at(document, &format!("{body}/schema")),
                });
            }
        }
        for (status, response) in responses(document, pointer) {
            let Some(response) = response else {
                out.push(elsewhere(Some(status), pointer));
                continue;
            };
            let Some(schema) = schema_at(document, &format!("{response}/schema")) else {
                continue;
            };
            for media_type in media("produces") {
                out.push(Body {
                    status: Some(status.clone()),
                    media_type: Some(media_type),
                    schema: Some(schema.clone()),
                });
            }
        }
        return out;
    }
    if operation.get("requestBody").is_some() {
        match follow(document, format!("{pointer}/requestBody")) {
            Some(body) => contents(document, &body, None, &mut out),
            None => out.push(elsewhere(None, &format!("{pointer}/requestBody"))),
        }
    }
    for (status, response) in responses(document, pointer) {
        match response {
            Some(response) => contents(document, &response, Some(&status), &mut out),
            None => out.push(elsewhere(Some(status), pointer)),
        }
    }
    out
}

/// The statuses of the responses of the operation at `pointer`.
pub(crate) fn statuses(document: &Value, pointer: &str) -> Vec<String> {
    responses(document, pointer)
        .into_iter()
        .map(|(s, _)| s)
        .collect()
}

/// A body that's a `$ref` into another file.
fn elsewhere(status: Option<String>, pointer: &str) -> Body {
    Body {
        status,
        media_type: None,
        schema: Some(pointer.to_owned()),
    }
}

/// Each response of the operation at `pointer`, by its status, with the
/// pointer to it after following `$ref`s; `None` for one in another file.
fn responses(document: &Value, pointer: &str) -> Vec<(String, Option<String>)> {
    let Some(Value::Object(responses)) = document.pointer(&format!("{pointer}/responses")) else {
        return Vec::new();
    };
    responses
        .keys()
        .map(|status| {
            let at = format!("{pointer}/responses/{}", token(status));
            (status.clone(), follow(document, at))
        })
        .collect()
}

/// The bodies in the `content` of the request body or response at `at`.
fn contents(document: &Value, at: &str, status: Option<&str>, out: &mut Vec<Body>) {
    let Some(Value::Object(content)) = document.pointer(&format!("{at}/content")) else {
        return;
    };
    for media_type in content.keys() {
        let schema = format!("{at}/content/{}/schema", token(media_type));
        out.push(Body {
            status: status.map(str::to_owned),
            media_type: Some(media_type.clone()),
            schema: schema_at(document, &schema),
        });
    }
}

/// `pointer`, when there's a value there.
fn schema_at(document: &Value, pointer: &str) -> Option<String> {
    document.pointer(pointer).map(|_| pointer.to_owned())
}

/// The pointer `pointer` leads to after following its `$ref`s into the same
/// file; `None` when one is into another file, or leads nowhere.
fn follow(document: &Value, pointer: String) -> Option<String> {
    let mut pointer = pointer;
    let mut seen = BTreeSet::new();
    loop {
        let value = document.pointer(&pointer)?;
        let Some(reference) = value.get("$ref").and_then(Value::as_str) else {
            return Some(pointer);
        };
        if !seen.insert(pointer.clone()) {
            return None;
        }
        pointer = decode(reference.strip_prefix('#')?);
    }
}

/// The spec as JSON Schema of `dialect`, for bodies in `direction`.
pub fn translated(document: &Value, dialect: Dialect, direction: Direction) -> Value {
    let mut out = document.clone();
    let hidden = match direction {
        Direction::Request => "readOnly",
        Direction::Response => "writeOnly",
    };
    let mut unrequired = Vec::new();
    walk(document, "", false, &mut |pointer, schema| {
        if let Some(names) = not_required(document, schema, hidden) {
            unrequired.push((pointer.to_owned(), names));
        }
    });
    for (pointer, names) in unrequired {
        if let Some(Value::Array(required)) = out
            .pointer_mut(&pointer)
            .and_then(|schema| schema.get_mut("required"))
        {
            required.retain(|name| name.as_str().is_none_or(|n| !names.contains(n)));
        }
    }
    if dialect == Dialect::Draft4 {
        to_draft_4(&mut out, false);
    }
    encode_references(&mut out);
    out
}

/// The names `schema` requires whose property is marked `hidden`, following
/// the property's `$ref`s into the same file.
fn not_required(
    document: &Value,
    schema: &Map<String, Value>,
    hidden: &str,
) -> Option<BTreeSet<String>> {
    let required = schema.get("required")?.as_array()?;
    let properties = schema.get("properties")?.as_object()?;
    let names: BTreeSet<String> = required
        .iter()
        .filter_map(Value::as_str)
        .filter(|name| {
            let mut property = properties.get(*name);
            let mut hops = 0;
            while let Some(reference) = property
                .and_then(|p| p.get("$ref"))
                .and_then(Value::as_str)
                .and_then(|r| r.strip_prefix('#'))
            {
                hops += 1;
                if hops > 16 {
                    break;
                }
                property = document.pointer(&decode(reference));
            }
            property
                .and_then(|p| p.get(hidden))
                .and_then(Value::as_bool)
                == Some(true)
        })
        .map(str::to_owned)
        .collect();
    (!names.is_empty()).then_some(names)
}

/// Keys whose value is a map of names to schemas or other objects: its keys
/// are names, never keywords.
const NAMES: &[&str] = &[
    "properties",
    "patternProperties",
    "definitions",
    "$defs",
    "dependentSchemas",
    "schemas",
    "paths",
    "responses",
    "content",
    "parameters",
    "requestBodies",
    "headers",
    "links",
    "callbacks",
    "encoding",
    "securitySchemes",
    "webhooks",
    "pathItems",
];

/// Keys whose value is data, never a schema.
fn is_data(key: &str) -> bool {
    matches!(key, "example" | "examples" | "enum" | "default" | "const") || key.starts_with("x-")
}

/// Calls `f` on every object that may be a schema, with its pointer: every
/// object outside data and not itself a map of names.
fn walk(value: &Value, pointer: &str, names: bool, f: &mut impl FnMut(&str, &Map<String, Value>)) {
    match value {
        Value::Object(map) => {
            if !names {
                f(pointer, map);
            }
            for (key, child) in map {
                if !names && is_data(key) {
                    continue;
                }
                let at = format!("{pointer}/{}", token(key));
                walk(child, &at, !names && NAMES.contains(&key.as_str()), f);
            }
        }
        Value::Array(items) => {
            for (i, item) in items.iter().enumerate() {
                walk(item, &format!("{pointer}/{i}"), false, f);
            }
        }
        _ => {}
    }
}

/// OpenAPI 3.0's and Swagger 2.0's schema keywords that draft 4 doesn't
/// have, translated in place.
fn to_draft_4(value: &mut Value, names: bool) {
    match value {
        Value::Object(map) => {
            if !names {
                let nullable = ["nullable", "x-nullable"]
                    .iter()
                    .filter_map(|key| map.remove(*key))
                    .any(|v| v == Value::Bool(true));
                if map.contains_key("$ref") {
                    map.retain(|key, _| key == "$ref");
                    return;
                }
                if map.get("type").and_then(Value::as_str) == Some("file") {
                    map.remove("type");
                }
                if nullable {
                    make_nullable(map);
                }
            }
            for (key, child) in map.iter_mut() {
                if !names && is_data(key) {
                    continue;
                }
                to_draft_4(child, !names && NAMES.contains(&key.as_str()));
            }
        }
        Value::Array(items) => items.iter_mut().for_each(|item| to_draft_4(item, false)),
        _ => {}
    }
}

/// Lets a schema's value be `null` too: in its `type` and its `enum`, or,
/// with no `type`, as an alternative.
fn make_nullable(map: &mut Map<String, Value>) {
    if let Some(Value::Array(values)) = map.get_mut("enum")
        && !values.contains(&Value::Null)
    {
        values.push(Value::Null);
    }
    match map.get_mut("type") {
        Some(Value::String(kind)) => {
            let kind = std::mem::take(kind);
            map.insert(
                "type".to_owned(),
                Value::from(vec![kind, "null".to_owned()]),
            );
        }
        Some(Value::Array(kinds)) => {
            if !kinds.iter().any(|k| k == "null") {
                kinds.push(Value::from("null"));
            }
        }
        _ => {
            let rest = Value::Object(std::mem::take(map));
            let null = Value::Object(Map::from_iter([("type".to_owned(), Value::from("null"))]));
            map.insert("anyOf".to_owned(), Value::Array(vec![rest, null]));
        }
    }
}

/// Each `$ref` into the same file with its fragment percent-encoded.
fn encode_references(value: &mut Value) {
    match value {
        Value::Object(map) => {
            if let Some(Value::String(reference)) = map.get_mut("$ref")
                && let Some(fragment) = reference.strip_prefix('#')
            {
                *reference = format!("#{}", encode(&decode(fragment)));
            }
            map.values_mut().for_each(encode_references);
        }
        Value::Array(items) => items.iter_mut().for_each(encode_references),
        _ => {}
    }
}
