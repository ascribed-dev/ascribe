---
type: reference
title: Agents in the editor
description: "Prompting your agent to fix a problem from VS Code, and the tools and skill an agent in VS Code gets."
---

The extension prompts your agent about the problems it shows, and gives an agent in VS Code the tools of Ascribe and of the editor.

## Prompt agent

On a problem, the lightbulb also offers **Prompt agent: fix this problem**, after the problem's quick fixes. It builds a prompt for your AI agent about the problem, and copies it, or opens your agent with it filled in, as `ascribe.agents.promptTarget` says. **Ascribe: Prompt Agent to Fix This File** and **Ascribe: Prompt Agent to Fix This Project** do the same for every problem in the file or the project. Nothing is sent: you read the prompt and send it. See [Prompt your agent](../agents/prompts.md).

## Agents in VS Code
@id {permalink=true}: agents-in-vs-code
@summary: An agent in VS Code, such as GitHub Copilot in agent mode, gets Ascribe's tools with nothing to set up: the extension offers `ascribe mcp` as an MCP server named Ascribe.

An agent in VS Code, such as GitHub Copilot in agent mode, gets Ascribe's tools with nothing to set up. In a workspace with an Ascribe project, the extension offers [`ascribe mcp`](../agents/mcp.md) as an MCP server, named **Ascribe** in VS Code's list of MCP servers. It runs the binary the first project uses (see [Which `ascribe` it runs](index.md#which-ascribe-it-runs)), in the workspace folder, and VS Code starts it when an agent first needs it. When the workspace's projects use different binaries, the project's output says which one the server runs. Its prompts, `new-page`, `fix`, and `review`, appear as slash commands in the chat.

An agent that writes files without opening them gets their problems once the project's language server is running: a running server follows changes on disk, and the Problems panel shows them, where Copilot can read them after its edits. A change on disk doesn't start a server, so open one of the project's files, or choose it in **Switch Project**, first. Like the panel, they're the editor's build's; the [hooks](../agents/hooks.md) and `ascribe check` cover every build. See [When servers start](settings.md#when-servers-start).

The extension also gives agents three tools of its own, for what only the editor knows: text that isn't saved, and the review that's open. Name one in chat with `#`, or let the agent pick it.

@include: ../_generated/editor-tools.md

`#ascribe_editor_problems` answers with the JSON `ascribe check --format json` writes, from the language server, so it includes unsaved edits. When a file changed since the server last checked it, the tool waits for the server, up to a second, and says which version of the document its answer is for. Like the Problems panel, it covers the [editor's build](problems.md#diagnostics) only; `ascribe check` checks every build. It never starts a language server: a project whose server isn't running gets a line saying so.

The review tools answer while [review](review.md) is on, and say so in one line when it's off; they never turn it on. `#ascribe_review_threads` gives each open thread's comments as data, under the same sentence a [prompt](../agents/commands.md#what-a-prompt-says) puts over them, so the agent doesn't take them as instructions. None of the tools changes a file.

An agent that runs `ascribe` runs the binary the editor runs. When `ascribe` on the window's path is another binary or none, the extension puts the folder of the binary each workspace folder's first project uses first on the path of that folder's terminals; a terminal that was open before asks to be relaunched. [Prompt agent](../agents/prompts.md)'s prompts then name the binary by its path, for an agent that doesn't work in the window's terminals, such as Claude Code's extension.

Chat has [Ascribe's skill](../agents/instructions.md#the-skill), which teaches an agent the loop of editing a page and checking it, with nothing to set up. The extension ships it, and offers it while no workspace folder has a copy from [`ascribe agents sync`](../agents/instructions.md) in `.agents/skills/ascribe/`, `.claude/skills/ascribe/`, or `.github/skills/ascribe/`. Once one does, chat lists that copy, the project's, instead. VS Code reads those folders only at the top of a workspace folder, and `sync` writes at the repository's root, so in a workspace opened below the root, chat keeps the extension's copy after a `sync`.
