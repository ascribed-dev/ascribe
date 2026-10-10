# The surface audit

One look at everything a person or an agent touches in Ascribe, to find where it has stopped being one thing, and to say what to change. The reasons, and what "the surface" covers, are in the [proposal](../surface-audit.md). This is the plan for doing it.

It's done once. It finds and recommends; every change it leads to is separate work.

## Who does what

- **The maintainer walks the people's journeys:** the author's, the reviewer's, and the site builder's. Nobody else is set up to test yet. Each journey has a script and a notes page, so it can be done in one sitting and picked up by someone else later.
- **An agent does the rest:** builds the inventory, runs the agents' journeys in fresh sessions, goes through the questions, and writes the findings.

## Decisions

These are settled. Don't reopen them in a phase; if one can't be met, stop and report.

1. **Once.** No step is added to releasing. The inventory stays generated, so a later audit is cheap if one is wanted.
2. **It recommends and doesn't fix.** No product behavior changes in this plan. The one piece of code it adds is the script that generates the inventory, with its test.
3. **Nothing has to be kept.** Ascribe is before 1.0 and has no users. A finding may recommend renaming or removing anything, and says what breaks.
4. **Every finding has evidence:** a line of a journey's notes, or two entries of the inventory side by side. "This feels inconsistent" isn't one.
5. **A journey is walked as written, then noted as it went.** The walker doesn't fix what they find, look at the code, or read a plan to get unstuck. Getting stuck is the finding.
6. **Agents' journeys run in fresh sessions,** with nothing but the project and what Ascribe put in it, and the whole transcript is kept.
7. **The docs site is in scope twice:** as the place people find things, and as the first site built with Ascribe. What it had to build by hand is evidence about the product.
8. **The conventions come out early.** The [permalinks plan](../permalinks/README.md) adds the first new command group in its phase 4. Phase 1 here produces the first conventions for the command line, so that plan doesn't wait for the whole audit.

## The thread

This isn't a feature, so there's no layer to cross. The same idea applies to the method: take one task through every stage of the audit before doing any stage in full.

**The use case:** fixing a broken link, walked four ways (in the editor, at the command line, by an agent through the hook, by an agent through MCP), with findings written from it and one convention drawn from them.

**The stages it crosses:** a slice of the inventory, a journey's script, the walk and its notes, the seven questions, findings, a convention.

**What it tests** is whether the method produces findings worth acting on. The risks are in the joins: a script at the wrong grain, notes that don't support a finding, questions that turn up opinions instead of evidence, a journey that takes an afternoon. If any of those is wrong, it's cheaper to learn it from one task than from eight.

**Left out, and where it comes back:**

| Left out | Phase |
|---|---|
| The inventory beyond the command line and what one task touches | 2 |
| The other seven journeys | 3 |
| The docs site | 4 |
| The capability table, the full report, the issues | 5 |

## Phases

| Phase | Result |
|---|---|
| [1: One task, start to finish](phase-1-thread.md) | Findings from fixing a broken link four ways, the first command-line conventions, and a method that's been tried. |
| [2: The inventory](phase-2-inventory.md) | A generated map of the whole surface, kept current by a test. |
| [3: The journeys](phase-3-journeys.md) | Notes from the other seven tasks, walked by the maintainer and by agents. |
| [4: The docs site](phase-4-docs-site.md) | What the first Ascribe site had to build for itself, and how well the docs lead to each part of the surface. |
| [5: Findings](phase-5-findings.md) | The capability table, the report, the conventions, and the issues. |

Phase 1 first. Phases 2, 3, and 4 can run in any order after it, and at the same time. Phase 5 needs all of them.

## Where things go

Everything the audit writes is in this folder:

| File | Holds |
|---|---|
| `journeys/<name>.md` | A journey's script, then each walk's notes |
| `journeys/transcripts/` | The agents' sessions, as recorded |
| `findings.md` | The findings, numbered, in order of what they cost a user |
| `capabilities.md` | What can be done where |

The generated inventory is the exception. It outlives the audit, so it goes where the repository keeps generated maps, decided in phase 2.

## Rules for every phase

- Branch before committing; never commit to `main`.
- No product code changes. If a walk or a question turns up a bug, it's an issue, linked from the findings, not a fix in this plan.
- A finding is written when it's seen, in the notes, and moved to `findings.md` in phase 5. Phase 1 writes its own early.
- Quote, don't paraphrase: the command typed, the message shown, the title in the palette.
- Before finishing a phase, `pnpm format:check && pnpm lint && pnpm typecheck && pnpm test` pass. Phase 2 adds Rust or TypeScript and runs the full set in `AGENTS.md`.

## Not in this plan

- Making the changes the findings recommend.
- How Ascribe looks, how fast it is, how its code is arranged.
- Whether a feature should exist, except where it's plainly left over.
- Redesigning the language.
