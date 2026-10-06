# tessera-diff

What changed between a git revision and the working tree, as readers will see it: which pages of each build changed, and which blocks on them were added, removed, changed, or moved; and which pages' examples changed while their words didn't. `ascribe diff` and `ascribe drift` are its command line; how to use them, and the JSON they write, are in the [command reference](https://ascribed-dev.com/reference/cli/#ascribe-diff). This README is about the code.

| Module | Role |
|---|---|
| `src/git.rs` | Running `git`: `Repository` (the repository's root and the project's folder in it), `Base` (the revision, and its merge base with `HEAD`), listing a commit's tree, listing the files that differ between a commit and the working tree (renames followed), and reading blobs through one `git cat-file --batch` |
| `src/gitfs.rs` | `GitFs`, a `tessera_resolve::FileSystem` over a commit's tree, with `DiskFs`'s rules for which files are sources and which folders are other projects'; `Revision`, a project's content model and files at a commit |
| `src/tree.rs` | A resolved page as a tree of nodes (blocks, list items, arms), each with its source anchor and a fingerprint of its content without positions |
| `src/align.rs` | Aligning two versions of a page's nodes: a longest-common-subsequence diff over fingerprints, pairing by kind and word overlap, recursion into changed containers, and moves |
| `src/words.rs` | The word-level diff inside changed prose |
| `src/compare.rs` | Comparing builds page by page, the files a change comes from (`because`), and the report's types |
| `src/drift.rs` | `drift`: the snippets whose code differs between a commit and the working tree, read only from the files `git diff` lists, and for each page that shows one whether the page changed apart from it (through `compare.rs`); and the snippets that resolved at the base and don't now |
| `src/html/` | The static report (`--format html`): rendering each changed page now and at the base with anchors, inlining its images, and writing one HTML file around the report's data. `report.js` and `report.css` are built from `packages/review` and `packages/elements` by `pnpm --filter @ascribed/review embed`; don't edit them by hand, and that package's `test/embedded.test.ts` fails while they're out of date. |

The binary links no git library and no HTTP client: `git` is run as a process with a fixed argument list, never through a shell, so nothing else in Ascribe depends on `git` being present. Nothing here is used by the language server.

## Rules

- **Positions never reach a fingerprint.** A block that moved down the file, or was rewrapped, fingerprints the same. Text is compared with whitespace collapsed; what a reader sees besides text (a link's resolved URL, an availability badge's labels, a glossary link, an arm's label) is in the fingerprint too.
- **Anchors follow the [site-render contract's source anchors](https://ascribed-dev.com/contracts/site-render/#7-source-anchors)**, the same strings the site output writes as `data-ascribe-source` and `data-ascribe-via`, and lines are counted as the JSON output's `lines` are.
- **Limits** keep a generated page from taking seconds: `words::MAX_TOKENS` for the word diff, `align::MAX_CHILDREN` for comparing inside a container, `align::MAX_PAIRS` for pairing within one unmatched run. The static report keeps a large change openable: `html::MAX_PAGES` pages rendered, `html::MAX_IMAGE_BYTES` per image.

## Tests

- `tests/git.rs`: reading a project at a revision in a temporary repository: at the root and in a subfolder, a content root above the project folder, a nested project, unusual paths, a rename, no project at the base, base discovery (`origin/main` without `origin/HEAD` included), and the errors, a shallow clone and unrelated histories among them.
- `tests/html.rs`: the static report on projects in memory: both sides rendered with anchors, pages shared across builds stored once, added and removed pages, the page limit, images inlined up to the size limit, and no requests to the network.
- `tests/compare.rs`: the comparison on projects in memory: each change kind, nesting, moves, the limits, and pages changed only through a fragment, a phrase, or a build's settings, page-level changes, CRLF line endings, and which linked pages count as a cause.
- `tests/drift.rs`: the drift report in a temporary repository: a region changed with and without its page, through a fragment, a fragment's snippet, a whole-file snippet, a renamed file and a moved source folder, edits outside a region and moves that aren't changes, a new region, a renamed region or file the page still names, a snippet that never resolved, two builds, and a project at the root and without sources.
- The benchmark is `ascribe diff` in `tests/corpora/benches/perf.rs`.
