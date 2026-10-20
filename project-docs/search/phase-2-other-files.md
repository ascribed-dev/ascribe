# Phase 2: Text from other files

Part of [Search](README.md). Needs phase 1.

## Goal

A fragment's line is found once with the pages that show it, a snippet's code at the code file's lines, and a page's title and description at their keys.

In `examples/quill`:

```console
$ ascribe search "docs repository"
docs/install-agent.md:8: The Quill agent watches your docs repository and syncs changes to Quill. This page…
docs/_fragments/prerequisites.md:4: A docs repository that Quill can read.
    on 1 page: install-agent.md #prerequisites

2 places match, on 1 page.
```

A search for words in a snippet's code gives the code file's line, with the page that shows it. A search for a page's title gives the `title:` line of that page first, then the links that borrow it, each at the link, as phase 1 already finds them.

## Context

- README decisions 8 to 11, and 22. The proposal's "One hit, several places", and the seam's list of what's new.
- `ResolvedBlock::via` (`IncludeSite`) and `snippet` (`Snippet`, its code file and region) in `crates/ascribe-resolve/src/build/tree.rs`.
- `crates/ascribe-resolve/src/snippet/`: how a region's lines are cut, and the tags taken out of it.
- `crates/ascribe-diff/src/tree.rs`: `files_of` and `anchor`, which name the include path, and `diff`'s words for it, "(through _fragments/prereqs.md)".
- The source index's frontmatter: where a page's `title` and its description field are, and their lines. `crates/ascribe-model/src/types.rs`, `FieldRole::Description`.
- `examples/quill/docs/_fragments/` and the pages that include them; a project in `tests/` with a fragment shown on many pages.

## Design

### Included text

The reading keeps the blocks that came through an include, each with its `via`. A match in one is a place at the fragment's line, found once for every page and build that shows it. Its second line is `on N pages:` and the first three, each with its heading and id, ordered by path, then `ascribe refs <fragment>` when there are more. A fragment included in a fragment names the page, not the fragment between.

A fragment's line shown on a page in one build and filtered out of it in another lists the builds it's in on each page only where they aren't all; the place lists the builds that show it anywhere.

### Snippets

A code block a `@snippet` became reads as its code. A match in it is a place at the code file, at the line in the file where the match starts; where tags taken out of the region make that line unknowable, the place is the region's first line, and the second line says so. The second line gives `through code:<file>#<region>` and the pages that show it, as for a fragment. A code file that's outside the project's folder is named as the snippet names it.

### The title and the description

The reading of a page starts with its title and, where its type has one, the field marked `role = "description"`, each a block of kind `title` or `description`, as a build publishes them: phrases in a title replaced, code spans as text. A match is a place at the key's line in the page's frontmatter. Places in titles come first in the order (decision 10).

## Tasks

1. Included blocks in the reading, the place at the fragment's line, and grouping across pages and builds, with tests: a fragment on one page, on thirty, in a fragment, and shown in one build only.
2. Snippets' code, with tests for a region with tags taken out and one without, and a code file outside the project's folder.
3. The title and the description, with tests for a phrase in a title and a type with no description field; the order with titles first.
4. The property test and the test against the plain output, widened to each.
5. The command's text output for each, in `crates/ascribe-cli/tests/output/`, observed again.
6. The docs: `ascribe search`'s section loses "a fragment's text isn't searched yet", and gains the result's forms (the proposal's "One hit, several places" table) and an example of a fragment's place; `changes/command/search-fragments.md`.

## Out of scope

Everything in the README's "Left out" table from phase 3 on.

## Acceptance criteria

- The goal's searches, in `crates/ascribe-cli/tests/`.
- Every character of every block in the reading has a place, by the property test, for each kind added.
- `node scripts/compare/outputs.ts --base main` shows every project byte for byte the same.

## Verify

```sh
cargo test -p ascribe-resolve reading
cargo test -p ascribe-query search
cargo test -p ascribe-cli search
node scripts/compare/outputs.ts --base main
```

## Commits

1. "Find a fragment's text once, with the pages that show it"
2. "Find words in a snippet's code at the code file's lines"
3. "Find a page's title and description"
