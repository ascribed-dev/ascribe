# Phase 2: Commands that answer questions

Part of [Agents](README.md). Needs no other phase, and can run at the same time as phases 1 and 3. Rust only.

## Goal

Six commands that tell an agent what it would otherwise guess: what a diagnostic means, what the content model allows, which ids a page has, how to write a link, what a reader of a build sees, and where something is used. Each wraps something the checks or the language server already compute.

## Context

- `crates/ascribe-cli/src/commands/`: how a command is declared, loads the project, and reports. `docs/cli.md`: the conventions for options, exit codes, and JSON.
- `tests/conformance/diagnostics.toml` and the code that generates `docs/diagnostics.md` from it (`tests/conformance/tests/docs.rs`): the source for `explain`. The registry has to reach the binary; see how `ascribe-check` reads it today.
- `crates/ascribe-model`: the resolved content model (page types, frontmatter, dimensions, phrases, features, glossary, widgets, builds).
- `crates/ascribe-lsp/src/` (`nav.rs`, `definition.rs`, `links.rs`, `refactor.rs`, `docs.rs`): heading ids, link resolution, references for rename, and the directive documentation hover shows.
- `crates/ascribe-emit` (`plain`): the resolved CommonMark output `render` reuses.
- [Decision 3](README.md#decisions): compact, versioned, capped.

## Design

Every command takes `--format text|json` (text by default, for people; JSON for tools, with `schema_version: 1`), finds the project as phase 1's `check` does (from the path given, or the current directory), and uses exit code 2 when it can't run.

| Command | Answers |
|---|---|
| `ascribe explain <CODE or slug>` | What the diagnostic means, its message, how to fix it, and a short wrong-and-right example. Needs no project. |
| `ascribe model` | The content model, resolved. |
| `ascribe outline <PAGE>` | The page's title, type, and headings with their ids and lines, so `page.md#id` links are right the first time. |
| `ascribe link <TARGET> --from <PAGE>` | Whether a link target (`keys.md`, `keys.md#rotate-keys`) exists as seen from that page, its title, and the link to write. |
| `ascribe render <PAGE> --build <NAME>` | The page as a reader of that build sees it: the plain output, to standard output. |
| `ascribe refs <TARGET>` | Where a page, a heading (`page.md#id`), a fragment, a phrase (`phrase:product`), a feature (`feature:sso`), or a glossary term is used: `file:line` for each. |

Details:

- **`explain`.** The example comes from a new optional registry field (`example.wrong`, `example.right`), written for the 30 or so diagnostics an agent meets most (links, frontmatter, attributes, variants, includes, phrases). Others show the message and fix alone. Many diagnostics fire only under a particular content model, so each example is checked against one: by default a small model kept beside the registry (`tests/conformance/explain-model.toml`, made in this phase and shown by `explain` when an example depends on it), or the example's own `example.model` fragment, laid over the default, when it needs something else. An unknown code is exit code 2, with the closest codes. `ascribe explain --list` prints every code and slug, one per line.
- **`model`.** Sections: `types` (each with its file patterns and frontmatter fields: name, type, required or not, allowed values), `dimensions`, `phrases` (key and value), `features`, `glossary`, `widgets` (with attributes), and `builds`. `--section <NAME>` prints one. The text form is a short Markdown summary an agent can read directly; it's also what phase 4 puts in agent instructions, so write it once and share it. The summary function takes a character budget and cuts long lists to fit, naming the command that gives the rest: `ascribe model` uses 4,000, and phase 4 passes a smaller one.
- **`outline`.** Includes headings that arrive through fragments, marked with the fragment. `--build` limits it to what one build publishes.
- **`link`.** It answers with the server's link resolution, so it agrees with the editor: `exists`, the target's `title`, the `href` to write from that page, and when it doesn't exist, the closest targets. Exit code 1 when it doesn't exist.
- **`render`.** `--build` is required when the model has more than one build. It prints only the page, with no frontmatter unless `--frontmatter` is given. A page the build doesn't publish is exit code 1 with a one-line reason.
- **`refs`.** Capped at 50 places in text and 500 in JSON. A cut list says so (`truncated`, `shown`, `total`, and `next_command`, as phase 1 defines). This is the question agents most need answered well: a text search for a phrase or a fragment finds the wrong things, and misses uses through includes. A target that doesn't exist is exit code 1.

## Tasks

1. `explain`, the registry's `example` field, the default model, and a registry test that every example's `wrong` text produces that diagnostic under its model and its `right` text doesn't.
2. `model`, with the shared text summary, tested against `examples/content-models/`.
3. `outline`, `link`, and `refs`, reusing the server's navigation code. If that code lives in `ascribe-lsp` and the CLI can't depend on it cleanly, move the shared part down to `ascribe-resolve` or `ascribe-check` in its own commit, with no behavior change.
4. `render`.
5. `ascribe --help`: examples before options, and a short "For agents" paragraph naming `check --format concise`, `explain`, and `model`. Agents read `--help` first.
6. `docs/cli.md` (a section per command, with its JSON), `CHANGELOG.md`.

## Out of scope

Anything that writes; `rename` (the editor has it; as a tool it returns edits, in phase 7); formatting (`ascribe fmt` exists).

## Acceptance criteria

- Each command's JSON is documented, versioned, and tested.
- `ascribe model` in text form fits in 4,000 characters for `examples/quill` and says when it cut a long list.
- `ascribe refs` and the editor's **Find All References** agree on the same target.
- Every command works from the repository root on a project in a subfolder, given a path into it.

## Verify

```sh
cargo test --workspace --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo fmt --all --check
```

## Commits

1. "Add ascribe explain"
2. "Add ascribe model"
3. "Add ascribe outline, link, and refs"
4. "Add ascribe render"
