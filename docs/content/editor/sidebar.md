---
type: reference
title: The sidebar and the status bar
description: "The Ascribe sidebar in VS Code, with its Project, This page, Pages, and Content model views, the status bar item, and switching the project they show."
---

The Ascribe icon in the activity bar opens the Ascribe sidebar, with three views: **This page**, **Pages**, and **Content model**. In a workspace with no project, Pages says how to start one, and in Restricted Mode, how to trust the folder. The views show the active file's project, named beside Pages' and Content model's titles. They follow the active editor, and show nothing for a project whose server hasn't started: filling them never starts one. When a project's server fails to start, Pages and Content model say so, with **Show Output** and **Restart**.

@available: ascribe 0.4.0
While review is on, a **Review** view comes first: see [The Review view](review.md#the-review-view).

@available: next
A **Project** view comes first of all, above Review: see [The Project view](#the-project-view).

The [status bar](#the-status-bar) item names the active file's project, with a menu of the jobs the Project view has.

## The Project view
@available: next

**Project** is about the whole project the sidebar shows, where This page is about one page. It shows the same project as the views below it, following the file you're in, and staying on its project when you click in it or open a file of no project.

![The Project view for the project quill, with the rest of the Ascribe sidebar closed: at its top the project's name, with a gear and a settings button beside it, its folder, its server "running" with buttons to show its output and restart it, the version of `ascribe` the server runs, and a Build switch showing "site, line 3.5, the current line, editor build". Health, "3 things to do", counts each thing with when it's from: no problems in every build, no tested examples changed or never tested, no reviews overdue, and nothing the permalinks list is behind on, each "Checked at 02:02 PM", the overdue reviews "for October 21"; 1 page nothing links to or includes and 2 content model entries nothing uses, "As the pages are now, unsaved changes too"; and 3 agent files out of date, "Checked at 02:02 PM, after the last save". Actions lists Check Every Build under Check; Apply Every Safe Fix, Format Every Page, Build, Sync Agent Files, and Observe Examples under Write; and Start Review and Prompt Agent to Fix This Project under Review and agents. About is closed.](../images/project-light.png){scheme=light}
![The Project view for the project quill, with the rest of the Ascribe sidebar closed: at its top the project's name, with a gear and a settings button beside it, its folder, its server "running" with buttons to show its output and restart it, the version of `ascribe` the server runs, and a Build switch showing "site, line 3.5, the current line, editor build". Health, "3 things to do", counts each thing with when it's from: no problems in every build, no tested examples changed or never tested, no reviews overdue, and nothing the permalinks list is behind on, each "Checked at 02:02 PM", the overdue reviews "for October 21"; 1 page nothing links to or includes and 2 content model entries nothing uses, "As the pages are now, unsaved changes too"; and 3 agent files out of date, "Checked at 02:02 PM, after the last save". Actions lists Check Every Build under Check; Apply Every Safe Fix, Format Every Page, Build, Sync Agent Files, and Observe Examples under Write; and Start Review and Prompt Agent to Fix This Project under Review and agents. About is closed.](../images/project-dark.png){scheme=dark}

- **Show Output** and **Restart** are there once the server has started, so a failed server is restarted where it's seen. They do what the [status bar](#the-status-bar) menu's **Show output** and **Restart server** do, for the project the view shows.
- **Build** is the build you're looking at, with every build of the project, and each line of a versioned build, to switch to. It's the same choice as the status bar's **Switch build** and the preview's **Build** picker: switching it in any of the three moves the others, the preview, and the [build lens](preview.md#the-build-lens) with it.
- The gear opens [Project Settings](settings.md) for the project, its `ascribe.toml`, which everyone working on it shares, and the second button opens VS Code's Settings at Ascribe's, which are your editor's and nobody else's.

The view has no badge: This page's badge is the one that counts what to do as you write.

**Health** counts what needs doing in the project. Each count is shown as current only when it is:

| Count | From | Opens |
|---|---|---|
| **Problems in every build** | "2 errors, 1 warning": the problems the last **Check Every Build** found in every build, by severity | The Problems panel |
| **Tested examples changed or never tested** | The same check's `example-stale` and `example-unobserved` | The Problems panel, and the first of them in the editor |
| **Reviews overdue** | The same check's `review-overdue`, for the day it ran | The Problems panel, and the first of them in the editor |
| **Permalinks the list is behind on** | The same check's `permalinks-list-behind` | The Problems panel, and the first of them in the editor |
| **Pages nothing links to or includes** | The language server's list of orphans, as Pages' **Orphans Only** has it | Pages, with **Orphans Only** on |
| **Content model entries nothing uses** | The language server's count of each entry's uses, as Content model's **Unused Only** has it | Content model, with **Unused Only** on |
| **Agent files out of date** | `ascribe agents sync --check`, from the last save | **Sync Agent Files**, which names the files it would change and asks first |

- A count of none shows `0`, dimmed, and opens nothing.
- A count that hasn't been made shows a dash and says what it needs, such as "Needs every build checked", with the action that makes it beside it.
- A count that's out of date shows a dash and "Out of date", with why, the time it was made, and the action that brings it current beside it. A dash is never dimmed, and an out-of-date count has a warning icon, so none and not known never look alike.

Each count goes out of date by its own rule:

- **The counts from Check Every Build** go out of date together, on any change to a file of the project, saved or not: a page, `ascribe.toml`, `ascribe.nav`, `ascribe.permalinks`, a memo, a template, or a file in one of its [sources](../examples/sources.md)' folders. A check that read files with unsaved changes says so: it reads files as they're saved.
- **Reviews overdue** depends on the day as well as the files: it says the day it's for, and goes out of date when the day changes, whatever else has.
- **Orphans and unused entries** come from the language server, which reads your unsaved text, so they're current as you type. While the server is starting, or isn't running, they show a dash.
- **Agent files** are made from the content model and the pages, so most saves can change them. The view checks them when it first shows the project, and again after each save of a file of the project while it's showing, at most once every few seconds and never as you type. A change on disk that isn't a save, such as a `git checkout`, makes the count out of date, with **Check Agent Files** beside it.

The view never checks every build by itself, since that can take a few seconds on a large project. **Actions** acts on the project the view shows, wherever the cursor is, and each says how it runs in its tooltip:

| Action | How it runs |
|---|---|
| **Check Every Build** | The extension checks every build and puts their problems in the Problems panel |
| **Apply Every Safe Fix** | The language server, as one edit, left unsaved: see [Edits to the whole project](#edits-to-the-whole-project) |
| **Format Every Page** | The language server, as one edit, left unsaved |
| **Build** | A task: `ascribe build`, in the terminal |
| **Sync Agent Files** | A task: `ascribe agents sync`, in the terminal |
| **Observe Examples** | A task: `ascribe observe`, in the terminal, in the folder of a source with [tested examples](../examples/tested.md). It lists the commands `ascribe.examples.toml` names and asks before running them. |
| **Start Review** | As **Ascribe: Start Review**: marks what changed in the preview, against a base it asks for. See [Review in the preview](review.md). |
| **Prompt Agent to Fix This Project** | As **Ascribe: Prompt Agent to Fix This Project**: builds a prompt about the project's problems for your agent, without sending it. See [Prompt your agent](../agents/prompts.md). |

The tasks are in **Tasks: Run Task** too, and [Tasks](settings.md#tasks) says where they run. **Escape** in the view puts the cursor back in the editor.

**About**, closed at first, says what the project is made of: its content root, its builds, with the editor build marked, the release lines of each versioned build, its [sources](../examples/sources.md), with those that have tested examples marked, and its content types, each with how many pages have it. The builds and content types are the language server's, so they're known once it runs; the sources are what `ascribe model` says. The view's sections stay open or closed as you leave them.

In a workspace with several projects, **Switch Project** is in the view's title bar: see [Switch Project](#switch-project).

In Restricted Mode, Ascribe runs nothing, since it would run the project's own `ascribe` binary. The view says so in place of the counts, with **Manage Workspace Trust**, and every action is off until you trust the folder.

### Edits to the whole project
@available: next

**Apply Every Safe Fix** applies every [safe fix](problems.md#apply-the-safe-fixes) of the problems the editor shows, in every page and fragment of the project, as **Apply the safe fixes in this page** applies one page's. They're the editor's build's problems, the ones in the Problems panel; a problem only another build has, or one that takes every build to tell, is [`ascribe check --fix`](../checks/check.md#applying-the-safe-fixes)'s to fix. **Format Every Page** formats every page and fragment as [Format Document](writing.md#formatting) and `ascribe fmt` do. Both are in the Command Palette too, as **Ascribe: Apply Every Safe Fix** and **Ascribe: Format Every Page**, for the active file's project.

Each is one edit to the text the editor has, so a page you've changed and not saved is fixed or formatted as you have it, never as it is on disk. Before it's made, the extension says how much it changes and asks: "Apply 6 safe fixes in 4 files? They're left unsaved, to read before you save them." When there's nothing to change, it says so and asks nothing.

The files are left unsaved on purpose. On a large project an edit can change hundreds of files, and each is opened with its changes unsaved, so you can read them before you save them: **File: Save All** saves them. **Undo** takes the edit back: VS Code asks whether to undo it in every file it changed or only in the one you're in. If a file changes while the question is up, nothing is applied; choose the action again.

## This page
@available: ascribe 0.4.0

**This page** says what's known about the active page, in four sections, and five while review is on, with **Review** after About ([Review in the preview](review.md#the-review-view)). About's **Markdown** row is "not published" when the project publishes no Markdown versions ([`agents = true`](../content-model/output.md#consumer)), and a fragment's address is only where it's included. With the cursor under a heading, Connections narrows to that heading's links and the includes of its section, whichever page they reach it through; Includes stays the page's. **Memos** lists the [memo files](actions.md#memo-files) about the heading, or about the page outside any heading, including those about a block in it, marked personal or outdated when it is. Each item opens the link, the include, or the memo; an include opens in the editor group beside the page's, with the focus left in the view, and This page stays on the page. Unsaved edits count.

![The This page view, with the rest of the Ascribe sidebar closed, beside the page Install the Quill agent with the cursor under its heading "Install the agent". At the view's top are the page's title, its type, page, and its path, install-agent.md. About gives its title, its type, its address, /install-agent/, its Markdown version, not published, its description's length, 102 characters, its availability, the builds that publish it, and its listing, "Listed (the default)". Health says its problems, none, and its size, "2,567 of 50,000 characters, in site@3.4". Connections, titled to “Install the agent”, lists under Linked from the file keys.md and its line that links there, with its line number, under Included by none, and under Includes the fragment _fragments/prerequisites.md. Memos, titled about “Install the agent”, lists one memo by its first line, with its file, memos/agent-first.md, beside it.](../images/this-page-light.png){scheme=light}
![The This page view, with the rest of the Ascribe sidebar closed, beside the page Install the Quill agent with the cursor under its heading "Install the agent". At the view's top are the page's title, its type, page, and its path, install-agent.md. About gives its title, its type, its address, /install-agent/, its Markdown version, not published, its description's length, 102 characters, its availability, the builds that publish it, and its listing, "Listed (the default)". Health says its problems, none, and its size, "2,567 of 50,000 characters, in site@3.4". Connections, titled to “Install the agent”, lists under Linked from the file keys.md and its line that links there, with its line number, under Included by none, and under Includes the fragment _fragments/prerequisites.md. Memos, titled about “Install the agent”, lists one memo by its first line, with its file, memos/agent-first.md, beside it.](../images/this-page-dark.png){scheme=dark}

@available: next
This page follows the page you're working in: the last editor of a page that was active. Clicking in the view, or moving to the preview, another view, a panel, or a file that isn't a page leaves it on its page; when that page's editor closes, it shows the page of the editor that takes its place. Only **About** starts open. The other sections' headers say whether to open them: Health's count of things to do, Connections' heading, and Memos' "none". Open or close a section and it stays that way on every page. Rows wrap, so each shows all it says at any width, with more in its tooltip, and an open section's header stays at the top of the view while you scroll through its rows. **Tab**, or **Up** and **Down**, moves between the section headers and the rows that open something, **Enter** or **Space** opens a section or a row, and **Escape** puts the cursor back in the page. As the cursor moves and the checks run, the view changes what it says in place, keeping its scroll and the row you're on.

@available: next
The title above the sections draws its code spans as code where its type's `title` field reads them ([`inline = "code"`](../content-model/index.md#53-code-in-a-field)), and a fragment's type is "fragment". A row whose state can warn says the state first, in the warning color when there's something to do: an example's "changed since it was tested" before its address, a description's "too long" before its length. The view's badge counts Health's things to do, so you see them with This page collapsed or Health closed; it goes when you move to another page while the view is collapsed, until you open it again. In place of a page, This page says when no page is open, when the page's language server is starting, and when it failed to start, with **Show Output** and **Restart**; in Restricted Mode it says Ascribe isn't checking the folder, with **Manage Workspace Trust**.

About's other rows say whether the page is published, listed, due, and finished. Each is shown only when it has something to say, apart from a description, which is shown "missing" where [`page-description-missing`](../checks/diagnostics/content-checks.md#ASC150) reports the page. A row with somewhere to go opens it:

| Row | Says | Opens |
|---|---|---|
| **Description** | Its length, or its field's default's when the page leaves the field out or empty, against the 300 characters its line in `llms.txt` reads best under when the project publishes one, led by "too long" with a warning only where [`description-too-long`](../checks/diagnostics/content-checks.md#ASC163) reports it, or "missing" | The field in the frontmatter |
| **Review** | "overdue, since 3 October" or "due in 12 days", from the field with the [`review-date` role](../content-model/index.md#54-field-roles), as [`review-overdue`](../checks/diagnostics/content-checks.md#ASC153) compares it with today, with a warning only where that check reports the page | The field |
| **Availability** | The page's own `available` | Nothing |
| **Builds** | "Published by site, agents; left out by cloud": the builds that publish the page and the ones that leave it out, with why in its tooltip | **Switch Build**, to see the page as a build publishes it; **Dim What the Build Leaves Out** shows what that build leaves out of it |
| **Listing** | "Listed (the default)" or "Unlisted, by `[listing.folders]`": what [`pages.json`](../content-model/checks.md#22-listing) says of the page, and the rule that sets it | The page's own `listing`, or the rule in `ascribe.toml` |
| **Permalink** | A [permalink](../permalinks/index.md) the page or one of its headings declares, one row each | Its `permalink` or its `@id` line |
| **Template** | "2 placeholders left, how-to template": what its [type's template](../templates/index.md#whats-left-to-fill-in) still leaves to fill in | **Go to Next Placeholder** |

**Health** has these rows:

| Row | Says | Opens |
|---|---|---|
| **Problems** | "1 warning, 2 advice": VS Code's problems for the page, as the Problems panel counts them | The Problems panel, where its filter's **Show Active File Only** keeps it to the page |
| **Size** | "31,200 of 50,000 characters, in site": the page's Markdown in the build where it's largest, measured as [`page-size`](../content-model/checks.md#19-checks) measures it, shown always so a page is seen growing toward the limit, and led by "too large" once it reaches it | Nothing; its tooltip says which build and the limit |
| **Example** | Each [tested example](../examples/tested.md) a `@snippet` of the page shows: its state first, tested, changed since it was tested, or not tested, then its address, what it was tested with, and the heading it's under | The file its `@snippet` shows, beside the page |

A fragment has no builds, listing, or size of its own: it's published only where it's included.

This page updates as you move between pages and headings, while it's open.

## Pages

**Pages** has **New Page** in its title bar ([Start a page](actions.md#start-a-page)), and shows the project's [navigation](../navigation/index.md) in its order, a page by its [`nav-label`](../navigation/index.md#a-page-s-label) or else its title, then the project's fragments by folder, each with the files that include it. A page's tooltip counts the links and includes that reach it. Its sections start closed, and opening a page selects it in Pages, opening the sections above it, so the open page's path is what you see. It's the easy way to change the order: drag a section or a page where you want it, and it writes `ascribe.nav`, unsaved. [Or drag them in the editor](../navigation/order.md#or-drag-them-in-the-editor) has how each drag is placed, **Place Here** for a page the file leaves out, and **Rename Label…**.

![The Pages view for the project quill, with the rest of the Ascribe sidebar closed and the status bar showing the cloud build. Start, the content root's section, lists its pages by title with their paths beside them, first Try Quill in the browser, quickstart.md, and Install the Quill agent, install-agent.md; four pages the cloud build leaves out, Configure the agent, Configure the agent (3.4), Upgrade the agent, and Webhooks, say "Not in cloud" in place of their paths, dimmed; and API keys, which ascribe.nav leaves out of the section's order, comes last, with a warning icon and "not placed". The section Reference, reference/, lists its own page first, then quill.yaml, then The quill command, reference/cli.md, in red with a count of 1 for the error on it. Fragments, closed, counts one.](../images/pages-light.png){scheme=light}
![The Pages view for the project quill, with the rest of the Ascribe sidebar closed and the status bar showing the cloud build. Start, the content root's section, lists its pages by title with their paths beside them, first Try Quill in the browser, quickstart.md, and Install the Quill agent, install-agent.md; four pages the cloud build leaves out, Configure the agent, Configure the agent (3.4), Upgrade the agent, and Webhooks, say "Not in cloud" in place of their paths, dimmed; and API keys, which ascribe.nav leaves out of the section's order, comes last, with a warning icon and "not placed". The section Reference, reference/, lists its own page first, then quill.yaml, then The quill command, reference/cli.md, in red with a count of 1 for the error on it. Fragments, closed, counts one.](../images/pages-dark.png){scheme=dark}

- A page or fragment with a problem is colored as it is in the Explorer: red for an error, yellow for a warning.
- A page the build the status bar shows leaves out is dimmed, its title as well as its icon, as the [build lens](preview.md#the-build-lens) dims text, and follows the build as you switch. A problem's red or yellow still shows over it. The dimming is the file's, so the Explorer and the editor's tabs dim it too.
- A page's menu has **Open Preview to the Side**, also its button; **Copy Link**, which copies its path from the content root, such as `/guides/install.md`, the link a page writes to it (its published address is This page's **Address** row); **Rename or Move…**, which asks for its new path and moves it as the Explorer does, the links to it following; **Find All References**, the links and includes that reach it, in a peek at its start; and **Reveal in Explorer**. A fragment's menu has the last three.
- **New Page Here**, in a section's menu, starts a page in its folder ([Start a page](actions.md#start-a-page)).
- **Orphans Only**, the filter button in the title bar, lists only the pages nothing links to or includes, apart from index pages, and the fragments nothing includes, each in its section or folder. Choose it again to list everything.

Pages updates when you save a file of the project, when a page is added or deleted, as you edit `ascribe.nav`, and when you switch to another project's file.

## Content model

**Content model** lists the project's page types, phrases, features, glossary terms, dimensions, note types, widgets, and builds, each with how many places use it, apart from builds, which pages don't name; a page type, with how many pages have it. An entry nothing uses is marked **unused**, a candidate to remove. Each dimension lists its values. Clicking an entry or a value opens its declaration in `ascribe.toml`, or a glossary term's heading on its page. The counts are what **Find All References** lists for the same entry.

- **Find All References**, in the menu of an entry pages use, lists those uses in a peek at its declaration. On a page type, it lists the pages whose `type` names it, at that value.
- **Rename…**, on a phrase or a dimension's value, asks for the new name and makes the edits `F2` on it in a page makes, shown in the refactor preview first.
- **Insert at the Cursor**, on a phrase or a widget, runs **Insert a phrase** or **Insert a widget** with it chosen, at the cursor of the project's last Markdown editor, which comes to the front. Where the cursor is somewhere the action doesn't apply, it says where to put it and changes nothing.
- **Unused Only**, the filter button in the title bar, lists only the entries nothing uses.

Content model updates when you save a file of the project, when a page is added or deleted, and when you switch to another project's file.

## The status bar

While a page or an `ascribe.toml` is the active editor, the status bar names its project and the [build you're looking at](preview.md#the-build-youre-looking-at), with an icon for its language server's state. Its tooltip gives the project's folder, its `ascribe.toml`, the `ascribe` binary in use, and the server's state. While the project has several builds and the Problems panel holds the problems of the editor build alone, it says what the panel covers; once you [check every build](problems.md#the-problems-panel), and until something changes, it doesn't. A project's language server starts when one of its pages opens, so until then nothing has checked it: while no page of a project is active, the status bar says so, counting the projects when there are several, and its tooltip names them. Otherwise a file that isn't in a project shows nothing.

![The status bar item in five states, one above another: "docs: not checked yet" with a slashed circle, before a page of the project has opened; "docs" with a spinning arrow, while its language server starts; "docs · site · problems: this build only" with a book, while it runs and the Problems panel covers the editor build alone; "docs · windows · problems: site only" with an eye, while the build lens dims by the windows build; and "docs" with a warning, when the server failed.](../images/status-bar-light.png){scheme=light}
![The status bar item in five states, one above another: "docs: not checked yet" with a slashed circle, before a page of the project has opened; "docs" with a spinning arrow, while its language server starts; "docs · site · problems: this build only" with a book, while it runs and the Problems panel covers the editor build alone; "docs · windows · problems: site only" with an eye, while the build lens dims by the windows build; and "docs" with a warning, when the server failed.](../images/status-bar-dark.png){scheme=dark}

In Restricted Mode, Ascribe runs nothing, since it would run the project's own `ascribe` binary. The status bar shows **Ascribe: not checking**, and the Problems panel's "No problems" doesn't mean the project is clean. Click it to manage the folder's trust; once you trust it, Ascribe starts.

Click it, or run **Ascribe: Project Menu**, for a menu of the project: **Switch build**, **Check every build**, **Show output**, **Switch project** in a workspace with several projects, **Restart server**, and, for a page, **Dim what the build leaves out** and **Open preview**. The [Project view](#the-project-view) has the same jobs for the project it shows, and each does the same there as here.

## Switch Project
@available: ascribe 0.4.0

In a workspace with several projects, **Ascribe: Switch Project** chooses the project the sidebar shows. It's in the Command Palette, in the [Project view](#the-project-view)'s title bar, and in the [status bar](#the-status-bar) item's menu; with one project, it's in none of them. It lists every project, the sidebar's first and marked, each with its folder, its server's state, such as "running, site build", "not checked yet", or "failed", and the `ascribe` binary in use. Each has buttons to show its server's output, and to restart its server when it's running or failed, and **Refresh** in its title asks the running servers for their builds again. Listing them starts no server.

Choosing a project shows it in the sidebar and starts its server if it hasn't started. It's a jump, not a pin: the sidebar stays on it until a file of another project becomes active, and then follows that file as before.
