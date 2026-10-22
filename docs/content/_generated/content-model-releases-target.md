@memo: Generated from crates/ascribe-model/settings.toml by tests/conformance/tests/docs.rs; edit the registry, then run `ASCRIBE_BLESS=1 cargo test -p ascribe-conformance --test docs` to rewrite it.
| Key | Type | Default | Description |
|---|---|---|---|
| `live` | array of strings (versions) | `[]` | The lines a reader can be on, in any order. A build [versioned](../content-model/output.md#16-buildsname) by the target gives one output for each. |
| `planned` | array of strings (versions) | `[]` | Lines that haven't shipped, each newer than every live line. A version in one is **planned**: its availability reads `Self-managed (planned, 3.6)` in every output, and stops being planned when its line moves to `live`, with no other edit. |
| `ends` | table of dates | none | The last day each live line is supported, by line: `"3.4" = 2027-01-31`, a TOML date, unquoted. |
| `archived` | table of tables | none | Lines that have left the window, each older than every live line, by line. Each takes `url`, where the line can still be read, and `ended`, the day its support ended, both optional: `"3.2" = {}` declares a line that's readable nowhere. |
| `label` | string | the target's name | The target's display label, as `[dimensions.<name>] labels` gives a value's, when the target is a value of the implicit `release` dimension: `label = "Ascribe"`. A target a declared dimension has takes its label from that dimension's `labels` instead. |
