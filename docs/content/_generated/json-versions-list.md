@memo: Generated from the Rust types by crates/ascribe-cli/src/shapes.rs, with schemas/versions-list.schema.json; change the types, then run `ASCRIBE_BLESS=1 cargo test -p ascribe-cli shapes` to rewrite it.
The fields of [`schemas/versions-list.schema.json`]({repo}/blob/v{version}/schemas/versions-list.schema.json):

**`VersionsListReport`**: Every target's release lines.

| Field | Type | Required | Nullable | Description |
| --- | --- | --- | --- | --- |
| `schema_version` | `integer` | yes | no | The version of this schema. It changes only when a field is removed or changes meaning. |
| `ascribe_version` | `string` | yes | no | The release of Ascribe that wrote it, as `0.4.5`. |
| `targets` | `object[]` | yes | no | Each target with a list in `[releases]`, in the order `ascribe.toml` declares them. |
| `targets[].target` | `string` | yes | no | The target, a versioned dimension value. |
| `targets[].builds` | `string[]` | yes | no | The builds versioned by it, which give each live line an output. |
| `targets[].lines` | `object[]` | yes | no | Its lines, newest first. |
| `targets[].lines[].line` | `string` | yes | no | The line, as the list writes it. |
| `targets[].lines[].status` | `string` | yes | no | `current` (the newest live line), `live`, `planned`, or `archived`. |
| `targets[].lines[].outputs` | `object[]` | yes | no | For a live line, its output in each build versioned by the target. |
| `targets[].lines[].outputs[].build` | `string` | yes | no | The output's build: the build's own name for the current line, `<build>@<line>` for an older one. |
| `targets[].lines[].outputs[].base` | `string` | yes | no | The path its pages are under, ending in `/`. |
| `targets[].lines[].outputs[].pages` | `integer` | yes | no | How many pages it publishes. |
| `targets[].lines[].ended` | `string` | no | no | For an archived line, the day its support ended. |
| `targets[].lines[].ends` | `string` | no | no | For a live line, the last day it's supported, from `ends`. |
| `targets[].lines[].url` | `string` | no | no | For an archived line, where it can still be read; absent when nowhere. |
