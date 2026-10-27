# ascribe-workspace

Finding and loading an Ascribe project: the content model a command works on, and the project around it, read once. This README is about the code.

## Pieces

| Module | Main types and functions | What it does |
|---|---|---|
| `src/locate.rs` | `locate`, `find_config`, `find_below`, `LocateError` | Finding `ascribe.toml`: the one `--config` names, or the nearest at or above a folder, and, when there's none, the projects below it that the error names. |
| `src/load.rs` | `Loader`, `Loaded`, `ModelFile`, `SourceFile`, `ReadFailure`, `LoadError`, `read_sources`, `read_side_files` | Reading a project: the content model and its text (`Loader::load_model`), then every source file through one `FileSystem`, with the listing they were read from, and the side files. A source that can't be read is still a source, with its `ReadFailure`. |

A `Loader` holds what loading takes from the crate that checks a project: the content model's loader, which runs the checks the content model's own load includes, and the list of side files, each an `ascribe_resolve::side_file::Entry`. `ascribe-check` gives the one every surface loads with, `ascribe_check::LOADER`, and builds its `Project` from what it returns.

```rust,ignore
let config = ascribe_workspace::locate(None)?;
let loaded = ascribe_check::LOADER.load(&config)?;
let project = ascribe_check::Project::from_loaded(loaded);
```

## Rules

- **It depends on `ascribe-core`, `ascribe-model`, and `ascribe-resolve`,** and not on `ascribe-check` or `ascribe-emit`: what's the checks' comes in through the `Loader`.
- **It reads, and writes no files.** Every read of a project's file goes through `ascribe_resolve::FileSystem`; the reads before there's one, of `ascribe.toml` and of the folders searched for it, say why in an `Outside FileSystem:` comment.
- **Errors have codes.** `LoadError` and `LocateError` are `ascribe_core::Coded`, and the binary's exit codes list them.

## Tests

- `tests/all/locate.rs`: `locate` takes a file or the folder holding one, and `find_below` lists the projects below a folder, as the error names them.
- `tests/all/load.rs`: the content model through the loader given, the sources in path order with the unreadable after, and only the side files the list names.
