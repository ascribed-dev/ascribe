# Proposal: a landing page and a small website

A proposal, not a plan. It says what would be built and what it would take, so the idea can be judged before anything is designed in detail.

## The goal

Someone who lands on <https://ascribed-dev.com> learns what Ascribe is and why it exists before they're handed a reference manual.

Today the site's root is the docs index: a list of links for a person who has already decided to use Ascribe. The pitch is only in the repository's README.

## Decided

- **One site, with a new root.** The landing page takes `/`, and the docs move to `/docs/`.
- **Three pages to start:** the landing page, "Why Ascribe", and the changelog.
- **The pitch is docs treated like code:** written in files, checked, reviewed, and built, with the same rigor as the product they describe.
- **It says what's being built.** Ascribe is early. The site describes where it's going and is plain about what works today.
- **The pages are written in Ascribe where that's practical,** to find out how far the language stretches beyond documentation.
- **The design work is part of this.** The visual design plan made the mark, the tokens, and a social card, and left marketing pages for later.

## The pages

### The landing page

In order:

1. **What it is, in a sentence,** with the status beside it: early, and open source.
2. **The demonstration.** A page's source on one side and what a reader sees on the other. The source is a real file in the repository, and the result is Ascribe's own build of it, so the demonstration can't be faked and can't fall behind.
3. **What "like code" means,** as four short claims, each with the smallest example that shows it:
   - **Checked.** A broken link or an undeclared name is an error, in the editor and in CI.
   - **One source, several outputs.** Variants and availability build a site, plain Markdown, and JSON from the same pages.
   - **Examples from real code.** A snippet is read from the code's own file, and a report says which pages to reread when it changes.
   - **Reviewed as pages.** A pull request is read as readers will see it, with comments beside what they're about.
4. **How to start:** the two install commands, and a link to getting started.
5. **Where it's going:** a short list of what's next, from the plans in this repository.

### Why Ascribe

The argument at length, for someone comparing tools: the problems docs-as-code teams report, what Ascribe does about each, and what it deliberately doesn't do. The research reports in `project-docs/reports/` are the source for the problems, and `examples/comparison/` already holds the same content written for Astro, Docker's docs, and Elastic's, beside Ascribe's.

It names its limits: no hosting, no model, no navigation in the content model yet, one site generator supported.

### Changelog

`CHANGELOG.md`, as a page of the site. The file stays where it is, since the release process and the packages' READMEs point at it.

## Saying what's built and what isn't

The site describes a product in progress, and Ascribe already has a way to say that about any piece of content: availability. The docs mark what no release has yet, and the landing page's claims can carry the same marks, drawn by the same element.

So a capability that's in `main` and not in a release says so on the landing page, by the mechanism the docs use, and stops saying so when the release ships. Nobody keeps a second list.

## Written in Ascribe

A landing page is mostly layout, and Ascribe has no layout constructs, on purpose. What it does have is project widgets: directives a project declares, each with a schema, which the site output writes as custom elements for the site to draw.

That's the way to try this without changing the language:

```markdown
@hero-banner {status=early}:
# Documentation, treated like code

Written in files. Checked, reviewed, and built like the product it describes.
@end

@source-and-result: demo/install.md
```

- **The content is Ascribe pages,** in a project of their own beside the docs' project, with a page type for marketing pages and a handful of widgets: a hero, a claim with its example, a source-and-result pair, a call to action.
- **The site draws the widgets.** Each is a small Astro component or a custom element. Layout stays the site's job.
- **Everything is checked:** links to docs pages, the snippets the demonstrations read, phrases such as the current version.

What this would show, either way, is worth knowing. If four or five widgets carry a landing page well, that's a use for Ascribe beyond docs, and a good example for the docs' own page on widgets. If the page fights the language, the fallback is plain Astro pages that pull their demonstrations from Ascribe, and the finding goes in the brainstorm.

The changelog is the exception. It isn't an Ascribe page and shouldn't become one: the site renders the Markdown file, and rewrites its links to `docs/content/` files into the site's routes.

## What it depends on

**Permalinks, first.** The binary builds full addresses into the docs today: each diagnostic links to its entry in the reference, and each command's `--help` ends with a link to its section. Moving the docs to `/docs/` would break all of them. The [permalinks proposal](permalinks.md) has the binary link to ids instead, which the site resolves. With that in place, the docs can move and nothing in the binary changes. So this work follows that one.

Ascribe is before 1.0 and has no users, so the addresses in released binaries aren't kept working. No redirects are written for them.

## What moving the docs takes

**Where docs live in a site is already a setting.** `[consumer] base-path` in `ascribe.toml` is the path every page is published under, and `/docs/` works today. What doesn't work is narrower: `@ascribed/astro` requires Astro's own `base` to equal it, and Astro's `base` covers the whole site.

| Site | Today |
|---|---|
| All docs, at `/` | Works |
| All docs, at `/docs/` | Works |
| A home page at `/`, with docs at `/docs/` | The integration refuses it |

The fix is small: accept a base path that's under Astro's `base`, not only one equal to it. It touches the check and the handler that serves the project's published files. It's worth doing as a feature, since anyone with a home page in front of their docs needs it, and the [versions proposal](versions.md) needs the same for a version published under its number.

**Two projects in one site.** The marketing pages and the docs are two Ascribe projects, with different page types and different base paths. The integration takes one project today. Either it's used twice, or it learns to take several.

**Links from the marketing pages into the docs** cross from one project to the other, which Ascribe doesn't check: a project's links stay inside it. They go through permalinks, so they survive the docs being reorganized, and `ascribe permalinks verify` checks them in the site's CI.

**The rest is changing one setting and what follows it:** `docs/ascribe.toml`'s base path, the files a test already holds to it (the Astro config, the READMEs, the packages' links), and the site's own tests, which assume the docs are at the root.

## Design

From the visual design work, this can reuse the mark, the wordmark, the tagline, the color and type tokens, and the social card. It needs:

- **Larger type.** The tokens' scale was made for documentation. A hero needs sizes above it, added to `design/tokens.toml` like any other.
- **A page frame without the docs' furniture:** no sidebar, no table of contents, a wider measure, and a header that links to Docs, Why Ascribe, Changelog, and GitHub.
- **The widgets' styles,** from the same tokens.
- **Very few images.** The demonstrations are real rendered output, which needs no screenshot. The editor and review can't be shown that way, so they need a small number of screenshots, each one a file that will go stale and has to be retaken. A test can at least fail when a screenshot is older than the release it claims to show.
- **A social card for each page,** generated as the existing one is.

Light and dark follow the system, as the docs do, and the contrast test covers the new pairings.

## What this leaves out

- A blog.
- A showcase of sites built with Ascribe. There aren't any yet.
- Pricing, sign-up, or anything that collects data. The site has no analytics, and this adds none.
- Layout constructs in the language. Widgets are the experiment; the language doesn't change.
- A page per feature. The guides are those pages.

## Open questions

- **What should the page ask a visitor to do first:** install the editor extension, run `npm install`, or read getting started? The three reach different people, and the page should lead with one.
- **Should the marketing pages follow `main` or the release?** The docs follow `main` through the nightly canary, so they can describe what no release has. A landing page that's a day ahead of the release is fine if it marks what's unreleased, and wrong if it doesn't.
- **Does "Why Ascribe" name other tools?** `examples/comparison/` does, with their own content. Naming them on the site is more useful to a reader and takes more care to keep fair and current.
