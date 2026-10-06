# ascribe-syntax

Parsing one Ascribe source file into Ascribe's syntax tree. `parse` reads the file with `comrak-ascribe`, the CommonMark parser with Ascribe's block-level changes, converts comrak's tree into Ascribe's own, and adds what CommonMark doesn't know: directive lines and their heads, containers and groups, titles, bindings, phrases, and attribute blocks on images and table rows. Every node has an exact byte span.

What the language allows is in [SPEC.md](../../SPEC.md): this crate implements SPEC §3, the structural rows of §8.2, and the inline extensions of §5.1 and §5.3.

## Pieces

| Module | Main types and functions | What it does |
|---|---|---|
| `src/lib.rs` | `parse`, `code_phrases` | The entry points. `parse` never panics: anything malformed is an issue, and the tree keeps as much as it can. |
| `src/options.rs` | `ParseOptions` | What decides which lines are directives: the directive schemas (built-ins and a project's widgets) and the note types. `ascribe_fmt::options_from_model` builds them from a content model. |
| `src/tree.rs` | `ParsedDocument`, `Block`, `BlockKind`, `DirectiveLine`, `Container`, `Group`, `Arm`, `Inline` | Ascribe's tree. It exposes none of comrak's types, so nothing downstream depends on the parser behind it. |
| `src/convert.rs` | | comrak's tree into Ascribe's, with spans turned from comrak's lines and columns into byte offsets. |
| `src/head.rs` | | A directive line's head: name, attribute block, colon, and primary, each with a span. |
| `src/structure/` | `bound_block`, `bound_heading` | The structure pass: containers and their end lines, groups of arms such as `@variant`, titles, bindings, and the structural issues. |
| `src/inline/` | | Phrase candidates (`{key}`) in text, link destinations, and opted-in code; attribute blocks after images and at the end of table rows. |
| `src/unknown.rs` | | Lines shaped like a directive with an unknown name: they stay text, with a warning and the closest known name. |

```rust
use ascribe_syntax::{parse, ParseOptions};

let doc = parse("@note {type=tip}: Careful.\n", &ParseOptions::default());
assert!(doc.issues.is_empty());
```

## Rules

- **It parses one file and knows nothing else:** not the content model (only the schemas in `ParseOptions`), not other files, not the file system. Whether a link's target exists, or an attribute's value is allowed, is decided later, in `ascribe-resolve` and `ascribe-check`.
- **It depends only on `ascribe-core` and `comrak-ascribe`.** No other crate sees comrak's types; a change to the fork stays behind this crate's tree.
- **Issues are reported by registry slug** (`ascribe_core::Issue`) and worded by `ascribe-check`.
- **A span covers exactly its node's source text,** counted in UTF-8 bytes from the start of the file, frontmatter included.

## Tests

- `tests/parse.rs`: directive heads, primaries, issues, and lines that only look like directives.
- `tests/structure.rs`: containers, groups, titles, bindings, and the structural issues.
- `tests/inline.rs`: phrase candidates and image attribute blocks, in every place the spec allows them.
- `tests/definitions.rs`: link reference definitions, with their spans and phrases.
- `tests/spans.rs`: every span covers exactly its source text, over the CommonMark examples, the example projects, and every conformance case.
- `tests/agreement.rs`: a property test that the head parser and the fork's block scanner agree, and that `parse` never panics.
- The conformance suite (`tests/conformance`) runs the spec's own examples through `parse`, and `tests/commonmark` runs the CommonMark examples against the fork.
