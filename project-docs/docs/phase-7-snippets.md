# Phase 7: Snippets

Part of [Docs](README.md). Requires phase 1. Can run at the same time as phases 2 to 5. A change to the language: the spec, the content model, the compiler, and the checks.

## Goal

A page takes a code example from a real file, by name, and Ascribe extracts it whenever it checks, builds, or diffs. The example can't go stale, because there's no copy: change the file and the page changes. This phase also adds named sources, the way a project says which files outside its content it may read.

This is brainstorm [section 1](../brainstorm.md#1-code-snippets-from-tested-code-bluehawk-built-in).

## Context

- The brainstorm section: what Bluehawk does, what Ascribe would add, and the hard parts. Bluehawk's license is unclear, so its tag syntax is reimplemented from its documentation, not its code. Check the license again and say what you found.
- `SPEC.md`: `@include` (how a fragment becomes part of a page, and how problems in it are reported at the include), fenced code blocks and their attributes, and §8.2's table of diagnostics. A language change starts here.
- `crates/tessera-model/` and the content-model reference and contract: how a table in `ascribe.toml` is declared, loaded, and checked.
- `crates/tessera-resolve/src/fs.rs`: the `FileSystem` trait and the rule that a project's files stay inside its folder. Sources are the one way out.
- `crates/tessera-syntax`, `tessera-resolve`, `tessera-check`, `tessera-emit` (each output, and source anchors), `tessera-diff` (`because`, and reading files at a revision through `GitFs`).
- `examples/monorepo`: `service/` beside `docs/`, the shape sources are for.
- [Decisions 7 and 9](README.md#decisions).

## Design

### Named sources

```toml
# A set of files outside the project's content that its pages may refer to.
[sources.code]
path = ".."                               # relative to the project's folder
include = ["crates/**", "examples/**"]    # what's readable; everything else isn't
ignore = ["**/target/**"]
```

- A project can declare several. Names follow the content model's rules for keys.
- `path` must be inside the same `git` repository as the project when there is one. A `path` that doesn't exist is an error in the content model.
- **`git`, `branch`, and `commit` are reserved keys** in a source's table: using one is an error that says sources in another repository aren't supported yet. Reserving them now keeps the table's meaning open (decision 9).
- Without a source, nothing outside the project's folder can be read, and the error says how to declare one.

### The address

Everywhere a page refers to code, it writes `<source>:<path>` with an optional `#<region>`:

```markdown
@snippet: code:examples/quill/ascribe.toml#dimensions
```

`<path>` is relative to the source's `path`, with `/` on every platform, and must match the source's `include`. There's no relative-path form: an address never depends on where the page is, or on where the code is checked out. Phase 8's `covers` uses the same address.

### In a page

- `@snippet` becomes a fenced code block. The language comes from the file's extension, or `{lang=toml}`. Attributes a code block takes (a title, `phrases`) apply as they do to a block written in the page.
- Without `#<region>`, the snippet is the whole file.
- It's a block, allowed wherever a code block is: in steps, variant arms, notes.

### In the code

Regions are marked in comments, with Bluehawk's tags, so teams that use it can keep their files:

```toml
# :snippet-start: dimensions
[dimensions.platform]
values = ["linux", "macos", "windows"]
# :snippet-end:
```

This phase reads: `:snippet-start: <name>` and `:snippet-end:`; `:remove-start:` and `:remove-end:`, and a line ending `:remove:`, which leave lines out. Nested and overlapping regions are allowed; tag lines never appear in a snippet. The extracted text is dedented by its common indentation.

A tag counts only in a line comment, for a small table of extensions and their comment markers (`//`, `#`, `--`, `;`, `<!-- -->`). A file with an extension not in the table can be used whole, and not by region; the table grows by request.

Reserved for later, and named in the spec: `state`, `replace`, `uncomment`, `emphasize`.

### Everywhere else

- **Check:** an unknown source, a file that doesn't exist or isn't included by its source, a region that doesn't exist (with the closest names), unbalanced or duplicate tags, and a file that isn't text. Each is reported at the `@snippet` line, with the place in the code as related information, as a problem in a fragment is.
- **Build:** every output gets the code block. In the JSON output and for source anchors, the block's source is the `@snippet` line, with the address and the code's lines beside it.
- **Diff and review:** a snippet whose code changed changes its pages, and `because` names the code by its address, as it names a fragment. `GitFs` reads the code at the base revision, through the same source.
- **Format:** `ascribe fmt` puts the directive in canonical form and never touches the code file.
- **No history, no network:** `check` and `build` read the working tree only (decision 7).

### Performance

Reading files outside the content root adds work to every check. A snippet's file is read once per check however many pages use it, and `diff`'s listing of the repository at a revision should be limited to the project and its sources' `include`, not the whole tree. Measure the corpora with and without snippets (add some to the synthetic project) and record it.

## Tasks

1. `SPEC.md`: sources, the address, the directive, the tags, the reserved tags and keys, and the diagnostics, with their §8.2 rows.
2. `[sources.<name>]` in the content model: loading, checks, the contract page, and conformance cases.
3. Extraction, with unit tests: regions, nesting, removal, dedenting, each comment marker, CRLF files, and a file with no trailing newline.
4. Resolve, check, and emit; then diff.
5. The benchmark.
6. The directive reference, the content-model reference, a new guide `guides/drift.md` (what the checks are; this phase writes the examples part), `CHANGELOG.md`.

## Out of scope

The editor beyond what the server already does for diagnostics (Later); running code; the reserved tags; highlighting lines; sources in another repository; `covers` (phase 8).

## Acceptance criteria

- Changing a tagged region changes every page that uses it, in `check`, `build`, and `diff`, with no other step.
- A missing region is an error at the `@snippet` line that names the regions the file has.
- A project with no source can't read outside its folder, and the error says how to declare one.
- No page contains a path that leaves the project (`../`): only addresses.
- The spec, the registry, and the conformance cases agree, and the tests that tie them pass.
- The benchmark's numbers are in the pull request.

## Verify

```sh
cargo test --workspace --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo fmt --all --check
```

## Commits

1. "Specify sources and snippets"
2. "Let a project name the sources its pages refer to"
3. "Extract a snippet from a tagged file"
4. "Resolve, check, and emit snippets"
5. "Follow a snippet's code in diff"
