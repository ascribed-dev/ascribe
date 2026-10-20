@memo: Generated from crates/ascribe-model/settings.toml by tests/conformance/tests/docs.rs; edit the registry, then run `ASCRIBE_BLESS=1 cargo test -p ascribe-conformance --test docs` to rewrite it.
| Key | Type | Default | Description |
|---|---|---|---|
| `build` | string | the only build, else `site` | The build whose page-level diagnostics the language server reports by default ([SPEC §8.1]({repo}/blob/main/SPEC.md#81-validation-levels) checks pages once per build). It MUST name a declared build. |
