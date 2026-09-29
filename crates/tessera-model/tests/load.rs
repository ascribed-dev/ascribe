#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Loading the phase 01 examples, the filesystem rules, and the queries.

use std::path::{Path, PathBuf};

use tessera_core::availability::parse_availability;
use tessera_core::{Attributes, Binding, FileId, Forms, Primary, TitleRule};
use tessera_model::{
    AvailabilityMode, AvailabilityProblem, ContentModel, TrailingSlash, TypeMatch, VariantMode,
    load, load_str, load_str_in,
};

fn examples() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/content-models")
}

fn example(name: &str) -> ContentModel {
    load(examples().join(name)).unwrap_or_else(|issues| panic!("{name}: {issues:#?}"))
}

#[test]
fn all_three_examples_load_with_no_issues() {
    for name in ["minimal.toml", "quill.toml", "full.toml"] {
        let model = example(name);
        assert!(model.warnings.is_empty(), "{name}: {:#?}", model.warnings);
    }
}

#[test]
fn minimal_gets_every_default() {
    let m = example("minimal.toml");
    assert_eq!(m.project.content_root, "docs");
    assert_eq!(m.project.output_dir, ".ascribe/build");
    assert_eq!(m.types.len(), 1);
    assert_eq!(m.types[0].name, "page");
    assert!(m.types[0].default);
    assert_eq!(m.builds.len(), 1);
    assert_eq!(m.builds[0].name, "site");
    assert_eq!(m.editor_default_build().name, "site");
    assert_eq!(m.lifecycle.len(), 5);
    assert_eq!(m.notes.len(), 5);
    assert!(m.dimensions.is_empty() && m.widgets.is_empty() && m.phrases.is_empty());
    assert_eq!(m.consumer.profile, "astro");
    assert_eq!(m.consumer.base_path, "/");
    assert_eq!(m.consumer.trailing_slash, TrailingSlash::Always);
    assert!(m.consumer.html);
    // Only the built-in directives.
    assert_eq!(
        m.directive_keywords(),
        [
            "id",
            "include",
            "variant",
            "available",
            "note",
            "steps",
            "details"
        ]
    );
}

#[test]
fn quill_matches_the_spec_appendix_b_model() {
    let m = example("quill.toml");
    // Declaration order of dimensions is kept (content-model.md §1.1).
    let dims: Vec<_> = m.dimensions.iter().map(|d| d.name.as_str()).collect();
    assert_eq!(dims, ["pm", "deployment"]);
    let deployment = m.dimension("deployment").unwrap();
    assert_eq!(deployment.label, "Deployment");
    assert!(m.is_versionless("cloud") && !m.is_versionless("self-managed"));
    assert_eq!(m.value_label("cloud"), Some("Quill Cloud"));
    assert_eq!(m.value_label("self-managed"), Some("self-managed"));
    assert_eq!(m.value_label("yarn"), Some("Yarn"));
    assert_eq!(m.dimension_of_value("pnpm").unwrap().name, "pm");
    let values: Vec<_> = m
        .dimension_values("pm")
        .unwrap()
        .iter()
        .map(|v| v.value.as_str())
        .collect();
    assert_eq!(values, ["npm", "pnpm", "yarn"]);
    assert!(m.has_phrase("product") && !m.has_phrase("nope"));
    assert_eq!(m.phrase("version"), Some("3.4.1"));
    let builds: Vec<_> = m.builds.iter().map(|b| b.name.as_str()).collect();
    assert_eq!(builds, ["site", "cloud", "self-managed-3.3"]);
    assert_eq!(
        m.build("cloud").unwrap().variants,
        VariantMode::Select(vec![("deployment".into(), vec!["cloud".into()])])
    );
    let sm = m.build("self-managed-3.3").unwrap();
    let AvailabilityMode::Filter { target, version } = &sm.availability else {
        panic!("{sm:?}")
    };
    assert_eq!(target, "self-managed");
    assert_eq!(version.as_ref().unwrap().components, [3, 3]);
    assert_eq!(m.editor_default_build().name, "site");
    assert_eq!(m.consumer().site.as_deref(), Some("https://docs.quill.dev"));
}

#[test]
fn full_covers_lifecycle_features_notes_widgets_and_fields() {
    let m = example("full.toml");
    assert_eq!(m.state_is_available("sunset"), Some(false));
    assert_eq!(m.state_is_available("removed"), Some(false));
    assert_eq!(m.state_is_available("ga"), Some(true));
    assert_eq!(m.state_is_available("nope"), None);
    assert_eq!(m.lifecycle_state("beta").unwrap().label, "Beta");
    assert!(m.note_type("security").is_some());
    assert_eq!(
        m.note_type("tip").map(|n| n.label.as_str()),
        Some("Pro tip")
    );
    assert!(m.feature("streaming-sync").is_some() && m.feature("sso").is_some());
    let guide = m.types.iter().find(|t| t.name == "guide").unwrap();
    let title = guide
        .frontmatter
        .fields
        .iter()
        .find(|f| f.name == "title")
        .unwrap();
    assert!(title.required && title.phrases);
    // Which type applies.
    assert!(matches!(m.type_for("reference/api.md"), TypeMatch::One(t) if t.name == "reference"));
    assert!(matches!(m.type_for("guides/setup.md"), TypeMatch::One(t) if t.name == "guide"));
    assert!(
        m.is_fragment("includes/x.md")
            && m.is_fragment("a/_x.md")
            && m.is_fragment("a/b.partial.md")
    );
    assert!(!m.is_fragment("guides/setup.md"));
}

#[test]
fn widgets_become_directive_schemas() {
    let m = example("full.toml");
    assert!(!m.widgets.is_empty());
    let schemas = m.directive_schemas();
    assert_eq!(schemas.len(), 7 + m.widgets.len());
    for w in &m.widgets {
        let s = m.widget_schema(&w.schema.name).unwrap();
        assert!(matches!(s.origin, tessera_core::Origin::Widget));
        assert!(m.directive_keywords().contains(&s.name));
        // A line form has a binding; a container-only widget has none.
        assert_eq!(s.forms.line, s.binding.is_some());
        assert!(matches!(s.attributes, Attributes::Declared(_)));
    }
}

#[test]
fn a_widget_is_converted_exactly() {
    let toml = r#"
spec = "0.1"
[widgets.quill-labspace]
description = "An embedded lab."
forms = ["line", "container"]
primary = "text?"
binding = "heading-or-block"
title = "accepted"
plain-fallback = "Try it."
plain-content = "drop"
[widgets.quill-labspace.attributes]
lab = "string"
height = "number?"
mode = { type = "enum(a, b)", default = "a" }
platform = "set(enum(x, y))?"
"#;
    let m = load_str(toml, FileId::new(0)).unwrap();
    let w = m.widget("quill-labspace").unwrap();
    let s = &w.schema;
    assert_eq!(s.forms, Forms::BOTH);
    assert_eq!(s.primary, Primary::Text { required: false });
    assert_eq!(s.binding, Some(Binding::HeadingOrBlock));
    assert_eq!(s.title, TitleRule::Accepted);
    assert!(!s.groupable);
    assert_eq!(w.plain_fallback.as_deref(), Some("Try it."));
    assert_eq!(w.plain_content, tessera_model::PlainContent::Drop);
    let Attributes::Declared(attrs) = &s.attributes else {
        panic!()
    };
    let keys: Vec<_> = attrs.iter().map(|a| (a.key.as_str(), a.required)).collect();
    assert_eq!(
        keys,
        [
            ("lab", true),
            ("height", false),
            ("mode", false),
            ("platform", false)
        ]
    );
}

#[test]
fn attribute_declaration_order_is_kept() {
    let toml = "spec = \"0.1\"\n[images.attributes]\nzeta = \"string?\"\nalpha = \"number?\"\nmid = \"boolean?\"\n";
    let m = load_str(toml, FileId::new(0)).unwrap();
    let keys: Vec<_> = m.image_attributes.iter().map(|a| a.key.as_str()).collect();
    assert_eq!(keys, ["zeta", "alpha", "mid"]);
}

#[test]
fn dimension_declaration_order_is_kept() {
    let toml = "spec = \"0.1\"\n[dimensions.zed]\nvalues=[\"a\"]\n[dimensions.alpha]\nvalues=[\"b\"]\n[dimensions.mid]\nvalues=[\"c\"]\n";
    let m = load_str(toml, FileId::new(0)).unwrap();
    let names: Vec<_> = m.dimensions.iter().map(|d| d.name.as_str()).collect();
    assert_eq!(names, ["zed", "alpha", "mid"]);
    // Inline and dotted spellings are the same model.
    let dotted = "spec = \"0.1\"\ndimensions.zed.values = [\"a\"]\ndimensions.alpha = { values = [\"b\"] }\n";
    let m = load_str(dotted, FileId::new(0)).unwrap();
    assert_eq!(m.dimensions.len(), 2);
}

#[test]
fn the_editor_build_defaults() {
    let one = load_str("spec = \"0.1\"\n[builds.only]\n", FileId::new(0)).unwrap();
    assert_eq!(one.editor_default_build().name, "only");
    let site = load_str(
        "spec = \"0.1\"\n[builds.a]\n[builds.site]\n",
        FileId::new(0),
    )
    .unwrap();
    assert_eq!(site.editor_default_build().name, "site");
    let named = load_str(
        "spec = \"0.1\"\n[builds.a]\n[builds.b]\n[editor]\nbuild = \"b\"\n",
        FileId::new(0),
    )
    .unwrap();
    assert_eq!(named.editor_default_build().name, "b");
}

#[test]
fn warnings_do_not_stop_loading() {
    let toml = "spec = \"0.1\"\n[dimensions.a]\nvalues = [\"Cloud\", \"cloud\"]\n";
    let m = load_str(toml, FileId::new(0)).unwrap();
    assert_eq!(m.warnings.len(), 1);
    assert_eq!(m.warnings[0].slug.as_str(), "model-name-case");
}

#[test]
fn several_errors_are_all_reported_in_file_order() {
    let toml = "spec = \"0.1\"\n[project]\nbogus = 1\ncontent-root = \"/abs\"\n[phrases]\nv = 3\n";
    let issues = load_str(toml, FileId::new(0)).unwrap_err();
    let slugs: Vec<_> = issues.iter().map(|i| i.slug.as_str()).collect();
    assert_eq!(
        slugs,
        [
            "model-unknown-key",
            "model-path-absolute",
            "model-phrase-value-type"
        ]
    );
}

#[test]
fn availability_specs_are_checked_against_the_model() {
    let m = example("quill.toml");
    let problems = |text: &str| m.check_availability(&parse_availability(text, 0).unwrap());
    assert!(problems("cloud, self-managed preview 3.3").is_empty());
    assert!(problems("self-managed (preview 3.3, ga 3.5, deprecated 4.0)").is_empty());
    assert!(problems("deployment").is_empty());
    assert!(matches!(
        problems("mars")[..],
        [AvailabilityProblem::UnknownTarget(_)]
    ));
    assert!(matches!(
        problems("cloud bogus")[..],
        [AvailabilityProblem::UnknownState(_)]
    ));
    assert!(matches!(
        problems("cloud 3.4")[..],
        [AvailabilityProblem::VersionlessVersion { .. }]
    ));
    assert!(matches!(
        problems("self-managed (ga 3.5, preview 3.3)")[..],
        [AvailabilityProblem::HistoryOrder { .. }]
    ));
    // A bare feature key is a feature, not a target.
    let full = example("full.toml");
    let spec = parse_availability("streaming-sync", 0).unwrap();
    assert!(full.check_availability(&spec).is_empty());
    assert_eq!(
        spec.bare_name()
            .and_then(|n| full.feature(&n.text))
            .map(|f| f.name.as_str()),
        Some("Streaming sync")
    );
}

#[test]
fn a_version_on_a_dimension_name_is_an_error() {
    // SPEC §4.4 (resolved Q29): whether the dimension is mixed or all versioned.
    let toml = "spec = \"0.1\"\n[dimensions.deployment]\nvalues = [\"cloud\", \"self-managed\"]\nversionless = [\"cloud\"]\n[dimensions.sdk]\nvalues = [\"python\", \"js\"]\n";
    let m = load_str(toml, FileId::new(0)).unwrap();
    for bad in [
        "deployment 3.4",
        "sdk 3.4",
        "sdk preview 3.4",
        "sdk (preview 3.3, ga 3.5)",
    ] {
        let problems = m.check_availability(&parse_availability(bad, 0).unwrap());
        assert!(
            matches!(problems[0], AvailabilityProblem::DimensionVersion { .. }),
            "{bad}: {problems:?}"
        );
    }
    // The message suggests a versioned value of the dimension.
    let problems = m.check_availability(&parse_availability("deployment 3.4", 0).unwrap());
    assert!(
        matches!(&problems[..], [AvailabilityProblem::DimensionVersion { example, .. }] if example == "self-managed"),
        "{problems:?}"
    );
    for ok in ["deployment", "deployment beta", "sdk", "sdk deprecated"] {
        assert!(
            m.check_availability(&parse_availability(ok, 0).unwrap())
                .is_empty(),
            "{ok}"
        );
    }
}

// ---- the filesystem rules ---------------------------------------------------

fn scratch(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("tessera-model-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

#[test]
fn the_content_root_must_exist_and_be_a_directory() {
    let dir = scratch("root");
    let text = "spec = \"0.1\"\n[project]\ncontent-root = \"docs\"\n";
    let issues = load_str_in(text, FileId::new(0), &dir).unwrap_err();
    assert_eq!(issues[0].slug.as_str(), "model-content-root-missing");
    assert_eq!(issues[0].variant, None);
    // Loading text alone skips the rule.
    assert!(load_str(text, FileId::new(0)).is_ok());
    std::fs::write(dir.join("docs"), "").unwrap();
    let issues = load_str_in(text, FileId::new(0), &dir).unwrap_err();
    assert_eq!(issues[0].variant, Some("not-directory"));
    std::fs::remove_file(dir.join("docs")).unwrap();
    std::fs::create_dir(dir.join("docs")).unwrap();
    assert!(load_str_in(text, FileId::new(0), &dir).is_ok());
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn glossary_links_must_name_existing_pages() {
    let dir = scratch("glossary");
    std::fs::create_dir_all(dir.join("docs/reference")).unwrap();
    std::fs::write(
        dir.join("docs/reference/glossary.md"),
        "---\ntitle: G\n---\n",
    )
    .unwrap();
    let ok = "spec = \"0.1\"\n[glossary.terms.a]\nterm = \"A\"\ndefinition = \"x\"\nlink = \"/reference/glossary.md#a\"\n";
    let m = load_str_in(ok, FileId::new(0), &dir).unwrap();
    assert_eq!(
        m.glossary.terms[0].link,
        Some(("reference/glossary.md".into(), Some("a".into())))
    );
    let missing = ok.replace("glossary.md#a", "nope.md");
    let issues = load_str_in(&missing, FileId::new(0), &dir).unwrap_err();
    assert_eq!(issues[0].slug.as_str(), "model-glossary-link");
    assert_eq!(issues[0].variant, None);
    // Without the project directory the existence check is skipped.
    assert!(load_str(&missing, FileId::new(0)).is_ok());
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn the_output_directory_is_compared_after_resolving_links() {
    let dir = scratch("links");
    std::fs::create_dir_all(dir.join("docs")).unwrap();
    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(dir.join("docs"), dir.join("alias")).unwrap();
        let text = "spec = \"0.1\"\n[project]\noutput-dir = \"alias/out\"\n";
        let issues = load_str_in(text, FileId::new(0), &dir).unwrap_err();
        assert_eq!(issues[0].slug.as_str(), "model-output-overlaps-content");
    }
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn an_unreadable_file_is_reported_not_a_panic() {
    let issues = load("/no/such/dir/ascribe.toml").unwrap_err();
    assert_eq!(issues[0].slug.as_str(), "model-toml-syntax");
}
