//! The generated Zod schema (SPEC §9.6): fixtures for `tests/zod/`, which
//! type-checks them with `tsc` and validates frontmatter with them, and a
//! snapshot of the schema of every field type.
//!
//! Run with `ASCRIBE_BLESS=1` to rewrite the fixtures in `tests/zod/generated/`
//! after a change; without it the test fails when they're out of date.

#![allow(clippy::expect_used, clippy::panic)]

mod support;

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use support::{FULL_MODEL, emit_build, load, memory_project, quill};
use tessera_emit::{SiteEmitter, zod};

fn generated_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/zod/generated")
}

/// Checks a fixture against what was generated, or rewrites it.
fn check_fixture(name: &str, actual: &str) {
    let path = generated_dir().join(name);
    if std::env::var_os("ASCRIBE_BLESS").is_some() {
        fs::write(&path, actual).expect("writes the fixture");
        return;
    }
    let expected = fs::read_to_string(&path).unwrap_or_default();
    assert_eq!(
        expected, actual,
        "tests/zod/generated/{name} is out of date: run `ASCRIBE_BLESS=1 cargo test -p tessera-emit --test zod`"
    );
}

/// The frontmatter of an emitted page, as JSON.
fn frontmatter_of(page: &str) -> serde_json::Value {
    let yaml = page
        .strip_prefix("---\n")
        .and_then(|rest| rest.split_once("\n---\n"))
        .map(|(yaml, _)| yaml)
        .expect("a page with frontmatter");
    let value: serde_yaml::Value = serde_yaml::from_str(yaml).expect("YAML");
    serde_json::to_value(value).expect("JSON")
}

#[test]
fn the_quill_schema_and_its_pages_frontmatter_are_the_fixtures() {
    let root = quill();
    let project = load(&root);
    let emitter = SiteEmitter::new(project.model());
    let out = emit_build(&root, &project, "site", &emitter);
    check_fixture("quill.schema.ts", &out["_ascribe/schema.ts"]);
    let pages: BTreeMap<String, serde_json::Value> = out
        .iter()
        .filter(|(path, _)| path.ends_with(".md"))
        .map(|(path, text)| (path.clone(), frontmatter_of(text)))
        .collect();
    assert_eq!(pages.len(), 3);
    let mut json = serde_json::to_string_pretty(&pages).expect("JSON");
    json.push('\n');
    check_fixture("quill.frontmatter.json", &json);
}

#[test]
fn the_full_models_schema_covers_every_field_type() {
    let project = memory_project(FULL_MODEL, &[]);
    let schema = zod::generate(project.model());
    insta::assert_snapshot!("zod-full-model", schema);
    check_fixture("full.schema.ts", &schema);
}

#[test]
fn the_schema_of_a_model_with_no_frontmatter_beyond_the_title() {
    let project = memory_project(
        "spec = \"0.1\"\n[project]\ncontent-root = \"docs\"\n[types.page]\ndefault = true\n[types.page.frontmatter]\ntitle = \"string\"\n",
        &[],
    );
    let schema = zod::generate(project.model());
    assert!(schema.contains("export const pageSchema = z.strictObject({\n  title: z.string(),\n"));
    assert!(schema.contains("export const schema = pageSchema;"));
}
