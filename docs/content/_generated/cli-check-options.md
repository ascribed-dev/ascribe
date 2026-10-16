@memo: Generated from the help text in crates/ascribe-cli/src/ by crates/ascribe-cli/src/docs.rs; edit the help text, then run `ASCRIBE_BLESS=1 cargo test -p ascribe-cli docs` to rewrite it.
- @available: ascribe 0.3.0
  `[PATHS]...`: Files and directories to report on, relative to the current directory. The whole project is still checked, since links, includes, and ids need it, and only the problems that count for these paths are shown: those in a file under one of them, or with a related place in one (a problem a fragment causes on the page that includes it). Such a problem in a fragment is shown once, at its first include. Without `--config`, the project is the nearest `ascribe.toml` at or above the first path, and every path must be in it. By default, the whole project.
- @available: ascribe 0.3.0
  `--stdin`: Check standard input as the text of the file `--path` names. The text is laid over the project on disk, which isn't changed; the file doesn't have to exist. Only the problems that count for it are shown.
- @available: ascribe 0.3.0
  `--path <PATH>`: The file standard input is checked as, relative to the current directory. Only with `--stdin`.
- `--build <NAME>`: Check only this build. Repeat it for several. By default, every build in `ascribe.toml`. An unknown build name is a usage error, and the message lists the builds.
- @available: ascribe 0.3.0
  `--editor-build`: Run the page-level checks of the editor's build only, as the editor does as you type. The editor's build is `[editor] build` in `ascribe.toml`, or the first build. With paths that name files, only those files and the pages that include them are checked, which is quick: for a check after every edit. A full `ascribe check` still covers every build.
- @available: ascribe 0.3.0
  `--code <CODE>`: Show only the problems with this code (`ASC036`) or name (`link-target-missing`), as `ascribe explain` takes it. Repeat it for several. The whole project is still checked, and the rest is left out of the report, as with paths: the counts, the exit code, a prompt, and the fixes `--fix` applies are about what's left. An unknown code is a usage error, and the message names the closest.
- @available: ascribe 0.3.0
  `--summary`: Show how many problems each code and each file has, most first, instead of listing them, then the command that lists the first code's problems.
- `--format <FORMAT>`: How to show the results.
  - `text` (the default): Problems with source snippets, for people.
  - `concise`: One line per problem, `file:line: [code] message`, grouped by file: for an agent. At most 50, then how many more and the command that narrows the check.
  - `json`: One JSON document, for tools.
  - `prompt`: A prompt for an agent that fixes the problems: about the file, when the paths name one file, else about the project or the paths. Nothing when there are no problems.
- `--deny-warnings`: Make warnings fail the command too (exit code 1), for CI.
- @available: ascribe 0.3.0
  `--fix`: Apply the fixes marked safe, then report what's left. A fix is safe when applying it can't change what the page says and leaves nothing to decide, such as linking to a page's file instead of its route. Only the fixes of the diagnostics reported are applied, so with paths, those that count for them. Unsafe fixes are never applied. The report, and the exit code, are the check after the fixes.
- @available: ascribe 0.3.0
  `--vale`: Check the prose with Vale too, as `[checks.vale]` in `ascribe.toml` sets it up. Vale runs once, on the files reported on. Its alerts are `prose` diagnostics, at their place in the source. A Vale that can't be run is one `prose-not-checked` advice, and the rest of the check is the same. `[checks.vale] in-check = true` checks the prose without this option. Without `[checks.vale]`, it's a usage error.
