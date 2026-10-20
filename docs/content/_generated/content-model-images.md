@memo: Generated from crates/ascribe-model/settings.toml by tests/conformance/tests/docs.rs; edit the registry, then run `ASCRIBE_BLESS=1 cargo test -p ascribe-conformance --test docs` to rewrite it.
| Key | Type | Default | Description |
|---|---|---|---|
| `attributes` | table of attribute types | `{}` | Accepted image attribute keys (`key` rule) and their types (§5). With no entries, an image accepts no attributes, and any attribute block after an image is an error ([SPEC §8.2]({repo}/blob/main/SPEC.md#82-diagnostics), "Unknown key"). Declaration order is canonical order ([SPEC §8.3]({repo}/blob/main/SPEC.md#83-canonical-form)). A declared default reaches the output: in the site output, every image carries the defaults of attributes it doesn't write. |
