# Proposal: Project Settings

A proposal, not a plan. It says what the change is and what it would look like, so it can be judged before anything is designed in detail. It's the settings half of the [Project view proposal](project-view.md), whose gear opens it.

## The goal

Anyone working on a project can find what it can set, see what's in effect, defaults included, see where each value is set, and change a simple value without knowing TOML, while `ascribe.toml` stays the file everyone reads, reviews, and commits.

Today the answer to "what can I set, and what is it now?" is the content model reference, read beside `ascribe.toml`, working out the defaults from the reference's tables. A writer who wants a check quieter, or the editor's build changed, opens a TOML file and looks up a key.

## What exists

**Two kinds of setting, kept apart on purpose.** The command checklist (`project-docs/checklists.md`, "A new editor command, action, or setting") says a VS Code setting is for one person's editor, and a choice about the content, which everyone working on the project must share, is in `ascribe.toml`. There are ten `ascribe.*` VS Code settings, and VS Code's Settings editor already shows and edits them. This proposal is about the second kind.

**Editing `ascribe.toml` in the editor:**

- **Problems as you type,** and Go to Definition and Find All References on a content model entry's key (`docs/content/editor/writing.md`).
- **`ModelFile`** (`crates/ascribe-lsp/src/model_file.rs`) edits the file and keeps its comments, blank lines, and order: each change is one text edit at its place, checked by parsing the result. It can set and insert entries, and remove one (`remove_entry`).
- **Actions that write to it already,** through `ascribe/edit` (`crates/ascribe-lsp/src/edit/model.rs`): Make this a phrase, Add to the glossary, and Change a feature's availability. Content model's Rename renames an entry everywhere.
- **The Content model view** lists phrases, features, terms, dimensions, and the rest, with their uses, and inserts or renames them.

**What knows the settings:**

- **The loader** (`crates/ascribe-model/src/loader.rs`) knows each table's keys, and reports any other (`check_keys`, as `check_keys("project", t, &["content-root", "output-dir"])`). A key's kind, its choices, and its default aren't in a list: each is in the code that reads that key. Nothing declares the settings as data.
- **Three `[consumer]` keys are also the site's.** `@ascribed/astro` gives Astro its `site`, `trailingSlash`, and `base` from `site`, `trailing-slash`, and `base-path` where `astro.config` leaves them unset, and otherwise every project must agree with Astro's (`supplyRouting` in `packages/astro/src/project.ts`). Only the site's build sees a mismatch.
- **The content model reference** (`docs/content/content-model/`) has a table for each section, `Key | Type | Default | Description`, written by hand. The surface map's generator already reads these pages for each section's first sentence (`scripts/surface/inventory.ts`).
- **`ascribe model --format json`** reports the types, dimensions, phrases, features, glossary, widgets, releases, builds, sources, and navigation, and not `[project]`, `[consumer]`, `[editor]`, `[checks]`, `[crawlers]`, `[listing]`, or `[permalinks]`.
- **The configurable checks** are marked `configurable = true` in `tests/conformance/diagnostics.toml`, forty-seven of them, each with its default severity, and thirty-eight with a `when` that says what it reports.

**A caution, from the research behind the views.** [VS Code extension UI patterns](../reports/VS%20Code%20extension%20UI%20patterns.md) warns against a custom editor that takes a file away from its writers, as one extension's preview did for Markdown by becoming the default. This proposal doesn't replace `ascribe.toml`'s editor; it's a tab beside it.

A mockup, kept locally with the other mockups (`.gitignore` leaves that folder out), shows the tab on Ascribe's own `docs/ascribe.toml`, with its real values and line numbers, and the file open beside it as values change.

## What it looks like

**Project Settings** is an editor tab, a webview panel, titled with the project's name. It's opened from the Project view's gear, from the Command Palette (**Ascribe: Open Project Settings**), and from a button in `ascribe.toml`'s editor title bar, offered only on the `ascribe.toml` of a project the extension found: not on one `ascribe.projects.exclude` leaves out, such as a fixture's under `tests/`.

**One tab per project,** found by the path of its `ascribe.toml`. Opening it again shows the tab already open. Closing and reopening `ascribe.toml` changes nothing: the tab reads the server's snapshot, which reads the disk when the file isn't open. When `ascribe.toml` is deleted or renamed, the project is gone, so the tab says so in a sentence, and closes with the next click.

- **At the top:** a search over every setting's name, key, and whole description; a filter, **Set in ascribe.toml**, for what the file sets; and a sentence saying the settings are shared with everyone on the project and written to `ascribe.toml`, with a link to **your editor's settings**, VS Code's Settings filtered to Ascribe's.
- **On the left,** the sections, in the reference's order: Project, Site and outputs, Builds, Editor, Checks, Crawlers, Content types, Dimensions, Releases, Phrases, features, and terms, and Sources.
- **Each setting** shows its name, its key (`[consumer] trailing-slash`), its default, where it's set (`ascribe.toml:167`, a link to the line) or that the file leaves it out, its description, and its value. A bar marks what the file sets, as VS Code's Settings editor marks what's changed.
- **Checks** are a list: each configurable check's name, its level, and what it reports, the ones the file sets first. A check with a setting of its own, `limit` for `page-size`, `image-large`, and `memo-long`, shows it. A level is one of the four the loader takes, `off`, `advice`, `warning`, and `error` (`model-invalid-value`), and the choices come from the declared list below. Every configurable check takes all four today; if one ever doesn't, the list says so, and the tab doesn't offer what it can't take.
- **`site`, `trailing-slash`, and `base-path`** say in their descriptions that a site's `astro.config` has to agree with them. The tab can't see the site's config, and doesn't try; the site's build reports a mismatch.
- **What has structure is a card:** each content type, build, dimension, release target, and source, with a summary and **Edit in ascribe.toml**, which opens the file at its table. Phrases, features, and terms open in Content model, where they're already added, renamed, and found.

### What it edits

**Simple values, in place:** a switch, a choice from a fixed list, a number, a short text, and a check's level. Each change is one `ModelFile` edit, sent back as a workspace edit: `ascribe.toml` changes, unsaved, with its comments, blank lines, and order kept, and Undo puts it back. A value the file doesn't set yet is added to its table.

**Reset to default** removes the line, and a comment that sits directly above it and touches it, with no blank line between: that comment is about the key, and left behind it would be about nothing. A comment with a blank line before the key stays, since it may be about the lines around it too. `remove_entry` keeps every comment today, so it learns this. The edit is unsaved, and Undo puts both back; a reviewer sees both lines go in the diff.

**The file is the truth, as it stands.** The tab draws from the language server's snapshot, unsaved text included, so typing in `ascribe.toml` updates the tab, and an edit from the tab shows in the open file. Nothing is saved for the author.

**The server checks every edit,** as it checks typing. A problem on a setting's line shows under the setting, in the server's words. The tab checks only what its controls can know (a number is a number, a choice is one of the list), and leaves every rule of the content model to the server, so the two never disagree.

**Everything else is the file's.** A table, a list, a new type or build or source: the tab links to the line. A form for each would be a second editor of the file's hardest parts, and those are the parts a reviewer most needs to read as text.

### The declared settings

`ascribe-model` gains a list of the settings the tab can edit, as data: each one's table, key, kind (switch, choice, number, text, level, or size), choices, and default. The loader reads its keys and its defaults from that list, in place of the lists in `check_keys` and the defaults written into each key's code, so there's one home for what a setting is, and the tab can't offer a key the loader would refuse or show a default it doesn't use. It's the small half of a schema in Rust: what a setting is, not what it means, and only the simple ones; a table with structure is read as today.

### The descriptions

Each setting's description is the reference's, as open question 1 recommends. A generator reads it from the reference's table and writes it, whole, as plain text, with a link's text kept and its address dropped, into a file the binary embeds, beside the declared list. The binary carries the descriptions, not the extension, because the extension runs whatever `ascribe` the project has: a file shipped with the extension could describe keys the project's binary doesn't have, or miss ones it does. A test fails when the reference's keys and the declared list's disagree, either way.

A check's description is its `when` in `tests/conformance/diagnostics.toml`, which the binary already carries.

### The request

The language server answers a new request, `ascribe/settings`, with every setting the tab shows: its table and key, its kind and choices, its default, its description, its value, and the span that sets it, if one does, from the parsed document. So the tab has one source at run time, the project's own binary. Its type goes through `crates/ascribe-cli/src/shapes.rs` to the extension's generated types, as `ascribe/inventory`'s does. Setting a value is a new `ascribe/edit` action for a table's key, made with `ModelFile`.

### The webview

A webview panel, so the This page proposal's rules apply: colors only from `--vscode-*`, with no fallbacks; the content security policy; text from the project set as text; no `<a>` elements; the ready handshake, with the newest answer sent. VS Code disposes a panel's document whenever its tab isn't visible, unless `retainContextWhenHidden` keeps it, as the preview does (`packages/vscode/src/preview/controller.ts`). The tab doesn't: as This page does, it runs the handshake every time its tab is shown again, and `getState` brings back its scroll, its search, and the section it was in.

## What changes besides the code

- **A decision file** for the request and its edit action, and for the rule the tab keeps: it edits simple values and links to the rest.
- **The loader reading the declared list is a clean-up:** every output and diagnostic stays byte for byte the same, which `node scripts/compare/outputs.ts --base main` shows.
- **The docs:** a section on Project Settings in `docs/content/editor/settings.md`, and a line in the content model reference's introduction pointing to it. An entry in `changes/editor/`.
- **The surface map,** for the new command.

## What this leaves out

- **Your editor's settings,** which are VS Code's, one link away.
- **`ascribe.nav`,** which is edited by dragging in Pages; **`ascribe.permalinks`,** which `ascribe permalinks sync` writes; and **`ascribe.examples.toml`**, rarely touched.
- **Forms for what has structure:** types, builds, dimensions, releases, sources, widgets, and `[listing]`'s rules.
- **Replacing `ascribe.toml`'s editor,** or opening the tab instead of the file.

## The thread

For a later plan, by the repository's [steel threads](../../.claude/skills/steel-threads/SKILL.md) skill. This isn't the plan.

- **The layers:** the declared settings and the loader that reads them (`ascribe-model`); the descriptions, generated from the reference and embedded; `ascribe/settings` and the edit action (`ascribe-lsp`, through `ModelFile`); the shapes; the extension's panel and its messages, and the webview's script and stylesheet; the docs.
- **First, before the thread:** the declared list, with the loader reading its keys and defaults from it and the outputs unchanged, and where the descriptions are carried. Every later step reads from both, so the plan's first phase settles them.
- **The riskiest seam** is the round trip: a value chosen in the tab becomes an edit to the file, unsaved, the server reads the edited text, and the tab draws what the server now says, the edit's problems included. If the tab and an open `ascribe.toml` can fall out of step, or an edit can lose a comment, the design changes before more settings are shown.
- **The thinnest use case:** in `examples/quill`, Project Settings shows `[consumer] trailing-slash` and `[consumer] base-path`, each with its value, its default, its description, and its line. Set the trailing slash to `never`: `ascribe.toml` has one line changed, unsaved, its comments as they were; the tab shows the new value; Undo in the file puts it back, and the tab follows. Type a base path with no leading `/`: the server's problem shows under it, in its words.
- **Left for the widening, in order of risk:** a key the file doesn't set, added to its table, and Reset to default with its comment; the checks, with their levels and limits; every simple setting; search and the filter; the cards and their links; opening from the Project view, the palette, and `ascribe.toml`'s title bar; several projects, and a deleted `ascribe.toml`; the docs and the decision.

## Open questions

1. **Where each setting's description comes from.** The reference's tables have them, written by hand, and the tab needs them as plain text. The AGENTS.md rule is one home per fact. *Recommend:* the reference's tables stay the home, carried by the binary as "The descriptions" says, whole, so search covers every sentence. A check's description is its `when` in `tests/conformance/diagnostics.toml`, which is already its home; the nine configurable checks without one gain one there, a change to that file and the diagnostics reference that can ship on its own before the tab. The alternative, a schema in Rust that writes the reference's tables, moves the home out of the docs, and makes every table in the reference a generated file. The declared list is the part of that schema the tab needs either way.
2. **Which checks the list shows.** *Recommend:* all forty-seven configurable checks, the ones the file sets first, then the rest in the diagnostics reference's order, since search finds any of them, and a check left out is one nobody learns can be set.

## Settled

Answered by the maintainer on 20 October 2026, after looking at the mockup: the editor's own settings stay VS Code's, one link away; the panel is for everyone who works on the project, to find what can be set and what's in effect, not only for those who don't know TOML; it's an editor tab that shows every setting with its description, its value, its default, and where it's set, edits simple values in place through the edits that keep comments, and links to the file for everything with structure; and it covers `ascribe.toml` alone. _Why:_ VS Code already has a settings editor for an editor's settings; a view of what's in effect serves everyone, where a form only replaces typing; and the parts of the file with structure are the ones a reviewer reads as text.
