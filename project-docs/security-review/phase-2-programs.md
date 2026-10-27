# Phase 2: Running other programs

Part of [Security review](README.md). Needs phase 0.

## Goal

Find whether content, configuration, or another program's output can make Ascribe run a command it didn't mean to, or pass a program something it treats as an option.

## Questions

### `git`

- `crates/ascribe-diff/src/git.rs` and `crates/ascribe-sources/src/remote.rs`: can a revision (`--base`, a `ref` in `[sources]`, `github.event.before` in CI) or a URL start with `-` and be read as an option, before the `--` that guards paths?
- The protocol allowlist: does every `git` invocation that can reach a remote have it, including one a test or a later change adds? Is `file` reachable, and should it be?
- A repository's own configuration: when Ascribe runs `git` in a checkout it didn't make (the editor's workspace, a pull request's checkout), can `core.fsmonitor`, `core.hooksPath`, `diff.external`, a filter, or `core.sshCommand` in that repository's `.git/config` run a command? Which of Ascribe's `git` calls would trigger them?
- A remote source's clone: hooks, submodules, LFS, and symbolic links in what's fetched.

### Lychee, afdocs, and Vale

- `crates/ascribe-check/src/tool.rs`: how is each program found (the `PATH`, a project-local path), and can a project's content choose which binary runs? Are arguments passed as a list, never through a shell? Do addresses go on standard input, as the code says?
- Their output is parsed as JSON or text: can a web page, through lychee's or afdocs' report, put anything into a diagnostic that's later rendered as HTML or Markdown that runs?
- Timeouts and output size limits, so a hostile server can't hang a report or exhaust memory.

### The binary the editor runs

- The extension runs a project's `node_modules/.bin/ascribe` before its bundled one (`packages/vscode/src/environment.ts`): is that only after Workspace Trust, every time, including the language server's restart and Check Every Build?
- `ascribe.path` is `machine-overridable`: can a workspace's settings set it?
- `@ascribed/astro`'s `run.ts` and `project.ts`, and `@ascribed/review`'s `git.ts`: how they find and run `ascribe` and `git`.

## Method

Trace every `Command::new` (`crates/ascribe-check/src/tool.rs`, `crates/ascribe-cli/src/agents/`, `crates/ascribe-cli/src/commands/check.rs`, `observe.rs`, `crates/ascribe-diff/src/git.rs`, `crates/ascribe-sources/src/remote.rs`) and every `child_process` call in `packages/`, and try each candidate input in a scratch repository: a revision starting with `-`, a `.git/config` with `core.fsmonitor`, a source URL with `ext::`.

## Output

Draft advisories for exploitable findings; `notes/phase-2.md` for hardening.
