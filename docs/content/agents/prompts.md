---
type: how-to
title: Prompt your agent
description: "Turn a problem, a review comment, or a changed page into a prompt for your agent, in VS Code or from the command line."
available: ascribe 0.3.0
---

**Prompt agent** turns a problem Ascribe reports into a prompt for your agent: what to fix, where, what Ascribe knows about it, and how to check the result. It fills the prompt in where you send it and never sends it for you, so you read it first. [What a prompt says](commands.md#what-a-prompt-says) lists its parts.

## In VS Code

@steps
1. Choose where prompts go with the [`ascribe.agents.promptTarget`](../editor/settings.md#editor-settings) setting: the clipboard (the default), to paste into any agent, or VS Code's chat, a Claude Code tab, or Cursor's chat, with the prompt in its input.
2. Build a prompt:
   - On a problem, the lightbulb (`Ctrl+.`, or `Cmd+.` on macOS) offers **Prompt agent: fix this problem**, after the problem's quick fixes. A problem with a safe quick fix is quicker to fix with it.
   - **Ascribe: Prompt Agent to Fix This File** builds one prompt for the active file's problems, and **Ascribe: Prompt Agent to Fix This Project** one for the project's. The Command Palette shows them when there are problems.
3. Read the prompt where it lands, and send it.

When the place you chose isn't there (the extension isn't installed, or the editor isn't Cursor), the prompt is copied instead, and the status bar says so. The first time you prompt an agent, a notification offers the others your editor has, once.

The prompt is about what the editor checks: the file's problems and its pages' problems in the editor's build (`[editor] build`), as you see them, unsaved edits included. It names the build, and says that `ascribe check` checks every build. Your agent reads the files on disk, so when the file has unsaved changes, the prompt and the editor both say to save it first.

## From the command line

`ascribe check --format prompt` writes the same prompt to standard output, for an agent in a terminal. With a file named, the prompt is about that file; otherwise, about the paths named or the whole project. A file's or a project's prompt counts what it's about as `ascribe check`'s summary does, and asks the agent to fix the errors and warnings; advice, which never fails the check, comes after them as something to look at. It writes nothing when there are no problems, and exits as `ascribe check` does. For a file on disk, `ascribe check <file> --editor-build --format prompt` writes the prompt the editor builds.

@snippet {lang=text}: code:crates/ascribe-cli/tests/output/check-prompt.txt

`ascribe agents prompt` prints the other prompts: `new-page` to write a page of a type; `fix` to check the project and fix what's reported; and `review` to review what a branch does to its pages, the prompt `ascribe diff --format prompt` writes. `ascribe agents prompt --list` lists them with their arguments:

```shell
ascribe agents prompt new-page --arg type=how-to --arg title="Rotate keys"
```

[`ascribe agents prompt`](commands.md#cli-agents-prompt) says where `new-page` puts the page, and what it gives the agent to start from.

## Memos in a prompt
@available: ascribe 0.4.0

A prompt about a page names the [memo files](../writing/memo-files.md) about it, so your agent reads why the page is written as it is before it changes it. They come after the problems, under a heading of their own, a line each: the memo's path, what its `about` names, and its state when it isn't placed. The agent opens the file and reads it whole; a prompt never has a memo's text in it.

- **`fix`**, `ascribe check --format prompt`, and **Prompt agent** on a problem, a file, or a project list the memos about each page they name, outdated and detached ones too, saying which.
- **`review`** and `ascribe diff --format prompt` list each memo on a changed block, on the section a change is in, or on the whole page, and ask the agent to check the change still agrees with it. A memo whose quote is in another block of the section isn't listed.

A [personal memo](../writing/memo-files.md#personal-memos) is listed in prompts made on the machine that has it, marked as personal. Each list stops at ten memos, and says how many more there are and the `ascribe outline` command that lists them.

## In review

While you [review](../review/index.md) a pull request, **Prompt agent** hands your agent a review comment, or the change itself:

- **On a comment.** Each thread beside the page has **Prompt agent**, which asks your agent to address the comment: the thread's file and lines, the block's text from the file on disk, and the comments. **Comments** lists every thread, with **Prompt agent: all open** for the open ones, two lines each. In the source editor, a thread's title bar has **Prompt Agent** too.
- **On the page.** **More review actions** (**⋯**) in the preview's header has **Prompt agent: review this page**, which asks your agent to review what the change does to the page as readers see it, and, when the page changed through a fragment, **Prompt agent: check this fragment's pages**, which asks it to check that the new text fits every page that shows it.

In VS Code, these go where `ascribe.agents.promptTarget` says. The site preview copies them, since a page in your browser can't reach your editor, and says "Prompt copied". The [HTML report](../review/diff.md#the-html-report) has **Copy prompt** for each page. A prompt about the change asks your agent for a report, not edits: you decide what changes.

A comment is someone else's text, so a prompt never passes it as an instruction: it goes inside a fence, as data. Read the prompt before you send it, as you would any request from a reviewer, and see [Agents that reach the network](whats-sent.md#agents-that-reach-the-network) for what the fence doesn't stop.

From the command line, `ascribe diff --format prompt` writes the prompt about the change, and `ascribe diff --format prompt <page>` the one about a page or a fragment, from the same base and builds `ascribe diff` uses:

@snippet {lang=text}: code:crates/ascribe-cli/tests/output/diff-prompt.txt
