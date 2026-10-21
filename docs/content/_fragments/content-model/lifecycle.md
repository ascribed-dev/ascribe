Lifecycle states ([SPEC §4.4]({repo}/blob/main/SPEC.md#44-available)). Five are built in:

| State | Counts as available | Default label |
|---|---|---|
| `preview` | yes | `preview` |
| `beta` | yes | `beta` |
| `ga` | yes | `GA` |
| `deprecated` | yes | `deprecated` |
| `removed` | no | `removed` |

A project adds states by declaring them, and can change the `available` flag or `label` of a built-in state. Built-in states can't be removed.

@snippet: code:examples/content-models/full.toml#lifecycle

`<state>` is the state's name (`name-word`).

@include: ../../_generated/content-model-lifecycle.md
