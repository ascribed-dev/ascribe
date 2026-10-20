@memo: Generated from crates/ascribe-model/settings.toml by tests/conformance/tests/docs.rs; edit the registry, then run `ASCRIBE_BLESS=1 cargo test -p ascribe-conformance --test docs` to rewrite it.
| Key | Type | Default | Description |
|---|---|---|---|
| `label` | string | built-in types: `Note`, `Tip`, `Important`, `Warning`, `Caution`; new types: **required** | The display label, used where a note's type is shown as text, such as the plain-markdown output's `**Tip: …**` ([SPEC §9.4]({repo}/blob/main/SPEC.md#94-outputs)). |
