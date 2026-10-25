//! The specs `[apis.<name>]` declares, read loosely, and the links to their
//! operations: each operation of Swagger 2.0 and OpenAPI 3.0 to 3.2, in YAML
//! and JSON, a spec outside the project's folder, and what a link to one
//! resolves to.

#![allow(clippy::expect_used, clippy::panic)]

use std::sync::Arc;

use ascribe_core::{Applicability, FileId, RelPath};
use ascribe_resolve::openapi::{Named, Operation, Unreadable, read_spec};
use ascribe_resolve::{DiskFs, Layout, MemoryFs, Project, Resolution};

const SWAGGER_2: &str = r#"
swagger: "2.0"
info: { title: Users, version: "1" }
paths:
  /users:
    get:
      operationId: getUsers
      summary: List users
      tags: [Users]
    post:
      operationId: createUser
      tags: [Users, Admin]
  /users/{id}:
    parameters: []
    get:
      operationId: getUser
      deprecated: true
    delete:
      summary: No id
"#;

const OPENAPI_3_0: &str = r#"{
  "openapi": "3.0.3",
  "info": { "title": "Billing", "version": "1" },
  "paths": {
    "/invoices": {
      "get": { "operationId": "activity/list invoices", "tags": ["Billing & Invoices"], "summary": "List" }
    }
  }
}"#;

const OPENAPI_3_1: &str = r#"
openapi: 3.1.0
info: { title: Health, version: "1" }
paths:
  /health:
    get:
      operationId: Health.Check_v2
      tags: [Café]
      responses:
        "200": { description: OK, content: { application/json: { schema: { type: [string, "null"] } } } }
webhooks:
  ping:
    post: { operationId: ping }
"#;

const OPENAPI_3_2: &str = r##"
openapi: 3.2.0
info: { title: Search, version: "1" }
paths:
  /search:
    query:
      operationId: searchUsers
    additionalOperations:
      PURGE:
        operationId: purgeSearch
  /shared:
    $ref: "./shared.yaml"
  /find:
    $ref: "#/paths/~1search"
"##;

fn op(method: &str, path: &str, id: Option<&str>) -> (String, String, Option<String>) {
    (method.into(), path.into(), id.map(str::to_owned))
}

fn ops(operations: &[Operation]) -> Vec<(String, String, Option<String>)> {
    operations
        .iter()
        .map(|o| (o.method.clone(), o.path.clone(), o.operation_id.clone()))
        .collect()
}

#[test]
fn openapi_swagger_2_reads_each_operation_with_its_summary_tag_and_deprecation() {
    let spec = read_spec(SWAGGER_2, false).expect("reads");
    assert_eq!(
        ops(&spec.operations),
        [
            op("get", "/users", Some("getUsers")),
            op("post", "/users", Some("createUser")),
            op("get", "/users/{id}", Some("getUser")),
            op("delete", "/users/{id}", None),
        ]
    );
    let get_users = spec.operation("getUsers").expect("getUsers");
    assert_eq!(get_users.summary.as_deref(), Some("List users"));
    assert_eq!(get_users.tag.as_deref(), Some("Users"));
    assert!(!get_users.deprecated);
    assert!(spec.operation("getUser").expect("getUser").deprecated);
    assert_eq!(
        spec.operation("createUser").and_then(|o| o.tag.as_deref()),
        Some("Users")
    );
    assert!(spec.unread.is_empty());
}

#[test]
fn openapi_3_0_reads_as_json() {
    let spec = read_spec(OPENAPI_3_0, true).expect("reads");
    let list = spec.operation("activity/list invoices").expect("the id");
    assert_eq!(list.tag.as_deref(), Some("Billing & Invoices"));
    // JSON is YAML too, read the slow way.
    assert_eq!(read_spec(OPENAPI_3_0, false), Ok(spec));
}

#[test]
fn openapi_3_1_reads_paths_and_ignores_the_rest() {
    let spec = read_spec(OPENAPI_3_1, false).expect("reads");
    assert_eq!(
        ops(&spec.operations),
        [op("get", "/health", Some("Health.Check_v2"))]
    );
}

#[test]
fn openapi_3_2_reads_query_and_other_methods_and_says_what_it_skipped() {
    let spec = read_spec(OPENAPI_3_2, false).expect("reads");
    assert_eq!(
        ops(&spec.operations),
        [
            op("query", "/search", Some("searchUsers")),
            op("purge", "/search", Some("purgeSearch")),
        ]
    );
    // `/shared` is in another file; `/find` is `/search` again.
    assert_eq!(spec.unread.len(), 1, "{:?}", spec.unread);
}

#[test]
fn openapi_a_document_that_isnt_yaml_is_an_error_and_one_without_paths_is_empty() {
    assert!(read_spec("openapi: [3.1\n", false).is_err());
    assert!(read_spec("{", true).is_err());
    assert!(
        read_spec("openapi: 3.1.0\n", false)
            .expect("reads")
            .operations
            .is_empty()
    );
    assert!(
        read_spec("just text\n", false)
            .expect("reads")
            .operations
            .is_empty()
    );
}

const MODEL: &str = r#"
spec = "0.1"

[project]
content-root = "content"

[apis.public]
spec = "api/openapi.yaml"
reference = { renderer = "redoc", at = "/api/" }
"#;

fn project(model: &str, files: &[(&str, &str)]) -> Project {
    let model = ascribe_model::load_str(model, FileId::new(0)).expect("the model");
    let layout = Layout::from_model(&model);
    let mut fs = MemoryFs::new(&layout);
    for (path, text) in files {
        fs = match path.strip_prefix("content/") {
            Some(content) => fs.with_source(content, text),
            None => fs.with_file(path, text),
        };
    }
    Project::load(Arc::new(model), layout, &fs)
}

fn page(links: &str) -> String {
    format!("---\ntitle: Guide\n---\n\n{links}\n")
}

fn resolutions(project: &Project) -> Vec<Resolution> {
    project
        .resolutions(&RelPath::parse("guide.md").expect("a path"))
        .to_vec()
}

#[test]
fn openapi_a_link_to_a_declared_spec_with_a_fragment_is_to_an_operation() {
    let project = project(
        MODEL,
        &[
            ("api/openapi.yaml", SWAGGER_2),
            ("content/data.yaml", "a: 1\n"),
            (
                "content/guide.md",
                &page(
                    "[Get](../api/openapi.yaml#getUser) [Encoded](../api/openapi.yaml#activity%2Flist%20invoices) [File](../api/openapi.yaml) [Data](data.yaml#a)",
                ),
            ),
        ],
    );
    assert_eq!(
        resolutions(&project),
        [
            Resolution::Operation {
                api: "public".into(),
                id: "getUser".into()
            },
            Resolution::Operation {
                api: "public".into(),
                id: "activity/list invoices".into()
            },
            Resolution::Asset {
                path: RelPath::parse("../api/openapi.yaml").expect("a path"),
                fragment: None
            },
            Resolution::Asset {
                path: RelPath::parse("data.yaml").expect("a path"),
                fragment: Some("a".into())
            },
        ]
    );
    let spec = project.apis().spec("public").expect("read");
    assert_eq!(spec.operations.len(), 4);
}

#[test]
fn openapi_a_missing_operation_is_a_problem_with_the_closest_ids() {
    let project = project(
        MODEL,
        &[
            ("api/openapi.yaml", SWAGGER_2),
            (
                "content/guide.md",
                &page("[Get](../api/openapi.yaml#getUserz) [Ok](../api/openapi.yaml#getUsers)"),
            ),
        ],
    );
    let problems = project.problems(&RelPath::parse("guide.md").expect("a path"));
    assert_eq!(problems.len(), 1, "{problems:?}");
    let issue = &problems[0];
    assert_eq!(issue.slug.as_str(), "link-operation-missing");
    assert_eq!(issue.variant, Some("closest"));
    assert_eq!(issue.arg("closest"), Some("getUser"));
    assert_eq!(issue.arg("path"), Some("api/openapi.yaml"));
    let hrefs: Vec<&str> = issue
        .fixes
        .iter()
        .map(|f| f.edits[0].new_text.as_str())
        .collect();
    assert_eq!(hrefs[0], "../api/openapi.yaml#getUser");
}

#[test]
fn openapi_a_spec_outside_the_folder_is_read_and_linked_only_to_an_operation() {
    let dir = tempfile::tempdir().expect("a folder");
    let root = dir.path();
    std::fs::create_dir_all(root.join("api")).expect("api");
    std::fs::create_dir_all(root.join("docs/content")).expect("docs");
    std::fs::write(root.join("api/openapi.yaml"), SWAGGER_2).expect("spec");
    std::fs::write(
        root.join("docs/content/guide.md"),
        page("[Get](../../api/openapi.yaml#getUser) [File](../../api/openapi.yaml)"),
    )
    .expect("page");
    let text = MODEL.replace("api/openapi.yaml", "../api/openapi.yaml");
    let model = ascribe_model::load_str(&text, FileId::new(0)).expect("the model");
    let layout = Layout::from_model(&model);
    let fs = DiskFs::new(root.join("docs"), &layout);
    let project = Project::load(Arc::new(model), layout, &fs);
    assert_eq!(
        project.apis().spec("public").map(|s| s.operations.len()),
        Some(4)
    );
    assert_eq!(
        resolutions(&project),
        [
            Resolution::Operation {
                api: "public".into(),
                id: "getUser".into()
            },
            Resolution::SpecOutside {
                api: "public".into()
            },
        ]
    );
}

#[test]
fn openapi_a_spec_that_cant_be_read_is_a_problem_of_the_content_model() {
    let unread = project(MODEL, &[("content/guide.md", &page(""))]);
    assert_eq!(
        unread.apis().unreadable("public"),
        Some(&Unreadable::Missing)
    );
    let problems = unread.apis().problems(unread.model(), FileId::new(0));
    assert_eq!(problems[0].slug.as_str(), "api-spec-unreadable");
    assert_eq!(problems[0].variant, Some("missing"));

    let broken = project(MODEL, &[("api/openapi.yaml", "paths: [\n")]);
    let problems = broken.apis().problems(broken.model(), FileId::new(0));
    assert_eq!(problems[0].slug.as_str(), "api-spec-unreadable");
    assert_eq!(problems[0].variant, None);

    let empty = project(MODEL, &[("api/openapi.yaml", "openapi: 3.1.0\n")]);
    let problems = empty.apis().problems(empty.model(), FileId::new(0));
    assert_eq!(problems[0].slug.as_str(), "api-spec-empty");

    let case = project(MODEL, &[("api/OpenAPI.yaml", SWAGGER_2)]);
    assert!(matches!(
        case.apis().unreadable("public"),
        Some(Unreadable::Case(_))
    ));
}

const BY_PATH: &str = r#"
openapi: 3.1.0
info: { title: Users, version: "1" }
paths:
  /users:
    get:
      summary: List users
  /users/{id}:
    get:
      operationId: getUser
    delete:
      summary: Delete a user
  /files/~home:
    get:
      summary: The home folder
  /orgs:
    get:
      operationId: get/users
"#;

#[test]
fn openapi_a_fragment_names_an_operation_by_its_method_and_path_and_an_id_wins() {
    let spec = read_spec(BY_PATH, false).expect("reads");
    let named = |fragment: &str| {
        spec.named(fragment).map(|n| match n {
            Named::Id(o) => format!("id {} {}", o.method, o.path),
            Named::MethodAndPath(o) => format!("path {} {}", o.method, o.path),
        })
    };
    let at = |text: &str| Some(text.to_owned());
    assert_eq!(named("get/users"), at("id get /orgs"));
    assert_eq!(named("delete/users/{id}"), at("path delete /users/{id}"));
    assert_eq!(named("get/users/{id}"), at("path get /users/{id}"));
    assert_eq!(named("get/files/~home"), at("path get /files/~home"));
    assert_eq!(named("getUser"), at("id get /users/{id}"));
    // The method is in lowercase, and the path is as the spec writes it.
    assert_eq!(named("DELETE/users/{id}"), None);
    assert_eq!(named("delete/users/{userId}"), None);
    assert_eq!(named("delete/users/{id}/"), None);
    assert_eq!(named("post/users"), None);
}

#[test]
fn openapi_a_link_by_method_and_path_is_to_an_operation_and_one_with_an_id_is_advice() {
    let project = project(
        MODEL,
        &[
            ("api/openapi.yaml", BY_PATH),
            (
                "content/guide.md",
                &page(
                    "[Delete](../api/openapi.yaml#delete/users/{id}) [Get](../api/openapi.yaml#get/users/{id}) [Home](../api/openapi.yaml#get/files/~home)",
                ),
            ),
        ],
    );
    assert_eq!(
        resolutions(&project)[0],
        Resolution::Operation {
            api: "public".into(),
            id: "delete/users/{id}".into()
        }
    );
    let problems = project.problems(&RelPath::parse("guide.md").expect("a path"));
    assert_eq!(problems.len(), 1, "{problems:?}");
    let issue = &problems[0];
    assert_eq!(issue.slug.as_str(), "link-operation-by-path");
    assert_eq!(issue.arg("id"), Some("getUser"));
    assert_eq!(issue.fixes.len(), 1);
    assert_eq!(issue.fixes[0].applicability, Applicability::Safe);
    assert_eq!(
        issue.fixes[0].edits[0].new_text,
        "../api/openapi.yaml#getUser"
    );
}

#[test]
fn openapi_a_link_by_method_and_path_under_a_template_is_unsupported() {
    let model = MODEL.replace(
        "{ renderer = \"redoc\", at = \"/api/\" }",
        "\"/api/#/operations/{operationId}\"",
    );
    let project = project(
        &model,
        &[
            ("api/openapi.yaml", BY_PATH),
            (
                "content/guide.md",
                &page(
                    "[Delete](../api/openapi.yaml#delete/users/{id}) [Get](../api/openapi.yaml#get/users/{id})",
                ),
            ),
        ],
    );
    let problems = project.problems(&RelPath::parse("guide.md").expect("a path"));
    let slugs: Vec<&str> = problems.iter().map(|p| p.slug.as_str()).collect();
    assert_eq!(
        slugs,
        ["link-operation-form-unsupported", "link-operation-by-path"]
    );
    assert_eq!(problems[0].arg("operation"), Some("delete/users/{id}"));
}

#[test]
fn openapi_a_missing_operation_suggests_one_with_no_id_by_its_method_and_path() {
    let project = project(
        MODEL,
        &[
            ("api/openapi.yaml", BY_PATH),
            (
                "content/guide.md",
                &page("[Delete](../api/openapi.yaml#delete/user/{id})"),
            ),
        ],
    );
    let problems = project.problems(&RelPath::parse("guide.md").expect("a path"));
    assert_eq!(problems.len(), 1, "{problems:?}");
    assert_eq!(problems[0].slug.as_str(), "link-operation-missing");
    assert_eq!(problems[0].arg("closest"), Some("delete/users/{id}"));
    assert_eq!(
        problems[0].fixes[0].edits[0].new_text,
        "../api/openapi.yaml#delete/users/{id}"
    );
}
