<!-- Generated from the help text in crates/ascribe-cli/src/ by crates/ascribe-cli/src/docs.rs. Edit the help text, then run `ASCRIBE_BLESS=1 cargo test -p ascribe-cli docs`. -->

- @available: next
  `[PATHS]...`: Files and directories to report on, relative to the current directory. The whole project is still checked, since links, includes, and ids need it, and only the diagnostics that count for these paths are shown: those in a file under one of them, or with a related place in one (a problem a fragment causes on the page that includes it). Such a problem in a fragment is shown once, at its first include. Without `--config`, the project is the nearest `ascribe.toml` at or above the first path, and every path must be in it. By default, the whole project.
- @available: next
  `--stdin`: Check standard input as the text of the file `--path` names. The text is laid over the project on disk, which isn't changed; the file doesn't have to exist. Only the diagnostics that count for it are shown.
- @available: next
  `--path <PATH>`: The file standard input is checked as, relative to the current directory. Only with `--stdin`.
- `--build <NAME>`: Check only this build. Repeat it for several. By default, every build in `ascribe.toml`. An unknown build name is a usage error, and the message lists the builds.
- @available: next
  `--editor-build`: Run the page-level checks of the editor's build only, as the editor does as you type. The editor's build is `[editor] build` in `ascribe.toml`, or the first build. With paths that name files, only those files and the pages that include them are checked, which is quick: for a check after every edit. A full `ascribe check` still covers every build.
- @available: next
  `--summary`: Show how many diagnostics each code and each file has, most first, instead of listing them.
- `--format <FORMAT>`: How to show the results.
  - `text` (the default): Diagnostics with source snippets, for people.
  - `concise`: One line per diagnostic, `file:line: [code] message`, grouped by file: for an agent. At most 50, then how many more and the command that narrows the check.
  - `json`: One JSON document, for tools.
  - `prompt`: A prompt for an agent that fixes the problems: about the file, when the paths name one file, else about the project or the paths. Nothing when there are no problems.
- `--deny-warnings`: Make warnings fail the command too (exit code 1), for CI.
