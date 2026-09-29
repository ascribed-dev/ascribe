//! Build modes and availability (SPEC §4.4, §9.3): selection, filter, page
//! dropping, and scope problems.

#![allow(clippy::expect_used, clippy::panic)]

mod build_support;

use build_support::{project, published, resolve, summary};
use tessera_resolve::{DropReason, ResolvedKind, Scope};

const PAGE: &str = "---\ntitle: Test\n---\n\n";

fn page(body: &str) -> String {
    format!("{PAGE}{body}")
}

fn slugs(page: &tessera_resolve::ResolvedPage) -> Vec<String> {
    page.problems
        .iter()
        .map(|p| p.issue.slug.to_string())
        .collect()
}

// -- Selection --------------------------------------------------------------

const GROUPS: &str = "@variant {deployment=cloud}:
Cloud text.
@variant {deployment=self-managed}:
Self-managed text.
@end

@variant {pm=npm}:
npm text.
@variant {pm=yarn}:
yarn text.
@end
";

#[test]
fn switch_keeps_every_arm_of_every_group() {
    let p = project(&[("index.md", &page(GROUPS))]);
    assert_eq!(
        summary(&resolve(&p, "index.md", "site")),
        [
            "group[deployment=cloud | deployment=self-managed]",
            "  p Cloud text.",
            "  p Self-managed text.",
            "group[pm=npm | pm=yarn]",
            "  p npm text.",
            "  p yarn text.",
        ]
    );
}

#[test]
fn a_selection_reduces_the_deployment_group_and_leaves_the_pm_group_a_switcher() {
    // SPEC §9.3's example: the cloud build keeps the whole `pm` group.
    let p = project(&[("index.md", &page(GROUPS))]);
    assert_eq!(
        summary(&resolve(&p, "index.md", "cloud-only")),
        [
            "p Cloud text.",
            "group[pm=npm | pm=yarn]",
            "  p npm text.",
            "  p yarn text.",
        ]
    );
    // And the other way round.
    assert_eq!(
        summary(&resolve(&p, "index.md", "npm-only")),
        [
            "group[deployment=cloud | deployment=self-managed]",
            "  p Cloud text.",
            "  p Self-managed text.",
            "p npm text.",
        ]
    );
}

#[test]
fn several_surviving_arms_stay_a_group() {
    let p = project(&[(
        "index.md",
        &page(
            "@variant {pm=npm}:
npm.
@variant {pm=pnpm}:
pnpm.
@variant {pm=yarn}:
yarn.
@end
",
        ),
    )]);
    // `sm-npm-pnpm` selects `pm` values npm and pnpm: two arms remain.
    assert_eq!(
        summary(&resolve(&p, "index.md", "sm-npm-pnpm")),
        ["group[pm=npm | pm=pnpm]", "  p npm.", "  p pnpm."]
    );
}

#[test]
fn a_value_set_arm_applies_to_any_of_its_values() {
    let p = project(&[(
        "index.md",
        &page(
            "@variant {pm=npm|pnpm}:
npm or pnpm.
@variant {pm=yarn}:
yarn.
@end
",
        ),
    )]);
    assert_eq!(
        summary(&resolve(&p, "index.md", "npm-only")),
        ["p npm or pnpm."]
    );
}

#[test]
fn an_arm_with_several_keys_applies_only_when_all_match() {
    let p = project(&[(
        "index.md",
        &page(
            "@variant {deployment=cloud, pm=npm}:
Cloud npm.
@variant {deployment=self-managed, pm=npm}:
Server npm.
@variant {deployment=self-managed, pm=yarn}:
Server yarn.
@end
",
        ),
    )]);
    // Selecting deployment=self-managed and pm in [npm, pnpm]: only the arm
    // whose every key matches survives.
    assert_eq!(
        summary(&resolve(&p, "index.md", "sm-npm-pnpm")),
        ["p Server npm."]
    );
    // Selecting one dimension only removes arms that conflict along it.
    assert_eq!(
        summary(&resolve(&p, "index.md", "npm-only")),
        [
            "group[deployment=cloud,pm=npm | deployment=self-managed,pm=npm]",
            "  p Cloud npm.",
            "  p Server npm.",
        ]
    );
}

#[test]
fn groups_on_unselected_dimensions_and_labeled_groups_are_untouched() {
    // A one-arm group keyed only on `pm`, under a selection along
    // `deployment`, is rendered as in `switch`: it stays a group.
    let p = project(&[(
        "index.md",
        &page(
            "@variant {pm=npm}:
Only npm.
@end

.First
@variant:
One.

.Second
@variant:
Two.
@end
",
        ),
    )]);
    let out = summary(&resolve(&p, "index.md", "cloud-only"));
    assert_eq!(
        out,
        [
            "group[pm=npm]",
            "  p Only npm.",
            "group[First | Second]",
            "  p One.",
            "  p Two."
        ]
    );
}

#[test]
fn a_group_with_no_surviving_arm_is_removed_and_recorded_at_its_first_opener() {
    let p = project(&[(
        "index.md",
        &page(
            "Before.

@variant {deployment=self-managed}:
Self-managed only.
@variant {deployment=self-managed, pm=npm}:
Also.
@end

After.
",
        ),
    )]);
    let resolved = resolve(&p, "index.md", "cloud-only");
    assert_eq!(summary(&resolved), ["p Before.", "p After."]);
    let problem = &resolved.problems[0];
    assert_eq!(problem.issue.slug.as_str(), "variant-no-arm-survives");
    assert_eq!(problem.issue.arg("build"), Some("cloud-only"));
    // At the first opener, line 7 of the file.
    let source = &p
        .file(&build_support::path("index.md"))
        .expect("file")
        .source;
    let line = source[..problem.issue.location.span.start()]
        .matches('\n')
        .count()
        + 1;
    assert_eq!(line, 7);
    // Other builds are quiet.
    assert!(resolve(&p, "index.md", "site").problems.is_empty());
}

#[test]
fn a_group_inside_a_removed_arm_isnt_reported() {
    let p = project(&[(
        "index.md",
        &page(
            "@variant {deployment=cloud}:
Cloud.
@variant {deployment=self-managed}:
- item

  @variant {deployment=self-managed}:
  Nested.
  @end
@end
",
        ),
    )]);
    let resolved = resolve(&p, "index.md", "cloud-only");
    assert_eq!(summary(&resolved), ["p Cloud."]);
    assert!(resolved.problems.is_empty(), "{:?}", slugs(&resolved));
}

// -- Pages ------------------------------------------------------------------

const PAGES: &[(&str, &str)] = &[
    ("index.md", "---\ntitle: Home\n---\n"),
    (
        "cloud.md",
        "---\ntitle: Cloud\nvariant:\n  deployment: cloud\n---\n",
    ),
    (
        "sm.md",
        "---\ntitle: SM\nvariant:\n  deployment: self-managed\n---\n",
    ),
    (
        "both.md",
        "---\ntitle: Both\nvariant:\n  deployment: [cloud, self-managed]\n---\n",
    ),
    ("yarn.md", "---\ntitle: Yarn\nvariant:\n  pm: yarn\n---\n"),
    ("_shared.md", "Shared.\n"),
];

#[test]
fn a_selection_drops_pages_whose_variant_conflicts() {
    let p = project(PAGES);
    assert_eq!(
        published(&p, "site"),
        ["both.md", "cloud.md", "index.md", "sm.md", "yarn.md"]
    );
    assert_eq!(
        published(&p, "cloud-only"),
        ["both.md", "cloud.md", "index.md", "yarn.md"]
    );
    assert_eq!(
        published(&p, "npm-only"),
        ["both.md", "cloud.md", "index.md", "sm.md"]
    );
    // Both dimensions selected: `both` and `index` always stay.
    assert_eq!(
        published(&p, "sm-npm-pnpm"),
        ["both.md", "index.md", "sm.md"]
    );
    let build = p.model().build("cloud-only").expect("build");
    assert_eq!(
        p.dropped(&build_support::path("sm.md"), build),
        Some(DropReason::Variant)
    );
    assert_eq!(p.dropped(&build_support::path("cloud.md"), build), None);
}

#[test]
fn fragments_are_never_published() {
    let p = project(PAGES);
    let b = p.model().build("site").expect("build");
    assert!(
        p.resolve_page(
            &build_support::path("_shared.md"),
            b,
            &build_support::router(&p)
        )
        .is_none()
    );
    assert!(!published(&p, "site").contains(&"_shared.md".to_owned()));
}

// -- Filter -----------------------------------------------------------------

const STREAMING: &str = "# Page

Intro.

## Streaming
@available: cloud, self-managed preview 3.4

Streaming text.

## After

Closing text.
";

#[test]
fn a_section_before_its_first_state_is_removed_and_from_it_kept() {
    let p = project(&[("index.md", &page(STREAMING))]);
    let with = [
        "h1 Page",
        "p Intro.",
        "h2 Streaming",
        "@available cloud, self-managed preview 3.4",
        "p Streaming text.",
        "h2 After",
        "p Closing text.",
    ];
    // 3.3 precedes preview 3.4: not available. The section goes, with its
    // heading and its directive.
    assert_eq!(
        summary(&resolve(&p, "index.md", "sm-3.3")),
        ["h1 Page", "p Intro.", "h2 After", "p Closing text."]
    );
    // At 3.4 the state begins, so it stays, annotated.
    assert_eq!(summary(&resolve(&p, "index.md", "sm-3.4")), with);
    // Cloud is listed with no state: generally available.
    assert_eq!(summary(&resolve(&p, "index.md", "cloud")), with);
    // A badge build keeps everything.
    assert_eq!(summary(&resolve(&p, "index.md", "site")), with);
}

#[test]
fn the_state_in_effect_is_the_last_that_began_and_an_unavailable_state_removes() {
    let p = project(&[(
        "index.md",
        &page(
            "# Page

## Sunset
@available: self-managed (preview 3.3, sunset 3.5)

Sunset text.
",
        ),
    )]);
    let kept = [
        "h1 Page",
        "h2 Sunset",
        "@available self-managed (preview 3.3, sunset 3.5)",
        "p Sunset text.",
    ];
    // Before the first state, nothing is in effect.
    assert_eq!(summary(&resolve(&p, "index.md", "sm-3.3")), kept);
    // Between the states: preview still applies at 3.4.
    assert_eq!(summary(&resolve(&p, "index.md", "sm-3.4")), kept);
    // At 3.5 `sunset`, which the model says doesn't count as available.
    assert_eq!(summary(&resolve(&p, "index.md", "sm-3.5")), ["h1 Page"]);
    // A target the spec doesn't list isn't available.
    assert_eq!(summary(&resolve(&p, "index.md", "cloud")), ["h1 Page"]);
}

#[test]
fn a_state_that_doesnt_count_as_available_removes_content_at_the_first_state() {
    let p = project(&[(
        "index.md",
        &page("# Page\n\n## Gone\n@available: self-managed sunset 3.3\n\nGone.\n"),
    )]);
    assert_eq!(summary(&resolve(&p, "index.md", "sm-3.3")), ["h1 Page"]);
    // Before 3.3 nothing is in effect either.
    let earlier = project(&[(
        "index.md",
        &page("# Page\n\n## Later\n@available: self-managed 3.4\n\nLater.\n"),
    )]);
    assert_eq!(
        summary(&resolve(&earlier, "index.md", "sm-3.3")),
        ["h1 Page"]
    );
}

#[test]
fn a_versionless_target_with_a_removed_state_is_never_available() {
    let p = project(&[(
        "index.md",
        &page("# Page\n\n## Gone on cloud\n@available: cloud removed\n\nGone.\n"),
    )]);
    assert_eq!(summary(&resolve(&p, "index.md", "cloud")), ["h1 Page"]);
    // Nor is it available for a target it doesn't list.
    assert_eq!(summary(&resolve(&p, "index.md", "sm-3.3")), ["h1 Page"]);
    // A badge build annotates it.
    assert_eq!(
        summary(&resolve(&p, "index.md", "site")),
        [
            "h1 Page",
            "h2 Gone on cloud",
            "@available cloud removed",
            "p Gone."
        ]
    );
}

#[test]
fn a_bare_version_and_a_dimension_name() {
    let p = project(&[(
        "index.md",
        &page(
            "# Page

## Since 3.4
@available: self-managed 3.4

Text.

## Everywhere
@available: deployment

All.
",
        ),
    )]);
    assert_eq!(
        summary(&resolve(&p, "index.md", "sm-3.3")),
        [
            "h1 Page",
            "h2 Everywhere",
            "@available deployment",
            "p All."
        ]
    );
    assert_eq!(summary(&resolve(&p, "index.md", "sm-3.4")).len(), 7);
    // A dimension name stands for all its values, including versionless ones.
    assert_eq!(
        summary(&resolve(&p, "index.md", "cloud")),
        [
            "h1 Page",
            "h2 Everywhere",
            "@available deployment",
            "p All."
        ]
    );
}

#[test]
fn a_feature_key_filters_as_its_spec_and_the_kept_annotation_shows_the_spec() {
    let p = project(&[(
        "index.md",
        &page("# Page\n\n## Streaming\n@available: streaming-sync\n\nText.\n"),
    )]);
    assert_eq!(summary(&resolve(&p, "index.md", "sm-3.3")), ["h1 Page"]);
    let kept = resolve(&p, "index.md", "sm-3.4");
    assert_eq!(
        summary(&kept),
        [
            "h1 Page",
            "h2 Streaming",
            "@available cloud, self-managed preview 3.4",
            "p Text."
        ]
    );
    // The annotation says which key it was.
    let annotation = kept
        .blocks
        .iter()
        .find_map(|b| b.annotation.as_ref())
        .expect("an annotation");
    assert_eq!(annotation.feature.as_deref(), Some("streaming-sync"));
}

#[test]
fn a_section_spec_removes_the_section_with_its_subsections_and_a_block_spec_only_its_block() {
    let p = project(&[(
        "index.md",
        &page(
            "# Page

## Parent
@available: cloud

Parent text.

### Child

Child text.

## Sibling

Sibling text.

@available: cloud
Cloud-only block.

After block.
",
        ),
    )]);
    assert_eq!(
        summary(&resolve(&p, "index.md", "sm-3.3")),
        ["h1 Page", "h2 Sibling", "p Sibling text.", "p After block."]
    );
    let cloud = summary(&resolve(&p, "index.md", "cloud"));
    assert!(cloud.contains(&"h3 Child".to_owned()));
    assert!(cloud.contains(&"p Cloud-only block.".to_owned()));
    // Content that stays keeps its annotations.
    assert_eq!(
        cloud.iter().filter(|l| l.starts_with("@available")).count(),
        2
    );
}

#[test]
fn availability_is_inherited_and_every_node_has_its_effective_spec() {
    let p = project(&[(
        "index.md",
        "---\ntitle: T\navailable: cloud, self-managed 3.3\n---\n\nTop.\n\n## Section\n@available: cloud\n\nIn section.\n\n### Sub\n\nIn sub.\n\n## Other\n\nOther.\n",
    )]);
    let page = resolve(&p, "index.md", "site");
    assert_eq!(
        page.availability.as_ref().map(|a| a.text.as_str()),
        Some("cloud, self-managed 3.3")
    );
    let mut seen: Vec<(String, Option<(String, Scope)>)> = Vec::new();
    page.visit(&mut |b| {
        if let ResolvedKind::Leaf(leaf) = &b.kind
            && let tessera_syntax::BlockKind::Paragraph(par) = &leaf.kind
        {
            seen.push((
                build_support::plain(&par.inlines),
                b.availability.as_ref().map(|a| (a.text.clone(), a.scope)),
            ));
        }
    });
    let page_spec = Some(("cloud, self-managed 3.3".to_owned(), Scope::Page));
    let section_spec = Some(("cloud".to_owned(), Scope::Section));
    assert_eq!(
        seen,
        [
            ("Top.".to_owned(), page_spec.clone()),
            ("In section.".to_owned(), section_spec.clone()),
            ("In sub.".to_owned(), section_spec),
            ("Other.".to_owned(), page_spec),
        ]
    );
}

#[test]
fn a_page_whose_available_frontmatter_is_unavailable_is_dropped_from_a_filter_build() {
    let p = project(&[
        ("index.md", "---\ntitle: Home\n---\n"),
        (
            "newer.md",
            "---\ntitle: Newer\navailable: self-managed 3.4\n---\n",
        ),
        (
            "gone.md",
            "---\ntitle: Gone\navailable: cloud removed\n---\n",
        ),
    ]);
    assert_eq!(published(&p, "sm-3.3"), ["index.md"]);
    assert_eq!(published(&p, "sm-3.4"), ["index.md", "newer.md"]);
    assert_eq!(published(&p, "cloud"), ["index.md"]);
    assert_eq!(published(&p, "site"), ["gone.md", "index.md", "newer.md"]);
    let build = p.model().build("sm-3.3").expect("build");
    assert_eq!(
        p.dropped(&build_support::path("newer.md"), build),
        Some(DropReason::Unavailable)
    );
}

#[test]
fn filter_and_selection_apply_together() {
    let p = project(&[(
        "index.md",
        &page(
            "# Page

@variant {deployment=cloud}:
Cloud text.
@variant {deployment=self-managed}:
Self-managed text.
@end

## Self-managed features
@available: self-managed 3.3

Features.
",
        ),
    )]);
    assert_eq!(
        summary(&resolve(&p, "index.md", "cloud")),
        ["h1 Page", "p Cloud text."]
    );
    assert_eq!(
        summary(&resolve(&p, "index.md", "sm-3.3")),
        [
            "h1 Page",
            "group[deployment=cloud | deployment=self-managed]",
            "  p Cloud text.",
            "  p Self-managed text.",
            "h2 Self-managed features",
            "@available self-managed 3.3",
            "p Features."
        ]
    );
}

// -- Scope problems ---------------------------------------------------------

fn scope_problems(front: &str, body: &str) -> Vec<(String, Option<String>, Option<String>)> {
    let p = project(&[("index.md", &format!("---\ntitle: T\n{front}---\n\n{body}"))]);
    resolve(&p, "index.md", "site")
        .problems
        .iter()
        .filter(|p| p.issue.slug.as_str() == "available-exceeds-scope")
        .map(|p| {
            (
                p.issue.variant.unwrap_or("").to_owned(),
                p.issue.arg("target").map(str::to_owned),
                p.issue.arg("scope").map(str::to_owned),
            )
        })
        .collect()
}

#[test]
fn a_spec_that_lists_a_target_its_scope_doesnt_is_recorded() {
    let got = scope_problems(
        "available: cloud\n",
        "## S\n@available: cloud, self-managed\n\nT.\n",
    );
    assert_eq!(
        got,
        [(
            String::new(),
            Some("self-managed".into()),
            Some("page".into())
        )]
    );
}

#[test]
fn a_spec_that_starts_earlier_than_its_scope_is_recorded_with_versions() {
    let p = project(&[(
        "index.md",
        "---\ntitle: T\navailable: self-managed 3.3\n---\n\n## S\n@available: self-managed 3.2\n\nT.\n",
    )]);
    let page = resolve(&p, "index.md", "site");
    let issue = &page.problems[0].issue;
    assert_eq!(issue.variant, Some("version"));
    assert_eq!(issue.arg("version"), Some("3.2"));
    assert_eq!(issue.arg("enclosing"), Some("3.3"));
}

#[test]
fn a_dimension_name_stands_for_all_its_values_and_a_feature_key_for_its_spec() {
    // A dimension name exceeds a single value…
    let got = scope_problems("available: cloud\n", "## S\n@available: deployment\n\nT.\n");
    assert_eq!(got.len(), 1);
    // …fits where the scope lists the dimension or all its values…
    assert!(
        scope_problems("available: deployment\n", "## S\n@available: cloud\n\nT.\n").is_empty()
    );
    assert!(
        scope_problems(
            "available: cloud, self-managed\n",
            "## S\n@available: deployment\n\nT.\n"
        )
        .is_empty()
    );
    // …and a feature key is checked as the spec it stands for.
    let got = scope_problems(
        "available: cloud\n",
        "## S\n@available: streaming-sync\n\nT.\n",
    );
    assert_eq!(got.len(), 1);
    assert!(scope_problems("", "## S\n@available: streaming-sync\n\nT.\n").is_empty());
}

#[test]
fn a_block_spec_is_checked_against_its_section() {
    let got = scope_problems(
        "available: cloud, self-managed 3.3\n",
        "## S\n@available: cloud\n\nT.\n\n@available: cloud, self-managed 3.3\nA block.\n",
    );
    assert_eq!(
        got,
        [(
            String::new(),
            Some("self-managed".into()),
            Some("section".into())
        )]
    );
}

#[test]
fn a_spec_that_fits_its_scope_records_nothing() {
    assert!(scope_problems(
        "available: cloud, self-managed preview 3.3\n",
        "## S\n@available: self-managed preview 3.4\n\nT.\n\n@available: self-managed 3.5\nA block.\n",
    )
    .is_empty());
}

#[test]
fn a_directive_in_an_included_fragment_is_recorded_at_the_include_site_chain() {
    let p = project(&[
        (
            "index.md",
            "---\ntitle: T\navailable: cloud\n---\n\n@include: _f.md\n",
        ),
        ("_f.md", "## S\n@available: self-managed\n\nT.\n"),
    ]);
    let page = resolve(&p, "index.md", "site");
    let problem = &page.problems[0];
    assert_eq!(problem.issue.slug.as_str(), "available-exceeds-scope");
    // The issue is in the fragment; the report goes at the include in the page.
    assert_eq!(
        p.path_of(problem.issue.location.file)
            .map(|p| p.to_string()),
        Some("_f.md".into())
    );
    assert_eq!(problem.via.len(), 1);
    assert_eq!(
        p.path_of(problem.via[0].file).map(|p| p.to_string()),
        Some("index.md".into())
    );
}

// -- Problems concern what a build publishes (Q81) ---------------------------

#[test]
fn a_scope_problem_in_content_a_build_removes_is_not_recorded_for_that_build() {
    let p = project(&[(
        "index.md",
        &format!(
            "{PAGE}@variant {{deployment=cloud}}:\nCloud.\n@variant {{deployment=self-managed}}:\n## S\n@available: self-managed\n\nText.\n@end\n"
        )
        .replace("---\ntitle: Test\n---", "---\ntitle: Test\navailable: cloud, self-managed 3.3\n---"),
    )]);
    // `self-managed` exceeds `self-managed 3.3`? No: it starts earlier (no
    // version), so it does; in `switch` the arm is published.
    assert_eq!(
        slugs(&resolve(&p, "index.md", "site")),
        ["available-exceeds-scope"]
    );
    // Under the cloud selection the arm is gone, and so is the problem.
    assert!(slugs(&resolve(&p, "index.md", "cloud-only")).is_empty());
}

#[test]
fn an_include_problem_in_content_a_build_removes_is_not_recorded_for_that_build() {
    let p = project(&[
        (
            "index.md",
            &page(
                "@variant {deployment=cloud}:\nCloud.\n@variant {deployment=self-managed}:\n@include: _f.md#missing\n@end\n",
            ),
        ),
        ("_f.md", "## Here\n"),
    ]);
    assert_eq!(
        slugs(&resolve(&p, "index.md", "site")),
        ["include-id-missing"]
    );
    assert!(slugs(&resolve(&p, "index.md", "cloud-only")).is_empty());
}
