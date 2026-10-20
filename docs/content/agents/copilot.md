---
type: how-to
title: GitHub Copilot
description: "Set Ascribe up for GitHub Copilot, in VS Code and as the cloud agent that works on GitHub."
available: ascribe 0.3.0
---

In VS Code, Copilot reads `AGENTS.md`, the rules from `--target copilot`, and the skill, and the Ascribe extension gives it [the MCP server](../editor/agents.md#agents-in-vs-code), whose prompts `new-page`, `fix`, and `review` appear as slash commands in its chat. There's nothing more to set up.

Copilot's cloud agent works on GitHub, with no editor: you assign it an issue and it opens a pull request. It reads the skill from `.agents/skills/` and the hooks from `.github/hooks/`, as other agents do, but it needs `ascribe` installed, and it reads MCP servers only from the repository's settings. `--cloud` writes the rest:

```shell
ascribe agents sync --target copilot --with-hook --cloud
```

- **The setup steps.** In `.github/workflows/copilot-setup-steps.yml`, the job the cloud agent runs before it starts, steps that install Node.js and `@ascribed/cli` and run `ascribe --version`. When your project pins `@ascribed/cli` in a `package.json`, they install that folder's dependencies by its lockfile, without changing it (`npm ci`, `pnpm install --frozen-lockfile`, or `yarn install` with `--frozen-lockfile` for Yarn 1 and `--immutable` for later versions), and put its `node_modules/.bin` on the path. Otherwise they install the version of `ascribe` that wrote them, globally. Yarn's Plug'n'Play, the default from Yarn 2 on, writes no `node_modules/.bin`, so with it they install the pinned version globally, and `sync` stops if that version isn't one npm can install, such as `workspace:*`. `sync` creates the workflow when there's none, and otherwise adds its steps, between markers, at the end of the `copilot-setup-steps` job's steps, leaving your steps as they are. Projects in one repository that install the same way share one block.
- **The MCP server.** No file can change a repository's settings, so `sync` prints the server's JSON, a local server running `ascribe mcp` with each of its tools allowed (for a pinned project, the pinned binary, found from the repository's root with `git` as the hooks find it), to paste into the repository's **Settings → Copilot → MCP servers**. If you can't change the settings, add `--agent`: `sync` writes `.github/agents/ascribe-docs.md` instead, a custom agent for documentation work that carries the server, which you pick when you assign the cloud agent a task. When the repository's projects install Ascribe in different ways, so the setup steps have a block for each, the JSON and the custom agent carry a server for each block, under names told apart as the blocks are: `ascribe` for a global install, and `ascribe-docs` for one from `docs/package.json` (`ascribe-pinned` from the root's). Syncing one project prints them all, as one document to paste in place of any `ascribe` server pasted before.

GitHub reads all of these from the default branch, so they take effect once they're merged. `--check` covers the setup steps and the custom agent, and `sync` keeps them up to date without `--cloud` once they exist.

Ascribe writes no prompt files for Copilot (`.github/prompts/`): the cloud agent doesn't load them, and in VS Code the MCP server's prompts do the same.
