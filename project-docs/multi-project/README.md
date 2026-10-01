# Multi-project workspaces

Ascribe's VS Code extension and language server assume one project per workspace. A repository that keeps code and docs side by side, and may hold several Ascribe projects (the Ascribe repo itself has `examples/quill` and `examples/astro-site`), gets the wrong project, or none. This work makes every `ascribe.toml` in a workspace its own project, each with its own language server.

## The problem, as observed

Opening the repo root in VS Code with the 0.1.0 extension:

- No squiggles in `examples/quill/docs/quickstart.md`.
- The preview says the file "isn't a source of the project (it is outside the content root)".
- The server output says it used `examples/astro-site/node_modules/.bin/ascribe`.

Cause: the server loads one project per process. `find_config` in `crates/tessera-lsp/src/core.rs` looks upward from the workspace folder for an `ascribe.toml`; when there is none, it takes the first one below the folder, in alphabetical order (`astro-site`). The extension starts one server for the whole window and sends it every Markdown file.

## Decisions

These are settled. Do not reopen them in a phase; if one turns out to be unworkable, stop and report.

1. **One server per project.** The extension starts one `ascribe lsp` process per `ascribe.toml`, each with that file's folder as its workspace folder. The server stays one project per process. (Rejected: a multi-project server. It touches the core, watchers, rename, completion, and the preview request. Revisit if process count becomes a problem.)
2. **Nearest ancestor owns a file.** A file belongs to the closest `ascribe.toml` above it. Nested projects are allowed. A file inside a project's folder but outside its content root gets no diagnostics from that project.
3. **Commands follow the active file.** Preview and "Select Preview Build" act on the project that owns the active editor. "Restart Language Server" restarts every running server. "Show Server Output" opens the output channel for the active file's project.
4. **The server never guesses.** It looks only upward from the folder it is given. The downward search is removed. It always logs which `ascribe.toml` it chose.
5. **Servers start lazily.** Discovery of projects is cheap and eager; a project's server starts the first time one of its files is needed. The setting `ascribe.startServers` is `"onDemand"` (default) or `"all"`.
6. **The Problems panel covers running projects only.** A project whose server hasn't started has no diagnostics there. This is documented, not worked around. `ascribe check` still covers everything.
7. **Each project resolves its own binary:** the `ascribe.path` setting (overrides every project), then `node_modules/.bin/ascribe` from the project folder up to the workspace folder, then the bundled binary.

## Cost of running several servers

Measured on an Apple-silicon Mac against the 0.1.0 binary: an empty server uses about 6 MB; a project costs about 70 KB per page (100 pages 15 MB, 1,000 pages 78 MB, 3,000 pages 222 MB). Idle CPU is zero. Keystroke-to-diagnostics is 1-3 ms. Splitting a repo into several projects costs about the sum of the projects plus 6 MB each.

## Phases

Do them in order. Each phase leaves the repository green and can be its own pull request or a commit within one.

| Phase                                              | Result                                                                  |
| -------------------------------------------------- | ----------------------------------------------------------------------- |
| [1: Server](phase-1-server.md)                     | The server only looks upward and always logs its project.               |
| [2: Project registry](phase-2-registry.md)         | One lazily started server per `ascribe.toml`, with nearest-owner rules. |
| [3: Commands and preview](phase-3-commands.md)     | Commands, preview, format on save, and settings work per project.       |
| [4: Integration tests](phase-4-tests.md)           | A monorepo fixture and end-to-end coverage.                             |
| [5: Documentation](phase-5-docs.md)                | Editor guide, LSP README, changelog.                                    |
| [6: Release 0.1.1](phase-6-release.md)             | Version bump, tag, publish, verify.                                     |

## Rules for every phase

- Branch before committing; never commit to `main`.
- Match the surrounding code's style, comment density, and naming. Libraries don't panic on user input; `unwrap` and `expect` are linted.
- Run the checks in [CONTRIBUTING.md](../../CONTRIBUTING.md) that the phase touches. Before finishing a phase, all of these pass:

  ```sh
  cargo fmt --all --check
  cargo clippy --workspace --all-targets --locked -- -D warnings
  cargo test --workspace --locked
  pnpm format:check && pnpm lint && pnpm typecheck && pnpm test
  ```

- Tests that depend on paths must pass on Windows: no hard-coded `/` separators, no assuming `canonicalize` output has no `\\?\` prefix, build `file://` URIs with three slashes (`file:///C:/...`), and fold drive-letter case when comparing.
- Don't add attribution or phase-history prose to source files or docs. Describe what the code does now.
- If a phase's instructions conflict with the code, or a decision above can't be met, stop and report the conflict instead of choosing silently.
