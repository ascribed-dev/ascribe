//! Self-contained output: assets are copied, references are rewritten, and
//! the output works with the source directory removed (asset contract, §6).

#![allow(clippy::expect_used, clippy::panic)]

mod support;

use std::fs;
use std::path::{Path, PathBuf};

use comrak::nodes::NodeValue;
use comrak::{Arena, Options, parse_document};
use tessera_emit::EmitContext;
use tessera_emit::{Emitter, JsonEmitter, OutputDir, PlainEmitter, emit};
use tessera_resolve::AstroRouter;

const MODEL: &str = "spec = \"0.1\"\n\n[project]\ncontent-root = \"docs\"\n\n[types.page]\ndefault = true\n\n[types.page.frontmatter]\ntitle = \"string\"\n\n[consumer]\nsite = \"https://docs.example.com\"\n";

/// A fragment with an image beside it, a page that includes it, an image
/// beside the page, a downloadable file, and an asset outside the content
/// root (the project's `shared/` directory) with a name that needs escaping.
fn project_dir() -> tempfile::TempDir {
    let dir = tempfile::tempdir().expect("a temporary directory");
    let write = |path: &str, bytes: &[u8]| {
        let full = dir.path().join(path);
        fs::create_dir_all(full.parent().expect("a parent")).expect("directories");
        fs::write(full, bytes).expect("a file");
    };
    write("ascribe.toml", MODEL.as_bytes());
    write(
        "docs/guides/install.md",
        b"---\ntitle: Install\n---\n\n![Settings](img/settings.png)\n\n@include: ../_fragments/prerequisites.md\n\nGet the [sample config](../downloads/quill.yaml#top) and the [manual](../../shared/My%20Manual%20(v2).pdf).\n",
    );
    write(
        "docs/_fragments/prerequisites.md",
        b"Before you start:\n\n![Pipeline](pipeline.png)\n\nSee [the diagram](pipeline.png).\n",
    );
    write("docs/guides/img/settings.png", b"settings-bytes");
    write("docs/_fragments/pipeline.png", b"pipeline-bytes");
    write("docs/downloads/quill.yaml", b"agent: {}\n");
    write("shared/My Manual (v2).pdf", b"pdf-bytes");
    dir
}

/// Every reference in a markdown document: `(is_image, destination)`.
fn references(markdown: &str) -> Vec<(bool, String)> {
    let arena = Arena::new();
    let root = parse_document(&arena, markdown, &Options::default());
    root.descendants()
        .filter_map(|n| match &n.data.borrow().value {
            NodeValue::Image(l) => Some((true, l.url.clone())),
            NodeValue::Link(l) => Some((false, l.url.clone())),
            _ => None,
        })
        .collect()
}

/// Where a reference written in `page` (relative to `root`) points, if it is
/// a relative one.
fn resolve(root: &Path, page: &str, destination: &str) -> PathBuf {
    let dir = Path::new(page).parent().expect("a directory");
    let decoded = destination
        .replace("%25", "%")
        .replace("%23", "#")
        .replace("%3F", "?");
    let path = decoded.split('#').next().expect("a path");
    let mut out = root.to_path_buf();
    for part in dir.components().chain(Path::new(path).components()) {
        match part {
            std::path::Component::ParentDir => {
                out.pop();
            }
            std::path::Component::Normal(p) => out.push(p),
            _ => {}
        }
    }
    out
}

fn build_into(dir: &Path, emitter: &dyn Emitter) -> PathBuf {
    let project = support::load(dir);
    let build = project.model().build("site").expect("a build");
    let router = AstroRouter::from_consumer(&project.model().consumer);
    let resolved = project.resolve_build(build, &router);
    let cx = EmitContext::new(&project, dir, build);
    let emission = emit(emitter, &cx, &resolved).expect("emits");
    let output = OutputDir::lock(&dir.join(".tessera/build")).expect("locked");
    output
        .replace("site", emitter.name(), &emission.files)
        .expect("written");
    output.emitter_root("site", emitter.name())
}

#[test]
fn the_plain_output_works_with_the_source_removed() {
    let dir = project_dir();
    let root = build_into(dir.path(), &PlainEmitter);
    // Move the output out of the project, and delete everything else.
    let kept = tempfile::tempdir().expect("a temporary directory");
    let moved = kept.path().join("plain");
    copy_dir(&root, &moved);
    drop(dir);

    let page = fs::read_to_string(moved.join("guides/install.md")).expect("the page");
    let refs = references(&page);
    assert_eq!(refs.len(), 5, "{refs:?}\n{page}");
    for (is_image, destination) in &refs {
        assert!(
            destination.starts_with("./") || destination.starts_with("../"),
            "{destination}"
        );
        let target = resolve(&moved, "guides/install.md", destination);
        assert!(
            target.is_file(),
            "{destination} -> {} in\n{page}",
            target.display()
        );
        let _ = is_image;
    }
    // Where they point: beside the page, beside the fragment (mirrored), and
    // outside the content root (`_tessera/up`).
    let destinations: Vec<&str> = refs.iter().map(|(_, d)| d.as_str()).collect();
    assert!(
        destinations.contains(&"./img/settings.png"),
        "{destinations:?}"
    );
    assert!(
        destinations.contains(&"../_fragments/pipeline.png"),
        "{destinations:?}"
    );
    assert!(
        destinations.contains(&"../downloads/quill.yaml#top"),
        "{destinations:?}"
    );
    assert!(
        destinations.contains(&"../_tessera/up/shared/My Manual (v2).pdf"),
        "{destinations:?}"
    );
    assert_eq!(
        fs::read(moved.join("_fragments/pipeline.png")).expect("copy"),
        b"pipeline-bytes"
    );
    assert_eq!(
        fs::read(moved.join("_tessera/up/shared/My Manual (v2).pdf")).expect("copy"),
        b"pdf-bytes"
    );
}

#[test]
fn the_json_output_works_with_the_source_removed() {
    let dir = project_dir();
    let root = build_into(dir.path(), &JsonEmitter);
    let kept = tempfile::tempdir().expect("a temporary directory");
    let moved = kept.path().join("json");
    copy_dir(&root, &moved);
    drop(dir);

    let text = fs::read_to_string(moved.join("guides/install.json")).expect("the page");
    let doc: serde_json::Value = serde_json::from_str(&text).expect("JSON");
    let assets = doc["assets"].as_array().expect("assets");
    // One entry for each surviving reference, with the source path beside the copy.
    assert_eq!(assets.len(), 5, "{assets:#?}");
    for asset in assets {
        let reference = asset["reference"].as_str().expect("a reference");
        let target = resolve(&moved, "guides/install.json", reference);
        assert!(target.is_file(), "{reference} -> {}", target.display());
        assert!(asset["source"].is_string() && asset["path"].is_string());
    }
    let pipeline = assets
        .iter()
        .find(|a| a["source"] == "_fragments/pipeline.png")
        .expect("the fragment's image");
    assert_eq!(pipeline["reference"], "../_fragments/pipeline.png");
    assert_eq!(pipeline["writtenIn"], "_fragments/prerequisites.md");
}

#[test]
fn each_asset_is_copied_once_however_often_it_is_used() {
    let dir = project_dir();
    let root = build_into(dir.path(), &PlainEmitter);
    let manifest: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(dir.path().join(".tessera/build/site/plain.manifest.json"))
            .expect("manifest"),
    )
    .expect("JSON");
    let paths: Vec<&str> = manifest["files"]
        .as_array()
        .expect("files")
        .iter()
        .map(|f| f["path"].as_str().expect("a path"))
        .collect();
    assert_eq!(
        paths,
        [
            "_fragments/pipeline.png",
            "_tessera/up/shared/My Manual (v2).pdf",
            "downloads/quill.yaml",
            "guides/img/settings.png",
            "guides/install.md",
        ]
    );
    assert!(root.join("downloads/quill.yaml").is_file());
}

fn copy_dir(from: &Path, to: &Path) {
    fs::create_dir_all(to).expect("a directory");
    for entry in fs::read_dir(from).expect("a listing") {
        let entry = entry.expect("an entry");
        let target = to.join(entry.file_name());
        if entry.file_type().expect("a type").is_dir() {
            copy_dir(&entry.path(), &target);
        } else {
            fs::copy(entry.path(), target).expect("a copy");
        }
    }
}
