# Phase 9: Retiring a line

Part of [Versions](README.md). Needs phases 7 and 8.

## Goal

A line leaves the window in one command, and every mark it made dead is rewritten or offered for removal.

```console
$ ascribe versions retire "self-managed 3.4"
3.4 is archived, at https://docs.example.com/archive/3.4/.
Rewrote 212 marks in 64 files. No live line's output changed.
9 blocks are in no live line. Review each: ascribe versions retire "self-managed 3.4" --show-unused
3 pages were replaced by others. Their old addresses, and where each should go:
  /3.4/install-legacy/  →  /install/
```

Elsewhere this is the most expensive step in versioned docs: the research found a public runbook of 16 steps, with scripts to strip conditions and clean-up by hand. Ascribe can do it because it knows what each mark means.

## Context

- README decisions 16, 17, and 21, and phase 1's decisions 10 and 11.
- The [proposal](../proposals/versions.md): "Then clean up".
- Phase 3's checks: the marks that no longer do anything, which this phase gives fixes.
- Phase 7's `versions release`: how a command rewrites marks, proves no output changed, and refuses a dirty tree.
- The [search and answers plan](../search-and-answers/README.md)'s host phase: the shape of the headers list, which the redirects list follows.
- `ascribe check --fix`: what makes a fix `safe`.

## Design

### What's refused

Exit 2, with nothing written, when:

- the line isn't live;
- it's the current line, or it isn't the oldest live line;
- it has no address: neither `--url <address>` nor an `archived` entry that gives one. The message says to run `ascribe versions archive` first, and why: after the clean-up the source no longer holds what the line needs;
- a file it would write can't be put back, by the rule `release` follows: tracked and unchanged in git, or `--force`;
- no build is versioned by the target. A one-site project has nothing to archive and no mark that's inert, and `release` already keeps its list to one live line (phase 1, decision 10). The message says so.

### The list

The line moves from `live` to `archived`, with its address and `ended`, which is `ends` when the line had one and today otherwise. Today is the day every command that checks takes: UTC, or `ASCRIBE_TODAY` when it's set, so the command's tests don't change with the calendar. Its `ends` entry goes. `ascribe.toml` is edited in place.

### The marks

With the line gone, the window is smaller, and each mark for the target is rewritten to the simplest spec that means the same in every line still live.

| A mark | With 3.5 and 3.6 live | Why |
|---|---|---|
| `self-managed 3.4` | `self-managed` | Every live line has it |
| `self-managed deprecated 3.2` | `self-managed deprecated` | The state is still worth showing |
| `self-managed (preview 3.3, ga 3.5, deprecated 3.6)` | `self-managed (ga, deprecated 3.6)` | The steps before the window collapse into the one in effect when it opens |
| `cloud, self-managed 3.4` | `cloud, self-managed` | Other targets are untouched |
| `self-managed`, on a page with no narrower scope | Removed | What's left says nothing: every target its scope allows, generally available |

- These are safe fixes: no live line's output changes, and the command proves it by building before and after, as `release` does. That holds because a line's output never showed a version from before the window (phase 1, decision 11). A build that isn't versioned does show it, so a project that also has a plain badge build is told which of its pages change there, and the rewrite needs `--force` to go ahead.
- A feature's spec in `ascribe.toml` is rewritten the same way.
- A mark is removed only by the last row's rule, and never when a narrower scope depends on it.
- A planned line counts as in the window.

### Content in no live line

A block, a row, a section, or a page that no live or planned line has is listed and not deleted: it removes content. `--show-unused` prints each with its page and first line. `--remove-unused` deletes them, and says what it deleted. Until then phase 3's warning reports each.

### A page that was replaced

When a page being removed is one another page `replaces`:

- The `replaces` line is removed from the page that has it, with the page.
- The old address and the replacement's are reported, in the text and in the JSON as a list, each entry `from` and `to`, shaped as the headers list is. Nothing is written for a host (README decision 21).
- The same list holds every address of the retired line, each going to the same page in the current line, or to the archive when the current line has none.

### After it

`ascribe versions list` shows the line archived. `ascribe build` no longer writes its output, and removes the folder it wrote before, under the output directory's ownership rules. The site's list of lines shows it as an archive.

## Tasks

1. The refusals, and the list's edit.
2. The simplest spec for a mark given a set of lines: one function, with a table of cases as its tests, the histories among them, and a property test that the rewritten spec and the original agree in every remaining line.
3. The rewrite across a project, the features, and the proof that no output changed.
4. `--show-unused` and `--remove-unused`.
5. The redirects list, in text and JSON.
6. The fixes for phase 3's advice, so `ascribe check --fix` applies the safe ones outside a retirement too.
7. Retire `examples/quill`'s older line in a test, from a copy.
8. The command reference, the guide's section on retiring, with the order (archive, host, retire, redirects), entries in `changes/`, the map of the surface, and the decisions recorded.

## Out of scope

- Writing a host's redirects file.
- Deleting an archive, or checking that its address answers. `ascribe report links` already follows the addresses in the list once phase 8 lists them.

## Acceptance criteria

- With no address, the command exits 2 and names `ascribe versions archive`.
- In `docs/`, where no build is versioned, it exits 2 and says a release keeps the list.
- With `ASCRIBE_TODAY` set, `ended` is that day.
- After retiring 3.4 in a copy of `examples/quill`, every remaining line's output is byte for byte what it was, apart from the list of lines.
- Each row of the table above is a test, and the property test passes.
- No content is deleted without `--remove-unused`.
- The JSON's redirects list has an entry for every page the retired line published.
- Run twice, the second run refuses: the line isn't live.

## Verify

```sh
cargo test --workspace --locked
cargo run -p ascribe-cli -- versions retire "self-managed 3.4" --dry-run --url https://example.com/3.4/ --config examples/quill
```

## Commits

1. "Work out the simplest mark that means the same in every live line"
2. "Retire a line"
3. "List what a retired line leaves in no line"
4. "Report where a retired line's addresses should go"
