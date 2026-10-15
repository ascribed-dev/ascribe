# Writing a how-to in Ascribe

Ascribe's notes on the how-to, to read after its guide: only what's particular to writing one in an Ascribe project. They're yours to change, as the template is.

- **The steps are one procedure.** The template's numbered list starts with `@steps`, which tells the site and agents it's a procedure. Keep the task's steps in that one list, with a sub-step inside the step it belongs to.
- **A step that shows a file and its output can be a heading.** A list item holds a code block badly. When each step holds a file, a command, and what it prints, number the steps as headings of their own, `### 1. Install`, with no `@steps`: it marks a list, and these steps aren't one.
- **Alternatives by platform are a variant group.** Where a step differs on Windows, don't write "if you're on Windows". Write an arm for each value of a dimension your `ascribe.toml` declares:

  ````markdown
  @variant {platform=linux|macos}:
  ```shell
  ./install.sh
  ```
  @variant {platform=windows}:
  ```powershell
  .\install.ps1
  ```
  @end
  ````

  The site shows the arms as tabs and keeps the reader's choice on every page, and a build for one platform keeps only its arm.
- **A step's command comes from tested code.** `@snippet: code:scripts/rotate-keys.sh#rotate` puts a region of a file your tests run into the step, so the page can't drift from what works.
- **A warning is a note.** The guide's callouts are `@note {type=warning}:` or `{type=caution}`, written in the step they're about.
- **Links name the file.** In "See also", `[](rotate-keys.md)` shows that page's title as it is now, and `ascribe check` reports the link when the page moves.

The directives are in the [directives reference](https://ascribed-dev.com/docs/writing/directives/).
