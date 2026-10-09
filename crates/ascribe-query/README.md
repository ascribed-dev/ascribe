# ascribe-query

Answers to the questions an author or an agent asks about a project, without changing anything: what a diagnostic means, what the content model allows, which headings a page has, whether a link works, what a reader of a build sees, and where something is used. Each is one function that takes a loaded project and plain arguments and returns a typed answer that serializes to the command's JSON; `ascribe explain`, `model`, `outline`, `link`, `render`, and `refs` call them and print the answer, and `ascribe mcp` will call the same functions. None of them prints.

Each answer wraps something the checks or the language server already compute, so the three agree: `link` resolves a target with `ascribe_resolve::resolve_reference`, as `ascribe check` does; `outline` lists the headings the editor's completion offers after `page.md#` (`Project::link_headings`); and `refs` is the editor's **Find All References** (`Project::uses`), which `crates/ascribe-lsp/tests/references.rs` holds it to.

## Pieces

| Module | Main types and functions | What it does |
|---|---|---|
| `src/explain.rs` | `explain`, `list`, `Explanation`, `example_slugs` | A diagnostic from the registry (`ascribe_check::Registry`), by code or name, with its fix and its example. `example_slugs` checks an example against its model, for `tests/explain_examples.rs`. |
| `src/model.rs` | `model`, `summary`, `Section`, `ModelReport` | The content model as `ascribe model` shows it. `summary` is its Markdown, cut to a character budget, which agent instructions will use too. |
| `src/outline.rs` | `outline`, `Outline` | A page's title, type, and the headings a link can name, with a build's or without. |
| `src/link.rs` | `link`, `LinkAnswer` | Whether a link target works from a page, the link to write, and the closest targets when it doesn't. |
| `src/refs.rs` | `refs`, `Asked`, `Refs` | Where a page, a heading, a fragment, or a content model entry is used. |
| `src/render.rs` | `render`, `Rendered` | A page as a build's `plain` output writes it, without writing anything. |
| `src/error.rs` | `QueryError` | What stops an answer, each with a stable code (`ascribe_core::Coded`). |

## Rules

- **An answer is a contract.** Every field has a doc comment, the `json-schema` feature derives the schema `crates/ascribe-cli/src/shapes.rs` writes to `schemas/`, and `schema_version` rises only when a field is removed or changes meaning.
- **Every example is checked.** A diagnostic's `example` in `tests/conformance/diagnostics.toml` is checked against `tests/conformance/explain-model.toml`, or its own `example.model` laid over it: the wrong page reports that diagnostic and nothing else, and the right page reports nothing.
