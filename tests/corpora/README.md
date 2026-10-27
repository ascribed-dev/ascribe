# Corpora and performance

Tests and benchmarks against real documentation and a synthetic project of
3,000 pages. Nothing from the corpora is committed (`LICENSES.md`).

| | |
|---|---|
| `synthetic/` | `ascribe-synthetic`: the 3,000-page project every benchmark shares. No dependencies. |
| `src/corpus.rs` | The three corpora, and fetching them: shallow, sparse, blobless clones at pinned commits into `target/corpora/`. |
| `src/recognize.rs` | What the parser finds in unconverted Markdown, classed. |
| `src/convert/` | Elastic, Astro, and Docker constructs to Ascribe, at volume. |
| `src/perf.rs`, `src/bin/corpora.rs` | `corpora compare`: results against `baselines/perf.json`, the weekly run's or, with `--quick`, the pull request's. |
| `tests/` | `edge.rs` (hand-written reproductions, offline), `synthetic.rs`, `recognition.rs`, `conversion.rs`. |
| `benches/perf.rs` | `ascribe check` and `ascribe build` on the synthetic project and the converted Elastic sample. |
| `FINDINGS.md`, `RESULTS.md`, `LICENSES.md` | What was found, measured, and allowed. |
| `recognition.toml`, `baselines/` | The verdict on each class of recognition; recorded counts and timings. |

## Running

```sh
cargo test --workspace                      # offline and fast: corpus tests skip, with a message, unless cached
ASCRIBE_CORPORA=fetch cargo test --release -p ascribe-corpora -- --include-ignored --nocapture
corpora fetch|recognize <corpus>|convert <corpus>   # cargo run --release -p ascribe-corpora --bin corpora
```

`ASCRIBE_CORPORA` sets what a test does about a corpus: unset (or `skip`) never
touches the network, uses a cached checkout if there is one and otherwise
skips with a `SKIPPED:` line on stderr; `fetch` fetches what isn't cached and
skips if it can't; `require` fetches and fails if it can't (the CI job).
`ASCRIBE_CORPORA_DIR` moves the cache. `ASCRIBE_CORPORA_BLESS=1` rewrites the
recorded recognition and conversion counts after you've reviewed a change.

## The quick run on pull requests

The weekly Corpora workflow times everything and compares it with
`baselines/perf.json` (`RESULTS.md`, "The regression check"). A pull request
that changes `crates/` or `tests/corpora/`, and a push to `main` that does,
also runs a quick check, the `benchmarks (quick)` job in
`.github/workflows/rust.yml`, which fails on a slowdown before it merges. It
runs the benchmarks with `ASCRIBE_BENCH_QUICK` set, which measures only these
metrics, the ones the `quick` section of `baselines/perf.json` names:

| Metric | What |
|---|---|
| `check/synthetic-3000` | `ascribe check` on the 3,000-page project |
| `lsp/keystroke-page-3000` | a keystroke to its page's diagnostics |
| `lsp/keystroke-fragment-3000` | a keystroke in a fragment 30 pages include |
| `memory/check-synthetic-3000` | the peak memory of `ascribe check` |

Then `corpora compare bench-results.jsonl --quick` compares them with the same
recorded medians, failing one over **recorded median x 4 + 50 ms** (megabytes
for memory) or over its target, and prints each failure with the metric, its
baseline, its limit, and what was measured. The margin is wider than the
weekly run's 3x + 25 ms because the quick run measures once on whichever
runner it gets, and builds without the release profile's link-time
optimization, which would take more than ten minutes on its own: that build
measured about 20 percent slower for `ascribe check` and up to twice as slow
for a keystroke, which the margin absorbs. It still fails a change that makes
one of these paths several times slower, which is what it's for. The job
takes about five minutes with its cache warm, and times out at 15.

**Blessing a baseline change.** When a pull request makes one of these
metrics slower on purpose, record new baselines as the weekly run does: run
the Corpora workflow by hand on the pull request's branch with **record** set,
download `perf.json` from the `perf-baseline` artifact, and commit it in the
pull request, saying why in the commit. Both runs compare with that file, so
the quick run passes once it's in. To change which metrics the quick run
measures, or its margin, edit the `quick` section; a metric it names must be
one the quick benchmarks record.

## Bumping a pinned commit

Change `Corpus::commit` in `src/corpus.rs`, run the recognition and conversion
tests with `ASCRIBE_CORPORA_BLESS=1`, read the diff of `baselines/`, and add a
verdict to `recognition.toml` for any new class. A class with no verdict fails.

## Converter limits

The converters are for volume and realism, not fidelity: `FINDINGS.md` lists
what they get wrong (Markdig-only syntax, unclosed source directives, MDX
indentation) so those aren't mistaken for problems in the language.
