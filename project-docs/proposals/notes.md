# Proposal: notes

A proposal, not a plan. It says what the feature is and what it would look like, so the idea can be judged before anything is designed in detail. It comes from section 15 of the [brainstorm](../brainstorm.md), which also raised a cache and a search index. Those are proposals of their own: they hold derived data, and share no layer with this one.

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

Code keeps three kinds of commentary apart, and notes do too:

| Kind | Where it lives | Like | In this proposal |
|---|---|---|---|
| A short "why" about one block or section | In the page, beside it, never in any output | A code comment | `@comment` |
| A longer explanation of a page's or a section's approach | A file of its own | A decision record | A note file |
| Discussion | Pull request review, which Ascribe already shows beside blocks | An issue thread | Nothing new |

Notes aren't threads. There are no replies, and nothing here is live.

They're also authored, not derived. A note is its own source of truth, and deleting one loses it. So notes share nothing with a cache, whose one rule is that deleting it changes nothing but time.

## A comment in the page

```markdown
@comment: A table, not prose: readers compare the three plans side by side.
| Plan | Seats | Price |
|---|---|---|
```

A new built-in directive, a sibling of `@intended`. It isn't an extension of it: an acknowledgement names a check and is reported when the check has nothing to say, and a comment names nothing and is never stale by Ascribe's lights. One directive doing both would need a `check` that's sometimes required. What they share is the machinery, which is already there: the line primary, the binding, and the one place in build resolution that removes them.

- **Line form.** The text is a line primary, as `@intended`'s reason is: the rest of the line, plain text.
- **Container form,** for more than a line. By the rule every directive follows (SPEC §3.5), a colon with nothing after it opens a container:

  ```markdown
  ## Prerequisites
  @comment:
  This section is optional. Delete it if the task needs nothing set up first.

  Link each prerequisite to the page that explains it; don't repeat the steps.
  @end
  ```

- **What it's about** follows `@available`'s rule, so there's one rule to learn: at the top of a section it's about the section, and anywhere else it's about the block below it. Several stack. A comment with nothing after it is the error any following-block directive gets.
- **Its contents are text, and nothing reads them.** A container's lines are kept as written, as a code block's are. A `{phrase}` in one isn't a phrase, a link isn't checked and isn't a use of its target, an `@include` isn't followed, a heading gets no id, and Vale isn't given the text. Otherwise a comment could break a build, or keep an unused phrase alive.
- **No attributes.**
- **`ascribe fmt`** treats it as it treats `@intended`: the directive touches its block, and its text is left alone.

### Never published, concretely

A comment is removed where `@intended` is, when a page is resolved for a build. Everything downstream is then safe by construction:

| Place | What it has of a comment |
|---|---|
| Site, plain, and JSON outputs; `llms.txt` pages | Nothing. Each is byte for byte what it would be without the comment |
| Both HTML renderers | Nothing. The site output's markup doesn't change, so `tests/render/` gains no fixture and the two can't disagree about it |
| The editor's preview, the HTML report | Nothing |
| Source anchors | No anchor names a comment's lines. The blocks around it keep their own |
| A fragment's comment | Removed in every page that includes the fragment |
| `ascribe diff` and review | A change to a comment alone is no change. The page isn't listed |

The last row is a decision, not a side effect. Review answers "what will readers see differently", and a comment isn't that. The pull request's file diff shows the line, which is where a reviewer of the source looks.

HTML comments stay exactly as they are.

### The same construct serves page templates

The [page templates proposal](page-templates.md) needs a template's instructions to its author to be text that's never published, and leaves the construct to this proposal. It's this one: the conversion writes each instruction as a `@comment` container.

That proposal finds a leftover placeholder by comparing a page with its type's template. A leftover instruction is found the same way, since a comment's text is in the syntax tree. So a forgotten instruction is reported, and is never published either way. Nothing is added to the language for it.

## A note file

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

- **`about`** names one thing, in the form `ascribe refs` takes: a page or fragment by its path from the content root, a heading by its id, or a content model entry (`phrase:product`, `feature:next`, `term:workspace`). One grammar, one parser, and a note is a kind of use of what it names.
- **`quote`** is optional. It narrows the note to one block: the block in the named page or section whose source text holds the quote.
- **The body** is CommonMark, shown as written. Ascribe resolves nothing in it.
- **No author and no date.** The commit has both.
- **No build.** A note is about the source, not about what one build makes of it.

A note about a heading with no `@id` breaks when the heading is reworded, as a link to it does. The answer is the same: give the heading an `@id`.

### Where they live

| | Folder | Shared by |
|---|---|---|
| Team notes | `notes/`, beside `ascribe.toml` | Committing them |
| Personal notes | `.ascribe/notes/`, which projects already ignore | Nothing. They stay on one machine |

The format is the same in both. `notes/` must not be inside the content root, where an `.md` file would be a page; that's the rule `sources/<name>/` already has. A file's name means nothing to Ascribe, and subfolders are the team's to arrange.

Note files are read through `ascribe_resolve::FileSystem`, as a source's files are.

### Working out where a note is

The position is worked out each time the note is read. Nothing is written back, and no line number is stored.

| State | When | What happens |
|---|---|---|
| Placed | `about` exists, and the quote is found (or there's none) | The note is on that block, or on the page, section, or entry |
| Outdated | `about` exists, and the quote isn't found | The note stays on the page or section, marked outdated, with what it quoted |
| Detached | `about` names nothing | Reported. The note is listed, and on nothing |

The states are review's. The matching is deliberately dumber than review's: the quote is in the text, with whitespace runs treated as one space, or it isn't. No similarity score, so two machines never disagree about where a note is. A quote is matched against the source of the file `about` names, so a note about a fragment's text names the fragment.

What's reusable, said plainly:

- **In Rust:** the source index's headings and ids, `ascribe refs`'s target grammar and its reverse references, every block's file and span, and the editor's rename, which already rewrites links when a file or an id is renamed and would rewrite `about` too.
- **In TypeScript:** the source anchor strings (`path:first-last`), which are how a placed note would be put beside a rendered block if a view ever does that. Review's placement itself isn't reusable. It starts from a commit.

### Out of git's way

| Problem | Avoided by |
|---|---|
| Every content edit rewrites the notes | No stored positions. A note file changes only when someone edits the note |
| Merge conflicts | One file per note. Two people adding notes never touch the same file |
| Pull requests full of notes | A folder of their own |
| Notes pointing at nothing | Rename rewrites `about`; `ascribe check` reports the rest |
| Growth without limit | No replies, and advice on a long note. Scope is the real size control |

### Behind one interface

A note's format is the contract. Where notes come from sits behind one interface in the binary, with two implementations, the committed folder and the personal one.

Anything else comes later and doesn't change what a note is. A server for people without a checkout isn't part of this proposal; the brainstorm lists it among the things that might be charged for. The binary has no HTTP client and won't gain one, so a later store either writes note files for the binary to read, or lives in TypeScript where GitHub access does.

## Checks

Under the [content checks](https://github.com/ascribed-dev/ascribe/blob/76a46b26090c2dd0efbd7c918a29d58754f8eace/project-docs/content-checks/README.md) contract: each has a kind of next step, a level a project can change in `[checks]`, and a fix paragraph. Each is reported on the note file, at its `about` or `quote` line. The names are for illustration.

| Check | Reports | Level | Next step |
|---|---|---|---|
| `note-invalid` | Frontmatter that doesn't parse, no `about`, a key that isn't one of the two, or a `notes/` folder inside the content root | Error | `write` |
| `note-detached` | `about` names a page, heading, or entry that doesn't exist | Warning | `choose`: the closest ids on the page, or delete the note |
| `note-outdated` | The quote isn't in the page or section any more | Advice | `write`: the prompt carries the note and the section as it is now |
| `note-long` | A body over a limit, `[checks.note-long] limit` | Advice | `write` |

None is a `review` check, so none is acknowledged with `@intended`. A note that's wrong is edited or deleted.

They answer from the source index, so they run while writing and in `ascribe check`. `ascribe check` reads team notes only: what it reports can't depend on whose machine it runs on. Problems with personal notes show in the editor.

## How a note reaches someone

**A comment needs no delivery.** It's in the file. Anyone who opens the page to edit it, person or agent, reads it on the way to the block. That's most of the value, for none of the machinery.

**A note file has to be brought to the page.** Minimally:

- **`ascribe refs`** lists the notes about a target, with the links and includes it lists now. So does the editor's Used by view, which is where someone about to delete a heading looks.
- **`ascribe outline`** lists the notes about a page, by heading: each one's file, state, and first line. Its MCP tool, `ascribe_outline`, returns the same.
- **In the editor,** a lens on a heading or block that has notes, which opens them, and an action that creates one with `about` and `quote` filled in from the cursor. Nobody will write that frontmatter by hand.
- **The prompts** that name a page (`fix`, `review`, and Prompt agent's) add a line per note about it: the file's path, and what it's about. The `review` prompt says more for a changed section that has a note: check the change still agrees with it.
- **The rules `ascribe agents sync` writes** gain two sentences, in a project that has comments or notes: read a block's `@comment` before changing it and keep the two together, and `ascribe outline <page>` lists a page's notes.

Agents' output is capped and compact by decision ([decisions](../decisions.md) 51 and 56), and the rules have a budget of 4,000 characters. Notes fit that by never being pasted: Ascribe hands over paths and first lines, in lists that are cut the way every list is (`truncated`, `shown`, `total`, `next_command`), and the agent reads the file. A file an agent reads itself arrives whole.

Personal notes reach that person's editor and that person's agent, marked as personal.

A search over resolved pages, a separate proposal, could index notes beside pages.

## What breaks

Almost nothing. A project's own directives need a hyphen in their names, so none can be called `comment`. A project with a folder named `notes/` beside `ascribe.toml` that holds something else would have its `.md` files read as notes, and reported.

## What this leaves out

- **Replies, threads, and anything live.** Discussion stays in review.
- **A server,** and people without a checkout.
- **Notes or comments anywhere a reader could see them.** No build option publishes them.
- **Showing them in the preview or in review.** A view for authors, with notes beside the blocks they're about, is possible later: the anchors exist. The preview is what readers see, so it would be a switch, off by default.
- **A note about several things,** or about a folder. One note, one target.
- **A page-level comment in frontmatter.** `@intended` has a frontmatter form because checks report on whole pages. A reason about a whole page is a note file.
- **"The block changed and its comment didn't."** It's `ascribe drift`'s question asked of prose, and worth asking once comments are in use.
- **Converting HTML comments.**

## What to build first

The comment, alone. Page templates wait on it, it's useful without note files, and it's the smaller half.

A later plan can start from this:

- **Layers.** The comment: the language (`SPEC.md`, the schema in `ascribe-core`), `ascribe-syntax` and perhaps the comrak fork, `ascribe-resolve`, `ascribe-fmt`, `ascribe-check`, the language server, and the rules in `ascribe-query`. Note files add `ascribe-resolve` (the store and placing), `ascribe-check`, `ascribe-query`, the commands, the MCP tools, and the extension. `ascribe-emit`, the elements, Astro, and review are touched by neither: their part is tests that show nothing changed.
- **The riskiest seam** isn't "never published". Removing a comment at resolution is one line beside `@intended`'s, and `scripts/compare/outputs.ts` proves it. It's "nothing reads the contents". The source index, the file-level checks, `ascribe refs`, the prose handed to Vale, and the formatter all read the tree before resolution, and the fork decides a container's block structure before Ascribe sees it. The line form doesn't test this, since a line primary is already inert.
- **The thinnest use case** therefore uses the container form: one page with a two-paragraph `@comment` that holds a link to a file that doesn't exist and a `{word}` in braces. `ascribe check` reports nothing about either, `ascribe fmt` leaves it alone, `ascribe build` writes the three outputs without it, and `ascribe diff` says the page didn't change.

## Settled

- **Authored and derived data don't share a mechanism.** Notes have nothing to do with a cache.
- **Three kinds of commentary:** a comment in the page, a note file, and discussion in review. No replies.
- **Personal notes stay on one machine,** under `.ascribe/`. Team notes are committed.
- **Committed notes stay out of git's way:** anchored by id and a quote, positions never written back, one file per note, a folder of their own, a check for a lost anchor, and advice on length.
- **The format is the contract,** and where notes are stored is behind one interface. Committed files first.
- **One construct for text that's never published,** shared with page templates.
- **The comment is built first.**

## Open questions

- **What are the two things called?** "Note" already means a callout in three places: `@note`, `[notes.<type>]`, and `note:<type>` in `ascribe refs`. And "comment" is what the docs call a review thread. This proposal uses `@comment` and "note file" in a `notes/` folder. Is that the pair, or does the callout's claim on "note" push the files to another word?
- **Should a comment be able to say it must be gone before the page is finished?** Page templates can find a leftover instruction by comparing with the template, which needs nothing new. An attribute would make it work for a page whose template has since changed, and for a to-do an author writes by hand. Is that worth the first attribute?
- **Once `@comment` exists, should `ascribe check` say anything about an HTML comment in a page?** It reaches the published page's source, and an author who wrote one probably thinks it doesn't. Advice would say so; it would also fall on every comment written for another tool, Vale's aside.
- **Can a note be about more than one thing?** What an expert said may cover three pages. One target keeps "detached" simple and each note small; a list means one note instead of three that drift apart.
