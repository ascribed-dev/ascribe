# Phase 5: Findings

Part of [The surface audit](README.md). Requires phases 1 to 4.

## Goal

What the audit was for: a report of where the surface doesn't hold together, what to do about each, conventions for what's added next, and issues for the changes worth making.

## Context

- Everything the earlier phases wrote: the inventory, the journeys' notes and transcripts, the docs site's two lists, and phase 1's first findings and conventions.
- The [proposal](../surface-audit.md): the seven questions, and the five kinds of finding.
- The proposals about to add to the surface: [permalinks](../permalinks/README.md), [content checks](../content-checks/README.md), [tested examples](../tested-examples.md), and [versions](../versions.md). Each names commands, settings, and files that don't exist yet.
- `project-docs/checklists.md`: where conventions go.

## Design

### The capability table

`capabilities.md`: every thing a user can do, down the side; every surface, across. Each cell says how it's done there, or that it isn't.

The rows come from the journeys and the inventory, in a user's words ("find where a phrase is used"), not the surface's ("refs"). An empty cell is marked as deliberate, with where that's said, or as a gap. A row with four full cells is looked at too: four ways to do one thing is a cost someone pays in learning them.

### The seven questions

One at a time, across the whole inventory, with the journeys' notes as evidence. For each, the pattern first (what most of the surface does), then what departs from it.

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

For each of the four proposals: what it would add, laid against the conventions and the findings. Where a proposed name or shape would repeat a problem the audit found, say so, with what to change in the proposal. This is the part with a deadline, since those plans are next.

### Conventions

Phase 1's, completed and extended beyond the command line: how an action's title is worded, how a setting is named and where each kind of choice lives, how a tool for agents is named against the command it wraps, what a diagnostic's message and fix each say. Into `project-docs/checklists.md`, in the list each belongs to.

### Issues

One issue for each change worth making, with related renames grouped so they happen together. Each links to its finding. Findings recommended as "leave" get none.

## Tasks

1. The capability table.
2. The seven questions; the findings, in order.
3. The opening section.
4. The four proposals against the findings, with edits to each proposal where the audit changes it.
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

1. "Lay out what can be done where"
2. "Report what the surface audit found"
3. "Check the next four proposals against the findings"
4. "Write the surface's conventions into the checklists"
