# Developing the VS Code extension

How the extension is built and tested. The user guide is the [README](README.md) and [docs/editor.md](../../docs/editor.md).

## The binary

`src/binary.ts` finds the `ascribe` binary: the `ascribe.path` setting, then the project's `node_modules/.bin/ascribe` (in the project's folder, and its parents up to the workspace folder), then the binary bundled at `bin/<platform>-<arch>/ascribe` (`ascribe.exe` on Windows). It runs `--version` on each candidate, and warns when the version is older than `ascribe.minServerVersion` in `package.json`, which a release sets to its own version. Release packaging (`scripts/release/pack.ts`) stages the one binary each platform's package needs under `bin/`.

## Projects

`src/registry.ts` finds every `ascribe.toml` in the workspace (outside `node_modules`, at most 50) and keeps a `ProjectServer` (`src/client.ts`) for each: one `ascribe lsp`, with the project's folder as its workspace folder, its own binary, output channel, and crash count. A server starts the first time a file of its project is opened or previewed, or at discovery with `ascribe.startServers: "all"`. `src/projects.ts` decides which project owns a file (the nearest `ascribe.toml` above it), and each client is kept to its own project's files, so a parent project's server never sees a nested project's open documents. The user-facing behavior is in [docs/editor.md](../../docs/editor.md#workspaces-with-several-projects).

## The preview

**Ascribe: Open Preview to the Side** opens a panel that shows the active page
as the published site shows it, and follows the editor: edits appear within
about a tenth of a second (the debounce is 100 ms; measured end to end, median
113 ms, maximum 167 ms over 12 edits in the integration suite), including
unsaved ones; the scroll position stays; the panel scrolls to the section the
cursor is in; a click on a link to a page or a file opens it in the editor.
Its **Build** picker lists the content model's builds and starts at the
editor's (`[editor] build`).

It doesn't render anything itself. It sends the language server the custom
request `ascribe/preview` (`crates/tessera-lsp/README.md`), whose answer is
the site markdown (the site emitter) rendered by the same code that
the Astro plugin's fixtures pin (`render_site_html`, `tests/render/`). The
webview draws that HTML with `@ascribed/elements`, bundled into
`dist/webview/` (`elements.js`, `elements.css`) next to the preview's own
script and stylesheet, and gives the page a title and page-level availability
from the frontmatter. Assets aren't copied: the server names each
asset's source file, and the webview shows it from there.

**What the webview may read** (`localResourceRoots`): the extension's
`dist/webview/`, the project's content root, and the directory of each asset
the page uses that is outside the content root (`../shared/logo.png` makes
`shared/` readable), and nothing else: never the project root, `node_modules`,
or the output directory. An asset directly in the project root isn't shown,
with a warning.

**Content security policy** (`src/preview/html.ts`, tested in a real browser
in `test/webview/`):

```
default-src 'none'; script-src <origin>; style-src <origin>; img-src <origin>; font-src <origin>
```

`<origin>` is `webview.cspSource`: where VS Code serves the extension's and
the project's local files. The element library is a script file and a
stylesheet file; it uses no inline script, sets no inline style (it is styled
by the stylesheet and `--ascribe-*` custom properties), and needs no
`eval`, so it runs under the policy with **no `'unsafe-inline'`, no nonce, and no
`'unsafe-eval'`**. The cost is that raw HTML in a page that needs an inline
script, an inline `style` attribute, or a remote image behaves differently in
the preview than on the site. Link clicks are handled by the
extension, which opens only `http:`, `https:`, and `mailto:` URLs outside VS
Code.

`pnpm --filter ascribe-vscode build` bundles the element library, from the
`@ascribed/elements` package's source and stylesheet, into `dist/webview/`.

## Highlighting

`syntaxes/` holds two TextMate injections into markdown (one for top level, one
for lists and quotes, where the item's indentation is unknown): directive lines
(sigil, name, attribute block, colon, and the first line of the primary), `@end`,
and `{key}` phrases. TextMate can't see past a line, and doesn't know the content
model, so the rest is the server's semantic tokens: title lines, declared and
undeclared phrases, project widgets, and a text primary's later lines. The
`semanticTokenTypes` and `semanticTokenScopes` in `package.json` map the server's
legend (`crates/tessera-lsp/README.md`) to theme scopes; a unit test keeps them
in step.

## Development

```
pnpm --filter ascribe-vscode build              # bundle to dist/extension.cjs
pnpm --filter ascribe-vscode test               # unit tests and the webview tests (vitest, Chromium)
pnpm --filter ascribe-vscode test:integration   # VS Code integration tests
pnpm --filter ascribe-vscode test:parity        # the preview against the Astro site's built HTML
```

The integration tests download VS Code into `out/vscode-test` and need a
display: on Linux without one, use `pnpm --filter ascribe-vscode
test:integration:headless` (it runs under `xvfb-run -a`). They have five
suites: `activation` (no `ascribe.toml`: the extension stays off), `stub` (a
stub server in `test/stub-server`), `quill` (the real `ascribe lsp` on a
copy of `examples/quill` with a broken page added), `preview` (the preview
panel against the real server on a copy of `examples/quill`), and `monorepo`
(several projects, one nested in another, in `test/fixtures/monorepo`); the
last three run only when `ASCRIBE_BIN` names a built `ascribe`. `ASCRIBE_SUITE` runs one suite.

The webview tests (`test/webview/`) load the preview's shell and bundles into
Chromium under the real policy (`/opt/pw-browsers/chromium`, or
`ASCRIBE_CHROMIUM`) and post the messages the extension sends.

The parity test (`test/parity/`) builds `examples/astro-site` with its own
`build` script, in place and in two copies under its `.e2e-tmp/` (one with
every arm of every group, one filtered to the cloud), starts `ascribe lsp` on
each, and compares each page's preview HTML with Astro's built `<article>`:
elements, attributes, heading ids, image attributes, and asset URLs by source
file. It needs `cargo build -p tessera-cli` (or `ASCRIBE_BIN`), and the
`@ascribed/elements` and `@ascribed/astro` builds (the Astro site uses them). What it leaves out, and why,
is in `test/parity/normalize.ts`.

`test/fixtures/markdown.tmLanguage.json` is VS Code's markdown grammar (MIT,
microsoft/vscode), so the grammar tests see the scopes it really produces.
