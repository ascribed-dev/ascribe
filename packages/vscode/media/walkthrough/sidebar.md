# The Ascribe sidebar

- **Project**: the project's language server and binary, with Show Output and Restart, the build you're looking at, which switches the preview's and the status bar's too, its Actions, then its health: the problems in every build, stale examples, overdue reviews, orphans, unused entries, and agent files out of date, in groups by when each was counted. Each count opens its list, and each group says when a change has made it out of date, with the action that brings it up to date. Its Actions build the project, sync its agent files, and observe its tested examples, in the terminal, where `ascribe observe` asks before it runs the commands the examples name, and start a review or prompt your agent. About says what the project is made of, and the gear beside its name opens Project Settings.
- **Review**, while review is on: the changed pages, their problems, and the comments you haven't submitted.
- **This page**: the page you're on, headed by its title, type, and path, then About, with its addresses and whether it's published and listed, Health, with every row that can warn: its problems, a description too long, a review overdue, a template unfinished, its size, and its tested examples, and what links to it or to the heading at the cursor. Only About starts open; open another section and it stays open on every page. A row that warns says its state first, and Health's header and the view's badge count the rows that warn, so you see them with Health closed.
- **Pages**: the project's pages in the navigation's sections, and its fragments. **Orphans Only**, in its title bar, lists only the pages nothing links to.
- **Content model**: the project's page types, phrases, features, glossary terms, and the rest, with how many places use each.

The views show the project of the file you're in. In a workspace with several projects, **Switch Project** in the Project view's title bar lists them all, with whether each one's language server is running, and shows another.

Opening the sidebar starts no language server: a project's server starts when you open one of its pages, or choose it in Switch Project.
