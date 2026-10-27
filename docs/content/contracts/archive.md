---
type: reference
title: Archive contract
description: What ascribe versions export writes into a folder, how it records what it wrote, and what each page holds.
---

This contract says what `ascribe versions export` writes: one line of a versioned build, rendered once more by Ascribe alone into a folder of HTML that needs nothing else. An archive is a file format people keep for years, so what's here changes only by a recorded decision. [Export and retire a line](../versions/archive.md) is the guide.

## 1. The folder

The archive is written into the folder `--to` names, at its top, not under a build's folder:

```text
archive/3.4/
├── archive.manifest.json   the files Ascribe owns here
├── .lock                   held while the archive is written
├── index.html              the contents page, or the root page when the line has one
├── contents.html           the contents page, when index.html is the root page
├── all.html                every page in reading order, in one document
├── style.css               the stylesheet
├── search.js               the search script
├── search-index.js         the search index
├── robots.txt              User-agent: *, Disallow: /
├── install.html            one file for each page
├── guide/
│   └── index.html
└── images/diagram.png      each asset, at its mirrored path
```

- **A page** is written at its source path from the content root with `.html` for its extension: `guide/index.md` is `guide/index.html`.
- **An asset** a page uses, an image or a linked file, is copied to its mirrored path, as the plain output places it ([assets contract](assets.md) §3.1), and referred to relatively.
- **The manifest,** `archive.manifest.json`, is a build's manifest ([output layout contract](output-layout.md) §3) with `build` the line's build, such as `site@3.4`, and `emitter` `archive`, listing every file above but itself and `.lock`.

The folder is replaced by the output layout contract's rules: a file is written only where nothing that isn't Ascribe's is in the way, a file the previous manifest lists and this archive doesn't have is removed, and a file no manifest lists is never touched. A folder with anything in it besides `.lock` and no manifest isn't an archive Ascribe wrote, and is refused before anything is written there. The archive of a line is the same bytes when made twice from one commit.

## 2. A page

Each page is one complete HTML document that Ascribe gives one script, the search's (§6):

- `<html lang="en">`. The content model has no setting for a project's language yet; once it has one, the archive takes its language from it.
- `<meta name="robots" content="noindex">`, and a content security policy of `script-src 'self' file:`. No inline script runs, and no event handler or `javascript:` link. A script file does: the search's, and any a page's own raw HTML names with `<script src>` that the browser can load, from where the folder is served, or, opened from a disk, from anywhere on the reader's disk.
- A `<title>` of the page's title and the line: `Install the Quill agent · self-managed 3.4`.
- One stylesheet, `style.css`, linked relatively: the element library's stylesheet, then the archive's own. Every element of the [element contract](../astro/elements.md) renders as it does without its script; tabs show one after another, each under its label.
- A skip link first, `<a class="archive-skip" href="#archive-main">Skip to content</a>`, out of sight until it has focus, so a keyboard reaches the page without going through the statement and the sidebar, which come before it at a phone's width.
- The statement, `<header class="archive-statement">`: "Documentation for self-managed 3.4. This version is no longer maintained.", with the target's label (content model contract §6) and the line, and with `--current` a `<span class="archive-current">` holding a link to that address.
- The sidebar, `<nav class="archive-sidebar">`: the search script's `<script>` (§6), a link to the contents page, then the line's [navigation](../navigation/index.md) as its site's sidebar has it: the content root's pages first with no name, then each top section under its label, its own page first, then its pages, each nested section a list of its own in its place, headed by a link to its own page with the section's label. Each page is named by its label, its `nav-label` or else its title. A page that's `unlisted` isn't in it. The page shown has `aria-current="page"`.
- The page, `<main id="archive-main" class="archive-page">`: the title as an `<h1>`, the page's availability as `<ascribe-availability scope="page">`, the page's level 2 and 3 headings as `<nav class="archive-on-this-page">`, the content as `render_site_html` renders the site output, and `<nav class="archive-pager">` with `rel="prev"` and `rel="next"` links in the navigation's reading order (the [navigation contract](navigation.md), §3), each named by the page's label. An `unlisted` page has none.

## 3. Links

Every link between files of the archive is relative, starting `./` or `../`, so the folder works opened from a disk and served at any address.

- A link to a page of the line, with its `#id`, leads to that page's file.
- A link to a page the line doesn't have goes to the page that takes its place there (`replaces`), as in the line's output. A link that would lead out of the line to another line, and a link to an address under the site that no page of the line has, is written as its text, and the command lists each.
- A link with a scheme, to another site, is kept.

## 4. The other files

- **The contents page** lists the pages as the sidebar does, with a heading for each top section, and links to `all.html`. It's `index.html`, or `contents.html` when the line's root page is `index.html`.
- **`all.html`** holds every page once, in reading order, the `unlisted` ones last, in path order, each an `<article>` with its title and availability, its links relative to the top of the folder, in `<main id="archive-main" class="archive-all">`. It has the skip link and the statement and no sidebar, so no search. It's for printing the set, so its ids aren't unique: two pages with a heading of the same name both have its id, and a link to it in `all.html` goes to the first.
- **`robots.txt`** is `User-agent: *` and `Disallow: /`, for a host that serves the folder at its own origin.

## 5. Print

The stylesheet's print rules hide the skip link, the sidebar, the search box, the statement's link, and the previous and next links; keep the statement; write each link's address in parentheses after its text; and start each page of `all.html` on a new sheet.

## 6. Search

Every page with a sidebar, the contents page too, can search the archive. Nothing is requested from outside the folder, and nothing is stored in the browser.

- **The script,** `search.js`, is the same file in every archive one version of Ascribe writes. The sidebar starts with `<script src="…/search.js" data-index="…/search-index.js" data-root="…/" defer>`, each address relative to the page. The script puts the box in that element's place, a `<div class="archive-search" role="search">` with a labeled search field, a `role="status"` line that says how many results there are, and the results as links; with scripts off there's no box.
- **The index,** `search-index.js`, is a script that sets `globalThis.ascribeArchiveIndex`, since a page opened from a disk can't fetch a file. The box adds it to the page the first time it's used. Its value is JSON: `version` (1), and `pages`, every page in `all.html`'s order, each with `href`, its file's address from the top of the folder; `title`, as text; and `sections`, its text by heading, the text before its first heading first, each with `id`, the heading's id on the page (empty before the first heading); `heading`, as text; and `text`, whitespace collapsed. A page's text is taken from the resolved page, not its HTML: paragraphs, list items, table cells, code blocks, and directives' titles, without raw HTML. The archive of a line is the same bytes when made twice from one commit, its index included.
- **A query** finds the pages and headings that have every one of its words, ignoring case, a word matching inside a longer one: first each page whose title has them, then each heading that has them, then each heading whose text has them, each in `all.html`'s order and each address once. A result links to the page at its heading, and shows the words around the match with the query's words marked. Twenty are listed, and the status line says how many there are.
- **The command says the index's size,** and when it's over 8 MB, that a reader's first search waits for it to load. It writes it whatever its size.
