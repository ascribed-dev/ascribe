//! The typed content model (content-model.md) and the queries other crates
//! ask of it.

use tessera_core::availability::{AvailabilitySpec, Detail, Entry, Name, Version};
use tessera_core::{AttributeSchema, DirectiveSchema, Issue, Span, builtin_schemas};

use crate::pattern::Pattern;
use crate::types::FrontmatterSchema;

/// The spec versions this crate implements (`spec`, content-model.md §3).
pub const SUPPORTED_SPECS: &[&str] = &["0.1"];

/// A loaded, validated `ascribe.toml`.
///
/// Every default from content-model.md §19 is applied: the implicit `page`
/// type and `site` build exist when their tables are absent, and the built-in
/// lifecycle states and note types are always present. Collections keep
/// declaration order.
#[derive(Clone, Debug)]
pub struct ContentModel {
    /// The `spec` version.
    pub spec: String,
    /// `[project]`.
    pub project: Project,
    /// Page content types (`[types.<name>]`), or the implicit `page` type.
    pub types: Vec<ContentType>,
    /// `[fragments]`.
    pub fragments: Fragments,
    /// `[dimensions.<name>]`, in declaration order. That order is the
    /// canonical order of `@variant` attributes and decides which dimension a
    /// tab group syncs on (content-model.md §1.1).
    pub dimensions: Vec<Dimension>,
    /// `[versions] scheme`.
    pub version_scheme: VersionScheme,
    /// Lifecycle states: the built-ins (possibly adjusted), then declared ones.
    pub lifecycle: Vec<LifecycleState>,
    /// `[features.<key>]`.
    pub features: Vec<Feature>,
    /// Note types: the built-ins (possibly relabeled), then declared ones.
    pub notes: Vec<NoteType>,
    /// `[phrases]`, in declaration order.
    pub phrases: Vec<Phrase>,
    /// `[glossary]`.
    pub glossary: Glossary,
    /// `[images.attributes]`, in declaration order.
    pub image_attributes: Vec<AttributeSchema>,
    /// `[widgets.<name>]`.
    pub widgets: Vec<Widget>,
    /// `[consumer]`.
    pub consumer: Consumer,
    /// `[builds.<name>]`, or the implicit `site` build.
    pub builds: Vec<Build>,
    /// The build the editor checks by default (`[editor] build`, §18).
    pub editor_build: String,
    /// Warnings found while loading (`model-name-case`,
    /// `model-build-filter-excluded`). A model with errors doesn't load, so
    /// these are the only issues a loaded model has.
    pub warnings: Vec<Issue>,
}

/// `[project]`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Project {
    /// The content root, relative to the project root, as written.
    pub content_root: String,
    /// The output directory, relative to the project root, as written.
    pub output_dir: String,
}

/// A page content type (§5.1).
#[derive(Clone, Debug, PartialEq)]
pub struct ContentType {
    /// The type's name.
    pub name: String,
    /// The pages it applies to.
    pub files: Vec<Pattern>,
    /// Whether it applies to pages no type's `files` match.
    pub default: bool,
    /// The frontmatter schema.
    pub frontmatter: FrontmatterSchema,
}

/// `[fragments]` (§5.3).
#[derive(Clone, Debug)]
pub struct Fragments {
    /// Additional fragment patterns.
    pub patterns: Vec<Pattern>,
    /// The fragment schema.
    pub frontmatter: FrontmatterSchema,
}

/// A dimension (§7).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Dimension {
    /// The dimension's name.
    pub name: String,
    /// Its display label (the name when not declared).
    pub label: String,
    /// Its values, in display order.
    pub values: Vec<DimensionValue>,
}

/// One value of a dimension.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DimensionValue {
    /// The value.
    pub value: String,
    /// Its display label (the value when not declared).
    pub label: String,
    /// Whether the value is versionless (SPEC §4.4).
    pub versionless: bool,
}

/// The version scheme (§8).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VersionScheme {
    /// Dotted numbers of any length, compared numerically with missing
    /// components as 0.
    Numeric,
}

/// A lifecycle state (§9).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LifecycleState {
    /// The state's name.
    pub name: String,
    /// Whether content in this state counts as available.
    pub available: bool,
    /// The display label.
    pub label: String,
    /// Whether the state is built in.
    pub builtin: bool,
}

/// A feature (§10).
#[derive(Clone, Debug)]
pub struct Feature {
    /// The feature key.
    pub key: String,
    /// The display name.
    pub name: String,
    /// The spec text, as written.
    pub available_text: String,
    /// The parsed spec. Spans are offsets into `ascribe.toml`.
    pub available: AvailabilitySpec,
}

/// A note type (§11).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NoteType {
    /// The type (the value of `@note`'s `type` attribute).
    pub name: String,
    /// The display label.
    pub label: String,
    /// Whether the type is built in.
    pub builtin: bool,
}

/// A phrase (§12).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Phrase {
    /// The phrase key.
    pub key: String,
    /// The literal replacement text.
    pub value: String,
}

/// Which glossary occurrences are linked (§13.1).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GlossaryMatch {
    /// The first occurrence of each term on each page.
    First,
    /// Every occurrence.
    Every,
}

/// `[glossary]` (§13).
#[derive(Clone, Debug)]
pub struct Glossary {
    /// Which occurrences are linked.
    pub match_mode: GlossaryMatch,
    /// The default for terms that don't set `case-sensitive`.
    pub case_sensitive: bool,
    /// The terms, in declaration order.
    pub terms: Vec<GlossaryTerm>,
}

/// A glossary term (§13.2).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GlossaryTerm {
    /// The term id.
    pub id: String,
    /// The term as it appears in prose.
    pub term: String,
    /// Other forms that count as occurrences.
    pub aliases: Vec<String>,
    /// A short plain-text definition.
    pub definition: String,
    /// The page (relative to the content root, no leading `/`) and optional id
    /// the term links to, as `(path, fragment)`.
    pub link: Option<(String, Option<String>)>,
    /// Whether matching is case-sensitive for this term (the term's setting,
    /// else the glossary's).
    pub case_sensitive: bool,
}

/// A project widget (§15).
#[derive(Clone, Debug)]
pub struct Widget {
    /// The directive schema, exactly as `tessera-core` defines it.
    pub schema: DirectiveSchema,
    /// The plain-markdown fallback text.
    pub plain_fallback: Option<String>,
    /// Whether the plain-markdown output keeps the widget's wrapped content.
    pub plain_content: PlainContent,
}

/// What the plain-markdown output does with a widget's wrapped content.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PlainContent {
    /// Keep it after the fallback.
    Keep,
    /// Drop it.
    Drop,
}

/// How trailing slashes appear in routes (§16).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TrailingSlash {
    /// `/guides/setup/`.
    Always,
    /// `/guides/setup`.
    Never,
}

/// `[consumer]` (§16).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Consumer {
    /// The profile; `"astro"`.
    pub profile: String,
    /// The published site's origin.
    pub site: Option<String>,
    /// The URL path every route starts with.
    pub base_path: String,
    /// Trailing slash policy.
    pub trailing_slash: TrailingSlash,
    /// The slugger name; `"github"`.
    pub slugger: String,
    /// Whether the consumer renders raw HTML.
    pub html: bool,
}

/// A build's variant mode (§17).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum VariantMode {
    /// Keep every arm and page.
    Switch,
    /// Keep only the selected values: dimension name to values.
    Select(Vec<(String, Vec<String>)>),
}

/// A build's availability mode (§17).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AvailabilityMode {
    /// Keep everything and annotate it.
    Badge,
    /// Remove content not available for the target at the version.
    Filter {
        /// A dimension value.
        target: String,
        /// The version; present exactly when the target is versioned.
        version: Option<Version>,
    },
}

/// A build (§17).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Build {
    /// The build name.
    pub name: String,
    /// The variant mode.
    pub variants: VariantMode,
    /// The availability mode.
    pub availability: AvailabilityMode,
}

/// Which content type applies to a page (§5.1).
#[derive(Clone, Debug, PartialEq)]
pub enum TypeMatch<'a> {
    /// Exactly one type applies (by `files`, or the default).
    One(&'a ContentType),
    /// Several types' `files` match: an error (`content-type-unresolved`).
    Ambiguous(Vec<&'a ContentType>),
    /// None match and there's no default: an error (`content-type-unresolved`).
    None,
}

/// A problem in an availability spec that needs the model to detect. The
/// caller turns it into a diagnostic: the loader into `model-availability-*`,
/// the document checks into `available-*`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AvailabilityProblem {
    /// A target isn't a declared dimension value or dimension name.
    UnknownTarget(Name),
    /// A state isn't a declared lifecycle state.
    UnknownState(Name),
    /// A versionless target was given a version.
    VersionlessVersion {
        /// The target.
        target: Name,
        /// The offending version.
        version: Version,
    },
    /// A dimension name was given a version (SPEC §4.4): its values don't
    /// share one version line.
    DimensionVersion {
        /// The dimension name.
        target: Name,
        /// The offending version.
        version: Version,
        /// A value of the dimension to name instead: its first versioned
        /// value, or its first value if all are versionless.
        example: String,
    },
    /// A history isn't in chronological order.
    HistoryOrder {
        /// The target.
        target: Name,
        /// The step written first, whose version is the greater: "state version".
        later: String,
        /// The step written after it, with the lesser version.
        earlier: String,
        /// The span of the step written after (the out-of-order one).
        span: Span,
    },
}

impl ContentModel {
    /// Whether `key` is a declared phrase.
    pub fn has_phrase(&self, key: &str) -> bool {
        self.phrases.iter().any(|p| p.key == key)
    }

    /// A phrase's replacement text.
    pub fn phrase(&self, key: &str) -> Option<&str> {
        self.phrases
            .iter()
            .find(|p| p.key == key)
            .map(|p| p.value.as_str())
    }

    /// A dimension by name.
    pub fn dimension(&self, name: &str) -> Option<&Dimension> {
        self.dimensions.iter().find(|d| d.name == name)
    }

    /// The dimension a value belongs to (a value belongs to only one).
    pub fn dimension_of_value(&self, value: &str) -> Option<&Dimension> {
        self.dimensions
            .iter()
            .find(|d| d.values.iter().any(|v| v.value == value))
    }

    /// A dimension's values with their labels, in display order.
    pub fn dimension_values(&self, name: &str) -> Option<&[DimensionValue]> {
        self.dimension(name).map(|d| d.values.as_slice())
    }

    /// The display label of a dimension value.
    pub fn value_label(&self, value: &str) -> Option<&str> {
        self.dimension_of_value(value).and_then(|d| {
            d.values
                .iter()
                .find(|v| v.value == value)
                .map(|v| v.label.as_str())
        })
    }

    /// Whether `value` is a declared, versionless dimension value.
    pub fn is_versionless(&self, value: &str) -> bool {
        self.dimensions
            .iter()
            .flat_map(|d| &d.values)
            .any(|v| v.value == value && v.versionless)
    }

    /// A lifecycle state by name.
    pub fn lifecycle_state(&self, name: &str) -> Option<&LifecycleState> {
        self.lifecycle.iter().find(|s| s.name == name)
    }

    /// Whether content in the lifecycle state counts as available. `None`
    /// when the state isn't declared.
    pub fn state_is_available(&self, name: &str) -> Option<bool> {
        self.lifecycle_state(name).map(|s| s.available)
    }

    /// A feature by key.
    pub fn feature(&self, key: &str) -> Option<&Feature> {
        self.features.iter().find(|f| f.key == key)
    }

    /// A note type by name.
    pub fn note_type(&self, name: &str) -> Option<&NoteType> {
        self.notes.iter().find(|n| n.name == name)
    }

    /// A widget's directive schema.
    pub fn widget_schema(&self, name: &str) -> Option<&DirectiveSchema> {
        self.widgets
            .iter()
            .map(|w| &w.schema)
            .find(|s| s.name == name)
    }

    /// A widget by name.
    pub fn widget(&self, name: &str) -> Option<&Widget> {
        self.widgets.iter().find(|w| w.schema.name == name)
    }

    /// The schemas of every directive: the built-ins (SPEC §4), then the
    /// project's widgets. This is what the parser takes as input.
    pub fn directive_schemas(&self) -> Vec<DirectiveSchema> {
        let mut all = builtin_schemas();
        all.extend(self.widgets.iter().map(|w| w.schema.clone()));
        all
    }

    /// The directive keyword set: built-in names, then widget names.
    pub fn directive_keywords(&self) -> Vec<String> {
        self.directive_schemas()
            .into_iter()
            .map(|s| s.name)
            .collect()
    }

    /// A build by name.
    pub fn build(&self, name: &str) -> Option<&Build> {
        self.builds.iter().find(|b| b.name == name)
    }

    /// The build whose page-level diagnostics the editor reports by default.
    pub fn editor_default_build(&self) -> &Build {
        // Loading guarantees the build exists and that there is at least one.
        self.build(&self.editor_build)
            .unwrap_or_else(|| &self.builds[0])
    }

    /// The consumer settings.
    pub fn consumer(&self) -> &Consumer {
        &self.consumer
    }

    /// Whether a file (path relative to the content root, `/`-separated) is a
    /// fragment: any segment begins with `_`, or a fragment pattern matches
    /// (SPEC §2.2, content-model.md §5.3).
    pub fn is_fragment(&self, path: &str) -> bool {
        path.split('/').any(|s| s.starts_with('_'))
            || self.fragments.patterns.iter().any(|p| p.matches(path))
    }

    /// Which content type applies to a page (content-model.md §5.1). Don't
    /// call it for fragments.
    pub fn type_for(&self, path: &str) -> TypeMatch<'_> {
        let matching: Vec<&ContentType> = self
            .types
            .iter()
            .filter(|t| t.files.iter().any(|p| p.matches(path)))
            .collect();
        match matching.len() {
            1 => TypeMatch::One(matching[0]),
            0 => match self.types.iter().find(|t| t.default) {
                Some(t) => TypeMatch::One(t),
                None => TypeMatch::None,
            },
            _ => TypeMatch::Ambiguous(matching),
        }
    }

    /// The names an availability spec may use as targets: dimension values
    /// and dimension names.
    pub fn is_target(&self, name: &str) -> bool {
        self.dimension(name).is_some() || self.dimension_of_value(name).is_some()
    }

    /// Checks an availability spec against the model: names, versionless
    /// targets, and history order. A spec that is one bare name declared as a
    /// feature key is a feature reference and has no problems here.
    pub fn check_availability(&self, spec: &AvailabilitySpec) -> Vec<AvailabilityProblem> {
        if spec
            .bare_name()
            .is_some_and(|n| self.feature(&n.text).is_some())
        {
            return Vec::new();
        }
        check_entries(&self.dimensions, &self.lifecycle, &spec.entries)
    }
}

/// Checks availability entries against the declared dimensions and states.
pub(crate) fn check_entries(
    dimensions: &[Dimension],
    lifecycle: &[LifecycleState],
    entries: &[Entry],
) -> Vec<AvailabilityProblem> {
    let mut problems = Vec::new();
    let state_known = |n: &Name| lifecycle.iter().any(|s| s.name == n.text);
    for entry in entries {
        let target = &entry.target;
        let versionless = target_is_versionless(dimensions, &target.text);
        if versionless.is_none() {
            problems.push(AvailabilityProblem::UnknownTarget(target.clone()));
        }
        // SPEC §4.4: a dimension name takes no version, whether or not its
        // values are versionless (resolved Q29).
        let dimension = dimensions.iter().find(|d| d.name == target.text);
        let check_version = |v: &Version, problems: &mut Vec<AvailabilityProblem>| {
            if let Some(d) = dimension {
                let example = d
                    .values
                    .iter()
                    .find(|value| !value.versionless)
                    .or(d.values.first())
                    .map(|value| value.value.clone())
                    .unwrap_or_default();
                problems.push(AvailabilityProblem::DimensionVersion {
                    target: target.clone(),
                    version: v.clone(),
                    example,
                });
            } else if versionless == Some(true) {
                problems.push(AvailabilityProblem::VersionlessVersion {
                    target: target.clone(),
                    version: v.clone(),
                });
            }
        };
        match &entry.detail {
            Detail::None => {}
            Detail::Version(v) => check_version(v, &mut problems),
            Detail::State { state, version } => {
                if !state_known(state) {
                    problems.push(AvailabilityProblem::UnknownState(state.clone()));
                }
                if let Some(v) = version {
                    check_version(v, &mut problems);
                }
            }
            Detail::History(steps) => {
                for step in steps {
                    if !state_known(&step.state) {
                        problems.push(AvailabilityProblem::UnknownState(step.state.clone()));
                    }
                    check_version(&step.version, &mut problems);
                }
                for pair in steps.windows(2) {
                    if pair[0].version.compare(&pair[1].version).is_gt() {
                        let show = |s: &tessera_core::availability::HistoryStep| {
                            format!("{} {}", s.state.text, s.version.text)
                        };
                        problems.push(AvailabilityProblem::HistoryOrder {
                            target: target.clone(),
                            later: show(&pair[0]),
                            earlier: show(&pair[1]),
                            span: pair[1].span,
                        });
                        break;
                    }
                }
            }
        }
    }
    problems
}

/// `None` when `name` isn't a target. `Some(true)` when it is a versionless
/// dimension value. A dimension name is `Some(false)`; versions on it are
/// rejected separately (Q29).
fn target_is_versionless(dimensions: &[Dimension], name: &str) -> Option<bool> {
    if dimensions.iter().any(|d| d.name == name) {
        return Some(false);
    }
    dimensions
        .iter()
        .flat_map(|d| &d.values)
        .find(|v| v.value == name)
        .map(|v| v.versionless)
}
