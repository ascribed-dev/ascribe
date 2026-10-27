# Phase 3: The servers

Part of [Security review](README.md). Needs phase 0.

## Goal

Find what a client of the language server or the MCP server can make it read, write, or run, and whether a request can stop it.

## Questions

### The language server

- Every request that takes a URI or a path (`crates/ascribe-lsp/src/uri.rs`, `uri_to_path`, and the 25 places that use it): can a client name a file outside the workspace's projects and have it read, previewed, or returned in an answer?
- Edits the server returns (rename, `willRenameFiles`, code actions, `ascribe.addresses` entries): can content make one touch a file outside the project? The client applies them, but the user trusts what they see.
- The preview's assets (`crates/ascribe-lsp/src/preview.rs`): the answer lists source files for the client to turn into webview URIs. Can a page make the preview load a file outside the project?
- The kept base (redirects) and other caches: can a project make the server hold unbounded memory?

### The MCP server

- `crates/ascribe-cli/src/mcp/`: every tool's path arguments (`page`, `path`, `project`), resources (`ascribe://model/<project>`, `ascribe://instructions/<project>`), and prompts. Is each resolved from the server's working directory and held to the project, as the instructions say? Can `../`, an absolute path, or a symbolic link read another file?
- "No tool writes a file" (decision 56): verify for every tool, including the ones that return edits.
- A page's content reaches an agent through every tool's answer: does anything in an answer read as an instruction to the agent beyond the page's own text (prompt injection)? Ascribe can't stop a page from saying "ignore your instructions"; it can avoid adding authority to it, such as putting page text where the server's own guidance goes.
- Limits: an answer's size, a request that would resolve every build of a huge project.

### The agents' hook

- `crates/ascribe-cli/src/agents/hook.rs`: what it reads from the agent's input, what it runs, and what it prints back.

## Method

Drive each server with a scripted client (as the redirects spike drove `ascribe/page`), sending each candidate path and request, in a project with a symbolic link out of it and a secret file beside it.

## Output

Draft advisories for exploitable findings; `notes/phase-3.md` for hardening.
