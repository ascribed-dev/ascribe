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

`site/netlify.toml`'s build (`npm ci`, `npm run follow-next`, `npm run build`), with `@ascribed/*@0.1.2-next.2`, 14 pages, and Node 24.21. Locally, it ran in a copy of `docs/` and `site/` alone, with no `git` history. On Netlify, it was the first production deploy, after the merge of #101 (Ubuntu 24.04 image `noble-new-builds`, npm 11.19, with Netlify's dependency cache from an earlier build).

| Step | Locally (Linux x64, cloud session) | On Netlify |
|---|---|---|
| Netlify's own `npm install`, before the build command | | 1 s (from its cache: "up to date in 507ms") |
| Installing: `npm ci` | 4.8 s | 4 s |
| Installing: `follow-next` (the newest canary) | 2.4 s | 2 s |
| `ascribe build` | 0.3 s | under 1 s |
| Astro (including `ascribe build`) | 4.8 s | 2 s ("15 page(s) built in 1.92s") |
| Pagefind | 0.4 s | 0.08 s |
| **The build command** | **about 12 s** | **8.3 s** |

On Netlify, the build command is 8.3 s of a 10.8 s Netlify Build, and the whole request takes 22.4 s: 8 s initializing (2.6 s of it fetching the cache), the build, and 2 s caching and cleaning up. The site was live 21 s after the build started.

The build passed with no `git` history, as decision 7 requires. On Netlify, `npm install` runs before the build command, so the site's packages install twice; with Netlify's cache, the first install takes a second. npm 11.19 warns that `esbuild`'s install script isn't covered by `allowScripts`. The build doesn't need it, since `esbuild` takes its binary from a platform package.

How often production's build fails between a merge and the next canary isn't known yet. If it's more than occasionally, it's a signal about the canary's schedule.

## Issues filed from dogfooding

| Phase | Issues |
|---|---|
| 1 | #82, #83, #84, #85 |
| 4 | #91, #92, #93, #94, #95, #96 |
| 5 | None |
