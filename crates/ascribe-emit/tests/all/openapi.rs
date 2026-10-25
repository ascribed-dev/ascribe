//! A link to an operation of a declared API (`[apis.<name>]`) in each
//! output: its address in the published reference, Redoc's or a template's,
//! under the base path unless it's absolute, absolute in the plain output with
//! a site, and an `operation` target in the JSON output.

#![allow(clippy::expect_used, clippy::panic)]

use std::path::Path;

use ascribe_emit::JsonEmitter;

use crate::support::{emit_build, memory_project_with_files, page, plain, site};

const SPEC: &str = r#"
openapi: 3.1.0
info: { title: Users, version: "1" }
paths:
  /users/{id}:
    get:
      operationId: getUser
      tags: [Users]
  /invoices:
    get:
      operationId: activity/list invoices
      tags: ["Billing & Invoices"]
"#;

fn model(consumer: &str, reference: &str) -> String {
    format!(
        "spec = \"0.1\"\n\n[project]\ncontent-root = \"docs\"\n\n[consumer]\n{consumer}\n\n[apis.public]\nspec = \"api/openapi.yaml\"\nreference = {reference}\n"
    )
}

const REDOC: &str = "{ renderer = \"redoc\", at = \"/api/\" }";

const LINKS: &str = "Call [Get a user](../api/openapi.yaml#getUser) or [list invoices](../api/openapi.yaml#activity%2Flist%20invoices).\n";

fn project(consumer: &str, reference: &str) -> ascribe_resolve::Project {
    memory_project_with_files(
        &model(consumer, reference),
        &[("guide.md", &page(LINKS))],
        &[("api/openapi.yaml", SPEC)],
    )
}

#[test]
fn openapi_redoc_addresses_an_operation_by_its_id_under_the_base_path() {
    let project = project("base-path = \"/docs/\"", REDOC);
    let out = site(&project, "site", "guide.md");
    assert!(
        out.contains("[Get a user](/docs/api/#operation/getUser)"),
        "{out}"
    );
    assert!(
        out.contains("[list invoices](/docs/api/#operation/activity%2Flist%20invoices)"),
        "{out}"
    );
}

#[test]
fn openapi_a_template_fills_in_the_id_and_the_tag_percent_encoded() {
    let project = project("base-path = \"/\"", "\"/api/#/{tag}/{operationId}\"");
    let out = site(&project, "site", "guide.md");
    assert!(out.contains("[Get a user](/api/#/Users/getUser)"), "{out}");
    assert!(
        out.contains("(/api/#/Billing%20%26%20Invoices/activity%2Flist%20invoices)"),
        "{out}"
    );
}

#[test]
fn openapi_an_absolute_reference_isnt_under_the_base_path() {
    let project = project(
        "base-path = \"/docs/\"",
        "{ renderer = \"redoc\", at = \"https://api.example.com/reference/\" }",
    );
    let out = site(&project, "site", "guide.md");
    assert!(
        out.contains("(https://api.example.com/reference/#operation/getUser)"),
        "{out}"
    );
}

#[test]
fn openapi_the_plain_output_writes_the_address_absolute_with_a_site() {
    let with_site = project(
        "site = \"https://docs.example.com\"\nbase-path = \"/docs/\"\nagents = false",
        REDOC,
    );
    let out = plain(&with_site, "site", "guide.md");
    assert!(
        out.contains("[Get a user](https://docs.example.com/docs/api/#operation/getUser)"),
        "{out}"
    );
    let without = project("base-path = \"/docs/\"", REDOC);
    let out = plain(&without, "site", "guide.md");
    assert!(
        out.contains("[Get a user](/docs/api/#operation/getUser)"),
        "{out}"
    );
}

#[test]
fn openapi_the_json_output_gives_an_operation_target() {
    let project = project("base-path = \"/docs/\"", REDOC);
    let files = emit_build(Path::new("/nowhere"), &project, "site", &JsonEmitter);
    let doc: serde_json::Value =
        serde_json::from_str(&files["guide.json"]).expect("the page's JSON");
    let links = doc["blocks"][0]["links"].as_array().expect("links");
    assert_eq!(
        links[0]["target"],
        serde_json::json!({
            "type": "operation",
            "api": "public",
            "operationId": "getUser",
            "address": "/docs/api/#operation/getUser",
        })
    );
    assert_eq!(links[1]["target"]["operationId"], "activity/list invoices");
}

#[test]
fn openapi_an_older_line_links_to_the_site_s_reference() {
    let model = "spec = \"0.1\"\n\n[project]\ncontent-root = \"docs\"\n\n\
        [dimensions.deployment]\nvalues = [\"cloud\", \"self-managed\"]\nversionless = [\"cloud\"]\n\n\
        [releases.self-managed]\nlive = [\"3.4\", \"3.5\"]\n\n\
        [consumer]\nbase-path = \"/docs/\"\n\n\
        [builds.site]\nversions = \"self-managed\"\n\n\
        [apis.public]\nspec = \"api/openapi.yaml\"\nreference = { renderer = \"redoc\", at = \"/api/\" }\n";
    let project = memory_project_with_files(
        model,
        &[("guide.md", &page(LINKS))],
        &[("api/openapi.yaml", SPEC)],
    );
    for build in ["site", "site@3.4"] {
        let out = site(&project, build, "guide.md");
        assert!(
            out.contains("[Get a user](/docs/api/#operation/getUser)"),
            "{build}: {out}"
        );
    }
}
