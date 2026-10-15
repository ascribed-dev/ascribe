# Phase 3: A glossary page's rules

Part of [Glossary pages](README.md). Needs phase 2.

## Goal

An empty definition, a long one, and a term off a glossary page are each reported where they're written, and a term follows its section's availability.

```markdown
### Region
@term

@note: Region can't be changed after a project is created.
```

```console
$ ascribe check
glossary.md:12:1: error[glossary-definition-missing]: the term "Region" has no definition: its section doesn't start with a paragraph
```

## Context

- README decisions 6, 8, and 9, and phase 1's decisions 1 and 5.
- `crates/ascribe-check/src/`: where a page-level error is reported, and `description-too-long`, whose limit the long definition shares.
- `crates/ascribe-resolve/src/build/glossary.rs`: a term whose page the build doesn't publish, or whose id the build removes, isn't linked.
- `crates/ascribe-check/src/fix.rs`: a diagnostic's machine-applicable fix, for the one that adds a page to `pages`.
- `examples/quill/ascribe.toml`: a target with versions, and `self-managed-3.3`, a build filtered for one, the shape the availability cases copy.

## Design

### The three checks

- **`glossary-definition-missing`:** a term whose section doesn't start with a paragraph, by phase 1's decision 1. At the heading. An error, since there's nothing for a link's title or a hover.
- **`@intended` at a heading** (README decision 17): the directive gains `@available`'s heading-or-block binding, in SPEC §4.9 and the schema, so that at the top of a section it acknowledges a check that reports at that heading. `glossary-term-unused` reports there, and `term` leaves `[[intended]]`, with its conformance cases.
- **`glossary-definition-long`:** a definition, as derived for a build, over the limit phase 1's decision 5 settles. Advice, configurable in level as every advice is. Counted for each build and reported once, naming the builds where it's over. The fix paragraph says to end the first paragraph after the definition and put the rest in a second.
- **`term-off-glossary-page`:** `@term` on a page that isn't in `pages`. At the directive. Its fix adds the page to `pages`, in `ascribe.toml`, keeping its comments and order.

### A term in a build

A term follows its section. A build that removes the section, by `@available` or by a variant, has no such term: it isn't linked, it has no definition, and `glossary-term-unused` doesn't count that build. A build that removes only the first paragraph has a term with no definition in that build, which `glossary-definition-missing` reports, naming the build.

## Tasks

1. The three checks, their registry entries, and conformance cases.
2. The fix that adds a page to `pages`, with a test that it keeps the file's comments.
3. Conformance cases for a term in a filtered build: a section marked for a version the build doesn't have, and a definition in one arm of a variant.
4. The diagnostics reference, the content model reference, and `changes/`.

## Out of scope

- The editor's quick fixes (phase 4). `ascribe check --fix` applies the fix that adds a page; the editor offers it in phase 4.

## Acceptance criteria

- The page in the goal reports `glossary-definition-missing` at its heading.
- A definition of 320 characters reports `glossary-definition-long`, and one of 280 doesn't.
- `@term` on a page not in `pages` is an error, and `ascribe check --fix` adds the page and leaves no error.
- In a conformance case shaped like `examples/quill`, a term whose section is marked `@available: self-managed 3.5` is linked in the badge build and not in the build filtered for `self-managed 3.3`.

## Verify

```sh
cargo test --workspace --locked
cargo test -p ascribe-conformance --locked
```

## Commits

1. "Report a term with no definition, or one too long to show"
2. "Report @term off a glossary page, with a fix that lists the page"
3. "Link a term only in the builds that publish its section"
