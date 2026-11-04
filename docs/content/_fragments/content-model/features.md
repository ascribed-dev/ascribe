The features registry ([SPEC §4.4]({repo}/blob/main/SPEC.md#44-available)): named availability specs, so a feature going generally available takes one edit.

@snippet: code:examples/content-models/full.toml#features

`<key>` is the feature key (`name-word`). Writing it as the whole of an `@available` line after the colon (`@available: <key>`), or as the whole `available` frontmatter value, stands for the availability spec.

@include: ../../_generated/content-model-features.md
