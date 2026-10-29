//! Checks a code block marked as an example of an operation's request or
//! response against that body's schema, each disagreement at its place in
//! the block (decision #920).
//!
//! The marker's link is checked as a link is, so an operation the spec
//! doesn't have is `link-operation-missing`, and a spec that can't be read is
//! `api-spec-unreadable`; neither is said again here. The rest is:
//!
//! - `api-example-not-json`, a marked block that isn't a `json` block, or whose
//!   code isn't JSON, at where reading it stopped;
//! - `api-example-body-missing`, an operation with no body the marker names, or
//!   a marker whose link isn't to an operation;
//! - `api-example-schema-mismatch`, one for each disagreement;
//! - `api-example-schema-unsupported`, advice, for a schema that can't all be
//!   checked: a pattern the validator can't read, left out so the rest is
//!   still checked, or a `$ref` into another file, which isn't followed.

use std::collections::BTreeMap;

use ascribe_core::{Issue, Location, diagnostics};

use super::Operation;
use super::schemas::{Body, Direction, dialect, is_json, translated};
use crate::index::{ExampleMarker, ExampleOf, FileIndex};
use crate::json_schema::{self, Checked};
use crate::project::{Project, Resolution};

/// The problems with the marked examples of one file, given its references'
/// resolutions.
pub fn problems(project: &Project, index: &FileIndex, resolutions: &[Resolution]) -> Vec<Issue> {
    let mut translations: BTreeMap<(String, Direction), serde_json::Value> = BTreeMap::new();
    let mut out = Vec::new();
    for marker in &index.examples {
        let Some(resolution) = resolutions.get(marker.reference) else {
            continue;
        };
        let at = Location::new(index.file, marker.span);
        let link = index
            .references
            .get(marker.reference)
            .map(|r| r.destination.clone())
            .unwrap_or_default();
        let (api, id) = match resolution {
            Resolution::Operation { api, id } => (api, id),
            Resolution::External | Resolution::Source { .. } | Resolution::Asset { .. } => {
                out.push(
                    Issue::new(diagnostics::API_EXAMPLE_BODY_MISSING, at)
                        .with_variant("not-operation")
                        .with_arg("attribute", marker.of.attribute())
                        .with_arg("link", link),
                );
                continue;
            }
            // The link's own problem says what's wrong.
            _ => continue,
        };
        let (Some(spec), Some(declared)) = (project.apis().spec(api), project.model().api(api))
        else {
            continue;
        };
        let Some(named) = spec.named(id) else {
            continue;
        };
        let operation = named.operation();
        let name = operation
            .operation_id
            .clone()
            .unwrap_or_else(|| operation.method_and_path());
        let body_name = body_name(marker, &name);
        let its = match (marker.of, &marker.status) {
            (ExampleOf::Response, Some(status)) => format!("its `{status}` response"),
            (ExampleOf::Response, None) => "its response".to_owned(),
            (ExampleOf::Request, _) => "its request".to_owned(),
        };
        let issue = |slug| {
            Issue::new(slug, at)
                .with_arg("body", body_name.clone())
                .with_arg("its", its.clone())
                .with_arg("operation", name.clone())
                .with_arg("path", declared.spec.clone())
        };
        if !marker.language.eq_ignore_ascii_case("json") {
            out.push(
                issue(diagnostics::API_EXAMPLE_NOT_JSON)
                    .with_variant("language")
                    .with_arg("language", marker.language.clone()),
            );
            continue;
        }
        let body = match body_of(marker, operation) {
            Ok(body) => body,
            Err(missing) => {
                out.push(missing.issue(issue(diagnostics::API_EXAMPLE_BODY_MISSING), &link));
                continue;
            }
        };
        let Some(schema) = &body.schema else {
            continue;
        };
        let direction = match marker.of {
            ExampleOf::Request => Direction::Request,
            ExampleOf::Response => Direction::Response,
        };
        let dialect = dialect(&spec.document);
        let document = translations
            .entry((api.clone(), direction))
            .or_insert_with(|| translated(&spec.document, dialect, direction));
        match json_schema::check(document, dialect, schema, &marker.code) {
            Checked::Done {
                disagreements,
                patterns_left_out,
            } => {
                for pattern in patterns_left_out {
                    out.push(
                        issue(diagnostics::API_EXAMPLE_SCHEMA_UNSUPPORTED)
                            .with_arg("pattern", pattern),
                    );
                }
                for disagreement in disagreements {
                    let place = Location::new(index.file, marker.span_of(disagreement.at));
                    out.push(
                        Issue::new(diagnostics::API_EXAMPLE_SCHEMA_MISMATCH, place)
                            .with_arg("body", body_name.clone())
                            .with_arg("found", disagreement.found)
                            .with_arg("expected", disagreement.expected),
                    );
                }
            }
            Checked::NotJson {
                at: stopped,
                reason,
            } => {
                let place = Location::new(index.file, marker.span_of(stopped..stopped + 1));
                out.push(
                    Issue::new(diagnostics::API_EXAMPLE_NOT_JSON, place)
                        .with_arg("body", body_name.clone())
                        .with_arg("reason", reason),
                );
            }
            Checked::Elsewhere { reference } => out.push(
                issue(diagnostics::API_EXAMPLE_SCHEMA_UNSUPPORTED)
                    .with_variant("elsewhere")
                    .with_arg("reference", reference),
            ),
            Checked::Unusable(reason) => out.push(
                issue(diagnostics::API_EXAMPLE_SCHEMA_UNSUPPORTED)
                    .with_variant("unusable")
                    .with_arg("reason", reason),
            ),
        }
    }
    out
}

/// How messages name the body a marker names: "the request body of
/// `createUser`", "the `201` response of `createUser`".
fn body_name(marker: &ExampleMarker, operation: &str) -> String {
    match (marker.of, &marker.status) {
        (ExampleOf::Request, _) => format!("the request body of `{operation}`"),
        (ExampleOf::Response, Some(status)) => {
            format!("the `{status}` response of `{operation}`")
        }
        (ExampleOf::Response, None) => format!("the response of `{operation}`"),
    }
}

/// Why a marker names no body of its operation.
enum NoBody {
    /// It's a request, and the operation takes no request body.
    Request,
    /// It's a response with no status after its link.
    NoStatus,
    /// The operation has no response with its status.
    Response(String),
    /// The body has no content.
    Empty,
    /// The body has no content of the media type the marker names; the ones
    /// it has.
    MediaType(String, Vec<String>),
    /// It names no media type, and the body has no JSON one; the ones it has.
    NoJson(Vec<String>),
    /// It names no media type, and the body has more than one JSON one.
    Ambiguous(Vec<String>),
}

impl NoBody {
    fn issue(self, issue: Issue, link: &str) -> Issue {
        let listed = |types: Vec<String>| {
            types
                .iter()
                .map(|t| format!("`{t}`"))
                .collect::<Vec<_>>()
                .join(", ")
        };
        match self {
            NoBody::Request => issue,
            NoBody::NoStatus => issue.with_variant("no-status").with_arg("link", link),
            NoBody::Response(status) => issue.with_variant("response").with_arg("status", status),
            NoBody::Empty => issue.with_variant("empty"),
            NoBody::MediaType(wanted, types) => issue
                .with_variant("media-type")
                .with_arg("media", wanted)
                .with_arg("listed", listed(types)),
            NoBody::NoJson(types) => issue
                .with_variant("no-json")
                .with_arg("listed", listed(types)),
            NoBody::Ambiguous(types) => issue
                .with_variant("ambiguous")
                .with_arg("listed", listed(types)),
        }
    }
}

/// The body of `operation` a marker names.
fn body_of<'o>(marker: &ExampleMarker, operation: &'o Operation) -> Result<&'o Body, NoBody> {
    let status = match marker.of {
        ExampleOf::Request => None,
        ExampleOf::Response => Some(marker.status.as_deref().ok_or(NoBody::NoStatus)?),
    };
    let bodies: Vec<&Body> = operation
        .bodies
        .iter()
        .filter(|b| match (status, &b.status) {
            (None, None) => true,
            (Some(wanted), Some(status)) => wanted.eq_ignore_ascii_case(status),
            _ => false,
        })
        .collect();
    if bodies.is_empty() {
        return Err(match status {
            None => NoBody::Request,
            Some(status) if has_response(operation, status) => NoBody::Empty,
            Some(status) => NoBody::Response(status.to_owned()),
        });
    }
    // A body in another file: its schema says so when it's checked.
    if let Some(elsewhere) = bodies.iter().find(|b| b.media_type.is_none()) {
        return Ok(elsewhere);
    }
    let types = |bodies: &[&Body]| -> Vec<String> {
        bodies.iter().filter_map(|b| b.media_type.clone()).collect()
    };
    if let Some(wanted) = &marker.media_type {
        return bodies
            .iter()
            .find(|b| {
                b.media_type
                    .as_deref()
                    .is_some_and(|t| t.eq_ignore_ascii_case(wanted))
            })
            .copied()
            .ok_or_else(|| NoBody::MediaType(wanted.clone(), types(&bodies)));
    }
    let json: Vec<&Body> = bodies
        .iter()
        .filter(|b| b.media_type.as_deref().is_some_and(is_json))
        .copied()
        .collect();
    match json.as_slice() {
        [] => Err(NoBody::NoJson(types(&bodies))),
        [one] => Ok(one),
        // Swagger 2.0 gives every media type the one schema.
        [first, rest @ ..] if rest.iter().all(|b| b.schema == first.schema) => Ok(first),
        _ => Err(NoBody::Ambiguous(types(&json))),
    }
}

/// Whether the operation has a response with this status, with a body or
/// not.
fn has_response(operation: &Operation, status: &str) -> bool {
    operation
        .responses
        .iter()
        .any(|s| s.eq_ignore_ascii_case(status))
}
