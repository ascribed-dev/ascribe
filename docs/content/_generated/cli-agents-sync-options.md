<!-- Generated from the help text in crates/ascribe-cli/src/ by crates/ascribe-cli/src/docs.rs. Edit the help text, then run `ASCRIBE_BLESS=1 cargo test -p ascribe-cli docs`. -->

- `--check`: Write nothing, and exit with 1 when a file is out of date: for CI.
- `--target <TARGET>`: Also write this target's files. Repeat it for several. `agents-md` and `skills` are always written, and each other target whose files already exist is kept up to date.
  - `agents-md`: AGENTS.md beside ascribe.toml, and a short block in the repository root's.
  - `skills`: The skill, in the repository root's .agents/skills/ascribe/.
  - `claude`: An import of AGENTS.md in CLAUDE.md, creating one beside ascribe.toml, and the skill in .claude/skills/ascribe/; with --with-hook, the hooks in .claude/settings.json and the MCP server in .mcp.json.
  - `claude-rules`: The rules in .claude/rules/, loaded only for the project's pages.
  - `copilot`: The rules in .github/instructions/, loaded only for the project's pages; with --with-hook, the hooks in .github/hooks/ascribe.json; with --cloud, the cloud agent's setup.
  - `codex`: Nothing of its own, since Codex reads AGENTS.md and the skill; with --with-hook, the hooks in .codex/hooks.json.
- `--with-hook`: Also write the hooks that check each edit and the whole project before an agent finishes, for `claude`, `codex`, and `copilot`; and for `claude` the MCP server. Hooks written before are kept up to date without it.
- `--cloud`: Also write what Copilot's cloud agent needs on GitHub: steps in .github/workflows/copilot-setup-steps.yml that install Ascribe, and print the MCP server's JSON for the repository's settings. Implies `--target copilot`. Steps written before are kept up to date without it.
- `--agent`: With --cloud, write the MCP server into a custom agent, .github/agents/ascribe-docs.md, instead of printing its JSON: for a repository whose settings you can't change.
