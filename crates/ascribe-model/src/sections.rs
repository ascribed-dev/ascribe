//! Content types, fragments, glossary, images, widgets, consumer, builds,
//! and editor.

use std::cmp::Ordering;

use ascribe_core::availability::Version;
use ascribe_core::{
    AttributeSchema, Attributes, Binding, Date, DirectiveSchema, Forms, Origin, Primary, Span,
    TitleRule, diagnostics, reserved,
};
use toml::de::DeValue;

use crate::loader::{DimSpans, Loader, NameRule, is_absolute};
use crate::model::*;
use crate::names::{is_version, list};
use crate::pattern::{Pattern, PatternError};
use crate::settings;
use crate::toml_util::{V, entries, join, sp};
use crate::types::{Field, FieldRole, FieldType, FrontmatterSchema, SchemaOwner};

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
        self.check_table("fragments", t, "fragments");
        if let Some(p) = t.get("patterns") {
            out.patterns = self.patterns("fragments.patterns", p);
        }
        if let Some(f) = t.get("frontmatter")
            && let Some(ft) = self.as_table("fragments.frontmatter", f)
        {
            let reserved: &dyn Fn(&str) -> bool = &|n| {
                matches!(
                    n,
                    "available"
                        | "variant"
                        | "permalink"
                        | "listing"
                        | "replaces"
                        | "intended"
                        | "type"
                        | "nav-label"
                )
            };
            for (name, span, _) in entries(ft) {
                if reserved(name) {
                    // Fragments accept `intended`, and can't use the others.
                    let variant = if name == "intended" {
                        "fragment-accepted"
                    } else {
                        "fragment"
                    };
                    self.push(
                        self.issue(diagnostics::MODEL_FIELD_RESERVED, span)
                            .with_variant(variant)
                            .with_arg("field", name),
                    );
                }
            }
            out.frontmatter.fields = self.field_table("fragments.frontmatter", ft, Some(reserved));
        }
        out
    }

    /// `[frontmatter]`: the fields every page type has (SPEC §7.2), as
    /// written. A field named `title` that isn't a required string is
    /// reported and left out.
    pub fn shared_frontmatter(&mut self, v: Option<&V<'_>>) -> Vec<Field> {
        let Some(v) = v else { return Vec::new() };
        let Some(ft) = self.as_table("frontmatter", v) else {
            return Vec::new();
        };
        self.page_fields("frontmatter", None, ft)
    }

    pub fn types(&mut self, v: Option<&V<'_>>, shared: &[Field]) -> Vec<ContentType> {
        let mut out: Vec<ContentType> = Vec::new();
        if let Some(v) = v
            && let Some(t) = self.as_table("types", v)
        {
            for (name, name_span, item) in entries(t) {
                let path = join("types", name);
                self.name_ok(name, name_span, "content type", NameRule::Key);
                let Some(tt) = self.as_table(&path, item) else {
                    continue;
                };
                // A page names its type in its frontmatter (SPEC §7.2), so
                // `files` and `default`, which applied a type by path, are
                // gone: say where the type is given now.
                for (key, span, _) in entries(tt) {
                    if matches!(key, "files" | "default") {
                        self.push(
                            self.issue(diagnostics::MODEL_UNKNOWN_KEY, span)
                                .with_variant("type-key")
                                .with_arg("key", key)
                                .with_arg("table", path.as_str())
                                .with_arg("type", name),
                        );
                    }
                }
                self.check_table(&path, tt, "types.<name>");
                let purpose = tt
                    .get("purpose")
                    .and_then(|p| self.string(&format!("{path}.purpose"), p, true));
                let template = tt
                    .get("template")
                    .and_then(|t| self.type_file(&format!("{path}.template"), t));
                let guide = tt
                    .get("guide")
                    .map(|g| self.type_files(&format!("{path}.guide"), g))
                    .unwrap_or_default();
                let fm_path = format!("{path}.frontmatter");
                let mut own = Vec::new();
                if let Some(fm) = tt.get("frontmatter")
                    && let Some(ft) = self.as_table(&fm_path, fm)
                {
                    own = self.page_fields(&fm_path, Some(name), ft);
                }
                out.push(ContentType {
                    name: name.to_owned(),
                    implicit: false,
                    frontmatter: FrontmatterSchema {
                        owner: SchemaOwner::Type(name.to_owned()),
                        fields: type_fields(shared, own.clone()),
                    },
                    own,
                    purpose,
                    template,
                    guide,
                    span: name_span,
                });
            }
        }
        if out.is_empty() {
            out.push(ContentType {
                name: "page".into(),
                implicit: true,
                frontmatter: FrontmatterSchema {
                    owner: SchemaOwner::Type("page".into()),
                    fields: type_fields(shared, Vec::new()),
                },
                own: Vec::new(),
                purpose: None,
                template: None,
                guide: Vec::new(),
                span: Span::empty(0),
            });
        }
        out
    }

    /// The fields of a page's frontmatter table, `[frontmatter]` or
    /// `[types.<name>.frontmatter]` (`type_name`), with the keys a page type
    /// can't declare reported and left out.
    fn page_fields(
        &mut self,
        path: &str,
        type_name: Option<&str>,
        ft: &toml::de::DeTable<'_>,
    ) -> Vec<Field> {
        // Under the `astro` profile, `slug` is
        // the entry id Astro's loader uses instead of the path.
        // The site output writes `inline` fields' formatted form
        // under `formatted`, and each page's path under `source`.
        let reserved: &dyn Fn(&str) -> bool = &|n| {
            matches!(
                n,
                "available"
                    | "variant"
                    | "permalink"
                    | "listing"
                    | "replaces"
                    | "intended"
                    | "type"
                    | "nav-label"
                    | "slug"
                    | "formatted"
                    | "source"
            )
        };
        for (fname, span, _) in entries(ft) {
            if reserved(fname) {
                let issue = self
                    .issue(diagnostics::MODEL_FIELD_RESERVED, span)
                    .with_arg("field", fname)
                    .with_arg("table", path);
                self.push(match fname {
                    "slug" => issue.with_variant("slug"),
                    "formatted" => issue.with_variant("formatted"),
                    "source" => issue.with_variant("source"),
                    _ => issue,
                });
            }
        }
        let mut fields = self.field_table(path, ft, Some(reserved));
        // Every page type has a required string `title` (SPEC §7.2); one
        // that's declared changes only how it's read.
        if let Some(i) = fields.iter().position(|f| f.name == "title")
            && !(fields[i].ty == FieldType::String && fields[i].required)
        {
            let f = fields.remove(i);
            let found = if f.required {
                f.ty.describe()
            } else {
                format!("{} (optional)", f.ty.describe())
            };
            let span = ft.get("title").map_or(Span::empty(0), sp);
            let mut issue = self
                .issue(diagnostics::MODEL_FIELD_RESERVED, span)
                .with_variant("title")
                .with_arg("field", "title")
                .with_arg("table", path)
                .with_arg("found", found);
            if let Some(name) = type_name {
                issue = issue.with_arg("type", name);
            }
            self.push(issue);
        }
        fields
    }

    /// A type's `template`, or one entry of its `guide`: a path from the
    /// folder `ascribe.toml` is in. Where it leads is checked with the
    /// project's files (`TypeFile`).
    fn type_file(&mut self, path: &str, v: &V<'_>) -> Option<TypeFile> {
        let text = self.string(path, v, true)?;
        self.relative(path, text, sp(v))
    }

    /// A type's `guide`: an array of paths, as `files` is.
    fn type_files(&mut self, path: &str, v: &V<'_>) -> Vec<TypeFile> {
        let Some(items) = self.strings(path, v) else {
            return Vec::new();
        };
        let key = format!("{path}[]");
        items
            .into_iter()
            .filter_map(|(text, span)| {
                if text.trim().is_empty() {
                    let issue = self.issue(diagnostics::MODEL_EMPTY_TEXT, span);
                    self.push(issue.with_arg("key", key.as_str()));
                    return None;
                }
                self.relative(&key, text, span)
            })
            .collect()
    }

    fn relative(&mut self, key: &str, text: String, span: Span) -> Option<TypeFile> {
        if is_absolute(&text) {
            let issue = self.issue(diagnostics::MODEL_PATH_ABSOLUTE, span);
            self.push(issue.with_arg("key", key));
            return None;
        }
        Some(TypeFile {
            path: text,
            span,
            lock: None,
        })
    }

    // ---- glossary ---------------------------------------------------------------

    pub fn glossary(&mut self, v: Option<&V<'_>>, fragments: &Fragments) -> Glossary {
        let match_mode = |s: &str| {
            if s == "every" {
                GlossaryMatch::Every
            } else {
                GlossaryMatch::First
            }
        };
        let mut out = Glossary {
            match_mode: match_mode(settings::default_text("glossary", "match")),
            case_sensitive: settings::default_switch("glossary", "case-sensitive"),
            pages: Vec::new(),
        };
        let Some(v) = v else { return out };
        let Some(t) = self.as_table("glossary", v) else {
            return out;
        };
        // A term is a heading marked `@term` on a glossary page (SPEC §5.4),
        // so the table of terms is gone: say where a term is written now.
        for (key, span, _) in entries(t) {
            if key == "terms" {
                self.push(
                    self.issue(diagnostics::MODEL_UNKNOWN_KEY, span)
                        .with_variant("glossary-terms")
                        .with_arg("key", key)
                        .with_arg("table", "glossary"),
                );
            }
        }
        self.check_table("glossary", t, "glossary");
        if let Some(m) = t.get("match")
            && let Some(s) =
                self.choice("glossary.match", m, &settings::choices("glossary", "match"))
        {
            out.match_mode = match_mode(&s);
        }
        if let Some(c) = t.get("case-sensitive") {
            out.case_sensitive = self
                .boolean("glossary.case-sensitive", c)
                .unwrap_or(settings::default_switch("glossary", "case-sensitive"));
        }
        if let Some(pages) = t
            .get("pages")
            .and_then(|p| self.strings("glossary.pages", p))
        {
            for (text, span) in pages {
                if let Some(path) = self.glossary_page(&text, span, fragments)
                    && !out.has_page(&path)
                {
                    out.pages.push(GlossaryPage { path, span });
                }
            }
        }
        out
    }

    /// A path in `[glossary] pages`, from the content root, when it names a
    /// page.
    fn glossary_page(&mut self, text: &str, span: Span, fragments: &Fragments) -> Option<String> {
        let rel = text.trim_start_matches('/');
        let is_fragment = rel.split('/').any(|s| s.starts_with('_'))
            || fragments.patterns.iter().any(|p| p.matches(rel));
        if is_fragment {
            self.push(
                self.issue(diagnostics::MODEL_GLOSSARY_PAGE, span)
                    .with_variant("fragment")
                    .with_arg("path", text),
            );
            return None;
        }
        Some(rel.to_owned())
    }

    // ---- images and widgets --------------------------------------------------------

    pub fn images(&mut self, v: Option<&V<'_>>) -> Vec<AttributeSchema> {
        let Some(v) = v else { return Vec::new() };
        let Some(t) = self.as_table("images", v) else {
            return Vec::new();
        };
        self.check_table("images", t, "images");
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

    // Longer than the lint allows from before it was on. Split it only while
    // changing it for another reason.
    #[allow(clippy::too_many_lines)]
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
        const WIDGET: &str = "widgets.<name>";
        self.check_table(&path, t, WIDGET);

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
        let primary_of = |s: &str| match s {
            "identifier" => Primary::Identifier { required: true },
            "identifier?" => Primary::Identifier { required: false },
            "text" => Primary::Text { required: true },
            "text?" => Primary::Text { required: false },
            _ => Primary::None,
        };
        let primary = match primary_v {
            Some(p) => self
                .choice(
                    &format!("{path}.primary"),
                    p,
                    &settings::choices(WIDGET, "primary"),
                )
                .map(|s| primary_of(&s)),
            None => Some(primary_of(settings::default_text(WIDGET, "primary"))),
        };
        let title_of = |s: &str| match s {
            "accepted" => TitleRule::Accepted,
            "required" => TitleRule::Required,
            _ => TitleRule::None,
        };
        let title = match t.get("title") {
            Some(x) => self
                .choice(
                    &format!("{path}.title"),
                    x,
                    &settings::choices(WIDGET, "title"),
                )
                .map(|s| title_of(&s)),
            None => Some(title_of(settings::default_text(WIDGET, "title"))),
        };
        let binding_v = t.get("binding");
        let binding = binding_v.and_then(|b| {
            self.choice(
                &format!("{path}.binding"),
                b,
                &settings::choices(WIDGET, "binding"),
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
            .unwrap_or(settings::default_switch(WIDGET, "groupable"));

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
        let plain_content_of = |s: &str| {
            if s == "drop" {
                PlainContent::Drop
            } else {
                PlainContent::Keep
            }
        };
        let mut plain_content = plain_content_of(settings::default_text(WIDGET, "plain-content"));
        if let Some(pc) = t.get("plain-content")
            && let Some(s) = self.choice(
                &format!("{path}.plain-content"),
                pc,
                &settings::choices(WIDGET, "plain-content"),
            )
        {
            plain_content = plain_content_of(&s);
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
        const CONSUMER: &str = "consumer";
        let trailing_slash = |s: &str| {
            if s == "never" {
                TrailingSlash::Never
            } else {
                TrailingSlash::Always
            }
        };
        let mut c = Consumer {
            profile: settings::default_text(CONSUMER, "profile").into(),
            site: None,
            base_path: settings::default_text(CONSUMER, "base-path").into(),
            trailing_slash: trailing_slash(settings::default_text(CONSUMER, "trailing-slash")),
            slugger: settings::default_text(CONSUMER, "slugger").into(),
            html: settings::default_switch(CONSUMER, "html"),
            agents: false,
            permalink_path: None,
            host: None,
        };
        let Some(v) = v else { return c };
        let Some(t) = self.as_table("consumer", v) else {
            return c;
        };
        self.check_table(CONSUMER, t, CONSUMER);
        if let Some(p) = t.get("profile")
            && let Some(s) = self.choice(
                "consumer.profile",
                p,
                &settings::choices(CONSUMER, "profile"),
            )
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
        if let Some(p) = t.get("permalink-path")
            && let Some(text) = self.string("consumer.permalink-path", p, true)
        {
            if text.starts_with('/') {
                c.permalink_path = Some(text);
            } else {
                self.push(self.issue(diagnostics::MODEL_CONSUMER_PERMALINK_PATH, sp(p)));
            }
        }
        if let Some(s) = t.get("trailing-slash")
            && let Some(text) = self.choice(
                "consumer.trailing-slash",
                s,
                &settings::choices(CONSUMER, "trailing-slash"),
            )
        {
            c.trailing_slash = trailing_slash(&text);
        }
        if let Some(s) = t.get("slugger")
            && let Some(text) = self.choice(
                "consumer.slugger",
                s,
                &settings::choices(CONSUMER, "slugger"),
            )
        {
            c.slugger = text;
        }
        // Agents' outputs come with a site that has an address, unless
        // `agents` says otherwise below (SPEC §9.4).
        let site = c.site.is_some();
        c.agents = matches!(
            settings::default_for(CONSUMER, "agents", &|t, k| (t, k) == (CONSUMER, "site")
                && site),
            Some(settings::SettingValue::Switch(true))
        );
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
        if let Some(a) = t.get("agents")
            && let Some(value) = self.boolean("consumer.agents", a)
        {
            // Every link agents read is absolute, so it needs the origin; a
            // `site` that isn't one is reported on its own.
            if value && t.get("site").is_none() {
                self.push(self.issue(diagnostics::MODEL_CONSUMER_AGENTS_SITE, sp(a)));
            } else {
                c.agents = value;
            }
        }
        if let Some(h) = t.get("host")
            && let Some(text) =
                self.choice("consumer.host", h, &settings::choices(CONSUMER, "host"))
            && text == "netlify"
        {
            c.host = Some(Host::Netlify);
        }
        c
    }

    // ---- builds and editor ----------------------------------------------------------------

    pub fn builds(
        &mut self,
        v: Option<&V<'_>>,
        dimensions: &[Dimension],
        releases: &[Releases],
    ) -> Vec<Build> {
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
                self.check_table(&path, bt, "builds.<name>");
                let variants = self.variant_mode(name, &path, bt.get("variants"), dimensions);
                let availability =
                    self.availability_mode(name, &path, bt.get("availability"), dimensions);
                let versions = match bt.get("versions") {
                    Some(vv) => self
                        .build_versions(name, &path, vv, availability.as_ref(), releases)
                        .map(Some),
                    None => Some(None),
                };
                if let (Some(variants), Some(availability), Some(versions), true) =
                    (variants, availability, versions, name_ok)
                {
                    self.excluded_warning(name, &variants, &availability, dimensions, bt);
                    let build = Build {
                        name: name.to_owned(),
                        variants,
                        availability,
                        versions,
                        line: None,
                    };
                    out.extend(lines_of(build, releases));
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
                versions: None,
                line: None,
            });
        }
        out
    }

    /// A build's `versions`: the target it's versioned by, or `None` when
    /// it was reported.
    fn build_versions(
        &mut self,
        build: &str,
        path: &str,
        v: &V<'_>,
        availability: Option<&AvailabilityMode>,
        releases: &[Releases],
    ) -> Option<String> {
        let target = self.string(&format!("{path}.versions"), v, false)?;
        if let Some(AvailabilityMode::Filter { .. }) = availability {
            self.push(
                self.issue(diagnostics::MODEL_BUILD_VERSIONS, sp(v))
                    .with_variant("filter")
                    .with_arg("build", build)
                    .with_arg("target", target.as_str()),
            );
            return None;
        }
        if !releases
            .iter()
            .any(|r| r.target == target && !r.live.is_empty())
        {
            self.push(
                self.issue(diagnostics::MODEL_BUILD_VERSIONS, sp(v))
                    .with_arg("build", build)
                    .with_arg("target", target.as_str()),
            );
            return None;
        }
        Some(target)
    }

    // ---- releases ---------------------------------------------------------------------------

    /// The implicit `release` dimension: each `[releases.<target>]` no
    /// declared dimension has, as a value or as its name, is a value of it,
    /// labeled by `[releases.<target>] label`, and the dimension is labeled
    /// by `[releases] label`. A project that declares its own `release`
    /// dimension has no implicit one, and `releases` reports each target left
    /// to it.
    pub(crate) fn release_dimension(
        &mut self,
        v: Option<&V<'_>>,
        dimensions: &mut Vec<Dimension>,
        spans: &mut Vec<DimSpans>,
    ) {
        let Some(DeValue::Table(t)) = v.map(|v| v.get_ref()) else {
            return;
        };
        let mut label = settings::default_text("releases", "label").to_owned();
        let mut values: Vec<(DimensionValue, Span)> = Vec::new();
        for (target, target_span, item) in entries(t) {
            if target == "label" {
                if let Some(text) = self.string("releases.label", item, true) {
                    label = text;
                }
                continue;
            }
            let DeValue::Table(rt) = item.get_ref() else {
                continue;
            };
            let target_label = rt.get("label");
            let declared = dimensions
                .iter()
                .find(|d| d.values.iter().any(|v| v.value == target));
            if let Some(dimension) = declared {
                if let Some(l) = target_label {
                    self.push(
                        self.issue(diagnostics::MODEL_RELEASES_TARGET, sp(l))
                            .with_variant("label")
                            .with_arg("target", target)
                            .with_arg("dimension", dimension.name.as_str()),
                    );
                }
                continue;
            }
            if dimensions.iter().any(|d| d.name == target)
                || dimensions.iter().any(|d| d.name == RELEASE_DIMENSION)
                || !self.name_ok(target, target_span, "dimension value", NameRule::NameWord)
            {
                continue;
            }
            let path = format!("{}.label", join("releases", target));
            let target_label = match target_label {
                Some(l) => self.string(&path, l, true),
                None => None,
            };
            values.push((
                DimensionValue {
                    value: target.to_owned(),
                    label: target_label.unwrap_or_else(|| target.to_owned()),
                    versionless: false,
                },
                target_span,
            ));
        }
        let Some((_, first)) = values.first() else {
            return;
        };
        spans.push(DimSpans {
            name_span: *first,
            value_spans: values.iter().map(|(_, span)| *span).collect(),
        });
        dimensions.push(Dimension {
            name: RELEASE_DIMENSION.to_owned(),
            label,
            values: values.into_iter().map(|(value, _)| value).collect(),
            implicit: true,
        });
    }

    /// `[releases]`: each target's lines, newest first, and
    /// `planned-label`.
    pub fn releases(
        &mut self,
        v: Option<&V<'_>>,
        dimensions: &[Dimension],
    ) -> (Vec<Releases>, String) {
        let mut out = Vec::new();
        let mut planned_label = settings::default_text("releases", "planned-label").to_owned();
        let Some(v) = v else {
            return (out, planned_label);
        };
        let Some(t) = self.as_table("releases", v) else {
            return (out, planned_label);
        };
        for (target, target_span, item) in entries(t) {
            // `release_dimension` read the dimension's label.
            if target == "label" {
                continue;
            }
            if target == "planned-label" {
                if let Some(label) = self.string("releases.planned-label", item, true) {
                    planned_label = label;
                }
                continue;
            }
            let path = join("releases", target);
            let Some(rt) = self.as_table(&path, item) else {
                continue;
            };
            self.check_table(&path, rt, "releases.<target>");
            let target_ok = self.releases_target(target, target_span, dimensions);
            let mut list = |key: &str| match rt.get(key) {
                Some(lv) => self.release_lines(target, &format!("{path}.{key}"), lv),
                None => Some(Vec::new()),
            };
            let live = list("live");
            let planned = list("planned");
            let archived = match rt.get("archived") {
                Some(av) => self.archived_lines(target, &format!("{path}.archived"), av),
                None => Some(Vec::new()),
            };
            let (Some(live), Some(planned), Some(archived)) = (live, planned, archived) else {
                continue;
            };
            let mut releases = Releases {
                target: target.to_owned(),
                live,
                planned,
                ends: Vec::new(),
                archived,
            };
            let lists_ok = self.release_lists(&releases);
            if let Some(ev) = rt.get("ends") {
                match self.line_ends(&releases, &format!("{path}.ends"), ev) {
                    Some(ends) => releases.ends = ends,
                    None => continue,
                }
            }
            if target_ok && lists_ok {
                out.push(releases);
            }
        }
        (out, planned_label)
    }

    /// The rules between a target's lists: every line with the same number
    /// of parts, each line in one list, planned lines newer than every live
    /// line, and archived ones older.
    fn release_lists(&mut self, r: &Releases) -> bool {
        let target = r.target.as_str();
        let lists: [(&str, Vec<&Version>); 3] = [
            ("live", r.live.iter().collect()),
            ("planned", r.planned.iter().collect()),
            ("archived", r.archived.iter().map(|a| &a.line).collect()),
        ];
        let mut ok = true;
        let first = lists.iter().find_map(|(_, l)| l.first().copied());
        if let Some(first) = first {
            for line in lists.iter().flat_map(|(_, l)| l) {
                if line.components.len() != first.components.len() {
                    self.push(
                        self.issue(diagnostics::MODEL_RELEASES_LINE, line.span)
                            .with_arg("target", target)
                            .with_arg("a", first.text.as_str())
                            .with_arg("b", line.text.as_str()),
                    );
                    ok = false;
                }
            }
        }
        if !ok {
            return false;
        }
        for (i, (list, lines)) in lists.iter().enumerate() {
            for line in lines {
                let earlier = lists[..i]
                    .iter()
                    .find(|(_, l)| l.iter().any(|o| o.components == line.components));
                if let Some((other, _)) = earlier {
                    self.push(
                        self.issue(diagnostics::MODEL_RELEASES_ORDER, line.span)
                            .with_variant("twice")
                            .with_arg("target", target)
                            .with_arg("line", line.text.as_str())
                            .with_arg("a", *other)
                            .with_arg("b", *list),
                    );
                    ok = false;
                }
            }
        }
        if !ok {
            return false;
        }
        if let Some(newest) = r.live.first() {
            for line in &r.planned {
                if line.compare(newest) != Ordering::Greater {
                    self.push(
                        self.issue(diagnostics::MODEL_RELEASES_ORDER, line.span)
                            .with_arg("target", target)
                            .with_arg("line", line.text.as_str())
                            .with_arg("live", newest.text.as_str()),
                    );
                    ok = false;
                }
            }
        }
        if let Some(oldest) = r.live.last() {
            for line in r.archived.iter().map(|a| &a.line) {
                if line.compare(oldest) != Ordering::Less {
                    self.push(
                        self.issue(diagnostics::MODEL_RELEASES_ORDER, line.span)
                            .with_variant("archived")
                            .with_arg("target", target)
                            .with_arg("line", line.text.as_str())
                            .with_arg("live", oldest.text.as_str()),
                    );
                    ok = false;
                }
            }
        }
        ok
    }

    /// A line written as a key, `"3.4"`: its version, spanning the key.
    fn line_key(&mut self, target: &str, key: &str, span: Span) -> Option<Version> {
        // After the opening quote of a quoted key.
        let offset = span.start() + usize::from(self.text_of(span).starts_with(['"', '\'']));
        let parsed = is_version(key)
            .then(|| ascribe_core::availability::parse_version(key, offset).ok())
            .flatten();
        match parsed {
            Some(line) => Some(Version { span, ..line }),
            None => {
                self.push(
                    self.issue(diagnostics::MODEL_RELEASES_LINE, span)
                        .with_variant("invalid")
                        .with_arg("target", target)
                        .with_arg("line", key),
                );
                None
            }
        }
    }

    /// `[releases.<target>.archived]`: each line's table, newest first.
    fn archived_lines(&mut self, target: &str, path: &str, v: &V<'_>) -> Option<Vec<ArchivedLine>> {
        let t = self.as_table(path, v)?;
        let mut out: Vec<ArchivedLine> = Vec::new();
        let mut ok = true;
        for (key, span, item) in entries(t) {
            let item_path = join(path, key);
            let Some(line) = self.line_key(target, key, span) else {
                ok = false;
                continue;
            };
            let Some(at) = self.as_table(&item_path, item) else {
                ok = false;
                continue;
            };
            self.check_table(&item_path, at, "releases.<target>.archived.<line>");
            let url = match at.get("url") {
                Some(uv) => match self.string(&format!("{item_path}.url"), uv, true) {
                    Some(url) => Some(url),
                    None => {
                        ok = false;
                        continue;
                    }
                },
                None => None,
            };
            let ended = match at.get("ended") {
                Some(dv) => match self.date(&format!("{item_path}.ended"), dv) {
                    Some(date) => Some(date),
                    None => {
                        ok = false;
                        continue;
                    }
                },
                None => None,
            };
            if out.iter().any(|a| a.line.components == line.components) {
                self.push(
                    self.issue(diagnostics::MODEL_RELEASES_LINE, span)
                        .with_variant("duplicate")
                        .with_arg("target", target)
                        .with_arg("line", line.text.as_str()),
                );
                ok = false;
                continue;
            }
            out.push(ArchivedLine { line, url, ended });
        }
        out.sort_by(|a, b| b.line.compare(&a.line));
        ok.then_some(out)
    }

    /// `[releases.<target>.ends]`: a date for each live line it names.
    fn line_ends(&mut self, r: &Releases, path: &str, v: &V<'_>) -> Option<Vec<LineEnd>> {
        let t = self.as_table(path, v)?;
        let mut out = Vec::new();
        let mut ok = true;
        for (key, span, item) in entries(t) {
            let Some(line) = self.line_key(&r.target, key, span) else {
                ok = false;
                continue;
            };
            if !r.live.iter().any(|l| l.components == line.components) {
                let live: Vec<String> = r.live.iter().map(|l| l.text.clone()).collect();
                let mut issue = self
                    .issue(diagnostics::MODEL_RELEASES_ENDS, span)
                    .with_arg("target", r.target.as_str())
                    .with_arg("line", key);
                issue = if live.is_empty() {
                    issue.with_variant("none")
                } else {
                    issue.with_arg("live", list(&live))
                };
                self.push(issue);
                ok = false;
                continue;
            }
            match self.date(&join(path, key), item) {
                Some(date) => out.push(LineEnd { line, date }),
                None => ok = false,
            }
        }
        ok.then_some(out)
    }

    /// A date, written as TOML writes one: `2027-01-31`, unquoted.
    fn date(&mut self, path: &str, v: &V<'_>) -> Option<Date> {
        if let DeValue::Datetime(d) = v.get_ref()
            && let (Some(date), None, None) = (d.date, d.time, d.offset)
            && let Some(date) = Date::new(
                u32::from(date.year),
                u32::from(date.month),
                u32::from(date.day),
            )
        {
            return Some(date);
        }
        self.wrong_type(path, v, "a date, such as 2027-01-31");
        None
    }

    /// Whether a `[releases]` table's name is a versioned dimension value.
    fn releases_target(&mut self, target: &str, span: Span, dimensions: &[Dimension]) -> bool {
        let value = dimensions
            .iter()
            .flat_map(|d| &d.values)
            .find(|v| v.value == target);
        match value {
            Some(value) if !value.versionless => true,
            Some(_) => {
                self.push(
                    self.issue(diagnostics::MODEL_RELEASES_TARGET, span)
                        .with_variant("versionless")
                        .with_arg("target", target),
                );
                false
            }
            None => {
                let issue = self
                    .issue(diagnostics::MODEL_RELEASES_TARGET, span)
                    .with_arg("target", target);
                if let Some(d) = dimensions.iter().find(|d| d.name == target) {
                    let values: Vec<String> = d
                        .values
                        .iter()
                        .filter(|v| !v.versionless)
                        .map(|v| v.value.clone())
                        .collect();
                    self.push(
                        issue
                            .with_variant("dimension")
                            .with_arg("values", list(&values)),
                    );
                } else if dimensions
                    .iter()
                    .any(|d| d.name == RELEASE_DIMENSION && !d.implicit)
                {
                    // The implicit dimension's name is taken.
                    self.push(issue);
                }
                // Otherwise the target isn't a name, which
                // `release_dimension` reported.
                false
            }
        }
    }

    /// A list of release lines, newest first: each a version, none twice,
    /// and all with the same number of parts.
    fn release_lines(&mut self, target: &str, path: &str, v: &V<'_>) -> Option<Vec<Version>> {
        let items = self.strings(path, v)?;
        let mut lines: Vec<Version> = Vec::new();
        let mut ok = true;
        for (text, span) in items {
            // After the opening quote; a version has nothing to escape.
            let parsed = is_version(&text)
                .then(|| ascribe_core::availability::parse_version(&text, span.start() + 1).ok())
                .flatten();
            let Some(line) = parsed else {
                self.push(
                    self.issue(diagnostics::MODEL_RELEASES_LINE, span)
                        .with_variant("invalid")
                        .with_arg("target", target)
                        .with_arg("line", text),
                );
                ok = false;
                continue;
            };
            if let Some(first) = lines.first()
                && first.components.len() != line.components.len()
            {
                self.push(
                    self.issue(diagnostics::MODEL_RELEASES_LINE, span)
                        .with_arg("target", target)
                        .with_arg("a", first.text.as_str())
                        .with_arg("b", line.text.as_str()),
                );
                ok = false;
                continue;
            }
            if lines.iter().any(|l| l.components == line.components) {
                self.push(
                    self.issue(diagnostics::MODEL_RELEASES_LINE, span)
                        .with_variant("duplicate")
                        .with_arg("target", target)
                        .with_arg("line", line.text.as_str()),
                );
                ok = false;
                continue;
            }
            lines.push(line);
        }
        lines.sort_by(|a, b| b.compare(a));
        ok.then_some(lines)
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
                let parsed = ascribe_core::availability::parse_availability(
                    &format!("{target} {version}"),
                    0,
                )
                .ok()
                .and_then(|s| match s.entries.into_iter().next()?.detail {
                    ascribe_core::availability::Detail::Version(v) => Some(v),
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
        // A versioned build's older lines can be chosen, and aren't counted
        // when choosing a default: the build is one, named by its current line.
        let declared: Vec<String> = builds
            .iter()
            .filter(|b| b.line.as_ref().is_none_or(|l| l.current))
            .map(|b| b.name.clone())
            .collect();
        let mut chosen: Option<String> = None;
        let mut at = builds_table.map_or(Span::empty(0), sp);
        if let Some(v) = v
            && let Some(t) = self.as_table("editor", v)
        {
            at = sp(v);
            self.check_table("editor", t, "editor");
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
        let default = match declared.as_slice() {
            [only] => Some(only.clone()),
            _ => declared.iter().find(|n| *n == "site").cloned(),
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

/// A declared build as the builds it gives: itself when it isn't versioned,
/// and one for each live line, newest first, when it is.
fn lines_of(build: Build, releases: &[Releases]) -> Vec<Build> {
    let Some(live) = build
        .versions
        .as_deref()
        .and_then(|t| releases.iter().find(|r| r.target == t))
        .map(|r| &r.live)
    else {
        return vec![build];
    };
    let Some(oldest) = live.last() else {
        return vec![build];
    };
    live.iter()
        .enumerate()
        .map(|(i, line)| {
            let current = i == 0;
            let name = if current {
                build.name.clone()
            } else {
                format!("{}@{}", build.name, line.text)
            };
            Build {
                name,
                line: Some(BuildLine {
                    build: build.name.clone(),
                    line: line.clone(),
                    current,
                    oldest: oldest.clone(),
                }),
                ..build.clone()
            }
        })
        .collect()
}

/// A page type's fields: `[frontmatter]`'s, each in its place unless the
/// type declares a field of its name, which replaces it whole, then the
/// type's others, with `title` first when neither declares it (SPEC §7.2).
/// A role is the field's that's given it, the type's own before a shared
/// one; one no field is given is the field's of its name, when its type
/// takes the role.
fn type_fields(shared: &[Field], own: Vec<Field>) -> Vec<Field> {
    // Each field, and whether the type declares it.
    let mut fields: Vec<(Field, bool)> = shared.iter().map(|f| (f.clone(), false)).collect();
    for f in own {
        match fields.iter_mut().find(|(s, _)| s.name == f.name) {
            Some(slot) => *slot = (f, true),
            None => fields.push((f, true)),
        }
    }
    if !fields.iter().any(|(f, _)| f.name == "title") {
        fields.insert(0, (title_field(), false));
    }
    for role in FieldRole::ALL {
        if fields.iter().any(|(f, own)| *own && f.role == Some(role)) {
            for (f, own) in &mut fields {
                if !*own && f.role == Some(role) {
                    f.role = None;
                }
            }
        }
        if !fields.iter().any(|(f, _)| f.role == Some(role))
            && let Some((f, _)) = fields
                .iter_mut()
                .find(|(f, _)| f.name == role.name() && f.role.is_none() && role.accepts(&f.ty))
        {
            f.role = Some(role);
        }
    }
    fields.into_iter().map(|(f, _)| f).collect()
}

/// The `title` a page type has when it doesn't declare one: a required
/// string.
fn title_field() -> Field {
    Field {
        name: "title".into(),
        ty: FieldType::String,
        required: true,
        default: None,
        phrases: false,
        inline: None,
        role: None,
        description: None,
    }
}
