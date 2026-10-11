# Phase 2: The thread

Part of [Page templates](README.md). Requires phase 1. A script, Rust, and the docs. This is the plan's steel thread: the thinnest path through every layer, shipped and kept.

The names below are the [README](README.md#decisions)'s decisions 14 to 25.

## Goal

One template, taken from their release to a project, and one page checked against it:

```sh
$ ascribe templates eject how-to
wrote templates/how-to.md
wrote templates/how-to.guide.md
added [types.how-to] to ascribe.toml, for pages under docs/how-to/
$ mkdir -p docs/how-to && cp templates/how-to.md docs/how-to/rotate-keys.md
$ ascribe check
[ASC…] Advice: this page still has its template's `{Task name}`
```

One template, one command, one check. Everything it takes is built to the standard of the rest, and nothing in it is thrown away.

## Context

- [The thread](README.md#the-thread): the layers, and the seam this phase exists to test.
- Phase 1's decisions, in the [README](README.md#decisions): the keys, `templates/`, the command, what a placeholder is, what carries instructions, and where the set lives.
- Their repository at the pinned release: the how-to's template and guide, and `index.json`'s entry for it. Read both whole before writing a rule.
- SPEC §3's escapes, for `\{`; §4.6 for `@steps` and §4.5 for `@note`; and the frontmatter a type requires.
- `crates/ascribe-check/src/prose/eject.rs` and its command: the nearest thing built. What it writes, how it refuses, its result type, and its tests.
- `ascribe permalinks retire`: editing `ascribe.toml` in place.
- `crates/ascribe-model`: how a type is loaded, and how a path in the model is checked against the content root and the project's boundary.
- `crates/ascribe-resolve`: `FileSystem`, and `crates/ascribe-resolve/tests/all/file_reads.rs`.
- `scripts/`: how a script that writes committed files is laid out, with its test and `ASCRIBE_BLESS=1` (`scripts/surface/` is the newest).
- `project-docs/checklists.md`: the lists for a new command and for a change to the content model.

## Design

### The conversion

A script reads the pinned release and writes the how-to's two files into the repository. What it does, in order, each step small enough to predict from the source:

1. **What the template says about itself, removed** (README decision 22): the block quote it opens with, both paragraphs, and the one it closes with. The first holds the link to the guide, which isn't there once the template is alone, and a sentence with `{curly brackets}` in it, which escaped would be reported as a placeholder on every page.
2. **The title moved to frontmatter** (decision 21). The level-1 heading the body opens with becomes `title`, a quoted string with its text as it is, and the heading leaves the body. For the how-to that's `title: "Title"`. A `description` is added beside it, since the type asks for one and their template has none: one placeholder, in Ascribe's words, from the rule file, and the only words in a template that aren't theirs.
3. **Placeholders escaped.** Every opening `{` of theirs in the body becomes `\{`, wording untouched, wherever it is: a paragraph, a list item, a table's cell, a heading. A closing brace is left as it is. A brace inside a code span or a fenced block isn't a placeholder and isn't escaped; none of the six has one, so the script's own test has a made-up case for it.
4. **A placeholder that runs across blocks is left as their text has it** (decision 20): one brace escaped where it opens, in the "See also" paragraph, and its closing brace three list items later, untouched.
5. **The structural rules for this template,** from the data file. For the how-to that's likely one: its numbered procedure gains `@steps`. Each rule names the template and the text it applies at. One that doesn't find its place stops the script.
6. **Instructions inside a section left as they are** (decision 18).
7. **`ascribe fmt`'s canonical form.**

The template carries no line about where it came from (decision 23): a page copied from it would carry the line too.

The guide is converted too: it's a document to read, not a page, so it needs less. It isn't in a project's content, and nothing checks it as a page. It opens with a line that says it's generated, from their repository on GitLab, at which tag, and under which license.

**This is the seam, first half.** If the how-to can't be made valid by steps like these, with no step that's "then fix it by hand", stop and report.

### The set, in the repository and the binary

- The pin is a tag and its commit, at `gitlab.com/tgdp/templates` (decision 25). The script fetches that commit's files once, into an ignored folder.
- The copies are committed, in the crate that will carry them, with a generated file beside them listing each copy, their tag, and their index's summary of the type. A test regenerates them from the pinned release and fails on a difference; `ASCRIBE_BLESS=1` rewrites them. The test needs their source, and is skipped, saying so, where there's no network.
- A second test, which needs no network: every committed template passes `ascribe check` in a project that declares its type, and `ascribe fmt` leaves it unchanged.
- The binary carries the committed files as they are.

### The model

`[types.<name>]` gains `template`, a path, and `guide`, a list of paths (README decision 14). Both from the project's folder. Validation: each file exists, is inside the project's boundary, and is outside the content root. A type with neither is as today.

`ascribe model` shows both, in its text and its JSON.

### The command

`ascribe templates eject how-to`, and `ascribe templates list`.

- `eject` writes the type's files under `templates/`, and adds the type to `ascribe.toml` in place. It refuses a type that's declared or a file that's there, names what it refused, and exits 2. It writes nothing partial: it checks everything first.
- It writes `templates/templates.lock` (decision 23): each file it wrote, with their tag, Ascribe's version, a hash of the file as written, and the type's summary. A second `eject`, for another type, adds to it. Phase 6's `status` reads it, so every copy made from this phase on can say what it's a copy of.
- `list` shows what the binary's set has: each type, in a line, from their index's summary.
- The work is one function in a library crate, returning what it wrote, and the command reports it.
- Run where there's no `ascribe.toml`, `eject` says so and exits 2, and it always will (decision 24). Starting a project is `ascribe init`, which phase 5 builds; until then the message and the reference say how to write a first `ascribe.toml` by hand.

### The check

`template-placeholder-left`, advice, on a page whose type names a template. A page of a type with no template isn't looked at.

**In the body,** for each placeholder of the template, the page has the same text. A placeholder is read block by block (decision 20):

- inside one block, it's the text from the escaped brace to its closing brace;
- one that doesn't close in its block runs to the block's end, takes each block after it whole, and ends at the closing brace in the block that has it;
- the page has it when it has each piece, in order. It's reported once, at the first piece, with the placeholder's opening words quoted.

**In the frontmatter,** a field whose value is the same as the template's (decision 21). `title: "Title"` in a how-to page is the title left as it came, and so is a description nobody rewrote.

One diagnostic for each, at its place in the page.

- The template is read through `FileSystem`, as a project file outside the content root. Whether that needs a new kind of read, as snippets' sources did, is this phase's to find out and do once.
- A template that can't be read or parsed is the model's error, at the type, not a problem on every page.
- An instruction of the template found whole in a page is reported by the same check, with a message that says it's an instruction. In their templates most instructions are placeholders already, such as `{This section is optional}`, and the rest are quoted paragraphs.
- The comparison is exact but for how a line was wrapped. `{Task name }`, which their how-to writes with a space before the brace, isn't found once an editor has taken the space out. That's the accepted trade in decision 17, and a test holds it so it stays deliberate.

**This is the seam, second half.** If "the same text" isn't exact after the formatter and a writer's edits around it, stop and report: decision 5 in the README depends on it.

### Ascribe's use of it

`examples/` gains a small project made by the command, checked by the suite that keeps the examples valid, so the set is used by something the tests build.

## What this phase doesn't handle, said where a user will look

The command reference says, while each is true: the set has one template; a page is started by copying the template's file; and a project's first `ascribe.toml` is written by hand.

## Tasks

1. The pinned tag, the script, and the how-to's rules, with the two tests, and the script's own test of a brace inside code.
2. `template` and `guide` in the model, with validation and conformance cases: both, neither, a path inside the content root, a file that isn't there.
3. The set in the binary, and `ascribe templates list`.
4. `ascribe templates eject`, with tests: a clean project; a type already declared; a file in the way; several named with one refused; comments in `ascribe.toml` kept; `templates.lock` written, and added to by a second run.
5. The check, in the registry, with conformance cases: a placeholder left; all filled; an instruction left; the placeholder that runs across blocks, left whole, left in part, and filled; the title left as the template has it; a type with no template; and a placeholder changed by a word, or by the space inside its brace, neither of which is reported.
6. The example project, and its test.
7. The command reference, the content model reference, the diagnostics reference, a first version of the docs' page on templates with its credit, `CHANGELOG.md`, the map of the surface, and the decision in `project-docs/decisions.md`.

## Out of scope

The other five templates. Starting a page any other way than copying. A project with no `ascribe.toml`. Following a new release. `ascribe templates status`. `@comment`.

## Acceptance criteria

- The goal's commands, run in a copy of `examples/getting-started`, give what's shown: the two files, the type, and one advice for each placeholder the copied page still has. A test does exactly this.
- Filling in every placeholder and removing every instruction leaves `ascribe check` with nothing to say about the page.
- The committed how-to equals what the script writes from the pinned tag, and passes `ascribe check` and `ascribe fmt --check`. Its body has no level-1 heading and no block quote about the template, and its "See also" placeholder is reported once on a page that left it alone.
- A project that names no template builds and checks as before: `node scripts/compare/outputs.ts --base main` shows nothing.

## Verify

```sh
cargo test --workspace --locked
pnpm test
node scripts/compare/outputs.ts --base main
```

## Commits

1. "Convert the how-to template from The Good Docs Project"
2. "Let a page type name its template and its guide"
3. "Write a template into a project with ascribe templates eject"
4. "Report a template's placeholder left in a page"
