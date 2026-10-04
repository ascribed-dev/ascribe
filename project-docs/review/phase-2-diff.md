# Phase 2: `ascribe diff`

Part of [Review](README.md). Requires phase 1 only for the anchor format it reuses. Rust only.

## Goal

`ascribe diff` reports what changed between a base revision and the working tree, as readers will see it: which pages changed in which builds, and which blocks were added, removed, changed, or moved. This phase writes JSON; phase 3 adds the HTML report.

## Context

- `crates/tessera-resolve/src/fs.rs`: the `FileSystem` trait (`sources`, `read`, `probe`), `DiskFs`, `MemoryFs`. `crates/tessera-resolve/src/project.rs`: `Project::load(model, layout, fs)`.
- `crates/tessera-cli/src/context.rs` (`load_project`) and `commands/build.rs`: how a command finds `ascribe.toml`, loads the model, resolves each build, and reports.
- `crates/tessera-resolve/src/build/`: resolving a page for a build (`ResolvedPage`, `ResolvedBlock`, `DropReason`).
- `crates/tessera-emit/src/json.rs`: the resolved tree as JSON, including each block's `source`.
- `docs/cli.md`: option names, exit codes, and the JSON output's conventions (`schema_version`, "ignore fields you don't know").
- [`mockup.html`](mockup.html): this phase has no UI, but its JSON has to carry everything the mockup shows about changes. Check each against the design: added, changed, removed, and moved blocks; word-level highlights inside a changed block; a move's two ends linked to each other; a removed block's place on the page; a change inside a variant arm or a list item; a page that changed only through a fragment, with the fragment named; and the per-page counts ("5 changed · 3 added · 1 removed · 1 moved").
- `Cargo.toml`: `similar` already has a profile entry; check where it's used (tests) before adding it as a normal dependency.

## Design

### The command

```
ascribe diff [--base <REV>] [--build <NAME>]... [--format text|json]
```

- **The base** is a git revision. By default, the merge base of `HEAD` and the repository's default branch (`origin/HEAD`, falling back to `main`, then `master`). The comparison is from the merge base of `<REV>` and `HEAD`, as a pull request shows it, not from `<REV>` itself. `--base-exact` compares against `<REV>` itself.
- **The other side** is the working tree, unsaved edits excluded: what a build would publish now.
- **Text output:** per build, the changed pages with counts (`guides/install.md: 2 changed, 1 added`), and whether each page's own file changed or only something it uses did.
- **Exit codes:** `0` whether or not there are changes; `2` when it can't run (not a git repository, an unknown revision, `git` not found, or the project can't load on either side). `--exit-code` makes changes exit `1`, as `git diff` does.

### Reading the base

A new crate, `tessera-diff`, holds the git access and the comparison, so neither `tessera-resolve` nor the language server depends on `git` being present.

- `GitFs` implements `FileSystem` for a revision by shelling out to `git`: `git ls-tree -r` for the listing and `git cat-file --batch` (one process) for contents. No git library (README decision 5).
- The base's `ascribe.toml` is read from the same revision: the content model can differ between the two sides. If the project doesn't exist at the base, every page is added.
- Paths: git's are repository-relative with `/`; the project's folder inside the repository comes from `git rev-parse --show-prefix`. Handle a project that isn't at the repository's root, a content root above the project folder, and nested projects (skipped, as on disk: apply the same rule as `DiskFs`).
- Run `git` with a fixed argument list (never through a shell), `-c core.quotepath=off`, and `-z` where a listing is parsed.

### The comparison

Per build, for each page on either side:

1. Resolve the page on both sides. A page whose two resolved trees are equal (compare a hash of the tree without spans) is unchanged; skip it. This is what makes "changed through a fragment" fall out for free.
2. **Align blocks.** Fingerprint each top-level block by its content without positions. Run a longest-common-subsequence alignment over the fingerprints. Unmatched runs on both sides at the same place are paired by similarity (word overlap above a threshold) as **changed**; what's left is **added** or **removed**. A removed block whose fingerprint equals an added block elsewhere on the page is **moved**.
3. **Inside containers** (notes, steps, variant arms, details, list items), recurse when a container is matched but not equal, so a one-word change in a step marks the step's paragraph, not the whole list.
4. **Inside a changed prose block,** a word-level diff (`similar`) of its text, as ranges on both sides.

Set a size limit: above a number of blocks or words (choose one, test it), a pair falls back to "changed" without the inner diff. A diff must not take seconds on a generated page.

### The JSON

`--format json` writes one document, with `schema_version`, following `docs/cli.md`'s conventions:

```jsonc
{
  "schema_version": 1,
  "base": { "requested": "origin/main", "commit": "…", "mergeBase": "…" },
  "repository": { "root": "/…", "projectPrefix": "docs/" },
  "builds": [{
    "build": "site",
    "pages": [{
      "path": "guides/install.md",
      "route": "/guides/install/",
      "status": "changed",               // added | removed | changed
      "ownFileChanged": false,
      "because": ["_fragments/prereqs.md"],  // changed files this page's change comes from
      "changes": [{
        "kind": "changed",               // added | removed | changed | moved
        "now": { "source": "guides/install.md:12-14", "via": [] },
        "was": { "source": "guides/install.md:12-13", "via": [] },
        "words": { "now": [[4, 9]], "was": [[4, 7]] }  // word ranges, present for changed prose
      }]
    }]
  }]
}
```

`source` uses phase 1's anchor grammar, so a consumer finds the block in a rendered page by matching `data-ascribe-source`. A removed block has only `was`; it also says which surviving block it came after (`after`), so a marker can be placed.

`because` also names `ascribe.toml` when a model change (a phrase's value, a dimension label) is the cause.

## Tasks

1. `tessera-diff`: `GitFs` and base discovery, with tests in a temporary repository: a project at the root and in a subfolder, a file renamed, a nested project, a path with spaces and non-ASCII characters, and no repository.
2. The comparison, with unit tests for each change kind, nesting, a move, the size limit, and "changed only through a fragment", "through a phrase", and "through a build's settings".
3. The `diff` command with text and JSON output, and CLI tests (`crates/tessera-cli/tests/`).
4. A benchmark beside the existing ones (`tests/corpora/`), on the synthetic 3,000-page project with a change touching one page, a change to a fragment used by 100 pages, and a phrase used everywhere. Record the numbers in the pull request.
5. Document the command and its JSON in `docs/cli.md`; `CHANGELOG.md`.

## Out of scope

HTML output (phase 3); the language server (phase 4); comparing two arbitrary revisions with neither being the working tree (add `--head <REV>` only if it falls out for free).

## Acceptance criteria

- A page that changed only because a fragment, a phrase, or the model changed is reported, with `because` naming the cause.
- A pure reformat that doesn't change the resolved tree (`ascribe fmt`, rewrapped lines) reports no change.
- An unchanged project takes about as long as checking it twice; the benchmark's numbers are in the pull request.
- Works on Windows paths, and with the project in a subfolder of the repository.

## Verify

```sh
cargo test --workspace --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo fmt --all --check
```

## Commits

1. "Read a project from a git revision"
2. "Compare resolved pages block by block"
3. "Add ascribe diff"
