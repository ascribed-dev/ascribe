# CommonMark spec suite

The official [CommonMark spec](https://spec.commonmark.org/0.31.2/) examples, run against a markdown renderer. An Ascribe document is a CommonMark document (SPEC §1.4), so the parser must keep passing these.

- `spec.json`: the 652 examples of CommonMark 0.31.2, the version comrak targets, from <https://spec.commonmark.org/0.31.2/spec.json>.
- `src/lib.rs`: the runner, crate `ascribe-commonmark-suite`. It takes any `&str -> String` HTML renderer, compares each example's HTML byte for byte, and compares the result with a recorded baseline.
- `baselines/comrak.toml`: the baseline for unmodified comrak from crates.io (the version in `Cargo.lock`), with CommonMark options only and raw HTML allowed. It records the pass count and the exact failing examples.
- `tests/commonmark.rs`: runs the baseline and prints the pass count.
- `tests/fork.rs` and `baselines/comrak-ascribe-{off,on}.toml`: the same examples against the `comrak-ascribe` fork; see [below](#the-comrak-ascribe-fork).

## Baseline

Unmodified comrak 0.55.0 passes **652 of 652** examples.

## Running

```sh
cargo test -p ascribe-commonmark-suite --test commonmark

# Print every failing example with its expected and actual HTML.
COMMONMARK_VERBOSE=1 cargo test -p ascribe-commonmark-suite --test commonmark

# Rewrite the baseline after an intentional change, such as a comrak upgrade.
COMMONMARK_WRITE_BASELINE=1 cargo test -p ascribe-commonmark-suite --test commonmark
```

The test fails when the result differs from the baseline in either direction, so a change that fixes one example and breaks another is caught, and an improvement has to be recorded.

## The comrak-ascribe fork

`tests/fork.rs` runs the suite against the `comrak-ascribe` fork, with the same options as the baseline:

- **Ascribe option off** (`baselines/comrak-ascribe-off.toml`): **652 of 652**. The test also fails unless this result matches `baselines/comrak.toml`, so the fork behaves exactly as unmodified comrak.
- **Ascribe option on** (`baselines/comrak-ascribe-on.toml`), with the built-in keywords (`id`, `include`, `variant`, `available`, `note`, `steps`, `details`), `end`, and one project widget (`quill-demo`): **652 of 652**. No example renders differently with the option on.
- **Each example after a directive line**, option on: **652 of 652**. The test puts `@end` on its own line before every example and checks that the example's HTML is unchanged. This puts an Ascribe line directly above every kind of block the spec covers, and shows it closes cleanly without changing what follows. It has no baseline: every example must pass.

### Exceptions with the Ascribe option on

None. No example in CommonMark 0.31.2 has a line starting with `@`, so the Ascribe line can't change any of them. An example that fails only with the option on must involve a line that is a valid directive line; list it here with its justification if one ever appears (for example, after a spec update).

```sh
cargo test -p ascribe-commonmark-suite --test fork
COMMONMARK_WRITE_BASELINE=1 cargo test -p ascribe-commonmark-suite --test fork
```
