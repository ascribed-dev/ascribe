# Phase 8: Ascribe's own docs

Part of [Memos](README.md). Needs phases 2 to 7.

## Goal

Ascribe's docs have a guide to memos and use them, and the landing page stops listing them as coming.

A writer new to Ascribe reads one page and knows when to write `@memo`, when to write a memo file, and when a pull request comment is the right place instead. An agent working on Ascribe's docs is told about the memos on the page it's changing, because the docs have some.

## Context

- Every earlier phase's changes to `docs/content/`: the directives reference, the memo file reference, the editor's page, the agents guide, and the diagnostics reference.
- `docs/content/writing/`: the feature folder a guide to memos belongs in, with its pages of every type (decision #452).
- `docs/content/templates/index.md`, which phase 4 changed.
- `site/home/content/index.md`: "Where it's going", which lists this feature as "Notes".
- `project-docs/proposals/memos.md` and `project-docs/brainstorm.md`, which say what's built.

## Design

### The guide

A how-to in `docs/content/writing/`, "Leave a memo for whoever edits next": the three kinds of commentary (README decision 1), each with when to use it; a memo in the page, in both forms; a memo file, with `about` and `quote`, and what outdated and detached mean; personal memos; and what agents are told. It links each reference rather than repeating it.

### The docs' own memos

Ascribe's docs gain memos where a reason is worth keeping, written by the maintainer or from what the docs' decisions already say: why getting started creates `ascribe.toml` with `ascribe init` before it shows the file, why the directives reference lists the built-ins in the SPEC's order, and similar. Each is a memo in the page, or a file in `docs/memos/` where it's longer. They show the feature in use, and phase 7's prompts and rules then have something to say about the docs.

### The landing page, the proposal, and the brainstorm

"Notes" leaves "Where it's going". The proposal says it's built and links the guide, and the brainstorm's list of what's built gains memos.

## Tasks

1. The guide, with `type: how-to`.
2. The docs' memos, in pages and in `docs/memos/`, read by the maintainer.
3. The landing page, the proposal, and the brainstorm.
4. An entry in `changes/`.

## Out of scope

- A tutorial. The guide and the references are enough for a feature this size.
- Converting every reason in `project-docs/` into a memo.

## Acceptance criteria

- `ascribe check --deny-warnings` passes on `docs` and `site/home`.
- `ascribe outline getting-started.md` in `docs` lists at least one memo.
- The guide links the directive's reference, the memo file reference, and the editor's page, and each of them links the guide.
- The landing page's "Where it's going" doesn't list memos.

## Verify

```sh
cargo run -p ascribe-cli -- check --deny-warnings --config docs
cargo run -p ascribe-cli -- check --deny-warnings --config site/home
cargo run -p ascribe-cli -- outline getting-started.md --config docs
```

## Commits

1. "Add a guide to memos"
2. "Keep the reasons behind Ascribe's docs in memos"
3. "Say memos are built"
