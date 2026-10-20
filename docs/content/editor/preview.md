---
type: reference
title: The preview and builds
description: "The page preview and the site preview in VS Code, the status bar, and the build lens."
---

**Ascribe: Open Page Preview to the Side** (also the preview button in a Markdown editor's title bar) shows the current page as the published site shows it, with the same elements, including unsaved changes. It follows the editor: it updates as you type, keeps its scroll position, and opens a page or file you click in the editor. It scrolls with the editor by block, both ways: scrolling the editor brings the block at its top to the top of the preview, moving the cursor into a block the preview doesn't show brings it into view, scrolling the preview scrolls the editor to the block at its top, and double-clicking a block puts the cursor on its first line. A block from a fragment follows the line of its `@include`. Turn off `ascribe.preview.scrollPreviewWithEditor` or `ascribe.preview.scrollEditorWithPreview` to stop one direction. Its **Build** picker shows the page as any build publishes it, starting with the editor's build: it changes the [build you're looking at](preview.md#the-status-bar), which the status bar names and switches too.

A fragment isn't a page, so the preview names the pages that include it instead. A page a build doesn't publish says which build drops it, and why.

A file with no `ascribe.toml` above it has nothing to preview: "This file isn't part of an Ascribe project (no ascribe.toml above it)." A file in a project's folder but outside its content root isn't a page either, and the preview names both: "handbook/README.md is in the project handbook, but outside its content root (handbook/pages), so there is no page to preview."

The preview shows images and files from the content root, and from directories elsewhere in the project that a page uses. For safety, it runs no inline scripts, loads nothing remote, and submits no forms, so raw HTML that needs any of these looks different in the preview than on the site. What the preview's page asks the extension to do is checked first: it opens only files in the project, or ones the preview itself linked to, and only the **Pull**, **Push**, and **Fetch** git commands it has buttons for.

@available: ascribe 0.2.0
A [`@snippet`](../writing/directives.md#snippet) is read again when its page changes, not when its code file does, so the preview and the page's problems don't follow edits to the code. To see them, change the page, or run **Ascribe: Restart Language Server**.

@available: ascribe 0.3.2
A [tested example](../examples/tested.md)'s advice, `example-stale` and `example-unobserved`, follows its source's observations and `ascribe.examples.toml`: when `ascribe observe` writes an observation, or the config changes on disk, the pages that show the source's examples are checked again. An edit to the example's own file still waits for the page to change, and so does the preview's tested line.

## Site preview
@available: ascribe 0.3.0

The page preview is the page alone. The **site preview** is the same page in the real site, with its layout, navigation, and styles, from the site's dev server. With [`@ascribed/astro`](../astro/index.md), `astro dev` writes where it's running to `.ascribe/dev.json` in the project, and removes the file when it stops.

**Ascribe: Open Site Preview** opens the active page on the dev server in your browser, at the heading the editor shows. It's also the globe button in a Markdown editor's title bar while a dev server is running. With no dev server, or one that stopped without removing `dev.json`, it says to start one. It only opens an address on your machine (`localhost`, `127.0.0.1`, or `[::1]`), so a `dev.json` committed to a repository can't send it elsewhere, and it doesn't contact that address until you ask for the site preview. In a remote workspace, VS Code forwards the dev server's port first. The page preview's **Page | Site** switch, at the end of its toolbar, shows the site preview in the panel instead: the dev server's page for the same file, following the active file as **Page** does. Links you follow inside it stay until you open another file. The panel takes no messages from the dev server's page: only the extension changes what it shows. With no dev server, it says to start one, with **Try again**. In VS Code for the Web, where the panel can't show the dev server, **Site** opens the browser instead. [Review in the site preview](../astro/reference.md#review-in-the-site-preview) has the toolbar app that marks changes and shows comments there; each block's and thread's **Open source** brings you back to the file, at the line.

## The status bar
@available: ascribe 0.2.0

While a page or an `ascribe.toml` is the active editor, the status bar names its project and the build you're looking at, with an icon for its language server's state. Its tooltip gives the project's folder, its `ascribe.toml`, the `ascribe` binary in use, and the server's state. While the project has several builds and the Problems panel holds the problems of the editor build alone, it says what the panel covers; once you [check every build](settings.md#the-problems-panel), and until something changes, it doesn't. A project's language server starts when one of its pages opens, so until then nothing has checked it: while no page of a project is active, the status bar says so, counting the projects when there are several, and its tooltip names them. Otherwise a file that isn't in a project shows nothing.

![The status bar item in five states, one above another: "docs: not checked yet" with a slashed circle, before a page of the project has opened; "docs" with a spinning arrow, while its language server starts; "docs · site · problems: this build only" with a book, while it runs and the Problems panel covers the editor build alone; "docs · windows · problems: site only" with an eye, while the build lens dims by the windows build; and "docs" with a warning, when the server failed.](../images/status-bar-light.png){scheme=light}
![The status bar item in five states, one above another: "docs: not checked yet" with a slashed circle, before a page of the project has opened; "docs" with a spinning arrow, while its language server starts; "docs · site · problems: this build only" with a book, while it runs and the Problems panel covers the editor build alone; "docs · windows · problems: site only" with an eye, while the build lens dims by the windows build; and "docs" with a warning, when the server failed.](../images/status-bar-dark.png){scheme=dark}

In Restricted Mode, Ascribe runs nothing, since it would run the project's own `ascribe` binary. The status bar shows **Ascribe: not checking**, and the Problems panel's "No problems" doesn't mean the project is clean. Click it to manage the folder's trust; once you trust it, Ascribe starts.

Click it for a menu of the project: **Switch build**, **Check every build**, **Show output**, **Switch project** in a workspace with several projects, **Restart server**, and, for a page, **Dim what the build leaves out** and **Open preview**.

Each project has one **build you're looking at**, for the rest of the session: the preview renders it, the [build lens](preview.md#the-build-lens) dims by it, and the status bar names it. It starts as the editor build. **Switch build** (also **Ascribe: Switch Build**), which lists the project's builds and marks the editor build, and the preview's **Build** picker both change it, and in a workspace with several projects, each project keeps its own. Picking the editor build follows `[editor] build`, even when you change it. The editor build still decides the diagnostics: to change it, change `[editor] build` in `ascribe.toml`.

@available: ascribe 0.3.2
A [versioned](../versions/index.md) build's lines are listed under it, the current line first, such as `site@3.4` for line 3.4: the preview of a line is at its address, and the lens dims what that line leaves out, a page new in a later line whole.

## The build lens
@available: ascribe 0.3.0

**Ascribe: Dim What the Build Leaves Out** (also in the [status bar](preview.md#the-status-bar) item's menu) turns the build lens on for the active file's project. The editor then dims what the build you're looking at leaves out of each page: the variant arms the build doesn't select, and the sections, blocks, and table rows its availability filter removes, from the directive line through `@end`, each with a hover that says why. When the build doesn't publish the page at all, a line at its top says so, and the whole page is dimmed.

![The Lantern install page in VS Code with the build lens on for the windows build: the Linux and macOS install commands and the section "Start it at login" are dimmed, a hover over the macOS command says "Left out of windows. Shows only platform=windows.", the preview beside it shows the page as the windows build has it, with only the Windows command, and the status bar shows an eye and "platforms · windows · problems: site only".](../images/build-lens-light.png){scheme=light}
![The Lantern install page in VS Code with the build lens on for the windows build: the Linux and macOS install commands and the section "Start it at login" are dimmed, a hover over the macOS command says "Left out of windows. Shows only platform=windows.", the preview beside it shows the page as the windows build has it, with only the Windows command, and the status bar shows an eye and "platforms · windows · problems: site only".](../images/build-lens-dark.png){scheme=dark}

The lens has no build of its own: it dims by the [build you're looking at](preview.md#the-status-bar), so with the preview open the two always show the same build. It follows your edits, unsaved ones included, and changes to `ascribe.toml`. What's dimmed is what `ascribe build` leaves out, decided the same way. Content an `@include` brings in is in another file and isn't shown. Dimming changes nothing in the text, its folding, or its problems. Run the command again, or pick **Stop dimming what the build leaves out**, to turn it off; it stays on for the project until then, or until the window closes.
