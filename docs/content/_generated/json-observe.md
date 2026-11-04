@memo: Generated from the Rust types by crates/ascribe-cli/src/shapes.rs, with schemas/observe.schema.json; change the types, then run `ASCRIBE_BLESS=1 cargo test -p ascribe-cli shapes` to rewrite it.
The fields of [`schemas/observe.schema.json`]({repo}/blob/v{version}/schemas/observe.schema.json):

**`ObserveReport`**: What `ascribe observe --format json` writes: the commands it ran, how the test ended, the observation it wrote, and what changed since the one it replaced. With `--check`, nothing ran: the observation it compared, and whether it's current.

| Field | Type | Required | Nullable | Description |
| --- | --- | --- | --- | --- |
| `schema_version` | `integer` | yes | no | The version of this schema. It changes only when a field is removed or changes meaning. |
| `ascribe_version` | `string` | yes | no | The release of Ascribe that wrote it, as `0.4.5`. |
| `config` | `string` | yes | no | The config it ran, `ascribe.examples.toml`, in the current folder. |
| `check` | `boolean` | yes | no | Whether it was run with `--check`, which runs nothing. |
| `environment` | `object[]` | yes | no | The `[environment]` commands, in the order the config has them, each with the line it kept. Empty with `--check`. |
| `environment[].key` | `string` | yes | no | The entry's key. |
| `environment[].command` | `string` | yes | no | The line of shell. |
| `environment[].kept` | `string` | yes | no | The first line it printed, as the observation records it. |
| `test` | `object` | yes | yes | The test command, and how it ended. Absent with `--check`. |
| `test.command` | `string` | yes | no | The line of shell. |
| `test.passed` | `boolean` | yes | no | Whether it exited with 0. |
| `test.exit_code` | `integer` | yes | yes | Its exit code; absent when a signal stopped it. |
| `observation` | `string` | yes | yes | The observation it wrote, from the current folder: none when the test failed. With `--check`, the one it compared; with `--dry-run`, the one it would write. |
| `written` | `boolean` | yes | no | Whether it wrote the observation: false with `--check` or `--dry-run`, or when the test failed. |
| `current` | `boolean` | yes | yes | With `--check`, whether the observation is current: it lists every example in the folder, with the hash it has now, and nothing else. Absent without `--check`. |
| `changes` | `object` | yes | yes | What changed since the observation it replaced: absent when there was none, or the test failed. With `--check`, what differs between the observation and the folder. |
| `changes.added` | `string[]` | yes | no | Examples it lists that the earlier one didn't, by path. |
| `changes.removed` | `string[]` | yes | no | Examples the earlier one listed that it doesn't. |
| `changes.changed` | `string[]` | yes | no | Examples both list, with different hashes. |
| `changes.environment` | `object[]` | yes | no | Environment values that differ, in the newer one's order, then those only the earlier one had. |
| `changes.environment[].key` | `string` | yes | no | The entry's key: `os`, `arch`, or the config's. |
| `changes.environment[].was` | `string` | yes | yes | What the earlier observation had; absent when it had none. |
| `changes.environment[].now` | `string` | yes | yes | What the newer one has; absent when it has none. |
| `recorded` | `{ [name]: string }` | yes | no | The environment the observation records, by name: `os`, `arch`, and each entry's kept line. Empty when the test failed. |
| `files` | `{ [name]: string }` | yes | no | The hash of each example, by path from the current folder. Empty when the test failed. |
| `tested` | `string` | yes | yes | What a page says it was tested with: `Node 24.9.0 on Linux`. Absent when the test failed. |
