@memo: Generated from crates/ascribe-model/settings.toml by tests/conformance/tests/docs.rs; edit the registry, then run `ASCRIBE_BLESS=1 cargo test -p ascribe-conformance --test docs` to rewrite it.
| Key | Type | Default | Description |
|---|---|---|---|
| `available` | boolean | built-in states: as in the table above; new states: **required** | Whether content in this state counts as available ([SPEC §4.4]({repo}/blob/main/SPEC.md#44-available), §9.3). A new state must say so explicitly. |
| `label` | string | built-in states: as in the table above; new states: the state's name | The display label used in availability annotations, such as "Available: Quill Cloud (GA); self-managed (preview, 3.4+)" ([SPEC §9.4]({repo}/blob/main/SPEC.md#94-outputs)). |
