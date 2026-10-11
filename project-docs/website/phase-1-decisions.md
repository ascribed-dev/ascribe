# Phase 1: Decisions

Part of [The website](README.md). Needs nothing. No code: a pull request that records what the maintainer decides.

## Goal

Five choices are made before anything ships. The first two go into things that can't change quietly afterwards: a published package's options, and a folder that workflows, Netlify, and tests all name. The other three decide what the first page says, and the proposal left them open.

This phase can't be folded into phase 2. A thread may narrow what it handles. It may not ship a shape the plan expects to change.

## Context

- The [proposal](../proposals/website.md): "What moving the docs takes", and its open questions, which decisions 3 to 5 are.
- `packages/astro/src/index.ts`: `AscribeOptions`, and the single site output root, virtual module, permalinks route, dev file, review server, and toolbar app.
- `packages/astro/src/content.ts`: `ascribeCollection`, which reads the one root.
- The [versions proposal](../proposals/versions.md), which publishes a version under its number and needs the same from the integration.
- `site/README.md`: how production, previews, and the canary build the site.

## The decisions

Each has a recommendation. The maintainer may take it, change it, or ask for more.

### 1. How the marketing pages and the docs share a site

| Option | Means | Cost |
|---|---|---|
| **Two projects, one integration that takes a list** (recommended) | `ascribe({ projects: [{ project: "home", build: "site" }, { project: "../docs", build: "site" }] })`, and `ascribeCollection({ schema, project: "home" })`. The form with one project stays as it is | The integration's six single things each learn about several projects. Links between the two aren't checked by Ascribe |
| Two projects, the integration used twice | `integrations: [ascribe({…}), ascribe({…})]` | Two instances share a name, a virtual module, a toolbar app, and a route for permalinks. Each clash is settled by a rule a user has to learn |
| One project, with the docs in a subfolder | The landing page is `docs/content/index.md`, and every docs page moves under `docs/content/docs/` | No change to the integration. Links between the pages are checked for free. Marketing pages and docs share a content model, `llms.txt`, and builds, and a user with a home page in front of their docs still can't use Ascribe for the docs alone |
| Plain Astro pages for marketing | One project. The integration only learns to accept a base path under `base` | The least work. Gives up decision 5's experiment before trying it |

The recommendation rests on three things. A home page in front of docs is what most users' sites look like, so the integration should accept it. One call that owns several projects has one place to settle what they share. And the third option stays available as the fallback if the thread fails, while the reverse move would be a rewrite.

Whichever is taken, accepting a base path under Astro's `base` is built: every option but the third needs it, and so does the versions proposal.

**Decided: the list.** Two reasons beyond the ones above. The [versions proposal](../proposals/versions.md) needs a collection for each release line, each under its own base path, and a list of `{ project, build }` entries is the shape that later takes `{ project: "../docs", build: "3.4" }`; neither using the integration twice nor a subfolder gives it anything. And README decision 7 makes a home page in front of the docs a feature of the integration, while the plan's rules make the site a user's site: with the subfolder option the feature would be built and tested in `examples/astro-site` and used by no real site, and the seam the thread exists to test would go untested.

The subfolder option stays the fallback, and it's cheap now, since no user holds the docs' addresses. It's taken if review or the dev watcher can't follow several projects within phase 2. Phase 2 already lets review cover the first project listed; the watcher gets no such exemption, since a landing page that doesn't rebuild in `astro dev` isn't a working thread.

The cost recorded with it: two projects are two content models. The marketing project declares its own `[versions] current`, release dimension, and `next` feature for its marks, and its own `version` phrase. Each is a second home for a fact in `docs/ascribe.toml`, so `scripts/docs-site/facts.test.ts` holds the two files equal.

### 2. Where the marketing project lives

| Option | Means |
|---|---|
| **`site/home/`** (recommended) | Inside the Astro site, as a user's own project usually is. Workflows, Netlify's `ignore`, and the site's tests already cover `site/` |
| `website/` at the repository's root | Beside `docs/`, as its equal. One more folder at the root, and one more path in each workflow and in Netlify's settings |

**Decided: `site/home/`.** It's where a user's project usually sits, inside the Astro root, so the `project` option is a plain `"home"`. Netlify's `ignore`, the workflows' `site/**` patterns, and `site/.gitignore`'s `.ascribe/` line already cover it. A root `website/` adds a path to each of those for nothing.

### 3. What the landing page asks a visitor to do first

| Option | Reaches |
|---|---|
| **Read getting started** (recommended) | Everyone. The page shows the two install commands beside it, as the proposal has it, so someone ready to install isn't sent away |
| Install the editor extension | Writers, with the least setup. It shows nothing until there's a project |
| Run `npm install` | Developers who own a site already |

**Decided: read getting started.** The page's first two sections are the install command and the extension, so the link lands on the same two commands the landing page shows beside it. The extension's walkthrough is marked `next`, so leading with it would lead with something no release has. Leading with `npm install` reaches only someone who has already decided. The README's "Get started" block shows the same two commands, and nothing ties the copies together today; phase 3, which writes them onto the page, adds the test that does.

### 4. Whether the marketing pages follow `main` or the release

| Option | Means |
|---|---|
| **`main`, with marks** (recommended) | One build of one site, as the docs are. Decision 4 in the README already requires a mark on anything no release has |
| The release | The page is never ahead of what someone can install. It needs a second build, pinned, deployed together with a site that follows `main` |

**Decided: `main`, with marks.** Production is Netlify's nightly build from the `next` tag, and a page pinned to the release would need a second build and a second deploy to stay in step. The marks have their test already: when `[versions] current` moves at a release, `availability-left-behind` reports a `next` mark that's still there. That's the test phase 3 asks for, provided the marketing project declares the same release tables as the docs (decision 1).

### 5. Whether "Why Ascribe" names other tools

| Option | Means |
|---|---|
| **Describe the approaches, and link to the comparison** (recommended) | The page says what kinds of tool do what. `examples/comparison/` holds the same content written for Astro, Docker's docs, and Elastic's, and the page links to it. Nothing on the site goes out of date when another tool ships |
| Name them on the page | More useful to a reader choosing. Each sentence about another tool has to be kept fair and current |

**Decided: describe the approaches, and link to the comparison, on a condition.** The decisive reason is the plan's own rule: every claim on a marketing page is checked or marked, and a claim about another tool can be neither. The condition is that the comparison isn't fit to link today. Its README calls Ascribe "Tessera", cites `BRAINSTORM.md` sections, and says `tessera.md` carries gap markers it no longer has; the folder has no `ascribe.toml`, so nothing checks that its Ascribe page is valid, and nothing outside this plan references it. Phase 5 rewrites the Ascribe version in current Ascribe under an `ascribe.toml`, kept valid by a test as `examples/monorepo` is, and brings the README current. Until then the page links nowhere.

## Also confirmed here

Small choices that ship in phase 2:

- The docs are at `/docs/`, and permalinks stay at `/go/`, from the site's root.
- The marketing pages' addresses are `/`, `/why/`, and `/changelog/`.
- The marketing project publishes nothing for agents (`agents = false`). The docs' `llms.txt` moves to `/docs/llms.txt` with them, and the site's root has none until [search and answers](../proposals/search-and-answers.md) decides what belongs there.
- Search covers the docs only.

## Tasks

1. The maintainer decides each. The decisions are recorded under each option above.
2. Record them in this plan's README, as decisions 11 on, with the reason for each.
3. If decision 1 isn't the recommendation, rewrite [phase 2](phase-2-thread.md)'s design to match before it starts.

## Out of scope

Any code. Naming the widgets, which phase 2 and phase 3 do as they're needed.

## Acceptance criteria

- The README's decisions list holds all five, and the confirmed choices.
- Phase 2's design agrees with decision 1.

## Commits

1. "Record the website plan's decisions"
