# Changelog

Every Ascribe release: the `ascribe` binary, the npm packages (`@ascribed/cli`, `@ascribed/astro`, `@ascribed/elements`, `@ascribed/review`), and the VS Code extension share one version. Versions follow [semantic versioning](https://semver.org/); while the major version is 0, a minor version may change behavior.

## Unreleased

- **A faster `ascribe diff` and `ascribe drift`.** `diff` compares only the pages a change can reach, and reads the two versions side by side: on 3,000 pages it takes a third less time than it did, and a third less memory. `drift` takes a third less time when an example changed. The reports are the same.
- **JSON Schemas for the commands' JSON.** What `ascribe check`, `build`, `diff`, `drift`, `sources status`, and `sources update` write with `--format json` is described by a JSON Schema, generated from the code that writes it. See the [JSON report contract](docs/content/contracts/json-reports.md).
- **The `json` output's page `format` is `"ascribe-page"`,** where it was `"tessera-page"`, the project's working name. Its `schemaVersion` stays `1`. A tool that checks the value needs the new one. See [the JSON output](crates/ascribe-emit/README.md#json).
- **Smaller downloads.** The `ascribe` binary is built with link-time optimization: about a third smaller on macOS and Linux (7.0 MB on macOS arm64, from 10.5 MB) and 8 percent smaller on Windows, and a little faster.

## 0.2.0 (2026-10-06)

This release adds **review**: reading a pull request as readers will see it, page by page, with the changed blocks marked and the pull request's review comments beside them. It works in VS Code's page preview, in your site's own pages under `astro dev`, and as an HTML report CI can attach to every pull request. Comments are ordinary GitHub review comments, so there's nothing to set up or host. See [Review](docs/content/guides/review.md).

### The language

- **`@snippet`** puts a code example from a file in the page as a fenced code block: `@snippet: code:examples/quill/ascribe.toml#dimensions`. The file is read on every check, build, and diff, so the example changes when the code does. A region is marked in the code with Bluehawk's tags (`:snippet-start:`, `:snippet-end:`, `:remove:`). `ascribe check` reports a missing address, file, or region, and tags that don't balance (`snippet-address` and five more). See [`@snippet`](docs/content/reference/directives.md#snippet) and [Drift](docs/content/guides/drift.md).
- **`[sources.<name>]`** in `ascribe.toml` names a folder of code outside the content that snippets may read, with `include` and `ignore` patterns. It's the only way a page reads outside its project's folder, and the folder must be in the project's git repository. See [`[sources.<name>]`](docs/content/reference/content-model.md#18-sourcesname).
- **A source in another repository:** `git` (and `branch`) in place of `path`. The files snippets use are copied into `sources/<name>/` and pinned to a commit in `ascribe.lock`. Both are committed with the docs, so checking, building, diffing, and drift never reach the other repository. `ascribe check` reports copies that don't match the lock or the snippets (`lock-invalid`, `source-copy-changed`, and three more). See [a source in another repository](docs/content/reference/content-model.md#a-source-in-another-repository) and [Docs kept apart from the code](docs/content/guides/drift.md#docs-kept-apart-from-the-code).
- **A table row can have an availability:** an attribute block at the end of its first cell, `` | `stream` {available="self-managed preview 3.4"} | … | ``. A badge build marks the row, and a filter build removes it where it isn't available. The spec is checked as an `@available` line's is, within the table's own availability. **Behavior change:** a `{key=value}` that ends a body row's first cell was text and is now an attribute block, so a key other than `available` is an error (`attribute-unknown-key`). See [table rows](docs/content/reference/directives.md#table-rows).
- **A page title can show code.** `inline = "code"` on a content type's `string` field reads code spans in it: `title = { type = "string", inline = "code" }` lets a page be titled "`` `ascribe.toml` reference ``". The site output writes the field as plain text, for `<title>` and search, and as HTML under the new key `formatted` (`entry.data.formatted.title` in an Astro layout). The JSON output has `formatted` too. **Behavior changes:** a type can no longer declare a field named `formatted` (`model-field-reserved`), and in a field that turns `inline` on, a backslash before punctuation is an escape and is removed, as in Markdown. See [code in a field](docs/content/reference/content-model.md#53-code-in-a-field).
- **A link can name a heading from a fragment the page includes:** `setup.md#prerequisites` works when `prerequisites` comes from a fragment `setup.md` includes. It was an error (`link-id-in-fragment`, now retired). See [`@include`](docs/content/reference/directives.md#include).

### The `ascribe` command

- `ascribe diff` shows what changed between a git revision and the working tree, as readers will see it: each build's changed pages, and the blocks added, removed, changed, or moved on them, with the words that changed. It compares resolved pages, so a page that changed only through a fragment, a phrase, or a build's settings is listed with the cause, and reformatting is no change. The default base is the merge base with the default branch, as in a pull request. `--format json` adds each block's source lines, for tools. It needs only `git`. It also says how many errors the working tree has, as do the HTML report and both previews, so a broken render isn't read as the change. See [`ascribe diff`](docs/content/reference/cli.md#ascribe-diff).
- `ascribe diff --format html` writes the changes as one self-contained HTML file: every changed page rendered with its changes marked, and the page as it will be and as it was a click away. It makes no network requests and runs nothing from the pages, so CI can attach it to a pull request. See [the HTML report](docs/content/reference/cli.md#the-html-report) and [the report in CI](docs/content/guides/review.md#the-report-in-ci).
- `ascribe drift` lists the pages whose code examples changed between a git revision and the working tree: first those whose own text didn't change, so the words around the code may be out of date, then those that changed with their examples, then those whose examples no longer resolve. A region is compared as text, so an edit elsewhere in its file isn't a change. `--format summary` writes Markdown for a CI job's summary, `--format json` is for tools, and `--exit-code` exits 1 when an example broke, or changed while its page didn't. See [`ascribe drift`](docs/content/reference/cli.md#ascribe-drift) and [Drift](docs/content/guides/drift.md#when-an-example-changes).
- `ascribe build --emit site --anchors` marks each block of the site output with its source file and lines (`data-ascribe-source`, and `data-ascribe-via` for a block from a fragment). Without the flag, the output is unchanged. See the [site-render contract](docs/content/contracts/site-render.md#7-source-anchors).
- `ascribe sources fetch` makes the copies of sources in other repositories match `ascribe.lock`. `ascribe sources update` moves a source's pin to the head of its branch (or `--to`), copies again, and reports the commits, the copies, and the pages whose examples changed, as text, JSON, or Markdown for a pull request. `ascribe sources status` shows each pin and copy, offline. `fetch` and `update` run `git` with its own credentials, and are the only commands that reach the network. See [`ascribe sources`](docs/content/reference/cli.md#ascribe-sources).
- Each command's `--help` describes its options in the words of the [command reference](docs/content/reference/cli.md), whose option lists are generated from it, and ends with a link to its section on the docs site, <https://ascribed-dev.com>. `-h` shows the first paragraph of each.
- **Fixed:** the plain-markdown output (`--emit plain`) writes an inline `<br>` as a line break, or as `; ` in a table cell or heading. It was dropped, so code spans on either side ran together.

### The editor

- **Review in the preview.** **Ascribe: Start Review** compares the active page's project with a base: the base of the branch's pull request, the default branch, or a revision you type. The preview marks what changed on the page and follows your edits. Its header shows the base and the page's changes, switches between **Changes / As it will be / As it was**, and steps through the changes and the changed pages. **Ascribe: Changed Pages** lists the pages the change touches, and a status bar item shows whether review is on and starts it. See [Review in the preview](docs/content/guides/editor.md#review-in-the-preview).
- **Comments in the preview.** When the branch has an open pull request, the preview shows its review threads beside their blocks. You can reply, resolve, comment on any block, and submit or discard your review. New comments are visible only to you until you submit. **Ascribe: Refresh Comments** reads the threads again. Threads also show on their source lines, unless the GitHub Pull Requests extension shows them already (the new setting `ascribe.review.sourceComments`). Comments use VS Code's GitHub sign-in or the GitHub CLI's. Without either, review shows the changes only. See [Comments in the preview](docs/content/guides/editor.md#comments-in-the-preview).
- **The site preview.** **Ascribe: Open Site Preview** opens the active page on its project's dev server, in the browser, at the heading the editor shows. The preview panel's **Page | Site** switch shows it in the panel, following the active file. See [Site preview](docs/content/guides/editor.md#site-preview).
- **Ascribe: Open Preview to the Side** is now **Ascribe: Open Page Preview to the Side** (the command id is unchanged). The new **Ascribe: Open Page Preview** opens the preview in place of the editor.
- The preview and the editor scroll together by block instead of by heading, in both directions, and double-clicking a block puts the cursor on its source line. `ascribe.preview.scrollPreviewWithEditor` and `ascribe.preview.scrollEditorWithPreview` turn each direction off.
- The preview takes scripts, event handlers, and `javascript:` URLs out of a page's HTML before showing it, as the review report does.
- The Settings editor describes each setting in the words of the [editor guide](docs/content/guides/editor.md#settings), whose settings table is generated from it.

### Astro

- **Review in the site preview.** In `astro dev`, an **Ascribe review** app in Astro's dev toolbar marks a change's blocks on the real page, in your site's layout, and shows the pull request's review threads beside them. Commenting goes through the GitHub CLI, which the dev server runs. It stays off when the dev server listens on the network, or when the Vite config lets other pages reach it. The `review` option (on by default) turns the app off. `astro build` output has no anchors, overlay, or review code. See [Review in the site preview](docs/content/guides/astro.md#review-in-the-site-preview).
- `astro dev` writes its address to `.ascribe/dev.json` in the project, for the editor's site preview, and removes it when it stops.
- The `anchors` option turns source anchors on: `"dev"` in `astro dev` only, `true` always.
- A code block's title, such as a `@snippet`'s, shows above it, in a `<figure class="code-title">` with a `<figcaption>`. The `codeTitles` option turns it off. See [code block titles](docs/content/guides/astro.md#code-block-titles).
- `@ascribed/astro/Availability.astro` renders a page's `available` frontmatter as the element library's badge: `<Availability available={entry.data.available} />`.
- The integration copies the generated schema into `.astro/integrations/_ascribed_astro/schema.ts` after each build. A site whose Ascribe project is outside the Astro root imports that copy, so type-checking finds `astro/zod`. See [define the collection](docs/content/guides/astro.md#4-define-the-collection).
- A clean build's check summary (`checked 14 files: 0 errors, 0 warnings`) is logged as info, not as a warning.
- The npm packages' source maps include their sources, so Vite no longer warns that they point to missing files.

### The language server

- It answers `ascribe/review/setBase` and `ascribe/review/changes`, and `ascribe/preview` with `review: true` adds the page's changes, for editors other than VS Code. See [`crates/ascribe-lsp/README.md`](crates/ascribe-lsp/README.md).

### Installing

- The Linux binaries run on glibc 2.28 or later, instead of 2.39, so `@ascribed/cli` installs and runs in the build images of Vercel, AWS Amplify, and Cloudflare Pages as well as Netlify's.
- A canary of the npm packages, built from `main`, is published every night under the `next` tag: `npm install @ascribed/cli@next @ascribed/astro@next`. It comes with no promise of stability, and its `ascribe --version` names the commit it was built from. Releases still publish under `latest`.

### `@ascribed/review`

- A new package, for hosts that show review. Its Node part finds the open pull request for a checkout's branch, reads its review threads, places each on its rendered block, and posts comments and replies into the reviewer's pending review. It talks to GitHub through the GitHub CLI or a token its host supplies, and stores no token. Its browser part marks a rendered page's changed blocks and draws the threads beside them. Comment bodies render from a safe subset of Markdown, with no raw HTML, and no image loads without a click. See [`packages/review`](packages/review/README.md).

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
