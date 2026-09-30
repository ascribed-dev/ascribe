# Editing

The Ascribe extension for VS Code turns the editor into an Ascribe authoring environment. Its features come from the language server, `ascribe lsp`, which runs the same checks as `ascribe check`: what the editor reports is what CI reports.

## Install

Install **Ascribe** (`Ascribe.ascribe-vscode`) from the Visual Studio Marketplace: search for "Ascribe" in the Extensions view, or run:

```sh
code --install-extension Ascribe.ascribe-vscode
```

The extension activates in a workspace that contains an `ascribe.toml`. It needs VS Code 1.90 or later, and a trusted workspace, since it runs the project's `ascribe` binary. It's available for macOS on Apple silicon, Linux (x64 and arm64), and Windows (x64); Intel Macs aren't supported.

### Which `ascribe` it runs

The extension includes an `ascribe` binary for your platform, so it works immediately. It looks for a binary in this order:

1. The `ascribe.path` setting, if you set it. A path that doesn't work is an error; the extension doesn't fall back.
2. The project's own `node_modules/.bin/ascribe`, installed with `npm install --save-dev @ascribed/cli`. It looks in each folder that holds an `ascribe.toml`, and its parents up to the workspace folder, so a monorepo works.
3. The binary included in the extension.

A project that installs `@ascribed/cli` gets exactly the version it pins, in the editor as in CI. When that version is older than the extension expects, the extension warns you to update it. **Ascribe: Show Server Output** shows which binary is in use.

## What it does

### Diagnostics

Every [diagnostic](diagnostics.md) appears as you type, including in unsaved files, for the whole project: fix a heading's id, and the broken links to it in other files clear. Page-level diagnostics are for the build named by `[editor] build` in `ascribe.toml`.

### Quick fixes

Many diagnostics offer a fix (the light bulb, or `Ctrl+.` / `Cmd+.`):

- **Link to the page's file instead of its route**, for a link written as a URL.
- **Quote the attribute value**, for a value that needs quotes.
- **Add a trailing colon** or **Remove the stray colon**, for a container written the wrong way.
- **Remove the blank line before the bound block**.
- **Declare phrase in ascribe.toml**, or **Escape this phrase as literal text**, for an undeclared `{key}`.
- **Add a stable @id for this heading**, for a heading whose id could change.
- Did-you-mean corrections for misspelled directives, attributes, and frontmatter keys.
- **Remove** text on a directive line that isn't part of the directive.

### Completion

- After `@` at the start of a line: the built-in directives and the project's widgets, with their descriptions.
- In an attribute block: the directive's attribute keys, then their allowed values, such as note types or a dimension's values with their labels. After an image: the declared image attributes.
- In `@available:` and frontmatter `available:`: targets, dimension names, feature keys, and lifecycle states.
- After `{` in text: the declared phrases, each with its value.
- In `@include:`: source files, then, after `#`, the ids of the file.
- In a link destination: pages and headings, **searched by title**. It inserts the file path and id, so you never need to type a path.

### Hover

- A link, image, or include: its target's path, title, and the first paragraph, with phrases filled in. A fragment, a missing file or id, a link written as a URL, and a name whose case doesn't match are called out.
- A phrase: its value.
- An availability spec or feature key: what it means, in the words the published site uses.
- A directive or attribute key: its description. A dimension key: its values and labels.

### Navigation

- **Go to definition** on a link or include opens the target file at its heading. On an `@id`, it goes to the heading; on a phrase or feature key, to its entry in `ascribe.toml`.
- **Document links**: links, images, and includes are clickable.
- **CodeLens** above each `@include` names the included file and section, and opens it.
- **Inline hints** show the title an empty-text link, `[](keys.md#rotate-keys)`, will get.

### Refactoring

- **Rename or move a file** in the Explorer, and the links and includes that point to it are updated.
- **Rename** (`F2`) a heading's `@id` to update the links to it, or a phrase key to update its uses and its declaration.

### Formatting

**Format Document** rewrites Ascribe constructs into canonical form, as `ascribe fmt` does. Turn on `ascribe.formatOnSave` to do it on every save. It changes only Ascribe constructs, never how a page renders.

### Highlighting

Directive lines, attribute blocks, `@end`, and phrases are highlighted as soon as a file opens. Once the server is running, it adds what depends on the content model: declared and undeclared phrases, project widgets, and **title lines**, shown distinctly so a paragraph that accidentally became a title stands out.

### Preview

**Ascribe: Open Preview to the Side** (also the preview button in a Markdown editor's title bar) shows the current page as the published site shows it, with the same elements, including unsaved changes. It follows the editor: it updates as you type, keeps its scroll position, scrolls to the section the cursor is in, and opens a page or file you click in the editor. Its **Build** picker shows the page as any build publishes it, starting with the editor's build.

A fragment isn't a page, so the preview names the pages that include it instead. A page a build doesn't publish says which build drops it, and why.

The preview shows images and files from the content root, and from directories elsewhere in the project that a page uses. For safety, it runs no inline scripts and loads nothing remote, so raw HTML that needs either looks different in the preview than on the site.

## Settings

| Setting | Default | What it does |
|---|---|---|
| `ascribe.path` | empty | The `ascribe` binary to run. When empty, the project's, then the included one. |
| `ascribe.formatOnSave` | `false` | Format Ascribe constructs when saving. |
| `ascribe.maxCrashes` | `5` | After this many crashes of the language server, since it was last restarted by hand, the extension stops restarting it and explains why. |
| `ascribe.trace.server` | `off` | `messages` or `verbose` logs the conversation with the server, for reporting a problem. |

## Commands

| Command | What it does |
|---|---|
| **Ascribe: Restart Language Server** | Stops and starts the server, and forgets earlier crashes. |
| **Ascribe: Show Server Output** | Opens the server's log. |
| **Ascribe: Open Preview to the Side** | Opens the preview. |
| **Ascribe: Select Preview Build** | Picks the build the preview shows. |

## Other editors

Any editor with a Language Server Protocol client can run `ascribe lsp` from a project installed with `@ascribed/cli`. It speaks LSP over standard input and output, and needs the workspace folder that holds `ascribe.toml`. The preview is a VS Code feature.

## Other formatters

A Markdown formatter that reflows paragraphs, such as Prettier with `proseWrap: always`, doesn't know that title and directive lines start new blocks, and would join them with the text below. Exclude Ascribe sources from it, for example with `**/*.md` in `.prettierignore`, and use `ascribe fmt`.
