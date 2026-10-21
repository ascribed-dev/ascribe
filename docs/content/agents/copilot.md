---
type: how-to
title: GitHub Copilot
description: "Set Ascribe up for GitHub Copilot, in VS Code and as the cloud agent that works on GitHub."
---

Copilot reads `AGENTS.md` and the skill that [`ascribe agents sync`](instructions.md) writes, and, with `--target copilot`, rules in `.github/instructions/` applied only to your pages. What else it needs depends on where it runs.

Copilot's files are for a repository, so `--target copilot` and `--cloud` need one. Outside a git repository, `sync` treats your project's folder as the root and says so.

## In VS Code

The Ascribe extension gives Copilot [the MCP server](../editor/agents.md#agents-in-vs-code), whose prompts `new-page`, `fix`, and `review` appear as slash commands in its chat. There's nothing more to set up.

## The cloud agent

Copilot's cloud agent works on GitHub, with no editor: you assign it an issue and it opens a pull request. It reads the skill from `.agents/skills/` and the hooks from `.github/hooks/`, as other agents do, but it needs `ascribe` installed, and it reads MCP servers only from the repository's settings.

@steps
1. Write the rules, the [hooks](hooks.md), and what the cloud agent needs:

   ```shell
   ascribe agents sync --target copilot --with-hook --cloud
   ```

   [`--cloud`](commands.md#cli-agents-sync) adds steps that install Ascribe to `.github/workflows/copilot-setup-steps.yml`, the job the cloud agent runs before it starts, and prints the MCP server's JSON.
2. Paste the JSON into the repository's **Settings → Copilot → MCP servers**. It's a local server running `ascribe mcp` with each of its tools allowed. If you can't change the settings, run `sync` again with `--agent`: it writes `.github/agents/ascribe-docs.md` instead, a custom agent for documentation work that carries the server, which you pick when you assign the cloud agent a task.
3. Merge the files. GitHub reads them from the default branch, so they take effect once they're merged.

`--check` covers the setup steps and the custom agent, and `sync` keeps them up to date without `--cloud` once they exist.

## How the setup steps install Ascribe

The steps install Node.js and `@ascribed/cli` and run `ascribe --version`. When your project pins `@ascribed/cli` in a `package.json`, they install that folder's dependencies by its lockfile, without changing it (`npm ci`, `pnpm install --frozen-lockfile`, or `yarn install` with `--frozen-lockfile` for Yarn 1 and `--immutable` for later versions), and put its `node_modules/.bin` on the path; the MCP server then runs the pinned binary, found from the repository's root with `git` as the hooks find it. Otherwise they install the version of `ascribe` that wrote them, globally. Yarn's Plug'n'Play, the default from Yarn 2 on, writes no `node_modules/.bin`, so with it they install the pinned version globally, and `sync` stops if that version isn't one npm can install, such as `workspace:*`.

Projects in one repository that install the same way share one block of steps. When they install Ascribe in different ways, so the steps have a block for each, the JSON and the custom agent carry a server for each block, under names told apart as the blocks are: `ascribe` for a global install, and `ascribe-docs` for one from `docs/package.json` (`ascribe-pinned` from the root's). Syncing one project prints them all, as one document to paste in place of any `ascribe` server pasted before.

Ascribe writes no prompt files for Copilot (`.github/prompts/`): the cloud agent doesn't load them, and in VS Code the MCP server's prompts do the same.
