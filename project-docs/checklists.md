# Change checklists

What a change has to touch, as lists a reviewer can tick. Each list was built from the code; when the code grows a new surface, the list changes in the same pull request. Not every line applies to every change: a reviewer ticks a line or says why it doesn't apply.

The rules every change follows (the checks to run, conformance cases, the fork, the docs) are in [CONTRIBUTING.md](../CONTRIBUTING.md). The decisions behind them are in [decisions.md](decisions.md).

## A change to the language

New or changed syntax, a directive, a block or inline form, an attribute, or what a content model can say.

**What it means**

- [ ] `SPEC.md` says what the change means, and its examples parse.
- [ ] Conformance cases in `tests/conformance/cases/`, with outlines and builds as `tests/conformance/README.md` describes. A case the change can't pass yet is listed in `tests/conformance/SKIPS.toml`, with the reason.

**Reading it**

- [ ] Parsing, in `crates/ascribe-syntax/`. A change to the parser itself is in `crates/comrak-ascribe/`, marked as `crates/comrak-ascribe/FORK.md` says.
- [ ] The content model, in `crates/ascribe-model/`, if `ascribe.toml` can say something new.
- [ ] Resolving, in `crates/ascribe-resolve/`: includes, snippets, phrases, links, and builds.

**Checking it**

- [ ] The checks, in `crates/ascribe-check/src/checks/`.
- [ ] Each new or changed diagnostic, in `tests/conformance/diagnostics.toml` only; the diagnostics reference is generated from it.

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
- [ ] Document links (`DocumentLinkRequest`), CodeLens (`CodeLensRequest`), and inlay hints (`InlayHintRequest`), all in `crates/ascribe-lsp/src/links.rs`.
- [ ] Quick fixes and other code actions (`CodeActionRequest`; `crates/ascribe-lsp/src/code_action.rs`).
- [ ] Formatting (`Formatting`; `crates/ascribe-lsp/src/formatting.rs`), which calls `ascribe-fmt`.
- [ ] Rename (`Rename`), and the edits when a file is moved (`WillRenameFiles`), both in `crates/ascribe-lsp/src/refactor.rs`.
- [ ] The command a CodeLens runs (`ExecuteCommand`, `ascribe.openFile`).
- [ ] The page preview (`ascribe/preview`; `crates/ascribe-lsp/src/preview.rs`).
- [ ] Review's changes (`ascribe/review/setBase` and `ascribe/review/changes`; `crates/ascribe-lsp/src/review.rs`).
- [ ] What's at a position (`ascribe/context`; `crates/ascribe-lsp/src/context.rs`): a new block or inline construct is a node kind, and a new token is a token kind.
- [ ] What actions can point at (`ascribe/targets`; `crates/ascribe-lsp/src/targets.rs`): a new kind of content model entry is a kind of target.
- [ ] The server's tests in `crates/ascribe-lsp/tests/`, and `lsp_parity` in `crates/ascribe-cli/tests/`, which compares the server's diagnostics with `ascribe check`'s.

**The editor**

- [ ] The VS Code grammar, in `packages/vscode/syntaxes/`.

**The docs**

- [ ] The reference: `docs/content/reference/directives.md` or `docs/content/reference/content-model.md`, and any guide that shows the form. What no release has yet is marked `@available: next`.
- [ ] A line in the unreleased section of `CHANGELOG.md`.

## A new command or option

- [ ] The arguments and help text, in `crates/ascribe-cli/src/cli.rs` and `crates/ascribe-cli/src/commands/`. The work itself is one entry function in a library crate, taking plain arguments and returning a typed result, so the language server and other callers can reach it; the command parses its arguments, calls it, and reports. A new command's entry goes in ARCHITECTURE.md's [Each command's entry](../ARCHITECTURE.md#each-commands-entry).
- [ ] Exit codes from `crates/ascribe-cli/src/exit.rs` only, and the command's "Exit codes" section in `docs/content/reference/cli.md` says which it uses.
- [ ] The command reference's fragments in `docs/content/_generated/`, regenerated from the help text by `crates/ascribe-cli/src/docs.rs` (`ASCRIBE_BLESS=1 cargo test -p ascribe-cli docs`). A new command also gets a section in `docs/content/reference/cli.md` that includes its fragment. An option no release has yet is listed in `UNRELEASED` in `docs.rs`.
- [ ] With `--format json`: the shape's fields documented in the command's section of `docs/content/reference/cli.md`, keys in snake_case, and a schema version that rises when a field is removed or changes meaning.
- [ ] The shape's schema and TypeScript, generated from its Rust types by `crates/ascribe-cli/src/shapes.rs` (`ASCRIBE_BLESS=1 cargo test -p ascribe-cli shapes`): a doc comment on every field, a new shape added to `SHAPES` (and to `PACKAGES` when a package reads it), and a new command's schema included in `docs/content/contracts/json-reports.md`. The packages read the generated `packages/astro/src/shapes.ts`, `packages/review/src/shapes.ts`, and `packages/vscode/src/shapes.ts`, never a type of their own.
- [ ] The TypeScript that runs the command, if it passes the new option: `packages/astro/src/run.ts`.
- [ ] Tests in `crates/ascribe-cli/tests/`. Output the docs show is written to `crates/ascribe-cli/tests/output/` by `crates/ascribe-cli/tests/output.rs`.
- [ ] A line in the unreleased section of `CHANGELOG.md`.
- [ ] The agents plan's table of tools, once that plan has started (see [agents](agents/README.md)).

## A change to the site output's markup

An element, an attribute, a class, or the order of what the `site` output writes.

- [ ] The site output, in `crates/ascribe-emit/src/site/`, and its contract in `docs/content/contracts/site-render.md`.
- [ ] A new or renamed element, attribute, class, or id: in `crates/ascribe-core/src/names.rs`, its one home, then each package's generated `names.ts` (`packages/astro/src/names.ts`, `packages/elements/src/names.ts`, `packages/review/src/names.ts`, and `packages/vscode/src/names.ts`), rewritten by `ASCRIBE_BLESS=1 cargo test -p ascribe-core --test names`. Code uses the constant, never the literal; that test fails on a literal elsewhere, and on a stylesheet selecting a name that isn't declared.
- [ ] Ascribe's own renderer, `render_site_html` in `crates/ascribe-emit/src/render/`, which the page preview and the HTML report use.
- [ ] The fixtures in `tests/render/`, which both renderers must pass: ours, and the site's through `@ascribed/astro`.
- [ ] Source anchors in review mode (`crates/ascribe-emit/src/site/anchor.rs`, tested by `crates/ascribe-emit/tests/site_anchors.rs`): the page with anchors is the page without them, plus the anchors.
- [ ] `@ascribed/elements`: `packages/elements/src/`, `packages/elements/css/style.css`, and its contract, `packages/elements/CONTRACT.md`.
- [ ] `@ascribed/astro`: `packages/astro/src/rehype.ts`, `packages/astro/src/attributes.ts`, `packages/astro/src/code-titles.ts`, and the components `packages/astro/src/Elements.astro` and `packages/astro/src/Availability.astro`.
- [ ] `@ascribed/review`'s marks and placement, in `packages/review/src/marks/` and `packages/review/src/place/`.
- [ ] The page preview in VS Code: `packages/vscode/src/preview/` and `packages/vscode/src/webview/`.
- [ ] The HTML report, `crates/ascribe-diff/src/html/`. Its embedded script and styles are rebuilt from `packages/review` and `packages/elements` by `pnpm --filter @ascribed/review embed`.
- [ ] The Astro example's end-to-end tests, in `examples/astro-site/test/e2e/`.
- [ ] The docs site's own styles, in `site/src/styles/site.css`, if they style the changed markup.
- [ ] A line in the unreleased section of `CHANGELOG.md`.

## A change to how something looks

A color, a font, the type scale, a space or radius, the mark, or one of its images, on any surface: the docs site, the element library, review, the HTML report, the Astro toolbar, or the extension. [design/README.md](../design/README.md) has the steps.

- [ ] The value is in `design/tokens.toml`, and a stylesheet takes it through its `[emit]` block; no stylesheet writes a color of its own. A changed shared color is changed in `design/candidates/chosen.toml` too.
- [ ] The generated blocks rewritten (`ASCRIBE_BLESS=1 pnpm exec vitest run scripts/design/tokens.test.ts`), and the diff read.
- [ ] The report's embedded stylesheet rebuilt (`pnpm --filter @ascribed/review embed`).
- [ ] The mark's images rewritten (`node scripts/design/assets.ts`) if a shared color or a source SVG changed, and looked at.
- [ ] Contrast: a new text or meaningful graphic color has its pairings in `design/candidates/pairs.toml`, and `pnpm exec vitest run scripts/design` passes.
- [ ] The specimen rewritten (`node scripts/design/specimen.ts`) and looked at.
- [ ] Screenshots in light and dark of each surface it reaches, in the pull request.
- [ ] `node scripts/compare/outputs.ts --base main`: only the HTML report's files differ, accepted by name, with the label `outputs changed`.
- [ ] A changed default of an `--ascribe-*` or `--ascribe-review-*` property: a **Behavior change** in the unreleased section of `CHANGELOG.md`, with the previous values (decision 44). A new property is a contract addition, recorded as a decision.
- [ ] In the extension: a codicon and a theme color first (`packages/vscode/DEVELOPMENT.md`, "Icons and colors").
