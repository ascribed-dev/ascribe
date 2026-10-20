@memo: Generated from crates/ascribe-model/settings.toml by tests/conformance/tests/docs.rs; edit the registry, then run `ASCRIBE_BLESS=1 cargo test -p ascribe-conformance --test docs` to rewrite it.
| Key | Type | Default | Description |
|---|---|---|---|
| `name` | string | **required** | The feature's display name, shown on hover and available to emitters. |
| `available` | string (availability spec) | **required** | The feature's availability, in [SPEC §4.4]({repo}/blob/main/SPEC.md#44-available)'s syntax. |
