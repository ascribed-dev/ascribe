# Changelog

Every Ascribe release: the `ascribe` binary, the npm packages (`@ascribed/cli`, `@ascribed/astro`, `@ascribed/elements`), and the VS Code extension share one version. Versions follow [semantic versioning](https://semver.org/); while the major version is 0, a minor version may change behavior.

## 0.1.1 (unreleased)

### The language

- **Behavior change:** a directory under the content root that holds an `ascribe.toml`, other than the project's own folder, is another project's folder, and is skipped like a directory whose name starts with `.`. `ascribe check`, `ascribe build`, `ascribe fmt`, and the language server leave its files to that project, so an editor shows each file's diagnostics from one project only. A link or an include from the outer project to a Markdown file in it is reported as a missing file.

### The editor

- A workspace can hold several Ascribe projects. Every `ascribe.toml` is a project with its own language server, and a file belongs to the nearest `ascribe.toml` above it, so projects can be nested. Diagnostics, completion, the preview, format on save, and the other features use the project that owns the file. See [Workspaces with several projects](docs/editor.md#workspaces-with-several-projects).
- A project's server starts the first time one of its files is opened or previewed. The new setting `ascribe.startServers` (`onDemand` or `all`) can start every project's server when the workspace opens instead. The Problems panel lists only the projects whose server is running; `ascribe check` covers any project.
- Each project uses its own `node_modules/.bin/ascribe`, and logs to its own output channel, `Ascribe (<project>)`.
- **Ascribe: Restart Language Server** restarts every server that has started. **Ascribe: Show Server Output** shows the active file's project, or asks which project. The preview's build choice is kept for each project.

### The language server

- **Behavior change for editors other than VS Code:** `ascribe lsp` looks for `ascribe.toml` only in its workspace folder and the folders above it. A workspace folder whose projects are all in subfolders gets no project; the server doesn't pick one of them. Start one `ascribe lsp` per project, rooted at the folder that holds its `ascribe.toml` ([Other editors](docs/editor.md#other-editors)).
- It logs the project it uses, or why it has none.

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
