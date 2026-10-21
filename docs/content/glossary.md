---
type: reference
title: Glossary
description: The terms these docs use, each with a sentence on what it means and a link to the page that documents it.
---

Each term below is linked from the pages that use it, with its first paragraph as the link's title. Build, phrase, and variant are ordinary words too, so they're linked only where a page links them here.

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

## Fragment
@term {aliases=fragments}

A source file that exists only to be included in other files: any file with a path segment starting with `_`, or whose path matches one of the content model's [`[fragments] patterns`](content-model/index.md#4-typesname-and-fragments).

See [Files](writing/frontmatter.md#files) and [`@include`](writing/directives.md#include).

## Page preview
@term

The editor's preview of the active page, rendered as the site shows it.

See [the preview](editor/preview.md).

## Phrase
@term {aliases=phrases, match=marked}

A reusable string declared in `ascribe.toml` and written `{key}` in a page.

See [Phrases](writing/inline.md#phrases).

## Site preview
@term

The active page on the project's own dev server, in the browser or the preview panel.

See [the site preview](editor/preview.md#site-preview).

## Variant
@term {aliases=variants, match=marked}

Alternative content for a value of a dimension, written with `@variant`.

See [`@variant`](writing/directives.md#variant) and [Content that varies](writing/variants.md).
