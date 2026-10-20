@memo: Generated from crates/ascribe-model/settings.toml by tests/conformance/tests/docs.rs; edit the registry, then run `ASCRIBE_BLESS=1 cargo test -p ascribe-conformance --test docs` to rewrite it.
| Key | Type | Default | Description |
|---|---|---|---|
| `patterns` | array of patterns | `[]` | Additional fragment patterns ([SPEC §2.2]({repo}/blob/main/SPEC.md#22-pages-and-fragments)). A file is a fragment if any segment of its path begins with `_`, or its path matches one of these patterns. |
| `frontmatter` | table of fields | `{}` (no fields) | The fragment schema ([SPEC §2.2]({repo}/blob/main/SPEC.md#22-pages-and-fragments)): the frontmatter every fragment is validated against. Fields as in §5. `title` is not required. |
