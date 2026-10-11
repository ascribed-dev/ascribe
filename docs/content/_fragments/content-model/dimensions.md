A **dimension** is an axis content varies along ([SPEC §4.3]({repo}/blob/main/SPEC.md#43-variant)), and whose values are availability targets ([SPEC §4.4]({repo}/blob/main/SPEC.md#44-available)).

@snippet: code:examples/content-models/full.toml#deployment

`<name>` is the dimension's name (`key` rule, §1.2). It's written as an attribute key in `@variant`, as a key in `variant` frontmatter and build selections, and as a target in availability specs, where it stands for all its values.

| Key | Type | Default | Description |
|---|---|---|---|
| `values` | array of strings | **required** | The dimension's values (`name-word`), in display order. At least one; no duplicates. |
| `labels` | table of strings | `{}` | Display labels for values, keyed by value ([SPEC §9.4]({repo}/blob/main/SPEC.md#94-outputs): "Labels for dimension values come from the content model's display labels"). A value without a label is displayed as the value itself. Every key MUST be a declared value. |
| `label` | string | the dimension's name | The display label for the dimension itself, used where a dimension name appears as an availability target or names a tab group. |
| `versionless` | array of strings | `[]` | Values that are versionless ([SPEC §4.4]({repo}/blob/main/SPEC.md#44-available)): availability for them takes a single state and no versions. Every entry MUST be a declared value. Values not listed are versioned. |
