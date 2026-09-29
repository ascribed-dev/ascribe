# Corpora and performance (phase 26)

Tests and benchmarks against real documentation and a synthetic project of
3,000 pages. Nothing from the corpora is committed (`LICENSES.md`).

| | |
|---|---|
| `synthetic/` | `tessera-synthetic`: the 3,000-page project every benchmark shares (phases 13, 15, 26; phase 16's completion benchmark should too). No dependencies. |
| `src/corpus.rs` | The three corpora, and fetching them: shallow, sparse, blobless clones at pinned commits into `target/corpora/`. |
| `src/recognize.rs` | What the parser finds in unconverted Markdown, classed. |
| `src/convert/` | Elastic, Astro, and Docker constructs to Ascribe, at volume. |
| `src/perf.rs`, `src/bin/corpora.rs` | `corpora compare`: results against `baselines/perf.json`. |
| `tests/` | `edge.rs` (hand-written reproductions, offline), `synthetic.rs`, `recognition.rs`, `conversion.rs`. |
| `benches/perf.rs` | `ascribe check` and `ascribe build` on the synthetic project and the converted Elastic sample. |
| `FINDINGS.md`, `RESULTS.md`, `LICENSES.md` | What was found, measured, and allowed. |
| `recognition.toml`, `baselines/` | The verdict on each class of recognition; recorded counts and timings. |

## Running

```sh
cargo test --workspace                      # offline and fast: corpus tests skip, with a message, unless cached
ASCRIBE_CORPORA=fetch cargo test --release -p tessera-corpora -- --include-ignored --nocapture
corpora fetch|recognize <corpus>|convert <corpus>   # cargo run --release -p tessera-corpora --bin corpora
```

`ASCRIBE_CORPORA` sets what a test does about a corpus: unset (or `skip`) never
touches the network, uses a cached checkout if there is one and otherwise
skips with a `SKIPPED:` line on stderr; `fetch` fetches what isn't cached and
skips if it can't; `require` fetches and fails if it can't (the CI job).
`ASCRIBE_CORPORA_DIR` moves the cache. `ASCRIBE_CORPORA_BLESS=1` rewrites the
recorded recognition and conversion counts after you've reviewed a change.

## Bumping a pinned commit

Change `Corpus::commit` in `src/corpus.rs`, run the recognition and conversion
tests with `ASCRIBE_CORPORA_BLESS=1`, read the diff of `baselines/`, and add a
verdict to `recognition.toml` for any new class. A class with no verdict fails.

## Converter limits

The converters are for volume and realism, not fidelity: `FINDINGS.md` lists
what they get wrong (Markdig-only syntax, unclosed source directives, MDX
indentation) so those aren't mistaken for problems in the language.
