//! What the content model allows, resolved: its page types and their
//! frontmatter, dimensions, phrases, features, glossary, widgets, builds,
//! and sources. [`model`] gives every section in full, for tools; [`summary`]
//! gives a short Markdown summary for an agent to read, cut to a character
//! budget.

use ascribe_core::schema::{AttributeType, Attributes, DefaultValue, SetMember};
use std::collections::BTreeMap;

use ascribe_core::RelPath;
use ascribe_model::examples::Status;
use ascribe_model::{
    AvailabilityMode, ContentModel, Field, FieldType, GlossaryMatch, Source, VariantMode,
};
use ascribe_resolve::{FileSystem, FolderExamples, Probe};
use serde::Serialize;

use crate::ASCRIBE_VERSION;

/// The version of [`ModelReport`]'s schema: 2 since a type has no `files` or
/// `default` (decision 450).
pub const MODEL_SCHEMA_VERSION: u32 = 2;

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
    /// Sources.
    Sources,
}

impl Section {
    /// Every section, in the order answers list them.
    pub const ALL: [Section; 8] = [
        Section::Types,
        Section::Dimensions,
        Section::Phrases,
        Section::Features,
        Section::Glossary,
        Section::Widgets,
        Section::Builds,
        Section::Sources,
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
            Section::Sources => "sources",
        }
    }

    pub(crate) fn heading(self) -> &'static str {
        match self {
            Section::Types => "Page types",
            Section::Dimensions => "Dimensions",
            Section::Phrases => "Phrases",
            Section::Features => "Features",
            Section::Glossary => "Glossary",
            Section::Widgets => "Widgets",
            Section::Builds => "Builds",
            Section::Sources => "Sources",
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
    /// Sources, in declaration order.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sources: Option<Vec<ModelSource>>,
}

/// A page type.
#[derive(Debug, Serialize)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
pub struct ModelType {
    /// Its name, which each page of the type gives as its `type`; the
    /// implicit `page` type's pages give none.
    pub name: String,
    /// What a page of the type is for, as the content model says; absent
    /// when it says nothing.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub purpose: Option<String>,
    /// The page a page of the type starts from, from the folder
    /// `ascribe.toml` is in; absent when the type names none.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub template: Option<String>,
    /// What to read before writing a page of the type, in the order to read
    /// it, from the folder `ascribe.toml` is in; absent when the type names
    /// none.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub guide: Vec<String>,
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

/// A source: a folder of files, such as code, that pages can show.
#[derive(Debug, Serialize)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
pub struct ModelSource {
    /// Its name, as addresses write it: `code:` for a source named `code`.
    pub name: String,
    /// Its folder, relative to the project's folder: the `path` written,
    /// or, for a source in another repository, the folder of its copies.
    pub path: String,
    /// The patterns of the files it makes readable, relative to its folder;
    /// empty for every file.
    pub include: Vec<String>,
    /// The patterns of the files it doesn't make readable, though `include`
    /// matches them.
    pub ignore: Vec<String>,
    /// The repository it's in, for a source in another repository.
    pub git: Option<ModelRemote>,
    /// Its tested examples, for a source with an `ascribe.examples.toml`:
    /// absent for any other. For a source in another repository, they're
    /// read from its copies.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub examples: Option<ModelExamples>,
}

/// A source's tested examples.
#[derive(Debug, Serialize)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
pub struct ModelExamples {
    /// Whether its `ascribe.examples.toml` can be read. One that can't makes
    /// no file an example.
    pub valid: bool,
    /// How many of the files the source makes readable its config makes
    /// examples.
    pub files: usize,
    /// Its observations, in name order: those that can't be read left out.
    pub observations: Vec<ModelObservation>,
}

/// An observation of a source's examples.
#[derive(Debug, Serialize)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
pub struct ModelObservation {
    /// Its name: `default`, or the name `ascribe observe --name` gave it.
    pub name: String,
    /// Its environment as it has it, by name: `os`, `arch`, and each
    /// entry's kept line.
    pub environment: BTreeMap<String, String>,
    /// What a page says it was tested with, by this observation alone.
    pub tested: String,
    /// How many files it lists. For a source in another repository, only
    /// those it has copies of, since the others weren't copied.
    pub listed: usize,
    /// How many of those are as it found them: the rest changed since, or
    /// are gone.
    pub current: usize,
}

/// The repository a source is in.
#[derive(Debug, Serialize)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
pub struct ModelRemote {
    /// Its URL.
    pub url: String,
    /// The branch an update follows; `null` for the repository's default.
    pub branch: Option<String>,
}

/// The content model's sections, or only `section`. With `fs`, each source in
/// the same repository says what its tested examples are.
pub fn model(
    model: &ContentModel,
    section: Option<Section>,
    fs: Option<&dyn FileSystem>,
) -> ModelReport {
    let wants = |s: Section| section.is_none_or(|wanted| wanted == s).then_some(());
    ModelReport {
        schema_version: MODEL_SCHEMA_VERSION,
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
        sources: wants(Section::Sources).map(|()| sources(model, fs)),
    }
}

fn sources(model: &ContentModel, fs: Option<&dyn FileSystem>) -> Vec<ModelSource> {
    let patterns = |list: &[ascribe_model::Pattern]| -> Vec<String> {
        list.iter().map(|p| p.as_str().to_owned()).collect()
    };
    model
        .sources
        .iter()
        .map(|s| ModelSource {
            name: s.name.clone(),
            path: s.path.clone(),
            include: patterns(&s.include),
            ignore: patterns(&s.ignore),
            git: s.git.as_ref().map(|g| ModelRemote {
                url: g.url.clone(),
                branch: g.branch.clone(),
            }),
            examples: fs.and_then(|fs| examples(s, fs)),
        })
        .collect()
}

/// A source's tested examples, read through `fs`: `None` for a source
/// without an `ascribe.examples.toml`. A source in another repository's are
/// read from its copies.
fn examples(source: &Source, fs: &dyn FileSystem) -> Option<ModelExamples> {
    let folder = RelPath::parse(&source.path).ok()?;
    let read = FolderExamples::read(fs, &folder);
    let config = read.config.as_ref()?;
    let Ok(config) = config else {
        return Some(ModelExamples {
            valid: false,
            files: 0,
            observations: Vec::new(),
        });
    };
    let files = fs
        .files_in(&folder)
        .iter()
        .filter_map(|p| p.relative_to(&folder))
        .filter(|p| source.reads(p.as_str()) && config.is_example(p.as_str()))
        .count();
    let observations = read
        .readable()
        .map(|(name, observation)| {
            // A file's bytes, or `None` when it's gone.
            let bytes = |path: &str| {
                folder
                    .join(path)
                    .ok()
                    .filter(|p| fs.probe(p) == Probe::File)
                    .and_then(|p| fs.read_file(&p).ok())
            };
            // A copy has only the files snippets use: the others say nothing.
            let listed: Vec<(&String, Option<Vec<u8>>)> = observation
                .files
                .keys()
                .map(|path| (path, bytes(path)))
                .filter(|(_, bytes)| source.git.is_none() || bytes.is_some())
                .collect();
            ModelObservation {
                name: name.to_owned(),
                environment: observation.environment.iter().cloned().collect(),
                tested: observation.tested_line(config),
                listed: listed.len(),
                current: listed
                    .iter()
                    .filter(|(path, bytes)| {
                        bytes
                            .as_ref()
                            .is_some_and(|b| observation.status(path, b) == Status::Current)
                    })
                    .count(),
            }
        })
        .collect();
    Some(ModelExamples {
        valid: true,
        files,
        observations,
    })
}

fn types(model: &ContentModel) -> Vec<ModelType> {
    model
        .types
        .iter()
        .map(|t| ModelType {
            name: t.name.clone(),
            purpose: t.purpose.clone(),
            template: t.template.as_ref().map(|f| f.path.clone()),
            guide: t.guide.iter().map(|f| f.path.clone()).collect(),
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
pub fn summary(
    model: &ContentModel,
    section: Option<Section>,
    budget: usize,
    fs: Option<&dyn FileSystem>,
) -> String {
    let report = self::model(model, section, fs);
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
    let taken = fit(&all, |taken| {
        render(&sections, taken).chars().count() <= budget
    });
    render(&sections, &taken)
}

/// How many entries of each list to keep, given each list's length (`all`)
/// and whether a choice `fits`. Entries are taken a round at a time, one
/// from each list in turn, while the result fits: every list keeps its first
/// entries, and a long one gives up its tail.
pub(crate) fn fit(all: &[usize], fits: impl Fn(&[usize]) -> bool) -> Vec<usize> {
    let mut taken = vec![0; all.len()];
    loop {
        let mut grew = false;
        for i in 0..all.len() {
            if taken[i] == all[i] {
                continue;
            }
            taken[i] += 1;
            if fits(&taken) {
                grew = true;
            } else {
                taken[i] -= 1;
            }
        }
        if !grew {
            break;
        }
    }
    taken
}

/// A line of Markdown for each entry of a section, as the summary lists
/// them.
pub(crate) fn entries(model: &ContentModel, section: Section) -> Vec<String> {
    lines(&self::model(model, Some(section), None), section)
}

/// A line of Markdown for each page type, as the summary lists them, saying
/// what the type is for: its own `purpose`, else its entry in `summaries`.
pub(crate) fn type_entries(
    model: &ContentModel,
    summaries: &std::collections::BTreeMap<String, String>,
) -> Vec<String> {
    types(model)
        .into_iter()
        .map(|mut t| {
            if t.purpose.is_none() {
                t.purpose = summaries.get(&t.name).cloned();
            }
            type_line(&t)
        })
        .collect()
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
        Section::Sources => report.sources.iter().flatten().map(source_line).collect(),
    }
}

fn source_line(s: &ModelSource) -> String {
    let code = |list: &[String]| -> String {
        list.iter()
            .map(|p| format!("`{p}`"))
            .collect::<Vec<_>>()
            .join(", ")
    };
    let mut notes = vec![format!("folder `{}`", s.path)];
    if !s.include.is_empty() {
        notes.push(format!("files {}", code(&s.include)));
    }
    if !s.ignore.is_empty() {
        notes.push(format!("not {}", code(&s.ignore)));
    }
    if let Some(git) = &s.git {
        match &git.branch {
            Some(branch) => notes.push(format!("from `{}`, branch `{branch}`", git.url)),
            None => notes.push(format!("from `{}`", git.url)),
        }
    }
    if let Some(examples) = &s.examples {
        notes.push(examples_note(examples));
    }
    format!("`{}:` ({})", s.name, notes.join("; "))
}

/// What a source's tested examples are, for the summary: how many files,
/// then each observation, what it says it was tested with, and how many of
/// its files are as it found them.
fn examples_note(examples: &ModelExamples) -> String {
    if !examples.valid {
        return "an `ascribe.examples.toml` that can't be read".to_owned();
    }
    let mut note = format!(
        "{} tested {}",
        examples.files,
        if examples.files == 1 {
            "example"
        } else {
            "examples"
        }
    );
    if examples.observations.is_empty() {
        note.push_str(", no observations");
    }
    for o in &examples.observations {
        note.push_str(&format!(
            ", observation `{}`: {}, {} of {} files as observed",
            o.name,
            markdown_text(&o.tested),
            o.current,
            o.listed
        ));
    }
    note
}

/// Text from an observation, already made safe to show, with what Markdown
/// would read as markup escaped.
fn markdown_text(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for c in text.chars() {
        if matches!(
            c,
            '\\' | '`' | '*' | '_' | '[' | ']' | '<' | '>' | '&' | '#' | '|'
        ) {
            out.push('\\');
        }
        out.push(c);
    }
    out
}

fn type_line(t: &ModelType) -> String {
    let mut text = format!("`{}`", t.name);
    let mut notes = Vec::new();
    if let Some(template) = &t.template {
        notes.push(format!("template `{template}`"));
    }
    if !t.guide.is_empty() {
        let guide: Vec<String> = t.guide.iter().map(|g| format!("`{g}`")).collect();
        notes.push(format!("guide {}", guide.join(", ")));
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
    if let Some(purpose) = &t.purpose {
        // One line, so a `purpose` written over several can't end the list
        // item it's in.
        let purpose = purpose.split_whitespace().collect::<Vec<_>>().join(" ");
        text.push_str(&format!(". {}", purpose.trim_end_matches('.')));
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
