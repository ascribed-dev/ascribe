<!-- Generated from the help text in crates/ascribe-cli/src/ by crates/ascribe-cli/src/docs.rs. Edit the help text, then run `ASCRIBE_BLESS=1 cargo test -p ascribe-cli docs`. -->

- `--check`: Write nothing, and exit with 1 when a file is out of date: for CI.
- `--target <TARGET>`: Also write this target's files. Repeat it for several. `agents-md` and `skills` are always written, and each other target whose files already exist is kept up to date.
  - `agents-md`: AGENTS.md beside ascribe.toml, and a short block in the repository root's.
  - `skills`: The skill, in the repository root's .agents/skills/ascribe/.
  - `claude`: An import of AGENTS.md in CLAUDE.md, creating one beside ascribe.toml, and the skill in .claude/skills/ascribe/.
  - `claude-rules`: The rules in .claude/rules/, loaded only for the project's pages.
  - `copilot`: The rules in .github/instructions/, loaded only for the project's pages.
