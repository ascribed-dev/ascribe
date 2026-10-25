//! `[apis.<name>]`: the OpenAPI specs a project's pages link into, and where
//! each one's reference is published. `spec` is a file from the project
//! root, which may be outside the project's folder in the same git
//! repository, checked as a source's folder is; whether it reads is for the
//! resolver to say, when it reads it. `reference` is Redoc's page by name or
//! a template of an operation's address.

use std::path::Path;

use ascribe_core::{RelPath, Span, diagnostics};
use toml::de::DeValue;

use crate::SourceBound;
use crate::loader::{Loader, NameRule, is_absolute, repository_root};
use crate::model::{API_PLACEHOLDERS, Api, ApiReference};
use crate::settings;
use crate::toml_util::{V, entries, sp};

impl Loader<'_> {
    /// `[apis.<name>]`, in declaration order.
    pub(crate) fn apis(&mut self, v: Option<&V<'_>>) -> Vec<Api> {
        let mut out = Vec::new();
        let Some(v) = v else { return out };
        let Some(t) = self.as_table("apis", v) else {
            return out;
        };
        for (name, name_span, item) in entries(t) {
            self.name_ok(name, name_span, "API", NameRule::Key);
            let path = format!("apis.{name}");
            let Some(at) = self.as_table(&path, item) else {
                continue;
            };
            self.check_table(&path, at, "apis.<name>");
            let spec = self
                .require(&path, at, name_span, "spec")
                .and_then(|v| Some((self.spec_path(name, &path, v)?, sp(v))));
            let reference = self
                .require(&path, at, name_span, "reference")
                .and_then(|v| self.reference(name, &path, v));
            if let (Some((spec, spec_span)), Some(reference)) = (spec, reference) {
                out.push(Api {
                    name: name.to_owned(),
                    spec,
                    reference,
                    span: name_span,
                    spec_span,
                });
            }
        }
        out
    }

    /// `spec`: a relative path, normalized, of a file in the project's git
    /// repository when it's in one, or, with none, inside a folder
    /// [`SourceBound::Within`] names when the model is loaded with one.
    fn spec_path(&mut self, name: &str, path: &str, v: &V<'_>) -> Option<String> {
        let key = format!("{path}.spec");
        let text = self.string(&key, v, true)?;
        if is_absolute(&text) {
            self.push(
                self.issue(diagnostics::MODEL_PATH_ABSOLUTE, sp(v))
                    .with_arg("key", key),
            );
            return None;
        }
        let spec = RelPath::parse(&text).ok()?.to_string();
        if let Some(dir) = self.project_dir {
            self.spec_inside(name, &text, dir, sp(v));
        }
        Some(spec)
    }

    /// The same-repository rule for a spec, as `source_folder` has it for a
    /// source's folder.
    fn spec_inside(&mut self, name: &str, spec: &str, dir: &Path, span: Span) {
        let resolved = self.resolve(spec);
        let variant = match repository_root(dir) {
            Some(repository) => (!resolved.starts_with(&repository)).then_some(None),
            None => match self.bound {
                // Outside FileSystem: the folders the user opened, compared
                // as the spec is, with links resolved.
                SourceBound::Within(opened) => (!opened.iter().any(|o| {
                    let o = o
                        .canonicalize()
                        .unwrap_or_else(|_| ascribe_core::path::normalize(o));
                    resolved.starts_with(o)
                }))
                .then_some(Some("opened")),
                SourceBound::Anywhere => None,
            },
        };
        if let Some(variant) = variant {
            let issue = self
                .issue(diagnostics::MODEL_API_OUTSIDE_REPOSITORY, span)
                .with_arg("api", name)
                .with_arg("path", spec);
            self.push(match variant {
                Some(variant) => issue.with_variant(variant),
                None => issue,
            });
        }
    }

    /// `reference`: `{ renderer = "redoc", at = "…" }`, or a template whose
    /// placeholders are [`API_PLACEHOLDERS`] and that names the operation.
    fn reference(&mut self, name: &str, path: &str, v: &V<'_>) -> Option<ApiReference> {
        let key = format!("{path}.reference");
        match v.get_ref() {
            DeValue::Table(t) => {
                self.check_table(&key, t, "apis.<name>.reference");
                let renderer = self.require(&key, t, sp(v), "renderer");
                let at = self.require(&key, t, sp(v), "at");
                let renderer = renderer.and_then(|r| {
                    self.choice(
                        &format!("{key}.renderer"),
                        r,
                        &settings::choices("apis.<name>.reference", "renderer"),
                    )
                });
                let at = at.and_then(|a| self.string(&format!("{key}.at"), a, true));
                // Redoc is the one renderer `renderer` names.
                renderer.and(at).map(|at| ApiReference::Redoc { at })
            }
            DeValue::String(_) => {
                let template = self.string(&key, v, true)?;
                self.template_ok(name, &template, sp(v))
                    .then_some(ApiReference::Template(template))
            }
            _ => {
                self.wrong_type(&key, v, "a table or a string");
                None
            }
        }
    }

    /// A template's placeholders are [`API_PLACEHOLDERS`], and
    /// `{operationId}` is one of them, so each operation has an address of
    /// its own.
    fn template_ok(&mut self, name: &str, template: &str, span: Span) -> bool {
        let mut rest = template;
        while let Some(open) = rest.find('{') {
            let after = &rest[open + 1..];
            let Some(close) = after.find('}') else { break };
            let placeholder = &after[..close];
            if !API_PLACEHOLDERS.contains(&placeholder) {
                self.push(
                    self.issue(diagnostics::MODEL_API_REFERENCE, span)
                        .with_variant("placeholder")
                        .with_arg("api", name)
                        .with_arg("placeholder", placeholder),
                );
                return false;
            }
            rest = &after[close + 1..];
        }
        if !template.contains("{operationId}") {
            self.push(
                self.issue(diagnostics::MODEL_API_REFERENCE, span)
                    .with_arg("api", name),
            );
            return false;
        }
        true
    }
}
