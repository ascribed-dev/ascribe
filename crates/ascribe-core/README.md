# ascribe-core

The types every Ascribe crate shares, and the three small parsers they all need. It holds types and traits, with only the behavior they need to be exact: line and column conversion, applying edits, normalizing paths.

Every crate builds on it, so a change here is a change to all of them.

## Pieces

| Module | Main types and functions | What it's for |
|---|---|---|
| `src/span.rs` | `FileId`, `Span`, `Location` | Where things are: UTF-8 byte ranges in a file, by id. |
| `src/line_index.rs` | `LineIndex`, `LineCol`, `WideEncoding` | Byte offsets to lines, and to UTF-8, UTF-16, or scalar-value columns, for the editor and the reports. |
| `src/text_edit.rs` | `TextEdit`, `apply_edits` | Edits to a file: fixes, formatting, renames. |
| `src/error.rs` | `Coded` | Errors a caller tells apart by a stable code (`git_not_found`), whatever the message says. Every error a command can stop on implements it; the CLI's `exit.rs` lists the codes. |
| `src/git.rs` | `OPTIONS`, `CLEARED` | What every `git` Ascribe starts runs with, so that reading a repository runs no command a committed file names. `ascribe-diff` and `ascribe-sources` use them; `packages/review/src/shared/git.ts` holds the same lists, and its test compares them. |
| `src/issue.rs` | `Issue`, `DiagnosticSlug`, `Fix`, `Related` | A problem, by registry slug, with its message arguments. Crates find problems as `Issue`s; `ascribe-check` words them. |
| `src/diagnostics.rs`, `build.rs` | One `DiagnosticSlug` constant per diagnostic, and `REGISTRY` | Every slug in the diagnostics registry, `data/diagnostics.toml`, in its order, generated from it by `build.rs`; and the registry's text, which `ascribe-check` reads. A `DiagnosticSlug` can't be made outside this crate, so a misspelled slug doesn't compile. |
| `src/path.rs` | `RelPath`, `classify_destination`, `percent_decode`, `normalize`, `relative_path` | Every path the crates exchange: relative, `/`-separated on every platform, normalized. How a link or image destination becomes one. The one place for path arithmetic on disk paths: resolving `.` and `..` lexically, and the path from one directory to another, with Windows drive letters matched in either case. |
| `src/attributes.rs`, `src/attribute_block.rs` | `parse_attribute_block`, `AttributeBlock` | The attribute-block parser (SPEC §3.3), and what it returns, with spans. |
| `src/info_string.rs` | `InfoString`, `words`, `quote` | A fenced code block's info string (SPEC §3.3.1): its language, and its `key=value` pairs (`phrases`, `title`, `tested`, `request`, `response`). Every reader of one reads it here; `packages/astro/src/code-titles.ts` reads `title` and `tested` by the same rule. |
| `src/availability.rs` | `parse_availability`, `AvailabilitySpec` | The syntax of `@available`, the `available` frontmatter value, and a feature's spec (SPEC §4.4). It doesn't know the content model; checking names against it is `ascribe-model`'s job and the checks'. |
| `src/schema.rs` | `DirectiveSchema`, `builtin_schemas` | What each directive takes, and the built-in directives of SPEC §4. |
| `src/reserved.rs` | `IMAGE_KEYS`, `WIDGET_KEYS`, `HTML_GLOBAL_ATTRIBUTES` | Attribute keys a content model can't declare (SPEC §7.2). |
| `src/consumer.rs` | `Slugger`, `Router`, `ConsumerProfile` | The consumer profile's pieces as traits (SPEC §9.5), so resolution and the emitters don't depend on the crates that implement them. |

## Rules

- **It depends on no other crate of ours,** and outside the workspace only on `serde` and `thiserror`, and `toml` in `build.rs`. A type that needs more belongs in the crate that needs it.
- **It never touches the file system.** `RelPath` is a path's spelling, not a file, and `normalize` and `relative_path` work on spellings too.
- **A diagnostic is added in one place,** `data/diagnostics.toml`. `build.rs` generates its constant, and fails the build on a slug that isn't kebab-case, a slug listed twice, an `off-by-default` entry that isn't `configurable`, or a `review` entry without a `place`.

## Tests

- Unit tests beside each module.
- `tests/attributes_proptest.rs`: the attribute parser never panics, and every span it returns is in range and on a character boundary, whatever the input.
- `tests/info_string_proptest.rs`: the info-string parser never panics, and every word and value it returns is in range and on a character boundary, whatever the input.
