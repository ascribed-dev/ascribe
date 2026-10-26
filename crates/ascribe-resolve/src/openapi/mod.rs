//! The OpenAPI specs a project declares (`[apis.<name>]`), read loosely: each
//! operation's method, path, `operationId`, `summary`, first tag, and
//! `deprecated`, from `paths`, the same way in Swagger 2.0 and OpenAPI 3.0
//! to 3.2, since `paths` hasn't changed shape across them, and its request
//! and response bodies ([`schemas`]), which a marked JSON example is checked
//! against ([`examples`]). Everything else is ignored, and nothing is
//! validated: a spec a strict reader rejects has operations as good as any.
//!
//! Each spec is read once per project load, through [`FileSystem`], from the
//! project root: `read_file` reaches a spec outside the project's folder,
//! which the content model has already held to the project's repository.

pub mod examples;
pub mod schemas;

use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

use ascribe_core::{
    Applicability, Builtin, Fix, Issue, Location, RelPath, Span, TextEdit, diagnostics,
};
use ascribe_model::{Api, ApiReference, ContentModel, edit_distance};
use ascribe_syntax::{Block, BlockKind, PrimaryValue};
use serde_yaml_ng::Value;

use crate::closest::closest_fixes;
use crate::fs::{FileSystem, Probe};
use crate::index::walk::walk_blocks;
use crate::index::{FileIndex, RefKind, Target};
use crate::json_schema::token;
use crate::links::encode_destination;
use crate::project::{Project, Resolution};
use crate::references::{operation_target, reference_target};

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
    /// The line of its method's key in the spec, counted from 0, when a
    /// look at the text finds it: where go to definition opens the spec.
    pub line: Option<usize>,
    /// The statuses of its responses, as the spec writes them, with a body
    /// or not.
    pub responses: Vec<String>,
    /// Its request body and responses, by media type.
    pub bodies: Vec<schemas::Body>,
}

/// What a spec holds, as far as it was read.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Spec {
    /// Its operations, in the order the spec writes them.
    pub operations: Vec<Operation>,
    /// The parts of `paths` that weren't read, each with why: a path item
    /// that's a `$ref` to another file, or one that isn't a mapping.
    pub unread: Vec<String>,
    /// The whole spec as JSON, where its bodies' schemas are.
    pub document: Arc<serde_json::Value>,
}

impl Operation {
    /// Its method and path as a link's fragment names them: `get/users/{id}`.
    pub fn method_and_path(&self) -> String {
        format!("{}{}", self.method, self.path)
    }

    /// Its method, in capitals, and its path: `POST /widgets`, as a
    /// reference shows it.
    pub fn method_and_path_shown(&self) -> String {
        format!("{} {}", self.method.to_ascii_uppercase(), self.path)
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

/// How much of one API a project's pages cover.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Coverage {
    /// The API's name.
    pub api: String,
    /// How many links name each of its spec's operations, in the spec's
    /// order: by id, or by method and path, in the text or as an
    /// `@operation`'s. A link in any file counts, so the count holds for
    /// every build.
    pub links: Vec<usize>,
    /// How many of its operations a checked example claims as its request
    /// or response.
    pub examples: usize,
}

impl Coverage {
    /// How many operations the spec has.
    pub fn operations(&self) -> usize {
        self.links.len()
    }

    /// How many of them a link names.
    pub fn linked(&self) -> usize {
        self.links.iter().filter(|n| **n > 0).count()
    }
}

impl Project {
    /// How much of each declared API whose spec was read the project's
    /// links cover, in the content model's order.
    pub fn api_coverage(&self) -> Vec<Coverage> {
        let mut out: Vec<(Coverage, &Spec)> = self
            .model()
            .apis
            .iter()
            .filter_map(|api| {
                let spec = self.apis().spec(&api.name)?;
                let coverage = Coverage {
                    api: api.name.clone(),
                    links: vec![0; spec.operations.len()],
                    examples: 0,
                };
                Some((coverage, spec))
            })
            .collect();
        if out.is_empty() {
            return Vec::new();
        }
        // The operations a marked example claims, by API and position.
        let mut claimed: BTreeSet<(String, usize)> = BTreeSet::new();
        for file in self.files() {
            let operations = self
                .resolutions(&file.path)
                .iter()
                .cloned()
                .chain(embedded_operations(self, file));
            for resolution in operations {
                let Resolution::Operation { api, id } = &resolution else {
                    continue;
                };
                if let Some(((coverage, _), at)) = position(&mut out, api, id) {
                    coverage.links[at] += 1;
                }
            }
            let resolutions = self.resolutions(&file.path);
            for marker in &file.examples {
                if let Some(Resolution::Operation { api, id }) = resolutions.get(marker.reference)
                    && let Some((_, at)) = position(&mut out, api, id)
                {
                    claimed.insert((api.clone(), at));
                }
            }
        }
        for (coverage, _) in &mut out {
            coverage.examples = claimed
                .iter()
                .filter(|(api, _)| *api == coverage.api)
                .count();
        }
        out.into_iter().map(|(coverage, _)| coverage).collect()
    }
}

/// The coverage of `api` and the position in its spec of the operation `id`
/// names.
fn position<'c, 's>(
    out: &'c mut [(Coverage, &'s Spec)],
    api: &str,
    id: &str,
) -> Option<(&'c mut (Coverage, &'s Spec), usize)> {
    let entry = out.iter_mut().find(|(c, _)| c.api == api)?;
    let operation = entry.1.named(id)?.operation();
    let at = entry
        .1
        .operations
        .iter()
        .position(|o| std::ptr::eq(o, operation))?;
    Some((entry, at))
}

/// What the links of a file's `@operation` lines name (SPEC §4.13), which
/// aren't among its references: each is resolved from the declaration
/// alone, as the build resolves it.
pub(crate) fn embedded_operations(project: &Project, file: &FileIndex) -> Vec<Resolution> {
    let model = project.model();
    let mut out = Vec::new();
    walk_blocks(&file.document.blocks, &mut |block: &Block| {
        let BlockKind::Directive(line) = &block.kind else {
            return;
        };
        if line.name != Builtin::Operation.name() {
            return;
        }
        let Some(PrimaryValue::Identifier(primary)) = &line.primary else {
            return;
        };
        let target = reference_target(RefKind::Link, &primary.text, &[], &file.path, model);
        out.extend(operation_target(&target, model, project.layout()));
    });
    out
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
    let mut spec = spec_of(&value);
    let lines = operation_lines(text);
    for operation in &mut spec.operations {
        operation.line = lines
            .get(&(operation.path.clone(), operation.method.clone()))
            .copied();
    }
    Ok(spec)
}

/// The line of each operation's method key, by its path and method, from
/// the text: the parsers keep no positions, so the lines are found as a
/// person reading the file finds them. Under the `paths` key, a key one
/// level in is a path, and a key one level into a path that's a method, or
/// one level into its `additionalOperations`, is an operation. That holds
/// for YAML's block style and for JSON written one member to a line, which
/// is how specs are written; an operation in a flow mapping, or on a line
/// with its path, has no line, and the spec opens at its top.
fn operation_lines(text: &str) -> BTreeMap<(String, String), usize> {
    let mut lines = BTreeMap::new();
    let mut paths: Option<usize> = None;
    let mut path_indent: Option<usize> = None;
    let mut path: Option<&str> = None;
    let mut method_indent: Option<usize> = None;
    // Inside a path's `additionalOperations`: its indent, and its keys'.
    let mut additional: Option<(usize, Option<usize>)> = None;
    for (number, line) in text.lines().enumerate() {
        let Some((indent, key)) = line_key(line) else {
            continue;
        };
        let Some(top) = paths else {
            if key == "paths" {
                paths = Some(indent);
            }
            continue;
        };
        if indent <= top {
            break;
        }
        let at_path = *path_indent.get_or_insert(indent);
        if indent <= at_path {
            path = (indent == at_path).then_some(key);
            method_indent = None;
            additional = None;
            continue;
        }
        let Some(current) = path else { continue };
        let at_method = *method_indent.get_or_insert(indent);
        if indent == at_method {
            additional = (key == "additionalOperations").then_some((indent, None));
            if METHODS.contains(&key) {
                lines
                    .entry((current.to_owned(), key.to_owned()))
                    .or_insert(number);
            }
        } else if let Some((at, inner)) = &mut additional
            && indent > *at
            && *inner.get_or_insert(indent) == indent
        {
            lines
                .entry((current.to_owned(), key.to_lowercase()))
                .or_insert(number);
        }
    }
    lines
}

/// The key a line starts, unquoted, and the line's indent: `/users/{id}` of
/// `  /users/{id}:` and of `    "/users/{id}": {`. `None` for a line that
/// starts no key: a comment, an item of a list, a value going on.
fn line_key(line: &str) -> Option<(usize, &str)> {
    let trimmed = line.trim_start();
    let indent = line.len() - trimmed.len();
    let quote = trimmed.chars().next()?;
    if quote == '"' || quote == '\'' {
        let end = trimmed[1..].find(quote)? + 1;
        let rest = trimmed[end + 1..].trim_start();
        return rest.starts_with(':').then(|| (indent, &trimmed[1..end]));
    }
    if quote == '#' || quote == '-' {
        return None;
    }
    // A colon followed by a space or the line's end, since a path can hold
    // one: `/v1/{name}:cancel`.
    let end = trimmed.match_indices(':').map(|(i, _)| i).find(|&i| {
        trimmed[i + 1..]
            .chars()
            .next()
            .is_none_or(|c| c == ' ' || c == '\t')
    })?;
    Some((indent, trimmed[..end].trim_end()))
}

/// What a parsed document holds: its `paths`, when it's a mapping that has
/// them.
fn spec_of(document: &Value) -> Spec {
    let json = schemas::json_of(document);
    let mut spec = Spec::default();
    let Some(Value::Mapping(paths)) = document.get("paths") else {
        spec.document = Arc::new(json);
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
        let item_pointer = format!("/paths/{}", token(path));
        let operations = METHODS
            .iter()
            .filter_map(|m| {
                let pointer = format!("{item_pointer}/{m}");
                item.get(*m).map(|o| ((*m).to_owned(), o, pointer))
            })
            // OpenAPI 3.2's other methods, by their names as written.
            .chain(
                item.get("additionalOperations")
                    .and_then(Value::as_mapping)
                    .into_iter()
                    .flatten()
                    .filter_map(|(m, o)| {
                        let m = m.as_str()?;
                        let pointer = format!("{item_pointer}/additionalOperations/{}", token(m));
                        Some((m.to_lowercase(), o, pointer))
                    }),
            );
        for (method, operation, pointer) in operations {
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
                line: None,
                responses: schemas::statuses(&json, &pointer),
                bodies: schemas::bodies(&json, &item_pointer, &pointer),
            });
        }
    }
    spec.document = Arc::new(json);
    spec
}

/// Each declared API's spec, as read when the project was loaded, or last
/// read again, and the method and path of each `operationId` an earlier
/// reading had.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ApiSpecs {
    specs: BTreeMap<String, Result<Spec, Unreadable>>,
    /// By API, then by `operationId`: the method and path the id had when
    /// last seen, in an earlier reading of the spec or a base revision's.
    earlier: BTreeMap<String, BTreeMap<String, (String, String)>>,
}

impl ApiSpecs {
    /// Reads every spec the content model declares, through `fs`.
    pub fn read(model: &ContentModel, fs: &dyn FileSystem) -> ApiSpecs {
        let specs = model
            .apis
            .iter()
            .map(|api| (api.name.clone(), read_api(api, fs)))
            .collect();
        ApiSpecs {
            specs,
            earlier: BTreeMap::new(),
        }
    }

    /// Reads every spec again, keeping where each id of this reading was,
    /// and of the readings it kept, so a link to an id the spec has lost
    /// since can be fixed ([`ApiSpecs::renamed`]).
    pub fn read_again(&self, model: &ContentModel, fs: &dyn FileSystem) -> ApiSpecs {
        let mut next = ApiSpecs::read(model, fs);
        next.earlier = self.earlier.clone();
        next.keep(self, true);
        next
    }

    /// Keeps where each id of another reading's specs was, a base
    /// revision's, without replacing what an earlier reading of these says.
    pub fn remember(&mut self, other: &ApiSpecs) {
        self.keep(other, false);
    }

    fn keep(&mut self, other: &ApiSpecs, newer: bool) {
        for (api, spec) in &other.specs {
            let Ok(spec) = spec else { continue };
            let known = self.earlier.entry(api.clone()).or_default();
            for operation in &spec.operations {
                let Some(id) = &operation.operation_id else {
                    continue;
                };
                let at = (operation.method.clone(), operation.path.clone());
                if newer {
                    known.insert(id.clone(), at);
                } else {
                    known.entry(id.clone()).or_insert(at);
                }
            }
        }
    }

    /// The operation an id the spec of API `api` no longer has was renamed
    /// to: the one operation at the method and path the id had when last
    /// seen, whose own id is new there. `None` when the spec has the id,
    /// nothing says where it was, or no operation, or more than one, is
    /// there now.
    pub fn renamed(&self, api: &str, id: &str) -> Option<&Operation> {
        let spec = self.spec(api)?;
        if spec.operation(id).is_some() {
            return None;
        }
        let known = self.earlier.get(api)?;
        let (method, path) = known.get(id)?;
        let mut there = spec
            .operations
            .iter()
            .filter(|o| &o.method == method && &o.path == path);
        let only = there.next()?;
        if there.next().is_some() {
            return None;
        }
        let new_id = only.operation_id.as_deref()?;
        // An id that was somewhere else moved here; it wasn't renamed.
        known
            .get(new_id)
            .is_none_or(|at| at == &(method.clone(), path.clone()))
            .then_some(only)
    }

    /// The APIs whose specs differ between this reading and `other`.
    pub fn changed(&self, other: &ApiSpecs) -> Vec<String> {
        let names: std::collections::BTreeSet<&String> =
            self.specs.keys().chain(other.specs.keys()).collect();
        names
            .into_iter()
            .filter(|name| self.specs.get(*name) != other.specs.get(*name))
            .cloned()
            .collect()
    }

    /// The names of the APIs read.
    pub fn names(&self) -> impl Iterator<Item = &str> {
        self.specs.keys().map(String::as_str)
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
///   names, with the closest as its fixes, the first safe when it's the
///   operation the id was renamed to ([`ApiSpecs::renamed`]), which the
///   message names then, with its method and path;
/// - `link-operation-form-unsupported`, when it names an operation with no id
///   by its method and path, and the reference is a template, which has no
///   address for it;
/// - `link-operation-by-path`, when it names an operation that has an id by
///   its method and path, with a fix that writes the id;
/// - `link-operation-deprecated`, when it names an operation the spec marks
///   `deprecated`.
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
    let deprecated = |operation: &Operation| {
        operation.deprecated.then(|| {
            Issue::new(diagnostics::LINK_OPERATION_DEPRECATED, here)
                .with_arg("path", path.clone())
                .with_arg("operation", id.clone())
        })
    };
    match spec.named(id) {
        Some(Named::Id(operation)) => deprecated(operation),
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
            None if matches!(declared.reference, ApiReference::Template(_)) => Some(
                Issue::new(diagnostics::LINK_OPERATION_FORM_UNSUPPORTED, here)
                    .with_arg("api", api.clone())
                    .with_arg("operation", id.clone()),
            ),
            None => deprecated(operation),
        },
        None => {
            let renamed = project.apis().renamed(api, id);
            let mut issue = Issue::new(diagnostics::LINK_OPERATION_MISSING, here)
                .with_arg("path", path.clone())
                .with_arg("id", id.clone());
            let by_path = !matches!(declared.reference, ApiReference::Template(_));
            if let Some((operation, closest)) =
                renamed.and_then(|o| Some((o, o.operation_id.clone()?)))
            {
                issue = issue
                    .with_variant("renamed")
                    .with_arg("closest", closest)
                    .with_arg("at", operation.method_and_path_shown());
            } else if let Some(closest) = closest_operations(project.apis(), api, id, by_path)
                .into_iter()
                .next()
            {
                issue = issue.with_variant("closest").with_arg("closest", closest);
            }
            issue.fixes = closest_fixes(project, from, target, resolution, at.file, destination);
            // The operation now at the method and path the id had is the one
            // it was renamed to, when it's the only one there: the
            // closest target, and safe to write.
            if renamed.is_some()
                && let Some(fix) = issue.fixes.first_mut()
            {
                fix.applicability = Applicability::Safe;
            }
            Some(issue)
        }
    }
}

/// How many operations are suggested for a link to a missing one.
const CLOSEST: usize = 5;

/// The names of the spec's operations closest to `fragment`, best first: an
/// operation's id, or its method and path when it has none and `by_path`,
/// as [`closest_ids`] has them, with the operation an id was renamed to
/// ([`ApiSpecs::renamed`]) first.
pub fn closest_operations(
    specs: &ApiSpecs,
    api: &str,
    fragment: &str,
    by_path: bool,
) -> Vec<String> {
    let Some(spec) = specs.spec(api) else {
        return Vec::new();
    };
    let mut out = closest_ids(spec, fragment, by_path);
    if let Some(id) = specs
        .renamed(api, fragment)
        .and_then(|o| o.operation_id.clone())
    {
        out.retain(|c| *c != id);
        out.insert(0, id);
        out.truncate(CLOSEST);
    }
    out
}

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
