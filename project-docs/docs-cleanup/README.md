# The docs after the audit: the plan

The [docs audit](../docs-audit.md) of 20 October 2026 found the docs accurate and the wrong shape: four pages that are several documents each, facts with second homes, a product that's visual described in prose, and the marks a release left behind, with 24 small items of drift. This plan fixes what it found, in phases of one pull request each, for an agent fleet working from the board.

Written 20 October 2026. The audit is the evidence: each phase names the sections of it that it acts on, and doesn't repeat them. Where this file and the audit disagree about what a phase does, this file wins; where either disagrees with the docs or the code as they are now, stop and ask the maintainer.

**No thread.** The steel-threads skill's test: this work touches one layer, the docs, and changes no behavior, so each phase is a slice of the docs on its own, and the order is by what readers gain soonest, the audit's "Order of work".

## Who it's for

- **A reader looking for one task,** who today reads 1,200 words of concept on the versions page before the steps, or scrolls a 6,000-word agents page with 21 headings.
- **A reader of a reference,** who reads a command's options twice, once in the generated table and again in paragraphs under it.
- **A reader of the editor guide,** who gets 1,400 to 1,800 words of prose per page about things that are shapes on a screen, and no picture.
- **Whoever maintains the docs,** writer or agent, who has to change a rule in three to six places, and a key in a reference and its contract both.

## Rules for every phase

These come from the audit and from the repository's own rules. They're settled; a phase doesn't reopen them.

1. **Sections are features, and a section is a folder** (decision #452). A page's type is in its frontmatter; it decides where a page sits in its section, never which section. A new page goes in the feature's folder, beside the pages about the same thing.
2. **One home per fact.** What the code declares is generated or linked, never written again: a command's options are its generated table; a JSON document's fields are its schema in `contracts/json-reports/`, shown on a reference page as a `@snippet` of real output, not as prose. A rule is stated once, where it belongs, and linked from everywhere else.
3. **A reference says what is.** Why a choice was made belongs in `project-docs/decisions/`, and how the docs got this way belongs in git history. Neither stays on a published page.
4. **A how-to is one task,** with at most eight to ten steps, as the how-to template's guide says. A how-to that holds several tasks becomes several how-tos; the section's own page is the overview that leads to them.
5. **A moved section keeps its ids.** Its `@id` lines go with it, so its permalinks keep working; `ascribe permalinks sync --config docs` rewrites the list, and `permalinks-list-behind` must report nothing. Every link to it is re-pointed: `ascribe refs <page>#<id>` lists them. No redirects: Ascribe is before 1.0. A heading linked from outside the docs (the READMEs, the skill, the extension's manifest) that has no permalink gets one before it moves, so the outside link goes through `/go/` and survives the move; `scripts/docs-site/links.test.ts` is the test that shows each of those links landing. A heading nothing outside links to may move without one.
6. **A new page is placed.** `docs/ascribe.nav` lists it where its section's order isn't the default, and `site/test/fixtures/sidebar.json` changes with the sidebar, as `site/test/e2e.test.ts` asks.
7. **Each phase cuts while it moves.** A page a phase splits or moves loses, in the same pull request, the re-narrated options, the JSON prose, the restated rules, and the rationale the audit lists for it. A phase that only moves text leaves the work for a second pass that nobody schedules.
8. **Generated files aren't edited by hand,** and no formatter runs over Markdown (AGENTS.md).
9. **A phase names a page's headings, not its lines,** where other work edits the page first: the lines will have moved by the time the phase runs. The audit keeps its line numbers as evidence.
10. **A command's output shown on a page is real output,** a `@snippet` of a file in `crates/ascribe-cli/tests/output/` that `crates/ascribe-cli/tests/all/output.rs` writes. Where a phase needs one that isn't there, adding it is a test change, named in the pull request's description, and observed again (`cargo run -p ascribe-cli -- observe`).

## Decisions

The plan's four open questions, answered by the maintainer on 20 October 2026. An issue quotes the answers its phase reads, so triage sees they're made.

1. **What a contract page holds** (the audit's section 2, "The `ascribe.toml` reference and its contract", and its "Order of work", item 6). **Decided:** a contract page holds what's normative and its own: the rules, stated with MUST; the file formats (`ascribe.lock`, `llms.txt`'s layout, the link cache's files); and the key tables, which it shares with the reference through the registry's generated fragments (#650), never a hand-written one, which would be a third home. No explanation. A reference page holds the tables, the explanation, examples, and tasks, and no MUST. Each links to the other at the top. The split is already half there: the contract pages have ten MUSTs and the reference none. Moving the 23 decisions out of `contracts/content-model/validation.md` §26 is itself a change to a published contract page, so D17 adds one decision file of its own, "what a contract page holds", and the 23 become decision files numbered by D17's issue with letters, as [the decisions folder's README](../decisions/README.md) has it for several from one source, or are named as superseded by the files that already replace two of them. *Read by:* D17.
2. **Correcting a contract page.** **Decided:** a correction that brings a contract page to what SPEC and the loader already enforce isn't a change to the contract, and needs no decision file: the page was wrong about it. It holds only where SPEC and the code agree, and a test or a conformance case shows it. Where SPEC and the loader disagree, there's nothing to correct toward, and that's a decision for the maintainer. Where the page is right and the code is wrong, it's a bug, with a pull request of its own (AGENTS.md). The pull request lists each correction with the line of SPEC, the code, and the test that settle it. *Read by:* D1.
3. **Where "what search engines and agents are told" lives** (the audit's section 2: listing, crawlers, and headers, one feature spread over three key references). **Decided:** a section of its own, `publishing/`: what a project tells search engines and AI crawlers, and what it delivers to agents (`llms.txt`, the Markdown versions, their headers). The name leaves room for what else is about how a site is published rather than written: the host's response headers, and redirects if they're built (the brainstorm's section 6). Deploying a site stays the getting started tutorial's and the Astro section's. Not the Astro section: `[listing]` and `[crawlers]` are content model settings, kept out of `[consumer]` for that reason, the page records are a file any generator reads, and `llms.txt` is the binary's. The brainstorm line that placed it in the Astro guide predates the search and answers plan. The Astro reference keeps one paragraph, what the integration writes from the records, and a link. *Read by:* D14.
4. **The pictures: scripted or by hand.** **Decided:** scripted. The editor's picture already is: `packages/vscode/scripts/screenshot.ts` takes it, and `scripts/docs-site/screenshots.ts` keeps its record and the command that retakes it, with `scripts/docs-site/screenshots.test.ts` failing one minor release behind. So the scenes extend a harness that exists. The risk is reach: a hover, a peek, and a status bar in several states are scenes a script has to drive, not only open. A scene the script can't reach falls back to a picture taken by hand with the same record, so the staleness test holds either way. *Read by:* D18, D19, D20.

## The phases

| Track | Phases, in order |
|---|---|
| Fixes | D1, D2 |
| Start and writing | D3, D4, D5 |
| Agents and the command | D6, D7, D8 |
| Feature sections | D9, D10, D11, D12, D13, D14 |
| The model and its contract | D17 |
| The editor | D15 |
| Pictures | D18, D19, D20 |

Within a track, one phase at a time, in its order. Across tracks, phases run at once as long as no two in flight own the same page ("What can't overlap"). D16 sweeps what the others leave, and runs last.

**Why this order.** The drift and the old marks first: a day's work, no structure changes, and every later phase starts from pages that are right. Then the commands in the wrong sections, which the audit counts 27 inbound links for, and the four splits a reader notices first. Then the rest of the sections, the model and its contract once the registry's tables are generated, the editor once its views stop changing, and the pictures as their subjects settle.

### What each phase needs

One thing here is code: #650, the views plan's phase that generates the content model reference's tables from the registry. D14 and D17 wait for it, and D1's items on those pages run before it starts or after it merges. Don't pick D14 or D17 up before #650 has merged.

| Phase | Needs | Why |
|---|---|---|
| D1 | Nothing | |
| D2 | #667 | The check that proves no old mark is left |
| D3 | Nothing | |
| D4 | D3 | `init`'s reference moves to the page D3 makes |
| D5 | D2, #669 | The marks and the generated table are off the page before it's split |
| D6 | Nothing | |
| D7 | D6 | |
| D8 | D7 | |
| D9 to D13 | Nothing | Each owns its own pages |
| D14 | #650 | The registry's tables are generated first |
| D15 | #651, #654, #661 | The views plan's phases that rewrite the editor's pages; and not in flight with #653 or #659, which D20 waits for |
| D16 | D1 to D15 | It sweeps what those left; D17 and D18 to D20 do their own pages' |
| D17 | #650, D1, D14 | The registry's fragments, the corrected pages, and the listing and crawlers sections moved out |
| D18 | Nothing | |
| D19 | D18 | The scenes |
| D20 | D18, D15, #653, #659 | The new views, and the editor's pages as they'll stay |

### Fixes

**D1. The drift.** The audit's section 4: the 24 confirmed items, the two links under "Also worth a line", and the doc comment at `crates/ascribe-resolve/src/pages.rs:232` that item 9 names. Each is corrected to what the code, SPEC, or `--help` says, and only that: no rewriting around it. The contract pages' items by decision 2: corrected only where SPEC and the code agree and a test shows it; a disagreement between them is reported to the maintainer, and a page that's right about wrong code is a bug for its own pull request. *Pages:* those the table names. *Done when:* every item is corrected or reported, each listed in the pull request with the SPEC line, the code, and the test that settle it, and `ascribe check --deny-warnings --config docs` passes.

**D2. The release's old marks.** The audit's section 1, "Marks and badges the release left behind". Every `@available` mark that names only versions at or before the oldest live line of `ascribe` (0.4.0 now) goes, or becomes the simpler spec without them, as `available-same-in-every-line` offers; the 0.4.0 marks stay. `templates/index.md`'s "removed 0.4.0" bullet goes, and `writing/memo-files.md`'s 0.4.0 mark inside a 0.4.0 page. *Pages:* every page with such a mark. `git grep -n "@available: ascribe 0\.[23]" -- docs/content` is a starting list, not the test: it misses `available:` in frontmatter, `{available=…}` on a table's row, and a history such as `(preview 0.2.0, ga 0.3.0)`. *Done when:* `ascribe check --config docs` reports no `available-same-in-every-line`, with #667 merged so that it would.

### Start and writing

**D3. An Install page, and a tutorial with one path.** The audit's section 2, `getting-started.md`. A new page, `install.md`, in the Start section beside the tutorial: Homebrew, the install script, PowerShell, the zip, npm, and three binaries side by side. The tutorial keeps one path (npm, `init`, write, check, build), drops the "write it by hand" branch for 0.3.0, moves its CI how-to beside `checks/check.md` as its own page, and ends at a built site or says plainly that Astro is the second half. *Pages:* `getting-started.md`, the new `install.md`, the CI page. *Done when:* the tutorial has one path and no install reference, and every install link leads to `install.md`.

**D4. The six-command page.** The audit's section 2, `examples/sources.md`. `sources` and `observe` stay; `permalinks` moves to `permalinks/`, `templates` to `templates/`, `vale` to `checks/vale.md`, and `init` to `install.md`, as "Start a project"; D4 owns `install.md` once D3 has merged. `command/index.md` then sends each command to its feature. The page's JSON prose becomes snippets and links to the schemas (rules 2 and 10); `sources fetch`, `sources status`, `observe`, `permalinks list`, and `templates status` have no output in `crates/ascribe-cli/tests/output/` today. *Pages:* `examples/sources.md`, `permalinks/index.md`, `templates/index.md`, `checks/vale.md`, `install.md`, `command/index.md`, `contracts/json-reports.md`. *Done when:* `examples/sources.md` has two commands, each command's reference is in its feature's section, and no link leads to a section that moved.

**D5. The directive reference, as four pages.** The audit's section 2, `writing/directives.md`: syntax; the built-in directives, scannable, with regions and permalinks linked out; frontmatter, a page the docs don't have, with the reserved keys as a table; and inline constructs. The hand-written Contents goes. The copies of the permalinks and versions guides become links. *Pages:* `writing/directives.md` and the three new pages in `writing/`. *Done when:* no page is over 3,000 words, the 37 headings are on the pages they belong to, and every link to a moved section is re-pointed (`ascribe refs writing/directives.md` lists 148).

### Agents and the command

**D6. Two references in the wrong section.** The audit's section 2, `review/diff.md` and `agents/commands.md`, and its "Order of work", item 2. `ascribe report`'s reference moves beside `checks/report.md`, and `review/diff.md` keeps the two git comparisons. `ascribe lsp` leaves `agents/commands.md` for the editor section, on `editor/index.md` until D15 makes the language server's page, and the page's title loses it. *Pages:* `review/diff.md`, `checks/report.md`, `agents/commands.md`, `editor/index.md`. *Done when:* each command's reference is in its feature's section.

**D7. Agents, as an overview and a page per task.** The audit's section 2, `agents/index.md`: a 300-word overview, and a page each for working with an agent, the agent files, prompting, the MCP server, hooks, Claude Code, Copilot, and indexing a product's docs for its agents; one concept page for what an agent reads and what's sent where. The VS Code part folds into `editor/agents.md`. What `agents/commands.md` already says (the hook's report, `--cloud`) is linked, not restated, and "Ascribe calls no model" is said once. *Pages:* `agents/index.md`, the new pages in `agents/`, `editor/agents.md`. It's the phase "The pull request says what moved where" exists for: ten pages, 39 inbound links, and a fold into another section, so its commits come in two kinds, the moves first, which change no words, then the cuts, each commit readable alone, and its description is page by page. *Done when:* the overview is about 300 words, each task is one page, and the 39 inbound links are re-pointed.

**D8. A command's reference says what its help can't.** The audit's section 1, "Options re-narrated" and "JSON fields written as prose", for the command references the splits don't take: `checks/check.md` (its options, from "Checking some files" to "Applying the safe fixes", and its two tables under "JSON output"), `agents/commands.md`, and `command/questions.md` (the JSON paragraphs under `ascribe explain`, `ascribe model`, and `ascribe render`, which #665 renames `ascribe resolve`). Each keeps the generated table, the exit codes, a snippet of real output, a link to its schema, and what the help can't say. Outputs it needs and doesn't find go in by rule 10. *Pages:* those three. *Done when:* no paragraph repeats an option's help, and no field list repeats a schema.

### Feature sections

**D9. Versions, as two how-tos and a reference.** The audit's section 2, `versions/index.md` and `versions/release.md`: how-to "Publish a site for each version", about 900 words; how-to "Archive and retire a line"; a concept and reference page "Release lines" (the levels, what a line holds, `versions.json`); the diagnostics list becomes a link. `release.md`'s "Release day" joins the how-to, and its JSON prose becomes snippets and links: the text outputs of `versions list`, `changes`, `release`, `archive`, and `retire` are in `crates/ascribe-cli/tests/output/`, and their JSON isn't, so rule 10 applies. *Pages:* `versions/`. *Done when:* the task is reached in the first screen of its page.

**D10. Review, said once.** The audit's section 2, `review/index.md`, and section 1's "One rule, many homes" row for review. The how-to keeps its steps; how comments map to the pull request and what stays private become an explanation page; the report in CI becomes its own how-to, which `review/diff.md` already points at. The review marks, the three-way switch, sign-in, the comment flow, Pull and Push, and the errors notice are each written once, on the page they belong to, and linked from `editor/review.md`, `review/diff.md`, and `review/troubleshooting.md`. The review panel's walkthrough leaves `astro/reference.md` for `review/`. *Pages:* `review/`, `editor/review.md`, `astro/reference.md`'s review panel section. *Done when:* each of those rules is written on one page.

**D11. Examples and templates.** The audit's section 2, `examples/index.md` and `templates/index.md`. `examples/index.md` becomes the section's overview of three how-tos: snippets from tested files, docs in another repository, the drift report. `tested.md`'s "Count a project's examples" moves to the report. `templates/index.md` keeps its task, about 900 words, with three lines and two links for ASC195 and ASC229 and the migration story in one paragraph. *Pages:* `examples/index.md`, the new how-tos in `examples/`, `examples/tested.md`, `templates/index.md`. *Done when:* each how-to is one task.

**D12. The report, and Vale's policy.** The audit's section 2, `checks/report.md`, and section 5's note on `vale.md`. External links become a how-to of their own; the site and agents part moves beside `agents/measuring.md`; the link cache and history files are described once, here, and linked from the content model reference and its contract. `checks/vale.md`'s security policy, under "Where Vale runs", is cut to what a reader does with it. *Pages:* `checks/report.md`, the new external-links how-to, `checks/vale.md`, `agents/measuring.md`. *Done when:* the report page is about reading the report.

**D13. Astro, permalinks, and navigation.** The audit's section 2, `astro/index.md`, `astro/reference.md`, and `permalinks/index.md`, and section 5's note on navigation. "A site with more than the docs" and "Publish for AI agents" become how-tos; the component notes go to the reference; nginx and the hosts' configuration become one how-to, "Serve the Markdown versions"; the permalinks forwarding page and the 404 page move to the Astro reference, and the permalinks how-to keeps checking a product's links and retiring a name. The rule about a nested section's place is written once, in `navigation/index.md`, and `navigation/order.md` links to it. *Pages:* `astro/`, `permalinks/index.md`, `navigation/`. *Done when:* each how-to is one task, and the navigation rule is on one page.

**D14. What search engines and agents are told.** The audit's section 2, the listing, crawlers, and headers row, by decision 3: a new section, `publishing/`, whose own page says what a project tells search engines and AI crawlers and what it delivers to agents (listing, `[crawlers]` and `robots.txt`, the response headers, `llms.txt`, and the Markdown versions), and the key references keep their tables. The crawler policy essay in `content-model/checks.md` (165 to 177) goes to the section as what a reader decides, without its dated citations. The three `listing` values are said once. The Astro reference keeps one paragraph on what the integration writes from the page records, with a link. `docs/ascribe.nav` places the section. *Pages:* the new section, `content-model/index.md`'s listing section, `content-model/checks.md`'s crawlers section, `content-model/output.md`'s headers section, and `astro/reference.md`'s paragraph. *Done when:* the feature has one section, each key reference keeps its table and a link, and the brainstorm's "Built" line points at the section, not the Astro guide.

### The editor

**D15. The editor's pages, settled.** It starts once the views plan's phases that rewrite these pages have merged (#651, #654, #661), and doesn't run beside #653 or #659. The audit's section 2, `editor/index.md`, `editor/sidebar.md`, `editor/preview.md`, and `editor/settings.md`, and section 5's note on its seams. A language server reference page, `editor/language-server.md`, for an editor other than VS Code: the requests, from the surface map's "The language server" section, and what `editor/index.md` says at 56 to 60, with `ascribe lsp` from D6. `editor/sidebar.md` gets one H2 for each view, and its drag rules go to `navigation/order.md`. The status bar leaves `preview.md`; the Problems panel leaves `settings.md` for `problems.md`; formatters go to `writing.md`. The seams: `settings.md` introduced once, the formatter offer once, Insert at the Cursor described once, and `actions.md:52`'s completion on `writing.md`. *Pages:* `editor/`. *Done when:* `settings.md` is what its title says, every `ascribe/*` request in the surface map is on the language server page, and the sidebar's 13 inbound links land on the right view's heading.

### Last

**D16. The rest of the second homes.** What the audit's section 1 lists and no phase from D1 to D15 took. It doesn't sweep the model and its contract, which are D17's, or the pages the pictures change, where D18 to D20 cut the prose each picture replaces. What it takes: each row of "One rule, many homes" not already said once (memos' text, the HTML comment that's a memo, `ASCRIBE_TODAY`, the build you're looking at); the rationale in `contracts/assets.md`, `contracts/output-layout.md` §5, `contracts/site-render.md`, and `writing/directives.md`'s `include-limit`; `writing/memos.md`'s second half; `writing/memo-files.md`'s rename paragraph; `content-model/content.md`'s case study; `content-model/glossary.md`'s reference tail. *Pages:* those named. *Done when:* `git grep` for each rule's distinctive phrase finds it on one page, and no reference page carries a decision's reasoning.

**D17. The reference and the contract, each with its own job.** The audit's section 2, "The `ascribe.toml` reference and its contract", by decision 1: the contract keeps what's normative and its own, the reference keeps the explanation, the prose they share is cut from the one it doesn't belong in, the key tables they share are the registry's generated fragments alone, the fragments carry no MUST into the reference, and each links to the other. A decision file of its own, numbered by this phase's issue, says what a contract page holds; the 23 decisions in `validation.md` §26 become files numbered by the same issue with letters, or are named as superseded by the files that already replace them. *Pages:* `content-model/` and `contracts/content-model/`, and the fragments they share. *Done when:* no sentence appears in both outside a shared fragment, no hand-written key table is left in either, and §26 is gone from the contract, each of its decisions a file in `project-docs/decisions/` or named as superseded by one.

### Pictures

Each picture replaces the prose it shows: a phase that adds one cuts the paragraph it stands for. The audit's section 3 ranks them, with the words each replaces.

**A picture has a text form.** Its alt text says in a sentence what the cut paragraph said, so a reader with a screen reader loses nothing: decision 46's "nothing is told by color alone" holds for a picture too. `image-alt-missing` is the check that holds an image to having one; the review of each phase holds the sentence to saying what the picture shows.

**D18. The scenes, and the editor's pictures that won't change.** By decision 4: `packages/vscode/scripts/screenshot.ts`, which takes the editor's picture today, takes a named scene each, and `scripts/docs-site/screenshots.ts` keeps each scene's record and retakes it. The hover scene goes first, since a hover is the hardest to stage. The first scenes are the build lens with its hover (and the variants tutorial's payoff), the actions bar, the Review view, the status bar item in its states, and "Shown in 3 pages" with its peek (the audit's 2, 4, 6, 7, and 9). Pictures go in `docs/content/images/`, light and dark, under `image-large`'s limit. *Done when:* each picture is on its page with alt text that says what its cut paragraph said, its prose is cut, and the freshness test covers it; a scene the script can't reach is taken by hand, says so in the pull request, and has the same record, so the test covers it too.

**D19. The site's and the report's pictures.** The review header, cropped from the picture there is; the HTML report with its four mark kinds; the elements in light and dark; the starter's sidebar beside the navigation contract's §2; and the `<Versions>` switcher with `<VersionBanner>` (the audit's 1, 5, 10, 11, and 12). *Done when:* as D18.

**D20. The new views.** This page, Pages, and the Project view, once the views plan has drawn them (#653, #659), and once D15 has settled their pages (the audit's 3 and 8). *Done when:* as D18.

## What can't overlap

Two phases are in flight at once only when they own no page in common: each phase's entry lists its pages. A link re-pointed on another page doesn't count as owning it; the second to open its pull request merges `main` and keeps both sides.

Other work in flight owns pages too, and a phase doesn't run beside it:

- **#650,** which generates the content model reference's tables, owns `docs/content/content-model/` and `docs/content/_fragments/content-model/` until it merges (the [views roadmap](../roadmap.md#what-cant-overlap)). D1's items there, D14, and D17 wait for it, or run before it starts.
- **#665,** the rename of `ascribe render`, edits `command/questions.md`, `agents/index.md`, `agents/commands.md`, `review/diff.md`, `versions/index.md`, `writing/variants.md`, and `contracts/json-reports/questions.md`.
- **The search plan's #641, #643, and #644, and #670,** edit `command/questions.md` and the agents pages.
- **The views plan's #651, #653, #654, #659, and #661** edit `editor/sidebar.md`, `editor/settings.md`, the walkthrough, and the extension's README. D15 doesn't run beside any of them, and D20 waits for #653 and #659.

Three files change in most phases, and conflict when two phases add pages at once:

- **`docs/ascribe.permalinks`** is settled by regenerating: `ascribe permalinks sync --config docs` after merging `main`.
- **`docs/ascribe.nav`** is an order kept by hand. The phase that merges second takes `main`'s file and places its own pages in it again.
- **`site/test/fixtures/sidebar.json`** is written by hand too: `site/test/e2e.test.ts` compares the sidebar with it and has no step that rewrites it. The phase that merges second takes `main`'s fixture and adds its own pages' entries, and the test shows any it missed.

Files settled by regenerating, never by editing: `docs/content/_generated/` (each generator's test with `ASCRIBE_BLESS=1`), and `project-docs/surface.md` (`node scripts/surface/inventory.ts`).

## How a phase is run

### Its issue

Filed at once for every phase, so the needs can name issue numbers. Triage moves an issue to Ready when it's one pull request, says what done looks like, waits on nothing open, and asks no decision only the maintainer can make; each is written to pass:

- **Title:** "Docs: " and the phase's name, as above.
- **Body:** "Phase D7 of [the docs after the audit](link to this file). The spec is this phase's entry there and the audit's sections it names. **Done when** below is what done looks like, and it's one pull request." Then **Pages:** and **Done when:**, copied; **Decided:**, each decision above the phase reads, with its answer; and **Needs:**, as issue numbers.
- **Labels:** `documentation` and `docs-audit`. The `docs-audit` label is created before the first issue is filed.

### Its pull request

- **One pull request a phase,** from its own branch, built by `implement`, driven by `steward`, and reviewed and merged by `review`. Before review, merge `main` into the branch; never force-push or rebase a pushed branch.
- **The checks:** AGENTS.md's four commands, and `cargo run -p ascribe-cli -- check --deny-warnings --config docs`, and the site's tests, `pnpm --filter ascribe-docs-site test`, which hold the sidebar and the site's links.
- **What a reader would notice** gets an entry in `changes/docs/`: a page split, moved, or added.
- **The pull request says what moved where,** page by page, and what was cut, so the reviewer can tell a move from a rewrite.

### When to stop and ask

- A page, the audit, and the code disagree.
- A cut would remove something no other page says.
- A section can't move without breaking a permalink.
- A phase needs a page another phase in flight owns.

## When this file changes

When a phase merges, nothing here changes: the board says what's done. One change is allowed while phases run: a decision the maintainer takes during a phase, which its issue quotes, is added under Decisions, since this file is where triage reads them.
