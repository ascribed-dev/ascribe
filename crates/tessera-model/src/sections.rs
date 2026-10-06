//! Content types, fragments, glossary, images, widgets, consumer, builds,
//! and editor.

use std::collections::HashMap;

use tessera_core::availability::Version;
use tessera_core::{
    AttributeSchema, Attributes, Binding, DirectiveSchema, Forms, Origin, Primary, Span, TitleRule,
    diagnostics, reserved,
};
use toml::de::DeValue;

use crate::loader::{Loader, NameRule};
use crate::model::*;
use crate::names::{is_version, list};
use crate::pattern::{Pattern, PatternError};
use crate::toml_util::{V, entries, join, sp};
use crate::types::{Field, FieldType, FrontmatterSchema, SchemaOwner};

/// HTML's reserved custom-element names.
const HTML_RESERVED_ELEMENTS: &[&str] = &[
    "annotation-xml",
    "color-profile",
    "font-face",
    "font-face-src",
    "font-face-uri",
    "font-face-format",
    "font-face-name",
    "missing-glyph",
];

impl Loader<'_> {
    pub fn patterns(&mut self, path: &str, v: &V<'_>) -> Vec<Pattern> {
        let mut out = Vec::new();
        let Some(items) = self.strings(path, v) else {
            return out;
        };
        for (text, span) in items {
            match Pattern::new(&text) {
                Ok(p) => out.push(p),
                Err(e) => {
                    let issue = self.issue(diagnostics::MODEL_PATTERN_SYNTAX, span);
                    let issue = match e {
                        PatternError::LeadingSlash => issue
                            .with_variant("leading-slash")
                            .with_arg("pattern", text),
                        PatternError::Parent => {
                            issue.with_variant("parent").with_arg("pattern", text)
                        }
                        PatternError::Syntax(detail) => {
                            issue.with_arg("pattern", text).with_arg("detail", detail)
                        }
                    };
                    self.push(issue);
                }
            }
        }
        out
    }

    // ---- fragments and types --------------------------------------------------

    pub fn fragments(&mut self, v: Option<&V<'_>>) -> Fragments {
        let mut out = Fragments {
            patterns: Vec::new(),
            frontmatter: FrontmatterSchema {
                owner: SchemaOwner::Fragment,
                fields: Vec::new(),
            },
        };
        let Some(v) = v else { return out };
        let Some(t) = self.as_table("fragments", v) else {
            return out;
        };
        self.check_keys("fragments", t, &["patterns", "frontmatter"]);
        if let Some(p) = t.get("patterns") {
            out.patterns = self.patterns("fragments.patterns", p);
        }
        if let Some(f) = t.get("frontmatter")
            && let Some(ft) = self.as_table("fragments.frontmatter", f)
        {
            let reserved: &dyn Fn(&str) -> bool = &|n| matches!(n, "available" | "variant");
            for (name, span, _) in entries(ft) {
                if reserved(name) {
                    self.push(
                        self.issue(diagnostics::MODEL_FIELD_RESERVED, span)
                            .with_variant("fragment")
                            .with_arg("field", name),
                    );
                }
            }
            out.frontmatter.fields = self.field_table("fragments.frontmatter", ft, Some(reserved));
        }
        out
    }

    pub fn types(&mut self, v: Option<&V<'_>>, _fragments: &Fragments) -> Vec<ContentType> {
        let mut out: Vec<ContentType> = Vec::new();
        let mut default_at: Option<String> = None;
        if let Some(v) = v
            && let Some(t) = self.as_table("types", v)
        {
            for (name, name_span, item) in entries(t) {
                let path = join("types", name);
                self.name_ok(name, name_span, "content type", NameRule::Key);
                let Some(tt) = self.as_table(&path, item) else {
                    continue;
                };
                self.check_keys(&path, tt, &["files", "default", "frontmatter"]);
                let files = tt
                    .get("files")
                    .map(|f| self.patterns(&format!("{path}.files"), f))
                    .unwrap_or_default();
                let default_v = tt.get("default");
                let default = default_v
                    .and_then(|d| self.boolean(&format!("{path}.default"), d))
                    .unwrap_or(false);
                if default {
                    match &default_at {
                        Some(first) => self.push(
                            self.issue(
                                diagnostics::MODEL_TYPE_MULTIPLE_DEFAULTS,
                                default_v.map_or(name_span, sp),
                            )
                            .with_arg("a", first.as_str())
                            .with_arg("b", name),
                        ),
                        None => default_at = Some(name.to_owned()),
                    }
                }
                if files.is_empty() && !default {
                    self.push(
                        self.issue(diagnostics::MODEL_TYPE_UNREACHABLE, name_span)
                            .with_arg("type", name),
                    );
                }
                let fm_path = format!("{path}.frontmatter");
                let mut fields = Vec::new();
                if let Some(fm) = self.require(&path, tt, sp(item), "frontmatter")
                    && let Some(ft) = self.as_table(&fm_path, fm)
                {
                    // Under the `astro` profile, `slug` is
                    // the entry id Astro's loader uses instead of the path.
                    // The site output writes `inline` fields' formatted form
                    // under `formatted`.
                    let reserved: &dyn Fn(&str) -> bool =
                        &|n| matches!(n, "available" | "variant" | "slug" | "formatted");
                    for (fname, span, _) in entries(ft) {
                        if reserved(fname) {
                            let issue = self
                                .issue(diagnostics::MODEL_FIELD_RESERVED, span)
                                .with_arg("field", fname)
                                .with_arg("type", name);
                            self.push(match fname {
                                "slug" => issue.with_variant("slug"),
                                "formatted" => issue.with_variant("formatted"),
                                _ => issue,
                            });
                        }
                    }
                    fields = self.field_table(&fm_path, ft, Some(reserved));
                    self.check_title(name, &fields, ft, sp(fm));
                }
                out.push(ContentType {
                    name: name.to_owned(),
                    files,
                    default,
                    frontmatter: FrontmatterSchema {
                        owner: SchemaOwner::Type(name.to_owned()),
                        fields,
                    },
                });
            }
        }
        if out.is_empty() {
            out.push(ContentType {
                name: "page".into(),
                files: Vec::new(),
                default: true,
                frontmatter: FrontmatterSchema {
                    owner: SchemaOwner::Type("page".into()),
                    fields: vec![Field {
                        name: "title".into(),
                        ty: FieldType::String,
                        required: true,
                        default: None,
                        phrases: false,
                        inline: None,
                        description: None,
                    }],
                },
            });
        }
        out
    }

    /// Every page type declares `title` as a required string.
    fn check_title(
        &mut self,
        type_name: &str,
        fields: &[Field],
        table: &toml::de::DeTable<'_>,
        fm_span: Span,
    ) {
        match fields.iter().find(|f| f.name == "title") {
            Some(f) if f.ty == FieldType::String && f.required => {}
            Some(f) => {
                let span = table.get("title").map_or(fm_span, sp);
                let found = if f.required {
                    f.ty.describe()
                } else {
                    format!("{} (optional)", f.ty.describe())
                };
                self.push(
                    self.issue(diagnostics::MODEL_TYPE_TITLE, span)
                        .with_variant("not-string")
                        .with_arg("type", type_name)
                        .with_arg("found", found),
                );
            }
            // A `title` that failed to build already reported why.
            None if table.get("title").is_some() => {}
            None => self.push(
                self.issue(diagnostics::MODEL_TYPE_TITLE, fm_span)
                    .with_arg("type", type_name),
            ),
        }
    }

    // ---- glossary ---------------------------------------------------------------

    pub fn glossary(
        &mut self,
        v: Option<&V<'_>>,
        project: &crate::loader::ProjectInfo,
        fragments: &Fragments,
    ) -> Glossary {
        let mut out = Glossary {
            match_mode: GlossaryMatch::First,
            case_sensitive: false,
            terms: Vec::new(),
        };
        let Some(v) = v else { return out };
        let Some(t) = self.as_table("glossary", v) else {
            return out;
        };
        self.check_keys("glossary", t, &["match", "case-sensitive", "terms"]);
        if let Some(m) = t.get("match")
            && let Some(s) = self.choice("glossary.match", m, &["first", "every"])
        {
            out.match_mode = if s == "every" {
                GlossaryMatch::Every
            } else {
                GlossaryMatch::First
            };
        }
        if let Some(c) = t.get("case-sensitive") {
            out.case_sensitive = self.boolean("glossary.case-sensitive", c).unwrap_or(false);
        }
        let Some(terms) = t.get("terms") else {
            return out;
        };
        let Some(tt) = self.as_table("glossary.terms", terms) else {
            return out;
        };
        let mut spans: Vec<(String, Span)> = Vec::new();
        for (id, id_span, item) in entries(tt) {
            let path = join("glossary.terms", id);
            self.name_ok(id, id_span, "glossary term", NameRule::Key);
            let Some(term_t) = self.as_table(&path, item) else {
                continue;
            };
            self.check_keys(
                &path,
                term_t,
                &[
                    "term",
                    "aliases",
                    "definition",
                    "link",
                    "case-sensitive",
                    "match",
                ],
            );
            let term = self
                .require(&path, term_t, sp(item), "term")
                .and_then(|x| self.string(&format!("{path}.term"), x, true));
            let definition = self
                .require(&path, term_t, sp(item), "definition")
                .and_then(|x| self.string(&format!("{path}.definition"), x, true));
            let aliases = term_t
                .get("aliases")
                .and_then(|a| self.strings(&format!("{path}.aliases"), a))
                .unwrap_or_default();
            let case_sensitive = term_t
                .get("case-sensitive")
                .and_then(|c| self.boolean(&format!("{path}.case-sensitive"), c))
                .unwrap_or(out.case_sensitive);
            let match_mode = term_t
                .get("match")
                .and_then(|m| {
                    self.choice(&format!("{path}.match"), m, &["first", "every", "marked"])
                })
                .map_or(out.match_mode, |s| match s.as_str() {
                    "every" => GlossaryMatch::Every,
                    "marked" => GlossaryMatch::Marked,
                    _ => GlossaryMatch::First,
                });
            let link = term_t.get("link").and_then(|l| {
                let text = self.string(&format!("{path}.link"), l, true)?;
                self.glossary_link(id, &text, sp(l), project, fragments)
            });
            let (Some(term), Some(definition)) = (term, definition) else {
                continue;
            };
            spans.push((term.clone(), id_span));
            out.terms.push(GlossaryTerm {
                id: id.to_owned(),
                term,
                aliases: aliases.iter().map(|(a, _)| a.clone()).collect(),
                definition,
                link,
                case_sensitive,
                match_mode,
            });
        }
        self.glossary_duplicates(&out, tt);
        out
    }

    fn glossary_link(
        &mut self,
        id: &str,
        text: &str,
        span: Span,
        project: &crate::loader::ProjectInfo,
        fragments: &Fragments,
    ) -> Option<(String, Option<String>)> {
        let (path_part, anchor) = match text.split_once('#') {
            Some((p, a)) => (p, Some(a.to_owned())),
            None => (text, None),
        };
        let rel = path_part.trim_start_matches('/');
        let is_fragment = rel.split('/').any(|s| s.starts_with('_'))
            || fragments.patterns.iter().any(|p| p.matches(rel));
        if is_fragment {
            self.push(
                self.issue(diagnostics::MODEL_GLOSSARY_LINK, span)
                    .with_variant("fragment")
                    .with_arg("id", id)
                    .with_arg("path", path_part),
            );
            return None;
        }
        if let Some(dir) = self.project_dir {
            let file = dir.join(&project.value.content_root).join(rel);
            if !file.is_file() {
                self.push(
                    self.issue(diagnostics::MODEL_GLOSSARY_LINK, span)
                        .with_arg("id", id)
                        .with_arg("path", path_part),
                );
                return None;
            }
        }
        Some((rel.to_owned(), anchor))
    }

    fn glossary_duplicates(&mut self, g: &Glossary, terms: &toml::de::DeTable<'_>) {
        let mut seen: HashMap<String, Vec<(&GlossaryTerm, String)>> = HashMap::new();
        for t in &g.terms {
            let texts = std::iter::once(&t.term).chain(t.aliases.iter());
            for text in texts {
                let bucket = seen.entry(text.to_lowercase()).or_default();
                if let Some((other, _)) = bucket.iter().find(|(o, ot)| {
                    o.id != t.id && (ot == text || !o.case_sensitive || !t.case_sensitive)
                }) {
                    let span = terms
                        .get_key_value(t.id.as_str())
                        .map_or(Span::empty(0), |(k, _)| sp(k));
                    let issue = self
                        .issue(diagnostics::MODEL_GLOSSARY_DUPLICATE_TERM, span)
                        .with_arg("text", text.as_str())
                        .with_arg("a", other.id.as_str())
                        .with_arg("b", t.id.as_str());
                    self.push(issue);
                    continue;
                }
                bucket.push((t, text.clone()));
            }
        }
    }

    // ---- images and widgets --------------------------------------------------------

    pub fn images(&mut self, v: Option<&V<'_>>) -> Vec<AttributeSchema> {
        let Some(v) = v else { return Vec::new() };
        let Some(t) = self.as_table("images", v) else {
            return Vec::new();
        };
        self.check_keys("images", t, &["attributes"]);
        let Some(a) = t.get("attributes") else {
            return Vec::new();
        };
        let Some(at) = self.as_table("images.attributes", a) else {
            return Vec::new();
        };
        self.attribute_table(
            "images.attributes",
            at,
            &reserved::is_reserved_image_attribute,
            None,
        )
    }

    pub fn widgets(&mut self, v: Option<&V<'_>>) -> Vec<Widget> {
        let mut out = Vec::new();
        let Some(v) = v else { return out };
        let Some(t) = self.as_table("widgets", v) else {
            return out;
        };
        for (name, name_span, item) in entries(t) {
            if let Some(w) = self.widget(name, name_span, item) {
                out.push(w);
            }
        }
        out
    }

    fn widget(&mut self, name: &str, name_span: Span, item: &V<'_>) -> Option<Widget> {
        let path = join("widgets", name);
        let mut ok = self.name_ok(name, name_span, "widget", NameRule::Widget);
        if ok && name.starts_with("ascribe-") {
            self.push(
                self.issue(diagnostics::MODEL_WIDGET_RESERVED_NAME, name_span)
                    .with_arg("name", name),
            );
            ok = false;
        } else if ok && HTML_RESERVED_ELEMENTS.contains(&name) {
            self.push(
                self.issue(diagnostics::MODEL_WIDGET_RESERVED_NAME, name_span)
                    .with_variant("html")
                    .with_arg("name", name),
            );
            ok = false;
        }
        let t = self.as_table(&path, item)?;
        self.check_keys(
            &path,
            t,
            &[
                "forms",
                "primary",
                "binding",
                "title",
                "groupable",
                "attributes",
                "plain-fallback",
                "plain-content",
                "description",
            ],
        );

        // forms
        let mut forms: Option<Forms> = None;
        if let Some(f) = self.require(&path, t, sp(item), "forms")
            && let Some(items) = self.strings(&format!("{path}.forms"), f)
        {
            let mut line = false;
            let mut container = false;
            let mut valid = !items.is_empty();
            for (s, _) in &items {
                match s.as_str() {
                    "line" if !line => line = true,
                    "container" if !container => container = true,
                    _ => valid = false,
                }
            }
            if valid {
                forms = Some(Forms { line, container });
            } else {
                self.push(self.issue(diagnostics::MODEL_WIDGET_FORMS, sp(f)));
            }
        }

        // primary, title, binding
        let primary_v = t.get("primary");
        let primary = match primary_v {
            Some(p) => self
                .choice(
                    &format!("{path}.primary"),
                    p,
                    &["none", "identifier", "identifier?", "text", "text?"],
                )
                .map(|s| match s.as_str() {
                    "identifier" => Primary::Identifier { required: true },
                    "identifier?" => Primary::Identifier { required: false },
                    "text" => Primary::Text { required: true },
                    "text?" => Primary::Text { required: false },
                    _ => Primary::None,
                }),
            None => Some(Primary::None),
        };
        let title = match t.get("title") {
            Some(x) => self
                .choice(
                    &format!("{path}.title"),
                    x,
                    &["none", "accepted", "required"],
                )
                .map(|s| match s.as_str() {
                    "accepted" => TitleRule::Accepted,
                    "required" => TitleRule::Required,
                    _ => TitleRule::None,
                }),
            None => Some(TitleRule::None),
        };
        let binding_v = t.get("binding");
        let binding = binding_v.and_then(|b| {
            self.choice(
                &format!("{path}.binding"),
                b,
                &["self", "heading", "block", "heading-or-block"],
            )
            .map(|s| match s.as_str() {
                "self" => Binding::SelfBound,
                "heading" => Binding::Heading,
                "block" => Binding::Block,
                _ => Binding::HeadingOrBlock,
            })
        });
        let groupable_v = t.get("groupable");
        let groupable = groupable_v
            .and_then(|g| self.boolean(&format!("{path}.groupable"), g))
            .unwrap_or(false);

        if let Some(forms) = forms {
            if forms.line && binding_v.is_none() {
                self.push(
                    self.issue(diagnostics::MODEL_WIDGET_BINDING, name_span)
                        .with_arg("name", name),
                );
            }
            if !forms.line
                && let Some(b) = binding_v
            {
                self.push(
                    self.issue(diagnostics::MODEL_WIDGET_BINDING, sp(b))
                        .with_variant("container-only")
                        .with_arg("name", name),
                );
            }
            if let Some(primary) = primary {
                let span = primary_v.map_or(name_span, sp);
                if forms.container && !forms.line && primary != Primary::None {
                    self.push(
                        self.issue(diagnostics::MODEL_WIDGET_CONTAINER_PRIMARY, span)
                            .with_variant("container-only")
                            .with_arg("name", name),
                    );
                } else if forms.container && primary.is_required() {
                    let kind = match primary {
                        Primary::Identifier { .. } => "identifier",
                        _ => "text",
                    };
                    self.push(
                        self.issue(diagnostics::MODEL_WIDGET_CONTAINER_PRIMARY, span)
                            .with_arg("name", name)
                            .with_arg("kind", kind),
                    );
                }
            }
            if groupable && (forms.line || !forms.container) {
                self.push(
                    self.issue(
                        diagnostics::MODEL_WIDGET_GROUPABLE_FORM,
                        groupable_v.map_or(name_span, sp),
                    )
                    .with_arg("name", name),
                );
            }
        }

        // attributes
        let attributes = match t.get("attributes") {
            Some(a) => match self.as_table(&format!("{path}.attributes"), a) {
                Some(at) => self.attribute_table(
                    &format!("{path}.attributes"),
                    at,
                    &reserved::is_reserved_widget_attribute,
                    Some(name),
                ),
                None => Vec::new(),
            },
            None => Vec::new(),
        };

        let plain_fallback = t
            .get("plain-fallback")
            .and_then(|p| self.string(&format!("{path}.plain-fallback"), p, false));
        let mut plain_content = PlainContent::Keep;
        if let Some(pc) = t.get("plain-content")
            && let Some(s) = self.choice(&format!("{path}.plain-content"), pc, &["keep", "drop"])
        {
            if s == "drop" {
                plain_content = PlainContent::Drop;
            }
            let wraps = forms.is_some_and(|f| f.container)
                || matches!(binding, Some(Binding::Block | Binding::HeadingOrBlock));
            if !wraps && forms.is_some() {
                self.push(
                    self.issue(diagnostics::MODEL_WIDGET_PLAIN_CONTENT, sp(pc))
                        .with_arg("name", name),
                );
            }
        }
        let description = t
            .get("description")
            .and_then(|d| self.string(&format!("{path}.description"), d, true));

        let (forms, primary, title) = (forms?, primary?, title?);
        if !ok {
            return None;
        }
        Some(Widget {
            schema: DirectiveSchema {
                name: name.to_owned(),
                origin: Origin::Widget,
                forms,
                primary,
                binding: if forms.line { binding } else { None },
                title,
                groupable,
                attributes: Attributes::Declared(attributes),
                description,
            },
            plain_fallback,
            plain_content,
        })
    }

    // ---- consumer ---------------------------------------------------------------------

    pub fn consumer(&mut self, v: Option<&V<'_>>) -> Consumer {
        let mut c = Consumer {
            profile: "astro".into(),
            site: None,
            base_path: "/".into(),
            trailing_slash: TrailingSlash::Always,
            slugger: "github".into(),
            html: true,
        };
        let Some(v) = v else { return c };
        let Some(t) = self.as_table("consumer", v) else {
            return c;
        };
        self.check_keys(
            "consumer",
            t,
            &[
                "profile",
                "site",
                "base-path",
                "trailing-slash",
                "slugger",
                "html",
            ],
        );
        if let Some(p) = t.get("profile")
            && let Some(s) = self.choice("consumer.profile", p, &["astro"])
        {
            c.profile = s;
        }
        if let Some(s) = t.get("site")
            && let Some(text) = self.string("consumer.site", s, true)
        {
            if is_origin(&text) {
                c.site = Some(text);
            } else {
                self.push(self.issue(diagnostics::MODEL_CONSUMER_SITE, sp(s)));
            }
        }
        if let Some(b) = t.get("base-path")
            && let Some(text) = self.string("consumer.base-path", b, true)
        {
            if text.starts_with('/') {
                c.base_path = text;
            } else {
                self.push(self.issue(diagnostics::MODEL_CONSUMER_BASE_PATH, sp(b)));
            }
        }
        if let Some(s) = t.get("trailing-slash")
            && let Some(text) = self.choice("consumer.trailing-slash", s, &["always", "never"])
        {
            c.trailing_slash = if text == "never" {
                TrailingSlash::Never
            } else {
                TrailingSlash::Always
            };
        }
        if let Some(s) = t.get("slugger")
            && let Some(text) = self.choice("consumer.slugger", s, &["github"])
        {
            c.slugger = text;
        }
        if let Some(h) = t.get("html")
            && let Some(value) = self.boolean("consumer.html", h)
        {
            if value {
                c.html = true;
            } else {
                self.push(
                    self.issue(diagnostics::MODEL_CONSUMER_UNSUPPORTED, sp(h))
                        .with_arg("profile", c.profile.as_str())
                        .with_arg("key", "html")
                        .with_arg("value", "false")
                        .with_arg("values", "true"),
                );
            }
        }
        c
    }

    // ---- builds and editor ----------------------------------------------------------------

    pub fn builds(&mut self, v: Option<&V<'_>>, dimensions: &[Dimension]) -> Vec<Build> {
        let mut out: Vec<Build> = Vec::new();
        if let Some(v) = v
            && let Some(t) = self.as_table("builds", v)
        {
            let mut seen: Vec<String> = Vec::new();
            for (name, name_span, item) in entries(t) {
                let path = join("builds", name);
                let name_ok = self.name_ok(name, name_span, "build", NameRule::Build);
                if let Some(prev) = seen.iter().find(|s| s.eq_ignore_ascii_case(name)) {
                    self.push(
                        self.issue(diagnostics::MODEL_BUILD_NAME_CASE, name_span)
                            .with_arg("a", prev.as_str())
                            .with_arg("b", name),
                    );
                }
                seen.push(name.to_owned());
                let Some(bt) = self.as_table(&path, item) else {
                    continue;
                };
                self.check_keys(&path, bt, &["variants", "availability"]);
                let variants = self.variant_mode(name, &path, bt.get("variants"), dimensions);
                let availability =
                    self.availability_mode(name, &path, bt.get("availability"), dimensions);
                if let (Some(variants), Some(availability), true) =
                    (variants, availability, name_ok)
                {
                    self.excluded_warning(name, &variants, &availability, dimensions, bt);
                    out.push(Build {
                        name: name.to_owned(),
                        variants,
                        availability,
                    });
                }
            }
        }
        if out.is_empty()
            && v.is_none_or(|v| matches!(v.get_ref(), DeValue::Table(t) if t.is_empty()))
        {
            out.push(Build {
                name: "site".into(),
                variants: VariantMode::Switch,
                availability: AvailabilityMode::Badge,
            });
        }
        out
    }

    fn variant_mode(
        &mut self,
        build: &str,
        path: &str,
        v: Option<&V<'_>>,
        dimensions: &[Dimension],
    ) -> Option<VariantMode> {
        let Some(v) = v else {
            return Some(VariantMode::Switch);
        };
        match v.get_ref() {
            DeValue::String(s) if s.as_ref() == "switch" => Some(VariantMode::Switch),
            DeValue::Table(t) => {
                if t.is_empty() {
                    self.push(
                        self.issue(diagnostics::MODEL_BUILD_VARIANTS, sp(v))
                            .with_variant("empty")
                            .with_arg("build", build),
                    );
                    return None;
                }
                let mut select = Vec::new();
                let mut ok = true;
                for (dim, dim_span, val) in entries(t) {
                    let values: Vec<(String, Span)> = match val.get_ref() {
                        DeValue::String(s) => vec![(s.to_string(), sp(val))],
                        DeValue::Array(a) if !a.is_empty() => {
                            let mut vals = Vec::new();
                            for item in a.iter() {
                                match item.get_ref() {
                                    DeValue::String(s) => vals.push((s.to_string(), sp(item))),
                                    _ => {
                                        self.push(
                                            self.issue(diagnostics::MODEL_BUILD_VARIANTS, sp(item)),
                                        );
                                        ok = false;
                                    }
                                }
                            }
                            vals
                        }
                        _ => {
                            self.push(self.issue(diagnostics::MODEL_BUILD_VARIANTS, sp(val)));
                            ok = false;
                            continue;
                        }
                    };
                    let Some(d) = dimensions.iter().find(|d| d.name == dim) else {
                        self.push(
                            self.issue(diagnostics::MODEL_BUILD_UNKNOWN_DIMENSION, dim_span)
                                .with_arg("build", build)
                                .with_arg("dimension", dim),
                        );
                        ok = false;
                        continue;
                    };
                    let names: Vec<String> = d.values.iter().map(|x| x.value.clone()).collect();
                    for (value, span) in &values {
                        if !names.contains(value) {
                            self.push(
                                self.issue(diagnostics::MODEL_BUILD_UNKNOWN_VALUE, *span)
                                    .with_arg("build", build)
                                    .with_arg("value", value.as_str())
                                    .with_arg("dimension", dim)
                                    .with_arg("values", list(&names)),
                            );
                            ok = false;
                        }
                    }
                    select.push((dim.to_owned(), values.into_iter().map(|(v, _)| v).collect()));
                }
                let _ = path;
                ok.then_some(VariantMode::Select(select))
            }
            _ => {
                self.push(self.issue(diagnostics::MODEL_BUILD_VARIANTS, sp(v)));
                None
            }
        }
    }

    fn availability_mode(
        &mut self,
        build: &str,
        _path: &str,
        v: Option<&V<'_>>,
        dimensions: &[Dimension],
    ) -> Option<AvailabilityMode> {
        let Some(v) = v else {
            return Some(AvailabilityMode::Badge);
        };
        let bad = |l: &mut Self, span: Span| {
            l.push(l.issue(diagnostics::MODEL_BUILD_AVAILABILITY, span));
            None
        };
        match v.get_ref() {
            DeValue::String(s) if s.as_ref() == "badge" => Some(AvailabilityMode::Badge),
            DeValue::Table(t) if t.len() == 1 && t.get("filter").is_some() => {
                let f = t.get("filter")?;
                let DeValue::String(text) = f.get_ref() else {
                    return bad(self, sp(f));
                };
                self.filter(build, text, sp(f), dimensions)
            }
            _ => bad(self, sp(v)),
        }
    }

    fn filter(
        &mut self,
        build: &str,
        text: &str,
        span: Span,
        dimensions: &[Dimension],
    ) -> Option<AvailabilityMode> {
        let tokens: Vec<&str> = text.split_whitespace().collect();
        let target = match tokens.as_slice() {
            [t] | [t, _] if crate::names::is_name_word(t) => *t,
            _ => {
                self.push(self.issue(diagnostics::MODEL_BUILD_AVAILABILITY, span));
                return None;
            }
        };
        let value = dimensions
            .iter()
            .flat_map(|d| &d.values)
            .find(|v| v.value == target);
        let Some(value) = value else {
            let mut issue = self
                .issue(diagnostics::MODEL_BUILD_FILTER_TARGET, span)
                .with_arg("build", build)
                .with_arg("target", target);
            if let Some(d) = dimensions.iter().find(|d| d.name == target) {
                let values: Vec<String> = d.values.iter().map(|v| v.value.clone()).collect();
                issue = issue
                    .with_variant("dimension")
                    .with_arg("values", list(&values));
            }
            self.push(issue);
            return None;
        };
        match (tokens.get(1), value.versionless) {
            (None, false) => {
                self.push(
                    self.issue(diagnostics::MODEL_BUILD_FILTER_VERSION, span)
                        .with_arg("build", build)
                        .with_arg("target", target),
                );
                None
            }
            (Some(_), true) => {
                self.push(
                    self.issue(diagnostics::MODEL_BUILD_FILTER_VERSION, span)
                        .with_variant("versionless")
                        .with_arg("build", build)
                        .with_arg("target", target),
                );
                None
            }
            (None, true) => Some(AvailabilityMode::Filter {
                target: target.to_owned(),
                version: None,
            }),
            (Some(version), false) => {
                if !is_version(version) {
                    self.push(
                        self.issue(diagnostics::MODEL_BUILD_FILTER_VERSION, span)
                            .with_variant("invalid")
                            .with_arg("build", build)
                            .with_arg("version", *version),
                    );
                    return None;
                }
                let parsed = tessera_core::availability::parse_availability(
                    &format!("{target} {version}"),
                    0,
                )
                .ok()
                .and_then(|s| match s.entries.into_iter().next()?.detail {
                    tessera_core::availability::Detail::Version(v) => Some(v),
                    _ => None,
                });
                match parsed {
                    Some(Version {
                        text, components, ..
                    }) => Some(AvailabilityMode::Filter {
                        target: target.to_owned(),
                        version: Some(Version {
                            text,
                            components,
                            span,
                        }),
                    }),
                    None => {
                        // Too large for a version number.
                        self.push(
                            self.issue(diagnostics::MODEL_BUILD_FILTER_VERSION, span)
                                .with_variant("invalid")
                                .with_arg("build", build)
                                .with_arg("version", *version),
                        );
                        None
                    }
                }
            }
        }
    }

    fn excluded_warning(
        &mut self,
        build: &str,
        variants: &VariantMode,
        availability: &AvailabilityMode,
        dimensions: &[Dimension],
        table: &toml::de::DeTable<'_>,
    ) {
        let (VariantMode::Select(select), AvailabilityMode::Filter { target, .. }) =
            (variants, availability)
        else {
            return;
        };
        let Some(dim) = dimensions
            .iter()
            .find(|d| d.values.iter().any(|v| v.value == *target))
        else {
            return;
        };
        let Some((_, kept)) = select.iter().find(|(d, _)| *d == dim.name) else {
            return;
        };
        if !kept.contains(target) {
            let span = table.get("availability").map_or(Span::empty(0), sp);
            self.warnings.push(
                self.issue(diagnostics::MODEL_BUILD_FILTER_EXCLUDED, span)
                    .with_arg("build", build)
                    .with_arg("target", target.as_str())
                    .with_arg("dimension", dim.name.as_str())
                    .with_arg("values", list(kept)),
            );
        }
    }

    pub fn editor(
        &mut self,
        v: Option<&V<'_>>,
        builds_table: Option<&V<'_>>,
        builds: &[Build],
    ) -> String {
        let names: Vec<String> = builds.iter().map(|b| b.name.clone()).collect();
        let mut chosen: Option<String> = None;
        let mut at = builds_table.map_or(Span::empty(0), sp);
        if let Some(v) = v
            && let Some(t) = self.as_table("editor", v)
        {
            at = sp(v);
            self.check_keys("editor", t, &["build"]);
            if let Some(b) = t.get("build")
                && let Some(name) = self.string("editor.build", b, true)
            {
                if names.contains(&name) {
                    chosen = Some(name);
                } else {
                    self.push(
                        self.issue(diagnostics::MODEL_EDITOR_BUILD_UNKNOWN, sp(b))
                            .with_arg("build", name)
                            .with_arg("builds", list(&names)),
                    );
                }
            }
        }
        if let Some(name) = chosen {
            return name;
        }
        let default = match names.as_slice() {
            [only] => Some(only.clone()),
            _ => names.iter().find(|n| *n == "site").cloned(),
        };
        match default {
            Some(d) => d,
            None => {
                let explicit = v
                    .and_then(|v| match v.get_ref() {
                        DeValue::Table(t) => t.get("build"),
                        _ => None,
                    })
                    .is_some();
                if !explicit && !names.is_empty() {
                    self.push(self.issue(diagnostics::MODEL_EDITOR_BUILD_REQUIRED, at));
                }
                names.first().cloned().unwrap_or_else(|| "site".into())
            }
        }
    }
}

/// `http` or `https`, a host, and nothing after it.
fn is_origin(s: &str) -> bool {
    let Some(rest) = s
        .strip_prefix("https://")
        .or_else(|| s.strip_prefix("http://"))
    else {
        return false;
    };
    !rest.is_empty()
        && !rest
            .chars()
            .any(|c| matches!(c, '/' | '?' | '#') || c.is_whitespace())
}
