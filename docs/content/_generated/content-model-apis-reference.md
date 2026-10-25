@memo: Generated from crates/ascribe-model/settings.toml by tests/conformance/tests/docs.rs; edit the registry, then run `ASCRIBE_BLESS=1 cargo test -p ascribe-conformance --test docs` to rewrite it.
| Key | Type | Default | Description |
|---|---|---|---|
| `renderer` | string | **required** | The renderer that publishes the reference: `"redoc"`. |
| `at` | string | **required** | The address of Redoc's page, relative to `[consumer] base-path`, or absolute when the reference is published elsewhere. |
