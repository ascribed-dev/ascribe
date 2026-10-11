# Phase 2: The thread

Part of [Search and answers](README.md). Requires phase 1. Rust, `@ascribed/astro`, and the docs site. This is the plan's steel thread: the thinnest path through every layer, shipped and kept.

The names below are phase 1's recommendations. If it decided otherwise, this file was brought in line there.

## Goal

A page marked as not listed is treated that way everywhere, and Ascribe's own site has what a site at a host's root should:

```sh
$ grep -c unlisted docs/content/contracts/json-reports.md     # one line of frontmatter
1
$ curl -s https://ascribed-dev.com/docs/contracts/json-reports/ | grep -o '<meta name="robots"[^>]*>'
<meta name="robots" content="noindex">
$ curl -s https://ascribed-dev.com/sitemap.xml | grep -c contracts/json-reports
0
$ curl -s https://ascribed-dev.com/docs/llms.txt | grep -c contracts/json-reports
0
$ curl -s https://ascribed-dev.com/robots.txt | tail -1
Sitemap: https://ascribed-dev.com/sitemap.xml
```

Which page of the docs is marked, if any, is the maintainer's to say. The site's tests use a fixture, not a real page, if none should be.

One page, one value, one check. Everything it takes is built to the standard of the rest, and nothing in it is thrown away.

## Context

- [The thread](README.md#the-thread): the layers, and the seam this phase exists to test.
- Phase 1's decisions, in the [README](README.md#decisions): the key and its values, `[crawlers]`, `_ascribe/pages.json`, one sitemap and `robots.txt` for a site, and `Head.astro`.
- `SPEC.md` §2.1, and `crates/ascribe-model/src/sections.rs`: the reserved frontmatter keys, how a value of the wrong shape is reported, and why a fragment can't use them.
- `crates/ascribe-resolve`: `llms`, which builds `llms.txt` from a build's pages, and how a build's pages get their routes and titles.
- `crates/ascribe-emit/src/site/mod.rs` and `permalinks.rs`: `PERMALINKS_PATH`, how a generated file is listed in the manifest, and the list's shape. `crates/ascribe-emit/src/plain/mod.rs`: `llms.txt`, and the pointer that opens each Markdown page.
- `docs/content/contracts/output-layout.md` and `site-render.md`: what the site output promises.
- `packages/astro/src/index.ts`: `injectRoute` for permalinks, and how the integration holds several projects. `packages/astro/src/permalinks.ts`: reading a build's list, and refusing a version it doesn't know. `packages/astro/src/files.ts`: what it copies to a project's base path.
- `site/src/layouts/Base.astro`: the head the site writes by hand. `site/test/`: the built site's tests.
- `examples/astro-site/test/e2e/`: the end-to-end suite.
- `project-docs/checklists.md`: the lists for a change to the language, to the content model, and to an output. This phase is all three.
- [#273](https://github.com/ascribed-dev/ascribe/issues/273) and [#274](https://github.com/ascribed-dev/ascribe/issues/274): if either has landed, build on it; if not, don't do its work here.

## Design

### The language and the model

- **A reserved frontmatter key, `listing`,** for pages: `listed`, `unquoted`, or `unlisted`. Absent means `listed`. Any other value is the wrong type for a reserved key, reported as the others are. A fragment can't use it, and a type can't declare it.
- **`unquoted` is accepted and does nothing yet** but appear in the record: phase 3 gives it its outputs. The reference says so while that's true. The alternative, refusing a value the language will have next phase, would ship a contract that changes.
- **`[crawlers]`,** with one key: `lines`, a list of strings written into `robots.txt` as they are. A line is one line: no line break, and not empty.

### The record

For each page a build publishes: its route, its source path, its title, its description, its canonical address, its listing, and its Markdown version's address when the build has one. Built once, in the resolver, beside where `llms.txt` is built, and read by everything below.

### What a build writes

- **`_ascribe/pages.json`,** in the site output and in the manifest:

  ```json
  {
    "version": 1,
    "site": "https://docs.example.com",
    "pages": {
      "/guides/install/": {
        "route": "/guides/install/",
        "source": "guides/install.md",
        "title": "Install the agent",
        "description": "Install and connect the sync agent.",
        "canonical": "https://docs.example.com/guides/install/",
        "listing": "listed",
        "markdown": "/guides/install.md"
      }
    },
    "robots": ["Disallow: /drafts/"]
  }
  ```

  Written when the build has no pages too. `site` and `canonical` are absent when `[consumer] site` isn't set, and the reference says what a site loses without it: canonical links, and a sitemap, which needs absolute addresses.
- **`llms.txt`** lists only pages that are offered: not one that's `unlisted`. Its sections and its splitting follow from what's left.
- **A Markdown page that isn't listed** still opens with its pointer to the index. The index doesn't point back.

### In an Astro site

- **`@ascribed/astro/Head.astro`,** given a page's entry: `<title>`, the description, the canonical link, and `<meta name="robots">` when the record asks for one (`noindex` for `unlisted`). A site says how its titles read with one property. A page with no record, such as a site's own Astro page, gets nothing from it, and the guide says how such a page writes its own.
- **One sitemap,** at the site's root, `sitemap.xml`: every listed page of every project that has routes, by absolute address, and nothing else. No dates. A site with no `site` gets no sitemap and a build warning that says why.
- **One `robots.txt`,** at the site's root: each project's `lines`, once each, then the sitemap's address. A site that has its own `robots.txt` in `public/` keeps it: the integration says so and writes none, and the guide says how to take Ascribe's lines into a file the site owns.
- **Both are served in `astro dev`** and written by `astro build`, at the host's root whatever a project's base path is.

**This is the seam.** If a head can't be supplied as one component a layout includes, or the two files can't be written at the root for several projects in both commands, stop and report before going on. [The thread](README.md#the-thread) says what each failure would change.

### The check

One, advice, configurable: a page that isn't listed, at an address the project's own `robots.txt` lines disallow for every crawler. The crawler is told not to fetch the page, so it never reads the rule that says not to list it, and the page can stay in a search index by its address alone. The message says which line, and the fix is to remove one of the two.

It reads the lines as `robots.txt` is read: the group for `*`, the longest match. A line it can't read is left alone, not guessed at.

### Ascribe's site

- `site/src/layouts/Base.astro` includes `Head.astro` for a page that has an entry, and keeps what's the site's own: the icons, the theme color, the Open Graph tags and the cards, until phase 6.
- The site's three projects make one sitemap: the docs' pages and the site's own. The `demo` project's page isn't in it.
- The changelog is an Astro page with no record. It writes its own title and canonical link, as now.
- `site/test/` gains: every page in the sitemap is a built page that isn't `noindex`, and every built page that is `noindex` is absent from it and from `llms.txt`.

## What this phase doesn't handle, said where a user will look

The content model reference says, while each is true: the setting can be made only on a page; `unquoted` isn't acted on; a project says nothing about crawlers by purpose; and the Markdown version of a page that isn't listed can still be indexed, since only a header prevents that.

## Tasks

1. The key, in the model and `SPEC.md`, with conformance cases: each value, a wrong one, and a fragment.
2. `[crawlers]` and `lines`, in the model, its reference, and its contract.
3. The record in the resolver, and `llms.txt` built from it.
4. `_ascribe/pages.json` from the site emitter, in the manifest and the output-layout contract, with its schema.
5. The check, in the registry, with conformance cases: disallowed and unlisted; one without the other; a line for a named crawler only, which isn't reported.
6. `Head.astro`, the sitemap, and `robots.txt` in `@ascribed/astro`, with unit tests, and end-to-end tests in `examples/astro-site`: a built site and `astro dev`; a project under a base path; two projects; a site with its own `robots.txt`; a site with no `site`.
7. The docs site: the layout, the tests, and a look at the pull request's preview.
8. `docs/content/reference/content-model.md`, `guides/astro.md`, the diagnostics reference, `CHANGELOG.md`, the map of the surface, and the decision in `project-docs/decisions.md`.

## Out of scope

The setting anywhere but a page; what `unquoted` writes; crawlers by purpose; headers; the rest of the head; structured data; dates in the sitemap; the other checks. Each is a later phase, or carved out.

## Acceptance criteria

- In `examples/astro-site`, built and served, a page with `listing: unlisted` has `noindex` in its head, isn't in `sitemap.xml`, and isn't in `llms.txt`; a page beside it with nothing set is in both. The end-to-end test does exactly this, in a build and under `astro dev`.
- A test reads `_ascribe/pages.json` and checks the head, the sitemap, and `llms.txt` of a built site against it, page by page.
- A project that sets nothing builds the same site output as before, plus `_ascribe/pages.json`. `node scripts/compare/outputs.ts --base main` shows that file and nothing else.
- `https://ascribed-dev.com/robots.txt` and `/sitemap.xml` answer on the pull request's preview, and the sitemap holds no address that answers with `noindex` or a redirect.
- The night after this merges, the same is true of the live site. The pull request says when it was checked.

## Verify

```sh
cargo test --workspace --locked
pnpm --filter @ascribed/astro test
pnpm --filter @ascribed/example-astro-site test:e2e
node scripts/compare/outputs.ts --base main
cd site && npm ci && npm run build:checkout && npm run typecheck && npm test
```

## Commits

1. "Let a page say it isn't listed"
2. "Write each build's page records beside its site output"
3. "Leave a page that isn't listed out of llms.txt"
4. "Report a rule no crawler can read"
5. "Give an Astro site its head, sitemap, and robots.txt"
