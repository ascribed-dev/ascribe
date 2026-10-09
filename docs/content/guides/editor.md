---
title: Editing
description: The VS Code extension, workspaces with several projects, and other editors.
---

The Ascribe extension for VS Code turns the editor into an Ascribe authoring environment. Its features come from the language server, `ascribe lsp`, which runs the same checks as `ascribe check`: what the editor reports is what CI reports.

## Install

Install **Ascribe** (`Ascribe.ascribe-vscode`) from the Visual Studio Marketplace: search for "Ascribe" in the Extensions view, or run:

```sh
code --install-extension Ascribe.ascribe-vscode
```

The extension activates in a workspace that has an `ascribe.toml` anywhere in it, and a workspace can hold several projects (see [Workspaces with several projects](#workspaces-with-several-projects)). It needs VS Code {vscode} or later, and a trusted workspace, since it runs the project's `ascribe` binary. It's available for macOS on Apple silicon, Linux (x64 and arm64), and Windows (x64); Intel Macs aren't supported.

### Which `ascribe` it runs

The extension includes an `ascribe` binary for your platform, so it works immediately. For each project, it looks for a binary in this order:

1. The `ascribe.path` setting, if you set it. It applies to every project. A path that doesn't work is an error; the extension doesn't fall back.
2. The project's own `node_modules/.bin/ascribe`, installed with `npm install --save-dev @ascribed/cli`. It looks in the folder that holds the project's `ascribe.toml`, then in each parent up to the workspace folder, so a project in a monorepo finds the `node_modules` at the repository root.
3. The binary included in the extension.

A project that installs `@ascribed/cli` gets exactly the version it pins, in the editor as in CI, and two projects in one workspace can pin different versions. When a version is older than the extension expects, the extension warns you to update it. **Ascribe: Show Server Output** shows which binary a project uses.

## What it does

### Diagnostics

Every [diagnostic](../reference/diagnostics.md) appears as you type, including in unsaved files, for every file of the project, open or not: fix a heading's id, and the broken links to it in other files clear. Page-level diagnostics are for the [build](../reference/content-model.md#16-buildsname) named by `[editor] build` in `ascribe.toml`.

In a workspace with several projects, the Problems panel lists only the projects whose language server is running. See [The Problems panel](#the-problems-panel).

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

### Actions
@available: next

Actions write Ascribe for you: wrap a paragraph in a note, turn a numbered list into steps, link the selected text to a page, insert content that varies by a dimension, mark where a section is available. Each one asks what it needs in short steps, choosing from what the project has (its note types, pages and headings, phrases, dimensions and their values, features, images, fragments, sources, and widgets), so you never type a path, a key, or attribute syntax. What it writes is in canonical form, and one **Undo** takes it back.

Every action is in three places:

- The **Command Palette**, as `Ascribe: <action>`, in a Markdown file of a project. Run where it doesn't apply, it says where it does: "Put the cursor in a note to change its kind."
- The editor's context menu, under **Ascribe**, which lists the actions that apply where the cursor is.
- The lightbulb (`Ctrl+.` / `Cmd+.`), for the actions that rewrite what's at the cursor, beside the quick fixes.

**Copy a link to this section** copies the heading's destination from the content root, such as `/guides/install.md#install-cli`, which works pasted into any page of the project.

@include: ../_generated/editor-actions.md

### Formatting

**Format Document** rewrites Ascribe constructs into canonical form, as `ascribe fmt` does. Turn on `ascribe.formatOnSave` to do it on every save. It changes only Ascribe constructs, never how a page renders.

### Highlighting

Directive lines, attribute blocks, `@end`, and phrases are highlighted as soon as a file opens. Once the server is running, it adds what depends on the content model: declared and undeclared phrases, project widgets, and **title lines**, shown distinctly so a paragraph that accidentally became a title stands out.

### Preview

**Ascribe: Open Page Preview to the Side** (also the preview button in a Markdown editor's title bar) shows the current page as the published site shows it, with the same elements, including unsaved changes. It follows the editor: it updates as you type, keeps its scroll position, and opens a page or file you click in the editor. It scrolls with the editor by block, both ways: scrolling the editor brings the block at its top to the top of the preview, moving the cursor into a block the preview doesn't show brings it into view, scrolling the preview scrolls the editor to the block at its top, and double-clicking a block puts the cursor on its first line. A block from a fragment follows the line of its `@include`. Turn off `ascribe.preview.scrollPreviewWithEditor` or `ascribe.preview.scrollEditorWithPreview` to stop one direction. Its **Build** picker shows the page as any build publishes it, starting with the editor's build. In a workspace with several projects, each project keeps its own choice of build.

A fragment isn't a page, so the preview names the pages that include it instead. A page a build doesn't publish says which build drops it, and why.

A file with no `ascribe.toml` above it has nothing to preview: "This file isn't part of an Ascribe project (no ascribe.toml above it)." A file in a project's folder but outside its content root isn't a page either, and the preview names both: "handbook/README.md is in the project handbook, but outside its content root (handbook/pages), so there is no page to preview."

The preview shows images and files from the content root, and from directories elsewhere in the project that a page uses. For safety, it runs no inline scripts and loads nothing remote, so raw HTML that needs either looks different in the preview than on the site.

@available: next
A [`@snippet`](../reference/directives.md#snippet) is read again when its page changes, not when its code file does, so the preview and the page's problems don't follow edits to the code. To see them, change the page, or run **Ascribe: Restart Language Server**.

### Site preview
@available: next

The page preview is the page alone. The **site preview** is the same page in the real site, with its layout, navigation, and styles, from the site's dev server. With [`@ascribed/astro`](astro.md), `astro dev` writes where it's running to `.ascribe/dev.json` in the project, and removes the file when it stops.

**Ascribe: Open Site Preview** opens the active page on the dev server in your browser, at the heading the editor shows. It's also the globe button in a Markdown editor's title bar while a dev server is running. With no dev server, or one that stopped without removing `dev.json`, it says to start one. It only opens an address on your machine (`localhost`, `127.0.0.1`, or `[::1]`), so a `dev.json` committed to a repository can't send it elsewhere. In a remote workspace, VS Code forwards the dev server's port first. The page preview's **Page | Site** switch, at the end of its toolbar, shows the site preview in the panel instead: the dev server's page for the same file, following the active file as **Page** does. Links you follow inside it stay until you open another file. With no dev server, it says to start one, with **Try again**. In VS Code for the Web, where the panel can't show the dev server, **Site** opens the browser instead. [Review in the site preview](astro.md#review-in-the-site-preview) has the toolbar app that marks changes and shows comments there; each block's and thread's **Open source** brings you back to the file, at the line.

### Review in the preview
@available: next

The preview can mark what changed against a git revision, the **base**, as [`ascribe diff`](../reference/cli.md#ascribe-diff) reports it: added and changed blocks with a bar and a label, the changed words highlighted, removed blocks where they were, and moved blocks linked to where they came from. A tab's label and a `details`' summary say what changed in what they can hide: "new" for an added tab, else how many changes. It's for reading a change as a page, your own before you push it or someone else's on a checkout of their branch. [Review](review.md) walks through reviewing a pull request, start to finish; this section and the next are the reference for the editor's part.

**Ascribe: Start Review** turns it on for the active page's project. It asks for the base: the base of your branch's pull request, when it has one, the default branch (the first of `origin/HEAD`, `origin/main`, `origin/master`, `main`, and `master` that exists), or a branch, tag, or commit you type. Either way it compares with the point where your branch left it, as a pull request does. The status bar shows **Review: off**, or the base, while an Ascribe page or the preview is active; click it to start review, or to list the changed pages once it's on. While review is off, the preview has no review header and looks as it always does; its title bar has a **Start Review** button, which becomes **Changed Pages** once review is on. Review stays on for the project until **Ascribe: Stop Review**, or until the window closes. The tooltips on the base, in the header and the status bar, give the commit compared with. When that point moves (you pull, rebase, or fetch a newer base branch), review follows it the next time the preview regains focus, and says so.

With review on, the preview's header shows the base, how many changes the page has (click it for the breakdown), **Changes / As it will be / As it was**, and next and previous change, with your place: "3 of 10 on this page". Past the last change, it offers the next changed page. A page that changed only through something it uses, such as a fragment, says so and links to the file. When the project has errors, the header says how many, since a page with an error may not show as it will once it's fixed, with **Show problems** to open the Problems panel. The count is the Problems panel's, which checks pages for the editor's build only. Clicking a mark's label opens the block's source with its lines selected. The marks follow your edits as you type, saved or not.

**Ascribe: Changed Pages**, also the list button in the preview's title bar, lists the pages the change touches in the preview's build, by title, each with its counts and, when its own file didn't change, what it changed through. A title with [code](../reference/content-model.md#53-code-in-a-field) shows it between backticks, as in "`` `loom.yaml` options ``"; the preview's heading shows it as code. Choosing one opens it and its preview.

### Comments in the preview
@available: next

When your branch has an open pull request on GitHub, review shows its review threads beside the blocks they're on, and lets you review the change there. **Start Review** looks for the pull request first, and then offers its base as the first choice. The header names the pull request ("#128 against main"); the number opens it on GitHub, and **Comments (N)** lists every comment on the pull request's pages.

- **Reading.** Each thread sits in a column beside its block, highlighted with a line to the block when you point at or focus either. A narrow preview shows a count on each block instead, which opens its threads. Resolved threads are collapsed. A tab's label and a `details`' summary say how many comments are inside, and opening a thread inside a tab that isn't showing, or a closed `details`, shows it first. **Outdated** and **Detached** threads are described in [How comments map to the pull request](review.md#how-comments-map-to-the-pull-request).
- **Commenting.** Point at or focus any block and choose **Comment**. The comment is unsent until you submit; when GitHub can't take it on the block's lines, the box says so before you write, and it goes in your review's summary. A block with unsaved changes can't take a comment until you save its file, since GitHub counts the file's saved lines; threads stay beside their blocks as you type.
- **Replying.** **Add to review** holds a reply with your other comments. **Reply now** sends it at once, except while you have unsent comments, when it's disabled and says why.
- **Resolving.** **Resolve** and **Reopen** act on GitHub at once.
- **Submitting.** While you have unsent comments, a bar at the bottom counts them, with **Submit review…**: it lists them, and submits them as a comment, an approval, or a request for changes, with an optional summary. **Discard…** in the same dialog deletes them, after asking.

The same threads show on their lines in the source editor, where you can reply, resolve, and comment on any line into the same review. The GitHub Pull Requests extension shows them there already, so by default Ascribe leaves the source editor to it when it's active; `ascribe.review.sourceComments` decides.

Comments need GitHub. Review asks to sign in to GitHub in VS Code when you start it, never sooner; the permission it asks for (`repo`) is the one posting review comments needs. If you decline, review shows the changes only, and the header offers **Sign in to see comments**, and **Use GitHub CLI** when `gh` is signed in. Once you choose the GitHub CLI, review uses it in that workspace from then on, until you choose **Sign in to see comments**. The preview itself never holds the sign-in or makes a request: everything goes through the extension.

The comments are read when review starts, on **Ascribe: Refresh Comments** (the refresh button in the preview's title bar), and after your own actions. When your checkout isn't the pull request's latest commit, the header says so: behind, with **Pull**, since some comments may be on lines you don't have; ahead, with **Push**, since you can comment only on lines that are on GitHub.

Review runs `git` once you start it. Before that, only the offer does: once you've used review in a workspace, opening a page runs `git` to find its branch, and asks GitHub whether the branch has an open pull request, to offer review ([Review](review.md) describes the offer). A project that isn't in a git repository, a revision that doesn't exist, or `git` missing from the path stops it from starting, with a message that says which; the preview works as before. Starting review reads the project as it was at the base, which takes about as much memory again as the project; stopping review frees it.

## Workspaces with several projects

Every `ascribe.toml` in the workspace is a project, with its own language server. A repository that keeps its code and its documentation together can hold several, such as `site/ascribe.toml` and `handbook/ascribe.toml`, and you can work on all of them in one window. Folders named `node_modules` aren't searched.

A file belongs to the nearest `ascribe.toml` above it, and only that project's server reports on it. Projects can be nested: a file in `handbook/internal/` belongs to `handbook/internal/ascribe.toml`, not to `handbook/ascribe.toml`. A project nested in another's content root isn't part of it: the outer project's server, `ascribe check`, and `ascribe build` skip the nested project's folder, so a link or an include from the outer project to one of its pages is reported as a missing file. A file in a project's folder but outside its content root isn't one of the project's sources, and gets no diagnostics.

Each project runs its own `ascribe`, found as [Which `ascribe` it runs](#which-ascribe-it-runs) describes.

### When servers start

A project's server starts the first time you open one of its Markdown files or its `ascribe.toml`, or preview one of its pages. It then runs until you close the window or delete its `ascribe.toml`. A project you never open costs nothing.

To start every project's server when the workspace opens, set `ascribe.startServers` to `"all"`.

An `ascribe.toml` you add or delete is picked up as you work. If the editor misses a new one, opening or switching to one of the new project's files picks it up. The extension serves at most 50 projects in a workspace: the first 50 by path. When there are more, the first project's output says so.

Each server uses about 6 MB of memory, plus about 70 KB per page of its project.

### The Problems panel

The Problems panel lists diagnostics only for the projects whose server is running. A project whose files you haven't opened shows nothing there, even if it has problems. Open one of its files, or set `ascribe.startServers` to `"all"`, to see them.

`ascribe check` checks a project whether or not the editor has it open. Run it in the project's folder, or from anywhere with `--config` naming the project's folder or its `ascribe.toml`:

```sh
npx ascribe check --config handbook
```

### Commands and output

Commands act on the project that owns the active file: the preview, its **Build** picker, review, and **Show Server Output**. **Restart Language Server** restarts every project's server that has started. [Commands](#commands) has the details.

Each project's server logs to its own output channel, `Ascribe (<project>)`. The project's name is its folder relative to the workspace folder, such as `Ascribe (handbook/internal)`, or the workspace folder's name for a project at its root. In a workspace with one project, the channel is just `Ascribe`.

A channel's name is set when the channel is first needed, usually when its project's server first starts. So if a workspace with one project gains a second, the first project's channel keeps the name `Ascribe`, beside `Ascribe (<other project>)`, until you reload the window.

When it starts, a server logs the `ascribe.toml` it uses: `using the project at <path>`.

## Settings

@include: ../_generated/editor-settings.md

## Commands

@include: ../_generated/editor-commands.md

## Other editors

Any editor with a Language Server Protocol client can run `ascribe lsp` from a project installed with `@ascribed/cli`. It speaks LSP over standard input and output. The preview is a VS Code feature.

A server serves one project: the nearest `ascribe.toml` at or above the workspace folder the editor gives it. It never looks below that folder, so a server started at the root of a repository whose projects are all in subfolders has no project. For a workspace with several projects, start one `ascribe lsp` per project, rooted at the folder that holds its `ascribe.toml`, and give each file to the server of the nearest `ascribe.toml` above it. In Neovim, for example, `root_markers = { "ascribe.toml" }` in the server's `vim.lsp.config` does both.

When it starts, the server logs to standard error which project it uses: `using the project at <path>`, or, when it finds none, `no ascribe.toml at or above <folder>`. A server with no project reports nothing. If an `ascribe.toml` is then created in the workspace folder itself, the server loads it, when the editor supports watching files for the server.

When one project is nested in another's content root, the outer project's server leaves the nested project's files alone, open or not, so each file gets diagnostics from one server only. A nested project's `ascribe.toml` created or deleted while the server runs changes which files it checks, when the editor supports watching files for the server.

## Other formatters

A Markdown formatter that reflows paragraphs, such as Prettier with `proseWrap: always`, doesn't know that title and directive lines start new blocks, and would join them with the text below. Exclude Ascribe sources from it, for example with `**/*.md` in `.prettierignore`, and use `ascribe fmt`.
