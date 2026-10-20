@memo: Generated from crates/ascribe-model/settings.toml by tests/conformance/tests/docs.rs; edit the registry, then run `ASCRIBE_BLESS=1 cargo test -p ascribe-conformance --test docs` to rewrite it.
| Key | Type | Default | Description |
|---|---|---|---|
| `level` | string | the check's own level | The check's level: `"off"`, `"advice"`, `"warning"`, or `"error"`. Without it, the check keeps its own. |
