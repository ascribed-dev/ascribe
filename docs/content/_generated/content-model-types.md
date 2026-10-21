@memo: Generated from crates/ascribe-model/settings.toml by tests/conformance/tests/docs.rs; edit the registry, then run `ASCRIBE_BLESS=1 cargo test -p ascribe-conformance --test docs` to rewrite it.
| Key | Type | Default | Description |
|---|---|---|---|
| `frontmatter` | table of fields | **required** | The frontmatter schema: one entry per field, keyed by field name (`name-word`). Values are field types (§5). It MUST declare `title` as a required string (below). A type can't declare a field `type`, which is reserved (`model-field-reserved`). |
| `purpose` | string | none | What a page of the type is for, in a few words. The rules `ascribe agents sync` writes say it on the type's line, and `ascribe model` shows it. A type copied from the starter set says its summary there without one (below). |
| `template` | string (path) | none | The page a page of the type starts from: a file from the folder `ascribe.toml` is in, outside the content root (below). |
| `guide` | array of paths | `[]` | What to read before writing a page of the type, in the order to read it, such as a guide and a project's own notes on it: files from the folder `ascribe.toml` is in, outside the content root (below). |
