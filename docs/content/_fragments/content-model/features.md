The features registry ([SPEC §4.4]({repo}/blob/main/SPEC.md#44-available)): named availability specs, so a feature going generally available takes one edit.

@snippet: code:examples/content-models/full.toml#features

`<key>` is the feature key (`name-word`). Writing it as a whole `@available` primary, or as the whole `available` frontmatter value, stands for the spec.

@include: ../../_generated/content-model-features.md

**Rules.** The spec is checked like one written in a document: it MUST parse (`model-availability-syntax`); every target MUST be a declared dimension value or dimension name, and every state a declared state (`model-availability-unknown-name`); versionless targets and dimension names take no versions (`model-availability-versionless`); histories MUST be in chronological order (`model-availability-history-order`). It MUST NOT be a feature key itself (`model-feature-nested`), so features never refer to each other. Feature keys take part in the one-role rule.
