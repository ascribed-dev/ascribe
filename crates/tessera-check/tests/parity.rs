//! `ascribe check` and the source index agree about references.
//!
//! Both run `tessera_resolve::references`, so for the same project on disk
//! they must report the same file-level problems with includes, links, and
//! images, at the same places, and number the files the same way. This test
//! builds both over one directory tree that has every kind of reference
//! problem, and compares them.

#![allow(clippy::expect_used, clippy::panic)]

use std::collections::BTreeSet;
use std::fs;
use std::path::Path;
use std::sync::Arc;

use tessera_check::{Project, check_files};
use tessera_core::{FileId, RelPath};
use tessera_resolve::{DiskFs, Layout};

/// The slugs whose rules live in `tessera_resolve::references`.
const REFERENCE_SLUGS: &[&str] = &[
    "include-target-missing",
    "link-target-missing",
    "image-source-missing",
    "link-to-fragment",
    "link-route",
];

const MODEL: &str = r#"spec = "0.1"

[project]
content-root = "docs"
output-dir = "out"

[phrases]
api = "https://api.quill.dev/"
here = "keys"
"#;

const PAGE: &str = r#"---
title: Home
---

@include: _f.md
@include: /guides/setup.md#install
@include: missing.md
@include: ../README.md
@include: _F.md

[ok](keys.md) [case](Keys.md) [gone](gone.md) [fragment](_f.md)
[route](/guides/setup/) [route no page](/nowhere/) [dir route](guides/)
[phrase]({api}streaming) [local phrase]({here}.md) [asset](../assets/file.pdf)
[outside](../../elsewhere.md) [output](../out/site/x.png) [self](#top)

![logo](logo.png) ![case](Logo.png) ![gone](gone.png) ![](#) ![up](../assets/pic.png)
![outside](../../pic.png)

[defined url][d-api] [defined file][d-file] [defined missing][d-gone] ![defined image][d-img]
[collapsed][] [shortcut]

[d-api]: {api}streaming
[d-file]: {here}.md
[d-gone]: {here}2.md
[d-img]: {here}.png
[collapsed]: {api}collapsed
[shortcut]: gone-shortcut.md
"#;

fn write(root: &Path, path: &str, text: &str) {
    let full = root.join(path);
    fs::create_dir_all(full.parent().expect("a parent")).expect("mkdir");
    fs::write(full, text).expect("write");
}

/// The shared tree: pages, a fragment, a page in a subdirectory, a dot
/// directory whose Markdown isn't a source, assets inside and outside the
/// content root, and one in the output directory.
fn tree() -> tempfile::TempDir {
    let dir = tempfile::tempdir().expect("a temp dir");
    let root = dir.path();
    write(root, "ascribe.toml", MODEL);
    write(root, "docs/index.md", PAGE);
    write(root, "docs/keys.md", "---\ntitle: Keys\n---\n\n## Top\n");
    write(
        root,
        "docs/_f.md",
        "Fragment text. [back](index.md#top) [self](#x)\n",
    );
    write(
        root,
        "docs/guides/setup.md",
        "---\ntitle: Setup\n---\n\n## Install\n",
    );
    write(root, "docs/guides/index.md", "---\ntitle: Guides\n---\n");
    write(root, "docs/.drafts/hidden.md", "---\ntitle: Hidden\n---\n");
    write(root, "docs/logo.png", "png");
    write(root, "assets/file.pdf", "pdf");
    write(root, "assets/pic.png", "png");
    write(root, "out/site/x.png", "png");
    write(root, "README.md", "# Readme\n");
    dir
}

type Found = BTreeSet<(String, String, usize, usize)>;

fn from_check(root: &Path) -> (Found, Vec<(u32, String)>) {
    let project = Project::load(&root.join("ascribe.toml")).expect("the project loads");
    let found = check_files(&project)
        .into_iter()
        .filter(|d| REFERENCE_SLUGS.contains(&d.slug.to_string().as_str()))
        .map(|d| {
            let file = project.file(d.location.file).expect("a file");
            (
                d.slug.to_string(),
                file.content_path.expect("a source").to_string(),
                d.location.span.start(),
                d.location.span.end(),
            )
        })
        .collect();
    let ids = project
        .sources()
        .iter()
        .map(|s| (s.id.index(), s.path.to_string()))
        .collect();
    (found, ids)
}

fn from_index(root: &Path) -> (Found, Vec<(u32, String)>) {
    let model = tessera_model::load_str(MODEL, FileId::new(0)).expect("the model loads");
    let layout = Layout::from_model(&model);
    let disk = DiskFs::new(root, &layout);
    let project = tessera_resolve::Project::load(Arc::new(model), layout, &disk);
    let mut found = Found::new();
    let mut ids = Vec::new();
    for file in project.files() {
        ids.push((file.file.index(), file.path.to_string()));
        for issue in project.problems(&file.path) {
            let slug = issue.slug.to_string();
            if REFERENCE_SLUGS.contains(&slug.as_str()) {
                found.insert((
                    slug,
                    file.path.to_string(),
                    issue.location.span.start(),
                    issue.location.span.end(),
                ));
            }
        }
    }
    (found, ids)
}

#[test]
fn check_and_the_source_index_report_the_same_reference_problems() {
    let dir = tree();
    let (check, check_ids) = from_check(dir.path());
    let (index, index_ids) = from_index(dir.path());

    // The same source files, with the same ids; the dot directory isn't one.
    assert_eq!(check_ids, index_ids);
    assert!(
        check_ids.iter().all(|(id, _)| *id >= 1),
        "id 0 is ascribe.toml"
    );
    assert!(!check_ids.iter().any(|(_, p)| p.contains(".drafts")));

    // The same problems, at the same places.
    assert_eq!(check, index);

    // And the tree exercises every rule.
    let slugs: BTreeSet<&str> = check.iter().map(|(s, ..)| s.as_str()).collect();
    for slug in REFERENCE_SLUGS {
        assert!(slugs.contains(slug), "no case for {slug}: {check:#?}");
    }
    assert!(check.len() >= 15, "{check:#?}");
}

#[test]
fn an_existing_file_that_isnt_a_source_cant_be_included() {
    let dir = tree();
    let (check, _) = from_check(dir.path());
    let readme = RelPath::parse("index.md").expect("path").to_string();
    let includes: Vec<_> = check
        .iter()
        .filter(|(slug, file, ..)| slug == "include-target-missing" && *file == readme)
        .collect();
    // `missing.md`, `../README.md` (exists, but isn't a source), and
    // `_F.md` (a case twin of `_f.md`).
    assert_eq!(includes.len(), 3, "{includes:#?}");
}

#[test]
fn a_definitions_phrases_are_applied_to_the_references_that_use_it() {
    // `{here}.png` in a definition is `keys.png`, which doesn't exist,
    // and `{api}` is a URL. Both implementations agree, at the reference
    // (a reference form's diagnostics point at the whole link).
    let dir = tree();
    let (check, _) = from_check(dir.path());
    let page = fs::read_to_string(dir.path().join("docs/index.md")).expect("the page");
    let texts: BTreeSet<(String, &str)> = check
        .iter()
        .filter(|(_, file, ..)| file == "index.md")
        .map(|(slug, _, start, end)| (slug.clone(), &page[*start..*end]))
        .collect();
    assert!(texts.contains(&("image-source-missing".to_owned(), "![defined image][d-img]")));
    assert!(texts.contains(&(
        "link-target-missing".to_owned(),
        "[defined missing][d-gone]"
    )));
    assert!(
        !texts
            .iter()
            .any(|(_, t)| t.contains("d-api") || t.contains("d-file"))
    );
}
