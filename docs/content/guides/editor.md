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

### The walkthrough
@available: next

After you install the extension, VS Code opens its walkthrough, **Get Started with Ascribe**, which goes through what a writer does first: open a project, preview a page, use the actions bar, look at the Ascribe sidebar, see what a build leaves out, and check the project in CI. To see it again, open **Help → Welcome**, or run **Welcome: Open Walkthrough…** and choose it. Its buttons act on the page you last had open.

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

### Prompt agent

@available: next
On a problem, the lightbulb also offers **Prompt agent: fix this problem**, after the problem's quick fixes. It builds a prompt for your AI agent about the problem, and copies it, or opens your agent with it filled in, as `ascribe.agents.promptTarget` says. **Ascribe: Prompt Agent to Fix This File** and **Ascribe: Prompt Agent to Fix This Project** do the same for every problem in the file or the project. Nothing is sent: you read the prompt and send it. See [Prompt your agent](agents.md#prompt-your-agent).

### Agents in VS Code

@available: next
An agent in VS Code, such as GitHub Copilot in agent mode, gets Ascribe's tools with nothing to set up. In a workspace with an Ascribe project, the extension offers [`ascribe mcp`](agents.md#the-mcp-server) as an MCP server, named **Ascribe** in VS Code's list of MCP servers. It runs the binary the first project uses (see [Which `ascribe` it runs](#which-ascribe-it-runs)), in the workspace folder, and VS Code starts it when an agent first needs it. When the workspace's projects use different binaries, the project's output says which one the server runs.

The extension also gives agents three tools of its own, for what only the editor knows: text that isn't saved, and the review that's open. Name one in chat with `#`, or let the agent pick it.

@include: ../_generated/editor-tools.md

`#ascribe_editor_problems` answers with the JSON `ascribe check --format json` writes, from the language server, so it includes unsaved edits. When a file changed since the server last checked it, the tool waits for the server, up to a second, and says which version of the document its answer is for. Like the Problems panel, it covers the [editor's build](#diagnostics) only; `ascribe check` checks every build. It never starts a language server: a project whose server isn't running gets a line saying so.

The review tools answer while [review](#review-in-the-preview) is on, and say so in one line when it's off; they never turn it on. `#ascribe_review_threads` gives each open thread's comments as data, under the same sentence a [prompt](agents.md#what-a-prompt-says) puts over them, so the agent doesn't take them as instructions. None of the tools changes a file.

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
- **Find All References** (`Shift+F12`, or **Go to References**) lists every place that uses what's under the cursor: on a heading or its `@id`, the links to it, including links through a page that includes it, and the includes of its section; at the start of a page, or on its frontmatter or title, the links and includes from other files; on a link or include, those of what it names; on a phrase, a feature key, a glossary term, a note's type, a widget, or a `@variant` attribute, every use of that phrase, feature, term, note type, widget, or dimension. A glossary term counts where its text, or an alias, is in prose, whether or not the build links it there. A term with `match = "marked"` is only a word until you link it, so it has no uses of its own: use **Find All References** on its page or heading instead.
- **Document links**: links, images, and includes are clickable.
- **CodeLens** above each `@include` names the included file and section, and opens it.
- **Inline hints** show the title an empty-text link, `[](keys.md#rotate-keys)`, will get.

### Refactoring

- **Rename or move a file** in the Explorer, and the links and includes that point to it are updated.
- **Rename** (`F2`) a heading's `@id` to update the links to it, or a phrase key to update its uses and its declaration.
- **Rename** (`F2`) a dimension value, in a `@variant` attribute or in its dimension's `values` in `ascribe.toml`, to update it everywhere: the dimension's `labels` and `versionless`, every `@variant` attribute and `variant:` field, every availability spec in pages, table rows, frontmatter, and `[features]`, and the builds' `variants` and `filter`.
- A phrase rename reaches every use: prose, link text and destinations, frontmatter fields with `phrases = true`, code blocks with `phrases=true`, and the files a `@snippet {phrases=true}` shows.
- `F2` on something that can't be renamed says why, and on something that can, selects what will be renamed. A new name the content model doesn't allow, or that's already taken, changes nothing.

### Formatting

**Format Document** rewrites Ascribe constructs into canonical form, as `ascribe fmt` does. Turn on `ascribe.formatOnSave` to do it on every save. It changes only Ascribe constructs, never how a page renders.

### Highlighting

Directive lines, attribute blocks, `@end`, and phrases are highlighted as soon as a file opens. Once the server is running, it adds what depends on the content model: declared and undeclared phrases, project widgets, and **title lines**, shown distinctly so a paragraph that accidentally became a title stands out.

### Preview

**Ascribe: Open Page Preview to the Side** (also the preview button in a Markdown editor's title bar) shows the current page as the published site shows it, with the same elements, including unsaved changes. It follows the editor: it updates as you type, keeps its scroll position, and opens a page or file you click in the editor. It scrolls with the editor by block, both ways: scrolling the editor brings the block at its top to the top of the preview, moving the cursor into a block the preview doesn't show brings it into view, scrolling the preview scrolls the editor to the block at its top, and double-clicking a block puts the cursor on its first line. A block from a fragment follows the line of its `@include`. Turn off `ascribe.preview.scrollPreviewWithEditor` or `ascribe.preview.scrollEditorWithPreview` to stop one direction. Its **Build** picker shows the page as any build publishes it, starting with the editor's build: it changes the [build you're looking at](#the-status-bar), which the status bar names and switches too.

A fragment isn't a page, so the preview names the pages that include it instead. A page a build doesn't publish says which build drops it, and why.

A file with no `ascribe.toml` above it has nothing to preview: "This file isn't part of an Ascribe project (no ascribe.toml above it)." A file in a project's folder but outside its content root isn't a page either, and the preview names both: "handbook/README.md is in the project handbook, but outside its content root (handbook/pages), so there is no page to preview."

The preview shows images and files from the content root, and from directories elsewhere in the project that a page uses. For safety, it runs no inline scripts and loads nothing remote, so raw HTML that needs either looks different in the preview than on the site.

@available: next
A [`@snippet`](../reference/directives.md#snippet) is read again when its page changes, not when its code file does, so the preview and the page's problems don't follow edits to the code. To see them, change the page, or run **Ascribe: Restart Language Server**.

### Site preview
@available: next

The page preview is the page alone. The **site preview** is the same page in the real site, with its layout, navigation, and styles, from the site's dev server. With [`@ascribed/astro`](astro.md), `astro dev` writes where it's running to `.ascribe/dev.json` in the project, and removes the file when it stops.

**Ascribe: Open Site Preview** opens the active page on the dev server in your browser, at the heading the editor shows. It's also the globe button in a Markdown editor's title bar while a dev server is running. With no dev server, or one that stopped without removing `dev.json`, it says to start one. It only opens an address on your machine (`localhost`, `127.0.0.1`, or `[::1]`), so a `dev.json` committed to a repository can't send it elsewhere. In a remote workspace, VS Code forwards the dev server's port first. The page preview's **Page | Site** switch, at the end of its toolbar, shows the site preview in the panel instead: the dev server's page for the same file, following the active file as **Page** does. Links you follow inside it stay until you open another file. With no dev server, it says to start one, with **Try again**. In VS Code for the Web, where the panel can't show the dev server, **Site** opens the browser instead. [Review in the site preview](astro.md#review-in-the-site-preview) has the toolbar app that marks changes and shows comments there; each block's and thread's **Open source** brings you back to the file, at the line.

### Actions
@available: next

Actions write Ascribe for you: wrap a paragraph in a note, turn a numbered list into steps, link the selected text to a page, insert content that varies by a dimension, mark where a section is available. Each one asks what it needs in short steps, choosing from what the project has (its note types, pages and headings, phrases, dimensions and their values, features, images, fragments, sources, and widgets), so you never type a path, a key, or attribute syntax. What it writes is in canonical form, and one **Undo** takes it back.

Every action is in four places:

- The **actions bar** (`Ctrl+K A` / `Cmd+K A`), which lists only what applies where the cursor is. See [The actions bar](#the-actions-bar).
- The **Command Palette**, as `Ascribe: <action>`, in a Markdown file of a project. Run where it doesn't apply, it says where it does: "Put the cursor in a note to change its kind."
- The editor's context menu, under **Ascribe**, which lists the actions that apply where the cursor is.
- The lightbulb (`Ctrl+.` / `Cmd+.`), for the actions that rewrite what's at the cursor, beside the quick fixes.

@include: ../_generated/editor-actions.md

#### The actions bar

Press `Ctrl+K A` (`Cmd+K A` on macOS), or run **Ascribe: Actions for the Cursor**, in a page of a project to see what you can do where the cursor is. The bar lists the fixes for problems at the cursor first, then the actions that apply to the cursor or the selection, grouped as **Write**, **Structure**, **Link**, **Media**, and **Content model**. Each shows what it does and the syntax it writes. Type to filter, and choose one: an action that needs something asks for it in the same box. When nothing applies, the bar says where to put the cursor.

The bar holds only what applies to the cursor or selection, so it never lists every action as the Command Palette does, and it has no searches or commands of its own.

To use another key, open **Preferences: Open Keyboard Shortcuts**, search for `ascribe.actions`, and change its keybinding. The default is on only in a Markdown file of a project, with the editor focused.

#### Changing the content model

The actions in the bar's **Content model** group change `ascribe.toml` from a page: **Make this a phrase**, **Add to the glossary**, **Change a feature's availability**, **Rename this phrase everywhere**, and **Rename a dimension value everywhere**. A rename changes what `F2` does (see [Refactoring](#refactoring)), for the phrase or value at the cursor, or one you pick from the project's wherever the cursor is.

These change `ascribe.toml` in place, keeping its comments, blank lines, and order, and an open `ascribe.toml` with unsaved changes is edited as it is. A rename that changes other files lists every change in the refactor preview first, where you check the changes to make and **Apply** them, or **Discard** them all.

### The Ascribe sidebar
@available: next

The Ascribe icon in the activity bar opens the Ascribe sidebar, with four views: **Projects**, **Used by**, **Pages**, and **Content model**.

#### The Projects view

The **Projects** view lists every project in the workspace, started or not, by name, with its folder and an icon for its server's state. Under each is its `ascribe.toml`, which opens it, and, while its server runs, its editor build and the `ascribe` binary in use. Each project's buttons show its server's output, and restart its server when it's running or failed. The view updates as projects come and go and servers start and stop, and opening it starts no server. In a workspace with no project, it says how to start one.

#### Used by, Pages, and Content model

The other three views show the active file's project. They follow the active editor, and show nothing for a project whose server hasn't started: filling them never starts one.

- **Used by** lists what links to the active page, grouped by page, then what includes it. For a fragment, that's the files that include it. With the cursor on a heading, it narrows to the links to that heading, whichever page they reach it through. Each item opens the link or include.
- **Pages** lists the project's pages by content type, each by its title with its path beside it, then its fragments, each with the files that include it, then its **orphans**: pages that no other file links to or includes, apart from index pages (`index.md`). Ascribe doesn't know your site's navigation, so an orphan may still be reachable from a menu: it's a hint, not an error.
- **Content model** lists the project's phrases, features, glossary terms, dimensions, note types, widgets, and builds, each with how many places use it, apart from builds, which pages don't name, and glossary terms with `match = "marked"`. An entry nothing uses is marked **unused**, a candidate to remove. Clicking an entry opens its declaration in `ascribe.toml`. The counts are what **Find All References** lists for the same entry.

Pages and Content model update when you save a file of the project, when a page is added or deleted, and when you switch to another project's file; Used by updates as you move between pages and headings, while it's open.

### The status bar
@available: next

While a page or an `ascribe.toml` is the active editor, the status bar names its project and the build you're looking at, such as **docs · site**, with an icon for its language server: a book while it runs, a spinning arrow while it starts, and a warning when it failed. Its tooltip gives the project's folder, its `ascribe.toml`, the `ascribe` binary in use, and the server's state. A file that isn't in a project shows nothing.

Click it for a menu of the project: **Switch build**, **Show output**, **Restart server**, and, for a page, **Dim what the build leaves out** and **Open preview**.

Each project has one **build you're looking at**, for the rest of the session: the preview renders it, the [build lens](#the-build-lens) dims by it, and the status bar names it. It starts as the editor build. **Switch build** (also **Ascribe: Switch Build**), which lists the project's builds and marks the editor build, and the preview's **Build** picker both change it, and in a workspace with several projects, each project keeps its own. Picking the editor build follows `[editor] build`, even when you change it. The editor build still decides the diagnostics: to change it, change `[editor] build` in `ascribe.toml`.

### The build lens
@available: next

**Ascribe: Dim What the Build Leaves Out** (also in the [status bar](#the-status-bar) item's menu) turns the build lens on for the active file's project. The editor then dims what the build you're looking at leaves out of each page: the variant arms the build doesn't select, and the sections, blocks, and table rows its availability filter removes, from the directive line through `@end`. Hover over dimmed text for why, such as **Left out of self-hosted.** Shows only edition=self-hosted. When the build doesn't publish the page at all, a line at its top says so, and the whole page is dimmed. "What will self-hosted readers see?" is answered in the source, without opening the preview.

The lens has no build of its own: it dims by the [build you're looking at](#the-status-bar), so with the preview open the two always show the same build. While the lens is on, the status bar shows an eye, such as **docs · self-hosted**. It follows your edits, unsaved ones included, and changes to `ascribe.toml`. What's dimmed is what `ascribe build` leaves out, decided the same way. Content an `@include` brings in is in another file and isn't shown. Dimming changes nothing in the text, its folding, or its problems. Run the command again, or pick **Stop dimming what the build leaves out**, to turn it off; it stays on for the project until then, or until the window closes.

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
- **Prompting your agent.** **Prompt agent** on a thread, **Prompt agent: all open** in the list of comments, and **More review actions** (**⋯**) in the header, for the page's changes or a fragment's pages, build a prompt for your agent and deliver it as `ascribe.agents.promptTarget` says. In the source editor, a thread's title bar has **Prompt Agent**. See [In review](agents.md#in-review).
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

A project's server starts the first time you open one of its Markdown files or its `ascribe.toml`, or preview one of its pages. It also starts when one of its files changes on disk while the window is open, so an agent or a script that writes the project's files without opening them still gets their problems in the Problems panel. A file counts if it's the project's `ascribe.toml`, is under its content root (not in a project nested there), or is in the folder of one of its [sources](../reference/content-model.md#18-sourcesname), and isn't in its output directory, `node_modules`, or `.git`. Changes are gathered for up to a second, so a `git checkout` that touches hundreds of files starts each project's server once. Opening the window starts nothing, and neither does adding an `ascribe.toml`. A server runs until you close the window or delete its `ascribe.toml`. A project nothing opens or changes costs nothing.

To start every project's server when the workspace opens, set `ascribe.startServers` to `"all"`.

An `ascribe.toml` you add or delete is picked up as you work. If the editor misses a new one, opening or switching to one of the new project's files picks it up. The extension serves at most 50 projects in a workspace: the first 50 by path. When there are more, the first project's output says so.

Each server uses about 6 MB of memory, plus about 70 KB per page of its project.

### The Problems panel

The Problems panel lists diagnostics only for the projects whose server is running. A project whose files you haven't opened or changed shows nothing there, even if it has problems. Open one of its files, or set `ascribe.startServers` to `"all"`, to see them.

A running server reports the problems in each file, and the problems of its pages as the editor's build (`[editor] build`) has them. A problem only another build has, such as a link to a page that build leaves out, isn't in the Problems panel. An agent that reads the Problems panel sees the same: before calling a project clean, run `ascribe check`, which checks every build.

`ascribe check` checks a project whether or not the editor has it open. Run it in the project's folder, or from anywhere with `--config` naming the project's folder or its `ascribe.toml`:

```sh
npx ascribe check --config handbook
```

### Commands and output

Commands act on the project that owns the active file: the preview, its **Build** picker, **Switch Build**, review, and **Show Server Output**. The [status bar](#the-status-bar) names that project, and the [Projects view](#the-projects-view) lists them all. **Restart Language Server** restarts every project's server that has started. [Commands](#commands) has the details.

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
