# Phase 1: One task, start to finish

Part of [The surface audit](README.md). Needs nothing. This is the plan's thread: one task taken through every stage, to find out whether the method works before it's used eight times.

## Goal

- **Findings from one task:** fixing a broken link, walked four ways.
- **The first conventions for the command line,** which the [permalinks plan](../permalinks/README.md) needs before it adds a command group.
- **A method that's been tried,** with the script, the notes page, and the finding's form adjusted to what the first use showed.

## Context

- The [proposal](../surface-audit.md): the seven questions and the five kinds of finding.
- What fixing a broken link touches. Read each as a user would meet it, not in the code:
  - the editor: the Problems panel, the quick fix, the actions bar, **Prompt agent**;
  - the command line: `ascribe check` and its formats, `ascribe explain`, `ascribe link`;
  - the hook and the skill in `plugins/ascribe/`, and what `ascribe agents sync` writes;
  - the MCP server's tools (`ascribe_check`, `ascribe_explain`, `ascribe_link`).
- `docs/content/_generated/cli-*-options.md`: every command's options, generated, so they can be laid side by side.
- `examples/quill`: the project the walks use, in a fresh copy each time.
- How the agents plan's last phase ran real agents and recorded them. Its plan is gone from the repository; `project-docs/decisions.md` says where its last version is.

## Design

### The slice of inventory

Two tables, built by hand this once from the generated option lists:

- **Every command and subcommand,** with its arguments, its options, the formats it writes, and its exit codes.
- **Everything that reports or fixes a problem,** on any surface, with what it's called there.

Phase 2 replaces both with the generated inventory.

### The script

`journeys/fix-a-broken-link.md`. One starting point for all four walks: a copy of `examples/quill` with one link's target renamed, so the link is broken, and nothing said about where.

The script gives the goal ("the project has a broken link; find it and fix it") and the walker's constraints, and nothing about how. Four walks:

| Walk | By | With |
|---|---|---|
| In the editor | The maintainer | VS Code and the extension, no terminal |
| At the command line | The maintainer | A terminal and a plain editor, no extension |
| Through the hook | An agent, fresh session | The plugin installed |
| Through MCP | An agent, fresh session | The MCP server registered, no plugin |

### The notes

Under the script, one section for each walk, filled in as it goes:

- each step taken, with what was typed or clicked, and what came back, quoted;
- each moment of doubt: what was expected, what happened, how it was resolved;
- how long it took;
- for an agent, the transcript's path, and what it tried before the thing that worked.

The maintainer's two walks should take under half an hour together. If they take much longer, that's a finding about the script, and it changes before phase 3.

### The questions, on this slice

All seven, against the two tables and the four sets of notes. Expect most from three of them here: one word for one thing (what a problem is called in each place), the same thing the same way (options and formats across the commands), and what a problem says to do next (the same broken link, as each surface reports it).

### Findings, and one convention

Findings go in `findings.md`, in the form phase 5 will use: a number, the kind, where, the evidence, what it costs a user, a recommendation, a size.

From the command-line findings, a first set of conventions, written into `project-docs/checklists.md` under the list for a change to a command:

- how a command and a group of subcommands are named;
- which words `--format` takes and what each means;
- how a page, a build, and a project are named in arguments;
- what each exit code means.

Where today's commands already agree, the convention is what they do. Where they don't, it says which way is right, and the disagreement is a finding.

### What the thread is for

After the walks and the findings, a short note at the end of this phase's pull request: what about the method worked, what didn't, and what changes for phases 3 and 5. Then the script template and the finding's form are updated to match.

## Tasks

1. The two tables.
2. The script. The maintainer's part: the two walks, with notes.
3. The two agents' walks, in fresh sessions, with transcripts kept.
4. The seven questions; the findings.
5. The command-line conventions, in `project-docs/checklists.md`.
6. The note on the method, and the templates adjusted.

## Out of scope

Any other task; surfaces fixing a link doesn't touch; the capability table; fixing anything.

## Acceptance criteria

- `findings.md` has findings from this task, each with evidence quoted from a walk's notes or from the tables.
- The checklist for a change to a command says how a new group such as `ascribe permalinks` should be named and what its `--format` should accept, without anyone having to ask.
- The maintainer has said whether a walk was a reasonable ask, and the script reflects the answer.

## Verify

```sh
pnpm test
```

`scripts/project-docs/checklists.test.ts` checks the checklists.

## Commits

1. "Lay the commands and the problem surfaces side by side"
2. "Record fixing a broken link, four ways"
3. "Write the first findings, and the command line's conventions"
