@memo: Generated from crates/ascribe-model/settings.toml by tests/conformance/tests/docs.rs; edit the registry, then run `ASCRIBE_BLESS=1 cargo test -p ascribe-conformance --test docs` to rewrite it.
| Key | Type | Default | Description |
|---|---|---|---|
| `pages` | array of strings (paths) | `[]` | The glossary pages, each a path from the content root, written as in a link ([SPEC §5.2]({repo}/blob/main/SPEC.md#52-links); a leading `/` is allowed and means the same). Each MUST be a page, not a fragment. A term is linked to the first of its pages that a build publishes. |
| `match` | string | `"first"` | Which occurrences are linked: `"first"`, the first occurrence of each term on each page, or `"every"`, every occurrence. "Page" means the resolved page for a build, after includes and build modes. A term can override it. |
| `case-sensitive` | boolean | `false` | Whether occurrences must match a term's case exactly. When `false`, matching ignores case. A term can override it. |
