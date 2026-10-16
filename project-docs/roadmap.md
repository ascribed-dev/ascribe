# The plans, in order

Five plans have phase files: [versions](versions/README.md), [memos](memos/README.md), [the editor's views](editor-ui/README.md), [navigation](navigation/README.md), and [glossary pages](glossary-pages/README.md). This file says which phase runs when, what can run at the same time, and how each is run. It doesn't say what a phase builds: its phase file does, and it's the spec. Where this file and a phase file disagree about a phase's content, the phase file wins; where they disagree about order, stop and ask the maintainer.

Written 16 October 2026, with every plan's decisions answered.

## Where each plan stands

| Plan | Done | Left | Decisions |
|---|---|---|---|
| Versions | Phases 1 to 8, and 10 | Phase 9, issue #502 | All recorded |
| Memos | Phase 1 | Phases 2 to 8 | 1 to 26, in its README |
| The editor's views | Nothing | Phases 1 to 7 | 1 to 17, in its README; no decision phase |
| Navigation | Phase 1 | Phases 2 to 6 | 1 to 27, in its README |
| Glossary pages | Phase 1 | Phases 2 to 5 | 1 to 21, in its README |

A decision phase that's done has its answers in its plan's README and in its `phase-1-decisions.md` under each **Decided:**. None of the three has a pull request of its own: the answers are committed to `main` with this file.

## The order

Two tracks run side by side. **Track A** is the compiler: the language, the model, resolve, check, emit, the commands. **Track B** is the editor: the language server's requests and the extension. Within a track, one phase is in flight at a time, in the order below. Across tracks, two phases may be in flight at once, within the limits in "What can't overlap".

Short names: V for versions, M for memos, E for the editor's views, N for navigation, G for glossary pages, then the phase's number.

| Step | Track A | Track B | Why here |
|---|---|---|---|
| 1 | V9 | E1 | V9 finishes a plan and is already an issue. E1 is the editor's thread, and M6 needs it. |
| 2 | M2 | E2 | M2 is memos' thread, through the parser fork; best while no other phase touches the parser. E2 meets two seams nothing in the extension uses yet. |
| 3 | M3 | E3 | |
| 4 | M4 | E4 | M4 is what page templates have waited for. |
| 5 | M5 | | M5 is memos' second thread, memo files. Track B waits for it. |
| 6 | M7 | M6 | M6 needs M5 and E1, and puts memos in This page. M7's personal memos need M6: build M7's other parts first, or start it when M6 merges. |
| 7 | M8 | | Memos' docs. Track B waits for N3. |
| 8 | N2 | | Navigation's thread, from `ascribe.nav` to an Astro site's sidebar. |
| 9 | N3 | | N3 goes out in the same release as N2 (navigation README, "The thread"). |
| 10 | N4 | N5 | N5 needs N3, and changes Pages' structure before E5 adds to it. N4 changes `llms.txt` for every project. |
| 11 | N6 | E5 | N6 needs N4 and N5. E5 adds Pages' and Content model's actions onto navigation's sections. |
| 12 | G2 | E6 | G2 changes every project's terms at once; nothing else changing outputs is in flight. E6 removes the Projects view, once the other views' test APIs have settled. |
| 13 | G3 | E7 | |
| 14 | G5 | G4 | G4 needs G3, and its Content model entries build on E5. G5 needs G2 to G4: start it when G4 merges. |

The order inside each plan is its README's. Across plans, it's this:

- **Memos before navigation and glossary pages.** Memos is decided and unblocked, page templates wait on it, and its thread changes the parser, which neither of the others touches.
- **E1 before M6,** so memos land in This page and never in Used by.
- **N5 before E5,** so Pages' menus, filters, colors, and dimming are built once, on sections.
- **N4 and G2 apart,** each in a step of its own on track A, since each changes outputs across every project and each difference is accepted file by file.
- **Glossary pages last.** A glossary works today through `ascribe.toml`; this plan changes mostly how it's written, and little of what readers get. If the maintainer wants it sooner, it can swap with navigation as a block (N2 to N6 for G2 to G5), since neither needs the other.

### What each phase needs

From each phase file's "Needs", plus what this order adds:

| Phase | Needs | Added here |
|---|---|---|
| V9 | V7, V8 (merged) | |
| M2 | M1 (done) | |
| M3 | M2 | |
| M4 | M3 | |
| M5 | M1, M2 | M4, to keep track A in one line |
| M6 | M5 | E1 |
| M7 | M5; its personal memos M6 | |
| M8 | M2 to M7 | |
| E1 | | |
| E2 | E1 | |
| E3 | | E2, by risk |
| E4 | E1 | E3 |
| E5 | E4 | N5 |
| E6 | | E5, so the tests move once |
| E7 | | E6 |
| N2 | N1 (done) | M8 |
| N3 | N2 | |
| N4 | N3 | |
| N5 | N3 | M6 (both change the sidebar's code) |
| N6 | N4, N5 | |
| G2 | G1 (done) | N4, so the two output changes don't overlap |
| G3 | G2 | |
| G4 | G3 | E5 |
| G5 | G2 to G4 | |

## What can't overlap

Two phases on different tracks are in flight at once only when none of these is true:

- **Both change outputs.** A phase whose phase file names output changes to accept (M3, N2, N4, G2) runs while no other such phase is in flight, so `node scripts/compare/outputs.ts --base main` on each shows only its own.
- **Both add diagnostics.** `tests/conformance/diagnostics.toml` and `crates/ascribe-core/src/diagnostics.rs` are kept in the same order, and two branches adding to them conflict in both. M3, M5, N2, N3, N4, G2, and G3 add diagnostics; they're all on track A, which keeps them apart.
- **Both change the sidebar's code** (`packages/vscode/src/ui/sidebar.ts`, `sidebarViews.ts`) or `ascribe/inventory`: E1, E4, E5, E6, M6, N5, and G4, all on track B.

Some files every phase may touch, and a conflict in them is settled by regenerating, never by editing: `project-docs/surface.md` (`node scripts/surface/inventory.ts`), `docs/content/_generated/` (each generator's test with `ASCRIBE_BLESS=1`), the extension's `src/shapes.ts` (`ASCRIBE_BLESS=1 cargo test -p ascribe-cli shapes`), and `examples.observations/default.toml` (`cargo run -p ascribe-cli -- observe`). `SPEC.md` and the docs' pages conflict as text: merge `main` into the branch and resolve by reading both.

## How a phase is run

### Its issue

One issue per phase, on [the project board](https://github.com/orgs/ascribed-dev/projects/1), in the shape of #502:

- **Title:** the plan's name, a colon, and the phase's result in a few words: "Memos: a memo in a page is never published".
- **Body:** "Phase N of the [<Plan> plan](link to its README). The phase file is the spec: [phase-N-….md](link). Its **Acceptance criteria** are what done looks like, and its **Commits** list the commits of one pull request." Then **Needs:** with the issues of the phases it needs, from the table above.

An issue whose needs aren't all merged stays in Backlog. When they are, it's ready for `triage` to move to Ready. The [`board`](../.claude/skills/board/SKILL.md) skill says how to read and move cards from a session that can't reach GitHub Projects.

### Its pull request

- **One pull request per phase,** from its own branch, with the phase file's commits. Never a commit to `main`. The [`implement`](../.claude/skills/implement/SKILL.md) skill builds it, [`steward`](../.claude/skills/steward/SKILL.md) drives it to mergeable, and [`review`](../.claude/skills/review/SKILL.md) reviews, approves, and merges it.
- **Before review,** merge `main` into the branch. Never force-push or rebase a pushed branch: the maintainer's sessions refuse it.
- **The checks** are AGENTS.md's four commands, and the plan's "Rules for every phase". A phase whose file or plan says so runs the full matrix from its branch (Actions → CI → Run workflow): E6 and N2 by their files, N3 and N5 by navigation's rule on matching names against files, and any phase about paths, case, or links (AGENTS.md).
- **CI is reported** check by check, every one listed with its state, not as "passing".
- **A phase's docs** ship in its pull request: the pages its phase file names, marked `@available: next` where they're new, and an entry in `changes/`. A change to a published package's API or a behavior a project relies on says **Behavior change** in its entry. There's no compatibility work: Ascribe is before 1.0, and a project changes with it.

### Decisions

- **A plan's decisions stay in its README** while it runs, numbered within the plan. Don't reopen one in a phase.
- **A phase that changes a contract** (the language, an output, a command's JSON, a published API) adds a decision file to `project-docs/decisions/` in its pull request, named by its issue, as [the folder's README](decisions/README.md) says.
- **When a plan's last phase merges,** its decisions that still bind the work become files there, numbered by the plan's last issue, and the plan's folder stays as the record of what was intended.

### When to stop and ask

A phase stops and reports to the maintainer, instead of choosing, when:

- its phase file and the code disagree, or a decision can't be met;
- a thread's seam fails: E1's measurement shows `ascribe/page` slower than Used by's requests; M2 finds a reader of a page (the source index, a check, `ascribe refs`, the prose extraction) that sees a memo's contents; N2's navigation can't be joined to Astro's collection from what the output carries; G2's terms can't be read before resolution without a second parse, or the editor's keystroke budget is broken;
- an output changes that its phase file doesn't name;
- a phase needs another plan's file changed beyond what "The other plans" in its README says.

## Releases

The maintainer releases from `main` ([RELEASING.md](../RELEASING.md)); no phase does. Two things for whoever plans a release:

- **N2 and N3 go out together.** Between them, a misspelled name in `ascribe.nav` does nothing and says nothing.
- **N2, N4, and G2 each change what every project's outputs hold,** and E6 removes a view people know. Their `changes/` entries say so, and a release that carries one is worth a line in its notes.

## When this file changes

When a plan's phase merges, nothing here changes: the board says what's done. This file changes when the order does: a plan added, two swapped, or a phase split. Its "Where each plan stands" is as of the date above.
