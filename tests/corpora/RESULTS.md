# Results

Measured with `cargo bench` in a release build. Reproduce everything below with:

```sh
cargo build --release -p tessera-cli -p tessera-corpora
export ASCRIBE_CORPORA=fetch   # the Elastic part of the benchmark needs the corpus
ASCRIBE_BENCH_OUT=results.jsonl cargo bench -p tessera-corpora --bench perf
ASCRIBE_BENCH_OUT=results.jsonl cargo bench -p tessera-lsp --bench keystroke
ASCRIBE_BENCH_OUT=results.jsonl cargo bench -p tessera-resolve --bench incremental
./target/release/corpora compare results.jsonl
```

**Machine:** a cloud container, Intel Xeon @ 2.10 GHz, 4 logical cores, 15 GB,
Linux 6.18, system calls noticeably slower than on bare metal. A developer
laptop is faster on both counts, so treat these as an upper bound. Recorded
2026-09-29 at the commits pinned in `src/corpus.rs`.

## Targets (PLAN.md, Testing)

| Target | Result | Verdict |
|---|---|---|
| `ascribe check` of a 3,000-page project: a few seconds | synthetic: **0.78 s** median (0.71 to 1.29 s) | met |
| the same on the converted Elastic sample (3,008 pages, 21 MB) | **7.4 s** median (6.9 to 11.1 s), `--format json` | **missed here**; see below |
| language-server diagnostics after a keystroke: about 50 ms | page **3.0 ms** median (p95 3.9); fragment included by 30 pages **9.0 ms** (p95 11.2), at 3,000 pages | met, by 5x to 17x |
| language-server completion: about 50 ms | not measured: phase 16 hasn't merged | hook left (below) |

### The Elastic miss

The synthetic project's pages average half a kilobyte; the Elastic sample's
average 7 KB (14 times), so the work isn't comparable page for page. Where the
7.4 s goes (in process, one run): loading 34 ms; file-level checks 1.2 s;
file plus page-level checks for the one build 8.7 s (the page-level checks
re-index and re-resolve; phase 14's notes name this); and, separately, 2.5 to
4.4 s of system time in the command itself, from 23,000 directory listings and
72,000 small writes (FINDINGS P2). The text report, the default output, takes
over 100 s on this sample (FINDINGS P1), which is a far bigger problem than the
miss. Follow-ups: P1, P2, P3 in `FINDINGS.md`, owned by phases 10, 12, 14. The
miss isn't accepted by anyone; it's recorded.

**Update, 2026-09-30.** P1, P2, and P4 are fixed (`FINDINGS.md`). On an
Apple-silicon laptop, `ascribe check` of the converted Elastic sample takes
**2.0 s** in either format (it was 3.4 to 3.8 s there before the fixes), which
meets the target. What's left is P3's page-level work (1.75 s of user time),
recorded and not changed.

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

These reproduce phase 13's and phase 15's own tables after their generators
moved into `tessera-synthetic` (page keystroke 3.0 ms both times, fragment 9.0
ms both times), which is the check that the shared generator made the same
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

## The regression check

`.github/workflows/corpora.yml` (manual, `workflow_dispatch` only) runs the
three benchmarks with `ASCRIBE_BENCH_OUT` set and then `corpora compare`, which
reads `baselines/perf.json`. A metric fails when its median is over

> **recorded median x 3 + 25 ms**

or over an absolute target (the spec's: 5 s for the 3,000-page check, 50 ms for
the language server). Why so wide:

- **Noise on one machine.** Repeated runs here differed by 1.6 to 1.8 times
  (synthetic check 0.71 to 1.29 s; Elastic 6.9 to 11.1 s; unchanged build 2.3
  to 4.0 s), before any change to the code.
- **A different machine.** The baseline was recorded on this container; a shared
  runner may be slower or faster by a similar factor.
- **What the job is for.** Phases 13 to 15 got their results by removing work
  that grew with the project; a change that undoes one costs 10 times or more, not
  30 percent. A 3x margin catches that and not the scheduler.
- **The floor.** 25 ms keeps the microsecond metrics (an incremental edit takes
  85 µs) from failing on a single slow run. It's small against the targets.

A metric with no baseline is listed, not failed; a required one missing from
the results (the ones in `src/perf.rs`) fails, so a benchmark that stops
running is noticed. Re-record after an intended change with
`corpora compare results.jsonl --record`, on the machine the job runs on, and
say so in the commit.

## Completion (phase 16)

Phase 16 hadn't merged when this was written. The hook: a
`crates/tessera-lsp/benches/completion.rs` that builds `Synthetic::standard()`
(dev-dependency `tessera-synthetic`, already in the crate) and calls
`tessera_synthetic::report::record("lsp/completion-3000", &mut times)`. The
workflow runs it when the file exists, `baselines/perf.json` already has the
50 ms target for that name, and this file needs the row.
