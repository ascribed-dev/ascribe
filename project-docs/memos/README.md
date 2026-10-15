# Memos

The people and agents who maintain a docs set can find out why a page is written the way it is, at the place they're about to change. A memo is that reason, kept beside what it's about and never published: `@memo` in a page for a line or two, and a memo file for something longer. The idea is in the [proposal](../proposals/memos.md). This is the plan for building it.

## Who it's for

- **Whoever edits a page next,** who today finds the reason for a table, an order, or a caveat in a wiki, an old pull request, or nowhere.
- **An agent editing a page,** which most needs to know why a page is written the way it is and can't work it out. A reason handed over with the page does more than the same reason anywhere else.
- **A writer starting from a page template,** whose template's instructions are quoted paragraphs today: text that would be published if it were left in.
- **A team that writes HTML comments in pages,** which it takes for private and which are in every published page's source.

## What exists today

- **`@intended` is the nearest thing.** A line in the page, bound to the block below, with a reason that's a line primary (SPEC §4.9). Build resolution removes it (`drop_acknowledgements` in `crates/ascribe-resolve/src/build/mod.rs`), so no output, renderer, or `ascribe diff` sees one. Its schema is a built-in in `crates/ascribe-core/src/schema.rs`, beside `@note` and `@details`, which have a container form.
- **A container's contents are parsed as Markdown.** The comrak fork (`crates/comrak-ascribe`, changes listed in its `FORK.md`) makes each directive line a block, an `AscribeLine`, and parses everything between an opener and its `@end` as blocks of CommonMark. `ascribe-syntax` pairs openers with `@end` afterwards (`src/structure/`). No directive's contents are kept as raw text, and an unclosed code fence inside a container swallows its `@end`.
- **An HTML comment isn't private,** and binds to nothing. It's kept in the site output and the JSON output, so it's in every published page's source, and a reworded one is a change to `ascribe diff`. The proposal's table has every place. Ascribe passes Vale's own comments, `<!-- vale off -->`, through to Vale (`crates/ascribe-check/src/prose/extract.rs`).
- **The docs' generated fragments say what generates them in an HTML comment.** About 63 of the 76 comments under `docs/content/` are those, one at the top of each fragment a test rewrites with `ASCRIBE_BLESS=1`.
- **Page templates shipped without `@memo`.** The page templates plan carried a template's instructions as quoted paragraphs until `@memo` exists, and said that then "the conversion gains one rule, the set is regenerated, and a project's existing copies are its own to change or not" ([its decisions 18, 22, 33, and 55](https://github.com/ascribed-dev/ascribe/blob/180eee83/project-docs/page-templates/README.md#decisions)). But none of the six templates it shipped has a quoted instruction inside a section. Their instructions to the writer are paragraphs that are one placeholder, `\{This section is optional}` or `\{Optional: Specify when and why your user might want to perform the task.}`, beside placeholders for content such as `\{insert a brief description of the task}`: 28 such paragraphs across the six. A placeholder is published as written until it's replaced, and `template-placeholder-left` reports it. The conversion is `scripts/templates/convert.ts`, whose `rules.toml` names each text it changes; the set is `crates/ascribe-templates/set/`; the check is `crates/ascribe-check/src/checks/template.rs`, and `template_places` gives the editor what's left to fill in.
- **`ascribe refs` names anything in a project:** a page or fragment, a heading as `page#id`, `phrase:`, `feature:`, `term:`, `dimension:`, `note:`, and `widget:` (`TargetKind` in `crates/ascribe-query/src/refs.rs`). Each place it lists has a `use`.
- **`ascribe outline` lists a page's headings,** with each one's id, file, and line, and has a `schema_version` (`crates/ascribe-query/src/outline.rs`). The MCP tool `ascribe_outline` returns the same.
- **The prompts that name a page** are `fix` and `review` (`crates/ascribe-cli/src/agents/prompts.rs`), and the editor's Prompt agent builds its own (`crates/ascribe-lsp/src/agent_prompt.rs`). The rules `ascribe agents sync` writes have a budget of 4,000 characters (`RULES_BUDGET` in `crates/ascribe-query/src/rules.rs`).
- **Renaming a page or a heading's id** rewrites the links to it in pages and in `ascribe.toml`, from the command (`ascribe rename`) and from the editor (`crates/ascribe-lsp/src/rename.rs`).
- **`.ascribe/` is the project's own folder, which the docs tell a team to ignore** (`docs/content/command/project-files.md`): builds, Vale's configuration, and the link report's store.
- **The word `memo` is used nowhere,** and a project's own directives need a hyphen in their names, so none is called `memo`.
- **The landing page's "Where it's going"** lists this feature as "Notes".

## Decisions

These are settled. Don't reopen them in a phase; if one can't be met, stop and report.

From the proposal:

1. **Three kinds of commentary, kept apart:** a memo in the page, a memo file, and discussion in review. Memos have no replies, and nothing about them is live.
2. **Memos are authored, not derived.** They share nothing with a cache: deleting one loses it.
3. **`@memo` is a built-in directive,** a sibling of `@intended` and not an extension of it. It has a line form, whose text is a line primary, and a container form, by the rule every directive follows (SPEC §3.5). It takes no attributes.
4. **What a memo is about follows `@available`'s rule:** at the top of a section it's about the section, anywhere else the block below it. Several stack. A memo with nothing after it is the error any following-block directive gets.
5. **Nothing reads a memo's contents.** A `{phrase}` in one isn't a phrase, a link isn't checked and isn't a use of its target, an `@include` isn't followed, a heading gets no id, and Vale isn't given the text.
6. **A memo is removed where `@intended` is,** when a page is resolved for a build. Every output, both renderers, the preview, the HTML report, and the source anchors are what they'd be without it. A fragment's memo is removed in every page that includes it. The site output's markup doesn't change, so `tests/render/` gains no fixture.
7. **A change to a memo alone is no change** to `ascribe diff` and to review. The page isn't listed.
8. **`ascribe fmt` treats a memo as it treats `@intended`:** the directive touches its block, and its text is left alone.
9. **HTML comments stay as they are.** Once `@memo` exists, an HTML comment in a page is advice, `html-comment`: a review check, acknowledged with `@intended` where the comment is for another tool. Vale's comments are exempt. It ships with the in-page memo.
10. **A page template's instructions to its writer become memos,** and the conversion is the only thing that changes for them.
11. **A memo file is Markdown with two frontmatter keys:** `about`, required, in the form `ascribe refs` takes, and `quote`, optional, which narrows the memo to the block whose source holds it. The body is CommonMark, shown as written, with nothing resolved. No author, no date, and no build.
12. **Team memos are in `memos/`, beside `ascribe.toml`, and personal memos in `.ascribe/memos/`.** The format is the same. `memos/` can't be inside the content root. A file's name means nothing, and subfolders are the team's. Memo files are read through `ascribe_resolve::FileSystem`.
13. **A memo's position is worked out each time it's read,** and nothing is written back. It's placed, outdated (the quote isn't found), or detached (`about` names nothing). The quote is in the text, with whitespace runs read as one space, or it isn't; no similarity score.
14. **A memo is about one thing.**
15. **Where memos come from is behind one interface** in the binary, with two implementations: the committed folder and the personal one. The binary gains no HTTP client.
16. **The checks report on the memo file.** `ascribe check` reads team memos only; problems with personal memos show in the editor.
17. **A memo file is brought to the page by** `ascribe refs`, `ascribe outline` and its MCP tool, a lens and a create action in the editor, a line in each prompt that names the page, and two sentences in the rules. Agents get paths and first lines, in lists cut the way every list is, never a memo's text pasted in.
18. **A memo can't say it must be gone before the page is finished.** No attribute and no to-do marker.
19. **The memo in the page is built first.**
20. **The word is "memo"** for both, decided by the maintainer on 11 October 2026.

[Phase 1](phase-1-decisions.md) settles six more, each a contract a phase ships: how a container memo's contents are kept, which of a template's words become memos and what the check then reports, the diagnostics' names and levels, what `about` can name and how `quote` is matched, what `ascribe outline` and `ascribe refs` say about memo files, and whether the memo folders can be moved.

## The thread

**The use case:** a page in `examples/quill` has a two-paragraph `@memo` container, holding a link to a file that doesn't exist and a word in braces. `ascribe check` reports nothing about either, `ascribe fmt --check` passes with the memo as written, `ascribe build` writes the site, plain, and JSON outputs byte for byte as they are without the memo, and `ascribe diff` says the page didn't change.

**The layers it crosses:**

| Layer | Where | What the thread needs from it |
|---|---|---|
| The language | `SPEC.md`, `ascribe-core`'s schema | `@memo`, its two forms, and what's in a container |
| The parser | `comrak-ascribe`, `ascribe-syntax` | A container whose contents nothing parses, if phase 1 decides so |
| Resolve | `ascribe-resolve`, `src/build/` | The memo removed beside `@intended` |
| Check | `ascribe-check` | Nothing in a memo checked, indexed, or given to Vale |
| Format | `ascribe-fmt` | The text left as written |
| Diff | `ascribe-diff` | A change to a memo alone isn't a change |
| The editor | `ascribe-lsp` | No diagnostic inside a memo, and no link, phrase, or id from it |
| The docs | `SPEC.md`, `docs/content/` | The directive's reference |

**The seam it tests** isn't "never published": removing a block at resolution is a line beside `@intended`'s, and `scripts/compare/outputs.ts` proves it. It's "nothing reads the contents". The source index, the page-level checks, `ascribe refs`, the prose handed to Vale, the formatter, and the language server all read the tree before resolution, and the fork decides a container's blocks before Ascribe sees them. A line memo doesn't test this, since a line primary is already inert. If the seam fails, either every reader learns to skip a memo, and each new reader has to remember, or memos are line form only.

**Left out, and where it comes back:**

| Left out | Phase |
|---|---|
| Completion, highlighting, and hover for `@memo`, `html-comment`, and the docs' generated notes as memos | 3 |
| Page templates' instructions as memos | 4 |
| Memo files, their states, their checks, `ascribe refs`, and `ascribe outline` | 5 |
| The editor's lens and create action, rename, and personal memos | 6 |
| The prompts and the rules | 7 |
| A guide, and Ascribe's own docs using memos | 8 |

After phase 2, a memo works everywhere a page is read and published, and the editor shows it as an unknown-looking directive with no completion. The reference says what a memo is and does; it doesn't yet mention memo files.

## Phases

| Phase | Result |
|---|---|
| [1: Decisions](phase-1-decisions.md) | The maintainer settles the six contracts the phases ship. |
| [2: The thread](phase-2-thread.md) | A `@memo` in a page, in either form, is never published, never checked, never reformatted, and never a change. |
| [3: The memo in every surface](phase-3-surfaces.md) | The editor knows `@memo`, an HTML comment in a page is advice with a fix that makes it a memo, and the docs' generated fragments say what generates them in one. |
| [4: Page templates](phase-4-templates.md) | A page started from a template has its instructions to the writer as memos, never published, and its placeholders as before. |
| [5: Memo files](phase-5-files.md) | A longer memo in `memos/` is placed on its page, section, or block, reported when it's outdated or detached, and listed by `ascribe outline` and `ascribe refs`. |
| [6: Memo files in the editor](phase-6-editor.md) | A memo is a click to read and a command to write, a rename keeps it attached, and a personal memo stays on one machine. |
| [7: Agents](phase-7-agents.md) | An agent asked to fix or review a page is told which memos are about it, and the rules tell every agent to read and keep them. |
| [8: Ascribe's own docs](phase-8-docs.md) | Ascribe's docs have a guide to memos and use them, and the landing page stops listing them as coming. |

Phase 1 comes first because phase 2 ships a directive, a parser rule, and a SPEC section, and phase 5 ships a file format and two commands' JSON. Phase 2 is the thread through the riskiest seam. After it, the order is by value: phase 3 finishes the in-page memo, since `html-comment` names it as the fix; phase 4 is what page templates have waited for. Phase 5 is a thread of its own, through a second seam: placing a file's memo on a block by `about` and `quote`, against pages, fragments, and the content model. Phases 6 and 7 bring memo files to the people and agents who need them, and phase 8 is the docs.

Each phase's result is something a person or an agent can run or open. None builds a layer for a later phase to use.

## Carved out, and waiting

- **Showing memos in the preview or in review.** A view for authors, with memos beside their blocks, is possible later: the source anchors exist. The preview is what readers see, so it would be a switch, off by default.
- **"The block changed and its memo didn't."** It's `ascribe drift`'s question asked of prose, and worth asking once memos are in use.
- **A server for people without a checkout.** A later store either writes memo files for the binary to read, or lives in TypeScript where GitHub access does (decision 15).
- **Search.** The [search proposal](../proposals/search.md) could index memos beside pages.
- **Converting HTML comments in bulk.** `html-comment`'s fix converts one at a time.

## The other plans

- **Page templates** is finished, and its decisions 18 and 22 left the instructions to this plan. Phase 4 changes its conversion and regenerates its set. A project's copies are its own.
- **Versions.** Nothing shared. A memo has no build and no line (decision 11), so a versioned build removes memos as every build does.
- **The [glossary pages proposal](../proposals/glossary-pages.md)** would make a term a heading on a page. A memo about `term:<id>` then names the term, as `ascribe refs` does, whatever carries it.

## Rules for every phase

- Branch before committing; never commit to `main`.
- Read the current code before the phase file's pointers. If the phase file and the code disagree, or a decision above can't be met, stop and report instead of choosing silently.
- **A project without memos changes in nothing.** Every phase runs `node scripts/compare/outputs.ts --base main` and shows that every project is byte for byte the same, apart from files a phase names with `--accept` and a recorded decision.
- **The language and the contracts change only by a recorded decision.** Go through the matching list in `project-docs/checklists.md` first, and record the decision in `project-docs/decisions/` in the same pull request.
- **A new diagnostic** is added to `tests/conformance/diagnostics.toml` and `crates/ascribe-core/src/diagnostics.rs`, in the same order, with a conformance case. A check about quality is advice, configurable, with its kind of next step.
- **Behavior changes come with a conformance case** in `tests/conformance/cases/`.
- **`crates/comrak-ascribe` is a fork.** A change to it is marked `// ASCRIBE:` and listed in its `FORK.md`.
- **Project files are read through `ascribe_resolve::FileSystem`.** Libraries don't print, and don't panic on input.
- **The map of the surface** is regenerated when a phase adds to it (`node scripts/surface/inventory.ts`), and its diff is read against the conventions.
- **User-visible changes** update the page under `docs/content/` and add an entry in `changes/` in the same phase.
- No phase history in code or docs. Describe what the code does now.
- Before finishing a phase, all of these pass:

  ```sh
  cargo fmt --all --check
  cargo clippy --workspace --all-targets --locked -- -D warnings
  cargo test --workspace --locked
  pnpm format:check && pnpm lint && pnpm typecheck && pnpm test
  ```

## Later, not in this plan

- Replies, threads, and anything live. Discussion stays in review.
- A memo about several things, or about a folder (decision 14).
- A page-level memo in frontmatter. A reason about a whole page is a memo file.
- Publishing memos. No build option does.
