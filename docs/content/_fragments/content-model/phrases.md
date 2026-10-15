The phrases registry ([SPEC §5.1]({repo}/blob/main/SPEC.md#51-phrases)).

@snippet: code:examples/content-models/full.toml#phrases

Every key is a phrase key (`key` rule, the same rule as SPEC Appendix A's `phrase`), and every value MUST be a TOML string (`model-phrase-value-type`). `version = 3.4` is a float and an error; that's the kind of silent type change TOML exists to prevent ([SPEC §7.1]({repo}/blob/main/SPEC.md#71-role)). Values are inserted as literal text ([SPEC §5.1]({repo}/blob/main/SPEC.md#51-phrases)).

**A target's line.** A value MAY instead be a table with one key, `line`, naming a target whose lines `[releases.<target>]` (§7.1) lists: `release = { line = "self-managed" }`. The phrase is then the target's current line, its newest live one, as the list writes it, so it changes when a line moves to `live` and has no second home to keep. The target MUST have a live line (`model-phrase-line`); any other key is an error (`model-unknown-key`).

**Phrases in frontmatter.** [SPEC §5.1]({repo}/blob/main/SPEC.md#51-phrases) lets the content model decide which frontmatter fields accept phrases. A field accepts them when its table form sets `phrases = true` (§5.2):

```toml
[types.guide.frontmatter]
title = { type = "string", phrases = true }
```

Only `string` and `list(string)` fields can accept phrases (`model-phrases-field-type`). By default, no field does. The setting lives on the field, not in `[phrases]`, so that a phrase key can never collide with a setting's name.
