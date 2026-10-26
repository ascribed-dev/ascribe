---
type: reference
title: Glossary
description: The terms these docs use, each with a sentence on what it means and a link to the page that documents it.
---

Each term below is linked from the pages that use it, with its first paragraph as the link's title. Most of them are ordinary words too, so they're linked only where a page links them here: acknowledgement, address, agent, build, example, phrase, report, review, source, target, and variant. Each has one meaning in these docs, and where a word is also used another way, its entry says where and what it means there.

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

Most pages mean the agent you work with on the docs, as [Agents](agents/index.md) does. The outputs for agents, [`[consumer] agents`](content-model/output.md#consumer), and [`ascribe report delivery`](checks/report.md#the-site-and-agents) are about the agents that read the published site, and [`ascribe agents index`](agents/product-index.md) writes for a product's own agents.

## Availability
@term

Where content applies and its lifecycle state there, such as `self-managed preview 3.4`, written with `@available` or a page's `available`.

See [`@available`](writing/directives.md#available). Each build either marks it or removes what isn't available.

## Baseline
@term

The list in `ascribe.baseline` of the advice a project had when it was written, which `ascribe check` holds back so it reports only what's new.

See [The baseline](checks/baseline.md).

## Build
@term {aliases=builds, match=marked}

A named set of choices, in `ascribe.toml`, for which variants and which availability a page's output keeps.

Each build is declared in [`[builds.<name>]`](content-model/output.md#16-buildsname).

## Content model
@term {aliases="content models"}

A project's schema, `ascribe.toml`: its content types, dimensions, phrases, and builds.

[The content model](content-model/index.md) documents each of its sections.

## Dimension
@term {aliases=dimensions}

An axis content varies along, such as a package manager or a deployment, declared in `ascribe.toml`.

See [`[dimensions]`](content-model/content.md#dimensions) and [Content that varies](writing/variants.md).

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

## Permalink
@term {aliases=permalinks}

A name for a page or a heading, such as `go/weave-config`, whose address leads to it wherever it moves, for linking to the docs from outside them.

See [Permalinks](permalinks/index.md) and [`@id`'s permalinks](writing/directives.md#permalinks).

## Phrase
@term {aliases=phrases, match=marked}

A reusable string declared in `ascribe.toml` and written `{key}` in a page.

See [Phrases](writing/inline.md#phrases).

## Recheck date
@term {aliases="recheck dates"}

When a page is next due to be read against what it describes: its field with the `recheck-date` role, which [`recheck-overdue`](checks/diagnostics/content-checks.md#ASC153) compares with today. It was the review date, and the role `review-date`, which still works for a release.

See [Field roles](content-model/index.md#54-field-roles).

## Release line
@term {aliases="release lines"}

A version of a target as `[releases]` lists it, such as `3.4`, which holds every version that starts with it: live, planned, or archived.

See [Release lines](versions/lines.md). Pages about versions call it a line. Elsewhere a line is a file's line, and a code block's tested line is the line under it that says what it was [tested with](examples/tested.md).

## Report
@term {aliases=reports, match=marked}

What `ascribe report` says about the state of a whole project: its problems, inventory, builds, pages, external links, and published site.

See [The project report](checks/report.md). The [HTML preview](#html-preview), which `ascribe diff --format html` writes for a pull request, is part of [review](#review), not this.

## Review
@term {aliases=reviews, match=marked}

Reading a pull request's changes as readers will see the pages, in the page preview, the site preview, or the HTML preview, and commenting there.

See [Review](review/index.md). A diagnostic whose next step is `review` may be fine as it is: see [What to do next](checks/diagnostics.md#what-to-do-next). When to check a page again is its [recheck date](#recheck-date), not a review.

## Site preview
@term

The active page on the project's own dev server, in the browser or the preview panel.

See [the site preview](editor/preview.md#site-preview).

## Source
@term {aliases=sources, match=marked}

A folder or repository outside the content, declared in `[sources.<name>]`, that pages take code examples from with `@snippet`.

See [`[sources.<name>]`](content-model/output.md#18-sourcesname) and [`ascribe sources`](examples/sources.md#cli-sources). A page's source file, the Markdown it's written in, is the ordinary sense: a source file, as against what a build writes from it.

## Target
@term {aliases=targets, match=marked}

A dimension value that availability names, such as `self-managed` in `self-managed preview 3.4`; a versioned one has release lines, which `[releases.<target>]` lists.

See [Availability specs](writing/directives.md#availability-specs) and [`[releases.<target>]`](content-model/content.md#71-releasestarget). A link's target, as `ascribe link`, `refs`, and `rename` take it, is what the link leads to, and `ascribe agents sync --target` names an agent whose files it writes.

## Variant
@term {aliases=variants, match=marked}

Alternative content for a value of a dimension, written with `@variant`.

See [`@variant`](writing/directives.md#variant) and [Content that varies](writing/variants.md).
