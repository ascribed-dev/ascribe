# Phase 1: Decisions

Part of [The website](README.md). Needs nothing. No code: a pull request that records what the maintainer decides.

## Goal

Five choices were made before anything shipped, and are recorded in the [README](README.md#decisions) as decisions 11 to 20. This file keeps the options each was chosen from.

The first two go into things that can't change quietly afterwards: a published package's options, and a folder that workflows, Netlify, and tests all name. The other three decide what the first page says, and the proposal left them open.

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

**Decided: the list, with a name on each entry.** README decisions 11, 12, and 13 have the reasons, the fallback, and the cost. The option's table above shows the shape as first proposed, keyed by `project`; a review of the plan found that two entries for one project with two builds, which the versions proposal needs, would share a path, so each entry has a `name` and the collection is keyed by it.

### 2. Where the marketing project lives

| Option | Means |
|---|---|
| **`site/home/`** (recommended) | Inside the Astro site, as a user's own project usually is. Workflows, Netlify's `ignore`, and the site's tests already cover `site/` |
| `website/` at the repository's root | Beside `docs/`, as its equal. One more folder at the root, and one more path in each workflow and in Netlify's settings |

**Decided: `site/home/`.** README decision 14.

### 3. What the landing page asks a visitor to do first

| Option | Reaches |
|---|---|
| **Read getting started** (recommended) | Everyone. The page shows the two install commands beside it, as the proposal has it, so someone ready to install isn't sent away |
| Install the editor extension | Writers, with the least setup. It shows nothing until there's a project |
| Run `npm install` | Developers who own a site already |

**Decided: read getting started.** README decision 15.

### 4. Whether the marketing pages follow `main` or the release

| Option | Means |
|---|---|
| **`main`, with marks** (recommended) | One build of one site, as the docs are. Decision 4 in the README already requires a mark on anything no release has |
| The release | The page is never ahead of what someone can install. It needs a second build, pinned, deployed together with a site that follows `main` |

**Decided: `main`, with marks.** README decision 16.

### 5. Whether "Why Ascribe" names other tools

| Option | Means |
|---|---|
| **Describe the approaches, and link to the comparison** (recommended) | The page says what kinds of tool do what. `examples/comparison/` holds the same content written for Astro, Docker's docs, and Elastic's, and the page links to it. Nothing on the site goes out of date when another tool ships |
| Name them on the page | More useful to a reader choosing. Each sentence about another tool has to be kept fair and current |

**Decided: describe the approaches, and link to the comparison, on a condition.** README decision 17 has the condition, and [phase 5](phase-5-why.md) the work it makes.

## Also confirmed here

Small choices that ship in phase 2, recorded as README decisions 18 to 20: the addresses, that the marketing project publishes nothing for agents and where `llms.txt` goes, and that search covers the docs only.

## Tasks

1. The maintainer decides each. Done.
2. Record them in this plan's README, as decisions 11 on, with the reason for each. Done: 11 to 20.
3. If decision 1 isn't the recommendation, rewrite [phase 2](phase-2-thread.md)'s design to match before it starts. It is the recommendation, with a name added to each entry, and phase 2's design has it.

## Out of scope

Any code. Naming the widgets, which phase 2 and phase 3 do as they're needed.

## Acceptance criteria

- The README's decisions list holds all five, and the confirmed choices.
- Phase 2's design agrees with decision 1.

## Commits

1. "Record the website plan's decisions"
