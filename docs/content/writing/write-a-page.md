---
type: how-to
title: Write a page
description: "Build one real page with what most pages use: a note, steps, a collapsible block, a link to another page, an image with a width, a phrase, and an include."
permalink: write-a-page
---

Most pages need the same few things: a callout, a procedure, a block readers can open when they want the detail, links to the pages around it, a picture, a name written once, and a part several pages share. This guide builds one page that has each of them, a step at a time, and says where the rest of each one's rules are.

It's for a writer with a project already, as [Getting started](../getting-started.md) sets one up. The page is the `configure.md` that guide's `install.md` links to, so its `ascribe.toml` already declares the phrases `{product}` and `{version}` used below.

## 1. Start the page
@id: start-the-page

Create `docs/configure.md` with its frontmatter, the fields its type asks for, and an opening paragraph:

```markdown
---
type: page
title: Configure Quill
description: Set up quill.yaml, connect Quill to your repository, and check the result.
---

{product} reads its settings from `quill.yaml`, at the root of your repository.
```

`type: page` is a type `ascribe.toml` declares, and `title` and `description` are the fields it asks for; [frontmatter](frontmatter.md) has the rest of what a page's frontmatter can say.

## 2. Write a name once, as a phrase
@id: write-a-name-once-as-a-phrase

`{product}` above is a **phrase**: `ascribe.toml`'s `[phrases]` says what it stands for, and each output writes that in its place. Write the phrase wherever the product's name or version goes, in prose, headings, and links, so a rename or a release is one edit:

```markdown
This page is for {product} {version} and later.
```

A phrase isn't replaced in code. A fenced code block opts in with `phrases=true` in its info string:

````markdown
```yaml phrases=true
agent:
  version: {version}
```
````

`ascribe check` reports a phrase's value written out, such as `Quill 3.4.1`, and offers to write the phrase. See [phrases](inline.md#phrases).

## 3. Add a note
@id: add-a-note

A note is a callout. Write `@note`, its type if it isn't a plain note, and its text after a colon:

```markdown
@note {type=warning}: `quill init` overwrites an existing `quill.yaml`.
```

The types are `note`, `tip`, `important`, `warning`, and `caution`. For a note longer than a paragraph, end the line with a colon, write the blocks below it, and close it with `@end`:

```markdown
@note {type=tip}:
Keep `quill.yaml` in version control.

Every clone then syncs the same folders.
@end
```

A line starting with `.` directly above the note gives it a title, as `.Before you start` would. See [`@note`](directives.md#note).

## 4. Write the steps
@id: write-the-steps

Put `@steps` directly above a numbered list to mark it as a procedure. The site shows it as one, and readers and agents can tell it from any other list:

```markdown
@steps
1. Run `quill init` to write `quill.yaml`.
2. Set `watch` to the folder your docs are in.
3. Run `quill check` to confirm the agent can read it.
```

A directive inside a step, such as a note, is indented to the step's text. See [`@steps`](directives.md#steps).

## 5. Fold away the detail
@id: fold-away-the-detail

Some content most readers skip: every option of a file, or a long log. `@details` shows it collapsed, under a title readers click to open, so the title is required:

````markdown
.Every option of quill.yaml
@details:
| Option | Meaning |
|---|---|
| `watch` | The folder the agent syncs. |
| `interval` | Seconds between syncs, when streaming is off. |
@end
````

Without the colon, `@details` folds away just the next block. See [`@details`](directives.md#details).

## 6. Link to another page
@id: link-to-another-page

A link names the page's **file**, with a path relative to the page you're writing, and Ascribe writes each output's address for it. Add a heading's id after `#` to link to a section:

```markdown
To change the key the agent uses, see [Rotate API keys](keys.md#rotate-keys).
Install it first: [](install.md).
```

Leave the text empty, as in the second link, and Ascribe fills in the page's title, or the heading's. `ascribe check` reports a link to a file or a heading that isn't there, so a page that moves doesn't leave a broken link behind. `ascribe outline <page>` lists a page's headings with their ids, and `ascribe link <target> --from <page>` says what to write. See [links](inline.md#links).

## 7. Add an image with a width
@id: add-an-image-with-a-width

An image is a Markdown image with alt text. Put the image file in your content folder, beside the page or in a folder of images, and link it by its path from the page:

```markdown
![The {product} settings page, with Watch set to docs](images/settings.png){width=600}
```

The `{width=600}` block, directly after the image with no space, gives it attributes. Ascribe accepts only the attributes `ascribe.toml` declares, so declare `width` once:

```toml
[images.attributes]
width = "number?"
```

`ascribe check` reports an image that isn't there, or whose name differs in case, and one without alt text. See [images](inline.md#images).

## 8. Include a shared part
@id: include-a-shared-part

When several pages say the same thing, such as what a reader needs before they start, write it once in a **fragment**, a file whose name starts with `_`, and include it in each page.

Create `docs/_prerequisites.md`, with no frontmatter:

```markdown
Before you configure {product}, you need:

- {product} {version} or later, [installed](install.md).
- A docs repository {product} can read.
```

Then include it where the page needs it:

```markdown
## Before you start

@include: _prerequisites.md
```

The path is relative to the page. The fragment's phrases, links, and images work as if it were written in the page, and a link in it is relative to the fragment, so it works on every page that includes it. A fragment isn't a page: link to a page that includes it, not to the fragment. See [`@include`](directives.md#include).

## The whole page

Put together, `docs/configure.md` reads:

````markdown
---
type: page
title: Configure Quill
description: Set up quill.yaml, connect Quill to your repository, and check the result.
---

{product} reads its settings from `quill.yaml`, at the root of your repository.

## Before you start

@include: _prerequisites.md

## Write quill.yaml

@note {type=warning}: `quill init` overwrites an existing `quill.yaml`.

@steps
1. Run `quill init` to write `quill.yaml`.
2. Set `watch` to the folder your docs are in.
3. Run `quill check` to confirm the agent can read it.

![The {product} settings page, with Watch set to docs](images/settings.png){width=600}

.Every option of quill.yaml
@details:
| Option | Meaning |
|---|---|
| `watch` | The folder the agent syncs. |
| `interval` | Seconds between syncs, when streaming is off. |
@end

## Next

To change the key the agent uses, see [Rotate API keys](keys.md#rotate-keys).
````

## Check it

Run the check on the page:

```sh
npx ascribe check docs/configure.md
```

It reports anything the page names that isn't there: here, `images/settings.png` until you add the picture, and `keys.md` until that page exists. In VS Code, the editor reports the same problems as you type, and its actions bar writes directives for you ([Writing in the editor](../editor/writing.md)).

## Next

- The [directive reference](directives.md) opens with the directives most pages use, and has every rule of each.
- [Write it once](write-it-once.md) shares a fragment among pages, or one section of it, and says when a phrase, a fragment, or a variant fits.
- [Content that varies](variants.md) adds content that differs by platform or product, as `install.md`'s tabs do.
- [Leave a memo for whoever edits next](memos.md) keeps the reasons for a page beside it, unpublished.
