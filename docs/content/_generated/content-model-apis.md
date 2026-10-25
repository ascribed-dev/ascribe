@memo: Generated from crates/ascribe-model/settings.toml by tests/conformance/tests/docs.rs; edit the registry, then run `ASCRIBE_BLESS=1 cargo test -p ascribe-conformance --test docs` to rewrite it.
| Key | Type | Default | Description |
|---|---|---|---|
| `spec` | string (path) | **required** | The spec's file, YAML or JSON, relative to the project root. It may be outside the project's folder, inside the git repository the project is in, when it's in one. |
| `reference` | table or string | **required** | Where the published reference is: `{ renderer = "redoc", at = "/api/" }` for Redoc's page, or a template of an operation's address, with `{operationId}` and `{tag}`, for another renderer. |
