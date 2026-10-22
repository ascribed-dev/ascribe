@memo: Generated from crates/ascribe-model/settings.toml by tests/conformance/tests/docs.rs; edit the registry, then run `ASCRIBE_BLESS=1 cargo test -p ascribe-conformance --test docs` to rewrite it.
| Key | Type | Default | Description |
|---|---|---|---|
| `frontmatter` | table of fields | `{}` (no fields) | The fields every page type has, as a type's own `frontmatter` declares them (below), so a field every page has is written once. A type's own fields add to them, and one of the same name replaces the shared field whole, for that type. The implicit `page` type of a project with no types has them too. |
