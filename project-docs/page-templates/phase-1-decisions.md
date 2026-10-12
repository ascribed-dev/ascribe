# Phase 1: Decisions

Part of [Page templates](README.md). Needs nothing. No code: a pull request that records what the maintainer decides.

## Goal

Six choices are made before anything ships, because phase 2 puts each into a contract or into every project that takes the set: two keys in `ascribe.toml`, a command's arguments, a diagnostic, and the files a project ends up owning.

This phase can't be folded into phase 2. A thread may narrow what it handles. It may not ship a shape the plan expects to change.

## Context

- The [proposal](../proposals/page-templates.md): "In a project", and "The conversion".
- `docs/content/reference/content-model.md` §4: what a type has today, and how a path in `ascribe.toml` is written (from the project's folder).
- `ascribe vale eject`, in the command reference: what it writes, what it refuses, and its exit codes.
- `project-docs/checklists.md`, "A new command or option": naming, `--format`, how a command names things, and exit codes.
- The [memos proposal](../proposals/memos.md): `@memo`, and that nothing has built it.
- Their repository at v1.6.0: a template's quoted instruction paragraphs, its placeholders, and `index.json`.

## The decisions

Each has a recommendation. The maintainer may take it, change it, or ask for more.

### 1. The keys on a type

| Option | Looks like | For and against |
|---|---|---|
| **`template`, a file, and `guide`, one file or a list** (recommended) | `template = "templates/how-to.md"`, `guide = ["templates/how-to.guide.md", "templates/how-to.ascribe.md"]` | Their guide and Ascribe's notes stay two files, as decision 2 has them, and a type names both in the order to read them. A team with one guide writes a string |
| `template`, `guide`, and `notes` | Three keys | Says which file is whose. A third key for something that's "more guidance" to everyone but Ascribe |
| `template` and `guide`, one file each, with Ascribe's notes joined to the guide when it's copied | Two strings | The simplest model. Their words and ours are one file in every project, so "what changed upstream" can't be shown for the guide alone |

Both are paths from the project's folder, to files outside the content root: a template inside it would be a page. A path into the content root is an error.

**Decided: `template`, a file, and `guide`, a list.** Two keys, not three: a third named `notes` would give "note" a fourth meaning beside `@note`, `[notes.<type>]`, and `note:` targets, which the surface audit's finding 9 already counts against. The guide and Ascribe's notes stay two files, so `templates status` can show what changed upstream in the guide alone, and a type lists them in the order to read them. `guide` is always a list, as `files` is: no key in the content model takes a string or a list, and one shape is one rule to document and validate. A team with one guide writes `guide = ["templates/how-to.guide.md"]`. README decision 14.

### 2. Where the copied files go

| Option | Means |
|---|---|
| **`templates/`, beside `ascribe.toml`** (recommended) | `templates/how-to.md`, `templates/how-to.guide.md`, `templates/how-to.ascribe.md`. One folder, named for what it holds, that a team finds without being told |
| Under `.ascribe/` | Out of the way. It's the folder the docs say can be deleted, and these are files a team edits |

With `--to <DIR>` to put them elsewhere, a path from the current folder.

**Decided: `templates/`, beside `ascribe.toml`.** `.ascribe/` is the folder the docs say can be deleted, and these are files the team edits and commits. `--to <DIR>` is a path from the current folder, as the checklist has it, and the paths written into `ascribe.toml` are from the project's folder whatever `--to` was. A `templates/` that's already there is a file in the way (decision 3). README decision 15.

### 3. What the command takes, and what it refuses

`ascribe templates eject [NAME]...`: the types to write, and all of the starter set when none is named.

| Question | Recommended |
|---|---|
| A type of that name is declared already | Refuse that one, name it, and write nothing for it. With several named, write the others, and exit 2 if any was refused: the same rule as `ascribe vale eject`, which never overwrites |
| A file is in the way | The same |
| `ascribe.toml` | Edited in place, keeping comments and order, as `ascribe permalinks retire` does. Each type added gets `files`, `template`, `guide`, and its frontmatter |
| The type's `files` | `<name>/**` under the content root, said in the output, for the team to change |
| `--format` | `text` and `json`, the JSON listing each file written and each type added or refused |
| A list of what the set has | `ascribe templates list`, reading only the binary |

**Decided: as the table has it, with one change: nothing is written when anything named is refused.** The table's "write the others" would leave `ascribe.toml` edited for some types and an error for the rest, which is the partial write `ascribe vale eject` avoids by checking everything first, and what [phase 2](phase-2-thread.md)'s design already says. So `eject` checks every type named, or the whole set when none is, and if any is declared already or any file is in the way it names each one, writes nothing, and exits 2; the message says to name the types wanted. A run that's refused is cheap to repeat with fewer names. The rest stands: `ascribe.toml` edited in place with its comments and order kept, `files = ["<name>/**"]` said in the output, `--format text|json`, and `list` reading only the binary. README decision 16.

### 4. What counts as a placeholder

The check in phase 2 compares a page with its type's template, so "a placeholder" needs one meaning.

| Option | Means |
|---|---|
| **A run of literal text in the template that an escaped brace opens and its closing brace ends** (recommended) | `\{Task name}`, and `\{insert a brief description of the task}`. Found by reading the template as Ascribe reads any page. A page of the type that has the same text, braces included, has one left |
| Any text the template and the page share | Catches a forgotten instruction too. Also catches a heading the writer was meant to keep |

The comparison is of text as written, after `ascribe fmt`'s canonical form, ignoring how a line was wrapped. A placeholder a writer changed by one word isn't found, and that's accepted: the check finds what was never touched.

The check is `template-placeholder-left`, advice, configurable, with the next step `write`.

**Decided: a run of literal text an escaped brace opens and its closing brace ends.** It's what decision 4 in the README already makes a placeholder, found by reading the template as any page is read, so the check and the conversion agree by construction. "Any shared text" would report a heading the writer was meant to keep, on every page of the type. The comparison is of canonical form, ignoring wrapping; a placeholder changed by a word isn't found, and that's accepted. `template-placeholder-left`, advice, configurable, next step `write`. README decision 17.

### 5. What carries a template's instructions, until `@memo` exists

Their templates give instructions to the author as quoted paragraphs in the body. Decision 8 says they become text that's never published, by a construct nothing has built.

| Option | Means |
|---|---|
| **They stay as quoted paragraphs, and the check reports one left in a page** (recommended) | Nothing is lost, and nothing waits. A forgotten instruction is published, as it would be from their template used anywhere else, and `ascribe check` says so first. When `@memo` exists the conversion gains a rule |
| The conversion drops them from the template, and the guide carries the same advice | A page can't publish one. The hints at the place they apply are gone, which is most of their use |
| Wait for the memos plan | Clean. This plan is then second in line behind one that hasn't started |

Taking the first means decision 4's check covers instructions as well: text in a page that's a whole quoted instruction of its type's template.

**Decided: they stay as quoted paragraphs, and the check reports one left whole in a page.** Nothing is lost and nothing waits on the memos plan; a forgotten instruction is reported before it's published, which is more than their template gets anywhere else. The check matches a whole quoted paragraph of the template, so a writer's own block quote isn't reported. When `@memo` exists the conversion gains one rule and the set is regenerated, as the README's "Carved out" says. README decision 18.

### 6. Where the set lives here, and what's pinned

| Question | Recommended |
|---|---|
| The pinned release | v1.6.0, by its tag and commit, in one file the script and the test both read |
| Their source | Fetched by the script, into a folder git ignores. Not committed: it's theirs, and the commit pins it |
| Ascribe's copies | Committed, in the crate that carries them into the binary, with a line at the top of each saying it's generated, from what, and at which release |
| The structural rules | A data file beside the script |
| Ascribe's notes | Hand-written files beside the copies, which the script never opens |

**Decided: as the table has it.** The pin is one file the script and the test both read, so a release is changed in one place; their source is fetched and ignored, since the commit pins it; the copies are committed in the crate that carries them, each with a line saying it's generated, from what, and at which release, so the generated-files test holds them; the rules are data beside the script; and Ascribe's notes are hand-written files the script never opens. README decision 19.

## Also confirmed here

- The six: concept, how-to, tutorial, reference, troubleshooting, and release notes. The thread's one is the how-to.
- A type's name in the starter set is its template's name in their index, in lower case with hyphens.
- A site's starter files are an issue of their own, [#344](https://github.com/ascribed-dev/ascribe/issues/344), not this plan's (README, "Carved out").
- The docs' page on templates says where they came from, at which release, and under which license.

## Tasks

1. The maintainer decides each, on a card or in the pull request. _Done:_ each is recorded under its options above, and in the README as decisions 14 to 19.
2. Bring [phase 2](phase-2-thread.md) in line with anything that isn't the recommendation, before it starts. _Done:_ decision 3 took phase 2's "writes nothing partial" over the table's "write the others", so phase 2 already agrees; decision 1 makes `guide` always a list, which phase 2's model section should say.
3. Open the issue for a site's starter files, linked from the README. _Done:_ [#344](https://github.com/ascribed-dev/ascribe/issues/344).

## Out of scope

Any code. The structural rules themselves, which phases 2 and 3 write as each template needs one.

## Acceptance criteria

- The README's decisions list holds all six, and the confirmed choices.
- Phase 2's design uses the names decided.

## Commits

1. "Record the page templates plan's decisions"
