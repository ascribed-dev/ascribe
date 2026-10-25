//! `@operation` in each output (SPEC §4.13): an `<ascribe-operation>` element
//! holding a link to the operation's address in the site output, the same
//! line of text in the plain output, and a block of type `operation` in the
//! JSON output; and nothing for one whose operation the spec doesn't have.

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
      summary: Get a user
      tags: [Users]
    delete:
      summary: >-
        Delete a user
        & their keys
      deprecated: true
"#;

const PAGE: &str = "@operation: ../api/openapi.yaml#getUser\n\n\
    @operation: ../api/openapi.yaml#delete/users/{id}\n\n\
    @operation: ../api/openapi.yaml#gone\n\n\
    The end.\n";

fn project(consumer: &str) -> ascribe_resolve::Project {
    let model = format!(
        "spec = \"0.1\"\n\n[project]\ncontent-root = \"docs\"\n\n[consumer]\n{consumer}\n\n\
         [apis.public]\nspec = \"api/openapi.yaml\"\n\
         reference = {{ renderer = \"redoc\", at = \"/api/\" }}\n"
    );
    memory_project_with_files(
        &model,
        &[("guide.md", &page(PAGE))],
        &[("api/openapi.yaml", SPEC)],
    )
}

#[test]
fn operation_the_site_output_writes_its_element_holding_the_link() {
    let out = site(&project("base-path = \"/docs/\""), "site", "guide.md");
    let body = out.split("---\n").last().expect("a body");
    assert_eq!(
        body.trim(),
        "<ascribe-operation api=\"public\" method=\"get\" path=\"/users/{id}\" operation-id=\"getUser\">\n\
         <a href=\"/docs/api/#operation/getUser\">GET /users/{id}: Get a user</a>\n\
         </ascribe-operation>\n\n\
         <ascribe-operation api=\"public\" method=\"delete\" path=\"/users/{id}\" deprecated=\"\">\n\
         <a href=\"/docs/api/#paths/~1users~1%7Bid%7D/delete\">DELETE /users/{id}: Delete a user &amp; their keys</a> \
         <span class=\"ascribe-operation-deprecated\">(deprecated)</span>\n\
         </ascribe-operation>\n\n\
         The end.",
        "{out}"
    );
}

#[test]
fn operation_the_plain_output_writes_the_line_with_the_address_absolute() {
    let out = plain(
        &project("site = \"https://docs.example.com\"\nbase-path = \"/docs/\"\nagents = false"),
        "site",
        "guide.md",
    );
    assert!(
        out.contains(
            "[GET /users/{id}: Get a user](https://docs.example.com/docs/api/#operation/getUser)\n\n\
             [DELETE /users/{id}: Delete a user & their keys](https://docs.example.com/docs/api/#paths/~1users~1%7Bid%7D/delete) (deprecated)\n\n\
             The end."
        ),
        "{out}"
    );
}

#[test]
fn operation_the_json_output_writes_a_block_with_the_operation() {
    let files = emit_build(
        Path::new("/nowhere"),
        &project("base-path = \"/docs/\""),
        "site",
        &JsonEmitter,
    );
    let doc: serde_json::Value =
        serde_json::from_str(&files["guide.json"]).expect("the page's JSON");
    let blocks = doc["blocks"].as_array().expect("blocks");
    let block = |n: usize| {
        let mut b = blocks[n].clone();
        b.as_object_mut().expect("an object").remove("source");
        b
    };
    assert_eq!(
        block(0),
        serde_json::json!({
            "type": "operation",
            "api": "public",
            "operationId": "getUser",
            "method": "get",
            "path": "/users/{id}",
            "summary": "Get a user",
            "deprecated": false,
            "address": "/docs/api/#operation/getUser",
        })
    );
    assert_eq!(block(1)["type"], "operation");
    assert_eq!(block(1)["deprecated"], true);
    assert!(block(1).get("operationId").is_none(), "{}", block(1));
    // One that names no operation the spec has stays its directive.
    assert_eq!(block(2)["type"], "directive");
    assert_eq!(block(2)["name"], "operation");
}
