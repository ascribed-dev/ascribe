//! What the content model allows, resolved: its page types and their
//! frontmatter, dimensions, phrases, features, glossary, widgets, builds,
//! and sources, and every other section of `ascribe.toml`, each key with its
//! value in effect. [`model`] gives every section in full, for tools;
//! [`summary`] gives a short Markdown summary for an agent to read, cut to a
//! character budget.

use ascribe_core::schema::{AttributeType, Attributes, DefaultValue, SetMember};
use std::collections::BTreeMap;

use ascribe_core::RelPath;
use ascribe_model::examples::Status;
use ascribe_model::settings::{self as registry, Setting, SettingValue};
use ascribe_model::{
    AvailabilityMode, ContentModel, Field, FieldType, GlossaryMatch, Listing, Source, VariantMode,
};
use ascribe_resolve::navigation::ROOT_LABEL;
use ascribe_resolve::{
    FileSystem, FolderExamples, Probe, Project, Section as NavSection, file_label,
};
use serde::Serialize;

use crate::ASCRIBE_VERSION;

/// The version of [`ModelReport`]'s schema: 3 since a glossary term has no
/// `definition` or `link`, and says where its heading is (decision 545a).
pub const MODEL_SCHEMA_VERSION: u32 = 3;

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
    /// Release lines: each target's, with its status.
    Releases,
    /// Builds.
    Builds,
    /// Sources.
    Sources,
    /// `[project]`'s settings.
    Project,
    /// `[consumer]`'s settings.
    Consumer,
    /// `[editor]`'s settings.
    Editor,
    /// `[checks]`'s settings.
    Checks,
    /// `[crawlers]`'s settings.
    Crawlers,
    /// `[listing]`'s settings.
    Listing,
    /// `[permalinks]`'s settings.
    Permalinks,
}

impl Section {
    /// Every section, in the order answers list them.
    pub const ALL: [Section; 16] = [
        Section::Types,
        Section::Dimensions,
        Section::Phrases,
        Section::Features,
        Section::Glossary,
        Section::Widgets,
        Section::Releases,
        Section::Builds,
        Section::Sources,
        Section::Project,
        Section::Consumer,
        Section::Editor,
        Section::Checks,
        Section::Crawlers,
        Section::Listing,
        Section::Permalinks,
    ];

    /// Whether it's a table of settings, each key [`ModelSetting`]: shown
    /// with `--section`, and only named in the summary without it.
    pub fn is_settings(self) -> bool {
        matches!(
            self,
            Section::Project
                | Section::Consumer
                | Section::Editor
                | Section::Checks
                | Section::Crawlers
                | Section::Listing
                | Section::Permalinks
        )
    }

    /// The section's name, as `--section` takes it.
    pub fn name(self) -> &'static str {
        match self {
            Section::Types => "types",
            Section::Dimensions => "dimensions",
            Section::Phrases => "phrases",
            Section::Features => "features",
            Section::Glossary => "glossary",
            Section::Widgets => "widgets",
            Section::Releases => "releases",
            Section::Builds => "builds",
            Section::Sources => "sources",
            Section::Project => "project",
            Section::Consumer => "consumer",
            Section::Editor => "editor",
            Section::Checks => "checks",
            Section::Crawlers => "crawlers",
            Section::Listing => "listing",
            Section::Permalinks => "permalinks",
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
            Section::Releases => "Releases",
            Section::Builds => "Builds",
            Section::Sources => "Sources",
            Section::Project => "`[project]`",
            Section::Consumer => "`[consumer]`",
            Section::Editor => "`[editor]`",
            Section::Checks => "`[checks]`",
            Section::Crawlers => "`[crawlers]`",
            Section::Listing => "`[listing]`",
            Section::Permalinks => "`[permalinks]`",
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
    /// Glossary terms, in the order of `[glossary] pages`, and on each page
    /// in document order.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub glossary: Option<Vec<ModelTerm>>,
    /// Project widgets, in declaration order.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub widgets: Option<Vec<ModelWidget>>,
    /// `[releases.<target>]`, in declaration order: each target's lines,
    /// with their status.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub releases: Option<Vec<ModelReleases>>,
    /// Builds, in declaration order, a versioned build as one for each live
    /// line, the current line first.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub builds: Option<Vec<ModelBuild>>,
    /// Sources, in declaration order.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sources: Option<Vec<ModelSource>>,
    /// `[project]`: each key, with its value in effect.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub project: Option<Vec<ModelSetting>>,
    /// `[consumer]`: each key, with its value in effect.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub consumer: Option<Vec<ModelSetting>>,
    /// `[editor]`: each key, with its value in effect.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub editor: Option<Vec<ModelSetting>>,
    /// `[checks]`: each key of `[checks.vale]` and `[checks.links]`, with its
    /// value in effect, and each check the file sets, by its slug.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub checks: Option<Vec<ModelSetting>>,
    /// `[crawlers]`: each key, with its value in effect.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub crawlers: Option<Vec<ModelSetting>>,
    /// `[listing]`: each key, with its value in effect.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub listing: Option<Vec<ModelSetting>>,
    /// `[permalinks]`: each key, with its value in effect.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub permalinks: Option<Vec<ModelSetting>>,
    /// The project's navigation, its root section: every page that isn't
    /// `unlisted`, in order, as each build's `_ascribe/nav.json` has its
    /// share of it. Only in the whole report.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub navigation: Option<ModelNavSection>,
}

/// A section of the navigation: a folder and its pages, in order.
#[derive(Debug, Serialize)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
pub struct ModelNavSection {
    /// The folder, from the content root, ending in `/`; `""` for the root.
    pub folder: String,
    /// Its label: from `ascribe.nav`, else its own page's title as plain
    /// text, else its folder's name with `-` and `_` as spaces and its first
    /// letter capitalized, and "Start" for the root.
    pub label: String,
    /// Its own page: its `index.md`, or the page beside the folder with its
    /// name, which is then one of the parent's `pages` too.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page: Option<ModelNavPage>,
    /// Its pages in order, its own page not among them.
    pub pages: Vec<ModelNavPage>,
    /// Its nested sections with a page, in order: each one whose own page
    /// is among `pages` right after that page, then the rest.
    pub sections: Vec<ModelNavSection>,
    /// Whether `ascribe.nav` lists it.
    pub listed: bool,
}

/// A page of the navigation.
#[derive(Debug, Serialize)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
pub struct ModelNavPage {
    /// Its path from the content root.
    pub source: String,
    /// Its label as plain text: its `nav-label`, else its title, else its
    /// file name without `.md`.
    pub label: String,
    /// Its content type's name, when it has one.
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub type_name: Option<String>,
}

/// The project's navigation, for [`ModelReport::navigation`]: `ascribe.nav`'s
/// order over the default, every page that isn't `unlisted`, and the
/// sections left with one.
pub fn navigation(project: &Project) -> ModelNavSection {
    let navigation = project.navigation();
    nav_section(project, &navigation.root, true).unwrap_or_else(|| ModelNavSection {
        folder: String::new(),
        label: navigation
            .root
            .label
            .clone()
            .unwrap_or_else(|| ROOT_LABEL.to_owned()),
        page: None,
        pages: Vec::new(),
        sections: Vec::new(),
        listed: navigation.root.listed,
    })
}

fn nav_section(project: &Project, section: &NavSection, root: bool) -> Option<ModelNavSection> {
    let page = section.page.as_ref().and_then(|p| nav_page(project, p));
    let pages: Vec<ModelNavPage> = section
        .pages
        .iter()
        .filter_map(|p| nav_page(project, p))
        .collect();
    let sections: Vec<ModelNavSection> = section
        .sections
        .iter()
        .filter_map(|s| nav_section(project, s, false))
        .collect();
    if !root && page.is_none() && pages.is_empty() && sections.is_empty() {
        return None;
    }
    Some(ModelNavSection {
        folder: section.folder.clone(),
        label: section.label(project),
        page,
        pages,
        sections,
        listed: section.listed,
    })
}

/// The page at `path`, unless it's `unlisted`.
fn nav_page(project: &Project, path: &RelPath) -> Option<ModelNavPage> {
    let model = project.model();
    let file = project.file(path)?;
    if file.listing(model) == Listing::Unlisted {
        return None;
    }
    let type_name = file.content_type(model).map(|t| t.name.as_str());
    Some(ModelNavPage {
        source: path.to_string(),
        label: file_label(file, model),
        type_name: type_name.map(str::to_owned),
    })
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

/// A glossary term: a heading marked `@term` on a glossary page. Its
/// definition is the paragraph under the heading, which differs by build:
/// the JSON output of a build has it.
#[derive(Debug, Serialize)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
pub struct ModelTerm {
    /// Its id: its heading's id.
    pub id: String,
    /// The term: its heading's text.
    pub term: String,
    /// Other ways it's written.
    pub aliases: Vec<String>,
    /// Which occurrences are linked: `first` on each page, `every` one, or
    /// only those an author links (`marked`).
    #[serde(rename = "match")]
    pub match_mode: &'static str,
    /// Whether an occurrence must match its case.
    pub case_sensitive: bool,
    /// The file its heading is written in, from the content root: its
    /// glossary page, or a fragment the page includes.
    pub page: String,
    /// The heading's line in that file, from 1.
    pub line: usize,
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
    /// For a line of a versioned build, the target it's versioned by.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub versions: Option<String>,
    /// For a line of a versioned build, the line, as `[releases.<target>]`
    /// writes it.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line: Option<String>,
}

/// A target's release lines.
#[derive(Debug, Serialize)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
pub struct ModelReleases {
    /// The target: a versioned dimension value.
    pub target: String,
    /// Its lines, newest first: planned, live, then archived.
    pub lines: Vec<ModelLine>,
}

/// A release line.
#[derive(Debug, Serialize)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
pub struct ModelLine {
    /// The line, as the list writes it.
    pub line: String,
    /// `current`, `live`, `planned`, or `archived`.
    pub status: &'static str,
    /// The builds that are this line, one for each build versioned by the
    /// target: `site@3.4`, or `site` for the current line.
    pub builds: Vec<String>,
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

/// The project's glossary terms, as [`ModelReport::glossary`] lists them,
/// read from `index`, the project's source index.
pub fn terms(index: &Project) -> Vec<ModelTerm> {
    index
        .glossary()
        .terms
        .into_iter()
        .map(|t| {
            let line = index.file(&t.written_in).map_or(1, |f| {
                f.source
                    .get(..t.heading.start())
                    .map_or(0, |before| before.matches('\n').count())
                    + 1
            });
            ModelTerm {
                id: t.id,
                term: t.term,
                aliases: t.aliases,
                match_mode: match t.match_mode {
                    GlossaryMatch::First => "first",
                    GlossaryMatch::Every => "every",
                    GlossaryMatch::Marked => "marked",
                },
                case_sensitive: t.case_sensitive,
                page: t.written_in.to_string(),
                line,
            }
        })
        .collect()
}

/// The content model's sections, or only `section`, with the glossary's
/// `terms` ([`terms`]). `config` is the text of the `ascribe.toml` the model
/// was loaded from, for its settings' values: without it, each is its
/// default. With `fs`, each source in the same repository says what its
/// tested examples are.
pub fn model(
    model: &ContentModel,
    config: Option<&str>,
    terms: Vec<ModelTerm>,
    section: Option<Section>,
    fs: Option<&dyn FileSystem>,
) -> ModelReport {
    let wants = |s: Section| section.is_none_or(|wanted| wanted == s).then_some(());
    let file: Option<toml::Table> = config.and_then(|text| text.parse().ok());
    let settings = |s: Section| wants(s).map(|()| settings(s.name(), file.as_ref()));
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
        glossary: wants(Section::Glossary).map(|()| terms),
        widgets: wants(Section::Widgets).map(|()| widgets(model)),
        releases: wants(Section::Releases).map(|()| releases(model)),
        builds: wants(Section::Builds).map(|()| builds(model)),
        sources: wants(Section::Sources).map(|()| sources(model, fs)),
        project: settings(Section::Project),
        consumer: settings(Section::Consumer),
        editor: settings(Section::Editor).map(|mut keys| {
            // The build the loader chose, which isn't one default.
            for key in keys.iter_mut().filter(|k| k.key == "build" && !k.set) {
                key.value = Some(serde_json::Value::String(model.editor_build.clone()));
            }
            keys
        }),
        checks: settings(Section::Checks),
        crawlers: settings(Section::Crawlers),
        listing: settings(Section::Listing),
        permalinks: settings(Section::Permalinks),
        navigation: None,
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
            versions: b.line.as_ref().and(b.versions.clone()),
            line: b.line.as_ref().map(|l| l.line.text.clone()),
        })
        .collect()
}

fn releases(model: &ContentModel) -> Vec<ModelReleases> {
    model
        .releases
        .iter()
        .map(|releases| ModelReleases {
            target: releases.target.clone(),
            lines: releases
                .lines()
                .into_iter()
                .map(|(line, status)| ModelLine {
                    line: line.text.clone(),
                    status: status.as_str(),
                    builds: model
                        .builds
                        .iter()
                        .filter(|b| {
                            b.versions.as_deref() == Some(releases.target.as_str())
                                && b.line
                                    .as_ref()
                                    .is_some_and(|l| l.line.components == line.components)
                        })
                        .map(|b| b.name.clone())
                        .collect(),
                })
                .collect(),
        })
        .collect()
}

/// The content model as a short Markdown summary, for an agent to read: a
/// heading per section that declares anything, and a line per entry, then
/// the sections of settings, named. When the whole would be longer than
/// `budget` characters, long lists are cut, each where it's cut naming the
/// command that lists the rest. With a `section`, only that one, never cut:
/// a section of settings with every key and its value in effect.
pub fn summary(
    model: &ContentModel,
    config: Option<&str>,
    terms: Vec<ModelTerm>,
    section: Option<Section>,
    budget: usize,
    fs: Option<&dyn FileSystem>,
) -> String {
    let report = self::model(model, config, terms, section, fs);
    let sections: Vec<(Section, Vec<String>)> = Section::ALL
        .into_iter()
        .filter(|s| match section {
            Some(wanted) => wanted == *s,
            None => !s.is_settings(),
        })
        .map(|s| (s, lines(&report, s)))
        .filter(|(s, lines)| !lines.is_empty() || section == Some(*s))
        .collect();
    let all: Vec<usize> = sections.iter().map(|(_, lines)| lines.len()).collect();
    if section.is_some() {
        return render(&sections, &all);
    }
    let named = settings_named();
    let budget = budget.saturating_sub(named.chars().count());
    let taken = if render(&sections, &all).chars().count() <= budget {
        all
    } else {
        fit(&all, |taken| {
            render(&sections, taken).chars().count() <= budget
        })
    };
    render(&sections, &taken) + &named
}

/// The end of the whole model's summary: the sections of settings, which
/// only `--section` shows.
fn settings_named() -> String {
    let names: Vec<String> = Section::ALL
        .into_iter()
        .filter(|s| s.is_settings())
        .map(|s| format!("`{}`", s.name()))
        .collect();
    format!(
        "\n## Settings\n\n- {}, each with its keys and values: `ascribe model --section <name>`\n",
        names.join(", ")
    )
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

/// A line of Markdown for each entry of a section that the content model
/// declares alone (not the glossary's), as the summary lists them.
pub(crate) fn entries(model: &ContentModel, section: Section) -> Vec<String> {
    lines(
        &self::model(model, None, Vec::new(), Some(section), None),
        section,
    )
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
        Section::Releases => report
            .releases
            .iter()
            .flatten()
            .map(|r| {
                let lines: Vec<String> = r
                    .lines
                    .iter()
                    .map(|l| {
                        let builds: Vec<String> = l.builds.iter().map(|b| code(b)).collect();
                        if builds.is_empty() {
                            format!("{} ({})", l.line, l.status)
                        } else {
                            format!("{} ({}, {})", l.line, l.status, builds.join(", "))
                        }
                    })
                    .collect();
                format!("{}: {}", code(&r.target), lines.join("; "))
            })
            .collect(),
        Section::Builds => report
            .builds
            .iter()
            .flatten()
            .map(|b| {
                let editor = if b.editor { ", the editor's build" } else { "" };
                let line = match (&b.line, &b.versions) {
                    (Some(line), Some(target)) => format!(", line {line} of {}", code(target)),
                    _ => String::new(),
                };
                format!(
                    "{}{editor}: variants {}, availability {}{line}",
                    code(&b.name),
                    b.variants,
                    b.availability
                )
            })
            .collect(),
        Section::Sources => report.sources.iter().flatten().map(source_line).collect(),
        Section::Project => setting_lines(section, &report.project),
        Section::Consumer => setting_lines(section, &report.consumer),
        Section::Editor => setting_lines(section, &report.editor),
        Section::Checks => setting_lines(section, &report.checks),
        Section::Crawlers => setting_lines(section, &report.crawlers),
        Section::Listing => setting_lines(section, &report.listing),
        Section::Permalinks => setting_lines(section, &report.permalinks),
    }
}

/// A line for each key of a section of settings: its name from the
/// section's table, its value in effect, and its default.
fn setting_lines(section: Section, settings: &Option<Vec<ModelSetting>>) -> Vec<String> {
    let json = |v: &serde_json::Value| format!("`{v}`");
    settings
        .iter()
        .flatten()
        .map(|s| {
            let key = match s.table.strip_prefix(section.name()) {
                Some(rest) if !rest.is_empty() => format!("{}.{}", &rest[1..], s.key),
                _ => s.key.clone(),
            };
            let default = match (&s.default, &s.default_text) {
                (Some(d), _) => format!("default {}", json(d)),
                (None, Some(text)) => format!("default: {text}"),
                (None, None) => "no default".to_owned(),
            };
            match (&s.value, s.set) {
                (Some(v), true) => format!("`{key}`: {} ({default})", json(v)),
                (Some(v), false) => format!("`{key}`: {} (the default)", json(v)),
                (None, _) => format!("`{key}`: not set ({default})"),
            }
        })
        .collect()
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

/// A key of a section of settings, such as `[consumer]`, as the registry of
/// `ascribe.toml`'s keys has it, with the value in effect.
#[derive(Debug, Serialize)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
pub struct ModelSetting {
    /// The table it's in, as `ascribe.toml` writes it: `consumer`,
    /// `checks.vale`.
    pub table: String,
    /// The key: in `checks`, a check's slug.
    pub key: String,
    /// What its value is: `switch`, `choice`, `number`, `text`, `level`,
    /// `list`, or `table`.
    pub kind: String,
    /// The value in effect: the file's, else the default. Absent when the
    /// file leaves it out and its default isn't one value.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<serde_json::Value>,
    /// Whether `ascribe.toml` sets it.
    pub set: bool,
    /// The value used when the file leaves it out, when that's one value.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default: Option<serde_json::Value>,
    /// What's used when the file leaves it out, in the reference's words,
    /// when that isn't one value.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_text: Option<String>,
}

/// The keys of the section of settings `name`, from the registry, each with
/// its value in `file`, the parsed `ascribe.toml`.
fn settings(name: &str, file: Option<&toml::Table>) -> Vec<ModelSetting> {
    let mut out = Vec::new();
    let table = file
        .and_then(|f| f.get(name))
        .and_then(toml::Value::as_table);
    settings_of(name, name, table, &mut out);
    out
}

/// The keys of the registry's table `table`, which the file writes as
/// `path`, with their values in `values`: a table of keys of its own, such
/// as `[checks.vale]`, by each of its keys, and a part the project names,
/// such as a check in `[checks]`, for each one `values` has.
fn settings_of(table: &str, path: &str, values: Option<&toml::Table>, out: &mut Vec<ModelSetting>) {
    let all = registry::settings();
    let own: Vec<&Setting> = all.iter().filter(|s| s.table == table).collect();
    for s in &own {
        let value = values.and_then(|t| t.get(&s.key));
        let inner = format!("{table}.{}", s.key);
        if s.kind == "table" && all.iter().any(|t| t.table == inner) {
            let path = format!("{path}.{}", s.key);
            settings_of(&inner, &path, value.and_then(toml::Value::as_table), out);
        } else {
            out.push(setting(path, &s.key, s, value));
        }
    }
    // A part the project names is a table of the registry's, such as
    // `checks.<check>`, whose value can be its first key's alone.
    let named = all.iter().find(|s| {
        s.table
            .strip_prefix(table)
            .and_then(|rest| rest.strip_prefix(".<"))
            .is_some_and(|rest| !rest.contains('.'))
    });
    let (Some(named), Some(values)) = (named, values) else {
        return;
    };
    for (key, value) in values {
        if own.iter().any(|s| &s.key == key) {
            continue;
        }
        let mut entry = setting(path, key, named, Some(value));
        if value.is_table() {
            entry.kind = "table".to_owned();
        }
        out.push(entry);
    }
}

fn setting(table: &str, key: &str, s: &Setting, value: Option<&toml::Value>) -> ModelSetting {
    let default = s.default.as_ref().map(|d| match d {
        SettingValue::Switch(b) => serde_json::Value::Bool(*b),
        SettingValue::Number(n) => serde_json::Value::from(*n),
        SettingValue::Text(t) => serde_json::Value::String(t.clone()),
        SettingValue::List(items) => serde_json::Value::Array(
            items
                .iter()
                .map(|i| serde_json::Value::String(i.clone()))
                .collect(),
        ),
        SettingValue::EmptyTable => serde_json::Value::Object(serde_json::Map::new()),
    });
    let set = value.and_then(|v| serde_json::to_value(v).ok());
    ModelSetting {
        table: table.to_owned(),
        key: key.to_owned(),
        kind: s.kind.clone(),
        set: set.is_some(),
        value: set.or_else(|| default.clone()),
        default,
        default_text: s.default_text.clone().filter(|t| !t.is_empty()),
    }
}
