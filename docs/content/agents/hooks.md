---
type: how-to
title: Check an agent's edits with hooks
description: "Have Claude Code, Codex, or GitHub Copilot check each file it writes, and the whole project before it finishes."
---

A hook is a command your agent runs at a point in its work. Ascribe's checks an agent's work without the agent having to remember: each file it writes, as it writes it, and the whole project before it finishes. One command, [`ascribe agents hook`](commands.md#cli-agents-hook), serves Claude Code, Codex, and GitHub Copilot, which share a hook format. Cursor's and Gemini CLI's formats differ, and Ascribe writes neither.

## Add the hooks

@steps
1. Put the release's binary on your path, with `brew install ascribed-dev/tap/ascribe` or the install script ([Install](../install.md#without-node) has both). `npm install -g @ascribed/cli` works too, but the npm package's launcher adds about 60 ms to each run, more than a check after an edit takes, so prefer the binary for hooks.
2. Write the entries that run the hooks, for the targets you name or already have:

   ```shell
   ascribe agents sync --target claude --with-hook
   ```

   [`ascribe agents sync`](commands.md#cli-agents-sync) lists the file and the entries for each target, and how it merges them into a file you share.
3. In Codex, trust the project's hooks when it asks: it runs them only once you have.

The entries run `ascribe` from your path. When your project pins `@ascribed/cli` in its `node_modules`, the hooks run that binary instead, found from the repository's root (`$CLAUDE_PROJECT_DIR` for Claude Code, `git rev-parse --show-toplevel` for Codex and Copilot), so an agent started in a subfolder finds it too. Two entries still run `ascribe` from your path, because neither can find the root on every system: the MCP server in `.mcp.json`, and Codex's hooks on Windows.

In Claude Code, [the plugin](claude-code.md) has the stop hook too: use the plugin or the project's entries, not both, or it hears about each problem twice.

## What the agent hears

- **After an edit,** the errors in the files the agent wrote, in the editor's build, and the advice in them that has a safe fix, with what to change and where. The hook makes no fix, so the agent makes the edit itself and sees every change. It says nothing when there's neither. The advice a project put off in [`ascribe.baseline`](../checks/baseline.md#for-agents) isn't the agent's to fix: it's left out, and counted.
- **Before the agent finishes,** the errors in every build of each project the working tree changes. While there are any, it keeps the agent working, once. Changes the agent has committed before it stops aren't in the working tree, so they aren't checked; check them in CI.

Neither stops an edit, writes a file, or holds the agent up: a check that takes too long is given up. [`ascribe agents hook`](commands.md#cli-agents-hook) says exactly what each reports, and how the check server it starts keeps a check after an edit to milliseconds.

## Prose, through Vale

A project that checks its prose with [Vale](../checks/vale.md) has the hooks run it only with `in-check = true` in `[checks.vale]`, as `ascribe check` does: after an edit on the files the agent wrote, and before it finishes on the whole project. Vale gets half of what the hook waits; when it takes longer, its alerts are left out, and the rest of the check reaches the agent as usual. Only Vale's error-level alerts reach the agent, as with every other check; with `max-level = "advice"` or `"warning"`, none do. A hook can't ask you before it runs a program, so it runs Vale only when `command` is a name found on your `PATH`, such as `vale`, never when it's a path to a program in the project: then it tells the agent, with the errors it reports, that the prose wasn't checked and that `ascribe check --vale` checks it.
