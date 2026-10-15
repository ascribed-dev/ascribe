# Phase 4: Page templates

Part of [Memos](README.md). Needs phase 3.

## Goal

A page started from a template has its instructions to the writer as memos, never published, and its placeholders as before.

```markdown
## Overview
@memo: Optional: Specify when and why your user might want to perform the task.

This guide explains how to \{insert a brief description of the task}.
```

The instruction was the last paragraph of "Overview", so its memo moves to the top of the section and is about the section: a memo is about the block it touches, and binding the heading below it is an error.

The writer replaces the placeholder and reads the memo. If the memo is still there when the page is published, readers never see it.

## Context

- README decisions 4, 10, and 22.
- SPEC §3.8: what a following-block directive binds, and that it can't bind a heading.
- `scripts/templates/convert.ts` and `rules.toml`: what the conversion changes, each rule named by the text it applies at. `convert.ts` rewrites the copies, and `--check` lists the stale ones. `scripts/templates/update.ts` moves the pin to a new release and makes `examples/templates` again with `ascribe templates eject`.
- `crates/ascribe-templates/set/`: the six templates as converted, `set.toml`, and the notes in `crates/ascribe-templates/notes/`.
- `crates/ascribe-check/src/checks/template.rs`: `template-placeholder-left`, the quoted-paragraph rule, and `template_places`.
- `crates/ascribe-cli/src/agents/prompts.rs`: the `new-page` prompt, which tells an agent to write over each placeholder and each quoted instruction.
- `docs/templates/` and `examples/templates/templates/`: projects in the repository that took the set, with their `templates.lock`.
- `docs/content/templates/index.md`, "What the check finds".

## Design

### The conversion

Each template's table in `rules.toml` gains `memos`, its paragraphs that are instructions, each named by its text before any brace is escaped (decision 22). The conversion writes each as a line memo, `@memo: ` and their words, without the braces. A paragraph of more than one line of theirs is joined into one, since a line memo is one line; one too long to read on a line is a container instead.

**Where the memo goes** depends on what follows the instruction in its container, since a memo is about the block it touches, and binding a heading is an error (SPEC §3.8):

1. **A block follows it:** the memo takes the paragraph's place and touches that block, the blank line between them removed. Above `@steps`, it stacks with it, and is about the list.
2. **It's the last block of a section:** the memo moves to the top of the section, under the heading and its `@id`, and is about the section. In the how-to, `\{Optional: Specify when and why…}` ends "Overview" and `\{This section is optional. Include a sub-task…}` ends the sub-task's section.
3. **It's the last block of a container with no heading,** such as a list item: it can't be a memo, since nothing in the container comes after it and there's no section to be about. The conversion stops, naming it, as a rule that doesn't find its text does, and it stays a placeholder. In the how-to, `\{Optional: The result of completing this step.}` is the last block of the first step; it reads as a slot for the step's result, like the two `Optional:` lines above it, more than as an instruction, which the reading below decides.

Which paragraphs are instructions is a reading of their words, made once and recorded in the list. The pull request lists them, with the ones left as placeholders, so the maintainer reads both.

### The check

The quoted-paragraph rule goes, with its tests and its line in the templates guide. The check reports nothing of a memo that has the template's words (decision 22). Placeholders are reported as now.

`template_places` gives the editor what's left to fill in. A memo isn't among them, so **Go to Next Placeholder** passes over it.

### The prompt

The `new-page` prompt stops telling an agent to write over each quoted instruction. It says the page's memos are the template's instructions, to follow and then delete or keep.

### The repository's own copies

`docs/templates/` and `examples/templates/templates/` are projects that took the set. Their templates are taken again, with `ascribe templates eject --keep-type` or by hand, so `ascribe templates status` reports no change in either.

## Tasks

1. `memos` in `rules.toml`, its handling in `convert.ts`, and the conversion's tests: an instruction before a block, before `@steps`, at the end of a section, and at the end of a list item, which stops the script.
2. The set rewritten with `convert.ts`, and `set.toml` with it.
3. The check: the quoted-paragraph rule removed, and a memo with the template's words not reported, with conformance cases.
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
- No memo in the set has nothing after it in its container, and `ascribe check` on a page made from each template reports no binding error.
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
