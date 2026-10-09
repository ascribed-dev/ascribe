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

use ascribe_core::{AssetUse, RelPath, names};
use ascribe_emit::assets::encode_path;
use ascribe_emit::{
    EmitContext, FormattedPiece, SiteEmitter, emit_page, formatted_title, render_site_html,
};
use ascribe_resolve::{AstroRouter, Project};
use serde::Serialize;

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
        // Outside FileSystem: an image is read from where the build copies
        // it (`EmitContext::asset_source`), so the report shows what the
        // build writes. The build's asset copies don't go through
        // FileSystem either; moving both is a behavior change, not this
        // clean-up's.
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
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
struct Data<'a> {
    /// The version of Ascribe that wrote it.
    ascribe_version: &'static str,
    /// What was compared with.
    base: &'a BaseInfo,
    /// How many errors `ascribe check` finds in the working tree.
    working_tree_errors: usize,
    /// What changed, per build.
    builds: Vec<BuildData<'a>>,
    /// Each rendered page, once however many builds render it alike.
    pages: BTreeMap<String, RenderedPage>,
    /// Each image, as a `data:` URL, once however many pages use it.
    images: BTreeMap<String, String>,
    /// How many changed pages the report renders, and how many it left out.
    limit: Limit,
    /// The size, in bytes, above which an image isn't included.
    image_limit: usize,
}

/// The schema of the data the report's script reads, generated into
/// `@ascribed/review`'s TypeScript by `crates/ascribe-cli/src/shapes.rs`.
#[cfg(feature = "json-schema")]
pub fn data_schema(generator: &mut schemars::SchemaGenerator) -> schemars::Schema {
    generator.root_schema_for::<Data<'static>>()
}

/// How many changed pages a report renders, and how many it left out.
#[derive(Serialize)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
struct Limit {
    /// The most it renders.
    pages: usize,
    /// How many it left out.
    omitted: usize,
}

/// One build's changed pages.
#[derive(Serialize)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
struct BuildData<'a> {
    /// The build's name.
    build: &'a str,
    /// Its changed pages, in path order.
    pages: Vec<PageData<'a>>,
}

/// One changed page: `ascribe diff --format json`'s, and its renderings.
#[derive(Serialize)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
struct PageData<'a> {
    #[serde(flatten)]
    diff: &'a PageDiff,
    /// Its title, when the build has one.
    title: Option<String>,
    /// Its title formatted, when its field sets `inline = "code"`: for the
    /// page list and the page's heading. `title` stays the plain text.
    formatted_title: Option<Vec<FormattedPiece>>,
    /// The page now, as a key of `pages`; `null` when there's none.
    now: Option<String>,
    /// The page before, as a key of `pages`; `null` when there's none.
    was: Option<String>,
    /// Whether it's beyond the limit, so not rendered.
    omitted: bool,
}

/// A page rendered: its HTML, and what each image reference in it is.
#[derive(Clone, PartialEq, Serialize)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
struct RenderedPage {
    /// The page's content, rendered with source anchors.
    html: String,
    /// Each image reference the HTML writes, and what it is.
    images: BTreeMap<String, ImageRef>,
}

/// An image a page refers to.
#[derive(Clone, PartialEq, Serialize)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
struct ImageRef {
    /// The image's source file.
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
    pages: BTreeMap<String, RenderedPage>,
    page_keys: Vec<(RenderedPage, String)>,
    images: BTreeMap<String, String>,
    image_keys: HashMap<String, String>,
}

impl Store {
    fn page(&mut self, rendered: RenderedPage) -> String {
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
                formatted_title: None,
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
                    (data.title, data.formatted_title) = (title.plain, title.formatted);
                    data.now = Some(store.page(rendered));
                }
                if diff.status != PageStatus::Added
                    && let Some(base) = base
                    && let Some((rendered, title)) = render(base, &build.build, &path, &mut store)
                {
                    if data.title.is_none() {
                        (data.title, data.formatted_title) = (title.plain, title.formatted);
                    }
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
        working_tree_errors: report.working_tree_errors,
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
         <div id=\"{root}\"><noscript>This report draws its pages with JavaScript; turn it on to see them.</noscript></div>\n\
         <script type=\"application/json\" id=\"{data}\">{json}</script>\n\
         <script>{SCRIPT}</script>\n\
         </body>\n\
         </html>\n",
        version = env!("CARGO_PKG_VERSION"),
        title = escape_text(title),
        root = names::ID_REPORT,
        data = names::ID_REPORT_DATA,
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

/// A page's title: as text, and formatted when its field sets `inline`.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct PageTitle {
    /// The title as text, for tooltips and search; `None` when the page has
    /// none.
    pub plain: Option<String>,
    /// The title formatted, for a heading or a list entry; `None` when its
    /// field doesn't set `inline`.
    pub formatted: Option<Vec<FormattedPiece>>,
}

/// One page of one build of `version`, rendered with anchors, and its title.
fn render(
    version: Version<'_>,
    build_name: &str,
    path: &RelPath,
    store: &mut Store,
) -> Option<(RenderedPage, PageTitle)> {
    let (emitted, title) = emit(version.project, build_name, path)?;
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
    Some((RenderedPage { html, images }, title))
}

/// The page at `path` of the build `build_name` of `project`, rendered as the
/// page preview renders it, with source anchors, and its title: the HTML
/// [`render_site_html`] makes of the site output, without the frontmatter
/// and without a layout. `None` when the build doesn't publish the page.
pub fn page_html(
    project: &Project,
    build_name: &str,
    path: &RelPath,
) -> Option<(String, PageTitle)> {
    let (emitted, title) = emit(project, build_name, path)?;
    Some((render_site_html(without_frontmatter(&emitted.text)), title))
}

/// The site output of one page, with anchors, and its title.
fn emit(
    project: &Project,
    build_name: &str,
    path: &RelPath,
) -> Option<(ascribe_emit::EmittedPage, PageTitle)> {
    let model = project.model();
    let build = model.build(build_name)?;
    let router = AstroRouter::from_consumer(&model.consumer);
    let page = project.resolver(build, &router).page(path)?;
    let emitter = SiteEmitter::new(model).with_anchors(true);
    let cx = EmitContext::new(project, Path::new(""), build);
    let emitted = emit_page(&emitter, &cx, &page).ok()?;
    let title = PageTitle {
        plain: page.title.clone(),
        formatted: formatted_title(&page),
    };
    Some((emitted, title))
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
    let mime = path
        .extension()
        .and_then(ascribe_core::image_media_type)
        .unwrap_or("application/octet-stream");
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
