# Phase 3 notes: the servers

Hardening notes from [phase 3](../phase-3-servers.md), reviewed on 27 October 2026 by one agent and checked by a second, adversarial one. Method, as the phase asked: scripted clients for the language server and the MCP server over standard input and output, driven against a copy of `examples/quill` with links out of it and a secret file beside it, every request and tool sent each candidate path form, the hook fed each harness's input, and `ascribe preview`'s server probed over HTTP.

**Two private findings exist: one medium, one low** (README decision 1). The medium one is a stall of the language server that a project's content causes; the low one is in `ascribe preview`'s server. Each is with the maintainer with its input. A third candidate, the editor's preview listing a linked asset as servable, is a sink of phase 1's first private finding and is recorded there. The notes below are the rest.

## H1: a diagnostic's message can carry a newline from page text, which the prompts and the hook print as a line of their own

- **Where:** `crates/ascribe-cli/src/report/concise.rs` (`head`, the `file:line: [CODE] message` form), `crates/ascribe-check/src/prompt.rs` (`Message:`), and the hook's `edit_text` and `stop_text` in `crates/ascribe-cli/src/agents/hook.rs`; a message that quotes a heading's text (ASC192) keeps a setext heading's line break.
- **What it costs:** page text starts a fresh line inside what an agent reads as Ascribe's own list, in `--format concise`, `--format prompt`, and the hook's answer. It stays inside the message's backticks, so it doesn't yet read as an instruction of Ascribe's; this is the one seam the phase found where page text could reach the server's own voice (the tools' answers are the commands' JSON, the guidance is static text, and the prompts name memo files by path without their text).
- **Recommendation:** replace a line break in a diagnostic's message with a space where messages are made, or in the three printers.
- **Size:** small.

## H2: `workspace/willRenameFiles` accepts a move into a linked folder that `ascribe rename` refuses

- **Where:** `crates/ascribe-lsp/src/refactor.rs` (`will_rename`, which checks the new path's text), against `crates/ascribe-lsp/src/rename.rs`, whose command path refuses a path through a link.
- **What it costs:** the server returns the link rewrites for a move the editor then makes through the link, outside the project. The person chose the folder in the explorer, so they see it; the two paths disagreeing is the fragility. `ascribe/edit`'s `newPage` into a linked folder is the same family.
- **Recommendation:** the same link check on the new path in `will_rename`, returning an empty edit.
- **Size:** small.

## H3: `ascribe.openFile` forwards any `file:` URI to `window/showDocument`

- **Where:** `crates/ascribe-lsp/src/server.rs`, `execute_command`, which checks only that the URI is a path.
- **What it costs:** nothing today: the server makes the lenses that carry the command, and a client that can call `executeCommand` can open a file itself. It would matter if a `command:` link ever reached a trusted Markdown surface, which phase 4 found none of.
- **Recommendation:** refuse a URI outside the project's root or a source's folder.
- **Size:** small.

## H4: the index sees the files under a linked folder, so a link out is an existence oracle

- **Where:** the source index's walk (`crates/ascribe-resolve/src/fs.rs`), which decision 79(b) has follow a link out of the content root on purpose, as SPEC §2.1 says, and report its files as `source-unreadable`; `ascribe_link` then answers that a file under the link exists, by name.
- **What it costs:** an agent, or the report, learns whether a file exists anywhere a committed link points, and its name; no content, since the read refuses the link. The scale is the maintainer's to weigh: a link to `/` lists every `.md` on the disk in the diagnostics (3,748 here), in `ascribe check`, the MCP `ascribe_check` tool, and the `fix` prompt, once per run.
- **Recommendation:** none while 79(b) stands; if the listing is unwanted, report a linked folder once, at the link, instead of each file under it.
- **Size:** small.

## H5: an MCP answer has no size bound

- **Where:** `crates/ascribe-cli/src/mcp/tools.rs`: `call_check` with `response_format: detailed` passes no limit; `call_resolve` returns a whole page, up to decision 79's 16 MiB.
- **What it costs:** one page with 30,000 broken links made a `detailed` check answer 22 MB. The host that puts the answer in the model's context pays for it; the server holds it once.
- **Recommendation:** cap `detailed` as `concise` is capped, with `truncated: true` and the command that lists the rest, and say so in the tool's description.
- **Size:** small.

## H6: the `new-page` prompt repeats the agent's `title` argument verbatim, line breaks included

- **Where:** `crates/ascribe-cli/src/agents/prompts.rs`, `new_page`.
- **What it costs:** nothing new in authority, since the agent wrote the argument; a title with a line break makes a prompt whose `ascribe search` line spans lines. `ascribe/edit`'s `newPage` already refuses one.
- **Recommendation:** refuse a `title` with a line break, as `edit/start.rs` does.
- **Size:** small.

## H7: a check's cost is quadratic in a page's links

- **Where:** the page-level checks; unprofiled. Measured: 300 links 0.1 s, 3,000 1 s, 15,000 20 s, 30,000 82 s, the same whether the links resolve or not, in the CLI, the MCP server, and the hook.
- **What it costs:** a slow request the servers recover from, not a hang; a realistic page is far below it. A crafted page of 30,000 link lines holds a tool call or a hook for over a minute.
- **Recommendation:** profile the check on such a page and make it linear, or cap the links a page is checked for with a diagnostic.
- **Size:** medium.

## H8: the MCP server answers about any Ascribe project on disk named by an absolute path

- **Where:** `crates/ascribe-cli/src/answer.rs`'s `locate`, which finds `ascribe.toml` from any path an argument names; decision 77 bounded a project's sources to the folder the server was started in, not which project a tool resolves. The docs call the working directory the default, not a boundary. A path that isn't in an Ascribe project is refused.
- **What it costs:** an agent on a host with no shell reaches another project's pages, model, and search, and `git` in that project's repository through `ascribe_diff`, which decision 76's options bound. An agent with a shell could read the files itself. The maintainer may rule it by design; the threat model's row says the tools are "held to the project".
- **Recommendation:** the maintainer's ruling; if closed, hold a tool's `page`, `path`, and `project` to the working directory, as `resources/list` already is.
- **Size:** small to medium.

## Checked, nothing found

- **Every standard language server request with a URI outside the project** (hover, definition, completion, document links, code lenses, folding, formatting, semantic tokens, references, prepare rename, code actions, inlay hints), for a linked page, a file under a linked folder, `../secret.txt`, `/etc/passwd`, another project's page, an unnormalised path, and a memo: every answer `null`. A linked file classifies as an asset, never a source; an outside path as nothing.
- **Ascribe's own requests with those URIs:** the project-wide ones answer about the project for a URI inside its folder and empty for one outside; `preview` of an outside document gives builds and a problem, no page; `review/baseFile` of an outside or linked file gives no text. `didOpen` of an outside file keeps the text but reads nothing from disk; a page is read from disk only through `DiskFs`, which refuses a link.
- **Watcher events for a path outside the project** (`/etc`, `/usr/share`, `/`): skipped before any walk, in 0.1 s each. Inside the project through a link is the private finding.
- **Rename's edits:** every file in the `changes` map is the project's; other files are read only through `FileSystem::read_file`. `ascribe/edit`'s `newPage` outside the content root is refused.
- **A revision written as a `git` option** through `ascribe/review/setBase`, the MCP `ascribe_diff`, `ascribe_addresses_move`, and the `review` prompt's `base`: each refused as not a revision; `commit_of` passes `--end-of-options`.
- **Document links** (decision 75): `file:` targets into the project and the `#L` form only.
- **The kept base and the caches:** keyed by the project's own pages and builds, bounded by its size; `setBase` with the same base reuses one read. A client opening unbounded documents is the client's own doing.
- **A request that stops the server:** every handler runs under `catch_unwind`, and a compile-time check holds the panic strategy; a malformed message ends the session through the protocol's own error. The exception is phase 1's private finding, which isn't a panic.
- **The language server's Vale question** (decision 76 d): a client without `window.showMessage` runs nothing and the advice says so; a dismissed question runs nothing; "Run it" runs it; a restarted server asks again.
- **The MCP server never starts Vale** (76 b): with a project-path command and with a bare name on the `PATH`, `ascribe_check` and the `fix` prompt answer `prose-not-checked` and nothing runs. **The hook** (76 c): a relative command never runs, a bare name on the `PATH` runs, and an absolute path outside the project's folder runs, which is the model author's choice by the rule's definition; the hook's `command::detect` only reads `package.json` and lockfiles.
- **No MCP tool writes a file** (decision 56): after every probe `git status` showed only the reviewer's own files; `ascribe_fmt` returns `written: false`, and `ascribe_rename` and the `addresses` tools return edits and text.
- **Every MCP path argument** (`page`, `path`, `paths`, `from`, `target`, `project`, `old`, `new`) with `..`, absolute, percent-encoded, and linked forms, and the resources with `../` and absolute forms: each refused, naming the project, the link, or the missing `ascribe.toml`; `resources/list` names only projects at or below the working directory. The one way out is H8.
- **Memos whose `about` names a file outside the project:** a diagnostic each, nothing read.
- **The hook's input:** `file_path` of an outside, linked, or memo file, a Codex patch naming an outside file, `cwd` of `/etc` or `/tmp`, and input that isn't JSON: nothing printed, or the "can't read the hook's input" error. A path in another project is checked in that project, which is the harness's own folder choice. The mailbox's files are `0600` under the user's own cache folder.
- **`ascribe preview`'s server:** `..` in every encoding, `//etc/passwd`, `%00`, a request into `.ascribe/preview/` itself, a missing or foreign `Host`, `POST` and `OPTIONS`, a 20 KB header line, and a request with no line end: `404`, `403`, `405`, cut at `MAX_HEAD`, or timed out on its own thread; no CORS headers, so another origin can't read an answer; a rendered page's policy blocks a planted inline script. A link planted in the folder before any archive exists is refused. One beside a manifest is the low private finding.
