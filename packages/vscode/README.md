<p><img src="https://raw.githubusercontent.com/ascribed-dev/ascribe/main/design/out/header-vscode.png" alt="Ascribe" width="266" height="96"></p>

# Ascribe for VS Code

Write [Ascribe](https://github.com/ascribed-dev/ascribe) documentation with the checks, completion, navigation, and preview a programming language gets. Ascribe is Markdown with directives for the structure documentation needs: callouts, procedures, alternatives by platform or product, availability, and reusable content.

The extension activates in a workspace that contains an `ascribe.toml`. A workspace can hold several projects, each with its own language server: see [workspaces with several projects](https://ascribed-dev.com/guides/editor/#workspaces-with-several-projects).

## Features

- **Diagnostics as you type**, across the whole project, the same ones `ascribe check` reports in CI. Many come with a quick fix.
- **Completion** for directives, attributes and their values, phrases, availability, include paths, and links, which you search by page and heading title rather than by path.
- **Hover** for links and includes (the target's title and first paragraph), phrases (their values), and availability (what it means on the published site).
- **Navigation**: go to definition, clickable links, a CodeLens on each include, and hints that show the title an empty-text link will get.
- **Refactoring**: renaming or moving a file updates the links and includes that point to it; renaming a heading's `@id` or a phrase updates its uses.
- **Formatting** into canonical form, on request or on save.
- **Highlighting** of directives, phrases, and title lines.
- **A live page preview** that renders the page as the published site does, as you type, for any build, and a **site preview** that opens it on the site's dev server.
- **Review**: the preview marks what changed against a git revision, lists the pages a change touches, and shows the pull request's review comments beside the blocks they're about, where you can reply, resolve, comment, and submit your review. [Reviewing a pull request](https://ascribed-dev.com/guides/review/) walks through it.

## The `ascribe` binary

The extension includes `ascribe` for your platform, so it works immediately. When a project installs its own (`npm install --save-dev @ascribed/cli`), the extension uses that one for it instead, so the editor matches the version the project pins for CI. The `ascribe.path` setting overrides both.

## Settings

| Setting | What it does |
|---|---|
| `ascribe.path` | The `ascribe` binary to run, for every project. When empty, each project's own, then the included one. |
| `ascribe.startServers` | When each project's language server starts: `onDemand` (default), the first time one of its files is opened, or `all`, when the workspace opens. |
| `ascribe.formatOnSave` | Format Ascribe constructs when saving. |
| `ascribe.preview.scrollPreviewWithEditor` | Scroll the preview with the editor. Default on. |
| `ascribe.preview.scrollEditorWithPreview` | Scroll the editor with the preview. Default on. |
| `ascribe.review.sourceComments` | Show the pull request's review threads in the source editor: `on`, `off`, or `auto` (default), which leaves them to the GitHub Pull Requests extension when it's active. |
| `ascribe.maxCrashes` | How many crashes of the language server make the extension stop restarting it. Default 5. |
| `ascribe.trace.server` | Log the conversation with the language server: `off`, `messages`, or `verbose`. |

## Commands

- **Ascribe: Open Page Preview** and **Ascribe: Open Page Preview to the Side**, also the preview button in a Markdown editor's title bar.
- **Ascribe: Open Site Preview**: the page on the site's dev server (`astro dev` with `@ascribed/astro`), in the browser. The page preview's **Page | Site** switch shows it in the preview panel.
- **Ascribe: Select Preview Build**
- **Ascribe: Start Review**, **Ascribe: Stop Review**, **Ascribe: Changed Pages**, and **Ascribe: Refresh Comments**
- **Ascribe: Restart Language Server**
- **Ascribe: Show Server Output**

## Learn more

- [Editing with Ascribe](https://ascribed-dev.com/guides/editor/): everything the extension does.
- [Review](https://ascribed-dev.com/guides/review/): reviewing a pull request in the preview.
- [Getting started](https://ascribed-dev.com/getting-started/) with Ascribe.
- [Diagnostics](https://ascribed-dev.com/reference/diagnostics/): every problem Ascribe reports, and its fix.

Other Markdown formatters that reflow paragraphs don't know that Ascribe's directive and title lines start new blocks; exclude Ascribe files from them.
