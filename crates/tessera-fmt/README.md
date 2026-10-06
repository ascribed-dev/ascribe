# tessera-fmt

The formatter: it rewrites Ascribe's own constructs into canonical form (SPEC §8.3) and returns the smallest edits that do it. `ascribe fmt` and the language server's document formatting both call it. It never re-renders Markdown: prose, tables, code, and every line that isn't an Ascribe construct stay byte for byte as written.

## Pieces

| Module | Main types and functions | What it does |
|---|---|---|
| `src/lib.rs` | `format`, `format_parsed`, `format_source`, `options_from_model` | The entry points, and the walk over the tree. `format` returns sorted, non-overlapping `TextEdit`s; `format_source` applies them. `options_from_model` gives the parse options a project's content model implies. |
| `src/head.rs` | | A directive line's head: the space before the attribute block, the colon, and the space after it. |
| `src/attributes.rs` | | Attribute blocks, on directives and after images: spacing, order, quoting, and empty blocks. |
| `src/indent.rs` | | The indentation of directive lines and end lines inside the document, a list item, or a block quote. |
| `src/blank.rs` | | The blank lines between a following-block directive and its block. |
| `src/skip.rs` | `NON_BLOCKING` | Which reported issues make the formatter leave a construct alone. |

```rust
use tessera_core::FileId;

let model = tessera_model::load_str("spec = \"0.1\"\n", FileId::new(0)).expect("a valid model");
let options = tessera_fmt::options_from_model(&model);
let out = tessera_fmt::format_source("@note{ type = tip }:Careful.\n", &options, &model);
assert_eq!(out, "@note {type=tip}: Careful.\n");
```

## Rules

- **Edit only Ascribe's bytes.** A rule works from spans in the syntax tree and touches only a directive's head, an attribute block, an indentation, or a blank line. Link reference definitions aren't Ascribe constructs, so they're never touched.
- **Leave alone what has an error.** A construct the parser reported an error on is skipped, since the formatter can't know what was meant. Warnings don't stop it; `skip.rs` lists them, and `tests/registry.rs` keeps that list in step with the registry.
- **Idempotent, and the outline is kept.** Formatting twice gives what formatting once did, and the result parses to the same blocks, directives, attribute values, and text.
- **It depends on `tessera-core`, `tessera-syntax`, and `tessera-model`,** and not on `tessera-resolve` or `tessera-check`: formatting one file needs nothing about the others. It doesn't read or write files; the caller does.

## Tests

- `tests/rules.rs`: one group per rule of SPEC §8.3, and what the formatter leaves alone.
- `tests/inputs.rs`: idempotence and the outline, over every conformance input, the example projects, the spec and project docs, and the CommonMark examples, each first made ugly in several ways.
- `tests/fuzz.rs`: the same over random documents.
- `tests/registry.rs`: `NON_BLOCKING` agrees with the registry.
- The conformance suite's `format` cases (`tests/conformance/cases/format/`) check the canonical forms from the spec.
