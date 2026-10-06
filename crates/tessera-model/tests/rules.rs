#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! One failing fixture per loading rule (each `model-` diagnostic), checking
//! the slug, the message variant, and the line the issue points at.
//!
//! In a fixture, `#!` at the end of a line marks the line the issue must
//! point at. A fixture without one checks only the slug.

use std::collections::BTreeSet;
use std::path::Path;

use tessera_core::{FileId, Issue, LineIndex};
use tessera_model::load_str;

struct Case {
    slug: &'static str,
    variant: Option<&'static str>,
    toml: &'static str,
}

const fn case(slug: &'static str, variant: Option<&'static str>, toml: &'static str) -> Case {
    Case {
        slug,
        variant,
        toml,
    }
}

fn issues_of(text: &str) -> Vec<Issue> {
    match load_str(text, FileId::new(0)) {
        Ok(m) => m.warnings,
        Err(issues) => issues,
    }
}

fn line_of(text: &str, offset: usize) -> usize {
    let index = LineIndex::new(text);
    index.line_col(offset).map_or(0, |lc| lc.line as usize) + 1
}

fn cases() -> Vec<Case> {
    vec![
        // 21.1 File and structure
        case("model-toml-syntax", None, "spec = \"0.1\"\nx = = 1 #!\n"),
        case(
            "model-unknown-key",
            None,
            "spec = \"0.1\"\n[project]\nbogus = 1 #!\n",
        ),
        case(
            "model-unknown-key",
            Some("suggestion"),
            "spec = \"0.1\"\n[project]\ncontent-rot = \"docs\" #!\n",
        ),
        case("model-unknown-key", None, "spec = \"0.1\"\nbogus = 1 #!\n"),
        case("model-missing-key", None, "[project]\n"),
        case(
            "model-missing-key",
            None,
            "spec = \"0.1\"\n[features.x] #!\nname = \"X\"\n",
        ),
        case(
            "model-wrong-type",
            None,
            "spec = \"0.1\"\n[project]\ncontent-root = true #!\n",
        ),
        case("model-wrong-type", Some("quote"), "spec = 0.1 #!\n"),
        case(
            "model-invalid-value",
            None,
            "spec = \"0.1\"\n[versions]\nscheme = \"semver\" #!\n",
        ),
        case("model-spec-unsupported", None, "spec = \"9.9\" #!\n"),
        case(
            "model-invalid-name",
            None,
            "spec = \"0.1\"\n[dimensions.Deployment] #!\nvalues = [\"a\"]\n",
        ),
        case(
            "model-empty-text",
            None,
            "spec = \"0.1\"\n[notes.tip]\nlabel = \"\" #!\n",
        ),
        // 21.2 Project
        case(
            "model-path-absolute",
            None,
            "spec = \"0.1\"\n[project]\ncontent-root = \"/docs\" #!\n",
        ),
        case(
            "model-output-overlaps-content",
            None,
            "spec = \"0.1\"\n[project]\ncontent-root = \"docs\"\noutput-dir = \"docs/out\" #!\n",
        ),
        case(
            "model-output-overlaps-content",
            Some("content-inside-output"),
            "spec = \"0.1\"\n[project]\ncontent-root = \"out/docs\" #!\noutput-dir = \"out\"\n",
        ),
        case(
            "model-output-overlaps-content",
            Some("same"),
            "spec = \"0.1\"\n[project]\ncontent-root = \"x\"\noutput-dir = \"./x\" #!\n",
        ),
        // 21.8 Sources
        case(
            "model-source-remote",
            None,
            "spec = \"0.1\"\n[sources.code]\npath = \"code\"\ngit = \"https://example.com/x.git\" #!\n",
        ),
        case(
            "model-path-absolute",
            None,
            "spec = \"0.1\"\n[sources.code]\npath = \"/code\" #!\n",
        ),
        case(
            "model-source-remote",
            Some("neither"),
            "spec = \"0.1\"\n[sources.code] #!\ninclude = [\"*.py\"]\n",
        ),
        case(
            "model-source-remote",
            Some("branch"),
            "spec = \"0.1\"\n[sources.code]\npath = \"code\"\nbranch = \"main\" #!\n",
        ),
        case(
            "model-source-remote",
            Some("url"),
            "spec = \"0.1\"\n[sources.code]\ngit = \"--upload-pack=touch x\" #!\n",
        ),
        case(
            "model-source-remote",
            Some("branch-name"),
            "spec = \"0.1\"\n[sources.code]\ngit = \"https://example.com/x.git\"\nbranch = \"a..b\" #!\n",
        ),
        case(
            "model-source-remote",
            Some("inside-content"),
            "spec = \"0.1\"\n[project]\ncontent-root = \".\"\n[sources.code] #!\ngit = \"https://example.com/x.git\"\n",
        ),
        // 21.3 Content types, fragments, and fields
        case(
            "model-type-multiple-defaults",
            None,
            "spec = \"0.1\"\n[types.a]\ndefault = true\n[types.a.frontmatter]\ntitle = \"string\"\n[types.b]\ndefault = true #!\n[types.b.frontmatter]\ntitle = \"string\"\n",
        ),
        case(
            "model-type-unreachable",
            None,
            "spec = \"0.1\"\n[types.a.frontmatter] #!\ntitle = \"string\"\n[types.b]\nfiles = [\"b/**\"]\n[types.b.frontmatter]\ntitle = \"string\"\n",
        ),
        case(
            "model-type-title",
            None,
            "spec = \"0.1\"\n[types.a]\ndefault = true\n[types.a.frontmatter] #!\nother = \"string\"\n",
        ),
        case(
            "model-type-title",
            Some("not-string"),
            "spec = \"0.1\"\n[types.a]\ndefault = true\n[types.a.frontmatter]\ntitle = \"string?\" #!\n",
        ),
        case(
            "model-field-reserved",
            None,
            "spec = \"0.1\"\n[types.a]\ndefault = true\n[types.a.frontmatter]\ntitle = \"string\"\navailable = \"string?\" #!\n",
        ),
        case(
            "model-field-reserved",
            Some("slug"),
            "spec = \"0.1\"\n[types.a]\ndefault = true\n[types.a.frontmatter]\ntitle = \"string\"\nslug = \"string?\" #!\n",
        ),
        case(
            "model-field-reserved",
            Some("formatted"),
            "spec = \"0.1\"\n[types.a]\ndefault = true\n[types.a.frontmatter]\ntitle = \"string\"\nformatted = \"string?\" #!\n",
        ),
        case(
            "model-field-reserved",
            Some("fragment"),
            "spec = \"0.1\"\n[fragments.frontmatter]\nvariant = \"string?\" #!\n",
        ),
        case(
            "model-type-syntax",
            None,
            "spec = \"0.1\"\n[types.a]\ndefault = true\n[types.a.frontmatter]\ntitle = \"string\"\nx = \"strng?\" #!\n",
        ),
        case(
            "model-type-syntax",
            None,
            "spec = \"0.1\"\n[images.attributes]\nwhen = \"date\" #!\n",
        ),
        case(
            "model-type-syntax",
            None,
            "spec = \"0.1\"\n[fragments.frontmatter]\nowner = \"object\" #!\n",
        ),
        case(
            "model-type-fields",
            None,
            "spec = \"0.1\"\n[fragments.frontmatter]\nowner = { type = \"object\" } #!\n",
        ),
        case(
            "model-type-fields",
            Some("not-object"),
            "spec = \"0.1\"\n[fragments.frontmatter]\nowner = { type = \"string\", fields = { a = \"string\" } } #!\n",
        ),
        case(
            "model-enum-values",
            None,
            "spec = \"0.1\"\n[fragments.frontmatter]\nlevel = \"enum()\" #!\n",
        ),
        case(
            "model-enum-values",
            Some("duplicate"),
            "spec = \"0.1\"\n[fragments.frontmatter]\nlevel = \"enum(a, a)\" #!\n",
        ),
        case(
            "model-enum-values",
            Some("both"),
            "spec = \"0.1\"\n[fragments.frontmatter]\nlevel = { type = \"enum(a)\", values = [\"b\"] } #!\n",
        ),
        case(
            "model-enum-values",
            Some("bare"),
            "spec = \"0.1\"\n[fragments.frontmatter]\nlevel = { type = \"enum\" } #!\n",
        ),
        case(
            "model-enum-values",
            Some("duplicate"),
            "spec = \"0.1\"\n[fragments.frontmatter]\nlevel = { type = \"enum\", values = [\"a\", \"a\"] } #!\n",
        ),
        case(
            "model-set-token",
            None,
            "spec = \"0.1\"\n[images.attributes]\nplatform = { type = \"set(enum)\", values = [\"a b\"] } #!\n",
        ),
        case(
            "model-default-type",
            None,
            "spec = \"0.1\"\n[fragments.frontmatter]\ncount = { type = \"number\", default = \"three\" } #!\n",
        ),
        case(
            "model-default-type",
            Some("not-a-value"),
            "spec = \"0.1\"\n[fragments.frontmatter]\nlevel = { type = \"enum(a, b)\", default = \"c\" } #!\n",
        ),
        case(
            "model-default-type",
            Some("not-a-value"),
            "spec = \"0.1\"\n[images.attributes]\nloading = { type = \"enum(lazy, eager)\", default = \"never\" } #!\n",
        ),
        case(
            "model-phrases-field-type",
            None,
            "spec = \"0.1\"\n[fragments.frontmatter]\ncount = { type = \"number\", phrases = true } #!\n",
        ),
        case(
            "model-inline-field",
            None,
            "spec = \"0.1\"\n[types.a]\ndefault = true\n[types.a.frontmatter]\ntitle = \"string\"\ntags = { type = \"list(string)\", inline = \"code\" } #!\n",
        ),
        case(
            "model-inline-field",
            Some("value"),
            "spec = \"0.1\"\n[types.a]\ndefault = true\n[types.a.frontmatter]\ntitle = { type = \"string\", inline = \"emphasis\" } #!\n",
        ),
        case(
            "model-inline-field",
            Some("nested"),
            "spec = \"0.1\"\n[fragments.frontmatter]\nlabel = { type = \"string\", inline = \"code\" } #!\n",
        ),
        case(
            "model-inline-field",
            Some("nested"),
            "spec = \"0.1\"\n[types.a]\ndefault = true\n[types.a.frontmatter]\ntitle = \"string\"\nmeta = { type = \"object\", fields = { label = { type = \"string\", inline = \"code\" } } } #!\n",
        ),
        case(
            "model-pattern-syntax",
            None,
            "spec = \"0.1\"\n[fragments]\npatterns = [\"a/**b\"] #!\n",
        ),
        case(
            "model-pattern-syntax",
            Some("leading-slash"),
            "spec = \"0.1\"\n[fragments]\npatterns = [\"/includes/**\"] #!\n",
        ),
        case(
            "model-pattern-syntax",
            Some("parent"),
            "spec = \"0.1\"\n[fragments]\npatterns = [\"../x/**\"] #!\n",
        ),
        case(
            "model-attribute-reserved",
            None,
            "spec = \"0.1\"\n[images.attributes]\nsrc = \"string?\" #!\n",
        ),
        case(
            "model-attribute-reserved",
            None,
            "spec = \"0.1\"\n[images.attributes]\nonclick = \"string?\" #!\n",
        ),
        case(
            "model-attribute-reserved",
            Some("widget"),
            "spec = \"0.1\"\n[widgets.quill-lab]\nforms = [\"line\"]\nbinding = \"self\"\n[widgets.quill-lab.attributes]\nheading = \"string?\" #!\n",
        ),
        // 21.4 Dimensions, names, lifecycle, notes, and features
        case(
            "model-name-multiple-roles",
            None,
            "spec = \"0.1\"\n[dimensions.channel]\nvalues = [\"beta\"] #!\n",
        ),
        case(
            "model-name-multiple-roles",
            None,
            "spec = \"0.1\"\n[dimensions.deployment]\nvalues = [\"cloud\"]\n[features.cloud] #!\nname = \"Cloud\"\navailable = \"deployment\"\n",
        ),
        case(
            "model-name-multiple-roles",
            None,
            "spec = \"0.1\"\n[lifecycle.sunset]\navailable = false\n[dimensions.sunset] #!\nvalues = [\"a\"]\n",
        ),
        case(
            "model-name-case",
            None,
            "spec = \"0.1\"\n[dimensions.a]\nvalues = [\"Cloud\", \"cloud\"] #!\n",
        ),
        case(
            "model-dimension-empty",
            None,
            "spec = \"0.1\"\n[dimensions.a]\nvalues = [] #!\n",
        ),
        case(
            "model-dimension-value-duplicate",
            None,
            "spec = \"0.1\"\n[dimensions.a]\nvalues = [\"x\", \"x\"] #!\n",
        ),
        case(
            "model-dimension-value-shared",
            None,
            "spec = \"0.1\"\n[dimensions.a]\nvalues = [\"x\"]\n[dimensions.b]\nvalues = [\"x\"] #!\n",
        ),
        case(
            "model-label-undeclared",
            None,
            "spec = \"0.1\"\n[dimensions.a]\nvalues = [\"x\"]\nlabels = { y = \"Why\" } #!\n",
        ),
        case(
            "model-versionless-undeclared",
            None,
            "spec = \"0.1\"\n[dimensions.a]\nvalues = [\"x\"]\nversionless = [\"y\"] #!\n",
        ),
        case(
            "model-lifecycle-available-required",
            None,
            "spec = \"0.1\"\n[lifecycle.sunset] #!\nlabel = \"sunset\"\n",
        ),
        case(
            "model-lifecycle-ga-unavailable",
            None,
            "spec = \"0.1\"\n[lifecycle.ga]\navailable = false #!\n",
        ),
        case(
            "model-note-label-required",
            None,
            "spec = \"0.1\"\n[notes.security] #!\n",
        ),
        case(
            "model-availability-syntax",
            None,
            "spec = \"0.1\"\n[features.x]\nname = \"X\"\navailable = \"cloud,\" #!\n",
        ),
        case(
            "model-availability-unknown-name",
            None,
            "spec = \"0.1\"\n[features.x]\nname = \"X\"\navailable = \"mars\" #!\n",
        ),
        case(
            "model-availability-unknown-name",
            Some("state"),
            "spec = \"0.1\"\n[dimensions.d]\nvalues = [\"a\"]\n[features.x]\nname = \"X\"\navailable = \"a bogus 1.0\" #!\n",
        ),
        case(
            "model-availability-versionless",
            None,
            "spec = \"0.1\"\n[dimensions.d]\nvalues = [\"a\"]\nversionless = [\"a\"]\n[features.x]\nname = \"X\"\navailable = \"a preview 3.4\" #!\n",
        ),
        case(
            "model-availability-versionless",
            Some("dimension"),
            "spec = \"0.1\"\n[dimensions.d]\nvalues = [\"a\", \"b\"]\nversionless = [\"a\"]\n[features.x]\nname = \"X\"\navailable = \"d 3.4\" #!\n",
        ),
        case(
            "model-availability-history-order",
            None,
            "spec = \"0.1\"\n[dimensions.d]\nvalues = [\"a\"]\n[features.x]\nname = \"X\"\navailable = \"a (ga 3.5, preview 3.3)\" #!\n",
        ),
        case(
            "model-feature-nested",
            None,
            "spec = \"0.1\"\n[features.x]\nname = \"X\"\navailable = \"y\" #!\n[features.y]\nname = \"Y\"\navailable = \"y\"\n",
        ),
        // 21.5 Versions, phrases, glossary, and images
        case(
            "model-phrase-value-type",
            None,
            "spec = \"0.1\"\n[phrases]\nversion = 3.4 #!\n",
        ),
        case(
            "model-glossary-duplicate-term",
            None,
            "spec = \"0.1\"\n[glossary.terms.a]\nterm = \"API key\"\ndefinition = \"x\"\n[glossary.terms.b] #!\nterm = \"api key\"\ndefinition = \"y\"\n",
        ),
        case(
            "model-glossary-link",
            Some("fragment"),
            "spec = \"0.1\"\n[glossary.terms.a]\nterm = \"A\"\ndefinition = \"x\"\nlink = \"/_shared/a.md\" #!\n",
        ),
        // 21.6 Widgets
        case(
            "model-widget-reserved-name",
            None,
            "spec = \"0.1\"\n[widgets.ascribe-lab] #!\nforms = [\"container\"]\n",
        ),
        case(
            "model-widget-reserved-name",
            Some("html"),
            "spec = \"0.1\"\n[widgets.font-face] #!\nforms = [\"container\"]\n",
        ),
        case(
            "model-widget-forms",
            None,
            "spec = \"0.1\"\n[widgets.quill-lab]\nforms = [\"line\", \"line\"] #!\nbinding = \"self\"\n",
        ),
        case(
            "model-widget-binding",
            None,
            "spec = \"0.1\"\n[widgets.quill-lab] #!\nforms = [\"line\"]\n",
        ),
        case(
            "model-widget-binding",
            Some("container-only"),
            "spec = \"0.1\"\n[widgets.quill-lab]\nforms = [\"container\"]\nbinding = \"self\" #!\n",
        ),
        case(
            "model-widget-container-primary",
            None,
            "spec = \"0.1\"\n[widgets.quill-lab]\nforms = [\"line\", \"container\"]\nbinding = \"self\"\nprimary = \"text\" #!\n",
        ),
        case(
            "model-widget-container-primary",
            Some("container-only"),
            "spec = \"0.1\"\n[widgets.quill-lab]\nforms = [\"container\"]\nprimary = \"text?\" #!\n",
        ),
        case(
            "model-widget-groupable-form",
            None,
            "spec = \"0.1\"\n[widgets.quill-lab]\nforms = [\"line\"]\nbinding = \"self\"\ngroupable = true #!\n",
        ),
        case(
            "model-widget-plain-content",
            None,
            "spec = \"0.1\"\n[widgets.quill-lab]\nforms = [\"line\"]\nbinding = \"self\"\nplain-content = \"drop\" #!\n",
        ),
        // 21.7 Consumer, builds, and editor
        case(
            "model-consumer-unsupported",
            None,
            "spec = \"0.1\"\n[consumer]\nhtml = false #!\n",
        ),
        case(
            "model-consumer-site",
            None,
            "spec = \"0.1\"\n[consumer]\nsite = \"https://docs.example.com/docs\" #!\n",
        ),
        case(
            "model-consumer-base-path",
            None,
            "spec = \"0.1\"\n[consumer]\nbase-path = \"docs\" #!\n",
        ),
        case(
            "model-build-name-case",
            None,
            "spec = \"0.1\"\n[builds.site]\n[builds.Site] #!\n",
        ),
        case(
            "model-build-variants",
            None,
            "spec = \"0.1\"\n[builds.site]\nvariants = \"all\" #!\n",
        ),
        case(
            "model-build-variants",
            Some("empty"),
            "spec = \"0.1\"\n[builds.site]\nvariants = {} #!\n",
        ),
        case(
            "model-build-unknown-dimension",
            None,
            "spec = \"0.1\"\n[builds.site]\nvariants = { nope = \"x\" } #!\n",
        ),
        case(
            "model-build-unknown-value",
            None,
            "spec = \"0.1\"\n[dimensions.d]\nvalues = [\"a\"]\n[builds.site]\nvariants = { d = \"z\" } #!\n",
        ),
        case(
            "model-build-availability",
            None,
            "spec = \"0.1\"\n[builds.site]\navailability = \"hide\" #!\n",
        ),
        case(
            "model-build-filter-target",
            None,
            "spec = \"0.1\"\n[builds.site]\navailability = { filter = \"mars\" } #!\n",
        ),
        case(
            "model-build-filter-target",
            Some("dimension"),
            "spec = \"0.1\"\n[dimensions.d]\nvalues = [\"a\"]\n[builds.site]\navailability = { filter = \"d\" } #!\n",
        ),
        case(
            "model-build-filter-version",
            None,
            "spec = \"0.1\"\n[dimensions.d]\nvalues = [\"a\"]\n[builds.site]\navailability = { filter = \"a\" } #!\n",
        ),
        case(
            "model-build-filter-version",
            Some("versionless"),
            "spec = \"0.1\"\n[dimensions.d]\nvalues = [\"a\"]\nversionless = [\"a\"]\n[builds.site]\navailability = { filter = \"a 1.0\" } #!\n",
        ),
        case(
            "model-build-filter-version",
            Some("invalid"),
            "spec = \"0.1\"\n[dimensions.d]\nvalues = [\"a\"]\n[builds.site]\navailability = { filter = \"a 1.x\" } #!\n",
        ),
        case(
            "model-build-filter-excluded",
            None,
            "spec = \"0.1\"\n[dimensions.d]\nvalues = [\"a\", \"b\"]\nversionless = [\"a\", \"b\"]\n[builds.site]\nvariants = { d = \"a\" }\navailability = { filter = \"b\" } #!\n",
        ),
        case(
            "model-editor-build-unknown",
            None,
            "spec = \"0.1\"\n[builds.site]\n[editor]\nbuild = \"nope\" #!\n",
        ),
        case(
            "model-editor-build-required",
            None,
            "spec = \"0.1\"\n[builds.a] #!\n[builds.b]\n",
        ),
    ]
}

#[test]
fn every_rule_has_a_failing_fixture_with_the_documented_error() {
    let mut failures = Vec::new();
    for c in cases() {
        let text = c.toml;
        let issues = issues_of(text);
        let hit = issues
            .iter()
            .find(|i| i.slug.as_str() == c.slug && i.variant == c.variant);
        let Some(issue) = hit else {
            let got: Vec<String> = issues
                .iter()
                .map(|i| {
                    format!(
                        "{}:{:?}@{}",
                        i.slug,
                        i.variant,
                        line_of(text, i.location.span.start())
                    )
                })
                .collect();
            failures.push(format!(
                "{} {:?}: not reported; got {got:?}\n{text}",
                c.slug, c.variant
            ));
            continue;
        };
        if let Some(marker) = text.lines().position(|l| l.trim_end().ends_with("#!")) {
            let line = line_of(text, issue.location.span.start());
            if line != marker + 1 {
                failures.push(format!(
                    "{} {:?}: reported at line {line}, expected {}\n{text}",
                    c.slug,
                    c.variant,
                    marker + 1
                ));
            }
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n---\n"));
}

/// Every `model-` slug in the registry has a fixture here or in `fs.rs`.
#[test]
fn every_registered_rule_is_covered() {
    let registry =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/conformance/diagnostics.toml");
    let text = std::fs::read_to_string(registry).unwrap();
    let doc: toml::Table = text.parse().unwrap();
    let mut registered = BTreeSet::new();
    for d in doc["diagnostic"].as_array().unwrap() {
        let slug = d["slug"].as_str().unwrap();
        if slug.starts_with("model-") {
            registered.insert(slug.to_owned());
        }
    }
    let mut covered: BTreeSet<String> = cases().iter().map(|c| c.slug.to_owned()).collect();
    covered.extend(
        [
            "model-content-root-missing",
            "model-glossary-link",
            "model-source-path-missing",
            "model-source-outside-repository",
        ]
        .map(String::from),
    );
    let missing: Vec<_> = registered.difference(&covered).collect();
    assert!(missing.is_empty(), "no fixture for: {missing:?}");
}

/// Every issue names each placeholder of the template it uses, so a message
/// can be rendered from it.
#[test]
fn issues_carry_every_placeholder_of_their_message() {
    let registry =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/conformance/diagnostics.toml");
    let doc: toml::Table = std::fs::read_to_string(registry).unwrap().parse().unwrap();
    let entries = doc["diagnostic"].as_array().unwrap();
    let mut problems = Vec::new();
    for c in cases() {
        for issue in issues_of(c.toml) {
            let entry = entries
                .iter()
                .find(|e| e["slug"].as_str() == Some(issue.slug.as_str()))
                .unwrap();
            let template = match issue.variant {
                None => entry["message"].as_str().unwrap(),
                Some(v) => entry["messages"][v]
                    .as_str()
                    .unwrap_or_else(|| panic!("{} has no message variant {v}", issue.slug)),
            };
            let mut rest = template.replace("{{", "").replace("}}", "");
            while let Some(start) = rest.find('{') {
                let Some(end) = rest[start..].find('}') else {
                    break;
                };
                let name = &rest[start + 1..start + end];
                if issue.arg(name).is_none() {
                    problems.push(format!(
                        "{} {:?}: missing arg `{name}`",
                        issue.slug, issue.variant
                    ));
                }
                rest = rest[start + end + 1..].to_owned();
            }
        }
    }
    problems.sort();
    problems.dedup();
    assert!(problems.is_empty(), "{}", problems.join("\n"));
}
