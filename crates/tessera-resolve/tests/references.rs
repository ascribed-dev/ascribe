//! Links, images, and assets: what each names, from which file, and what's
//! wrong with it (SPEC §5.2, §5.3, §9.4; the asset contract).

#![allow(clippy::expect_used, clippy::panic)]

mod support;

use support::{path, project, slugs};
use tessera_resolve::{Missing, RefKind, Resolution};

const PAGE: &str = "---\ntitle: Test\n---\n\n";

fn asset_paths(project: &tessera_resolve::Project, page: &str) -> Vec<String> {
    project
        .assets(&path(page))
        .iter()
        .map(|a| a.path.to_string())
        .collect()
}

#[test]
fn a_fragments_relative_link_and_image_resolve_from_the_fragments_location() {
    let p = project(&[
        (
            "docs/guides/install.md",
            &format!(
                "{PAGE}@include: ../_fragments/prerequisites.md\n\n![Settings](img/settings.png)\n"
            ),
        ),
        (
            "docs/_fragments/prerequisites.md",
            "![Pipeline](pipeline.png)\n\nSee [the config](../downloads/quill.yaml) and [keys](keys.md).\n",
        ),
        (
            "docs/_fragments/keys.md",
            "---\ntitle: Fragment keys\n---\n",
        ),
        ("docs/_fragments/pipeline.png", ""),
        ("docs/downloads/quill.yaml", "agent: {}\n"),
        ("docs/guides/img/settings.png", ""),
        // The page has neither `pipeline.png` nor `keys.md` beside it.
    ]);
    assert!(p.problems(&path("guides/install.md")).is_empty());
    let fragment_problems = p.problems(&path("_fragments/prerequisites.md"));
    // `keys.md` beside the fragment is a fragment, which a link can't target;
    // it did resolve from the fragment's location.
    assert_eq!(
        slugs(&p, "_fragments/prerequisites.md", &fragment_problems),
        [("link-to-fragment".to_owned(), 3)]
    );
    assert_eq!(
        asset_paths(&p, "guides/install.md"),
        [
            "_fragments/pipeline.png",
            "downloads/quill.yaml",
            "guides/img/settings.png"
        ]
    );
    // Where each reference is written, and how it got to the page.
    let assets = p.assets(&path("guides/install.md"));
    assert_eq!(assets[0].written_in, path("_fragments/prerequisites.md"));
    assert_eq!(assets[0].kind, RefKind::Image);
    assert_eq!(assets[0].via.len(), 1);
    assert_eq!(assets[1].kind, RefKind::Link);
    assert_eq!(assets[2].written_in, path("guides/install.md"));
    assert!(assets[2].via.is_empty());
}

#[test]
fn the_fragment_image_is_not_looked_for_beside_the_page() {
    // The page has no `pipeline.png`, but the fragment does: nothing to report.
    let p = project(&[
        (
            "docs/guides/install.md",
            &format!("{PAGE}@include: ../_fragments/f.md\n"),
        ),
        ("docs/_fragments/f.md", "![Pipeline](pipeline.png)\n"),
        ("docs/_fragments/pipeline.png", ""),
    ]);
    assert!(p.problems(&path("_fragments/f.md")).is_empty());
    assert!(p.problems(&path("guides/install.md")).is_empty());
    assert_eq!(
        asset_paths(&p, "guides/install.md"),
        ["_fragments/pipeline.png"]
    );
}

#[test]
fn an_asset_of_a_section_include_is_only_listed_when_its_section_is_included() {
    let p = project(&[
        ("docs/index.md", &format!("{PAGE}@include: _f.md#kept\n")),
        (
            "docs/_f.md",
            "## Kept\n\n![a](a.png)\n\n## Dropped\n\n![b](b.png)\n",
        ),
        ("docs/a.png", ""),
        ("docs/b.png", ""),
    ]);
    assert_eq!(asset_paths(&p, "index.md"), ["a.png"]);
}

#[test]
fn an_asset_used_twice_on_a_page_is_listed_per_reference() {
    let p = project(&[
        (
            "docs/index.md",
            &format!("{PAGE}![a](a.png) ![a again](a.png)\n"),
        ),
        ("docs/a.png", ""),
    ]);
    assert_eq!(asset_paths(&p, "index.md"), ["a.png", "a.png"]);
    assert_eq!(p.asset_users(&path("a.png")).len(), 2);
}

#[test]
fn every_image_form_names_its_definitions_destination() {
    let p = project(&[
        (
            "docs/index.md",
            &format!(
                "{PAGE}![Inline](a.png) ![Full][r] ![Collapsed][] ![shortcut]\n\n[r]: b.png\n[Collapsed]: c.png\n[shortcut]: d.png\n"
            ),
        ),
        ("docs/a.png", ""),
        ("docs/b.png", ""),
        ("docs/c.png", ""),
    ]);
    assert_eq!(asset_paths(&p, "index.md"), ["a.png", "b.png", "c.png"]);
    let problems = p.problems(&path("index.md"));
    assert_eq!(
        slugs(&p, "index.md", &problems),
        [("image-source-missing".to_owned(), 5)]
    );
}

#[test]
fn a_link_to_a_file_that_is_not_a_page_is_an_asset() {
    let p = project(&[
        (
            "docs/index.md",
            &format!(
                "{PAGE}[Config](downloads/quill.yaml#top) and [Missing](downloads/nope.yaml)\n"
            ),
        ),
        ("docs/downloads/quill.yaml", "x"),
    ]);
    let assets = p.assets(&path("index.md"));
    assert_eq!(assets.len(), 1);
    assert_eq!(assets[0].path, path("downloads/quill.yaml"));
    assert_eq!(assets[0].fragment.as_deref(), Some("top"));
    let problems = p.problems(&path("index.md"));
    assert_eq!(
        slugs(&p, "index.md", &problems),
        [("link-target-missing".to_owned(), 5)]
    );
    assert_eq!(problems[0].arg("path"), Some("downloads/nope.yaml"));
}

#[test]
fn a_percent_encoded_destination_names_the_decoded_file() {
    let p = project(&[
        (
            "docs/index.md",
            &format!("{PAGE}![d](my%20diagram.png) ![e](<my diagram.png>)\n"),
        ),
        ("docs/my diagram.png", ""),
    ]);
    assert!(p.problems(&path("index.md")).is_empty());
    assert_eq!(
        asset_paths(&p, "index.md"),
        ["my diagram.png", "my diagram.png"]
    );
}

#[test]
fn external_destinations_are_never_assets_or_problems() {
    let p = project(&[(
        "docs/index.md",
        &format!(
            "{PAGE}[a](https://example.com/a.pdf) ![b](//cdn.example.com/b.png) <mailto:x@example.com> [c](mailto:x@y.z)\n"
        ),
    )]);
    assert!(p.problems(&path("index.md")).is_empty());
    assert!(p.assets(&path("index.md")).is_empty());
    assert!(
        p.resolutions(&path("index.md"))
            .iter()
            .all(|r| *r == Resolution::External)
    );
}

#[test]
fn references_in_raw_html_are_not_assets() {
    let p = project(&[(
        "docs/index.md",
        &format!("{PAGE}<img src=\"missing.png\">\n\n<div><a href=\"nope.pdf\">x</a></div>\n"),
    )]);
    assert!(p.problems(&path("index.md")).is_empty());
    assert!(p.assets(&path("index.md")).is_empty());
}

#[test]
fn a_destination_names_a_file_with_exact_case() {
    let p = project(&[
        (
            "docs/index.md",
            &format!("{PAGE}![A](P.png) [Keys](Keys.md)\n"),
        ),
        ("docs/p.png", ""),
        ("docs/keys.md", "---\ntitle: K\n---\n"),
    ]);
    let problems = p.problems(&path("index.md"));
    assert_eq!(problems.len(), 2);
    let image = problems
        .iter()
        .find(|i| i.slug.as_str() == "image-source-missing")
        .expect("image");
    assert_eq!(image.variant, Some("case"));
    assert_eq!(image.arg("actual"), Some("p.png"));
    let link = problems
        .iter()
        .find(|i| i.slug.as_str() == "link-target-missing")
        .expect("link");
    assert_eq!(link.variant, Some("case"));
    assert_eq!(link.arg("actual"), Some("keys.md"));
    assert!(p.assets(&path("index.md")).is_empty());
}

#[test]
fn a_file_outside_the_project_or_in_the_output_directory_does_not_exist() {
    let p = project(&[
        (
            "docs/index.md",
            &format!(
                "{PAGE}![a](../../outside.png)\n\n![b](../.tessera/build/site/p.png)\n\n![c](../shared/logo.png)\n\n[d](../../outside.md)\n"
            ),
        ),
        ("outside.png", ""),
        (".tessera/build/site/p.png", ""),
        ("shared/logo.png", ""),
    ]);
    let problems = p.problems(&path("index.md"));
    assert_eq!(
        slugs(&p, "index.md", &problems),
        [
            ("image-source-missing".to_owned(), 5),
            ("image-source-missing".to_owned(), 7),
            ("link-target-missing".to_owned(), 11),
        ]
    );
    assert!(problems.iter().all(|i| i.variant == Some("outside")));
    // A file beside the content root, inside the project, is an asset under a
    // source path starting with `..`.
    assert_eq!(asset_paths(&p, "index.md"), ["../shared/logo.png"]);
}

#[test]
fn a_root_relative_destination_starts_at_the_content_root() {
    let p = project(&[
        (
            "docs/guides/deep/page.md",
            &format!("{PAGE}![a](/img/a.png)\n"),
        ),
        ("docs/img/a.png", ""),
    ]);
    assert!(p.problems(&path("guides/deep/page.md")).is_empty());
    assert_eq!(asset_paths(&p, "guides/deep/page.md"), ["img/a.png"]);
}

#[test]
fn a_link_that_looks_like_a_route_gets_the_route_warning_instead() {
    let p = project(&[
        (
            "docs/index.md",
            &format!(
                "{PAGE}[a](/guides/install/) [b](/guides/install) [c](../guides/install)\n\n[d](guides/setup.md)\n"
            ),
        ),
        ("docs/guides/install.md", "---\ntitle: I\n---\n"),
        ("docs/guides/setup.md", "---\ntitle: S\n---\n"),
    ]);
    let problems = p.problems(&path("index.md"));
    assert_eq!(problems.len(), 3);
    assert!(problems.iter().all(|i| i.slug.as_str() == "link-route"));
    assert_eq!(problems[0].arg("page"), Some("guides/install.md"));
    assert_eq!(problems[0].arg("suggestion"), Some("/guides/install.md"));
    // From `index.md`, `../guides/install` leaves the content root.
    assert_eq!(problems[2].arg("suggestion"), Some("../guides/install.md"));
}

/// The consumer profile's router names the page a route belongs to, where the
/// conventional mapping finds no file (Q148): the base path and the entry ids
/// Astro gives files.
#[test]
fn a_route_names_the_page_the_astro_router_gives_it() {
    use std::sync::Arc;
    use tessera_core::FileId;
    use tessera_resolve::{Layout, MemoryFs, Project};

    let model = tessera_model::load_str(
        "spec = \"0.1\"\n[project]\ncontent-root = \"docs\"\n[consumer]\nbase-path = \"/docs/\"\n",
        FileId::new(0),
    )
    .expect("a model");
    let layout = Layout::from_model(&model);
    let fs = MemoryFs::new(&layout)
        .with_file(
            "docs/index.md",
            &format!(
                "{PAGE}[a](/docs/guides/my-setup/#x) [b](/guides/my-setup) [c](guides/my-setup/) [d](/docs/nope/)\n"
            ),
        )
        .with_file("docs/guides/My Setup.md", "---\ntitle: S\n---\n")
        .with_file("docs/_fragments/my-setup.md", "text\n");
    let p = Project::load(Arc::new(model), layout, &fs);
    let problems = p.problems(&path("index.md"));
    assert_eq!(problems.len(), 4);
    let route = |n: usize| {
        (
            problems[n].arg("page").map(str::to_owned),
            problems[n].arg("suggestion").map(str::to_owned),
        )
    };
    // With and without the base path, and relative to the file.
    for n in 0..3 {
        assert_eq!(route(n).0.as_deref(), Some("guides/My Setup.md"), "{n}");
    }
    assert_eq!(route(0).1.as_deref(), Some("/guides/My Setup.md#x"));
    assert_eq!(route(2).1.as_deref(), Some("guides/My Setup.md"));
    // No page has that route: the conventional guess, with no fix to offer.
    assert_eq!(route(3).0.as_deref(), Some("docs/nope.md"));
}

#[test]
fn an_extensionless_file_that_exists_is_an_asset_not_a_route() {
    let p = project(&[
        ("docs/index.md", &format!("{PAGE}[License](LICENSE)\n")),
        ("docs/LICENSE", "text"),
    ]);
    assert!(p.problems(&path("index.md")).is_empty());
    assert_eq!(asset_paths(&p, "index.md"), ["LICENSE"]);
}

#[test]
fn links_to_pages_and_ids_are_checked_against_the_targets_own_source_ids() {
    let p = project(&[
        (
            "docs/index.md",
            &format!(
                "{PAGE}[ok](keys.md#rotate-keys) [explicit](keys.md#other-id) [slug](keys.md#other) [none](keys.md#nope)\n\n[frag](keys.md#from-fragment) [self](#top)\n\n## Top\n"
            ),
        ),
        (
            "docs/keys.md",
            &format!("{PAGE}## Rotate keys\n\n## Other\n@id: other-id\n\n@include: _f.md\n"),
        ),
        ("docs/_f.md", "## From fragment\n"),
    ]);
    let problems = p.problems(&path("index.md"));
    let got: Vec<(&str, &str)> = problems
        .iter()
        .map(|i| (i.slug.as_str(), i.arg("id").unwrap_or("")))
        .collect();
    assert_eq!(
        got,
        [
            // `other` was replaced by the `@id`.
            ("link-id-missing", "other"),
            ("link-id-missing", "nope"),
            // Q6: an id that exists only in an included fragment.
            ("link-id-in-fragment", "from-fragment"),
        ]
    );
    let in_fragment = &problems[2];
    assert_eq!(in_fragment.arg("fragment"), Some("_f.md"));
    assert_eq!(in_fragment.arg("path"), Some("keys.md"));
}

#[test]
fn a_link_to_a_fragment_file_is_an_error() {
    let p = project(&[
        (
            "docs/index.md",
            &format!("{PAGE}[Frag](_frag.md) and [Frag id](_frag.md#x)\n"),
        ),
        ("docs/_frag.md", "## X\n"),
    ]);
    let problems = p.problems(&path("index.md"));
    assert_eq!(problems.len(), 2);
    assert!(
        problems
            .iter()
            .all(|i| i.slug.as_str() == "link-to-fragment")
    );
}

#[test]
fn a_link_to_a_missing_page_is_reported() {
    let p = project(&[("docs/index.md", &format!("{PAGE}[x](nope.md)\n"))]);
    let problems = p.problems(&path("index.md"));
    assert_eq!(problems.len(), 1);
    assert_eq!(problems[0].slug.as_str(), "link-target-missing");
    assert_eq!(
        p.resolutions(&path("index.md")),
        [Resolution::SourceMissing { actual: None }]
    );
}

#[test]
fn a_phrase_in_an_inline_destination_is_applied_before_it_is_resolved() {
    let p = project(&[
        (
            "docs/index.md",
            &format!("{PAGE}[api]({{api}}streaming) ![i]({{product}}.png)\n"),
        ),
        ("docs/Quill.png", ""),
    ]);
    // `{api}` is a URL, so the first is external; `{product}.png` is a file.
    assert!(p.problems(&path("index.md")).is_empty());
    assert_eq!(asset_paths(&p, "index.md"), ["Quill.png"]);
}

#[test]
fn a_phrase_in_a_definitions_destination_is_applied_like_an_inline_ones() {
    // Q43: `{api}` in a definition is a URL, as it is inline.
    let p = project(&[(
        "docs/index.md",
        &format!(
            "{PAGE}[api][r] and [r] and [also][]\n\n[r]: {{api}}streaming\n[also]: {{api}}more\n"
        ),
    )]);
    assert_eq!(
        p.resolutions(&path("index.md")),
        [
            Resolution::External,
            Resolution::External,
            Resolution::External
        ]
    );
    assert!(p.problems(&path("index.md")).is_empty());
}

#[test]
fn a_definitions_phrase_can_name_a_file() {
    let p = project(&[
        (
            "docs/index.md",
            &format!("{PAGE}![Logo][logo] and [the logo][logo]\n\n[logo]: {{product}}.png\n"),
        ),
        ("docs/Quill.png", ""),
    ]);
    assert!(p.problems(&path("index.md")).is_empty());
    assert_eq!(asset_paths(&p, "index.md"), ["Quill.png", "Quill.png"]);
    // The candidates are indexed once, where the definition is.
    let file = p.file(&path("index.md")).expect("the file");
    assert_eq!(
        file.phrases
            .iter()
            .filter(|u| u.phrase.key == "product")
            .count(),
        1
    );
}

#[test]
fn a_definitions_phrase_that_makes_a_missing_file_is_reported() {
    let p = project(&[(
        "docs/index.md",
        &format!("{PAGE}![Logo][logo]\n\n[logo]: {{product}}.png\n"),
    )]);
    let problems = p.problems(&path("index.md"));
    assert_eq!(problems.len(), 1);
    assert_eq!(problems[0].slug.as_str(), "image-source-missing");
    assert_eq!(problems[0].arg("path"), Some("Quill.png"));
}

#[test]
fn a_reference_finds_its_definition_by_normalized_label() {
    // Two definitions with different phrases; each reference uses its own,
    // whatever the case and spacing of its label.
    let p = project(&[(
        "docs/index.md",
        &format!(
            "{PAGE}[x][My  Label] and [y][other]\n\n[my label]: {{api}}one\n[other]: {{product}}.md\n"
        ),
    )]);
    let resolutions = p.resolutions(&path("index.md"));
    assert_eq!(resolutions[0], Resolution::External);
    // `Quill.md` isn't a source file of the project.
    assert!(matches!(resolutions[1], Resolution::SourceMissing { .. }));
}

#[test]
fn reverse_edges_say_who_points_here() {
    let p = project(&[
        (
            "docs/index.md",
            &format!("{PAGE}@include: _f.md\n\n[k](keys.md#rotate-keys) ![i](a.png)\n"),
        ),
        (
            "docs/other.md",
            &format!("{PAGE}[k](keys.md) [k2](keys.md#rotate-keys) ![i](a.png)\n"),
        ),
        ("docs/_f.md", "[k](keys.md#other)\n\n@include: _g.md\n"),
        ("docs/_g.md", "G.\n"),
        (
            "docs/keys.md",
            &format!("{PAGE}## Rotate keys\n\n## Other\n"),
        ),
        ("docs/a.png", ""),
    ]);
    let files = |sites: &[tessera_resolve::LinkSite]| -> Vec<String> {
        sites.iter().map(|s| s.file.to_string()).collect()
    };
    assert_eq!(
        files(p.links_to(&path("keys.md"))),
        ["_f.md", "index.md", "other.md", "other.md"]
    );
    let to_rotate: Vec<String> = p
        .links_to_id(&path("keys.md"), "rotate-keys")
        .map(|s| s.file.to_string())
        .collect();
    assert_eq!(to_rotate, ["index.md", "other.md"]);
    let users: Vec<String> = p
        .asset_users(&path("a.png"))
        .iter()
        .map(|s| s.file.to_string())
        .collect();
    assert_eq!(users, ["index.md", "other.md"]);

    let includers: Vec<String> = p
        .includers(&path("_g.md"))
        .iter()
        .map(|e| e.file.to_string())
        .collect();
    assert_eq!(includers, ["_f.md"]);
    // Through `_f.md`, the page `index.md` includes `_g.md`.
    assert_eq!(p.including_pages(&path("_g.md")), [path("index.md")]);
    assert_eq!(p.including_pages(&path("_f.md")), [path("index.md")]);
    assert!(p.including_pages(&path("other.md")).is_empty());
}

#[test]
fn missing_files_are_not_edges() {
    let p = project(&[(
        "docs/index.md",
        &format!("{PAGE}[x](nope.md) ![y](nope.png) @include-ish\n\n@include: _nope.md\n"),
    )]);
    assert!(p.links_to(&path("nope.md")).is_empty());
    assert!(p.asset_users(&path("nope.png")).is_empty());
    assert!(p.includers(&path("_nope.md")).is_empty());
}

#[test]
fn references_in_titles_and_notes_count() {
    let p = project(&[(
        "docs/index.md",
        &format!(
            "{PAGE}.See [the guide](missing.md)\n@details:\n\n@note: A [note link](gone.pdf).\n\n@end\n"
        ),
    )]);
    let problems = p.problems(&path("index.md"));
    let got: Vec<&str> = problems.iter().map(|i| i.slug.as_str()).collect();
    assert_eq!(got, ["link-target-missing", "link-target-missing"]);
}

#[test]
fn missing_reasons_are_told_apart() {
    let p = project(&[("docs/index.md", &format!("{PAGE}![a](x.png)\n"))]);
    assert_eq!(
        p.resolutions(&path("index.md")),
        [Resolution::AssetMissing(Missing::Absent)]
    );
}

#[test]
fn an_image_with_no_path_is_missing_but_a_fragment_only_link_is_not() {
    // Resolved Q59, as phase 10 implements it.
    let p = project(&[(
        "docs/index.md",
        &format!("{PAGE}![A]()\n\n![B](#top)\n\n[Self](#top)\n\n## Top\n"),
    )]);
    let problems = p.problems(&path("index.md"));
    assert_eq!(
        slugs(&p, "index.md", &problems),
        [
            ("image-source-missing".to_owned(), 5),
            ("image-source-missing".to_owned(), 7)
        ]
    );
    assert_eq!(problems[0].arg("path"), Some("(no source)"));
    assert_eq!(problems[1].arg("path"), Some("#top"));
}
