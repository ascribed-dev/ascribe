---
type: reference
title: Editing
description: The VS Code extension, workspaces with several projects, and other editors.
permalink: editor-guide
---

The Ascribe extension for VS Code turns the editor into an Ascribe authoring environment. Its features come from the language server, `ascribe lsp`, which runs the same checks as `ascribe check`: what the editor reports is what CI reports.

Every setting and command the extension adds, and how it works in a workspace with several projects, are on [their own page](settings.md).

## Install

Install **Ascribe** (`Ascribe.ascribe-vscode`) from the Visual Studio Marketplace: search for "Ascribe" in the Extensions view, or run:

```sh
code --install-extension Ascribe.ascribe-vscode
```

The extension activates in a workspace that has an `ascribe.toml` anywhere in it, and a workspace can hold several projects (see [Workspaces with several projects](settings.md#workspaces-with-several-projects)). It needs VS Code {vscode} or later, and a trusted workspace, since it runs the project's `ascribe` binary. It's available for macOS on Apple silicon, Linux (x64 and arm64), and Windows (x64); Intel Macs aren't supported.

### Which `ascribe` it runs

The extension includes an `ascribe` binary for your platform, so it works immediately. For each project, it looks for a binary in this order:

1. The `ascribe.path` setting, if you set it. It applies to every project. A path that doesn't work is an error; the extension doesn't fall back.
2. The project's own `node_modules/.bin/ascribe`, installed with `npm install --save-dev @ascribed/cli`. It looks in the folder that holds the project's `ascribe.toml`, then in each parent up to the workspace folder, so a project in a monorepo finds the `node_modules` at the repository root.
3. The binary included in the extension.

A project that installs `@ascribed/cli` gets exactly the version it pins, in the editor as in CI, and two projects in one workspace can pin different versions. When a version is older than the extension expects, the extension warns you to update it. **Ascribe: Show Server Output** shows which binary a project uses.

### The walkthrough
@available: ascribe 0.3.0

After you install the extension, VS Code opens its walkthrough, **Get Started with Ascribe**, which goes through what a writer does first: open a project, preview a page, use the actions bar, look at the Ascribe sidebar, see what a build leaves out, and check the project in CI. To see it again, open **Help → Welcome**, or run **Welcome: Open Walkthrough…** and choose it. Its buttons act on the page you last had open.

## What it does

### Diagnostics

Every [diagnostic](../checks/diagnostics.md) appears as you type, including in unsaved files, for every file of the project, open or not: fix a heading's id, and the broken links to it in other files clear. Page-level diagnostics are for the [build](../content-model/output.md#16-buildsname) named by `[editor] build` in `ascribe.toml`.

@available: ascribe 0.3.2
When that build is [versioned](../versions/index.md), they're for every line of it, and a problem only some lines have says which, such as "(only in build `site@3.4`)".

The [content checks](../checks/diagnostics/content-checks.md) about the whole project, such as a page nothing links to or a phrase nothing uses, need every page, so they run when you save, not as you type. Until you save a file you've changed, it doesn't show theirs. The image checks run only in `ascribe check`.

In a workspace with several projects, the Problems panel lists only the projects whose language server is running. See [The Problems panel](settings.md#the-problems-panel).

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

@available: next
- **Rewrite as a memo**, for an [HTML comment](../writing/directives.md#memo) that's a block of its own. One line becomes a line memo and more a container, about the block below it, or at the top of the section when the comment is the section's last block. A comment at the end of a list item, or before a page's first heading, gets no fix, since where its memo belongs is yours to say.

@available: ascribe 0.3.2
- **Copy the command**, for an [example](../examples/tested.md) that changed after its last test run or that no test run lists. It copies the line that runs `ascribe observe` in the example's folder, for you to run in a terminal: for PowerShell on Windows, and for `sh` elsewhere. The editor never runs a test itself.

### Apply the safe fixes
@available: next

**Source Action… → Apply the safe fixes in this page** applies, in one edit, every fix marked safe for the problems the editor shows in a page or a fragment, such as linking to a page's file instead of its route, or removing an empty memo, and leaves the rest. It takes them as [`ascribe check --fix`](../checks/check.md#applying-the-safe-fixes) does: two that would edit the same text are one too many, so the first is applied and the other's problem stays, for the next time. The actions bar lists it first among its fixes when the page has two or more. A problem that takes every build to tell, such as an acknowledgement that covers nothing, isn't one the editor shows, so `ascribe check --fix` is where its fix is applied.

To apply them each time you save a page, set `editor.codeActionsOnSave` for Markdown in your settings. Nothing sets it for you:

```json
"[markdown]": {
  "editor.codeActionsOnSave": { "source.fixAll.ascribe": "explicit" }
}
```

### Prompt agent

@available: ascribe 0.3.0
On a problem, the lightbulb also offers **Prompt agent: fix this problem**, after the problem's quick fixes. It builds a prompt for your AI agent about the problem, and copies it, or opens your agent with it filled in, as `ascribe.agents.promptTarget` says. **Ascribe: Prompt Agent to Fix This File** and **Ascribe: Prompt Agent to Fix This Project** do the same for every problem in the file or the project. Nothing is sent: you read the prompt and send it. See [Prompt your agent](../agents/index.md#prompt-your-agent).

### Agents in VS Code
@id {permalink=true}: agents-in-vs-code

@available: ascribe 0.3.0
An agent in VS Code, such as GitHub Copilot in agent mode, gets Ascribe's tools with nothing to set up. In a workspace with an Ascribe project, the extension offers [`ascribe mcp`](../agents/index.md#the-mcp-server) as an MCP server, named **Ascribe** in VS Code's list of MCP servers. It runs the binary the first project uses (see [Which `ascribe` it runs](index.md#which-ascribe-it-runs)), in the workspace folder, and VS Code starts it when an agent first needs it. When the workspace's projects use different binaries, the project's output says which one the server runs.

The extension also gives agents three tools of its own, for what only the editor knows: text that isn't saved, and the review that's open. Name one in chat with `#`, or let the agent pick it.

@include: ../_generated/editor-tools.md

`#ascribe_editor_problems` answers with the JSON `ascribe check --format json` writes, from the language server, so it includes unsaved edits. When a file changed since the server last checked it, the tool waits for the server, up to a second, and says which version of the document its answer is for. Like the Problems panel, it covers the [editor's build](index.md#diagnostics) only; `ascribe check` checks every build. It never starts a language server: a project whose server isn't running gets a line saying so.

The review tools answer while [review](index.md#review-in-the-preview) is on, and say so in one line when it's off; they never turn it on. `#ascribe_review_threads` gives each open thread's comments as data, under the same sentence a [prompt](../agents/commands.md#what-a-prompt-says) puts over them, so the agent doesn't take them as instructions. None of the tools changes a file.

An agent that runs `ascribe` runs the binary the editor runs. When `ascribe` on the window's path is another binary or none, the extension puts the folder of the binary each workspace folder's first project uses first on the path of that folder's terminals; a terminal that was open before asks to be relaunched. [Prompt agent](../agents/index.md#prompt-your-agent)'s prompts then name the binary by its path, for an agent that doesn't work in the window's terminals, such as Claude Code's extension.

### Completion

- After `@` at the start of a line: the built-in directives and the project's widgets, with their descriptions.
- In an attribute block: the directive's attribute keys, then their allowed values, such as note types or a dimension's values with their labels. After an image: the declared image attributes.
- In `@available:` and frontmatter `available:`: targets, dimension names, feature keys, and lifecycle states.
- In frontmatter `type:`: the content types `ascribe.toml` declares, each with what it's for.
- After `{` in text: the declared phrases, each with its value.
- In `@include:`: source files, then, after `#`, the ids of the file.
- In a link destination: pages and headings, **searched by title**. It inserts the file path and id, so you never need to type a path.

@available: ascribe 0.3.2
- In a source's [`ascribe.examples.toml`](../examples/tested.md): its keys, and an environment entry's `label` and `command`. The file's problems appear as you type, where each is. A config no source in `ascribe.toml` names is just a TOML file.

### Hover

- A link, image, or include: its target's path, title, and the first paragraph, with phrases filled in. A fragment, a missing file or id, a link written as a URL, and a name whose case doesn't match are called out.
- A phrase: its value.
- An availability spec or feature key: what it means, in the words the published site uses.
- A directive or attribute key: its description. A dimension key: its values and labels.

@available: ascribe 0.3.2
- A version in an availability spec, for a target with [release lines](../versions/index.md): the line it falls in, and that line's status, such as "in line 3.4, live".

@available: ascribe 0.3.2
- `replaces` in frontmatter: the page it replaces, and which page each line publishes. Go to Definition opens the replaced page.

@available: ascribe 0.3.2
- A `@snippet`'s address: its file, as a link that opens it, and its region and lines. For a [tested example](../examples/tested.md), what it was tested with, "Changed after its last test run", or "Not tested", with the observations behind it, and where to run `ascribe observe`.

### Navigation

- **Go to definition** on a link or include opens the target file at its heading. On an `@id`, it goes to the heading; on a phrase or feature key, to its entry in `ascribe.toml`.
- **Find All References** (`Shift+F12`, or **Go to References**) lists every place that uses what's under the cursor: on a heading or its `@id`, the links to it, including links through a page that includes it, and the includes of its section; at the start of a page, or on its frontmatter or title, the links and includes from other files; on a link or include, those of what it names; on a phrase, a feature key, a glossary term, a note's type, a widget, or a `@variant` attribute, every use of that phrase, feature, term, note type, widget, or dimension. A glossary term counts where its text, or an alias, is in prose, whether or not the build links it there. A term with `match = "marked"` is only a word until you link it, so it has no uses of its own: use **Find All References** on its page or heading instead.
- **Document links**: links, images, and includes are clickable. A link outside the docs is clickable when it's a web address (`http`, `https`) or an email address (`mailto`); one with any other scheme, such as `tel:` or `vscode:`, stays as it's written for the site's readers, and isn't a link in the editor.
- **CodeLens** above each `@include` names the included file and section, and opens it.
- **Inline hints** show the title an empty-text link, `[](keys.md#rotate-keys)`, will get.

### Code files a page shows
@available: next

A code file a page shows with [`@snippet`](../writing/directives.md#snippet) says which pages show it: above each region a snippet takes, and on the first line of a file a snippet takes whole, a lens reads "Shown in 3 pages". A page counts once however many of its `@snippet`s show the region, and a fragment's `@snippet` counts each page that includes the fragment. Choose the lens to see the places in the references peek, as **Find All References** lists them, and choose one to open the page at its `@snippet`. It's how you find the pages a change to your code will change, as you make it.

The lens is placed from the saved file, as a build reads it: the editor doesn't send code files to the language server. Lines you add above a region leave its lens where the region was until you save the file. It's there for the files of the content model's [sources](../content-model/output.md#18-sourcesname), while the project's language server is running, and nowhere else. A file in more than one project's sources gets a lens from each project, each counting its own pages and naming the project, such as "docs: shown in 3 pages", since those pages are built and published apart.

### Refactoring

- **Rename or move a file** in the Explorer, and the links and includes that point to it are updated, and so are the glossary terms' `link` and the retired permalinks' `to` in `ascribe.toml` that name it, and the `about` of each [memo file](index.md#memo-files) about it or a heading in it.
- **Rename** (`F2`) a heading's `@id`, or its text, to update the links to it, the glossary terms' `link` and the retired permalinks' `to` in `ascribe.toml` among them, and the memo files about it, or a phrase key to update its uses and its declaration.
- **Rename** (`F2`) a dimension value, in a `@variant` attribute or in its dimension's `values` in `ascribe.toml`, to update it everywhere: the dimension's `labels` and `versionless`, every `@variant` attribute and `variant:` field, every availability spec in pages, table rows, frontmatter, and `[features]`, and the builds' `variants` and `filter`.
- A phrase rename reaches every use: prose, link text and destinations, frontmatter fields with `phrases = true`, code blocks with `phrases=true`, and the files a `@snippet {phrases=true}` shows.
- `F2` on something that can't be renamed says why, and on something that can, selects what will be renamed. A new name the content model doesn't allow, or that's already taken, changes nothing.
- From the command line, [`ascribe rename`](../command/questions.md#cli-rename) makes the same edits, for a phrase, a dimension value, a heading's id, or a page that moves.

### Formatting

**Format Document** rewrites Ascribe constructs into canonical form, as `ascribe fmt` does. Turn on `ascribe.formatOnSave` to do it on every save. It changes only Ascribe constructs, never how a page renders.

When another extension is VS Code's formatter for Markdown, such as Prettier through `editor.defaultFormatter`, it formats your pages too, and can change what they build to: `version: {version}` becomes `version: { version }`. When you open a page and another formatter is set, the extension offers to be the Markdown formatter for that workspace folder. **Use Ascribe** writes `editor.defaultFormatter` under `[markdown]` in the folder's `.vscode/settings.json`, so **Format Document** and `editor.formatOnSave` run Ascribe on your pages. Ascribe formats only its projects' pages, so other Markdown files in the folder are no longer formatted. **Keep** leaves the setting as it is and doesn't ask again for that folder.

### Highlighting

Directive lines, attribute blocks, `@end`, and phrases are highlighted as soon as a file opens. Once the server is running, it adds what depends on the content model: declared and undeclared phrases, project widgets, and **title lines**, shown distinctly so a paragraph that accidentally became a title stands out.

@available: next
A [memo](../writing/directives.md#memo)'s text is colored as a comment, in either form, and nothing in it is colored as a link, a phrase, or a heading. A container memo folds to its first line, beside the folding of headings, lists, and code blocks.

### Preview

**Ascribe: Open Page Preview to the Side** (also the preview button in a Markdown editor's title bar) shows the current page as the published site shows it, with the same elements, including unsaved changes. It follows the editor: it updates as you type, keeps its scroll position, and opens a page or file you click in the editor. It scrolls with the editor by block, both ways: scrolling the editor brings the block at its top to the top of the preview, moving the cursor into a block the preview doesn't show brings it into view, scrolling the preview scrolls the editor to the block at its top, and double-clicking a block puts the cursor on its first line. A block from a fragment follows the line of its `@include`. Turn off `ascribe.preview.scrollPreviewWithEditor` or `ascribe.preview.scrollEditorWithPreview` to stop one direction. Its **Build** picker shows the page as any build publishes it, starting with the editor's build: it changes the [build you're looking at](index.md#the-status-bar), which the status bar names and switches too.

A fragment isn't a page, so the preview names the pages that include it instead. A page a build doesn't publish says which build drops it, and why.

A file with no `ascribe.toml` above it has nothing to preview: "This file isn't part of an Ascribe project (no ascribe.toml above it)." A file in a project's folder but outside its content root isn't a page either, and the preview names both: "handbook/README.md is in the project handbook, but outside its content root (handbook/pages), so there is no page to preview."

The preview shows images and files from the content root, and from directories elsewhere in the project that a page uses. For safety, it runs no inline scripts, loads nothing remote, and submits no forms, so raw HTML that needs any of these looks different in the preview than on the site. What the preview's page asks the extension to do is checked first: it opens only files in the project, or ones the preview itself linked to, and only the **Pull**, **Push**, and **Fetch** git commands it has buttons for.

@available: ascribe 0.2.0
A [`@snippet`](../writing/directives.md#snippet) is read again when its page changes, not when its code file does, so the preview and the page's problems don't follow edits to the code. To see them, change the page, or run **Ascribe: Restart Language Server**.

@available: ascribe 0.3.2
A [tested example](../examples/tested.md)'s advice, `example-stale` and `example-unobserved`, follows its source's observations and `ascribe.examples.toml`: when `ascribe observe` writes an observation, or the config changes on disk, the pages that show the source's examples are checked again. An edit to the example's own file still waits for the page to change, and so does the preview's tested line.

### Site preview
@available: ascribe 0.3.0

The page preview is the page alone. The **site preview** is the same page in the real site, with its layout, navigation, and styles, from the site's dev server. With [`@ascribed/astro`](../astro/index.md), `astro dev` writes where it's running to `.ascribe/dev.json` in the project, and removes the file when it stops.

**Ascribe: Open Site Preview** opens the active page on the dev server in your browser, at the heading the editor shows. It's also the globe button in a Markdown editor's title bar while a dev server is running. With no dev server, or one that stopped without removing `dev.json`, it says to start one. It only opens an address on your machine (`localhost`, `127.0.0.1`, or `[::1]`), so a `dev.json` committed to a repository can't send it elsewhere, and it doesn't contact that address until you ask for the site preview. In a remote workspace, VS Code forwards the dev server's port first. The page preview's **Page | Site** switch, at the end of its toolbar, shows the site preview in the panel instead: the dev server's page for the same file, following the active file as **Page** does. Links you follow inside it stay until you open another file. The panel takes no messages from the dev server's page: only the extension changes what it shows. With no dev server, it says to start one, with **Try again**. In VS Code for the Web, where the panel can't show the dev server, **Site** opens the browser instead. [Review in the site preview](../astro/reference.md#review-in-the-site-preview) has the toolbar app that marks changes and shows comments there; each block's and thread's **Open source** brings you back to the file, at the line.

### Actions
@available: ascribe 0.3.0

Actions write Ascribe for you: wrap a paragraph in a note, turn a numbered list into steps, link the selected text to a page, insert content that varies by a dimension, mark where a section is available. Each one asks what it needs in short steps, choosing from what the project has (its note types, pages and headings, phrases, dimensions and their values, features, images, fragments, sources, and widgets), so you never type a path, a key, or attribute syntax. What it writes is in canonical form, and one **Undo** takes it back.

Every action is in four places:

- The **actions bar** (`Ctrl+K A` / `Cmd+K A`), which lists only what applies where the cursor is. See [The actions bar](index.md#the-actions-bar).
- The **Command Palette**, as `Ascribe: <action>`, in a Markdown file of a project. Run where it doesn't apply, it says where it does: "Put the cursor in a note to change its kind."
- The editor's context menu, under **Ascribe**, which lists the actions that apply where the cursor is.
- The lightbulb (`Ctrl+.` / `Cmd+.`), for the actions that rewrite what's at the cursor, beside the quick fixes.

@include: ../_generated/editor-actions.md

#### The actions bar

Press `Ctrl+K A` (`Cmd+K A` on macOS), or run **Ascribe: Actions for the Cursor**, in a page of a project to see what you can do where the cursor is. The bar lists the fixes for problems at the cursor first, then the actions that apply to the cursor or the selection, grouped as **Write**, **Structure**, **Link**, **Media**, **Content model**, and **Memos**. Each shows what it does and the syntax it writes. Type to filter, and choose one: an action that needs something asks for it in the same box. When nothing applies, the bar says where to put the cursor.

The bar holds only what applies to the cursor or selection, so it never lists every action as the Command Palette does, and it has no searches or commands of its own.

To use another key, open **Preferences: Open Keyboard Shortcuts**, search for `ascribe.actions`, and change its keybinding. The default is on only in a Markdown file of a project, with the editor focused.

#### Changing the content model

The actions in the bar's **Content model** group change `ascribe.toml` from a page: **Make this a phrase**, **Add to the glossary**, **Change a feature's availability**, **Rename this phrase everywhere**, and **Rename a dimension value everywhere**. A rename changes what `F2` does (see [Refactoring](index.md#refactoring)), for the phrase or value at the cursor, or one you pick from the project's wherever the cursor is.

These change `ascribe.toml` in place, keeping its comments, blank lines, and order, and an open `ascribe.toml` with unsaved changes is edited as it is. A rename that changes other files lists every change in the refactor preview first, where you check the changes to make and **Apply** them, or **Discard** them all.

### Start a page
@available: ascribe 0.3.1

**Ascribe: New Page** makes a page from its type's [template](../templates/index.md). Run it from the Command Palette, from the **Pages** view's title bar, or on a folder in the Explorer. It asks for the page's type, each with what it's for (the type's `purpose`, or for a type from the starter set, their summary of it), then its title. The page is the template with its `type` and `title` set, named for the title, in the content root; run on a folder, it makes the page in that folder. A page names its type in its frontmatter, so any folder of pages takes every type; a fragment's folder takes none. It's saved and opened with the first thing the template left to fill in selected, usually the description, so typing replaces it. A page is never made over a file: with a file of that name there, it says so, and you give another title.

**Ascribe: Go to Next Placeholder** selects the next thing the template left to fill in after the cursor: a frontmatter field still as the template has it, or a `\{placeholder}`, a whole one even when it runs across a paragraph and a list. After the last, it goes back to the first. They're the placeholders and fields [`template-placeholder-left`](../templates/index.md#whats-left-to-fill-in) reports, found the same way; the template's instructions aren't among them, since you read those rather than fill them in. To give it a key, open **Preferences: Open Keyboard Shortcuts** and search for `ascribe.nextPlaceholder`.

A type with no template makes a page with its frontmatter: its `type`, the title, and each other field the type requires, empty.

### Memo files
@available: next

A [memo file](../writing/memo-files.md) says why a section or a block is written the way it is. The editor shows them where they apply, and writes them for you. [Leave a memo for whoever edits next](../writing/memos.md) says when to write one.

- A **CodeLens** above a heading or a block that has memos counts them: "1 memo", "2 memos", or "3 memos, 1 outdated" when one's quote isn't found any more. A lens at the top of the page counts the memos about the whole page. Click it to open the memo, or, with several, to pick one; each opens at its text.
- **Ascribe: Add a memo** makes a file in `memos/` about the section the cursor is in, by its id, or about the page when the cursor is above its first heading with an id. Its `quote` is the selected text, or the first line of the block at the cursor; on a heading it has none, and it's about the heading's section. The file is named from the quote's first words, or the heading's id, and never made over a file that's there. It's saved and opened at its text, so you start writing why. It's in the actions bar's **Memos** group, the Command Palette, and the editor's **Ascribe** submenu.
- **Ascribe: Add a personal memo** does the same in `.ascribe/memos/`, which stays on your machine: see [Personal memos](../writing/memo-files.md#personal-memos).
- In a memo file, completion after `about:` offers the project's pages, their headings with ids as `page.md#id`, and its phrases, features, glossary terms, and the rest by `prefix:name`, as [`ascribe refs`](../command/questions.md#cli-refs) takes them. `about` is a link to what it names, and the file shows the problems `ascribe check` would report in it as you type, a personal memo's too.

**This page**'s **Memos** section lists the memos about the heading at the cursor, or the page: see [This page, Pages, and Content model](index.md#this-page-pages-and-content-model).

### The Ascribe sidebar
@available: ascribe 0.3.0

The Ascribe icon in the activity bar opens the Ascribe sidebar, with four views: **Projects**, **This page**, **Pages**, and **Content model**.

@available: next
While review is on, a **Review** view comes first: see [The Review view](index.md#the-review-view).

#### The Projects view

The **Projects** view lists every project in the workspace, started or not, by name, with its folder and an icon for its server's state. Under each is its `ascribe.toml`, which opens it, and, while its server runs, its editor build and the `ascribe` binary in use. Each project's buttons show its server's output, and restart its server when it's running or failed. The view updates as projects come and go and servers start and stop, and opening it starts no server. In a workspace with no project, it says how to start one.

#### This page, Pages, and Content model

The other three views show the active file's project. They follow the active editor, and show nothing for a project whose server hasn't started: filling them never starts one.

@available: next
**This page** says what's known about the active page, in three sections. **About** gives its title, its content type, its address, and its Markdown version's address, or "not published" when the project publishes none ([`agents = true`](../content-model/output.md#consumer)); a fragment's address is only where it's included. **Connections** lists what links to the page, grouped by file, under **Linked from**, and what includes it under **Included by**. With the cursor under a heading, Connections narrows to that heading's links and the includes of its section, whichever page they reach it through, and its title names the heading. **Memos** lists the [memo files](index.md#memo-files) about the heading, or about the page outside any heading, including those about a block in it, each by its first line with its file beside it, marked personal or outdated when it is. Each item opens the link, the include, or the memo. Unsaved edits count.

- **Pages** has **New Page** in its title bar ([Start a page](index.md#start-a-page)), and lists the project's pages by content type, each by its title with its path beside it, then its fragments, each with the files that include it, then its **orphans**: pages that no other file links to or includes, apart from index pages (`index.md`). Ascribe doesn't know your site's navigation, so an orphan may still be reachable from a menu: it's a hint, not an error.
- **Content model** lists the project's phrases, features, glossary terms, dimensions, note types, widgets, and builds, each with how many places use it, apart from builds, which pages don't name, and glossary terms with `match = "marked"`. An entry nothing uses is marked **unused**, a candidate to remove. Clicking an entry opens its declaration in `ascribe.toml`. The counts are what **Find All References** lists for the same entry.

Pages and Content model update when you save a file of the project, when a page is added or deleted, and when you switch to another project's file; This page updates as you move between pages and headings, while it's open.

### The status bar
@available: ascribe 0.2.0

While a page or an `ascribe.toml` is the active editor, the status bar names its project and the build you're looking at, such as **docs · site**, with an icon for its language server: a book while it runs, a spinning arrow while it starts, and a warning when it failed. Its tooltip gives the project's folder, its `ascribe.toml`, the `ascribe` binary in use, and the server's state. While the project has several builds and the Problems panel holds the problems of the editor build alone, it adds what the panel covers, such as **docs · site · problems: this build only**, or **docs · cloud · problems: site only** while you look at another build; once you [check every build](settings.md#the-problems-panel), and until something changes, it doesn't. A project's language server starts when one of its pages opens, so until then nothing has checked it: while no page of a project is active, the status bar says so, such as **docs: not checked yet**, or **2 projects: not checked yet**, and its tooltip names them. Otherwise a file that isn't in a project shows nothing.

In Restricted Mode, Ascribe runs nothing, since it would run the project's own `ascribe` binary. The status bar shows **Ascribe: not checking**, and the Problems panel's "No problems" doesn't mean the project is clean. Click it to manage the folder's trust; once you trust it, Ascribe starts.

Click it for a menu of the project: **Switch build**, **Check every build**, **Show output**, **Restart server**, and, for a page, **Dim what the build leaves out** and **Open preview**.

Each project has one **build you're looking at**, for the rest of the session: the preview renders it, the [build lens](index.md#the-build-lens) dims by it, and the status bar names it. It starts as the editor build. **Switch build** (also **Ascribe: Switch Build**), which lists the project's builds and marks the editor build, and the preview's **Build** picker both change it, and in a workspace with several projects, each project keeps its own. Picking the editor build follows `[editor] build`, even when you change it. The editor build still decides the diagnostics: to change it, change `[editor] build` in `ascribe.toml`.

@available: ascribe 0.3.2
A [versioned](../versions/index.md) build's lines are listed under it, the current line first, such as `site@3.4` for line 3.4: the preview of a line is at its address, and the lens dims what that line leaves out, a page new in a later line whole.

### The build lens
@available: ascribe 0.3.0

**Ascribe: Dim What the Build Leaves Out** (also in the [status bar](index.md#the-status-bar) item's menu) turns the build lens on for the active file's project. The editor then dims what the build you're looking at leaves out of each page: the variant arms the build doesn't select, and the sections, blocks, and table rows its availability filter removes, from the directive line through `@end`. Hover over dimmed text for why, such as **Left out of self-hosted.** Shows only edition=self-hosted. When the build doesn't publish the page at all, a line at its top says so, and the whole page is dimmed. "What will self-hosted readers see?" is answered in the source, without opening the preview.

The lens has no build of its own: it dims by the [build you're looking at](index.md#the-status-bar), so with the preview open the two always show the same build. While the lens is on, the status bar shows an eye, such as **docs · self-hosted**. It follows your edits, unsaved ones included, and changes to `ascribe.toml`. What's dimmed is what `ascribe build` leaves out, decided the same way. Content an `@include` brings in is in another file and isn't shown. Dimming changes nothing in the text, its folding, or its problems. Run the command again, or pick **Stop dimming what the build leaves out**, to turn it off; it stays on for the project until then, or until the window closes.

### Review in the preview
@available: ascribe 0.2.0

The preview can mark what changed against a git revision, the **base**, as [`ascribe diff`](../review/diff.md#cli-diff) reports it: added and changed blocks with a bar and a label, the changed words highlighted, removed blocks where they were, and moved blocks linked to where they came from. A tab's label and a `details`' summary say what changed in what they can hide: "new" for an added tab, else how many changes. It's for reading a change as a page, your own before you push it or someone else's on a checkout of their branch. [Review](../review/index.md) walks through reviewing a pull request, start to finish; this section and the next are the reference for the editor's part.

**Ascribe: Start Review** turns it on for the active page's project. It asks for the base: the base of your branch's pull request, when it has one, the default branch (the first of `origin/HEAD`, `origin/main`, `origin/master`, `main`, and `master` that exists), or a branch, tag, or commit you type. Either way it compares with the point where your branch left it, as a pull request does. The status bar shows **Review: off**, or the base, while an Ascribe page or the preview is active; click it to start review, or to list the changed pages once it's on. While review is off, the preview has no review header and looks as it always does; its title bar has a **Start Review** button, which becomes **Changed Pages** once review is on. Review stays on for the project until **Ascribe: Stop Review**, or until the window closes. The tooltips on the base, in the header and the status bar, give the commit compared with. When that point moves (you pull, rebase, or fetch a newer base branch), review follows it the next time the preview regains focus, and says so.

With review on, the preview's header shows the base, how many changes the page has (click it for the breakdown), **Changes / As it will be / As it was**, and next and previous change, with your place: "3 of 10 on this page". Past the last change, it offers the next changed page. A page that changed only through something it uses, such as a fragment, says so and links to the file. When the project has errors, the header says how many, since a page with an error may not show as it will once it's fixed, with **Show problems** to open the Problems panel. The count is the Problems panel's, which checks pages for the editor's build only. Clicking a mark's label opens the block's source with its lines selected. The marks follow your edits as you type, saved or not.

**Ascribe: Changed Pages**, also the list button in the preview's title bar, lists the pages the change touches in the preview's build, by title, each with its counts and, when its own file didn't change, what it changed through. A title with [code](../content-model/index.md#53-code-in-a-field) shows it between backticks, as in "`` `loom.yaml` options ``"; the preview's heading shows it as code. Choosing one opens it and its preview, or, for a removed page, the preview of the page as it was.

#### The Review view
@available: next

While review is on for the active page's project, a **Review** view is first in the Ascribe sidebar, and it's gone once review stops. It lists the same pages as Changed Pages, the ones [`ascribe diff`](../review/diff.md#cli-diff) lists for the same base, under **Added**, **Changed**, and **Removed**: each by its title, with its counts, what it changed through when its own file didn't change (such as "via _fragments/requirements.md"), and how many open threads show on it, such as "3 changed · 1 open thread". Choosing a page opens it and its preview at its first change; a removed page has no file to open, so choosing it shows the page in the preview as it was at the base, with the review header saying so and no **As it will be** to switch to. Each page's diff button opens its own source changes: its file and the files it changed through. The pages follow your edits, saved or not.

**Problems on these pages**, the view's last group, lists the errors and warnings VS Code has for the changed pages' files, and nothing else; the eye button in the view's title bar adds advice. They're every problem on those pages, whether the branch made it or it was there before. Each opens its place.

The view's badge counts the comments you haven't submitted. Its title bar has **Submit Review…** while there are some, which opens the preview's submit dialog, **Open Source Changes**, and **Stop Review**.

**Ascribe: Open Source Changes** opens VS Code's diff of every changed source file at once: each changed page's file, and the fragments, pages, and `ascribe.toml` they changed through, each at the base against the working tree. A snippet's code file isn't among them. The base side comes from VS Code's Git extension, or from Ascribe's language server where the Git extension is off. VS Code's diff may repeat file headers past 30 files, so on a large branch a page's own diff button is the better way through.

While review is on, **This page** has a **Review** section between About and Connections: whether the branch changes the page, with its counts and what it changed through, or "this branch doesn't change it", and **Open threads**, the pull request's unresolved threads on the page or on a file it includes. Choosing a thread opens the page's preview at it.

### Comments in the preview
@available: ascribe 0.2.0

When your branch has an open pull request on GitHub, review shows its review threads beside the blocks they're on, and lets you review the change there. **Start Review** looks for the pull request first, and then offers its base as the first choice. The header names the pull request ("#128 against main"); the number opens it on GitHub, and **Comments (N)** lists every comment on the pull request's pages.

- **Reading.** Each thread sits in a column beside its block, highlighted with a line to the block when you point at or focus either. A narrow preview shows a count on each block instead, which opens its threads. Resolved threads are collapsed. A tab's label and a `details`' summary say how many comments are inside, and opening a thread inside a tab that isn't showing, or a closed `details`, shows it first. **Outdated** and **Detached** threads are described in [How comments map to the pull request](../review/index.md#how-comments-map-to-the-pull-request).
- **Commenting.** Point at or focus any block and choose **Comment**. The comment is unsent until you submit; when GitHub can't take it on the block's lines, the box says so before you write, and it goes in your review's summary. A block with unsaved changes can't take a comment until you save its file, since GitHub counts the file's saved lines; threads stay beside their blocks as you type.
- **Replying.** **Add to review** holds a reply with your other comments. **Reply now** sends it at once, except while you have unsent comments, when it's disabled and says why.
- **Resolving.** **Resolve** and **Reopen** act on GitHub at once.
- **Prompting your agent.** **Prompt agent** on a thread, **Prompt agent: all open** in the list of comments, and **More review actions** (**⋯**) in the header, for the page's changes or a fragment's pages, build a prompt for your agent and deliver it as `ascribe.agents.promptTarget` says. In the source editor, a thread's title bar has **Prompt Agent**. See [In review](../agents/index.md#in-review).
- **Submitting.** While you have unsent comments, a bar at the bottom counts them, with **Submit review…**: it lists them, and submits them as a comment, an approval, or a request for changes, with an optional summary. **Discard…** in the same dialog deletes them, after asking.

The same threads show on their lines in the source editor, where you can reply, resolve, and comment on any line into the same review. The GitHub Pull Requests extension shows them there already, so by default Ascribe leaves the source editor to it when it's active; `ascribe.review.sourceComments` decides.

Comments need GitHub. Review asks to sign in to GitHub in VS Code when you start it, never sooner; the permission it asks for (`repo`) is the one posting review comments needs. If you decline, review shows the changes only, and the header offers **Sign in to see comments**, and **Use GitHub CLI** when `gh` is signed in. Once you choose the GitHub CLI, review uses it in that workspace from then on, until you choose **Sign in to see comments**. The preview itself never holds the sign-in or makes a request: everything goes through the extension.

The comments are read when review starts, on **Ascribe: Refresh Comments** (the refresh button in the preview's title bar), and after your own actions. When your checkout isn't the pull request's latest commit, the header says so: behind, with **Pull**, since some comments may be on lines you don't have; ahead, with **Push**, since you can comment only on lines that are on GitHub.

Review runs `git` once you start it. Before that, only the offer does: once you've used review in a workspace, opening a page runs `git` to find its branch, and asks GitHub whether the branch has an open pull request, to offer review ([Review](../review/index.md) describes the offer). A project that isn't in a git repository, a revision that doesn't exist, or `git` missing from the path stops it from starting, with a message that says which; the preview works as before. Starting review reads the project as it was at the base, which takes about as much memory again as the project; stopping review frees it.

## Other editors

Any editor with a Language Server Protocol client can run `ascribe lsp` from a project installed with `@ascribed/cli`. It speaks LSP over standard input and output. The preview is a VS Code feature.

A server serves one project: the nearest `ascribe.toml` at or above the workspace folder the editor gives it. It never looks below that folder, so a server started at the root of a repository whose projects are all in subfolders has no project. For a workspace with several projects, start one `ascribe lsp` per project, rooted at the folder that holds its `ascribe.toml`, and give each file to the server of the nearest `ascribe.toml` above it. In Neovim, for example, `root_markers = { "ascribe.toml" }` in the server's `vim.lsp.config` does both.

The lens in a code file a page shows and the action that applies a page's safe fixes come from the server, so any editor that asks gets them. The server asks for lenses in its sources' files by registering them for those files (`client/registerCapability` for `textDocument/codeLens`), when the editor supports registering lenses; one that doesn't gets the lenses on `@include` in the pages it sends. The lens's command, `ascribe.showReferences`, is the editor's to run, with a file, a position in it, and the places, as VS Code's `editor.action.showReferences` takes them. The action's kind is `source.fixAll.ascribe`.

When it starts, the server sends a `window/logMessage` at Info with its version and the project it uses: `ascribe-lsp <version>: using the project at <path>`, or, when it finds none, `ascribe-lsp <version>: no ascribe.toml at or above <folder>`. Its other log lines go to standard error. A server with no project reports nothing. If an `ascribe.toml` is then created in the workspace folder itself, the server loads it, when the editor supports watching files for the server.

When one project is nested in another's content root, the outer project's server leaves the nested project's files alone, open or not, so each file gets diagnostics from one server only. A nested project's `ascribe.toml` created or deleted while the server runs changes which files it checks, when the editor supports watching files for the server.

## Other formatters

A Markdown formatter that reflows paragraphs, such as Prettier with `proseWrap: always`, doesn't know that title and directive lines start new blocks, and would join them with the text below. Exclude Ascribe sources from it, for example with `**/*.md` in `.prettierignore`, and use `ascribe fmt`. In VS Code, the extension offers to be the Markdown formatter for your pages; see [Formatting](index.md#formatting).
