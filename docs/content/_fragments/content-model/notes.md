Note types ([SPEC §4.5]({repo}/blob/main/SPEC.md#45-note)), the values of `@note`'s `type` attribute. Five are built in: `note`, `tip`, `important`, `warning`, and `caution`. A project adds types by declaring them, and can relabel a built-in type. Built-in types can't be removed.

@snippet: code:examples/content-models/full.toml#notes

`<type>` is the note type (`key` rule).

@include: ../../_generated/content-model-notes.md

Under `[notes]`, the inline form `security = { label = "Security" }` is equivalent.
