Lifecycle states ([SPEC §4.4]({repo}/blob/main/SPEC.md#44-available)). Five are built in:

| State | Counts as available | Default label |
|---|---|---|
| `preview` | yes | `preview` |
| `beta` | yes | `beta` |
| `ga` | yes | `GA` |
| `deprecated` | yes | `deprecated` |
| `removed` | no | `removed` |

A project adds states by declaring them, and MAY change the `available` flag or `label` of a built-in state. Built-in states can't be removed.

@snippet: code:examples/content-models/full.toml#lifecycle

`<state>` is the state's name (`name-word`).

| Key | Type | Default | Description |
|---|---|---|---|
| `available` | boolean | built-in states: as in the table above; new states: **required** | Whether content in this state counts as available ([SPEC §4.4]({repo}/blob/main/SPEC.md#44-available), §9.3). A new state must say so explicitly. |
| `label` | string | built-in states: as in the table above; new states: the state's name | The display label used in availability annotations, such as "Available: Quill Cloud (GA); self-managed (preview, 3.4+)" ([SPEC §9.4]({repo}/blob/main/SPEC.md#94-outputs)). |

**Rules.** `ga` MUST count as available (`model-lifecycle-ga-unavailable`), because content with no state is `ga` ([SPEC §4.4]({repo}/blob/main/SPEC.md#44-available)). Lifecycle states take part in the one-role rule, including the built-in ones: a dimension value named `beta` is an error.
