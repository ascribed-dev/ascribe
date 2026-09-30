//! Field types and attribute types, and frontmatter
//! validation against them.

use serde_yaml::Value;
use tessera_core::{Issue, Location, diagnostics};

use crate::names::suggest;

/// The type of a frontmatter field.
#[derive(Clone, Debug, PartialEq)]
pub enum FieldType {
    /// A YAML string.
    String,
    /// A YAML integer or float.
    Number,
    /// YAML `true` or `false`.
    Boolean,
    /// A calendar date, `YYYY-MM-DD`.
    Date,
    /// One of the listed values.
    Enum(Vec<String>),
    /// A sequence whose items all have the inner type.
    List(Box<FieldType>),
    /// A mapping with the listed fields; unknown keys are errors.
    Object(Vec<Field>),
}

/// One frontmatter field.
#[derive(Clone, Debug, PartialEq)]
pub struct Field {
    /// The field name (SPEC `name-word`).
    pub name: String,
    /// Its type.
    pub ty: FieldType,
    /// Whether the field must be present: not optional and no default.
    pub required: bool,
    /// The value used when absent.
    pub default: Option<Value>,
    /// Whether phrases (SPEC §5.1) are substituted in the value.
    pub phrases: bool,
    /// Help text.
    pub description: Option<String>,
}

/// Whose schema this is, for messages.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SchemaOwner {
    /// A page content type, by name.
    Type(String),
    /// The fragment schema, `[fragments.frontmatter]`.
    Fragment,
}

/// A frontmatter schema: the fields of a content type or of fragments.
#[derive(Clone, Debug, PartialEq)]
pub struct FrontmatterSchema {
    /// Who owns the schema.
    pub owner: SchemaOwner,
    /// The fields, in declaration order.
    pub fields: Vec<Field>,
}

impl FieldType {
    /// The type in short form: `string`, `list(string)`, `enum(a, b)`.
    pub fn describe(&self) -> String {
        match self {
            FieldType::String => "string".into(),
            FieldType::Number => "number".into(),
            FieldType::Boolean => "boolean".into(),
            FieldType::Date => "date".into(),
            FieldType::Enum(v) => format!("enum({})", v.join(", ")),
            FieldType::List(inner) => format!("list({})", inner.describe()),
            FieldType::Object(_) => "object".into(),
        }
    }

    fn expected(&self) -> String {
        match self {
            FieldType::String => "a string".into(),
            FieldType::Number => "a number".into(),
            FieldType::Boolean => "a boolean".into(),
            FieldType::Date => "a date written YYYY-MM-DD".into(),
            FieldType::Enum(v) => format!("one of: {}", v.join(", ")),
            FieldType::List(inner) => format!("a list of {}", inner.plural()),
            FieldType::Object(_) => "a mapping".into(),
        }
    }

    fn plural(&self) -> String {
        match self {
            FieldType::String => "strings".into(),
            FieldType::Number => "numbers".into(),
            FieldType::Boolean => "booleans".into(),
            FieldType::Date => "dates".into(),
            FieldType::Enum(v) => format!("values of: {}", v.join(", ")),
            FieldType::List(_) => "lists".into(),
            FieldType::Object(_) => "mappings".into(),
        }
    }
}

/// How YAML (core schema) reads a value, for messages.
fn found(v: &Value) -> String {
    match v {
        Value::Null => "empty".into(),
        Value::Bool(_) => "a boolean".into(),
        Value::Number(_) => "a number".into(),
        Value::String(_) => "a string".into(),
        Value::Sequence(_) => "a list".into(),
        Value::Mapping(_) => "a mapping".into(),
        Value::Tagged(t) => found(&t.value),
    }
}

/// Whether `s` is a real calendar date written `YYYY-MM-DD`.
pub fn is_calendar_date(s: &str) -> bool {
    let b = s.as_bytes();
    if b.len() != 10 || b[4] != b'-' || b[7] != b'-' {
        return false;
    }
    let digits = |r: std::ops::Range<usize>| b[r].iter().all(u8::is_ascii_digit);
    if !(digits(0..4) && digits(5..7) && digits(8..10)) {
        return false;
    }
    let (Ok(y), Ok(m), Ok(d)) = (
        s[0..4].parse::<u32>(),
        s[5..7].parse::<u32>(),
        s[8..10].parse::<u32>(),
    ) else {
        return false;
    };
    let leap = (y % 4 == 0 && y % 100 != 0) || y % 400 == 0;
    let days = match m {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if leap => 29,
        2 => 28,
        _ => return false,
    };
    (1..=days).contains(&d)
}

/// Validates parsed frontmatter against a schema.
///
/// `value` is the frontmatter parsed as YAML (the core schema). `at`
/// is where the frontmatter is; every issue is reported there, and names the
/// offending field in its `field` or `key` argument (a path such as
/// `author.name` or `tags[1]`), so a caller with finer spans can relocate it.
///
/// Reports the registry's `frontmatter-*` slugs:
/// - `frontmatter-unknown-key`, with a `suggestion` variant when a declared
///   field is close;
/// - `frontmatter-missing-field`;
/// - `frontmatter-type-mismatch`;
/// - `frontmatter-reserved-in-fragment`, for `available` and `variant` in a
///   fragment. On a page, those two keys are accepted here; their values are
///   checked elsewhere (SPEC §4.3).
///
/// Choosing which schema applies to a page is [`crate::ContentModel::type_for`].
pub fn validate_frontmatter(schema: &FrontmatterSchema, value: &Value, at: Location) -> Vec<Issue> {
    let mut cx = Cx {
        schema,
        at,
        issues: Vec::new(),
    };
    match value {
        // No frontmatter at all is an empty mapping.
        Value::Null => cx.object("", &schema.fields, None),
        Value::Mapping(map) => cx.object("", &schema.fields, Some(map)),
        other => cx.issues.push(
            Issue::new(diagnostics::FRONTMATTER_TYPE_MISMATCH, at)
                .with_arg("field", "frontmatter")
                .with_arg("expected", "a mapping of fields")
                .with_arg("found", found(other)),
        ),
    }
    cx.issues
}

struct Cx<'a> {
    schema: &'a FrontmatterSchema,
    at: Location,
    issues: Vec<Issue>,
}

impl Cx<'_> {
    fn type_name(&self) -> String {
        match &self.schema.owner {
            SchemaOwner::Type(name) => name.clone(),
            SchemaOwner::Fragment => "fragment".into(),
        }
    }

    fn object(&mut self, path: &str, fields: &[Field], map: Option<&serde_yaml::Mapping>) {
        let join = |name: &str| {
            if path.is_empty() {
                name.to_owned()
            } else {
                format!("{path}.{name}")
            }
        };
        if let Some(map) = map {
            for (key, value) in map {
                let Some(key) = key.as_str() else {
                    self.issues.push(
                        Issue::new(diagnostics::FRONTMATTER_TYPE_MISMATCH, self.at)
                            .with_arg("field", join("(key)"))
                            .with_arg("expected", "a string key")
                            .with_arg("found", found(key)),
                    );
                    continue;
                };
                match fields.iter().find(|f| f.name == key) {
                    Some(field) => self.value(&join(key), value, &field.ty),
                    None => self.unknown(path, key, fields),
                }
            }
        }
        for field in fields.iter().filter(|f| f.required) {
            let present = map.is_some_and(|m| m.contains_key(field.name.as_str()));
            if !present {
                self.issues.push(
                    Issue::new(diagnostics::FRONTMATTER_MISSING_FIELD, self.at)
                        .with_arg("field", join(&field.name))
                        .with_arg("type", self.type_name()),
                );
            }
        }
    }

    fn unknown(&mut self, path: &str, key: &str, fields: &[Field]) {
        let full = if path.is_empty() {
            key.to_owned()
        } else {
            format!("{path}.{key}")
        };
        let top = path.is_empty();
        if top && matches!(key, "available" | "variant") {
            match self.schema.owner {
                SchemaOwner::Type(_) => {}
                SchemaOwner::Fragment => {
                    let mut issue =
                        Issue::new(diagnostics::FRONTMATTER_RESERVED_IN_FRAGMENT, self.at)
                            .with_arg("key", key);
                    if key == "available" {
                        issue = issue.with_variant("available");
                    }
                    self.issues.push(issue);
                }
            }
            return;
        }
        let mut issue = Issue::new(diagnostics::FRONTMATTER_UNKNOWN_KEY, self.at)
            .with_arg("key", full)
            .with_arg("type", self.type_name());
        match (
            &self.schema.owner,
            suggest(key, fields.iter().map(|f| f.name.as_str())),
        ) {
            (_, Some(s)) => issue = issue.with_variant("suggestion").with_arg("suggestion", s),
            (SchemaOwner::Fragment, None) if top => issue = issue.with_variant("fragment"),
            _ => {}
        }
        self.issues.push(issue);
    }

    fn mismatch(&mut self, path: &str, expected: &FieldType, v: &Value) {
        self.issues.push(
            Issue::new(diagnostics::FRONTMATTER_TYPE_MISMATCH, self.at)
                .with_arg("field", path)
                .with_arg("expected", expected.expected())
                .with_arg("found", found(v)),
        );
    }

    fn value(&mut self, path: &str, v: &Value, ty: &FieldType) {
        let ok = match (ty, v) {
            (FieldType::String, Value::String(_)) => true,
            (FieldType::Number, Value::Number(_)) => true,
            (FieldType::Boolean, Value::Bool(_)) => true,
            (FieldType::Date, Value::String(s)) => is_calendar_date(s),
            (FieldType::Enum(values), Value::String(s)) => values.iter().any(|x| x == s),
            (FieldType::List(inner), Value::Sequence(items)) => {
                for (i, item) in items.iter().enumerate() {
                    self.value(&format!("{path}[{i}]"), item, inner);
                }
                return;
            }
            (FieldType::Object(fields), Value::Mapping(map)) => {
                self.object(path, fields, Some(map));
                return;
            }
            _ => false,
        };
        if !ok {
            self.mismatch(path, ty, v);
        }
    }
}

/// Checks a default value against a field type. Returns what was found when
/// it doesn't conform.
pub fn default_mismatch(ty: &FieldType, v: &Value) -> Option<String> {
    let schema = FrontmatterSchema {
        owner: SchemaOwner::Fragment,
        fields: vec![Field {
            name: "default".into(),
            ty: ty.clone(),
            required: true,
            default: None,
            phrases: false,
            description: None,
        }],
    };
    let mut map = serde_yaml::Mapping::new();
    map.insert(Value::String("default".into()), v.clone());
    let at = Location::new(tessera_core::FileId::new(0), 0..0);
    validate_frontmatter(&schema, &Value::Mapping(map), at)
        .into_iter()
        .find_map(|i| i.arg("found").map(str::to_owned))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tessera_core::FileId;

    fn field(name: &str, ty: FieldType, required: bool) -> Field {
        Field {
            name: name.into(),
            ty,
            required,
            default: None,
            phrases: false,
            description: None,
        }
    }

    fn schema(owner: SchemaOwner) -> FrontmatterSchema {
        FrontmatterSchema {
            owner,
            fields: vec![
                field("title", FieldType::String, true),
                field(
                    "level",
                    FieldType::Enum(vec!["a".into(), "b".into()]),
                    false,
                ),
                field("tags", FieldType::List(Box::new(FieldType::String)), false),
                field("updated", FieldType::Date, false),
                field(
                    "author",
                    FieldType::Object(vec![field("name", FieldType::String, true)]),
                    false,
                ),
            ],
        }
    }

    fn run(owner: SchemaOwner, yaml: &str) -> Vec<(String, String)> {
        let value: Value = serde_yaml::from_str(yaml).unwrap();
        let at = Location::new(FileId::new(0), 0..0);
        validate_frontmatter(&schema(owner), &value, at)
            .into_iter()
            .map(|i| {
                let detail = i
                    .arg("field")
                    .or(i.arg("key"))
                    .unwrap_or_default()
                    .to_owned();
                (i.slug.to_string(), detail)
            })
            .collect()
    }

    fn page() -> SchemaOwner {
        SchemaOwner::Type("guide".into())
    }

    #[test]
    fn valid_frontmatter_passes() {
        let yaml = "title: Hi\nlevel: a\ntags: [x, y]\nupdated: 2024-02-29\nauthor: {name: K}\navailable: cloud\nvariant: {pm: npm}";
        assert!(run(page(), yaml).is_empty());
    }

    #[test]
    fn errors_are_reported_by_slug_and_path() {
        let got = run(
            page(),
            "level: c\ntags: [x, 1]\nupdated: 2026-02-30\nauthor: {}\nbogus: 1\n",
        );
        let slugs: Vec<_> = got.iter().map(|(s, d)| format!("{s} {d}")).collect();
        assert_eq!(
            slugs,
            [
                "frontmatter-type-mismatch level",
                "frontmatter-type-mismatch tags[1]",
                "frontmatter-type-mismatch updated",
                "frontmatter-missing-field author.name",
                "frontmatter-unknown-key bogus",
                "frontmatter-missing-field title",
            ]
        );
    }

    #[test]
    fn core_schema_reads_yes_as_a_string_and_3_10_as_a_number() {
        assert!(run(page(), "title: yes").is_empty());
        assert_eq!(run(page(), "title: 3.10").len(), 1);
    }

    #[test]
    fn empty_frontmatter_lacks_required_fields() {
        let got = run(page(), "");
        assert_eq!(got, [("frontmatter-missing-field".into(), "title".into())]);
    }

    #[test]
    fn fragments_reject_reserved_keys() {
        let got = run(
            SchemaOwner::Fragment,
            "title: x\navailable: cloud\nvariant: {}",
        );
        assert_eq!(
            got.iter().map(|(s, _)| s.as_str()).collect::<Vec<_>>(),
            [
                "frontmatter-reserved-in-fragment",
                "frontmatter-reserved-in-fragment"
            ]
        );
    }

    #[test]
    fn did_you_mean() {
        let value: Value = serde_yaml::from_str("title: x\ntitel: y").unwrap();
        let at = Location::new(FileId::new(0), 0..0);
        let issues = validate_frontmatter(&schema(page()), &value, at);
        assert_eq!(issues[0].variant, Some("suggestion"));
        assert_eq!(issues[0].arg("suggestion"), Some("title"));
    }

    #[test]
    fn dates() {
        assert!(is_calendar_date("2024-02-29") && !is_calendar_date("2023-02-29"));
        assert!(!is_calendar_date("2024-13-01") && !is_calendar_date("2024-1-01"));
    }
}
