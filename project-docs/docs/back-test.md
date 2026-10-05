# The back-test

Part of [Docs](README.md). The write-up for [phase 6](phase-6-back-test.md).

## Decision

**Don't build file-level coverage. Phase 8 reports changed examples only.**

Over 66 merged pull requests, a file-level report would have made 42 reports, and 11 of them were right: **26%**, under the plan's threshold of about a third. Since `docs/` became the user docs (pull requests #39 to #81), it was right 4 times in 25 (16%). Reporting only changes to the regions a page is really about would have cut almost every "fine" report, but kept only 2 of the 7 "right" ones on the pages tried. That's why phase 8 keeps the snippet-based report, where a region is an example the page shows, and drops the file-level one. It also leaves out an explicit `covers` field: regions named only to say a page covers them came out about a third right in the estimate, and phase 9 measures whether they'd help before anyone builds them. [Phase 8](phase-8-coverage.md) and [phase 9](phase-9-adopt.md) say so now.

| | Reports | Right | Fine | Unclear | Share right |
|---|---|---|---|---|---|
| All pull requests (#1 to #81) | 42 | 11 | 31 | 0 | 26% |
| Before `docs/` (#1 to #36; the pages' earlier files) | 17 | 7 | 10 | 0 | 41% |
| Since `docs/` (#39 to #81) | 25 | 4 | 21 | 0 | 16% |

## What was run

- **The map**, [`back-test/map.json`](back-test/map.json), committed and pushed (14e57f6) before the script existed. For each page in `docs/`, the files that held it over time and the globs of the code it describes. Code globs cover sources (`src/`) and manifests, never tests. Before `docs/` was written (42f1c18, 2026-09-30, a commit straight to `main`), some pages lived elsewhere: the command reference in `crates/tessera-cli/README.md`, the editor guide in `packages/vscode/README.md`, the Astro guide in `packages/astro/README.md`, and the contracts in `project-docs/`. The map lists those as earlier paths, and the script uses whichever existed at the pull request.
- **The script**, [`back-test/back-test.mjs`](back-test/back-test.mjs), run on `main` at 7a6c73e (#81). It takes each first-parent commit that's a GitHub merge or squash (66 pull requests; #35, #38, #41, #50, #51, #56, #65 to #72, and #74 aren't on `main`), lists the files it changed against its first parent, and reports each page that existed then, whose globs match a changed file, and whose own file didn't change. Output: [`back-test/results.tsv`](back-test/results.tsv). It takes about 6 seconds.
- **No retuning.** The map is the one committed first. Two entries turned out wrong (see [The map's mistakes](#the-maps-mistakes)); fixing them can only add reports, and taking out the one "right" that depends on a mistake leaves 10 in 41 (24%), so the decision is the same either way. There's one run.

Pages are already whole files, with no includes, so "the page or anything it includes" is the page's file.

## The reports

| PR | Page (file then) | Verdict | Why | Fixed |
|---|---|---|---|---|
| #7 | contracts/content-model.md (`project-docs/content-model.md`) | right | `check_availability` started rejecting a version on a dimension name (Q29); the contract said only that versionless targets take none. | c19ca37 (#12), 0.05 days |
| #13 | contracts/assets.md | fine: refactor | `references.rs` merged the reference rules into one module; no asset behavior changed. | |
| #15 | cli.md (`crates/tessera-cli/README.md`) | right | Added `fmt`; the page still said "Only `check` exists so far" and had no `fmt` section. | 38a92e8 (#20) in part, 0.09 days; 42f1c18 (no PR) in full, 1.8 days |
| #16 | contracts/assets.md | fine: below the page's detail | Include paths and an empty-image message, which the contract doesn't cover. | |
| #17 | contracts/assets.md | fine: implements the text | Reference-style destinations resolve from their definition, as §2 already said. | |
| #20 | contracts/output-layout.md | right | The store removes its own file in the way of a directory (Q117); the contract said the build fails. | a072969 (#23), 0.01 days |
| #20 | contracts/assets.md | fine: implements the text | The new `assets.rs` does what §3.1 and §4 say. | |
| #23 | cli.md (`crates/tessera-cli/README.md`) | fine: comments | Two comments in `build.rs`. | |
| #26 | contracts/output-layout.md | right | The Zod schema got its name (`_tessera/schema.ts`); the contract still said a later phase would choose it. The name is set in `site/mod.rs`, outside this page's globs. | 38418ae (#48), 1.9 days |
| #26 | contracts/assets.md | fine: below the page's detail | Route-like links, not assets. | |
| #26 | contracts/site-render.md | right | Images carry their attributes' declared defaults (Q141); the contract said an image without attributes has no marker. | 3830417 (#29), 0.004 days |
| #29 | cli.md (`crates/tessera-cli/README.md`) | fine: comments | One comment in `build.rs`. | |
| #29 | editor.md (`packages/vscode/README.md`) | fine: comments | Comments in `binary.ts`, `client.ts`, `crash.ts`. | |
| #29 | contracts/assets.md | fine: comments | One comment in `references.rs`. | |
| #32 | editor.md (`packages/vscode/README.md`) | right | Added completion, hover, definition, links, lenses, and hints; the page still said "later completion and navigation". | 42f1c18 (no PR), 0.7 days |
| #33 | contracts/output-layout.md | fine: refactor | `emit_page` pulled out of `emit`; output unchanged. | |
| #33 | contracts/assets.md | right | The preview never serves an asset in the project root, `node_modules`, `.git`, or the output folder (Q182); §7 says a fragment's image shows "exactly as it's found for a build". The behavior is in `preview.rs` and `controller.ts`; the covered `refs.ts` only matched by chance. | Not fixed: `docs/contracts/assets.md:97` |
| #39 | directives.md | fine: dependency | `serde_yaml` to `serde_yaml_ng`. | |
| #39 | content-model.md | fine: dependency | The same swap. | |
| #39 | astro.md | fine: dependency | `engines.node` raised to 24; the page states no Node version (getting-started does, and changed). | |
| #39 | contracts/content-model.md | fine: dependency | The same swap. | |
| #39 | contracts/site-render.md | fine: dependency | The same swap. | |
| #43 | diagnostics.md | fine: refactor | The registry's `rule` became `group`, one for one, and the generator maps groups to the same titles, so the page is unchanged. | |
| #52 | getting-started.md | fine: version | 0.1.0 to 0.1.1. | |
| #52 | editor.md | fine: version | The same, and `minServerVersion`; the page states the rule, not the number. | |
| #52 | astro.md | fine: version | The same. | |
| #53 | editor.md | fine: styling | `preview.css` only. | |
| #54 | editor.md | fine: internal | Development scripts in `package.json`. | |
| #59 | review.md | fine: below the page's detail | Library APIs that nothing used yet; documented in the package README. | |
| #61 | cli.md | fine: refactor | A function pulled out of the HTML report; output unchanged. | |
| #62 | cli.md | fine: styling | `report.css` variables, and `report.js` rebuilt for code that moved. | |
| #75 | review.md | fine: implements the text | A symlink fix that makes the page's statement true. | |
| #76 | editor.md | right | Review now offers itself on a branch with a pull request, running `git` and asking GitHub unprompted; the page says "Review runs `git`, only once you start it." The pull request updated review.md instead. | Not fixed: `docs/editor.md:124` |
| #77 | review.md | fine: below the page's detail | Held comments quote the shown text; the summary gets an end marker. The page's statements hold; the format is in the package README. | |
| #77 | editor.md | fine: below the page's detail | One argument passed through. | |
| #77 | astro.md | fine: below the page's detail | The same. | |
| #79 | editor.md | right | The preview's review header gained "This project has N errors…" and **Show problems**; the page lists the header's contents without it. | Not fixed |
| #79 | astro.md | right | The site panel gained the working tree's errors notice; the page lists the panel's notices without it. | Not fixed |
| #80 | cli.md | fine: styling | `report.css` only. | |
| #80 | review.md | fine: below the page's detail | Site-preview polish; the details went to astro.md, and review.md's general wording holds. | |
| #81 | cli.md | right | The HTML report's tabs and `details` summaries now say "new" or "N changes"; the report's section lists every other mark. editor.md got the line in the same pull request. | Not fixed (the last pull request) |
| #81 | review.md | fine: below the page's detail | The same change; the guide points to the references for marks. | |

The judging rule: **right** means that after the pull request the page said something untrue, or left out something it should have had, because of what the pull request changed. A page that was already wrong doesn't count. Close calls, all judged "fine": #39 on astro.md (the install step could name Node 24), #77 on review.md (the summary marker isn't mentioned, and carries no data), and #79, which is "right" on two reference pages but was documented in review.md and cli.md.

## Why "fine"

| Cause | Reports |
|---|---|
| A dependency or version change the page doesn't state | 8 |
| Below the page's level of detail | 8 |
| A refactor with the same behavior | 4 |
| Comments only | 4 |
| Styling | 3 |
| Code that does what the page already said | 3 |
| Internal (development scripts) | 1 |

Twelve of the 31 are mechanical: a `version` field, an import path, a comment. A report that ignored those would be right 11 times in 30 (37%), still not half. The rest need a person to tell: a refactor, a detail below the page, code catching up with a contract.

Two more things inflate the count. One change reported twice: `crates/tessera-diff/src/html/**` is covered by both cli.md and review.md, and #62, #80, and #81 each produced two reports from one CSS or script change. And one file can be noisy for a page: `references.rs` produced four of the seven assets reports, none of them about assets.

## Where it was right

Seven of the 11 come from before `docs/`, when phase pull requests built the answer to an open spec question into the code, and a "Resolve spec questions" pull request wrote it into the contract minutes or hours later (#7, #20, #26 twice). The project's own process already closed those gaps; a report would have said what the next pull request was about to do.

The four since `docs/` (#76, #79 twice, #81) share a cause: a pull request documented its change in one page and missed another page that describes the same thing at reference level. They're all still open.

## Misses

Drift fixed later, and whether the report would have flagged the pull request that caused it. Nine fixes were found by reading every commit that changed a page or its earlier file.

| Fixed in | Page | What was wrong | Caused by | Flagged | Days |
|---|---|---|---|---|---|
| 38a92e8 (#20) | cli.md | "Only `check` exists so far" after `fmt` | #15 | yes | 0.1 |
| a072969 (#23) | contracts/output-layout.md | A file in the way of a directory fails the build | #20 | yes | 0.0 |
| 3830417 (#29) | contracts/site-render.md | An image without attributes has no marker | #26 | yes | 0.0 |
| 3830417 (#29) | contracts/content-model.md | Image defaults, the root route, and duplicate ids left out | #26 | no: the page changed in the same pull request | 0.0 |
| 42f1c18 (no PR) | cli.md | `--build` "checks one build only", after it became repeatable | #20 | no: the page changed in the same pull request | 1.7 |
| 42f1c18 (no PR) | cli.md | "`check`, `build`, and `fmt` exist so far", without `lsp` | #27 | no: the page changed in the same pull request | 1.0 |
| 42f1c18 (no PR) | cli.md | `tessera --version` after the rename | #31 | no: the page changed in the same pull request | 0.8 |
| 42f1c18 (no PR) | editor.md | "later completion and navigation" after both shipped | #32 | yes | 0.7 |
| 1b9ae41 (#73) | astro.md | "Nothing runs until you open the app and choose Start Review"; opening it starts review | #63 | no: the page changed in the same pull request | 0.1 |

Four of nine flagged. Every miss is the same kind: the pull request edited the page and left one sentence on it stale. A file-level report can't see that, and a region-level one can't either. None of the misses was code outside the map, and none was caused by a commit made straight on `main` (several such commits changed covered code, and none led to a fix).

## Days to fix

| Right | Fixed | Days |
|---|---|---|
| #7, contracts/content-model.md | #12 | 0.05 |
| #15, cli.md | #20 in part; 42f1c18 in full | 0.09; 1.8 |
| #20, contracts/output-layout.md | #23 | 0.01 |
| #26, contracts/output-layout.md | #48 | 1.9 |
| #26, contracts/site-render.md | #29 | 0.004 |
| #32, editor.md | 42f1c18 | 0.7 |
| #33, contracts/assets.md | not fixed | 6 so far |
| #76, editor.md | not fixed | under 1 so far |
| #79, editor.md | not fixed | under 1 so far |
| #79, astro.md | not fixed | under 1 so far |
| #81, cli.md | not fixed | under 1 so far |

Every fix took under two days. This repository is a week old and was written in phases whose plans said to update the docs, so it's a hard place for a coverage report to earn its keep, and an easy one for drift to be caught by the next pull request. The five open items are the ones nobody's next pull request was about.

## The region-level estimate

An estimate, judged by hand, not run. For five pages, the regions a snippet or a `covers` region would plausibly name, and which reports would survive:

- **cli.md:** each command's `Args` in its module, `Command` in `cli.rs`, the exit codes in `exit.rs`, the JSON report's types. Kept: #15 (right, a new command). Cut: #23 and #29 (comments in a function body), #61, #62, #80 (the HTML report's internals and styles). Lost: #81 (a behavior of `report.js`, which no snippet shows).
- **editor.md:** `contributes` in the extension's `package.json` (settings and commands), the server's capabilities in `server.rs`. Kept: #32 (right, new capabilities). Cut: #29, #52, #53, #54, #77. Lost: #76 and #79 (an offer and a notice in the preview's code).
- **astro.md:** the integration's options type. Cut: #39, #52, #77. Lost: #79.
- **review.md:** the CI job (a workflow file) and the comment marker's format. Cut: #59, #75, #80, #81. Kept: #77 (the marker changed; fine).
- **contracts/assets.md:** `mirrored_path`, `relative_reference`, and `encode_path` in `assets.rs`; the asset rules in `references.rs`. Kept: #20 (fine, the code being written) and probably #13 (fine, the rules moving). Cut: #16, #17, #26, #29. Lost: #33.

On these pages the file-level report made 33 reports, 7 right (21%). The region-level one would make about 6, 2 right (about a third), and miss 5 of the 7 right ones. Regions cut the noise, as expected, but what went stale here was mostly behavior a page describes in words (a notice, an offer, a mark), not code a page shows. Where a page does show code, a changed region is a change to the example itself, which is the first group in phase 8's report and the one this result supports. A region named only to say "this page covers that" is better than a file, and still not often right.

## The map's mistakes

- **assets.md** covers `packages/vscode/src/preview/refs.ts` for the preview's asset handling; the handling is in `crates/tessera-lsp/src/preview.rs` and `packages/vscode/src/preview/controller.ts`. #33's "right" matched only through `refs.ts`.
- **output-layout.md** doesn't cover `crates/tessera-emit/src/site/mod.rs`, where the schema's file name is set. #26 matched through `emitter.rs` and `lib.rs`.
- **`references.rs`** on assets.md and **the HTML report** on both cli.md and review.md made reports that were never about the page.

Both fixes add globs, so they add reports; neither would move the result above a third.

## Stale pages found

To fix in their own pull requests:

- `docs/contracts/assets.md` §7: the preview doesn't serve every asset a build would (#33).
- `docs/editor.md:124`: review can run `git` before you start it, to offer itself (#76).
- `docs/editor.md`, Review in the preview: the errors notice and **Show problems** (#79).
- `docs/astro.md`, Review in the site preview: the working tree's errors notice (#79).
- `docs/cli.md`, The HTML report: the "new" and "N changes" hints (#81).
- `docs/getting-started.md`: the CI example sets `node-version: 22`, and the packages need 24 (#39 updated the prose and not the example; found while judging, not reported, since the page changed).
