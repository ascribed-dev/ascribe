# Phase 2: For agents

Part of [A page in the site output](README.md). Needs phase 1.

## Goal

`ascribe_resolve` with `emit: "site"` gives an agent the same as the command, and the agents' rules say when to ask for it.

```json
{ "name": "ascribe_resolve", "arguments": { "page": "install-agent.md", "build": "site", "emit": "site" } }
```

returns the site output's Markdown as its text and the command's JSON, with `"emit": "site"`, as its `structuredContent`.

## Context

- README decision 5; the proposal's "For agents"; decision 56.
- `crates/ascribe-cli/src/mcp/tools.rs`, the `ascribe_resolve` spec and how its call builds the command's arguments.
- `crates/ascribe-cli/src/agents/skill.rs` and the rules `ascribe agents sync` writes.
- `crates/ascribe-cli/tests/all/mcp.rs`.

## Design

- **The argument:** `emit`, a string enum of `plain` and `site`, described as the command's option is, with `plain` the default. With `go:<name>`, an `emit` is the command's usage error, returned as the tool's error.
- **The description** gains one example: `{"page": "docs/keys.md", "emit": "site"}` returns the page as a site receives it, with Ascribe's elements, to see what a switcher or a badge becomes.
- **The skill and the rules:** the skill's `ascribe resolve` line gains `--emit site`, "to see what a reader of the site gets"; the rules that mention `ascribe_resolve` say the same, and link the reference page's section on reading the site output.

## Tasks

1. `emit` on the tool, with tests in `crates/ascribe-cli/tests/all/mcp.rs`: `site`, the default, and `go:` refused.
2. The skill and the rules, with their tests' expected text blessed and read.
3. A decision file for the argument (decision 56); the surface map regenerated.
4. The docs: `docs/content/agents/commands.md`'s `ascribe_resolve`; `changes/agents/resolve-emit.md`.

## Out of scope

Anything for HTML or the JSON output (the proposal's Settled 1 and 3).

## Acceptance criteria

- The goal's call, in a test, returns the site output's Markdown and `"emit": "site"`.
- `ascribe agents sync`'s written rules mention `emit: "site"`, in their test.

## Verify

```sh
cargo test -p ascribe-cli --test all mcp
cargo test -p ascribe-cli --test all agents
```

## Commits

1. "ascribe_resolve takes emit, and the agents' rules say when to ask for the site form"
