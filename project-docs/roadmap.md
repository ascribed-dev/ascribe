# The editor's next views: the plan

Three proposals, built as one plan: [This page as a webview](proposals/this-page-webview.md), [the Project view](proposals/project-view.md), and [Project Settings](proposals/project-settings.md). This file splits them into phases of one pull request each, says which needs which, what can run at the same time, and how an agent fleet takes each from an issue to a merge.

Written 20 October 2026.

**The proposals are the spec for what's built.** Each phase below names the proposal sections it builds and cuts their scope; it doesn't repeat them. Where a phase entry and a proposal disagree about what's in a phase, this file wins; where they disagree about how something works, the proposal wins; where either disagrees with the code, stop and ask the maintainer.

## Decisions

The proposals left these open. The maintainer answered all eight on 20 October 2026, and each proposal's Settled section records its own. An issue quotes the decisions its phase reads, so triage sees they're made.

1. **The icon font.** The webviews draw codicons with the `@vscode/codicons` font, a new dependency, loaded from the extension's files. **Decided:** the font. The decision file says why decision 48 ("no web font anywhere"), which is about a reader downloading a font for text, doesn't reach an icon font the extension ships, as decision 50 already has it ship one. *Read by:* T1.
2. **How far Apply Every Safe Fix reaches.** **Decided:** the editor's build, as one workspace edit, its caption saying so; every build's fixes stay `ascribe check --fix`'s, as decision #544e has it. *Read by:* V3.
3. **Where Switch Project lives.** **Decided:** the Project view's title bar alone; Pages and Content model lose it. *Read by:* V5.
4. **A badge on the Project view.** **Decided:** none. *Read by:* V5.
5. **What the status bar menu keeps.** **Decided:** everything it has, each entry running the same command the view runs. *Read by:* V5.
6. **Where a setting's description lives.** **Decided:** every key of `ascribe.toml` has one home, a registry, `crates/ascribe-model/settings.toml`, which the binary embeds: the loader reads its keys and defaults from it, the reference's tables are generated from it, and `ascribe/settings` answers from it (Project Settings, "The registry"). A check's description is its `when` in `tests/conformance/diagnostics.toml`. *Read by:* P2, P3, P4.
7. **Which checks Project Settings lists.** **Decided:** all forty-seven configurable checks, the ones the file sets first. *Read by:* S2.
8. **Reset to default and the comment above a line.** **Decided:** a comment that touches the line goes with it; one after a blank line stays. *Read by:* S2.

## The phases

Four tracks. Each track runs one phase at a time, in its order; across tracks, phases run at once within "What can't overlap". A phase's ID is its track's letter and its place: P for preparation, T for This page, V for the Project view, S for Project Settings.

| Step | Track P: preparation | Track T: This page | Track V: Project view | Track S: Project Settings |
|---|---|---|---|---|
| 1 | P1, P2, P3 | | | |
| 2 | P4 | T1 | | |
| 3 | | T2 | V1 | S1 |
| 4 | | T3 | V2 | S2 |
| 5 | | | V3 | S3 |
| 6 | | | V4 | |
| 7 | | | V5 | |

Track P's first three phases have no needs and touch different files, so they run at once; P4 follows P3.

**Why this order.** This page goes first because its thread builds what the other two views stand on: the webview base, its handshake, its stylesheet rules, and the icons. The preparation phases change nothing a reader sees, and the registry they make is what Project Settings reads, so they start at once and finish before Project Settings begins. Each proposal's own order, by risk, holds within its track.

### What each phase needs

| Phase | Needs | Why |
|---|---|---|
| P1, P2, P3 | Nothing | |
| P4 | P3 | The registry |
| T1 | P1 | Both change `packages/vscode/src/ui/sidebar.ts`; P1 is one line, and goes first |
| T2 | T1 | |
| T3 | T2 | |
| V1 | T1 | The webview base |
| V2 | V1 | |
| V3 | V2 | Keeps track V one at a time |
| V4 | V1 | |
| V5 | V4, S1 | Its gear opens Project Settings, which S1 adds |
| S1 | T1, P4 | The webview base, and the registry with its descriptions |
| S2 | S1, P2 | Every check's description |
| S3 | S2 | |

### Preparation

**P1. Pages starts with every section closed.** This page proposal, "Alongside". `pagesLook`'s `expanded` is false for every section; opening a page still reveals it. *Done when:* Pages opens with its sections closed, the open page's path is revealed, the unit test says so, and `changes/editor/` and `docs/content/editor/sidebar.md` say so.

**P2. A `when` for every configurable check.** Project Settings, "The registry", by decision 6. The nine configurable checks in `tests/conformance/diagnostics.toml` without a `when` gain one, in the words of the other thirty-eight; the diagnostics reference is regenerated from the file. *Done when:* every entry with `configurable = true` has a `when`, and a test fails on one without.

**P3. The registry, and the loader reading it.** Project Settings, "The registry", by decision 6. `crates/ascribe-model/settings.toml` holds every key of `ascribe.toml`, each with its table, key, kind, choices, default, description as the reference writes it, and availability, in the reference's order; the binary embeds it; the loader reads its keys and defaults from it, in place of the lists in `check_keys` and the defaults in each key's code. A clean-up: labeled `optimization`. *Done when:* `node scripts/compare/outputs.ts --base main` shows every output and diagnostic the same, and a test fails when the loader accepts a key the registry doesn't have, or the registry has one the loader doesn't read.

**P4. The reference's tables, generated from the registry.** Project Settings, "The registry". Each `Key | Type | Default | Description` table in `docs/content/content-model/` and `docs/content/_fragments/content-model/` becomes a generated fragment in `docs/content/_generated/`, included where the table was, rewritten by its test with `ASCRIBE_BLESS=1`; the prose around each stays written. A clean-up: labeled `optimization`. The contract pages include the same fragments, so the generated tables reproduce today's text byte for byte. *Done when:* the built docs are the same, by `node scripts/compare/outputs.ts --base main`; editing a description in the registry without regenerating fails a test; and no `Key | Type | Default | Description` table is left written by hand. *Stop and ask* if a table can't be reproduced as it is.

### Track T: This page

**T1. This page is a webview.** This page proposal: "What it looks like", "What the tree gave" (points 1 to 4, 6 to 10, and 12), "What a webview changes" (the states aside), "Building it", and "Which page it shows"; the thread. The view draws every section the model builds, with rows wrapped and only About open, each row opening what it opens today. Built here, for all three views: the webview base in `packages/vscode/src/webview/`, the message protocol with its rising numbers, the ready handshake, the content security policy, the stylesheet with `--vscode-*` variables only and its entry in `design/tokens.toml`'s `[emit]`, the icons by decision 1, and the drawn message the tests wait for. The decision file that follows #544a, and decision 1's. *Done when:* the thread's use case passes as an integration test, with the focus rule's test and the handshake's; the measurement of point 10 is in the pull request, with the tree's beside it; and the keyboard and screen reader behavior of points 1 and 2 is there from the start, so no release has a This page worse to use than the tree. *Stop and ask* if the measured delay is one a person sees.

**T2. Includes open beside, and the view keeps its place.** This page proposal: "Includes open beside the page", "An update keeps the reader's place", and point 1's Up and Down. *Done when:* an include, a snippet, and an example open beside the page with the view still on it, and a cursor move keeps the view's scroll and the keyboard's focus, in integration tests.

**T3. The head, the states, and the badge.** This page proposal: the head with its formatted title (`ascribe/page` gains the title's pieces, through `shapes.rs`), "Every state is the webview's to draw", rows that lead with their state, sticky headers (point 11), and the badge. *Done when:* each state has a test, the badge counts Health's things to do, and the docs page, the walkthrough, and the extension's README describe the view as it is.

### Track V: the Project view

**V1. The Project view counts every build's problems, honestly.** Project view proposal: "What it looks like" (the head's name, state, and version only), "When each count is from" (Check Every Build's rule), and the thread. The view is first in the sidebar. The decision file for the views' order. *Done when:* the thread's use case passes as an integration test: unknown, then current after Check Every Build, then out of date after an edit. *Stop and ask* if `isCurrent` misses a change that matters.

**V2. Tasks.** Project view proposal: "Writing outputs and running programs is a task", and "Observe Examples runs commands the repository names". The `ascribe` task type, with Build, Sync Agent Files, and Observe Examples, in the view and in **Run Task**. The decision file for the task type's properties, which `tasks.json` names. Runs the full matrix from its branch (Actions → CI → Run workflow): it's about paths, a binary, and `--config` on every platform. *Done when:* each task runs the server's binary in the project's folder, a project that isn't the workspace folder gets `--config`, Observe Examples shows its commands and asks, and none of them is offered in Restricted Mode.

**V3. Every safe fix and every page formatted, as one edit.** Project view proposal: "Edits to pages go through the language server", by decision 2. The language server makes each edit for the whole project, and the extension asks with the count of fixes and files before applying it. *Done when:* both are one undoable workspace edit over unsaved text, the question names the counts, and the docs say the files are left unsaved on purpose.

**V4. The rest of the counts.** Project view proposal: the Health table's other rows, and the rest of "When each count is from": the day rule for reviews overdue, `ascribe/inventory`'s counts, and the agent files' count rerun after a save. *Done when:* each count opens its list, each goes out of date by its rule, in tests, and out of date and zero look different.

**V5. The head, About, and Restricted Mode.** Project view proposal: the head's server buttons, its build switch (the `ChosenBuilds` choice), and its two settings buttons, the gear opening Project Settings; About; Restricted Mode; Start Review and Prompt Agent; and decisions 3, 4, and 5. *Done when:* switching the build in the view moves the status bar and the preview, Switch Project is in the view's title bar and not in Pages' or Content model's, Restricted Mode turns every action off, and the docs describe the view as it is.

### Track S: Project Settings

**S1. Project Settings, through the round trip.** Project Settings proposal: "What it looks like" for two settings, "What it edits" (setting a value the file has), "The request", "The webview", and the thread. `ascribe/settings`, answering from the registry, and the edit action; the panel, opened from the Command Palette. The decision file for the request and the rule the tab keeps. *Done when:* the thread's use case passes as an integration test, with the comments of `ascribe.toml` as they were, and Undo followed. *Stop and ask* if the tab and an open `ascribe.toml` can fall out of step.

**S2. Every simple setting, and the checks.** Project Settings proposal: every simple setting, a key the file doesn't set added to its table, Reset to default by decision 8 (`remove_entry` learns the comment rule), and the checks list with its levels and limits, by decision 7. *Done when:* each kind of setting is set, added, and reset in tests, and the checks list shows every configurable check with its `when`.

**S3. Finding settings, and opening the tab.** Project Settings proposal: search and the filter, the cards and their links, the button in `ascribe.toml`'s title bar (only on a project the extension found), one tab per project and a deleted `ascribe.toml`, and the docs. *Done when:* search finds a word in any sentence of a description, the button is offered on `docs/ascribe.toml` and not on a fixture's, and the docs describe the tab.

## What can't overlap

Two phases on different tracks run at once only when none of these is true:

- **Both change the webview base** (`packages/vscode/src/webview/`'s shared files, `esbuild.mjs`'s webview entries, or the stylesheet's `[emit]` entry). T1 makes it; a later phase that has to change it says so in its pull request, and runs alone among the three extension tracks while it does.
- **Both change `packages/vscode/src/ui/sidebarViews.ts` or the views in `package.json`** in the same lines. T2, T3, V1, and V5 touch them. They may be in flight at once, since each adds to them rather than rewriting them; the second to open its pull request merges `main` first and resolves by reading both. A phase that has to restructure either file says so in its pull request, and runs alone among tracks T and V while it does.
- **Both change the loader or the registry** (`crates/ascribe-model/src/loader.rs`, `crates/ascribe-model/settings.toml`): P3, then P4.
- **Both change the content model reference's pages** (`docs/content/content-model/`, `docs/content/_fragments/content-model/`) while P4 is in flight. P4 moves every table there, so nothing else edits those pages until it merges.

Some files every phase may touch, and a conflict in them is settled by regenerating, never by editing: `project-docs/surface.md` (`node scripts/surface/inventory.ts`), `docs/content/_generated/` (each generator's test with `ASCRIBE_BLESS=1`), the extension's `src/shapes.ts` (`ASCRIBE_BLESS=1 cargo test -p ascribe-cli shapes`), the extension's docs tables (`ASCRIBE_BLESS=1 pnpm --filter ascribe-vscode exec vitest run test/unit/docs.test.ts`), and `examples.observations/default.toml` (`cargo run -p ascribe-cli -- observe`). `package.json`'s `contributes` conflicts as text: merge `main` into the branch and keep both sides' entries.

No phase changes an output, and none adds a diagnostic. A phase that finds it must stops and asks.

## How a phase is run

### Its issue

The maintainer, or a session the maintainer starts for it, files every phase's issue at once, so the dependencies can name issue numbers. Triage doesn't file issues: it moves an issue from Backlog to Ready when it's one pull request, says what done looks like, waits on nothing open, and asks no decision only the maintainer can make ([the triage skill](../.claude/skills/triage/SKILL.md)). Each issue is written to pass those four:

- **Title:** the phase's name, as above, after its area: "This page: includes open beside, and the view keeps its place".
- **Body:**
  - "Phase T2 of the [editor's next views](link to this file). The spec is this phase's entry there and the proposal sections it names. **Done when** below is what done looks like, and it's one pull request."
  - **Done when:** the phase's line, copied.
  - **Decided:** each decision the phase reads, quoted with its answer, so no question is left for the maintainer. T1 quotes decision 1, which approves the new dependency.
  - **Needs:** the issues of the phases it needs, from the table above.
- **Labels:** `editor-webviews` on every phase's issue, so the plan's issues can be listed together on the board; and `optimization` on P3 and P4 too.

An issue whose needs aren't all merged stays in Backlog, and triage leaves it there: "waiting on another open issue" is one of its four. When they merge, it's ready.

### One issue a phase, not smaller

Each phase is one pull request, and its issue is that pull request: the test triage applies first. Smaller issues would split a phase's slice across pull requests that each deliver nothing a person can use, which the steel-threads skill counts as a layer, not a slice, and they would put two implementors in the same files. A phase that turns out bigger than one pull request is split by its implementor, as [the implement skill](../.claude/skills/implement/SKILL.md) says: ship the first useful part, and open an issue for the rest, in Backlog, naming this phase.

### Its pull request

- **One pull request a phase,** from its own branch. Never a commit to `main`. `implement` builds it, `steward` drives it to mergeable, and `review` reviews, approves, and merges it.
- **Before review,** merge `main` into the branch. Never force-push or rebase a pushed branch.
- **The checks** are AGENTS.md's four commands. V2 runs the full matrix from its branch. P3 and P4 run `node scripts/compare/outputs.ts --base main`, and CI runs it with the `optimization` label.
- **The extension's views have tests** through its test API, unit tests where the logic is the extension's, and integration tests where VS Code's behavior is; the integration tests run in CI only. A webview's test waits for its drawn message, as T1 sets up.
- **CI is reported** check by check, each with its state.
- **Its docs ship in it:** the pages its phase names, marked `@available: next` where they're new, the extension's README and walkthrough where they describe what changed, and an entry in `changes/editor/` (or `changes/language-server/`). P3 and P4 change nothing a reader sees, and add no entry.
- **A decision file,** where its phase names one, in `project-docs/decisions/`, named by its issue.

### When to stop and ask

A phase stops and reports to the maintainer, instead of choosing, when:

- its entry, a proposal, and the code disagree, or a decision above can't be met;
- a seam fails: T1's focus rule or handshake, or a visible delay in T1's measurement; V1's `isCurrent` missing a change; V2's task unable to run the server's binary on a platform; S1's tab falling out of step with `ascribe.toml`, or an edit losing a comment;
- P4 can't reproduce a table as it is;
- an output or a diagnostic changes;
- it needs a dependency the decisions above don't name.

## Releases

The maintainer releases from `main` ([RELEASING.md](../RELEASING.md)); no phase does. Every phase leaves the extension as usable as before it, so none has to wait for another to go out. For whoever writes a release's notes: T1 replaces a view people know, and V1 adds the first view of the sidebar, so a release that carries either is worth a line.

## When this file changes

When a phase merges, nothing here changes: the board says what's done. This file changes when the order changes, or when a phase is split.
