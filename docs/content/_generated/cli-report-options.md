<!-- Generated from the help text in crates/ascribe-cli/src/ by crates/ascribe-cli/src/docs.rs. Edit the help text, then run `ASCRIBE_BLESS=1 cargo test -p ascribe-cli docs`. -->

- `[SECTION]...`: The sections to report, in any order. By default, those that need nothing outside the project: `problems`, `inventory`, and `builds`.
  - `problems`: What `ascribe check` finds, counted by check, by file, and by kind of next step; and every acknowledgement, with its reason.
  - `inventory`: Pages by type and by owner; overdue reviews, orphan pages, and unused entries.
  - `builds`: What each build leaves out that another keeps.
  - `links`: External links that fail, redirect, or time out. Needs a link checker, lychee, and the network.
  - `agents`: The delivery spec's checks on a built site. Needs `afdocs`, the network, and `--site`.
- `--format <FORMAT>`: How to show the report.
  - `text` (the default): Each section, for people.
  - `summary`: Markdown, for a CI job's summary or an issue's body.
  - `json`: One JSON document, for tools.
  - `prompt`: A prompt for an agent that acts on one section's findings: only with one section, `problems`, `links`, or `agents`. Nothing when it finds nothing.
- `--build <NAME>`: Report only on this build: its problems, and what it leaves out. Repeat it for several. By default, every build in `ascribe.toml`. An unknown build name is a usage error, and the message lists the builds.
- `--site <URL>`: The address of a built site, for `agents`: the published site, or a preview of it.
- `--limit <N>`: List at most this many items in each list. By default, 20 in text and Markdown and 500 in JSON. What's left out is counted, with the command that lists it.
- `--exit-code[=<LEVEL>]`: Exit with 1 when a section finds something at this severity or above, and with 2 when a section asked for couldn't run: for a scheduled job that opens an issue. `--exit-code` alone is `--exit-code=advice`: any finding fails. Without it, the report exits with 0 whenever it ran.
  - `advice`: Any finding.
  - `warning`: Warnings and errors.
  - `error`: Errors only.
