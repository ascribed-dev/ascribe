# Phase 5: Findings

Part of [The surface audit](README.md). Requires phases 1 to 4.

## Goal

What the audit was for: a report of where the surface doesn't hold together, what to do about each, conventions for what's added next, and issues for the changes worth making. The capability table is phase 2's; this phase finishes it against what the sessions showed.

## Context

- Everything the earlier phases wrote: the inventory and the capability table, the sessions' notes and transcripts, the docs site's two lists, and phase 1's first findings and conventions.
- The [proposal](https://github.com/ascribed-dev/ascribe/blob/460259f6529f9e7d5a8e06e600a9f28b43ae03f5/project-docs/proposals/surface-audit.md): the seven questions, and the five kinds of finding.
- The proposals and plans about to add to the surface: [permalinks](https://github.com/ascribed-dev/ascribe/blob/76a46b26090c2dd0efbd7c918a29d58754f8eace/project-docs/permalinks/README.md), [tested examples](https://github.com/ascribed-dev/ascribe/blob/395fe80d/project-docs/proposals/tested-examples.md), [versions](https://github.com/ascribed-dev/ascribe/blob/395fe80d/project-docs/proposals/versions.md), [page templates](https://github.com/ascribed-dev/ascribe/blob/395fe80d/project-docs/proposals/page-templates.md), and the [website](https://github.com/ascribed-dev/ascribe/blob/e80d764ef544f7fb617dec8084248a3e161a6d23/project-docs/proposals/website.md). Each names commands, settings, or files that don't exist yet. Content checks has landed, so its commands are in the inventory.
- `project-docs/checklists.md`: where conventions go.

## Design

### The capability table, finished

Phase 2's `capabilities.md`, checked against the sessions: a cell that says a thing can be done where an agent couldn't do it is corrected, with the transcript as evidence, and every cell still open is settled as deliberate, with where that's said, or as a gap.

### The seven questions

One at a time, across the inventory and the capability table, with the sessions' notes as evidence. For each, the pattern first (what most of the surface does), then what departs from it.

### The findings

`findings.md`, each one:

| Field | Holds |
|---|---|
| Number | For referring to it |
| Kind | A contradiction, a duplicate, a gap, a naming problem, or friction |
| Where | The surfaces involved, by the names in the inventory |
| Evidence | Quoted: a walk's note, or inventory entries side by side |
| Cost | What it does to a user, in a sentence |
| Recommendation | One of: rename, merge, remove, add, document, leave. With what breaks, if anything |
| Size | Small, medium, or large |

In order of cost to a user, not of how easy the fix is. "Leave" is a real recommendation, with its reason, for something that looks inconsistent and is right.

Then a short opening section: the five or six findings that matter most, and what the audit found about the surface as a whole. If the answer is that it holds together well, it says that.

### The proposals, against the findings

For each of those proposals: what it would add, laid against the conventions and the findings. Two have left a name open for this: the command that writes the starter set of [page templates](https://github.com/ascribed-dev/ascribe/blob/395fe80d/project-docs/proposals/page-templates.md) into a project, and every command group the others propose. Where a proposed name or shape would repeat a problem the audit found, say so, with what to change in the proposal. This is the part with a deadline, since those plans are next.

### Conventions

Phase 1's, completed and extended beyond the command line: how an action's title is worded, how a setting is named and where each kind of choice lives, how a tool for agents is named against the command it wraps, what a diagnostic's message and fix each say. Into `project-docs/checklists.md`, in the list each belongs to.

### Issues

One issue for each change worth making, with related renames grouped so they happen together. Each links to its finding. Findings recommended as "leave" get none.

## Tasks

1. The capability table, finished against the sessions.
2. The seven questions; the findings, in order.
3. The opening section.
4. The pending proposals against the findings, with edits to each where the audit changes it, and a name for each command one has left open.
5. The conventions, in the checklists.
6. The issues, after the maintainer has read the findings and said which to open.

## Out of scope

Making any change. Deciding between two findings' recommendations where they conflict: say that they do, and leave it to the maintainer.

## Acceptance criteria

- Every finding has quoted evidence and one recommendation.
- Every cell of the capability table is filled, or marked as a gap or as deliberate.
- A new command, action, setting, or tool can be named from the checklists without asking.
- The maintainer has read the findings before any issue is opened.

## Verify

```sh
pnpm test
```

## Commits

1. "Finish the capability table against the sessions"
2. "Report what the surface audit found"
3. "Check the pending proposals against the findings"
4. "Write the surface's conventions into the checklists"
