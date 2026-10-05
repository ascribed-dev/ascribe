# Changelog

Every Ascribe release: the `ascribe` binary, the npm packages (`@ascribed/cli`, `@ascribed/astro`, `@ascribed/elements`, `@ascribed/review`), and the VS Code extension share one version. Versions follow [semantic versioning](https://semver.org/); while the major version is 0, a minor version may change behavior.

## Unreleased

This release adds **review**: reading a pull request as readers will see it, page by page, with the changed blocks marked and the pull request's review comments beside them. It works in VS Code's page preview, in your site's own pages under `astro dev`, and as an HTML report CI can attach to every pull request. Comments are ordinary GitHub review comments, so nothing else needs to be set up or hosted. [Review](docs/content/guides/review.md) walks through it.

### The language

- **`@snippet`** takes a code example from a file, and puts it in the page as a fenced code block: `@snippet: code:examples/quill/ascribe.toml#dimensions`. The file is read whenever Ascribe checks, builds, or diffs, so the example changes when the code does, with no copy to update. A region is marked in the code with Bluehawk's tags (`:snippet-start:`, `:snippet-end:`, `:remove:`), so files already tagged for Bluehawk work as they are. `ascribe check` reports an address, file, or region that doesn't exist and tags that don't balance, at the `@snippet` line (`snippet-address` and five more). In the JSON output, the code block names its address and lines; `ascribe diff` lists a page whose snippet's code changed, through its address. See [`@snippet`](docs/content/reference/directives.md#snippet) and [Drift](docs/content/guides/drift.md).
- **`[sources.<name>]`** in `ascribe.toml` names a folder of code outside the content that snippets may read, with `include` and `ignore` patterns. It's the only way a page can read outside its project's folder, and the folder must be in the project's git repository. See [`[sources.<name>]`](docs/content/reference/content-model.md#18-sourcesname).

### The `ascribe` command

- `ascribe diff` shows what changed between a git revision and the working tree, as readers will see it: the changed pages of each build, and the blocks on them that were added, removed, changed, or moved, with the words that changed. It compares resolved pages, so a page that changed only through a fragment, a phrase, or a build's settings is listed with the cause, and reformatting is no change. By default it compares with the merge base of the default branch, as a pull request does. `--format json` writes the changes, with each block's source lines, for tools. It needs `git`, and nothing else. Pages with errors are compared as they are, but `ascribe diff` says how many errors the working tree has (and `--format json` counts them), and so do the HTML report and review in both previews, so a broken render isn't read as the change. See [`ascribe diff`](docs/content/reference/cli.md#ascribe-diff).
- `ascribe diff --format html` writes the changes as one self-contained HTML file: every changed page rendered with its changes marked, a removed block shown where it was, and the page as it will be and as it was a click away. It makes no network requests and runs nothing from the pages, so CI can upload it for reviewers to open from the pull request. See [the HTML report](docs/content/reference/cli.md#the-html-report) and [the report in CI](docs/content/guides/review.md#the-report-in-ci).
- `ascribe drift` lists the pages whose code examples changed between a git revision and the working tree: first those whose own text didn't change, so the words around the new code may be out of date, then those that changed along with their examples, and the pages whose examples no longer resolve (a renamed region or a moved file). A page covers the regions and files it takes snippets from, its fragments' included; a region is compared as text, so an edit elsewhere in its file or a move isn't a change, and a renamed file is followed. It reads only the changed files a snippet uses at the base. `--format summary` writes Markdown for a CI job's summary, and nothing when no example changed; `--format json` is for tools; `--exit-code` exits 1 when an example broke, or changed while its page didn't. See [`ascribe drift`](docs/content/reference/cli.md#ascribe-drift) and [Drift](docs/content/guides/drift.md#when-an-example-changes).
- `ascribe build --emit site --anchors` marks each block of the site output with the source file and lines it came from (`data-ascribe-source`, and `data-ascribe-via` for a block from a fragment). The site output's manifest records `"anchors": true`. Without the flag, the output is unchanged. See the [site-render contract](docs/content/contracts/site-render.md#7-source-anchors).
- Each command's `--help` describes its options in the words of the [command reference](docs/content/reference/cli.md), whose option lists are generated from it. `-h` shows the first paragraph of each.
- Each command's `--help` ends with a link to its section of the command reference on the docs site, <https://ascribed-dev.com>.

### The editor

- **Review in the preview.** **Ascribe: Start Review** compares the active page's project with a base: the base of the branch's pull request, the default branch, or a revision you type, from where your branch left it. The preview marks what changed on the page, following your edits as you type. Its header shows the base, the page's changes with a breakdown, **Changes / As it will be / As it was**, and next and previous change, and offers the next changed page after the last change. **Ascribe: Changed Pages** lists the pages the change touches, and a status bar item shows whether review is on and starts it. While review is off, the preview looks as it did. See [Review in the preview](docs/content/guides/editor.md#review-in-the-preview).
- **Comments in the preview.** When the branch has an open pull request, the preview shows its review threads beside the blocks they're on, and you can reply (at once, or with your review), resolve, comment on any block, and submit or discard your review there. New comments are unsent, visible only to you, until you submit. The header says when your checkout is behind or ahead of the pull request, and **Ascribe: Refresh Comments** reads the threads again. The same threads show on their source lines, unless the GitHub Pull Requests extension shows them already (the new setting `ascribe.review.sourceComments`). Comments use VS Code's GitHub sign-in, asked for only when you start review, or the GitHub CLI's; without either, review shows the changes only. See [Comments in the preview](docs/content/guides/editor.md#comments-in-the-preview).
- **The site preview.** **Ascribe: Open Site Preview** opens the active page on its project's dev server, in the browser, at the heading the editor shows, and the preview panel's **Page | Site** switch shows it in the panel, following the active file. See [Site preview](docs/content/guides/editor.md#site-preview).
- **Ascribe: Open Preview to the Side** is now **Ascribe: Open Page Preview to the Side** (the command id is unchanged), and the new **Ascribe: Open Page Preview** opens the preview in place of the editor.
- The preview scrolls with the editor by block instead of by heading, both ways: scrolling the editor scrolls the preview to the block at its top, moving the cursor shows its block, scrolling the preview scrolls the editor, and double-clicking a block puts the cursor on its source line. A block from a fragment follows its `@include` line. `ascribe.preview.scrollPreviewWithEditor` and `ascribe.preview.scrollEditorWithPreview` turn each direction off.
- The preview takes scripts, event handlers, and `javascript:` URLs out of a page's HTML before showing it, as the review report does.
- The Settings editor describes each setting in the words of the [editor guide](docs/content/guides/editor.md#settings), whose settings table is generated from it, and `ascribe.trace.server` describes its choices.

### Astro

- **Review in the site preview.** In `astro dev`, an **Ascribe review** app in Astro's dev toolbar marks a change's blocks on the real page, in your site's layout, and shows the pull request's review threads beside them, with commenting, replying, resolving, and submitting through the GitHub CLI, which the dev server runs. Comments stay off when the dev server listens on the network, or when the Vite config lets other pages reach it. A route that isn't an Ascribe page lists the changed pages, and a page whose layout drops the source anchors lists its changes and threads. The `review` option (on by default) turns the app off; `astro build` output has no anchors, overlay, or review code. See [Review in the site preview](docs/content/guides/astro.md#review-in-the-site-preview).
- `astro dev` writes its address to `.ascribe/dev.json` in the project, for the editor's site preview, and removes it when it stops.
- The `anchors` option turns source anchors on: `"dev"` in `astro dev` only, `true` always. The Markdown plugins apply them.

### The language server

- It answers `ascribe/review/setBase` and `ascribe/review/changes`, and `ascribe/preview` with `review: true` adds the page's changes, for editors other than VS Code. See [`crates/tessera-lsp/README.md`](crates/tessera-lsp/README.md).

### Installing

- The Linux binaries run on glibc 2.28 or later, instead of 2.39, so `@ascribed/cli` installs and runs in the build images of Vercel, AWS Amplify, and Cloudflare Pages as well as Netlify's. The release workflow runs each one on glibc 2.28, so the floor can't rise unnoticed.
- A canary of the npm packages, built from `main`, is published every night under the `next` tag: `npm install @ascribed/cli@next @ascribed/astro@next`. It comes with no promise of stability, and its `ascribe --version` names the commit it was built from. Releases still publish under `latest`.

### `@ascribed/review`

- A new package, for hosts that show review. Its Node part finds the open pull request for a checkout's branch, reads its review threads, places each on the rendered block it's on (following local edits, fragments shown on several pages, removed text, and outdated threads, and returning those it can't place as detached), and posts comments and replies into the reviewer's pending review. It talks to GitHub through the GitHub CLI or a token its host supplies, and stores no token. Its browser part marks the changed blocks of a rendered page, and draws the overlay of threads beside them, with replying, resolving, commenting, and submitting through a host that keeps GitHub to itself; comment bodies render from a safe subset of Markdown, with no raw HTML, and no image loads without a click. See [`packages/review`](packages/review/README.md).

## 0.1.1 (2026-10-02)

### The language

- **Behavior change:** a directory under the content root that holds an `ascribe.toml`, other than the project's own folder, is another project's folder, and is skipped like a directory whose name starts with `.`. `ascribe check`, `ascribe build`, `ascribe fmt`, and the language server leave its files to that project, so an editor shows each file's diagnostics from one project only. A link or an include from the outer project to a Markdown file in it is reported as a missing file.

### The editor

- A workspace can hold several Ascribe projects. Every `ascribe.toml` is a project with its own language server, and a file belongs to the nearest `ascribe.toml` above it, so projects can be nested. Diagnostics, completion, the preview, format on save, and the other features use the project that owns the file. See [Workspaces with several projects](docs/content/guides/editor.md#workspaces-with-several-projects).
- A project's server starts the first time one of its files is opened or previewed. The new setting `ascribe.startServers` (`onDemand` or `all`) can start every project's server when the workspace opens instead. The Problems panel lists only the projects whose server is running; `ascribe check` covers any project.
- The extension serves at most 50 projects in a workspace, the first 50 by path, and says so in the first project's output when there are more.
- Each project uses its own `node_modules/.bin/ascribe`, and logs to its own output channel, `Ascribe (<project>)`.
- The extension warns about a project's `ascribe` older than 0.1.1: an older server checks a nested project's files as its own.
- **Ascribe: Restart Language Server** restarts every server that has started. **Ascribe: Show Server Output** shows the active file's project, or asks which project. The preview's build choice is kept for each project.

### The language server

- **Behavior change for editors other than VS Code:** `ascribe lsp` looks for `ascribe.toml` only in its workspace folder and the folders above it. A workspace folder whose projects are all in subfolders gets no project; the server doesn't pick one of them. Start one `ascribe lsp` per project, rooted at the folder that holds its `ascribe.toml` ([Other editors](docs/content/guides/editor.md#other-editors)).
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
- 126 diagnostics, each with a code (`ASC001` to `ASC126`) and a stable name. See [docs/content/reference/diagnostics.md](docs/content/reference/diagnostics.md).
- Binaries for macOS on Apple silicon, Linux (arm64 and x64, glibc 2.39 or later), and Windows (x64). Intel Macs aren't supported. Each binary's archive and package carries a `THIRD-PARTY-NOTICES` file with the license text of the crates in it.

### The editor

- The VS Code extension: diagnostics as you type, completion, hover, go to definition, document links, CodeLens, inline hints for empty links, rename and move refactoring, quick fixes, formatting, semantic highlighting, and a live preview that renders pages as the site does.
- It uses the project's own `ascribe` when the project installs one, and the binary bundled with it otherwise.

### Astro

- `@ascribed/astro`: builds the site output before Astro loads content, provides the content collection and its generated schema, applies heading ids and image attributes in Astro's markdown pipeline, loads the element library, serves linked files, and rebuilds in `astro dev`.
- `@ascribed/elements`: the web components the site output uses (`<ascribe-note>`, `<ascribe-tabs>`, `<ascribe-availability>`, and the rest), styled with CSS custom properties.
