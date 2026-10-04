# Changelog

Every Ascribe release: the `ascribe` binary, the npm packages (`@ascribed/cli`, `@ascribed/astro`, `@ascribed/elements`, `@ascribed/review`), and the VS Code extension share one version. Versions follow [semantic versioning](https://semver.org/); while the major version is 0, a minor version may change behavior.

## Unreleased

### The command

- `ascribe diff` shows what changed between a git revision and the working tree, as readers will see it: the changed pages of each build, and the blocks on them that were added, removed, changed, or moved, with the words that changed. It compares resolved pages, so a page that changed only through a fragment, a phrase, or a build's settings is listed with the cause, and reformatting is no change. By default it compares with the merge base of the default branch, as a pull request does. `--format json` writes the changes with each block's source lines for tools. See [`ascribe diff`](docs/cli.md#ascribe-diff).
- `ascribe diff --format html` writes the changes as one self-contained HTML file: every changed page rendered with its changes marked, a removed block shown where it was, and the page as it will be and as it was a click away. It makes no network requests, so CI can upload it for reviewers to open from the pull request. See [the HTML report](docs/cli.md#the-html-report) and [Report in CI](docs/review.md#report-in-ci).

### The compiler

- `ascribe build --emit site --anchors` marks each block of the site output with the source file and lines it came from (`data-ascribe-source`, and `data-ascribe-via` for a block from a fragment), for review. The site output's manifest records `"anchors": true`. Without the flag, the output is unchanged. See the [site-render contract](docs/contracts/site-render.md#7-source-anchors).

### Astro

- The integration's `anchors` option turns source anchors on: `"dev"` in `astro dev` only, `true` always. The Markdown plugins apply them.

### Review

- A new package, `@ascribed/review`. Its Node part finds the open pull request for a checkout's branch, reads its review threads, places each on the rendered block it's on (following local edits, fragments shown on several pages, removed text, and outdated threads, and returning threads it can't place as detached), and posts comments and replies into the reviewer's pending review, which nobody else sees until it's submitted. It talks to GitHub through the GitHub CLI or a token its host supplies, and stores no token. See [`packages/review`](packages/review/README.md).
- `@ascribed/review/overlay` draws a pull request's threads beside a rendered page's blocks, in the browser, with replying, resolving, commenting, and submitting through a host that keeps GitHub to itself. Comment bodies render from a safe subset of Markdown, built as DOM nodes: no raw HTML, and no image loads without a click.

### The editor

- Review in the preview: **Ascribe: Start Review** compares the active page's project with a git revision (the default branch, or one you type, from where your branch left it), and the preview marks what changed on the page, as `ascribe diff` reports it, following your edits as you type. Its header shows the base, the page's change count and breakdown, **Changes / As it will be / As it was**, and next and previous change, and offers the next changed page after the last change. **Ascribe: Changed Pages** lists the pages the change touches. A status bar item shows whether review is on. See [Review in the preview](docs/editor.md#review-in-the-preview).
- Comments in the preview: when the branch has an open pull request, review shows its review threads beside the blocks they're on, and you can reply (at once, or with your review), resolve, comment on any block, and submit or discard your review there. **Start Review** offers the pull request's base first, the header says when your checkout is behind or ahead of the pull request, and **Ascribe: Refresh Comments** reads the threads again. The same threads show on their source lines, unless the GitHub Pull Requests extension shows them already (`ascribe.review.sourceComments`). It uses VS Code's GitHub sign-in, asked for only when you start review, or the GitHub CLI's; without either, review shows the changes only. See [Comments in the preview](docs/editor.md#comments-in-the-preview).
- The preview takes scripts, event handlers, and `javascript:` URLs out of a page's HTML before showing it, as the review report does.
- The language server answers `ascribe/review/setBase` and `ascribe/review/changes`, and `ascribe/preview` with `review: true` adds the page's changes. See [`crates/tessera-lsp/README.md`](crates/tessera-lsp/README.md).
- The preview scrolls with the editor by block instead of by heading, both ways: scrolling the editor scrolls the preview to the block at its top, moving the cursor shows its block, scrolling the preview scrolls the editor, and double-clicking a block puts the cursor on its source line. A block from a fragment follows its `@include` line. `ascribe.preview.scrollPreviewWithEditor` and `ascribe.preview.scrollEditorWithPreview` turn each direction off.

## 0.1.1 (2026-10-02)

### The language

- **Behavior change:** a directory under the content root that holds an `ascribe.toml`, other than the project's own folder, is another project's folder, and is skipped like a directory whose name starts with `.`. `ascribe check`, `ascribe build`, `ascribe fmt`, and the language server leave its files to that project, so an editor shows each file's diagnostics from one project only. A link or an include from the outer project to a Markdown file in it is reported as a missing file.

### The editor

- A workspace can hold several Ascribe projects. Every `ascribe.toml` is a project with its own language server, and a file belongs to the nearest `ascribe.toml` above it, so projects can be nested. Diagnostics, completion, the preview, format on save, and the other features use the project that owns the file. See [Workspaces with several projects](docs/editor.md#workspaces-with-several-projects).
- A project's server starts the first time one of its files is opened or previewed. The new setting `ascribe.startServers` (`onDemand` or `all`) can start every project's server when the workspace opens instead. The Problems panel lists only the projects whose server is running; `ascribe check` covers any project.
- The extension serves at most 50 projects in a workspace, the first 50 by path, and says so in the first project's output when there are more.
- Each project uses its own `node_modules/.bin/ascribe`, and logs to its own output channel, `Ascribe (<project>)`.
- The extension warns about a project's `ascribe` older than 0.1.1: an older server checks a nested project's files as its own.
- **Ascribe: Restart Language Server** restarts every server that has started. **Ascribe: Show Server Output** shows the active file's project, or asks which project. The preview's build choice is kept for each project.

### The language server

- **Behavior change for editors other than VS Code:** `ascribe lsp` looks for `ascribe.toml` only in its workspace folder and the folders above it. A workspace folder whose projects are all in subfolders gets no project; the server doesn't pick one of them. Start one `ascribe lsp` per project, rooted at the folder that holds its `ascribe.toml` ([Other editors](docs/editor.md#other-editors)).
- It logs the project it uses, or why it has none.
- `ascribe lsp --config` is an error (exit code `2`). The server takes its project from the editor's workspace folders, and ignored the option.

## 0.1.0 (2026-10-01)

The first release. It implements version 0.1 of the [Ascribe specification](SPEC.md).

### The language

- Directives: `@id`, `@include`, `@variant`, `@available`, `@note`, `@steps`, and `@details`, with attributes, titles, line and container forms, groups, and binding to headings and blocks.
- Project widgets: directives a project declares in `ascribe.toml`.
- Phrases (`{key}`), links and includes by file path, image attributes, heading ids, and a glossary.
- The content model, `ascribe.toml`: content types and frontmatter schemas, fragments, dimensions, versions, lifecycle states, features, note types, phrases, glossary, image attributes, widgets, the consumer profile, builds, and the editor's build.

### The `ascribe` command

- `ascribe check`: every diagnostic, for every build, as text or JSON.
- `ascribe build`: the site output (markdown with web components, and a generated Zod schema), the plain-markdown output, and the JSON output, for each build.
- `ascribe fmt`: rewrites Ascribe constructs into canonical form; `--check` for CI.
- `ascribe lsp`: the language server.
- 126 diagnostics, each with a code (`ASC001` to `ASC126`) and a stable name. See [docs/diagnostics.md](docs/diagnostics.md).
- Binaries for macOS on Apple silicon, Linux (arm64 and x64, glibc 2.39 or later), and Windows (x64). Intel Macs aren't supported. Each binary's archive and package carries a `THIRD-PARTY-NOTICES` file with the license text of the crates in it.

### The editor

- The VS Code extension: diagnostics as you type, completion, hover, go to definition, document links, CodeLens, inline hints for empty links, rename and move refactoring, quick fixes, formatting, semantic highlighting, and a live preview that renders pages as the site does.
- It uses the project's own `ascribe` when the project installs one, and the binary bundled with it otherwise.

### Astro

- `@ascribed/astro`: builds the site output before Astro loads content, provides the content collection and its generated schema, applies heading ids and image attributes in Astro's markdown pipeline, loads the element library, serves linked files, and rebuilds in `astro dev`.
- `@ascribed/elements`: the web components the site output uses (`<ascribe-note>`, `<ascribe-tabs>`, `<ascribe-availability>`, and the rest), styled with CSS custom properties.
