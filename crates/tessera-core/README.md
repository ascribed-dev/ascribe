# tessera-core

The types every Ascribe crate shares, and the two small parsers they all need. It holds types and traits, with only the behavior they need to be exact: line and column conversion, applying edits, normalizing paths.

Every crate builds on it, so a change here is a change to all of them.

## Pieces

| Module | Main types and functions | What it's for |
|---|---|---|
| `src/span.rs` | `FileId`, `Span`, `Location` | Where things are: UTF-8 byte ranges in a file, by id. |
| `src/line_index.rs` | `LineIndex`, `LineCol`, `WideEncoding` | Byte offsets to lines, and to UTF-8, UTF-16, or scalar-value columns, for the editor and the reports. |
| `src/text_edit.rs` | `TextEdit`, `apply_edits` | Edits to a file: fixes, formatting, renames. |
| `src/error.rs` | `Coded` | Errors a caller tells apart by a stable code (`git_not_found`), whatever the message says. Every error a command can stop on implements it; the CLI's `exit.rs` lists the codes. |
| `src/issue.rs` | `Issue`, `DiagnosticSlug`, `Fix`, `Related` | A problem, by registry slug, with its message arguments. Crates find problems as `Issue`s; `tessera-check` words them. |
| `src/diagnostics.rs` | One `DiagnosticSlug` constant per diagnostic | Every slug in `tests/conformance/diagnostics.toml`, in the same order. A `DiagnosticSlug` can't be made outside this crate, so a misspelled slug doesn't compile. |
| `src/path.rs` | `RelPath`, `classify_destination`, `percent_decode`, `normalize`, `relative_path` | Every path the crates exchange: relative, `/`-separated on every platform, normalized. How a link or image destination becomes one. The one place for path arithmetic on disk paths: resolving `.` and `..` lexically, and the path from one directory to another, with Windows drive letters matched in either case. |
| `src/attributes.rs`, `src/attribute_block.rs` | `parse_attribute_block`, `AttributeBlock` | The attribute-block parser (SPEC §3.3), and what it returns, with spans. |
| `src/availability.rs` | `parse_availability`, `AvailabilitySpec` | The syntax of `@available`, the `available` frontmatter value, and a feature's spec (SPEC §4.4). It doesn't know the content model; checking names against it is `tessera-model`'s job and the checks'. |
| `src/schema.rs` | `DirectiveSchema`, `builtin_schemas` | What each directive takes, and the built-in directives of SPEC §4. |
| `src/reserved.rs` | `IMAGE_KEYS`, `WIDGET_KEYS`, `HTML_GLOBAL_ATTRIBUTES` | Attribute keys a content model can't declare (SPEC §7.2). |
| `src/consumer.rs` | `Slugger`, `Router`, `ConsumerProfile` | The consumer profile's pieces as traits (SPEC §9.5), so resolution and the emitters don't depend on the crates that implement them. |

## Rules

- **It depends on no other crate of ours,** and outside the workspace only on `serde` and `thiserror`. A type that needs more belongs in the crate that needs it.
- **It never touches the file system.** `RelPath` is a path's spelling, not a file, and `normalize` and `relative_path` work on spellings too.
- **A diagnostic is added in two places,** `tests/conformance/diagnostics.toml` and `src/diagnostics.rs`, in the same order. `tests/registry.rs` fails until they match.

## Tests

- Unit tests beside each module.
- `tests/attributes_proptest.rs`: the attribute parser never panics, and every span it returns is in range and on a character boundary, whatever the input.
- `tests/registry.rs`: `src/diagnostics.rs` holds the registry's slugs, in its order.
