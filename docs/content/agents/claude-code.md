---
type: how-to
title: Claude Code
description: "Give Claude Code the skill, the MCP server, the stop hook, the language server, and commands in one install, with the Ascribe plugin."
available: ascribe 0.3.0
---

## The plugin

The Ascribe plugin, in `plugins/ascribe/` in Ascribe's repository, gives Claude Code everything in one install:

- **The skill**, for every agent: how to write, check, and fix pages.
- **The MCP server**, whose tools are for an agent with no shell; one with a shell does as well with the commands.
- **The stop hook**, and **the language server**, which Claude Code starts for `.md` files and whose diagnostics it reads after each edit: they check for an agent that doesn't check its own work. The language server reports the editor's build, as the edit hook does, and only for edits Claude Code makes with its own tools, not through its shell.
- **Commands:** `/ascribe:check`, `/ascribe:new-page`, and `/ascribe:review`, which run the [named prompts](commands.md#cli-agents-prompt). `/ascribe:new-page` takes what the page is for: the agent chooses the page type from `ascribe model`, then runs the prompt.

Add it from a copy of the repository, in Claude Code in a terminal (its VS Code extension doesn't offer `/plugin`):

```shell
/plugin marketplace add ./plugins/ascribe
/plugin install ascribe@ascribe
```

The plugin has both kinds of manifest: `.claude-plugin/plugin.json`, which Copilot's CLI reads too, and the Agent Plugins `plugin.json`, which VS Code and Cursor read. Claude Code doesn't start a plugin's language server in a cloud session; there, use the project's entries from `--with-hook`, which Claude Code reads from the repository. To keep the language server's diagnostics out of the conversation on a project with many problems, see [the plugin's README](https://github.com/ascribed-dev/ascribe/tree/main/plugins/ascribe#readme); the stop hook still checks before Claude finishes.
