//! Include expansion (SPEC §4.2).

#![allow(clippy::expect_used, clippy::panic)]

mod support;

use support::{path, project, slugs};
use tessera_resolve::{ExpandedBlock, ExpandedKind, ExpandedPage, Project};
use tessera_syntax::BlockKind;

const PAGE: &str = "---\ntitle: Test\n---\n\n";

/// The text of every leaf paragraph and heading of the page, in order, read
/// from the file each was written in.
fn texts(project: &Project, page: &ExpandedPage) -> Vec<String> {
    let mut out = Vec::new();
    page.visit(&mut |block: &ExpandedBlock| {
        if let ExpandedKind::Leaf(leaf) = &block.kind
            && matches!(leaf.kind, BlockKind::Paragraph(_) | BlockKind::Heading(_))
        {
            let index = project.file_by_id(block.file).expect("known file");
            out.push(index.source[block.span.range()].to_owned());
        }
    });
    out
}

fn expand(project: &Project, page: &str) -> ExpandedPage {
    project.expand(&path(page)).expect("the page exists")
}

#[test]
fn an_include_is_replaced_by_the_fragments_content() {
    let p = project(&[
        (
            "docs/index.md",
            &format!("{PAGE}# Page\n\nBefore.\n\n@include: _f.md\n\nAfter.\n"),
        ),
        ("docs/_f.md", "The {product} fragment.\n"),
    ]);
    let page = expand(&p, "index.md");
    assert!(page.problems.is_empty());
    assert_eq!(
        texts(&p, &page),
        ["# Page", "Before.", "The {product} fragment.", "After."]
    );
}

#[test]
fn every_block_keeps_its_file_and_the_includes_it_came_through() {
    let p = project(&[
        ("docs/index.md", &format!("{PAGE}@include: _a.md\n")),
        ("docs/_a.md", "A.\n\n@include: _b.md\n"),
        ("docs/_b.md", "B.\n"),
    ]);
    let page = expand(&p, "index.md");
    let a = p.file(&path("_a.md")).expect("_a.md").file;
    let b = p.file(&path("_b.md")).expect("_b.md").file;
    let index = p.file(&path("index.md")).expect("index.md").file;
    let mut seen = Vec::new();
    page.visit(&mut |block: &ExpandedBlock| {
        seen.push((
            block.file,
            block.via.iter().map(|s| s.file).collect::<Vec<_>>(),
        ));
    });
    assert_eq!(seen, [(a, vec![index]), (b, vec![index, a])]);
}

#[test]
fn nested_include_paths_resolve_from_the_file_they_are_written_in() {
    let p = project(&[
        (
            "docs/guides/a.md",
            &format!("{PAGE}@include: _shared/one.md\n"),
        ),
        ("docs/guides/_shared/one.md", "One.\n\n@include: two.md\n"),
        ("docs/guides/_shared/two.md", "Two.\n"),
        ("docs/two.md", "Wrong.\n"),
    ]);
    let page = expand(&p, "guides/a.md");
    assert!(page.problems.is_empty());
    assert_eq!(texts(&p, &page), ["One.", "Two."]);
}

#[test]
fn a_path_starting_with_a_slash_is_relative_to_the_content_root() {
    let p = project(&[
        (
            "docs/guides/deep/page.md",
            &format!("{PAGE}@include: /_shared/x.md\n"),
        ),
        ("docs/_shared/x.md", "From the root.\n"),
    ]);
    assert_eq!(
        texts(&p, &expand(&p, "guides/deep/page.md")),
        ["From the root."]
    );
}

#[test]
fn a_bare_filename_is_not_searched_for() {
    let p = project(&[
        ("docs/guides/page.md", &format!("{PAGE}@include: _f.md\n")),
        ("docs/_f.md", "Elsewhere.\n"),
    ]);
    let problems = p.problems(&path("guides/page.md"));
    assert_eq!(
        slugs(&p, "guides/page.md", &problems),
        [("include-target-missing".to_owned(), 5)]
    );
    // The directive stays, so the page still shows what the author wrote.
    let page = expand(&p, "guides/page.md");
    assert!(page.problems.is_empty());
    assert!(matches!(
        &page.blocks[0].kind,
        ExpandedKind::Leaf(b) if matches!(b.kind, BlockKind::Directive(_))
    ));
}

#[test]
fn an_include_of_a_file_whose_name_differs_in_case_says_so() {
    let p = project(&[
        ("docs/index.md", &format!("{PAGE}@include: _F.md\n")),
        ("docs/_f.md", "x\n"),
    ]);
    let problems = p.problems(&path("index.md"));
    assert_eq!(problems.len(), 1);
    assert_eq!(problems[0].variant, Some("case"));
    assert_eq!(problems[0].arg("actual"), Some("_f.md"));
}

#[test]
fn a_section_is_selected_by_source_id() {
    let p = project(&[
        (
            "docs/index.md",
            &format!("{PAGE}# Page\n\n@include: _f.md#install\n"),
        ),
        (
            "docs/_f.md",
            "## Prerequisites\n\nPre.\n\n## Install\n\nInstall text.\n\n### Detail\n\nDetail text.\n\n## Next\n\nNext text.\n",
        ),
    ]);
    let page = expand(&p, "index.md");
    assert!(page.problems.is_empty());
    assert_eq!(
        texts(&p, &page),
        [
            "# Page",
            "## Install",
            "Install text.",
            "### Detail",
            "Detail text."
        ]
    );
}

#[test]
fn a_section_is_selected_by_explicit_id_and_the_slug_no_longer_names_it() {
    let files = |include: &str| {
        [
            (
                "docs/index.md".to_owned(),
                format!("{PAGE}# Page\n\n@include: _f.md#{include}\n"),
            ),
            (
                "docs/_f.md".to_owned(),
                "## A title\n@id: custom-id\n\nBody text.\n\n## Other\n\nElsewhere.\n".to_owned(),
            ),
        ]
    };
    let build = |include: &str| {
        let owned = files(include);
        let refs: Vec<(&str, &str)> = owned
            .iter()
            .map(|(a, b)| (a.as_str(), b.as_str()))
            .collect();
        project(&refs)
    };
    let p = build("custom-id");
    let page = expand(&p, "index.md");
    assert!(page.problems.is_empty());
    assert_eq!(texts(&p, &page), ["# Page", "## A title", "Body text."]);

    let p = build("a-title");
    let page = expand(&p, "index.md");
    assert_eq!(page.problems.len(), 1);
    assert_eq!(page.problems[0].issue.slug.as_str(), "include-id-missing");
    assert_eq!(page.problems[0].issue.arg("id"), Some("a-title"));
}

#[test]
fn heading_false_drops_the_included_sections_heading_and_keeps_its_directives() {
    let p = project(&[
        (
            "docs/index.md",
            &format!("{PAGE}# Page\n\n@include {{heading=false}}: _f.md#custom-id\n"),
        ),
        (
            "docs/_f.md",
            "## A title\n@id: custom-id\n\nBody text.\n\n## Other\n\nElsewhere.\n",
        ),
    ]);
    let page = expand(&p, "index.md");
    assert!(page.problems.is_empty());
    assert_eq!(texts(&p, &page), ["# Page", "Body text."]);
    // The `@id` directive stays.
    let mut directives = 0;
    page.visit(&mut |b: &ExpandedBlock| {
        if let ExpandedKind::Leaf(leaf) = &b.kind
            && let BlockKind::Directive(d) = &leaf.kind
            && d.name == "id"
        {
            directives += 1;
        }
    });
    assert_eq!(directives, 1);
}

#[test]
fn heading_true_keeps_the_heading() {
    let p = project(&[
        (
            "docs/index.md",
            &format!("{PAGE}@include {{heading=true}}: _f.md#part\n"),
        ),
        ("docs/_f.md", "## Part\n\nBody.\n"),
    ]);
    assert_eq!(texts(&p, &expand(&p, "index.md")), ["## Part", "Body."]);
}

#[test]
fn heading_false_without_an_id_drops_nothing() {
    let p = project(&[
        (
            "docs/index.md",
            &format!("{PAGE}@include {{heading=false}}: _f.md\n"),
        ),
        ("docs/_f.md", "## Part\n\nBody.\n"),
    ]);
    assert_eq!(texts(&p, &expand(&p, "index.md")), ["## Part", "Body."]);
}

#[test]
fn included_headings_keep_their_levels() {
    let p = project(&[
        (
            "docs/index.md",
            &format!("{PAGE}#### Deep\n\n@include: _f.md#part\n"),
        ),
        ("docs/_f.md", "## Part\n\nBody.\n"),
    ]);
    let page = expand(&p, "index.md");
    let mut levels = Vec::new();
    page.visit(&mut |b: &ExpandedBlock| {
        if let ExpandedKind::Leaf(leaf) = &b.kind
            && let BlockKind::Heading(h) = &leaf.kind
        {
            levels.push(h.level);
        }
    });
    assert_eq!(levels, [4, 2]);
}

#[test]
fn a_section_inside_a_container_ends_with_the_container() {
    let p = project(&[
        ("docs/index.md", &format!("{PAGE}@include: _f.md#inner\n")),
        (
            "docs/_f.md",
            "@details:\n## Inner\n\nIn.\n\n@end\n\n## Inner two\n\nOut.\n",
        ),
    ]);
    // `@details` is a container: the heading's section stops at `@end`.
    let page = expand(&p, "index.md");
    assert!(page.problems.is_empty(), "{:?}", page.problems);
    assert_eq!(texts(&p, &page), ["## Inner", "In."]);
}

#[test]
fn includes_inside_lists_and_containers_are_expanded_in_place() {
    let p = project(&[
        (
            "docs/index.md",
            &format!(
                "{PAGE}- item\n\n  @include: _f.md\n\n.Title\n@details:\n@include: _g.md\n@end\n"
            ),
        ),
        ("docs/_f.md", "In the list.\n"),
        ("docs/_g.md", "In the details.\n"),
    ]);
    let page = expand(&p, "index.md");
    assert!(page.problems.is_empty(), "{:?}", page.problems);
    assert_eq!(
        texts(&p, &page),
        ["item", "In the list.", "In the details."]
    );
}

#[test]
fn an_include_cycle_is_reported_where_it_closes() {
    let p = project(&[
        ("docs/index.md", &format!("{PAGE}@include: _a.md\n")),
        ("docs/_a.md", "A.\n\n@include: _b.md\n"),
        ("docs/_b.md", "B.\n\n@include: _a.md\n"),
    ]);
    let page = expand(&p, "index.md");
    assert_eq!(page.problems.len(), 1);
    let problem = &page.problems[0];
    assert_eq!(problem.issue.slug.as_str(), "include-cycle");
    assert_eq!(p.path_of(problem.issue.location.file), Some(&path("_b.md")));
    assert_eq!(
        problem.issue.arg("cycle"),
        Some("`_a.md` → `_b.md` → `_a.md`")
    );
    // Reported at the include site in the page, via the chain.
    assert_eq!(problem.via.len(), 2);
    assert_eq!(texts(&p, &page), ["A.", "B."]);
}

#[test]
fn a_file_that_includes_itself_is_a_cycle() {
    let p = project(&[
        ("docs/index.md", &format!("{PAGE}@include: _self.md\n")),
        ("docs/_self.md", "Text.\n\n@include: _self.md\n"),
    ]);
    let page = expand(&p, "index.md");
    assert_eq!(page.problems.len(), 1);
    assert_eq!(page.problems[0].issue.slug.as_str(), "include-cycle");
    assert_eq!(
        p.path_of(page.problems[0].issue.location.file),
        Some(&path("_self.md"))
    );
}

#[test]
fn a_page_that_includes_itself_is_a_cycle() {
    let p = project(&[(
        "docs/index.md",
        &format!("{PAGE}Text.\n\n@include: index.md\n"),
    )]);
    let page = expand(&p, "index.md");
    assert_eq!(page.problems.len(), 1);
    assert_eq!(page.problems[0].issue.slug.as_str(), "include-cycle");
}

#[test]
fn a_cycle_among_fragments_no_page_includes_is_found_when_the_fragment_is_expanded() {
    let p = project(&[
        ("docs/index.md", &format!("{PAGE}Text.\n")),
        ("docs/_a.md", "@include: _b.md\n"),
        ("docs/_b.md", "@include: _a.md\n"),
    ]);
    assert_eq!(expand(&p, "_a.md").problems.len(), 1);
    assert_eq!(expand(&p, "_b.md").problems.len(), 1);
}

#[test]
fn an_include_of_a_section_of_the_including_file_is_not_a_cycle() {
    // A cycle is expanding the same file, or the same
    // section, again while it's still being expanded.
    let p = project(&[("docs/_a.md", "## Y\n\nWhy.\n\n## X\n\n@include: _a.md#y\n")]);
    let page = expand(&p, "_a.md");
    assert!(page.problems.is_empty(), "{:?}", page.problems);
    assert_eq!(texts(&p, &page), ["## Y", "Why.", "## X", "## Y", "Why."]);
}

#[test]
fn a_section_that_includes_itself_is_a_cycle() {
    let p = project(&[("docs/_a.md", "## X\n\n@include: _a.md#x\n")]);
    let page = expand(&p, "_a.md");
    assert_eq!(page.problems.len(), 1);
    assert_eq!(page.problems[0].issue.slug.as_str(), "include-cycle");
}

#[test]
fn the_same_fragment_included_twice_is_expanded_twice() {
    let p = project(&[
        (
            "docs/index.md",
            &format!("{PAGE}@include: _f.md\n\n@include: _f.md\n"),
        ),
        ("docs/_f.md", "## Shared\n@id: shared\n\nText.\n"),
    ]);
    let page = expand(&p, "index.md");
    // Not a cycle, and not expansion's to call a duplicate: id uniqueness is
    // a page-level check.
    assert!(page.problems.is_empty());
    assert_eq!(
        texts(&p, &page),
        ["## Shared", "Text.", "## Shared", "Text."]
    );
    // The two copies came through different include sites.
    let mut sites = Vec::new();
    page.visit(&mut |b: &ExpandedBlock| {
        if let Some(site) = b.via.first() {
            sites.push(site.span);
        }
    });
    sites.dedup();
    assert_eq!(sites.len(), 2);
    assert_eq!(p.includers(&path("_f.md")).len(), 2);
}

#[test]
fn a_page_id_of_an_included_section_is_still_its_source_id_in_the_fragment() {
    let p = project(&[
        (
            "docs/index.md",
            &format!("{PAGE}## Setup\n\n@include: _f.md\n"),
        ),
        ("docs/_f.md", "## Setup\n\nAgain.\n"),
    ]);
    let page = expand(&p, "index.md");
    let headings = page.headings(&p);
    assert_eq!(headings.len(), 2);
    // Each is `setup` in its own file: page ids come from build resolution.
    assert!(headings.iter().all(|(_, h)| h.source_id == "setup"));
}

/// A source file is read where its links lead, and a link out of the content
/// root makes it unreadable: no file from elsewhere on disk reaches a page,
/// by `@include` or as a page of its own. A link that stays inside is read.
#[cfg(unix)]
#[test]
fn a_link_out_of_the_content_root_is_not_read() {
    use std::fs;
    use std::os::unix::fs::symlink;
    use tessera_resolve::{DiskFs, Layout};

    let dir = tempfile::tempdir().expect("a temp dir");
    let root = dir.path();
    fs::create_dir_all(root.join("project/docs/inside")).expect("docs");
    fs::create_dir_all(root.join("elsewhere")).expect("elsewhere");
    fs::write(root.join("secret.md"), "The secret.\n").expect("secret");
    fs::write(root.join("elsewhere/far.md"), "Far away.\n").expect("far");
    fs::write(root.join("project/docs/inside/_near.md"), "Nearby.\n").expect("near");
    symlink("../../secret.md", root.join("project/docs/_out.md")).expect("out");
    symlink("../../elsewhere", root.join("project/docs/linked")).expect("linked");
    symlink("inside/_near.md", root.join("project/docs/_alias.md")).expect("alias");
    fs::write(
        root.join("project/docs/index.md"),
        format!(
            "{PAGE}# Page\n\n@include: _alias.md\n\n@include: _out.md\n\n@include: linked/far.md\n"
        ),
    )
    .expect("the page");

    let model = support::model();
    let layout = Layout::from_model(&model);
    let fs = DiskFs::new(root.join("project"), &layout);
    let p = Project::load(model, layout, &fs);
    let mut unreadable: Vec<(String, bool)> = p
        .unreadable()
        .iter()
        .map(|u| {
            (
                u.path.to_string(),
                u.reason.contains("leads out of the content root"),
            )
        })
        .collect();
    unreadable.sort();
    assert_eq!(
        unreadable,
        [
            ("_out.md".to_owned(), true),
            ("linked/far.md".to_owned(), true)
        ]
    );
    let page = expand(&p, "index.md");
    assert_eq!(texts(&p, &page), ["# Page", "Nearby."]);
    let issues = p.problems(&path("index.md"));
    assert_eq!(
        slugs(&p, "index.md", &issues),
        [
            ("include-target-missing".to_owned(), 9),
            ("include-target-missing".to_owned(), 11)
        ]
    );
}
