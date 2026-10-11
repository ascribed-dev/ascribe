# Platforms

The project the [guide to content that varies](https://ascribed-dev.com/docs/guides/variants/) builds: a dimension, `platform`, in `ascribe.toml`, and one page, `docs/install.md`, with a variant group and an availability mark. It has two builds: `site`, which keeps every platform, and `windows`, which keeps only what Windows readers need. The guide takes its examples from here, and `crates/ascribe-cli/tests/all/output.rs` renders the page for both builds.
