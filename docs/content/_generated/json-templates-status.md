@memo: Generated from the Rust types by crates/ascribe-cli/src/shapes.rs, with schemas/templates-status.schema.json; change the types, then run `ASCRIBE_BLESS=1 cargo test -p ascribe-cli shapes` to rewrite it.
The fields of [`schemas/templates-status.schema.json`]({repo}/blob/v{version}/schemas/templates-status.schema.json):

**`TemplatesStatusReport`**: What `ascribe templates status` found: for each template the project took, whether the set and the project have changed it since the copy, and each template of the set it doesn't have.

| Field | Type | Required | Nullable | Description |
| --- | --- | --- | --- | --- |
| `schema_version` | `integer` | yes | no | The version of this schema. It changes only when a field is removed or changes meaning. |
| `ascribe_version` | `string` | yes | no | The release of Ascribe that wrote it, as `0.4.5`. |
| `set` | `object` | yes | no | The set this `ascribe` carries, which the copies are compared with. |
| `set.repository` | `string` | yes | no | Their repository. |
| `set.tag` | `string` | yes | no | Their release's tag. |
| `set.commit` | `string` | yes | no | The tag's commit. |
| `upstream_changed` | `boolean` | yes | no | Whether the set has changed a template the project took since the copy, as `templates.lock` remembers it, or no longer has one. What `--exit-code` exits with 1 for. |
| `templates` | `object[]` | yes | no | Each template the project took, in the set's order, then any the set no longer has, by name. |
| `templates[].name` | `string` | yes | no | Its name, which is its type's. |
| `templates[].record` | `Record` | yes | no | What `templates.lock` remembers of it. |
| `templates[].lock` | `string` | yes | no | The lock its entry is in, or the one looked in when there's no entry, from the folder `ascribe.toml` is in. |
| `templates[].lock_problem` | `string` | yes | yes | Why the lock can't be read, when `record` is `invalid_lock`; otherwise null. |
| `templates[].copied_from` | `string` | yes | yes | Their release the copy was made from, as the lock records it; null without an entry. |
| `templates[].upstream` | `UpstreamChange` | yes | no | Whether the set has changed it since the copy: `changed` when any of its files has. |
| `templates[].project` | `Change` | yes | no | Whether the project has changed its copy: `changed` when any of its files has, or is gone. |
| `templates[].files` | `object[]` | yes | no | Its files: the template, their guide, and Ascribe's notes. |
| `templates[].files[].path` | `string` | yes | no | The file, from the folder `ascribe.toml` is in. |
| `templates[].files[].kind` | `FileKind` | yes | no | Which of the template's files it is. |
| `templates[].files[].upstream` | `UpstreamChange` | yes | no | Whether the set's text has changed since the copy. |
| `templates[].files[].project` | `Change` | yes | no | Whether the project has changed the file since the copy. |
| `templates[].files[].matches_set` | `boolean` | yes | yes | Whether the file is the set's text now; null when it isn't there, or the set doesn't have it. |
| `templates[].files[].sections` | `object[]` | yes | yes | What's different in the set by section, when the set changed it and the project didn't, so the file is what was written; otherwise null. |
| `templates[].files[].sections[].heading` | `string` | yes | yes | The section's heading, as written; null for what comes before the first heading, frontmatter included. |
| `templates[].files[].sections[].change` | `SectionChangeKind` | yes | no | How it's different. |
| `not_taken` | `object[]` | yes | no | Each template of the set the project doesn't have, in the set's order. |
| `not_taken[].name` | `string` | yes | no | Its name. |
| `not_taken[].summary` | `string` | yes | no | Their index's summary of it. |

**`Record`**: What `templates.lock` remembers of a template. One of:

| Value | Description |
| --- | --- |
| `"entry"` | An entry, with a hash of each file as written. |
| `"no_lock"` | There's no `templates.lock` where it was looked for. |
| `"no_entry"` | The lock is there, and has no entry for it. |
| `"invalid_lock"` | The lock is there, and isn't one eject wrote. |

**`UpstreamChange`**: Whether the set has changed a template, or a file of one, since the copy. One of:

| Value | Description |
| --- | --- |
| `"unchanged"` | The set's text is what was written. |
| `"changed"` | The set's text isn't what was written. |
| `"unknown"` | Nothing is remembered of what was written, so it can't be told. |
| `"removed"` | The set no longer has it. |

**`Change`**: Whether the project has changed a template, or a file of one, since the copy. One of:

| Value | Description |
| --- | --- |
| `"unchanged"` | The file is what was written. |
| `"changed"` | The file isn't what was written. |
| `"deleted"` | The file isn't there. |
| `"unknown"` | Nothing is remembered of what was written, so it can't be told. |

**`FileKind`**: Which of a template's files a file is. One of:

| Value | Description |
| --- | --- |
| `"template"` | The template, which a page is copied from. |
| `"guide"` | Their guide. |
| `"notes"` | Ascribe's notes on the type. |

**`SectionChangeKind`**: How a section is different in the set. One of:

| Value | Description |
| --- | --- |
| `"added"` | The set has it, and the copy doesn't. |
| `"removed"` | The copy has it, and the set doesn't. |
| `"changed"` | Both have it, with different text. |
