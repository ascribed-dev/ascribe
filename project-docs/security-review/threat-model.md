# Threat model

Phase 0 of the [security review](README.md). One page every later phase reads: what's worth protecting, who might attack it, where untrusted input crosses into Ascribe, and what each entry point is allowed to do. Written by an agent on 27 October 2026 from [the surface map](../surface.md) and the code, for the maintainer to settle (README decision 7).

## Assets

What a breach costs, from the worst down:

1. **The user's machine.** A command run that the user didn't ask for: through `git`, Vale, lychee, afdocs, the shell `ascribe observe` starts, the binary the editor picks, or a process the extension starts.
2. **Secrets in the environment.** `GITHUB_TOKEN` and `NPM_TOKEN`-class secrets in CI; the user's git credentials and VS Code's GitHub sign-in on a writer's machine; the Marketplace token in the release pipeline.
3. **The user's files outside the project.** Read: shown in a preview, returned from a server, copied into an output, or sent to GitHub. Written or deleted: by a build's output directory, `fmt`, `rename`, `check --fix`, `sources fetch`, `templates eject`, `agents sync`, or an edit the language server returns.
4. **Ascribe's releases.** A package, binary, or extension that carries code the maintainer didn't write, reaching every installer.
5. **The published site's readers.** Script running in their browser from a source other than the page's author: a review comment, a help address, an element's reading of the URL.
6. **The editor's session.** Script in a webview acting as the extension: opening files, running `git`, or posting to the extension with messages it trusts.
7. **The availability of the long-running processes.** The language server, the MCP server, the hook's check server, and `ascribe preview`'s server: a crash or a hang a project's content causes, which takes the editor's features or the agent's tools with it.

## Actors

Each with what they control and the asset they can reach:

| Actor | Controls | Reaches |
|---|---|---|
| A contributor sending a pull request from a fork | Every file of the project in CI: pages, `ascribe.toml` and its sibling files, `memos/`, `sources/` copies, `ascribe.lock`, `.vale.ini`, an OpenAPI spec, workflow-adjacent files | CI's secrets and tokens, the runner, a cache a later run restores |
| The author of a repository someone opens in the editor | The same files, plus `.vscode/settings.json`, `.git/config` of the checkout, `node_modules/.bin/ascribe`, `.ascribe/dev.json`, a planted bare repository, links | The writer's machine, their sign-in, files outside the project, the editor's session |
| The owner of a remote `[sources]` repository | What `git` fetches: files, links, submodule and LFS pointers, hooks in the tree | The copies under `sources/`, and through them what snippets show and what a build writes |
| The producer of an OpenAPI spec | A YAML or JSON file, inside or outside the project's folder | The parser's time and memory, `@operation`'s values on a page, an `api-example-*` diagnostic's text |
| An agent driven by a prompt someone else wrote | The MCP server's tool calls and the hook's input; what a page says to the agent | Whatever the agent's host lets it do with the server's answers |
| A commenter on a pull request | A review thread's body and its position | The review overlay, `ascribe diff`'s HTML report, the editor's preview and its comment threads |
| A web page a link check fetches | What lychee and afdocs read, and so their reports | A diagnostic's text, the report's time and memory, the link cache |
| Someone who compromises a dependency or a CI step | A crate, an npm package, a GitHub action's commit, a workflow's input | The releases, CI's secrets |

The person running Ascribe on their own project, and the content model they wrote, are trusted (README decision 3). An author writing HTML on a page is trusted where the site output passes it through (README decision 4).

## Trust boundaries

Each boundary, the process on each side, and what crosses it. README decision 3 names who's untrusted; this says where that input arrives.

| Boundary | Trusted side | Untrusted side | What crosses |
|---|---|---|---|
| **B1: The CLI in CI** | The workflow the maintainer wrote, the runner's token | A fork's pull request: the whole checkout | Every file the commands read; `git` run in the checkout; what lychee, afdocs, and Vale print |
| **B2: The CLI on a writer's machine** | The writer and their model | A repository they cloned and haven't read; a remote source; a fetched spec | The same files; `ascribe.examples.toml`'s shell lines, which only `ascribe observe` runs, after asking at a terminal |
| **B3: The language server inside the editor** | The editor, after Workspace Trust | A repository opened before trust; a client sending requests | File events and request parameters (URIs, positions, names); the content it reads; `[checks.vale] command` |
| **B4: The MCP server under an agent** | The host that approves tools | The agent, and the pages it reads | Tool arguments (paths, revisions, names), resource addresses, prompt arguments; page text in every answer |
| **B5: The agents' hook** | The harness | The agent's input JSON and the files it edited | `tool_input` paths, the patch text (Codex), the `command` it runs |
| **B6: The extension's webviews** | The extension host | The preview's page (which draws a page's raw HTML), the site preview's frame, a review comment | `postMessage` in both directions; webview URIs for assets; `command:` links in hovers |
| **B7: The published site and its review overlay** | The site's author | A reader's URL, a review comment from GitHub, a help address's caller | The address (`?pm=`), the comment body, the `go/<name>.json` response |
| **B8: The release pipeline** | The maintainer running `release.yml` | A dependency, a pinned action, a pull request's commit, a cache | Code and inputs into the built binaries and packages |

## Entry points

One row each. **Reads it** names the code that takes the input; **May** is what the entry point is allowed to do with it; **Breach** is what would count as an exploitable finding (README decision 2). **Phase** names the phase that reviews it. Built from the surface map's commands, requests, tools, and files, and from `crates/ascribe-cli/tests/all/program_starts.rs` (every program start) and `crates/ascribe-resolve/tests/all/file_reads.rs` (every read outside `FileSystem`).

### Content a project holds

| Entry point | Reads it | May | Breach | Phase |
|---|---|---|---|---|
| Source pages and fragments (`.md` under the content root) | `ascribe_syntax::parse` through `comrak-ascribe`; frontmatter in `ascribe-resolve`; `index_file` | Produce blocks, diagnostics, and outputs for the page | A panic or hang in a server; a read outside `FileSystem`'s boundary through a link, include, image, or snippet path; script in an output's attribute or element that Ascribe wrote | 1 |
| Raw HTML on a page | `render_site_html` (comrak `unsafe`), the Astro plugin, the editor's preview, the HTML report | Pass through to the site | Reaching the editor's preview or the review overlay with more than the policy allows | 1 (outputs), 4 (where it's shown) |
| `@include` targets, `@snippet` sources and files, image and link paths, a template's path, a nav entry, a permalink's or address's key | `ascribe_resolve` (`FileSystem`, `snippet/`, `side_file.rs`), `ascribe-emit`'s assets | Name files inside the content root, a declared source, or the project | `..`, an absolute path, a drive or UNC path, a backslash, a link, a case difference, or a percent-encoded segment reaching a file outside what `FileSystem` allows, or a build's asset copy escaping the output directory | 1 |
| `ascribe.toml` and `[from]`'s file | `ascribe_model::load_str_in`, `loader.rs`, `from.rs` | Shape the model: types, patterns, builds, sources, apis, checks, `[checks.vale] command` | A regex or a pattern that runs for far too long; a path (`content-root`, `output-dir`, a source's `path`, an api's `file`, `[from]`) reaching outside the repository; a `command` run where a person didn't ask (decision 76 bounds this) | 1, 2 |
| `ascribe.permalinks`, `ascribe.addresses`, `ascribe.nav`, `ascribe.baseline`, `ascribe.lock`, `ascribe.examples.toml`, `examples.observations/` | `side_file.rs` and `SIDE_FILES`; `ascribe_model::examples`; `crates/ascribe-check/src/side_files.rs` | Name pages, ids, addresses, and files of the project | A panic or hang; a write steered outside the project by a key (`permalinks sync`, `addresses`, `rename`, `check --fix`) | 1 |
| `memos/` and `.ascribe/memos/` | `crates/ascribe-resolve/src/memos.rs`, `crates/ascribe-lsp/src/memos.rs` | Attach text to a page or block | A panic or hang; `about` naming a file outside the project and the server reading it | 1, 3 |
| Code a snippet reads (a `[sources]` folder in the repository, or `sources/<name>/` copies) | `crates/ascribe-resolve/src/snippet/`, `ascribe_core::SourceBoundary` | Read text files under the source's folder | A region tag or a path reaching outside the source; a non-text file read without limit | 1 |
| An OpenAPI spec (`[apis.<name>] file`, inside or outside the project's folder, in the same repository) | `crates/ascribe-model/src/apis.rs`, the OpenAPI reader, the JSON example validator | Produce operations and schemas for links and `@operation` | A YAML alias bomb or `$ref` cycle hanging a server; a spec's summary, path, or schema text reaching HTML unescaped | 1 |
| Assets under `_ascribe/up/` and images a page names | `EmitContext::asset_source`, `crates/ascribe-emit/src/store.rs`, `DiskAssets` in `crates/ascribe-diff/src/html/mod.rs` (both skip the boundary, case, and link rules) | Copy or inline files into an output | A source path outside the project; a destination outside the output directory; a link followed to a file outside the project (decision 77 bounds the output directory) | 1 |
| The output directory and its manifest | `crates/ascribe-emit/src/store.rs` (`OutputDir`) | Delete and write what the manifest says Ascribe owns | A committed manifest or link making a build delete or write outside the output directory (decision 77 fixed one form) | 1 |

### Other repositories and programs

| Entry point | Reads it | May | Breach | Phase |
|---|---|---|---|---|
| A remote `[sources]` repository: its URL, `branch`, `ref`, and what `git` fetches | `crates/ascribe-sources/src/remote.rs` (with `ascribe_core::git::OPTIONS` and `CLEARED`, `protocol.allow=never`, and the five transports SPEC §7.4 allows, `file` among them), `copies.rs` | Fetch at a pin into a cache, copy the files snippets name into `sources/<name>/` | A URL or ref read as a `git` option; a transport outside the allowlist (`ext::`); a `file://` source reading a repository the person didn't mean (a documented transport, so a hardening note unless another surface fetches); a link, submodule, LFS pointer, or hook in what's fetched reaching the user's machine or files outside `sources/` | 2 |
| Git history at a base revision (`--base`, `github.event.before`, the review base) | `crates/ascribe-diff/src/git.rs` (`Repository`, `GitFs`), `packages/review/src/shared/git.ts` | Read trees and blobs at a revision, list changes, `--` before paths | A revision read as an option; a `.git/config` in a checkout Ascribe didn't make running a command (decision 76 bounds `core.fsmonitor` and a planted bare repository; `core.hooksPath`, `diff.external`, filters, and `core.sshCommand` are this phase's to check) | 2 |
| What Vale, lychee, and afdocs print | `crates/ascribe-check/src/tool.rs` (one `Command::new(program)`, arguments as a list, addresses on standard input), `prose/`, `links.rs`, `site.rs` | Be parsed as JSON or text into diagnostics and the link cache | A program chosen by the project's content; output read as HTML or Markdown that runs; no timeout or size limit, so a server hangs a report | 2 |
| `[checks.vale] command`, and the program each check finds on the `PATH` or at a project-local path | `tool.rs`, `crates/ascribe-check/src/prose/setup.rs`, and the surfaces decision 76 names | Run Vale where a person asked (`ascribe check --vale`, the editor after asking once) | A run from the MCP server or the hook of a program the project supplies; the language server's question skipped on a restart | 2, 3 |
| `ascribe.examples.toml`'s `test` and `[environment]` lines | `crates/ascribe-cli/src/commands/observe.rs` (the one shell start; asks at a terminal, `--yes` skips it; without a terminal it doesn't ask) | Run the project's test command when a person runs `ascribe observe` | A run from CI or an agent's shell without the person having read the line; a line from a file outside the folder it was run in | 2 |
| The binary the editor runs (`node_modules/.bin/ascribe` before the bundled one; `ascribe.path`, `machine-overridable`) | `packages/vscode/src/environment.ts`, `processes.ts`, `registry.ts` | Run the project's own Ascribe after Workspace Trust | A run before trust, on a restart, from Check Every Build, from a task, or from a workspace setting | 2 |
| `@ascribed/astro`'s `ascribe` and `git`; `@ascribed/review`'s `git` and `gh` | `packages/astro/src/run.ts`, `project.ts`, `review/diff.ts`, `review/github.ts`; `packages/review/src/shared/git.ts`, `github/transport.ts` | Run the binary the `binary` option or the package names, and `git` in the Astro root | An argument from the project read as an option; a binary chosen by the project | 2 |
| The hook's check server (this binary, started again) | `crates/ascribe-cli/src/agents/checker.rs` (files in the user's cache folder; `powershell.exe` on Windows) | Keep a project loaded and answer hooks through files | Another local user or process writing the cache files; a project choosing the program started | 2, 3 |

### The servers

| Entry point | Reads it | May | Breach | Phase |
|---|---|---|---|---|
| The language server's standard requests (semantic tokens, completion, hover, definition, references, document links, code lenses, inlay hints, code actions, folding, formatting, rename, prepare rename, `willRenameFiles`, `executeCommand`) and file events | `crates/ascribe-lsp/src/server.rs`, `uri.rs` (`uri_to_path`, used in 26 places), `core.rs` | Read the workspace's projects and answer about them; return edits the client applies | A URI outside the workspace's projects read, previewed, or returned; an edit touching a file outside the project; a link with a scheme that does more than open (decision 75 bounds document links); a request that stops the server | 3 |
| Ascribe's own requests: `ascribe/agentPrompt`, `buildView`, `context`, `edit`, `inventory`, `page`, `preview`, `projectEdit`, `review/setBase`, `review/changes`, `review/baseFile`, `search`, `settings`, `targets` | The module the surface map names for each | The same, plus `preview`'s list of asset files for the client to turn into webview URIs, and `review/baseFile`'s read of a file at the base | The preview listing a file outside the project; `review/baseFile` or `setBase` reading or running outside the repository; a cache (the kept base, redirects) growing without bound | 3 |
| The MCP server's tools (`ascribe_check`, `explain`, `model`, `outline`, `link`, `refs`, `search`, `resolve`, `fmt`, `diff`, `rename`, `addresses_move`, `addresses_retire`), resources (`ascribe://directives`, `ascribe://model/{+project}`, `ascribe://instructions/{+project}`), and prompts (`new-page`, `fix`, `review`) | `crates/ascribe-cli/src/mcp/tools.rs`, `mod.rs`, `cache.rs`; each tool's `answer` function | Resolve `page`, `path`, and `project` from the server's working directory, held to the project; return what the command writes; write nothing (decision 56) | A `../`, absolute, or linked path reading another file; a tool writing a file; a revision read as a `git` option; page text placed where the server's own guidance goes; an answer or a request without a size bound | 3 |
| The agents' hook's input (`PostToolUse` and `Stop` for Claude Code and Codex, `postToolUse` and `agentStop` for Copilot) | `crates/ascribe-cli/src/agents/hook.rs` (`tool_input` paths, a Codex patch's `*** Add File:` lines) and `checker.rs` | Check the files the agent edited and print advice as `additionalContext` | A path from the input read outside the project; the `command` it runs chosen by the project (decision 76 bounds it to a bare name on the `PATH`); page text printed back as the harness's own instruction | 3 |
| `ascribe preview`'s server at `127.0.0.1` | `crates/ascribe-cli/src/commands/preview/serve.rs` | Serve the folder it wrote | A request path reaching outside `.ascribe/preview/`; a cross-origin page reading it | 3 |

### The editor and the web

| Entry point | Reads it | May | Breach | Phase |
|---|---|---|---|---|
| The extension's webviews and their messages (the page preview, the This page view, project settings, the review panel) | `packages/vscode/src/webview/viewShell.ts` (the policy, a nonce), `preview/messages.ts` (one check per `FromWebview` type, decision 78), `ui/webviewView.ts`, `ui/projectSettings.ts` | Draw a page's HTML under the policy; act on a checked message | `unsafe-inline` or `unsafe-eval` in a policy; `localResourceRoots` wider than the project; a message acted on unchecked in any view but the preview; a page's script reaching the extension | 4 |
| Workspace Trust and the settings a workspace can set | `packages/vscode/src/extension.ts`, `ui/tasks.ts`, `processes.ts`, `diskChanges.ts`, `gitIgnored.ts`, `ui/checkEveryBuild.ts`, `package.json`'s `restrictedConfigurations` | Run nothing in Restricted Mode | A process started before trust; a setting that changes what runs and isn't restricted | 2, 4 |
| `command:` links in hovers and Markdown | `crates/ascribe-lsp` (document links, decision 75), `packages/vscode/src/` (hovers, `MarkdownString`) | Open `http`, `https`, and `mailto` | A `command:` URI a page can place in something the editor trusts (`isTrusted`) | 4 |
| Review comments from GitHub (bodies and positions) | `packages/review/src/github/`, `overlay/markdown.ts`, `overlay/overlay.ts`; `packages/vscode/src/preview/threads.ts`, `threadsText.ts`; the HTML report in `crates/ascribe-diff/src/html/` | Be rendered as text or sanitized Markdown beside the block they're about | A body rendered as raw HTML in the overlay, the report, or the editor; a position steering a read | 4 |
| The GitHub transport's token | `packages/review/src/github/transport.ts` (`HostToken`, decision 80), VS Code's `github` and `github-enterprise` sign-ins | Send the token to the host it was issued for, never following a redirect | A token sent to another host; a token stored where another extension or the project reads it | 4 |
| `set:html` and `innerHTML` in `@ascribed/astro`, `@ascribed/elements`, and the webviews | `packages/astro/src/`, `packages/elements/src/`, `packages/vscode/src/webview/` | Write Ascribe's own markup and the author's | A value from a reader, a commenter, or a spec reaching one unescaped | 4 |
| `<ascribe-tabs>` reading the page and the address (`?pm=pnpm`) | `packages/elements/src/` | Pick a tab from the address, with values from the page | An address injecting into the page | 4 |
| Help addresses, `go/<name>.json`, with `Access-Control-Allow-Origin: *` | `_ascribe/headers.json` from `ascribe_resolve::pages::header_rules`, written into `_headers` by `@ascribed/astro` | Serve public text to any origin | Anything but the help text under that header | 4 |

### Supply chain and release

| Entry point | Reads it | May | Breach | Phase |
|---|---|---|---|---|
| Rust and npm dependencies | `Cargo.lock` (`cargo deny` in `rust.yml`); `pnpm-lock.yaml` (nothing audits it); `fuzz/Cargo.lock`, outside the workspace | Be compiled into the binary and bundled into the packages and the extension | A dependency with an install script or a known advisory; one the lockfile doesn't pin | 5 |
| Workflow triggers and inputs: `pull_request`, `workflow_run` (`site-npm.yml`), `schedule` (`report.yml`, `canary.yml`, `corpora.yml`, `fuzz.yml`, `templates.yml`), `workflow_dispatch` inputs (`approve.yml`, `board.yml`, `release.yml`, `templates.yml`), `${{ github.event.* }}` in `run:` steps | `.github/workflows/*.yml` | Run with the `permissions` each sets | A fork's code run with a write token or a secret; an artifact or a checkout from a fork's run used by a `workflow_run` job; an input reaching a shell unquoted; a cache a pull request poisons and a release restores | 5 |
| The release: `release.yml`, `release-build.yml`, `canary.yml` | The workflows; npm trusted publishing with provenance; the Marketplace token; the Homebrew tap's pull request | Publish from the commit the maintainer dispatched on | A pull request influencing what the canary publishes; a token readable outside the publishing job; binaries with no checksums | 5 |

## What the earlier round already fixed

A first pass of this review, on 26 October 2026, landed decisions 75 to 80 in [decisions.md](../decisions.md) before this plan was written; the plan's README doesn't list them under "What exists today", so they're here. A phase that meets one of these verifies the fix holds rather than finding it again:

- **75:** the language server gives a document link only for `http`, `https`, and `mailto`.
- **76:** every `git` runs with `safe.bareRepository=explicit` and `core.fsmonitor=false` after clearing the `GIT_*` variables that point elsewhere; the MCP server never starts Vale; the hook never runs a `command` that's a program in the project; the language server asks once before running one.
- **77:** the output directory and the other files Ascribe writes into a project aren't written through a link; a local source with no repository around the project is bounded by the editor's workspace or the MCP server's folder.
- **78:** the page preview and the extension check each other's messages; `dev.json` is used only when the author asks.
- **79:** includes expand a page to at most 100,000 blocks and 16 MiB; every walk that follows links goes into each folder once.
- **80:** a GitHub sign-in goes only to the host it was issued for.

Two more corrections to the README's "What exists today":

- **Fuzzing exists.** `fuzz/` holds three `cargo-fuzz` targets (`parse`, `model`, `format`), and `fuzz.yml` runs them nightly, opening an issue per distinct crash (decision #1020). Phase 1's new targets (frontmatter, the OpenAPI reader, the example validator) join that crate and that job rather than starting one.
- **The hook's check server starts this binary again,** and on Windows starts `powershell.exe` (`crates/ascribe-cli/src/agents/checker.rs`); `program_starts.rs` lists it. Phase 2 and 3 cover it.

## Since 27 October

`main` moved two hundred commits between the review and its re-check on 29 October. What they added to the entry points, and the phase that holds each:

- **`ascribe-workspace`** (decision #994): one loaded project shared by every surface, with its own three reads outside `FileSystem` (finding and reading `ascribe.toml`, and a listing of the content root for stamps that doesn't descend a linked folder). Phase 1.
- **The git guard** (`crates/ascribe-cli/src/guard.rs`, decision #965): `git rev-parse` and `git status` before every writing command and the MCP tools that write, so a new caller of `git` in a checkout Ascribe didn't make, with decision 76's options. It checks that a file is restorable, not that it's inside the project. Phase 2.
- **`ascribe-edit`**: the rename, release, retire, memo, nav, and model edits shared by the commands and the language server. Phase 3.
- **`ascribe-report`**: `tool.rs` and the link checks, moved out of `ascribe-check`; the cache and histories are written by the CLI's `DiskStore`. Phases 2 and 5.
- **`ascribe new`**, **`check --format github`**, the `--check` unified diffs, `observe --yes` (decision #979), and `sources fetch --dry-run`. Phases 1 and 2.
- **`ci.yml`'s `record` job**, `rust.yml`'s `agents sync --check`, and `site.yml`'s `check --base`. Phase 5.

## Settled, and to settle

README decision 3 is the settlement of who's trusted; this page applies it. One point the table surfaces that the README doesn't settle, for the maintainer (README decision 7 records the answer):

- **`ascribe observe` without a terminal.** It runs the shell line `ascribe.examples.toml` names, and when there's no terminal to ask at it doesn't ask. The file is trusted as the content model is (the person wrote it), and CI runs it on purpose. The table counts it as trusted input run by a person's command, so a shell line in a fork's `ascribe.examples.toml` is in scope only where a workflow runs `observe` on a pull request's checkout. This repository's `rust.yml` does, for `examples/tested/code` (`ascribe observe --name node-22`, in the job that already runs `cargo test` on the same checkout with a read-only token and no secrets), so the line adds nothing a fork's `build.rs` or a test couldn't do; phase 5 holds the workflows to that.
