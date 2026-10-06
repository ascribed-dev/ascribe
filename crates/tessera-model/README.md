# tessera-model

Loading and validating a project's content model, `ascribe.toml`, into a typed `ContentModel`, and reading and writing `ascribe.lock`. Every loading rule is a `model-` diagnostic in the registry, reported at the span of the key or value that breaks it.

What the content model may contain is SPEC §7 and the [content model reference](https://ascribed-dev.com/reference/content-model/). This README is about the code.

## Pieces

| Module | Main types and functions | What it does |
|---|---|---|
| `src/lib.rs` | `load`, `load_with_file`, `load_str`, `load_str_in` | The entry points. `load` reads the file and runs the rules that need the disk (the content root exists, glossary links name files); `load_str` runs every other rule on text alone, for unsaved buffers and tests; `load_str_in` is text with a project folder. |
| `src/model.rs` | `ContentModel`, `Build`, `Source`, `Consumer`, `Dimension`, `Feature`, `Widget` | The typed model, and the questions other crates ask of it: `directive_schemas`, `type_for`, `is_fragment`, `build`, `check_availability`, and the rest. |
| `src/loader.rs` | | The walk over the TOML tree, and the sections that define names: project, dimensions, lifecycle, features, notes, phrases. |
| `src/sections.rs` | | Content types, fragments, the glossary, images, widgets, the consumer, builds, and the editor. |
| `src/fields.rs`, `src/types.rs` | `Field`, `FieldType`, `validate_frontmatter` | Field and attribute types, and checking a page's frontmatter against its type. |
| `src/inline.rs` | `InlineMarkup`, `Segment` | Code spans in a field that sets `inline = "code"`. |
| `src/pattern.rs` | `Pattern` | The glob syntax of `files` and fragment patterns. |
| `src/lock.rs` | `Lock`, `LockedSource`, `file_hash` | `ascribe.lock`: the commit each source in another repository is pinned to, and the hash of each copied file. |
| `src/names.rs` | `suggest` | Name grammars, and the closest known name for a misspelled one. |

```rust,ignore
let model = tessera_model::load("ascribe.toml")?;
for build in &model.builds {
    println!("{}", build.name);
}
```

Warnings (`model-name-case`, `model-build-filter-excluded`) don't stop loading; they're in `ContentModel::warnings`. When loading fails, the issues returned include the warnings found before the failure.

## Rules

- **It depends only on `tessera-core`.** It doesn't parse sources, so it doesn't use `tessera-syntax`; the availability-spec parser is in `tessera-core`, and this crate checks a spec's names against the model.
- **It reads one file,** `ascribe.toml`, in `load` (`src/lib.rs`), and touches the disk otherwise only for the rules that need it. Everything else about a project's files is `tessera-resolve`'s.
- **Every rule is a registry diagnostic** (`tests/conformance/diagnostics.toml`), reported as an `Issue` at the offending key or value.
- **Declaration order is kept** (`toml`'s `preserve_order`): the order a model declares builds, dimensions, and attributes in is the order the other crates use them in.

## Tests

- Unit tests beside each module.
- `tests/load.rs`: loading the example content models in `examples/content-models/`, the rules that need the file system, and the queries.
- `tests/rules.rs`: one failing model per loading rule, checking the slug, the message variant, and the line the issue points at.
- The conformance suite's `model` cases (`tests/conformance/cases/model/`) check the same rules from the spec.
