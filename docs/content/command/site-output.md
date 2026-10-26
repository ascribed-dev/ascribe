---
type: reference
title: A page in the site output
description: "What `ascribe resolve --emit site` writes: a page as the site output gives it to a site, its frontmatter, a line of a versioned build, and how to read its elements without their script."
available: ascribe 0.4.4
---

[`ascribe resolve PAGE --emit site`](questions.md#cli-resolve) writes one page as the site output writes it, Ascribe's elements and all, without building the project: the file `ascribe build --emit site` writes for the page, which `@ascribed/astro` receives. This page says what's in it, for a writer asking what a directive becomes, an agent checking what a reader of the site gets, or a test of a site's integration.

## Its frontmatter
@id: frontmatter

The site form has no `#` title: the title is in its frontmatter, which `--frontmatter` adds. That's the site output's frontmatter, which an Astro content collection reads, not the page's as the plain form writes it: with the [keys](../astro/reference.md#components) the site output adds, such as `formatted`, a title's formatted form as HTML, `available`, as a list of targets with their states, versions, and text, and `source`, the page's path from the content root. From `examples/astro-site`, whose titles can show code:

@snippet {lang=markdown}: code:crates/ascribe-cli/tests/output/resolve-site-frontmatter.txt

For a line of a versioned build, the page's links are the line's, under its base path, and `available` is the page's targets. From `examples/quill`, `ascribe resolve upgrade.md --build site@3.4 --emit site --frontmatter`:

@snippet {lang=markdown}: code:crates/ascribe-cli/tests/output/resolve-site-line.txt

## Assets and anchors
@id: assets

The site form has no source anchors, as `ascribe build` writes none without `--anchors`. Images and linked files are referred to as the site output refers to them, relative to the page's file, and none is copied: a path under `_ascribe/` names where a build would put the file, which doesn't exist until one does.

## Reading the site output
@id {permalink=true}: reading-site-output
@summary: The site output's elements read without their script: tabs hold every panel, an attribute marker isn't content, and the note for agents isn't the page's text.

The site output's elements are custom elements that `@ascribed/elements` defines, and their Markdown reads without the script. What a reader of `--emit site`, such as an agent, should know:

- **`<ascribe-for-agents>` is the site's note for agents, not the page's text.** It opens each page, when the build publishes the outputs for agents, and points to `llms.txt` and the page's Markdown; the element library hides it from sight.
- **Tabs hold every panel,** as a page without JavaScript shows them. A reader of the site sees one `<ascribe-tab>` at a time, chosen by its `<ascribe-tabs>`'s `sync` dimension.
- **An attribute marker isn't content.** `<ascribe-attributes id="connect"></ascribe-attributes>` after a heading or an image carries its attributes for the site to apply; the heading's text ends before it.
- **A glossary link's `title` is the term's definition,** written so a reader sees it on hover, beside a marker naming the term:

  @snippet {lang=markdown}: code:crates/ascribe-cli/tests/output/resolve-site-glossary.txt

- **An availability badge** is an `<ascribe-availability>` element of `<ascribe-availability-target>`s, each with its target, dimension, and states as attributes, and its text as the reader sees it.
