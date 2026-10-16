---
type: reference
title: Archive contract
description: What ascribe versions archive writes into a folder, how it records what it wrote, and what each page holds.
available: next
---

This contract says what `ascribe versions archive` writes: one line of a versioned build, rendered once more by Ascribe alone into a folder of HTML that needs nothing else. An archive is a file format people keep for years, so what's here changes only by a recorded decision. [Archive a line](../versions/index.md#archive-a-line) is the guide.

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
├── robots.txt              User-agent: *, Disallow: /
├── install.html            one file for each page
├── guide/
│   └── index.html
└── images/diagram.png      each asset, at its mirrored path
```

- **A page** is written at its source path from the content root with `.html` for its extension: `guide/index.md` is `guide/index.html`.
- **An asset** a page uses, an image or a linked file, is copied to its mirrored path, as the plain output places it ([assets contract](assets.md) §2), and referred to relatively.
- **The manifest,** `archive.manifest.json`, is a build's manifest ([output layout contract](output-layout.md) §2) with `build` the line's build, such as `site@3.4`, and `emitter` `archive`, listing every file above but itself and `.lock`.

The folder is replaced by the output layout contract's rules: a file is written only where nothing that isn't Ascribe's is in the way, a file the previous manifest lists and this archive doesn't have is removed, and a file no manifest lists is never touched. A folder with anything in it besides `.lock` and no manifest isn't an archive Ascribe wrote, and is refused before anything is written there. The archive of a line is the same bytes when made twice from one commit.

## 2. A page

Each page is one complete HTML document that loads nothing from outside the folder and runs no script:

- `<meta name="robots" content="noindex">`, and a content security policy of `script-src 'none'`.
- A `<title>` of the page's title and the line: `Install the Quill agent · self-managed 3.4`.
- One stylesheet, `style.css`, linked relatively: the element library's stylesheet, then the archive's own. Every element of the [element contract](../astro/elements.md) renders as it does without its script; tabs show one after another, each under its label.
- The statement, `<header class="archive-statement">`: "Documentation for self-managed 3.4. This version is no longer maintained.", with the target's label (content model contract §6) and the line, and with `--current` a `<span class="archive-current">` holding a link to that address.
- The sidebar, `<nav class="archive-sidebar">`: a link to the contents page, then each folder's pages, the content root's first with no name, then each folder in path order named by the title of its `index` page, or by its path. In a folder, its `index` page comes first, then the rest by path. The page shown has `aria-current="page"`.
- The page, `<main class="archive-page">`: the title as an `<h1>`, the page's availability as `<ascribe-availability scope="page">`, the page's level 2 and 3 headings as `<nav class="archive-on-this-page">`, the content as `render_site_html` renders the site output, and `<nav class="archive-pager">` with `rel="prev"` and `rel="next"` links in the sidebar's order.

## 3. Links

Every link between files of the archive is relative, starting `./` or `../`, so the folder works opened from a disk and served at any address.

- A link to a page of the line, with its `#id`, leads to that page's file.
- A link to a page the line doesn't have goes to the page that takes its place there (`replaces`), as in the line's output. A link that would lead out of the line to another line, and a link to an address under the site that no page of the line has, is written as its text, and the command lists each.
- A link with a scheme, to another site, is kept.

## 4. The other files

- **The contents page** lists every page by folder, as the sidebar does, with a heading for each named folder, and links to `all.html`. It's `index.html`, or `contents.html` when the line's root page is `index.html`.
- **`all.html`** holds every page once, in the sidebar's order, each an `<article>` with its title and availability, its links relative to the top of the folder. It has the statement and no sidebar.
- **`robots.txt`** is `User-agent: *` and `Disallow: /`, for a host that serves the folder at its own origin.

## 5. Print

The stylesheet's print rules hide the sidebar, the statement's link, and the previous and next links; keep the statement; write each link's address in parentheses after its text; and start each page of `all.html` on a new sheet.
