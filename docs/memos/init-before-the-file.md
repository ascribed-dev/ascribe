---
about: getting-started.md#create-ascribetoml
quote: Start the project with the command
---

Getting started has the reader run `ascribe init` before it shows them
`ascribe.toml`, so the file they're reading about is the file they now have,
comments and all, and not one they've copied from the page and got slightly
wrong.

That only holds while the snippet is what the command writes. It's the `init`
region of `examples/getting-started/ascribe.toml`, and
`crates/ascribe-cli/tests/all/init.rs`
(`getting_started_s_ascribe_toml_starts_with_what_init_writes`) fails when the
example stops starting with what `ascribe init` writes. To change what the page
shows, change the command and its template, then the example; don't edit the
example alone.
