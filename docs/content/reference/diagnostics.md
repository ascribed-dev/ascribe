---
title: Diagnostics
description: Every problem Ascribe reports, with its code and fix.
---

<!-- Generated from tests/conformance/diagnostics.toml by tests/conformance/tests/docs.rs. Edit the registry, then run `ASCRIBE_BLESS=1 cargo test -p tessera-conformance --test docs`. -->

Every problem Ascribe reports, with its code, its name, and how to fix it. `ascribe check`, `ascribe build`, and the editor report the same diagnostics, with the same codes.

- An **error** makes `ascribe check` fail, and stops `ascribe build` from writing anything. A **warning** doesn't, unless you pass `--deny-warnings`.
- A **file-level** diagnostic is about one file on its own. A **page-level** diagnostic is about a page after its includes are expanded and a build's modes are applied, so it can depend on the build; the message names the builds it appears in.
- A diagnostic about `ascribe.toml` (a name that starts with `model-`) stops everything else when it's an error: every other check depends on the content model.
- In the messages below, `{name}` stands for a value filled in from your source.

In the editor, many diagnostics offer a quick fix. See [Editing](../guides/editor.md).

| Code | Name | Severity | Level |
|---|---|---|---|
| [ASC001](#asc001-attribute-unknown-key) | `attribute-unknown-key` | Error | File |
| [ASC002](#asc002-attribute-type-mismatch) | `attribute-type-mismatch` | Error | File |
| [ASC003](#asc003-attribute-bare-key) | `attribute-bare-key` | Error | File |
| [ASC004](#asc004-attribute-unquoted-reserved) | `attribute-unquoted-reserved` | Error | File |
| [ASC005](#asc005-directive-unknown) | `directive-unknown` | Warning | File |
| [ASC006](#asc006-directive-primary) | `directive-primary` | Error | File |
| [ASC007](#asc007-container-unclosed) | `container-unclosed` | Error | File |
| [ASC008](#asc008-container-colon-unexpected) | `container-colon-unexpected` | Error | File |
| [ASC009](#asc009-container-colon-missing) | `container-colon-missing` | Error | File |
| [ASC010](#asc010-container-open-at-arm) | `container-open-at-arm` | Error | File |
| [ASC011](#asc011-end-unmatched) | `end-unmatched` | Error | File |
| [ASC012](#asc012-end-indent-mismatch) | `end-indent-mismatch` | Error | File |
| [ASC013](#asc013-container-nesting-deep) | `container-nesting-deep` | Warning | File |
| [ASC014](#asc014-binding-no-block) | `binding-no-block` | Error | File |
| [ASC015](#asc015-binding-heading) | `binding-heading` | Error | File |
| [ASC016](#asc016-binding-blank-line) | `binding-blank-line` | Warning | File |
| [ASC017](#asc017-binding-not-section-top) | `binding-not-section-top` | Error | File |
| [ASC018](#asc018-title-not-accepted) | `title-not-accepted` | Warning | File |
| [ASC019](#asc019-title-dot-space) | `title-dot-space` | Warning | File |
| [ASC020](#asc020-id-duplicate) | `id-duplicate` | Error | Page |
| [ASC021](#asc021-include-target-missing) | `include-target-missing` | Error | File |
| [ASC022](#asc022-include-id-missing) | `include-id-missing` | Error | Page |
| [ASC023](#asc023-include-cycle) | `include-cycle` | Error | Page |
| [ASC024](#asc024-variant-no-arm-survives) | `variant-no-arm-survives` | Warning | Page |
| [ASC025](#asc025-variant-unknown) | `variant-unknown` | Error | File |
| [ASC026](#asc026-variant-mixed-arms) | `variant-mixed-arms` | Error | File |
| [ASC027](#asc027-variant-arm-kind) | `variant-arm-kind` | Error | File |
| [ASC028](#asc028-variant-no-shared-dimension) | `variant-no-shared-dimension` | Error | File |
| [ASC029](#asc029-available-unknown) | `available-unknown` | Error | File |
| [ASC030](#asc030-available-history-order) | `available-history-order` | Error | File |
| [ASC031](#asc031-available-versionless) | `available-versionless` | Error | File |
| [ASC032](#asc032-available-exceeds-scope) | `available-exceeds-scope` | Error | Page |
| [ASC033](#asc033-steps-not-ordered-list) | `steps-not-ordered-list` | Error | File |
| [ASC034](#asc034-details-title-missing) | `details-title-missing` | Error | File |
| [ASC035](#asc035-widget-schema) | `widget-schema` | Error | File |
| [ASC036](#asc036-link-target-missing) | `link-target-missing` | Error | File |
| [ASC037](#asc037-link-id-missing) | `link-id-missing` | Error | Page |
| [ASC038](#asc038-link-to-fragment) | `link-to-fragment` | Error | File |
| [ASC039](#asc039-link-id-in-fragment) | `link-id-in-fragment` | Error | Page |
| [ASC040](#asc040-link-id-removed) | `link-id-removed` | Error | Page |
| [ASC041](#asc041-link-route) | `link-route` | Warning | File |
| [ASC042](#asc042-image-source-missing) | `image-source-missing` | Error | File |
| [ASC043](#asc043-image-alt-missing) | `image-alt-missing` | Warning | File |
| [ASC044](#asc044-phrase-undeclared) | `phrase-undeclared` | Warning | File |
| [ASC045](#asc045-heading-phrase-without-id) | `heading-phrase-without-id` | Warning | File |
| [ASC046](#asc046-heading-duplicate-without-id) | `heading-duplicate-without-id` | Warning | Page |
| [ASC047](#asc047-frontmatter-unknown-key) | `frontmatter-unknown-key` | Error | File |
| [ASC048](#asc048-frontmatter-missing-field) | `frontmatter-missing-field` | Error | File |
| [ASC049](#asc049-frontmatter-type-mismatch) | `frontmatter-type-mismatch` | Error | File |
| [ASC050](#asc050-frontmatter-reserved-in-fragment) | `frontmatter-reserved-in-fragment` | Error | File |
| [ASC051](#asc051-content-type-unresolved) | `content-type-unresolved` | Error | File |
| [ASC052](#asc052-list-ended-by-directive) | `list-ended-by-directive` | Warning | File |
| [ASC053](#asc053-directive-indented-code) | `directive-indented-code` | Warning | File |
| [ASC054](#asc054-steps-numbering-continued) | `steps-numbering-continued` | Warning | File |
| [ASC055](#asc055-attribute-syntax) | `attribute-syntax` | Error | File |
| [ASC056](#asc056-attribute-duplicate-key) | `attribute-duplicate-key` | Error | File |
| [ASC057](#asc057-available-syntax) | `available-syntax` | Error | File |
| [ASC058](#asc058-id-invalid) | `id-invalid` | Error | File |
| [ASC059](#asc059-image-attribute-missing) | `image-attribute-missing` | Error | File |
| [ASC060](#asc060-model-toml-syntax) | `model-toml-syntax` | Error | File |
| [ASC061](#asc061-model-unknown-key) | `model-unknown-key` | Error | File |
| [ASC062](#asc062-model-missing-key) | `model-missing-key` | Error | File |
| [ASC063](#asc063-model-wrong-type) | `model-wrong-type` | Error | File |
| [ASC064](#asc064-model-invalid-value) | `model-invalid-value` | Error | File |
| [ASC065](#asc065-model-spec-unsupported) | `model-spec-unsupported` | Error | File |
| [ASC066](#asc066-model-invalid-name) | `model-invalid-name` | Error | File |
| [ASC067](#asc067-model-empty-text) | `model-empty-text` | Error | File |
| [ASC068](#asc068-model-path-absolute) | `model-path-absolute` | Error | File |
| [ASC069](#asc069-model-content-root-missing) | `model-content-root-missing` | Error | File |
| [ASC070](#asc070-model-output-overlaps-content) | `model-output-overlaps-content` | Error | File |
| [ASC071](#asc071-model-type-multiple-defaults) | `model-type-multiple-defaults` | Error | File |
| [ASC072](#asc072-model-type-unreachable) | `model-type-unreachable` | Error | File |
| [ASC073](#asc073-model-type-title) | `model-type-title` | Error | File |
| [ASC074](#asc074-model-field-reserved) | `model-field-reserved` | Error | File |
| [ASC075](#asc075-model-type-syntax) | `model-type-syntax` | Error | File |
| [ASC076](#asc076-model-type-fields) | `model-type-fields` | Error | File |
| [ASC077](#asc077-model-enum-values) | `model-enum-values` | Error | File |
| [ASC078](#asc078-model-set-token) | `model-set-token` | Error | File |
| [ASC079](#asc079-model-default-type) | `model-default-type` | Error | File |
| [ASC080](#asc080-model-phrases-field-type) | `model-phrases-field-type` | Error | File |
| [ASC081](#asc081-model-pattern-syntax) | `model-pattern-syntax` | Error | File |
| [ASC082](#asc082-model-name-multiple-roles) | `model-name-multiple-roles` | Error | File |
| [ASC083](#asc083-model-name-case) | `model-name-case` | Warning | File |
| [ASC084](#asc084-model-dimension-empty) | `model-dimension-empty` | Error | File |
| [ASC085](#asc085-model-dimension-value-duplicate) | `model-dimension-value-duplicate` | Error | File |
| [ASC086](#asc086-model-dimension-value-shared) | `model-dimension-value-shared` | Error | File |
| [ASC087](#asc087-model-label-undeclared) | `model-label-undeclared` | Error | File |
| [ASC088](#asc088-model-versionless-undeclared) | `model-versionless-undeclared` | Error | File |
| [ASC089](#asc089-model-lifecycle-available-required) | `model-lifecycle-available-required` | Error | File |
| [ASC090](#asc090-model-lifecycle-ga-unavailable) | `model-lifecycle-ga-unavailable` | Error | File |
| [ASC091](#asc091-model-note-label-required) | `model-note-label-required` | Error | File |
| [ASC092](#asc092-model-availability-syntax) | `model-availability-syntax` | Error | File |
| [ASC093](#asc093-model-availability-unknown-name) | `model-availability-unknown-name` | Error | File |
| [ASC094](#asc094-model-availability-versionless) | `model-availability-versionless` | Error | File |
| [ASC095](#asc095-model-availability-history-order) | `model-availability-history-order` | Error | File |
| [ASC096](#asc096-model-feature-nested) | `model-feature-nested` | Error | File |
| [ASC097](#asc097-model-phrase-value-type) | `model-phrase-value-type` | Error | File |
| [ASC098](#asc098-model-glossary-duplicate-term) | `model-glossary-duplicate-term` | Error | File |
| [ASC099](#asc099-model-glossary-link) | `model-glossary-link` | Error | File |
| [ASC100](#asc100-model-widget-reserved-name) | `model-widget-reserved-name` | Error | File |
| [ASC101](#asc101-model-widget-forms) | `model-widget-forms` | Error | File |
| [ASC102](#asc102-model-widget-binding) | `model-widget-binding` | Error | File |
| [ASC103](#asc103-model-widget-container-primary) | `model-widget-container-primary` | Error | File |
| [ASC104](#asc104-model-widget-groupable-form) | `model-widget-groupable-form` | Error | File |
| [ASC105](#asc105-model-widget-plain-content) | `model-widget-plain-content` | Error | File |
| [ASC106](#asc106-model-consumer-unsupported) | `model-consumer-unsupported` | Error | File |
| [ASC107](#asc107-model-consumer-site) | `model-consumer-site` | Error | File |
| [ASC108](#asc108-model-consumer-base-path) | `model-consumer-base-path` | Error | File |
| [ASC109](#asc109-model-build-name-case) | `model-build-name-case` | Error | File |
| [ASC110](#asc110-model-build-variants) | `model-build-variants` | Error | File |
| [ASC111](#asc111-model-build-unknown-dimension) | `model-build-unknown-dimension` | Error | File |
| [ASC112](#asc112-model-build-unknown-value) | `model-build-unknown-value` | Error | File |
| [ASC113](#asc113-model-build-availability) | `model-build-availability` | Error | File |
| [ASC114](#asc114-model-build-filter-target) | `model-build-filter-target` | Error | File |
| [ASC115](#asc115-model-build-filter-version) | `model-build-filter-version` | Error | File |
| [ASC116](#asc116-model-build-filter-excluded) | `model-build-filter-excluded` | Warning | File |
| [ASC117](#asc117-model-editor-build-unknown) | `model-editor-build-unknown` | Error | File |
| [ASC118](#asc118-model-editor-build-required) | `model-editor-build-required` | Error | File |
| [ASC119](#asc119-model-attribute-reserved) | `model-attribute-reserved` | Error | File |
| [ASC120](#asc120-directive-extra-text) | `directive-extra-text` | Error | File |
| [ASC121](#asc121-link-page-dropped) | `link-page-dropped` | Error | Page |
| [ASC122](#asc122-frontmatter-syntax) | `frontmatter-syntax` | Error | File |
| [ASC123](#asc123-source-unreadable) | `source-unreadable` | Error | File |
| [ASC124](#asc124-heading-empty-slug) | `heading-empty-slug` | Warning | File |
| [ASC125](#asc125-include-heading-without-id) | `include-heading-without-id` | Warning | File |
| [ASC126](#asc126-phrase-double-braces) | `phrase-double-braces` | Warning | File |

## Source files

### Attributes

#### ASC001 `attribute-unknown-key`

Error · file level · [SPEC §3.3]({repo}/blob/main/SPEC.md#33-attributes)

**When:** Unknown key for the directive or image.

**Message:** `@{name}` has no attribute `{key}`; its attributes are: \{keys}

**Fix:** Fix the key's spelling, or remove it. To accept a new image attribute, declare it in `[images.attributes]`; for a project widget, in its `attributes`.

#### ASC002 `attribute-type-mismatch`

Error · file level · [SPEC §3.3]({repo}/blob/main/SPEC.md#33-attributes)

**When:** Value doesn't match the key's declared type.

**Message:** `{key}` must be \{expected}, but it's "\{value}"

**Fix:** Give a value of the key's declared type: one of the enumeration's values, `true` or `false` for a boolean, or a plain number such as `600` for a number.

#### ASC003 `attribute-bare-key`

Error · file level · [SPEC §3.3]({repo}/blob/main/SPEC.md#33-attributes)

**When:** Bare key without a value.

**Message:** `{key}` needs a value, such as `{key}=<value>`; booleans are written out too: `{key}=true`

**Fix:** Give the key a value: `key=value`. Booleans are written out too: `open=true`.

#### ASC004 `attribute-unquoted-reserved`

Error · file level · [SPEC §3.3]({repo}/blob/main/SPEC.md#33-attributes)

**When:** Unquoted value containing a reserved character.

**Message:** the value of `{key}` contains `{character}`, so it must be quoted: `{key}="{value}"`

**Fix:** Quote the value: `label="Using other images"`.

#### ASC055 `attribute-syntax`

Error · file level · [SPEC §3.3]({repo}/blob/main/SPEC.md#33-attributes)

**When:** Attribute block that doesn't parse (such as an unclosed quote or brace, or `=` with no value).

**Message:** this attribute block isn't valid: \{detail}

**Fix:** Fix the attribute block: close every quote and brace, give every `=` a value, and separate pairs with commas.

#### ASC056 `attribute-duplicate-key`

Error · file level · [SPEC §3.3]({repo}/blob/main/SPEC.md#33-attributes)

**When:** The same key given more than once.

**Message:** `{key}` is given more than once; give each attribute once

**Fix:** Give the key once. For several values of a set-valued key, write a value set: `platform=cloud|on-prem`.

### Directives

#### ASC005 `directive-unknown`

Warning · file level · [SPEC §3.2]({repo}/blob/main/SPEC.md#32-recognition)

**When:** Directive-shaped line (`@word` followed by `{`, `:`, or end of line) with an unknown name.

**Message:** `@{name}` isn't a known directive, so this line is text; write `\@{name}` if that's what you mean

**Fix:** Correct the directive's name, or declare a project widget with that name in `ascribe.toml`. If the line is meant as text, write `\@` at its start.

#### ASC006 `directive-primary`

Error · file level · [SPEC §3.4]({repo}/blob/main/SPEC.md#34-the-primary)

**When:** Primary given to a directive that takes none, or a required primary missing.

**Message:** `@{name}` doesn't take a primary; remove the text after its colon

**Fix:** Remove the text after the colon, or add the primary the directive needs, such as the path in `@include: setup.md`.

#### ASC120 `directive-extra-text`

Error · file level · [SPEC §3.1]({repo}/blob/main/SPEC.md#31-syntax)

**When:** Text on a directive line that fits no part of it: after the name or attributes, after `@end`, or after an identifier primary.

**Message:** `@{name}`'s primary is a single word, so `{extra}` isn't part of it; remove it

**Fix:** Remove the extra text. Text a directive should show belongs in its primary, after the colon, or in its content.

### Container

#### ASC007 `container-unclosed`

Error · file level · [SPEC §3.5]({repo}/blob/main/SPEC.md#35-forms)

**When:** Container not closed before its enclosing block ends.

**Message:** the `@{name}` container opened here isn't closed before \{end}; add `@end` where it should end

**Fix:** Add an `@end` line where the container's content ends, indented like its opener.

#### ASC008 `container-colon-unexpected`

Error · file level · [SPEC §3.5]({repo}/blob/main/SPEC.md#35-forms)

**When:** Trailing `:` on a directive with no container form.

**Message:** `@{name}` has no container form, so its line can't end in a colon; remove the trailing `:`

**Fix:** Remove the trailing colon. The directive applies to what its binding says, such as the block after it.

#### ASC009 `container-colon-missing`

Error · file level · [SPEC §3.5]({repo}/blob/main/SPEC.md#35-forms)

**When:** Container-only directive without a trailing `:`.

**Message:** `@{name}` is always a container; end its line with a colon to open it

**Fix:** End the opener's line with a colon, and close the container with `@end`.

#### ASC010 `container-open-at-arm`

Error · file level · [SPEC §3.6]({repo}/blob/main/SPEC.md#36-groups)

**When:** Container still open when the next arm of its group begins (reported at that arm's opener).

**Message:** the `@{name}` container opened on line \{line} is still open where this `@{group}` arm begins; close it with `@end` first

**Fix:** Close the inner container with `@end` before the next arm's opener.

#### ASC011 `end-unmatched`

Error · file level · [SPEC §3.5]({repo}/blob/main/SPEC.md#35-forms)

**When:** End line with no open container.

**Message:** this `@end` has no open container to close; if a directive above should be a container, end its line with a colon

**Fix:** Remove the `@end`, or make the directive it should close a container by ending that directive's line with a colon.

#### ASC012 `end-indent-mismatch`

Error · file level · [SPEC §3.9]({repo}/blob/main/SPEC.md#39-directives-inside-lists-and-blockquotes)

**When:** End line indented differently from its opener.

**Message:** this `@end` is indented differently from the `@{name}` it would close; indent them the same

**Fix:** Indent the `@end` exactly as its opener is indented.

#### ASC013 `container-nesting-deep`

Warning · file level · [SPEC §3.10]({repo}/blob/main/SPEC.md#310-nesting)

**When:** Nesting deeper than two levels.

**Message:** containers are nested \{depth} levels deep here; more than two is hard to follow, so flatten them, for example with a following-block form

**Fix:** Flatten the structure: use a directive's following-block form instead of a container, or split the content into sections.

### Binding

#### ASC014 `binding-no-block`

Error · file level · [SPEC §3.8]({repo}/blob/main/SPEC.md#38-binding)

**When:** Following-block directive with no following block in its container.

**Message:** `@{name}` applies to the block after it, but nothing follows it in this \{container}

**Fix:** Put the block the directive applies to directly below it, in the same container, or remove the directive.

#### ASC015 `binding-heading`

Error · file level · [SPEC §3.8]({repo}/blob/main/SPEC.md#38-binding)

**When:** Following-block directive bound to a heading.

**Message:** `@{name}` applies to the next block, and the next block here is a heading; a directive can't be bound to a heading

**Fix:** Move the directive below the heading, directly above the block it describes.

#### ASC016 `binding-blank-line`

Warning · file level · [SPEC §3.8]({repo}/blob/main/SPEC.md#38-binding)

**When:** Blank line between a following-block directive and its block.

**Message:** a blank line separates `@{name}` from the block it applies to; remove it so the directive touches its block

**Fix:** Remove the blank line, so the directive touches its block. `ascribe fmt` does this.

#### ASC017 `binding-not-section-top`

Error · file level · [SPEC §3.8]({repo}/blob/main/SPEC.md#38-binding)

**When:** Heading-bound directive that isn't at the top of its section.

**Message:** `@{name}` applies to its heading's section, so it must come directly under the heading, before any other content

**Fix:** Move the directive directly under its heading, before any other content of the section.

### Title

#### ASC018 `title-not-accepted`

Warning · file level · [SPEC §3.7]({repo}/blob/main/SPEC.md#37-titles)

**When:** Title given to a directive that doesn't accept one.

**Message:** this line looks like a title, but `@{name}` below it doesn't take one, so it's text; if that's intended, start it with `\.` to silence this

**Fix:** Remove the title line, or start it with `\.` if it's text that begins with a dot.

#### ASC019 `title-dot-space`

Warning · file level · [SPEC §3.7]({repo}/blob/main/SPEC.md#37-titles)

**When:** A `. ` line (dot and space) directly above a directive that accepts a title.

**Message:** a title line has no space after its dot; write `.{title}` to make this the title of the `@{name}` below

**Fix:** Remove the space after the dot (`.Title`) to make the line a title, or separate it from the directive with a blank line if it's text.

### `@id`

#### ASC020 `id-duplicate`

Error · page level · [SPEC §4.1]({repo}/blob/main/SPEC.md#41-id)

**When:** Duplicate id on a page, including ids from included content.

**Message:** the id `{id}` is used more than once on this page; every id on a page must be unique

**Fix:** Give one of the headings a different `@id`. When the second id comes from an included fragment, change it in the page or in the fragment, or include the fragment once.

#### ASC058 `id-invalid`

Error · file level · [SPEC §4.1]({repo}/blob/main/SPEC.md#41-id)

**When:** Id containing characters other than letters, digits, hyphens, underscores, and periods.

**Message:** `{id}` isn't a valid id: use only letters, digits, hyphens, underscores, and periods

**Fix:** Use only letters, digits, hyphens, underscores, and periods in the id.

### `@include`

#### ASC021 `include-target-missing`

Error · file level · [SPEC §4.2]({repo}/blob/main/SPEC.md#42-include)

**When:** Target file doesn't exist.

**Message:** the included file `{path}` doesn't exist

**Fix:** Fix the path. It's relative to the including file, or to the content root when it starts with `/`, and the file must be a Markdown source file inside the content root.

#### ASC022 `include-id-missing`

Error · page level · [SPEC §4.2]({repo}/blob/main/SPEC.md#42-include)

**When:** Target id doesn't exist in the target file.

**Message:** `{path}` has no heading with the id `{id}`

**Fix:** Fix the `#id`, or give the heading you want to include that id with `@id`.

#### ASC023 `include-cycle`

Error · page level · [SPEC §4.2]({repo}/blob/main/SPEC.md#42-include)

**When:** Include cycle.

**Message:** including `{path}` here creates a cycle: \{cycle}

**Fix:** Remove one of the includes in the cycle. A section can't include itself, directly or through other files.

#### ASC125 `include-heading-without-id`

Warning · file level · [SPEC §4.2]({repo}/blob/main/SPEC.md#42-include)

**When:** `{heading=false}` without an `#id`, which has no effect.

**Message:** `heading=false` only applies to an include of a section (`file.md#id`), so it does nothing here; remove it, or name the section

**Fix:** Remove `heading=false`, or include a section: `file.md#id`.

### `@variant`

#### ASC024 `variant-no-arm-survives`

Warning · page level · [SPEC §9.3]({repo}/blob/main/SPEC.md#93-build-modes)

**When:** No arm of a group survives a build's selection.

**Message:** build `{build}` removes every arm of this `@variant` group, so none of its content is published in that build

**Fix:** Add an arm for the build's selection, or check the build's `variants` in `ascribe.toml`. If the content is meant to be absent from that build, the warning can be ignored.

#### ASC025 `variant-unknown`

Error · file level · [SPEC §4.3]({repo}/blob/main/SPEC.md#43-variant)

**When:** Unknown dimension or value.

**Message:** `{dimension}` isn't a declared dimension; the dimensions are: \{dimensions}

**Fix:** Use a dimension and value declared in `[dimensions]`, or declare them there.

#### ASC026 `variant-mixed-arms`

Error · file level · [SPEC §4.3]({repo}/blob/main/SPEC.md#43-variant)

**When:** Group mixes labeled and dimensional arms.

**Message:** this group mixes labeled and dimensional arms; give every arm attributes, or every arm a title

**Fix:** Make every arm of the group dimensional (attributes, such as `{pm=npm}`) or every arm labeled (a title line).

#### ASC027 `variant-arm-kind`

Error · file level · [SPEC §4.3]({repo}/blob/main/SPEC.md#43-variant)

**When:** Arm has both a title and attributes, or neither.

**Message:** this `@variant` arm has both a title and attributes; use one or the other

**Fix:** Give the arm either attributes or a title line: one, not both and not neither.

#### ASC028 `variant-no-shared-dimension`

Error · file level · [SPEC §4.3]({repo}/blob/main/SPEC.md#43-variant)

**When:** Dimensional arms share no dimension key.

**Message:** the arms of this group have no dimension in common; every arm must name at least one of the same dimensions

**Fix:** Name at least one dimension that every arm of the group shares.

### `@available`

#### ASC029 `available-unknown`

Error · file level · [SPEC §4.4]({repo}/blob/main/SPEC.md#44-available)

**When:** Unknown target or state.

**Message:** `{name}` isn't a declared dimension value, dimension, or feature key

**Fix:** Use a declared dimension value, dimension, feature key, or lifecycle state, or declare the name in `ascribe.toml`.

#### ASC030 `available-history-order`

Error · file level · [SPEC §4.4]({repo}/blob/main/SPEC.md#44-available)

**When:** History out of chronological order.

**Message:** the history for `{target}` must be in chronological order, but \{later} comes before \{earlier}

**Fix:** List the target's states oldest first: `self-managed (preview 3.3, ga 3.5)`.

#### ASC031 `available-versionless`

Error · file level · [SPEC §4.4]({repo}/blob/main/SPEC.md#44-available)

**When:** Versions given for a versionless target.

**Message:** `{target}` is versionless, so it takes a state but no version

**Fix:** Remove the version. A versionless target, or a dimension name, takes a state but no version: `cloud beta`.

#### ASC032 `available-exceeds-scope`

Error · page level · [SPEC §4.4]({repo}/blob/main/SPEC.md#44-available)

**When:** Spec exceeds its enclosing scope.

**Message:** this availability includes `{target}`, which the enclosing \{scope} doesn't

**Fix:** Narrow this availability to what the enclosing page or section allows, or widen the enclosing one.

#### ASC057 `available-syntax`

Error · file level · [SPEC §4.4]({repo}/blob/main/SPEC.md#44-available)

**When:** Spec that doesn't parse, in a directive or in `available` frontmatter.

**Message:** "\{spec}" isn't a valid availability spec: \{detail}

**Fix:** Fix the availability spec. Its syntax is in the [directive reference](directives.md#availability-specs).

### `@steps`

#### ASC033 `steps-not-ordered-list`

Error · file level · [SPEC §4.6]({repo}/blob/main/SPEC.md#46-steps)

**When:** Bound block isn't an ordered list.

**Message:** `@steps` must be followed by an ordered list (`1.`, `2.`, …), but the next block is \{found}

**Fix:** Put an ordered list directly below `@steps`.

### `@details`

#### ASC034 `details-title-missing`

Error · file level · [SPEC §4.7]({repo}/blob/main/SPEC.md#47-details)

**When:** Missing title.

**Message:** `@details` needs a title line directly above it: the text readers see while the content is collapsed

**Fix:** Add a title line directly above `@details`: `.Show the full configuration`.

### Project widget

#### ASC035 `widget-schema`

Error · file level · [SPEC §6]({repo}/blob/main/SPEC.md#6-project-widgets)

**When:** Violates its declared schema.

**Message:** `@{name}` doesn't match its declaration in ascribe.toml: \{detail}

**Fix:** Change the widget's use to match its declaration in `[widgets]`, or change the declaration.

### Links

#### ASC036 `link-target-missing`

Error · file level · [SPEC §5.2]({repo}/blob/main/SPEC.md#52-links)

**When:** Target file doesn't exist.

**Message:** `{path}` doesn't exist

**Fix:** Fix the path, or create the file. Names must match exactly, including case, and the file must be inside the project.

#### ASC037 `link-id-missing`

Error · page level · [SPEC §5.2]({repo}/blob/main/SPEC.md#52-links)

**When:** Target id doesn't exist in the target file.

**Message:** `{path}` has no heading with the id `{id}`

**Fix:** Fix the `#id`, or give the heading you mean that id with `@id`.

#### ASC038 `link-to-fragment`

Error · file level · [SPEC §4.2]({repo}/blob/main/SPEC.md#42-include)

**When:** Target is a fragment.

**Message:** `{path}` is a fragment, which isn't published on its own; link to a page that includes it

**Fix:** Link to a page that includes the fragment.

#### ASC039 `link-id-in-fragment`

Error · page level · [SPEC §4.2]({repo}/blob/main/SPEC.md#42-include)

**When:** Target id exists only inside a fragment the target page includes.

**Message:** `{id}` is a heading in the fragment `{fragment}`, not in `{path}` itself; link to a page, not to an id that exists only inside a fragment

**Fix:** Link to the page without the `#id`, or to a heading the page itself contains.

#### ASC040 `link-id-removed`

Error · page level · [SPEC §9.3]({repo}/blob/main/SPEC.md#93-build-modes)

**When:** Target id is removed by a build.

**Message:** build `{build}` removes the heading `{id}` from `{path}`, so this link would be broken in that build

**Fix:** Put the link in a `@variant` arm that the same build removes, or keep the heading in that build.

#### ASC041 `link-route`

Warning · file level · [SPEC §5.2]({repo}/blob/main/SPEC.md#52-links)

**When:** Destination is a route rather than a file path.

**Message:** this looks like the published route of `{page}`; link to the file instead: `{suggestion}`

**Fix:** Link to the page's file, not its URL: Ascribe writes each output's URLs. The editor's quick fix makes the change.

#### ASC121 `link-page-dropped`

Error · page level · [SPEC §5.2]({repo}/blob/main/SPEC.md#52-links)

**When:** Target page isn't published by a build.

**Message:** build `{build}` doesn't publish `{path}`, so this link would be broken in that build; move the link into a `@variant` arm the build removes

**Fix:** Put the link in a `@variant` arm that the same build removes, or publish the page in that build.

### Images

#### ASC042 `image-source-missing`

Error · file level · [SPEC §5.3]({repo}/blob/main/SPEC.md#53-images)

**When:** Local source doesn't exist.

**Message:** the image `{path}` doesn't exist

**Fix:** Fix the path, or add the image. Names must match exactly, including case, and the file must be inside the project.

#### ASC043 `image-alt-missing`

Warning · file level · [SPEC §5.3]({repo}/blob/main/SPEC.md#53-images)

**When:** Missing alt text.

**Message:** this image has no alt text; describe it between the brackets for readers who can't see it

**Fix:** Describe the image between the brackets: `![The settings page](settings.png)`.

#### ASC059 `image-attribute-missing`

Error · file level · [SPEC §5.3]({repo}/blob/main/SPEC.md#53-images)

**When:** Required image attribute missing.

**Message:** this image is missing the required attribute `{key}`

**Fix:** Add the attribute in the block after the image: `![Alt](image.png){width=600}`.

### Phrases

#### ASC044 `phrase-undeclared`

Warning · file level · [SPEC §5.1]({repo}/blob/main/SPEC.md#51-phrases)

**When:** `{key}` in prose whose key isn't declared.

**Message:** `{key}` isn't a declared phrase, so its braces are literal text; declare it in [phrases], or write `\{` to keep it literal

**Fix:** Declare the key in `[phrases]` if it's meant as a phrase. Otherwise write `\{` to keep the braces as text.

#### ASC126 `phrase-double-braces`

Warning · file level · [SPEC §5.1]({repo}/blob/main/SPEC.md#51-phrases)

**When:** A declared `{key}` directly between two more braces (`{{key}}`), usually a substitution left over from another tool.

**Message:** `{key}` is a declared phrase between two more braces, so its value appears between literal braces; remove the outer braces, or write `\{` to keep them

**Fix:** Remove the outer braces, or write `\{` for a brace that's meant.

### Headings

#### ASC045 `heading-phrase-without-id`

Warning · file level · [SPEC §5.5]({repo}/blob/main/SPEC.md#55-heading-ids)

**When:** No `@id`, and the heading contains a phrase.

**Message:** this heading contains a phrase, so its id changes whenever the phrase's value does; give it a stable id with `@id`

**Fix:** Give the heading a stable id: an `@id` line directly under it.

#### ASC046 `heading-duplicate-without-id`

Warning · page level · [SPEC §5.5]({repo}/blob/main/SPEC.md#55-heading-ids)

**When:** No `@id`, and the heading's slug is the same as another heading's on the page, so its id is numbered.

**Message:** this heading's text gives the same slug as another heading on the page, so its id is numbered and can change when headings move; give it a stable id with `@id`

**Fix:** Give the heading a stable id: an `@id` line directly under it.

#### ASC124 `heading-empty-slug`

Warning · file level · [SPEC §5.5]({repo}/blob/main/SPEC.md#55-heading-ids)

**When:** No `@id`, and the heading's slug is empty (its text is only punctuation or emoji).

**Message:** this heading's text gives it an empty id, so nothing can link to it reliably; give it an `@id`

**Fix:** Give the heading an id: an `@id` line directly under it.

### Frontmatter

#### ASC047 `frontmatter-unknown-key`

Error · file level · [SPEC §7.2]({repo}/blob/main/SPEC.md#72-declarations)

**When:** Key the file's content type or the fragment schema doesn't declare, other than a reserved key on a page.

**Message:** `{key}` isn't a field of the content type `{type}`

**Fix:** Fix the key's spelling or remove it, or declare the field in the content type's `frontmatter` (`[fragments.frontmatter]` for a fragment).

#### ASC048 `frontmatter-missing-field`

Error · file level · [SPEC §7.2]({repo}/blob/main/SPEC.md#72-declarations)

**When:** Required field missing.

**Message:** the required field `{field}` is missing; the content type `{type}` needs it

**Fix:** Add the field to the page's frontmatter, or give it a default in the content type.

#### ASC049 `frontmatter-type-mismatch`

Error · file level · [SPEC §7.2]({repo}/blob/main/SPEC.md#72-declarations)

**When:** Value doesn't match the field's declared type.

**Message:** `{field}` must be \{expected}, but it's \{found}

**Fix:** Give a value of the field's type. Quote a string that YAML would read as something else: `version: "3.10"`.

#### ASC050 `frontmatter-reserved-in-fragment`

Error · file level · [SPEC §2.2]({repo}/blob/main/SPEC.md#22-pages-and-fragments)

**When:** Reserved key (`available`, `variant`) on a fragment.

**Message:** `{key}` is reserved for pages, and fragments can't use it

**Fix:** Remove the key from the fragment's frontmatter. Use `@available` or `@variant` in the fragment's content, or set the key on the pages that include it.

#### ASC051 `content-type-unresolved`

Error · file level · [SPEC §7.2]({repo}/blob/main/SPEC.md#72-declarations)

**When:** Page matches more than one content type, or matches none and there's no default type.

**Message:** this page matches the content types \{types}, but a page can match only one; make their `files` patterns exclusive

**Fix:** Make the content types' `files` patterns match each page once, or mark one type `default = true` for the pages no pattern matches.

#### ASC122 `frontmatter-syntax`

Error · file level · [SPEC §2.1]({repo}/blob/main/SPEC.md#21-files)

**When:** Frontmatter that isn't valid YAML.

**Message:** the frontmatter isn't valid YAML: \{detail}

**Fix:** Fix the YAML between the `---` lines.

### Lists

#### ASC052 `list-ended-by-directive`

Warning · file level · [SPEC §3.9]({repo}/blob/main/SPEC.md#39-directives-inside-lists-and-blockquotes)

**When:** Unindented directive line ends a list.

**Message:** this unindented `@{name}` ends the list above it; indent it to the list item's content to keep it in the item

**Fix:** Indent the directive to the list item's content, to keep it in the item. To end the list there, put a blank line before the directive.

#### ASC053 `directive-indented-code`

Warning · file level · [SPEC §3.9]({repo}/blob/main/SPEC.md#39-directives-inside-lists-and-blockquotes)

**When:** Directive line over-indented into an indented code block.

**Message:** this `@{name}` is indented four or more spaces past its container's content, so it's part of an indented code block, not a directive

**Fix:** Indent the directive less, to its container's content. If it's meant as code, put it in a fenced code block.

#### ASC054 `steps-numbering-continued`

Warning · file level · [SPEC §4.6]({repo}/blob/main/SPEC.md#46-steps)

**When:** An ordered list continues the numbering of a list bound by `@steps` right after it ends (usually an unindented directive split the list).

**Message:** this list continues the numbering of the `@steps` list above it, which usually means an unindented line split that list; indent the line to keep one list

**Fix:** Indent the line that splits the list, so the steps stay one list.

### Files

#### ASC123 `source-unreadable`

Error · file level · [SPEC §2.1]({repo}/blob/main/SPEC.md#21-files)

**When:** Source file that can't be read, or isn't valid UTF-8.

**Message:** this file can't be read: \{reason}

**Fix:** Check the file's permissions, and save it as UTF-8.

## The content model

### The file

#### ASC060 `model-toml-syntax`

Error · file level · [SPEC §7.1]({repo}/blob/main/SPEC.md#71-role)

**Message:** ascribe.toml isn't valid TOML: \{detail}

**Fix:** Fix the TOML syntax at the place the message names.

#### ASC061 `model-unknown-key`

Error · file level · [SPEC §7.1]({repo}/blob/main/SPEC.md#71-role)

**Message:** unknown key `{key}` in `[{table}]`

**Fix:** Fix the key's spelling, or remove it.

#### ASC062 `model-missing-key`

Error · file level · [SPEC §7.1]({repo}/blob/main/SPEC.md#71-role)

**Message:** `[{table}]` is missing the required key `{key}`

**Fix:** Add the key.

#### ASC063 `model-wrong-type`

Error · file level · [SPEC §7.1]({repo}/blob/main/SPEC.md#71-role)

**Message:** `{key}` must be \{expected}, but it's \{found}

**Fix:** Give the key a value of the expected type. Strings are always quoted in TOML.

#### ASC064 `model-invalid-value`

Error · file level · [SPEC §7.1]({repo}/blob/main/SPEC.md#71-role)

**Message:** `{key}` can't be "\{value}"; use one of: \{values}

**Fix:** Use one of the values the message lists.

#### ASC065 `model-spec-unsupported`

Error · file level · [SPEC §11]({repo}/blob/main/SPEC.md#11-versioning)

**Message:** ascribe.toml targets spec version "\{spec}", but this processor implements \{supported}

**Fix:** Set `spec = "0.1"`, the specification version this release implements, or update Ascribe.

#### ASC066 `model-invalid-name`

Error · file level · [SPEC §7.2]({repo}/blob/main/SPEC.md#72-declarations)

**Message:** `{name}` isn't a valid \{role} name: \{rule}

**Fix:** Rename it following the rule the message gives.

#### ASC067 `model-empty-text`

Error · file level · [SPEC §7.2]({repo}/blob/main/SPEC.md#72-declarations)

**Message:** `{key}` can't be empty

**Fix:** Give the key a value, or remove it.

### `[project]`

#### ASC068 `model-path-absolute`

Error · file level · [SPEC §2.2]({repo}/blob/main/SPEC.md#22-pages-and-fragments)

**Message:** `{key}` must be a path relative to ascribe.toml, not an absolute path

**Fix:** Write the path relative to the directory of `ascribe.toml`.

#### ASC069 `model-content-root-missing`

Error · file level · [SPEC §2.2]({repo}/blob/main/SPEC.md#22-pages-and-fragments)

**Message:** content root `{path}` doesn't exist

**Fix:** Create the directory, or correct `[project] content-root`.

#### ASC070 `model-output-overlaps-content`

Error · file level · [SPEC §9.4]({repo}/blob/main/SPEC.md#94-outputs)

**Message:** output directory `{output}` is inside content root `{content}`; move it outside, or builds will read their own output as source

**Fix:** Keep the output directory and the content root apart: neither inside the other, and not the same. The default output directory, `.ascribe/build`, is outside the default content root, `docs`.

### Content types, fields, and attributes

#### ASC071 `model-type-multiple-defaults`

Error · file level · [SPEC §7.2]({repo}/blob/main/SPEC.md#72-declarations)

**Message:** only one content type can be the default, but `{a}` and `{b}` both set default = true

**Fix:** Keep `default = true` on one content type only.

#### ASC072 `model-type-unreachable`

Error · file level · [SPEC §7.2]({repo}/blob/main/SPEC.md#72-declarations)

**Message:** content type `{type}` has no `files` and isn't the default, so no page can use it

**Fix:** Give the type `files` patterns, make it the default type, or remove it.

#### ASC073 `model-type-title`

Error · file level · [SPEC §7.2]({repo}/blob/main/SPEC.md#72-declarations)

**Message:** content type `{type}` must declare title = "string": it's the page title, used for empty link text

**Fix:** Declare `title = "string"` in the type's `frontmatter`.

#### ASC074 `model-field-reserved`

Error · file level · [SPEC §2.1]({repo}/blob/main/SPEC.md#21-files)

**Message:** `{field}` is reserved by the Ascribe spec and every page accepts it; remove it from `[types.{type}.frontmatter]`

**Fix:** Remove the field from the type. Every page accepts it already, with the meaning the specification gives it.

#### ASC075 `model-type-syntax`

Error · file level · [SPEC §7.2]({repo}/blob/main/SPEC.md#72-declarations)

**Message:** "\{type}" isn't a valid \{kind} type: \{detail}

**Fix:** Fix the type. The types are `string`, `number`, `boolean`, `date`, `enum(a, b)`, `list(T)`, and, for attributes, `set(T)`; a `?` at the end makes one optional.

#### ASC076 `model-type-fields`

Error · file level · [SPEC §7.2]({repo}/blob/main/SPEC.md#72-declarations)

**Message:** field `{field}` is an object, so it needs `fields`

**Fix:** Give an `object` field its `fields`, or remove `fields` from a field that isn't an object.

#### ASC077 `model-enum-values`

Error · file level · [SPEC §7.2]({repo}/blob/main/SPEC.md#72-declarations)

**Message:** `{field}` has an empty enumeration

**Fix:** List the enumeration's values once each, in `enum(a, b)` or in `values`, not both.

#### ASC078 `model-set-token`

Error · file level · [SPEC §3.3]({repo}/blob/main/SPEC.md#33-attributes)

**Message:** "\{value}" can't be in a value set: members can't contain spaces or any of , | \{ } = "

**Fix:** Remove spaces and the characters `,` `|` `{` `}` `=` `"` from the value, or make the attribute a single value instead of a set.

#### ASC079 `model-default-type`

Error · file level · [SPEC §7.2]({repo}/blob/main/SPEC.md#72-declarations)

**Message:** default for `{field}` must be \{type}, but it's \{found}

**Fix:** Give a default of the field's type.

#### ASC080 `model-phrases-field-type`

Error · file level · [SPEC §5.1]({repo}/blob/main/SPEC.md#51-phrases)

**Message:** phrases = true only works on string and list(string) fields, and `{field}` is "\{type}"

**Fix:** Remove `phrases = true`, or make the field a `string` or `list(string)`.

#### ASC081 `model-pattern-syntax`

Error · file level · [SPEC §2.2]({repo}/blob/main/SPEC.md#22-pages-and-fragments)

**Message:** "\{pattern}" isn't a valid pattern: \{detail}

**Fix:** Fix the pattern. Patterns are relative to the content root, with no leading `/` and no `..` segment.

#### ASC119 `model-attribute-reserved`

Error · file level · [SPEC §7.2]({repo}/blob/main/SPEC.md#72-declarations)

**Message:** `{key}` can't be an image attribute: HTML already uses it on the \<img> element

**Fix:** Rename the attribute.

### Dimensions, names, lifecycle states, notes, and features

#### ASC082 `model-name-multiple-roles`

Error · file level · [SPEC §7.2]({repo}/blob/main/SPEC.md#72-declarations)

**Message:** `{name}` is used as both \{role-a} and \{role-b}; a name can have only one role, so availability specs stay unambiguous

**Fix:** Rename one of them, so that each name has one role.

#### ASC083 `model-name-case`

Warning · file level · [SPEC §7.2]({repo}/blob/main/SPEC.md#72-declarations)

**Message:** `{a}` and `{b}` differ only in case; names are case-sensitive, so they're easy to confuse

**Fix:** Rename one of them, so they differ by more than case.

#### ASC084 `model-dimension-empty`

Error · file level · [SPEC §7.2]({repo}/blob/main/SPEC.md#72-declarations)

**Message:** dimension `{dimension}` has no values

**Fix:** Add the dimension's values, or remove the dimension.

#### ASC085 `model-dimension-value-duplicate`

Error · file level · [SPEC §7.2]({repo}/blob/main/SPEC.md#72-declarations)

**Message:** `{value}` appears twice in dimensions.\{dimension}.values

**Fix:** Remove the repeated value.

#### ASC086 `model-dimension-value-shared`

Error · file level · [SPEC §7.2]({repo}/blob/main/SPEC.md#72-declarations)

**Message:** `{value}` is a value of both `{a}` and `{b}`; a value can belong to only one dimension

**Fix:** Rename the value in one of the dimensions.

#### ASC087 `model-label-undeclared`

Error · file level · [SPEC §7.2]({repo}/blob/main/SPEC.md#72-declarations)

**Message:** dimensions.\{dimension}.labels has a label for `{value}`, which isn't one of its values: \{values}

**Fix:** Correct the label's key, or add the value to the dimension's `values`.

#### ASC088 `model-versionless-undeclared`

Error · file level · [SPEC §7.2]({repo}/blob/main/SPEC.md#72-declarations)

**Message:** dimensions.\{dimension}.versionless lists `{value}`, which isn't one of its values: \{values}

**Fix:** Correct the entry, or add the value to the dimension's `values`.

#### ASC089 `model-lifecycle-available-required`

Error · file level · [SPEC §7.2]({repo}/blob/main/SPEC.md#72-declarations)

**Message:** new lifecycle state `{state}` must set available = true or available = false

**Fix:** Add `available = true` or `available = false` to the state.

#### ASC090 `model-lifecycle-ga-unavailable`

Error · file level · [SPEC §7.2]({repo}/blob/main/SPEC.md#72-declarations)

**Message:** `ga` must count as available: content with no lifecycle state is ga

**Fix:** Remove `available = false` from `[lifecycle.ga]`.

#### ASC091 `model-note-label-required`

Error · file level · [SPEC §4.5]({repo}/blob/main/SPEC.md#45-note)

**Message:** new note type `{type}` needs a label, such as label = "\{Type}"

**Fix:** Give the note type a `label`.

#### ASC092 `model-availability-syntax`

Error · file level · [SPEC §4.4]({repo}/blob/main/SPEC.md#44-available)

**Message:** feature `{key}`: "\{spec}" isn't a valid availability spec: \{detail}

**Fix:** Fix the feature's `available` spec. Its syntax is in the [directive reference](directives.md#availability-specs).

#### ASC093 `model-availability-unknown-name`

Error · file level · [SPEC §4.4]({repo}/blob/main/SPEC.md#44-available)

**Message:** feature `{key}`: `{name}` isn't a declared dimension value or dimension name

**Fix:** Use a declared dimension value, dimension, or lifecycle state, or declare the name.

#### ASC094 `model-availability-versionless`

Error · file level · [SPEC §4.4]({repo}/blob/main/SPEC.md#44-available)

**Message:** feature `{key}`: `{target}` is versionless, so it takes a state but no version

**Fix:** Remove the version. A versionless target, or a dimension name, takes a state but no version.

#### ASC095 `model-availability-history-order`

Error · file level · [SPEC §4.4]({repo}/blob/main/SPEC.md#44-available)

**Message:** feature `{key}`: the history for `{target}` must be in chronological order, but \{later} comes before \{earlier}

**Fix:** List the target's states oldest first.

#### ASC096 `model-feature-nested`

Error · file level · [SPEC §4.4]({repo}/blob/main/SPEC.md#44-available)

**Message:** feature `{key}`: available must be an availability spec, not another feature (`{other}`)

**Fix:** Write the feature's availability out in full. One feature can't refer to another.

### Phrases and the glossary

#### ASC097 `model-phrase-value-type`

Error · file level · [SPEC §5.1]({repo}/blob/main/SPEC.md#51-phrases)

**Message:** phrase `{key}` must be a quoted string, but it's \{found}; write \{key} = "\{value}"

**Fix:** Quote the value.

#### ASC098 `model-glossary-duplicate-term`

Error · file level · [SPEC §5.4]({repo}/blob/main/SPEC.md#54-glossary-terms)

**Message:** "\{text}" is declared by both glossary terms `{a}` and `{b}`

**Fix:** Remove the text from one of the terms, or make the terms distinct.

#### ASC099 `model-glossary-link`

Error · file level · [SPEC §5.4]({repo}/blob/main/SPEC.md#54-glossary-terms)

**Message:** glossary term `{id}` links to \{path}, which doesn't exist

**Fix:** Fix the link, or link to a page that includes the fragment.

### Widgets

#### ASC100 `model-widget-reserved-name`

Error · file level · [SPEC §6]({repo}/blob/main/SPEC.md#6-project-widgets)

**Message:** widget name `{name}` is reserved: names starting with ascribe- belong to Ascribe's element library

**Fix:** Rename the widget, for example with your project's name as its prefix: `quill-aside`.

#### ASC101 `model-widget-forms`

Error · file level · [SPEC §6]({repo}/blob/main/SPEC.md#6-project-widgets)

**Message:** forms must be ["line"], ["container"], or ["line", "container"]

**Fix:** Set `forms` to `["line"]`, `["container"]`, or `["line", "container"]`.

#### ASC102 `model-widget-binding`

Error · file level · [SPEC §6]({repo}/blob/main/SPEC.md#6-project-widgets)

**Message:** widget `{name}` has a line form, so it needs a binding: "self", "heading", "block", or "heading-or-block"

**Fix:** Give a widget with a line form a `binding`. Remove `binding` from a container-only widget.

#### ASC103 `model-widget-container-primary`

Error · file level · [SPEC §6]({repo}/blob/main/SPEC.md#6-project-widgets)

**Message:** widget `{name}` has a container form, whose opener has no primary, so its primary can't be required; use "\{kind}?"

**Fix:** Make the primary optional (`"text?"`), or remove the widget's container form. A container-only widget takes no primary.

#### ASC104 `model-widget-groupable-form`

Error · file level · [SPEC §6]({repo}/blob/main/SPEC.md#6-project-widgets)

**Message:** widget `{name}` is groupable, so it must be container-only: forms = ["container"]

**Fix:** Set `forms = ["container"]`, or remove `groupable`.

#### ASC105 `model-widget-plain-content`

Error · file level · [SPEC §6]({repo}/blob/main/SPEC.md#6-project-widgets)

**Message:** widget `{name}` doesn't wrap content, so plain-content has no effect; remove it

**Fix:** Remove `plain-content`.

### The consumer, builds, and the editor

#### ASC106 `model-consumer-unsupported`

Error · file level · [SPEC §9.5]({repo}/blob/main/SPEC.md#95-consumer-profile)

**Message:** the \{profile} profile doesn't support \{key} = \{value}; use \{values}

**Fix:** Use one of the values the message lists.

#### ASC107 `model-consumer-site`

Error · file level · [SPEC §9.5]({repo}/blob/main/SPEC.md#95-consumer-profile)

**Message:** site must be an origin such as "https://docs.example.com"; put any path in base-path

**Fix:** Set `site` to the origin alone, such as `"https://docs.example.com"`, and put any path in `base-path`.

#### ASC108 `model-consumer-base-path`

Error · file level · [SPEC §9.5]({repo}/blob/main/SPEC.md#95-consumer-profile)

**Message:** base-path must start with "/", such as "/docs/"

**Fix:** Start `base-path` with `/`: `"/docs/"`.

#### ASC109 `model-build-name-case`

Error · file level · [SPEC §9.3]({repo}/blob/main/SPEC.md#93-build-modes)

**Message:** builds `{a}` and `{b}` differ only in case, so they'd share an output directory on some file systems

**Fix:** Rename one of the builds.

#### ASC110 `model-build-variants`

Error · file level · [SPEC §9.3]({repo}/blob/main/SPEC.md#93-build-modes)

**Message:** variants must be "switch" or a selection such as \{ deployment = "cloud" }

**Fix:** Set `variants` to `"switch"`, or to a selection that names at least one dimension.

#### ASC111 `model-build-unknown-dimension`

Error · file level · [SPEC §9.3]({repo}/blob/main/SPEC.md#93-build-modes)

**Message:** build `{build}` selects dimension `{dimension}`, which isn't declared

**Fix:** Correct the dimension's name, or declare the dimension in `[dimensions]`.

#### ASC112 `model-build-unknown-value`

Error · file level · [SPEC §9.3]({repo}/blob/main/SPEC.md#93-build-modes)

**Message:** build `{build}`: `{value}` isn't a value of `{dimension}`; values: \{values}

**Fix:** Use one of the values the message lists.

#### ASC113 `model-build-availability`

Error · file level · [SPEC §9.3]({repo}/blob/main/SPEC.md#93-build-modes)

**Message:** availability must be "badge" or \{ filter = "\<target> \<version>" }

**Fix:** Set `availability` to `"badge"`, or to `{ filter = "<target> <version>" }`.

#### ASC114 `model-build-filter-target`

Error · file level · [SPEC §9.3]({repo}/blob/main/SPEC.md#93-build-modes)

**Message:** build `{build}` filters for `{target}`, which isn't a declared dimension value

**Fix:** Filter for a declared dimension value, not a dimension name.

#### ASC115 `model-build-filter-version`

Error · file level · [SPEC §9.3]({repo}/blob/main/SPEC.md#93-build-modes)

**Message:** build `{build}` filters for `{target}`, which is versioned, so it needs a version, such as "\{target} 3.3"

**Fix:** Give a versioned target a version (`"self-managed 3.3"`), and a versionless target none.

#### ASC116 `model-build-filter-excluded`

Warning · file level · [SPEC §9.3]({repo}/blob/main/SPEC.md#93-build-modes)

**Message:** build `{build}` filters for `{target}`, but its selection keeps only \{dimension} = \{values}, so pages marked for `{target}` are dropped

**Fix:** Make the build's `variants` selection keep the target it filters for, or filter for a target it keeps.

#### ASC117 `model-editor-build-unknown`

Error · file level · [SPEC §10]({repo}/blob/main/SPEC.md#10-authoring-environment)

**Message:** editor.build is `{build}`, which isn't a declared build; builds: \{builds}

**Fix:** Set `[editor] build` to one of the builds the message lists.

#### ASC118 `model-editor-build-required`

Error · file level · [SPEC §10]({repo}/blob/main/SPEC.md#10-authoring-environment)

**Message:** there are several builds and none is named site; set [editor] build to the one the editor should check

**Fix:** Set `[editor] build` to the build the editor should check, or name one of the builds `site`.
