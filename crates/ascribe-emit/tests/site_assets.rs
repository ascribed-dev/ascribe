//! Assets in the site output follow the `astro` profile's rules:
//! images are mirrored beside their pages and referenced relatively (so
//! Astro's image processing applies), and other files pages link to are
//! published under `_ascribe/files/` and referenced by URL. The output works
//! with the source removed.

#![allow(clippy::expect_used, clippy::panic)]

mod support;

use std::fs;
use std::path::{Path, PathBuf};

use ascribe_emit::{EmitContext, Emitter, OutputDir, SiteEmitter, emit};
use ascribe_resolve::AstroRouter;

const MODEL: &str = "spec = \"0.1\"\n\n[project]\ncontent-root = \"docs\"\n\n[types.page]\ndefault = true\n\n[types.page.frontmatter]\ntitle = \"string\"\n\n[consumer]\nbase-path = \"/docs/\"\ntrailing-slash = \"never\"\n";

/// A fragment with an image beside it, a page that includes it, an image
/// beside the page, downloadable files (one outside the content root, with a
/// name that needs escaping), and a link to the fragment's image.
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

fn build_into(dir: &Path) -> PathBuf {
    let project = support::load(dir);
    let build = project.model().build("site").expect("a build");
    let router = AstroRouter::from_consumer(&project.model().consumer);
    let resolved = project.resolve_build(build, &router);
    let cx = EmitContext::new(&project, dir, build);
    let emitter = SiteEmitter::new(project.model());
    let emission = emit(&emitter, &cx, &resolved).expect("emits");
    let output = OutputDir::lock(&dir.join(".ascribe/build")).expect("locked");
    output
        .replace("site", emitter.name(), &emission.files)
        .expect("written");
    output.emitter_root("site", emitter.name())
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

/// The destinations of the images (`![..](..)`) and of the links (`[..](..)`)
/// in some markdown, by scanning for `](`.
fn destinations(markdown: &str) -> (Vec<String>, Vec<String>) {
    let (mut images, mut links) = (Vec::new(), Vec::new());
    let mut rest = markdown;
    while let Some(at) = rest.find("](") {
        let before = &rest[..at];
        let open = before.rfind('[').expect("an opening bracket");
        let is_image = before[..open].ends_with('!');
        let after = &rest[at + 2..];
        let end = after.find(')').expect("a closing parenthesis");
        let mut destination = after[..end].to_owned();
        // A destination in angle brackets holds spaces and parentheses.
        if destination.starts_with('<') {
            let close = after.find(">)").expect("a closing angle bracket");
            destination = after[1..close].to_owned();
        }
        if is_image {
            images.push(destination);
        } else {
            links.push(destination);
        }
        rest = &after[end + 1..];
    }
    (images, links)
}

/// Where a relative reference written in `page` points in `root`.
fn resolve_relative(root: &Path, page: &str, destination: &str) -> PathBuf {
    let dir = Path::new(page).parent().expect("a directory");
    let path = destination.split('#').next().expect("a path");
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

#[test]
fn the_site_output_works_with_the_source_removed() {
    let dir = project_dir();
    let root = build_into(dir.path());
    let kept = tempfile::tempdir().expect("a temporary directory");
    let moved = kept.path().join("site");
    copy_dir(&root, &moved);
    drop(dir);

    let page = fs::read_to_string(moved.join("guides/install.md")).expect("the page");
    let (images, links) = destinations(&page);

    // Images stay relative, so Astro processes them: beside the page, and
    // beside the fragment (mirrored), both of which still exist.
    assert_eq!(
        images,
        ["./img/settings.png", "../_fragments/pipeline.png"],
        "{page}"
    );
    for image in &images {
        let target = resolve_relative(&moved, "guides/install.md", image);
        assert!(target.is_file(), "{image} -> {}", target.display());
    }

    // Links to other files are URLs under the base path, served from
    // `_ascribe/files/`: each names a file in the output.
    assert_eq!(
        links,
        [
            "/docs/_ascribe/files/_fragments/pipeline.png",
            "/docs/_ascribe/files/downloads/quill.yaml#top",
            "/docs/_ascribe/files/_ascribe/up/shared/My%20Manual%20%28v2%29.pdf",
        ],
        "{page}"
    );
    for link in &links {
        let served = link
            .strip_prefix("/docs/")
            .and_then(|p| p.split('#').next())
            .expect("under the base path");
        let decoded = served
            .replace("%20", " ")
            .replace("%28", "(")
            .replace("%29", ")");
        assert!(moved.join(&decoded).is_file(), "{link} -> {decoded}");
    }
    assert_eq!(
        fs::read(moved.join("_ascribe/files/downloads/quill.yaml")).expect("a copy"),
        b"agent: {}\n"
    );
    // A file used as an image and as a link is copied twice: once mirrored,
    // once published, since the two placements differ.
    assert!(moved.join("_fragments/pipeline.png").is_file());
    assert!(
        moved
            .join("_ascribe/files/_fragments/pipeline.png")
            .is_file()
    );
}

#[test]
fn the_manifest_lists_each_copy_with_its_source_and_published_ones_with_their_url() {
    let dir = project_dir();
    build_into(dir.path());
    let manifest: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(dir.path().join(".ascribe/build/site/site.manifest.json"))
            .expect("manifest"),
    )
    .expect("JSON");
    let files = manifest["files"].as_array().expect("files");
    let entry = |path: &str| {
        files
            .iter()
            .find(|f| f["path"] == path)
            .unwrap_or_else(|| panic!("{path} is listed: {files:#?}"))
    };
    let yaml = entry("_ascribe/files/downloads/quill.yaml");
    assert_eq!(yaml["kind"], "asset");
    assert_eq!(yaml["source"], "downloads/quill.yaml");
    assert_eq!(yaml["url"], "/docs/_ascribe/files/downloads/quill.yaml");
    let image = entry("guides/img/settings.png");
    assert_eq!(image["kind"], "asset");
    assert!(image.get("url").is_none() || image["url"].is_null());
    assert_eq!(entry("_ascribe/schema.ts")["kind"], "generated");
    assert_eq!(entry("guides/install.md")["kind"], "page");
}
