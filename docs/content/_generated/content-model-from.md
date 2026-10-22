@memo: Generated from crates/ascribe-model/settings.toml by tests/conformance/tests/docs.rs; edit the registry, then run `ASCRIBE_BLESS=1 cargo test -p ascribe-conformance --test docs` to rewrite it.
| Key | Type | Default | Description |
|---|---|---|---|
| `take` | array of strings | **required** | What's taken from the file, each a table or a key as a dotted path: `releases` takes the whole table, `consumer.site` one key of `[consumer]`. It's read as if written here, and can't be written here too. |
