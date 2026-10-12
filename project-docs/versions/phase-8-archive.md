# Phase 8: The archive

Part of [Versions](README.md). Needs phases 4 and 5.

## Goal

A line is rendered once more, by Ascribe alone, into a folder that needs nothing else.

```console
$ ascribe versions archive "self-managed 3.4" --to archive/3.4
Wrote 40 pages to archive/3.4. Open archive/3.4/index.html.
Host the folder, then record its address:
  [releases.self-managed.archived]
  "3.4" = { url = "…" }
```

Every page says what it is, the folder has a contents page and a sidebar, it prints, it's marked so crawlers leave it alone, and it works opened from a disk with no server.

This is the fourth level's own thread. Its seam is one Ascribe hasn't crossed: its renderer draws one page with no layout today, and here it draws a whole site a person can find their way around.

## Context

- README decisions 15 and 18.
- The [proposal](../proposals/versions.md): "Archive first".
- `crates/ascribe-emit/src/render/`: `render_site_html`, the renderer that needs no site generator. `ARCHITECTURE.md`, "Two HTML renderers".
- `crates/ascribe-diff/src/html/`: the review report, the one self-contained HTML Ascribe writes today, and how its script and stylesheet are embedded from `packages/review` and `packages/elements`.
- `design/README.md`: colors are written in `design/tokens.toml` only, and a stylesheet in the binary is an output.
- `docs/content/contracts/assets.md`: how a page's images and files are copied.

## Decisions first

Two choices open this phase, each recorded before code, since an archive is a file format people keep for years:

1. **Which emitter it is.** Recommended: a fourth emitter, `archive`, written only by this command and never by `ascribe build`, with the store pointed at `--to` instead of the output directory and its manifest written there. The store is rooted in the output directory today, and an archive is deliberately somewhere else, so this is the one change the store needs: a root it's given. Then the ownership rules, the asset contract, and the refusal of a folder Ascribe didn't write apply as they are. The other way is a writer of its own beside the review report's, which would repeat the asset copying and the ownership rules.
2. **What the elements do with no script.** Recommended: an archive ships the element library's stylesheet and no script. Tabs are drawn as labeled sections, one after another, which is what the element contract already promises without JavaScript, and what prints. An archive with a script would depend on it working in a browser ten years on.

## Design

### What's rendered

The source as it stands, for the line, with phase 2's rule for what's in a line and phase 4's rule for links, with one difference: nothing links out. A link to a page that isn't in the line goes to the page that takes its place, and when there's none it's text, with advice at the command naming each one.

The line may be live or planned. The command says so when it's the current line, since archiving that is unusual, and goes on.

### A page

Each page is one HTML file, complete: its own `<head>`, the stylesheet inline or beside it in the folder, no request to anywhere else, and no script.

- **A statement, at the top of every page,** in words that don't age: "Documentation for Self-managed 3.4. This version is no longer maintained." With `--current <url>`, a link to the current documentation follows. No date that will read wrongly later, and no "latest".
- **`<meta name="robots" content="noindex">`** on every page, and a `robots.txt` at the folder's root for a host that serves the folder at its own origin.
- **The title, the availability, the contents of the page,** from its headings.
- **Links between pages are relative,** so the folder works from a disk and at any address.

### A way around

The content model has no navigation, so the archive writes one:

- **A sidebar on every page:** the folders, each named by the title of its `index.md` or by the folder, and the pages under each by title, in path order, with the current page marked.
- **A contents page,** `index.html` at the root when the project's root page isn't there to be it, and `contents.html` otherwise: every page, by folder.
- **Previous and next,** in that order.

It needs no script: the sidebar is in each page's HTML. With many pages that's weight in every file, and the phase measures it on the docs and says what it is.

### Print

A print stylesheet: the sidebar and the statement's link go, the statement stays, a page breaks before a top heading where it can, and a link's address is printed after its text. And one more file, `all.html`: every page in order, in one document, so a browser can make one PDF of the set.

### The command

`ascribe versions archive "<target> <line>" --to <DIR>`

- `--to` is from the current folder, and required: an archive doesn't go in the output directory, which is deleted and rebuilt.
- It refuses a folder that isn't empty and isn't an archive it wrote, which the store decides from the manifest in the folder, as it does for a build's. Making an archive again in the same folder replaces what the manifest lists and nothing else.
- `--build <name>` picks the versioned build when a project has two for the target.
- `--current <url>` is the address the statement links to.
- `--format text|json`. Exit 0, or 2.
- It writes nothing into the project. The text says what to add to `[releases]`.

### An archived line where a reader sees it

`versions.json` already carries an archived line's `url`. `Versions.astro` lists archived lines that have one, after the live ones, as links out of the site. A line with no address isn't listed.

## Tasks

1. Record the two decisions.
2. The store given its root, and the archive emitter: pages, assets, and the manifest in the folder.
3. The statement, the robots marks, and relative links.
4. The sidebar, the contents page, previous and next.
5. The print stylesheet and `all.html`.
6. The command.
7. Tests: the folder opened with `file:` in a browser, with no network, by the end-to-end suite; every link followed; no request leaves the folder.
8. Archive `examples/quill`'s older line in CI and keep the folder as an artifact, so each pull request shows one.
9. The command reference, a contract for the archive's layout, the guide's section, an entry in `changes/`, and the map of the surface.

## Out of scope

- Search. Phase 10.
- Moving the line to `archived`, and cleaning up. Phase 9.
- Hosting, and a PDF engine.
- A theme. An archive looks like Ascribe's own render, and a project's site design isn't carried into it.

## Acceptance criteria

- `ascribe versions archive "self-managed 3.4" --to <dir>` in `examples/quill` writes a folder that opens from disk with the network off, and every link in it leads to a file in it.
- No page makes a request outside the folder, and none has a script.
- Every page carries the statement and `noindex`.
- A page can be reached from the contents page, and from any other page by the sidebar.
- Printing a page hides the sidebar, and `all.html` holds every page once.
- The archive of a line is the same bytes when made twice from one commit.

## Verify

```sh
cargo test --workspace --locked
cargo run -p ascribe-cli -- versions archive "self-managed 3.4" --to /tmp/quill-3.4 --config examples/quill
pnpm test
```

## Commits

1. "Record how an archive is written"
2. "Render a line into a folder that needs nothing else"
3. "Give an archive a way around"
4. "Make an archive print"
5. "List a project's archived lines beside its live ones"
