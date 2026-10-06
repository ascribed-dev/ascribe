# Lints tried

Part of [Optimization](README.md), from [phase 3, part C](phase-3-trails.md#part-c-three-lints-tried). Three lints were turned on in a scratch run, counted per crate, sampled, and decided on. One is kept.

| Lint | Flagged in library code | Flagged in all targets | Decision |
|---|--:|--:|---|
| `clippy::too_many_lines`, threshold 150 | 6 | 9 | Kept |
| `unreachable_pub` | 135 | 274 | Not kept |
| `clippy::indexing_slicing` | 214 | 1,061 | Not kept |

## How it was counted

On `main` at `a647e6d`, with Rust 1.98.1 and clippy 0.1.98:

```sh
cargo clippy --workspace --all-targets --locked --message-format=json -- \
  -W unreachable_pub -W clippy::too_many_lines -W clippy::indexing_slicing
```

"All targets" is that run, which is what CI checks. "Library code" is the same run without `--all-targets`, so it leaves out tests, benchmarks, and `#[cfg(test)]` modules. Each warning is counted once by file and line. `crates/comrak-tessera` doesn't use the workspace lints and isn't counted.

## `clippy::too_many_lines`: kept

Clippy counts a function's lines without blanks or comments. With the threshold at 1, it measured all 3,011 functions in the workspace:

| Longer than | Functions |
|--:|--:|
| 60 | 96 |
| 80 | 56 |
| 100 (clippy's default) | 33 |
| 120 | 22 |
| 150 | 9 |
| 200 | 4 |

The threshold is 150, set in `clippy.toml`. That flags the worst nine and no more, and there's no natural gap lower down to choose instead: below 150 the counts climb steadily. The nine allow the lint by name, with a comment saying they're split only while being changed for another reason:

| Lines | Function |
|--:|---|
| 442 | `apply` in `crates/tessera-resolve/src/incremental/mod.rs` |
| 258 | `compare_nodes` in `tests/conformance/src/outline.rs` |
| 243 | `run` in `crates/tessera-lsp/benches/keystroke.rs` |
| 208 | `preview` in `crates/tessera-lsp/src/preview.rs` |
| 200 | `widget` in `crates/tessera-model/src/sections.rs` |
| 195 | `block` in `crates/tessera-diff/src/tree.rs` |
| 163 | `actions` in `crates/tessera-lsp/src/code_action.rs` |
| 162 | `serve` in `crates/tessera-lsp/src/server.rs` |
| 153 | `replace_staged` in `crates/tessera-emit/src/store.rs` |

The change is 34 added lines and no code moved. It stops new functions past 150 lines, and the inventory's "function sizes weren't measured" is now measured by the lint.

## `unreachable_pub`: not kept

| Crate | Library code | All targets |
|---|--:|--:|
| `tessera-cli` | 67 | 67 |
| `tessera-model` | 42 | 42 |
| `tessera-diff` | 12 | 12 |
| `tessera-check` | 7 | 7 |
| `tessera-lsp` | 7 | 47 |
| `tessera-resolve` | 0 | 37 |
| `tessera-sources` | 0 | 21 |
| `tests/conformance` | 0 | 18 |
| `tessera-emit` | 0 | 11 |
| `tessera-fmt` | 0 | 9 |
| `tessera-syntax` | 0 | 3 |

The lint flags a `pub` item that can't be reached from outside its crate, such as one in a private module. Every hit is already invisible to other crates, so `missing_docs` doesn't ask for its documentation and making it `pub(crate)` changes nothing a caller sees. The sample bore that out: the hits are a binary's modules (`tessera-cli`, where everything is private), helpers in private modules (`tessera-model`'s `loader.rs`, `names.rs`, `sections.rs`, `toml_util.rs`), and the `support` modules shared by test files.

It doesn't reach the problem it was tried for. `tessera-resolve`'s public functions are reachable through its `pub mod`s and re-exports, so it flags none of them in library code. What would help is a count of public items no other crate uses, which rustc can't see. A rough name search finds the size of that:

| Crate | Distinct `pub fn` names | Never named in another crate |
|---|--:|--:|
| `tessera-resolve` | 113 | 26 |
| `tessera-model` | 80 | 18 |
| `tessera-core` | 58 | 4 |
| `tessera-syntax` | 9 | 1 |

That search is by name only, so a method sharing a name with something elsewhere counts as used. It's an estimate, not a list. It's filed as [#130](https://github.com/ascribed-dev/ascribe/issues/130), since narrowing a library's API is phase 4 or 5's kind of work, not a lint's.

If it's wanted anyway, the change is mechanical: with the lint on, `cargo clippy --fix --workspace --all-targets` rewrites 273 lines across 49 files to `pub(crate)`, and the workspace then passes clippy with `-D warnings` as it is. Nothing needed hand editing, and no crate came near the 300-line stop.

## `clippy::indexing_slicing`: not kept

| Crate | Library code | All targets |
|---|--:|--:|
| `tessera-diff` | 44 | 97 |
| `tessera-model` | 40 | 63 |
| `tessera-resolve` | 32 | 145 |
| `tessera-syntax` | 27 | 184 |
| `tessera-core` | 20 | 55 |
| `tessera-lsp` | 16 | 232 |
| `tessera-emit` | 11 | 45 |
| `tests/corpora` | 9 | 17 |
| `tessera-check` | 5 | 66 |
| `tests/conformance` | 5 | 27 |
| `tessera-fmt` | 3 | 6 |
| `tessera-sources` | 2 | 29 |
| `tessera-cli` | 0 | 94 |
| `tests/commonmark` | 0 | 1 |

A random sample of 30 of the 200 library hits in `crates/` was read, and none can panic on input. They fall into a few kinds:

- **Inside a bounds check:** `while i < bytes.len() { … bytes[i] … }` in `tessera-core/src/path.rs`, `tessera-syntax/src/convert.rs`, `tessera-model/src/inline.rs`, and `tessera-diff/src/words.rs`.
- **After a length test in the same expression:** `b.len() >= 2 && b[0] …` in `tessera-lsp/src/uri.rs`, `tessera-model/src/loader.rs`, and `tessera-model/src/types.rs`.
- **Slices that can't overrun:** `chars[i + 1..]` where `i` is a valid index, `theirs[common..]` where `common` is at most the length, and `pair[0]` from `windows(2)`.
- **Indexes from the data they index:** positions from a diff of the same two lists in `tessera-diff/src/align.rs`, and `blocks[end - 1]` where `end` is past the block that found it in `tessera-resolve/src/index/headings.rs`.
- **Two vectors built in step:** `LineIndex`'s `wide[line]` in `tessera-core/src/line_index.rs` follows a `lines.get(line)?`, and `new` pushes to both together.

`git log` holds no fix for an index or slice panic. Where input does decide a position, as in the language server's line and column, the code already uses `get`. Keeping the lint would turn about 200 safe lines into `get` calls with fallback branches that can't run, which hide the invariant rather than state it. If a fuzzing or property test ever finds an index panic, that's the time to try again, with `allow-indexing-slicing-in-tests` set so tests stay quiet.
