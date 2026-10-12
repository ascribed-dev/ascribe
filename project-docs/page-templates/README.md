# Page templates

Someone starting a page of a known kind (a how-to, a concept, a troubleshooting page) starts from the right shape, with guidance on what belongs in it. So does an agent asked to write one. The shape and the guidance come from [The Good Docs Project](https://www.thegooddocsproject.dev/template), and stay current with it. The idea is in the [proposal](../proposals/page-templates.md). This is the plan for building it.

## Who it's for

- **A writer starting a page,** who today copies a neighbor.
- **An agent asked for a page,** which today gets a type's frontmatter and nothing about what follows it.
- **A team starting a project,** which today begins from one line, `spec = "0.1"`, and a block copied out of getting started.

## What exists today

- **A page type is fields, not a body.** `[types.<name>]` in `ascribe.toml` has `files` and a frontmatter schema (`docs/content/reference/content-model.md` §4).
- **`ascribe agents prompt new-page`** writes a new page's frontmatter for a type and where its file goes. The plugin's `ascribe:new-page` command wraps it. The surface audit's finding 59 is about that command needing the type before it can ask.
- **One thing ships inside the binary and is written out once as the project's own:** the Vale preset, by `ascribe vale eject` (`crates/ascribe-check/src/prose/eject.rs`). It refuses when its files are there already, and exits 0 or 2.
- **One command edits `ascribe.toml` in place,** keeping its comments and order: `ascribe permalinks retire`.
- **`\{` is a literal brace** (SPEC §3, the escapes table), so `\{Task name}` is ordinary text that shows as `{Task name}`.
- **Checks about quality are advice** a project can raise, turn off, or acknowledge, each with a kind of next step.
- **Generated files have a pattern:** a test fails when one is stale, and `ASCRIBE_BLESS=1` rewrites it (`ARCHITECTURE.md`). A scheduled job that follows another repository and opens a pull request exists too, in `examples/docs-repository/.github/workflows/update-sources.yml`.
- **No construct holds text that's never published.** The [memos proposal](../proposals/memos.md) designs one, `@memo`, and it isn't built.
- **Nothing starts a project or a page for a person** (the surface audit's finding 53), and every Astro site rewrites the same four files (finding 29).

**In The Good Docs Project,** release v1.6.0, tagged 11 June 2026: 25 templates in three packs, a guide with each, an `index.json` naming each template's files, and the MIT No Attribution license. Their repository is on GitLab, `gitlab.com/tgdp/templates`. The copy under their GitHub organisation is archived, with tags only to v0.2.0, and isn't the source. GitLab's tags can be read without signing in, and its releases can't. Checked unchanged, six Core templates gave 5 errors and 18 warnings, from two causes: placeholders in curly brackets, and each template's relative link to its guide.

## Decisions

These are settled. Don't reopen them in a phase; if one can't be met, stop and report.

1. **Convert, don't rewrite.** A script writes Ascribe's copies from a pinned release of theirs. Nobody edits a converted file. A sentence of theirs that's wrong is fixed in their repository.
2. **Three layers, kept apart:** their template and guide, converted; a few structural rules of ours, as data the script reads; and Ascribe's notes on a type, written by hand in a file of their own.
3. **A structural rule that no longer matches fails loudly.** The script stops, naming the template and the place.
4. **Placeholders are their curly brackets, escaped.** Their wording isn't touched.
5. **A leftover is found by comparing a page with its type's template,** with nothing added to the language. It only knows the template as it is now.
6. **The starter set ships inside the binary,** and gets into a project by being copied, once. From then on the files are the project's, and nothing in Ascribe overwrites them.
7. **A type names its template and its guide,** as files in the project, outside its content. A type with neither works as it does today.
8. **One construct for text that's never published,** shared with notes. Templates don't get their own.
9. **A template is a page with things to fill in.** No conditions, loops, or variables beyond the title.
10. **What ships first is the Core pack, less the README:** concept, how-to, tutorial, reference, troubleshooting, and release notes.
11. **Credit is given, though the license asks for none.** Each converted file says where it came from and at which release.
12. **Ascribe's own docs use the types.**
13. **The command is `ascribe templates eject`,** and `ascribe templates status` says what's changed upstream since the copy, by the surface audit's conventions.
14. **A type names its files with two keys: `template`, one path, and `guide`, a list of paths,** both from the project's folder and outside the content root. The guide and Ascribe's notes are two entries of the list, in the order to read them, so `templates status` can show the guide's upstream changes alone. `guide` is always a list, as `files` is, since no key in the content model takes two shapes. There's no `notes` key: it would give "note" a fourth meaning.
15. **The copied files go in `templates/`, beside `ascribe.toml`,** or where `--to <DIR>` says, a path from the current folder. Not under `.ascribe/`, which the docs say can be deleted: these are files the team edits and commits.
16. **`ascribe templates eject [NAME]...` checks everything first and writes nothing when anything is refused.** A type declared already or a file in the way is named, nothing is written, and it exits 2, as `ascribe vale eject` does. With no name it takes the whole set. It edits `ascribe.toml` in place, keeping comments and order, giving each type `files = ["<name>/**"]`, `template`, `guide`, and its frontmatter, and says so. `--format text|json`. `ascribe templates list` reads only the binary.
17. **A placeholder is a run of literal text that an escaped brace opens and its closing brace ends,** found by reading the template as any page is read. The check, `template-placeholder-left`, is advice, configurable, next step `write`, comparing canonical form and ignoring how a line was wrapped. A placeholder changed by a word isn't found: the check finds what was never touched.
18. **A template's instructions stay as quoted paragraphs until `@memo` exists,** and the same check reports one left whole in a page, with a message that says it's an instruction. Nothing waits on the memos plan, and a forgotten instruction is reported before it's published.
19. **Their release is pinned in one file the script and the test both read; their source is fetched into an ignored folder, never committed; Ascribe's copies are committed in the crate that carries them,** each with a first line saying it's generated, from what, and at which release; the structural rules are a data file beside the script; and Ascribe's notes are hand-written files the script never opens.

The options each of 14 to 19 was chosen from are in [phase 1](phase-1-decisions.md).

Six more, from a review that read their v1.6.0 files against the plan, before phase 2 started. Each says what a decision above left unsaid, and two amend one.

20. **A placeholder can run across blocks, and is one placeholder.** Their how-to's "See also" opens a brace in a paragraph and closes it three list items later. Inline text doesn't cross a block, so decision 17's "run of literal text" is read block by block: in the block where the brace opens, the placeholder runs to its closing brace or to the block's end; it takes every block after that whole; and in the block that holds the closing brace, from the block's start to the brace. The conversion escapes the one opening brace and changes nothing else. The check compares each piece and reports the placeholder once, where its first piece is found.
21. **A template's title is its frontmatter's, and a field left as the template has it is a placeholder left.** Every one of their templates opens with a level-1 heading, `# Title` in the how-to and `# Release notes - {Product name} {Release number - 0.0.0}` in the release notes. Ascribe takes a page's title from its frontmatter, and a site's layout draws the heading. So the conversion moves that heading's text into `title`, as a quoted string with its braces as they are, and takes the heading out of the body. Decision 17's definition is for a page's body. In frontmatter the rule is plainer: a field whose value is the same as its type's template has, braces or not, was never filled in.
22. **What a template says about itself goes.** Each opens with a block quote, the link to its guide and then a paragraph about the template and its `{curly brackets}`, and closes with another. Both are about the template and not about the page a writer is making, so the conversion removes them whole. Decision 18's instructions are the ones inside a section, about that section.
23. **A converted template carries no line saying where it came from,** which amends decisions 11 and 19: the credit is in each guide, in `templates.lock`, and on the docs' page, and not in a template. A page copied from a template carries every line the template has, and nothing unpublished exists to hold one. The guide says it, in words, since a guide isn't a page. And `ascribe templates eject` writes one small file under `templates/`, `templates.lock`, listing each file it wrote with their release, Ascribe's version, a hash of what was written, and their index's summary of the type. It's Ascribe's file, as `ascribe.lock` is: written by the command, committed with the templates, and not edited. `ascribe templates status` reads it.
24. **Starting a project is `ascribe init`, and `ascribe templates eject` always needs a project to add to.** A command named for writing templates out that also wrote a project's first `ascribe.toml` would have two meanings, which the surface audit's conventions exist to catch, and deciding it after `eject` is documented would mean a rename. `ascribe init` writes the first `ascribe.toml`, and with `--templates` takes the starter set too, by calling what `eject` calls. Phase 5 builds it. Until then `eject` says there's no project and how to make one by hand.
25. **Their repository is read by its tags,** at `gitlab.com/tgdp/templates`. The pin is a tag and its commit. What changed between two releases is the diff between their tags: the index's `last_updated` isn't a guide to it, since the how-to's is 15 December 2022, four releases before v1.6.0.

## The thread

**The use case:** in a project, `ascribe templates eject how-to` writes the how-to template and its guide and declares the type; a page copied from the template and half filled in makes `ascribe check` say which of the template's placeholders are still there.

**The layers it crosses:**

| Layer | Where | What the thread needs from it |
|---|---|---|
| Their repository | A pinned release, read by a script | One template and its guide |
| The conversion | `scripts/`, and the committed copies | The how-to, as a valid Ascribe page, with a test that it matches the release |
| The binary | `ascribe-cli`, and the crate that holds the set | The copies, carried inside it |
| The content model | `ascribe-model`, `ascribe.toml` | `template` and `guide` on a type |
| The command | `ascribe-cli` | Writing the files, and the type into `ascribe.toml` in place |
| Check | `ascribe-check`, the registry | One advice check, which reads a file outside the content |

**The seam it tests** is text Ascribe didn't write becoming a page Ascribe accepts, mechanically, and staying comparable afterwards. Two assumptions meet there. One is that a script with a short table of rules can turn their Markdown into a page that passes `ascribe check` and `ascribe fmt` unchanged, without a person touching it. The other is that "the same text as a placeholder in the template" is an exact comparison once both have been through the formatter and a writer's editor. If the first fails, the set has to be maintained by hand, and following upstream becomes a merge. If the second fails, a leftover needs a construct of its own, which the proposal chose not to add.

**Left out, and where it comes back:**

| Left out | Phase |
|---|---|
| The other five templates, their rules, and Ascribe's notes on each | 3 |
| Starting a page: the prompt, the editor's action, the rules for agents | 4 |
| A project with no `ascribe.toml` | 5 |
| Following a new release of theirs; `ascribe templates status` | 6 |
| Ascribe's own docs taking the types | 7 |

After phase 2, a page is started from a template by copying the file. The command reference says so while that's true.

## Phases

| Phase | Result |
|---|---|
| [1: Decisions](phase-1-decisions.md) | The maintainer settles the keys on a type, where the files go, what counts as a placeholder, and what carries a template's instructions until `@memo` exists. |
| [2: The thread](phase-2-thread.md) | `ascribe templates eject how-to` gives a project a how-to template and guide, and `ascribe check` reports a placeholder left in a page. |
| [3: The six](phase-3-six.md) | The whole starter set: six templates, six guides, Ascribe's notes on each, and a content model that declares the six types. |
| [4: Starting a page](phase-4-start.md) | A person starts a page from a type in the editor, and an agent gets the template and is sent to the guide. |
| [5: A project from nothing](phase-5-project.md) | In an empty folder, `ascribe init` gives a first `ascribe.toml`, and the starter set with it when asked, and getting started points at it. |
| [6: Following their releases](phase-6-upstream.md) | A new release of theirs arrives as a pull request to read, and a project can see what's changed since its copy. |
| [7: Ascribe's own docs](phase-7-docs.md) | The docs use the types, and the templates have met pages that already exist. |

Phase 1 comes first because phase 2 ships two keys in `ascribe.toml`, a command, and a diagnostic. After the thread the order is by risk: phase 3 puts the conversion through five more inputs, which is where a rule could prove not to be mechanical. Phases 4 and 5 widen who can use it, phase 6 is the upkeep, and phase 7 is the test on real pages.

Each phase's result is something a person can run or open. None builds a layer for a later phase to use.

## Carved out, and waiting

- **A template's instructions as text that's never published.** They wait for `@memo`, which the [memos proposal](../proposals/memos.md) designs and nothing has built. Until it exists they're carried as phase 1 decides. When it does, the conversion gains one rule, the set is regenerated, and a project's existing copies are its own to change or not. That's a small change to this plan's output and not a phase of it.
- **A site's starter files** (the surface audit's finding 29): the collection, the route, a layout that loads the elements, and the sidebar's list. They'd be written by `@ascribed/astro`, not the binary, into an Astro project, which is a different package, a different seam, and a design that depends on navigation, which has no proposal. It has an issue of its own, [#344](https://github.com/ascribed-dev/ascribe/issues/344), and this plan's command is named so a second thing to eject fits beside it.
- **Checking a page's shape against its type** ("a how-to has a procedure"). It needs a way to describe a page's outline in the content model. Later, as the proposal says.

## The other plans

- **The surface audit.** Its conventions name the command and its verb, say what `--format` words it takes and what it exits with, and say a path given to a command is from the current folder. Its finding 53 is phase 5, and its finding 59, the plugin's `new-page` command, is in phase 4's way and should be read first.
- **Search and answers.** A template's page has a title and a description from its first line of frontmatter, so pages made from one start with what that plan's checks ask for.
- **Memos.** This plan is the first user of `@memo`, and doesn't wait for it.

## Rules for every phase

- Branch before committing; never commit to `main`.
- Read the current code before the phase file's pointers. If the phase file and the code disagree, or a decision above can't be met, stop and report instead of choosing silently.
- **Nobody edits a converted file.** A change to one is a change to the script, to a rule, or to their repository. The test that compares the copies with the pinned release fails otherwise.
- **The language and the contracts change only by a recorded decision.** Go through the matching list in `project-docs/checklists.md` first, and add the decision to `project-docs/decisions.md` in the same pull request.
- **A new diagnostic** is added to `tests/conformance/diagnostics.toml` and `crates/ascribe-core/src/diagnostics.rs`, in the same order, with a conformance case. A check about quality is advice, configurable, with its kind of next step.
- **Project files are read through `ascribe_resolve::FileSystem`.** A template is a project file outside the content root, and how it's reached is phase 2's to get right once. Libraries don't print, and don't panic on input.
- **Every converted page passes `ascribe check` and is unchanged by `ascribe fmt`.** A test runs both on the whole set.
- **The map of the surface** is regenerated when a phase adds to it (`node scripts/surface/inventory.ts`), and its diff is read against the conventions.
- **User-visible changes** update the page under `docs/content/` and the unreleased section of `CHANGELOG.md` in the same phase.
- No phase history in code or docs. Describe what the code does now.
- Before finishing a phase, all of these pass:

  ```sh
  cargo fmt --all --check
  cargo clippy --workspace --all-targets --locked -- -D warnings
  cargo test --workspace --locked
  pnpm format:check && pnpm lint && pnpm typecheck && pnpm test
  ```

## Later, not in this plan

- The rest of their templates: quickstart and the installation guide first, then the API pair.
- Templates for a fragment or a section.
- Converting a project's existing pages to a type.
- Checking that Ascribe's glossary, its Vale preset, and its changelog agree with their guidance on each.
