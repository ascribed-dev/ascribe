# Phase 4: Page templates

Part of [Memos](README.md). Needs phase 3.

## Goal

A page started from a template has its instructions to the writer as memos, never published, and its placeholders as before.

```markdown
## Overview

This guide explains how to \{insert a brief description of the task}.

@memo: Optional: Specify when and why your user might want to perform the task.
```

The writer replaces the placeholder and reads the memo. If the memo is still there when the page is published, readers never see it.

## Context

- README decision 10, and phase 1's decision 2.
- `scripts/templates/convert.ts` and `rules.toml`: what the conversion changes, each rule named by the text it applies at. `convert.ts` rewrites the copies, and `--check` lists the stale ones. `scripts/templates/update.ts` moves the pin to a new release and makes `examples/templates` again with `ascribe templates eject`.
- `crates/ascribe-templates/set/`: the six templates as converted, `set.toml`, and the notes in `crates/ascribe-templates/notes/`.
- `crates/ascribe-check/src/checks/template.rs`: `template-placeholder-left`, the quoted-paragraph rule, and `template_places`.
- `crates/ascribe-cli/src/agents/prompts.rs`: the `new-page` prompt, which tells an agent to write over each placeholder and each quoted instruction.
- `docs/templates/` and `examples/templates/templates/`: projects in the repository that took the set, with their `templates.lock`.
- `docs/content/templates/index.md`, "What the check finds".

## Design

### The conversion

As phase 1 decides. With the recommended option, each template's table in `rules.toml` gains `memos`, its paragraphs that are instructions, each named by its text before any brace is escaped. The conversion writes each as a line memo in its place: `@memo: ` and their words, without the braces. A paragraph of more than one line of theirs is joined into one, since a line memo is one line; one too long to read on a line is a container instead.

Which paragraphs are instructions is a reading of their words, made once and recorded in the list. The pull request lists them, with the ones left as placeholders, so the maintainer reads both.

### The check

The quoted-paragraph rule goes, with its tests and its line in the templates guide. What the check reports of a memo is phase 1's decision. Placeholders are reported as now.

`template_places` gives the editor what's left to fill in. A memo isn't among them, so **Go to Next Placeholder** passes over it.

### The prompt

The `new-page` prompt stops telling an agent to write over each quoted instruction. It says the page's memos are the template's instructions, to follow and then delete or keep.

### The repository's own copies

`docs/templates/` and `examples/templates/templates/` are projects that took the set. Their templates are taken again, with `ascribe templates eject --keep-type` or by hand, so `ascribe templates status` reports no change in either.

## Tasks

1. `memos` in `rules.toml`, its handling in `convert.ts`, and the conversion's tests.
2. The set rewritten with `convert.ts`, and `set.toml` with it.
3. The check: the quoted-paragraph rule removed, and phase 1's decision on memos, with conformance cases.
4. The `new-page` prompt, and its test.
5. The repository's two copies.
6. The templates guide, the notes where they mention instructions, and `changes/`.

## Out of scope

- A project's own copies of a template. They're its own (page templates decision 18); `ascribe templates status` says the set has changed.
- Changing their words. A memo holds them as they are.

## Acceptance criteria

- `ascribe templates eject how-to` into an empty project writes a template whose instructions are memos and whose placeholders are as before.
- A page made from it with **Ascribe: New Page**, built with nothing filled in, has none of the instructions in any output, and still reports its placeholders.
- Every paragraph the six templates write as one placeholder is either in `memos` or listed in the pull request as a placeholder that stays.
- `ascribe templates status` on `docs` and on `examples/templates` reports nothing changed.
- `node scripts/compare/outputs.ts --base main` shows every project's outputs unchanged.

## Verify

```sh
node scripts/templates/convert.ts --check
cargo test -p ascribe-templates -p ascribe-check --locked
cargo run -p ascribe-cli -- templates status --config docs
node scripts/compare/outputs.ts --base main
```

## Commits

1. "Write a template's instructions to its writer as memos"
2. "Find a template's placeholders, and leave its memos to the writer"
