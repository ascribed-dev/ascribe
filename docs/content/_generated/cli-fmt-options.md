@memo: Generated from the help text in crates/ascribe-cli/src/ by crates/ascribe-cli/src/docs.rs; edit the help text, then run `ASCRIBE_BLESS=1 cargo test -p ascribe-cli docs` to rewrite it.
- `--check`: Change nothing, and show what would change as a unified diff, which `git apply` applies. Exits with 1 if any file would change. Use it in CI.
- `--dry-run`: Say what it would change, and change nothing.
- `--force`: Change files even when git couldn't put them back as they are: ones with changes not committed, ones git doesn't track, or ones in no repository.
- `[PATHS]...`: Files and directories to format: the `.md` files among them. By default, every `.md` file under the content root. Directories whose names start with `.`, `node_modules`, and directories inside the searched ones that hold an `ascribe.toml` other than the project's own (another project, formatted under its own model) are skipped. So is a file in the content root reached through a symbolic link that leads to a file that isn't a source file of the content root, and it's reported.
- `--format <FORMAT>`: How to show what changed. `json` lists each file with the edits that format it; with `--check`, that's the edits without writing them. `github`, only with `--check`, is for GitHub Actions.
  - `text` (the default): Short text, for people and agents to read.
  - `json`: One JSON document, for tools.
  - `github`: A GitHub Actions workflow command per file that would change, which shows it on the pull request at its first change, with the path from the repository's root: for CI.
