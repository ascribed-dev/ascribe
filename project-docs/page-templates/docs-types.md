# The docs' pages, sorted by type

Part of [Page templates](README.md), [phase 7](phase-7-docs.md), task 1: "Sort before moving". Every page under `docs/content/`, the type it is today, the type it should be, and what would have to change for it to fit that type's template. Nothing moves until the maintainer has read it.

Read with it: the six templates and their guides in `crates/ascribe-templates/set/`, Ascribe's notes on each in `crates/ascribe-templates/notes/`, and the surface audit's [notes on the docs' structure](../surface-audit/docs-site.md#3-notes-on-the-structure) and findings 38 to 41 and 54. Of those findings, 38 (`guides/variants.md`), 39 (`guides/permalinks.md`), 40 and 54 (the home page's list and getting started's second half), and 41 (the shared fragments of the two `ascribe.toml` pages) have landed. The table is of the pages as `main` has them now.

## How to read it

- **Today** is the type `docs/ascribe.toml` gives the page: `reference` for `reference/**` and `contracts/**`, and `guide`, the default, for everything else.
- **Should be** is one of the six: concept, how-to, tutorial, reference, troubleshooting, release notes. "None" is a page that fits none of them, and a finding about the set.
- **What changes** is what a page would need to fit its template, read as phase 7 says to: their guides say what a kind of page is for, and a page that already does that is left alone. It isn't a rewrite heading for heading. "Nothing" means the page already reads as its type.
- **Two in one** marks a page that's two kinds, which phase 7 splits. A page with several tasks of one kind isn't two in one: the how-to's template has sub-tasks for that.
- Fragments (`_fragments/`) and generated files (`_generated/`) aren't pages, and aren't listed. A page that's an `@include` of a generated list is listed, as the page it is.

The template's own headings are the yardstick for "what changes":

| Type | Its template's shape |
|---|---|
| How-to | Overview; Before you start (optional); a task heading with its steps under `@steps`, and sub-tasks; See also |
| Tutorial | Overview, with who it's for and what they'll be able to do; Background (optional); Before you start; a task heading with `@steps`; Summary; Next steps |
| Concept | A summary and a definition, with no heading; Background, Use cases, and Comparison (some optional); Related resources |
| Reference | What the page lists and where it stops; tables or lists of entries; Commands (optional) |
| Troubleshooting | A symptom per heading, each cause and its solution under it; For more information |
| Release notes | New features, Improvements, Bug fixes, Known issues, Deprecations |

## How many fall under each type

| Should be | Pages | After the splits |
|---|---|---|
| Reference | 34 | 35 |
| How-to | 8 | 8 |
| Tutorial | 2 | 2 |
| Concept | 1 | 1 |
| Troubleshooting | 0 | 1 |
| Release notes | 0 | 0 |
| None | 1 | 1 |
| **All** | **46** | **48** |

Three pages are two in one: the Astro, review, and agents guides. Splitting them adds two pages, since the agents guide's reference half goes into the command reference page that already exists. Of the 46, 32 are `reference` today and stay `reference`; of the 14 `guide` pages, 2 become references, and the other 12 are how-tos, tutorials, a concept, and the one that fits nothing.

## Start

| Page | Today | Should be | Two in one | What changes |
|---|---|---|---|---|
| `index.md` | guide | None | | A list of the other pages, by hand, and a paragraph for implementers. No template is for a landing page. See [what fits nothing](#what-fits-nothing). |
| `getting-started.md` | guide | Tutorial | | It already reads as one: a made-up first project, built up a step at a time, with a "Next" list. To fit: an overview that says who it's for and what they'll be able to do by the end; "You need Node.js" as Before you start; and a Summary. Its sections are its steps, as headings (`## Install the command` to `## Format it`), not a `@steps` list, and should stay headings: each holds code and output. "Check it in CI" and "Work with an AI agent" are how-tos that end the tutorial early; they become Next steps, with a line each, pointing at their pages. |

## Guides

| Page | Today | Should be | Two in one | What changes |
|---|---|---|---|---|
| `guides/astro.md` | guide | How-to | How-to and reference | "Set up a site" (steps 1 to 5), "A home page in front of the docs", "Several projects in one site", and "Publish for AI agents" are tasks. "Options", "What the integration does", "Search engines and crawlers" (the head, the sitemap, `robots.txt`, response headers), "Review in the site preview" ("this section is the reference for the site's side"), "Styling", and "Other Markdown processors" are the integration's reference: about 300 of its 480 lines. Split into a how-to, which keeps the page and its permalink `astro-guide`, and a reference for `@ascribed/astro`. The three headings with permalinks (`a-home-page-in-front-of-the-docs`, `several-projects-in-one-site`, `review-in-the-site-preview`) move with their sections. The set-up steps are headings with numbers (`### 1. Install`); a `@steps` list can't hold the files they show, so they stay headings. |
| `guides/editor.md` | guide | Reference | | It's the extension's reference already: "What it does" is a section per feature, and its review sections call themselves "the reference for the editor's part". "Install" is the one task, and getting started installs the extension too; it stays as the reference's first section, which says how to get what it describes. Only the type changes. |
| `guides/editor/settings.md` | guide | Reference | | The extension's settings and commands, from generated tables, and how it treats a workspace with several projects. Only the type changes. |
| `guides/variants.md` | guide | Tutorial | | Written as one: an example project (`examples/platforms`), four numbered parts, and "Where to go from here". It teaches the four ideas a page that varies needs, which is what a tutorial is for, not one task a reader already understands. To fit: who it's for (a reader who has done getting started), a Summary, and "Where to go from here" as Next steps. Its numbered headings stay headings, as getting started's do. |
| `guides/review.md` | guide | How-to | How-to and troubleshooting | "Review a pull request" is a how-to, with steps 1 to 5 as headings, and "The report in CI" a second task. "When something's missing" is a troubleshooting page inside it: a symptom in bold, then its cause and what to do, seven times. Split it out, symptom per heading. "What you need for each part", "How comments map to the pull request", and "Privacy and security" are reference the how-to links to; they're short, and stay. The page keeps its permalink, `review-guide`. |
| `guides/drift.md` | guide | How-to | | Three tasks: take examples from tested code (steps 1 to 3 as headings), follow code in another repository (with the update pull request, and the App it needs, as a numbered list), and report changed examples in CI. Each is a sub-task. To fit: the two numbered lists of actions under `@steps` ("Docs kept apart from the code" and "The access it needs"), and See also in place of the last line's link. |
| `guides/agents.md` | guide | How-to | How-to and reference | Setting an agent up (`agents sync`, the MCP server, hooks, the plugin, Copilot, other agents) is the task. The reference half is what each part writes and says: the table of agent files, what a prompt says, the MCP server's tools, and the hooks' entries by target. Most of it is said again, in other words, in [`ascribe agents`, `mcp`, and `lsp`](../../docs/content/agents/commands.md), so it moves there and no page is added. "What's sent where" and "Agents that reach the network" explain rather than instruct, and stay as the how-to's last sections; there's no concept page they'd be enough for. The page keeps its permalink, `agents-guide`. |
| `guides/vale.md` | guide | How-to | | "Start from the quiet preset" is a task with numbered steps; "Bring your own Vale configuration" and "From the preset to a configuration of your own" are two more. To fit: the first list under `@steps`, Vale 3.16 as Before you start, and See also for the closing line's two links. "Where Vale runs" is reference, and short enough to stay. |
| `guides/report.md` | guide | How-to | | Acting on what `ascribe report` finds, a section of the report at a time, then running it on a schedule; the command's options and JSON are on the command's page. Each of the report's sections is a sub-task ("Problems", "External links", and the rest), and "On a schedule" and "Handing a section to an agent" are two more, so nothing splits. To fit: an Overview that says what the reader will do with the report, `lychee` and `afdocs` as Before you start for the sections that need them, and See also for the command's page and the measuring guide. |
| `guides/measuring.md` | guide | Concept | | Explains what Ascribe can't see, where a team looks instead, and how strong the evidence is for each piece of advice. No steps. Its sections are its own and say what the concept template's use cases and comparison would. To fit: a Related resources list, for the report guide and `[crawlers]` it links in passing. |
| `guides/permalinks.md` | guide | How-to | | Five tasks as sections: publish the addresses, link from a product, check the links in its CI, move or retire one, and show a 404 page that names it. The first paragraph explains what a permalink is; a how-to's Overview says what the reader will do, and a link to [the directive reference's permalinks](../../docs/content/writing/directives.md#permalinks) carries the rest. See also for the references it links in passing. |
| `guides/templates.md` | guide | How-to | | Taking the templates, changing one, and starting a page are the tasks. "Where the templates come from" and "Ascribe's notes" are short and stay as the Overview; the credit to The Good Docs Project is needed on this page (decision 23). "What's left to fill in" says what `template-placeholder-left` reports, which [its diagnostic's entry](../../docs/content/checks/diagnostics/content-checks.md) has; the how-to keeps a paragraph and links there. "Start a page" is two commands and a sentence: a `@steps` list of three. |

## Reference

| Page | Today | Should be | Two in one | What changes |
|---|---|---|---|---|
| `reference/cli.md` | reference | Reference | | Nothing. |
| `reference/cli/check.md` | reference | Reference | | Its `ascribe check` subsections are named for tasks ("Checking some files", "A quick check after each edit", "Output for agents") but each says what one option does. Nothing needs to move; the headings could name the option instead (`--stdin`, `--editor-build`, `--format concise`). That's a heading change on a page with permalinks under it, and not one the type needs. |
| `reference/cli/diff.md` | reference | Reference | | Nothing. |
| `reference/cli/questions.md` | reference | Reference | | Nothing. |
| `reference/cli/agents.md` | reference | Reference | | Nothing, apart from taking the agents guide's reference half. |
| `reference/cli/sources.md` | reference | Reference | | Nothing. |
| `reference/content-model.md` | reference | Reference | | Nothing. Two pages still call it a guide, as the audit's §3 found: the home page ("the guide for writing one") and the contract ("For a guide to writing one"). Those two sentences change, not this page. There's still no guide to writing a content model past getting started's one section; that's a page the docs don't have. |
| `reference/content-model/content.md` | reference | Reference | | Nothing. |
| `reference/content-model/output.md` | reference | Reference | | Nothing. |
| `reference/content-model/checks.md` | reference | Reference | | Nothing. |
| `reference/directives.md` | reference | Reference | | Nothing. Its first sections ("Files", "Directive lines", "Binding", "Lists and block quotes") explain how the language is read before the directives are listed, which is more than a reference's description usually does, but a reader looking up `@variant` needs them, and splitting them off would leave both halves needing the other. |
| `reference/diagnostics.md` | reference | Reference | | Nothing. |
| `reference/diagnostics/content-checks.md` | reference | Reference | | Nothing. |
| `reference/diagnostics/content-model.md` | reference | Reference | | Nothing: a sentence and a generated list. |
| `reference/diagnostics/directives.md` | reference | Reference | | Nothing: a sentence and a generated list. |
| `reference/diagnostics/pages.md` | reference | Reference | | Nothing: a sentence and a generated list. |
| `reference/elements.md` | reference | Reference | | Nothing. "A project's own note types and lifecycle states" and "Choosing a tab from a script" are short how-tos, each a CSS or JS example, of the kind the reference guide says to keep brief; they stay. |
| `reference/project-files.md` | reference | Reference | | Nothing. |

## Contracts

| Page | Today | Should be | Two in one | What changes |
|---|---|---|---|---|
| `contracts/content-model.md` | reference | Reference | | Nothing in its shape. It's normative, with rules a loader MUST enforce; see [what fits nothing](#what-fits-nothing). |
| `contracts/content-model/content.md` | reference | Reference | | As above. |
| `contracts/content-model/output.md` | reference | Reference | | As above. |
| `contracts/content-model/checks.md` | reference | Reference | | As above. |
| `contracts/content-model/validation.md` | reference | Reference | | As above. Its "Decisions", with what was considered for each, is the furthest a contract gets from the reference template. |
| `contracts/site-render.md` | reference | Reference | | As above. "Why a marker" is rationale. |
| `contracts/output-layout.md` | reference | Reference | | As above. "Why this shape" is rationale. |
| `contracts/assets.md` | reference | Reference | | As above. |
| `contracts/json-reports.md` | reference | Reference | | Nothing: the rules every schema follows, and two generated schemas. |
| `contracts/json-reports/diff.md` | reference | Reference | | Nothing: generated schemas. |
| `contracts/json-reports/report.md` | reference | Reference | | Nothing: a generated schema. |
| `contracts/json-reports/sources.md` | reference | Reference | | Nothing: generated schemas. |
| `contracts/json-reports/templates.md` | reference | Reference | | Nothing: generated schemas. |
| `contracts/json-reports/questions.md` | reference | Reference | | Nothing: generated schemas. |
| `contracts/permalinks-list.md` | reference | Reference | | As the other contracts. |

## What fits nothing

These are findings about the set, not about the pages.

- **A landing page.** `index.md` is a list of the other pages with a sentence each, and a paragraph for implementers. None of the six is that, and the set's seventh Core template, the README, which decision 10 left out, is for a repository and not a docs site's home page either. A type with no template and no guide works as types do today (decision 7), so the default is a type of the docs' own for it, `index`, with neither key. The alternative is to call it a reference, a list of pages, which it would fit only by a stretch.
- **A normative contract.** The 14 contract pages are `reference`, as the maintainer settled in review, since their shape is a reference's: what the page covers, then a numbered section for each table or file, with its keys and rules. But they're written for implementers, their rules say MUST, and three of them carry what a reference guide says to leave out: why the shape is what it is ("Why a marker", "Why this shape"), and a decision log with the alternatives considered (`contracts/content-model/validation.md` §26). The Good Docs Project has no template for a specification, and the reference's is close enough: the six cover every page but one. The sidebar keeps its Contracts group, since the sidebar isn't the type.
- **No release notes page.** The type has no page under `docs/content/`. The docs' release notes are `CHANGELOG.md`, gathered from `changes/` and rendered by the site as its changelog, outside the content root. Phase 7 doesn't eject the type: it ejects only the types the table needs.
- **No troubleshooting page today,** and one after the split, out of the review guide. The audit's task "fix a broken link" lands on a diagnostic's entry and the specification, and still has no page; a troubleshooting page for links would be the one to write, and is a new page, out of this phase's scope.
- **No concept page for the language's own ideas.** A dimension, a build, a fragment, and the content model are explained inside the two tutorials and the references, never on a page of their own. `guides/measuring.md` is the only concept page, and it's about what happens after a deploy. That's a gap the types make visible, not a page to change.

## Left for the later tasks

- **The docs' own types.** The phase file says the docs have two types, `guide` and `reference`, which is still true. Taking the six means removing both and putting back `inline = "code"` on `title` and `role = "description"` on `description` by hand, as the phase says; that's task 2, and this table doesn't change it.
- **The sidebar.** By the table, the sidebar's "Guides" group becomes how-tos, tutorials, and one concept, and two of its pages join "Reference". Task 4 decides the groups.
