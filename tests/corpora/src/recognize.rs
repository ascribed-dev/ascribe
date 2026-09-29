//! Recognition checks: what Ascribe's parser and file-level checks find in
//! documentation that was never written for Ascribe.
//!
//! The corpora are plain Markdown (or MDX, read as Markdown), so nothing in
//! them is meant as a directive, a title line, a phrase, or an image attribute
//! block. Whatever the parser recognizes is therefore either a false positive
//! or a documented consequence of the specification (for instance SPEC §5.1's
//! warning about `{key}` text). [`recognize`] lists every such recognition as a
//! [`Finding`] with a *class*; `tests/corpora/recognition.toml` records the
//! verdict for each class, and the tests fail on a class with no verdict.
//!
//! Classes:
//!
//! - `directive:<name>`, `end-line`, `container:<name>`, `group:<name>`,
//!   `title`: recognized Ascribe lines and structures.
//! - `phrase-candidate:prose|destination|fence`, `escaped-phrase`,
//!   `image-attributes`: recognized inline constructs.
//! - `diagnostic:<slug>`: what `check_file` reports, other than
//!   [`OUT_OF_SCOPE`].

use std::collections::BTreeMap;
use std::path::PathBuf;

use serde::Serialize;
use tessera_check::{Project, SourceFile, check_file};
use tessera_core::{FileId, LineCol, LineIndex, RelPath, Span};
use tessera_syntax::{
    Block, BlockKind, DirectiveLine, Inline, InlineKind, ParseOptions, PrimaryValue, parse,
};

/// Diagnostics that are about a project's files existing, not about what the
/// parser recognized: a corpus is checked as loose pages with no assets and no
/// includes, so they say nothing about recognition.
pub const OUT_OF_SCOPE: &[&str] = &[
    "link-target-missing",
    "link-id-missing",
    "link-to-fragment",
    "link-id-in-fragment",
    "link-route",
    "image-source-missing",
    "include-target-missing",
    "include-id-missing",
    // Frontmatter is checked against the project's content types, which an
    // empty project doesn't declare; that's configuration, not recognition.
    "frontmatter-unknown-key",
    "frontmatter-missing-field",
    "frontmatter-type-mismatch",
    "content-type-unresolved",
];

/// One recognition.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Finding {
    /// What was recognized (see the module documentation).
    pub class: String,
    /// The page, relative to the corpus's content directory.
    pub file: String,
    /// The 1-based line.
    pub line: u32,
    /// The 1-based column, in bytes.
    pub column: u32,
    /// The source line, for a person reading a report (never stored).
    pub excerpt: String,
    /// The construct's name or key (`note`, `es`), when it has one.
    pub detail: String,
}

/// What [`recognize`] found.
#[derive(Clone, Debug, Default)]
pub struct Report {
    /// Pages read.
    pub files: usize,
    /// Bytes of source.
    pub bytes: usize,
    /// Every recognition, in page and source order.
    pub findings: Vec<Finding>,
    /// Pages that couldn't be given a path (not a valid content path).
    pub skipped: Vec<String>,
}

impl Report {
    /// How many findings there are of each class.
    pub fn counts(&self) -> BTreeMap<String, usize> {
        let mut counts = BTreeMap::new();
        for f in &self.findings {
            *counts.entry(f.class.clone()).or_insert(0) += 1;
        }
        counts
    }

    /// How many pages have at least one finding of each class.
    pub fn pages_per_class(&self) -> BTreeMap<String, usize> {
        let mut seen: BTreeMap<&str, Vec<&str>> = BTreeMap::new();
        for f in &self.findings {
            let files = seen.entry(&f.class).or_default();
            if files.last() != Some(&f.file.as_str()) {
                files.push(&f.file);
            }
        }
        seen.into_iter()
            .map(|(class, mut files)| {
                files.sort_unstable();
                files.dedup();
                (class.to_owned(), files.len())
            })
            .collect()
    }

    /// The findings of a class.
    pub fn of_class<'a>(&'a self, class: &'a str) -> impl Iterator<Item = &'a Finding> + 'a {
        self.findings.iter().filter(move |f| f.class == class)
    }
}

/// The Markdown path a page is checked under: `.mdx` is read as Markdown.
fn markdown_path(page: &str) -> String {
    match page.rsplit_once('.') {
        Some((stem, "md")) => format!("{stem}.md"),
        Some((stem, _)) => format!("{stem}.md"),
        None => format!("{page}.md"),
    }
}

/// Parses every page with the built-in directives only (no project widgets, no
/// declared phrases: the project is empty), and lists what is recognized.
pub fn recognize(pages: &[(String, String)]) -> Report {
    let mut report = Report::default();
    let model = match tessera_model::load_str("spec = \"0.1\"\n", FileId::new(0)) {
        Ok(model) => model,
        Err(_) => return report,
    };
    let mut sources = Vec::new();
    let mut names: Vec<(String, String)> = Vec::new();
    for (path, text) in pages {
        let md = markdown_path(path);
        let Ok(rel) = RelPath::parse(&md) else {
            report.skipped.push(path.clone());
            continue;
        };
        if sources.iter().any(|s: &SourceFile| s.path == rel) {
            report.skipped.push(path.clone());
            continue;
        }
        sources.push(SourceFile {
            id: FileId::new(sources.len() as u32 + 1),
            path: rel,
            text: text.clone(),
            unreadable: None,
        });
        names.push((path.clone(), text.clone()));
    }
    let project = Project::from_parts(
        PathBuf::from("."),
        RelPath::root(),
        model,
        String::new(),
        sources,
    );
    for (source, (name, text)) in project.sources().iter().zip(&names) {
        report.files += 1;
        report.bytes += text.len();
        let lines = LineIndex::new(text);
        let mut found = Vec::new();
        let doc = parse(text, &ParseOptions::new(tessera_core::builtin_schemas()));
        let mut walker = Walker {
            out: &mut found,
            text,
            lines: &lines,
        };
        walker.blocks(&doc.blocks);
        for phrase in &doc.escaped_phrases {
            walker.add("escaped-phrase", phrase.span);
        }
        for definition in &doc.definitions {
            for phrase in &definition.destination_phrases {
                walker.add("phrase-candidate:destination", phrase.span);
            }
        }
        for d in check_file(&project, source) {
            if OUT_OF_SCOPE.contains(&d.slug.as_str()) {
                continue;
            }
            let class = format!("diagnostic:{}", d.slug.as_str());
            if d.slug.as_str() == "phrase-undeclared" {
                // The same classes as the candidate it's about.
                let key = text
                    .get(d.location.span.range())
                    .unwrap_or("")
                    .trim_matches(['{', '}'])
                    .to_owned();
                walker.diagnostic_phrase(&class, d.location.span, &key);
            } else {
                walker.add(&class, d.location.span);
            }
        }
        found.sort_by(|a, b| (a.line, a.column, &a.class).cmp(&(b.line, b.column, &b.class)));
        for mut f in found {
            f.file = name.clone();
            report.findings.push(f);
        }
    }
    report
}

struct Walker<'a> {
    out: &'a mut Vec<Finding>,
    text: &'a str,
    lines: &'a LineIndex,
}

impl Walker<'_> {
    fn add(&mut self, class: &str, span: Span) {
        self.add_detail(class, span, "");
    }

    /// A phrase candidate in prose, classed by what surrounds it.
    fn phrase(&mut self, key: &str, span: Span) {
        let before = self
            .text
            .get(..span.start())
            .and_then(|t| t.chars().next_back());
        let after = self.text.get(span.end()..).and_then(|t| t.chars().next());
        let at = self.lines.line_col(span.start()).map_or(0, |p| p.line);
        let line = self
            .lines
            .line_span(at)
            .and_then(|s| self.text.get(s.range()))
            .unwrap_or("")
            .trim_start();
        let context = if before == Some('{') && after == Some('}') {
            "mustache"
        } else if line.starts_with(":::") {
            "directive-fence"
        } else {
            "plain"
        };
        self.add_detail(&format!("phrase-candidate:prose:{context}"), span, key);
    }

    fn diagnostic_phrase(&mut self, class: &str, span: Span, key: &str) {
        let before = self
            .text
            .get(..span.start())
            .and_then(|t| t.chars().next_back());
        let after = self.text.get(span.end()..).and_then(|t| t.chars().next());
        let line_start = self
            .text
            .get(..span.start())
            .map_or(0, |t| t.rfind('\n').map_or(0, |n| n + 1));
        let line = self.text.get(line_start..).unwrap_or("").trim_start();
        let context = if before == Some('{') && after == Some('}') {
            "mustache"
        } else if line.starts_with(":::") {
            "directive-fence"
        } else {
            "plain"
        };
        self.add_detail(&format!("{class}:{context}"), span, key);
    }

    fn add_detail(&mut self, class: &str, span: Span, detail: &str) {
        let at = self
            .lines
            .line_col(span.start())
            .unwrap_or(LineCol { line: 0, col: 0 });
        let line = self
            .lines
            .line_span(at.line)
            .and_then(|s| self.text.get(s.range()))
            .unwrap_or("");
        let excerpt: String = line.trim().chars().take(120).collect();
        self.out.push(Finding {
            class: class.to_owned(),
            file: String::new(),
            line: at.line + 1,
            column: at.col + 1,
            excerpt,
            detail: detail.to_owned(),
        });
    }

    fn blocks(&mut self, blocks: &[Block]) {
        for block in blocks {
            self.block(block);
        }
    }

    fn directive(&mut self, line: &DirectiveLine) {
        self.add(&format!("directive:{}", line.name), line.span);
        if let Some(title) = &line.title {
            self.add("title", title.span);
            self.inlines(&title.inlines);
        }
        if let Some(PrimaryValue::Text(primary)) = &line.primary {
            self.inlines(&primary.inlines);
        }
    }

    fn block(&mut self, block: &Block) {
        match &block.kind {
            BlockKind::Heading(h) => self.inlines(&h.inlines),
            BlockKind::Paragraph(p) => self.inlines(&p.inlines),
            BlockKind::CodeBlock(c) => {
                for phrase in c.phrases.iter().flatten() {
                    self.add("phrase-candidate:fence", phrase.span);
                }
            }
            BlockKind::BlockQuote(q) => self.blocks(&q.children),
            BlockKind::List(l) => {
                for item in &l.items {
                    self.blocks(&item.children);
                }
            }
            BlockKind::Table(t) => {
                for row in &t.rows {
                    for cell in &row.cells {
                        self.inlines(&cell.inlines);
                    }
                }
            }
            BlockKind::Directive(line) => self.directive(line),
            BlockKind::End(_) => self.add("end-line", block.span),
            BlockKind::Container(c) => {
                self.add(&format!("container:{}", c.opener.name), block.span);
                self.directive(&c.opener);
                self.blocks(&c.children);
            }
            BlockKind::Group(g) => {
                self.add(&format!("group:{}", g.name), block.span);
                for arm in &g.arms {
                    self.directive(&arm.opener);
                    if let Some(title) = &arm.title {
                        self.add("title", title.span);
                    }
                    self.blocks(&arm.children);
                }
            }
            BlockKind::Title(t) => self.add("title", t.span),
            BlockKind::HtmlBlock(_) | BlockKind::ThematicBreak => {}
        }
    }

    fn inlines(&mut self, inlines: &[Inline]) {
        for inline in inlines {
            match &inline.kind {
                InlineKind::Phrase(p) => self.phrase(&p.key, p.span),
                InlineKind::Emphasis(c) | InlineKind::Strong(c) => self.inlines(c),
                InlineKind::Link(l) => {
                    for p in &l.destination_phrases {
                        self.add("phrase-candidate:destination", p.span);
                    }
                    self.inlines(&l.children);
                }
                InlineKind::Image(i) => {
                    for p in &i.destination_phrases {
                        self.add("phrase-candidate:destination", p.span);
                    }
                    if i.attributes.is_some() {
                        self.add("image-attributes", inline.span);
                    }
                    self.inlines(&i.children);
                }
                InlineKind::Text(_)
                | InlineKind::Code(_)
                | InlineKind::SoftBreak
                | InlineKind::HardBreak
                | InlineKind::Html(_) => {}
            }
        }
    }
}
