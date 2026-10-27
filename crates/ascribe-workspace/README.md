# ascribe-workspace

Finding and loading an Ascribe project: the content model a command works on, and the project around it, read once. This README is about the code.

## Pieces

| Module | Main types and functions | What it does |
|---|---|---|
| `src/locate.rs` | `locate`, `find_config`, `find_below`, `LocateError` | Finding `ascribe.toml`: the one `--config` names, or the nearest at or above a folder, and, when there's none, the projects below it that the error names. |
| `src/load.rs` | `Loader`, `ModelFile`, `SourceFile`, `ReadFailure`, `LoadError` | Reading a project: the content model and its text (`Loader::load_model`), then every source file through one `FileSystem`, once each, and the side files. A source that can't be read is still a source, with its `ReadFailure`. |
| `src/workspace.rs` | `Workspace`, `WorkspaceSnapshot` | The project, loaded once: `Workspace::load` and `load_with` read it, and `from_texts` builds one over texts given. It owns the project's `ascribe_resolve::IncrementalProject`, loaded without what only an update needs (`IncrementalProject::load_deferred`), and indexes the texts it read without reading them again. `Workspace::snapshot` gives the `WorkspaceSnapshot` everything reads: the source index, the content model and its text, the sources, which are the index's files with its ids and texts, then those that couldn't be read, the side files, and what's read on demand (code files, images, memo files). A snapshot never changes; `with_source` and `with_texts` give another with text laid over it, indexed again, and `of_index` makes one of an index someone else keeps, as the language server does. |

A `Loader` holds what loading takes from the crate that checks a project: the content model's loader, which runs the checks the content model's own load includes, and the list of side files, each an `ascribe_resolve::side_file::Entry`. `ascribe-check` gives the one every surface loads with, `ascribe_check::LOADER`, and its `Project` is the checks' view of a snapshot.

```rust,ignore
let config = ascribe_workspace::locate(None)?;
let workspace = ascribe_workspace::Workspace::load(&ascribe_check::LOADER, &config)?;
let project = ascribe_check::Project::new(workspace.snapshot());
```

## Rules

- **It depends on `ascribe-core`, `ascribe-model`, and `ascribe-resolve`,** and not on `ascribe-check` or `ascribe-emit`: what's the checks' comes in through the `Loader`.
- **It reads, and writes no files.** Every read of a project's file goes through `ascribe_resolve::FileSystem`; the reads before there's one, of `ascribe.toml` and of the folders searched for it, say why in an `Outside FileSystem:` comment.
- **Errors have codes.** `LoadError` and `LocateError` are `ascribe_core::Coded`, and the binary's exit codes list them.

## Tests

- `tests/all/locate.rs`: `locate` takes a file or the folder holding one, and `find_below` lists the projects below a folder, as the error names them.
- `tests/all/load.rs`: the content model through the loader given, the sources in path order with the unreadable after, and only the side files the list names.
- `tests/all/workspace.rs`: a loaded snapshot is the source index `ascribe_resolve::Project::load` makes of the same files, each source is listed once and read once, a project from texts numbers them in path order, and text laid over a snapshot is indexed again, with new ids or the same.
