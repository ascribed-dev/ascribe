# Proposal: the Project view

A proposal, not a plan. It says what the view is and what it would look like, so it can be judged before anything is designed in detail. It follows the [This page proposal](this-page-webview.md), which makes This page a webview: This page is about one page, and this view is about the project the page is in.

## The goal

One place in the editor says what state the whole project is in and does the jobs that work on the whole project, so an author doesn't have to know which command line, palette entry, or title bar holds each. The view is where they're found; the commands it runs stay one each, and the status bar menu and the Command Palette reach the same ones ("Settled").

A writer can't tell today, from inside VS Code, whether the project has problems in builds the editor doesn't check, how many pages nothing links to, or whether the agent files have fallen behind. Each answer exists. They're in a status bar menu, two views' title bars, the Command Palette, and a dozen commands of the CLI.

## What exists

**Project-wide, in the editor:**

- **The status bar item's menu** (`packages/vscode/src/ui/statusBar.ts`): Switch build, Check every build, Show output, Switch project, Restart server, Dim what the build leaves out, and Open preview.
- **Pages' and Content model's title bars:** New Page, Orphans Only, Unused Only, and Switch Project while the workspace has two or more projects ([decision #544a](../decisions/544a-sidebar-views.md)).
- **The Command Palette:** Prompt Agent to Fix This Project, Start Review, and Open Site Preview.
- **Check Every Build** (`src/ui/checkEveryBuild.ts`) runs `ascribe check --format json` for the project, puts every build's problems in the Problems panel, and knows whether its result is current: nothing in the project has changed since (`EveryBuildApi.isCurrent`). Its problems include `permalinks-list-behind`, when `ascribe.permalinks` is behind the permalinks the pages declare.
- **The build you're looking at** is one choice per project, `ChosenBuilds` (`src/ui/chosenBuild.ts`): the status bar's Switch Build and the preview's picker both set it, and the preview and the dimming follow it.
- **The sidebar's project** follows the active file, and keeps the one it shows when the active file belongs to no project (`followedProject` in `src/ui/sidebar.ts`).
- **`ascribe/inventory`** (`crates/ascribe-lsp/src/inventory.rs`) already carries the project's orphans and each content model entry's count of uses. Pages' Orphans Only and Content model's Unused Only are drawn from it.
- **Safe fixes for a page** are one action, `source.fixAll.ascribe`, for the problems the editor shows; a problem only every build can tell is `ascribe check --fix`'s alone ([decision #544e](../decisions/544e-fix-all.md)).
- **Formatting a page** is the language server's, with `ascribe_fmt::format`, the code `ascribe fmt` runs.
- **The extension runs no tasks.** It has no task provider, and every command it runs, it runs itself.

**Project-wide, on the command line only:** `ascribe build`, `check --fix`, `fmt`, `report`, `agents sync`, `observe`, `sources fetch` and `update`, `permalinks sync`, `versions release`, and `templates eject`.

**A caution, from the research behind the views.** [VS Code extension UI patterns](../reports/VS%20Code%20extension%20UI%20patterns.md) argues against a content dashboard: Ascribe's findings have places, so they belong where places are listed, and a dashboard repeats lists badly. This view follows that by counting, never listing.

Mockups of the view, kept locally with the other mockups (`.gitignore` leaves that folder out), show it on Ascribe's own docs, in Dark Modern and Light Modern, with the counts current and not, and in Restricted Mode.

## What it looks like

A webview view, **Project**, first in the Ascribe sidebar, built on what the This page proposal builds: the same messages, stylesheet rules, keyboard, and handshake. It shows the sidebar's project, which follows the open file as the other views do, and keeps the project it shows when the active file belongs to none, as `followedProject` does. A click in the view makes no editor active, so it never moves the view off its project.

- **A head:** the project's name and folder, the server's state, the binary's version, and the build you're looking at.
  - **The server:** **Show Output** and **Restart** sit beside its state. The view is where a failed server is seen, so it's where it's restarted, and the Projects view that offered both beside a project is gone.
  - **The build:** its switch is the same `ChosenBuilds` choice the status bar and the preview's picker set, so switching in any of the three moves all of them, the preview and the dimming with them. It isn't a second choice.
  - **Settings:** beside the name, a gear for **Project Settings** (the [settings proposal](project-settings.md)) and a second button for VS Code's Settings, filtered to Ascribe's: the project's settings, which everyone shares, and your editor's, which nobody else gets.
- **Health:** counts, each with what it counts and where it opens. Its header says how many things there are to do.
- **Actions:** grouped as Check, Write, and Review and agents, each saying how it runs.
- **About,** closed at first: the content root, the builds, the release lines, the sources, and the content types.

The view's sections, their open state, and its keyboard follow This page's.

### Health

| Count | From | Opens |
|---|---|---|
| Problems in every build, by severity | The last Check Every Build | The Problems panel |
| Tested examples changed since they were tested, or never tested | The same run's `example-stale` and `example-unobserved` | The Problems panel, at the first |
| Reviews overdue | The same run's `review-overdue` | The Problems panel, at the first |
| A permalinks list that's behind | The same run's `permalinks-list-behind` | The Problems panel, at it |
| Pages nothing links to or includes | `ascribe/inventory`'s orphans | Pages, with Orphans Only |
| Content model entries nothing uses | `ascribe/inventory`'s uses | Content model, with Unused Only |
| Agent files out of date | `ascribe agents sync --check` | Sync Agent Files, with the files it would change |

### When each count is from

A count is shown as current only when it is. Each comes from one of three places, and each has its rule:

- **The last Check Every Build:** problems, examples, overdue reviews, and the permalinks list. The view says when the check ran. A count is current while `isCurrent` says nothing in the project has changed since. **Reviews overdue** depends on the day as well as the files, so it also says the day it's for, and goes out of date when the day changes, whatever `isCurrent` says. The view never runs the check by itself: it takes a second or more on a large project.
- **`ascribe/inventory`:** orphans and unused entries. The language server answers from its snapshot, unsaved text included, so these are current as you type, and the view asks again as it does for Pages.
- **`ascribe agents sync --check`:** agent files, the one count that's no problem `check` reports. The agent files are made from the content model and the pages, so most saves can change it. It runs again after each save of a file of the project, at most once every few seconds, never as you type, and the view says it's from the last save, or that it's running.

**Out of date and none look different.** A count that's out of date, or from a check that hasn't run, shows a dash and says why ("Needs every build checked", "Checked yesterday"), with the action that brings it current beside it. A count of zero shows `0`, dimmed. A reader tells "none" from "not checked" at a glance.

### Actions

| Action | How it runs |
|---|---|
| **Check Every Build** | As today: the extension runs the check and fills the Problems panel |
| **Apply Every Safe Fix** | The language server, as one workspace edit: unsaved, undoable, and over unsaved text |
| **Format Every Page** | The language server, as one workspace edit |
| **Build** | A task: `ascribe build` in the terminal |
| **Sync Agent Files** | A task: `ascribe agents sync` |
| **Observe Examples** | A task: `ascribe observe`, which runs the project's test commands |
| **Start Review** | As today |
| **Prompt Agent to Fix This Project** | As today |

**Edits to pages go through the language server.** A workspace edit is what the editor already makes for one page's safe fixes and formatting: the files change unsaved, Undo puts them back, and an open file's unsaved text is what's edited, never the disk under it. On a large project that can be hundreds of files, each opened unsaved, so the action says how many it will change before it does: the server makes the edit first, and the extension asks, "Apply 6 safe fixes in 4 files? They're left unsaved, to read before you save them." The docs say the files are left unsaved on purpose.

**Writing outputs and running programs is a task.** The extension contributes a task type, `ascribe`, so these run in the terminal with their output in view, as they would in CI, and the same tasks are in **Tasks: Run Task** and can be bound to keys or named in `tasks.json`. Each runs the binary the project's server runs, in the project's folder, with `--config` naming its `ascribe.toml` when the project isn't the workspace folder. A task has no problem matcher: its output is read in the terminal, and the Problems panel stays Check Every Build's and the editor's, so the two never disagree.

**Observe Examples runs commands the repository names,** the test commands its `ascribe.examples.toml` lists. It's offered only in a trusted workspace, and before it runs it shows those commands and asks, each time: a task from a tree of someone else's repository is the case Restricted Mode exists for. In Restricted Mode, every action is off and the counts are replaced by a sentence saying Ascribe isn't running in the folder.

**What stays on the command line:** `versions release`, `templates eject`, and `sources update`. Each is rare, and each changes many files at once in a way that wants reading before it's done. `ascribe report` stays too: its findings are already `check`'s, and its external links need the network.

## What changes besides the code

- **The views' order:** Project, then Review while review is on, This page, Pages, and Content model. Decision #544a names the order, so a decision file in the pull request that adds the view says the new one, and where Switch Project is offered.
- **The docs:** "The Ascribe sidebar" in `docs/content/editor/sidebar.md` gains the view, and `docs/content/editor/settings.md` the task type. The walkthrough's sidebar step and the extension's README, where they list the views. An entry in `changes/editor/`.
- **The surface map,** for the new commands and the task type.

## What this leaves out

- **Lists of what's counted.** Each count opens the list that already exists.
- **History:** counts over time, a chart, a trend. The report's history (decision 125) stays the report's.
- **External links,** which only `ascribe report` checks, with the network.
- **Every project at once.** The view shows the sidebar's project; Switch Project moves it.
- **The actions above that stay on the command line.**

## The thread

For a later plan, by the repository's [steel threads](../../.claude/skills/steel-threads/SKILL.md) skill. This isn't the plan.

- **The layers:** the extension's view and its messages (`packages/vscode/src/ui/`), the webview's script and stylesheet (`packages/vscode/src/webview/`), Check Every Build's result, `ascribe/inventory`, a task provider, and the docs.
- **The riskiest seam** is a count that has to be honest about when it's from. A count from the last Check Every Build goes stale with the first edit, and a view that shows it as current misleads more than one that shows nothing. If `isCurrent` can't follow every change that matters, the design changes before more counts are added.
- **The second seam is the task type,** a contribution point the extension hasn't used: a task that runs the binary the server runs, in the right folder, with the right `--config`, on every platform the extension supports.
- **The thinnest use case:** in `docs/`, the Project view shows the problems in every build as unknown. Choose **Check Every Build** in it: the count appears, says it's current, and opens the Problems panel. Edit a page: the count says it's out of date.
- **Left for the widening, in order of risk:** the first task, Build, through the task provider (the second seam); the edit actions, with their count of files; the counts from `ascribe/inventory`; overdue reviews and their day; the agent files' count and its rerun on save; the head, with the server's buttons, the build switch, and the two settings buttons; Restricted Mode, and Observe Examples' question; About; the docs and the decision.

## Settled

Answered by the maintainer on 20 October 2026, after looking at the mockup: the project gets a view of its own, a webview, first in the sidebar, not a section of This page or a longer status bar menu; it counts, and each count opens the list that already exists; its first actions are the eight above; edits to pages are workspace edits and the rest are tasks; and the three commands named stay on the command line. _Why:_ This page is about one page and should collapse on its own; counts that open the existing lists keep the research's caution; a task shows what ran, and a workspace edit can be undone.

The proposal's four open questions, answered by the maintainer on 20 October 2026:

- **Apply Every Safe Fix reaches the editor's build,** as one workspace edit, its caption saying so; every build's fixes stay `ascribe check --fix`'s, as decision #544e has it. _Why:_ the language server knows the editor's build's problems, and a fix only every build can tell is one the editor doesn't show.
- **Switch Project lives in the Project view's title bar alone,** and Pages and Content model lose it. _Why:_ the view is about the project.
- **The Project view has no badge.** _Why:_ two badges in one sidebar compete, and This page's is the one an author acts on as they write.
- **The status bar menu keeps what it has:** Switch build, Check every build, Show output, Switch project, and Restart server, each running the same command the view runs. _Why:_ the status bar is there with the sidebar closed, and its menu is the keyboard's quick route; there's one of each job, and two ways to it, and the view is the one that explains.
