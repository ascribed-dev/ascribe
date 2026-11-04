---
type: how-to
title: Set up the MCP server
description: "Give an agent with no shell Ascribe's answers as tools, through `ascribe mcp`."
---

`ascribe mcp` is a server for the [Model Context Protocol](https://modelcontextprotocol.io), which most agents' hosts speak. It offers the commands that answer questions as tools an agent calls, for an agent with no shell, or a host that asks you to approve each command it runs. An agent with a shell needs no server: the commands give the same answers.

In VS Code, the extension offers the server itself, so there's nothing to set up: see [Agents in the editor](../editor/agents.md#agents-in-vs-code). In Claude Code, [the plugin](claude-code.md) has it, and `ascribe agents sync --target claude --with-hook` writes it into `.mcp.json`.

## Add the server to your agent's host

Your agent's host starts the server, in the repository's root, and one server serves every project in it.

@steps
1. Open the file your host reads its servers from, such as `.mcp.json` at the repository's root for Claude Code, or `.cursor/mcp.json` for Cursor.
2. Add Ascribe to it:

   ```json
   {
     "mcpServers": {
       "ascribe": {
         "command": "ascribe",
         "args": ["mcp"]
       }
     }
   }
   ```

3. Start a new session, and check that the host lists the server's tools. If it doesn't, see [Troubleshooting](../troubleshooting.md#mcp-tools-missing).

This runs the `ascribe` on the host's path, which you install [on your path](../install.md#without-node), and it's what `sync` writes into `.mcp.json`: from a repository's root, no one command finds a project's own `ascribe` on every system.

### With `@ascribed/cli` in the project

When `@ascribed/cli` is in the `package.json` at the repository's root, as [Install Ascribe](../install.md#npm) sets up, the host can run that one instead. Put this in place of the `ascribe` server in step 2:

@variant {pm=npm}:
```json
{
  "mcpServers": {
    "ascribe": {
      "command": "npx",
      "args": ["--no-install", "ascribe", "mcp"]
    }
  }
}
```
@variant {pm=pnpm}:
```json
{
  "mcpServers": {
    "ascribe": {
      "command": "pnpm",
      "args": ["exec", "ascribe", "mcp"]
    }
  }
}
```
@end

@note {type=warning}: Keep `--no-install`. The npm package named `ascribe` isn't Ascribe, and when `npx` doesn't find `@ascribed/cli` in the folder or above it, it installs that package and runs it, without asking, since a host starts its servers with no terminal. With `--no-install`, `npx` stops with an error instead. `pnpm exec` never installs anything.

`npx` and `pnpm exec` look in the folder the host starts them in and the folders above it, never below, so a project whose `package.json` is in a folder under the root, such as `docs/`, needs `ascribe` on the path.

## What it offers

Its tools are the commands that answer questions; its resources are each directive's syntax, and each project's content model and rules; and its prompts are `new-page`, `fix`, and `review`, which many hosts show as slash commands. Every tool is read-only, so `ascribe_check` never runs [Vale](../checks/vale.md): `ascribe check --vale` in a shell, which your host asks you about, checks the prose. A tool's result is its command's JSON, whose schema is in the [JSON report contract](../contracts/json-reports.md), except for `ascribe_check`, `ascribe_refs`, and `ascribe_search`: with their default `response_format: "concise"`, the result is the command's concise text, one line each, and its `structuredContent` is that text as `{"concise": "…"}`. With `response_format: "detailed"`, they give the JSON. No tool gives an output schema, which would make the tool list about four times as long. [`ascribe mcp`](commands.md#cli-mcp) lists each tool and the command it runs, and the resources.
