# Phase 1: Decisions

Part of [Memos](README.md). Needs nothing. No code: a pull request that records what the maintainer decides.

## Goal

Six choices are made before anything ships, because each goes into a contract: the language, a parser rule, the page templates' output, the diagnostics, a file format, and two commands' JSON.

This phase can't be folded into phase 2. A thread may narrow what it handles. It may not ship a shape the plan expects to change.

## Context

- The [proposal](../proposals/memos.md), whole.
- SPEC §3.5, "Forms", and §4.9, `@intended`.
- `crates/comrak-ascribe/FORK.md`, and `src/ascribe.rs` in that crate: how a directive line becomes a block, and that a container's contents are parsed as CommonMark before Ascribe pairs the opener with its `@end`.
- `crates/ascribe-syntax/src/tree.rs`: the raw source text of a span, with container prefixes stripped.
- The page templates plan's [decisions 18, 22, 33, and 55](https://github.com/ascribed-dev/ascribe/blob/180eee83/project-docs/page-templates/README.md#decisions), `crates/ascribe-check/src/checks/template.rs`, and `crates/ascribe-templates/set/how-to.md`.
- `docs/content/checks/diagnostics.md`, "What to do next": the kinds of next step.
- `crates/ascribe-query/src/refs.rs` and `outline.rs`: the targets `ascribe refs` takes, and the shape of both commands' JSON.

## The decisions

Each has a recommendation. The maintainer may take it, change it, or ask for more.

### 1. How a container memo's contents are kept

A container's contents are parsed as CommonMark today, then paired with their `@end`. Decision 5 says nothing reads a memo's contents. There are two ways to get there.

| Option | Means |
|---|---|
| **Raw lines** (recommended) | The parser keeps the lines between `@memo:` and its `@end` as text, as it keeps a fenced code block's. Nothing inside is a block, a directive, an inline, or a heading, so every reader, now and later, gets decision 5 without knowing memos exist, and `ascribe fmt` leaves the bytes alone by construction. An unclosed fence in a memo can't swallow its `@end`. It's a change to the fork: the line scanner knows that `memo` opens a raw container, and SPEC §3.5 says what a raw container is. A line that's only `@end`, at the memo's indent, closes it, so a memo can't hold one |
| Parsed, then skipped | No change to the fork. Every reader that walks the tree before resolution (the source index, the page checks, `ascribe refs`, Vale's extraction, the formatter, the language server's links and ids) skips a memo's children, and each new reader has to remember to. A memo can still break its page: an unclosed fence inside it swallows the `@end` |

Both keep a container's prefixes as they are: a memo in a list item or a block quote is indented or quoted like anything else in it.

**Decided: raw lines.** Decision 5 holds by construction, for every reader there is and every one added later, instead of by a rule each has to remember; `ascribe fmt` leaves the bytes alone for the same reason; and a memo can't break its page. The cost is one change to the fork, marked and listed in `FORK.md`, and a sentence in SPEC §3.5 saying what a raw container is: its lines are text until a line that's only `@end` at its indent, with the container's prefixes as they are. README decision 21.

### 2. Which of a template's words become memos, and what the check reports

Decision 10 settles that a template's instructions to its writer become memos. The page templates plan expected them in block quotes; the six templates it shipped have none inside a section. Their instructions are paragraphs that are one placeholder, beside placeholders that stand for content:

```markdown
\{Optional: Specify when and why your user might want to perform the task.}

This guide explains how to \{insert a brief description of the task}.
```

The first is said to the writer, and the second is replaced. Nothing in their shape tells them apart: `\{Optional: Release notes version - 0}` is a whole paragraph and stands for content.

**Which words:**

| Option | Means |
|---|---|
| **Named in the conversion's rules** (recommended) | `rules.toml` gains `memos`, a list for each template of its paragraphs that are instructions, each named by its text, as `about` names what's removed. The conversion writes each as a line memo in its place, with their words and without the braces, since a memo isn't something to replace. A paragraph that isn't named stays a placeholder. A rule that doesn't find its text stops the script, as every rule does, so an upstream release that rewords one is caught. Which of the 28 are instructions is read once, by a person, in phase 4's pull request |
| Every paragraph that's one placeholder | No list to keep. `\{Optional: Release notes version - 0}` and the like become memos, and a writer has nothing to replace them with |
| None | They stay placeholders, published until replaced and reported until then. Phase 4 has nothing to do but remove the quoted-paragraph rule |

**What the check reports** of a memo with the same text as one of the template's:

| Option | Means |
|---|---|
| **Nothing** (recommended) | It's never published, so it costs a reader nothing, and an instruction such as "This section is optional" is still true after the section is written. A section left empty is another matter: its placeholders are reported as they are now |
| Advice, as a placeholder is | A writer who keeps one deletes it or acknowledges it, on every page made from a template |

Either way, the rule that a quoted paragraph is an instruction (page templates decision 33) goes: no template in the set has one, and a project's copies are its own.

**Decided: named in the conversion's rules, and the check reports nothing of them.** Nothing in a paragraph's shape says whether it's said to the writer or stands for content, so a person reads the 28 once and the rule file keeps the answer, where a reworded upstream release trips it as every rule does. The conversion writes each as a line memo, with the words and without the braces. A memo with the template's text costs a reader nothing and is often still true, so the check leaves it alone; an empty section is still reported by its placeholders. The quoted-paragraph rule goes. README decision 22.

### 3. The diagnostics' names, levels, and next steps

| Reports | Recommended name | Level | Next step |
|---|---|---|---|
| A memo file's frontmatter doesn't parse, has no `about`, or has another key; or `memos/` is inside the content root | `memo-invalid` | Error | `write` |
| `about` names a page, heading, or entry that doesn't exist | `memo-detached` | Warning | `choose`: the closest ids on the page, or delete the memo |
| The quote isn't in the page or section any more | `memo-outdated` | Advice | `write` |
| A memo file's body is over `[checks.memo-long] limit` characters, 2,000 by default | `memo-long` | Advice | `write` |
| An HTML comment in a page or fragment, other than Vale's | `html-comment` | Advice | `review` |

The first four are reported on the memo file, at its `about` or `quote` line, and aren't review checks: a memo that's wrong is edited or deleted. `html-comment` is a review check, reported at the comment, and acknowledged with `@intended`. An empty `@memo`, one with attributes, and one with nothing below it are the errors any directive gets, with no names of their own.

**Decided: as the table has it.** Four checks on a memo file, named for what's wrong with it, at the line that's wrong; one review check on a page, acknowledged where it reports; and nothing of its own for what the directive's own rules already report. `memo-long` takes `limit`, as `page-size` and `image-large` do, 2,000 by default: a memo longer than that is a document, and belongs in one. README decision 23.

### 4. What `about` can name, and how `quote` is matched

| Question | Recommended |
|---|---|
| The targets | Every one `ascribe refs` takes: a page, a fragment, a heading (`page#id`), and `phrase:`, `feature:`, `term:`, `dimension:`, `note:`, and `widget:`. One grammar, one parser |
| `quote` with a target that has no text | An error, `memo-invalid`: only a page, a fragment, or a heading has blocks |
| What the quote is matched against | The source of each block in the target, as written in its file, with whitespace runs read as one space and case kept. The editor's create action fills it from the source, so it matches exactly |
| A quote in two blocks | The first, in source order |
| A quote that runs across two blocks | Not found: the memo is outdated |
| A heading in a fragment | Named in the fragment, `_fragments/keys.md#rotate`, as `ascribe refs` names it. A quote is matched in the file `about` names, never in a page that includes it |

**Decided: as the table has it.** One grammar and one parser, `ascribe refs`'s, so a memo can name whatever a reference can, and `quote` only where there's text to hold it. Matching source, not rendered text, with whitespace folded and case kept, is what lets two machines never disagree about where a memo is, and the editor's create action fills the quote from the same source. README decision 24.

### 5. What `ascribe outline` and `ascribe refs` say about memo files

Both commands' JSON is a contract. The recommendation adds keys and changes none, so neither `schema_version` moves.

- **`ascribe outline`:** the page gains `memos`, for memos about the page as a whole, and each heading gains `memos`, for memos about that section or a block in it. Each entry is `file`, `state` (`placed`, `outdated`), `quote` when it has one, `first_line` (the body's first line, cut as every line is), and `personal` when it's from `.ascribe/memos/`. A detached memo is about nothing on the page, so it isn't listed. The text output adds one line under the heading for each.
- **`ascribe refs`:** a memo file is a place, with `use` `memo`, at its `about` line. A detached memo is listed under the target it names, as a link to a page that's gone is.

**Decided: as recommended.** Both commands gain keys and change none, so neither contract's version moves. A detached memo is about nothing on a page, so `outline` doesn't list it, and `refs` does, under the target it names, which is where someone deleting that target looks. README decision 25.

### 6. Whether the memo folders can be moved

| Option | Means |
|---|---|
| **Fixed: `memos/` and `.ascribe/memos/`** (recommended) | One place to look in every project, for a person and for an agent. Moving it later is additive: a key with these defaults |
| A key, `[project] memos-dir` | A team with a `memos/` folder already gets to choose. One more thing to configure, and every surface that says where memos are has to read it |

**Decided: fixed, `memos/` and `.ascribe/memos/`.** One place to look, for a person and for an agent, in every project. A team with a `memos/` folder of its own has its `.md` files read as memos and reported, which the proposal already names as the one thing that breaks; a key with these two as defaults is additive if that ever matters. README decision 26.

## Tasks

1. The maintainer reads the six and answers each. _Done,_ 15 October 2026: each recommendation is taken.
2. Record each answer in this file, under its heading, as **Decided:** with the reason, and in the README's decisions, numbered from 21. _Done._
3. Record them in `project-docs/decisions/`, numbered by this plan's issue, as the repository does. _Waits:_ the plan has no issue and no pull request yet, and a decision file is named by one of the two.
4. Add to the proposal what the answers change in it. _Done:_ a container's raw lines, how a template's instructions are chosen, what `about` can name, and `memo-long`'s default.

## Out of scope

- The wording of the diagnostics' messages and fixes. Each phase writes its own, against the registry's conventions.
- Anything about a server or a store other than the two folders.

## Acceptance criteria

- Each of the six has an answer and a reason.
- Nothing in a later phase file contradicts one.
- Phase 2 can be started without asking the maintainer anything.

## Commits

1. "Record the decisions the memos plan ships"
