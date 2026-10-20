@memo: Generated from crates/ascribe-model/settings.toml by tests/conformance/tests/docs.rs; edit the registry, then run `ASCRIBE_BLESS=1 cargo test -p ascribe-conformance --test docs` to rewrite it.
| Key | Type | Default | Description |
|---|---|---|---|
| `spec` | string | **required** | The version of the Ascribe specification this project targets (SPEC §11). It MUST be quoted: `spec = 0.1` is a TOML float and is an error. A processor accepts only the spec versions it implements, compared as exact strings; this reference defines `"0.1"`. |
