# Phase 4: For agents

Part of [Search](README.md). Needs phase 3.

## Goal

An agent searches through `ascribe_search` or `--format json`, and is told to before it writes a page.

In `examples/quill`, `ascribe search '"sign in to quill cloud"' --format json` writes the proposal's document: `schema_version` 1, the place at `docs/install-agent.md` line 62 with `in: "text"`, `match` in characters, `through: [{"phrase": "cloud"}]`, `arm`, its one page with heading and id, and the totals. Through `ascribe mcp`, `ascribe_search` with `{"query": "\"sign in to quill cloud\"", "project": "examples/quill"}` returns the text the command writes; with `response_format: "detailed"`, the document. A second call with another query, the project unchanged, doesn't resolve the project again. After `ascribe agents sync`, the rules and the skill say to search before writing a page, and `ascribe agents prompt new-page` does too.

## Context

- README decisions 5, 11 to 15, 17, and 20. The proposal's "For agents", with its JSON example; [decision 52](../decisions.md) (the questions' JSON), [51](../decisions.md) (a cut list's fields), and [56](../decisions.md) (the tools are a contract).
- "A new command or option" in `project-docs/checklists.md`, its last three items above all.
- `crates/ascribe-query/src/refs.rs` (`Refs`, `Place`) and `crates/ascribe-cli/src/commands/refs.rs` (`answer`, `next_command`, the two limits).
- `crates/ascribe-cli/src/shapes.rs` (`SHAPES`) and `schemas/`; `docs/content/contracts/json-reports.md`.
- `crates/ascribe-cli/src/mcp/tools.rs` (`Spec`, `call_refs`, `builds()`, `response_format()`, `with_concise`) and `crates/ascribe-cli/src/mcp/cache.rs` with `crate::answer::Loaded`.
- `crates/ascribe-cli/src/agents/skill.rs`, `ascribe_query::rules`, and `crates/ascribe-cli/src/agents/prompts.rs` (`new-page`); `plugins/ascribe/`, generated from the binary; `@ascribed/cli`'s copy of the skill, held equal by a test.

## Design

### The document

`ascribe_query::Search` serializes as the proposal shows, with phases 2 and 3 filled in:

- **Each place:** `file`, `line`, `column` (decision 20), `in`, `text` (the block as a reader reads it, whole), `match` (`[start, end]` in characters of `text`), `through` (each `{"phrase": key}`, `{"include": path}`, `{"code": "<file>#<region>"}`, or `{"title": page}`, in order), `arm` (or null), `pages` (each `page`, `title`, `heading`, `id`, and `builds`, at most three), `pages_total`, and `refs_command` (or null). A place wholly in a phrase's value is `in: "phrase"` at `ascribe.toml`, with `pages_total` counting the places that use it and `refs_command`. A snippet's place that can only be given as its region's lines has `end_line`, the region's last; every other place has `end_line` null.
- **The document:** `schema_version` 1, `ascribe_version`, `query`, `builds` (searched), `places`, `pages_matched`, `total`, `shown`, `truncated`, `next_command`, and `source_places` (phase 3's, each `file` and `line`, at most three, empty when something matched) with `not_searched_builds`.
- **With `--summary`:** `pages`, each `page` and `places`, in place of `places`.

The JSON's limit is 500 places, the text's 50, as `refs`.

### The tool

`ascribe_search`, titled "Search pages", with the command's arguments under their names (the checklist): `query`, `paths`, `build` (`builds()`), `type`, `case`, `word`, `summary`, `limit`, `project` (a file or folder in the project, when no path is given, as `ascribe_refs` has), and `response_format`, concise by default. Its description has one example, as `ascribe_refs`'s does, and says to try other words when nothing matches. It returns what the command writes, its output schema `search.schema.json` with the concise form.

### What the server keeps

The MCP server keeps each build's reading of each page with the project it has loaded, made when a search first asks for it and dropped when the listing of the project's files changes, as the project is. A second search is then the scan alone.

### Telling agents

Each of these gains a line, in the words of each:

- **The rules** `ascribe agents sync` writes: before writing a page about a subject, `ascribe search` for it, and link to what's there or extend it.
- **The skill:** `ascribe search` among the commands that answer questions, with what it finds that `grep` doesn't.
- **The `new-page` prompt:** its first step searches for the subject, and a hit changes the request to editing the page found.

The repository's own synced copies are rewritten with `ascribe agents sync`, and the plugin and `@ascribed/cli`'s skill by their tests.

### The decision

A file in `project-docs/decisions/`, named by this phase's issue: `ascribe search`'s JSON, its schema and version, its exit codes, and the tool's name and arguments, under decisions 51, 52, and 56.

## Tasks

1. The document's types in `ascribe-query`, with doc comments, `SHAPES`, `schemas/search.schema.json`, and the contract page.
2. `--format json` and `--summary`'s JSON in the command, with output in `crates/ascribe-cli/tests/output/` (`search.json`), observed again.
3. The tool, its test through the MCP server's harness, and its test that a second query doesn't load the project again.
4. The readings kept in the server's cache, dropped with the project.
5. The rules, the skill, and the `new-page` prompt; the synced copies, the plugin, and `@ascribed/cli`'s skill regenerated, and their diffs read.
6. The decision file.
7. The docs: `ascribe search`'s section gains the JSON document's fields and an example; `docs/content/contracts/json-reports.md` includes its schema; `docs/content/agents/commands.md`'s tools table under `ascribe mcp`; `docs/content/agents/index.md`'s table of commands in "The loop" and the skill's list; `changes/agents/search-tool.md`; and the surface map.

## Out of scope

The editor.

## Acceptance criteria

- The goal's command, tool call, and second call, in tests.
- `ascribe agents sync --check` passes in the repository.
- `node scripts/compare/outputs.ts --base main` shows every project byte for byte the same.

## Verify

```sh
cargo test -p ascribe-cli search
cargo test -p ascribe-cli mcp
ASCRIBE_BLESS=1 cargo test -p ascribe-cli shapes   # then read the diff
cargo run -p ascribe-cli -- agents sync --check
node scripts/compare/outputs.ts --base main
```

## Commits

1. "Write ascribe search's answer as JSON"
2. "Offer search to agents as ascribe_search"
3. "Tell agents to search before they write a page"
