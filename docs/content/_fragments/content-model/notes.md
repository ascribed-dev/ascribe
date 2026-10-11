Note types ([SPEC §4.5]({repo}/blob/main/SPEC.md#45-note)), the values of `@note`'s `type` attribute. Five are built in: `note`, `tip`, `important`, `warning`, and `caution`. A project adds types by declaring them, and MAY relabel a built-in type. Built-in types can't be removed.

@snippet: code:examples/content-models/full.toml#notes

`<type>` is the note type (`key` rule).

| Key | Type | Default | Description |
|---|---|---|---|
| `label` | string | built-in types: `Note`, `Tip`, `Important`, `Warning`, `Caution`; new types: **required** | The display label, used where a note's type is shown as text, such as the plain-markdown output's `**Tip: …**` ([SPEC §9.4]({repo}/blob/main/SPEC.md#94-outputs)). |

Under `[notes]`, the inline form `security = { label = "Security" }` is equivalent.
