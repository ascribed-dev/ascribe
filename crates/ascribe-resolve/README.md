# ascribe-resolve

The project graph, and everything between parsing a file and emitting a page: reading a project's files, indexing each one, resolving what its references name, expanding includes and snippets, resolving a page for a build, and keeping all of that current as files change. It reports nothing itself: the problems it finds are recorded, by registry slug, for `ascribe-check` to report.

It's the largest of our crates, and the one the others reach the file system through.

## Pieces

| Module | Main types and functions | What it does |
|---|---|---|
| `src/fs.rs` | `FileSystem`, `DiskFs`, `MemoryFs`, `Probe`, `Sources`, `is_source_path`, `in_nested_project` | The one way to read a project's files. It decides which files are sources (`.md`, not under a `.` name, not in a nested project's folder), reads them, probes for files with exact-case names on every platform, follows symbolic links, and finds a file's real path. `DiskFs` reads a real project, optionally caching directory listings; `MemoryFs` holds one for tests and for unsaved text. `ascribe_diff::GitFs` implements it over a commit. |
| `src/layout.rs` | `Layout` | Where the content root and the output directory are, and the boundary a local file must stay inside (`Layout::is_allowed`). |
| `src/index/` | `index_file`, `FileIndex`, `Heading`, `Reference`, `Include` | What's true of one file alone: its tree, frontmatter, title, headings and their source ids (SPEC §5.5), includes, links, images, phrases, snippets, and availability markers. A pure function of the file's text, path, and the content model; it never touches the file system. |
| `src/project.rs` | `Project`, `Resolution` | Every file's index, what each reference names, and the edges between files, forward and back (`includers`, `links_to`, `asset_users`). `Project::load` builds it from a `FileSystem`. |
| `src/uses.rs` | `Usable`, `Use`, `Project::uses`, `Project::use_counts` | Where each page, fragment, heading, and content model entry is used. The language server's Find All References and its inventory both ask it, so a count is the length of the list. |
| `src/references.rs` | `reference_target`, `include_target`, `resolve_reference` | The one implementation of the rules for links, images, and includes: what a destination names, whether it's inside the boundary, and whether it's there. The index and `ascribe-check`'s file-level checks both call it. |
| `src/snippet/` | `Address`, `resolve_snippet`, `snippet_issues`, `CodeFiles` | The one implementation of the rules for `@snippet` (SPEC §4.8): reading an address, finding its file through a source, and taking a region out by its tags (`src/snippet/tags.rs`). |
| `src/expand.rs` | `Project::expand`, `ExpandedPage` | Includes replaced by their targets, and snippets by their code, recursively. Every block keeps the file and span it was written in, and the includes it came through. |
| `src/build/` | `BuildResolver`, `ResolvedPage`, `ResolvedBuild`, `DefaultRouter` | Resolving a page for one build, in SPEC §9.2's order: availability, build modes, phrases, page ids, links and assets, glossary. Each pass is its own module; `src/build/mod.rs` lists them. |
| `src/incremental/` | `IncrementalProject`, `Change`, `Affected`, `Snapshot`, `Version` | Keeping a project current as files change, redoing only what each change affects. The language server runs on it. `src/incremental/mod.rs` says what each change invalidates and how file ids are kept. |
| `src/slug/` | `GithubSlugger`, `slugger_by_name` | Heading slugs: a port of `github-slugger`, with a table and fixtures generated from the npm package by `src/slug/generate.mjs`. |
| `src/astro.rs` | `AstroRouter` | The `astro` profile's routes: the URL Astro gives each page. |

```rust,ignore
let layout = Layout::from_model(&model);
let fs = DiskFs::new(project_root, &layout);
let project = Project::load(Arc::new(model), layout, &fs);
let page = project.expand(&RelPath::parse("index.md")?);
let resolved = project.resolve_build(&build, &router);
```

## Rules

- **Read files only through `FileSystem`.** Nothing else in this crate touches the disk, outside tests. A crate that reads a project's files any other way has to repeat the boundary, case, and link rules, and has got them wrong before.
- **The index is a pure function of one file.** `index_file` knows nothing about other files or the disk, so it can be cached until the file or the model changes. What depends on other files is in `Project`.
- **One implementation of each rule.** References and snippets are decided in `src/references.rs` and `src/snippet/`, and every tool calls them, so `ascribe check`, the build, and the language server can't disagree.
- **Report nothing.** Problems are recorded as `Issue`s, on the project (`Project::problems`) or on a resolved page (`ResolvedPage::problems`), for the checks to word and report.
- **Incremental equals from scratch.** After any sequence of changes, an `IncrementalProject`'s snapshot equals a `Project::load_with_ids` of the same files, and the file ids mean what they mean to `ascribe_check::Project`.

## Dependencies

It uses `ascribe-core`, `ascribe-syntax`, and `ascribe-model`, and no other crate of ours: `ascribe-check`, `ascribe-emit`, `ascribe-diff`, `ascribe-sources`, `ascribe-lsp`, and `ascribe-cli` all build on it. Outside the workspace it uses only `serde_yaml_ng`, for frontmatter.

## Tests

- `tests/headings.rs`: source ids, titles, and the rest of the per-file index.
- `tests/includes.rs`: include expansion.
- `tests/uses.rs`: where pages, headings, and the content model's entries are used.
- `tests/references.rs`: links, images, and assets: what each names, from which file, and what's wrong with it.
- `tests/snippets.rs`: the code block a `@snippet` becomes, and which problem `snippet_issues` reports.
- `tests/build_content.rs`, `tests/build_modes.rs`: build resolution: phrases, page ids, links, assets, the glossary, build modes, and availability.
- `tests/quill.rs`, `tests/build_quill.rs`: `examples/quill` indexes, expands, and resolves under each of its builds with no problems.
- `tests/incremental.rs`: what each kind of change invalidates and caches.
- `tests/file_reads.rs`: every read of the disk in the workspace's crates outside `FileSystem` says why, in a comment starting `Outside FileSystem:`.
- `tests/incremental_differential.rs`: a property test that, after each step of random edits, the incremental project equals a load from scratch.
- Unit tests in `src/fs.rs` cover discovery and links on a real disk; the slugger's run the upstream fixtures.
- `benches/incremental.rs`: an incremental update on 3,000 pages, run with `cargo bench -p ascribe-resolve --bench incremental`.
