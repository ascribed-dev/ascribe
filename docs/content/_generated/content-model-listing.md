@memo: Generated from crates/ascribe-model/settings.toml by tests/conformance/tests/docs.rs; edit the registry, then run `ASCRIBE_BLESS=1 cargo test -p ascribe-conformance --test docs` to rewrite it.
| Key | Type | Default | Description |
|---|---|---|---|
| `default` | string | `"listed"` | Every page's listing. |
| `folders` | table of strings | `{}` | A listing for every page under a folder, keyed by the folder's path from the content root, ending in `/`, with its name's case as it is on disk. A folder's pages are every page whose path starts with it, in folders under it too. |
| `types` | table of strings | `{}` | A listing for every page of a content type, keyed by the type's name. |
