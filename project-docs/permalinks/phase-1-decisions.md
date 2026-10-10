# Phase 1: Three decisions

Part of [Permalinks](README.md). Needs nothing. No code: a pull request that records what the maintainer decides.

## Goal

Three choices are made before anything ships, because phase 2 puts each into a contract: the language, the content model, and the build's output. None can be changed afterwards without breaking what was promised.

This phase can't be skipped or folded into phase 2. A thread may narrow what it handles. It may not ship a shape the plan expects to change.

## Context

- The [proposal](../permalinks.md)'s open questions, which these are.
- `SPEC.md` §4.1: an `@id` is letters, digits, hyphens, underscores, and periods.
- The [versions proposal](../versions.md), "A page that replaces another": what the second decision would replace.
- DITA's `resourceid`, which pairs an id with an application's name, and Mozilla's in-product links, where a name is registered before any page uses it. Both are described in the proposal's research.

## The decisions

Each has a recommendation. The maintainer may take it, change it, or ask for more.

### 1. What a name may look like

Whether a docs set that covers several products needs a separate set of names for each.

| Option | Looks like | Cost |
|---|---|---|
| **One flat set, and a convention** (recommended) | `agent-settings`, `cli-settings`, at `/go/agent-settings` | Nothing to build. `verify` can't check one product's names alone, except by prefix |
| A namespace in the model | `settings` declared for `agent`, at `/go/agent/settings` | A new idea in the language and the content model, and a second segment in every address |

The recommendation rests on this: a flat set can grow a namespace later, by treating a prefix as one. A namespace can't be taken back.

It also fixes the characters a permalink may use. Recommended: what `@id` allows today, with nothing added, so a `/` never appears in a name.

### 2. How far a name must be unique

| Option | Means |
|---|---|
| **Within what one build publishes** (recommended) | Two pages may carry one permalink if no build publishes both. That's how "the same page in two versions" would be said, without a new key |
| Across the whole project | Simpler to explain and to check. A page rewritten for a new version needs `replaces`, as the versions proposal has it |

The recommendation rests on this: unique within a build is the looser rule, and a project can be held to the stricter one later by a check. The reverse would break projects.

Taking it doesn't decide the versions question. It leaves it open.

### 3. Where a name may be declared

| Option | Means |
|---|---|
| **Only on a page or a heading that exists** (recommended) | A product's team proposes a name by a pull request against the docs. Every permalink leads somewhere real |
| Also reserved in `ascribe.toml`, before its page exists | A product can ship a help link first. A reserved name needs somewhere to lead meanwhile, and a check for one never filled |

The recommendation rests on this: aliases (phase 5) already give a name that isn't on a heading, pointing at one that is. Reserving can be added as "an alias with no target yet" if it's ever wanted.

## Also confirmed here

Small choices the proposal made, each a name that ships in phase 2:

- The attribute and the frontmatter key are both `permalink`.
- The addresses' path is set by `[consumer] permalink-path`, a path from the site's root. Unset, it's `go/` under the base path.
- The build's list is `_ascribe/permalinks.json` in the site output.

## Tasks

1. Put each decision to the maintainer, with its table.
2. Record the answers in this plan's README, under "Decisions", numbered on from 9, each with its reason.
3. Update the proposal: move the three questions out of "Open questions", with what was decided.
4. If the second decision isn't the recommended one, say so in the [versions proposal](../versions.md), which then keeps `replaces`.

## Out of scope

Everything else. No code, no change to `SPEC.md`: phase 2 writes the language change, with these decisions in hand.

## Acceptance criteria

- The README's decisions cover the three, and the three names above.
- Phase 2's file needs no choice made in it that this phase left open. Read it against the decisions; if one is missing, add it here.

## Verify

```sh
pnpm test
```

`scripts/repo-docs/paths.test.ts` and `scripts/project-docs/checklists.test.ts` check the paths these files name.

## Commits

"Decide what a permalink's name is, how far it's unique, and where it's declared"
