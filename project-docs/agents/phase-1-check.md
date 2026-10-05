# Phase 1: Check one file

Part of [Agents](README.md). Needs no other phase, and can run at the same time as phases 2 and 3. Rust only.

## Goal

An agent can check the file it just wrote, or text it hasn't written yet, and gets back each problem with its fix advice and a link to its documentation. This is the "check" of write, check, fix, and the hook in phase 9 runs it after every edit, so it has to be quick and its output small.

## Context

- `crates/tessera-cli/src/commands/check.rs` and `diagnose.rs`: the command and what it shares with `build`. `crates/tessera-cli/src/context.rs` (`load_project`): finding `ascribe.toml` and loading the project.
- `docs/cli.md`, "`ascribe check`": the options, exit codes, and JSON schema this phase extends. The JSON already has `fixes`, and says a field's meaning can't change without a new `schema_version`.
- `tests/conformance/diagnostics.toml`: each diagnostic's `fix` paragraph. `crates/tessera-check/src/registry.rs` embeds the registry in the binary, but its `Entry` doesn't read `fix` yet: add the field. `docs/diagnostics.md` is generated from the registry, with one heading per code (`#asc036-link-target-missing`).
- `crates/tessera-lsp/src/fsx.rs` and `tessera_resolve::incremental`: how the server lays unsaved text over the files on disk. `--stdin` needs the same thing.
- `crates/tessera-lsp/README.md`, "Diagnostics": the server publishes file-level diagnostics plus the page-level ones of **the editor's build only** (`[editor] build`; `ContentModel::editor_default_build`). `check` covers every build.
- `tests/corpora/RESULTS.md`: a whole-project check, all builds, release build, takes 0.78 s on the synthetic 3,000-page project and 2.0 s on the Elastic sample. A per-edit check that pays that is too slow for a hook.

## Design

### Checking paths

```
ascribe check [PATH]... [--stdin --path <PATH>] [--editor-build] [--summary] [--format text|concise|json] [--build <NAME>]...
```

- **Paths are relative to the current directory,** like any command's. A path is a file or a directory. A path that doesn't exist, or isn't in an Ascribe project, is exit code 2 with a message naming it.
- **The project is found from the first path** when `--config` isn't given: the nearest `ascribe.toml` at or above it. So `ascribe check docs/guides/install.md` works from a repository's root, and `ascribe check guides/install.md` from `docs/`. Paths in more than one project are exit code 2, saying so.
- **The project is still loaded whole** (links, includes, and ids need it), and only some diagnostics are shown: **a diagnostic counts for a path when its `file`, or the file of one of its `related` places, is in that path.** A problem in included content is reported at the include, with its place in the fragment as related information, so:
  - checking a page shows the problems its fragments cause on it;
  - checking a fragment shows the problems in it. The same problem appears at every include, so these are collapsed: diagnostics with the same code and the same related place in the fragment are shown once, at the first include in file order, with a new field `repeats` saying at how many other includes it also appears.
- `files_checked` keeps its meaning (how many source files were checked: the project's). A new field, `files_reported`, is how many files the paths named.

### Checking text that isn't saved

`--stdin --path guides/new.md` reads standard input and checks it as if it were that file's content, laid over the project on disk. The file doesn't have to exist. Only diagnostics that count for that file are shown. `--stdin` without `--path`, or with other paths, is exit code 2.

### One build, for speed

`--editor-build` checks the file-level diagnostics and the page-level ones of the editor's build only: what the language server reports as you type. It's for a check after every edit; a full `ascribe check` before finishing still covers every build. It can't be combined with `--build`. The JSON gains `builds_checked`, the names of the builds whose page-level checks ran, so a reader knows what a clean result covers.

### Output for agents

- **`--format concise`** writes one line per diagnostic, `file:line: [code] message`, grouped by file, then the summary line. It's the format an agent reads as text, and the one the hook (phase 9), the skill, and the instructions (phase 4) all use, so an agent meets one shape everywhere. It shows at most 50 diagnostics, then "and N more" with the command that narrows the check.
- **`--summary`** replaces the list with counts by code and by file (most first), in any format. On a project with hundreds of warnings, an agent can choose a rule or a file to work through instead of reading every line.
- **Wherever a list is cut,** the JSON says so with `truncated`, `shown`, and `total`, and `next_command`, the command that gives the rest. `check`'s JSON list isn't cut today; phase 2's `refs` is.

### What each diagnostic gains

New fields in the JSON, and nothing removed or changed:

| Field | Type | Meaning |
|---|---|---|
| `help` | string | The registry's `fix` paragraph for this diagnostic: how to fix it, in general |
| `docs` | string | The URL of its entry in the diagnostics reference |
| `repeats` | number | For a problem in a fragment, when checking the fragment: how many other includes report it too. `0` otherwise. |

Each entry in `fixes` gains one field:

| Field | Type | Meaning |
|---|---|---|
| `applicability` | string | `"safe"` when applying the fix can't change what the page says and leaves nothing to decide; `"unsafe"` otherwise |

Every fix the checks offer is labeled where it's made, and a test fails for one that isn't. When unsure, a fix is unsafe. Nothing applies fixes in this plan; the label is there so an agent reading the JSON can tell a spelling correction from a rewrite, and so a later `--fix` can be limited to safe ones. Ruff, rustc, oxlint, and Biome all label fixes this way ([the research report](../../reports/Agent%20first%20interfaces%20for%20docs%20tools.md)).

The URL's base is one constant, pointing at `docs/diagnostics.md` in the repository at the tag of the running version. **Since docs phase 5, the docs are a site:** the base is the diagnostics page on it, `https://ascribe-docs.netlify.app/reference/diagnostics/`, with each code's anchor (`#asc036-link-target-missing`). Build it from `docs_site!` in `crates/tessera-cli/src/cli.rs`, whose test checks it against `[consumer] site` in `docs/ascribe.toml`. The site follows `main`, not a release's tag. In the language server, the same URL is each diagnostic's `codeDescription.href`, so the code is a link in the Problems panel and agents that read LSP diagnostics get it.

Text output is unchanged apart from honoring paths.

### Speed

Measure on both 3,000-page corpora, in a release build, and add the numbers to `tests/corpora/RESULTS.md`: `ascribe check <one file>` for a clean page, a page with an error, and a fragment used by 100 pages, each with every build and with `--editor-build`. Filtering the report doesn't make the check faster by itself; if checking only the pages that the named files reach is a small change, make it, and say what you did. This phase adds no cache and no daemon. Say in the pull request whether a single-file check with `--editor-build` is under half a second on both corpora: phase 9's hook depends on it.

## Tasks

1. Paths in `check`, with CLI tests: a file, a directory, two files, a fragment (collapsed, with `repeats`), a page that includes a broken fragment, a path that doesn't exist, a path from the repository root into a subfolder project, and paths in two projects.
2. `--stdin --path`, with tests: a new file, an existing file overridden, a file whose text breaks a link from another page (not shown: it isn't in this file), and misuse.
3. `--editor-build` and `builds_checked`; `--format concise` and `--summary`.
4. The registry's `fix` in `Entry`; `help`, `docs`, and each fix's `applicability` in the JSON; `codeDescription` in the server, with a scenario test.
5. The benchmarks and `RESULTS.md`.
6. `docs/cli.md`, `crates/tessera-lsp/README.md`, `CHANGELOG.md`.

## Out of scope

Rewording diagnostics' messages (they already say the fix where there's one); applying fixes; a watch mode.

## Acceptance criteria

- `ascribe check docs/guides/install.md --format json` from the repository's root, and `ascribe check guides/install.md` from `docs/`, report the same thing: only that file's problems, each with `help`, `docs`, and `fixes`.
- Checking a fragment reports each problem in it once.
- Text piped to `--stdin` is checked without touching the disk.
- Without `--editor-build`, the diagnostics are the whole-project check's, filtered by the rule above, in the same order.
- Every fix has an `applicability`.
- `schema_version` is still `1`, and no existing field changed meaning.
- The timings are in `RESULTS.md` and the pull request.

## Verify

```sh
cargo test --workspace --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo fmt --all --check
```

## Commits

1. "Check only the files named"
2. "Check text from standard input as a file"
3. "Check the editor's build alone"
4. "Add concise and summary output to check"
5. "Give each diagnostic its fix advice, a link, and its fixes' safety"
