# Phase 7: The report

Part of [Content checks](README.md). Requires phases 1, 4, and 6; uses phase 2's acknowledgements. Rust.

## Goal

`ascribe report`: one command for what's too slow, too networked, or too broad for `ascribe check`. It answers "what state is this project in?", and every finding in it has a next step, as in the editor.

## Context

- README decisions 5, 6, 8, and 10.
- `crates/ascribe-cli/src/commands/`: how a command is declared and documented, and `drift.rs`, the closest in shape (`--format summary` writes Markdown for a CI job's summary; `--exit-code` decides whether findings fail).
- `crates/ascribe-cli/src/shapes.rs`: a command's JSON gets a schema, TypeScript types, and a docs fragment.
- The agents plan's decision 3 (compact, versioned, capped output) and its prompt format.
- `reports/Ascribe fit for docs pain points.md`, "Three cost classes": external links take minutes and are flaky, so they belong on a schedule, not on a pull request.
- [lychee](https://lychee.cli.rs) and `afdocs`, each read at its current version before designing around its output.

## Design

```sh
ascribe report [SECTION…] [--format text|summary|json] [--build <NAME>] [--site <URL>]
```

With no section, it runs those that need nothing outside the project. Each section is independent, and one that can't run says why and doesn't stop the others.

| Section | What it reports | Needs |
|---|---|---|
| `problems` | `check`'s findings, counted by check, by page, and by kind of next step; and every acknowledgement with its reason | Nothing |
| `inventory` | Pages by type and by owner; overdue reviews; orphans and unused entries (phase 4's checks, as lists) | Nothing |
| `builds` | What each build leaves out that another keeps: pages, sections, variant arms | Nothing |
| `links` | External links that fail, redirect, or time out, each at its place in the source | A link checker, installed; the network |
| `agents` | The delivery spec's checks, each passing, failing, or skipped | `afdocs`, installed; `--site`, a built site's address |

Details:

- **`links`.** Ascribe lists the project's external links with their source places, hands the URLs to the checker, and maps results back (decision 5). A link that redirects permanently is `choose` (use the new address, as a fix); a dead one is `review`, with the status and the sentence around it as evidence. A site that blocks checkers is what acknowledgement is for, and `[checks.links] ignore` takes patterns for whole hosts. Results aren't cached in this phase.
- **`agents`.** A failing check Ascribe's output should have satisfied is reported as Ascribe's bug, with the issue tracker's address. A failing hosting check is `outside`: what to set, and where, for the hosts the docs name. Nothing here asks the author to edit a page.
- **Prompts in batches.** `--format prompt` on a section writes one prompt for its findings, capped, with the evidence each kind carries and the command that verifies the fix. It's built with the agents plan's prompt module.
- **Exit codes.** 0 when it ran; `--exit-code` makes findings at or above a level exit 1, so a scheduled job can open an issue. A section that couldn't run is never a pass: it's named in the output and, with `--exit-code`, is exit code 2.
- **`summary`** is Markdown for a CI job's summary or an issue body.

### In CI

Document one recipe, and use it on Ascribe's own docs: a scheduled workflow that runs `ascribe report links agents --site https://ascribed-dev.com --format summary --exit-code`, and opens or updates one issue when it fails. Not on pull requests.

## Tasks

1. The command and its three local sections, with the JSON's schema blessed.
2. `links`, with a fake checker in tests and one real run in a scheduled workflow.
3. `agents`, the same way.
4. `--format prompt`.
5. The scheduled workflow for the docs site, with a timeout on every job (`scripts/ci/workflows.test.ts` requires one).
6. `docs/content/reference/cli.md`, a guide page on reading the report and acting on it, `CHANGELOG.md`.

## Out of scope

A visual report or dashboard (decision 10); trends, which need stored history; fixing anything automatically.

## Acceptance criteria

- With no tools installed and no network, `ascribe report` runs its three local sections and says, for each of the other two, what's missing and how to get it.
- Every finding carries its kind of next step, and no `outside` finding suggests editing the source.
- The JSON is documented, versioned, and capped as the agents plan's lists are.

## Verify

```sh
cargo test --workspace --locked
target/debug/ascribe report --config docs
ASCRIBE_BLESS=1 cargo test -p ascribe-cli shapes && git diff --stat schemas
```

## Commits

1. "Add ascribe report: problems, inventory, and builds"
2. "Report external links"
3. "Report the delivery spec's checks on a built site"
4. "Write a report's findings as one prompt"
5. "Run the report on the docs site every week"
