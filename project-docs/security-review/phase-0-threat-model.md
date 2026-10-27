# Phase 0: Threat model

Part of [Security review](README.md). Needs nothing. Written by an agent, settled by the maintainer.

## Goal

One page every later phase reads: what's worth protecting, who might attack it, where untrusted input crosses into Ascribe, and what each entry point is allowed to do.

## What it holds

- **Assets:** the user's files outside the project; the user's machine (running commands); secrets in the environment (tokens in CI, a `GITHUB_TOKEN`, the user's git credentials); the published site's readers (script running in their browser); the editor's session; Ascribe's releases (a package that installs malware); the availability of the language server and MCP server.
- **Actors:** a contributor sending a pull request from a fork; the author of a repository someone opens in the editor; the owner of a remote `[sources]` repository; the producer of an OpenAPI spec; an agent driven by a prompt someone else wrote (prompt injection through a page's content); a commenter on a pull request; a web page a link check fetches; someone who compromises a dependency or a CI step.
- **Trust boundaries,** README decision 3, each with the processes on either side: the CLI in CI; the CLI on a writer's machine; the language server inside the editor; the MCP server under an agent; the extension's webviews; the published site and its review overlay; the release pipeline.
- **Entry points,** one row each: what reads it, what it may do, and what would be a breach. At least: source pages and fragments; `ascribe.toml` and its sibling files (`ascribe.permalinks`, `ascribe.addresses`, `ascribe.nav`, `ascribe.baseline`, `ascribe.examples.toml`, `ascribe.lock`); code a snippet reads; an OpenAPI spec; a remote source; git history at a base revision; the language server's requests and file events; the MCP server's tool calls, resources, and prompts; the agents' hook's input; the output of lychee, afdocs, Vale, and `git`; review comments from GitHub; the extension's webview messages; CI's triggers, artifacts, and caches.

## Method

Build the entry point table from [the surface map](../surface.md) and the code, not from memory: each command, each language server request, each MCP tool, each file Ascribe reads, each program it runs. Mark each entry point with the phase that reviews it.

## Output

`project-docs/security-review/threat-model.md`, and a decision file recording the maintainer's settlement of the trust boundaries. Nothing else changes.
