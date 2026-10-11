# Phase 5: Reaching the host

Part of [Search and answers](README.md). Requires phase 3. The site emitter, `@ascribed/astro`, and the docs site's host.

## Goal

A page that isn't listed has a Markdown version that isn't either. A Markdown file has no head to carry a rule, so the rule has to be a response header, and a header is the host's:

```sh
$ curl -sI https://docs.example.com/internal/runbook.md | grep -i x-robots-tag
x-robots-tag: noindex
```

On a host that reads `_headers`, a build makes that so. On any other, the same rules are a list a few lines of glue can read.

## Context

- README decision 9, and the proposal's "Reaching the host": three layers, and why Ascribe names one format.
- The record, with each page's listing and its Markdown version's address.
- `packages/astro/src/files.ts`: how the integration copies the plain output to a project's base path, and serves it in `astro dev`.
- `site/netlify.toml`, `site/public/_redirects`, and `site/netlify/edge-functions/markdown.ts`: what Ascribe's own host is already told, and by which file. The edge function answers a page's own address with its Markdown when a request asks for `text/markdown`, by fetching the `.md` and returning its body.
- Netlify's documentation for `_headers`: its syntax, where the file must be, and what it does with a site that has one in `public/` already. Read it; don't work from memory.
- Whether search engines index the Markdown versions at all today: search for a page of Ascribe's docs by its `.md` address before deciding how much this phase is worth saying about.

## Design

### The neutral list

`_ascribe/headers.json`, in the site output: each rule as a path and the headers it gets. This phase writes one kind: `X-Robots-Tag` for the Markdown version of each page that's `unlisted` or `unquoted`, with the value its head would have. Shaped so redirects, when they're designed, are a second list beside it and not a change to this one.

A project that publishes no Markdown versions, or lists everything, gets the file with an empty list.

### One format

With `[consumer] headers = "netlify"` (the key and its value decided here, by a recorded decision), `@ascribed/astro` writes the rules into the site's `_headers`:

- When the site has no `_headers`, the integration writes one.
- When it has one in `public/`, the integration doesn't touch it, and fails the build with the lines to add, unless they're there. A file the site owns isn't rewritten by Ascribe: the same rule as `robots.txt` in phase 2.
- Rules are by exact path, not by pattern, so one for a page can't catch its neighbor.

Without the setting, nothing is written for a host, and the guide says how to read the neutral list from a deploy script.

### A page asked for as Markdown

There's a second way to get a page's Markdown: its own address, with `Accept: text/markdown`, where a host negotiates. That response has no head, and a rule in `_headers` is on the `.md` path, not the page's. So an agent that fetches a page that isn't listed this way gets it with no rule at all.

Whatever answers such a request passes the Markdown version's headers on. For Ascribe's site that's the edge function: it copies `x-robots-tag`, and the canonical `Link` if this phase adds one, from the `.md` response to its own. The guide's section on content negotiation says the same of any host's rewrite or function, since the delivery spec asks sites to negotiate and this is the part that's easy to miss.

### In development

`astro dev` answers a Markdown version with the header it would have, so the end-to-end test needs no host.

### `noindex` for every Markdown version

A separate question this phase answers by a decision, not by default: whether the Markdown versions of listed pages should be indexed. They're duplicates of pages a search engine already has, and hosted platforms' changelogs record "raw Markdown files indexed by Google" as a bug they fixed. The recommendation is a canonical header (`Link: <page>; rel="canonical"`) on every Markdown version, which says which one to list without hiding the other from an agent's search. It's the same list and the same file.

### What stays the host's

Content negotiation, status codes, bot protection, logs. The guide's section on the host says what each is and that Ascribe doesn't do it.

## Tasks

1. The list, in the site emitter, the manifest, and the output-layout contract, with its schema.
2. The decision on the setting and on canonical headers, recorded.
3. `_headers` in `@ascribed/astro`: writing one, and the build's refusal with the lines to add. Unit tests for both.
4. The header in `astro dev`, and an end-to-end test.
5. The edge function passing the headers on, with `site/test/negotiation.test.ts` covering a page that isn't listed.
6. Ascribe's site: the setting, and a test of the built `_headers` against the record.
7. The content model reference; `guides/astro.md`'s section on the host, with the glue for a host that isn't Netlify; `CHANGELOG.md`; the map of the surface.

## Out of scope

Redirects. Any second host format. Headers for anything but how a page is listed. Rewriting a site's own `_headers`.

## Acceptance criteria

- On the pull request's preview, the Markdown version of a page that isn't listed answers with `x-robots-tag: noindex`, and one of a listed page doesn't.
- The same page's own address, asked for with `Accept: text/markdown`, answers with the same header. Tried with `curl` on the preview, and said in the pull request.
- A test reads `_ascribe/headers.json` against the record: every page that isn't `listed` and has a Markdown version has a rule, and no other page does.
- A site with its own `_headers` that lacks the lines fails its build, and the message is the lines.
- The reference no longer says a Markdown version of an unlisted page can be indexed, for a site with the setting, and says it still can without.

## Verify

```sh
cargo test --workspace --locked
pnpm --filter @ascribed/astro test
pnpm --filter @ascribed/example-astro-site test:e2e
cd site && npm run build:checkout && npm test
```

## Commits

1. "Write the headers a build's pages need, as a list"
2. "Give a site on Netlify its headers"
