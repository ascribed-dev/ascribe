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
- **Checks** are a list: each configurable check's name, its level, and what it reports, the ones the file sets first. A check with a setting of its own, `limit` for `page-size`, `image-large`, and `memo-long`, shows it. A level is one of the four the loader takes, `off`, `advice`, `warning`, and `error` (`model-invalid-value`), and the choices come from the registry below. Every configurable check takes all four today; if one ever doesn't, the list says so, and the tab doesn't offer what it can't take.
- **`site`, `trailing-slash`, and `base-path`** say in their descriptions that a site's `astro.config` has to agree with them. The tab can't see the site's config, and doesn't try; the site's build reports a mismatch.
- **What has structure is a card:** each content type, build, dimension, release target, and source, with a summary and **Edit in ascribe.toml**, which opens the file at its table. Phrases, features, and terms open in Content model, where they're already added, renamed, and found.

### What it edits

**Simple values, in place:** a switch, a choice from a fixed list, a number, a short text, and a check's level. Each change is one `ModelFile` edit, sent back as a workspace edit: `ascribe.toml` changes, unsaved, with its comments, blank lines, and order kept, and Undo puts it back. A value the file doesn't set yet is added to its table.

**Reset to default** removes the line, and a comment that sits directly above it and touches it, with no blank line between: that comment is about the key, and left behind it would be about nothing. A comment with a blank line before the key stays, since it may be about the lines around it too. `remove_entry` keeps every comment today, so it learns this. The edit is unsaved, and Undo puts both back; a reviewer sees both lines go in the diff.

**The file is the truth, as it stands.** The tab draws from the language server's snapshot, unsaved text included, so typing in `ascribe.toml` updates the tab, and an edit from the tab shows in the open file. Nothing is saved for the author.

**The server checks every edit,** as it checks typing. A problem on a setting's line shows under the setting, in the server's words. The tab checks only what its controls can know (a number is a number, a choice is one of the list), and leaves every rule of the content model to the server, so the two never disagree.

**Everything else is the file's.** A table, a list, a new type or build or source: the tab links to the line. A form for each would be a second editor of the file's hardest parts, and those are the parts a reviewer most needs to read as text.

### The registry

Every key of `ascribe.toml` has one home: a registry, `crates/ascribe-model/settings.toml`, which the binary embeds. It's the pattern `tests/conformance/diagnostics.toml` already sets for the diagnostics: one data file that the binary answers from and the docs are generated from. Each entry has its table, key, kind, choices, default, description, and availability, in the reference's order.

- **Every key, not only the simple ones.** A key of a table with structure (a type's `template`, a build's `variants`, a source's `include`) is in it too, with a kind such as "table" or "list", since the loader and the reference need it. The tab edits only the simple kinds: switch, choice, number, text, level, and size.
- **The loader reads its keys and defaults from it,** in place of the lists in `check_keys` and the defaults written into each key's code. The tab then can't offer a key the loader would refuse, or show a default it doesn't use.
- **The reference's tables are generated from it,** as the command reference's options and the diagnostics reference are: each `Key | Type | Default | Description` table becomes a fragment in `docs/content/_generated/`, included where today's table is, and rewritten with `ASCRIBE_BLESS=1`. The prose around each table stays written in the docs. About twenty-five tables move, in the reference's four pages and in `docs/content/_fragments/content-model/`.
- **A description is Markdown,** as the reference writes it: phrases such as `{repo}`, links, code spans, and bold lead-ins. A key's availability mark, `{available="ascribe 0.3.0"}` in today's tables, is a field of its entry. The generated tables have them as written.
- **The first generation changes no text.** The contract pages (`docs/content/contracts/content-model.md` and its parts) include the same fragments, so the generated tables come out byte for byte as today's, and the move is a clean-up, not a change to a contract. A table that can't be reproduced stops the move, and is reported.

A check's description is its `when` in `tests/conformance/diagnostics.toml`, which the binary already carries, so the checks need no entry here.

### The request

The language server answers a new request, `ascribe/settings`, with every setting the tab shows: its table and key, its kind and choices, its default, its description, its value, and the span that sets it, if one does, from the parsed document. So the tab has one source at run time, the project's own binary, whatever version of `ascribe` the project runs. The description goes to the tab as plain text, with its phrases replaced, a link's text kept and its address dropped, and code spans as pieces, as This page's titles go, so the webview parses nothing. Its type goes through `crates/ascribe-cli/src/shapes.rs` to the extension's generated types, as `ascribe/inventory`'s does. Setting a value is a new `ascribe/edit` action for a table's key, made with `ModelFile`.

### The webview

A webview panel, so the This page proposal's rules apply: colors only from `--vscode-*`, with no fallbacks; the content security policy; text from the project set as text; no `<a>` elements; the ready handshake, with the newest answer sent. VS Code disposes a panel's document whenever its tab isn't visible, unless `retainContextWhenHidden` keeps it, as the preview does (`packages/vscode/src/preview/controller.ts`). The tab doesn't: as This page does, it runs the handshake every time its tab is shown again, and `getState` brings back its scroll, its search, and the section it was in.

## What changes besides the code

- **A decision file** for the request and its edit action, and for the rule the tab keeps: it edits simple values and links to the rest.
- **The loader reading the registry is a clean-up:** every output and diagnostic stays byte for byte the same, which `node scripts/compare/outputs.ts --base main` shows.
- **The reference's tables, generated, are a clean-up too:** the docs' source changes, from a table to an include, and the built docs don't.
- **The docs:** a section on Project Settings in `docs/content/editor/settings.md`, and a line in the content model reference's introduction pointing to it. An entry in `changes/editor/`.
- **The surface map,** for the new command.

## What this leaves out

- **Your editor's settings,** which are VS Code's, one link away.
- **`ascribe.nav`,** which is edited by dragging in Pages; **`ascribe.permalinks`,** which `ascribe permalinks sync` writes; and **`ascribe.examples.toml`**, rarely touched.
- **Forms for what has structure:** types, builds, dimensions, releases, sources, widgets, and `[listing]`'s rules.
- **Replacing `ascribe.toml`'s editor,** or opening the tab instead of the file.

## The thread

For a later plan, by the repository's [steel threads](../../.claude/skills/steel-threads/SKILL.md) skill. This isn't the plan.

- **The layers:** the registry, and the loader that reads it (`ascribe-model`); the reference's tables generated from it; `ascribe/settings` and the edit action (`ascribe-lsp`, through `ModelFile`); the shapes; the extension's panel and its messages, and the webview's script and stylesheet; the docs.
- **First, before the thread:** the registry, with the loader reading its keys and defaults from it and the outputs unchanged, and then the reference's tables generated from it with the docs unchanged. Every later step reads from it, so the plan's first phases make it.
- **The riskiest seam** is the round trip: a value chosen in the tab becomes an edit to the file, unsaved, the server reads the edited text, and the tab draws what the server now says, the edit's problems included. If the tab and an open `ascribe.toml` can fall out of step, or an edit can lose a comment, the design changes before more settings are shown.
- **The thinnest use case:** in `examples/quill`, Project Settings shows `[consumer] trailing-slash` and `[consumer] base-path`, each with its value, its default, its description, and its line. Set the trailing slash to `never`: `ascribe.toml` has one line changed, unsaved, its comments as they were; the tab shows the new value; Undo in the file puts it back, and the tab follows. Type a base path with no leading `/`: the server's problem shows under it, in its words.
- **Left for the widening, in order of risk:** a key the file doesn't set, added to its table, and Reset to default with its comment; the checks, with their levels and limits; every simple setting; search and the filter; the cards and their links; opening from the Project view, the palette, and `ascribe.toml`'s title bar; several projects, and a deleted `ascribe.toml`; the docs and the decision.

## Settled

Answered by the maintainer on 20 October 2026, after looking at the mockup: the editor's own settings stay VS Code's, one link away; the panel is for everyone who works on the project, to find what can be set and what's in effect, not only for those who don't know TOML; it's an editor tab that shows every setting with its description, its value, its default, and where it's set, edits simple values in place through the edits that keep comments, and links to the file for everything with structure; and it covers `ascribe.toml` alone. _Why:_ VS Code already has a settings editor for an editor's settings; a view of what's in effect serves everyone, where a form only replaces typing; and the parts of the file with structure are the ones a reviewer reads as text.

The proposal's two open questions, and Reset to default's comment, answered by the maintainer on 20 October 2026:

- **Every key of `ascribe.toml` has one home, the registry,** as "The registry" says: the loader reads its keys and defaults from it, the reference's tables are generated from it, and `ascribe/settings` answers from it. A check's description is its `when` in `tests/conformance/diagnostics.toml`; the nine configurable checks without one gain one there, a change that can ship on its own before the tab. _Why:_ the diagnostics and the command reference's options already work this way, and keeping the reference's hand-written tables as the home would have left two homes held equal by a test, with a key's kind and choices in neither, and a file of descriptions the extension shipped for whatever binary the project runs.
- **The checks list shows all forty-seven configurable checks,** the ones the file sets first, then the rest in the diagnostics reference's order. _Why:_ search makes the length harmless, and a check left out is one nobody learns can be set.
- **Reset to default takes a comment that touches the line with it,** and leaves one after a blank line, as "What it edits" says. _Why:_ a comment directly above a key is about it, and left behind it's about nothing.
