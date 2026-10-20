---
type: reference
title: The Ascribe sidebar
description: "The Ascribe sidebar in VS Code: This page, Pages, Content model, and switching the project they show."
available: ascribe 0.3.0
---

The Ascribe icon in the activity bar opens the Ascribe sidebar, with three views: **This page**, **Pages**, and **Content model**. In a workspace with no project, Pages says how to start one, and in Restricted Mode, how to trust the folder.

@available: ascribe 0.4.0
While review is on, a **Review** view comes first: see [The Review view](review.md#the-review-view).

## This page, Pages, and Content model

The views show the active file's project, named beside Pages' and Content model's titles. They follow the active editor, and show nothing for a project whose server hasn't started: filling them never starts one. When a project's server fails to start, Pages and Content model say so, with **Show Output** and **Restart**.

@available: ascribe 0.4.0
**This page** says what's known about the active page, in four sections. **About** gives its title, its content type, its address, and its Markdown version's address, or "not published" when the project publishes none ([`agents = true`](../content-model/output.md#consumer)); a fragment's address is only where it's included. **Health** says how the page stands. **Connections** lists what links to the page, grouped by file, under **Linked from**, what includes it under **Included by**, and what it includes under **Includes**. With the cursor under a heading, Connections narrows to that heading's links and the includes of its section, whichever page they reach it through, and its title names the heading; Includes stays the page's. **Memos** lists the [memo files](actions.md#memo-files) about the heading, or about the page outside any heading, including those about a block in it, each by its first line with its file beside it, marked personal or outdated when it is. Each item opens the link, the include, or the memo. Unsaved edits count.

About's other rows say whether the page is published, listed, due, and finished. Each is shown only when it has something to say, apart from a description, which is shown "missing" where [`page-description-missing`](../checks/diagnostics/content-checks.md#ASC150) reports the page. A row with somewhere to go opens it:

| Row | Says | Opens |
|---|---|---|
| **Description** | Its length, or its field's default's when the page leaves the field out or empty, against the 300 characters its line in `llms.txt` reads best under when the project publishes one, with a warning only where [`description-too-long`](../checks/diagnostics/content-checks.md#ASC163) reports it, or "missing" | The field in the frontmatter |
| **Review** | "overdue since 3 October" or "due in 12 days", from the field with the [`review-date` role](../content-model/index.md#54-field-roles), as [`review-overdue`](../checks/diagnostics/content-checks.md#ASC153) compares it with today, with a warning only where that check reports the page | The field |
| **Availability** | The page's own `available` | Nothing |
| **Builds** | "Published by site, agents; left out by cloud": the builds that publish the page and the ones that leave it out, with why in its tooltip | **Switch Build**, to see the page as a build publishes it; **Dim What the Build Leaves Out** shows what that build leaves out of it |
| **Listing** | "Listed (the default)" or "Unlisted, by `[listing.folders]`": what [`pages.json`](../content-model/checks.md#22-listing) says of the page, and the rule that sets it | The page's own `listing`, or the rule in `ascribe.toml` |
| **Permalink** | A [permalink](../permalinks/index.md) the page or one of its headings declares, one row each | Its `permalink` or its `@id` line |
| **Template** | "how-to template, 2 placeholders left": what its [type's template](../templates/index.md#whats-left-to-fill-in) still leaves to fill in | **Go to Next Placeholder** |

**Health** has these rows:

| Row | Says | Opens |
|---|---|---|
| **Problems** | "1 warning, 2 advice": VS Code's problems for the page, as the Problems panel counts them | The Problems panel, where its filter's **Show Active File Only** keeps it to the page |
| **Size** | "31,200 of 50,000 characters, in site": the page's Markdown in the build where it's largest, measured as [`page-size`](../content-model/checks.md#19-checks) measures it, shown always so a page is seen growing toward the limit | Nothing; its tooltip says which build and the limit |
| **Example** | Each [tested example](../examples/tested.md) a `@snippet` of the page shows: tested, changed since it was tested, or not tested | Its `@snippet` line |

A fragment has no builds, listing, or size of its own: it's published only where it's included.

- **Pages** has **New Page** in its title bar ([Start a page](actions.md#start-a-page)), and shows the project's [navigation](../navigation/index.md): **Start**, the content root's section, with the home page first, then each section in order, each by its label with its pages by theirs, a page's [`nav-label`](../navigation/index.md#a-page-s-label) or else its title, and their paths beside them, then the project's fragments by folder, each with the files that include it. A page's tooltip counts the links and includes that reach it. Its sections start closed, and opening a page selects it in Pages, opening the sections above it, so the open page's path is what you see. It's the easy way to change the order, which it writes to `ascribe.nav` as typing does: the file opens with the change unsaved, and **Undo** takes it back, the file too when the drag made it.
  - **Drag a section** onto another beside it, and it's put before it when you drag it up and after it when you drag it down. The sections beside it are listed in the file in the order they had, so nothing else moves.
  - A section whose own page sits beside its folder, such as `checks/diagnostics/` with `checks/diagnostics.md`, is drawn where that page sits among its parent's pages, and dragging it moves that page in the parent's `pages`. At the top level there's no such placing: `guides/` with `guides.md` is drawn after Start, with the page first in it, and is dragged among the top sections like any other.
  - **Drag a page** onto another of its section the same way, or onto its section to put it first. The section's `pages` lists its pages as they stood, with the one you moved where you put it.
  - **Drag a page onto another section**, or a page of one, and it asks first, since the page's address changes. On **Move**, its file moves to that section's folder, and the links to it and `ascribe.nav` follow, as they do when you move a file in the Explorer.
  - A page `ascribe.nav` leaves out of a section that lists its pages is at the end of its section, marked **not placed**; **Place Here** in its menu adds it to the end of the section's `pages`. A page that's `unlisted` is in no sidebar, so it's marked **unlisted** instead, and needs no place.
  - **Rename Label…** in a section's menu sets its `label` in `ascribe.nav`, and in a page's menu its `nav-label`, in its frontmatter, unsaved as a drag's change is. It offers the label as it's written; leave it empty for the default, which for a page is its title.
  - A page or fragment with a problem is colored as it is in the Explorer: red for an error, yellow for a warning.
  - A page the build the status bar shows leaves out is dimmed, and says "Not in cloud" for the build `cloud`, as the [build lens](preview.md#the-build-lens) dims text. It follows the build as you switch.
  - A page's menu has **Open Preview to the Side**, also its button; **Copy Link**, which copies its path from the content root, such as `/guides/install.md`, the link a page writes to it (its published address is This page's **Address** row); **Rename or Move…**, which asks for its new path and moves it as the Explorer does, the links to it following; **Find All References**, the links and includes that reach it, in a peek at its start; and **Reveal in Explorer**. A fragment's menu has the last three.
  - **New Page Here**, in a section's menu, starts a page in its folder ([Start a page](actions.md#start-a-page)).
  - **Orphans Only**, the filter button in the title bar, lists only the pages nothing links to or includes, apart from index pages, and the fragments nothing includes, each in its section or folder. Choose it again to list everything.
- **Content model** lists the project's page types, phrases, features, glossary terms, dimensions, note types, widgets, and builds, each with how many places use it, apart from builds, which pages don't name; a page type, with how many pages have it. An entry nothing uses is marked **unused**, a candidate to remove. Each dimension lists its values. Clicking an entry or a value opens its declaration in `ascribe.toml`, or a glossary term's heading on its page. The counts are what **Find All References** lists for the same entry.
  - **Find All References**, in the menu of an entry pages use, lists those uses in a peek at its declaration. On a page type, it lists the pages whose `type` names it, at that value.
  - **Rename…**, on a phrase or a dimension's value, asks for the new name and makes the edits `F2` on it in a page makes, shown in the refactor preview first.
  - **Insert at the Cursor**, on a phrase or a widget, runs **Insert a phrase** or **Insert a widget** with it chosen, at the cursor of the project's last Markdown editor, which comes to the front. Where the cursor is somewhere the action doesn't apply, it says where to put it and changes nothing.
  - **Unused Only**, the filter button in the title bar, lists only the entries nothing uses.

Pages and Content model update when you save a file of the project, when a page is added or deleted, Pages as you edit `ascribe.nav` too, and when you switch to another project's file; This page updates as you move between pages and headings, while it's open.

## Switch Project
@available: ascribe 0.4.0

In a workspace with several projects, **Ascribe: Switch Project** chooses the project the sidebar shows. It's in the Command Palette, in Pages' and Content model's title bars, and in the [status bar](preview.md#the-status-bar) item's menu; with one project, it's in none of them. It lists every project, the sidebar's first and marked, each with its folder, its server's state, such as "running, site build", "not checked yet", or "failed", and the `ascribe` binary in use. Each has buttons to show its server's output, and to restart its server when it's running or failed, and **Refresh** in its title asks the running servers for their builds again. Listing them starts no server.

Choosing a project shows it in the sidebar and starts its server if it hasn't started. It's a jump, not a pin: the sidebar stays on it until a file of another project becomes active, and then follows that file as before.
