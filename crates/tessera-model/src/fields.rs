//! Field types and attribute types (content-model.md §6): parsing the short
//! form, reading the table form, and building fields and attribute schemas.

use serde_yaml::Value;
use tessera_core::{AttributeSchema, AttributeType, DefaultValue, SetMember, Span, diagnostics};
use toml::de::{DeTable, DeValue};

use crate::loader::{Loader, NameRule};
use crate::toml_util::{V, describe, entries, join, sp, to_yaml};
use crate::types::{Field, FieldType, default_mismatch};

/// Which type language is being read.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum Kind {
    Field,
    Attribute,
}

impl Kind {
    fn word(self) -> &'static str {
        match self {
            Kind::Field => "field",
            Kind::Attribute => "attribute",
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
enum Scalar {
    String,
    Number,
    Boolean,
    Date,
}

/// A type as written, before `fields` and `values` are applied.
#[derive(Clone, Debug, PartialEq)]
enum Base {
    Scalar(Scalar),
    /// `enum(a, b)` (Some) or bare `enum` (None).
    Enum(Option<Vec<String>>),
    List(Box<Base>),
    Object,
    Set(Box<Base>),
}

#[derive(Debug)]
enum TypeErr {
    Syntax(String),
    EmptyEnum,
    DuplicateEnum(String),
}

fn is_enum_value(s: &str) -> bool {
    !s.is_empty()
        && s.chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.'))
}

/// Characters a set member can't contain (SPEC §3.3).
fn is_set_token(s: &str) -> bool {
    !s.is_empty()
        && !s
            .chars()
            .any(|c| c.is_whitespace() || matches!(c, ',' | '|' | '{' | '}' | '=' | '"'))
}

fn expected_list(kind: Kind) -> &'static str {
    match kind {
        Kind::Field => "expected string, number, boolean, date, enum(…), or list(…)",
        Kind::Attribute => "expected string, number, boolean, enum(…), or set(…)",
    }
}

fn parse_type(s: &str, kind: Kind) -> Result<(Base, bool), TypeErr> {
    if s != s.trim() {
        return Err(TypeErr::Syntax("remove the spaces around the type".into()));
    }
    let (body, optional) = match s.strip_suffix('?') {
        Some(b) => (b, true),
        None => (s, false),
    };
    Ok((parse_base(body, kind, false)?, optional))
}

fn parse_base(s: &str, kind: Kind, in_container: bool) -> Result<Base, TypeErr> {
    match s {
        "string" => return Ok(Base::Scalar(Scalar::String)),
        "number" => return Ok(Base::Scalar(Scalar::Number)),
        "boolean" => return Ok(Base::Scalar(Scalar::Boolean)),
        "date" => {
            return match kind {
                Kind::Field => Ok(Base::Scalar(Scalar::Date)),
                Kind::Attribute => Err(TypeErr::Syntax(
                    "attributes can't be dates; use string, number, boolean, enum(…), or set(…)"
                        .into(),
                )),
            };
        }
        "object" => {
            return match kind {
                Kind::Field => Ok(Base::Object),
                Kind::Attribute => Err(TypeErr::Syntax(
                    "attributes can't be objects; use string, number, boolean, enum(…), or set(…)"
                        .into(),
                )),
            };
        }
        "enum" => return Ok(Base::Enum(None)),
        _ => {}
    }
    if let Some(inner) = s.strip_prefix("enum(").and_then(|r| r.strip_suffix(')')) {
        let mut values: Vec<String> = Vec::new();
        for raw in inner.split(',') {
            let v = raw.trim_matches(' ');
            if v.is_empty() && inner.trim().is_empty() {
                return Err(TypeErr::EmptyEnum);
            }
            if !is_enum_value(v) {
                return Err(TypeErr::Syntax(format!(
                    "\"{v}\" isn't a simple enumeration value; use letters, digits, `-`, `_`, or `.`, or list the values with `values`"
                )));
            }
            if values.iter().any(|x| x == v) {
                return Err(TypeErr::DuplicateEnum(v.to_owned()));
            }
            values.push(v.to_owned());
        }
        return Ok(Base::Enum(Some(values)));
    }
    if let Some(inner) = s.strip_prefix("list(").and_then(|r| r.strip_suffix(')')) {
        if kind == Kind::Attribute {
            return Err(TypeErr::Syntax(
                "attributes can't be lists; use set(…) for several values".into(),
            ));
        }
        if in_container {
            return Err(TypeErr::Syntax("lists can't nest".into()));
        }
        let inner = inner.trim_matches(' ');
        return match parse_base(inner, kind, true)? {
            b @ (Base::Scalar(_) | Base::Enum(_) | Base::Object) => Ok(Base::List(Box::new(b))),
            _ => Err(TypeErr::Syntax(
                "a list holds string, number, boolean, date, enum(…), or object".into(),
            )),
        };
    }
    if let Some(inner) = s.strip_prefix("set(").and_then(|r| r.strip_suffix(')')) {
        if kind == Kind::Field {
            return Err(TypeErr::Syntax(
                "set(…) is only for attributes; use list(…) for frontmatter".into(),
            ));
        }
        let inner = inner.trim_matches(' ');
        return match parse_base(inner, kind, true)? {
            b @ (Base::Scalar(Scalar::String) | Base::Enum(_)) => Ok(Base::Set(Box::new(b))),
            _ => Err(TypeErr::Syntax(
                "a set holds string or enum(…) members".into(),
            )),
        };
    }
    Err(TypeErr::Syntax(expected_list(kind).into()))
}

fn needs_fields(b: &Base) -> bool {
    match b {
        Base::Object => true,
        Base::List(inner) => matches!(**inner, Base::Object),
        _ => false,
    }
}

fn bare_enum(b: &Base) -> bool {
    match b {
        Base::Enum(None) => true,
        Base::List(i) | Base::Set(i) => matches!(**i, Base::Enum(None)),
        _ => false,
    }
}

fn has_inline_enum(b: &Base) -> bool {
    match b {
        Base::Enum(Some(_)) => true,
        Base::List(i) | Base::Set(i) => matches!(**i, Base::Enum(Some(_))),
        _ => false,
    }
}

fn is_set(b: &Base) -> bool {
    matches!(b, Base::Set(_))
}

/// The written form, for messages.
fn base_text(b: &Base) -> String {
    match b {
        Base::Scalar(Scalar::String) => "string".into(),
        Base::Scalar(Scalar::Number) => "number".into(),
        Base::Scalar(Scalar::Boolean) => "boolean".into(),
        Base::Scalar(Scalar::Date) => "date".into(),
        Base::Enum(Some(v)) => format!("enum({})", v.join(", ")),
        Base::Enum(None) => "enum".into(),
        Base::List(i) => format!("list({})", base_text(i)),
        Base::Object => "object".into(),
        Base::Set(i) => format!("set({})", base_text(i)),
    }
}

fn to_field_type(b: &Base, values: &[String], fields: &[Field]) -> FieldType {
    match b {
        Base::Scalar(Scalar::String) => FieldType::String,
        Base::Scalar(Scalar::Number) => FieldType::Number,
        Base::Scalar(Scalar::Boolean) => FieldType::Boolean,
        Base::Scalar(Scalar::Date) => FieldType::Date,
        Base::Enum(Some(v)) => FieldType::Enum(v.clone()),
        Base::Enum(None) => FieldType::Enum(values.to_vec()),
        Base::List(i) => FieldType::List(Box::new(to_field_type(i, values, fields))),
        Base::Object => FieldType::Object(fields.to_vec()),
        // Sets are attribute-only; they never reach a field.
        Base::Set(i) => to_field_type(i, values, fields),
    }
}

fn to_attribute_type(b: &Base, values: &[String]) -> AttributeType {
    match b {
        Base::Scalar(Scalar::Number) => AttributeType::Number,
        Base::Scalar(Scalar::Boolean) => AttributeType::Boolean,
        Base::Enum(Some(v)) => AttributeType::Enum(v.clone()),
        Base::Enum(None) => AttributeType::Enum(values.to_vec()),
        Base::Set(i) => AttributeType::Set(match &**i {
            Base::Enum(Some(v)) => SetMember::Enum(v.clone()),
            Base::Enum(None) => SetMember::Enum(values.to_vec()),
            _ => SetMember::String,
        }),
        _ => AttributeType::String,
    }
}

fn attribute_text(t: &AttributeType) -> String {
    match t {
        AttributeType::String => "string".into(),
        AttributeType::Number => "number".into(),
        AttributeType::Boolean => "boolean".into(),
        AttributeType::Enum(v) => format!("enum({})", v.join(", ")),
        AttributeType::Set(SetMember::String) => "set(string)".into(),
        AttributeType::Set(SetMember::Enum(v)) => format!("set(enum({}))", v.join(", ")),
        AttributeType::NoteType => "note type".into(),
    }
}

/// What a table-form entry reads to.
struct Common {
    base: Base,
    optional: bool,
    values: Vec<String>,
    fields: Vec<Field>,
    default: Option<(Span, &'static str)>,
    phrases: bool,
    description: Option<String>,
}

impl Loader<'_> {
    /// `[…frontmatter]` or a nested `fields` table: name-word keys, field types.
    pub fn field_table(
        &mut self,
        path: &str,
        t: &DeTable<'_>,
        reserved: Option<&dyn Fn(&str) -> bool>,
    ) -> Vec<Field> {
        let mut out = Vec::new();
        for (name, name_span, item) in entries(t) {
            let ok = self.name_ok(name, name_span, "frontmatter field", NameRule::NameWord);
            if let Some(is_reserved) = reserved
                && is_reserved(name)
            {
                continue;
            }
            if !ok {
                continue;
            }
            let p = join(path, name);
            if let Some(f) = self.field(&p, name, item) {
                out.push(f);
            }
        }
        out
    }

    /// `[images.attributes]` and `[widgets.<name>.attributes]`.
    pub fn attribute_table(
        &mut self,
        path: &str,
        t: &DeTable<'_>,
        reserved: &dyn Fn(&str) -> bool,
        widget: Option<&str>,
    ) -> Vec<AttributeSchema> {
        let mut out = Vec::new();
        for (name, name_span, item) in entries(t) {
            let ok = self.name_ok(name, name_span, "attribute", NameRule::Key);
            if ok && reserved(name) {
                let mut issue = self
                    .issue(diagnostics::MODEL_ATTRIBUTE_RESERVED, name_span)
                    .with_arg("key", name);
                if let Some(w) = widget {
                    issue = issue.with_variant("widget").with_arg("name", w);
                }
                self.push(issue);
                continue;
            }
            if !ok {
                continue;
            }
            let p = join(path, name);
            if let Some(a) = self.attribute(&p, name, item) {
                out.push(a);
            }
        }
        out
    }

    fn field(&mut self, path: &str, name: &str, v: &V<'_>) -> Option<Field> {
        let c = self.common(path, name, v, Kind::Field)?;
        let ty = to_field_type(&c.base, &c.values, &c.fields);
        if c.phrases && !phrase_capable(&ty) {
            self.push(
                self.issue(diagnostics::MODEL_PHRASES_FIELD_TYPE, sp(v))
                    .with_arg("field", name)
                    .with_arg("type", base_text(&c.base)),
            );
        }
        let mut default = None;
        if let Some((span, _)) = c.default {
            let dv = default_value(v, span);
            if let Some(dv) = dv {
                let yaml = to_yaml(dv.get_ref());
                let mismatch = default_mismatch(&ty, &yaml);
                let enum_miss = matches!(&ty, FieldType::Enum(vals) if matches!(&yaml, Value::String(s) if !vals.contains(s)));
                if let (FieldType::Enum(vals), true) = (&ty, enum_miss) {
                    self.push(
                        self.issue(diagnostics::MODEL_DEFAULT_TYPE, span)
                            .with_variant("not-a-value")
                            .with_arg("field", name)
                            .with_arg("value", yaml.as_str().unwrap_or_default())
                            .with_arg("values", vals.join(", ")),
                    );
                } else if mismatch.is_some() {
                    self.push(
                        self.issue(diagnostics::MODEL_DEFAULT_TYPE, span)
                            .with_arg("field", name)
                            .with_arg("type", ty.describe())
                            .with_arg("found", describe(dv.get_ref())),
                    );
                } else {
                    default = Some(yaml);
                }
            }
        }
        Some(Field {
            name: name.to_owned(),
            required: !c.optional && default.is_none(),
            ty,
            default,
            phrases: c.phrases,
            description: c.description,
        })
    }

    fn attribute(&mut self, path: &str, name: &str, v: &V<'_>) -> Option<AttributeSchema> {
        let c = self.common(path, name, v, Kind::Attribute)?;
        let ty = to_attribute_type(&c.base, &c.values);
        let mut default = None;
        if let Some((span, _)) = c.default
            && let Some(dv) = default_value(v, span)
        {
            default = self.attribute_default(name, &ty, dv, span);
        }
        Some(AttributeSchema {
            key: name.to_owned(),
            required: !c.optional && default.is_none(),
            ty,
            default,
            description: c.description,
        })
    }

    fn attribute_default(
        &mut self,
        name: &str,
        ty: &AttributeType,
        dv: &V<'_>,
        span: Span,
    ) -> Option<DefaultValue> {
        let mismatch = |l: &mut Self| {
            l.push(
                l.issue(diagnostics::MODEL_DEFAULT_TYPE, span)
                    .with_arg("field", name)
                    .with_arg("type", attribute_text(ty))
                    .with_arg("found", describe(dv.get_ref())),
            );
            None
        };
        let not_a_value = |l: &mut Self, value: &str, values: &[String]| {
            l.push(
                l.issue(diagnostics::MODEL_DEFAULT_TYPE, span)
                    .with_variant("not-a-value")
                    .with_arg("field", name)
                    .with_arg("value", value)
                    .with_arg("values", values.join(", ")),
            );
            None
        };
        match (ty, dv.get_ref()) {
            (AttributeType::String | AttributeType::NoteType, DeValue::String(s)) => {
                Some(DefaultValue::Text(s.to_string()))
            }
            (AttributeType::Boolean, DeValue::Boolean(b)) => Some(DefaultValue::Boolean(*b)),
            // `DefaultValue` has no number variant, so a number default is
            // its source text (resolved Q28).
            (AttributeType::Number, DeValue::Integer(i)) => {
                Some(DefaultValue::Text(i.as_str().to_owned()))
            }
            (AttributeType::Number, DeValue::Float(f)) => {
                Some(DefaultValue::Text(f.as_str().to_owned()))
            }
            (AttributeType::Enum(values), DeValue::String(s)) => {
                if values.iter().any(|x| x == s.as_ref()) {
                    Some(DefaultValue::Text(s.to_string()))
                } else {
                    not_a_value(self, s, values)
                }
            }
            (AttributeType::Set(member), DeValue::String(_) | DeValue::Array(_)) => {
                let members: Vec<String> = match dv.get_ref() {
                    DeValue::String(s) => vec![s.to_string()],
                    DeValue::Array(items) => {
                        let mut out = Vec::new();
                        for item in items.iter() {
                            match item.get_ref() {
                                DeValue::String(s) => out.push(s.to_string()),
                                _ => return mismatch(self),
                            }
                        }
                        out
                    }
                    _ => return mismatch(self),
                };
                if let SetMember::Enum(values) = member
                    && let Some(bad) = members.iter().find(|m| !values.contains(m))
                {
                    return not_a_value(self, bad, values);
                }
                Some(DefaultValue::Set(members))
            }
            _ => mismatch(self),
        }
    }

    /// Reads one entry in short or table form.
    fn common(&mut self, path: &str, name: &str, v: &V<'_>, kind: Kind) -> Option<Common> {
        let (type_span, type_text, table): (Span, String, Option<&DeTable<'_>>) = match v.get_ref()
        {
            DeValue::String(s) => (sp(v), s.to_string(), None),
            DeValue::Table(t) => {
                let allowed: &[&str] = match kind {
                    Kind::Field => &[
                        "type",
                        "fields",
                        "values",
                        "default",
                        "phrases",
                        "description",
                    ],
                    Kind::Attribute => &["type", "values", "default", "description"],
                };
                self.check_keys(path, t, allowed);
                let ty = self.require(path, t, sp(v), "type")?;
                let text = self.string(&format!("{path}.type"), ty, false)?;
                (sp(ty), text, Some(t))
            }
            _ => {
                self.wrong_type(path, v, "a type string or a table");
                return None;
            }
        };
        let (base, optional) = match parse_type(&type_text, kind) {
            Ok(x) => x,
            Err(e) => {
                self.type_error(name, type_span, &type_text, kind, e);
                return None;
            }
        };
        let short = table.is_none();
        if short && (needs_fields(&base) || bare_enum(&base)) {
            let detail = if bare_enum(&base) {
                "a bare enum needs table form, with `values`: { type = \"enum\", values = [\"a\", \"b\"] }"
            } else {
                "objects need table form, with `fields`: { type = \"object\", fields = { … } }"
            };
            self.push(
                self.issue(diagnostics::MODEL_TYPE_SYNTAX, type_span)
                    .with_arg("type", type_text.as_str())
                    .with_arg("kind", kind.word())
                    .with_arg("detail", detail),
            );
            return None;
        }
        let mut c = Common {
            base,
            optional,
            values: Vec::new(),
            fields: Vec::new(),
            default: None,
            phrases: false,
            description: None,
        };
        let Some(t) = table else { return Some(c) };

        // fields
        match (t.get("fields"), needs_fields(&c.base)) {
            (Some(f), true) => {
                if let Some(ft) = self.as_table(&format!("{path}.fields"), f) {
                    c.fields = self.field_table(&format!("{path}.fields"), ft, None);
                }
            }
            (Some(f), false) => self.push(
                self.issue(diagnostics::MODEL_TYPE_FIELDS, sp(f))
                    .with_variant("not-object")
                    .with_arg("field", name)
                    .with_arg("type", type_text.as_str()),
            ),
            (None, true) => self.push(
                self.issue(diagnostics::MODEL_TYPE_FIELDS, sp(v))
                    .with_arg("field", name),
            ),
            (None, false) => {}
        }

        // values
        let values = t.get("values");
        match (values, bare_enum(&c.base), has_inline_enum(&c.base)) {
            (Some(vv), true, _) => {
                if let Some(vals) = self.strings(&format!("{path}.values"), vv) {
                    c.values = self.enum_values(name, &c.base, vv, vals);
                }
            }
            (Some(vv), false, true) => self.push(
                self.issue(diagnostics::MODEL_ENUM_VALUES, sp(vv))
                    .with_variant("both")
                    .with_arg("field", name),
            ),
            (Some(vv), false, false) => {
                let span = t.get_key_value("values").map_or(sp(vv), |(k, _)| sp(k));
                self.push(
                    self.issue(diagnostics::MODEL_UNKNOWN_KEY, span)
                        .with_arg("key", "values")
                        .with_arg("table", path),
                );
            }
            (None, true, _) => self.push(
                self.issue(diagnostics::MODEL_ENUM_VALUES, sp(v))
                    .with_variant("bare")
                    .with_arg("field", name),
            ),
            (None, false, _) => {}
        }
        // Inline set(enum(…)) members must be tokens.
        if is_set(&c.base)
            && let Base::Set(inner) = &c.base
            && let Base::Enum(Some(vals)) = &**inner
        {
            for m in vals {
                if !is_set_token(m) {
                    self.set_token(m, type_span);
                }
            }
        }

        if let Some(d) = t.get("default") {
            c.default = Some((sp(d), "default"));
        }
        if let Some(p) = t.get("phrases") {
            c.phrases = self.boolean(&format!("{path}.phrases"), p).unwrap_or(false);
        }
        if let Some(d) = t.get("description") {
            c.description = self.string(&format!("{path}.description"), d, true);
        }
        Some(c)
    }

    fn enum_values(
        &mut self,
        name: &str,
        base: &Base,
        at: &V<'_>,
        vals: Vec<(String, Span)>,
    ) -> Vec<String> {
        if vals.is_empty() {
            self.push(
                self.issue(diagnostics::MODEL_ENUM_VALUES, sp(at))
                    .with_arg("field", name),
            );
        }
        let mut out: Vec<String> = Vec::new();
        for (val, span) in vals {
            if val.is_empty() {
                self.push(
                    self.issue(diagnostics::MODEL_EMPTY_TEXT, span)
                        .with_arg("key", format!("{name}.values")),
                );
                continue;
            }
            if out.contains(&val) {
                self.push(
                    self.issue(diagnostics::MODEL_ENUM_VALUES, span)
                        .with_variant("duplicate")
                        .with_arg("field", name)
                        .with_arg("value", val),
                );
                continue;
            }
            if matches!(base, Base::Set(_)) && !is_set_token(&val) {
                self.set_token(&val, span);
                continue;
            }
            out.push(val);
        }
        out
    }

    fn set_token(&mut self, value: &str, span: Span) {
        self.push(
            self.issue(diagnostics::MODEL_SET_TOKEN, span)
                .with_arg("value", value),
        );
    }

    fn type_error(&mut self, name: &str, span: Span, text: &str, kind: Kind, e: TypeErr) {
        match e {
            TypeErr::Syntax(detail) => self.push(
                self.issue(diagnostics::MODEL_TYPE_SYNTAX, span)
                    .with_arg("type", text)
                    .with_arg("kind", kind.word())
                    .with_arg("detail", detail),
            ),
            TypeErr::EmptyEnum => self.push(
                self.issue(diagnostics::MODEL_ENUM_VALUES, span)
                    .with_arg("field", name),
            ),
            TypeErr::DuplicateEnum(value) => self.push(
                self.issue(diagnostics::MODEL_ENUM_VALUES, span)
                    .with_variant("duplicate")
                    .with_arg("field", name)
                    .with_arg("value", value),
            ),
        }
    }
}

/// The `default` value node of a table-form entry.
fn default_value<'a, 'i>(entry: &'a V<'i>, span: Span) -> Option<&'a V<'i>> {
    let DeValue::Table(t) = entry.get_ref() else {
        return None;
    };
    t.get("default").filter(|d| sp(*d) == span)
}

/// `string` and `list(string)` accept phrases (content-model.md §6.2).
fn phrase_capable(ty: &FieldType) -> bool {
    match ty {
        FieldType::String => true,
        FieldType::List(inner) => matches!(**inner, FieldType::String),
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ok(s: &str, kind: Kind) -> (Base, bool) {
        parse_type(s, kind).unwrap_or_else(|e| panic!("{s}: {e:?}"))
    }

    #[test]
    fn short_forms() {
        assert_eq!(
            ok("string", Kind::Field),
            (Base::Scalar(Scalar::String), false)
        );
        assert_eq!(ok("date?", Kind::Field), (Base::Scalar(Scalar::Date), true));
        assert_eq!(
            ok("enum(a, b)", Kind::Field).0,
            Base::Enum(Some(vec!["a".into(), "b".into()]))
        );
        assert_eq!(
            ok("list(string)?", Kind::Field),
            (Base::List(Box::new(Base::Scalar(Scalar::String))), true)
        );
        assert!(matches!(
            ok("set(enum(a, b))", Kind::Attribute).0,
            Base::Set(_)
        ));
    }

    #[test]
    fn rejects() {
        for (s, kind) in [
            ("strng?", Kind::Field),
            ("date", Kind::Attribute),
            ("list(string)", Kind::Attribute),
            ("set(string)", Kind::Field),
            ("list(list(string))", Kind::Field),
            (" string", Kind::Field),
            ("enum(a b)", Kind::Field),
            ("enum(a,,b)", Kind::Field),
            ("string??", Kind::Field),
        ] {
            assert!(parse_type(s, kind).is_err(), "{s}");
        }
        assert!(matches!(
            parse_type("enum()", Kind::Field),
            Err(TypeErr::EmptyEnum)
        ));
        assert!(matches!(
            parse_type("enum(a, a)", Kind::Field),
            Err(TypeErr::DuplicateEnum(_))
        ));
    }
}
