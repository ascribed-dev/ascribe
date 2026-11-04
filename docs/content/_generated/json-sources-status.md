@memo: Generated from the Rust types by crates/ascribe-cli/src/shapes.rs, with schemas/sources-status.schema.json; change the types, then run `ASCRIBE_BLESS=1 cargo test -p ascribe-cli shapes` to rewrite it.
The fields of [`schemas/sources-status.schema.json`]({repo}/blob/v{version}/schemas/sources-status.schema.json):

**`SourcesStatusReport`**: What `ascribe sources status --format json` writes: each source's pin and the state of its copies, read from the files alone.

| Field | Type | Required | Nullable | Description |
| --- | --- | --- | --- | --- |
| `schema_version` | `integer` | yes | no | The version of this schema. It changes only when a field is removed or changes meaning. |
| `ascribe_version` | `string` | yes | no | The release of Ascribe that wrote it, as `0.4.5`. |
| `sources` | `object[]` | yes | no | Each source in another repository, in declaration order. |
| `sources[].name` | `string` | yes | no | The source. |
| `sources[].git` | `string` | yes | no | Its repository. |
| `sources[].branch` | `string` | yes | yes | The branch an update follows; `None` for the repository's default. |
| `sources[].commit` | `string` | yes | yes | Its pin, when it has one to this repository. |
| `sources[].files` | `object[]` | yes | no | Each copy, and each file snippets name with none, by path. |
| `sources[].files[].path` | `string` | yes | no | Its path in the source. |
| `sources[].files[].state` | `CopyState` | yes | no | Its state. |

**`CopyState`**: The state of a copy. One of:

| Value | Description |
| --- | --- |
| `"current"` | It's the file the lock pins, and a snippet uses it, or it's the source's tested examples' config or one of their observations. |
| `"changed"` | It isn't the file the lock pins. |
| `"missing"` | The lock lists it, and it isn't there. |
| `"unlocked"` | It's there, and the lock doesn't list it. |
| `"unused"` | No snippet names it. |
| `"not_copied"` | A snippet names it, and it hasn't been copied. |
| `"not_at_pin"` | A snippet names it, and it isn't in the repository at the pin. |
