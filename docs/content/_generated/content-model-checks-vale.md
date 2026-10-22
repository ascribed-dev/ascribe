@memo: Generated from crates/ascribe-model/settings.toml by tests/conformance/tests/docs.rs; edit the registry, then run `ASCRIBE_BLESS=1 cargo test -p ascribe-conformance --test docs` to rewrite it.
| Key | Type | Default | Description |
|---|---|---|---|
| `preset` | string | | Ascribe's own selection of Vale rules: `quiet`, rules that are almost never wrong. Nothing is downloaded: the preset is written under `.ascribe/vale/`. |
| `config` | string | | Instead of a preset, the path of the project's own `.vale.ini`, from the project root. |
| `off` | array of strings | `[]` | Rules of the preset to turn off, by the names alerts give them, such as `Ascribe.Repeated`. Only with `preset`. |
| `command` | string | `"vale"` | The command that runs Vale. A path with a separator is from the project root; a bare name is looked up as the shell would. A path is a program in the project: the editor asks before it runs one, and the agents' hooks don't run one ([Prose, through Vale](../checks/vale.md#where-vale-runs)). |
| `in-check` | boolean | `false` | Whether `ascribe check`, and the agents' hook, check the prose without `--vale`. It also says the project wants its prose checked everywhere: without it, the editor doesn't report that Vale couldn't be run (`prose-not-checked`), since a machine without Vale isn't one the project asked to check. |
| `max-level` | string | `"error"` | The most an alert can be: `"advice"`, `"warning"`, or `"error"`. Vale's `suggestion`, `warning`, and `error` are advice, a warning, and an error up to it; with `"advice"`, Vale's alerts never fail `--deny-warnings`. |
