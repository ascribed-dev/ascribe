# Phase 8: The build lens

Part of [Editor UI](README.md). Requires phase 6 (the status bar item). Server and extension.

## Goal

Choose a build, and the editor dims what that build leaves out of the page: variant arms the build doesn't select, and content whose availability the build filters out, with a hover that says why. "What will self-hosted readers see?" is answered in the source, without opening the preview.

## Context

- `crates/ascribe-resolve`: how a build resolves a page (`VariantMode` selection, `AvailabilityMode` filter, `DropReason`), already used by `ascribe/preview` (`crates/ascribe-lsp/src/preview.rs`).
- `crates/ascribe-model`: `Build`, its `variants` and `availability` settings.
- Phase 6's status bar item, and phase 1's `ascribe/targets` (`builds`).
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

Ranges are source ranges of whole blocks or arms (from the directive line to `@end` for a container), so dimming covers the syntax too. Content from an included fragment isn't in this file and isn't shown. Reuse the resolver's drop decisions; don't decide inclusion a second way.

### Extension

- **Choosing the build:** a command, **Ascribe: Show the page as a build** (also in the status bar item's quick pick), lists the project's builds and **Off**. The choice is per project and lasts for the session.
- **Showing it:** the status bar item shows the lens build when one is on (`$(eye) docs · self-hosted`).
- **Dimming:** excluded ranges are dimmed (reduced opacity, respecting the theme), with a hover message that gives the reason and detail. If the whole page is excluded, an info line at the top says so.
- **Updates:** on edits (debounced), on switching files, and when the model changes. Dimming never changes the text, folding, or diagnostics.

## Tasks

1. `ascribe/buildView` with scenario tests:
   - variant selection, including nested groups inside steps;
   - availability filtering of sections and blocks;
   - a page dropped entirely;
   - a `switch` build, where nothing is excluded;
   - unsaved edits.
2. The command, the status bar integration, and the decorations, with unit tests for mapping results to decorations, and an integration test on a copy of `examples/monorepo/docs`: `self-hosted` dims the cloud arm in `getting-started.md` and the scheduled-rollouts section in `guides/rollouts.md`.
3. Document `ascribe/buildView` in `crates/ascribe-lsp/README.md`; the lens in `docs/editor.md`; `CHANGELOG.md`.

## Out of scope

Dimming in the preview (it already renders a chosen build); showing fragments' content inline.

## Acceptance criteria

- What's dimmed matches what `ascribe build --build <name> --emit plain` leaves out of the page. Test it by comparing the two for every page and build of both example projects.
- Turning the lens off removes every decoration.

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
