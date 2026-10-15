# Getting started

The project the [getting started guide](https://ascribed-dev.com/go/getting-started/) builds: its `ascribe.toml` and its first page, which the guide takes from here. `ascribe.toml` is what `ascribe init` writes, then what the guide adds at its end; `crates/ascribe-cli/tests/all/init.rs` checks that it is. The page links to `configure.md`, which doesn't exist yet, so `ascribe check` reports one error, as the guide says it will. `crates/ascribe-cli/tests/all/output.rs` checks that it does.
