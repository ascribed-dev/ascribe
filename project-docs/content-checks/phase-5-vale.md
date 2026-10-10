# Phase 5: Prose, through Vale

Part of [Content checks](README.md). Requires phase 1. Rust, and a little of the extension.

## Goal

A project that uses [Vale](https://vale.sh) gets its alerts as Ascribe diagnostics: on prose only, with phrases resolved, at the right place in the source, in one Problems list and in `ascribe check`. A project that doesn't have a Vale configuration yet starts from a preset that's quiet by design, instead of from a style package that flags everything.

## Context

- The brainstorm's section 5: why Vale and not rules of our own, and what goes wrong when Vale reads Ascribe source directly.
- README decision 5 (run it, don't absorb it) and decision 7 (not on the keystroke path).
- Vale's documentation, checked against the current version: `--output=JSON`, reading from standard input with `--ext`, `.vale.ini`, vocabularies (`accept.txt`, `reject.txt`), and the alert fields (`Check`, `Message`, `Severity`, `Span`, `Line`, `Action`).
- `crates/ascribe-syntax` and `crates/ascribe-resolve`: which text is prose, and where phrases, includes, and snippets come from.
- `crates/ascribe-cli/src/commands/sources.rs`: the one place the binary runs another program (`git`) today, and how it reports that program missing or failing.
- `reports/Ascribe fit for docs pain points.md`, "Three cost classes": Vale across a whole project is in the slow class.
- Vale's style packages (Microsoft, Google, write-good, proselint, and others on its package hub): the rules a preset would choose from. Read each one's license before copying a rule; `scripts/release/notices.ts` is where third-party notices are gathered.
- `tests/corpora/README.md`: the three real documentation sets, fetched at pinned commits and converted to Ascribe, for measuring how noisy a rule is. Nothing from them is committed.

## Design

### What Vale is given

For each page, the prose as Markdown that Vale's own Markdown reader understands: paragraphs, headings, list items, and table cells, with phrases substituted, and with directive lines, attribute blocks, and frontmatter left out. Code stays as fenced code, which Vale already skips. Beside it, a map from each range of that text back to its range in the source.

- **Text from a phrase** maps back to the `{key}` in the page. An alert inside it is reported there, and says the text comes from the phrase.
- **Text from an include** isn't given with the page. The fragment is linted as its own file, once.
- **Source text, by default.** Variants and availability aren't resolved, so every arm is linted. Per-build linting is a later option.

### Running it

- `[checks.vale]` in `ascribe.toml` turns it on, with one of `preset` (below) or `config` (a path to the project's own `.vale.ini`), and optionally `command` (default `vale`). Naming both, or neither, is a model error that says to pick one.
- **In `ascribe check`:** only with `--vale`, or when `[checks.vale] in-check = true`. One Vale process for the project, not one per page.
- **In a hook:** the agents plan's hook checks the files it's given; Vale runs on those alone.
- **In the editor:** on open and on save, for that file, off the server's request path, so a slow Vale never delays a diagnostic or a completion. Results are dropped if the document changed since they were asked for.
- **Vale missing or failing** is one `advice` on `ascribe.toml` naming the command tried (decision 5), not a failed check.

### Reporting

- One registry entry, `prose`, whose message is Vale's, prefixed with its rule (`Microsoft.Contractions: Use 'it's' instead of 'it is'.`). The rule is in the JSON as its own field, so a tool can group by it.
- Vale's `suggestion`, `warning`, and `error` become `advice`, `warning`, and `error`. A project that doesn't want Vale's warnings to fail `--deny-warnings` sets `[checks.vale] max-level = "advice"`.
- Kind: `fix` when the alert carries a replacement Vale's `Action` gives, offered as a quick fix labeled `unsafe` (it changes what the page says); otherwise `write`. The prompt's evidence is the rule's name, its message, and its link when the style gives one.
- **Acknowledging** a prose alert is Vale's own business: its comments and `.vale.ini`. Ascribe passes Vale's inline comments through to it and adds nothing of its own, so a project has one way to quiet a rule.

### Presets

Most of Vale's reputation for noise comes from turning on a whole style package at once. A preset is the opposite: a short list of rules chosen because they're almost never wrong.

- **What a preset is.** A Vale configuration and its rule files, shipped inside Ascribe as text. Nothing is downloaded: no `vale sync`, no network. Ascribe writes them under the project's `.ascribe/` directory, which is already ignored, and runs Vale with that configuration.
- **One preset to start: `quiet`.** Rules with close to no false positives: a repeated word, a common misspelling, a term written two ways in one project, a missing or doubled space. Its alerts are all `advice`. A stricter preset is added when someone asks for it, not before.
- **How a rule gets in.** Run the candidate on `docs/`, `examples/`, and the converted corpora, and read what it finds. A rule that is wrong more than once in a hundred alerts on that sample is left out. The pull request lists each rule with its count and its misses.
- **Where the rules come from.** Vale's own rule types, configured here, or rules taken from an existing style package where its license allows, with the notice the license requires. Ascribe writes no rule language of its own (README decision 5); a preset is a selection.
- **Adjusting one.** `[checks.vale] off = ["<rule>"]` turns a preset's rule off. A project that wants more than that has outgrown the preset: an eject command (named by the command reference's conventions) writes the preset out as an ordinary `.vale.ini` and styles folder the project owns, and switches `[checks.vale]` to `config`.
- **A preset is public once released.** Its name can't change, and a rule added to it makes new findings appear on upgrade. They're `advice`, so nothing fails; say so in the changelog each time, under **Behavior change**.

### Vocabulary

Ascribe writes the project's own words (phrase values, glossary terms and aliases, dimension and feature labels) to a Vale vocabulary file, so they aren't spelling errors. It's a generated file: decide where it lives with Vale's rules for vocabularies in view, and never write into a file the project wrote.

## Tasks

1. Prose extraction with the map, tested on its own: every character of extracted prose maps to a source range, for pages with phrases, directives inside list items, tables, and non-ASCII text.
2. Running Vale and reading its JSON, with a fake `vale` in tests (as `packages/review/test/helpers/fake-gh.ts` fakes `gh`) and one test against a real Vale in CI.
3. `[checks.vale]`, `--vale`, and the missing-tool advice.
4. The `quiet` preset: the rules, the measurements that chose them, their license notices, and writing them under `.ascribe/`. The eject command.
5. The server's on-save run, with a scenario test that a slow Vale doesn't delay other requests.
6. The vocabulary file.
7. A guide page, `docs/content/guides/`: starting from the preset, bringing an existing Vale configuration (and what to remove from it: the ignore patterns for directives), and moving from the preset to a configuration of the project's own. `CHANGELOG.md`.

## Out of scope

Bundling or downloading Vale: the project installs it. A second preset. Per-build linting. A rule language of Ascribe's own.

## Acceptance criteria

- No alert is reported on a directive line, an attribute block, frontmatter, or code.
- An alert in a phrase's text lands on the `{key}`.
- With Vale not installed, `ascribe check` behaves as it does today plus one advice.
- With `preset = "quiet"` and Vale installed, a new project gets prose linting with no other file to write, and no network request.
- The `quiet` preset reports nothing wrong on `docs/` after this phase's fixes, and every rule in it met the one-in-a-hundred bar on the sample.
- The keystroke benchmark is unchanged with Vale on.

## Verify

```sh
cargo test --workspace --locked
cargo bench -p ascribe-lsp --bench keystroke
```

## Commits

1. "Extract a page's prose, with a map back to its source"
2. "Report Vale's alerts as diagnostics"
3. "Add the quiet preset"
4. "Lint on save in the editor"
5. "Give Vale the project's own words"
