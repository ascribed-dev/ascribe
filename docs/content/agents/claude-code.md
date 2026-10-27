---
type: how-to
title: Claude Code
description: "Give Claude Code the skill, the MCP server, the stop hook, the language server, and commands in one install, with the Ascribe plugin."
---

Claude Code takes Ascribe in one of two ways: the plugin, which gives it everything in one install, or the project's own entries, written by `ascribe agents sync`. Use one, not both: with both, it hears about each problem twice.

## The plugin

The Ascribe plugin, in `plugins/ascribe/` in Ascribe's repository, gives Claude Code:

- **The skill**: how to write, check, and fix pages.
- **The MCP server**, whose tools are for an agent with no shell; one with a shell does as well with the commands.
- **The stop hook**, and **the language server**, which Claude Code starts for `.md` files and whose diagnostics it reads after each edit: they check for an agent that doesn't check its own work. The language server reports the editor's build, as the edit hook does, and only for edits Claude Code makes with its own tools, not through its shell.
- **Commands:** `/ascribe:check`, `/ascribe:new-page`, and `/ascribe:review`, which run the [named prompts](commands.md#cli-agents-prompt). `/ascribe:new-page` takes what the page is for: the agent chooses the page type from `ascribe model`, then runs the prompt.

Add it from a copy of the repository, in Claude Code in a terminal (its VS Code extension doesn't offer `/plugin`):

```shell
/plugin marketplace add ./plugins/ascribe
/plugin install ascribe@ascribe
```

The plugin runs the `ascribe` binary the VS Code extension would (see [Which `ascribe` it runs](../editor/index.md#which-ascribe-it-runs)), with an environment variable in place of the setting: the one `ASCRIBE_PATH` names, then the project's `node_modules/.bin/ascribe`, then `ascribe` on your path. So a project that pins its version with `npm install --save-dev @ascribed/cli` gets that version in every tool, whatever is installed globally.

The plugin has both kinds of manifest: `.claude-plugin/plugin.json`, which Copilot's CLI reads too, and the Agent Plugins `plugin.json`, which VS Code and Cursor read. To keep the language server's diagnostics out of the conversation on a project with many problems, see [the plugin's README](https://github.com/ascribed-dev/ascribe/tree/main/plugins/ascribe#readme); the stop hook still checks before Claude finishes.

## The project's entries

Claude Code doesn't start a plugin's language server in a cloud session. There, use the project's entries, which Claude Code reads from the repository:

```shell
ascribe agents sync --target claude --with-hook
```

That writes the skill where Claude Code reads it, `.claude/skills/ascribe/`, an import of `AGENTS.md` in `CLAUDE.md`, and [the hooks](hooks.md) and [the MCP server](mcp.md) in `.claude/settings.json` and `.mcp.json`.

Claude Code reads `AGENTS.md` by itself only while there's no `CLAUDE.md` at or above the folder it works in. So `sync` never creates a `CLAUDE.md` unless you ask with `--target claude`, and when there is one, it adds the import to it rather than copying the rules. A `CLAUDE.md` that imports `AGENTS.md` itself, with its own `@AGENTS.md` line, is left as it is, so it doesn't load the file twice. A personal `CLAUDE.local.md` also stops Claude Code reading `AGENTS.md`; `sync` doesn't edit it, but says so, and you can add `@AGENTS.md` to it yourself.
