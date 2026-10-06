//! Phrases, page ids, links, assets, and the glossary in a resolved page
//! (SPEC §5, §9.2 steps 4 to 7), and what the resolved tree keeps.

#![allow(clippy::expect_used, clippy::panic)]

mod build_support;

use build_support::{MODEL, path, plain, project, project_with, resolve, summary};
use tessera_model::Segment;
use tessera_resolve::{LinkTarget, RefKind, ResolvedBlock, ResolvedKind, ResolvedPage};
use tessera_syntax::{BlockKind, InlineKind};

const PAGE: &str = "---\ntitle: Test\n---\n\n";

fn page(body: &str) -> String {
    format!("{PAGE}{body}")
}

fn headings(page: &ResolvedPage) -> Vec<(String, String, String)> {
    page.headings()
        .into_iter()
        .map(|(_, h)| (h.text.clone(), h.source_id.clone(), h.page_id.clone()))
        .collect()
}

/// Every link of a page, in document order.
fn links(page: &ResolvedPage) -> Vec<tessera_resolve::ResolvedLink> {
    let mut out = Vec::new();
    page.visit(&mut |b| out.extend(b.links.iter().cloned()));
    out
}

// -- Phrases ----------------------------------------------------------------

#[test]
fn phrases_are_substituted_in_prose_headings_link_text_and_destinations() {
    let p = project(&[(
        "index.md",
        &page(
            "# Connect to {product}\n@id: connect\n\nSign in to {cloud} and copy an API key.\n\n{cloud}'s sync, {cloud}-hosted.\n\nSee the [{product} API]({api}streaming) and *{product}*.\n",
        ),
    )]);
    let resolved = resolve(&p, "index.md", "site");
    assert_eq!(
        summary(&resolved),
        [
            "h1 Connect to Quill",
            "@id connect",
            "p Sign in to Quill Cloud and copy an API key.",
            "p Quill Cloud's sync, Quill Cloud-hosted.",
            "p See the Quill API and Quill.",
        ]
    );
    // The link destination is substituted too.
    let all = links(&resolved);
    assert_eq!(all[0].destination, "https://api.quill.dev/v3/streaming");
    assert_eq!(all[0].target, LinkTarget::External);
}

#[test]
fn undeclared_candidates_and_escapes_stay_literal_and_values_are_not_rescanned() {
    let p = project(&[(
        "index.md",
        &page("The {nope} key, \\{product} and {product}, and {nested}.\n"),
    )]);
    assert_eq!(
        summary(&resolve(&p, "index.md", "site")),
        ["p The {nope} key, {product} and Quill, and {product}."]
    );
}

#[test]
fn phrases_apply_in_opted_in_fences_only_and_never_in_code_or_html() {
    let p = project(&[(
        "index.md",
        &page(
            "Use `{product}` here.\n\n```yaml\nname: {product}\n```\n\n```yaml phrases=true\nname: {product}\n```\n\n    {product} indented\n\n<div>{product}</div>\n",
        ),
    )]);
    assert_eq!(
        summary(&resolve(&p, "index.md", "site")),
        [
            "p Use {product} here.",
            "code name: {product}",
            "code name: Quill",
            "code {product} indented",
            "html <div>{product}</div>",
        ]
    );
}

#[test]
fn substitutions_are_listed_over_the_source_text() {
    let source = page("Hi {product} and {cloud}.\n");
    let p = project(&[("index.md", &source)]);
    let resolved = resolve(&p, "index.md", "site");
    let subs = &resolved.blocks[0].substitutions;
    assert_eq!(subs.len(), 2);
    assert_eq!(&source[subs[0].span.range()], "{product}");
    assert_eq!(subs[0].value, "Quill");
    assert_eq!(&source[subs[1].span.range()], "{cloud}");
}

#[test]
fn frontmatter_phrases_apply_only_to_the_fields_that_opt_in() {
    let p = project(&[(
        "index.md",
        "---\ntitle: Get {product}\ndescription: About {product}\ntags: [\"{cloud}\", plain]\nnote: \"{product}\"\n---\n\nBody.\n",
    )]);
    let resolved = resolve(&p, "index.md", "site");
    assert_eq!(resolved.title.as_deref(), Some("Get Quill"));
    let fm = resolved.frontmatter.expect("frontmatter");
    assert_eq!(fm["title"].as_str(), Some("Get Quill"));
    // `description` and `note` don't set `phrases = true`.
    assert_eq!(fm["description"].as_str(), Some("About {product}"));
    assert_eq!(fm["note"].as_str(), Some("{product}"));
    assert_eq!(fm["tags"][0].as_str(), Some("Quill Cloud"));
    assert_eq!(fm["tags"][1].as_str(), Some("plain"));
}

#[test]
fn an_inline_field_is_plain_text_beside_its_pieces() {
    let model = MODEL.replace(
        "title = { type = \"string\", phrases = true }",
        "title = { type = \"string\", phrases = true, inline = \"code\" }\nsummary = { type = \"string\", inline = \"code\", default = \"`x` {product}\" }",
    );
    let p = project_with(
        &model,
        &[(
            "index.md",
            "---\ntitle: \"`{product}` for {product}\"\n---\n\nBody.\n",
        )],
    );
    let resolved = resolve(&p, "index.md", "site");
    // Phrases are substituted in the text, not in code spans (SPEC §5.1).
    assert_eq!(resolved.title.as_deref(), Some("{product} for Quill"));
    let fm = resolved.frontmatter.as_ref().expect("frontmatter");
    assert_eq!(fm["title"].as_str(), Some("{product} for Quill"));
    assert_eq!(
        resolved.formatted_title(),
        Some(
            &[
                Segment::Code("{product}".into()),
                Segment::Text(" for Quill".into())
            ][..]
        )
    );
    // A field the page leaves out has its default's pieces; a default
    // doesn't take phrases.
    assert_eq!(resolved.formatted[1].name, "summary");
    assert_eq!(
        resolved.formatted[1].segments,
        vec![
            Segment::Code("x".into()),
            Segment::Text(" {product}".into())
        ]
    );
}

// -- Page ids ---------------------------------------------------------------

#[test]
fn page_ids_are_numbered_across_the_expanded_page_and_differ_from_source_ids() {
    // The fragment's `Intro` is `intro` in its own file; on the page it's
    // the second `Intro`.
    let p = project(&[
        ("index.md", &page("# Intro\n\n@include: _f.md\n")),
        ("_f.md", "## Intro\n\nText.\n"),
    ]);
    let resolved = resolve(&p, "index.md", "site");
    assert_eq!(
        headings(&resolved),
        [
            ("Intro".to_owned(), "intro".to_owned(), "intro".to_owned()),
            ("Intro".to_owned(), "intro".to_owned(), "intro-1".to_owned()),
        ]
    );
    // The page's own source id is unchanged: ids are per file.
    assert_eq!(
        p.heading(&path("index.md"), "intro").map(|h| h.level),
        Some(1)
    );
}

#[test]
fn explicit_ids_are_page_ids_and_dont_take_part_in_numbering() {
    let p = project(&[(
        "index.md",
        &page("## Setup\n@id: intro\n\n## Intro\n\n## Intro\n"),
    )]);
    let resolved = resolve(&p, "index.md", "site");
    // The slug `intro` is numbered only against earlier slugs, so the first
    // `Intro` is `intro` (which duplicates the `@id`, a page-level check).
    assert_eq!(
        headings(&resolved)
            .iter()
            .map(|(_, _, page_id)| page_id.as_str())
            .collect::<Vec<_>>(),
        ["intro", "intro", "intro-1"]
    );
    let flags: Vec<bool> = resolved
        .headings()
        .iter()
        .map(|(_, h)| h.explicit)
        .collect();
    assert_eq!(flags, [true, false, false]);
}

#[test]
fn ids_are_assigned_after_build_modes() {
    // The first `Setup` is removed by the selection, so the second is `setup`.
    let p = project(&[(
        "index.md",
        &page(
            "@variant {deployment=self-managed}:\n## Setup\n\nServer.\n@variant {deployment=cloud}:\n## Setup\n\nCloud.\n@end\n\n## Setup\n",
        ),
    )]);
    let switch = resolve(&p, "index.md", "site");
    assert_eq!(
        headings(&switch)
            .iter()
            .map(|h| h.2.clone())
            .collect::<Vec<_>>(),
        ["setup", "setup-1", "setup-2"]
    );
    let cloud = resolve(&p, "index.md", "cloud-only");
    assert_eq!(
        headings(&cloud)
            .iter()
            .map(|h| h.2.clone())
            .collect::<Vec<_>>(),
        ["setup", "setup-1"]
    );
}

#[test]
fn a_section_include_without_its_heading_keeps_the_included_content() {
    let p = project(&[
        (
            "index.md",
            &page("# Page\n\n@include {heading=false}: _f.md#install\n"),
        ),
        (
            "_f.md",
            "## Install\n@id: install\n\nRun it.\n\n### Detail\n\nMore.\n",
        ),
    ]);
    let resolved = resolve(&p, "index.md", "site");
    assert_eq!(
        summary(&resolved),
        [
            "h1 Page",
            "@id install",
            "p Run it.",
            "h3 Detail",
            "p More."
        ]
    );
}

// -- Links ------------------------------------------------------------------

#[test]
fn a_link_to_a_page_points_at_its_route_and_an_empty_one_takes_the_pages_title() {
    let p = project(&[
        (
            "index.md",
            &page("See [](keys.md) and [text](guides/setup.md).\n"),
        ),
        ("keys.md", "---\ntitle: Rotate {product} keys\n---\n"),
        ("guides/setup.md", "---\ntitle: Setup\n---\n"),
    ]);
    let resolved = resolve(&p, "index.md", "site");
    assert_eq!(summary(&resolved), ["p See Rotate Quill keys and text."]);
    let all = links(&resolved);
    assert_eq!(
        all[0].target,
        LinkTarget::Page {
            page: path("keys.md"),
            id: None,
            url: "/docs/keys/".into(),
            text_filled: true,
        }
    );
    assert_eq!(all[1].destination, "guides/setup.md");
    let ResolvedKind::Leaf(leaf) = &resolved.blocks[0].kind else {
        panic!("a paragraph");
    };
    let BlockKind::Paragraph(par) = &leaf.kind else {
        panic!("a paragraph");
    };
    let urls: Vec<&str> = par
        .inlines
        .iter()
        .filter_map(|i| match &i.kind {
            InlineKind::Link(l) => Some(l.destination.as_str()),
            _ => None,
        })
        .collect();
    assert_eq!(urls, ["/docs/keys/", "/docs/guides/setup/"]);
}

#[test]
fn a_link_to_an_id_points_at_the_headings_page_id_and_an_empty_one_takes_its_text() {
    // On `keys.md` the second `Rotate` is `rotate-1`; the link says `rotate`
    // by its source id and must land on the page id.
    let p = project(&[
        (
            "index.md",
            &page("[](keys.md#rotate) and [](keys.md#with-id).\n"),
        ),
        (
            "keys.md",
            "---\ntitle: Keys\n---\n\n## Rotate\n\n@include: _f.md\n\n## Later\n@id: with-id\n",
        ),
        ("_f.md", "## Rotate\n\nFragment.\n"),
    ]);
    let resolved = resolve(&p, "index.md", "site");
    assert_eq!(summary(&resolved), ["p Rotate and Later."]);
    let all = links(&resolved);
    let LinkTarget::Page { id, url, .. } = &all[0].target else {
        panic!("a page link");
    };
    // The page's own `Rotate` is the first on its page: `rotate`.
    assert_eq!(id.as_deref(), Some("rotate"));
    assert_eq!(url, "/docs/keys/#rotate");
    let LinkTarget::Page { url, .. } = &all[1].target else {
        panic!("a page link");
    };
    assert_eq!(url, "/docs/keys/#with-id");
}

#[test]
fn a_link_follows_its_targets_page_id_when_an_include_moves_the_slug() {
    // `keys.md` includes a fragment whose heading comes first, so the page's
    // own `Rotate` is numbered `rotate-1` on the published page.
    let p = project(&[
        ("index.md", &page("[link](keys.md#rotate)\n")),
        (
            "keys.md",
            "---\ntitle: Keys\n---\n\n@include: _f.md\n\n## Rotate\n",
        ),
        ("_f.md", "## Rotate\n"),
    ]);
    let resolved = resolve(&p, "index.md", "site");
    let all = links(&resolved);
    let LinkTarget::Page { url, .. } = &all[0].target else {
        panic!("a page link");
    };
    assert_eq!(url, "/docs/keys/#rotate-1");
}

#[test]
fn a_hash_only_link_in_a_fragment_points_at_the_page_id_on_each_including_page() {
    // `[](#top)` in the fragment names the fragment's own heading, which
    // is `top` on one page and `top-1` on the other.
    let p = project(&[
        ("a.md", &page("# Top\n\n@include: _f.md\n")),
        ("b.md", &page("# Other\n\n@include: _f.md\n")),
        ("_f.md", "## Top\n\nBack to [](#top).\n"),
    ]);
    let on = |page: &str| {
        let resolved = resolve(&p, page, "site");
        let all = links(&resolved);
        assert_eq!(all.len(), 1);
        match &all[0].target {
            LinkTarget::Page { url, .. } => url.clone(),
            other => panic!("expected a page link, got {other:?}"),
        }
    };
    assert_eq!(on("a.md"), "/docs/a/#top-1");
    assert_eq!(on("b.md"), "/docs/b/#top");
}

#[test]
fn a_link_to_a_page_the_build_drops_is_recorded_and_left_unresolved() {
    let p = project(&[
        (
            "index.md",
            &page(
                "[Server setup](sm.md)\n\n@variant {deployment=cloud}:\nCloud.\n@variant {deployment=self-managed}:\nSee [the server setup](sm.md).\n@end\n",
            ),
        ),
        (
            "sm.md",
            "---\ntitle: SM\nvariant:\n  deployment: self-managed\n---\n",
        ),
    ]);
    // Every page is published under `site`: no problems.
    assert!(resolve(&p, "index.md", "site").problems.is_empty());
    let cloud = resolve(&p, "index.md", "cloud-only");
    let recorded: Vec<_> = cloud
        .problems
        .iter()
        .map(|p| {
            (
                p.issue.slug.as_str(),
                p.issue.arg("path"),
                p.issue.arg("build"),
            )
        })
        .collect();
    // Only the link in surviving content: the one in the removed arm isn't.
    assert_eq!(
        recorded,
        [("link-page-dropped", Some("sm.md"), Some("cloud-only"))]
    );
    assert_eq!(links(&cloud)[0].target, LinkTarget::Unresolved);
}

#[test]
fn a_link_to_a_heading_the_build_removes_is_recorded() {
    let p = project(&[
        (
            "index.md",
            &page("[Later](keys.md#streaming) and [Intro](keys.md#intro).\n"),
        ),
        (
            "keys.md",
            "---\ntitle: Keys\n---\n\n## Intro\n\n## Streaming\n@available: self-managed preview 3.4\n",
        ),
    ]);
    assert!(resolve(&p, "index.md", "sm-3.4").problems.is_empty());
    let older = resolve(&p, "index.md", "sm-3.3");
    let recorded: Vec<_> = older
        .problems
        .iter()
        .map(|p| {
            (
                p.issue.slug.as_str(),
                p.issue.arg("id"),
                p.issue.arg("path"),
            )
        })
        .collect();
    assert_eq!(
        recorded,
        [("link-id-removed", Some("streaming"), Some("keys.md"))]
    );
    let all = links(&older);
    assert_eq!(all[0].target, LinkTarget::Unresolved);
    assert!(matches!(all[1].target, LinkTarget::Page { .. }));
}

#[test]
fn a_link_to_a_heading_removed_by_a_selection_is_recorded() {
    let p = project(&[
        ("index.md", &page("[SM](keys.md#sm-setup)\n")),
        (
            "keys.md",
            "---\ntitle: Keys\n---\n\n## Setup\n\n@variant {deployment=cloud}:\nCloud.\n@variant {deployment=self-managed}:\n### Self-managed setup\n@id: sm-setup\n\nText.\n@end\n",
        ),
    ]);
    assert!(resolve(&p, "index.md", "site").problems.is_empty());
    let cloud = resolve(&p, "index.md", "cloud-only");
    assert_eq!(cloud.problems.len(), 1);
    assert_eq!(cloud.problems[0].issue.slug.as_str(), "link-id-removed");
}

#[test]
fn links_with_a_missing_target_or_id_are_left_unresolved_without_a_build_problem() {
    // The source index reports these (`link-target-missing`, `link-id-missing`,
    // `link-to-fragment`); the build records nothing more.
    let p = project(&[
        (
            "index.md",
            &page("[a](nope.md) [b](keys.md#missing) [d](_f.md)\n"),
        ),
        ("keys.md", "---\ntitle: Keys\n---\n\n@include: _f.md\n"),
        ("_f.md", "## Frag\n"),
    ]);
    let resolved = resolve(&p, "index.md", "site");
    assert!(resolved.problems.is_empty());
    assert!(
        links(&resolved)
            .iter()
            .all(|l| l.target == LinkTarget::Unresolved)
    );
}

#[test]
fn a_link_to_a_heading_from_an_included_fragment_resolves_on_the_including_page() {
    // SPEC §4.2: the ids of included fragments are the page's own (#90).
    let p = project(&[
        ("index.md", &page("[](keys.md#frag) [x](keys.md#nested)\n")),
        (
            "keys.md",
            "---\ntitle: Keys\n---\n\n## Frag\n@id: own\n\n@include: _f.md\n",
        ),
        ("_f.md", "## Frag\n\n@include: _g.md\n"),
        ("_g.md", "### Nested\n"),
    ]);
    let resolved = resolve(&p, "index.md", "site");
    assert!(resolved.problems.is_empty(), "{:?}", resolved.problems);
    let targets: Vec<_> = links(&resolved)
        .into_iter()
        .map(|l| match l.target {
            LinkTarget::Page { page, id, .. } => (page.to_string(), id),
            other => panic!("unresolved: {other:?}"),
        })
        .collect();
    assert_eq!(
        targets,
        [
            ("keys.md".to_owned(), Some("frag".to_owned())),
            ("keys.md".to_owned(), Some("nested".to_owned())),
        ]
    );
}

#[test]
fn a_link_to_a_fragment_heading_a_build_removes_is_recorded() {
    let p = project(&[
        ("index.md", &page("[SM](keys.md#sm-setup)\n")),
        (
            "keys.md",
            "---\ntitle: Keys\n---\n\n@variant {deployment=cloud}:\nCloud.\n@variant {deployment=self-managed}:\n@include: _sm.md\n@end\n",
        ),
        ("_sm.md", "## Self-managed setup\n@id: sm-setup\n\nText.\n"),
    ]);
    assert!(resolve(&p, "index.md", "site").problems.is_empty());
    let cloud = resolve(&p, "index.md", "cloud-only");
    assert_eq!(cloud.problems.len(), 1);
    assert_eq!(cloud.problems[0].issue.slug.as_str(), "link-id-removed");
}

#[test]
fn a_reference_link_resolves_through_its_definition_and_its_phrases() {
    // `{api}` in a definition is a URL, and a definition can name a page.
    let p = project(&[
        (
            "index.md",
            &page(
                "[api][a] and [keys][k] and [kd].\n\n[a]: {api}streaming\n[k]: keys.md#rotate\n[kd]: keys.md\n",
            ),
        ),
        ("keys.md", "---\ntitle: Keys\n---\n\n## Rotate\n"),
    ]);
    let resolved = resolve(&p, "index.md", "site");
    let all = links(&resolved);
    assert_eq!(all[0].destination, "https://api.quill.dev/v3/streaming");
    assert_eq!(all[0].target, LinkTarget::External);
    assert!(matches!(&all[1].target, LinkTarget::Page { url, .. } if url == "/docs/keys/#rotate"));
    assert!(matches!(&all[2].target, LinkTarget::Page { url, .. } if url == "/docs/keys/"));
}

// -- Assets -----------------------------------------------------------------

#[test]
fn the_assets_of_a_page_are_the_ones_its_surviving_content_references() {
    let p = project(&[
        (
            "index.md",
            &page(
                "![Shared](shared.png)\n\n@variant {deployment=cloud}:\n![Cloud](cloud.png)\n@variant {deployment=self-managed}:\n![SM](sm.png) and [data](data.csv)\n@end\n",
            ),
        ),
        ("shared.png", ""),
        ("cloud.png", ""),
        ("sm.png", ""),
        ("data.csv", "a"),
    ]);
    let names = |build: &str| -> Vec<String> {
        resolve(&p, "index.md", build)
            .assets
            .iter()
            .map(|a| a.path.to_string())
            .collect()
    };
    assert_eq!(
        names("site"),
        ["shared.png", "cloud.png", "sm.png", "data.csv"]
    );
    assert_eq!(names("cloud-only"), ["shared.png", "cloud.png"]);
    let kinds: Vec<RefKind> = resolve(&p, "index.md", "site")
        .assets
        .iter()
        .map(|a| a.kind)
        .collect();
    assert_eq!(
        kinds,
        [
            RefKind::Image,
            RefKind::Image,
            RefKind::Image,
            RefKind::Link
        ]
    );
    // A link to a page is not an asset.
    let with_page = project(&[
        ("index.md", &page("[Keys](keys.md) [Data](data.csv)\n")),
        ("keys.md", "---\ntitle: K\n---\n"),
        ("data.csv", "a"),
    ]);
    assert_eq!(resolve(&with_page, "index.md", "site").assets.len(), 1);
}

#[test]
fn an_asset_in_an_included_fragment_keeps_where_it_is_written() {
    let p = project(&[
        (
            "guides/install.md",
            &page("@include: ../_fragments/pre.md\n"),
        ),
        ("_fragments/pre.md", "![Pipeline](pipeline.png)\n"),
        ("_fragments/pipeline.png", ""),
    ]);
    let resolved = resolve(&p, "guides/install.md", "site");
    let asset = &resolved.assets[0];
    assert_eq!(asset.path.to_string(), "_fragments/pipeline.png");
    assert_eq!(asset.written_in.to_string(), "_fragments/pre.md");
    assert_eq!(asset.via.len(), 1);
}

// -- Glossary ---------------------------------------------------------------

fn glossary_project(body: &str) -> tessera_resolve::Project {
    project(&[
        ("index.md", &page(body)),
        (
            "glossary.md",
            "---\ntitle: Glossary\n---\n\n## API key\n\n## API\n\n## Go\n\n## Build\n\n## Mode\n",
        ),
    ])
}

fn glossed(page: &ResolvedPage) -> Vec<(String, String)> {
    let mut out = Vec::new();
    page.visit(&mut |b| out.extend(b.glossary.iter().map(|g| (g.term.clone(), g.text.clone()))));
    out
}

#[test]
fn the_first_occurrence_of_a_term_is_linked_and_the_longest_term_wins() {
    let p = glossary_project("Copy your API key. The API key rotates. The API answers.\n");
    let resolved = resolve(&p, "index.md", "site");
    assert_eq!(
        glossed(&resolved),
        [
            ("api-key".to_owned(), "API key".to_owned()),
            ("api".to_owned(), "API".to_owned())
        ]
    );
    // The text is unchanged; it's now a link.
    assert_eq!(
        summary(&resolved),
        ["p Copy your API key. The API key rotates. The API answers."]
    );
    let urls: Vec<String> = resolved.blocks[0]
        .glossary
        .iter()
        .map(|g| g.url.clone())
        .collect();
    assert_eq!(urls, ["/docs/glossary/#api-key", "/docs/glossary/#api"]);
}

#[test]
fn matching_is_whole_word_and_case_insensitive_unless_the_term_says_otherwise() {
    let p = glossary_project("An api key, APIs, capital and Go, go and Going.\n");
    let resolved = resolve(&p, "index.md", "site");
    // `api key` matches ignoring case; `APIs` isn't the word `API`; `Go` is
    // case-sensitive, so `go` and `Going` don't match.
    assert_eq!(
        glossed(&resolved),
        [
            ("api-key".to_owned(), "api key".to_owned()),
            ("go".to_owned(), "Go".to_owned())
        ]
    );
}

#[test]
fn a_term_can_set_its_own_match() {
    let p = glossary_project(
        "Every build has a mode. Builds differ by mode. Rotate the API key, then the API key.\n\nSee [a build](glossary.md#build).\n",
    );
    let resolved = resolve(&p, "index.md", "site");
    // `build` is `marked`: never linked automatically, only by the author's
    // link. `mode` is `every`; `API key` keeps the glossary's `first`.
    assert_eq!(
        glossed(&resolved),
        [
            ("mode".to_owned(), "mode".to_owned()),
            ("mode".to_owned(), "mode".to_owned()),
            ("api-key".to_owned(), "API key".to_owned())
        ]
    );
    let ResolvedKind::Leaf(leaf) = &resolved.blocks[1].kind else {
        panic!("a paragraph");
    };
    let BlockKind::Paragraph(par) = &leaf.kind else {
        panic!("a paragraph");
    };
    let InlineKind::Link(link) = &par.inlines[1].kind else {
        panic!("the author's link");
    };
    assert_eq!(link.destination, "/docs/glossary/#build");
}

#[test]
fn glossary_terms_are_linked_in_prose_only() {
    let p = glossary_project(
        "# The API key\n\nSee [the API key](x.md) and `API key` and ![API key](a.png).\n\nAn *API key* in emphasis.\n",
    );
    let resolved = resolve(&p, "index.md", "site");
    // Not in the heading, link text, code, or alt text; in emphasis, yes.
    assert_eq!(
        glossed(&resolved),
        [("api-key".to_owned(), "API key".to_owned())]
    );
}

#[test]
fn a_term_without_a_link_or_on_its_own_page_isnt_linked() {
    let p = glossary_project("An orphan and the API key.\n");
    assert_eq!(
        glossed(&resolve(&p, "index.md", "site")),
        [("api-key".to_owned(), "API key".to_owned())]
    );
    // The glossary page doesn't link to itself.
    assert!(glossed(&resolve(&p, "glossary.md", "site")).is_empty());
}

#[test]
fn a_glossary_link_is_an_ordinary_link_in_the_resolved_inlines() {
    let p = glossary_project("Use the API key.\n");
    let resolved = resolve(&p, "index.md", "site");
    let ResolvedKind::Leaf(leaf) = &resolved.blocks[0].kind else {
        panic!("a paragraph");
    };
    let BlockKind::Paragraph(par) = &leaf.kind else {
        panic!("a paragraph");
    };
    assert_eq!(par.inlines.len(), 3);
    let InlineKind::Link(link) = &par.inlines[1].kind else {
        panic!("a link");
    };
    assert_eq!(link.destination, "/docs/glossary/#api-key");
    assert_eq!(plain(&link.children), "API key");
}

// -- What the tree keeps ----------------------------------------------------

#[test]
fn every_block_keeps_its_file_span_and_the_includes_it_came_through() {
    let p = project(&[
        ("index.md", &page("Own.\n\n@include: _f.md\n")),
        ("_f.md", "Fragment.\n\n@include: _g.md\n"),
        ("_g.md", "Nested.\n"),
    ]);
    let resolved = resolve(&p, "index.md", "site");
    let mut seen: Vec<(String, String, usize)> = Vec::new();
    resolved.visit(&mut |b: &ResolvedBlock| {
        let name = p.path_of(b.file).expect("a file").to_string();
        let index = p.file(&path(&name)).expect("file");
        seen.push((name, index.source[b.span.range()].to_owned(), b.via.len()));
    });
    assert_eq!(
        seen,
        [
            ("index.md".to_owned(), "Own.".to_owned(), 0),
            ("_f.md".to_owned(), "Fragment.".to_owned(), 1),
            ("_g.md".to_owned(), "Nested.".to_owned(), 2),
        ]
    );
}

#[test]
fn a_page_knows_its_route_and_the_build_it_was_resolved_for() {
    let p = project(&[("guides/setup.md", &page("x\n"))]);
    let resolved = resolve(&p, "guides/setup.md", "cloud");
    assert_eq!(resolved.route, "/docs/guides/setup/");
    assert_eq!(resolved.build, "cloud");
    assert_eq!(resolved.title.as_deref(), Some("Test"));
}

#[test]
fn a_build_lists_the_pages_it_drops_and_the_assets_it_copies() {
    let p = project(&[
        ("index.md", &page("![a](a.png)\n")),
        (
            "sm.md",
            "---\ntitle: SM\nvariant:\n  deployment: self-managed\n---\n\n![b](b.png)\n",
        ),
        ("a.png", ""),
        ("b.png", ""),
    ]);
    let build = p.model().build("cloud-only").expect("build");
    let resolved = p.resolve_build(build, &build_support::router(&p));
    assert_eq!(resolved.pages.len(), 1);
    assert_eq!(resolved.dropped.len(), 1);
    assert_eq!(resolved.dropped[0].path.to_string(), "sm.md");
    assert_eq!(
        resolved
            .assets()
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>(),
        ["a.png"]
    );
}
