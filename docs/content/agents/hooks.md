---
type: how-to
title: Check an agent's edits with hooks
description: "Have Claude Code, Codex, or GitHub Copilot check each file it writes, and the whole project before it finishes."
available: ascribe 0.3.0
---

A hook is a command your agent runs at a point in its work. Ascribe's checks an agent's work without the agent having to remember: each file it writes, as it writes it, and the whole project before it finishes. One command serves Claude Code, Codex, and GitHub Copilot, which share a hook format, with the agent as its argument, `claude-code`, `codex`, or `copilot`:

- **After an edit**, `ascribe agents hook <agent>` checks the files the agent wrote, as `ascribe check <file> --editor-build` does, and adds their errors to what the agent reads next, at most 10, with the build it checked. It adds the advice in them that has a safe fix too, each with what its fix changes and where, such as "Add `new.md` to `guides/`'s pages in ascribe.nav" for a page the [navigation file](../navigation/index.md) doesn't place; the hook makes no fix, so the agent makes the edit itself and sees every change. It says nothing when there's neither, and leaves warnings and other advice out, which would interrupt every edit.
- **Before the agent finishes**, `ascribe agents hook <agent> --event stop` checks every build of each project whose pages or `ascribe.toml` the working tree changes, anywhere in the repository. While there are errors, or a project can't be checked at all, it keeps the agent working, and tells it why. It asks once: when the agent is already continuing because of it, it lets the agent stop. Changes the agent has committed before it stops aren't in the working tree, so they aren't checked; check them in CI.

@available: ascribe 0.3.0
A project that checks its prose with [Vale](../checks/vale.md) has the hooks run it only with `in-check = true` in `[checks.vale]`, as `ascribe check` does: after an edit on the files the agent wrote, and before it finishes on the whole project. Vale gets half of what the hook waits; when it takes longer, its alerts are left out, and the rest of the check reaches the agent as usual. Only Vale's error-level alerts reach the agent, as with every other check; with `max-level = "advice"` or `"warning"`, none do. A hook can't ask you before it runs a program, so it runs Vale only when `command` is a name found on your `PATH`, such as `vale`, never when it's a path to a program in the project: then it tells the agent, with the errors it reports, that the prose wasn't checked and that `ascribe check --vale` checks it.

Neither stops an edit, writes a file, or holds the agent up: a check that takes too long is given up. The first hook in a project starts a check server in the background, which keeps the project loaded, so a check after an edit takes milliseconds on a project of thousands of pages, and stops itself after 10 idle minutes. See [`ascribe agents hook`](commands.md#cli-agents-hook).

`ascribe agents sync --with-hook` writes the entries that run them, with the targets you name or already have:

```shell
ascribe agents sync --target claude --with-hook
```

[`ascribe agents sync`](commands.md#cli-agents-sync) lists the file and the entries for each target, and how it merges them into a file you share.

The entries run `ascribe` from your path. Put the release's binary on your path, with `brew install ascribed-dev/tap/ascribe` or the install script ([Install](../install.md#without-node) has both), or install it with `npm install -g @ascribed/cli`; the npm package's launcher adds about 60 ms to each run, more than a check after an edit takes, so prefer the binary for hooks. When your project pins `@ascribed/cli` in its `node_modules`, the hooks run that binary instead, found from the repository's root (`$CLAUDE_PROJECT_DIR` for Claude Code, `git rev-parse --show-toplevel` for Codex and Copilot), so an agent started in a subfolder finds it too. Two entries still run `ascribe` from your path, because neither can find the root on every system: the MCP server in `.mcp.json`, and Codex's hooks on Windows. Codex asks you to trust a project's hooks before it runs them.

For Claude Code, use the plugin or the project's entries, not both: with both, it hears about each problem twice.
