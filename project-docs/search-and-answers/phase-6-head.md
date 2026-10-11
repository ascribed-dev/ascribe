# Phase 6: The whole head

Part of [Search and answers](README.md). Requires phase 2. `@ascribed/astro` and the docs site, with a small addition to the record.

## Goal

A layout includes one component for everything a page's head says about the page, and writes none of it itself:

```astro
<head>
  <meta charset="utf-8" />
  <Head entry={entry} title={(page) => `${page} · Ascribe`} card={card} />
  <Elements />
</head>
```

Ascribe's own layout loses its fifteen hand-written lines, and the next site never writes them.

## Context

- `site/src/layouts/Base.astro`: what a real head has today. Sort every tag into what's about the page (Ascribe's), and what's about the site or the device (the site's).
- `Head.astro`, as phases 2 and 3 left it, and the property that says how titles read.
- The proposal: "What a build would write", and its table of what the head takes from the record.
- Google's documentation for breadcrumb and site-name structured data, and for the Open Graph tags a link preview reads. Read each for what's required; the research found most "rules" about these aren't documented anywhere.
- The surface audit's finding 28: there's no navigation in the content model, so a page's place in the site is known only by its folders.
- `site/test/e2e.test.ts`: what the built site's head is already held to.

## Design

### What the component writes

From the record, and nothing a site has to pass:

- the title and the description (phase 2), and the same two for link previews (`og:title`, `og:description`), with `og:url` as the canonical address and `og:type`;
- the canonical link and the robots rule (phases 2 and 3);
- a link to the Markdown version, `rel="alternate" type="text/markdown"`, when the build publishes one and the page is offered;
- breadcrumbs, as structured data.

From the site, by property, since Ascribe doesn't know them: the site's name (for `og:site_name` and the site-name structured data), how a title reads, and the page's social card with its text. A site that passes no card gets no image tags.

What stays in the layout, and the guide lists it: the character set, the viewport, icons, theme colors, styles, and the elements.

### Breadcrumbs

A page's place, from its route's folders: each folder that has an index page is a step, named by that page's title, down to the page itself. A folder with no index page isn't a step, since there's nothing to link to.

Said as what they are, in the guide: they change how a search result is displayed, and nothing else. And as what they aren't: a site's navigation. When the content model has navigation, they come from that, and the record's field for them doesn't change.

The steps are in the record, so a site can draw visible breadcrumbs from the same list.

### The site's name

Structured data for the site's name is written once, on the page at the host's root, as its documentation says. With several projects, that's whichever has the root. A site whose root isn't an Ascribe page writes its own, and the guide has the four lines.

### Nothing twice

A layout that includes the component and also writes a title, a canonical link, or a robots tag has two. `astro dev` warns, once, naming the tag. It can't be an error: a site may have a reason.

## Tasks

1. The breadcrumb steps in the record and its schema, with tests: a page at the root, in a folder with an index, in one without, and under a base path.
2. The component: each tag, with a unit test for each, and for a page with a card, without one, unlisted, and with no Markdown version.
3. The warning for a tag written twice.
4. Ascribe's site: `Base.astro` takes its head from the component, for docs pages and the site's own. Its end-to-end test of the head passes unchanged, which is the proof.
5. `guides/astro.md`: the head, what the site still writes, and a page with no record. `CHANGELOG.md`. The map of the surface.

## Out of scope

Structured data of any other kind. Social card images, which a site makes. Tags for one network that Open Graph doesn't cover. Visible breadcrumbs as a component.

## Acceptance criteria

- The docs site's built head, tag for tag, is what it was before this phase, apart from what's added (the Markdown link, the structured data). A test compares a saved copy.
- Google's Rich Results Test accepts a docs page's breadcrumbs, tried once on the preview, and said in the pull request.
- A test reads the record against every tag the component writes.
- `examples/astro-site`'s layout has no tag about the page left in it.

## Verify

```sh
cargo test --workspace --locked
pnpm --filter @ascribed/astro test
pnpm --filter @ascribed/example-astro-site test:e2e
cd site && npm run build:checkout && npm run typecheck && npm test
```

## Commits

1. "Say each page's place in the site in its record"
2. "Write a page's whole head from its record"
3. "Take the docs site's head from Ascribe"
