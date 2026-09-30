//! Loading `ascribe.toml`: the walk over the TOML tree, and the sections that
//! define names (project, dimensions, lifecycle, features, notes, phrases).
//! Fields and attributes are in `fields.rs`; types, glossary, images,
//! widgets, consumer, builds, and editor are in `sections.rs`.

use std::collections::HashMap;
use std::path::{Component, Path, PathBuf};

use tessera_core::availability::{AvailabilitySpec, parse_availability};
use tessera_core::{FileId, Issue, Location, Span, diagnostics};
use toml::de::{DeTable, DeValue};

use crate::model::*;
use crate::names::{self, is_key, is_name_word, list, suggest};
use crate::toml_util::{V, describe, entries, join, sp};

pub(crate) struct Loader<'s> {
    pub src: &'s str,
    pub file: FileId,
    pub project_dir: Option<&'s Path>,
    pub issues: Vec<Issue>,
    pub warnings: Vec<Issue>,
}

pub(crate) fn load(
    src: &str,
    file: FileId,
    project_dir: Option<&Path>,
) -> Result<ContentModel, Vec<Issue>> {
    let mut l = Loader {
        src,
        file,
        project_dir,
        issues: Vec::new(),
        warnings: Vec::new(),
    };
    let model = l.run();
    let mut all: Vec<Issue> = Vec::new();
    all.append(&mut l.issues);
    let has_errors = !all.is_empty();
    match model {
        Some(mut m) if !has_errors => {
            m.warnings = std::mem::take(&mut l.warnings);
            m.warnings.sort_by_key(|i| i.location.span.start());
            Ok(m)
        }
        _ => {
            all.append(&mut l.warnings);
            all.sort_by_key(|i| i.location.span.start());
            Err(all)
        }
    }
}

/// A declared name that plays a role in availability specs (SPEC §7.2).
struct RoleName {
    name: String,
    role: String,
    span: Span,
    builtin: bool,
}

impl<'s> Loader<'s> {
    // ---- helpers ---------------------------------------------------------

    pub fn loc(&self, span: Span) -> Location {
        Location::new(self.file, span)
    }

    pub fn issue(&self, slug: tessera_core::DiagnosticSlug, span: Span) -> Issue {
        Issue::new(slug, self.loc(span))
    }

    pub fn push(&mut self, issue: Issue) {
        self.issues.push(issue);
    }

    pub fn text_of(&self, span: Span) -> &'s str {
        self.src.get(span.range()).unwrap_or("")
    }

    pub fn wrong_type(&mut self, path: &str, v: &V<'_>, expected: &str) {
        let mut issue = self
            .issue(diagnostics::MODEL_WRONG_TYPE, sp(v))
            .with_arg("key", path)
            .with_arg("expected", expected)
            .with_arg("found", describe(v.get_ref()));
        if expected == "a string" {
            let value = match v.get_ref() {
                DeValue::Integer(i) => Some(i.as_str().to_owned()),
                DeValue::Float(f) => Some(f.as_str().to_owned()),
                _ => None,
            };
            if let Some(value) = value {
                issue = issue.with_variant("quote").with_arg("value", value);
            }
        }
        self.push(issue);
    }

    pub fn as_table<'a, 'i>(&mut self, path: &str, v: &'a V<'i>) -> Option<&'a DeTable<'i>> {
        match v.get_ref() {
            DeValue::Table(t) => Some(t),
            _ => {
                self.wrong_type(path, v, "a table");
                None
            }
        }
    }

    pub fn string(&mut self, path: &str, v: &V<'_>, non_empty: bool) -> Option<String> {
        match v.get_ref() {
            DeValue::String(s) => {
                if non_empty && s.trim().is_empty() {
                    self.push(
                        self.issue(diagnostics::MODEL_EMPTY_TEXT, sp(v))
                            .with_arg("key", path),
                    );
                    return None;
                }
                Some(s.to_string())
            }
            _ => {
                self.wrong_type(path, v, "a string");
                None
            }
        }
    }

    pub fn boolean(&mut self, path: &str, v: &V<'_>) -> Option<bool> {
        match v.get_ref() {
            DeValue::Boolean(b) => Some(*b),
            _ => {
                self.wrong_type(path, v, "a boolean");
                None
            }
        }
    }

    /// An array of strings, each with its span.
    pub fn strings(&mut self, path: &str, v: &V<'_>) -> Option<Vec<(String, Span)>> {
        let DeValue::Array(items) = v.get_ref() else {
            self.wrong_type(path, v, "an array of strings");
            return None;
        };
        let mut out = Vec::new();
        let mut ok = true;
        for item in items.iter() {
            match item.get_ref() {
                DeValue::String(s) => out.push((s.to_string(), sp(item))),
                _ => {
                    self.wrong_type(&format!("{path}[]"), item, "a string");
                    ok = false;
                }
            }
        }
        ok.then_some(out)
    }

    /// A string that must be one of `allowed`.
    pub fn choice(&mut self, path: &str, v: &V<'_>, allowed: &[&str]) -> Option<String> {
        let s = self.string(path, v, false)?;
        if allowed.contains(&s.as_str()) {
            Some(s)
        } else {
            self.push(
                self.issue(diagnostics::MODEL_INVALID_VALUE, sp(v))
                    .with_arg("key", path)
                    .with_arg("value", s)
                    .with_arg("values", list(allowed)),
            );
            None
        }
    }

    /// Reports unknown keys in a table whose keys this reference defines.
    pub fn check_keys(&mut self, path: &str, t: &DeTable<'_>, allowed: &[&str]) {
        for (key, span, _) in entries(t) {
            if allowed.contains(&key) {
                continue;
            }
            let mut issue = self
                .issue(diagnostics::MODEL_UNKNOWN_KEY, span)
                .with_arg("key", key)
                .with_arg("table", path);
            if let Some(s) = suggest(key, allowed.iter().copied()) {
                issue = issue.with_variant("suggestion").with_arg("suggestion", s);
            }
            self.push(issue);
        }
    }

    pub fn require<'a, 'i>(
        &mut self,
        path: &str,
        t: &'a DeTable<'i>,
        table_span: Span,
        key: &str,
    ) -> Option<&'a V<'i>> {
        let v = t.get(key);
        if v.is_none() {
            self.push(
                self.issue(diagnostics::MODEL_MISSING_KEY, table_span)
                    .with_arg("table", path)
                    .with_arg("key", key),
            );
        }
        v
    }

    /// Checks a name against a grammar; reports `model-invalid-name`.
    pub fn name_ok(&mut self, name: &str, span: Span, role: &str, rule: NameRule) -> bool {
        let (ok, text) = match rule {
            NameRule::Key => (is_key(name), names::KEY_RULE),
            NameRule::NameWord => (is_name_word(name), names::NAME_WORD_RULE),
            NameRule::Widget => (names::is_widget_name(name), names::WIDGET_RULE),
            NameRule::Build => (names::is_build_name(name), names::BUILD_RULE),
        };
        if !ok {
            self.push(
                self.issue(diagnostics::MODEL_INVALID_NAME, span)
                    .with_arg("name", name)
                    .with_arg("role", role)
                    .with_arg("rule", text),
            );
        }
        ok
    }

    // ---- the run ----------------------------------------------------------

    fn run(&mut self) -> Option<ContentModel> {
        let parsed = match DeTable::parse(self.src) {
            Ok(t) => t,
            Err(e) => {
                let span = e
                    .span()
                    .map_or(Span::empty(0), |r| Span::new(r.start, r.end));
                self.push(
                    self.issue(diagnostics::MODEL_TOML_SYNTAX, span)
                        .with_arg("detail", e.message().trim()),
                );
                return None;
            }
        };
        let root_span = sp(&parsed);
        let root = parsed.get_ref();
        self.check_keys(
            "ascribe.toml",
            root,
            &[
                "spec",
                "project",
                "types",
                "fragments",
                "dimensions",
                "versions",
                "lifecycle",
                "features",
                "notes",
                "phrases",
                "glossary",
                "images",
                "widgets",
                "consumer",
                "builds",
                "editor",
            ],
        );

        let spec = self.spec(root, root_span);
        let project = self.project(root.get("project"));
        let (dimensions, dim_spans) = self.dimensions(root.get("dimensions"));
        let (lifecycle, declared_states) = self.lifecycle(root.get("lifecycle"));
        let features = self.features_raw(root.get("features"));
        let notes = self.notes(root.get("notes"));
        let phrases = self.phrases(root.get("phrases"));
        self.check_roles(
            &dimensions,
            &dim_spans,
            &lifecycle,
            &declared_states,
            &features,
        );
        let features = self.features_checked(features, &dimensions, &lifecycle);

        let fragments = self.fragments(root.get("fragments"));
        let types = self.types(root.get("types"), &fragments);
        let glossary = self.glossary(root.get("glossary"), &project, &fragments);
        let image_attributes = self.images(root.get("images"));
        let widgets = self.widgets(root.get("widgets"));
        let consumer = self.consumer(root.get("consumer"));
        let builds = self.builds(root.get("builds"), &dimensions);
        let editor_build = self.editor(root.get("editor"), root.get("builds"), &builds);
        self.version_scheme(root.get("versions"));
        self.project_paths(&project, root.get("project"));

        Some(ContentModel {
            spec: spec?,
            project: project.value,
            types,
            fragments,
            dimensions,
            version_scheme: VersionScheme::Numeric,
            lifecycle,
            features,
            notes,
            phrases,
            glossary,
            image_attributes,
            widgets,
            consumer,
            builds,
            editor_build,
            warnings: Vec::new(),
        })
    }

    fn spec(&mut self, root: &DeTable<'_>, root_span: Span) -> Option<String> {
        let v = self.require(
            "ascribe.toml",
            root,
            Span::new(root_span.start(), root_span.start()),
            "spec",
        )?;
        let s = self.string("spec", v, false)?;
        if SUPPORTED_SPECS.contains(&s.as_str()) {
            Some(s)
        } else {
            let supported = SUPPORTED_SPECS
                .iter()
                .map(|s| format!("\"{s}\""))
                .collect::<Vec<_>>()
                .join(", ");
            self.push(
                self.issue(diagnostics::MODEL_SPEC_UNSUPPORTED, sp(v))
                    .with_arg("spec", s)
                    .with_arg("supported", supported),
            );
            None
        }
    }

    fn version_scheme(&mut self, v: Option<&V<'_>>) {
        let Some(v) = v else { return };
        let Some(t) = self.as_table("versions", v) else {
            return;
        };
        self.check_keys("versions", t, &["scheme"]);
        if let Some(s) = t.get("scheme") {
            self.choice("versions.scheme", s, &["numeric"]);
        }
    }

    // ---- project -----------------------------------------------------------

    fn project(&mut self, v: Option<&V<'_>>) -> ProjectInfo {
        let mut info = ProjectInfo {
            value: Project {
                content_root: "docs".into(),
                output_dir: ".ascribe/build".into(),
            },
            content_span: None,
            output_span: None,
        };
        let Some(v) = v else { return info };
        let Some(t) = self.as_table("project", v) else {
            return info;
        };
        self.check_keys("project", t, &["content-root", "output-dir"]);
        for (key, slot) in [("content-root", 0), ("output-dir", 1)] {
            let Some(item) = t.get(key) else { continue };
            let path = format!("project.{key}");
            let Some(s) = self.string(&path, item, true) else {
                continue;
            };
            if is_absolute(&s) {
                self.push(
                    self.issue(diagnostics::MODEL_PATH_ABSOLUTE, sp(item))
                        .with_arg("key", path),
                );
                continue;
            }
            if slot == 0 {
                info.value.content_root = s;
                info.content_span = Some(sp(item));
            } else {
                info.value.output_dir = s;
                info.output_span = Some(sp(item));
            }
        }
        info
    }

    /// The output directory / content root relation, and the content root's
    /// existence.
    fn project_paths(&mut self, project: &ProjectInfo, table: Option<&V<'_>>) {
        let fallback = table.map_or(Span::empty(0), sp);
        let content_span = project.content_span.unwrap_or(fallback);
        let output_span = project.output_span.unwrap_or(fallback);
        let content = &project.value.content_root;
        let output = &project.value.output_dir;
        let (c, o) = (self.resolve(content), self.resolve(output));
        if c == o {
            self.push(
                self.issue(diagnostics::MODEL_OUTPUT_OVERLAPS_CONTENT, output_span)
                    .with_variant("same")
                    .with_arg("path", content.as_str()),
            );
        } else if o.starts_with(&c) {
            self.push(
                self.issue(diagnostics::MODEL_OUTPUT_OVERLAPS_CONTENT, output_span)
                    .with_arg("output", output.as_str())
                    .with_arg("content", content.as_str()),
            );
        } else if c.starts_with(&o) {
            self.push(
                self.issue(diagnostics::MODEL_OUTPUT_OVERLAPS_CONTENT, content_span)
                    .with_variant("content-inside-output")
                    .with_arg("output", output.as_str())
                    .with_arg("content", content.as_str()),
            );
        }
        if let Some(dir) = self.project_dir {
            let root = dir.join(content);
            if !root.exists() {
                self.push(
                    self.issue(diagnostics::MODEL_CONTENT_ROOT_MISSING, content_span)
                        .with_arg("path", content.as_str()),
                );
            } else if !root.is_dir() {
                self.push(
                    self.issue(diagnostics::MODEL_CONTENT_ROOT_MISSING, content_span)
                        .with_variant("not-directory")
                        .with_arg("path", content.as_str()),
                );
            }
        }
    }

    /// A path relative to the project root, normalized (`.` and `..`), and,
    /// when the project directory is known, with symbolic links resolved for
    /// as much of it as exists.
    fn resolve(&self, rel: &str) -> PathBuf {
        let base = self.project_dir.map(Path::to_path_buf).unwrap_or_default();
        let joined = normalize(&base.join(rel));
        if self.project_dir.is_none() {
            return joined;
        }
        // Canonicalize the longest existing prefix.
        let mut existing = joined.clone();
        let mut rest = Vec::new();
        while !existing.exists() {
            match existing.file_name() {
                Some(name) => rest.push(name.to_owned()),
                None => break,
            }
            if !existing.pop() {
                break;
            }
        }
        let mut out = existing.canonicalize().unwrap_or(existing);
        for name in rest.into_iter().rev() {
            out.push(name);
        }
        out
    }

    // ---- dimensions --------------------------------------------------------

    fn dimensions(&mut self, v: Option<&V<'_>>) -> (Vec<Dimension>, Vec<DimSpans>) {
        let mut out = Vec::new();
        let mut spans = Vec::new();
        let Some(v) = v else { return (out, spans) };
        let Some(t) = self.as_table("dimensions", v) else {
            return (out, spans);
        };
        let mut value_owner: HashMap<String, String> = HashMap::new();
        for (name, name_span, item) in entries(t) {
            let path = join("dimensions", name);
            self.name_ok(name, name_span, "dimension", NameRule::Key);
            let Some(dt) = self.as_table(&path, item) else {
                continue;
            };
            self.check_keys(&path, dt, &["values", "labels", "label", "versionless"]);
            let label = match dt.get("label") {
                Some(l) => self.string(&format!("{path}.label"), l, true),
                None => None,
            }
            .unwrap_or_else(|| name.to_owned());
            let mut values: Vec<(String, Span)> = Vec::new();
            if let Some(vv) = self.require(&path, dt, sp(item), "values") {
                {
                    if let Some(vals) = self.strings(&format!("{path}.values"), vv) {
                        if vals.is_empty() {
                            self.push(
                                self.issue(diagnostics::MODEL_DIMENSION_EMPTY, sp(vv))
                                    .with_arg("dimension", name),
                            );
                        }
                        for (val, span) in vals {
                            if !self.name_ok(&val, span, "dimension value", NameRule::NameWord) {
                                continue;
                            }
                            if values.iter().any(|(x, _)| *x == val) {
                                self.push(
                                    self.issue(diagnostics::MODEL_DIMENSION_VALUE_DUPLICATE, span)
                                        .with_arg("value", val)
                                        .with_arg("dimension", name),
                                );
                                continue;
                            }
                            if let Some(other) = value_owner.get(&val)
                                && other != name
                            {
                                self.push(
                                    self.issue(diagnostics::MODEL_DIMENSION_VALUE_SHARED, span)
                                        .with_arg("value", val.as_str())
                                        .with_arg("a", other.as_str())
                                        .with_arg("b", name),
                                );
                                continue;
                            }
                            value_owner.insert(val.clone(), name.to_owned());
                            values.push((val, span));
                        }
                    }
                }
            }
            let names_of: Vec<String> = values.iter().map(|(v, _)| v.clone()).collect();
            let mut labels: HashMap<String, String> = HashMap::new();
            if let Some(lv) = dt.get("labels")
                && let Some(lt) = self.as_table(&format!("{path}.labels"), lv)
            {
                for (k, kspan, x) in entries(lt) {
                    let lpath = join(&format!("{path}.labels"), k);
                    if !names_of.iter().any(|n| n == k) {
                        self.push(
                            self.issue(diagnostics::MODEL_LABEL_UNDECLARED, kspan)
                                .with_arg("dimension", name)
                                .with_arg("value", k)
                                .with_arg("values", list(&names_of)),
                        );
                        continue;
                    }
                    if let Some(s) = self.string(&lpath, x, true) {
                        labels.insert(k.to_owned(), s);
                    }
                }
            }
            let mut versionless: Vec<String> = Vec::new();
            if let Some(vl) = dt.get("versionless")
                && let Some(items) = self.strings(&format!("{path}.versionless"), vl)
            {
                for (val, span) in items {
                    if names_of.contains(&val) {
                        versionless.push(val);
                    } else {
                        self.push(
                            self.issue(diagnostics::MODEL_VERSIONLESS_UNDECLARED, span)
                                .with_arg("dimension", name)
                                .with_arg("value", val)
                                .with_arg("values", list(&names_of)),
                        );
                    }
                }
            }
            spans.push(DimSpans {
                name_span,
                value_spans: values.iter().map(|(_, s)| *s).collect(),
            });
            out.push(Dimension {
                name: name.to_owned(),
                label,
                values: values
                    .into_iter()
                    .map(|(value, _)| DimensionValue {
                        label: labels.get(&value).cloned().unwrap_or_else(|| value.clone()),
                        versionless: versionless.contains(&value),
                        value,
                    })
                    .collect(),
            });
        }
        (out, spans)
    }

    // ---- lifecycle, notes, phrases ------------------------------------------

    fn lifecycle(&mut self, v: Option<&V<'_>>) -> (Vec<LifecycleState>, Vec<(String, Span)>) {
        let mut declared: Vec<(String, Span)> = Vec::new();
        let mut states: Vec<LifecycleState> = [
            ("preview", true, "preview"),
            ("beta", true, "beta"),
            ("ga", true, "GA"),
            ("deprecated", true, "deprecated"),
            ("removed", false, "removed"),
        ]
        .into_iter()
        .map(|(name, available, label)| LifecycleState {
            name: name.into(),
            available,
            label: label.into(),
            builtin: true,
        })
        .collect();
        let Some(v) = v else {
            return (states, declared);
        };
        let Some(t) = self.as_table("lifecycle", v) else {
            return (states, declared);
        };
        for (name, name_span, item) in entries(t) {
            let path = join("lifecycle", name);
            self.name_ok(name, name_span, "lifecycle state", NameRule::NameWord);
            let Some(st) = self.as_table(&path, item) else {
                continue;
            };
            self.check_keys(&path, st, &["available", "label"]);
            let available = st.get("available").and_then(|a| {
                self.boolean(&format!("{path}.available"), a)
                    .map(|b| (b, sp(a)))
            });
            let label = st
                .get("label")
                .and_then(|l| self.string(&format!("{path}.label"), l, true));
            let existing = states.iter().position(|s| s.name == name);
            if name == "ga"
                && let Some((false, span)) = available
            {
                self.push(self.issue(diagnostics::MODEL_LIFECYCLE_GA_UNAVAILABLE, span));
            }
            match existing {
                Some(i) => {
                    // `ga` stays available (already reported above).
                    if let Some((a, _)) = available
                        && (name != "ga" || a)
                    {
                        states[i].available = a;
                    }
                    if let Some(l) = label {
                        states[i].label = l;
                    }
                }
                None => {
                    let Some((available, _)) = available else {
                        if st.get("available").is_none() {
                            self.push(
                                self.issue(
                                    diagnostics::MODEL_LIFECYCLE_AVAILABLE_REQUIRED,
                                    name_span,
                                )
                                .with_arg("state", name),
                            );
                        }
                        continue;
                    };
                    declared.push((name.to_owned(), name_span));
                    states.push(LifecycleState {
                        name: name.to_owned(),
                        available,
                        label: label.unwrap_or_else(|| name.to_owned()),
                        builtin: false,
                    });
                }
            }
        }
        (states, declared)
    }

    fn notes(&mut self, v: Option<&V<'_>>) -> Vec<NoteType> {
        let mut notes: Vec<NoteType> = [
            ("note", "Note"),
            ("tip", "Tip"),
            ("important", "Important"),
            ("warning", "Warning"),
            ("caution", "Caution"),
        ]
        .into_iter()
        .map(|(name, label)| NoteType {
            name: name.into(),
            label: label.into(),
            builtin: true,
        })
        .collect();
        let Some(v) = v else { return notes };
        let Some(t) = self.as_table("notes", v) else {
            return notes;
        };
        for (name, name_span, item) in entries(t) {
            let path = join("notes", name);
            self.name_ok(name, name_span, "note type", NameRule::Key);
            let Some(nt) = self.as_table(&path, item) else {
                continue;
            };
            self.check_keys(&path, nt, &["label"]);
            let label = nt
                .get("label")
                .and_then(|l| self.string(&format!("{path}.label"), l, true));
            match notes.iter().position(|n| n.name == name) {
                Some(i) => {
                    if let Some(l) = label {
                        notes[i].label = l;
                    }
                }
                None => match label {
                    Some(label) => notes.push(NoteType {
                        name: name.to_owned(),
                        label,
                        builtin: false,
                    }),
                    None => {
                        if nt.get("label").is_none() {
                            let mut cap = name.to_owned();
                            if let Some(first) = cap.get_mut(0..1) {
                                first.make_ascii_uppercase();
                            }
                            self.push(
                                self.issue(diagnostics::MODEL_NOTE_LABEL_REQUIRED, name_span)
                                    .with_arg("type", name)
                                    .with_arg("Type", cap),
                            );
                        }
                    }
                },
            }
        }
        notes
    }

    fn phrases(&mut self, v: Option<&V<'_>>) -> Vec<Phrase> {
        let mut out = Vec::new();
        let Some(v) = v else { return out };
        let Some(t) = self.as_table("phrases", v) else {
            return out;
        };
        for (key, key_span, item) in entries(t) {
            self.name_ok(key, key_span, "phrase", NameRule::Key);
            match item.get_ref() {
                DeValue::String(s) => out.push(Phrase {
                    key: key.to_owned(),
                    value: s.to_string(),
                }),
                other => {
                    let raw = self.text_of(sp(item)).to_owned();
                    let shown = match other {
                        DeValue::Table(_) | DeValue::Array(_) => String::new(),
                        _ => raw,
                    };
                    self.push(
                        self.issue(diagnostics::MODEL_PHRASE_VALUE_TYPE, sp(item))
                            .with_arg("key", key)
                            .with_arg("found", describe(other))
                            .with_arg("value", shown),
                    );
                }
            }
        }
        out
    }

    // ---- names and roles -----------------------------------------------------

    /// The one-role rule (SPEC §7.2), and the warning for names that differ
    /// only in case.
    fn check_roles(
        &mut self,
        dimensions: &[Dimension],
        dim_spans: &[DimSpans],
        lifecycle: &[LifecycleState],
        declared_states: &[(String, Span)],
        features: &[RawFeature],
    ) {
        let mut all: Vec<RoleName> = Vec::new();
        for s in lifecycle.iter().filter(|s| s.builtin) {
            all.push(RoleName {
                name: s.name.clone(),
                role: "a lifecycle state (built in)".into(),
                span: Span::empty(0),
                builtin: true,
            });
        }
        for (name, span) in declared_states {
            all.push(RoleName {
                name: name.clone(),
                role: "a lifecycle state".into(),
                span: *span,
                builtin: false,
            });
        }
        for (d, spans) in dimensions.iter().zip(dim_spans) {
            all.push(RoleName {
                name: d.name.clone(),
                role: "a dimension name".into(),
                span: spans.name_span,
                builtin: false,
            });
            for (v, span) in d.values.iter().zip(&spans.value_spans) {
                all.push(RoleName {
                    name: v.value.clone(),
                    role: format!("a dimension value (in dimensions.{})", d.name),
                    span: *span,
                    builtin: false,
                });
            }
        }
        for f in features {
            all.push(RoleName {
                name: f.key.clone(),
                role: "a feature key".into(),
                span: f.key_span,
                builtin: false,
            });
        }
        let mut first: HashMap<String, usize> = HashMap::new();
        let mut folded: HashMap<String, usize> = HashMap::new();
        for (i, r) in all.iter().enumerate() {
            let prior = first.get(&r.name).copied();
            match prior {
                Some(p) if all[p].role.split(" (").next() != r.role.split(" (").next() => {
                    let (a, b) = if all[p].builtin || all[p].span.start() <= r.span.start() {
                        (&all[p], r)
                    } else {
                        (r, &all[p])
                    };
                    let at = if b.builtin { a.span } else { b.span };
                    self.push(
                        self.issue(diagnostics::MODEL_NAME_MULTIPLE_ROLES, at)
                            .with_arg("name", r.name.as_str())
                            .with_arg("role-a", a.role.as_str())
                            .with_arg("role-b", b.role.as_str()),
                    );
                    continue;
                }
                Some(_) => continue,
                None => {
                    first.insert(r.name.clone(), i);
                }
            }
            let lower = r.name.to_lowercase();
            match folded.get(&lower) {
                Some(&p) if all[p].name != r.name => {
                    let at = if r.builtin { all[p].span } else { r.span };
                    self.warnings.push(
                        self.issue(diagnostics::MODEL_NAME_CASE, at)
                            .with_arg("a", all[p].name.as_str())
                            .with_arg("b", r.name.as_str()),
                    );
                }
                Some(_) => {}
                None => {
                    folded.insert(lower, i);
                }
            }
        }
    }

    // ---- features -------------------------------------------------------------

    fn features_raw(&mut self, v: Option<&V<'_>>) -> Vec<RawFeature> {
        let mut out = Vec::new();
        let Some(v) = v else { return out };
        let Some(t) = self.as_table("features", v) else {
            return out;
        };
        for (key, key_span, item) in entries(t) {
            let path = join("features", key);
            self.name_ok(key, key_span, "feature", NameRule::NameWord);
            let Some(ft) = self.as_table(&path, item) else {
                continue;
            };
            self.check_keys(&path, ft, &["name", "available"]);
            let name = self
                .require(&path, ft, sp(item), "name")
                .and_then(|n| self.string(&format!("{path}.name"), n, true));
            let available = self.require(&path, ft, sp(item), "available");
            let spec = available.and_then(|a| {
                let text = self.string(&format!("{path}.available"), a, true)?;
                Some((text, sp(a)))
            });
            let (Some(name), Some((text, value_span))) = (name, spec) else {
                continue;
            };
            out.push(RawFeature {
                key: key.to_owned(),
                key_span,
                name,
                text,
                value_span,
            });
        }
        out
    }

    fn features_checked(
        &mut self,
        raw: Vec<RawFeature>,
        dimensions: &[Dimension],
        lifecycle: &[LifecycleState],
    ) -> Vec<Feature> {
        let keys: Vec<String> = raw.iter().map(|f| f.key.clone()).collect();
        let mut out = Vec::new();
        for f in raw {
            // Where the spec text starts in the file: after the opening quote
            // (one or three characters, plus a newline that a multi-line
            // string drops). Specs contain no escapes in practice.
            let raw_text = self.text_of(f.value_span);
            let quote = if raw_text.starts_with("\"\"\"") || raw_text.starts_with("'''") {
                3
            } else {
                1
            };
            let offset = f.value_span.start() + quote;
            let spec: AvailabilitySpec = match parse_availability(&f.text, offset) {
                Ok(spec) => spec,
                Err(e) => {
                    let span = if f.text.is_empty() {
                        f.value_span
                    } else {
                        e.span
                    };
                    self.push(
                        self.issue(diagnostics::MODEL_AVAILABILITY_SYNTAX, span)
                            .with_arg("key", f.key.as_str())
                            .with_arg("spec", f.text.as_str())
                            .with_arg("detail", e.detail),
                    );
                    continue;
                }
            };
            if let Some(n) = spec.bare_name()
                && keys.contains(&n.text)
            {
                self.push(
                    self.issue(diagnostics::MODEL_FEATURE_NESTED, n.span)
                        .with_arg("key", f.key.as_str())
                        .with_arg("other", n.text.as_str()),
                );
                continue;
            }
            for problem in check_entries(dimensions, lifecycle, &spec.entries) {
                self.push(self.feature_problem(&f.key, problem));
            }
            out.push(Feature {
                key: f.key,
                name: f.name,
                available_text: f.text,
                available: spec,
            });
        }
        out
    }

    fn feature_problem(&self, key: &str, problem: AvailabilityProblem) -> Issue {
        match problem {
            AvailabilityProblem::UnknownTarget(n) => self
                .issue(diagnostics::MODEL_AVAILABILITY_UNKNOWN_NAME, n.span)
                .with_arg("key", key)
                .with_arg("name", n.text),
            AvailabilityProblem::UnknownState(n) => self
                .issue(diagnostics::MODEL_AVAILABILITY_UNKNOWN_NAME, n.span)
                .with_variant("state")
                .with_arg("key", key)
                .with_arg("name", n.text),
            AvailabilityProblem::VersionlessVersion { target, version } => self
                .issue(diagnostics::MODEL_AVAILABILITY_VERSIONLESS, version.span)
                .with_arg("key", key)
                .with_arg("target", target.text),
            AvailabilityProblem::DimensionVersion {
                target,
                version,
                example,
            } => self
                .issue(diagnostics::MODEL_AVAILABILITY_VERSIONLESS, version.span)
                .with_variant("dimension")
                .with_arg("key", key)
                .with_arg("target", target.text)
                .with_arg("example", example),
            AvailabilityProblem::HistoryOrder {
                target,
                later,
                earlier,
                span,
            } => self
                .issue(diagnostics::MODEL_AVAILABILITY_HISTORY_ORDER, span)
                .with_arg("key", key)
                .with_arg("target", target.text)
                .with_arg("later", later)
                .with_arg("earlier", earlier),
        }
    }
}

/// Which name grammar to check.
#[derive(Clone, Copy)]
pub(crate) enum NameRule {
    Key,
    NameWord,
    Widget,
    Build,
}

pub(crate) struct ProjectInfo {
    pub value: Project,
    pub content_span: Option<Span>,
    pub output_span: Option<Span>,
}

struct DimSpans {
    name_span: Span,
    value_spans: Vec<Span>,
}

struct RawFeature {
    key: String,
    key_span: Span,
    name: String,
    text: String,
    value_span: Span,
}

fn is_absolute(s: &str) -> bool {
    let b = s.as_bytes();
    s.starts_with('/')
        || s.starts_with('\\')
        || (b.len() >= 2 && b[0].is_ascii_alphabetic() && b[1] == b':')
}

/// Removes `.` and resolves `..` lexically.
fn normalize(path: &Path) -> PathBuf {
    let mut out = PathBuf::new();
    for c in path.components() {
        match c {
            Component::CurDir => {}
            Component::ParentDir => {
                if !out.pop() {
                    out.push("..");
                }
            }
            other => out.push(other.as_os_str()),
        }
    }
    out
}
