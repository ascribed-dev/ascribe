# Phase 5: Memo files

Part of [Memos](README.md). Needs phases 1 and 2. A thread of its own.

## Goal

A longer memo in `memos/` is placed on its page, section, or block, reported when it's outdated or detached, and listed by `ascribe outline` and `ascribe refs`. In `examples/quill`, `memos/agent-first.md`:

```markdown
---
about: install-agent.md#install-agent
quote: Install the agent package
---

The agent goes first because its installer writes the config file that
`quill init` reads. The other way round, `init` fails with an error that
doesn't mention the agent.
```

```console
$ ascribe outline install-agent.md
…
## Install the agent  #install-agent
   memo  memos/agent-first.md  The agent goes first because its installer writes the config file that
…
```

Reword "Install the agent package" in the page, and `ascribe check` reports `memo-outdated` at the memo's `quote`. Change the heading's id, and it reports `memo-detached` at `about`, naming the ids the page has.

This is the thinnest slice through the second seam: a file that isn't a page, placed on a page's block by what it names and what it quotes, without anything written back.

## Context

- README decisions 11 to 16 and 23 to 26.
- `crates/ascribe-resolve/src/fs.rs` and `layout.rs`: `FileSystem`, its `files_in` and `read_file`, which list and read a folder relative to the project root for a source's copies, and what a project's layout knows about its folders.
- `crates/ascribe-resolve/tests/all/file_reads.rs`: the test that catches a project read outside `FileSystem`.
- `crates/ascribe-resolve/src/index/`: the source index's headings, ids, and references, and `references.rs` beside it.
- `crates/ascribe-query/src/refs.rs`: the target grammar and `Place`; `outline.rs`: `Outline` and `OutlineHeading`.
- `crates/ascribe-cli/src/commands/outline.rs` and `refs`, and the MCP tool `ascribe_outline` in `crates/ascribe-mcp/`.
- How `sources/<name>/` is kept out of the content root, for the same rule on `memos/`.
- `examples/quill/docs/install-agent.md`.

## Design

### Reading memos

A memo store gives the memos a project has: each file's path, `about`, `quote`, body, and where each key is. The binary has one interface and, in this phase, one implementation: the committed folder, `memos/` beside `ascribe.toml`, every `.md` file in it and its subfolders. Phase 6 adds the personal one.

**It's read through `FileSystem`, with what's there.** `memos/` is outside the content root, so no page listing reaches it. `files_in` lists a folder relative to the project root and `read_file` reads a file there; they're how a source's copies under `sources/<name>/` are read, and every file system implements both: `DiskFs`, `MemoryFs`, the editor's `BufferFs` and `LayerFs`, which see an unsaved memo, and `GitFs`, which reads one at a commit. So the store adds no new kind of read, and `file_reads.rs` holds that it adds no read outside them.

A memo file is parsed on its own: frontmatter, then a body that's kept as text. Nothing in the body is resolved, and it isn't a page: it has no route, no outputs, and no place in `ascribe build`.

### Placing

Each time memos are read, each is placed against the source index, as decision 24 says:

1. `about` is parsed by `ascribe refs`'s grammar. A target that doesn't exist makes the memo detached.
2. With no `quote`, the memo is on its target: a page, a section, or a content model entry.
3. With a `quote`, the blocks of the target are searched in source order, each block's source with whitespace runs read as one space. The first that holds it is the memo's block. None does: the memo is outdated, and stays on the page or section.

Nothing is cached between runs, and nothing is written to the memo.

### Checks

`memo-invalid`, `memo-detached`, `memo-outdated`, and `memo-long` (decision 23), each reported on the memo file, at its `about` or `quote` line. `ascribe check` reads the committed folder only.

`memos/` is beside `ascribe.toml` by definition, so it's inside the content root only in a project whose `content-root` is `.`. That's the case `memo-invalid` is for: once, at the folder's first file, saying to set a content root below the project folder, and the folder's files are read as pages, as they are today.

### `ascribe outline` and `ascribe refs`

As decision 25 says: the page and each heading gain `memos`, and a memo is a place in `ascribe refs` with `use` `memo`. Their schemas are rewritten and the text output gains a line for each. The MCP tool returns what the command does.

## Tasks

1. The memo store over `files_in` and `read_file`, the committed folder, and parsing a memo file, with tests on `MemoryFs` and `file_reads.rs` passing.
2. Placing, with tests for each kind of target, a quote in the first of two blocks, a quote across two blocks, a quote in a fragment, and whitespace.
3. The four checks, their registry entries, and conformance cases: a project with memos in `tests/conformance/cases/memos/`, and one with `content-root = "."` and a `memos/` folder.
4. `ascribe outline` and `ascribe refs`, their JSON schemas, and the MCP tool.
5. `examples/quill/memos/`, with the memo from the goal and one about `phrase:product`.
6. A reference page for memo files in `docs/content/writing/`, the outline and refs references, the diagnostics reference, `docs/content/command/project-files.md` for `memos/`, `changes/`, and the surface map.

## Out of scope

- The editor (phase 6). Until then, a memo file's problems appear in the editor only as `ascribe check` reports them.
- Rename (phase 6). Until then, renaming a page or an id leaves its memos detached, and `memo-detached` says so.
- Personal memos (phase 6).
- The prompts and the rules (phase 7).

## Acceptance criteria

- `ascribe outline install-agent.md` in `examples/quill` lists the memo under its heading, and `--format json` has it under that heading's `memos` with `state` `placed`.
- `ascribe refs install-agent.md#install-agent` lists the memo file, with `use` `memo`.
- Rewording the quoted text makes `ascribe check` report `memo-outdated`, and the outline lists the memo as outdated on the section.
- Changing the heading's id makes `ascribe check` report `memo-detached`, and the outline doesn't list it.
- `ascribe build` writes nothing from `memos/`, and `node scripts/compare/outputs.ts --base main` shows every project's outputs unchanged.
- A memo file is read through `FileSystem`: `crates/ascribe-resolve/tests/all/file_reads.rs` passes.

## Verify

```sh
cargo test --workspace --locked
cargo run -p ascribe-cli -- outline install-agent.md --config examples/quill
cargo run -p ascribe-cli -- check --config examples/quill
node scripts/compare/outputs.ts --base main
```

## Commits

1. "Read a project's memos from memos/"
2. "Place a memo on the block it quotes"
3. "Report a memo that's outdated, detached, invalid, or long"
4. "List a page's memos in ascribe outline and ascribe refs"
