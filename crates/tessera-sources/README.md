# tessera-sources

Sources in another repository (SPEC §7.4): copying the code files a project's snippets use into it, and moving a source's pin. `ascribe sources fetch`, `update`, and `status` are its command line; how to use them is in the [command reference](https://ascribed-dev.com/reference/cli/). This README is about the code.

A project reads a source in another repository through copies kept in `sources/<name>/` and pinned to a commit by `ascribe.lock`, so checking and building need no network. This crate makes and moves the copies; `tessera-check` reports on them.

## Pieces

| Module | Main types and functions | What it does |
|---|---|---|
| `src/lib.rs` | `Workspace`, `fetch`, `update`, `status`, `Options`, `SourcesError` | The three operations. `fetch` makes the copies match the lock, pinning a source that has no pin yet; `update` moves a pin to the head of its branch, or to a given revision, and says what changed; `status` reads the pins and copies alone, without `git` or the network. Each returns a report (`FetchReport`, `UpdateReport`, `StatusReport`) that the CLI prints as text or JSON. |
| `src/remote.rs` | | Running `git` against another repository: one bare repository per URL in a cache outside the project (`ASCRIBE_CACHE_DIR`, or the user's cache folder), fetched without blobs, then only the files asked for. |
| `src/copies.rs` | `SIZE_LIMIT` | The copies in `sources/<name>/`: listing, reading, writing, and removing them without leaving the folder. |

```rust,ignore
let workspace = tessera_sources::Workspace::new(root, &model, &index)?;
let report = tessera_sources::fetch(&workspace, &[], &tessera_sources::Options::from_env()?)?;
```

`Workspace::new` takes the content model and the project's source index (`tessera_resolve::Project`), for the files its snippets name.

## Rules

- **This is the only code in Ascribe that reaches another repository.** It runs the `git` executable with a fixed argument list, never through a shell, with hooks, submodules, and prompts turned off. No git, HTTP, or async library is linked.
- **Copies stay inside their folder.** A copy is never written through a symbolic link that leads out of `sources/<name>/`, and only text files up to `SIZE_LIMIT` are copied.
- **Failures are typed.** Every way an operation can't run is a `SourcesError` variant; a file that can't be copied is a `Failure` in the report, not an error.
- **It depends on `tessera-core`, `tessera-model`, and `tessera-resolve`.** It doesn't use `tessera-check` (except in its tests), and `tessera-check` doesn't use it: the checks read the copies and the lock as files.

## Tests

Each test makes a code repository and a docs project in temporary folders, and reaches the code repository by a `file://` URL: a real `git` fetch, with no network (`tests/common/mod.rs`).

- `tests/fetch.rs`: a first fetch pins and copies; a second reads the cache and never moves a pin; files that can't be copied; copies no snippet uses are removed; several sources; no access; a link out of the folder.
- `tests/update.rs`: a moved pin copies again and lists the commits; nothing to move; counting commits from history; `--to` back and forward.
- `crates/tessera-cli/tests/sources.rs` tests the command line.
