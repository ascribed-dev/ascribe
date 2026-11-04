---
type: how-to
title: Write it once
description: "Writing a name once as a phrase and a passage once as a fragment, including the fragment on several pages, or one section of it, and choosing between a phrase, a fragment, and a variant."
---

When several pages say the same thing, each copy is a place to forget when it changes. Ascribe keeps one copy: a **phrase** for a name or a value, and a **fragment** for a passage, which pages include. This guide makes a fragment of what a reader needs before they start, includes it on two pages and one section of it on a third, and adds a phrase the fragment uses.

It's for a writer with a project already, as [Getting started](../getting-started.md) sets one up, so its `ascribe.toml` declares the phrases `{product}` and `{version}`, and its content root is `docs/`.

## Phrase, fragment, or variant

Three things keep one copy of something, and each fits a different kind of repetition:

| Use | When | Written |
|---|---|---|
| A [phrase](inline.md#phrases) | A name or a value, such as a product, a version, or an address, appears in many places, in prose, headings, and links | Once in `ascribe.toml`'s `[phrases]`, and as `{key}` wherever it goes |
| A [fragment](#make-a-fragment) | A passage of a paragraph or more, such as prerequisites or a warning, is the same on several pages | Once in a file whose path starts with `_`, and included with `@include` on each page |
| A [variant](variants.md) | One page differs by reader, such as the install command for each platform | On the page, as alternatives in `@variant` arms, one for each value of a dimension |

A phrase and a fragment are the same for every reader on every page; a variant is one page that's different for different readers. They combine: a fragment can use phrases and hold variants.

## Make a fragment

A file is a **fragment** when any part of its path starts with `_`, so every file in a folder named `_fragments/` is one. A fragment isn't published on its own, has no `type`, and names nothing in its frontmatter but `intended`, so it usually has none. [Files](frontmatter.md#files) has the rest of the rule, and `ascribe.toml`'s [`[fragments]`](../content-model/index.md#4-typesname-and-fragments) can name other files fragments by pattern.

Create `docs/_fragments/prerequisites.md`:

```markdown
## Before you start
@id: prerequisites

You need {product} {version} or later, [installed](../install.md), and a docs repository {product} can read.

### Access to the repository
@id: repository-access

{product} reads the repository with a token from {cloud} that can read its contents. Put it in `quill.yaml`: see [](../configure.md#write-quillyaml).
```

Each heading has an [`@id`](directives.md#id), so a page can include its section by a name that stays when the heading's text changes. The links start with `../` because they're written from the `_fragments/` folder: [Links in a fragment](#links-in-a-fragment) says why.

## Include it on two pages

Include the whole fragment where each page needs it. In `docs/configure.md`:

```markdown
@include: _fragments/prerequisites.md

## Write quill.yaml
```

And in `docs/sync.md`:

```markdown
@include: _fragments/prerequisites.md

## Sync the repository
```

The path is relative to the page, or to the content root when it starts with `/`, so `/_fragments/prerequisites.md` works from a page in any folder. The fragment's headings keep the levels they're written with, so its `##` heading sits beside each page's own. Change the fragment, and both pages change with it.

## Include one section of it

A page that needs only part of a fragment includes the section of one heading, by its id. `docs/rotate-token.md` needs the access section without its heading, which would sit oddly under the page's own text:

```markdown
When the token expires, make a new one with the same access:

@include {heading=false}: _fragments/prerequisites.md#repository-access
```

- `#repository-access` includes that heading's section only: the heading and what's below it, up to the next heading at its level or above.
- `{heading=false}` leaves out the section's own heading, and keeps the rest. Without `#id`, it does nothing, and Ascribe warns.
- An included id is one of the page's, and ids on a page must be unique, so a page doesn't include the whole fragment and a section of it too. `ascribe check` reports the id that would be on the page twice.

## Add a phrase

The fragment names `{cloud}`, the name of the hosted product, which `ascribe.toml` doesn't declare yet. Until it does, `{cloud}` is literal text, and `ascribe check` warns about it. Add it to `[phrases]`:

```toml
[phrases]
product = "Quill"
version = "3.4.1"
cloud = "Quill Cloud"
```

Each output writes `Quill Cloud` wherever a page or a fragment writes `{cloud}`: in prose, headings, link text, and link destinations. A phrase isn't replaced in code, such as the code span `` `quill.yaml` `` above, unless a fenced code block opts in with `phrases=true`. When the hosted product is renamed, the rename is this one line.

`ascribe check` reports a phrase's value written out, such as `Quill Cloud` in a page's prose, and offers to write the phrase. [Phrases](inline.md#phrases) has every rule.

## Links in a fragment

A relative link or image path in a fragment resolves from the **fragment's** folder, not from the page that includes it. That's why the fragment above links to `../install.md`: from `docs/_fragments/`, the install page is one folder up. The link then works on every page that includes the fragment, wherever that page is.

- **Link to a page, never to a fragment.** A fragment isn't published, so a link to it is an error, and `ascribe check` offers a page that includes it instead.
- **A heading a fragment brings in belongs to each page that includes it.** Link to it through one of those pages: `[what you need](sync.md#prerequisites)`.
- **Check a link from the fragment.** `ascribe link install.md --from docs/_fragments/prerequisites.md` says what to write.

## Check it

Run the check on the project:

```sh
npx ascribe check
```

It reports a fragment path or an id that isn't there, an id that two includes put on one page, a link to a fragment, and `{cloud}` text before the phrase is declared. [`@include`](directives.md#include) has the rest of the rules.
