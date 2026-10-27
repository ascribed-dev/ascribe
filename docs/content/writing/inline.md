---
type: reference
title: Phrases, links, and images
description: "What a page writes in its prose: phrases, links between files, images, heading ids, and glossary terms."
---

What Ascribe adds to a page's prose, and the rules it checks there. Directives are in the [directive reference](directives.md), and how one is written in [directive syntax](syntax.md). The [Ascribe specification]({repo}/blob/main/SPEC.md) is the normative definition; each section links to its part of the spec.

## Phrases

([SPEC §5.1]({repo}/blob/main/SPEC.md#51-phrases))

A phrase inserts a string declared in `ascribe.toml`'s `[phrases]`, such as a product name, a version, or a URL base:

```markdown
Sign in to {cloud} and copy an API key.
See the [streaming API reference]({api}streaming).
```

- `{key}` is a phrase only when `key` is declared; otherwise it's literal text. Ascribe warns about undeclared `{key}` text in prose, since declaring the key later would silently change it. Write `\{key}` for text you mean.
- Phrases apply in prose, headings, link text, and link destinations, and in frontmatter fields declared with `phrases = true`.
- In a fenced code block, only when its info string has `phrases=true`: ` ```yaml phrases=true `. Never in code spans, indented code, or raw HTML.
- Values are inserted as literal text.
- `{{key}}` puts the value between literal braces, which is usually a leftover from another tool's templates; Ascribe warns.
- A phrase's value written out, `Quill 3.4.1` where `{product} {version}` would do, goes stale when the value changes: [`phrase-text-written`](../checks/diagnostics/content-checks.md#ASC288) reports it, with a fix that writes the phrase. Acknowledge text meant as written, such as a changelog's version, with `@intended {check=phrase-text-written}` above its block.

## Links

([SPEC §5.2]({repo}/blob/main/SPEC.md#52-links))

Links point at **files**, not URLs. Ascribe writes each output's URLs.

```markdown
[Rotate API keys](keys.md#rotate-keys)
See [](keys.md#rotate-keys).
```

- Paths are relative to the file, or to the content root when they start with `/`. `#id` names a heading on the target page by its id: one of its own, or one from a fragment it includes.
- The file, and the id, must exist. A link to a fragment is an error: link to a page that includes it.
- **Empty link text** is filled in with the target's title: the heading's text with an id, or the page's `title`.
- External URLs (`https://…`) pass through unchanged.
- A destination that looks like a published URL (`/guides/install/`) instead of a file gets a warning, and the editor offers to convert it.
- In each build, a link's target page must be published. To link from shared content to a page a build drops, put the link in a `@variant` arm the same build removes.
- A link to an OpenAPI spec the content model declares, with an `operationId` as its fragment, `[Get a user](../api/openapi.yaml#getUser)`, is a link to that operation: the spec must have it, and each output writes the operation's address in the published reference. See [Link to your API reference](../openapi/index.md). An operation with no id is linked by its method and path, `#delete/users/{id}`, with Redoc 3: see [Link to an operation with no id](../openapi/no-id.md).

### Links between lines

In a build versioned by a target, a link to a page a line doesn't publish isn't an error: it goes to the page that [replaces it](frontmatter.md#a-page-that-replaces-another), or that it replaces, and otherwise to the page in the nearest line that has it. [Links between lines](../versions/lines.md#links-between-lines), in the release lines reference, has each step and what the output says about a link out of its line. `ascribe link --from <page> <target>` says where a link goes in each line.

## Images

([SPEC §5.3]({repo}/blob/main/SPEC.md#53-images))

Images are CommonMark images. Attributes, such as a width, go in a block directly after the image, with no space between; `ascribe.toml`'s `[images.attributes]` declares which are accepted.

```markdown
![The Quill settings page](settings.png){width=600}
```

- A local image must exist, and names must match exactly, including case.
- Ascribe warns about an image without alt text.
- The image is copied into each output, so outputs don't need the source files.

## Heading ids

([SPEC §5.5]({repo}/blob/main/SPEC.md#55-heading-ids))

Every heading has an id: its [`@id`](directives.md#id) if it has one, and otherwise a slug of its text, computed the way the site's slugger computes it (GitHub's, for Astro). Links and includes name headings by these ids. Ascribe warns about a heading whose id could change without the heading being edited (its text has a phrase, or duplicates another heading's) and about a heading whose slug is empty; add `@id` to fix each.

## Glossary terms

([SPEC §5.4]({repo}/blob/main/SPEC.md#54-glossary-terms))

A glossary is a page, or several, that `[glossary] pages` in `ascribe.toml` lists, and each of its headings marked [`@term`](directives.md#term) is a term, defined by the paragraph under it. Ascribe links terms to their definitions in each built page, and authors don't mark terms where they're used. A link you write to a term's heading, as `[build](glossary.md#build)`, is a use of the term too: it's how you link a `marked` term, which isn't linked automatically. See [`[glossary]`](../content-model/content.md#12-glossary).
