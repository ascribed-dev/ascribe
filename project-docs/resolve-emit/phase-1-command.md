# Phase 1: The command

Part of [A page in the site output](README.md). Needs nothing.

## Goal

`ascribe resolve PAGE --emit site` writes the page as the site output does, and its JSON says which output it is.

From `examples/quill`:

````console
$ ascribe resolve install-agent.md --build site --emit site
…
## Connect to Quill <ascribe-attributes id="connect"></ascribe-attributes>

<ascribe-tabs sync="deployment">

<ascribe-tab value="cloud" label="Quill Cloud">
…
$ ascribe resolve install-agent.md --build site --emit site --format json | jq .emit
"site"
$ ascribe resolve install-agent.md --build site --emit site,plain
error: invalid value 'site,plain' for '--emit <OUTPUT>' [possible values: plain, site]
$ ascribe resolve go:connect --emit site
error: a permalink's help has one form, help.json's; leave out --emit
````

Each error exits `2`. The exact text of each is the implementer's, in the command's voice.

## Context

- README decisions 1 to 4; the proposal's "The command", "The JSON", and "Elements without their script".
- `crates/ascribe-cli/src/commands/resolve.rs` and `crates/ascribe-query/src/resolve.rs`.
- `crates/ascribe-lsp/src/preview.rs` around `emit_page`, for the steps; `crates/ascribe-emit/src/emitter.rs`, `emit_page`, and what it returns (the page's file text, and its frontmatter).
- `crates/ascribe-cli/tests/output/resolve.*`, and how the output tests are blessed.
- `crates/ascribe-cli/src/shapes.rs` for `resolve.schema.json`.

## Design

- **`ascribe_query::resolve`** takes an `Emit` (`Plain`, `Site`). `Plain` is today's code. `Site` resolves the page for the build with the build's router and writes it through `emit_page` with `SiteEmitter::new(model)`, without anchors. `emit_page`'s text is the site output's file, its frontmatter first; with `frontmatter` it's written whole, and without it the frontmatter block is left off and the body written alone. The assets `emit_page` returns are only named, never written. A page the build doesn't publish answers as today, with `not_published` and an empty `text`.
- **`Resolved`** gains `emit`, an enum serialized as `"plain"` or `"site"`.
- **The command:** `--emit <OUTPUT>`, `Option<Emit>` with clap's value enum, help "Which output to write the page as, one of `plain` (the default) or `site`, as `ascribe build --emit` names them. One output, not a list." `go:<name>` with `--emit` given is a usage error, exit `2`.

## Tasks

1. `Emit` and the site form in `ascribe-query`, with unit tests against `emit_page`'s own output for the same page, so the two can't drift.
2. `--emit` on the command, and the two refusals.
3. Output tests in `crates/ascribe-cli/tests/output/`: `resolve-site.txt` (quill's `install-agent.md`: tabs, a note, steps), `resolve-site-frontmatter.txt` (the site output's keys, `type`, `formatted`, `available`), `resolve-site.json`, a glossary link with its definition as `title`, an include, a line of a versioned build with its links under the line's base path, and a page the build doesn't publish. `resolve.json` gains `"emit": "plain"`; bless it and read the diff.
4. `emit` in the schema: bless `schemas/resolve.schema.json` and `docs/content/_generated/json-resolve.md`, and read them. A decision file for `--emit` and `emit`.
5. The docs: `docs/content/command/questions.md`'s `ascribe resolve` gains the site form, the site output's frontmatter, a line of a versioned build, and a section on reading the site output (the proposal's four points); the example, decision 4, listed in `ascribe.examples.toml` with its test, observed again (`cargo run -p ascribe-cli -- observe`); `changes/command/resolve-emit.md`.

## Out of scope

The tool and the agents' rules (phase 2).

## Acceptance criteria

- The goal's commands, in output tests, with their exit codes.
- The site form of a page equals `emit_page`'s output for it, in a test.
- `node scripts/compare/outputs.ts --base main` shows no change.

## Verify

```sh
cargo test -p ascribe-query resolve
cargo test -p ascribe-cli --test all output
cargo run -p ascribe-cli -- resolve install-agent.md --config examples/quill/ascribe.toml --build site --emit site
node scripts/compare/outputs.ts --base main
```

## Commits

1. "Write a page as the site output does, in ascribe_query::resolve"
2. "ascribe resolve --emit site, and emit in its JSON"
