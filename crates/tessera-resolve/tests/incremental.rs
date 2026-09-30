//! Incremental updates: what each kind of change invalidates, what is
//! cached, and how snapshots and file ids behave.

#![allow(clippy::expect_used, clippy::panic, clippy::unwrap_used)]

mod incremental_support;

use std::collections::BTreeSet;
use std::sync::Arc;

use tessera_core::{FileId, RelPath};
use tessera_resolve::{
    ApplyError, Change, DefaultRouter, FileKind, IncrementalProject, Layout, MemoryFs, ModelImpact,
    Project, ResolvedCache,
};
use tessera_syntax::BlockKind;

use incremental_support::{ModelSpec, created, deleted, edited, load, path, router};

fn set(paths: &[&str]) -> BTreeSet<RelPath> {
    paths.iter().map(|p| path(p)).collect()
}

fn resolved(project: &Project, page: &str, build: &str) -> String {
    let build = project.model().build(build).expect("a build");
    format!(
        "{:?}",
        project
            .resolve_page(&path(page), build, &router(project))
            .expect("a published page")
    )
}

// -- Acceptance: what a change reaches -------------------------------------------

#[test]
fn editing_a_fragment_re_resolves_the_pages_that_include_it_and_no_others() {
    let (mut inc, _) = load(
        &ModelSpec::base(),
        &[
            ("a.md", "---\ntitle: A\n---\n\n@include: _f.md\n"),
            ("b.md", "---\ntitle: B\n---\n\n@include: _mid.md\n"),
            ("_mid.md", "@include: _f.md\n"),
            ("c.md", "---\ntitle: C\n---\n\nNothing shared.\n"),
            (
                "d.md",
                "---\ntitle: D\n---\n\n@include: _other.md\n\n[to a](a.md)\n",
            ),
            ("_other.md", "Other.\n"),
            ("_f.md", "Some prose.\n"),
        ],
    );
    let before = inc.snapshot();
    let affected = inc.apply([edited("_f.md", "Different prose.\n")]).unwrap();
    assert_eq!(affected.re_resolve, set(&["a.md", "b.md"]));
    // Only the fragment was reparsed.
    assert_eq!(affected.parsed, set(&["_f.md"]));
    assert_eq!(affected.indexed, set(&["_f.md"]));
    // The prose isn't something a link can see, so the page that links to `a`
    // isn't touched, and doesn't need another look.
    assert!(!affected.re_resolve.contains(&path("d.md")));
    assert!(!affected.recheck.contains(&path("d.md")));
    assert!(inc.is_file_current(&before, &path("d.md")));
    assert!(!inc.is_file_current(&before, &path("_f.md")));
}

#[test]
fn a_change_to_what_a_link_can_see_reaches_the_pages_that_link_to_it() {
    let (mut inc, _) = load(
        &ModelSpec::base(),
        &[
            ("a.md", "---\ntitle: A\n---\n\n@include: _f.md\n"),
            ("d.md", "---\ntitle: D\n---\n\n[to a](a.md#one)\n"),
            ("e.md", "---\ntitle: E\n---\n\nNo links.\n"),
            ("_f.md", "## One\n\nBody.\n"),
        ],
    );
    let affected = inc.apply([edited("_f.md", "## Uno\n\nBody.\n")]).unwrap();
    // `a` shows a different heading, so `d`'s link to `#one` now has no target.
    assert_eq!(affected.re_resolve, set(&["a.md", "d.md"]));
    assert!(affected.recheck.contains(&path("d.md")));
    assert!(!affected.re_resolve.contains(&path("e.md")));
    let problems = inc.snapshot().problems(&path("d.md"));
    assert!(
        problems
            .iter()
            .any(|p| p.slug.as_str() == "link-id-missing"),
        "{problems:?}"
    );
}

#[test]
fn declaring_a_new_widget_changes_the_parse_of_a_fragment_no_one_has_open() {
    let without = ModelSpec::base();
    let with = ModelSpec {
        widgets: 1,
        ..ModelSpec::base()
    };
    let (mut inc, _) = load(
        &without,
        &[
            ("index.md", "---\ntitle: Home\n---\n\n@include: _note.md\n"),
            ("_note.md", "@quill-callout: Heads up\n"),
            ("other.md", "---\ntitle: Other\n---\n\nPlain.\n"),
        ],
    );
    let kind_of = |snapshot: &Project| {
        let file = snapshot.file(&path("_note.md")).expect("the fragment");
        matches!(file.document.blocks[0].kind, BlockKind::Directive(_))
    };
    assert!(
        !kind_of(&inc.snapshot()),
        "an unknown keyword isn't a directive"
    );

    let stats = inc.stats();
    let affected = inc.apply([Change::Model(with.model())]).unwrap();
    assert_eq!(affected.model, Some(ModelImpact::Keywords));
    // Every file was reparsed, though none was edited.
    assert_eq!(affected.parsed, set(&["index.md", "_note.md", "other.md"]));
    assert_eq!(inc.stats().parses - stats.parses, 3);
    assert!(kind_of(&inc.snapshot()), "now it is a directive");
    // And every file is re-checked, every page re-resolved.
    assert_eq!(affected.recheck, set(&["index.md", "_note.md", "other.md"]));
    assert_eq!(affected.re_resolve, set(&["index.md", "other.md"]));
    // The expansion of the including page shows the new directive.
    let page = inc.snapshot().expand(&path("index.md")).expect("a page");
    assert!(page.blocks.iter().any(|b| matches!(
        &b.kind,
        tessera_resolve::ExpandedKind::Leaf(l) if matches!(l.kind, BlockKind::Directive(_))
    )));
}

#[test]
fn deleting_a_file_makes_broken_references_in_the_files_that_used_it() {
    let (mut inc, _) = load(
        &ModelSpec::base(),
        &[
            (
                "index.md",
                "---\ntitle: Home\n---\n\n@include: _f.md\n\n[guide](guide.md) ![logo](logo.png)\n",
            ),
            ("guide.md", "---\ntitle: Guide\n---\n\nGuide.\n"),
            ("_f.md", "Fragment.\n"),
            ("unrelated.md", "---\ntitle: U\n---\n\n[me](unrelated.md)\n"),
        ],
    );
    let affected = inc
        .apply([Change::AssetCreated {
            path: path("docs/logo.png"),
        }])
        .unwrap();
    assert_eq!(affected.recheck, set(&["index.md"]));
    let snapshot = inc.snapshot();
    assert!(snapshot.problems(&path("index.md")).is_empty());
    assert_eq!(snapshot.asset_users(&path("logo.png")).len(), 1);

    let affected = inc
        .apply([
            deleted("guide.md"),
            deleted("_f.md"),
            Change::AssetDeleted {
                path: path("docs/logo.png"),
            },
        ])
        .unwrap();
    assert_eq!(affected.removed, set(&["guide.md", "_f.md"]));
    assert_eq!(affected.recheck, set(&["index.md"]));
    assert_eq!(affected.re_resolve, set(&["index.md"]));
    let slugs: Vec<&str> = inc
        .snapshot()
        .problems(&path("index.md"))
        .iter()
        .map(|p| p.slug.as_str())
        .collect::<Vec<_>>()
        .into_iter()
        .map(|s| match s {
            "include-target-missing" => "include-target-missing",
            "link-target-missing" => "link-target-missing",
            "image-source-missing" => "image-source-missing",
            other => panic!("unexpected {other}"),
        })
        .collect();
    assert_eq!(
        slugs,
        [
            "include-target-missing",
            "link-target-missing",
            "image-source-missing"
        ]
    );
    assert!(!affected.recheck.contains(&path("unrelated.md")));
    // The reverse edges forgot the deleted files.
    let snapshot = inc.snapshot();
    assert!(snapshot.includers(&path("_f.md")).is_empty());
    assert!(snapshot.links_to(&path("guide.md")).is_empty());
    assert!(snapshot.asset_users(&path("logo.png")).is_empty());

    // And they come back with the file.
    let affected = inc
        .apply([created("guide.md", "---\ntitle: Guide\n---\n")])
        .unwrap();
    assert_eq!(affected.recheck, set(&["guide.md", "index.md"]));
    assert_eq!(inc.snapshot().links_to(&path("guide.md")).len(), 1);
}

#[test]
fn a_file_that_becomes_unreadable_is_listed_until_a_change_to_its_path() {
    // Resolved Q133: it leaves the index like a deleted file, but it still
    // exists, so it's listed as unreadable (its `source-unreadable`), not
    // removed.
    let (mut inc, _) = load(
        &ModelSpec::base(),
        &[
            ("index.md", "---\ntitle: Home\n---\n\n[guide](guide.md)\n"),
            ("guide.md", "---\ntitle: Guide\n---\n\nGuide.\n"),
        ],
    );
    let unreadable = || Change::Unreadable {
        path: path("guide.md"),
        reason: "stream did not contain valid UTF-8".to_owned(),
    };
    let listed = |inc: &IncrementalProject| -> Vec<RelPath> {
        inc.snapshot()
            .unreadable()
            .iter()
            .map(|u| u.path.clone())
            .collect()
    };

    let affected = inc.apply([unreadable()]).unwrap();
    assert_eq!(affected.unreadable, set(&["guide.md"]));
    assert!(affected.removed.is_empty());
    assert!(affected.recheck.contains(&path("index.md")));
    assert!(inc.snapshot().file(&path("guide.md")).is_none());
    assert_eq!(listed(&inc), [path("guide.md")]);

    // The same news again changes nothing.
    assert!(inc.apply([unreadable()]).unwrap().is_empty());

    // Readable again: back in the index, and off the list.
    let affected = inc
        .apply([edited("guide.md", "---\ntitle: Guide\n---\n")])
        .unwrap();
    assert!(affected.recheck.contains(&path("guide.md")));
    assert!(affected.unreadable.is_empty());
    assert!(inc.snapshot().file(&path("guide.md")).is_some());
    assert!(listed(&inc).is_empty());

    // Unreadable, then deleted: removed, and off the list.
    inc.apply([unreadable()]).unwrap();
    let affected = inc.apply([deleted("guide.md")]).unwrap();
    assert_eq!(affected.removed, set(&["guide.md"]));
    assert!(listed(&inc).is_empty());
}

#[test]
fn renaming_a_heading_updates_the_links_that_show_it() {
    let (mut inc, _) = load(
        &ModelSpec::base(),
        &[
            (
                "index.md",
                "---\ntitle: Home\n---\n\n[](guide.md#setup) and [](guide.md)\n",
            ),
            (
                "guide.md",
                "---\ntitle: The guide\n---\n\n## Setup\n@id: setup\n\nSteps.\n",
            ),
            ("elsewhere.md", "---\ntitle: E\n---\n\nText.\n"),
        ],
    );
    let shown = resolved(&inc.snapshot(), "index.md", "site");
    assert!(shown.contains("\"Setup\""), "{shown}");

    let affected = inc
        .apply([edited(
            "guide.md",
            "---\ntitle: The guide\n---\n\n## Installation\n@id: setup\n\nSteps.\n",
        )])
        .unwrap();
    assert_eq!(affected.re_resolve, set(&["guide.md", "index.md"]));
    let shown = resolved(&inc.snapshot(), "index.md", "site");
    assert!(shown.contains("\"Installation\""), "{shown}");
    assert!(!shown.contains("\"Setup\""), "{shown}");

    // A new title for the page changes the link that shows the page's title.
    let affected = inc
        .apply([edited(
            "guide.md",
            "---\ntitle: Renamed\n---\n\n## Installation\n@id: setup\n\nSteps.\n",
        )])
        .unwrap();
    assert_eq!(affected.re_resolve, set(&["guide.md", "index.md"]));
    assert!(resolved(&inc.snapshot(), "index.md", "site").contains("\"Renamed\""));
}

#[test]
fn a_rename_is_a_deletion_and_a_creation() {
    let (mut inc, _) = load(
        &ModelSpec::base(),
        &[
            (
                "index.md",
                "---\ntitle: Home\n---\n\n[old](old.md) [new](new.md)\n",
            ),
            ("old.md", "---\ntitle: Old\n---\n\n## Same\n"),
        ],
    );
    let affected = inc
        .apply([Change::Renamed {
            from: path("old.md"),
            to: path("new.md"),
        }])
        .unwrap();
    assert_eq!(affected.removed, set(&["old.md"]));
    assert_eq!(affected.recheck, set(&["index.md", "new.md"]));
    let snapshot = inc.snapshot();
    let slugs: Vec<&str> = snapshot
        .problems(&path("index.md"))
        .iter()
        .map(|p| p.slug.as_str())
        .collect();
    assert_eq!(slugs, ["link-target-missing"]);
    assert_eq!(snapshot.links_to(&path("new.md")).len(), 1);
    assert_eq!(
        snapshot.file(&path("new.md")).unwrap().title.as_deref(),
        Some("Old")
    );
}

#[test]
fn a_fragment_becoming_a_page_and_back() {
    // Renaming a file into a directory that starts with `_` makes it a
    // fragment: page and fragment classification is part of what changes.
    let (mut inc, _) = load(
        &ModelSpec::base(),
        &[
            ("index.md", "---\ntitle: Home\n---\n\n[t](topic.md)\n"),
            ("topic.md", "---\ntitle: Topic\n---\n\nText.\n"),
        ],
    );
    let affected = inc
        .apply([Change::Renamed {
            from: path("topic.md"),
            to: path("_topic.md"),
        }])
        .unwrap();
    assert!(affected.recheck.contains(&path("index.md")));
    assert_eq!(
        inc.snapshot().file(&path("_topic.md")).map(|f| f.kind),
        Some(FileKind::Fragment)
    );
    assert_eq!(affected.re_resolve, set(&["index.md"]));
}

// -- Caching ---------------------------------------------------------------------

#[test]
fn unchanged_files_are_never_reparsed() {
    let mut files: Vec<(String, String)> = Vec::new();
    for i in 0..40 {
        files.push((
            format!("p{i}.md"),
            format!(
                "---\ntitle: P{i}\n---\n\n## H{i}\n\n[n](p{}.md)\n",
                (i + 1) % 40
            ),
        ));
    }
    let refs: Vec<(&str, &str)> = files
        .iter()
        .map(|(a, b)| (a.as_str(), b.as_str()))
        .collect();
    let (mut inc, _) = load(&ModelSpec::base(), &refs);

    let before = inc.stats();
    let affected = inc
        .apply([edited(
            "p7.md",
            "---\ntitle: P7\n---\n\n## Changed\n\n[n](p8.md)\n",
        )])
        .unwrap();
    assert_eq!(affected.parsed, set(&["p7.md"]));
    assert_eq!(inc.stats().parses - before.parses, 1);
    assert_eq!(inc.stats().indexes - before.indexes, 1);

    // The same text again is nothing at all: no parse, no version.
    let version = inc.version();
    let before = inc.stats();
    let affected = inc
        .apply([edited(
            "p7.md",
            "---\ntitle: P7\n---\n\n## Changed\n\n[n](p8.md)\n",
        )])
        .unwrap();
    assert!(affected.is_empty());
    assert_eq!(inc.version(), version);
    assert_eq!(inc.stats(), before);
}

#[test]
fn a_model_change_the_index_reads_reindexes_but_reuses_every_parse() {
    let (mut inc, _) = load(
        &ModelSpec::base(),
        &[
            (
                "index.md",
                "---\ntitle: '{product}'\n---\n\n## {product} tips\n",
            ),
            ("guide.md", "---\ntitle: G\n---\n\nText.\n"),
        ],
    );
    let stats = inc.stats();
    let affected = inc
        .apply([Change::Model(
            ModelSpec {
                product: "Quill Pro",
                ..ModelSpec::base()
            }
            .model(),
        )])
        .unwrap();
    assert_eq!(affected.model, Some(ModelImpact::Index));
    assert!(affected.parsed.is_empty(), "the parse doesn't read phrases");
    assert_eq!(affected.indexed, set(&["index.md", "guide.md"]));
    assert_eq!(inc.stats().parses, stats.parses);
    // The heading's text (and so its slug) took the new phrase.
    let snapshot = inc.snapshot();
    let heading = &snapshot.file(&path("index.md")).unwrap().headings[0];
    assert_eq!(heading.text, "Quill Pro tips");
}

#[test]
fn a_model_change_only_later_passes_read_reparses_and_reindexes_nothing() {
    let (mut inc, _) = load(
        &ModelSpec::base(),
        &[
            ("index.md", "---\ntitle: Home\n---\n\n@include: _f.md\n"),
            ("_f.md", "Text.\n"),
        ],
    );
    inc.snapshot().expand(&path("index.md"));
    let stats = inc.stats();
    let affected = inc
        .apply([Change::Model(
            ModelSpec {
                hybrid: true,
                ..ModelSpec::base()
            }
            .model(),
        )])
        .unwrap();
    assert_eq!(affected.model, Some(ModelImpact::Resolution));
    assert!(affected.parsed.is_empty() && affected.indexed.is_empty());
    assert_eq!(inc.stats().parses, stats.parses);
    assert_eq!(inc.stats().indexes, stats.indexes);
    assert_eq!(affected.recheck, set(&["index.md", "_f.md"]));
    assert_eq!(affected.re_resolve, set(&["index.md"]));
    // Fragment patterns decide page or fragment, which the index records.
    let affected = inc
        .apply([Change::Model(
            ModelSpec {
                hybrid: true,
                shared_pattern: true,
                ..ModelSpec::base()
            }
            .model(),
        )])
        .unwrap();
    assert_eq!(affected.model, Some(ModelImpact::Index));
}

#[test]
fn a_model_that_only_moves_its_own_warnings_changes_no_file() {
    let (mut inc, _) = load(
        &ModelSpec::base(),
        &[("index.md", "---\ntitle: H\n---\n\nText.\n")],
    );
    let affected = inc
        .apply([Change::Model(
            ModelSpec {
                comment: true,
                ..ModelSpec::base()
            }
            .model(),
        )])
        .unwrap();
    // The comment shifts no warning here, so nothing at all changed.
    assert!(affected.is_empty());
    // An identical model is nothing either.
    let affected = inc
        .apply([Change::Model(ModelSpec::base().model())])
        .unwrap();
    assert!(affected.is_empty());
}

#[test]
fn a_new_content_root_cannot_be_applied_in_place() {
    let (mut inc, _) = load(&ModelSpec::base(), &[("index.md", "x\n")]);
    let other = tessera_model::load_str(
        "spec = \"0.1\"\n[project]\ncontent-root = \"content\"\n",
        FileId::new(0),
    )
    .expect("a model");
    let version = inc.version();
    assert_eq!(
        inc.apply([edited("index.md", "y\n"), Change::Model(Arc::new(other))])
            .unwrap_err(),
        ApplyError::LayoutChanged
    );
    // Nothing was applied, not even the edit before it.
    assert_eq!(inc.version(), version);
    assert_eq!(
        &*inc.snapshot().file(&path("index.md")).unwrap().source,
        "x\n"
    );
}

#[test]
fn expansions_are_kept_for_pages_a_change_cannot_reach() {
    let (mut inc, _) = load(
        &ModelSpec::base(),
        &[
            ("a.md", "@include: _f.md\n"),
            ("b.md", "Plain.\n"),
            ("_f.md", "Text.\n"),
        ],
    );
    let first = inc.snapshot();
    let b = first.expansion(&path("b.md")).unwrap();
    let a = first.expansion(&path("a.md")).unwrap();
    inc.apply([edited("_f.md", "New text.\n")]).unwrap();
    let second = inc.snapshot();
    // The page nothing reached shares its cached expansion; the reached one
    // was expanded again.
    assert!(Arc::ptr_eq(&b, &second.expansion(&path("b.md")).unwrap()));
    assert!(!Arc::ptr_eq(&a, &second.expansion(&path("a.md")).unwrap()));
    // The old snapshot still answers from its own state.
    assert!(Arc::ptr_eq(&a, &first.expansion(&path("a.md")).unwrap()));
}

#[test]
fn the_resolved_page_cache_redoes_only_what_an_update_lists() {
    let (mut inc, _) = load(
        &ModelSpec::base(),
        &[
            ("a.md", "---\ntitle: A\n---\n\n@include: _f.md\n"),
            ("b.md", "---\ntitle: B\n---\n\nPlain.\n"),
            ("_f.md", "Text.\n"),
        ],
    );
    let mut cache = ResolvedCache::new();
    let router = DefaultRouter::new();
    let build = inc
        .snapshot()
        .model()
        .build("site")
        .expect("a build")
        .clone();
    for page in ["a.md", "b.md"] {
        cache
            .resolve_page(&inc.snapshot(), &path(page), &build, &router)
            .expect("a page");
    }
    assert_eq!(cache.computed(), 2);
    // Asking again is free.
    cache.resolve_page(&inc.snapshot(), &path("a.md"), &build, &router);
    assert_eq!(cache.computed(), 2);

    let affected = inc.apply([edited("_f.md", "Other text.\n")]).unwrap();
    cache.apply(&affected);
    let snapshot = inc.snapshot();
    let b = cache
        .resolve_page(&snapshot, &path("b.md"), &build, &router)
        .expect("a page");
    assert_eq!(cache.computed(), 2, "b was kept");
    let a = cache
        .resolve_page(&snapshot, &path("a.md"), &build, &router)
        .expect("a page");
    assert_eq!(cache.computed(), 3, "a was resolved again");
    assert_eq!(
        format!("{a:?}"),
        format!(
            "{:?}",
            snapshot
                .resolve_page(&path("a.md"), &build, &router)
                .unwrap()
        )
    );
    assert_eq!(
        format!("{b:?}"),
        format!(
            "{:?}",
            snapshot
                .resolve_page(&path("b.md"), &build, &router)
                .unwrap()
        )
    );
}

// -- Snapshots -------------------------------------------------------------------

#[test]
fn snapshots_are_tagged_and_never_change() {
    let (mut inc, _) = load(
        &ModelSpec::base(),
        &[
            ("a.md", "---\ntitle: A\n---\n\n[b](b.md)\n"),
            ("b.md", "---\ntitle: B\n---\n"),
        ],
    );
    let v0 = inc.snapshot();
    assert!(v0.is_current());
    assert_eq!(v0.file_version(&path("a.md")), Some(v0.version()));

    inc.apply([edited("b.md", "---\ntitle: B2\n---\n")])
        .unwrap();
    let v1 = inc.snapshot();
    assert!(v1.version() > v0.version());
    assert!(!v0.is_current() && v1.is_current());
    // The old snapshot still shows the old text.
    assert_eq!(v0.file(&path("b.md")).unwrap().title.as_deref(), Some("B"));
    assert_eq!(v1.file(&path("b.md")).unwrap().title.as_deref(), Some("B2"));
    // Each file knows when its text last changed.
    assert_eq!(v1.file_version(&path("a.md")), Some(v0.version()));
    assert_eq!(v1.file_version(&path("b.md")), Some(v1.version()));

    // A result for `a` computed from v0 isn't stale by the edit to `b`'s
    // paragraph-free front matter... unless `a` links to it: it does, and the
    // title changed, so `a` is listed.
    assert!(!inc.is_file_current(&v0, &path("a.md")));
    inc.apply([edited(
        "b.md",
        "---\ntitle: B2\n---\n\nAdded a paragraph.\n",
    )])
    .unwrap();
    // A paragraph changes nothing `a` can see.
    assert!(inc.is_file_current(&v1, &path("a.md")));
    assert!(!inc.is_file_current(&v1, &path("b.md")));
    assert!(
        !inc.is_file_current(&v0, &path("a.md")),
        "v0 still spans the title change"
    );

    // A model change beyond warnings makes every file stale.
    let v2 = inc.snapshot();
    inc.apply([Change::Model(
        ModelSpec {
            hybrid: true,
            ..ModelSpec::base()
        }
        .model(),
    )])
    .unwrap();
    assert!(!inc.is_file_current(&v2, &path("a.md")));
    assert_eq!(inc.snapshot().model_revision(), 1);
}

#[test]
fn a_snapshot_too_old_for_the_history_is_treated_as_stale() {
    let (mut inc, _) = load(&ModelSpec::base(), &[("a.md", "x\n"), ("b.md", "y\n")]);
    let first = inc.snapshot();
    for i in 0..300 {
        inc.apply([edited("b.md", &format!("y{i}\n"))]).unwrap();
    }
    assert!(
        !inc.is_file_current(&first, &path("a.md")),
        "can't be sure, so not current"
    );
    let recent = inc.snapshot();
    inc.apply([edited("b.md", "z\n")]).unwrap();
    assert!(inc.is_file_current(&recent, &path("a.md")));
}

// -- File ids --------------------------------------------------------------------

#[test]
fn ids_name_paths_and_are_never_reused() {
    let (mut inc, _) = load(&ModelSpec::base(), &[("b.md", "x\n"), ("a.md", "y\n")]);
    let id = |inc: &IncrementalProject, p: &str| inc.snapshot().file(&path(p)).map(|f| f.file);
    // A fresh load numbers files in path order, from 1.
    assert_eq!(id(&inc, "a.md"), Some(FileId::new(1)));
    assert_eq!(id(&inc, "b.md"), Some(FileId::new(2)));

    // Editing keeps the id; creating takes the next number.
    inc.apply([edited("a.md", "changed\n"), created("c.md", "z\n")])
        .unwrap();
    assert_eq!(id(&inc, "a.md"), Some(FileId::new(1)));
    assert_eq!(id(&inc, "c.md"), Some(FileId::new(3)));

    // Deleting retires the file: no one has its id, and no other path takes it.
    inc.apply([deleted("a.md")]).unwrap();
    assert_eq!(inc.snapshot().path_of(FileId::new(1)), None);
    inc.apply([created("d.md", "w\n")]).unwrap();
    assert_eq!(id(&inc, "d.md"), Some(FileId::new(4)));
    assert_eq!(inc.snapshot().path_of(FileId::new(1)), None);

    // A file that comes back at the same path gets its id back.
    inc.apply([created("a.md", "back\n")]).unwrap();
    assert_eq!(id(&inc, "a.md"), Some(FileId::new(1)));

    // A rename is a deletion and a creation: the new path has its own id.
    inc.apply([Change::Renamed {
        from: path("b.md"),
        to: path("e.md"),
    }])
    .unwrap();
    assert_eq!(inc.snapshot().path_of(FileId::new(2)), None);
    assert_eq!(id(&inc, "e.md"), Some(FileId::new(5)));

    // Ids in a batch are assigned in path order, whatever order it lists them.
    inc.apply([created("z.md", "1\n"), created("m.md", "2\n")])
        .unwrap();
    assert_eq!(id(&inc, "m.md"), Some(FileId::new(6)));
    assert_eq!(id(&inc, "z.md"), Some(FileId::new(7)));
    // Every location an index holds names its own file's id.
    for file in inc.snapshot().files() {
        assert_eq!(file.document.file, file.file);
    }
}

// -- Assets and the file system --------------------------------------------------

#[test]
fn assets_from_the_base_file_system_can_be_removed_and_added() {
    let model = ModelSpec::base().model();
    let layout = Layout::from_model(&model);
    let fs = MemoryFs::new(&layout)
        .with_source(
            "index.md",
            "---\ntitle: H\n---\n\n![a](logo.png) ![b](pic.png)\n",
        )
        .with_file("docs/logo.png", "");
    let mut inc = IncrementalProject::load(model, layout, fs);
    let slug_count = |inc: &IncrementalProject| inc.snapshot().problems(&path("index.md")).len();
    assert_eq!(slug_count(&inc), 1, "pic.png is missing");

    let affected = inc
        .apply([Change::AssetCreated {
            path: path("docs/pic.png"),
        }])
        .unwrap();
    assert_eq!(affected.recheck, set(&["index.md"]));
    assert_eq!(affected.re_resolve, set(&["index.md"]));
    assert_eq!(slug_count(&inc), 0);

    let affected = inc
        .apply([Change::AssetDeleted {
            path: path("docs/logo.png"),
        }])
        .unwrap();
    assert_eq!(affected.re_resolve, set(&["index.md"]));
    assert_eq!(slug_count(&inc), 1);

    // A file that isn't there can't be deleted, and one that is can't be created.
    assert!(
        inc.apply([Change::AssetDeleted {
            path: path("docs/logo.png")
        }])
        .unwrap()
        .is_empty()
    );
    assert!(
        inc.apply([Change::AssetCreated {
            path: path("docs/pic.png")
        }])
        .unwrap()
        .is_empty()
    );
    // A file no reference names changes nothing for anyone.
    let affected = inc
        .apply([Change::AssetCreated {
            path: path("docs/unused.png"),
        }])
        .unwrap();
    assert!(affected.recheck.is_empty() && affected.re_resolve.is_empty());
}

#[test]
fn a_batch_is_taken_by_its_net_effect() {
    let (mut inc, _) = load(&ModelSpec::base(), &[("a.md", "x\n")]);
    let version = inc.version();
    // Created and deleted in one batch: never there.
    let affected = inc
        .apply([created("b.md", "y\n"), deleted("b.md")])
        .unwrap();
    assert!(affected.is_empty());
    // Edited and edited back: unchanged.
    let affected = inc
        .apply([edited("a.md", "changed\n"), edited("a.md", "x\n")])
        .unwrap();
    assert!(affected.is_empty());
    assert_eq!(inc.version(), version);
    // Paths that aren't sources (dot directories) are files, not sources, and
    // a file nothing names changes nothing anyone can see.
    let affected = inc.apply([created(".hidden/x.md", "y\n")]).unwrap();
    assert!(affected.is_empty());
    assert!(inc.snapshot().file(&path(".hidden/x.md")).is_none());
    assert_eq!(inc.version(), version);
}
