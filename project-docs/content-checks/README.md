# Content checks

Everything Ascribe can report about a project's content and its published site, beyond "this isn't valid Ascribe": prose style through Vale, external links, page size and structure for AI agents (the Web Documentation Delivery Spec), orphans and unused entries, overdue reviews. One project, because they share one question: when Ascribe tells an author about something that isn't an error, what does the author do next?

This plan comes from sections 5, 6, and 7 of the [brainstorm](../brainstorm.md).

## Who it's for

Authors and the people who run a docs project. An author wants to know, while writing, what a reader or an agent will trip on. A lead wants to know, on demand, what state the whole project is in. Neither wants a list they learn to ignore.

## What exists today

- **141 diagnostics** in `tests/conformance/diagnostics.toml`, each an `error` or a `warning`, each with a `fix` paragraph. Almost all say "this is invalid", and the next step is to correct it. `ascribe check` reports them for every build; the language server for the editor's build.
- **Internal links, heading ids, includes, images, and snippets** are checked. `ascribe drift` reports code examples that changed while their page didn't.
- **Nothing can be acknowledged.** There's no way to say "this is intended" about one problem in one place, and no level below `warning`. `--deny-warnings` fails on every warning.
- **The `plain` output** is resolved CommonMark for each page and build, which is most of what the delivery spec asks a site to serve. Nothing publishes it, and there's no `llms.txt`.
- **The [agents plan](../agents/README.md) isn't built.** Its phase 1 adds `help`, a docs link, and a `safe` or `unsafe` label on each fix to `check`'s JSON; its phase 5 adds **Prompt agent**. This plan needs both.

## Decisions

These are settled. Don't reopen them in a phase; if one can't be met, stop and report.

1. **No check ships without its next step.** Every check declares one kind of next step (decision 2), a `fix` paragraph, a page in the diagnostics reference, and what its agent prompt carries. A test fails for a check that lacks any of them. This holds for the 141 that exist as well as the new ones.
2. **Five kinds of next step.** `fix`: Ascribe can make the edit. `choose`: the author picks among options Ascribe lists. `write`: it needs writing or judgment, so the offer is a prompt for the author's agent and the docs page. `outside`: nothing in the source can fix it (hosting, another team's site), so the report says what to set and where. `review`: it may be fine as it is, so the offer includes acknowledging it.
3. **A third level, `advice`.** New checks about quality arrive as `advice`: shown, never failing `ascribe check`, with or without `--deny-warnings`. A project raises a check to `warning` or `error`, or turns it off, in `ascribe.toml`. Errors about validity can't be lowered or turned off.
4. **"This is intended" is part of the language.** An author can acknowledge one check in one place, with a reason, in the source. It has these properties, whatever its spelling (phase 2 proposes that, and the maintainer chooses):
   - the reason is required;
   - it names one check and applies to one page, block, or entry, never a folder or the project;
   - it never reaches any output;
   - one that no longer matches a problem is itself reported, so acknowledgements don't outlive what they excused;
   - `ascribe check` says how many problems are acknowledged, and the report lists them with their reasons;
   - only checks of kind `review` can be acknowledged. A validity error can't.
5. **Ascribe runs other tools; it doesn't absorb them.** Vale, an external link checker, and `afdocs` are run as programs the project installs, and their results come back as Ascribe diagnostics at the right place in the source. The binary gains no HTTP client and no rule language. A tool that isn't installed is one `advice` saying so, not a failure. Ascribe doesn't bundle or download them. It does ship Vale configuration, as presets.
6. **A check runs at the earliest stage it can afford, and every later one.** Three stages: while writing (the language server), before a commit and in CI (`ascribe check`), and on demand (`ascribe report`). A check's stage is recorded with it. Nothing that needs the network or a built site runs in the first two.
7. **The language server's speed is not spent.** A check in the first stage answers from the snapshot and adds no more than 1 ms to the keystroke benchmark at 3,000 pages (`crates/ascribe-lsp/benches/keystroke.rs`). Anything slower runs on save, or moves a stage later.
8. **Prompts carry evidence.** A prompt for a `write` check includes what an agent can't cheaply find: the sizes of a long page's sections, the status and the sentence around a dead link, the pages that mention an orphan's topic. It also names the command that verifies the fix. Report results are prompted in batches ("fix these 14 links"), capped as the agents plan's lists are.
9. **Delivery belongs to the agents plan.** How a prompt is copied or opened in chat, `--format concise`, and the hook are that plan's. This plan supplies each check's fields and evidence, and builds nothing of its own to deliver them.
10. **No dashboard, yet.** The report is a command with text, Markdown, and JSON forms. A visual report waits until the editor UI has shipped.

## The inventory

Each check, the earliest stage it runs at, and its kind. "New" checks are this plan's; the rest exist and gain a kind.

### While writing

| Check | Kind | Phase |
|---|---|---|
| Syntax, directives, attributes, frontmatter, the content model (exist) | `fix` or `choose`, each classified | 1 |
| Internal links, ids, includes, images, snippets (exist) | `choose` | 1 |
| A page's Markdown over the delivery spec's size limit, in the editor's build | `write` | 3 |
| A page with no description, which `llms.txt` needs | `write` | 3 |
| Heading levels that skip; more than one top-level heading | `fix` | 3 |
| A code fence with no language | `choose` | 3 |
| A review date that has passed | `review` | 3 |
| Prose rules, through Vale, on save and open | `fix` where Vale suggests one, else `write` | 5 |

### Before a commit and in CI

| Check | Kind | Phase |
|---|---|---|
| Everything above, in every build | | |
| Orphan pages; unused fragments, phrases, glossary terms, features, images | `review` | 4 |
| Two pages with one title; a description too long for `llms.txt` | `write` | 4 |
| Image files over a size | `review` | 4 |
| Availability left behind: a feature still marked unreleased after its release | `fix` | 4 |
| Vale across the project, opt-in; changed files only in a hook | as above | 5 |
| `llms.txt`, Markdown pages, and the pointer on each page | true by construction | 6 |

### On demand

| Check | Kind | Phase |
|---|---|---|
| External links | `choose` when the target moved, else `review` | 7 |
| The delivery spec's checks on the built site, with `afdocs` | `outside` for hosting, else a bug in Ascribe | 7 |
| The content inventory: by type and owner, overdue, orphans, unused | a summary | 7 |
| What one build leaves out that another keeps | a summary | 7 |

## Phases

| Phase | Result |
|---|---|
| [1: The contract](phase-1-contract.md) | Every diagnostic has a kind of next step, and the registry, the JSON, and the docs carry it. The `advice` level, and `[checks]` in `ascribe.toml`. |
| [2: "This is intended"](phase-2-acknowledge.md) | The language for acknowledging a check, chosen by the maintainer, then built. |
| [3: Checks while writing](phase-3-page-checks.md) | Page size, descriptions, heading levels, code fence languages, review dates. |
| [4: Checks across the project](phase-4-project-checks.md) | Orphans, unused entries, duplicate titles, large images, availability left behind. |
| [5: Prose, through Vale](phase-5-vale.md) | Vale's alerts as Ascribe diagnostics, on prose only, with phrases resolved, and a quiet preset to start from. |
| [6: Output agents can read](phase-6-agent-output.md) | `llms.txt`, a Markdown version of each page, and the pointer, emitted with each build and tested with `afdocs`. |
| [7: The report](phase-7-report.md) | `ascribe report`: external links, the delivery spec on a built site, the inventory, and build differences. |

Phase 1 comes first. Phases 2 to 6 need phase 1 and can run in any order, except that phases 3, 4, and 7 offer acknowledgement only once phase 2 has landed. Phase 7 needs phases 4 and 6.

## The other plans

- **Agents.** This plan's phase 1 needs the agents plan's phase 1 (`help`, `docs`, and `applicability` in `check`'s JSON, and `fix` read from the registry). Whichever starts first builds those as that phase file describes, and the other reuses them. Prompts are delivered by the agents plan's phase 5 (decision 9); if it hasn't landed, this plan's checks still carry their evidence in the JSON, and gain the action when it does.
- **Editor UI.** Its phase 7 computes use counts for its Content model view (`ascribe/inventory`), and the agents plan adds `ascribe refs`. Phase 4 here needs the same search. All three share one implementation, in a crate below the language server, as the editor UI plan's README says.
- **Features this plan waits for, and doesn't build:** navigation in the content model (for "a page in no group"), redirects (for "a moved page with no redirect"), and translation status. Each brings its own checks when it's built, under decisions 1 to 4.

## Rules for every phase

- Branch before committing; never commit to `main`.
- Read the current code before the phase file's pointers. If the phase file and the code disagree, or a decision above can't be met, stop and report instead of choosing silently.
- **A new diagnostic** is added to `tests/conformance/diagnostics.toml` and `crates/ascribe-core/src/diagnostics.rs`, in the same order, with a conformance case in `tests/conformance/cases/`, and with every field decision 1 requires.
- **The language and the contracts change only by a recorded decision.** Phases 1, 2, and 6 each change one (`ascribe.toml`, the language, the outputs), and say how. Go through the matching list in `project-docs/checklists.md` first.
- **`check`'s JSON is a contract.** A new field is an addition; no field's meaning changes without a new `schema_version`.
- **Project files are read through `ascribe_resolve::FileSystem`.** Libraries don't print, and don't panic on input.
- **User-visible changes** update the page under `docs/content/` and the unreleased section of `CHANGELOG.md` in the same phase.
- No phase history in code or docs. Describe what the code does now.
- Before finishing a phase, all of these pass:

  ```sh
  cargo fmt --all --check
  cargo clippy --workspace --all-targets --locked -- -D warnings
  cargo test --workspace --locked
  pnpm format:check && pnpm lint && pnpm typecheck && pnpm test
  ```

## Later, not in this plan

- A visual report or dashboard (decision 10), and trends over time, which need stored history.
- Rendered accessibility checks on a built site (contrast, landmarks), which need a browser.
- Running code examples.
- Checks Ascribe would have to guess at. A check is in scope only when its answer is the same on every run.
