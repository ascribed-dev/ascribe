# Phase 7: A release, and what changed

Part of [Versions](README.md). Needs phase 3.

## Goal

Release day is one command, and what differs between two lines is one report.

```console
$ ascribe versions release "ascribe 0.3.0" --next 0.4.0
0.3.0 is live, and current.
Rewrote 86 marks from `next` to `ascribe 0.3.0`, in 31 files.
`next` now names 0.4.0, which is planned.
No output changed.
```

Ascribe's own release uses it, in place of the step done by hand after 0.2.0.

It needs phase 3 and nothing after, so it can be built beside phases 4 to 6.

## Context

- README decisions 14 and 20, and phase 1's decisions 8 and 10.
- The surface audit's finding 30, in `project-docs/surface-audit/findings.md`: the 86 marks rewritten by hand.
- `ascribe permalinks retire`: the command that edits `ascribe.toml` in place, keeping its comments and order.
- `ascribe check --fix`: how a safe fix is applied to source files and reported.
- `crates/ascribe-diff/src/git.rs`: how `diff` runs git, finds the repository, and says there isn't one. The binary runs git nowhere else but for sources.
- `RELEASING.md`, "Each release".
- `project-docs/checklists.md`: "A new command or option".

## Design

### `ascribe versions list`

Each target with a list: its lines, newest first, with status, the build and base path of each live one, `ends`, an archived line's address, and how many pages each live line publishes. `--format text|json`. It reads the project and builds nothing.

### `ascribe versions release "<target> <line>"`

In this order, so that no step changes an output:

1. Refuse, with exit 2 and nothing written, when: the line is planned and isn't the oldest planned line; it's neither planned nor a line between the current one and the oldest planned (phase 1, decision 10: a patch release nobody planned); a feature names it and no planned line follows and `--next` isn't given; or a file it would write can't be put back (below).
2. Rewrite every mark that names a feature whose spec is the line (`@available: next`, `available: next` in frontmatter, and a table row's attribute) to the line's spec. While the feature still names the line, the two mean the same.
3. Point each such feature at the following planned line, or at `--next`, which is added to `planned`.
4. Move the line from `planned` to `live`, or add it there.
5. When no build is versioned by the target, move the line that was current to `archived`, with no address (phase 1, decision 10). A one-site project keeps one live line, and no mark is rewritten: its badge build still shows each version.
6. Build every build before and after in memory, and compare.

Step 4 is the one that changes outputs, and only in the ways a release should: the line's content stops reading "planned", and in a versioned build the old current line moves under its number. Step 5 changes none. The command says which outputs changed and how, and anything else is an error it reports before writing.

**A file it can't put back.** The command rewrites marks across a project, so it writes only where the change can be undone. It asks git, as `diff` does, whether each file it would write is tracked and unchanged, and refuses when one isn't, naming the files. In a folder that isn't a git repository, or with no git to run, nothing could undo the rewrite, so it refuses and says why. `--force` writes in either case. `--dry-run` never needs git.

- `--dry-run` says what it would do and writes nothing.
- `--format text|json`. The JSON lists the files and the marks.
- It edits `ascribe.toml` in place, keeping comments and order.
- A feature with a spec of several targets is rewritten for the released target and keeps the rest, which the command names so a person can look.

### `ascribe versions changes "<target> <from>" <to>`

From the marks alone, with nothing built: by page, the blocks, rows, sections, and pages that are in one line and not the other, and those whose state differs.

- Three groups: added, removed, and changed state, each with the page, the heading it's under, and its first line.
- A page that `replaces` another is one entry, not a removal and an addition.
- `--format text|json|markdown`. The Markdown form is a starting draft of release notes, with headings by page.
- Either line may be live, planned, or archived, as long as the source still holds it. A line `retire` has cleaned up is refused, with the archive's address.

### Ascribe's own release

`RELEASING.md`'s step after a release becomes the command, and its hand-written instructions go. The release workflow's check that nothing is left behind stays, as the proof.

## Tasks

1. `versions list`.
2. `versions release`: the refusals, the rewrite, the feature, the list, and the comparison.
3. `versions changes`, with its three formats.
4. Tests in a copy of `docs/` at the commit before 0.2.0's hand rewrite: the command's result against what was done by hand.
5. `RELEASING.md`.
6. The command reference, the guide's section on release day, entries in `changes/`, the map of the surface, and the decisions recorded.

## Out of scope

- Publishing anything. The command edits a project's files.
- Rewriting marks for a line that's left. Phase 9.

## Acceptance criteria

- On the commit before 0.2.0's rewrite, `ascribe versions release` gives the files the hand rewrite gave, or the pull request says where they differ and which is right.
- Run twice, the second run refuses: the line is already live.
- With no following planned line and no `--next`, it exits 2 and writes nothing.
- In `docs/`, where no build is versioned, the release leaves one live line and the old one archived with no address, and every mark for an earlier release is as it was. In `examples/quill`, the old current line stays live.
- A patch release that isn't in `planned` is released, and `next` still names what it named.
- In a folder that isn't a repository it refuses without `--force`, and `--dry-run` works there.
- `ascribe versions changes "self-managed 3.4" 3.5` in `examples/quill` lists the page new in 3.5 as added and the rewritten page once.
- The three commands follow the checklist for a new command, and the surface map's diff is read against it.

## Verify

```sh
cargo test --workspace --locked
cargo run -p ascribe-cli -- versions list --config examples/quill
cargo run -p ascribe-cli -- versions changes "self-managed 3.4" 3.5 --config examples/quill
cargo run -p ascribe-cli -- versions release "ascribe 0.3.0" --next 0.4.0 --dry-run --config docs
```

## Commits

1. "List a project's lines"
2. "Make a release one command"
3. "Report what differs between two lines"
4. "Release Ascribe's own docs with the command"
