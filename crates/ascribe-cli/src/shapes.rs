//! The JSON that the commands and the language server write has one home: the
//! Rust types that write it. This test derives a JSON Schema from each
//! (`schemars`, through each crate's `json-schema` feature) and checks three
//! things generated from them:
//!
//! - the schemas, in `schemas/`;
//! - the TypeScript types of the packages that read the JSON, in each one's
//!   `src/shapes.ts`, which replace the types they declared by hand;
//! - the commands' schemas in the docs, `docs/content/_generated/json-*.md`,
//!   which the JSON reports contract includes.
//!
//! It fails when one is out of date; run it with `ASCRIBE_BLESS=1` to rewrite
//! them. To change what one says, change the Rust type and its doc comments.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

mod typescript;

use std::collections::BTreeMap;
use std::path::PathBuf;

use schemars::generate::SchemaSettings;
use schemars::{Schema, SchemaGenerator};
use serde_json::Value;

const BLESS: &str = "ASCRIBE_BLESS=1 cargo test -p ascribe-cli shapes";

/// A JSON document Ascribe writes.
struct Shape {
    /// Its schema's file, `schemas/<file>.schema.json`.
    file: &'static str,
    /// Its schema's title, the name of its type in TypeScript.
    title: &'static str,
    /// What writes it, for the docs.
    written_by: &'static str,
    /// Whether it's a command's output, which the docs publish.
    command: bool,
    schema: fn(&mut SchemaGenerator) -> Schema,
}

const SHAPES: &[Shape] = &[
    Shape {
        file: "check",
        title: "CheckReport",
        written_by: "`ascribe check --format json` and `ascribe build --format json`",
        command: true,
        schema: |g| g.root_schema_for::<crate::report::json::Report<'static>>(),
    },
    Shape {
        file: "diff",
        title: "DiffReport",
        written_by: "`ascribe diff --format json`",
        command: true,
        schema: |g| g.root_schema_for::<ascribe_diff::Report>(),
    },
    Shape {
        file: "drift",
        title: "DriftReport",
        written_by: "`ascribe drift --format json`",
        command: true,
        schema: |g| g.root_schema_for::<ascribe_diff::DriftReport>(),
    },
    Shape {
        file: "sources-status",
        title: "SourcesStatusReport",
        written_by: "`ascribe sources status --format json`",
        command: true,
        schema: |g| g.root_schema_for::<crate::commands::sources::StatusJson<'static>>(),
    },
    Shape {
        file: "sources-update",
        title: "SourcesUpdateReport",
        written_by: "`ascribe sources update --format json`",
        command: true,
        schema: |g| g.root_schema_for::<crate::commands::sources::UpdateJson<'static>>(),
    },
    Shape {
        file: "diff-html-data",
        title: "ReportData",
        written_by: "`ascribe diff --format html`, for the report's script",
        command: false,
        schema: ascribe_diff::html::data_schema,
    },
    Shape {
        file: "lsp-preview",
        title: "PreviewResult",
        written_by: "the language server, answering `ascribe/preview`",
        command: false,
        schema: |g| g.root_schema_for::<ascribe_lsp::PreviewResult>(),
    },
    Shape {
        file: "lsp-set-base",
        title: "SetBaseResult",
        written_by: "the language server, answering `ascribe/review/setBase`",
        command: false,
        schema: |g| g.root_schema_for::<ascribe_lsp::SetBaseResult>(),
    },
    Shape {
        file: "lsp-changes",
        title: "ChangesResult",
        written_by: "the language server, answering `ascribe/review/changes`",
        command: false,
        schema: |g| g.root_schema_for::<ascribe_lsp::ChangesResult>(),
    },
    Shape {
        file: "lsp-context",
        title: "ContextResult",
        written_by: "the language server, answering `ascribe/context`",
        command: false,
        schema: |g| g.root_schema_for::<ascribe_lsp::ContextResult>(),
    },
    Shape {
        file: "lsp-targets",
        title: "TargetsResult",
        written_by: "the language server, answering `ascribe/targets`",
        command: false,
        schema: |g| g.root_schema_for::<ascribe_lsp::TargetsResult>(),
    },
    Shape {
        file: "lsp-inventory",
        title: "InventoryResult",
        written_by: "the language server, answering `ascribe/inventory`",
        command: false,
        schema: |g| g.root_schema_for::<ascribe_lsp::InventoryResult>(),
    },
    Shape {
        file: "lsp-edit",
        title: "EditResult",
        written_by: "the language server, answering `ascribe/edit`",
        command: false,
        schema: |g| g.root_schema_for::<ascribe_lsp::EditResult>(),
    },
    Shape {
        file: "lsp-build-view",
        title: "BuildViewResult",
        written_by: "the language server, answering `ascribe/buildView`",
        command: false,
        schema: |g| g.root_schema_for::<ascribe_lsp::BuildViewResult>(),
    },
    Shape {
        file: "lsp-agent-prompt",
        title: "AgentPromptResult",
        written_by: "the language server, answering `ascribe/agentPrompt`",
        command: false,
        schema: |g| g.root_schema_for::<ascribe_lsp::AgentPromptResult>(),
    },
];

/// The packages that read Ascribe's JSON, and the shapes each reads.
const PACKAGES: &[(&str, &[&str])] = &[
    ("packages/astro", &["diff"]),
    ("packages/review", &["diff", "diff-html-data"]),
    (
        "packages/vscode",
        &[
            "lsp-preview",
            "lsp-set-base",
            "lsp-changes",
            "lsp-context",
            "lsp-targets",
            "lsp-inventory",
            "lsp-edit",
            "lsp-build-view",
            "lsp-agent-prompt",
        ],
    ),
];

fn repo() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// The schema of a shape, as JSON.
fn schema(shape: &Shape) -> Value {
    let mut generator = SchemaSettings::draft2020_12()
        .for_serialize()
        .into_generator();
    let mut schema = (shape.schema)(&mut generator).to_value();
    schema["title"] = shape.title.into();
    plain_descriptions(&mut schema);
    schema
}

/// Rewrites rustdoc's links in descriptions as code: "[`Report`]" is
/// "`Report`" outside Rust.
fn plain_descriptions(value: &mut Value) {
    match value {
        Value::Object(map) => {
            for (key, value) in map.iter_mut() {
                match value {
                    Value::String(text) if key == "description" => *text = unlinked(text),
                    _ => plain_descriptions(value),
                }
            }
        }
        Value::Array(items) => items.iter_mut().for_each(plain_descriptions),
        _ => {}
    }
}

/// `text` with each "[`name`]" not followed by "(" made "`name`".
fn unlinked(text: &str) -> String {
    let mut out = String::new();
    let mut rest = text;
    while let Some(start) = rest.find("[`") {
        let after = &rest[start + 1..];
        let Some(end) = after.find("`]") else {
            break;
        };
        let code = &after[..end + 1];
        let tail = &after[end + 2..];
        out.push_str(&rest[..start]);
        if tail.starts_with('(') || code[1..code.len() - 1].contains('`') {
            out.push('[');
            rest = after;
            continue;
        }
        out.push_str(code);
        rest = tail;
    }
    out.push_str(rest);
    out
}

fn pretty(value: &Value) -> String {
    let mut text = serde_json::to_string_pretty(value).unwrap();
    text.push('\n');
    text
}

/// Checks a generated file, or rewrites it.
fn check(path: &str, expected: &str, stale: &mut Vec<String>) {
    let full = repo().join(path);
    if std::env::var_os("ASCRIBE_BLESS").is_some() {
        std::fs::create_dir_all(full.parent().unwrap()).unwrap();
        std::fs::write(&full, expected).unwrap();
        return;
    }
    if std::fs::read_to_string(&full).unwrap_or_default() != expected {
        stale.push(path.to_owned());
    }
}

fn assert_current(stale: &[String]) {
    assert!(
        stale.is_empty(),
        "out of date: {}. Run `{BLESS}`",
        stale.join(", ")
    );
}

#[test]
fn the_schemas_are_current() {
    let mut stale = Vec::new();
    for shape in SHAPES {
        let path = format!("schemas/{}.schema.json", shape.file);
        check(&path, &pretty(&schema(shape)), &mut stale);
    }
    assert_current(&stale);
}

/// The generated `src/shapes.ts` of a package that reads `files`.
fn package_types(files: &[&str]) -> String {
    let mut defs: BTreeMap<String, Value> = BTreeMap::new();
    let mut add = |name: &str, schema: Value| {
        if let Some(other) = defs.get(name) {
            assert_eq!(
                other, &schema,
                "two shapes of one package define {name} differently"
            );
        }
        defs.insert(name.to_owned(), schema);
    };
    let mut titles = Vec::new();
    for file in files {
        let shape = SHAPES.iter().find(|s| s.file == *file).unwrap();
        titles.push(format!("{} ({})", shape.title, shape.written_by));
        let mut root = schema(shape);
        let root_defs = root
            .as_object_mut()
            .and_then(|map| map.remove("$defs"))
            .unwrap_or_default();
        if let Value::Object(root_defs) = root_defs {
            for (name, def) in root_defs {
                add(&name, def);
            }
        }
        for key in ["$schema", "title"] {
            root.as_object_mut().unwrap().remove(key);
        }
        add(shape.title, root);
    }
    let mut out = String::from(
        "// The JSON Ascribe writes that this package reads, as TypeScript types.\n\
         // Generated from the Rust types that write it, through the JSON Schemas in\n\
         // schemas/, by crates/ascribe-cli/src/shapes.rs. Change the Rust types, then\n\
         // run `ASCRIBE_BLESS=1 cargo test -p ascribe-cli shapes`. Don't edit it.\n\
         //\n",
    );
    for title in titles {
        out.push_str(&format!("// - {title}\n"));
    }
    out.push_str(&typescript::module(&defs));
    out
}

#[test]
fn each_package_has_the_generated_types() {
    let mut stale = Vec::new();
    for (package, files) in PACKAGES {
        check(
            &format!("{package}/src/shapes.ts"),
            &package_types(files),
            &mut stale,
        );
    }
    assert_current(&stale);
}

#[test]
fn the_docs_show_the_commands_schemas() {
    let mut stale = Vec::new();
    for shape in SHAPES.iter().filter(|s| s.command) {
        let fragment = format!(
            "<!-- Generated from the Rust types by crates/ascribe-cli/src/shapes.rs, \
             with schemas/{file}.schema.json. Change the types, then run `{BLESS}`. -->\n\
             [`schemas/{file}.schema.json`]({{repo}}/blob/main/schemas/{file}.schema.json):\n\n\
             ```json\n{schema}```\n",
            file = shape.file,
            schema = pretty(&schema(shape)),
        );
        check(
            &format!("docs/content/_generated/json-{}.md", shape.file),
            &fragment,
            &mut stale,
        );
    }
    assert_current(&stale);
}

/// Each property, and each type, that has no description, as `<type>.<name>`.
fn undescribed(name: &str, schema: &Value, out: &mut Vec<String>) {
    if schema.get("description").is_none() {
        out.push(name.to_owned());
    }
    let properties = schema.get("properties").and_then(Value::as_object);
    for (property, value) in properties.into_iter().flatten() {
        if value.get("description").is_none() {
            out.push(format!("{name}.{property}"));
        }
    }
}

#[test]
fn every_type_and_field_is_described() {
    for shape in SHAPES {
        let schema = schema(shape);
        let mut missing = Vec::new();
        undescribed(shape.title, &schema, &mut missing);
        let defs = schema.get("$defs").and_then(Value::as_object);
        for (name, def) in defs.into_iter().flatten() {
            undescribed(name, def, &mut missing);
        }
        assert!(
            missing.is_empty(),
            "{}: give these a doc comment: {}",
            shape.file,
            missing.join(", ")
        );
    }
}

#[test]
fn every_shape_is_read_or_published() {
    for shape in SHAPES {
        let read = PACKAGES
            .iter()
            .any(|(_, files)| files.contains(&shape.file));
        assert!(
            read || shape.command,
            "{} is neither read by a package nor a command's output",
            shape.file
        );
    }
}
