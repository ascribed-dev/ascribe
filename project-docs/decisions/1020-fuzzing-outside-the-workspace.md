---
section: Clean-ups and checks
---

**The fuzz targets live in `fuzz/`, a crate outside the workspace, and run on a dated nightly in a scheduled workflow only.** `libfuzzer-sys` is a dependency of `fuzz/` alone, with its own `Cargo.lock`, so it never reaches the workspace's lockfile or the published crates. The nightly is pinned to a date in `fuzz/rust-toolchain.toml` and used only by `.github/workflows/fuzz.yml`, never by a pull request's checks. The workflow opens one issue per distinct crash, keyed by the crash's stack signature, and comments on an open one rather than filing a duplicate each night. _Why:_ the libraries promise not to panic on input, and nothing fed them hostile input; the limits keep the new dependencies out of everything users and pull requests touch. _From:_ Kyle, in [#1020](https://github.com/ascribed-dev/ascribe/issues/1020#issuecomment-6418261625), 26 October 2026.
