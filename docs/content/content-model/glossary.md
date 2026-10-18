---
type: how-to
title: Set up a glossary
description: Write a project's terms on a glossary page, choose which of their occurrences are linked, and keep terms that differ by version or product, so every page links its terms to their definitions.
available: next
---

A glossary gives each of a project's terms one definition, and every page that uses the term links to it. In Ascribe, a glossary is a page you write like any other: a heading for each term, with its definition under it. `ascribe.toml` says which pages are glossaries, and Ascribe does the linking, so you never mark a term where you use it.

This guide sets one up, from the first term to what a site shows. It's for a writer who has a project already, as [Getting started](../getting-started.md) sets one up. [Ascribe's own glossary](../glossary.md) is an example of most of it, and [its source]({repo}/blob/main/docs/content/glossary.md) shows how it's written.

## Write the page

@steps
1. Make a page for the glossary, such as `glossary.md` in the content root, with the frontmatter its type needs.
2. For each term, write a heading whose text is the term, and `@term` on the line under it.
3. Under that, as the first paragraph of the section, write the definition: a sentence that says what the term means and stands on its own, since it's shown where the term is used, away from the glossary.
4. Write anything more after it, in later paragraphs, such as a link to the page that documents the term.

```markdown
---
type: page
title: Glossary
description: The words these docs use, each with what it means.
---

## API key
@term {aliases="API keys|keys"}

A secret that identifies a program calling the API, made in the dashboard.

[Install the agent](install.md) says where to make one.
```

`aliases` lists other forms that count as the term, such as its plural, separated by `|`. The term's id is its heading's id, `api-key` here: it's the id a link to the term names. To keep the id when you reword the heading, give the heading an [`@id`](../writing/directives.md#id), on the line above `@term`.

Only the first paragraph is the definition, so a section that opens with a list, a note, or a code block has none: write a sentence before it. Keep it to a sentence or two. A link's title and the editor's hover show it whole, and `ascribe check` reports one over 300 characters as advice.

## List it in `ascribe.toml`

Name the page in `[glossary] pages`, as its path from the content root:

```toml
[glossary]
pages = ["glossary.md"]
```

Run `ascribe check`. Every other page now links the first place it uses each term to the term's heading. `@term` on a page `pages` doesn't list is an error, `term-off-glossary-page`, and the editor's quick fix for it lists the page.

Once the page is listed, the editor can write a term for you: select a word in a page, press `Ctrl+K A` (`Cmd+K A` on macOS), and choose **Add to the glossary**. It adds the heading, `@term`, and the definition you give it to the glossary page. See [Changing the content model](../editor/index.md#changing-the-content-model).

## Choose which occurrences are linked

By default a page links the first occurrence of each term, and the rest of the page reads as plain text. `[glossary] match` chooses for the whole glossary: `"first"`, or `"every"` to link each occurrence.

```toml
[glossary]
pages = ["glossary.md"]
match = "every"
```

A term can choose for itself with `@term`'s own `match`, and with `case-sensitive`, for a term such as `Go` that's also an ordinary word in lowercase:

```markdown
## Go
@term {case-sensitive=true, match=every}

The programming language the agent is written in.
```

An occurrence is a whole word in prose, matched ignoring case unless the glossary or the term says otherwise; a heading, a link's text, and code are never linked. Where two terms overlap, the longer wins, so `API key` is linked rather than `API`. The glossary page itself doesn't link the terms it defines. The [`[glossary]` reference](content.md#12-glossary) has every setting and rule.

## A term that's also an ordinary word

Some terms are ordinary words most of the time: a "flag" that's a feature flag, and a "flag" that's anything else. Linking every one would be wrong as often as right, so mark the term `match=marked`, and it isn't linked by itself:

```markdown
## Flag
@term {aliases=flags, match=marked}

A named switch that turns a feature on for some users and off for the rest.
```

Then link the occurrences that mean the term to its heading, as you'd link any heading:

```markdown
Every [flag](glossary.md#flag) starts off. Flag the release for review.
```

A link to a term's heading is the term's link, whoever wrote it: on the site it reads exactly as an automatic one does, with the definition as its title, and it counts as a use of the term. A link to the glossary page, or to another heading in the term's section, is an ordinary link. This works for any term, so for one that's linked `first`, a link you write is the page's first, and no later occurrence on the page is linked.

Ascribe's own glossary marks three of its terms this way: build, phrase, and variant.

## A term that differs by version or product

A term follows its section into each build. Mark the section with [`@available`](../writing/directives.md#available), under `@term`, and the term exists only in the builds that keep it; in the rest, its occurrences are plain text.

```markdown
## Usage limit
@term {aliases="usage limits"}
@available: cloud

The most requests a workspace can make in an hour.
```

When the definition differs by product, open the section with a [`@variant`](../writing/directives.md#variant) group instead of a paragraph. Each arm opens with the definition for its builds:

```markdown
## Region
@term

@variant {deployment=cloud}:
The data center a workspace's data is kept in, chosen when it's made.
@variant {deployment=self-managed}:
The data center your own servers are in, as the license names it.
@end
```

Each build links a term to the definition it publishes. A build that keeps the heading but removes the paragraph has no definition for the term, and doesn't link it either. [Content that varies](../writing/variants.md) sets up the dimensions and builds these use.

## A glossary in several pages

A long glossary, or one for each product, can be several pages. List each in `pages`:

```toml
[glossary]
pages = ["glossary.md", "self-managed/glossary.md"]
```

Write a term the pages share once, in a [fragment](../writing/directives.md#files), and include it on each:

```markdown
@include: _glossary/shared.md
```

A heading the fragment brings in is one term, wherever it's shown. Each build links it to the first page in `pages` that the build publishes, so order `pages` with the page you'd rather readers land on first. Two different headings with the same text are an error, `glossary-term-duplicate`, since a term has one definition, and so are two with the same id, since the id names the term: give one of them another with `@id`.

## What `ascribe check` reports

Each of these is in [the diagnostics reference](../checks/diagnostics.md), with its fix:

| Code | Reports | Level |
|---|---|---|
| [`model-glossary-page`](../checks/diagnostics/content-model.md#ASC238) | A path in `pages` that isn't a page | Error |
| [`term-off-glossary-page`](../checks/diagnostics/pages.md#ASC242) | `@term` on a page `pages` doesn't list | Error |
| [`glossary-term-duplicate`](../checks/diagnostics/pages.md#ASC239) | A term or alias with the same text as another, or a term with the same id, at the second | Error |
| [`glossary-definition-missing`](../checks/diagnostics/pages.md#ASC240) | A term whose section doesn't open with a paragraph | Error |
| [`glossary-definition-long`](../checks/diagnostics/content-checks.md#ASC241) | A definition over 300 characters in a build | Advice |
| [`glossary-term-unused`](../checks/diagnostics/content-checks.md#ASC158) | A term no other page uses or links | Advice |

A term you keep though no page uses it yet is acknowledged at its heading, with [`@intended`](../writing/directives.md#intended) among the directives under it:

```markdown
## Rollback
@term
@intended {check=glossary-term-unused}: The rollback guide uses it once it's written.

Turning a flag back off for everyone it was on for.
```

## What a site gets

In the site output, a term's link is an ordinary link to its heading, with its definition as the link's title, so a reader sees the definition when they point at it. The link is marked with `data-ascribe-term`, holding the term's id:

```html
<a href="/glossary/#api-key" title="A secret that identifies a program calling the API, made in the dashboard." data-ascribe-term="api-key">API key</a>
```

The element library underlines a term's link with dots. A site restyles it, or shows the definition in a popover of its own, by selecting `a[data-ascribe-term]`: see [the element library](../astro/elements.md). The plain Markdown output links the term the same way, without the title or the marker, and the JSON output lists, for each block, the terms it links, with each one's definition.

Writers get the definitions too:

- **In the editor,** hovering over a term shows its definition, **Go to Definition** opens its heading, and **Find All References** lists its uses. See [the editor](../editor/index.md#hover).
- **From the command,** `ascribe refs term:api-key` lists a term's uses, and `ascribe model --section glossary` lists the project's terms, which is how an agent learns them.

## See also

- [`[glossary]`](content.md#12-glossary): the settings, terms, matching, and rules.
- [`@term`](../writing/directives.md#term): the directive, in the directive reference.
- [Glossary terms in the site output](../contracts/site-render.md): what the marker is, for a site that reads it.
