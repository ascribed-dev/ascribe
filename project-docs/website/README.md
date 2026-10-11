# The website

Someone who lands on <https://ascribed-dev.com> learns what Ascribe is and why it exists before they're handed a reference manual. The landing page takes `/`, and the docs move to `/docs/`. The idea is in the [proposal](../proposals/website.md). This is the plan for building it.

## Who it's for

- **Someone who has just heard of Ascribe,** and wants to know what it is, whether it's usable yet, and how to try it.
- **Someone comparing tools,** who wants the argument at length and the limits stated.
- **Anyone with a home page in front of their docs.** What Ascribe's own site needs from `@ascribed/astro`, their site needs too.

## What exists today

- **The site's root is the docs index.** `site/` is plain Astro with `@ascribed/astro`, one route (`src/pages/[...slug].astro`), and one layout with the docs' sidebar and table of contents. `docs/ascribe.toml` sets `[consumer] base-path = "/"`.
- **A base path works, and a home page in front of it doesn't.** `consumerMismatches` in `packages/astro/src/project.ts` requires Astro's `base` to equal `[consumer] base-path`, and Astro's `base` covers the whole site.
- **The integration takes one project.** One site output root (`virtual:ascribe/site`), one collection loader, one route for permalinks, one `.ascribe/dev.json`, one review server, one toolbar app.
- **The binary's links don't depend on where the docs are.** Every address Ascribe writes is a permalink under `/go/`, set from the site's root by `[consumer] permalink-path` (decision 71 in [decisions.md](../decisions.md)). Package READMEs and crate READMEs still link to pages by their full address.
- **What agents read is published at the base path:** `llms.txt` and each page's Markdown, copied by `copyAgentFiles`. A Netlify edge function (`site/netlify/edge-functions/markdown.ts`) answers a page asked for as Markdown.
- **Search is Pagefind,** run over the whole of `dist/` after the build.
- **Project widgets exist** (SPEC §6, `[widgets.<name>]`): a directive a project declares, written by the site output as a custom element with the widget's name and attributes. Ascribe's docs declare none.
- **The look is tokens.** `design/tokens.toml` holds every color and type size, and the site's stylesheet is generated from it. The scale was made for documentation.
- **Tests hold the site to the root:** `site/test/` (navigation, links, redirects, the built site in Chromium), `scripts/docs-site/`, and the tests that compare the site's address in the READMEs and the extension's manifest with `docs/ascribe.toml`.
- **Production follows `main`** through the nightly canary: the site installs `@ascribed/astro` from npm's `next` tag, so it gets a change to the integration the night after it merges.

## Decisions

These are settled. Don't reopen them in a phase; if one can't be met, stop and report.

1. **One site, with a new root.** The landing page is at `/`, and the docs are under `/docs/`.
2. **Three pages to start:** the landing page, "Why Ascribe", and the changelog.
3. **The pitch is docs treated like code:** written in files, checked, reviewed, and built, with the rigor of the product they describe.
4. **It says what's being built, and is plain about what works today.** A claim about something no release has carries the mark the docs use for it: availability, drawn by the same element. Nobody keeps a second list.
5. **The pages are written in Ascribe where that's practical,** with project widgets and no change to the language. Whether that works is a finding this plan produces. If it doesn't, the pages are plain Astro pages that take their demonstrations from Ascribe, and the finding goes in the [brainstorm](../brainstorm.md).
6. **The changelog is `CHANGELOG.md`,** rendered by the site. The file stays where it is and doesn't become an Ascribe page.
7. **A home page in front of the docs is a feature of `@ascribed/astro`,** documented and tested like any other, not a special case for this site.
8. **No work is done for the docs' old addresses.** Ascribe is before 1.0 and has no users, so nothing redirects `/guides/…` to `/docs/guides/…`.
9. **The site collects nothing.** No analytics, no sign-up, no form.
10. **Colors and type sizes are written in `design/tokens.toml` only.** What the marketing pages need is added there.

The rest were made in [phase 1](phase-1-decisions.md), which has the options each was chosen from. Each fixes something phase 2 ships: a published package's options, a folder, and what the first page says.

11. **One integration takes a list of projects, and each entry has a name.** Recorded as [decision 82](../decisions.md#the-website), where it's kept with the rest of its text.
12. **The fallback is one project with the docs in a subfolder.** Recorded as [decision 83](../decisions.md#the-website), where it's kept with the rest of its text.
13. **Two projects are two content models, and the marketing project's copies are held to the docs'.** Recorded as [decision 84](../decisions.md#the-website), where it's kept with the rest of its text.
14. **The marketing project is a folder named `home` in `site/`.** Recorded as [decision 85](../decisions.md#the-website), where it's kept with the rest of its text.
15. **The landing page asks a visitor to read getting started first.** Recorded as [decision 86](../decisions.md#the-website), where it's kept with the rest of its text.
16. **The marketing pages follow `main`, with marks.** Recorded as [decision 87](../decisions.md#the-website), where it's kept with the rest of its text.
17. **"Why Ascribe" describes approaches and links to the comparison, once the comparison is fit to link.** Recorded as [decision 88](../decisions.md#the-website), where it's kept with the rest of its text.
18. **Addresses.** Recorded as [decision 89](../decisions.md#the-website), where it's kept with the rest of its text.
19. **The marketing project publishes nothing for agents.** Recorded as [decision 90](../decisions.md#the-website), where it's kept with the rest of its text.
20. **Search covers the docs only.** Recorded as [decision 91](../decisions.md#the-website), where it's kept with the rest of its text.

One decision is still to make, and it blocks nothing before it: whether a social card shows its page's title, and with what font ([phase 6](phase-6-images.md#social-cards)).

## The thread

**The use case:** opening `https://ascribed-dev.com/` shows a landing page written as an Ascribe page, its link to getting started opens `/docs/getting-started/`, and `ascribe check --help`'s link still opens the section about `ascribe check`.

**The layers it crosses:**

| Layer | Where | What the thread needs from it |
|---|---|---|
| The integration | `@ascribed/astro` | A base path under Astro's `base`, and a second project in one site |
| The docs' project | `docs/ascribe.toml` | `base-path = "/docs/"` |
| A second project | A new folder, with its own `ascribe.toml` | One page type, one widget, one page |
| The site | `site/` | A frame without the docs' furniture, a route for each project, the widget drawn |
| The look | `design/tokens.toml` | A type size for a hero |
| What repeats the docs' addresses | READMEs, the extension's manifest, workflows, Netlify's files | Each moved, with the test that holds it |
| The product | `ascribe-cli` | Nothing. That's the claim permalinks made, and this tests it |

**The seam it tests** is one Astro site built from more than one Ascribe project, each under its own path. The integration assumes one project in six places, and nobody has tried a second. Three things would be learned if it fails:

- **If a base path under `base` can't be served well** (in `astro build` and `astro dev`, with published files, agents' files, and permalinks landing), the docs can't move without Astro's `base` moving too, and a home page has to be a second deploy. That would change decision 1.
- **If two projects can't share a site** (their permalinks, their dev rebuilds, review), the marketing pages become plain Astro pages now, not after the experiment.
- **If links from one project to the other can't be checked,** the pages need a check of their own in the site's tests.

The second seam is smaller: a page of project widgets, drawn by a site. The thread crosses it with one widget. Phase 3 is where it's tested properly.

**Left out, and where it comes back:**

| Left out | Phase |
|---|---|
| The demonstration, the four claims, what's next, marks for what's unreleased | 3 |
| The changelog | 4 |
| Why Ascribe | 5 |
| Screenshots, and a social card for each page | 6 |

After phase 2 the landing page is short and true: what Ascribe is, that it's early, and how to start. It's live the night after it merges.

## Phases

| Phase | Result |
|---|---|
| [1: Decisions](phase-1-decisions.md) | The maintainer settles how a site takes several projects, where the pages live, and what the landing page asks of a visitor. |
| [2: The thread](phase-2-thread.md) | The site's root is a landing page written in Ascribe, and the docs are at `/docs/`. Any Astro site can do the same. |
| [3: The landing page](phase-3-landing-page.md) | The page shows a real source beside its result, makes its four claims, and marks what no release has. The finding about widgets is recorded. |
| [4: The changelog](phase-4-changelog.md) | `/changelog/` is `CHANGELOG.md`, with its links leading into the site. |
| [5: Why Ascribe](phase-5-why.md) | `/why/` makes the argument at length and names its limits. |
| [6: Images](phase-6-images.md) | The editor and review are shown, each page has a social card, and a test says when a screenshot is out of date. |

Phase 1 came first because phase 2 ships a published package's options. After the thread the order is by risk: phase 3 is the experiment that could change how the other pages are written, phase 4 is the one new mechanism, and phases 5 and 6 are writing and pictures.

Each phase's result is something a person can open. None builds a layer for a later phase to use.

## The other plans

- **Permalinks** is what lets the docs move. This plan is its first real test: if a link the binary writes breaks in phase 2, that's a bug in that work, fixed there.
- **Search and answers** ([proposal](../proposals/search-and-answers.md)) owns `robots.txt`, a sitemap, and what a page's head says. This plan writes none of them by hand, so there's nothing to replace later. Phase 2 moves `llms.txt` with the docs and leaves the site's root without one.
- **Versions** ([proposal](../proposals/versions.md)) needs a base path under Astro's `base` too, for a version published under its number. Phase 2 builds it once.
- **The surface audit** ([findings](../surface-audit/findings.md)) has three neighbors of this plan. Its finding 32 is the site's address written in two files that must agree, which a second project makes three. Finding 43 is the READMEs linking to pages by address: if they link to permalinks first, phase 2's move of them is nothing. And its conventions say where a choice lives: the integration's options are for how a site uses a build.

## Rules for every phase

- Branch before committing; never commit to `main`.
- Read the current code before the phase file's pointers. If the phase file and the code disagree, or a decision above can't be met, stop and report instead of choosing silently.
- **A published package's API changes only by a recorded decision.** Phase 2 changes `@ascribed/astro`'s options. Add the decision to `project-docs/decisions.md` in the same pull request.
- **One home per fact.** The site's address and the docs' path are written in `docs/ascribe.toml`. Every copy is generated from it or compared with it by a test.
- **Colors are written in `design/tokens.toml` only.** A stylesheet change is an output change; `design/README.md` has the steps.
- **Every claim on a marketing page is checked or marked.** A link is checked, an example is read from a file that's built or tested, and a capability no release has carries its mark.
- **The site is a user's site.** It installs Ascribe from npm and uses nothing a user couldn't. What it needs from the integration is built as a feature, with its page in `docs/content/guides/astro.md`.
- **User-visible changes** update the page under `docs/content/` and the unreleased section of `CHANGELOG.md` in the same phase.
- No phase history in code or docs. Describe what the code does now.
- Before finishing a phase, all of these pass:

  ```sh
  cargo fmt --all --check
  cargo clippy --workspace --all-targets --locked -- -D warnings
  cargo test --workspace --locked
  pnpm format:check && pnpm lint && pnpm typecheck && pnpm test
  ```

  And, in `site/`, after `npm run build:checkout`: `npm run typecheck && npm test`.

## Later, not in this plan

- A blog.
- A showcase of sites built with Ascribe.
- A page per feature. The guides are those pages.
- Layout constructs in the language.
- `robots.txt`, a sitemap, and structured data: [search and answers](../proposals/search-and-answers.md).
- Links from one project into another, checked by Ascribe itself as it checks a project's own. Until then the site's tests check them, with `ascribe permalinks verify` (phase 2).
- Navigation in the content model, which the site's `src/nav.ts` stands in for.
