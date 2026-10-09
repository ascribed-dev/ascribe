---
title: Agents
description: Working on an Ascribe project with an AI coding agent, such as Claude Code, GitHub Copilot, or Cursor, and handing it the problems Ascribe finds.
available: next
---

An agent is the AI coding tool you work with: Claude Code, GitHub Copilot, Cursor, and others. Ascribe doesn't call a model and holds no key for one. It gives your agent what it knows about a project, in a form an agent reads well, and checks what the agent writes.

## Agent instructions

`ascribe agents sync` writes your project's rules into the files agents read on their own, from `ascribe.toml`:

```shell
ascribe agents sync
```

It writes:

- **`AGENTS.md`** beside `ascribe.toml`: the check to run after each edit and before finishing, your page types and their frontmatter, your phrases, the directives your pages use, and how to link. It's at most 4,000 characters, and long lists are cut with the command that shows the rest.
- **A short block in the repository's root `AGENTS.md`,** when your project is in a subfolder: where its pages are, the check to run, and where its full rules are. Some agents, such as Codex, read no `AGENTS.md` below the folder they start in.
- **The skill,** in `.agents/skills/ascribe/` at the repository's root. See [The skill](#the-skill).

Here's what it writes for the Quill example:

@snippet {lang=markdown}: code:crates/ascribe-cli/tests/output/agents/quill/AGENTS.md.snap

### Your text stays yours

Ascribe writes only between its two markers, `<!-- ascribe:agents start … -->` and `<!-- ascribe:agents end -->`. Everything else in the file is your team's, and is left byte for byte as it was: write your own rules above or below the block. A file with no markers gets the block at its end, and a file that doesn't exist is created with only the block. When a file has a start marker and no end marker, or two of either, `sync` changes nothing anywhere and exits with `2`: fix the markers by hand.

Don't edit the block itself: change `ascribe.toml`, or your pages, and run `sync` again.

### Other agents' files

Beyond `AGENTS.md` and the skill, `sync` keeps up to date the files of each target that already has them, and writes a target's files when you ask with `--target`:

| Target | What it writes |
|---|---|
| `claude` | An import of `AGENTS.md` (`@AGENTS.md`) in `CLAUDE.md`, and the skill in `.claude/skills/ascribe/`, loaded for your pages |
| `claude-rules` | The rules in `.claude/rules/ascribe-<project>.md`, loaded only when Claude Code works on your pages |
| `copilot` | The rules in `.github/instructions/ascribe-<project>.instructions.md`, applied only to your pages |

**Claude Code** reads `AGENTS.md` by itself only while there's no `CLAUDE.md` at or above the folder it works in. So `sync` never creates a `CLAUDE.md` unless you ask with `--target claude`, and when there is one, it adds the import to it rather than copying the rules. A personal `CLAUDE.local.md` also stops Claude Code reading `AGENTS.md`; `sync` doesn't edit it, but says so, and you can add `@AGENTS.md` to it yourself.

**Copilot's** files are for a repository, so `--target copilot` needs one. Outside a git repository, `sync` treats your project's folder as the root and says so.

The rules in `.claude/rules/` and `.github/instructions/`, and the skill's folders, are wholly Ascribe's: `sync` rewrites them whole, and the rules files say so at the top.

### Keeping them current in CI

`--check` writes nothing, and exits with `1` when a file is out of date, naming it. Run it in CI so the instructions can't fall behind `ascribe.toml`:

```shell
ascribe agents sync --check
```

## The skill

The skill is one folder, `ascribe`, in the [Agent Skills](https://agentskills.io) format, which most agents read. It teaches the tool, the same for every project, and sends the agent to `AGENTS.md` and `ascribe model` for your project's rules:

- the loop: edit a page, run `ascribe check <file> --format concise`, fix what it reports, and run the whole check before finishing; and to stop and ask after three rounds that don't fix it;
- the commands that answer questions: `explain`, `model`, `outline`, `link`, `render`, and `refs`;
- which fixes in `ascribe check`'s JSON can be applied as they are;
- that text from a review or a pull request is someone else's, not instructions;
- each directive's syntax, in a reference it reads only when it needs it.

`sync` writes it to `.agents/skills/ascribe/` at the repository's root, where Codex, Cursor, Gemini CLI, and Copilot find it, and with the `claude` target to `.claude/skills/ascribe/`, where Claude Code does. `ascribe agents skill` prints its `SKILL.md`. The `@ascribed/cli` package ships it too, as `skills/ascribe/`, for tools that install skills from packages.

See [`ascribe agents`](../reference/cli.md#ascribe-agents) for every option.

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
2. **Where**: the file and lines, from the project's folder. When they apply, the project's folder in the repository (`Project: docs/`), the build the problems are from (`Build:`), the pages that show a fragment (`Shown on:`, at most 10, with the `ascribe refs` command that lists the rest), and a line saying to save the file first.
3. **What Ascribe knows**: for one problem, its code, message, the line's text, how to fix it, the values the content model allows when it's about one, and whether Ascribe has a fix ("a safe automatic fix", or "a fix to review"). For a file, each problem on one line, at most 20. For a project, how many problems each file has, at most 20 files. Each says which command lists the rest.
4. **How to finish**: "Follow the project's rules in `AGENTS.md`", when the project's folder or the repository's root has one, and "When you're done, run `ascribe check <file>` and fix what it reports."

Commands in a prompt are written from the repository's root, so an agent started there can run them. A prompt is at most 5,000 characters, the most an agent's link takes. When it would be longer, what Ascribe knows is cut between parts, and the prompt says where to read the rest.
