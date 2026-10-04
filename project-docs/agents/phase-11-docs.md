# Phase 11: Docs and a full pass

Part of [Agents](README.md). Requires phases 1 to 10. Docs, and work by hand.

## Goal

The agents guide reads as one document, and the whole surface has been tried on set tasks with two real agents, with the results written down. Tests so far check what Ascribe writes; this phase checks whether agents do better with it.

## Context

- `docs/agents.md`, grown a section at a time by phases 4 to 10. `docs/README.md`, `docs/getting-started.md`, `docs/editor.md`, `docs/review.md`, `docs/cli.md`.
- `examples/quill` and `examples/monorepo`: the projects to try things on.
- The brainstorm's [suggested order](../brainstorm.md#suggested-order) and [open questions](../brainstorm.md#open-questions): several are about agents, and this phase can answer them.

## Design

### The guide

`docs/agents.md`, in this order: what Ascribe does for agents and what it doesn't (it calls no model); the loop (`check`, `explain`, `model`, `outline`, `link`, `render`, `refs`); agent instructions; **Prompt agent** (problems, then review); the MCP server; VS Code; hooks and the plugin; GitHub Copilot; other agents (what works with any: the commands, `AGENTS.md`, and the skill); what to watch for when an agent has both Ascribe and tools that reach the network (Ascribe reads and sends nothing, but a comment's text in a prompt is still other people's text); and what's sent where (nothing leaves the machine through Ascribe; the prompt goes wherever the user pastes it).

Read it start to finish as someone who uses Cursor, which has no section of its own, and fix whatever leaves them stuck. Link it from the README and from getting started.

### The pass

Seven tasks, each run in a fresh copy of `examples/quill` (and task 5 in `examples/monorepo`) in a scratch repository outside this one, with Claude Code and with GitHub Copilot in VS Code, **twice each: once with nothing from this plan set up, once with all of it**:

1. Add a guide page about a made-up feature, linked from the index.
2. Fix a project with five seeded problems that `check` reports (a broken link, a missing frontmatter field, an unknown attribute value, an unclosed container, an undeclared phrase), and one it doesn't: a literal product name where a phrase exists. No check finds that one, so it tests the instructions alone; judge it by reading the result, and record it in its own column.
3. Add a variant for a third platform to an existing page.
4. Rename a heading and keep every link to it working.
5. In the monorepo, add a page to the security handbook that meets that project's rules and not the docs project's.
6. Address three review comments on a scratch pull request, starting from **Prompt agent: all open**.
7. Clean up a project that already has a few hundred warnings (seed them across many pages, of five or six kinds), starting from `ascribe check --summary`. Record whether the agent works rule by rule or file by file, and whether it stops and asks when it stops making progress.

"Passes" means `ascribe check --deny-warnings` exits `0`, for every build. The Claude Code hook reports errors for the editor's build only, so a run where the hook was quiet can still fail this.

Then one more agent that has only what `ascribe agents sync` writes (Codex, or Cursor outside VS Code's extension): tasks 1 and 2, once. The skill and `AGENTS.md` are meant to reach agents this plan has no phase for, and this is the test of that.

For each run, record: whether the project passes at the end; how many times the person had to step in; whether the agent ran the check itself; and anything it got wrong that a tool could have told it. Put the table in the pull request and in `project-docs/agents/pass-results.md`.

Do not post to this repository's pull requests. For task 6, use the review fixture, rebuilt first (`scripts/review-fixture/setup.ts`; see the review plan's [phase 8](../review/phase-8-docs.md#the-fixture)).

### What to do with the results

- A failure that a small change fixes (a tool's description, a line in the instructions, a prompt's wording): fix it in this phase.
- A failure that needs design: open an issue, and list it in the results.
- Update the brainstorm: mark sections 8 to 13 as planned and built, and answer the open questions this pass settles ("which harnesses come first", "should Ascribe ever apply edits").

## Tasks

1. The guide, read as a whole and corrected, with every command and setting in it checked against the code.
2. The pass, and its results file.
3. The small fixes, each with a test where one fits.
4. The brainstorm and `CHANGELOG.md`.

## Out of scope

An automated evaluation that runs agents in CI; new features the pass suggests.

## Acceptance criteria

- Every task has four recorded runs, or a stated reason one couldn't be run.
- With everything set up, both agents finish tasks 1 to 5 and 7 with `ascribe check --deny-warnings` passing and without the person explaining Ascribe's syntax.
- The guide says plainly what Ascribe sends and to whom: nothing, to no one.

## Verify

```sh
cargo test --workspace --locked
pnpm format:check && pnpm lint && pnpm typecheck && pnpm test
```

## Commits

1. "Finish the agents guide"
2. "Record the agents pass and fix what it found"
