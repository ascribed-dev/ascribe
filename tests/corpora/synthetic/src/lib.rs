//! The synthetic project the performance tests share.
//!
//! One generator, so the benchmarks (`incremental`, `keystroke`, `completion`,
//! and `check` and `build` in `perf`) measure the same
//! project and their numbers can be compared. It returns plain strings and has
//! no dependencies, so any crate can use it as a dev-dependency.
//!
//! The project: `pages` pages in [`SECTIONS`] sections (`s0/p0.md`,
//! `s1/p1.md`, …), [`FRAGMENTS`] fragments in `_f/`, and [`IMAGES`] images in
//! `img/`, under the content root `docs`. Every page has frontmatter, a title,
//! a paragraph with a phrase, two more headings, an include of one fragment
//! (each fragment is included by `pages / FRAGMENTS` pages), three links to
//! other pages (one of them to a heading), and an image; one page in ten also
//! has a section that only some deployments see (`@available: cloud`). It has
//! no diagnostics of any severity, so checking it measures the work and not the
//! reporting; the `ascribe-corpora` tests check that.
//!
//! [`Synthetic::with_snippets`] adds a source, `code/`, of [`CODE_FILES`]
//! tagged Python files beside the content, and a `@snippet` of one of their
//! regions to every page, for measuring what snippets cost.
//!
//! ```
//! use ascribe_synthetic::Synthetic;
//!
//! let project = Synthetic::new(3000);
//! assert_eq!(project.page_path(1234), "s4/p1234.md");
//! assert!(project.page_text(1234, "typed: ").contains("@include: /_f/f34.md"));
//! ```

use std::fmt::Write as _;
use std::io;
use std::path::Path;

/// Sections (directories) the pages are spread over.
pub const SECTIONS: usize = 30;
/// Fragments, whatever the number of pages.
pub const FRAGMENTS: usize = 100;
/// Images, whatever the number of pages.
pub const IMAGES: usize = 60;
/// The number of pages of the standard performance project.
pub const STANDARD_PAGES: usize = 3000;
/// Code files, with snippets, whatever the number of pages.
pub const CODE_FILES: usize = 100;
/// Regions in each code file.
pub const REGIONS: usize = 3;

/// The project's `ascribe.toml`: one phrase, one dimension, and a site build
/// that uses variant switching and availability badges.
pub const MODEL: &str = r#"spec = "0.1"

[project]
content-root = "docs"

[phrases]
product = "Quill"

[dimensions.deployment]
values = ["cloud", "self-managed"]
versionless = ["cloud"]

[builds.site]
variants = "switch"
availability = "badge"
"#;

/// The content root of [`MODEL`], relative to the project root.
pub const CONTENT_ROOT: &str = "docs";

/// The zero-based line of a page's paragraph that ends with the `tail`
/// [`Synthetic::page_text`] was given: where a benchmark types.
pub const TYPING_LINE: u32 = 6;

/// The text before the tail on [`TYPING_LINE`].
pub const TYPING_PREFIX: &str = "Intro for {product}, ";

/// The end of the intro paragraph of a page that isn't being edited.
pub const DEFAULT_TAIL: &str = "welcome.";

/// The body of a fragment unless a benchmark writes another.
pub const FRAGMENT_BODY: &str = "Shared text.";

/// One generated file of the project.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct File {
    /// The path from the project root, `/`-separated (`docs/s0/p0.md`).
    pub path: String,
    /// The text (empty for images).
    pub text: String,
}

/// A synthetic project of some number of pages.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Synthetic {
    /// The number of pages.
    pub pages: usize,
    /// Whether every page has a snippet, from a source of code files.
    pub snippets: bool,
}

impl Synthetic {
    /// A project of `pages` pages (at least one).
    pub fn new(pages: usize) -> Synthetic {
        Synthetic {
            pages: pages.max(1),
            snippets: false,
        }
    }

    /// The same project with a source of code files, and a snippet on every
    /// page.
    pub fn with_snippets(self) -> Synthetic {
        Synthetic {
            snippets: true,
            ..self
        }
    }

    /// The project's `ascribe.toml`: [`MODEL`], with the source when the
    /// pages have snippets.
    pub fn model(&self) -> String {
        if self.snippets {
            format!("{MODEL}\n[sources.code]\npath = \"code\"\ninclude = [\"**/*.py\"]\n")
        } else {
            MODEL.to_owned()
        }
    }

    /// Code file `i`'s path relative to the project root.
    pub fn code_path(&self, i: usize) -> String {
        format!("code/m{i}.py")
    }

    /// Code file `i`'s text: [`REGIONS`] tagged regions, `r0`, `r1`, …, each
    /// a function with a line left out, and `body` as the first one's last
    /// line.
    pub fn code_text(&self, i: usize, body: &str) -> String {
        let mut t = format!("import os\n\n\nclass Module{i}:\n");
        for r in 0..REGIONS {
            let last = if r == 0 { body } else { "return value" };
            let _ = write!(
                t,
                "    # :snippet-start: r{r}\n    def step{r}(self, value):\n        \
                 log(value)  # :remove:\n        value = value + {r}\n        {last}\n    \
                 # :snippet-end:\n\n"
            );
        }
        t
    }

    /// The standard 3,000-page project.
    pub fn standard() -> Synthetic {
        Synthetic::new(STANDARD_PAGES)
    }

    /// Pages that include each fragment (the first ones, when the pages don't
    /// divide evenly).
    pub fn includers_per_fragment(&self) -> usize {
        self.pages / FRAGMENTS
    }

    /// Page `i`'s path relative to the content root.
    pub fn page_path(&self, i: usize) -> String {
        format!("s{}/p{i}.md", i % SECTIONS)
    }

    /// Page `i`'s text. Line [`TYPING_LINE`] is `TYPING_PREFIX` and `tail`, so
    /// a benchmark that types there finds it; a benchmark that edits the
    /// page without typing gives a revision number as the tail.
    pub fn page_text(&self, i: usize, tail: &str) -> String {
        let mut t = String::new();
        let _ = write!(
            t,
            "---\ntitle: Page {i}\n---\n\n## Page {i}\n\n{TYPING_PREFIX}{tail}\n\n"
        );
        let _ = write!(
            t,
            "## Setup\n\nSteps for page {i}.\n\n## Usage\n\nMore.\n\n"
        );
        let _ = write!(t, "@include: /_f/f{}.md\n\n", i % FRAGMENTS);
        if self.snippets {
            let _ = write!(
                t,
                "@snippet: code:m{}.py#r{}\n\n",
                i % CODE_FILES,
                i % REGIONS
            );
        }
        for k in 1..=3 {
            let target = (i * 7 + k * 131) % self.pages;
            if k == 1 {
                let _ = writeln!(t, "See [the setup](/{}#setup).", self.page_path(target));
            } else {
                let _ = writeln!(t, "See [page {target}](/{}).", self.page_path(target));
            }
        }
        let _ = write!(t, "\n![diagram](/img/i{}.png)\n", i % IMAGES);
        if i.is_multiple_of(10) {
            t.push_str("\n## Cloud only\n@available: cloud\n\nCloud text.\n");
        }
        t
    }

    /// Fragment `i`'s path relative to the content root.
    pub fn fragment_path(&self, i: usize) -> String {
        format!("_f/f{i}.md")
    }

    /// Fragment `i`'s text, with `body` as its paragraph.
    pub fn fragment_text(&self, i: usize, body: &str) -> String {
        format!("## Shared {i}\n\n{body}\n")
    }

    /// Image `i`'s path relative to the content root.
    pub fn image_path(&self, i: usize) -> String {
        format!("img/i{i}.png")
    }

    /// The pages' texts, as `(path relative to the content root, text)`, with
    /// the text each page has when it isn't being edited.
    pub fn pages_text(&self) -> impl Iterator<Item = (String, String)> + '_ {
        (0..self.pages).map(|i| (self.page_path(i), self.page_text(i, DEFAULT_TAIL)))
    }

    /// The Markdown sources, pages and fragments, as
    /// `(path relative to the content root, text)`.
    pub fn sources(&self) -> impl Iterator<Item = (String, String)> + '_ {
        let fragments =
            (0..FRAGMENTS).map(|i| (self.fragment_path(i), self.fragment_text(i, FRAGMENT_BODY)));
        self.pages_text().chain(fragments)
    }

    /// Every file of the project from its root: `ascribe.toml`, the sources
    /// and the images, under [`CONTENT_ROOT`], and any code files.
    pub fn files(&self) -> impl Iterator<Item = File> + '_ {
        let model = File {
            path: "ascribe.toml".into(),
            text: self.model(),
        };
        let code = (0..if self.snippets { CODE_FILES } else { 0 }).map(|i| File {
            path: self.code_path(i),
            text: self.code_text(i, "return value"),
        });
        let sources = self.sources().map(|(path, text)| File {
            path: format!("{CONTENT_ROOT}/{path}"),
            text,
        });
        let images = (0..IMAGES).map(|i| File {
            path: format!("{CONTENT_ROOT}/{}", self.image_path(i)),
            text: String::new(),
        });
        std::iter::once(model)
            .chain(sources)
            .chain(images)
            .chain(code)
    }

    /// Writes the project under `root` (which must exist).
    ///
    /// # Errors
    ///
    /// Any error creating a directory or writing a file.
    pub fn write_to(&self, root: &Path) -> io::Result<()> {
        for file in self.files() {
            let path = root.join(&file.path);
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            std::fs::write(path, file.text)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_page_types_on_the_typing_line() {
        let text = Synthetic::new(10).page_text(3, "typed: ");
        let line = text.lines().nth(TYPING_LINE as usize).unwrap_or_default();
        assert_eq!(line, "Intro for {product}, typed: ");
    }

    #[test]
    fn every_link_names_a_page_that_exists() {
        let project = Synthetic::new(50);
        let paths: Vec<String> = (0..50).map(|i| project.page_path(i)).collect();
        for i in 0..50 {
            for line in project.page_text(i, DEFAULT_TAIL).lines() {
                if let Some(rest) = line.split("](/").nth(1) {
                    let target = rest.split([')', '#']).next().unwrap_or_default();
                    if target.ends_with(".md") {
                        assert!(paths.iter().any(|p| p == target), "{target}");
                    }
                }
            }
        }
    }

    #[test]
    fn the_standard_project_has_the_size_the_benchmarks_say() {
        let project = Synthetic::standard();
        assert_eq!(project.sources().count(), STANDARD_PAGES + FRAGMENTS);
        assert_eq!(
            project.files().count(),
            1 + STANDARD_PAGES + FRAGMENTS + IMAGES
        );
        assert_eq!(project.includers_per_fragment(), 30);
        let snippets = project.with_snippets();
        assert_eq!(
            snippets.files().count(),
            1 + STANDARD_PAGES + FRAGMENTS + IMAGES + CODE_FILES
        );
        assert!(
            snippets
                .page_text(1234, DEFAULT_TAIL)
                .contains("@snippet: code:m34.py#r1")
        );
    }
}

/// Recording benchmark results for the regression check.
///
/// A benchmark calls [`record`] with its timings. When `ASCRIBE_BENCH_OUT`
/// names a file, one JSON line per metric is appended to it; the `corpora
/// compare` command reads those lines and compares them with the recorded
/// baselines (`tests/corpora/baselines/perf.json`). With the variable unset
/// nothing happens, so a benchmark run by hand only prints.
pub mod report {
    use std::io::Write as _;
    use std::time::Duration;

    /// Appends the median and 95th percentile of `times` (which is sorted) as
    /// the metric `name`. Failures to write are ignored: a benchmark's
    /// numbers matter more than its bookkeeping.
    pub fn record(name: &str, times: &mut [Duration]) {
        let mut ms: Vec<f64> = times.iter().map(|t| t.as_secs_f64() * 1000.0).collect();
        append(name, &mut ms);
    }

    /// Appends the median and 95th percentile of `megabytes` as the metric
    /// `name`, which starts with `memory/`. The line has the same fields as a
    /// time's, and `corpora compare` treats it the same way; only the unit
    /// differs.
    pub fn record_megabytes(name: &str, megabytes: &mut [f64]) {
        append(name, megabytes);
    }

    fn append(name: &str, values: &mut [f64]) {
        let Some(path) = std::env::var_os("ASCRIBE_BENCH_OUT") else {
            return;
        };
        if values.is_empty() {
            return;
        }
        values.sort_by(f64::total_cmp);
        let at = |q: f64| values[((values.len() as f64 - 1.0) * q).round() as usize];
        let line = format!(
            "{{\"name\":\"{name}\",\"median_ms\":{:.4},\"p95_ms\":{:.4},\"runs\":{}}}\n",
            at(0.5),
            at(0.95),
            values.len()
        );
        if let Ok(mut file) = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
        {
            let _ = file.write_all(line.as_bytes());
        }
    }
}
