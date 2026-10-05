# Phase 8: Coverage

Part of [Docs](README.md). Requires phases 6 and 7. Rust only.

**Region-level only.** [The back-test](back-test.md) found a file-level report right in 11 of 42 reports (26%), under the threshold of about a third, so this phase doesn't build file-level coverage: a page covers regions, never whole files or globs. Read the write-up before starting.

## Goal

`ascribe drift` reports, for a change, the pages whose covered regions changed while the page didn't. A page covers the regions it takes snippets from without being told to, and can name more regions in its frontmatter. The report goes in the CI job's summary and nowhere else.

## Context

- Phase 6's write-up: how often a file-level report was right on this repository, what made it wrong, and why only regions are built.
- Phase 7: sources, the address (`<source>:<path>#<region>`), and extraction.
- `crates/tessera-diff/`: reading a revision through `git`, finding the base and merge base, the shallow-clone error, and the rule that the binary shells out to `git` and links no git library. `crates/tessera-cli/src/commands/diff.rs`: a command built on it, with its options, exit codes, and JSON.
- `tests/conformance/diagnostics.toml`: where new diagnostics are registered.
- [Decisions 7, 10, 11, and 12](README.md#decisions).

## Design

### What a page covers

- **Implicitly:** every region it takes a snippet from, and every whole file it takes as a snippet.
- **Explicitly:** a reserved frontmatter field, `covers`, a list of addresses, each naming one region of one file.

  ```yaml
  covers:
    - code:crates/tessera-cli/src/commands/check.rs#options
    - code:crates/tessera-cli/src/commands/diff.rs#options
  ```

- An address with an unknown source, or outside the source's `include`, is an error, and so is one without a region: a whole file or a glob in `covers` is refused, with a message that says to name a region. One that matches no file or no region is a warning: the code moved, which is itself drift.
- A page's coverage is its own. Fragments have no frontmatter; a fragment's snippets count for every page that includes it.
- `ascribe check` validates `covers` and never reads `git`, so a clean project stays clean whatever the history.

### `ascribe drift`

```
ascribe drift [--base <REV>] [--build <NAME>]... [--format text|json|summary] [--exit-code]
```

It compares the working tree with a base, found as `ascribe diff` finds it.

1. **Which files to look at.** One listing of the changed paths between the two revisions, limited to the files that hold a covered region or a whole-file snippet, with renames followed.
2. **For each region,** the region is extracted from the file at both revisions and compared as text, after dedenting and trimming trailing spaces. An edit elsewhere in the file, or a region that only moved, isn't a change.
3. **Whether the page changed.** A page changed when its resolved content differs in any build compared (so a change to a fragment counts), or when its own file changed at all. This reuses `ascribe diff`'s comparison.

Three groups come out:

```text
compared with origin/main (1a2b3c4)

Examples that changed. The page shows the new code; check the words around it:
  reference/content-model.md
    code:examples/quill/ascribe.toml#dimensions

Covered regions that changed, on pages that didn't:
  reference/cli.md
    code:crates/tessera-cli/src/commands/check.rs#options  (+4 −1)

Changed along with the code they cover:
  guides/review.md
```

The first group is the one a snippet makes possible: the example updated itself, and the sentence that explains it didn't.

**Formats.** `text`, as above. `json`, versioned, with the base and, per page, each covered address that changed, its source, and whether the page changed. `summary`, Markdown for a CI job's summary: the same three groups, each page linked, nothing when there's nothing to report.

**Exit codes:** `0` whatever it finds; `2` when it can't run, including a shallow clone, with `diff`'s message. `--exit-code` makes the first two groups exit `1`. This repository doesn't use it (decision 11).

**Cost.** One `git` listing for the change, and file contents only for changed files that have a covered region or a whole-file snippet. Measure it on this repository and on the corpora, and put the numbers in the pull request.

### What it can't know

A changed region isn't always a changed behavior. The report says what changed and how much, and leaves the judgment to a person. Don't add rules for "meaningful" changes here; phase 6 measured how noisy a file-level report is, and phase 9 measures the region-level one on real pull requests. File-level coverage stays out unless phase 9's numbers say otherwise.

## Tasks

1. `covers`: the reserved field, its diagnostics in the registry, and conformance cases.
2. The comparison, with tests in a temporary repository: a covered region changed with and without the page; a page changed only through a fragment; a whole file or a glob in `covers`, refused; a renamed file; a region changed, moved, and untouched while its file changed; a snippet's region changed; two builds; a project in a subfolder; a shallow clone.
3. The command and its three formats, with CLI tests.
4. The timings.
5. The command reference, the content-model reference, `guides/drift.md` (the coverage part, and a CI recipe that writes the summary), `CHANGELOG.md`.

## Out of scope

History (`--history`), a way to mark a page as checked, and review dates; a pull request comment, the editor, review, and the agent hook; sources in another repository; using it on our own docs (phase 9).

## Acceptance criteria

- A pull request that changes a covered region without its page is reported, naming both; a change elsewhere in the region's file isn't.
- A snippet's region changing puts its pages in the first group, and an edit outside the region doesn't.
- A page changed in the same pull request, or through a fragment, is in the third group.
- `ascribe check` behaves the same whatever the repository's history.
- In a shallow clone, `drift` exits `2` and says to fetch more history.
- A project without sources is unaffected.

## Verify

```sh
cargo test --workspace --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo fmt --all --check
```

## Commits

1. "Let a page say which code it covers"
2. "Compare covered code between two revisions"
3. "Add ascribe drift"
