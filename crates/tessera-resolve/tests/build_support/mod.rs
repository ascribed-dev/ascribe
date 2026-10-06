//! Helpers for the build-resolution tests: a model with dimensions, a
//! lifecycle, features, phrases, a glossary, and builds; a project in memory;
//! and a compact summary of a resolved page.

#![allow(dead_code, clippy::expect_used, clippy::panic)]

use std::sync::Arc;

use tessera_core::{FileId, RelPath};
use tessera_model::load_str;
use tessera_resolve::{
    DefaultRouter, Layout, MemoryFs, Project, ResolvedBlock, ResolvedKind, ResolvedPage,
};
use tessera_syntax::{BlockKind, Inline, InlineKind, PrimaryValue};

pub const MODEL: &str = r#"
spec = "0.1"

[project]
content-root = "docs"
output-dir = ".ascribe/build"

[types.page]
default = true

[types.page.frontmatter]
title = { type = "string", phrases = true }
description = "string?"
tags = { type = "list(string)?", phrases = true }
note = "string?"

[dimensions.pm]
values = ["npm", "pnpm", "yarn"]

[dimensions.deployment]
values = ["cloud", "self-managed"]
versionless = ["cloud"]

[versions]
scheme = "numeric"

[lifecycle.sunset]
available = false

[features.streaming-sync]
name = "Streaming sync"
available = "cloud, self-managed preview 3.4"

[phrases]
product = "Quill"
cloud = "Quill Cloud"
api = "https://api.quill.dev/v3/"
nested = "{product}"

[glossary]
match = "first"

[glossary.terms.api-key]
term = "API key"
aliases = ["API keys"]
definition = "A secret token."
link = "/glossary.md#api-key"

[glossary.terms.api]
term = "API"
definition = "An interface."
link = "/glossary.md#api"

[glossary.terms.go]
term = "Go"
definition = "A language."
link = "/glossary.md#go"
case-sensitive = true

[glossary.terms.orphan]
term = "orphan"
definition = "No link."

[glossary.terms.build]
term = "build"
aliases = ["builds"]
definition = "A named output configuration."
link = "/glossary.md#build"
match = "marked"

[glossary.terms.mode]
term = "mode"
definition = "How a build treats content."
link = "/glossary.md#mode"
match = "every"

[consumer]
profile = "astro"
base-path = "/docs/"

[builds.site]
variants = "switch"
availability = "badge"

[builds.cloud]
variants = { deployment = "cloud" }
availability = { filter = "cloud" }

[builds.cloud-only]
variants = { deployment = "cloud" }
availability = "badge"

[builds.npm-only]
variants = { pm = "npm" }
availability = "badge"

[builds.sm-npm-pnpm]
variants = { deployment = "self-managed", pm = ["npm", "pnpm"] }
availability = "badge"

[builds."sm-3.3"]
variants = "switch"
availability = { filter = "self-managed 3.3" }

[builds."sm-3.4"]
variants = "switch"
availability = { filter = "self-managed 3.4" }

[builds."sm-3.5"]
variants = "switch"
availability = { filter = "self-managed 3.5" }
"#;

pub fn path(text: &str) -> RelPath {
    RelPath::parse(text).expect("a valid path")
}

/// A project with these content files (paths relative to the content root),
/// under [`MODEL`] or another model.
pub fn project_with(model_text: &str, files: &[(&str, &str)]) -> Project {
    let model = match load_str(model_text, FileId::new(0)) {
        Ok(model) => Arc::new(model),
        Err(issues) => panic!("the test model doesn't load: {issues:?}"),
    };
    let layout = Layout::from_model(&model);
    let mut fs = MemoryFs::new(&layout);
    for (name, text) in files {
        fs = fs.with_source(name, text);
    }
    Project::load(model, layout, &fs)
}

pub fn project(files: &[(&str, &str)]) -> Project {
    project_with(MODEL, files)
}

/// The router the tests use: base path `/docs/` and trailing slashes.
pub fn router(project: &Project) -> DefaultRouter {
    DefaultRouter::from_consumer(&project.model().consumer)
}

/// Resolves a page for a build; panics if the build doesn't publish it.
pub fn resolve(project: &Project, page: &str, build: &str) -> ResolvedPage {
    let b = project.model().build(build).expect("a build");
    project
        .resolve_page(&path(page), b, &router(project))
        .unwrap_or_else(|| panic!("{page} isn't published by {build}"))
}

/// The pages a build publishes, and the pages it drops.
pub fn published(project: &Project, build: &str) -> Vec<String> {
    let b = project.model().build(build).expect("a build");
    project
        .resolve_build(b, &router(project))
        .pages
        .iter()
        .map(|p| p.path.to_string())
        .collect()
}

/// The text of inline content, as it reads once resolved.
pub fn plain(inlines: &[Inline]) -> String {
    let mut out = String::new();
    for inline in inlines {
        match &inline.kind {
            InlineKind::Text(t) | InlineKind::Code(t) => out.push_str(t),
            InlineKind::SoftBreak | InlineKind::HardBreak => out.push(' '),
            InlineKind::Emphasis(c) | InlineKind::Strong(c) => out.push_str(&plain(c)),
            InlineKind::Link(l) => out.push_str(&plain(&l.children)),
            InlineKind::Image(i) => out.push_str(&plain(&i.children)),
            InlineKind::Html(h) => out.push_str(h),
            InlineKind::Phrase(p) => out.push_str(&format!("{{{}}}", p.key)),
        }
    }
    out
}

/// A compact description of a page's blocks, one line each, children
/// indented: `h2 Title`, `p text`, `code text`, `@directive primary`,
/// `group[pm=npm | pm=yarn]`.
pub fn summary(page: &ResolvedPage) -> Vec<String> {
    let mut out = Vec::new();
    lines(&page.blocks, 0, &mut out);
    out
}

fn lines(blocks: &[ResolvedBlock], depth: usize, out: &mut Vec<String>) {
    let pad = "  ".repeat(depth);
    for block in blocks {
        match &block.kind {
            ResolvedKind::Leaf(b) => match &b.kind {
                BlockKind::Heading(h) => {
                    out.push(format!("{pad}h{} {}", h.level, plain(&h.inlines)))
                }
                BlockKind::Paragraph(p) => out.push(format!("{pad}p {}", plain(&p.inlines))),
                BlockKind::CodeBlock(c) => {
                    out.push(format!("{pad}code {}", c.literal.trim_end_matches('\n')));
                }
                BlockKind::HtmlBlock(h) => out.push(format!("{pad}html {}", h.literal.trim_end())),
                BlockKind::Directive(line) => {
                    let primary = match (&block.annotation, &line.primary) {
                        (Some(a), _) => format!(" {}", a.text),
                        (None, Some(PrimaryValue::Identifier(p))) => format!(" {}", p.text),
                        (None, Some(PrimaryValue::Text(t))) => format!(" {}", plain(&t.inlines)),
                        _ => String::new(),
                    };
                    out.push(format!("{pad}@{}{primary}", line.name));
                }
                _ => out.push(format!("{pad}?")),
            },
            ResolvedKind::BlockQuote { children } => {
                out.push(format!("{pad}quote"));
                lines(children, depth + 1, out);
            }
            ResolvedKind::List { items, .. } => {
                out.push(format!("{pad}list"));
                for item in items {
                    lines(&item.children, depth + 1, out);
                }
            }
            ResolvedKind::Container {
                opener, children, ..
            } => {
                out.push(format!("{pad}@{}:", opener.name));
                lines(children, depth + 1, out);
            }
            ResolvedKind::Group { arms, .. } => {
                let names: Vec<String> = arms
                    .iter()
                    .map(|a| {
                        let attrs: Vec<String> = a
                            .opener
                            .attributes
                            .iter()
                            .flat_map(|b| &b.attributes)
                            .map(|attr| {
                                format!(
                                    "{}={}",
                                    attr.key,
                                    attr.value.as_ref().and_then(|v| v.as_text()).unwrap_or("|")
                                )
                            })
                            .collect();
                        if attrs.is_empty() {
                            a.opener
                                .title
                                .as_ref()
                                .map_or("?".to_owned(), |t| plain(&t.inlines))
                        } else {
                            attrs.join(",")
                        }
                    })
                    .collect();
                out.push(format!("{pad}group[{}]", names.join(" | ")));
                for arm in arms {
                    lines(&arm.children, depth + 1, out);
                }
            }
        }
    }
}
