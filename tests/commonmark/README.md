# CommonMark spec suite

The official [CommonMark spec](https://spec.commonmark.org/0.31.2/) examples, run against a markdown renderer. A Tessera document is a CommonMark document (SPEC §1.4), so the parser must keep passing these.

- `spec.json`: the 652 examples of CommonMark 0.31.2, the version comrak targets, from <https://spec.commonmark.org/0.31.2/spec.json>.
- `src/lib.rs`: the runner, crate `tessera-commonmark-suite`. It takes any `&str -> String` HTML renderer, compares each example's HTML byte for byte, and compares the result with a recorded baseline.
- `baselines/comrak.toml`: the baseline for unmodified comrak from crates.io (the version in `Cargo.lock`), with CommonMark options only and raw HTML allowed. It records the pass count and the exact failing examples.
- `tests/commonmark.rs`: runs the baseline and prints the pass count.

## Baseline

Unmodified comrak 0.55.0 passes **652 of 652** examples.

## Running

```sh
cargo test -p tessera-commonmark-suite --test commonmark

# Print every failing example with its expected and actual HTML.
COMMONMARK_VERBOSE=1 cargo test -p tessera-commonmark-suite --test commonmark

# Rewrite the baseline after an intentional change, such as a comrak upgrade.
COMMONMARK_WRITE_BASELINE=1 cargo test -p tessera-commonmark-suite --test commonmark
```

The test fails when the result differs from the baseline in either direction, so a change that fixes one example and breaks another is caught, and an improvement has to be recorded.

## For phase 04

Rerun the suite against the `comrak-tessera` fork by adding a test like `tests/commonmark.rs` with the fork's renderer and its own baseline file, once with the Tessera option off (it must match `comrak.toml`) and once with it on. Every example that fails only with the option on must involve a line that is a valid directive line, and needs a justification alongside its baseline.
