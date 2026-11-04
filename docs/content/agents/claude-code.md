---
type: how-to
title: Claude Code
description: "Give Claude Code the skill, the MCP server, the stop hook, the language server, and commands in one install, with the Ascribe plugin."
---

Claude Code takes Ascribe in one of two ways: the plugin, which gives it everything in one install, or the project's own entries, written by `ascribe agents sync`. Use one, not both: with both, it hears about each problem twice.

## The plugin

The Ascribe plugin, in `plugins/ascribe/` in [Ascribe's repository]({repo}), gives Claude Code:

- **The skill**: how to write, check, and fix pages.
- **The MCP server**, whose tools are for an agent with no shell; one with a shell does as well with the commands.
- **The stop hook**, and **the language server**, which Claude Code starts for `.md` files and whose diagnostics it reads after each edit: they check for an agent that doesn't check its own work. The language server reports the editor's build, as the edit hook does, and only for edits Claude Code makes with its own tools, not through its shell.
- **Commands:** `/ascribe:check`, `/ascribe:new-page`, and `/ascribe:review`, which run the [named prompts](commands.md#cli-agents-prompt). `/ascribe:new-page` takes what the page is for: the agent chooses the page type from `ascribe model`, then runs the prompt.

Ascribe's repository is the plugin's marketplace, so Claude Code adds it from GitHub, with no copy of the repository. In Claude Code in a terminal (its VS Code extension doesn't offer `/plugin`):

```shell
/plugin marketplace add ascribed-dev/ascribe
/plugin install ascribe@ascribe
```

From a copy of the repository, add the copy's folder in place of `ascribed-dev/ascribe`.

The plugin finds `ascribe` much as the VS Code extension does (see [Which `ascribe` it runs](../editor/index.md#which-ascribe-it-runs)), with an environment variable in place of the setting. It runs the one `ASCRIBE_PATH` names, then the project's `node_modules/.bin/ascribe`, then `ascribe` on your path, where the extension would run the binary it bundles. So a project that pins its version with `npm install --save-dev @ascribed/cli` gets that version in every tool, whatever is installed globally.

The plugin has both kinds of manifest: `.claude-plugin/plugin.json`, which Copilot's CLI reads too, and the Agent Plugins `plugin.json`, which VS Code and Cursor read. To keep the language server's diagnostics out of the conversation on a project with many problems, see [the plugin's README]({repo}/tree/main/plugins/ascribe#readme); the stop hook still checks before Claude finishes.

### Which version of the plugin

The plugin's version is the release of Ascribe it comes from, and its skill and commands are written for that release's `ascribe`. It runs whichever `ascribe` it finds, as above, so when a project pins an older `ascribe`, a command or a step of the skill can be one that version doesn't have. Added without a tag, the plugin is the one on `main`, which changes version at each release. To get the new version, run `/plugin marketplace update ascribe`, or turn on auto-update for the marketplace in `/plugin`, under Marketplaces, and Claude Code gets it on its own.

### Keep the plugin to your release

@available: ascribe planned

To keep the plugin to the release of the `ascribe` you run, add the marketplace at that release's tag: `v`, then the version `ascribe --version` prints.

```shell
/plugin marketplace add ascribed-dev/ascribe#v<version>
```

## The project's entries

Claude Code doesn't start a plugin's language server in a cloud session. There, use the project's entries, which Claude Code reads from the repository:

```shell
ascribe agents sync --target claude --with-hook
```

That writes the skill where Claude Code reads it, `.claude/skills/ascribe/`, an import of `AGENTS.md` in `CLAUDE.md`, and [the hooks](hooks.md) and [the MCP server](mcp.md) in `.claude/settings.json` and `.mcp.json`.

Claude Code reads `AGENTS.md` by itself only while there's no `CLAUDE.md` at or above the folder it works in. So `sync` never creates a `CLAUDE.md` unless you ask with `--target claude`, and when there is one, it adds the import to it rather than copying the rules. A `CLAUDE.md` that imports `AGENTS.md` itself, with its own `@AGENTS.md` line, is left as it is, so it doesn't load the file twice. A personal `CLAUDE.local.md` also stops Claude Code reading `AGENTS.md`; `sync` doesn't edit it, but says so, and you can add `@AGENTS.md` to it yourself.
