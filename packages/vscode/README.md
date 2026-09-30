# Ascribe for VS Code

Write [Ascribe](https://github.com/KyleBlankRollins/tessera) documentation with the checks, completion, navigation, and preview a programming language gets. Ascribe is Markdown with directives for the structure documentation needs: callouts, procedures, alternatives by platform or product, availability, and reusable content.

The extension activates in a workspace that contains an `ascribe.toml`.

## Features

- **Diagnostics as you type**, across the whole project, the same ones `ascribe check` reports in CI. Many come with a quick fix.
- **Completion** for directives, attributes and their values, phrases, availability, include paths, and links, which you search by page and heading title rather than by path.
- **Hover** for links and includes (the target's title and first paragraph), phrases (their values), and availability (what it means on the published site).
- **Navigation**: go to definition, clickable links, a CodeLens on each include, and hints that show the title an empty-text link will get.
- **Refactoring**: renaming or moving a file updates the links and includes that point to it; renaming a heading's `@id` or a phrase updates its uses.
- **Formatting** into canonical form, on request or on save.
- **Highlighting** of directives, phrases, and title lines.
- **A live preview** that renders the page as the published site does, as you type, for any build.

## The `ascribe` binary

The extension includes `ascribe` for your platform, so it works immediately. When the project installs its own (`npm install --save-dev @ascribed/cli`), the extension uses that one instead, so the editor matches the version the project pins for CI. The `ascribe.path` setting overrides both.

## Settings

| Setting | What it does |
|---|---|
| `ascribe.path` | The `ascribe` binary to run. When empty, the project's, then the included one. |
| `ascribe.formatOnSave` | Format Ascribe constructs when saving. |
| `ascribe.maxCrashes` | How many crashes of the language server make the extension stop restarting it. Default 5. |
| `ascribe.trace.server` | Log the conversation with the language server: `off`, `messages`, or `verbose`. |

## Commands

- **Ascribe: Open Preview to the Side**, also the preview button in a Markdown editor's title bar.
- **Ascribe: Select Preview Build**
- **Ascribe: Restart Language Server**
- **Ascribe: Show Server Output**

## Learn more

- [Editing with Ascribe](https://github.com/KyleBlankRollins/tessera/blob/main/docs/editor.md): everything the extension does.
- [Getting started](https://github.com/KyleBlankRollins/tessera/blob/main/docs/getting-started.md) with Ascribe.
- [Diagnostics](https://github.com/KyleBlankRollins/tessera/blob/main/docs/diagnostics.md): every problem Ascribe reports, and its fix.

Other Markdown formatters that reflow paragraphs don't know that Ascribe's directive and title lines start new blocks; exclude Ascribe files from them.
