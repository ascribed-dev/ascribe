# Phase 8: Coverage

Part of [Docs](README.md). Requires phases 6 and 7. Rust only.

**Snippets only.** [The back-test](back-test.md) found a file-level report right in 11 of 42 reports (26%), under the threshold of about a third, so this phase doesn't build file-level coverage. Its estimate for regions a page names only to say it covers them was about a third right, missing most of the real drift, so this phase doesn't build that either: a page covers the code it shows, and nothing else. Read the write-up before starting.

## Goal

`ascribe drift` reports, for a change, the pages whose examples changed while the words around them didn't. A page covers the regions and files it takes snippets from, without being told to. The report goes in the CI job's summary and nowhere else.

## Context

- Phase 6's write-up: how often a file-level report was right on this repository, what made it wrong, and why only snippets are built.
- Phase 7: sources, the address (`<source>:<path>#<region>`), and extraction.
- `crates/tessera-diff/`: reading a revision through `git`, finding the base and merge base, the shallow-clone error, and the rule that the binary shells out to `git` and links no git library. `crates/tessera-cli/src/commands/diff.rs`: a command built on it, with its options, exit codes, and JSON.
- `tests/conformance/diagnostics.toml`: where new diagnostics are registered.
- [Decisions 7, 10, 11, and 12](README.md#decisions).

## Design

### What a page covers

- Every region it takes a snippet from, and every whole file it takes as a snippet.
- A page's coverage is its own. A fragment's snippets count for every page that includes it.
- There's no field for naming more code. An explicit `covers` waits for phase 9's numbers from real pull requests; the back-test's estimate doesn't support it, and it would have authors tag code only so a page can point at it.

### `ascribe drift`

```
ascribe drift [--base <REV>] [--build <NAME>]... [--format text|json|summary] [--exit-code]
```

It compares the working tree with a base, found as `ascribe diff` finds it.

1. **Which files to look at.** One listing of the changed paths between the two revisions, limited to the files that hold a snippet's region or are a whole-file snippet, with renames followed.
2. **For each region,** the region is extracted from the file at both revisions and compared as text, after dedenting and trimming trailing spaces. An edit elsewhere in the file, or a region that only moved, isn't a change.
3. **Whether the page's own text changed.** Apart from its snippets, a page changed when its own file changed, or when its resolved content differs in any build compared through anything else it uses (so a change to a fragment counts). This reuses `ascribe diff`'s comparison.

Two groups come out:

```text
compared with origin/main (1a2b3c4)

Examples that changed. The page shows the new code; check the words around it:
  reference/content-model.md
    code:examples/quill/ascribe.toml#dimensions

Examples that changed along with the page:
  guides/review.md
```

The first group is the one a snippet makes possible: the example updated itself, and the sentence that explains it didn't. Each example says how much it changed (`+4 −1`).

**Formats.** `text`, as above. `json`, versioned, with the base and, per page, each snippet address that changed, its source, and whether the page's own text changed. `summary`, Markdown for a CI job's summary: the same two groups, each page linked, nothing when there's nothing to report.

**Exit codes:** `0` whatever it finds; `2` when it can't run, including a shallow clone, with `diff`'s message. `--exit-code` makes the first group exit `1`. This repository doesn't use it (decision 11).

**Cost.** One `git` listing for the change, and file contents only for changed files that a snippet uses. Measure it on this repository and on the corpora, and put the numbers in the pull request.

### What it can't know

A changed region isn't always a changed behavior. The report says what changed and how much, and leaves the judgment to a person. Don't add rules for "meaningful" changes here; phase 6 measured how noisy a file-level report is, and phase 9 measures this one on real pull requests. File-level coverage and an explicit `covers` stay out unless phase 9's numbers say otherwise.

## Tasks

1. The comparison, with tests in a temporary repository: a snippet's region changed with and without the page's own text; a page changed only through a fragment; a fragment's snippet changed; a whole-file snippet changed; a renamed file; a region changed, moved, and untouched while its file changed; two builds; a project in a subfolder; a shallow clone.
2. The command and its three formats, with CLI tests.
3. The timings.
4. The command reference, `guides/drift.md` (the drift part, and a CI recipe that writes the summary), `CHANGELOG.md`.

## Out of scope

An explicit `covers` field; history (`--history`), a way to mark a page as checked, and review dates; a pull request comment, the editor, review, and the agent hook; sources in another repository; using it on our own docs (phase 9).

## Acceptance criteria

- A pull request that changes a snippet's region without the page's own text puts the page in the first group, naming both; an edit outside the region doesn't.
- A page whose own text changed in the same pull request, or through a fragment, is in the second group.
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

1. "Compare snippets' code between two revisions"
2. "Add ascribe drift"
