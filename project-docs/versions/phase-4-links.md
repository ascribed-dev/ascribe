# Phase 4: Links between lines

Part of [Versions](README.md). Needs phase 2.

## Goal

A page added or rewritten inside the window costs no marks on the pages that link to it. In a versioned build, a link to a page that isn't in a line goes to the page that takes its place there, or out to the nearest line that has it, and says so. A page rewritten for a new line says which page it replaces, and the list of a page's lines follows.

After phase 2, every such link is an error, and the only way out is an `@available` on each paragraph that links. This phase is what makes a window usable past its first new page.

## Context

- README decisions 12 and 13.
- The [proposal](../proposals/versions.md): "Links to a page that isn't in a line", and "A page that replaces another".
- SPEC §5.2 (links) and §2.1 (reserved frontmatter keys).
- `crates/ascribe-resolve/src/build/links.rs`: where a link resolves, and where `link-page-dropped` is reported.
- `docs/content/contracts/site-render.md`: what a link is in the site output.
- `versions.json`, as phase 2 writes it.

## Design

### `replaces`

A reserved frontmatter key for pages, beside `available` and `variant`. Its value is a link destination to another page, written as any link to a page is.

```yaml
---
title: Install the agent
available: self-managed 3.5
replaces: install-legacy.md
---
```

Checked like a link, each an error at the page:

- The file exists and is a page.
- The two pages' availability for the build's target doesn't overlap: no line publishes both.
- A page doesn't replace itself, and no chain of `replaces` comes back to where it started.
- Two pages don't replace the same page in the same line.

A fragment can't use it, and a type can't declare a field with that name. A project whose types already do has to rename it, so it's a behavior change in the changelog entry.

It's read in every build and acts only in a versioned one. In a build that isn't versioned, the two are different pages.

### How a link resolves in a line

In order:

1. The page, when the line publishes it.
2. The page that replaces it, or that it replaces, in this line, following a chain to the one this line publishes.
3. The page in the nearest line that publishes it, as a link out of this line. Nearest is the closest newer line, then the closest older.

When no live line publishes the target, it's `link-page-dropped`, as now.

A link's fragment is kept through steps 2 and 3 when the page it lands on has a heading with that id, and dropped with advice when it doesn't.

### A link out, marked

A link out of a line says which line it goes to, and its text stays what the author wrote, or the target's title.

- **In the site output** the link carries an attribute with the line, and the two renderers agree on it. The element library's stylesheet shows the line after the link, as it shows "Available:" before a badge, so a site can restyle it, reword it, or hide it, and a reader with no stylesheet gets a plain link.
- **In the plain output** the line follows the link, outside its text: `[Configure tracing](https://…/configure-tracing/) (3.5)`. An agent reading the Markdown sees the target's real title as the link's text, and the line as a fact about where it leads.

A link whose text Ascribe fills from its target takes the target's title as it is in the line it lands on.

### What's reported

- **A link from the current line into an older one** is advice, next step `write`: it's usually a sign the newer page is missing.
- **A link that resolved by step 2 or 3** is reported nowhere. It's what the feature is for. `ascribe link` and `ascribe render` say how a link resolved in a line, for someone who asks.

### The shared file

A page's entry in `versions.json` gains `replaces` and `replaced-by`, each a source path. The lines a page is in don't change shape.

`Versions.astro` follows them: on the old page, the newer line's entry leads to the page that replaced it, and back.

## Tasks

1. `replaces`: the key, its checks, and the reserved-key errors.
2. The three steps, in `ascribe-resolve`, with conformance cases for each, for a chain, and for a fragment kept and dropped.
3. The link out's attribute in both renderers, the stylesheet's rule, and the plain output's suffix, with fixtures.
4. The advice for a link back into an older line.
5. `ascribe link` and `ascribe render` saying how a link resolved.
6. `replaces` and `replaced-by` in `versions.json`, and `Versions.astro` following them.
7. `examples/quill`: a page new in the newer line, linked from a shared page, and a page rewritten for it.
8. SPEC §2.1 and §5.2, the reference, the contracts, the guide, an entry in `changes/` that says **Behavior change**, and the decisions recorded.

## Out of scope

- A link to an archived line. Phase 8.
- Redirects from an old page's address to its replacement. README decision 21.

## Acceptance criteria

- In `examples/quill`, the shared page's link to the page new in 3.5 builds in `site@3.4` as a link out, with its line in an attribute and its text unchanged, and with no mark on the paragraph. In the plain output "(3.5)" follows the link and isn't inside its text.
- The rewritten page's old address in 3.4 and new address in 3.5 each list the other in the page's lines.
- Two pages that replace each other, a page that replaces itself, and two pages whose availability overlaps are each an error naming both files.
- A link from the current line to a page only 3.4 has is advice.
- A build that isn't versioned is byte for byte the same.

## Verify

```sh
cargo test --workspace --locked
pnpm test
pnpm --filter @ascribed/example-astro-site test:e2e
node scripts/compare/outputs.ts --base main --accept 'quill: *'
```

## Commits

1. "Let a page say which page it replaces"
2. "Resolve a link to a page that isn't in a line"
3. "Mark a link that leaves its line"
4. "Follow a replaced page in the list of a page's lines"
