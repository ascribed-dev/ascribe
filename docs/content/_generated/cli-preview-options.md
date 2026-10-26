@memo: Generated from the help text in crates/ascribe-cli/src/ by crates/ascribe-cli/src/docs.rs; edit the help text, then run `ASCRIBE_BLESS=1 cargo test -p ascribe-cli docs` to rewrite it.
- `--build <NAME>`: The build to preview. Needed when `ascribe.toml` has more than one build. Of a versioned build, its current line.
- `--port <PORT>`: The port to serve the preview on, at 127.0.0.1. By default 4880, or a free port the system picks when 4880 is taken.
