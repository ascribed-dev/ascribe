//! The JSON output agrees with its schema, `schemas/json-page.schema.json`
//! (the JSON output contract): every page of every build of every project
//! under `examples/` and `tests/conformance/cases/`. The schema is generated
//! from the types that write the pages, so this holds what those types say
//! they write, such as a field marked required, against what they do write.

#![allow(clippy::expect_used, clippy::panic)]

use std::path::Path;

use ascribe_emit::{EmitContext, JsonEmitter, emit_page};
use ascribe_resolve::json_schema::{Checked, Dialect, check};
use ascribe_resolve::{AstroRouter, Project};

use crate::support::{cases, example, models, repository};

/// The schema, with `$defs/pages`, a list of pages, added: validating every
/// page as one list compiles the schema once.
fn schema() -> serde_json::Value {
    // Outside FileSystem: the repository's schema, not a project file.
    let text = std::fs::read_to_string(repository().join("schemas/json-page.schema.json"))
        .expect("the schema");
    let mut schema: serde_json::Value = serde_json::from_str(&text).expect("the schema is JSON");
    schema["$defs"]["pages"] = serde_json::json!({ "type": "array", "items": { "$ref": "#" } });
    schema
}

/// Each page of the JSON output: who wrote it, and its text.
#[derive(Default)]
struct Pages {
    names: Vec<String>,
    texts: Vec<String>,
}

impl Pages {
    /// Adds every page of every build of `project`.
    fn add(&mut self, root: &Path, name: &str, project: &Project) {
        for build in &project.model().builds {
            let project = project.for_build(build);
            let project = project.as_ref();
            let router = AstroRouter::from_consumer(&project.model().consumer);
            let resolved = project.resolve_build(build, &router);
            let cx = EmitContext::new(project, root, build);
            for page in &resolved.pages {
                let Ok(emitted) = emit_page(&JsonEmitter, &cx, page) else {
                    continue;
                };
                self.names
                    .push(format!("{name}, build {}, {}", build.name, page.path));
                self.texts.push(emitted.text);
            }
        }
    }

    /// Each disagreement with the schema, one line each, naming its page.
    fn disagreements(&self) -> Vec<String> {
        let mut text = String::from("[");
        let mut starts = Vec::new();
        for (i, page) in self.texts.iter().enumerate() {
            if i > 0 {
                text.push(',');
            }
            starts.push(text.len());
            text.push_str(page);
        }
        text.push(']');
        match check(&schema(), Dialect::Draft2020, "/$defs/pages", &text) {
            Checked::Done { disagreements, .. } => disagreements
                .into_iter()
                .map(|d| {
                    let page = starts
                        .partition_point(|&s| s <= d.at.start)
                        .saturating_sub(1);
                    format!("{}: {}; {}", self.names[page], d.found, d.expected)
                })
                .collect(),
            other => panic!("{other:?}"),
        }
    }
}

#[test]
fn every_example_agrees_with_the_schema() {
    let mut found = Vec::new();
    models(&repository().join("examples"), &mut found);
    assert!(found.len() > 5, "the examples weren't found: {found:?}");
    let mut pages = Pages::default();
    for model in found {
        let (root, project) = example(&model).expect("an example loads");
        pages.add(&root, &model.display().to_string(), &project);
    }
    assert!(pages.texts.len() > 20, "{} pages", pages.texts.len());
    let problems = pages.disagreements();
    assert!(problems.is_empty(), "{}", problems.join("\n"));
}

#[test]
fn every_conformance_case_agrees_with_the_schema() {
    let conformance = repository().join("tests/conformance");
    let mut found = Vec::new();
    cases(
        &conformance.join("cases"),
        &conformance.join("_model/ascribe.toml"),
        &mut found,
    );
    assert!(
        found.len() > 100,
        "the cases weren't found: {}",
        found.len()
    );
    let mut pages = Pages::default();
    for (dir, project) in found {
        pages.add(&dir, &dir.display().to_string(), &project);
    }
    let problems = pages.disagreements();
    assert!(problems.is_empty(), "{}", problems.join("\n"));
}
