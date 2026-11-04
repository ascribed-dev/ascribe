---
type: reference
title: Glossary
description: The terms these docs use, each with a sentence on what it means and a link to the page that documents it.
---

Each term below is linked from the pages that use it, with its first paragraph as the link's title. Most of them are ordinary words too, so they're linked only where a page links them here: acknowledgement, address, agent, arm, binding, build, consumer, container, diagnostic, directive, example, phrase, preview, primary, problem, report, review, source, target, variant, and widget. Each has one meaning in these docs, and where a word is also used another way, its entry says where and what it means there.

## Acknowledgement
@term {aliases=acknowledgements, match=marked}

A note that a problem a check reports is intended, written with `@intended` beside the content or in `ascribe.toml`, so the check doesn't report it again.

Only a check whose next step is review can be acknowledged, and each diagnostic's entry says where. See [`@intended`](writing/directives.md#intended).

## Address
@term {aliases=addresses, match=marked}

Where a build publishes a page or its Markdown version, such as `/docs/install/`, which `ascribe.addresses` keeps leading somewhere when the page moves or goes.

See [Keep the addresses you've published](publishing/redirects.md). A snippet's address, such as `code:service/client.py#connect`, is another thing: the file and region of a [source](#source) that [`@snippet`](writing/directives.md#snippet) shows.

## Agent
@term {aliases=agents, match=marked}

An AI program, such as Claude Code or GitHub Copilot, that writes a project's docs with Ascribe's tools, or reads the docs a site publishes.

Most pages mean the agent you work with on the docs, as [Set up your agent](agents/index.md) does. The outputs for agents, [`[consumer] agents`](content-model/output.md#consumer), and [`ascribe report delivery`](checks/report.md#the-site-and-agents) are about the agents that read the published site, and [`ascribe agents index`](agents/product-index.md) writes for a product's own agents.

## Arm
@term {aliases=arms, match=marked}

One alternative of a group, such as a `@variant` group's alternative for one package manager: it starts at its opener and runs to the next opener or the group's `@end`.

See [Groups](writing/syntax.md#groups). A build that selects a value keeps that value's arm and leaves out the rest.

## Availability
@term

Where content applies and its lifecycle state there, such as `self-managed preview 3.4`, written with `@available` or a page's `available`.

See [`@available`](writing/directives.md#available). Each build either marks it or removes what isn't available.

## Availability mark
@term {aliases="availability marks"}

An `@available` line, or a page's `available` field, that says where content applies and its lifecycle state there.

See [Availability](#availability). It's a different thing from a [change mark](#change-mark), and to mark, the verb, keeps its ordinary meaning.

## Baseline
@term

The list in `ascribe.baseline` of the advice a project had when it was written, which `ascribe check` holds back so it reports only what's new.

See [The baseline](checks/baseline.md).

## Binding
@term {aliases=bindings, match=marked}

What a directive in line form applies to: its own primary, the heading above it and its section, or the block after it.

See [Binding: what a directive applies to](writing/syntax.md#binding-what-a-directive-applies-to). A widget's declaration sets its binding.

## Build
@term {aliases=builds, match=marked}

A named set of choices, in `ascribe.toml`, for which variants and which availability a page's output keeps.

Each build is declared in [`[builds.<name>]`](content-model/output.md#16-buildsname).

## Change mark
@term {aliases="change marks"}

The bar and label that review puts beside a block that changed against the base: **Added**, **Changed**, **Removed**, or **Moved**.

See [The change marks](review/how-it-works.md#the-marks). It's a different thing from an [availability mark](#availability-mark).

## Consumer
@term {aliases=consumers, match=marked}

The site generator that publishes a build's site output, such as Astro, which `[consumer]` in `ascribe.toml` names with the site's address and base path.

See [`[consumer]`](content-model/output.md#consumer).

## Container
@term {aliases=containers, match=marked}

A directive's form that holds blocks: a directive line ending in a colon, then the blocks, then `@end`.

See [Forms: line and container](writing/syntax.md#forms-line-and-container). A directive in its other form, a single line, applies to something by its [binding](#binding).

## Content model
@term {aliases="content models"}

A project's schema, `ascribe.toml`: its page types, dimensions, phrases, and builds.

[The content model](content-model/index.md) documents each of its sections.

## Content root
@term {aliases="content roots"}

The folder that holds a project's pages and fragments, `[project] content-root` in `ascribe.toml`: every Markdown file under it is a source file, and a link's path that starts with `/` starts from it.

See [`[project]`](content-model/index.md#3-project).

## Diagnostic
@term {aliases=diagnostics, match=marked}

A kind of [problem](#problem), with a code and a name, such as `ASC036 link-target-missing`, that the diagnostics reference and `ascribe explain` describe.

See [Diagnostics](checks/diagnostics.md). A problem is one place a diagnostic was found.

## Dimension
@term {aliases=dimensions}

An axis content varies along, such as a package manager or a deployment, declared in `ascribe.toml`.

See [`[dimensions]`](content-model/variation.md#dimensions) and [Content that varies](writing/variants.md).

## Directive
@term {aliases=directives, match=marked}

A line that starts with `@` and annotates the content around it, such as `@note` or `@variant`: a built-in one, or a project's [widget](#widget).

See [Directive syntax](writing/syntax.md) and [Directives](writing/directives.md).

## Example
@term {aliases=examples, match=marked}

Code a page shows from a source's file with `@snippet`, so the page has no copy to fall behind, and that `ascribe.examples.toml` can name so a test runs it.

See [Examples and drift](examples/index.md) and [Show what an example was tested with](examples/tested.md). A JSON block marked as an OpenAPI operation's request or response is an API example, which the `api-example-*` checks [compare with its schema](openapi/examples.md), and this repository's `examples/` folder holds example projects; neither is this.

## Fragment
@term {aliases=fragments}

A source file that exists only to be included in other files: any file with a path segment starting with `_`, or whose path matches one of the content model's [`[fragments] patterns`](content-model/index.md#4-typesname-and-fragments).

See [Files](writing/frontmatter.md#files) and [`@include`](writing/directives.md#include).

## HTML preview
@term

The HTML file `ascribe diff --format html` writes, showing a pull request's changed pages rendered and marked, which CI can attach to the pull request.

It's the third place to [review](#review), beside the page preview and the site preview, and needs no checkout. See [Attach the HTML preview to every pull request](review/ci-report.md).

## Memo
@term {aliases=memos}

A note for whoever edits a page next, saying why it's written the way it is, kept beside what it's about and never published.

It's written with `@memo` in the page, or as a memo file in `memos/`. See [Leave a memo for whoever edits next](writing/memos.md).

## Page preview
@term

The editor's preview of the active page, rendered as the site shows it.

See [the preview](editor/preview.md).

## Page type
@term {aliases="page types"}

A kind of page, declared in `ascribe.toml` as `[types.<name>]`, such as `how-to`, which a page names with `type:` in its frontmatter and which says what frontmatter it takes.

See [Define your page types](content-model/page-types.md). A key name, such as `[listing.types]`, still says "type".

## Permalink
@term {aliases=permalinks}

A name for a page or a heading, such as `go/weave-config`, whose address leads to it wherever it moves, for linking to the docs from outside them.

See [Link to the docs from your product](permalinks/index.md) and [`@id`'s permalinks](writing/directives.md#permalinks).

## Phrase
@term {aliases=phrases, match=marked}

A reusable string declared in `ascribe.toml` and written `{key}` in a page.

See [Phrases](writing/inline.md#phrases).

## Plain Markdown output
@term {aliases="plain Markdown outputs"}

The output a build writes in `plain/`: each page as ordinary Markdown with every directive resolved and no HTML, for search indexes, agents, and export.

See [`ascribe build`](checks/check.md#cli-build). With the outputs for agents on, a site publishes it beside its pages.

## Preview
@term {aliases=previews, match=marked}

Pages shown before they're published. These docs always say which of the four: the page preview, the site preview, the HTML preview, or `ascribe preview`.

The [page preview](#page-preview) is in the editor; the [site preview](#site-preview) is the project's own dev server; the [HTML preview](#html-preview) is a pull request's, from `ascribe diff --format html`; and [`ascribe preview`](checks/check.md#cli-preview) renders a build as a plain site and serves it on this computer with the command alone. None of them is the published site.

## Primary
@term {aliases=primaries, match=marked}

The value after a directive line's colon, the directive's one main value, such as a note's text or the file an `@include` brings in.

See [Directive lines](writing/syntax.md#directive-lines).

## Problem
@term {aliases=problems, match=marked}

What a check found in a project, at one place: an error, a warning, or advice, with its message and fixes.

See [Diagnostics](checks/diagnostics.md), which lists each kind of problem as a [diagnostic](#diagnostic). Counting advice, these docs say advice-level problems.

## Project root
@term {aliases="project roots"}

The folder that holds a project's `ascribe.toml`, which paths in the file, such as `content-root` and a source's `path`, are written from.

See [Conventions](content-model/index.md#1-conventions). The [content root](#content-root) is a folder inside it, or the same folder.

## Recheck date
@term {aliases="recheck dates"}

When a page is next due to be read against what it describes: its field with the `recheck-date` role, which [`recheck-overdue`](checks/diagnostics/content-checks.md#ASC153) compares with today. It was the review date, with the role `review-date` and the check `review-overdue`, and both old names still work, for now.

See [Field roles](content-model/index.md#54-field-roles).

## Release line
@term {aliases="release lines"}

A version of a target as `[releases]` lists it, such as `3.4`, which holds every version that starts with it: live, planned, or archived.

See [Release lines](versions/lines.md). Pages about versions call it a line. Elsewhere a line is a file's line, and a code block's tested line is the line under it that says what it was [tested with](examples/tested.md).

## Report
@term {aliases=reports, match=marked}

What `ascribe report` says about the state of a whole project: its problems, inventory, builds, pages, external links, and published site.

See [Act on the project report](checks/report.md). The [HTML preview](#html-preview), which `ascribe diff --format html` writes for a pull request, is part of [review](#review), not this.

## Review
@term {aliases=reviews, match=marked}

Reading a pull request's changes as readers will see the pages, in the page preview, the site preview, or the HTML preview, and commenting there.

See [Review a pull request](review/index.md). A diagnostic whose next step is `review` may be fine as it is: see [What to do next](checks/diagnostics.md#what-to-do-next). When to check a page again is its [recheck date](#recheck-date), not a review.

## Site preview
@term

The active page on the project's own dev server, in the browser or the preview panel. It has the published site's look, where `ascribe preview` has the element library's.

See [the site preview](editor/preview.md#site-preview).

## Source
@term {aliases=sources, match=marked}

A folder or repository outside the content, declared in `[sources.<name>]`, that pages take code examples from with `@snippet`.

See [`[sources.<name>]`](content-model/sources.md#20-sourcesname) and [`ascribe sources`](examples/sources.md#cli-sources). A page's source file, the Markdown it's written in, is the ordinary sense: a source file, as against what a build writes from it.

## Target
@term {aliases=targets, match=marked}

A dimension value that availability names, such as `self-managed` in `self-managed preview 3.4`; a versioned one has release lines, which `[releases.<target>]` lists.

See [Availability specs](writing/directives.md#availability-specs) and [`[releases.<target>]`](content-model/variation.md#121-releasestarget). A link's target, as `ascribe link`, `refs`, and `rename` take it, is what the link leads to, and `ascribe agents sync --target` names an agent whose files it writes.

## Variant
@term {aliases=variants, match=marked}

Alternative content for a value of a dimension, written with `@variant`.

See [`@variant`](writing/directives.md#variant) and [Content that varies](writing/variants.md).

## Widget
@term {aliases="widgets|project widget|project widgets", match=marked}

A directive a project declares in `[widgets.<name>]`, with a hyphen in its name, for what the built-in directives don't cover; the site output writes it as a custom element.

See [Project widgets](writing/directives.md#project-widgets).
