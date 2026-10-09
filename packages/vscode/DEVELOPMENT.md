# Developing the VS Code extension

How the extension is built and tested. The user guide is the [README](README.md) and [docs/content/guides/editor.md](../../docs/content/guides/editor.md).

## Running it locally

Open the repository in VS Code and press F5, or pick a configuration in **Run and Debug**:

- **Extension: examples/quill** opens `examples/quill`, one project.
- **Extension: several projects** opens [`examples/monorepo`](../../examples/monorepo), a repository with code and three projects, one nested in another. Its README lists things to try.

Either one runs the `extension: prepare` task first, then opens an Extension Development Host: a second VS Code window running this checkout's extension, with your other extensions turned off. The task builds the server (`cargo build -p ascribe-cli`), copies it to `bin/<platform>-<arch>/`, where the extension looks for its bundled binary, and bundles the extension and the webview files. The development window loads this extension in place of an installed copy of Ascribe. Breakpoints in `src/` work in the first window.

After changing the extension, rebuild and run **Developer: Reload Window** in the development window. To rebuild on each save, run the `extension: watch` task (or `pnpm --filter ascribe-vscode watch`), then only reload. After changing the server, run the `extension: stage server` task and restart it with **Ascribe: Restart Language Server**.

Without VS Code's Run and Debug, the same steps from a terminal are:

```sh
cargo build -p ascribe-cli
pnpm --filter ascribe-vscode stage-server
pnpm --filter ascribe-vscode build
code --extensionDevelopmentPath="$PWD/packages/vscode" examples/quill
```

The server logs to the development window's **Ascribe** output channels (**Ascribe: Show Server Output**). The extension's own errors are in **Help → Toggle Developer Tools** there, and in the **Debug Console** of the first window.

`bin/` is ignored by git, and release packaging empties it before it stages each target's binary, so a binary staged here never ships. `stage-server` takes another binary as an argument, such as a release build.

## The binary

`src/binary.ts` finds the `ascribe` binary: the `ascribe.path` setting, then the project's `node_modules/.bin/ascribe` (in the project's folder, and its parents up to the workspace folder), then the binary bundled at `bin/<platform>-<arch>/ascribe` (`ascribe.exe` on Windows). It runs `--version` on each candidate, and warns when the version is older than `ascribe.minServerVersion` in `package.json`, which a release sets to its own version. Release packaging (`scripts/release/pack.ts`) stages the one binary each platform's package needs under `bin/`.

## Projects

`src/registry.ts` finds every `ascribe.toml` in the workspace (outside `node_modules`, at most 50) and keeps a `ProjectServer` (`src/client.ts`) for each: one `ascribe lsp`, with the project's folder as its workspace folder, its own binary, output channel, and crash count. A server starts the first time a file of its project is opened or previewed, or at discovery with `ascribe.startServers: "all"`. `src/projects.ts` decides which project owns a file (the nearest `ascribe.toml` above it), and each client is kept to its own project's files (`src/scope.ts`), so a parent project's server never sees a nested project's open documents. The server leaves a nested project's files out of its sources anyway; the middleware is a second guard. The user-facing behavior is in [docs/content/guides/editor.md](../../docs/content/guides/editor.md#workspaces-with-several-projects).

## The preview

**Ascribe: Open Page Preview to the Side** opens a panel that shows the active page
as the published site shows it, and follows the editor: edits appear within
about a tenth of a second (the debounce is 100 ms; measured end to end, median
113 ms, maximum 167 ms over 12 edits in the integration suite), including
unsaved ones; the scroll position stays; the panel scrolls with the editor by
block, both ways; a click on a link to a page or a file opens it in the editor.
Its **Build** picker lists the content model's builds and starts at the
editor's (`[editor] build`).

It doesn't render anything itself. It sends the language server the custom
request `ascribe/preview` (`crates/ascribe-lsp/README.md`), whose answer is
the site markdown (the site emitter) rendered by the same code that
the Astro plugin's fixtures pin (`render_site_html`, `tests/render/`). The
webview draws that HTML with `@ascribed/elements`, bundled into
`dist/webview/` (`elements.js`, `elements.css`) next to the preview's own
script and stylesheet, and gives the page a title and page-level availability
from the frontmatter. Assets aren't copied: the server names each
asset's source file, and the webview shows it from there.

**Scrolling by block.** The preview's HTML has source anchors (site-render
contract §7): each block's element carries `data-ascribe-source` and, for a
block from a fragment, `data-ascribe-via`. `src/preview/blocks.ts` maps them to
lines of the previewed file (a fragment's block to its `@include` line) and a
line to its block, and the webview scrolls by them. The extension sends the
editor's top visible line (`revealLine`) and the cursor's line (`revealLine`
with `ifHidden`, so moving within what the preview shows scrolls nothing); the
webview sends the line of the block at its top when the reader scrolls
(`scrolled`) and of a block they double-click (`openLine`). Each side ignores
the scrolling its own reveal causes for a moment, so the two never chase each
other.

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

**The site preview** (`src/preview/site.ts`). `astro dev` with
`@ascribed/astro` writes `<project>/.ascribe/dev.json` (`url`, `build`, `pid`)
when it starts and removes it when it stops. **Open Site Preview** and the
panel's **Page | Site** switch read it, check the URL answers (any HTTP
response; a crashed server's stale file is ignored), ask `ascribe/preview` for
the page in the dev server's build, and use its `route` on the server's origin.
The address goes through `vscode.env.asExternalUri`, so a remote workspace's
port is forwarded. The switch frames that address in the webview; the policy
then adds `frame-src <dev server origin>`, and nothing else, so the shell is
reloaded when the origin changes. In VS Code for the Web, **Site** opens the
browser instead. The integration suite `site` drives both against a fake dev
server.

`pnpm --filter ascribe-vscode build` bundles the element library, from the
`@ascribed/elements` package's source and stylesheet, into `dist/webview/`.

## Actions

The editor's actions (**Wrap in a note**, **Insert content that varies**, and the rest) are defined once, in `src/actions/registry.ts`: each has a title and description for writers, where it applies (`applies`, from the language server's `ascribe/context` answer), what it asks (`ask`, a wizard of steps built from `ascribe/targets`), and what it does (an `ascribe/edit` operation, or a function of the extension's own, such as **Copy a link to this section**). `crates/ascribe-lsp/README.md` documents the three requests.

The registry feeds the Command Palette (`ascribe.action.<id>`), the editor's context menu (the **Ascribe** submenu, `ascribe.actions`), the lightbulb (code actions of kind `refactor.rewrite.ascribe`), and the actions bar (the command `ascribe.actions`, on `Ctrl+K A` / `Cmd+K A`). VS Code reads commands and menus from `package.json`, so it repeats each action's command, title, and menu `when` clause; `test/unit/actions.test.ts` fails when they differ from the registry, and checks that each `when` clause agrees with `applies` on sample contexts. The commands table and the actions table in the guide are generated from the registry by `test/unit/docs.test.ts`.

- **One cached context** (`src/actions/context.ts`). The extension asks `ascribe/context` for the active editor's selection once it has stayed put for a moment, cancelling the request for a selection it moved on from. The answer sets the context keys the menu's `when` clauses read (`ascribe.at.note`, `ascribe.insertable`, and the others in `CONTEXT_KEYS`). The lightbulb answers from that cache only, and with nothing when it's for another version or range: VS Code asks the lightbulb on every cursor move, and asks again. `ascribe.inProject` is true when the active editor's file belongs to a project.
- **Running an action** (`src/actions/run.ts`): the context for the selection, then the wizard (`src/actions/steps.ts` builds and runs the steps, `src/actions/ask.ts` shows them in the quick input with a step counter and Back), then `ascribe/edit` with the document's version through `ProjectServer.requestEdit`, which converts the server's `WorkspaceEdit` with the language client's converter. The edit is applied with `workspace.applyEdit`, one undo step, and its `select` range is selected. When the page changed after the context was asked for, the server refuses the stale version; the run asks for the context again and tries once more with the same answers.
- **The actions bar** (`src/actions/bar.ts` lists, `src/actions/barPick.ts` shows). A quick pick that shows at once, busy, and fills once the cached context and VS Code's quick fixes at the cursor (`vscode.executeCodeActionProvider`) are in: the fixes first, leaving out the registry's own `quickfix.ascribe` actions, then the actions that apply, by group, each with what it writes when that's known before the wizard (`preview` in the registry). A chosen action runs through the runner; its wizard's first step is shown before the bar is disposed, so it takes the bar's place in the quick input instead of closing it and opening another. Escape before then cancels the wizard.
- **Tests** answer a wizard from a script: `api.actions.answerNext([...])` before running the command, and `api.actions.runs` says what each run did. `api.actions.bars` says what each opening of the bar listed, and `api.actions.selectInBar(label)` makes a row active for `workbench.action.acceptSelectedQuickOpenItem` to choose.

## Highlighting

`syntaxes/` holds two TextMate injections into markdown (one for top level, one
for lists and quotes, where the item's indentation is unknown): directive lines
(sigil, name, attribute block, colon, and the first line of the primary), `@end`,
and `{key}` phrases. TextMate can't see past a line, and doesn't know the content
model, so the rest is the server's semantic tokens: title lines, declared and
undeclared phrases, project widgets, and a text primary's later lines. The
`semanticTokenTypes` and `semanticTokenScopes` in `package.json` map the server's
legend (`crates/ascribe-lsp/README.md`) to theme scopes; a unit test keeps them
in step.

## Icons and colors

The extension takes its look from the user's theme. Ascribe's own colors are in
the Marketplace icon and banner, and in the preview's webview only as fallbacks
for theme colors it can't read.

- **Colors.** The native UI (status bar, tree views, decorations) uses theme
  colors (`new vscode.ThemeColor("…")`), never a hex value; `test/unit/theme.test.ts`
  fails on a color written in `src/`. Declare one in `contributes.colors` only
  when no built-in color fits: an `ascribe.`-prefixed id, a description, and
  `light`, `dark`, `highContrast`, and `highContrastLight` defaults, each a
  reference to a built-in color where one fits. An id is public once released,
  since users override it in `workbench.colorCustomizations`, so it's listed in
  `docs/content/guides/editor.md`. None is declared today.
- **Icons.** [Codicons](https://microsoft.github.io/vscode-codicons/dist/codicon.html)
  for everything they cover, the same one for a concept everywhere:

  | Concept | Codicon |
  |---|---|
  | Project | `$(book)` |
  | Page | `$(file)` |
  | Fragment | `$(file-symlink-file)` |
  | Link | `$(link)` |
  | References, what uses a thing | `$(references)` |
  | Build | `$(package)` |
  | Problem | `$(error)`, `$(warning)`, `$(info)`, by severity |
  | Dimension | `$(symbol-enum)` |
  | Variant, one value of a dimension | `$(symbol-enum-member)` |
  | Phrase | `$(symbol-string)` |
  | Availability | `$(tag)` |

  An icon of Ascribe's own is added only for a concept no codicon reads right
  for. It's drawn as a one-color SVG on the codicon grid (16 pixels, 1 pixel
  strokes), built into one icon font with the mark's assets
  (`scripts/design/assets.ts`), declared in `contributes.icons` as
  `ascribe-<name>`, and used as `$(ascribe-<name>)`; the theme test fails on one
  that's used and not declared. There are none today.
- **The activity bar icon** is `media/activity.svg`, generated from the mark in
  one color for VS Code to tint. The Marketplace icon is `media/icon.png`, and the
  banner behind it (`galleryBanner`) is the icon's tile color, which
  `scripts/design/assets.test.ts` holds to the palette.
  [design/README.md](../../design/README.md#the-assets) has how they're made.

## Development

```
pnpm --filter ascribe-vscode build              # bundle to dist/extension.cjs
pnpm --filter ascribe-vscode watch              # rebuild on each change
pnpm --filter ascribe-vscode stage-server       # copy target/debug/ascribe into bin/
pnpm --filter ascribe-vscode test               # unit tests and the webview tests (vitest, Chromium)
pnpm --filter ascribe-vscode test:integration   # VS Code integration tests
pnpm --filter ascribe-vscode test:parity        # the preview against the Astro site's built HTML
```

The integration tests download VS Code into `out/vscode-test` and need a
display: on Linux without one, use `pnpm --filter ascribe-vscode
test:integration:headless` (it runs under `xvfb-run -a`). Their
suites include `activation` (no `ascribe.toml`: the extension stays off), `stub` (a
stub server in `test/stub-server`), `quill` (the real `ascribe lsp` on a
copy of `examples/quill` with a broken page added), `preview` (the preview
panel against the real server on a copy of `examples/quill`), `actions` (the
editor's actions, run through their commands, on a copy of `examples/quill`),
and `monorepo` (several projects, one nested in another, in
`test/fixtures/monorepo`); `test/integration/run.ts` lists them all. Those
against the real server run only when `ASCRIBE_BIN` names a built `ascribe`. `ASCRIBE_SUITE` runs one suite.

The webview tests (`test/webview/`) load the preview's shell and bundles into
Chromium under the real policy (`/opt/pw-browsers/chromium`, or
`ASCRIBE_CHROMIUM`) and post the messages the extension sends.

The parity test (`test/parity/`) builds `examples/astro-site` with its own
`build` script, in place and in two copies under its `.e2e-tmp/` (one with
every arm of every group, one filtered to the cloud), starts `ascribe lsp` on
each, and compares each page's preview HTML with Astro's built `<article>`:
elements, attributes, heading ids, image attributes, and asset URLs by source
file. It needs `cargo build -p ascribe-cli` (or `ASCRIBE_BIN`), and the
`@ascribed/elements` and `@ascribed/astro` builds (the Astro site uses them). What it leaves out, and why,
is in `test/parity/normalize.ts`.

`test/fixtures/markdown.tmLanguage.json` is VS Code's markdown grammar (MIT,
microsoft/vscode), so the grammar tests see the scopes it really produces.
