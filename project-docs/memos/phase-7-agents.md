# Phase 7: Agents

Part of [Memos](README.md). Needs phase 5; personal memos need phase 6.

## Goal

An agent asked to fix or review a page is told which memos are about it, and the rules tell every agent to read and keep them.

```console
$ ascribe agents prompt review
…
Memos about what changed, to read before judging it:
- memos/agent-first.md, about install-agent.md#install-agent: check the change still agrees with it.
…
```

The agent opens the file and reads it whole. Nothing pastes a memo's text into a prompt.

## Context

- README decisions 17 and 25.
- `crates/ascribe-cli/src/agents/prompts.rs`: `fix` is `ascribe check`'s prompt format, and `review` is `ascribe diff`'s (`crates/ascribe-diff/src/prompt.rs`). The MCP server offers the same prompts.
- `crates/ascribe-lsp/src/agent_prompt.rs`: the editor's Prompt agent.
- `crates/ascribe-query/src/rules.rs`: the rules, `RULES_BUDGET`, and the order lines are cut in.
- Decisions 51 and 56 in `project-docs/decisions.md`: agents' output is capped and compact. A list is cut with `truncated`, `shown`, `total`, and `next_command`, as `ascribe refs` cuts one.

## Design

### The prompts

- **`fix`** gains a line for each memo about a page it names: the memo's path, what it's about, and its state when it isn't placed. Under a heading of its own, after the problems.
- **`review`** gains the same for each changed section or block with a memo placed on it, or an outdated memo on its section, and asks the agent to check the change still agrees with each.
- **The editor's Prompt agent** gains the lines for the page it's about.

A memo in `.ascribe/memos/` is listed in prompts made on the machine that has it, marked personal. Each list is cut after ten entries, saying how many there are and the `ascribe outline` command that lists them all.

### The rules

In a project with a memo file, the rules gain a sentence after phase 3's: `ascribe outline <page>` lists the memos about a page, and a memo is read before the block it's about is changed. It's cut with phase 3's sentence, in the order phase 3 sets.

## Tasks

1. `ascribe check`'s and `ascribe diff`'s prompt formats, with tests over `examples/quill`.
2. The Prompt agent, with a test of the server.
3. The rules' sentence, the order they're cut in, and their tests.
4. The agents guide, `changes/`, and the surface map if anything in it changed.

## Out of scope

- Pasting a memo's text into anything. An agent reads the file.
- A memo store an agent reaches over a network.

## Acceptance criteria

- `ascribe agents prompt review` in a branch of `examples/quill` that changes the install section lists `memos/agent-first.md`, and one that changes only the quickstart doesn't.
- `ascribe agents prompt fix --arg path=install-agent.md` lists it, and a detached memo's line says so.
- The rules written for `examples/quill` have both sentences, and those for a project with no memos have neither.
- With 12 memos on one page, `fix` lists ten and says the command that lists the rest.

## Verify

```sh
cargo test --workspace --locked
cargo run -p ascribe-cli -- agents prompt fix --arg path=examples/quill/docs/install-agent.md
cargo run -p ascribe-cli -- agents rules --config examples/quill
```

## Commits

1. "Name the memos about a page in the fix and review prompts"
2. "Tell agents where a page's memos are"
