@memo: Generated from the Rust types by crates/ascribe-cli/src/shapes.rs, with schemas/versions-release.schema.json; change the types, then run `ASCRIBE_BLESS=1 cargo test -p ascribe-cli shapes` to rewrite it.
The fields of [`schemas/versions-release.schema.json`]({repo}/blob/v{version}/schemas/versions-release.schema.json):

**`VersionsReleaseReport`**: What `ascribe versions release --format json` writes: the line released, the files and marks rewritten, the features moved, and what the release changes in the outputs.

| Field | Type | Required | Nullable | Description |
| --- | --- | --- | --- | --- |
| `schema_version` | `integer` | yes | no | The version of this schema. It changes only when a field is removed or changes meaning. |
| `ascribe_version` | `string` | yes | no | The release of Ascribe that wrote it, as `0.4.5`. |
| `written` | `boolean` | yes | no | Whether the files were written: false with `--dry-run`. |
| `target` | `string` | yes | no | The target. |
| `line` | `string` | yes | no | The line released, as given. |
| `was_planned` | `boolean` | yes | no | Whether the line was in `planned`; a line that wasn't is a release nobody planned, such as a patch. |
| `next` | `string` | yes | yes | The line planned after the released one, from `--next`. |
| `next_added` | `boolean` | yes | no | Whether `next` was added to `planned`, from `--next`. |
| `archived` | `string` | yes | yes | The line that was current, moved to `archived` with no address, because no build is versioned by the target. |
| `files` | `object[]` | yes | no | Each source file whose marks are rewritten, from the project root. |
| `files[].file` | `string` | yes | no | The file, from the project root. |
| `files[].marks` | `object[]` | yes | no | Each mark rewritten. |
| `files[].marks[].line` | `integer` | yes | no | Its line, from 1. |
| `files[].marks[].from` | `string` | yes | no | The spec as written. |
| `files[].marks[].to` | `string` | yes | no | The spec as rewritten. |
| `cleared` | `object[]` | yes | no | Each source file with marks for the target that say nothing once `archived` is, removed or rewritten, from the project root. |
| `cleared[].file` | `string` | yes | no | The file, from the project root. |
| `cleared[].marks` | `object[]` | yes | no | Each mark removed or rewritten. |
| `cleared[].marks[].line` | `integer` | yes | no | Its line, from 1, before the release. |
| `cleared[].marks[].from` | `string` | yes | no | Its entry for the target, as written. |
| `cleared[].marks[].to` | `string` | yes | yes | The entry as rewritten; null when the mark is removed. |
| `cleared_features` | `object[]` | yes | no | Each feature whose spec says nothing for the target once `archived` is, rewritten. |
| `cleared_features[].key` | `string` | yes | no | Its key. |
| `cleared_features[].from` | `string` | yes | no | Its spec before. |
| `cleared_features[].to` | `string` | yes | no | Its spec after. |
| `cleared_features[].kept` | `string[]` | yes | no | The other targets its spec names, left as they were. |
| `outputs` | `object[]` | yes | no | What the release changes in the builds' outputs, by build and output. |
| `outputs[].build` | `string` | yes | no | The build: `site`, or an older line's, `site@3.4`. Empty for the file the lines share. |
| `outputs[].output` | `string` | yes | no | The output: `site`, `plain`, or `json`, or `_ascribe` for the file the lines share. |
| `outputs[].added` | `string[]` | yes | no | The files it gains, by their path in the output. |
| `outputs[].removed` | `string[]` | yes | no | The files it loses. |
| `outputs[].changed` | `string[]` | yes | no | The files that change. |
| `from` | `string` | yes | yes | The `ascribe.toml` the target's lines are written in, from the current folder, when the project's `[from]` takes them from another project's: the release is made in that project, and `files` are from its root. `null` when they're written in the project's own. |
