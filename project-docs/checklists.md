# Change checklists

What a change has to touch, as lists a reviewer can tick. Each list was built from the code; when the code grows a new surface, the list changes in the same pull request. Not every line applies to every change: a reviewer ticks a line or says why it doesn't apply.

The rules every change follows (the checks to run, conformance cases, the fork, the docs) are in [CONTRIBUTING.md](../CONTRIBUTING.md). The decisions behind them are in [decisions.md](decisions.md) and [decisions/](decisions/README.md).

## A change to the language

New or changed syntax, a directive, a block or inline form, an attribute, or what a content model can say.

**What it means**

- [ ] `SPEC.md` says what the change means, and its examples parse.
- [ ] Conformance cases in `tests/conformance/cases/`, with outlines and builds as `tests/conformance/README.md` describes. A case the change can't pass yet is listed in `tests/conformance/SKIPS.toml`, with the reason.

**Reading it**

- [ ] Parsing, in `crates/ascribe-syntax/`. A change to the parser itself is in `crates/comrak-ascribe/`, marked as `crates/comrak-ascribe/FORK.md` says.
- [ ] The content model, in `crates/ascribe-model/`, if `ascribe.toml` can say something new.
- [ ] Resolving, in `crates/ascribe-resolve/`: includes, snippets, phrases, links, and builds.
- [ ] Incremental updates: a new kind of change (a file the project reads, or a new `Change`) is a row of the invalidation table in `crates/ascribe-resolve/src/incremental/mod.rs`, and the differential test's generator makes it, with sources that use it (`crates/ascribe-resolve/tests/all/incremental_differential.rs`, and its vocabulary in `crates/ascribe-resolve/tests/incremental_support/`).

**Checking it**

- [ ] The checks, in `crates/ascribe-check/src/checks/`.
- [ ] Each new or changed diagnostic, in `tests/conformance/diagnostics.toml` only; the diagnostics reference is generated from it. Its `reference` names the section of the directive or `ascribe.toml` reference its entry sends a reader on to.
- [ ] **What a diagnostic says.** Its message says what's wrong, naming the thing in backticks, starting in lower case, with no full stop: `` `{path}` doesn't exist ``. Its fix text says what to do, in sentences. Its kind of next step is true of every instance: `choose` only when Ascribe lists what to choose from, `fix` only when there's a fix to apply. People and agents are told about a *problem*; *diagnostic* is the kind of problem, with a code, that the reference and `ascribe explain` describe.

**Each output, and its contract**

- [ ] `site`: `crates/ascribe-emit/src/site/`, and the contract in `docs/content/contracts/site-render.md`. Then the [site output's markup](#a-change-to-the-site-outputs-markup) list below.
- [ ] `plain`: `crates/ascribe-emit/src/plain/`.
- [ ] `json`: `crates/ascribe-emit/src/json.rs`, and its shape in `crates/ascribe-emit/README.md`. A removed field, or one that changes meaning, raises the schema version.
- [ ] The Zod schema for frontmatter, in `crates/ascribe-emit/src/zod/`, checked by `tests/zod/`.
- [ ] The diff, in `crates/ascribe-diff/`: a new kind of block aligns and reports its changes.

**Formatting**

- [ ] `crates/ascribe-fmt/` writes the new form canonically, and `ascribe fmt` changes nothing on a page already in canonical form.

**The language server, feature by feature**

The features `crates/ascribe-lsp/src/server.rs` handles. Each new form is considered for each one.

- [ ] Diagnostics, pushed as the project changes (`crates/ascribe-lsp/src/core.rs`).
- [ ] Semantic tokens, full and range (`SemanticTokensFullRequest`, `SemanticTokensRangeRequest`; `crates/ascribe-lsp/src/tokens.rs`). A new token type is added to the legend in `crates/ascribe-lsp/README.md` and to `semanticTokenTypes` and `semanticTokenScopes` in `packages/vscode/package.json`.
- [ ] Completion (`Completion`; `crates/ascribe-lsp/src/complete.rs`), and its trigger characters in `server.rs` if the form starts with a new one.
- [ ] Hover (`HoverRequest`; `crates/ascribe-lsp/src/hover.rs`).
- [ ] Go to definition (`GotoDefinition`; `crates/ascribe-lsp/src/definition.rs`).
- [ ] Find all references (`References`; `crates/ascribe-lsp/src/references.rs`), which counts uses through `Project::uses` in `crates/ascribe-resolve/src/uses.rs`: a new form that uses a page, heading, or content model entry is a kind of use.
- [ ] Document links (`DocumentLinkRequest`), CodeLens (`CodeLensRequest`), and inlay hints (`InlayHintRequest`), all in `crates/ascribe-lsp/src/links.rs`.
- [ ] Quick fixes and other code actions (`CodeActionRequest`; `crates/ascribe-lsp/src/code_action.rs`).
- [ ] Formatting (`Formatting`; `crates/ascribe-lsp/src/formatting.rs`), which calls `ascribe-fmt`.
- [ ] Folding ranges (`FoldingRangeRequest`; `crates/ascribe-lsp/src/folding.rs`): a new container that should fold to its first line, as a container `@memo` does.
- [ ] Rename (`Rename`), what can be renamed at a position (`PrepareRenameRequest`), and the edits when a file is moved (`WillRenameFiles`), all in `crates/ascribe-lsp/src/refactor.rs`.
- [ ] The command a CodeLens runs (`ExecuteCommand`, `ascribe.openFile`).
- [ ] The page preview (`ascribe/preview`; `crates/ascribe-lsp/src/preview.rs`).
- [ ] Review's changes (`ascribe/review/setBase`, `ascribe/review/changes`, and `ascribe/review/baseFile`; `crates/ascribe-lsp/src/review.rs`).
- [ ] What's at a position (`ascribe/context`; `crates/ascribe-lsp/src/context.rs`): a new block or inline construct is a node kind, and a new token is a token kind.
- [ ] What actions can point at (`ascribe/targets`; `crates/ascribe-lsp/src/targets.rs`): a new kind of content model entry is a kind of target.
- [ ] The project's inventory (`ascribe/inventory`; `crates/ascribe-lsp/src/inventory.rs`): a new kind of content model entry is a kind of entry.
- [ ] What's known about a page (`ascribe/page`; `crates/ascribe-lsp/src/page.rs`), whose connections come from `Project::uses_of_file`: a new form that uses a page or a heading is a kind of use there too.
- [ ] Project Settings (`ascribe/settings`, and `ascribe/edit`'s `setSetting`; `crates/ascribe-lsp/src/settings.rs`), which answers from the registry in `crates/ascribe-model/settings.toml`: a new key of a simple kind is a setting there.
- [ ] Page edits (`ascribe/edit`; `crates/ascribe-lsp/src/edit.rs`): an operation that writes a changed construct writes it the new way, and a new directive may need an operation.
- [ ] One edit for the whole project (`ascribe/projectEdit`; `crates/ascribe-lsp/src/project_edit.rs`), whose safe fixes are `code_action.rs`'s `safe_fixes` and whose formatting is `ascribe_fmt::format`: a new safe fix or formatting rule reaches it through those.
- [ ] What a build leaves out (`ascribe/buildView`; `crates/ascribe-lsp/src/build_view.rs`): a new way a build removes content is removed in `crates/ascribe-resolve/src/build/modes.rs`, which says why, and the build lens shows it.
- [ ] A prompt for an agent (`ascribe/agentPrompt`; `crates/ascribe-lsp/src/agent_prompt.rs`), built by `ascribe_check::prompt`: a new diagnostic's prompt is worth reading, and a value it's about goes in its `values` placeholder, which the prompt lists as the allowed values.
- [ ] Where the pages say something (`ascribe/search`; `crates/ascribe-lsp/src/search.rs`), found by `ascribe_resolve::reading::find`, as `ascribe search` finds it: a new kind of block the reading reads is searched in the editor too, and `crates/ascribe-lsp/tests/all/search.rs` holds the two to the same places.
- [ ] The server's tests in `crates/ascribe-lsp/tests/`, and `lsp_parity` in `crates/ascribe-cli/tests/`, which compares the server's diagnostics and agent prompts with `ascribe check`'s, and `build_view_parity`, which compares what the build lens dims with what `ascribe build` leaves out.

**The editor**

- [ ] The VS Code grammar, in `packages/vscode/syntaxes/`.
- [ ] The editor's actions, in `packages/vscode/src/actions/registry.ts`: a new `ascribe/edit` operation gets an action (a test fails without one), and a new node kind in `ascribe/context` is considered for where each action applies.

**The docs**

- [ ] The reference: `docs/content/writing/directives.md` (or `syntax.md`, `frontmatter.md`, or `inline.md` beside it), or the page of `docs/content/content-model/` with the section, and any guide that shows the form. What no release has yet is marked `@available: ascribe planned`.
- [ ] A new built-in: the directive reference's table of built-ins, generated from `builtin_schemas()` by `crates/ascribe-cli/src/docs/builtins.rs` (`ASCRIBE_BLESS=1 cargo test -p ascribe-cli docs`), with its availability, `ascribe planned`, in `available` there; and its section in the reference, in SPEC §4's order, which the same test holds.
- [ ] Its changelog entry, a file of its own in `changes/` ([changes/README.md](../changes/README.md)), not a line in `CHANGELOG.md`.

## A new command or option

- [ ] **The shared flags,** as [CONTRIBUTING.md](../CONTRIBUTING.md#command-line-conventions)'s command-line conventions say: `--dry-run` and the git guard with `--force` on a command that writes, `--check`, `--exit-code`, `--format json`, and `--base`. The command is in `WRITES` or `READS` in `crates/ascribe-cli/src/conventions.rs`, which holds them, and takes no exception there.
- [ ] **Its name.** A command is one lowercase word: a verb for what it does to the project (`build`, `check`, `resolve`), or a noun for what it shows (`model`, `outline`, `refs`). Commands about one kind of thing are a group, named with that thing's plural (`sources`, `agents`), whose subcommands are verbs (`fetch`, `update`, `sync`), or a noun where one prints a thing (`rules`, `status`). A second command about a thing makes a group: it doesn't get a second top-level word. So `ascribe permalinks verify` and `ascribe versions retire`, and exporting a version is `ascribe versions export`, not `ascribe export`. A verb alone is right while it's the only command about its thing, as `ascribe observe` would be.
- [ ] **The same verb for the same job.** `sync` writes files Ascribe goes on owning and rewrites, with `--check` to say whether they're current, exiting with `1` when they're not (`agents sync --check`, `permalinks sync --check`). `eject` writes Ascribe's own defaults into the project once, as the project's from then on. A command that writes a starter set into a project is that group's `eject`. `list`, `status`, and `verify` only read.
- [ ] **`--format`'s words,** and only the ones the command needs, with `text` the default. `text` is for a person, and isn't a contract. `json` is the contract: one document with a schema version. `concise` is one line for each thing found, for an agent. `prompt` is text to hand to an agent. `html` is one file that stands alone. A report in Markdown, for a CI job's summary, a pull request's description, or an issue's body, is `markdown` (decision 104). No command takes a word outside these for a new meaning, and a shorter report is an option of its own (`--summary`), not a format. A command that takes `text` takes `concise` too, an alias of `text` where it has no one-line form (decision [#965](decisions/965-cli-conventions.md)).
- [ ] **How it names things.** A page is a path from the current folder: `PAGE` when the command is about one, `PATHS...` when it takes several, and an option named for its part when it's a second one (`--from`). A build is `--build <NAME>`: repeatable where the command covers several, with every build when it's absent, and single where it shows one reader's view. The project is found from the path arguments, or from the current folder, and `--config` says otherwise; a command adds no option or argument of its own for it. Nothing is an option named `--version`, which prints Ascribe's: a version of the docs is an argument. An option word means one thing across the commands: `--list` is a switch that lists, so a path gets another word.
- [ ] **The words for what it reports.** What Ascribe found is a *problem*; the kind of problem, with a code, is a *diagnostic*; how severe one is, is an error, a warning, or advice, and a count of them is a `summary` (decision 119). A command doesn't call them findings, alerts, or issues.
- [ ] **What it exits with.** `0`: it ran, and there's nothing to act on. `1`: it ran, and the answer is the one to act on: problems, a target that isn't there, a file that would change. `2`: it couldn't do its work. A name of Ascribe's own that doesn't exist (a build, a diagnostic's code, a section) is `2`; a thing of the project's that doesn't exist (a page, a heading, a permalink) is `1`. A command whose whole job is a yes or a no exits `1` unasked (`check`, `link`, a group's `verify`). One that reports exits `0` whatever it finds, and `1` only with `--exit-code` (`diff`, `drift`, `report`).
- [ ] The arguments and help text, in `crates/ascribe-cli/src/cli.rs` and `crates/ascribe-cli/src/commands/`. The work itself is one entry function in a library crate, taking plain arguments and returning a typed result, so the language server and other callers can reach it; the command parses its arguments, calls it, and reports. A new command's entry goes in ARCHITECTURE.md's [Each command's entry](../ARCHITECTURE.md#each-commands-entry).
- [ ] Exit codes from `crates/ascribe-cli/src/exit.rs` only, and the command's "Exit codes" section, on its page of the command reference, which `docs/content/command/index.md` lists, says which it uses.
- [ ] The command reference's fragments in `docs/content/_generated/`, regenerated from the help text by `crates/ascribe-cli/src/docs.rs` (`ASCRIBE_BLESS=1 cargo test -p ascribe-cli docs`). A new command also gets a section in `docs/content/command/index.md` that includes its fragment. An option no release has yet is listed in `UNRELEASED` in `docs.rs`.
- [ ] With `--format json`: the shape's fields documented in the command's section, on its page of the command reference, which `docs/content/command/index.md` lists, keys in snake_case, and a schema version that rises when a field is removed or changes meaning.
- [ ] The shape's schema and TypeScript, generated from its Rust types by `crates/ascribe-cli/src/shapes.rs` (`ASCRIBE_BLESS=1 cargo test -p ascribe-cli shapes`): a doc comment on every field, a new shape added to `SHAPES` (and to `PACKAGES` when a package reads it), and a new command's schema included in `docs/content/contracts/json-reports.md`. The packages read the generated `packages/astro/src/shapes.ts`, `packages/review/src/shapes.ts`, and `packages/vscode/src/shapes.ts`, never a type of their own.
- [ ] The TypeScript that runs the command, if it passes the new option: `packages/astro/src/run.ts`.
- [ ] Tests in `crates/ascribe-cli/tests/`. Output the docs show is written to `crates/ascribe-cli/tests/output/` by `crates/ascribe-cli/tests/all/output.rs`.
- [ ] Its changelog entry, a file of its own in `changes/` ([changes/README.md](../changes/README.md)), not a line in `CHANGELOG.md`.
- [ ] The map of the surface, `project-docs/surface.md`, rewritten by `node scripts/surface/inventory.ts` after the fragments above. Its diff is what the change added to the surface: read it against the conventions at the top of this list.
- [ ] **A tool for it,** if an agent with no shell needs the answer: named `ascribe_` and the command's own word (`ascribe_search` for `ascribe search`), with the command's arguments under the same names, `response_format` with `concise` the default, and nothing that writes a file. A command that changes the project gets no tool that changes it: its tool gives the changes to make, as `ascribe_fmt` and `ascribe_rename` do (decision 56, decision [#962](decisions/962-mcp-tool-names.md)).
- [ ] **A path in what it prints, or in a prompt it writes,** is from the folder the command was run in, the form it takes as an argument. A prompt says once which folder its commands are for.
- [ ] The tools of `ascribe mcp`, whose names and arguments are a contract (decision 56): the tools table under `ascribe mcp` in `docs/content/agents/commands.md` and each tool's command's section, in `docs/content/command/questions.md`.

## A new editor command, action, or setting

- [ ] The manifest, `packages/vscode/package.json`, and for an action, `packages/vscode/src/actions/registry.ts`. The docs' tables are generated from both (`packages/vscode/test/unit/docs.test.ts`), and so is the map of the surface.
- [ ] **A command's title** is in title case, as VS Code's own are, under the category **Ascribe**: **Open Page Preview**, **Start Review**. It names what happens, not the mechanism.
- [ ] **An action's label** is a short sentence without a full stop, in the words of the person writing: **Change where the link goes**, **Wrap in a note**. It's what the actions bar and the lightbulb show. Where the palette shows it too, it's the same label, so the palette has two styles on purpose: commands that work the editor, and actions that change the page.
- [ ] **A setting** is `ascribe.<name>`, or `ascribe.<area>.<name>` once an area has two, in camel case. It's for one person's editor: how it scrolls, when servers start, which binary runs. A choice about the content, which everyone working on the project must share, is in `ascribe.toml`. A choice about how a site uses a build is an option of `@ascribed/astro`. A choice about one run is a flag.
- [ ] **What the editor says when it isn't looking.** A state in which Ascribe hasn't checked (an untrusted folder, a server that hasn't started, a build other than the editor's) is said in the status bar, never left to read as "no problems".
- [ ] A tool for agents in VS Code only for what the editor alone has: text that isn't saved, or the review that's open (decision 57). Everything else is the MCP server's.

## A change to the site output's markup

An element, an attribute, a class, or the order of what the `site` output writes.

- [ ] The site output, in `crates/ascribe-emit/src/site/`, and its contract in `docs/content/contracts/site-render.md`.
- [ ] A new or renamed element, attribute, class, or id: in `crates/ascribe-core/src/names.rs`, its one home, then each package's generated `names.ts` (`packages/astro/src/names.ts`, `packages/elements/src/names.ts`, `packages/review/src/names.ts`, and `packages/vscode/src/names.ts`), rewritten by `ASCRIBE_BLESS=1 cargo test -p ascribe-core --test names`. Code uses the constant, never the literal; that test fails on a literal elsewhere, and on a stylesheet selecting a name that isn't declared.
- [ ] Ascribe's own renderer, `render_site_html` in `crates/ascribe-emit/src/render/`, which the page preview and the HTML preview use.
- [ ] The fixtures in `tests/render/`, which both renderers must pass: ours, and the site's through `@ascribed/astro`.
- [ ] Source anchors in review mode (`crates/ascribe-emit/src/site/anchor.rs`, tested by `crates/ascribe-emit/tests/all/site_anchors.rs`): the page with anchors is the page without them, plus the anchors.
- [ ] `@ascribed/elements`: `packages/elements/src/`, `packages/elements/css/style.css`, and its contract, `packages/elements/CONTRACT.md`.
- [ ] `@ascribed/astro`: `packages/astro/src/rehype.ts`, `packages/astro/src/attributes.ts`, `packages/astro/src/code-titles.ts`, and the components `packages/astro/src/Elements.astro` and `packages/astro/src/Availability.astro`.
- [ ] `@ascribed/review`'s marks and placement, in `packages/review/src/marks/` and `packages/review/src/place/`.
- [ ] The page preview in VS Code: `packages/vscode/src/preview/` and `packages/vscode/src/webview/`.
- [ ] The HTML preview, `crates/ascribe-diff/src/html/`. Its embedded script and styles are rebuilt from `packages/review` and `packages/elements` by `pnpm --filter @ascribed/review embed`.
- [ ] The Astro example's end-to-end tests, in `examples/astro-site/test/e2e/`.
- [ ] The docs site's own styles, in `site/src/styles/site.css`, if they style the changed markup.
- [ ] Its changelog entry, a file of its own in `changes/` ([changes/README.md](../changes/README.md)), not a line in `CHANGELOG.md`.

## A change to how something looks

A color, a font, the type scale, a space or radius, the mark, or one of its images, on any surface: the docs site, the element library, review, the HTML preview, the Astro toolbar, or the extension. [design/README.md](../design/README.md) has the steps.

- [ ] The value is in `design/tokens.toml`, and a stylesheet takes it through its `[emit]` block; no stylesheet writes a color of its own.
- [ ] The generated blocks, and `design/candidates/chosen.toml`'s shared colors, rewritten (`ASCRIBE_BLESS=1 pnpm exec vitest run scripts/design/tokens.test.ts`), and the diff read.
- [ ] The report's embedded stylesheet rebuilt (`pnpm --filter @ascribed/review embed`).
- [ ] The mark's images rewritten (`node scripts/design/assets.ts`) if a shared color or a source SVG changed, and looked at.
- [ ] Contrast: a new text or meaningful graphic color has its pairings in `design/candidates/pairs.toml`, and `pnpm exec vitest run scripts/design` passes.
- [ ] The specimen rewritten (`node scripts/design/specimen.ts`) and looked at.
- [ ] Screenshots in light and dark of each surface it reaches, in the pull request.
- [ ] `node scripts/compare/outputs.ts --base main`: only the HTML preview's files differ, accepted by name, with the label `outputs changed`.
- [ ] A changed default of an `--ascribe-*` or `--ascribe-review-*` property: a **Behavior change** in its changelog entry in `changes/`, with the previous values (decision 44). A new property is a contract addition, recorded as a decision.
- [ ] In the extension: a codicon and a theme color first (`packages/vscode/DEVELOPMENT.md`, "Icons and colors").
