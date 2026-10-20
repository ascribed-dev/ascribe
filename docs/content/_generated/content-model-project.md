@memo: Generated from crates/ascribe-model/settings.toml by tests/conformance/tests/docs.rs; edit the registry, then run `ASCRIBE_BLESS=1 cargo test -p ascribe-conformance --test docs` to rewrite it.
| Key | Type | Default | Description |
|---|---|---|---|
| `content-root` | string (path) | `"docs"` | The content root ([SPEC §2.2]({repo}/blob/main/SPEC.md#22-pages-and-fragments)): the directory holding every source file. Paths in links and includes that begin with `/` are relative to it. It must exist and be a directory. |
| `output-dir` | string (path) | `".ascribe/build"` | Where `ascribe build` writes output. Each build and emitter writes under `<output-dir>/<build>/<emitter>/`. It need not exist. It may be outside the project's folder, with `..`, and a build then writes there. A build never writes or removes through a symbolic link on the way to it, or in it ([output-layout contract](../contracts/output-layout.md#2-what-ascribe-owns)). |
