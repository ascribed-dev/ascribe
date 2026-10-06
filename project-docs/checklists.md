# Change checklists

What a change has to touch, as lists a reviewer can tick. Each list was built from the code; when the code grows a new surface, the list changes in the same pull request. Not every line applies to every change: a reviewer ticks a line or says why it doesn't apply.

The rules every change follows (the checks to run, conformance cases, the fork, the docs) are in [CONTRIBUTING.md](../CONTRIBUTING.md). The decisions behind them are in [decisions.md](decisions.md).

## A change to the language

New or changed syntax, a directive, a block or inline form, an attribute, or what a content model can say.

**What it means**

- [ ] `SPEC.md` says what the change means, and its examples parse.
- [ ] Conformance cases in `tests/conformance/cases/`, with outlines and builds as `tests/conformance/README.md` describes. A case the change can't pass yet is listed in `tests/conformance/SKIPS.toml`, with the reason.

**Reading it**

- [ ] Parsing, in `crates/tessera-syntax/`. A change to the parser itself is in `crates/comrak-tessera/`, marked as `crates/comrak-tessera/FORK.md` says.
- [ ] The content model, in `crates/tessera-model/`, if `ascribe.toml` can say something new.
- [ ] Resolving, in `crates/tessera-resolve/`: includes, snippets, phrases, links, and builds.

**Checking it**

- [ ] The checks, in `crates/tessera-check/src/checks/`.
- [ ] Each new or changed diagnostic, in `tests/conformance/diagnostics.toml` only; the diagnostics reference is generated from it.

**Each output, and its contract**

- [ ] `site`: `crates/tessera-emit/src/site/`, and the contract in `docs/content/contracts/site-render.md`. Then the [site output's markup](#a-change-to-the-site-outputs-markup) list below.
- [ ] `plain`: `crates/tessera-emit/src/plain/`.
- [ ] `json`: `crates/tessera-emit/src/json.rs`, and its shape in `crates/tessera-emit/README.md`. A removed field, or one that changes meaning, raises the schema version.
- [ ] The Zod schema for frontmatter, in `crates/tessera-emit/src/zod/`, checked by `tests/zod/`.
- [ ] The diff, in `crates/tessera-diff/`: a new kind of block aligns and reports its changes.

**Formatting**

- [ ] `crates/tessera-fmt/` writes the new form canonically, and `ascribe fmt` changes nothing on a page already in canonical form.

**The language server, feature by feature**

The features `crates/tessera-lsp/src/server.rs` handles. Each new form is considered for each one.

- [ ] Diagnostics, pushed as the project changes (`crates/tessera-lsp/src/core.rs`).
- [ ] Semantic tokens, full and range (`SemanticTokensFullRequest`, `SemanticTokensRangeRequest`; `crates/tessera-lsp/src/tokens.rs`). A new token type is added to the legend in `crates/tessera-lsp/README.md` and to `semanticTokenTypes` and `semanticTokenScopes` in `packages/vscode/package.json`.
- [ ] Completion (`Completion`; `crates/tessera-lsp/src/complete.rs`), and its trigger characters in `server.rs` if the form starts with a new one.
- [ ] Hover (`HoverRequest`; `crates/tessera-lsp/src/hover.rs`).
- [ ] Go to definition (`GotoDefinition`; `crates/tessera-lsp/src/definition.rs`).
- [ ] Document links (`DocumentLinkRequest`), CodeLens (`CodeLensRequest`), and inlay hints (`InlayHintRequest`), all in `crates/tessera-lsp/src/links.rs`.
- [ ] Quick fixes and other code actions (`CodeActionRequest`; `crates/tessera-lsp/src/code_action.rs`).
- [ ] Formatting (`Formatting`; `crates/tessera-lsp/src/formatting.rs`), which calls `tessera-fmt`.
- [ ] Rename (`Rename`), and the edits when a file is moved (`WillRenameFiles`), both in `crates/tessera-lsp/src/refactor.rs`.
- [ ] The command a CodeLens runs (`ExecuteCommand`, `ascribe.openFile`).
- [ ] The page preview (`ascribe/preview`; `crates/tessera-lsp/src/preview.rs`).
- [ ] Review's changes (`ascribe/review/setBase` and `ascribe/review/changes`; `crates/tessera-lsp/src/review.rs`).
- [ ] The server's tests in `crates/tessera-lsp/tests/`, and `lsp_parity` in `crates/tessera-cli/tests/`, which compares the server's diagnostics with `ascribe check`'s.

**The editor**

- [ ] The VS Code grammar, in `packages/vscode/syntaxes/`.

**The docs**

- [ ] The reference: `docs/content/reference/directives.md` or `docs/content/reference/content-model.md`, and any guide that shows the form. What no release has yet is marked `@available: next`.
- [ ] A line in the unreleased section of `CHANGELOG.md`.

## A new command or option

- [ ] The arguments and help text, in `crates/tessera-cli/src/cli.rs` and `crates/tessera-cli/src/commands/`. The work itself is one entry function in a library crate, taking plain arguments and returning a typed result, so the language server and other callers can reach it; the command parses its arguments, calls it, and reports. A new command's entry goes in ARCHITECTURE.md's [Each command's entry](../ARCHITECTURE.md#each-commands-entry).
- [ ] Exit codes from `crates/tessera-cli/src/exit.rs` only, and the command's "Exit codes" section in `docs/content/reference/cli.md` says which it uses.
- [ ] The command reference's fragments in `docs/content/_generated/`, regenerated from the help text by `crates/tessera-cli/src/docs.rs` (`ASCRIBE_BLESS=1 cargo test -p tessera-cli docs`). A new command also gets a section in `docs/content/reference/cli.md` that includes its fragment. An option no release has yet is listed in `UNRELEASED` in `docs.rs`.
- [ ] With `--format json`: the shape's fields documented in the command's section of `docs/content/reference/cli.md`, keys in snake_case, and a schema version that rises when a field is removed or changes meaning.
- [ ] The TypeScript that reads the shape, if any: `packages/astro/src/review/protocol.ts`, `packages/review/src/report/index.ts`, `packages/vscode/src/preview/protocol.ts`, and `packages/vscode/src/preview/reviewText.ts` read the `diff` JSON today.
- [ ] The TypeScript that runs the command, if it passes the new option: `packages/astro/src/run.ts`.
- [ ] Tests in `crates/tessera-cli/tests/`. Output the docs show is written to `crates/tessera-cli/tests/output/` by `crates/tessera-cli/tests/output.rs`.
- [ ] A line in the unreleased section of `CHANGELOG.md`.
- [ ] The agents plan's table of tools, once that plan has started (see [agents](agents/README.md)).

## A change to the site output's markup

An element, an attribute, a class, or the order of what the `site` output writes.

- [ ] The site output, in `crates/tessera-emit/src/site/`, and its contract in `docs/content/contracts/site-render.md`.
- [ ] A new or renamed element, attribute, class, or id: in `crates/tessera-core/src/names.rs`, its one home, then each package's generated `names.ts` (`packages/astro/src/names.ts`, `packages/elements/src/names.ts`, `packages/review/src/names.ts`, and `packages/vscode/src/names.ts`), rewritten by `ASCRIBE_BLESS=1 cargo test -p tessera-core --test names`. Code uses the constant, never the literal; that test fails on a literal elsewhere, and on a stylesheet selecting a name that isn't declared.
- [ ] Ascribe's own renderer, `render_site_html` in `crates/tessera-emit/src/render/`, which the page preview and the HTML report use.
- [ ] The fixtures in `tests/render/`, which both renderers must pass: ours, and the site's through `@ascribed/astro`.
- [ ] Source anchors in review mode (`crates/tessera-emit/src/site/anchor.rs`, tested by `crates/tessera-emit/tests/site_anchors.rs`): the page with anchors is the page without them, plus the anchors.
- [ ] `@ascribed/elements`: `packages/elements/src/`, `packages/elements/css/style.css`, and its contract, `packages/elements/CONTRACT.md`.
- [ ] `@ascribed/astro`: `packages/astro/src/rehype.ts`, `packages/astro/src/attributes.ts`, `packages/astro/src/code-titles.ts`, and the components `packages/astro/src/Elements.astro` and `packages/astro/src/Availability.astro`.
- [ ] `@ascribed/review`'s marks and placement, in `packages/review/src/marks/` and `packages/review/src/place/`.
- [ ] The page preview in VS Code: `packages/vscode/src/preview/` and `packages/vscode/src/webview/`.
- [ ] The HTML report, `crates/tessera-diff/src/html/`. Its embedded script and styles are rebuilt from `packages/review` and `packages/elements` by `pnpm --filter @ascribed/review embed`.
- [ ] The Astro example's end-to-end tests, in `examples/astro-site/test/e2e/`.
- [ ] The docs site's own styles, in `site/src/styles/site.css`, if they style the changed markup.
- [ ] A line in the unreleased section of `CHANGELOG.md`.
