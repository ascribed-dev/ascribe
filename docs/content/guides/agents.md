---
title: Agents
description: Working on an Ascribe project with an AI coding agent, such as Claude Code, GitHub Copilot, or Cursor, and handing it the problems Ascribe finds.
available: next
---

An agent is the AI coding tool you work with: Claude Code, GitHub Copilot, Cursor, and others. Ascribe doesn't call a model and holds no key for one. It gives your agent what it knows about a project, in a form an agent reads well, and checks what the agent writes.

## Prompt your agent

**Prompt agent** turns a problem Ascribe reports into a prompt for your agent: what to fix, where, what Ascribe knows about it, and how to check the result. It fills the prompt in where you send it and never sends it for you, so you read it first.

### In VS Code

- On a problem, the lightbulb (`Ctrl+.`, or `Cmd+.` on macOS) offers **Prompt agent: fix this problem**, after the problem's quick fixes. A problem with a safe quick fix is quicker to fix with it.
- **Ascribe: Prompt Agent to Fix This File** builds one prompt for the active file's problems, and **Ascribe: Prompt Agent to Fix This Project** one for the project's. The Command Palette shows them when there are problems.

The prompt is about what the editor checks: the file's problems and its pages' problems in the editor's build (`[editor] build`), as you see them, unsaved edits included. It names the build, and says that `ascribe check` checks every build. Your agent reads the files on disk, so when the file has unsaved changes, the prompt and the editor both say to save it first.

The `ascribe.agents.promptTarget` setting says where the prompt goes:

| Value | Where the prompt goes |
|---|---|
| `clipboard` (the default) | The clipboard: paste it into any agent. |
| `chat` | VS Code's chat, in its input. |
| `claude-code` | A new Claude Code tab, in its input, through the Claude Code extension. |
| `cursor` | Cursor's chat, in its input, through its `cursor://` link. Only in Cursor. |

Each one fills the prompt in and leaves it for you to send. When the one you chose isn't there (the extension isn't installed, or the editor isn't Cursor), the prompt is copied instead, and the status bar says so. The first time you prompt an agent, a notification offers the others your editor has, once.

### From the command line

`ascribe check --format prompt` writes the same prompt to standard output, for an agent in a terminal. With a file named, the prompt is about that file; otherwise, about the paths named or the whole project. It writes nothing when there are no problems, and exits as `ascribe check` does. For a file on disk, `ascribe check <file> --editor-build --format prompt` writes the prompt the editor builds.

@snippet {lang=text}: code:crates/ascribe-cli/tests/output/check-prompt.txt

### What a prompt says

Every prompt has the same parts, in order:

1. **The task**, in one sentence: "Fix this problem in `docs/keys.md`."
2. **Where**: the file and lines, from the project's folder. When they apply, the project's folder in the repository (`Project: docs/`), the build the problems are from (`Build:`), the pages that show a fragment (`Shown on:`), and a line saying to save the file first.
3. **What Ascribe knows**: for one problem, its code, message, the line's text, how to fix it, the values the content model allows when it's about one, and whether Ascribe has a fix ("a safe automatic fix", or "a fix to review"). For a file, each problem on one line, at most 20. For a project, how many problems each file has, at most 20 files. Each says which command lists the rest.
4. **How to finish**: "Follow the project's rules in `AGENTS.md`", when the project's folder or the repository's root has one, and "When you're done, run `ascribe check <file>` and fix what it reports."

Commands in a prompt are written from the repository's root, so an agent started there can run them. A prompt is at most 5,000 characters, the most an agent's link takes. When it would be longer, what Ascribe knows is cut between parts, and the prompt says where to read the rest.
