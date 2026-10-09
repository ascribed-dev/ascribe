//! What the content model allows, resolved: its page types and their
//! frontmatter, dimensions, phrases, features, glossary, widgets, and
//! builds. [`model`] gives every section in full, for tools; [`summary`]
//! gives a short Markdown summary for an agent to read, cut to a character
//! budget.

use ascribe_core::schema::{AttributeType, Attributes, DefaultValue, SetMember};
use ascribe_model::{AvailabilityMode, ContentModel, Field, FieldType, GlossaryMatch, VariantMode};
use serde::Serialize;

use crate::{ASCRIBE_VERSION, SCHEMA_VERSION};

/// A section of the content model.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Section {
    /// Page types and their frontmatter.
    Types,
    /// Dimensions and their values.
    Dimensions,
    /// Phrases.
    Phrases,
    /// Features.
    Features,
    /// Glossary terms.
    Glossary,
    /// Project widgets.
    Widgets,
    /// Builds.
    Builds,
}

impl Section {
    /// Every section, in the order answers list them.
    pub const ALL: [Section; 7] = [
        Section::Types,
        Section::Dimensions,
        Section::Phrases,
        Section::Features,
        Section::Glossary,
        Section::Widgets,
        Section::Builds,
    ];

    /// The section's name, as `--section` takes it.
    pub fn name(self) -> &'static str {
        match self {
            Section::Types => "types",
            Section::Dimensions => "dimensions",
            Section::Phrases => "phrases",
            Section::Features => "features",
            Section::Glossary => "glossary",
            Section::Widgets => "widgets",
            Section::Builds => "builds",
        }
    }

    fn heading(self) -> &'static str {
        match self {
            Section::Types => "Page types",
            Section::Dimensions => "Dimensions",
            Section::Phrases => "Phrases",
            Section::Features => "Features",
            Section::Glossary => "Glossary",
            Section::Widgets => "Widgets",
            Section::Builds => "Builds",
        }
    }
}

/// What `ascribe model --format json` answers: the content model's sections,
/// or the one asked for.
#[derive(Debug, Serialize)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
pub struct ModelReport {
    /// The version of this schema. It changes only when a field is removed
    /// or changes meaning.
    pub schema_version: u32,
    /// The version of Ascribe that wrote it.
    pub ascribe_version: &'static str,
    /// Page types, in declaration order: the implicit `page` type when none
    /// is declared.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub types: Option<Vec<ModelType>>,
    /// Dimensions, in declaration order.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dimensions: Option<Vec<ModelDimension>>,
    /// Phrases, in declaration order.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub phrases: Option<Vec<ModelPhrase>>,
    /// Features, in declaration order.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub features: Option<Vec<ModelFeature>>,
    /// Glossary terms, in declaration order.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub glossary: Option<Vec<ModelTerm>>,
    /// Project widgets, in declaration order.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub widgets: Option<Vec<ModelWidget>>,
    /// Builds, in declaration order.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub builds: Option<Vec<ModelBuild>>,
}

/// A page type.
#[derive(Debug, Serialize)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
pub struct ModelType {
    /// Its name.
    pub name: String,
    /// The patterns of the pages it applies to, relative to the content root.
    pub files: Vec<String>,
    /// Whether it applies to pages no type's `files` match.
    pub default: bool,
    /// Its frontmatter fields, in declaration order.
    pub fields: Vec<ModelField>,
}

/// A frontmatter field.
#[derive(Debug, Serialize)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
pub struct ModelField {
    /// Its name.
    pub name: String,
    /// Its type in short form: `string`, `list(string)`, `enum(a, b)`.
    #[serde(rename = "type")]
    pub field_type: String,
    /// Whether every page of the type must give it.
    pub required: bool,
    /// The values it allows, for an enumeration or a list of one; empty
    /// otherwise.
    pub values: Vec<String>,
    /// What it's for, as the content model describes it.
    pub description: Option<String>,
    /// An object's fields; empty for any other type.
    pub fields: Vec<ModelField>,
}

/// A dimension.
#[derive(Debug, Serialize)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
pub struct ModelDimension {
    /// Its name.
    pub name: String,
    /// Its label.
    pub label: String,
    /// Its values, in display order.
    pub values: Vec<ModelDimensionValue>,
}

/// A value of a dimension.
#[derive(Debug, Serialize)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
pub struct ModelDimensionValue {
    /// The value.
    pub value: String,
    /// Its label.
    pub label: String,
    /// Whether it has no versions, so an availability spec gives it a state
    /// but no version.
    pub versionless: bool,
}

/// A phrase.
#[derive(Debug, Serialize)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
pub struct ModelPhrase {
    /// Its key, written `{key}` in a page.
    pub key: String,
    /// Its value.
    pub value: String,
}

/// A feature.
#[derive(Debug, Serialize)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
pub struct ModelFeature {
    /// Its key, which an availability spec can name.
    pub key: String,
    /// Its name.
    pub name: String,
    /// Its availability spec, as written.
    pub availability: String,
}

/// A glossary term.
#[derive(Debug, Serialize)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
pub struct ModelTerm {
    /// Its id.
    pub id: String,
    /// The term.
    pub term: String,
    /// Other ways it's written.
    pub aliases: Vec<String>,
    /// Its definition.
    pub definition: String,
    /// Which occurrences are linked: `first` on each page, `every` one, or
    /// only those an author links (`marked`).
    #[serde(rename = "match")]
    pub match_mode: &'static str,
}

/// A project widget.
#[derive(Debug, Serialize)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
pub struct ModelWidget {
    /// Its name, written `@name`.
    pub name: String,
    /// What it's for, as the content model describes it.
    pub description: Option<String>,
    /// Whether it may be one line.
    pub line: bool,
    /// Whether it may be a container, closed by `@end`.
    pub container: bool,
    /// Its attributes, in canonical order.
    pub attributes: Vec<ModelAttribute>,
}

/// An attribute a widget accepts.
#[derive(Debug, Serialize)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
pub struct ModelAttribute {
    /// The key.
    pub key: String,
    /// The value's type: `string`, `number`, `boolean`, `enum`, `set`, or
    /// `note-type`.
    #[serde(rename = "type")]
    pub value_type: &'static str,
    /// The values it allows, for `enum` and a `set` of named values; empty
    /// otherwise.
    pub values: Vec<String>,
    /// Whether every use must give it.
    pub required: bool,
    /// The value used when it's left out, as written (a set's members joined
    /// by `|`).
    pub default: Option<String>,
    /// What it's for.
    pub description: Option<String>,
}

/// A build.
#[derive(Debug, Serialize)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
pub struct ModelBuild {
    /// Its name.
    pub name: String,
    /// Which variant arms it keeps: `switch` for all of them, or the
    /// selection, such as `deployment=cloud`.
    pub variants: String,
    /// How it treats availability: `badge`, or the target it filters for,
    /// such as `filter self-managed 3.3`.
    pub availability: String,
    /// Whether it's the build the editor checks (`[editor] build`).
    pub editor: bool,
}

/// The content model's sections, or only `section`.
pub fn model(model: &ContentModel, section: Option<Section>) -> ModelReport {
    let wants = |s: Section| section.is_none_or(|wanted| wanted == s).then_some(());
    ModelReport {
        schema_version: SCHEMA_VERSION,
        ascribe_version: ASCRIBE_VERSION,
        types: wants(Section::Types).map(|()| types(model)),
        dimensions: wants(Section::Dimensions).map(|()| dimensions(model)),
        phrases: wants(Section::Phrases).map(|()| {
            model
                .phrases
                .iter()
                .map(|p| ModelPhrase {
                    key: p.key.clone(),
                    value: p.value.clone(),
                })
                .collect()
        }),
        features: wants(Section::Features).map(|()| {
            model
                .features
                .iter()
                .map(|f| ModelFeature {
                    key: f.key.clone(),
                    name: f.name.clone(),
                    availability: f.available_text.clone(),
                })
                .collect()
        }),
        glossary: wants(Section::Glossary).map(|()| {
            model
                .glossary
                .terms
                .iter()
                .map(|t| ModelTerm {
                    id: t.id.clone(),
                    term: t.term.clone(),
                    aliases: t.aliases.clone(),
                    definition: t.definition.clone(),
                    match_mode: match t.match_mode {
                        GlossaryMatch::First => "first",
                        GlossaryMatch::Every => "every",
                        GlossaryMatch::Marked => "marked",
                    },
                })
                .collect()
        }),
        widgets: wants(Section::Widgets).map(|()| widgets(model)),
        builds: wants(Section::Builds).map(|()| builds(model)),
    }
}

fn types(model: &ContentModel) -> Vec<ModelType> {
    model
        .types
        .iter()
        .map(|t| ModelType {
            name: t.name.clone(),
            files: t.files.iter().map(|p| p.as_str().to_owned()).collect(),
            default: t.default,
            fields: t.frontmatter.fields.iter().map(field).collect(),
        })
        .collect()
}

fn field(f: &Field) -> ModelField {
    let values = match &f.ty {
        FieldType::Enum(values) => values.clone(),
        FieldType::List(inner) => match &**inner {
            FieldType::Enum(values) => values.clone(),
            _ => Vec::new(),
        },
        _ => Vec::new(),
    };
    ModelField {
        name: f.name.clone(),
        field_type: f.ty.describe(),
        required: f.required,
        values,
        description: f.description.clone(),
        fields: match &f.ty {
            FieldType::Object(fields) => fields.iter().map(field).collect(),
            _ => Vec::new(),
        },
    }
}

fn dimensions(model: &ContentModel) -> Vec<ModelDimension> {
    model
        .dimensions
        .iter()
        .map(|d| ModelDimension {
            name: d.name.clone(),
            label: d.label.clone(),
            values: d
                .values
                .iter()
                .map(|v| ModelDimensionValue {
                    value: v.value.clone(),
                    label: v.label.clone(),
                    versionless: v.versionless,
                })
                .collect(),
        })
        .collect()
}

fn widgets(model: &ContentModel) -> Vec<ModelWidget> {
    model
        .widgets
        .iter()
        .map(|w| {
            let schema = &w.schema;
            let attributes = match &schema.attributes {
                Attributes::Declared(list) => list
                    .iter()
                    .map(|a| {
                        let (value_type, values) = match &a.ty {
                            AttributeType::String => ("string", Vec::new()),
                            AttributeType::Number => ("number", Vec::new()),
                            AttributeType::Boolean => ("boolean", Vec::new()),
                            AttributeType::Enum(values) => ("enum", values.clone()),
                            AttributeType::Set(SetMember::String) => ("set", Vec::new()),
                            AttributeType::Set(SetMember::Enum(values)) => ("set", values.clone()),
                            AttributeType::NoteType => ("note-type", Vec::new()),
                        };
                        ModelAttribute {
                            key: a.key.clone(),
                            value_type,
                            values,
                            required: a.required,
                            default: a.default.as_ref().map(|d| match d {
                                DefaultValue::Text(text) => text.clone(),
                                DefaultValue::Boolean(b) => b.to_string(),
                                DefaultValue::Set(members) => members.join("|"),
                            }),
                            description: a.description.clone(),
                        }
                    })
                    .collect(),
                Attributes::Dimensions => Vec::new(),
            };
            ModelWidget {
                name: schema.name.clone(),
                description: schema.description.clone(),
                line: schema.forms.line,
                container: schema.forms.container,
                attributes,
            }
        })
        .collect()
}

fn builds(model: &ContentModel) -> Vec<ModelBuild> {
    model
        .builds
        .iter()
        .map(|b| ModelBuild {
            name: b.name.clone(),
            variants: match &b.variants {
                VariantMode::Switch => "switch".to_owned(),
                VariantMode::Select(selection) => selection
                    .iter()
                    .map(|(dimension, values)| format!("{dimension}={}", values.join("|")))
                    .collect::<Vec<_>>()
                    .join(", "),
            },
            availability: match &b.availability {
                AvailabilityMode::Badge => "badge".to_owned(),
                AvailabilityMode::Filter { target, version } => match version {
                    Some(version) => format!("filter {target} {}", version.text),
                    None => format!("filter {target}"),
                },
            },
            editor: b.name == model.editor_build,
        })
        .collect()
}

/// The content model as a short Markdown summary, for an agent to read: a
/// heading per section that declares anything, and a line per entry. When
/// the whole would be longer than `budget` characters, long lists are cut,
/// each where it's cut naming the command that lists the rest. With a
/// `section`, only that one, never cut.
pub fn summary(model: &ContentModel, section: Option<Section>, budget: usize) -> String {
    let report = self::model(model, section);
    let sections: Vec<(Section, Vec<String>)> = Section::ALL
        .into_iter()
        .filter(|s| section.is_none_or(|wanted| wanted == *s))
        .map(|s| (s, lines(&report, s)))
        .filter(|(s, lines)| !lines.is_empty() || section == Some(*s))
        .collect();
    let all: Vec<usize> = sections.iter().map(|(_, lines)| lines.len()).collect();
    if section.is_some() || render(&sections, &all).chars().count() <= budget {
        return render(&sections, &all);
    }
    // Take entries a round at a time, one from each section in turn, while
    // the result fits: every section keeps its first entries, and a long one
    // gives up its tail.
    let mut taken = vec![0; sections.len()];
    loop {
        let mut grew = false;
        for i in 0..sections.len() {
            if taken[i] == all[i] {
                continue;
            }
            taken[i] += 1;
            if render(&sections, &taken).chars().count() <= budget {
                grew = true;
            } else {
                taken[i] -= 1;
            }
        }
        if !grew {
            break;
        }
    }
    render(&sections, &taken)
}

/// The summary with the first `taken[i]` lines of section `i`.
fn render(sections: &[(Section, Vec<String>)], taken: &[usize]) -> String {
    let mut out = String::from("# Content model\n");
    for ((section, lines), &n) in sections.iter().zip(taken) {
        out.push_str(&format!("\n## {}\n\n", section.heading()));
        if lines.is_empty() {
            out.push_str("None.\n");
        }
        for line in &lines[..n] {
            out.push_str(&format!("- {line}\n"));
        }
        if n < lines.len() {
            out.push_str(&format!(
                "- …and {} more: `ascribe model --section {}`\n",
                lines.len() - n,
                section.name()
            ));
        }
    }
    out
}

/// A line for each entry of a section.
fn lines(report: &ModelReport, section: Section) -> Vec<String> {
    let code = |s: &str| format!("`{s}`");
    match section {
        Section::Types => report.types.iter().flatten().map(type_line).collect(),
        Section::Dimensions => report
            .dimensions
            .iter()
            .flatten()
            .map(|d| {
                let values: Vec<String> = d
                    .values
                    .iter()
                    .map(|v| {
                        let mut text = code(&v.value);
                        if v.versionless {
                            text.push_str(" (versionless)");
                        }
                        text
                    })
                    .collect();
                format!("{} ({}): {}", code(&d.name), d.label, values.join(", "))
            })
            .collect(),
        Section::Phrases => report
            .phrases
            .iter()
            .flatten()
            .map(|p| format!("`{{{}}}`: {}", p.key, p.value))
            .collect(),
        Section::Features => report
            .features
            .iter()
            .flatten()
            .map(|f| format!("{} ({}): {}", code(&f.key), f.name, f.availability))
            .collect(),
        Section::Glossary => report
            .glossary
            .iter()
            .flatten()
            .map(|t| {
                let mut text = format!("{}: {}", code(&t.id), t.term);
                if !t.aliases.is_empty() {
                    text.push_str(&format!(" (also {})", t.aliases.join(", ")));
                }
                text
            })
            .collect(),
        Section::Widgets => report.widgets.iter().flatten().map(widget_line).collect(),
        Section::Builds => report
            .builds
            .iter()
            .flatten()
            .map(|b| {
                let editor = if b.editor { ", the editor's build" } else { "" };
                format!(
                    "{}{editor}: variants {}, availability {}",
                    code(&b.name),
                    b.variants,
                    b.availability
                )
            })
            .collect(),
    }
}

fn type_line(t: &ModelType) -> String {
    let mut text = format!("`{}`", t.name);
    let mut notes = Vec::new();
    if t.default {
        notes.push("the default".to_owned());
    }
    if !t.files.is_empty() {
        let files: Vec<String> = t.files.iter().map(|f| format!("`{f}`")).collect();
        notes.push(format!("files {}", files.join(", ")));
    }
    if !notes.is_empty() {
        text.push_str(&format!(" ({})", notes.join("; ")));
    }
    let fields: Vec<String> = t
        .fields
        .iter()
        .map(|f| {
            let required = if f.required { ", required" } else { "" };
            format!("`{}` ({}{required})", f.name, f.field_type)
        })
        .collect();
    if fields.is_empty() {
        text.push_str(": no frontmatter fields");
    } else {
        text.push_str(&format!(": frontmatter {}", fields.join(", ")));
    }
    text
}

fn widget_line(w: &ModelWidget) -> String {
    let forms = match (w.line, w.container) {
        (true, true) => "line or container",
        (false, true) => "container",
        _ => "line",
    };
    let mut text = format!("`@{}` ({forms})", w.name);
    if !w.attributes.is_empty() {
        let attributes: Vec<String> = w
            .attributes
            .iter()
            .map(|a| {
                let ty = if a.values.is_empty() {
                    a.value_type.to_owned()
                } else {
                    format!("{}: {}", a.value_type, a.values.join("|"))
                };
                let required = if a.required { ", required" } else { "" };
                format!("`{}` ({ty}{required})", a.key)
            })
            .collect();
        text.push_str(&format!(": {}", attributes.join(", ")));
    }
    if let Some(description) = &w.description {
        text.push_str(&format!(". {description}"));
    }
    text
}
