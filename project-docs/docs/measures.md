# Measures

The numbers that check "customer #1" ([README](README.md#the-measures)), recorded as each phase takes them.

## Time to a working site

### Phase 4: following the guides, 2026-10-05

From an empty folder to a built site of `docs/`, following [getting-started](../../docs/content/getting-started.md) and the [Astro guide](../../docs/content/guides/astro.md), with `@ascribed/*@0.1.2-next.2` from npm, Astro 7.3.5, Node 24.21 and npm 11.19, on Linux x64.

| Step | Time (UTC) |
|---|---|
| Start: empty folder | 14:55:11 |
| `npm create astro@latest` failed (see below); `npm init` and `npm install astro@7` instead | 14:55:19 to 14:55:39 |
| `npm install --save-dev @ascribed/cli@next`, `npx ascribe --version` | 14:55:41 |
| `npm install @ascribed/astro@next sharp` | 14:55:51 |
| `[consumer]` in `ascribe.toml`; `astro.config.mjs`, `content.config.ts`, the route, and the layout, from the guide | 14:56:05 |
| `astro build`: 13 pages | 14:56:10 |

**About one minute**, by Claude in a cloud session, having read the guides first and typing nothing by hand. That's the floor, not a person's time: phase 5 measures a person, with nothing from this repository on the machine. The site's own layout, navigation, and search came after, and aren't counted.

Stumbles:

- **The guides didn't say how to start an Astro project.** Fixed: getting-started names `npm create astro@latest`. (It failed here for a reason of the environment: `create-astro` fetches its templates from GitHub, which the session couldn't reach.)
- **`ascribe.toml` outside the Astro project.** The Astro guide put it beside `astro.config.mjs` and didn't say what to change when it isn't: the `project` option, and the schema's path in `content.config.ts`. Fixed in the guide.
- **Type-checking the collection** fails when the project is outside the Astro root ([#93](https://github.com/ascribed-dev/ascribe/issues/93)).

### Phase 5: a person, with nothing from this repository

Not measured yet. It needs one person following [getting-started](../../docs/content/getting-started.md) and the [Astro guide](../../docs/content/guides/astro.md) on their own machine, from an empty folder to a deployed site, timed.

## The host's build time

### Phase 5: production, 2026-10-05

`site/netlify.toml`'s build (`npm ci`, `npm run follow-next`, `npm run build`), with `@ascribed/*@0.1.2-next.2`, 14 pages, Node 24.21, in a copy of `docs/` and `site/` alone, with no `git` history.

| Step | Locally (Linux x64, cloud session) | On Netlify |
|---|---|---|
| Installing: `npm ci` | 4.8 s | To record from the first production deploy's log |
| Installing: `follow-next` (the newest canary) | 2.4 s | |
| `ascribe build` | 0.3 s | |
| Astro (including `ascribe build`) | 4.8 s | |
| Pagefind | 0.4 s | |
| **Total** | **about 12 s** | |

The build passed with no `git` history, as decision 7 requires. On Netlify, `npm install` runs before the build command, so the site's packages install twice; it's in the installing time.

How often production's build fails between a merge and the next canary isn't known yet. If it's more than occasionally, it's a signal about the canary's schedule.

## Issues filed from dogfooding

| Phase | Issues |
|---|---|
| 1 | #82, #83, #84, #85 |
| 4 | #91, #92, #93, #94, #95, #96 |
| 5 | None |
