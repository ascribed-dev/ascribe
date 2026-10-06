# Results

Measured with `cargo bench` in a release build. Reproduce everything below with:

```sh
cargo build --release -p tessera-cli -p tessera-corpora
export ASCRIBE_CORPORA=fetch   # the Elastic part of the benchmark needs the corpus
ASCRIBE_BENCH_OUT=results.jsonl cargo bench -p tessera-corpora --bench perf
ASCRIBE_BENCH_OUT=results.jsonl cargo bench -p tessera-lsp --bench keystroke
ASCRIBE_BENCH_OUT=results.jsonl cargo bench -p tessera-resolve --bench incremental
ASCRIBE_BENCH_OUT=results.jsonl cargo bench -p tessera-lsp --bench completion
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
| incremental: edit one page (`apply`) | 85 µs median; with a snapshot held 3.0 ms |
| incremental: edit a fragment 30 pages include, rename its heading | 108 µs, 195 µs |
| incremental: a model change re-indexes all 3,100 files | 163 ms |

These agree with the figures recorded before the benchmarks shared one
generator, `tessera-synthetic` (page keystroke 3.0 ms both times, fragment 9.0
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
machine's numbers. Medians:

| | time |
|---|---|
| `ascribe check` | 704 ms |
| `ascribe build --emit plain,json`, first; again | 1.9 s; 1.7 s |
| `ascribe diff`, nothing changed | 1.45 s |
| `ascribe diff`, one page; a fragment 100 pages include; a phrase every page uses | 1.40 s; 1.42 s; 1.41 s |
| `ascribe diff` with snippets, nothing changed; a region ten pages show | 1.87 s; 1.86 s |
| `ascribe drift` with snippets, nothing changed; a region ten pages show | 518 ms; 923 ms |
| `ascribe check --format json`, converted Elastic sample | 3.3 s |
| language server: page keystroke, fragment keystroke, completion | 1.7 ms, 8.6 ms, 4.0 ms |

`diff` takes the same time whatever changed, which is
`project-docs/optimization/inventory.md` finding 4; phase 7 of that plan
sets its target. Completion is the slowest context, a link by page title, at
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
| `ascribe diff`, a phrase every page uses changed | 550 MB |
| language server, the project loaded and a page's diagnostics published | 186 MB |
| language server, after 100 edits to that page | 186 MB |

Runs differed by less than 0.2 MB. `diff` holds about twice what `check`
does, since it loads the project at both revisions. The language server
doesn't grow over 100 edits.

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
say so in the commit. A runner isn't always the same CPU (two runs on
2026-10-06 got an EPYC 9V74 and an EPYC 7763, about 15 percent apart), which
the margin absorbs.

## Completion

`crates/tessera-lsp/benches/completion.rs` times every completion context at
20 to 3,000 pages and records the slowest, a link by page title at 3,000
pages, as `lsp/completion-3000` with `tessera_synthetic::report::record`. The
workflow runs it, and `baselines/perf.json` has its 50 ms target and its
runner baseline.
