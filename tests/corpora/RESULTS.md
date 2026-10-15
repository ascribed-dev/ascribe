# Results

Measured with `cargo bench` in a release build. Reproduce everything below with:

```sh
cargo build --release -p ascribe-cli -p ascribe-corpora
export ASCRIBE_CORPORA=fetch   # the Elastic part of the benchmark needs the corpus
ASCRIBE_BENCH_OUT=results.jsonl cargo bench -p ascribe-corpora --bench perf
ASCRIBE_BENCH_OUT=results.jsonl cargo bench -p ascribe-lsp --bench keystroke
ASCRIBE_BENCH_OUT=results.jsonl cargo bench -p ascribe-resolve --bench incremental
ASCRIBE_BENCH_OUT=results.jsonl cargo bench -p ascribe-lsp --bench completion
./target/release/corpora compare results.jsonl
```

**Machine:** the sections from [Targets](#targets) to [The numbers](#the-numbers)
were measured on a cloud container, Intel Xeon @ 2.10 GHz, 4 logical cores,
15 GB, Linux 6.18, system calls noticeably slower than on bare metal, on
2026-09-29 at the commits pinned in `src/corpus.rs`. The baselines the
regression check uses were recorded later on GitHub's runner; see
[Baselines on the runner](#baselines-on-the-runner).

## Targets

| Target | Result | Verdict |
|---|---|---|
| `ascribe check` of a 3,000-page project: a few seconds | synthetic: **0.78 s** median (0.71 to 1.29 s) | met |
| the same on the converted Elastic sample (3,008 pages, 21 MB) | **7.4 s** median (6.9 to 11.1 s), `--format json` | missed on this container when recorded; fixed (below) |
| language-server diagnostics after a keystroke: about 50 ms | page **3.0 ms** median (p95 3.9); fragment included by 30 pages **9.0 ms** (p95 11.2), at 3,000 pages | met, by 5x to 17x |
| language-server completion: about 50 ms | **4.0 ms** median on the runner (a link by page title, the slowest context; `lsp/completion-3000`). On an Apple-silicon laptop, 2026-09-30: 2.6 ms at most (p95) | met |

### Elastic: where the time went

The synthetic project's pages average half a kilobyte; the Elastic sample's
average 7 KB (14 times), so the work isn't comparable page for page. Where the
7.4 s went when this was recorded (in process, one run): loading 34 ms;
file-level checks 1.2 s; file plus page-level checks for the one build 8.7 s
(the page-level checks re-index and re-resolve); and, separately, 2.5 to 4.4 s
of system time in the command itself, from 23,000 directory listings and 72,000
small writes (`FINDINGS.md` P2). The text report, the default output, took over
100 s on this sample (P1), which was a far bigger problem than the miss.

P1, P2, and P4 in `FINDINGS.md` are fixed. On an Apple-silicon laptop (2026-09-30),
`ascribe check` of the converted Elastic sample takes **2.0 s** in either format
(it was 3.4 to 3.8 s there before the fixes), which meets the target. What's
left is P3's page-level work (1.75 s of user time), recorded and not changed.

## The numbers

Synthetic project (3,000 pages, 100 fragments, 60 images), median of the runs:

| | time |
|---|---|
| `ascribe check` (text, no diagnostics) | 782 ms |
| `ascribe check --format json` | 781 ms |
| `ascribe build --emit plain,json`, first | 2.7 s |
| the same again, nothing changed | 3.6 s (2.3 to 4.0) |
| in process: load 19 ms; file-level 187 ms; all builds 679 ms | |
| in process: index 194 ms; resolve `site` 144 ms; emit plain 56 ms | |
| language server: load and first diagnostics | 755 ms |
| language server: page keystroke, fragment keystroke | 3.0 ms, 9.0 ms |
| language server: a page written on disk, to its diagnostics (2026-10-09) | 3.3 ms |
| incremental: edit one page (`apply`) | 85 µs median; with a snapshot held 3.0 ms |
| incremental: edit a fragment 30 pages include, rename its heading | 108 µs, 195 µs |
| incremental: a model change re-indexes all 3,100 files | 163 ms |

These agree with the figures recorded before the benchmarks shared one
generator, `ascribe-synthetic` (page keystroke 3.0 ms both times, fragment 9.0
ms both times), which is the check that the shared generator makes the same
project.

Noisy project (1,000 pages, a warning on each): JSON 255 ms, text 1.4 s.

Converted samples (`ascribe check --format json`, plus what they contain):

| Corpus | Pages | Errors | Warnings | Notes |
|---|---|---|---|---|
| Elastic | 3,008 | 1,203 | 1,692 | 7.4 s. Errors are the findings F1 to F5 and converter limits |
| Docker | 1,116 | 436 | 3,155 | 2.5 to 3.2 s |
| Astro | 422 | 13 | 2,343 | 1.2 s |

Recognition over the unconverted text: 18 s for the three corpora together in a
release build, 4,546 pages, 32 MB.

## Baselines on the runner

`baselines/perf.json` was recorded on 2026-10-06 by the Corpora workflow run by
hand with **record** set, on GitHub's `ubuntu-latest` runner (AMD EPYC 7763, 4
logical cores). Every metric was recorded again there, so the file holds one
machine's numbers. It was recorded again the same day, on the same CPU, once
the release profile was in (see [The release profile](#the-release-profile)),
which made each command 5 to 12 percent faster, and again once `diff` and
`drift` were made faster (optimization phase 7A, [below](#where-diff-and-drift-spend-their-time)).
Medians:

| | time |
|---|---|
| `ascribe check` | 667 ms |
| `ascribe build --emit plain,json`, first; again | 1.8 s; 1.6 s |
| `ascribe diff`, nothing changed | 843 ms |
| `ascribe diff`, one page; a fragment 100 pages include; a phrase every page uses | 792 ms; 814 ms; 1.02 s |
| `ascribe diff` with snippets, nothing changed; a region ten pages show | 1.07 s; 1.07 s |
| `ascribe drift` with snippets, nothing changed; a region ten pages show | 485 ms; 535 ms |
| `ascribe check --format json`, converted Elastic sample | 3.2 s |
| language server: page keystroke, fragment keystroke, completion | 1.6 ms, 8.0 ms, 3.2 ms |

`diff` took the same time whatever changed (1.28 to 1.32 s), which was
`project-docs/optimization/inventory.md` finding 4; now only a change that
reaches every page, such as a phrase, costs more. Completion is the slowest context, a link by page title, at
3,000 pages (`lsp/completion-3000`); the benchmark didn't record it before.

### Peak memory

The most resident memory the process held at once, from the operating
system (`getrusage`'s `ru_maxrss`, through `/usr/bin/time`), on the 3,000-page
synthetic project, median of three runs:

| | peak memory |
|---|---|
| `ascribe check` | 221 MB |
| `ascribe build --emit plain,json`, first | 265 MB |
| `ascribe diff`, nothing changed | 547 MB |
| `ascribe diff`, a phrase every page uses changed | 549 MB |
| language server, the project loaded and a page's diagnostics published | 185 MB |
| language server, after 100 edits to that page | 185 MB |

Runs differed by less than 0.2 MB. `diff` holds about twice what `check`
does, since it loads the project at both revisions. The language server's
peak memory doesn't rise above its load peak over 100 edits. That's a
high-water mark, not the memory it holds after them, so it wouldn't show a
small leak per edit.

The memory metrics are named `memory/…` and recorded in megabytes, in the same
JSON line fields as a time. The comparison is the same too: a metric fails
over its baseline x 3 + 25 (megabytes here). Memory varies far less than time
between runs, so that catches only a large change, such as holding a third
copy of the project; [decision 9](../../project-docs/optimization/README.md#proposed-decisions)
asks for no tighter target until a change needs one.

## The regression check

`.github/workflows/corpora.yml` (weekly, and by hand) runs the four benchmarks
with `ASCRIBE_BENCH_OUT` set and then `corpora compare`, which
reads `baselines/perf.json`. A metric fails when its median is over

> **recorded median x 3 + 25 ms**

or over an absolute target (the spec's: 5 s for the 3,000-page check, 50 ms for
the language server). Why so wide:

- **Noise on one machine.** Repeated runs here differed by 1.6 to 1.8 times
  (synthetic check 0.71 to 1.29 s; Elastic 6.9 to 11.1 s; unchanged build 2.3
  to 4.0 s), before any change to the code.
- **A different machine.** The baseline was recorded on this container; a shared
  runner may be slower or faster by a similar factor.
- **What the job is for.** The incremental and language-server results came from
  removing work that grew with the project; a change that undoes one costs 10
  times or more, not 30 percent. A 3x margin catches that and not the scheduler.
- **The floor.** 25 ms keeps the microsecond metrics (an incremental edit takes
  85 µs) from failing on a single slow run. It's small against the targets.

A metric with no baseline is listed, not failed; a required one missing from
the results (the ones in `src/perf.rs`) fails, so a benchmark that stops
running is noticed. Re-record after an intended change on the machine the job
runs on, never a laptop: run the Corpora workflow by hand from the branch with
**record** set, which runs `corpora compare results.jsonl --record`, prints the
new file, and uploads it as the `perf-baseline` artifact. Commit that file and
say so in the commit. A recording that lacks a required metric fails and
leaves the file as it was. A runner isn't always the same CPU (two runs on
2026-10-06 got an EPYC 9V74 and an EPYC 7763, about 15 percent apart), which
the margin absorbs.

## Where `diff` and `drift` spend their time

Measured on 2026-10-06 (optimization phase 7A), before changing anything, on
the cloud container described above, in a release build of `main` at
de59172: the commands' wall time (median of five), and each step timed in
process by a scratch program calling the same library functions. The
projects are the benchmark's: the 3,000-page synthetic project in git, with
and without a snippet on every page.

`ascribe diff`, nothing changed, 1.37 s:

| Step | time |
|---|--:|
| loading the working tree (`ascribe_check::Project::load`) | 40 ms |
| finding the repository and the base (three `git` calls) | 6 ms |
| reading the base from `git`: `ls-tree`, then one `cat-file --batch` for 3,261 blobs | 200 ms |
| indexing the base | 150 ms |
| indexing the working tree | 210 ms |
| counting the working tree's errors, a full check (beside the next step) | 600 ms |
| comparing: resolving and fingerprinting all 3,000 pages, on both sides | 475 ms |

The first five steps ran one after another, then the last two side by side,
so the command took about 600 ms plus the slower of the two. It took the
same whatever changed, since the comparison resolved every page, changed or
not. `git` itself was about 140 ms of it (`ls-tree` 5 ms, `cat-file` 120 ms
on the benchmark's loose objects), so the time isn't mostly in `git`.

**With snippets** (1.80 s with a region ten pages show changed): indexing
the base took 363 ms instead of 150, the full check 840 ms instead of 600,
and the comparison 530 ms. The base's indexing read each of the 100 code
files through its own `git cat-file` process, 102 processes in all; 100
such processes take 230 ms alone. Reading a code file once per page that
shows it is ruled out: the working tree's index reads each code file once
(`CodeFiles`), and the base's kept each blob it had read.

**`ascribe drift`**, with snippets: 0.38 s with nothing changed, which is
the working tree's index (295 ms) and one `git diff` (10 ms). With a region
ten pages show changed, 0.95 s: the same, then reading the base (215 ms) and
indexing it (350 ms, with the 100 `git` processes), one after the other.
Telling which of the ten pages changed apart from their examples took 9 ms.

What changed, in pull request order:

1. **A short path for what didn't change.** A page can differ only when its
   own file, a file it includes, or a file it links to reads differently:
   the text differs, the same text resolves differently (a link's target
   appeared, an image went away), or a snippet's code differs. The
   comparison finds those files from the two versions' indexes, adds the
   files that include them and the files that link to those, and resolves
   only those pages; any other page resolves the same on both sides. A
   change to the content model, or to a page the glossary links to, still
   compares every page. Nothing is taken from `git`'s listing.
   `crates/ascribe-diff/tests/all/reach.rs` checks, on random pairs of
   versions, that the result is exactly what comparing every page gives.
   The comparison went from 475 ms to 13 ms.
2. **Each code file once, through one `git` process,** kept open for the
   reads snippets ask for, instead of one process per file.
3. **The two versions side by side.** `diff` counts the errors beside
   everything else, and reads and indexes the base beside indexing the
   working tree. `drift` reads the base beside the working tree's index
   when `git diff` lists a file in one of the project's sources' folders,
   where an example's code is; otherwise it reads it only when an example
   changed, as before.

The commands' wall time on the same container, median of five:

| | before | after |
|---|--:|--:|
| `ascribe diff`, nothing changed | 1.37 s | 0.71 s |
| `ascribe diff` with snippets, a region ten pages show changed | 1.80 s | 0.88 s |
| `ascribe drift` with snippets, nothing changed | 383 ms | 375 ms |
| `ascribe drift` with snippets, a region ten pages show changed | 951 ms | 504 ms |
| `ascribe check`, for comparison | 614 ms | 606 ms |

`diff` now takes about as long as `check`, and can't take less: its report
counts the errors `ascribe check` finds in the working tree
(`working_tree_errors`), which is a full check. So the plan's target, a
quarter of the time before, isn't met. On the runner (the baselines above,
recorded before and after on the same CPU), with nothing changed `diff` went
from 1.28 s to 843 ms, a third less, 1.26 times `check`'s 667 ms; with one
page changed from 1.30 s to 792 ms; with snippets from 1.76 s to 1.07 s; and
`drift` with a region changed from 857 ms to 535 ms. The optimization plan's
measure is set to at most 1.3 times `check`. The other target, `diff` with
snippets at most twice `diff` without, is met (1.3 times). Peak memory of
`diff` with nothing changed went from 547 MB to 371 MB, since only the pages
a change reaches are resolved.

What `diff` spends over `check` (0.18 s on the runner, about 0.1 s on the
container), timed in process on the container: the full check alone took
590 to 605 ms; inside `diff`, beside the base's reading and indexing, the
working tree's indexing, and the `git` processes, all sharing four cores,
the whole took 640 to 745 ms, so 35 to 110 ms is the threads and `git`
contending for the cores (more on a busier machine); freeing both versions'
indexes and the report when the command ends takes another 50 ms (`check`
frees almost nothing); and the three `git` calls that find the repository
and the base take about 15 ms. Reading the base from `git` and comparing
aren't in it: they finish inside the check's time. The contention can't go
without making `check` itself faster, which this plan leaves alone. The
50 ms could go by not freeing the indexes at exit (`std::mem::forget` in
the CLI once the report is written), a one-line change with nothing else
to gain, so it isn't made here.

The reports are byte for byte the same before and after: `diff` as JSON,
HTML, and text and `drift` as JSON, text, and summary, on each of the
benchmark's six cases, on `docs/` against five bases from 5 to 100 commits
back, and on every example (`scripts/compare/outputs.ts`).

## The release profile

`[profile.release]` in the workspace's `Cargo.toml` sets `lto = "fat"` and
`codegen-units = 1`. Measured on 2026-10-06 (optimization phase 7B) with each
setting given to Cargo on its own and together, by a temporary workflow on the
release build's own runners, building as `.github/workflows/release-build.yml`
does (Zig and glibc 2.28 on Linux, a static C runtime on Windows). Binary size:

| Setting | macOS arm64 | Linux arm64 | Linux x64 | Windows x64 |
|---|--:|--:|--:|--:|
| none (before) | 10.49 MB | 11.91 MB | 12.38 MB | 9.44 MB |
| `lto = "thin"` | 10.47 MB | 11.90 MB | 12.41 MB | 9.68 MB |
| `lto = "fat"` | 7.35 MB | 8.08 MB | 8.88 MB | 8.91 MB |
| `codegen-units = 1` | 7.91 MB | 8.64 MB | 9.41 MB | 8.78 MB |
| `lto = "thin"`, `codegen-units = 1` | 7.74 MB | 8.50 MB | 9.38 MB | 9.11 MB |
| **`lto = "fat"`, `codegen-units = 1` (taken)** | **6.99 MB** | **7.66 MB** | **8.62 MB** | **8.65 MB** |
| `strip = "symbols"` | 7.83 MB | 7.89 MB | 9.16 MB | 9.43 MB |
| fat, one unit, `strip = "symbols"` | 5.98 MB | 6.21 MB | 7.54 MB | 8.65 MB |
| fat, one unit, `opt-level = "s"` | 5.84 MB | 6.48 MB | 6.71 MB | 6.18 MB |

The time of the 3,000-page commands, each binary in turn on one
`ubuntu-latest` runner (an AMD EPYC, faster than the one the baselines were
recorded on), the benchmark's median, best of two rounds:

| | none (before) | fat, one unit (taken) | fat, one unit, `strip = "symbols"` | fat, one unit, `opt-level = "s"` |
|---|--:|--:|--:|--:|
| `ascribe check` | 452 ms | 440 ms (-3%) | 434 ms (-4%) | 467 ms (+3%) |
| `ascribe check` with snippets | 598 ms | 579 ms (-3%) | 569 ms (-5%) | 602 ms (+1%) |
| `ascribe build`, first | 1.26 s | 1.22 s (-3%) | 1.22 s (-3%) | 1.37 s (+8%) |
| `ascribe diff`, nothing changed | 957 ms | 942 ms (-2%) | 889 ms (-7%) | 979 ms (+2%) |
| `ascribe drift` with snippets, a region changed | 620 ms | 593 ms (-4%) | 608 ms (-2%) | 631 ms (+2%) |

Peak memory didn't change (within 1 MB). Thin link-time optimization alone
changed neither size nor time.

A clean release build of `ascribe-cli` on those runners, one run each, went
from 108 s to 170 s on macOS, 69 s to 128 s on Linux arm64, 84 s to 129 s on
Linux x64, and 140 s to 218 s on Windows: about a minute more, and the same on
every job that builds a release binary (the release and canary workflows'
build jobs, the Astro end-to-end jobs, and the Corpora workflow).

Not taken:

- **`strip = "symbols"`** saves another 1.0 MB on macOS, 1.4 MB on Linux arm64,
  and 1.1 MB on Linux x64 (nothing on Windows, whose symbols are in a separate
  file), but a panic's backtrace would lose its function names. The release
  build keeps them on Linux on purpose (`release-build.yml`, pull request #78).
- **`opt-level = "s"`** saves another 1.2 to 2.5 MB, but makes `check` 3 to 4
  percent slower than before and 7 percent slower than the profile taken; the
  plan asked for no slower.
- **`panic = "abort"`** isn't an option: the language server catches a
  handler's panic and keeps running, which needs unwinding.
  `crates/ascribe-lsp/src/server.rs` refuses to compile with it.

## Completion

`crates/ascribe-lsp/benches/completion.rs` times every completion context at
20 to 3,000 pages and records the slowest, a link by page title at 3,000
pages, as `lsp/completion-3000` with `ascribe_synthetic::report::record`. The
workflow runs it, and `baselines/perf.json` has its 50 ms target and its
runner baseline.

## One file

`ascribe check <one file> --format json`, as an agent's hook runs it after an
edit, measured by the `perf` bench (`check-one/*`) on 2026-10-09 on the cloud
container described above (Intel Xeon @ 2.10 GHz, 4 logical cores), where the
whole Elastic check takes 3.8 s against 3.2 s on the runner. Median of five
runs. The fragment is one the bench adds, which the first 100 pages include.

| | synthetic, every build | synthetic, `--editor-build` | Elastic, every build | Elastic, `--editor-build` |
|---|---|---|---|---|
| a clean page | 677 ms | 258 ms | 3.71 s | 1.29 s |
| a page with an error | 667 ms | 258 ms | 3.94 s | 1.31 s |
| a fragment 100 pages include | 670 ms | 279 ms | 3.74 s | 1.40 s |

Without `--editor-build`, naming a file only filters the report: the whole
project is checked, every build, so the time is the whole check's. With
`--editor-build` and files named, only those files' file-level checks run, and
the page-level checks of the editor's build for the pages that are the files or
include them (`ascribe_check::diagnose_editor_build`).

What's left is loading the project and indexing all of it, which the page-level
checks need for links and ids: on Elastic, 70 ms to load and 914 ms to index in
process. So a single-file check with `--editor-build` is **under half a second
on the synthetic project, and not on the Elastic sample** (1.3 s here, likely
about 1.1 s on the runner). Getting it there takes not indexing every file
each run: a cache of the index, or a check server that stays running, which the
agents plan lists for later.
