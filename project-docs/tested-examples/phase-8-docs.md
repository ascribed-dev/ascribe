# Phase 8: Ascribe's own docs

Part of [Tested examples](README.md). Needs phase 4.

## Goal

The docs' own examples say what they were tested with. A page that shows what `ascribe check` prints, taken from a file a test compares the command's real output with, reads "Tested with Rust" and the toolchain's version under it.

It's the feature's test on a project that already has hundreds of snippets, most of which aren't examples.

## Context

- `docs/ascribe.toml`: one source, `code`, whose `path` is the repository's root.
- What the docs' snippets show today, which is three kinds of file: test output that a Rust test compares with the command's own (`crates/ascribe-cli/tests/output/`), files from the example projects (`examples/`), and source and configuration from the repository itself.
- Phase 1's decision 5: a file is an example only when the config's `include` matches it.
- `AGENTS.md`: the checks every pull request runs, and the generated files re-blessed with `ASCRIBE_BLESS=1`.
- The repository's `.gitattributes`, which keeps every text file's line endings as LF.

## Design

### What counts as an example here

An `ascribe.examples.toml` at the repository's root, whose `include` is the files a test proves:

- `crates/ascribe-cli/tests/output/**`: what the commands print, each compared with the real output by a test;
- the example projects' pages and content models that tests build and check.

Source files and configuration the docs quote aren't examples: nothing tests that a quoted line of `Cargo.toml` is right, beyond its being the file. They stay as they are, with no line and no advice.

The first task is a count: how many of the docs' snippets fall in each kind, and which files `include` would match. It goes in the pull request, and the `include` list is written from it, not from a guess.

### The command

`test` is the part of the repository's own checks that proves those files, most likely `cargo test -p ascribe-cli --locked` with the conformance tests. It has to be the smallest command that does, since it runs before every commit that touches one of the files.

`[environment]` records the Rust toolchain.

### The cost, measured before it's taken on

README decision 8 says one failing test blocks every observation, and decision 10 that any edit to a file makes it stale. Here that means: every pull request that changes what a command prints must run `ascribe observe` and commit the result, as it re-blesses the output file today.

So this phase measures before it commits the repository to it:

- how long the `test` command takes;
- over the last hundred merged pull requests, how many touched a file `include` matches.

If the answer is "most pull requests, and minutes each", the phase stops at a narrower `include`, or at the example projects alone, and says so. The docs using the feature is worth less than every contributor paying for it on every change.

### Held in CI

CI runs `ascribe observe --check`, so a pull request that changes an example without observing again fails, with a message that says the one command to run. `AGENTS.md` and `CONTRIBUTING.md` gain the line, beside the one about blessing generated files.

`ascribe check` on `docs/` raises `example-stale` to an error, so the docs can't publish a tested line that's no longer true. `example-unobserved` stays advice.

### The site's own pages

`site/home` copies the docs' `[sources.code]`. Its claims show real output too, so its pages get the same line, from the same observation, with nothing more to configure.

## Tasks

1. Count the docs' snippets by kind, and measure the two costs. Put all of it in the pull request.
2. `ascribe.examples.toml` at the root, with the `include` the count supports.
3. The first observation, committed.
4. `ascribe observe --check` in CI, and the lines in `AGENTS.md` and `CONTRIBUTING.md`.
5. `example-stale` raised to an error for `docs/`.
6. Read the docs' pages with tested lines, in the pull request's preview, with the maintainer: a line under every output block may be too much, and a page can drop one by showing a file that isn't an example.
7. The guide's sentence that Ascribe's own docs do this, and an entry in `changes/`.

## Out of scope

- An observation under more than one toolchain.
- Testing what isn't tested today. A quoted source file doesn't gain a test here.

## Acceptance criteria

- The pull request has the count and both measurements, and says what `include` they led to.
- A docs page that shows a command's output has a tested line, and one that quotes a source file doesn't.
- Changing a command's printed text, re-blessing, and not observing fails CI with the message that names the command.
- The docs build with `example-stale` as an error.
- If the measured cost was too high, the phase says so, ships the narrower form, and the README's table of phases is corrected to match.

## Verify

```sh
ascribe observe --check
cargo run -p ascribe-cli -- check --deny-warnings --config docs
cargo run -p ascribe-cli -- report --config docs
```

## Commits

1. "Say which of the repository's files are tested examples"
2. "Observe the repository's examples, and check they stay observed"
3. "Show what the docs' examples were tested with"
