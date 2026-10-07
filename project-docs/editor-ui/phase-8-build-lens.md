# Phase 8: The build lens

Part of [Editor UI](README.md). Requires phase 6 (the status bar item). Server and extension.

## Goal

Turn the lens on, and the editor dims what the build you're looking at leaves out of the page: variant arms the build doesn't select, and content whose availability the build filters out, table rows included, with a hover that says why. "What will self-hosted readers see?" is answered in the source, without opening the preview.

## Context

- `crates/ascribe-model/src/model.rs`: `Build`, whose fields `VariantMode` and `AvailabilityMode` say how it treats variants and availability.
- `crates/ascribe-resolve/src/build/tree.rs`: how a build resolves a page, and `DropReason`, why a build drops a whole page. `ascribe/preview` (`crates/ascribe-lsp/src/preview.rs`) already resolves a page for a chosen build.
- Phase 6's status bar item and `src/ui/chosenBuild.ts`, the one build per project that the preview renders (README decision 10), and phase 1's `ascribe/targets` (`builds`).
- VS Code: `window.createTextEditorDecorationType` (`opacity`), `setDecorations`, and hover messages on decorations.

## Design

### Server: `ascribe/buildView`

Params: `{ textDocument, build }`. Result:

```jsonc
{
  "build": "self-hosted",
  "pageIncluded": true,           // false if the build drops the whole page (its availability)
  "excluded": [
    { "range": …, "reason": "variant", "detail": "Shows only edition=self-hosted" },
    { "range": …, "reason": "availability", "detail": "Available on cloud only (preview)" }
  ]
}
```

Ranges are source ranges of whole blocks or arms (from the directive line to `@end` for a container), so dimming covers the syntax too. A table row a filter build removes is its whole line. Content from an included fragment isn't in this file and isn't shown. Reuse the resolver's drop decisions; don't decide inclusion a second way.

### Extension

- **Turning it on:** a command, **Ascribe: Dim What the Build Leaves Out** (also in the status bar item's quick pick), turns the lens on or off for the project, for the session. The lens has no build of its own: it dims by the project's chosen build, the one the preview renders, and **Switch build** or the preview's picker changes both.
- **Showing it:** the status bar item shows that the lens is on (`$(eye) docs · self-hosted`).
- **Dimming:** excluded ranges are dimmed (reduced opacity, respecting the theme), with a hover message that gives the reason and detail. If the whole page is excluded, an info line at the top says so.
- **Updates:** on edits (debounced), on switching files, when the chosen build changes, and when the model changes. Dimming never changes the text, folding, or diagnostics.

## Tasks

1. `ascribe/buildView` with scenario tests:
   - variant selection, including nested groups inside steps;
   - availability filtering of sections, blocks, and table rows;
   - a page dropped entirely;
   - a `switch` build, where nothing is excluded;
   - unsaved edits.
2. Derive `JsonSchema` on the result, add it to `SHAPES`, and bless.
3. The command, the status bar integration, and the decorations, with unit tests for mapping results to decorations, and an integration test on a copy of `examples/monorepo/docs`: `self-hosted` dims the cloud arm in `getting-started.md` and the scheduled-rollouts section in `guides/rollouts.md`; switching the build in the preview's picker changes what's dimmed.
4. Document `ascribe/buildView` in `crates/ascribe-lsp/README.md`. A description for the command in `docs.test.ts`'s `commands`, then bless the commands table. The lens in `docs/content/guides/editor.md`; `CHANGELOG.md`.

## Out of scope

A build for the lens that differs from the preview's; dimming in the preview (it already renders the chosen build); showing fragments' content inline.

## Acceptance criteria

- What's dimmed matches what `ascribe build --build <name> --emit plain` leaves out of the page. Test it by comparing the two for every page and build of both example projects.
- Turning the lens off removes every decoration.
- With the lens on and a preview open, both always show the same build.

## Verify

```sh
cargo test --workspace --locked
pnpm --filter ascribe-vscode test
cargo build -p ascribe-cli && ASCRIBE_BIN=$PWD/target/debug/ascribe pnpm --filter ascribe-vscode test:integration
pnpm lint && pnpm format:check && cargo fmt --all --check && cargo clippy --workspace --all-targets --locked -- -D warnings
```

## Commits

1. "Report what a build leaves out of a page: ascribe/buildView"
2. "Dim what the chosen build leaves out"
