# Proposal: the rest of the cache

A proposal, not a plan. It says what each piece is and what it would look like, so the ideas can be judged before anything is designed in detail. It picks up section 15 of the [brainstorm](../brainstorm.md#15-the-cache-whats-left), and follows the [first cache proposal](https://github.com/ascribed-dev/ascribe/blob/097cc3d4/project-docs/proposals/cache.md), whose link cache and link history are built.

## The goal

Ascribe remembers what it learned in an earlier run where that saves real time or tells a team something it can't see otherwise, and never where it could change what a check finds.

Four things are left from the brainstorm's list. They aren't one feature, and this proposal judges each on its own:

| | What it is | Kind |
|---|---|---|
| Vale results | A page's alerts, kept by the hash of its prose | A cache: changes how long a run takes |
| The delivery spec's results | `afdocs`'s answers for a page | Neither, as it turns out: see below |
| Differences between runs | "2 problems are new since the last report, and 5 are gone" | A history: changes what a report says |
| A baseline | A committed list of a project's existing findings, so a team adopting the checks sees only new ones | A record: authored, shared, and reviewed |

The rule from the first proposal holds for the first: deleting a cache changes nothing but time. The second and third live in `.ascribe/history/`, where deleting costs one report that can't say what changed. The fourth isn't a cache at all, and is committed.

## What exists

- **The link cache and the link history are built** (decision 121): `.ascribe/cache/links/`, one file an address, written whole through a rename, trusted by nothing that reads it; and `.ascribe/history/links.json`, from which `ascribe report links` holds a link that answered before as unconfirmed, and says which findings are new since the last run and how many are gone. `[checks.links]` has `reuse-days` and `confirm-days`; the report has `--recheck` and `--no-history`. The library is given the day, and `ASCRIBE_TODAY` sets it in tests. Only the report's code opens either store, and a test holds that.
- **The rest of `ascribe report` keeps nothing between runs:** the inventory, the problems `ascribe check` finds, the overdue and orphaned pages, the examples, and `agents`, which runs `afdocs` on a site's address (`crates/ascribe-check/src/site.rs`).
- **Vale runs once for every file asked about,** on the prose Ascribe extracts with a map back to the source (`crates/ascribe-check/src/prose/`). It runs with `ascribe check --vale`, or in every check with `[checks.vale] in-check = true`, in the editor on save, and in the agents' hook. `.ascribe/vale/` holds the preset and the vocabulary Ascribe writes for it, or a project brings its own `.vale.ini` and styles.
- **A finding has no identity that survives an edit.** A diagnostic is a code, a slug, a message, a place (file, line, column), and the builds it's in. Move a paragraph down a line and the same finding is at another place. `@intended` gets around this by sitting at the block, in the source.
- **Nothing is a baseline.** A team that turns on the advice checks in a project of 2,000 pages sees every existing finding at once. `[checks]` can turn a check off, and `@intended` acknowledges one finding with a reason. Neither says "these, for now".

## What was measured

On 14 October 2026, on an Apple M-series laptop, with Vale 3.24.0 and a release build of Ascribe from `main`.

| Project | `ascribe check` | With Vale's own style | With Google, Microsoft, and write-good |
|---|---|---|---|
| Ascribe's docs, 147 files, `quiet` preset | 0.16 s | 0.23 s | not run |
| The synthetic project, 3,100 short pages, `quiet` preset | 0.5 s | 1.4 s | not run |
| Ascribe's docs copied 20 times, 2,940 files and 2.9 million words, Vale alone | n/a | 1.7 s | 10.2 s |
| One of those pages, Vale alone | n/a | n/a | 0.11 s |

What follows from it:

- **Vale is the slow part of a full check,** and only with large style packages: about 10 seconds for 2.9 million words, against half a second for everything else Ascribe does to 3,000 pages.
- **One page is a tenth of a second,** even with the packages. The editor checks the file you saved, and the hook the files an agent wrote. A cache saves them nothing anyone would notice.
- **So a Vale cache is for full runs:** CI, and `ascribe check --vale` before a push. It turns 10 seconds into the time to lint what changed.

## Vale results

**An entry is one file's alerts,** keyed by everything that decides them:

- the hash of the prose Ascribe extracted for the file, not the file: an edit to frontmatter or to a directive's attributes leaves the prose, and its entry, as it was;
- the hash of the configuration Vale runs with: the `.vale.ini` Ascribe writes or the project's own, the vocabulary, and every file in the styles folder;
- Vale's version.

**The alerts are kept against the extracted prose,** by its line and column, and mapped back to the source through the map each run makes anyway. A page that moved a paragraph within unchanged prose gets its alerts at the paragraph's new place.

**It never expires,** since everything that decides the answer is in the key, and needs no history. A run deletes entries no file asked for in a month, so the folder holds what the project's pages have been recently.

**It's in `.ascribe/cache/vale/`,** one file an entry, written as the link cache's are, since the editor, the CLI, and the hook all run Vale and can write at once.

**Hashing the styles folder** is the one cost that's new: a project's `.vale/` can hold hundreds of small files. It's read once a run, and its hash is kept with the files' sizes and times, so an unchanged folder isn't read again.

**Is it worth building?** Not for a project like Ascribe's, or for any project on the `quiet` preset, where a full run's Vale costs a second. It is for a large project that uses style packages and runs `--vale` in CI, where it's most of the check's time. That project doesn't exist yet. This proposal recommends building it when one does, or when Ascribe's own CI runs Vale over a corpus that makes it matter, and not before.

## The delivery spec's results

The brainstorm listed them as a cache. They're not a fit for one. `afdocs` checks a published site, and a published site changes without a commit: a host's headers, a redirect, a CDN's cache. A page's hash can't say whether last week's answer still holds, so a reused answer could be wrong, which the cache's one rule forbids.

What a team does want from an earlier run is "this check passed last week and fails now", which is the next section's, with nothing held and nothing reused.

## Differences between runs

`ascribe report links` already says what's new since the last run and how many findings went away. The rest of the report should say the same:

```text
Problems
  14 problems, 2 of them new since 9 October; 5 the last report listed are gone.
    guides/install.md:40: [ASC172] …   new
    …
The site and agents
  23 checks: 21 passing, 2 failing, 1 of them failing since 9 October.
```

The JSON's sections gain `previous_run`, `new`, and `resolved`, as `links` has them, and each finding a `new` flag. The keys are added and none change.

**It needs a finding's identity,** which nothing has today. A finding is the same as last run's when it has the same:

1. check;
2. file;
3. fingerprint: the hash of the source text at the finding's place, with whitespace runs read as one space, or of the address for a link, or of the page's address and the check's id for `afdocs`;
4. and, among findings with the first three alike, its order in the file.

So a finding keeps its identity when lines are added above it, and becomes a new one when the text it's about is edited, which is usually right: the edit is what someone should look at. This is the approach SARIF's partial fingerprints and the baselines of several linters take, and it's the seam of the whole proposal, since the baseline below rests on it too.

**The history** is `.ascribe/history/report.json`: the last run's day, and each section's findings by identity. It's written whole through a rename after each run, as `links.json` is. A first run, or one with `--no-history`, says nothing about what's new.

## A baseline

A team turns on the advice checks and gets 3,400 findings. It wants to fix them over a quarter, and meanwhile to see only the ones it adds.

```console
$ ascribe baseline
Wrote ascribe.baseline: 3,412 findings in 1,906 files.
$ ascribe check
checked 2,061 files: 0 errors, 0 warnings, 2 advice, 3,412 in the baseline
```

- **`ascribe.baseline`,** beside `ascribe.toml`, committed. One line a finding: its check, file, fingerprint, and order, sorted, so two people adding to it rarely touch the same line, and a diff of it reads as a list of findings.
- **A finding in the baseline isn't reported,** and is counted, as an acknowledged one is.
- **Only advice can be in it.** An error makes a document non-conforming, and a warning is something a team has decided matters, so neither is put off. `ascribe baseline` skips both, and says how many.
- **It only shrinks by itself.** A finding that's fixed leaves the baseline the next time `ascribe baseline --prune` runs, and `ascribe check` reports an entry that matches nothing, as advice, so a stale baseline doesn't hide a later finding with the same fingerprint. `ascribe baseline` with no flag adds every current advice finding, which is how it starts and how a team deliberately accepts more.
- **It isn't `@intended`.** An acknowledgement is one finding, kept for good, with a reason, beside its block. A baseline is many, kept for now, with none. A finding a team means to keep is moved from the baseline to an `@intended`; the editor's quick fix on a baselined finding does that.
- **The editor and the agents read it,** so an agent working on a page isn't sent after the backlog. `ascribe check --no-baseline` reports everything.

## Where each lives

| | Path | Committed | If it's deleted |
|---|---|---|---|
| Vale results | `.ascribe/cache/vale/` | No | The next full run lints every file |
| The report's history | `.ascribe/history/report.json` | No | The next report can't say what's new |
| The baseline | `ascribe.baseline` | Yes | Every baselined finding is reported again |

Only the report's code opens the history, as for links, and only the prose checks open the Vale cache. A test holds each, as `tests/all/link_cache.rs` does. `check`, `build`, `diff`, and the language server's answers stay a function of the sources and, for the baseline, of a committed file.

## What this leaves out

- **Making `check` incremental.** It takes half a second at 3,000 pages, and the language server answers in milliseconds.
- **Trends across many runs.** The history is the last run, not a log.
- **A baseline for errors and warnings.**
- **Sharing a cache between machines,** beyond a CI platform's.
- **Caching `afdocs`.** Above.

## What to build first

1. **A finding's identity, and differences between runs for the whole report.** It's the seam the baseline depends on, and it needs no committed file and no new command: an identity that turns out wrong only mislabels a finding as new in one report. Its thread: two reports on Ascribe's docs, with a line added above an existing finding and a new finding added below it, and the second report saying one problem is new.
2. **The baseline,** on the same identity, once the first has shown it holds across real edits.
3. **The Vale cache,** when a project's full Vale run is slow enough to matter.

## Open questions

1. **Is a finding's identity its source text, or something narrower?** The text at the place is simple and survives moves. It also makes a reworded sentence a new finding, so a baseline entry is lost when someone fixes a typo near an old finding. A finding's own message arguments (the phrase, the link's target) are steadier but differ by check. Which?
2. **Does the baseline cover advice only, or warnings too?** A team adopting Ascribe on an existing site may have hundreds of warnings, such as `link-dead` across a migration. Keeping warnings out is simpler and stricter.
3. **Is `ascribe baseline` a command of its own, or `ascribe check --write-baseline`?** The surface conventions favor a flag on the command whose findings it writes, and a command of its own is easier to find.
4. **Does the Vale cache wait for a project that needs it, as recommended, or is it built with the rest,** since the link cache's code makes it small?
