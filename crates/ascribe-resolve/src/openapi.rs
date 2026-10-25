//! The OpenAPI specs a project declares (`[apis.<name>]`), read loosely: each
//! operation's method, path, `operationId`, `summary`, first tag, and
//! `deprecated`, from `paths`, the same way in Swagger 2.0 and OpenAPI 3.0
//! to 3.2, since `paths` hasn't changed shape across them. Everything else
//! is ignored, and nothing is validated: a spec a strict reader rejects has
//! operations as good as any.
//!
//! Each spec is read once per project load, through [`FileSystem`], from the
//! project root: `read_file` reaches a spec outside the project's folder,
//! which the content model has already held to the project's repository.

use std::collections::BTreeMap;

use ascribe_core::{Applicability, Fix, Issue, Location, RelPath, Span, TextEdit, diagnostics};
use ascribe_model::{Api, ApiReference, ContentModel, edit_distance};
use serde_yaml_ng::Value;

use crate::closest::closest_fixes;
use crate::fs::{FileSystem, Probe};
use crate::index::Target;
use crate::links::encode_destination;
use crate::project::{Project, Resolution};

/// The methods a path item's operations are under, in the order the spec
/// lists them. `query` is OpenAPI 3.2's; Swagger 2.0 has no `trace`.
pub const METHODS: [&str; 9] = [
    "get", "put", "post", "delete", "options", "head", "patch", "trace", "query",
];

/// One operation of a spec.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Operation {
    /// Its method, in lowercase.
    pub method: String,
    /// Its path, as the spec writes it.
    pub path: String,
    /// Its `operationId`, when it has one.
    pub operation_id: Option<String>,
    /// Its `summary`, when it has one.
    pub summary: Option<String>,
    /// Its first tag, when it has one.
    pub tag: Option<String>,
    /// Whether the spec marks it `deprecated`.
    pub deprecated: bool,
}

/// What a spec holds, as far as it was read.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Spec {
    /// Its operations, in the order the spec writes them.
    pub operations: Vec<Operation>,
    /// The parts of `paths` that weren't read, each with why: a path item
    /// that's a `$ref` to another file, or one that isn't a mapping.
    pub unread: Vec<String>,
}

impl Operation {
    /// Its method and path as a link's fragment names them: `get/users/{id}`.
    pub fn method_and_path(&self) -> String {
        format!("{}{}", self.method, self.path)
    }
}

/// How a link's fragment names an operation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Named<'s> {
    /// By its `operationId`.
    Id(&'s Operation),
    /// By its method and path, `get/users/{id}`, when no operation has the
    /// fragment as its id.
    MethodAndPath(&'s Operation),
}

impl<'s> Named<'s> {
    /// The operation, however it's named.
    pub fn operation(self) -> &'s Operation {
        match self {
            Named::Id(operation) | Named::MethodAndPath(operation) => operation,
        }
    }
}

impl Spec {
    /// The operation with this `operationId`: the first, when the spec gives
    /// two the same id.
    pub fn operation(&self, id: &str) -> Option<&Operation> {
        self.operations
            .iter()
            .find(|o| o.operation_id.as_deref() == Some(id))
    }

    /// The operation a link's fragment, percent-decoded, names: the one with
    /// it as its `operationId`, since an id is what a writer chose, and
    /// otherwise the one with its method and path, `get/users/{id}`.
    pub fn named(&self, fragment: &str) -> Option<Named<'_>> {
        if let Some(operation) = self.operation(fragment) {
            return Some(Named::Id(operation));
        }
        let (method, path) = method_and_path(fragment)?;
        self.operations
            .iter()
            .find(|o| o.method == method && o.path == path)
            .map(Named::MethodAndPath)
    }

    /// Every `operationId`, in the spec's order.
    pub fn ids(&self) -> impl Iterator<Item = &str> {
        self.operations
            .iter()
            .filter_map(|o| o.operation_id.as_deref())
    }
}

/// The method and the path a fragment is made of, when it's a method in
/// lowercase, then the path from its first `/`: `get` and `/users/{id}` of
/// `get/users/{id}`.
pub fn method_and_path(fragment: &str) -> Option<(&str, &str)> {
    let (method, _) = fragment.split_once('/')?;
    (!method.is_empty() && method.bytes().all(|b| b.is_ascii_lowercase()))
        .then(|| fragment.split_at(method.len()))
}

/// Why a spec can't be read.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Unreadable {
    /// No file is there with this exact name.
    Missing,
    /// A file is there whose name differs only in case: its path from the
    /// project root.
    Case(RelPath),
    /// The file couldn't be read: the operating system's reason.
    Io(String),
    /// The file isn't YAML or JSON: the parser's reason.
    Syntax(String),
}

/// Reads a spec's text: JSON when `json`, YAML otherwise.
pub fn read_spec(text: &str, json: bool) -> Result<Spec, String> {
    let value: Value = if json {
        serde_json::from_str(text).map_err(|e| e.to_string())?
    } else {
        serde_yaml_ng::from_str(text).map_err(|e| e.to_string())?
    };
    Ok(spec_of(&value))
}

/// What a parsed document holds: its `paths`, when it's a mapping that has
/// them.
fn spec_of(document: &Value) -> Spec {
    let mut spec = Spec::default();
    let Some(Value::Mapping(paths)) = document.get("paths") else {
        return spec;
    };
    for (path, item) in paths {
        let Some(path) = path.as_str() else { continue };
        let Value::Mapping(item) = item else {
            spec.unread.push(format!("`{path}` isn't a mapping"));
            continue;
        };
        if let Some(reference) = item.get("$ref") {
            // A pointer to another path item of `paths` is an alias of its
            // operations, which are read where they are.
            if !reference
                .as_str()
                .is_some_and(|r| r.starts_with("#/paths/"))
            {
                spec.unread
                    .push(format!("`{path}` is a `$ref`, which isn't followed"));
            }
            continue;
        }
        let operations = METHODS
            .iter()
            .filter_map(|m| item.get(*m).map(|o| ((*m).to_owned(), o)))
            // OpenAPI 3.2's other methods, by their names as written.
            .chain(
                item.get("additionalOperations")
                    .and_then(Value::as_mapping)
                    .into_iter()
                    .flatten()
                    .filter_map(|(m, o)| Some((m.as_str()?.to_lowercase(), o))),
            );
        for (method, operation) in operations {
            let Value::Mapping(operation) = operation else {
                continue;
            };
            let text = |key: &str| {
                operation
                    .get(key)
                    .and_then(Value::as_str)
                    .map(str::to_owned)
            };
            spec.operations.push(Operation {
                method,
                path: path.to_owned(),
                operation_id: text("operationId"),
                summary: text("summary"),
                tag: operation
                    .get("tags")
                    .and_then(Value::as_sequence)
                    .and_then(|tags| tags.first())
                    .and_then(Value::as_str)
                    .map(str::to_owned),
                deprecated: operation
                    .get("deprecated")
                    .and_then(Value::as_bool)
                    .unwrap_or(false),
            });
        }
    }
    spec
}

/// Each declared API's spec, as read when the project was loaded.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ApiSpecs {
    specs: BTreeMap<String, Result<Spec, Unreadable>>,
}

impl ApiSpecs {
    /// Reads every spec the content model declares, through `fs`.
    pub fn read(model: &ContentModel, fs: &dyn FileSystem) -> ApiSpecs {
        let specs = model
            .apis
            .iter()
            .map(|api| (api.name.clone(), read_api(api, fs)))
            .collect();
        ApiSpecs { specs }
    }

    /// The spec of the API named `name`, when it was read.
    pub fn spec(&self, name: &str) -> Option<&Spec> {
        self.specs.get(name)?.as_ref().ok()
    }

    /// Why the spec of the API named `name` couldn't be read, when it
    /// couldn't.
    pub fn unreadable(&self, name: &str) -> Option<&Unreadable> {
        self.specs.get(name)?.as_ref().err()
    }

    /// The problems with the declared specs, at each one's `spec` in
    /// `ascribe.toml`: one that can't be read or parsed
    /// (`api-spec-unreadable`), and one with no operations
    /// (`api-spec-empty`).
    pub fn problems(&self, model: &ContentModel, model_file: ascribe_core::FileId) -> Vec<Issue> {
        let mut out = Vec::new();
        for api in &model.apis {
            let at = Location::new(model_file, api.spec_span);
            let issue = |slug| {
                Issue::new(slug, at)
                    .with_arg("api", api.name.clone())
                    .with_arg("path", api.spec.clone())
            };
            match self.specs.get(&api.name) {
                Some(Ok(spec)) if spec.operations.is_empty() => {
                    out.push(issue(diagnostics::API_SPEC_EMPTY));
                }
                Some(Err(why)) => out.push(match why {
                    Unreadable::Missing => {
                        issue(diagnostics::API_SPEC_UNREADABLE).with_variant("missing")
                    }
                    Unreadable::Case(actual) => issue(diagnostics::API_SPEC_UNREADABLE)
                        .with_variant("case")
                        .with_arg("actual", actual.to_string()),
                    Unreadable::Io(reason) | Unreadable::Syntax(reason) => {
                        issue(diagnostics::API_SPEC_UNREADABLE).with_arg("reason", reason.clone())
                    }
                }),
                _ => {}
            }
        }
        out
    }
}

/// One API's spec, from the project root: with its exact name, and parsed as
/// JSON when its name ends in `.json`.
fn read_api(api: &Api, fs: &dyn FileSystem) -> Result<Spec, Unreadable> {
    let path = RelPath::parse(&api.spec).map_err(|_| Unreadable::Missing)?;
    match fs.probe(&path) {
        Probe::File => {}
        Probe::Missing => return Err(Unreadable::Missing),
        Probe::CaseMismatch(actual) => return Err(Unreadable::Case(actual)),
    }
    let bytes = fs
        .read_file(&path)
        .map_err(|e| Unreadable::Io(e.to_string()))?;
    let text = String::from_utf8(bytes)
        .map_err(|_| Unreadable::Syntax("it isn't valid UTF-8".to_owned()))?;
    let json = path
        .extension()
        .is_some_and(|e| e.eq_ignore_ascii_case("json"));
    read_spec(&text, json).map_err(Unreadable::Syntax)
}

/// The problem with a link to an operation, when `resolution` is one:
///
/// - `link-operation-missing`, when the spec has no operation the fragment
///   names, with the closest as its fixes;
/// - `link-operation-form-unsupported`, when it names an operation with no id
///   by its method and path, and the reference is a template, which has no
///   address for it;
/// - `link-operation-by-path`, when it names an operation that has an id by
///   its method and path, with a fix that writes the id.
///
/// Nothing when the spec couldn't be read: `api-spec-unreadable` says why.
///
/// `at` is the whole link, and `destination` the destination as written
/// when it's known, where the issue is reported and its fixes write.
pub fn operation_issue(
    project: &Project,
    from: &RelPath,
    target: &Target,
    resolution: &Resolution,
    at: Location,
    destination: Option<Span>,
) -> Option<Issue> {
    let Resolution::Operation { api, id } = resolution else {
        return None;
    };
    let spec = project.apis().spec(api)?;
    let declared = project.model().api(api)?;
    let path = declared.spec.clone();
    let here = Location::new(at.file, destination.unwrap_or(at.span));
    match spec.named(id) {
        Some(Named::Id(_)) => None,
        Some(Named::MethodAndPath(operation)) => match &operation.operation_id {
            Some(operation_id) => {
                let mut issue = Issue::new(diagnostics::LINK_OPERATION_BY_PATH, here)
                    .with_arg("path", path)
                    .with_arg("operation", id.clone())
                    .with_arg("id", operation_id.clone());
                if let (Some(span), Target::Local(local)) = (destination, target) {
                    let href = format!("{}#{}", local.written, encode_destination(operation_id));
                    issue.fixes = vec![Fix {
                        title: format!("Link to `{href}`"),
                        file: at.file,
                        edits: vec![TextEdit::replace(span, href)],
                        // The same operation, by the name its reference
                        // addresses it with in every renderer.
                        applicability: Applicability::Safe,
                    }];
                }
                Some(issue)
            }
            None => matches!(declared.reference, ApiReference::Template(_)).then(|| {
                Issue::new(diagnostics::LINK_OPERATION_FORM_UNSUPPORTED, here)
                    .with_arg("api", api.clone())
                    .with_arg("operation", id.clone())
            }),
        },
        None => {
            let mut issue = Issue::new(diagnostics::LINK_OPERATION_MISSING, here)
                .with_arg("path", path)
                .with_arg("id", id.clone());
            let by_path = !matches!(declared.reference, ApiReference::Template(_));
            if let Some(closest) = closest_ids(spec, id, by_path).first() {
                issue = issue
                    .with_variant("closest")
                    .with_arg("closest", closest.clone());
            }
            issue.fixes = closest_fixes(project, from, target, resolution, at.file, destination);
            Some(issue)
        }
    }
}

/// How many operations are suggested for a link to a missing one.
const CLOSEST: usize = 5;

/// The names of the spec's operations closest to `fragment`, best first: an
/// operation's id, or its method and path when it has none and `by_path`.
/// `by_path` is false under a template, which has no address for an
/// operation with no id, so linking to one by its method and path is
/// `link-operation-form-unsupported`. The spec exists, so its operations are
/// the likely targets, however far.
pub fn closest_ids(spec: &Spec, fragment: &str, by_path: bool) -> Vec<String> {
    let mut scored: Vec<(usize, String)> = spec
        .operations
        .iter()
        .filter_map(|o| match &o.operation_id {
            Some(id) => Some(id.clone()),
            None => by_path.then(|| o.method_and_path()),
        })
        .map(|candidate| (edit_distance(fragment, &candidate), candidate))
        .collect();
    scored.sort();
    scored.dedup();
    scored.into_iter().take(CLOSEST).map(|(_, c)| c).collect()
}
