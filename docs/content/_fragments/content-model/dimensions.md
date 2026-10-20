A **dimension** is an axis content varies along ([SPEC §4.3]({repo}/blob/main/SPEC.md#43-variant)), and whose values are availability targets ([SPEC §4.4]({repo}/blob/main/SPEC.md#44-available)).

@snippet: code:examples/content-models/full.toml#deployment

`<name>` is the dimension's name (`key` rule, §1.2). It's written as an attribute key in `@variant`, as a key in `variant` frontmatter and build selections, and as a target in availability specs, where it stands for all its values.

@include: ../../_generated/content-model-dimensions.md
