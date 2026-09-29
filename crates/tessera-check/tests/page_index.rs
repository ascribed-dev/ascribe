//! `PageChecker::with_index` and `check_resolved`: checking resolved pages over
//! an index the caller already has reports what `check` reports for them.

#![allow(clippy::expect_used, clippy::panic)]

use std::path::PathBuf;
use std::sync::Arc;

use tessera_check::{PageChecker, Project};
use tessera_core::{FileId, RelPath};
use tessera_resolve::{DefaultRouter, Layout, MemoryFs};

const MODEL: &str = "spec = \"0.1\"\n[project]\ncontent-root = \"docs\"\n[dimensions.pm]\nvalues = [\"npm\", \"pnpm\"]\n[builds.site]\nvariants = \"switch\"\navailability = \"badge\"\n";

const FILES: [(&str, &str); 6] = [
    (
        "a.md",
        "---\ntitle: A\n---\n# One\n@id: same\n\n# Two\n@id: same\n\n@include: _f.md#missing\n[x](b.md#nope)\n",
    ),
    ("b.md", "---\ntitle: B\n---\n# B\n\n@include: _cycle.md\n"),
    ("c.md", "---\ntitle: C\n---\n@include: _cycle.md\n"),
    ("_cycle.md", "## Loop\n@include: _cycle.md\n"),
    ("_f.md", "## Here\n@id: here\n"),
    ("d.md", "---\ntitle: D\n---\n[fine](a.md)\n"),
];

#[test]
fn checking_every_resolved_page_over_a_shared_index_equals_check() {
    let model = tessera_model::load_str(MODEL, FileId::new(0)).expect("a model");
    let layout = Layout::from_model(&model);
    let mut fs = MemoryFs::new(&layout);
    for (path, text) in FILES {
        fs = fs.with_source(path, text);
    }
    let index = tessera_resolve::Project::load(Arc::new(model.clone()), layout, &fs);
    let sources = Project::from_sources(
        FILES
            .iter()
            .map(|(p, t)| (RelPath::parse(p).expect("a path"), (*t).to_owned())),
    );
    let project = Project::from_parts(
        PathBuf::from("/nowhere"),
        RelPath::parse("docs").expect("a path"),
        model.clone(),
        MODEL.to_owned(),
        sources,
    );
    let build = model.editor_default_build().clone();
    let expected = PageChecker::new(&project).check(&build);
    assert!(
        expected.len() >= 4,
        "the fixture has problems: {expected:#?}"
    );

    let router = DefaultRouter::from_consumer(&model.consumer);
    let resolved: Vec<_> = index
        .pages()
        .filter_map(|p| index.resolve_page(&p.path, &build, &router))
        .collect();
    let refs: Vec<&_> = resolved.iter().collect();
    let shared = PageChecker::with_index(&project, &index).check_resolved(&build, &refs);
    assert_eq!(shared, expected);

    // A subset gives the diagnostics of those pages, and a cycle in a fragment
    // comes from either page that includes it.
    let only_b: Vec<_> = resolved
        .iter()
        .filter(|p| p.path.as_str() == "b.md")
        .collect();
    let from_b = PageChecker::with_index(&project, &index).check_resolved(&build, &only_b);
    assert!(from_b.iter().any(|d| d.slug.as_str() == "include-cycle"));
    assert!(from_b.iter().all(|d| d.slug.as_str() != "id-duplicate"));
}
