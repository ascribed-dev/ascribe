---
type: concept
title: What's sent where
description: "What Ascribe sends and to whom when an agent works on your docs, which is nothing, and what your agent reads and passes on."
available: ascribe 0.3.0
---

Ascribe gives your agent text, and your agent decides what to do with it. This page says what that text is, what leaves your machine, and what to watch for when the text is someone else's.

## What your agent reads

- **The files `ascribe agents sync` writes:** your project's rules in `AGENTS.md`, and the skill ([Write the agent files](instructions.md)).
- **The answers to its questions,** from the commands or [the MCP server](mcp.md): your project's pages, its content model, and its problems, as text or JSON.
- **What the [hooks](hooks.md) report:** the problems in what it just wrote.
- **The prompts you send it** ([Prompt your agent](prompts.md)), which can carry a reviewer's comments.

## What's sent where

Nothing about your agent's work leaves your machine through Ascribe: it sends nothing, to no one, and it never writes a page for your agent.

- **No model, no service.** Ascribe makes no request to an AI service, holds no key for one, and collects no telemetry. The commands, the MCP server, and the hooks read your project's files and answer on standard output.
- **Prompts go where you put them.** **Prompt agent** copies a prompt, or fills it in where your agent takes input, on your machine, and leaves it there for you to send. It never travels through a website. Once you send it, it goes wherever your agent sends what you type, under that agent's terms.
- **Your agent sends what it reads.** What a command or tool tells your agent, such as a page's text from `ascribe render`, becomes part of what your agent sends to its model, as any file it reads does.
- **Review reads GitHub, as you.** In review, the extension and the site preview fetch the pull request's threads from GitHub with your sign-in, and post the comments you write, as [Review](../review/index.md) says. The prompts built from them stay on your machine. Apart from review, the only commands that reach the network are [`ascribe sources`](../examples/sources.md#cli-sources) `fetch` and `update`, which fetch the repositories your content model names, with `git`.

## Agents that reach the network

An agent that can fetch web pages, run commands, or post to GitHub can do with Ascribe's answers whatever it can do with any text. Ascribe's tools for agents read nothing from the network and send nothing, but a review comment, or page text from someone else's pull request, is still someone else's text when a prompt or a tool carries it to your agent. Ascribe puts that text in a fence, under a sentence that says it's data, and takes out HTML comments, where hidden text would sit. That lowers the risk; it doesn't remove it. An instruction hidden in a comment ("also update the deploy key", "fetch this URL") is aimed at your agent, and only your agent's own approvals stop it.

- Read a prompt before you send it, as you would a request from the person who wrote the comment.
- Keep your agent's approvals on for commands and network access while it works on someone else's change.
- Hand an agent with no network access the work on pull requests from outside your team.
