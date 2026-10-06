# Phase 5: One home per job

Part of [Optimization](README.md). Needs phase 2, part A. Part C of phase 2 (link tests on Windows) should be merged before part A here. Rust only.

**Runs with:** phases 4, 6, and 7. Inside this phase, order matters:

- **Part B first** (command paths). The agents plan waits on it.
- **Part C after part B:** both rewrite the same functions in `crates/tessera-cli/src/commands/`.
- **Part A** can run beside part B, except in `crates/tessera-cli/src/commands/fmt.rs`, which part B owns: part A leaves that file for last and rebases.
- **Part D** is independent of A, B, and C. Don't run it beside phase 4's part A, which also edits `crates/tessera-emit/src/render/`.
- A lint from phase 3 that touches many files merges before this phase starts or after it ends.

## Goal

Each job is done in one place. A file is read through the one thing that knows the content root and what a link means. A command is argument handling, one call into a library, and reporting. A failure is a type a caller can tell from another. And the binary carries one Markdown parser.

Parts B and C are what the agents plan needs before its phase 2 ([decision 8](README.md#proposed-decisions)): its tools wrap commands, and a tool can't wrap logic that lives in the binary or act on an error that is a sentence.

## Context

- The [inventory](inventory.md): "Reading files and paths", "Command paths", "Rules in force", and findings 2, 6, and 7.
- `crates/tessera-resolve/src/fs.rs`: `FileSystem` (the trait), `DiskFs`, `MemoryFs`. `crates/tessera-diff/src/gitfs.rs`: `GitFs`. `crates/tessera-core/src/path.rs`: `RelPath`.
- `crates/tessera-check/src/project.rs`: `Project::load`, `find_config`, `file_system()`.
- `crates/tessera-cli/src/context.rs` (`load_project`), `commands/diagnose.rs` (`select_builds`, `diagnose`), `commands/build.rs` (`write_outputs`), `commands/fmt.rs` (`find_config`, `load_model`, `collect`, `format_all`), `commands/diff.rs`, `drift.rs`, `sources.rs`, `exit.rs`, and `report/`.
- `crates/tessera-lsp/src/core.rs` (its own loading, by design), `compute.rs`, `review.rs`, `fsx.rs`, `uri.rs`.
- `crates/tessera-cli/tests/lsp_parity.rs` and `crates/tessera-check/tests/parity.rs`: tests that the command and the server agree.
- `crates/tessera-emit/src/render/mod.rs`: `render_site_html`, which calls upstream `comrak`. `crates/comrak-tessera/FORK.md`: what the fork changes, and that it passes CommonMark with its option off.
- Reviews of #73, #99, and #118 on GitHub: the link bugs so far, and the two gaps #118's review left (`fmt` writes through a link out of the content root; a snippet follows a link out of a source whose path is `..`).
- `project-docs/agents/README.md`, decision 2, and `phase-7-mcp.md`'s tool table: the callers this prepares for.

## Design

### Part A: file reading

One pull request per crate, smallest first.

- **The rule:** code that reads a project's file, lists its folders, or decides whether a path is inside the project asks `FileSystem`, or one helper beside it. Code that writes outputs (`emit/store.rs`) or talks to `git` (`tessera-sources`) keeps its own writes, and uses the shared helpers for "is this path inside" and "what does this link point to".
- **The reads to move:** the 34 outside `tessera-resolve/src/fs.rs`, in `emit/store.rs` (9), `cli/commands/fmt.rs` (6), `sources/copies.rs` (5), `cli/docs.rs` (4), `lsp/core.rs` (3), `model/loader.rs` (2), and one each in `sources/lib.rs`, `resolve/slug/github.rs`, `model/lib.rs`, `diff/html/mod.rs`, `check/project.rs`. Some are right as they are (reading a file the user named on the command line; a build script's input). For each: move it, or leave it with a one-line comment saying why it isn't a project read.
- **`tessera-model` can't depend on `tessera-resolve`.** If its two reads need the shared rule, the rule's home moves down to `tessera-core`, as plain functions over paths. Decide this first, since it sets where everything else goes; say which you chose in the first pull request.
- **Path helpers:** the three `relative_to` functions and the two normalizers become one each, in `tessera-core`, with the union of their tests. The language server's version handles Windows drive letters; keep that behavior.
- **The two known gaps** are fixed here, each with a test, in its own commit. These are behavior changes: note them in `CHANGELOG.md`.

### Part B: command paths

One pull request per step.

1. **A library home for what the commands share.** `select_builds`, `diagnose`, and `write_outputs` move from the CLI crate to a library: `tessera-check` for the first two, `tessera-emit` for the third, unless the dependency graph says otherwise. The CLI keeps argument parsing, choosing a format, printing, and exit codes.
2. **One way to load.** `fmt` uses `context::load_project`, or the part of it that fits: `fmt` must still work on a project whose pages have errors, and must still format a file given by path. Its own `find_config`, `load_model`, and `collect` go.
3. **The second load, once.** `tessera_resolve::Project::load(model, layout, file_system)` is written out in `build`, `diff`, `drift`, and twice in `sources`. It becomes one method on the checked project (or one function), and the five sites call it. If two loads of the same files can become one, that's phase 7's; don't attempt it here.
4. **Each command gets one entry function** in a library, taking plain arguments (a project, options) and returning a result type: no `clap` types, no writers, no exit codes. The CLI's function becomes: parse, call, report. The language server calls the same entry where it does the same job; where it can't (checking, which is incremental there), say why in a comment and keep the parity test.

### Part C: failures and printing

One pull request, after part B.

- **The ten functions that return `Result<_, String>`** return an error type: an enum per area, with `thiserror`, as the library crates already do. Each variant has a stable code, a short lowercase identifier, exposed by a method; the message stays what users see today, word for word.
- **Exit codes don't change.** `crates/tessera-cli/src/exit.rs` maps error types to the codes it uses now.
- **Libraries don't print.** Deny `clippy::print_stderr` and `clippy::print_stdout` in `[workspace.lints]`; `tessera-cli`, benchmarks, and the corpora tools allow them at the crate root with a comment. The seven printing lines (`lsp/core.rs`, `lsp/server.rs`, `resolve/snippet/tags.rs`) become returned values where the caller can use them. The language server's log lines go through one function in `tessera-lsp` that writes to standard error, allowed in that one place, since standard error is the server's log by protocol.
- **No logging library** is added. If part B or C shows a real need for one, stop and report.

### Part D: one parser

One pull request, independent.

Test whether `render_site_html` can use `comrak-tessera` with its Ascribe option off in place of upstream `comrak`. The fork is upstream at the same version plus a marked patch, so it should give identical HTML; prove it with the comparison and with `tessera-emit`'s render tests. If it's identical, `comrak` leaves `tessera-emit`'s dependencies (it stays a dev-dependency of the CommonMark suite, which compares the fork against it). Report the build time and binary size before and after. If any output differs, stop and report the difference; don't adjust the output.

### Large files

Not a part. When a part above is already rewriting a file over 1,000 lines, split it along the seam the change exposes. Don't split one otherwise.

## Tasks

1. Part B, steps 1 to 4, each with the comparison's result in the pull request.
2. Part C.
3. Part A: the decision on where the rule lives; the reads, crate by crate; the path helpers; the two gaps.
4. Part D.
5. `ARCHITECTURE.md` and the crate READMEs, if phase 3 has merged: where loading, reading, and each command's entry now live.
6. After part B: a note on `project-docs/agents/README.md`'s pull request or issue saying the commands are ready to wrap, with the entry function for each. Don't edit that plan.

## Out of scope

- New commands, options, or JSON fields. The agents plan adds those.
- Changing the language server's incremental loading.
- Making anything faster, except where it falls out for free.
- Async, threads, or a logging library.
- Any change inside `crates/comrak-tessera/src`.

## Acceptance criteria

- `scripts/compare/outputs.ts --base <base>` reports `same` in every pull request, except the two commits that fix the known gaps, which say what changed.
- Every message and exit code a command gives for a failure is unchanged; the CLI's existing tests hold this, and a new test lists each error code once.
- `crates/tessera-cli/src/commands/*.rs` contains no call into `tessera_resolve::Project::load` and no direct file-system read except of a path the user gave.
- A test, outside the CLI crate, calls each command's entry function and gets a typed result.
- No library crate prints; the lint holds it.
- The measures in the [plan](README.md#measures) for this phase read: reads outside `FileSystem` 0 or each explained; ways the commands load a project 1; commands whose core is in the CLI crate 0; functions returning a string error 0; printing lines 0.

## Stop and report if

- Moving a read changes which files a project sees, beyond the two known gaps.
- A command can't get one entry function without changing its output or its exit codes.
- `fmt` on a broken project stops working when it shares the loader.
- The dependency graph would gain a cycle.

## Verify

```sh
cargo test --workspace --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo fmt --all --check
cargo build -p tessera-cli && ASCRIBE_BIN=$PWD/target/debug/ascribe pnpm --filter ascribe-vscode test:integration
node scripts/compare/outputs.ts --base origin/main
```

## Commits

1. "Move what check and build share into a library"
2. "Load a project one way in every command"
3. "Give each command one entry function"
4. "Return typed errors with stable codes; stop printing from libraries"
5. "Read project files through one place" (one per crate)
6. "Keep one relative-path and one normalize function"
7. "Don't format a file reached through a link out of the project"
8. "Don't follow a link out of a source above the project"
9. "Render HTML with the parser we already ship"
