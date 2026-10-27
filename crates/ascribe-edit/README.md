# ascribe-edit

The edits Ascribe's commands share with its language server: renaming, a release, and retiring a line, and the edits to pages, `ascribe.toml`, `ascribe.nav`, and memo files they're made of. None needs a client. `ascribe rename` and `ascribe versions` call them from the command line, and the language server calls the same code for the editor's rename (`F2`) and its edits when a file moves, so the command and the editor change the same places.

An edit works from a `nav::Ctx`: a snapshot of the project, the content model and its text, and the position encoding its edits are written in. The language server makes one for each request, with the documents the editor has open over the disk; a command loads one from the disk with `project::load`, as the server loads a project with no document open. Nothing here writes a file: each edit returns each file's text after, or its edits, and the caller writes them.

## Pieces

| Module | Main types and functions | What it does |
|---|---|---|
| `src/rename.rs` | `rename`, `Renaming`, `Renamed`, `RenameError` | `ascribe rename`: a phrase, a dimension's value, a heading's id, or a page that moves, renamed by name, with the edits of `refactor.rs`, and in the projects of the files `[from]` takes from (SPEC §7.9). |
| `src/refactor.rs`, `src/refactor/values.rs` | `rename`, `rename_model_key`, `will_rename` | The editor's rename at a position, in a page or in `ascribe.toml`, and the edits a page, a fragment, or a folder moving makes, as a `WorkspaceEdit`. |
| `src/release.rs` | `release`, `Releasing`, `Released`, `ReleaseError` | `ascribe versions release`: the edits to `ascribe.toml` and the marks in the sources, and the outputs before and after, built in memory. |
| `src/retire_line.rs` | `retire_line`, `unused_in`, `RetiringLine`, `RetiredLine`, `RetireError` | `ascribe versions retire`: a line archived, each mark rewritten to the simplest spec that means the same in the lines left, and the content no line has. |
| `src/model_file.rs` | `ModelFile`, `TakenFile`, `read_taken` | `ascribe.toml` edited as text, with the files `[from]` takes from: an entry goes after the last of its table, and a value is replaced where it's written, so comments, blank lines, and order stay. |
| `src/navigation.rs` | `NavChange`, `change`, `moved`, `text_edit` | `ascribe.nav` edited piece by piece (SPEC §7.8): the Pages view's changes, and what a move changes in it. |
| `src/nav_file.rs` | `sections`, `complete` | Completion's edits in `ascribe.nav`: a key, a `folder` the file doesn't list yet, or a page its section's `pages` doesn't name yet. |
| `src/memos.rs` | `read`, `id_edits`, `entry_edits`, `move_edits` | The memos, and the edits that keep a memo's `about` naming what it named when that's renamed or moves. |
| `src/nav.rs` | `Ctx`, `Lines`, `hit_at` | What an edit works from, and what's under a cursor in a file. |
| `src/project.rs` | `load`, `Load`, `Loaded` | A project loaded from the disk, as the language server loads one with no document open. |
| `src/fsx.rs` | `BufferFs`, `LayerFs` | The disk, with the editor's open buffers over it when there are any. |
| `src/position.rs`, `src/uri.rs` | `Encoding`, `path_to_uri`, `uri_to_path` | Positions in the client's encoding, and `file:` URIs. |

## Rules

- **It depends on `ascribe-core`, `ascribe-syntax`, `ascribe-model`, `ascribe-resolve`, `ascribe-check`, and `ascribe-emit`,** and on no crate that uses it: not `ascribe-lsp` or `ascribe-cli`. A release's outputs are built with `ascribe_emit::outputs_in_memory`, and an edit is checked with `ascribe_check`.
- **Edits are LSP's shapes.** An edit is a `lsp_types::WorkspaceEdit` in the `Ctx`'s encoding, as the editor applies it; a command's `Ctx` is UTF-8, and `rename` applies the edits to each file's text.
- **Nothing is written.** The caller writes each file, and moves the one that moves.

## Tests

- The unit tests beside the code: `model_file.rs`, `navigation.rs`, `nav_file.rs`, `position.rs`, `uri.rs`, `retire_line.rs`, and `refactor/values.rs`.
- Renaming by name is held by `ascribe-lsp`'s tests, which share the server's project fixture: `crates/ascribe-lsp/tests/all/rename.rs` (what only renaming by name does), `model_edits.rs` (the same places the editor's rename changes), and `from.rs` (the projects of the files `[from]` takes from). The editor's rename and file moves are tested there too, over the server.
- `ascribe rename` and `ascribe versions` are run as the binary in `crates/ascribe-cli/tests/all/rename.rs` and `versions.rs`.
