# Phase 3: The landing page

Part of [The website](README.md). Requires phase 2. The marketing project, `site/`, and `design/`.

## Goal

The landing page makes its whole case, and everything on it is checked or marked:

1. What Ascribe is, with its status.
2. A page's source beside what a reader sees, both real.
3. Four claims, each with the smallest example that shows it.
4. How to start.
5. Where it's going.

And the plan's experiment has its answer, written down: whether a handful of project widgets carry a marketing page.

## Context

- The [proposal](../proposals/website.md): "The landing page", "Saying what's built and what isn't", and "Written in Ascribe".
- README decisions 4 and 5, and phase 1's decisions 3 and 4.
- SPEC §6 and §7.2: what a widget's schema can say (forms, primary kind, attributes, title, groupable, plain fallback).
- `docs/content/contracts/site-render.md`: what the site output writes for a widget, and for availability.
- `@snippet` in `docs/content/reference/directives.md`, and `[sources.code]` in `docs/ascribe.toml`: reading an example from a file that's built or tested.
- `packages/astro/src/Availability.astro`, and how a docs page marks what no release has.
- `ascribe render`, in the command reference: what it writes is the plain output's Markdown, which is why the demonstration doesn't use it.
- README decisions 11 and 13: a list of named projects, and the marketing project's release tables and `[sources.code]`, which phase 2 declared.
- `examples/getting-started/`: a real project a demonstration could use.
- `site/home/` and `site/src/`, as phase 2 left them.
- `design/tokens.toml` and `design/README.md`, and the contrast test.

## Design

### The widgets

Named as they're needed, each declared in `site/home/ascribe.toml` and drawn by the site. The proposal's guess is four or five: the hero (phase 2), a claim with its example, a source beside its result, a call to action, and a list of what's next.

Keep to what a widget's schema can say. A widget that needs a new form, a new kind of attribute, or content the language can't hold is a finding, not a reason to change the language (README decision 5).

### The demonstration

A page's source on one side and its result on the other.

- **The source** is a real file in the repository, read with `@snippet`, so it can't fall behind.
- **The result** is that file as a site draws it: the file is a page of a project the site builds, and the landing page shows that page's rendered content. `ascribe render` can't do this. It writes a build's plain Markdown, so notes, tabs, and marks aren't drawn, and no release has the command yet while the site installs from npm.
- **That makes a third project in the list,** such as `{ name: "demo", project: "demo", build: "site" }`, holding the demonstrated page and its small content model. It has a base path of its own and no route: the site draws its one page inside the landing page and nowhere else. It's the second real use of phase 2's feature, and the first of a project whose pages a site places itself.
- **A test fails** if the source shown and the page rendered aren't the same file.

**This is the phase's risk.** A widget whose content is a path, with the site drawing another project's page in its place, may be more than a widget should carry. If so, the demonstration is an Astro component that the page's frame places, the rest of the page stays in Ascribe, and the finding says so. A third project may also show phase 2 something two didn't; if it does, that's fixed in the integration, not worked round here.

### The claims

Checked; one source, several outputs; examples from real code; reviewed as pages. Each has one example:

- an example of the first three is real: a diagnostic `ascribe check` actually prints for a file in the repository, a variant drawn by the tabs' own element, a snippet read from tested code;
- the fourth can't be shown without a picture, which is phase 6. Until then it's described in words.

A claim about a capability no release has carries its availability, drawn by the element the docs use. Check each of the four against the changelog's released sections when writing it. The marks use the release dimension, the `next` feature, and `[versions] current` that phase 2 declared in the marketing project and holds equal to the docs', so `availability-left-behind` reports a mark that's still there after its release (README decision 16). The examples are read through the project's `[sources.code]`; a folder an example needs that it doesn't include yet is added there and in the docs' copy's test.

"Examples from real code" claims what `@snippet` and `ascribe drift` do today. It doesn't claim tested examples, which is a [proposal](../proposals/tested-examples.md).

### Where it's going

A short list written from the plans and proposals in `project-docs/`, each item one sentence. It links to nothing in `project-docs/`, since those files move and are removed. A test doesn't hold it; `RELEASING.md` gets a step to reread it.

### The look

- The widgets' styles, from tokens. Any size or color they need is added to `design/tokens.toml`.
- Light and dark follow the system, and the contrast test covers each new pairing.
- The page works at phone width, with no script needed to read it.

### The finding

One of these, written where it belongs:

- **Widgets carried the page.** `docs/content/reference/content-model.md`'s section on widgets, or a guide, gains the landing page as its example.
- **They carried part of it.** The same, with what didn't fit said plainly in the [brainstorm](../brainstorm.md).
- **The page fought the language.** The pages become plain Astro pages that take their demonstrations from Ascribe, the marketing project is removed or reduced, and the brainstorm says what was learned. Phases 4 to 6 don't depend on which.

## Tasks

1. The demonstration: the third project and its page, the source, the result, the test that ties them, and its widget or component.
2. The claims, with their examples read through `[sources.code]` and their marks from the release tables.
3. How to start, as README decision 15 has it, with the test that holds the two install commands equal to the README's "Get started" block.
4. Where it's going, and the step in `RELEASING.md`.
5. Styles and tokens, with the contrast test.
6. The site's tests: the page's links, the marks on what's unreleased, phone width, and reading without a script.
7. The finding, written down.
8. The maintainer reads the page before it merges. More people will read these claims than any other page's.

## Out of scope

Screenshots (phase 6). Any change to the language or to what a widget's schema can say. The other pages.

## Acceptance criteria

- Changing the demonstration's source file changes both sides of it on the next build, and a test fails if they could differ.
- Every example on the page comes from a file that's built or tested. `ascribe check` passes on the marketing project.
- Each claim about something no release has is marked. Moving `[versions] current` past a marked claim's release, in a copy, makes `ascribe check` report `availability-left-behind` on the marketing project: tried once, and said in the pull request.
- The finding is written in the docs or the brainstorm, and the pull request quotes it.
- The maintainer has approved the text.

## Verify

```sh
cargo run -p ascribe-cli -- check site/home
pnpm test
cd site && npm run build:checkout && npm run typecheck && npm test
```

## Commits

1. "Show a page's source beside its result on the landing page"
2. "Make the landing page's claims, each with its example"
3. "Record what writing the landing page in Ascribe showed"
