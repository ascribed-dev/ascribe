//! The static report: [`write_html`] writes a [`Report`] as one
//! self-contained HTML file, every changed page rendered with its changes
//! marked, for `ascribe diff --format html`.
//!
//! Each page is rendered as the editor's page preview renders it: resolved
//! for the build, written as site markdown with source anchors, and turned
//! into HTML by [`render_site_html`]. The page as it is now and as it was are
//! both rendered, so a removed block can be shown as it looked. Neither has
//! the site's layout: it's Ascribe's bare render, and the report says so.
//!
//! Everything is inline: the report's stylesheet and script (built from
//! `packages/review` and `packages/elements`, and embedded here by
//! `pnpm --filter @ascribed/review embed`), the data the script draws from,
//! and every image as a `data:` URL up to [`MAX_IMAGE_BYTES`]. A content
//! security policy forbids every request but `data:` images, so nothing in a
//! page can reach the network either.

use std::collections::{BTreeMap, HashMap};
use std::fs;
use std::path::{Path, PathBuf};

use serde::Serialize;
use tessera_core::{AssetUse, RelPath};
use tessera_emit::assets::encode_path;
use tessera_emit::{EmitContext, SiteEmitter, emit_page, render_site_html};
use tessera_resolve::{AstroRouter, Project};

use crate::compare::{PageDiff, PageStatus};
use crate::git::Repository;
use crate::gitfs::GitFs;
use crate::{BaseInfo, Report};

/// The report's script: `packages/review/src/report/`, bundled with the
/// marks and the element library.
const SCRIPT: &str = include_str!("report.js");
/// The script's SHA-256, in base64: the content security policy lets only
/// it run, so nothing in a page's HTML (an event handler, a `javascript:`
/// link) can.
const SCRIPT_HASH: &str = include_str!("report.js.sha256");
/// The report's stylesheet: the element library's, the marks', and the
/// report's own.
const STYLE: &str = include_str!("report.css");

/// How many changed pages a report renders. The rest are listed by name, so
/// a report of a large change still opens.
pub const MAX_PAGES: usize = 300;

/// The largest image a report includes, in bytes. A larger one is shown as
/// a placeholder naming its file.
pub const MAX_IMAGE_BYTES: usize = 1024 * 1024;

/// The files of one version of a project that pages use as images.
pub trait AssetFiles {
    /// The contents of the file at a content path, or `None` when there's
    /// none.
    fn read(&self, content_path: &RelPath) -> Option<Vec<u8>>;
}

/// A project's files on disk, under its content root.
pub struct DiskAssets {
    content_dir: PathBuf,
}

impl DiskAssets {
    /// The files of the project whose `ascribe.toml` is in `project_root`.
    pub fn new(project_root: &Path, project: &Project) -> DiskAssets {
        DiskAssets {
            content_dir: project_root.join(project.layout().content_root.as_str()),
        }
    }
}

impl AssetFiles for DiskAssets {
    fn read(&self, content_path: &RelPath) -> Option<Vec<u8>> {
        fs::read(self.content_dir.join(content_path.as_str())).ok()
    }
}

/// A project's files at a revision, read from git.
pub struct GitAssets<'a> {
    repo: &'a Repository,
    fs: &'a GitFs,
}

impl<'a> GitAssets<'a> {
    /// The files of `fs`, read through `repo`.
    pub fn new(repo: &'a Repository, fs: &'a GitFs) -> GitAssets<'a> {
        GitAssets { repo, fs }
    }
}

impl AssetFiles for GitAssets<'_> {
    fn read(&self, content_path: &RelPath) -> Option<Vec<u8>> {
        let object = self.fs.object(content_path)?;
        self.repo.read_blobs(&[object]).ok()?.pop()
    }
}

/// One version of a project, for rendering its pages.
#[derive(Clone, Copy)]
pub struct Version<'a> {
    /// The project's source index.
    pub project: &'a Project,
    /// Its files.
    pub files: &'a dyn AssetFiles,
}

/// The data the report's script draws from. Its keys, like the JSON
/// report's, are snake_case.
#[derive(Serialize)]
struct Data<'a> {
    ascribe_version: &'static str,
    base: &'a BaseInfo,
    builds: Vec<BuildData<'a>>,
    pages: BTreeMap<String, Rendered>,
    images: BTreeMap<String, String>,
    limit: Limit,
    image_limit: usize,
}

#[derive(Serialize)]
struct Limit {
    pages: usize,
    omitted: usize,
}

#[derive(Serialize)]
struct BuildData<'a> {
    build: &'a str,
    pages: Vec<PageData<'a>>,
}

#[derive(Serialize)]
struct PageData<'a> {
    #[serde(flatten)]
    diff: &'a PageDiff,
    title: Option<String>,
    /// The page now and before, as keys of `pages`.
    now: Option<String>,
    was: Option<String>,
    /// Whether it's beyond the limit, so not rendered.
    omitted: bool,
}

/// A page rendered: its HTML, and what each image reference in it is.
#[derive(Clone, PartialEq, Serialize)]
struct Rendered {
    html: String,
    images: BTreeMap<String, ImageRef>,
}

#[derive(Clone, PartialEq, Serialize)]
struct ImageRef {
    /// The source file.
    path: String,
    /// Its key in `images`, when it's included.
    #[serde(skip_serializing_if = "Option::is_none")]
    image: Option<String>,
    /// Its size, when it's too large to include.
    #[serde(skip_serializing_if = "Option::is_none")]
    bytes: Option<usize>,
}

/// Each rendered page and image once, however many builds and pages share
/// it.
#[derive(Default)]
struct Store {
    pages: BTreeMap<String, Rendered>,
    page_keys: Vec<(Rendered, String)>,
    images: BTreeMap<String, String>,
    image_keys: HashMap<String, String>,
}

impl Store {
    fn page(&mut self, rendered: Rendered) -> String {
        if let Some((_, key)) = self.page_keys.iter().find(|(r, _)| *r == rendered) {
            return key.clone();
        }
        let key = format!("p{}", self.page_keys.len() + 1);
        self.pages.insert(key.clone(), rendered.clone());
        self.page_keys.push((rendered, key.clone()));
        key
    }

    fn image(&mut self, url: String) -> String {
        if let Some(key) = self.image_keys.get(&url) {
            return key.clone();
        }
        let key = format!("i{}", self.image_keys.len() + 1);
        self.images.insert(key.clone(), url.clone());
        self.image_keys.insert(url, key.clone());
        key
    }
}

/// Writes `report` as the static report: every changed page of every build,
/// rendered from `now` and, unless the project is new, from `base`, up to
/// [`MAX_PAGES`] pages.
pub fn write_html(report: &Report, base: Option<Version<'_>>, now: Version<'_>) -> String {
    write_html_with(report, base, now, MAX_PAGES)
}

/// [`write_html`], rendering at most `max_pages` pages.
pub fn write_html_with(
    report: &Report,
    base: Option<Version<'_>>,
    now: Version<'_>,
    max_pages: usize,
) -> String {
    let mut store = Store::default();
    let mut rendered_pages = 0;
    let mut omitted = 0;
    let mut builds = Vec::new();
    for build in &report.builds {
        let mut pages = Vec::new();
        for diff in &build.pages {
            let mut data = PageData {
                diff,
                title: None,
                now: None,
                was: None,
                omitted: false,
            };
            if rendered_pages >= max_pages {
                data.omitted = true;
                omitted += 1;
                pages.push(data);
                continue;
            }
            rendered_pages += 1;
            if let Ok(path) = RelPath::parse(&diff.path) {
                if diff.status != PageStatus::Removed
                    && let Some((rendered, title)) = render(now, &build.build, &path, &mut store)
                {
                    data.title = title;
                    data.now = Some(store.page(rendered));
                }
                if diff.status != PageStatus::Added
                    && let Some(base) = base
                    && let Some((rendered, title)) = render(base, &build.build, &path, &mut store)
                {
                    data.title = data.title.or(title);
                    data.was = Some(store.page(rendered));
                }
            }
            pages.push(data);
        }
        builds.push(BuildData {
            build: &build.build,
            pages,
        });
    }
    let paths: std::collections::BTreeSet<&str> = report
        .builds
        .iter()
        .flat_map(|b| b.pages.iter().map(|p| p.path.as_str()))
        .collect();
    let in_builds = report.builds.iter().filter(|b| !b.pages.is_empty()).count();
    let data = Data {
        ascribe_version: report.ascribe_version,
        base: &report.base,
        builds,
        pages: store.pages,
        images: store.images,
        limit: Limit {
            pages: max_pages,
            omitted,
        },
        image_limit: MAX_IMAGE_BYTES,
    };
    let json = serde_json::to_string(&data).unwrap_or_else(|_| "{}".to_owned());
    let mut title = match paths.len() {
        0 => "Review: no changes".to_owned(),
        1 => "Review: 1 changed page".to_owned(),
        n => format!("Review: {n} changed pages"),
    };
    if in_builds > 1 {
        title.push_str(&format!(" in {in_builds} builds"));
    }
    page_shell(&title, &script_json(&json))
}

/// The HTML file around the data.
fn page_shell(title: &str, json: &str) -> String {
    format!(
        "<!doctype html>\n\
         <html lang=\"en\">\n\
         <head>\n\
         <meta charset=\"utf-8\">\n\
         <meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">\n\
         <meta http-equiv=\"Content-Security-Policy\" content=\"default-src 'none'; img-src data:; style-src 'unsafe-inline'; script-src 'sha256-{SCRIPT_HASH}'\">\n\
         <meta name=\"generator\" content=\"Ascribe {version}\">\n\
         <title>{title}</title>\n\
         <style>\n{STYLE}</style>\n\
         </head>\n\
         <body>\n\
         <div id=\"ascribe-review\"><noscript>This report draws its pages with JavaScript; turn it on to see them.</noscript></div>\n\
         <script type=\"application/json\" id=\"ascribe-review-data\">{json}</script>\n\
         <script>{SCRIPT}</script>\n\
         </body>\n\
         </html>\n",
        version = env!("CARGO_PKG_VERSION"),
        title = escape_text(title),
    )
}

/// JSON safe inside a `<script>`: `<` is only ever in a string, where
/// `\u003c` means the same, so nothing can close the element or open a
/// comment.
fn script_json(json: &str) -> String {
    json.replace('<', "\\u003c")
}

fn escape_text(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

/// One page of one build of `version`, rendered with anchors, and its title.
fn render(
    version: Version<'_>,
    build_name: &str,
    path: &RelPath,
    store: &mut Store,
) -> Option<(Rendered, Option<String>)> {
    let project = version.project;
    let model = project.model();
    let build = model.build(build_name)?;
    let router = AstroRouter::from_consumer(&model.consumer);
    let page = project.resolver(build, &router).page(path)?;
    let emitter = SiteEmitter::new(model).with_anchors(true);
    let cx = EmitContext::new(project, Path::new(""), build);
    let emitted = emit_page(&emitter, &cx, &page).ok()?;
    let html = render_site_html(without_frontmatter(&emitted.text));
    let mut images = BTreeMap::new();
    for placed in &emitted.assets {
        if placed.usage != AssetUse::Image {
            continue;
        }
        let reference = encode_path(&placed.placement.reference);
        if images.contains_key(&reference) {
            continue;
        }
        let mut image = ImageRef {
            path: placed.source.to_string(),
            image: None,
            bytes: None,
        };
        if let Some(contents) = version.files.read(&placed.source) {
            if contents.len() > MAX_IMAGE_BYTES {
                image.bytes = Some(contents.len());
            } else {
                image.image = Some(store.image(data_url(&placed.source, &contents)));
            }
        }
        images.insert(reference, image);
    }
    Some((Rendered { html, images }, page.title.clone()))
}

/// The site output's markdown after its frontmatter (`---`, YAML, `---`).
fn without_frontmatter(text: &str) -> &str {
    let Some(rest) = text.strip_prefix("---\n") else {
        return text;
    };
    // The YAML serializer indents every continuation line, so a line that is
    // exactly `---` closes the block.
    let close = if rest.starts_with("---\n") {
        Some(0)
    } else {
        rest.find("\n---\n").map(|i| i + 1)
    };
    match close {
        Some(at) => rest[at + 4..].trim_start_matches('\n'),
        None => text,
    }
}

/// A `data:` URL of a file, typed by its extension.
fn data_url(path: &RelPath, contents: &[u8]) -> String {
    let extension = path.extension().unwrap_or_default().to_ascii_lowercase();
    let mime = match extension.as_str() {
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "svg" => "image/svg+xml",
        "webp" => "image/webp",
        "avif" => "image/avif",
        "ico" => "image/x-icon",
        "bmp" => "image/bmp",
        _ => "application/octet-stream",
    };
    format!("data:{mime};base64,{}", base64(contents))
}

/// Standard base64, with padding.
fn base64(bytes: &[u8]) -> String {
    const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity(bytes.len().div_ceil(3) * 4);
    for chunk in bytes.chunks(3) {
        let b = [
            chunk[0],
            chunk.get(1).copied().unwrap_or(0),
            chunk.get(2).copied().unwrap_or(0),
        ];
        let n = (u32::from(b[0]) << 16) | (u32::from(b[1]) << 8) | u32::from(b[2]);
        for i in 0..4 {
            if i <= chunk.len() {
                out.push(char::from(ALPHABET[((n >> (18 - 6 * i)) & 63) as usize]));
            } else {
                out.push('=');
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::{SCRIPT, STYLE, base64, script_json, without_frontmatter};

    #[test]
    fn base64_matches_the_standard() {
        assert_eq!(base64(b""), "");
        assert_eq!(base64(b"f"), "Zg==");
        assert_eq!(base64(b"fo"), "Zm8=");
        assert_eq!(base64(b"foo"), "Zm9v");
        assert_eq!(base64(b"foobar"), "Zm9vYmFy");
        assert_eq!(base64(&[0xff, 0xfe, 0xfd]), "//79");
    }

    #[test]
    fn json_cant_close_its_script() {
        assert_eq!(
            script_json(r#"{"a":"</script><!--"}"#),
            r#"{"a":"\u003c/script>\u003c!--"}"#
        );
    }

    #[test]
    fn the_embedded_files_cant_close_their_elements() {
        assert!(!SCRIPT.to_ascii_lowercase().contains("</script"));
        assert!(!STYLE.to_ascii_lowercase().contains("</style"));
    }

    #[test]
    fn frontmatter_is_dropped() {
        assert_eq!(without_frontmatter("---\ntitle: A\n---\n\n# B\n"), "# B\n");
        assert_eq!(without_frontmatter("# B\n"), "# B\n");
    }
}
