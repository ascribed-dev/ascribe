---
section: Clean-ups and checks
---

**RUSTSEC-2024-0449, `unsafe-libyaml` unmaintained, is ignored in `.cargo/deny.toml` until `serde_yaml_ng` is replaced by `serde-saphyr` in #1289.** The ignore names #1157 as its reason, and the pull request for #1289 removes it with the crate. _Why:_ the advisory says the crate is unmaintained, not that it has a known vulnerability, and the YAML Ascribe parses is the project's own files; the replacement will likely change parse errors' messages and positions, so it needs its own pull request, naming each changed output with `--accept`. `serde-saphyr` is maintained, safe Rust, and far more used than `noyalib`, which is still 0.0.x. _From:_ Kyle, in [#1157](https://github.com/ascribed-dev/ascribe/issues/1157#issuecomment-6616747024), 3 November 2026.
