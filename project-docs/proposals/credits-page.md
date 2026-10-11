# Proposal: a credits page

A proposal, not a plan. It says what the page is and what it would take, so the idea can be judged before anything is built.

## The goal

A page of the site that says three things plainly: who builds Ascribe, what it's built on, and where its ideas came from.

It's owed. Ascribe ships a fork of someone else's parser, reads another project's snippet tags so code tagged for that project works unchanged, and will ship another project's templates. Each of those is said somewhere in the repository today, and none of it is said in one place a visitor would find.

## What exists

- **The site's own pages are Ascribe pages,** in `site/home/content/`, each routed at the site's root: the landing page and "Why Ascribe". The changelog is an Astro page beside them.
- **The header links to Docs, Why Ascribe, Changelog, and GitHub.** The site has no footer.
- **Each page of the site's own has a generated social card,** and two tests list the pages that have one (`site/test/e2e.test.ts`, `scripts/design/assets.test.ts`).
- **The site's rule for these pages:** every claim is checked or marked. A link is checked, and a fact that lives in another file is read from it or compared with it by a test.
- **Where the credit is already written,** a line at a time:
  - `crates/comrak-ascribe/FORK.md`: the fork of comrak, its release, and every change to it.
  - `SPEC.md`, Appendix C and D: what each design choice was taken from, and the references.
  - The drift guide and the directive reference: the snippet tags are Bluehawk's.
  - The changelog and the report guide: the Web Documentation Delivery Spec.
  - The manifests: every dependency, with no word about any of them.
  - The commits: who wrote each, and with what.

## The page

One page, `/credits/`, written as an Ascribe page like the others, in three parts.

### Who builds Ascribe

Two authors, said as it is: Kyle Rollins, and Claude, Anthropic's model, working through Claude Code. A sentence on how the work is divided: the maintainer decides what's built and reads what ships, and most of the code, the tests, and the docs are written by Claude under those decisions.

It names Claude and not one model. The commits carry several model names, and the page shouldn't need editing when the next one arrives.

This part is short, and it's the one a visitor is most likely to quote. It should be exact and unembarrassed: a project whose pitch is docs held to the standard of code can say how its own code is written.

### What Ascribe is built on

Grouped by what a reader would want to know, not by manifest:

- **Inside the binary.** comrak first, with a paragraph of its own: Ascribe's parser is a fork of it, at a named release, and the page links to the list of changes. Then the libraries that do visible work: clap, ariadne (which draws the diagnostics), serde, toml, and schemars.
- **In the npm packages.** Astro, which the integration is for, and the libraries its Markdown goes through.
- **Programs Ascribe runs and doesn't ship.** Vale, lychee, and the delivery spec's checker. These are the ones a user installs themselves, so the page says what each is for.
- **Building Ascribe and its site.** Pagefind, Playwright, Vitest, and the rest of what a contributor meets.

Each is a name, a link, and a few words on what Ascribe uses it for. No versions on the page: they'd be stale by the next week, and the manifests have them.

### Where its ideas came from

The ones the repository already states, each with a sentence on what was taken and a link to where Ascribe says so:

| From | What Ascribe took |
|---|---|
| CommonMark | The language everything else is added to |
| Bluehawk | The snippet tags, kept as they are, so code already tagged for it works |
| AsciiDoc | A title on its own line |
| Swift's `@available` | Lifecycle states named by the version they start at |
| HTML's custom elements | A hyphen marks a project's own widgets |
| Elastic's docs-builder | Empty-text links filled from their target, and a large corpus to measure choices against |
| Ruby and Lua | A bare `@end` |
| The Web Documentation Delivery Spec | What a build publishes for AI agents |
| The Good Docs Project | The page templates, once they're built |
| Steel threads, by Jade Rubick | How every plan's phases are ordered |

The spec's rationale also names what Ascribe decided against, with reasons: MDX, the colon-fenced directive syntax, GitHub's alerts. Those aren't credits, and they stay in the spec.

## Keeping it true

The page makes three kinds of claim, and each is held differently:

- **"Ascribe uses this."** A test compares the page with the manifests: every project the page names as a dependency is one, in `Cargo.toml` or a `package.json`, and a named dependency that's removed fails the test until the page follows. The reverse isn't required: the page names the ones worth naming, not all of them.
- **"Ascribe took this from there."** Each row links to the place in the spec or the docs that says so, by permalink where the page has one, and the site's link test follows it. A row with nowhere to link is a claim the repository doesn't make yet, and gets its sentence added there first.
- **"This is who builds it."** Nothing can test it. The maintainer reads the page before it merges, as with the site's other pages.

comrak's release is read from `FORK.md`, not typed again.

## Where it's linked

A site footer, which the site doesn't have yet: one line under every page with Credits and the license. A credits page is what a footer is for, and a header with a fifth link starts to crowd at phone width.

The landing page's line about status could link to it too: "early, and open source" is where a visitor wonders who's behind it.

## What it would take

Small. One Ascribe page, a footer in the base layout with its styles from the tokens, the page's social card, the two tests that list the site's own pages, and one new test that holds the dependency names to the manifests. If a row of the ideas table needs somewhere to link, a sentence in the spec's rationale or the docs.

No change to Ascribe itself, and no decision to record beyond the page's wording.

## What it leaves out

- **Every dependency.** The direct ones number about thirty and the transitive ones hundreds. A full list is what a license report is for, and belongs to the release, not the site.
- **License texts.** The page links to each project. Ascribe's own obligations (comrak's license travels with the fork) are met in the repository already.
- **Versions.**
- **Contributors as a list.** There are two. When there are more, the repository's own contributor list is the honest source, and the page can point at it.
- **Thanks to people.** Projects and ideas only, unless the maintainer wants to name someone.

## Open questions

1. **What shaped Ascribe that isn't written down anywhere?** The table has only what the repository states. Tools and habits from the maintainer's own work, a docs methodology such as Diátaxis, a talk or a book: none appear in the spec or the docs, and only the maintainer knows which belong.
2. **How should Claude's part be worded?** "Built by Kyle Rollins with Claude" and "built by Kyle Rollins and Claude" say different things about authorship, and the page has to pick one.
3. **Is it "Credits"?** "Acknowledgements" and "Colophon" are the other names such a page goes by. "Credits" is the plainest, and fits a page that includes who built it.
4. **Does the footer carry anything else?** The license and the repository are the usual ones. A footer added for one link tends to collect more, and it's cheaper to decide what it's for now.
