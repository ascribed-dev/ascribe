@memo: Generated from the Rust types by crates/ascribe-cli/src/shapes.rs, with schemas/sources-fetch.schema.json; change the types, then run `ASCRIBE_BLESS=1 cargo test -p ascribe-cli shapes` to rewrite it.
The fields of [`schemas/sources-fetch.schema.json`]({repo}/blob/v{version}/schemas/sources-fetch.schema.json):

**`SourcesFetchReport`**: What `ascribe sources fetch --format json` writes: what was copied and removed for each source, and the files that couldn't be copied.

| Field | Type | Required | Nullable | Description |
| --- | --- | --- | --- | --- |
| `schema_version` | `integer` | yes | no | The version of this schema. It changes only when a field is removed or changes meaning. |
| `ascribe_version` | `string` | yes | no | The release of Ascribe that wrote it, as `0.4.5`. |
| `written` | `boolean` | yes | no | Whether the copies and the lock were written: false with `--dry-run`, which says what it would copy, remove, and write. |
| `sources` | `object[]` | yes | no | Each source looked at, in declaration order. |
| `sources[].name` | `string` | yes | no | The source. |
| `sources[].commit` | `string` | yes | yes | Its pin, when it has one. |
| `sources[].pinned` | `boolean` | yes | no | Whether this run pinned it, to the head of its branch. |
| `sources[].copied` | `string[]` | yes | no | The copies written. |
| `sources[].removed` | `string[]` | yes | no | The copies removed. |
| `sources[].failed` | `object[]` | yes | no | The files snippets name that couldn't be copied. |
| `sources[].failed[].path` | `string` | yes | no | Its path in the source. |
| `sources[].failed[].reason` | `string` | yes | no | Why. |
| `sources[].first_copy` | `boolean` | yes | no | Whether these are the first files copied from the source, which publishes them to everyone who can read the project. |
| `unpinned` | `string[]` | yes | no | The pins removed, of sources ascribe.toml no longer declares in another repository. |
| `lock_written` | `boolean` | yes | no | Whether `ascribe.lock` was written. |
