# Phase 2: "This is intended"

Part of [Content checks](README.md). Requires phase 1. A language change, so it has a stop in the middle.

## Goal

An author can acknowledge one check in one place, with a reason, and it stays acknowledged until the thing it excused goes away. Without this, the advisory checks of phases 3, 4, and 7 become a list people learn to ignore.

## Context

- README decision 4: the properties any design must have.
- `SPEC.md`: the language. §3 (directive lines, attributes), §4 (the built-in directives), §2.2 and §7 (frontmatter and reserved keys), and the one-role rule for names in `docs/content/reference/content-model.md` §1.2.
- How the last two reserved names were added, and what each cost existing projects: the frontmatter key `formatted` (`model-field-reserved`) and the row attribute `available`. Both are in `CHANGELOG.md` under 0.2.0 as behavior changes.
- `crates/ascribe-syntax` (parsing), `crates/ascribe-check` (where diagnostics are made and could be filtered), `crates/ascribe-fmt` (canonical form), `crates/ascribe-emit` (what must not carry it).
- `project-docs/checklists.md`: the list for a change to the language.
- What other tools do, for the proposal: ESLint's `eslint-disable-next-line` with a required description, Rust's `#[expect(lint, reason = "…")]` (which warns when the lint no longer fires), Vale's `<!-- vale off -->`, markdownlint's comments.

## Design

### Part one: the proposal, then stop

Write a short proposal in the pull request description (not a file in the repository) comparing these, each shown on the same three cases (a whole page that's an intended orphan; one long table that makes a page oversized; one external link to a site that blocks checkers):

| Where it's written | Sketch | Reaches |
|---|---|---|
| A directive line | `@intended {check=page-size, reason="The full option table"}` above a block, or at the top of a page | A block or a page |
| A reserved frontmatter key | `intended:` with a list of `{ check, reason }` | A page only |
| `ascribe.toml` | `[[checks.intended]]` with `check`, `path`, `reason` | A page or a model entry; the only place for an unused phrase or feature |

For each: what it costs existing projects (a new reserved name can collide with a project's widget or field), whether `ascribe fmt` has a canonical form for it, whether the editor can write it as a quick fix, and how a stale one is found.

Recommend one or a pair. A pair is likely: something in the page for pages and blocks, and `ascribe.toml` for model entries, which have no page. Then **stop** for the maintainer's choice. Nothing below starts before it.

### Part two: build what was chosen

- **Matching.** An acknowledgement names one check's slug and applies to the block it's bound to, or the page, or the entry. A problem it matches is not reported; it's counted.
- **Only `review` checks.** Naming a check of another kind is an error that says why. Naming an unknown check is an error with a did-you-mean.
- **The reason is required** and not empty. A missing one is an error.
- **Stale ones are reported.** An acknowledgement that matched nothing in any build is an `advice`: "nothing here needs acknowledging any more", with a fix that removes it. It's `fix`, not `review`: it can't itself be acknowledged.
- **Never in output.** The site, plain, and JSON outputs are byte for byte what they'd be without it. A conformance case holds this for each output.
- **Counted and listed.** `check`'s summary says how many problems are acknowledged. The JSON lists them, each with its check, place, and reason, under a new key, so a report or a reviewer can read every excuse in the project.
- **The quick fix.** Every `review` diagnostic offers "Mark as intended…", which writes the acknowledgement in canonical form and leaves the cursor in the reason. It's labeled `unsafe`: it needs a person's reason. An agent's prompt for a `review` check says acknowledging is allowed only when the user said so.

## Tasks

1. The proposal, and the stop.
2. `SPEC.md`, the parser, the formatter's canonical form, and the directive or content-model reference, for what was chosen.
3. Matching, the two errors, and the stale advice, with conformance cases for each, including an acknowledgement inside a fragment and one on a page two builds treat differently.
4. The counts in `check`'s text and JSON; bless the schemas.
5. The quick fix in the server, with a scenario test.
6. `docs/content/`: where the syntax belongs, plus a short guide section on when acknowledging is the right answer and when fixing is. `CHANGELOG.md`, with any reserved name under **Behavior change**. The decision in `project-docs/decisions.md`.

## Out of scope

Acknowledging by folder, by pattern, or for the whole project (turn the check off in `[checks]` instead); acknowledging validity errors; any new check.

## Acceptance criteria

- An acknowledged problem isn't reported, is counted, and comes back the moment its acknowledgement is removed.
- Removing the cause makes the acknowledgement itself the report.
- No output changes because of an acknowledgement: the outputs comparison shows only `check`'s JSON gaining its new key.
- Until phases 3 and 4 add `review` checks, the tests use a test-only one.

## Verify

```sh
cargo test --workspace --locked
node scripts/compare/outputs.ts --base main
```

## Commits

1. "Specify how a check is acknowledged"
2. "Acknowledge a check in one place, with a reason"
3. "Report an acknowledgement nothing needs"
4. "Offer to mark a problem as intended"
