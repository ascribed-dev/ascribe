@memo: Generated from the Rust types by crates/ascribe-cli/src/shapes.rs, with schemas/agents-sync.schema.json; change the types, then run `ASCRIBE_BLESS=1 cargo test -p ascribe-cli shapes` to rewrite it.
The fields of [`schemas/agents-sync.schema.json`]({repo}/blob/v{version}/schemas/agents-sync.schema.json):

**`AgentsSyncReport`**: What `ascribe agents sync --format json` writes: each file it writes, and what it has to say.

| Field | Type | Required | Nullable | Description |
| --- | --- | --- | --- | --- |
| `schema_version` | `integer` | yes | no | The version of this schema. It changes only when a field is removed or changes meaning. |
| `ascribe_version` | `string` | yes | no | The release of Ascribe that wrote it, as `0.4.5`. |
| `check` | `boolean` | yes | no | Whether it ran with `--check`, which writes nothing. |
| `written` | `boolean` | yes | no | Whether it wrote the files: false with `--check` or `--dry-run`. |
| `files` | `object[]` | yes | no | Each file, in the order it was first planned. |
| `files[].path` | `string` | yes | no | Its path, from the current folder, `/`-separated. |
| `files[].changed` | `boolean` | yes | no | Whether it changed: with `check`, whether it's out of date. |
| `files[].read_by` | `Agent[]` | yes | no | The agents that read it. |
| `notes` | `string[]` | yes | no | The notes for the person running it, which it also writes to standard error. |
| `unread` | `string[]` | yes | no | What an agent doesn't read of what's written, and the target that adds it, a sentence each. |
| `mcp_settings` | `any` | yes | no | With `--cloud` and no `--agent`, the MCP server's JSON to paste into the repository's settings for Copilot's cloud agent; else `null`. |

**`Agent`**: An agent that reads a file `sync` writes, as `sync` names it. One of:

| Value | Description |
| --- | --- |
| `"claude_code"` | Claude Code. |
| `"codex"` | Codex. |
| `"copilot"` | GitHub Copilot, in an editor or on GitHub. |
| `"copilot_cloud"` | Copilot's cloud agent only. |
| `"cursor"` | Cursor. |
| `"gemini_cli"` | Gemini CLI. |
