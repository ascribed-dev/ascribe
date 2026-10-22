# Phase 2: The landing page

Part of [The website, and the brand on everything else](README.md). Needs phase 1. It widens the thread from the hero to the whole landing page, and reads the site's other two pages against the story.

## Goal

- **A landing page built on the story,** from a mockup the maintainer approved, still written in Ascribe in `site/home/`.
- **Why Ascribe and Credits** in the story's words.

## Context

- [The story](../brand/story.md): the pillars and what proves each, what leads for each audience, the voice, and the words.
- `site/home/ascribe.toml` and its widgets: `hero-banner`, `source-and-result`, and `claim-example`. Their styles are in `site/src/styles/site.css`, and the demonstration is in `site/src/demo.ts`.
- `site/test/e2e.test.ts` and `scripts/docs-site/facts.test.ts`: what holds the landing page now, including the install commands that must match getting started's.
- Board 11, exported to `project-docs/brand/boards/`: `board.png`, with the docs site's mock in both schemes as `docs-light.png` and `docs-dark.png`. It's the reference for the look, and it has no landing page. Its lockup "with a reading face", the pixel mark beside a serif wordmark, is there for places where the pixel word would be loud. The hero may want it, though `scripts/design/assets.ts` doesn't draw it yet.
- `project-docs/mockups/`: earlier mockups, as HTML, for the form.
- [Brainstorm §18](../brainstorm.md): what the widgets carried on the first landing page, and what they didn't.

## Design

### The mockup first

`project-docs/mockups/landing.html` is one static file, in both schemes and at phone width. It uses the phase 1 tokens' values and the story's copy, word for word. The maintainer approves it, or changes it, before anything is built. What it shows is the layout. The copy is the story's, and isn't decided again here.

### The sections

In this order, which the mockup may change:

1. **The hero.** The headline, the sentence, the "early" badge, and the call to action above the fold: the install command, and a link to getting started.
2. **The demonstration.** A page's source beside what a reader sees, as now. It's the best proof of the first pillar, so it moves up.
3. **The three loud pillars.** Each is a claim with the features that prove it, each feature linked to its page in the docs by permalink:
   - **The tools of a docs team:** reusable content, variants, versions, the glossary, templates, and preview.
   - **Checked like code:** today's "Checked", "Examples from real code", and "Reviewed as pages", with their real output and screenshot.
   - **Built for AI agents, writing and reading:** the rules, the MCP server, and the hook for an agent writing pages; `llms.txt`, the Markdown versions, and the `AGENTS.md` index for one reading them.
4. **For each audience.** One short entry each for engineers, writers, and agents. Each says what leads for them, as the story has it, and links to where they start in the docs.
5. **Start.** As now.
6. **Plain files you can leave,** with **Early, and open source.** This pillar is quieter: a few sentences, not a section of its own. It names Bryo Studio as the maker.

### Widgets

Each new piece of layout is a widget in `home/ascribe.toml`, with a description, drawn by `site.css`. A pillar is likely a container holding its claim and a list of what proves it, and the audiences a container of three. If a layout can't be said with a widget, that's a finding about widgets. Write it in the pull request and in the brainstorm, and keep the page in Ascribe anyway.

### Why Ascribe and Credits

- **Why Ascribe** keeps its order by problem and its list of what Ascribe doesn't do. Its words are read against the story's list: "notes", "reusable content", "toolchain", and "docs as code".
- **Credits** keeps what it says. It's checked that it names Bryo Studio as the maker.

### The record

A changelog entry under the site's area, if `changes/README.md` gives the landing page one; otherwise none. The landing page isn't a product change.

## Tasks

1. Write the mockup and get the maintainer's approval.
2. Add the widgets and their styles.
3. Rewrite `index.md` in the approved order.
4. Read Why Ascribe and Credits against the story, and change what doesn't match.
5. Update the e2e test for the new sections, and add a check that each pillar's links land.

## Out of scope

- The split bar on notes, and the motion (phase 4).
- The words on surfaces other than the site (phase 3).
- The docs' own pages.

## Acceptance criteria

- The built landing page matches the approved mockup in both schemes and at phone width, with no sideways scroll.
- Every claim on the page links to a docs page that exists, by permalink, and every example is a snippet, tested output, or a scripted screenshot.
- `cd site && npm test` passes, with the e2e test covering the hero's call to action, each pillar, and the audiences.

## Verify

```sh
cargo build -p ascribe-cli && pnpm install
cd site && npm ci && npm run build:checkout && npm test && npm run preview
```

Then the full set in `AGENTS.md`.

## Commits

1. "Mock up the landing page from the story"
2. "Add the landing page's pillar and audience widgets"
3. "Rebuild the landing page around the pillars"
4. "Read Why Ascribe and Credits against the story"
