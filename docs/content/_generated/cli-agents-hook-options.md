<!-- Generated from the help text in crates/ascribe-cli/src/ by crates/ascribe-cli/src/docs.rs. Edit the help text, then run `ASCRIBE_BLESS=1 cargo test -p ascribe-cli docs`. -->

- `HARNESS`: The agent whose hook runs it: how its input is read and its answer written.
  - `claude-code`: Claude Code: `PostToolUse` and `Stop`.
  - `codex`: Codex: `PostToolUse` and `Stop`.
  - `copilot`: GitHub Copilot: `postToolUse` and `agentStop`.
- `--event <EVENT>`: When the hook runs: after the agent writes a file (`edit`), or when it's about to finish (`stop`).
  - `edit` (the default): After the agent writes a file: report the errors in it.
  - `stop`: When the agent is about to finish: check the whole project, and keep it working while there are errors.
