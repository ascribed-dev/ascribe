#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Loading the example content models (`examples/content-models`), the
//! file-system rules, and the queries.

use std::path::{Path, PathBuf};

#[path = "../../../tests/support/links.rs"]
mod links;

use ascribe_core::availability::parse_availability;
use ascribe_core::{Attributes, Binding, FileId, Forms, Primary, TitleRule};
use ascribe_model::{
    AvailabilityMode, AvailabilityProblem, ContentModel, GlossaryMatch, HeadersFormat, Listing,
    ListingFrom, ListingLevel, SourceBound, TrailingSlash, TypeMatch, VariantMode, load, load_str,
    load_str_bounded, load_str_in,
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
    assert_eq!(m.consumer.permalink_path(), "/go/");
    assert!(m.consumer.html);
    assert!(m.current_version.is_none());
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
            "details",
            "snippet",
            "intended"
        ]
    );
}

#[test]
fn permalinks_are_under_the_base_path_unless_a_path_is_set() {
    let path = |consumer: &str| {
        load_str(
            &format!("spec = \"0.1\"\n[consumer]\n{consumer}"),
            FileId::new(0),
        )
        .unwrap()
        .consumer
        .permalink_path()
    };
    assert_eq!(path("base-path = \"/docs/\"\n"), "/docs/go/");
    assert_eq!(path("base-path = \"/docs\"\n"), "/docs/go/");
    assert_eq!(
        path("base-path = \"/docs/\"\npermalink-path = \"/go/\"\n"),
        "/go/"
    );
    assert_eq!(path("permalink-path = \"/links\"\n"), "/links/");
}

#[test]
fn headers_name_the_hosts_format_and_nothing_else() {
    let load = |consumer: &str| {
        load_str(
            &format!("spec = \"0.1\"\n[consumer]\n{consumer}"),
            FileId::new(0),
        )
    };
    assert_eq!(load("").unwrap().consumer.headers, None);
    assert_eq!(
        load("headers = \"netlify\"\n").unwrap().consumer.headers,
        Some(HeadersFormat::Netlify)
    );
    let issues = load("headers = \"apache\"\n").unwrap_err();
    assert_eq!(issues.len(), 1, "{issues:#?}");
    assert_eq!(issues[0].slug.as_str(), "model-invalid-value");
}

#[test]
fn quill_matches_the_spec_appendix_b_model() {
    let m = example("quill.toml");
    // Declaration order of dimensions is kept.
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
    assert_eq!(schemas.len(), 9 + m.widgets.len());
    for w in &m.widgets {
        let s = m.widget_schema(&w.schema.name).unwrap();
        assert!(matches!(s.origin, ascribe_core::Origin::Widget));
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
    assert_eq!(w.plain_content, ascribe_model::PlainContent::Drop);
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
    // SPEC §4.4: whether the dimension is mixed or all versioned.
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

// ---- listing ----------------------------------------------------------------

const LISTING: &str = "spec = \"0.1\"\n\
    [types.guide]\ndefault = true\n[types.guide.frontmatter]\ntitle = \"string\"\n\
    [types.changelog]\nfiles = [\"changes/**\"]\n[types.changelog.frontmatter]\ntitle = \"string\"\n";

fn listing_model(rules: &str) -> ContentModel {
    load_str(&format!("{LISTING}{rules}"), FileId::new(0)).unwrap()
}

#[test]
fn a_page_is_listed_with_no_rules() {
    let m = listing_model("");
    assert!(m.listing.is_empty());
    assert_eq!(m.listing_rule("a.md"), None);
    assert_eq!(
        m.listing_of("a.md", None),
        (Listing::Listed, ListingFrom::Default)
    );
    assert_eq!(
        m.listing_of("a.md", Some(Listing::Unquoted)),
        (Listing::Unquoted, ListingFrom::Page)
    );
}

#[test]
fn the_strictest_rule_a_page_is_under_wins() {
    let m = listing_model(
        "[listing]\ndefault = \"listed\"\n\
         [listing.folders]\n\"internal/\" = \"unquoted\"\n\"internal/old/\" = \"unlisted\"\n\
         \"changes/\" = \"unlisted\"\n\
         [listing.types]\nchangelog = \"unquoted\"\n",
    );
    let level = |path: &str| m.listing_rule(path).map(|r| r.level.clone());
    assert_eq!(level("a.md"), Some(ListingLevel::Site));
    assert_eq!(
        level("internal/a.md"),
        Some(ListingLevel::Folder("internal/".into()))
    );
    assert_eq!(
        level("internal/old/a.md"),
        Some(ListingLevel::Folder("internal/old/".into()))
    );
    // Not a folder's page: the folder's name is a prefix of it only.
    assert_eq!(level("internal-notes.md"), Some(ListingLevel::Site));
    // A folder and a type: the stricter, and the folder of two as strict.
    assert_eq!(
        level("changes/a.md"),
        Some(ListingLevel::Folder("changes/".into()))
    );
    let rule = m.listing_rule("internal/a.md").unwrap();
    assert_eq!(rule.text, "\"internal/\" = \"unquoted\"");
    assert_eq!(rule.line, 13);
}

#[test]
fn a_page_may_tighten_its_rule_and_never_loosen_it() {
    let m = listing_model("[listing.folders]\n\"internal/\" = \"unquoted\"\n");
    let rule = ListingFrom::Rule(m.listing_rule("internal/a.md").unwrap().clone());
    assert_eq!(
        m.listing_of("internal/a.md", None),
        (Listing::Unquoted, rule.clone())
    );
    assert_eq!(
        m.listing_of("internal/a.md", Some(Listing::Unlisted)),
        (Listing::Unlisted, ListingFrom::Page)
    );
    assert_eq!(
        m.listing_of("internal/a.md", Some(Listing::Unquoted)),
        (Listing::Unquoted, ListingFrom::Page)
    );
    assert_eq!(
        m.listing_of("internal/a.md", Some(Listing::Listed)),
        (Listing::Unquoted, rule)
    );
}

#[test]
fn a_rule_as_strict_as_the_one_it_is_under_is_fine() {
    let m = listing_model(
        "[listing]\ndefault = \"unquoted\"\n[listing.folders]\n\"a/\" = \"unquoted\"\n\
         \"a/b/\" = \"unlisted\"\n[listing.types]\nchangelog = \"unlisted\"\n",
    );
    assert_eq!(m.listing.folders.len(), 2);
}

// ---- the filesystem rules ---------------------------------------------------

#[test]
fn a_listing_folder_must_exist_by_its_exact_name() {
    let dir = scratch("listing");
    std::fs::create_dir_all(dir.join("docs/internal/old")).unwrap();
    std::fs::write(dir.join("docs/page.md"), "").unwrap();
    let model = |folder: &str| {
        format!(
            "spec = \"0.1\"\n[project]\ncontent-root = \"docs\"\n\
             [listing.folders]\n\"{folder}\" = \"unlisted\"\n"
        )
    };
    let slug = |folder: &str| {
        load_str_in(&model(folder), FileId::new(0), &dir)
            .err()
            .map(|issues| (issues[0].slug.as_str().to_owned(), issues[0].variant))
    };
    assert_eq!(slug("internal/"), None);
    assert_eq!(slug("internal/old/"), None);
    assert_eq!(slug("drafts/"), Some(("model-listing-folder".into(), None)));
    assert_eq!(
        slug("Internal/"),
        Some(("model-listing-folder".into(), None))
    );
    assert_eq!(
        slug("page.md/"),
        Some(("model-listing-folder".into(), Some("not-directory")))
    );
    // Loading text alone skips the rule.
    assert!(load_str(&model("drafts/"), FileId::new(0)).is_ok());
}

fn scratch(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("ascribe-model-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

#[test]
fn a_source_folder_must_exist_be_a_directory_and_be_in_the_repository() {
    let outer = scratch("sources");
    let dir = outer.join("repo");
    std::fs::create_dir_all(dir.join(".git")).unwrap();
    std::fs::create_dir_all(dir.join("docs")).unwrap();
    let model = |path: &str| {
        format!(
            "spec = \"0.1\"\n[project]\ncontent-root = \"docs\"\n[sources.code]\npath = \"{path}\"\n"
        )
    };
    let slug = |path: &str| {
        let issues = load_str_in(&model(path), FileId::new(0), &dir).unwrap_err();
        (issues[0].slug.as_str().to_owned(), issues[0].variant)
    };
    assert_eq!(slug("code"), ("model-source-path-missing".into(), None));
    // Loading text alone skips the rule.
    assert!(load_str(&model("code"), FileId::new(0)).is_ok());
    std::fs::write(dir.join("code"), "").unwrap();
    assert_eq!(
        slug("code"),
        ("model-source-path-missing".into(), Some("not-directory"))
    );
    std::fs::remove_file(dir.join("code")).unwrap();
    std::fs::create_dir(dir.join("code")).unwrap();
    let loaded = load_str_in(&model("code"), FileId::new(0), &dir).unwrap();
    assert_eq!(loaded.source("code").map(|s| s.path.as_str()), Some("code"));
    // A folder next to the repository, not in it.
    std::fs::create_dir(outer.join("elsewhere")).unwrap();
    assert_eq!(
        slug("../elsewhere"),
        ("model-source-outside-repository".into(), None)
    );
    let _ = std::fs::remove_dir_all(&outer);
}

#[test]
fn a_source_in_another_repository_reads_its_copies() {
    let dir = scratch("remote");
    std::fs::create_dir_all(dir.join(".git")).unwrap();
    std::fs::create_dir_all(dir.join("docs")).unwrap();
    let text = "spec = \"0.1\"\n[sources.api]\ngit = \"https://github.com/acme/api.git\"\nbranch = \"release/2\"\ninclude = [\"src/**\"]\n[sources.cli]\ngit = \"git@github.com:acme/cli.git\"\n";
    // The copies' folder needn't exist yet.
    let model = load_str_in(text, FileId::new(0), &dir).unwrap();
    let api = model.source("api").unwrap();
    assert_eq!(api.path, "sources/api");
    let remote = api.git.as_ref().unwrap();
    assert_eq!(remote.url, "https://github.com/acme/api.git");
    assert_eq!(remote.branch.as_deref(), Some("release/2"));
    assert!(api.reads("src/auth.rs") && !api.reads("README.md"));
    let cli = model.source("cli").unwrap();
    assert_eq!(cli.git.as_ref().unwrap().branch, None);
    let _ = std::fs::remove_dir_all(&dir);

    let url = |url: &str| {
        let text = format!("spec = \"0.1\"\n[sources.api]\ngit = \"{url}\"\n");
        load_str(&text, FileId::new(0)).is_ok()
    };
    for good in [
        "https://example.com/a.git",
        "http://example.com/a",
        "ssh://git@example.com:22/a.git",
        "git://example.com/a.git",
        "file:///srv/git/a.git",
        "git@github.com:acme/a.git",
    ] {
        assert!(url(good), "{good}");
    }
    for bad in [
        "-uhello",
        "ext::sh -c touch% /tmp/x",
        "/srv/git/a.git",
        "../a.git",
        "C:/repos/a",
        "https://",
        "example.com/a b",
        "a/b@host:path",
    ] {
        assert!(!url(bad), "{bad}");
    }
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
fn a_glossary_term_can_set_its_own_match() {
    let text = "spec = \"0.1\"\n[glossary]\nmatch = \"every\"\n[glossary.terms.a]\nterm = \"A\"\ndefinition = \"x\"\nmatch = \"marked\"\n[glossary.terms.b]\nterm = \"B\"\ndefinition = \"x\"\n";
    let m = load_str(text, FileId::new(0)).unwrap();
    let modes: Vec<_> = m.glossary.terms.iter().map(|t| t.match_mode).collect();
    assert_eq!(modes, [GlossaryMatch::Marked, GlossaryMatch::Every]);
    // `marked` is a term setting, not a glossary default.
    let all = text.replace("match = \"every\"", "match = \"marked\"");
    let issues = load_str(&all, FileId::new(0)).unwrap_err();
    assert_eq!(issues[0].slug.as_str(), "model-invalid-value");
}

#[test]
fn the_output_directory_is_compared_after_resolving_links() {
    let dir = scratch("links");
    std::fs::create_dir_all(dir.join("docs")).unwrap();
    links::dir(dir.join("docs"), dir.join("alias"));
    let text = "spec = \"0.1\"\n[project]\noutput-dir = \"alias/out\"\n";
    let issues = load_str_in(text, FileId::new(0), &dir).unwrap_err();
    assert_eq!(issues[0].slug.as_str(), "model-output-overlaps-content");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn an_unreadable_file_is_reported_not_a_panic() {
    let issues = load("/no/such/dir/ascribe.toml").unwrap_err();
    assert_eq!(issues[0].slug.as_str(), "model-toml-syntax");
}

#[test]
fn full_marks_its_description_and_review_date() {
    use ascribe_model::{CheckLevel, FieldRole};
    let m = example("full.toml");
    let guide = &m.types[0].frontmatter;
    let name = |role| guide.field_with_role(role).map(|f| f.name.as_str());
    assert_eq!(name(FieldRole::Description), Some("description"));
    assert_eq!(name(FieldRole::ReviewDate), Some("review-by"));
    assert_eq!(
        m.types[1]
            .frontmatter
            .field_with_role(FieldRole::Description),
        None
    );
    let page_size = ascribe_core::diagnostics::PAGE_SIZE;
    assert_eq!(m.checks.limit(page_size), Some(40_000));
    assert_eq!(
        m.checks
            .level(ascribe_core::diagnostics::CODE_LANGUAGE_MISSING),
        Some(CheckLevel::Warning)
    );
}

#[test]
fn versions_current_is_a_version_in_the_file() {
    let toml = "spec = \"0.1\"\n[versions]\ncurrent = \"3.10\"\n";
    let m = load_str(toml, FileId::new(0)).unwrap();
    let current = m.current_version.unwrap();
    assert_eq!(current.components, [3, 10]);
    assert_eq!(&toml[current.span.range()], "3.10");
}

/// With no repository around the project, a source is bounded by what was
/// opened when the loader is given it (the language server, `ascribe mcp`),
/// and is anywhere the model says when it isn't (the commands): decision
/// 77.
#[test]
fn with_no_repository_a_source_is_bounded_by_what_was_opened() {
    let outer = scratch("opened");
    let project = outer.join("archive").join("docs");
    std::fs::create_dir_all(project.join("content")).unwrap();
    std::fs::create_dir_all(outer.join("archive").join("code")).unwrap();
    let text = |path: &str| {
        format!(
            "spec = \"0.1\"\n[project]\ncontent-root = \"content\"\n[sources.code]\npath = \"{path}\"\n"
        )
    };
    let load = |path: &str, opened: &[PathBuf]| {
        load_str_bounded(
            &text(path),
            FileId::new(0),
            &project,
            SourceBound::Within(opened),
        )
    };
    // The commands: anywhere, as before.
    assert!(load_str_in(&text("../../.."), FileId::new(0), &project).is_ok());
    assert!(
        load_str_bounded(
            &text("../../.."),
            FileId::new(0),
            &project,
            SourceBound::Anywhere
        )
        .is_ok()
    );
    // Opened on the project's folder: a source above it is refused.
    let issues = load("../code", std::slice::from_ref(&project)).unwrap_err();
    assert_eq!(
        (issues[0].slug.as_str(), issues[0].variant),
        ("model-source-outside-repository", Some("opened"))
    );
    assert!(load("../../..", std::slice::from_ref(&project)).is_err());
    // Opened on the folder that contains it: read.
    assert!(load("../code", &[outer.join("archive")]).is_ok());
    // Any of several folders will do.
    assert!(load("../code", &[outer.join("elsewhere"), outer.join("archive")]).is_ok());
    // With a repository, the repository is the rule, whatever was opened.
    std::fs::create_dir(outer.join("archive").join(".git")).unwrap();
    assert!(load("../code", std::slice::from_ref(&project)).is_ok());
    let _ = std::fs::remove_dir_all(&outer);
}
