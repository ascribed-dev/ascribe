# Proposal: memos

A proposal, not a plan. It says what the feature is and what it would look like, so the idea can be judged before anything is designed in detail. It comes from section 15 of the [brainstorm](../brainstorm.md), which also raised a cache and a search index. Those are proposals of their own: they hold derived data, and share no layer with this one.

It's built: [Leave a memo for whoever edits next](../../docs/content/writing/memos.md) is the guide, and the plan it was built from is in [memos/](../memos/README.md).

## The goal

The people and agents who maintain a docs set can find out why a page is written the way it is, at the place they're about to change.

Why this page uses a table and not prose. Why the install steps are in this order. What a subject expert said. None of it belongs in front of a reader, so today it's in a wiki, a pull request nobody will find again, or someone's head.

The biggest payoff may be for agents. "Why is this written this way" is what an agent editing a page most needs and can't work out. A reason Ascribe hands over with the page does more than the same reason in a wiki.

## What exists

- **`@intended` is the closest thing.** A line in the page with free text, bound to the block below it, that no output carries ([SPEC §4.9](../../SPEC.md#49-intended)). Its reason is a line primary: the rest of the line, never parsed as inline content. Build resolution removes it (`drop_acknowledgements` in `crates/ascribe-resolve/src/build/mod.rs`), so no emitter, neither renderer, and not `ascribe diff` ever sees one. A page whose only edit is an `@intended` line is "no changes" to `ascribe diff`. But it's about one check's finding, and one that excuses nothing is reported. It can't hold a reason that has no check.
- **An HTML comment isn't private.** What each place does with `<!-- why a table -->` in a page today:

  | Where | An HTML comment |
  |---|---|
  | Site output | Kept as written, as a block or inside a paragraph |
  | The published page, the preview, the HTML report | In the page's HTML. Not shown, and there for anyone who views the source |
  | Plain output, `llms.txt` pages, `ascribe render` | Dropped (SPEC §9.4) |
  | JSON output | Kept: an `html` block or inline holding the comment's text |
  | `ascribe diff` and review | A reworded comment is a changed block, counted and marked like a change readers see |

  It also binds to nothing. Ascribe can't say which block a comment is about, so it can't hand one over with that block. And it has a real use that should stay as it is: Vale's own comments, which Ascribe passes through to Vale.
- **Review anchors comments to rendered blocks,** with "outdated" and "detached" states (`packages/review/src/place/`). It follows a comment through git: a line at a commit, mapped to the working tree. A note has no commit to start from, so the states carry over and the mechanism doesn't.
- **`ascribe refs` already has a way to name anything in a project:** a page or fragment (`keys.md`), a heading (`keys.md#rotate-keys`), `phrase:<key>`, `feature:<key>`, `term:<id>`, and the rest.
- **Nothing else.** No directive holds text that's never published, and no file in a project is about another file.

## Three kinds, kept apart

Code keeps three kinds of commentary apart, and memos do too:

| Kind | Where it lives | Like | In this proposal |
|---|---|---|---|
| A short "why" about one block or section | In the page, beside it, never in any output | A code comment | `@memo` |
| A longer explanation of a page's or a section's approach | A file of its own | A decision record | A memo file |
| Discussion | Pull request review, which Ascribe already shows beside blocks | An issue thread | Nothing new |

Memos aren't threads. There are no replies, and nothing here is live.

They're also authored, not derived. A memo is its own source of truth, and deleting one loses it. So memos share nothing with a cache, whose one rule is that deleting it changes nothing but time.

## A memo in the page

```markdown
@memo: A table, not prose: readers compare the three plans side by side.
| Plan | Seats | Price |
|---|---|---|
```

A new built-in directive, a sibling of `@intended`. It isn't an extension of it: an acknowledgement names a check and is reported when the check has nothing to say, and a memo names nothing and is never stale by Ascribe's lights. One directive doing both would need a `check` that's sometimes required. What they share is the machinery, which is already there: the line primary, the binding, and the one place in build resolution that removes them.

- **Line form.** The text is a line primary, as `@intended`'s reason is: the rest of the line, plain text.
- **Container form,** for more than a line. By the rule every directive follows (SPEC §3.5), a colon with nothing after it opens a container:

  ```markdown
  ## Prerequisites
  @memo:
  This section is optional. Delete it if the task needs nothing set up first.

  Link each prerequisite to the page that explains it; don't repeat the steps.
  @end
  ```

- **What it's about** follows `@available`'s rule, so there's one rule to learn: at the top of a section it's about the section, and anywhere else it's about the block below it. Several stack. A memo with nothing after it is the error any following-block directive gets.
- **Its contents are text, and nothing reads them.** A container's lines are kept as written, as a code block's are: the parser keeps them as raw text, so nothing inside is a block, a directive, or a heading ([plan decision 21](../memos/README.md#decisions)). A `{phrase}` in one isn't a phrase, a link isn't checked and isn't a use of its target, an `@include` isn't followed, a heading gets no id, and Vale isn't given the text. Otherwise a memo could break a build, or keep an unused phrase alive.
- **No attributes.**
- **`ascribe fmt`** treats it as it treats `@intended`: the directive touches its block, and its text is left alone.

### Never published, concretely

A memo is removed where `@intended` is, when a page is resolved for a build. Everything downstream is then safe by construction:

| Place | What it has of a memo |
|---|---|
| Site, plain, and JSON outputs; `llms.txt` pages | Nothing. Each is byte for byte what it would be without the memo |
| Both HTML renderers | Nothing. The site output's markup doesn't change, so `tests/render/` gains no fixture and the two can't disagree about it |
| The editor's preview, the HTML report | Nothing |
| Source anchors | No anchor names a memo's lines. The blocks around it keep their own |
| A fragment's memo | Removed in every page that includes the fragment |
| `ascribe diff` and review | A change to a memo alone is no change. The page isn't listed |

The last row is a decision, not a side effect. Review answers "what will readers see differently", and a memo isn't that. The pull request's file diff shows the line, which is where a reviewer of the source looks.

HTML comments stay exactly as they are.

### The same construct serves page templates

The [page templates proposal](page-templates.md) needs a template's instructions to its author to be text that's never published, and leaves the construct to this proposal. It's this one: the conversion writes each instruction as a line memo, naming which paragraphs are instructions in its rules, since nothing in their shape says so ([plan decision 22](../memos/README.md#decisions)).

That proposal finds a leftover placeholder by comparing a page with its type's template. A leftover instruction is found the same way, since a memo's text is in the syntax tree. So a forgotten instruction is reported, and is never published either way. Nothing is added to the language for it.

## A memo file

A longer explanation is a file of its own:

```markdown
---
about: guides/install.md#install-the-agent
quote: Install the agent before you create a project
---

The agent goes first because its installer writes the config file that
`quill init` reads. The other way round, `init` fails with an error that
doesn't mention the agent.

From Dana Okafor (platform), in the 3.4 launch review.
```

- **`about`** names one thing, in the form `ascribe refs` takes: a page or fragment by its path from the content root, a heading by its id, or a content model entry (`phrase:product`, `feature:next`, `term:workspace`, and `dimension:`, `note:`, and `widget:` the same way). One grammar, one parser, and a memo is a kind of use of what it names.
- **`quote`** is optional. It narrows the memo to one block: the block in the named page or section whose source text holds the quote.
- **The body** is CommonMark, shown as written. Ascribe resolves nothing in it.
- **No author and no date.** The commit has both.
- **No build.** A memo is about the source, not about what one build makes of it.

A memo about a heading with no `@id` breaks when the heading is reworded, as a link to it does. The answer is the same: give the heading an `@id`.

### Where they live

| | Folder | Shared by |
|---|---|---|
| Team memos | `memos/`, beside `ascribe.toml` | Committing them |
| Personal memos | `.ascribe/memos/`, which projects already ignore | Nothing. They stay on one machine |

The format is the same in both. `memos/` must not be inside the content root, where an `.md` file would be a page; that's the rule `sources/<name>/` already has. A file's name means nothing to Ascribe, and subfolders are the team's to arrange.

Memo files are read through `ascribe_resolve::FileSystem`, as a source's files are.

### Working out where a memo is

The position is worked out each time the memo is read. Nothing is written back, and no line number is stored.

| State | When | What happens |
|---|---|---|
| Placed | `about` exists, and the quote is found (or there's none) | The memo is on that block, or on the page, section, or entry |
| Outdated | `about` exists, and the quote isn't found | The memo stays on the page or section, marked outdated, with what it quoted |
| Detached | `about` names nothing | Reported. The memo is listed, and on nothing |

The states are review's. The matching is deliberately dumber than review's: the quote is in the text, with whitespace runs treated as one space, or it isn't. No similarity score, so two machines never disagree about where a note is. A quote is matched against the source of the file `about` names, so a note about a fragment's text names the fragment.

What's reusable, said plainly:

- **In Rust:** the source index's headings and ids, `ascribe refs`'s target grammar and its reverse references, every block's file and span, and the editor's rename, which already rewrites links when a file or an id is renamed and would rewrite `about` too.
- **In TypeScript:** the source anchor strings (`path:first-last`), which are how a placed memo would be put beside a rendered block if a view ever does that. Review's placement itself isn't reusable. It starts from a commit.

### Out of git's way

| Problem | Avoided by |
|---|---|
| Every content edit rewrites the memos | No stored positions. A memo file changes only when someone edits the memo |
| Merge conflicts | One file per memo. Two people adding memos never touch the same file |
| Pull requests full of memos | A folder of their own |
| Memos pointing at nothing | Rename rewrites `about`; `ascribe check` reports the rest |
| Growth without limit | No replies, and advice on a long memo. Scope is the real size control |

### Behind one interface

A memo's format is the contract. Where memos come from sits behind one interface in the binary, with two implementations, the committed folder and the personal one.

Anything else comes later and doesn't change what a memo is. A server for people without a checkout isn't part of this proposal; the brainstorm lists it among the things that might be charged for. The binary has no HTTP client and won't gain one, so a later store either writes memo files for the binary to read, or lives in TypeScript where GitHub access does.

## Checks

Under the [content checks](https://github.com/ascribed-dev/ascribe/blob/76a46b26090c2dd0efbd7c918a29d58754f8eace/project-docs/content-checks/README.md) contract: each has a kind of next step, a level a project can change in `[checks]`, and a fix paragraph. Each is reported on the memo file, at its `about` or `quote` line. The names are for illustration.

| Check | Reports | Level | Next step |
|---|---|---|---|
| `memo-invalid` | Frontmatter that doesn't parse, no `about`, a key that isn't one of the two, or a `memos/` folder inside the content root | Error | `write` |
| `memo-detached` | `about` names a page, heading, or entry that doesn't exist | Warning | `choose`: the closest ids on the page, or delete the memo |
| `memo-outdated` | The quote isn't in the page or section any more | Advice | `write`: the prompt carries the memo and the section as it is now |
| `memo-long` | A body over a limit, `[checks.memo-long] limit`, 2,000 characters by default | Advice | `write` |
| `html-comment` | An HTML comment in a page, which reaches the published page's source. Vale's are exempt: Ascribe already recognizes them and passes them to Vale | Advice | `review`: rewrite it as `@memo`, or acknowledge it with `@intended` where it's for another tool |

The first four aren't `review` checks, so none is acknowledged with `@intended`: a note that's wrong is edited or deleted. `html-comment` is one, since a comment written for another tool is right as it is, and it's reported on the page, not on a note.

They answer from the source index, so they run while writing and in `ascribe check`. `ascribe check` reads team memos only: what it reports can't depend on whose machine it runs on. Problems with personal memos show in the editor.

## How a memo reaches someone

**A memo in the page needs no delivery.** It's in the file. Anyone who opens the page to edit it, person or agent, reads it on the way to the block. That's most of the value, for none of the machinery.

**A memo file has to be brought to the page.** Minimally:

- **`ascribe refs`** lists the memos about a target, with the links and includes it lists now. So does the editor's Used by view, which is where someone about to delete a heading looks.
- **`ascribe outline`** lists the memos about a page, by heading: each one's file, state, and first line. Its MCP tool, `ascribe_outline`, returns the same.
- **In the editor,** a lens on a heading or block that has memos, which opens them, and an action that creates one with `about` and `quote` filled in from the cursor. Nobody will write that frontmatter by hand.
- **The prompts** that name a page (`fix`, `review`, and Prompt agent's) add a line per note about it: the file's path, and what it's about. The `review` prompt says more for a changed section that has a note: check the change still agrees with it.
- **The rules `ascribe agents sync` writes** gain two sentences, in a project that has memos: read a block's `@memo` before changing it and keep the two together, and `ascribe outline <page>` lists a page's memos.

Agents' output is capped and compact by decision ([decisions](../decisions.md) 51 and 56), and the rules have a budget of 4,000 characters. Memos fit that by never being pasted: Ascribe hands over paths and first lines, in lists that are cut the way every list is (`truncated`, `shown`, `total`, `next_command`), and the agent reads the file. A file an agent reads itself arrives whole.

Personal memos reach that person's editor and that person's agent, marked as personal.

A search over resolved pages, a separate proposal, could index memos beside pages.

## What breaks

Almost nothing. A project's own directives need a hyphen in their names, so none can be called `memo`. A project with a folder named `memos/` beside `ascribe.toml` that holds something else would have its `.md` files read as memos, and reported.

## What this leaves out

- **Replies, threads, and anything live.** Discussion stays in review.
- **A server,** and people without a checkout.
- **Memos anywhere a reader could see them.** No build option publishes them.
- **Showing them in the preview or in review.** A view for authors, with notes beside the blocks they're about, is possible later: the anchors exist. The preview is what readers see, so it would be a switch, off by default.
- **A memo about several things,** or about a folder. One memo, one target.
- **A page-level memo in frontmatter.** `@intended` has a frontmatter form because checks report on whole pages. A reason about a whole page is a memo file.
- **"The block changed and its memo didn't."** It's `ascribe drift`'s question asked of prose, and worth asking once memos are in use.
- **Converting HTML comments.**

## What to build first

The memo in the page, alone. Page templates wait on it, it's useful without memo files, and it's the smaller half.

A later plan can start from this:

- **Layers.** The in-page memo: the language (`SPEC.md`, the schema in `ascribe-core`), `ascribe-syntax` and perhaps the comrak fork, `ascribe-resolve`, `ascribe-fmt`, `ascribe-check`, the language server, and the rules in `ascribe-query`. Memo files add `ascribe-resolve` (the store and placing), `ascribe-check`, `ascribe-query`, the commands, the MCP tools, and the extension. `ascribe-emit`, the elements, Astro, and review are touched by neither: their part is tests that show nothing changed.
- **The riskiest seam** isn't "never published". Removing a comment at resolution is one line beside `@intended`'s, and `scripts/compare/outputs.ts` proves it. It's "nothing reads the contents". The source index, the file-level checks, `ascribe refs`, the prose handed to Vale, and the formatter all read the tree before resolution, and the fork decides a container's block structure before Ascribe sees it. The line form doesn't test this, since a line primary is already inert.
- **The thinnest use case** therefore uses the container form: one page with a two-paragraph `@memo` that holds a link to a file that doesn't exist and a `{word}` in braces. `ascribe check` reports nothing about either, `ascribe fmt` leaves it alone, `ascribe build` writes the three outputs without it, and `ascribe diff` says the page didn't change.

## Settled

- **Authored and derived data don't share a mechanism.** Memos have nothing to do with a cache.
- **Three kinds of commentary:** a memo in the page, a memo file, and discussion in review. No replies.
- **Personal memos stay on one machine,** under `.ascribe/`. Team memos are committed.
- **Committed memos stay out of git's way:** anchored by id and a quote, positions never written back, one file per memo, a folder of their own, a check for a lost anchor, and advice on length.
- **The format is the contract,** and where memos are stored is behind one interface. Committed files first.
- **One construct for text that's never published,** shared with page templates.
- **The memo in the page is built first.**
- **Both things are memos: `@memo` in a page, and a memo file in `memos/`.** One word for one thing at two lengths, used nowhere in Ascribe today, meaning a record kept for whoever edits next. "Note" is the callout, in `@note`, `[notes.<type>]`, and `note:<type>` targets, and "comment" is what the review guide calls a review thread, fifty times over, which the `review` prompt would name beside an in-page one. Decided by the maintainer on 11 October 2026.
- **A memo can't say it must be gone before the page is finished.** No attribute, and no to-do marker. Page templates find a leftover instruction by comparing with the template, which needs nothing new; a to-do an author writes by hand is a memo that reads as one, and a memo left behind costs readers nothing, since it's never published. A check that a memo remains would fire on every memo meant to stay. An attribute is additive and can come later, once memos are in use and a case for it has been met.
- **Once `@memo` exists, an HTML comment in a page is advice,** `html-comment` in the table above: it reaches the published page's source, and an author who wrote one probably thinks it doesn't. It's a `review` check, acknowledged with `@intended` where the comment is for another tool, and turned off in `[checks]` by a project that uses one everywhere. Vale's comments are exempt, since Ascribe already recognizes them. It ships with the in-page memo, since the fix it names needs it.
- **A memo is about one thing.** `quote` narrows to one block, and the placed, outdated, and detached states are decided against one target; a list would need each state per target, and a quote that could match in several places. What an expert said about three pages is usually about something the pages share, which `about` can already name: a feature, a term, a phrase, or a heading in a fragment they include. Three pages with nothing shared get three short memos, or one memo and two that point at it. Revisit if real memos show the shared thing is often missing.
