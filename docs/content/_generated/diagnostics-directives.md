## Attributes
@memo: Generated from tests/conformance/diagnostics.toml by tests/conformance/tests/docs.rs; edit the registry, then run `ASCRIBE_BLESS=1 cargo test -p ascribe-conformance --test docs` to rewrite it.

### ASC001 `attribute-unknown-key`
@id {permalink=true}: ASC001
@summary: Unknown key for the directive, image, or table row.

Error · file level · next step: choose · [SPEC §3.3]({repo}/blob/main/SPEC.md#33-attributes)

**When:** Unknown key for the directive, image, or table row.

**Message:** `@{name}` has no attribute `{key}`; its attributes are: \{keys}

**Fix:** Fix the key's spelling, or remove it. To accept a new image attribute, declare it in `[images.attributes]`; for a project widget, in its `attributes`.

**Read more:** [Attributes](../writing/syntax.md#attributes) in the directive syntax reference.

### ASC002 `attribute-type-mismatch`
@id {permalink=true}: ASC002
@summary: Value doesn't match the key's declared type.

Error · file level · next step: choose · [SPEC §3.3]({repo}/blob/main/SPEC.md#33-attributes)

**When:** Value doesn't match the key's declared type.

**Message:** `{key}` must be \{expected}, but it's "\{value}"

**Fix:** Give a value of the key's declared type: one of the enumeration's values, `true` or `false` for a boolean, or a plain number such as `600` for a number.

**Read more:** [Attributes](../writing/syntax.md#attributes) in the directive syntax reference.

### ASC003 `attribute-bare-key`
@id {permalink=true}: ASC003
@summary: Bare key without a value.

Error · file level · next step: write · [SPEC §3.3]({repo}/blob/main/SPEC.md#33-attributes)

**When:** Bare key without a value.

**Message:** `{key}` needs a value, such as `{key}=<value>`; booleans are written out too: `{key}=true`

**Fix:** Give the key a value: `key=value`. Booleans are written out too: `open=true`.

**Read more:** [Attributes](../writing/syntax.md#attributes) in the directive syntax reference.

### ASC004 `attribute-unquoted-reserved`
@id {permalink=true}: ASC004
@summary: Unquoted value containing a reserved character.

Error · file level · next step: fix · [SPEC §3.3]({repo}/blob/main/SPEC.md#33-attributes)

**When:** Unquoted value containing a reserved character.

**Message:** the value of `{key}` contains `{character}`, so it must be quoted: `{key}="{value}"`

**Fix:** Quote the value: `label="Using other images"`.

**Read more:** [Attributes](../writing/syntax.md#attributes) in the directive syntax reference.

### ASC055 `attribute-syntax`
@id {permalink=true}: ASC055
@summary: Attribute block that doesn't parse (such as an unclosed quote or brace, or `=` with no value).

Error · file level · next step: write · [SPEC §3.3]({repo}/blob/main/SPEC.md#33-attributes)

**When:** Attribute block that doesn't parse (such as an unclosed quote or brace, or `=` with no value).

**Message:** this attribute block isn't valid: \{detail}

**Fix:** Fix the attribute block: close every quote and brace, give every `=` a value, and separate pairs with commas.

**Read more:** [Attributes](../writing/syntax.md#attributes) in the directive syntax reference.

### ASC056 `attribute-duplicate-key`
@id {permalink=true}: ASC056
@summary: The same key given more than once.

Error · file level · next step: write · [SPEC §3.3]({repo}/blob/main/SPEC.md#33-attributes)

**When:** The same key given more than once.

**Message:** `{key}` is given more than once; give each attribute once

**Fix:** Give the key once. For several values of a set-valued key, write a value set: `platform=cloud|on-prem`.

**Read more:** [Attributes](../writing/syntax.md#attributes) in the directive syntax reference.

### ASC284 `info-pair-malformed`
@id {permalink=true}: ASC284
@summary: A word after a fenced code block's language that starts like a pair a processor reads (its key in any case, then optional spaces and `=`) but isn't one.

Warning · file level · next step: write · SPEC §3.3.1

**When:** A word after a fenced code block's language that starts like a pair a processor reads (its key in any case, then optional spaces and `=`) but isn't one.

**Message:** `{written}` starts like the pair `{key}` but isn't one, so it isn't read; write `{key}=` with nothing around `=`, then the value, in double quotes if it has a space or a quote

**Fix:** Write the pair as SPEC §3.3.1 reads it: the key in lowercase, `=` with nothing around it, and the value, in double quotes when it has a space or a quote. Inside the quotes, write a quote as `\\"` and a backslash as `\\\\` in the page, since CommonMark takes one backslash off before the pair is read. A word that isn't meant as a pair can't start with `phrases`, `title`, `tested`, `request`, or `response` and `=`.

**Read more:** [Code block info strings](../writing/syntax.md#code-block-info-strings) in the directive syntax reference.

## Directives

### ASC005 `directive-unknown`
@id {permalink=true}: ASC005
@summary: Directive-shaped line (`@word` followed by `{`, `:`, or end of line) with an unknown name.

Warning · file level · next step: choose · [SPEC §3.2]({repo}/blob/main/SPEC.md#32-recognition)

**When:** Directive-shaped line (`@word` followed by `{`, `:`, or end of line) with an unknown name.

**Message:** `@{name}` isn't a known directive, so this line is text; write `\@{name}` if that's what you mean

**Fix:** Correct the directive's name, or declare a project widget with that name in `ascribe.toml`. If the line is meant as text, write `\@` at its start.

**Read more:** [Directive lines](../writing/syntax.md#directive-lines) in the directive syntax reference.

### ASC006 `directive-primary`
@id {permalink=true}: ASC006
@summary: Primary given to a directive that takes none, or a required primary missing.

Error · file level · next step: write · [SPEC §3.4]({repo}/blob/main/SPEC.md#34-the-primary)

**When:** Primary given to a directive that takes none, or a required primary missing.

**Message:** `@{name}` doesn't take a primary; remove the text after its colon

**Fix:** Remove the text after the colon, or add the primary the directive needs, such as the path in `@include: setup.md`.

**Read more:** [Directive lines](../writing/syntax.md#directive-lines) in the directive syntax reference.

### ASC120 `directive-extra-text`
@id {permalink=true}: ASC120
@summary: Text on a directive line that fits no part of it: after the name or attributes, after `@end`, or after an identifier primary.

Error · file level · next step: fix · [SPEC §3.1]({repo}/blob/main/SPEC.md#31-syntax)

**When:** Text on a directive line that fits no part of it: after the name or attributes, after `@end`, or after an identifier primary.

**Message:** `@{name}`'s primary is a single word, so `{extra}` isn't part of it; remove it

**Fix:** Put attributes in braces, as in `@note {type=warning}: text`. Remove any other extra text: text a directive should show belongs in its primary, after the colon, or in its content.

**Read more:** [Directive lines](../writing/syntax.md#directive-lines) in the directive syntax reference.

## Container

### ASC007 `container-unclosed`
@id {permalink=true}: ASC007
@summary: Container not closed before its enclosing block ends.

Error · file level · next step: write · [SPEC §3.5]({repo}/blob/main/SPEC.md#35-forms)

**When:** Container not closed before its enclosing block ends.

**Message:** the `@{name}` container opened here isn't closed before \{end}; add `@end` where it should end

**Fix:** Add an `@end` line where the container's content ends, indented like its opener.

**Read more:** [Forms: line and container](../writing/syntax.md#forms-line-and-container) in the directive syntax reference.

### ASC008 `container-colon-unexpected`
@id {permalink=true}: ASC008
@summary: Trailing `:` on a directive with no container form.

Error · file level · next step: fix · [SPEC §3.5]({repo}/blob/main/SPEC.md#35-forms)

**When:** Trailing `:` on a directive with no container form.

**Message:** `@{name}` has no container form, so its line can't end in a colon; remove the trailing `:`

**Fix:** Remove the trailing colon. The directive applies to what its binding says, such as the block after it.

**Read more:** [Forms: line and container](../writing/syntax.md#forms-line-and-container) in the directive syntax reference.

### ASC009 `container-colon-missing`
@id {permalink=true}: ASC009
@summary: Container-only directive without a trailing `:`.

Error · file level · next step: fix · [SPEC §3.5]({repo}/blob/main/SPEC.md#35-forms)

**When:** Container-only directive without a trailing `:`.

**Message:** `@{name}` is always a container; end its line with a colon to open it

**Fix:** End the opener's line with a colon, and close the container with `@end`.

**Read more:** [Forms: line and container](../writing/syntax.md#forms-line-and-container) in the directive syntax reference.

### ASC010 `container-open-at-arm`
@id {permalink=true}: ASC010
@summary: Container still open when the next arm of its group begins (reported at that arm's opener).

Error · file level · next step: write · [SPEC §3.6]({repo}/blob/main/SPEC.md#36-groups)

**When:** Container still open when the next arm of its group begins (reported at that arm's opener).

**Message:** the `@{name}` container opened on line \{line} is still open where this `@{group}` arm begins; close it with `@end` first

**Fix:** Close the inner container with `@end` before the next arm's opener.

**Read more:** [Groups](../writing/syntax.md#groups) in the directive syntax reference.

### ASC011 `end-unmatched`
@id {permalink=true}: ASC011
@summary: End line with no open container.

Error · file level · next step: write · [SPEC §3.5]({repo}/blob/main/SPEC.md#35-forms)

**When:** End line with no open container.

**Message:** this `@end` has no open container to close; if a directive above should be a container, end its line with a colon

**Fix:** Remove the `@end`, or make the directive it should close a container by ending that directive's line with a colon.

**Read more:** [Forms: line and container](../writing/syntax.md#forms-line-and-container) in the directive syntax reference.

### ASC012 `end-indent-mismatch`
@id {permalink=true}: ASC012
@summary: End line indented differently from its opener.

Error · file level · next step: write · [SPEC §3.9]({repo}/blob/main/SPEC.md#39-directives-inside-lists-and-blockquotes)

**When:** End line indented differently from its opener.

**Message:** this `@end` is indented differently from the `@{name}` it would close; indent them the same

**Fix:** Indent the `@end` exactly as its opener is indented.

**Read more:** [Lists and block quotes](../writing/syntax.md#lists-and-block-quotes) in the directive syntax reference.

### ASC013 `container-nesting-deep`
@id {permalink=true}: ASC013
@summary: Nesting deeper than two levels.

Warning · file level · next step: write · [SPEC §3.10]({repo}/blob/main/SPEC.md#310-nesting)

**When:** Nesting deeper than two levels.

**Message:** containers are nested \{depth} levels deep here; more than two is hard to follow, so flatten them, for example with a following-block form

**Fix:** Flatten the structure: use a directive's following-block form instead of a container, or split the content into sections.

**Read more:** [Forms: line and container](../writing/syntax.md#forms-line-and-container) in the directive syntax reference.

## Binding

### ASC014 `binding-no-block`
@id {permalink=true}: ASC014
@summary: Following-block directive with no following block in its container.

Error · file level · next step: write · [SPEC §3.8]({repo}/blob/main/SPEC.md#38-binding)

**When:** Following-block directive with no following block in its container.

**Message:** `@{name}` applies to the block after it, but nothing follows it in this \{container}

**Fix:** Put the block the directive applies to directly below it, in the same container, or remove the directive.

**Read more:** [Binding: what a directive applies to](../writing/syntax.md#binding-what-a-directive-applies-to) in the directive syntax reference.

### ASC015 `binding-heading`
@id {permalink=true}: ASC015
@summary: Following-block directive bound to a heading.

Error · file level · next step: write · [SPEC §3.8]({repo}/blob/main/SPEC.md#38-binding)

**When:** Following-block directive bound to a heading.

**Message:** `@{name}` applies to the next block, and the next block here is a heading; a directive can't be bound to a heading

**Fix:** Move the directive below the heading, directly above the block it describes.

**Read more:** [Binding: what a directive applies to](../writing/syntax.md#binding-what-a-directive-applies-to) in the directive syntax reference.

### ASC016 `binding-blank-line`
@id {permalink=true}: ASC016
@summary: Blank line between a following-block directive and its block.

Warning · file level · next step: fix · [SPEC §3.8]({repo}/blob/main/SPEC.md#38-binding)

**When:** Blank line between a following-block directive and its block.

**Message:** a blank line separates `@{name}` from the block it applies to; remove it so the directive touches its block

**Fix:** Remove the blank line, so the directive touches its block. `ascribe fmt` does this.

**Read more:** [Binding: what a directive applies to](../writing/syntax.md#binding-what-a-directive-applies-to) in the directive syntax reference.

### ASC017 `binding-not-section-top`
@id {permalink=true}: ASC017
@summary: Heading-bound directive that isn't at the top of its section.

Error · file level · next step: write · [SPEC §3.8]({repo}/blob/main/SPEC.md#38-binding)

**When:** Heading-bound directive that isn't at the top of its section.

**Message:** `@{name}` applies to its heading's section, so it must come directly under the heading, before any other content

**Fix:** Move the directive directly under its heading, before any other content of the section.

**Read more:** [Binding: what a directive applies to](../writing/syntax.md#binding-what-a-directive-applies-to) in the directive syntax reference.

## Title

### ASC018 `title-not-accepted`
@id {permalink=true}: ASC018
@summary: Title given to a directive that doesn't accept one.

Warning · file level · next step: write · [SPEC §3.7]({repo}/blob/main/SPEC.md#37-titles)

**When:** Title given to a directive that doesn't accept one.

**Message:** this line looks like a title, but `@{name}` below it doesn't take one, so it's text; if that's intended, start it with `\.` to silence this

**Fix:** Remove the title line, or start it with `\.` if it's text that begins with a dot.

**Read more:** [Titles](../writing/syntax.md#titles) in the directive syntax reference.

### ASC019 `title-dot-space`
@id {permalink=true}: ASC019
@summary: A `. ` line (dot and space) directly above a directive that accepts a title.

Warning · file level · next step: write · [SPEC §3.7]({repo}/blob/main/SPEC.md#37-titles)

**When:** A `. ` line (dot and space) directly above a directive that accepts a title.

**Message:** a title line has no space after its dot; write `.{title}` to make this the title of the `@{name}` below

**Fix:** Remove the space after the dot (`.Title`) to make the line a title, or separate it from the directive with a blank line if it's text.

**Read more:** [Titles](../writing/syntax.md#titles) in the directive syntax reference.

## `@id`

### ASC020 `id-duplicate`
@id {permalink=true}: ASC020
@summary: Duplicate id on a page, including ids from included content.

Error · page level · next step: write · [SPEC §4.1]({repo}/blob/main/SPEC.md#41-id)

**When:** Duplicate id on a page, including ids from included content.

**Message:** the id `{id}` is used more than once on this page; every id on a page must be unique

**Fix:** Give one of the headings a different `@id`. When the second id comes from an included fragment, change it in the page or in the fragment, or include the fragment once.

**Read more:** [`@id`](../writing/directives.md#id) in the directive reference.

### ASC058 `id-invalid`
@id {permalink=true}: ASC058
@summary: Id, or a page's `permalink`, containing characters other than letters, digits, hyphens, underscores, and periods.

Error · file level · next step: write · [SPEC §4.1]({repo}/blob/main/SPEC.md#41-id)

**When:** Id, or a page's `permalink`, containing characters other than letters, digits, hyphens, underscores, and periods.

**Message:** `{id}` isn't a valid id: use only letters, digits, hyphens, underscores, and periods

**Fix:** Use only letters, digits, hyphens, underscores, and periods in the id, or in the page's `permalink`.

**Read more:** [`@id`](../writing/directives.md#id) in the directive reference.

### ASC173 `permalink-duplicate`
@id {permalink=true}: ASC173
@summary: Permalink published by one build in more than one place, names compared ignoring case: on two pages, or as a page's own and a heading's.

Error · page level · next step: write · [SPEC §4.1]({repo}/blob/main/SPEC.md#41-id)

**When:** Permalink published by one build in more than one place, names compared ignoring case: on two pages, or as a page's own and a heading's.

**Message:** the permalink `{name}` is also on `{page}`; a build can publish each permalink in one place only

**Fix:** Give one of them a different name (a heading's `@id`, or a page's `permalink`), or stop it being a permalink: remove `{permalink=true}` from the `@id`, or `permalink` from the frontmatter. When the permalink is in a fragment that several pages include, mark the heading in one page instead, or make sure no build publishes two of those pages.

**Read more:** [Permalinks](../writing/directives.md#permalinks) in the directive reference.

## `@include`

### ASC021 `include-target-missing`
@id {permalink=true}: ASC021
@summary: Target file doesn't exist.

Error · file level · next step: choose · [SPEC §4.2]({repo}/blob/main/SPEC.md#42-include)

**When:** Target file doesn't exist.

**Message:** the included file `{path}` doesn't exist

**Fix:** Fix the path. It's relative to the including file, or to the content root when it starts with `/`, and the file must be a Markdown source file inside the content root.

**Read more:** [`@include`](../writing/directives.md#include) in the directive reference.

### ASC022 `include-id-missing`
@id {permalink=true}: ASC022
@summary: Target id doesn't exist in the target file.

Error · page level · next step: choose · [SPEC §4.2]({repo}/blob/main/SPEC.md#42-include)

**When:** Target id doesn't exist in the target file.

**Message:** `{path}` has no heading with the id `{id}`

**Fix:** Fix the `#id`, or give the heading you want to include that id with `@id`.

**Read more:** [`@include`](../writing/directives.md#include) in the directive reference.

### ASC023 `include-cycle`
@id {permalink=true}: ASC023
@summary: Include cycle.

Error · page level · next step: write · [SPEC §4.2]({repo}/blob/main/SPEC.md#42-include)

**When:** Include cycle.

**Message:** including `{path}` here creates a cycle: \{cycle}

**Fix:** Remove one of the includes in the cycle. A section can't include itself, directly or through other files.

**Read more:** [`@include`](../writing/directives.md#include) in the directive reference.

### ASC125 `include-heading-without-id`
@id {permalink=true}: ASC125
@summary: `{heading=false}` without an `#id`, which has no effect.

Warning · file level · next step: write · [SPEC §4.2]({repo}/blob/main/SPEC.md#42-include)

**When:** `{heading=false}` without an `#id`, which has no effect.

**Message:** `heading=false` only applies to an include of a section (`file.md#id`), so it does nothing here; remove it, or name the section

**Fix:** Remove `heading=false`, or include a section: `file.md#id`.

**Read more:** [`@include`](../writing/directives.md#include) in the directive reference.

### ASC182 `include-limit`
@id {permalink=true}: ASC182
@summary: A page's includes expand it past the processor's limit.

Error · page level · next step: write · [SPEC §4.2]({repo}/blob/main/SPEC.md#42-include)

**When:** A page's includes expand it past the processor's limit.

**Message:** including `{path}` here takes this page past \{limit}, the most a page's includes may expand it to; it isn't expanded, and no include after it is

**Fix:** Include less on this page. Usually a fragment is included more than once on the way: a few files that each include the next twice double the page at every step. Include each fragment once, or split the page.

**Read more:** [`@include`](../writing/directives.md#include) in the directive reference.

## `@variant`

### ASC024 `variant-no-arm-survives`
@id {permalink=true}: ASC024
@summary: No arm of a group survives a build's selection.

Warning · page level · next step: write · [SPEC §9.3]({repo}/blob/main/SPEC.md#93-build-modes)

**When:** No arm of a group survives a build's selection.

**Message:** build `{build}` removes every arm of this `@variant` group, so none of its content is published in that build

**Fix:** Add an arm for the build's selection, or check the build's `variants` in `ascribe.toml`. If the content is meant to be absent from that build, the warning can be ignored.

**Read more:** [`@variant`](../writing/directives.md#variant) in the directive reference.

### ASC025 `variant-unknown`
@id {permalink=true}: ASC025
@summary: Unknown dimension or value.

Error · file level · next step: choose · [SPEC §4.3]({repo}/blob/main/SPEC.md#43-variant)

**When:** Unknown dimension or value.

**Message:** `{dimension}` isn't a declared dimension; the dimensions are: \{dimensions}

**Fix:** Use a dimension and value declared in `[dimensions]`, or declare them there: `[dimensions.<name>]` with `values = ["<value>", …]`.

**Read more:** [`@variant`](../writing/directives.md#variant) in the directive reference.

### ASC026 `variant-mixed-arms`
@id {permalink=true}: ASC026
@summary: Group mixes labeled and dimensional arms.

Error · file level · next step: write · [SPEC §4.3]({repo}/blob/main/SPEC.md#43-variant)

**When:** Group mixes labeled and dimensional arms.

**Message:** this group mixes labeled and dimensional arms; give every arm attributes, or every arm a title

**Fix:** Make every arm of the group dimensional (attributes, such as `{pm=npm}`) or every arm labeled (a title line).

**Read more:** [`@variant`](../writing/directives.md#variant) in the directive reference.

### ASC027 `variant-arm-kind`
@id {permalink=true}: ASC027
@summary: Arm has both a title and attributes, or neither.

Error · file level · next step: write · [SPEC §4.3]({repo}/blob/main/SPEC.md#43-variant)

**When:** Arm has both a title and attributes, or neither.

**Message:** this `@variant` arm has both a title and attributes; use one or the other

**Fix:** Give the arm either attributes or a title line: one, not both and not neither.

**Read more:** [`@variant`](../writing/directives.md#variant) in the directive reference.

### ASC028 `variant-no-shared-dimension`
@id {permalink=true}: ASC028
@summary: Dimensional arms share no dimension key.

Error · file level · next step: write · [SPEC §4.3]({repo}/blob/main/SPEC.md#43-variant)

**When:** Dimensional arms share no dimension key.

**Message:** the arms of this group have no dimension in common; every arm must name at least one of the same dimensions

**Fix:** Name at least one dimension that every arm of the group shares.

**Read more:** [`@variant`](../writing/directives.md#variant) in the directive reference.

## `@available`

### ASC029 `available-unknown`
@id {permalink=true}: ASC029
@summary: Unknown target or state.

Error · file level · next step: choose · [SPEC §4.4]({repo}/blob/main/SPEC.md#44-available)

**When:** Unknown target or state.

**Message:** `{name}` isn't a declared dimension value, dimension, or feature key

**Fix:** Use a declared dimension value, dimension, feature key, or lifecycle state, or declare the name in `ascribe.toml`.

**Read more:** [`@available`](../writing/directives.md#available) in the directive reference.

### ASC030 `available-history-order`
@id {permalink=true}: ASC030
@summary: History out of chronological order.

Error · file level · next step: write · [SPEC §4.4]({repo}/blob/main/SPEC.md#44-available)

**When:** History out of chronological order.

**Message:** the history for `{target}` must be in chronological order, but \{later} comes before \{earlier}

**Fix:** List the target's states oldest first: `self-managed (preview 3.3, ga 3.5)`.

**Read more:** [Availability specs](../writing/directives.md#availability-specs) in the directive reference.

### ASC031 `available-versionless`
@id {permalink=true}: ASC031
@summary: Versions given for a versionless target.

Error · file level · next step: write · [SPEC §4.4]({repo}/blob/main/SPEC.md#44-available)

**When:** Versions given for a versionless target.

**Message:** `{target}` is versionless, so it takes a state but no version

**Fix:** Remove the version. A versionless target, or a dimension name, takes a state but no version: `cloud beta`.

**Read more:** [Availability specs](../writing/directives.md#availability-specs) in the directive reference.

### ASC032 `available-exceeds-scope`
@id {permalink=true}: ASC032
@summary: Spec exceeds its enclosing scope.

Error · page level · next step: write · [SPEC §4.4]({repo}/blob/main/SPEC.md#44-available)

**When:** Spec exceeds its enclosing scope.

**Message:** this availability includes `{target}`, which the enclosing \{scope} doesn't

**Fix:** Narrow this availability to what the enclosing page or section allows, or widen the enclosing one.

**Read more:** [`@available`](../writing/directives.md#available) in the directive reference.

### ASC057 `available-syntax`
@id {permalink=true}: ASC057
@summary: Spec that doesn't parse, in a directive or in `available` frontmatter.

Error · file level · next step: write · [SPEC §4.4]({repo}/blob/main/SPEC.md#44-available)

**When:** Spec that doesn't parse, in a directive or in `available` frontmatter.

**Message:** "\{spec}" isn't a valid availability spec: \{detail}

**Fix:** Fix the availability spec. Its syntax is in the [directive reference](../writing/directives.md#availability-specs).

**Read more:** [Availability specs](../writing/directives.md#availability-specs) in the directive reference.

### ASC210 `available-version-ahead`
@id {permalink=true}: ASC210
@summary: Version newer than every release line its target declares.

Error · file level · next step: choose · [SPEC §4.4]({repo}/blob/main/SPEC.md#44-available)

**When:** Version newer than every release line its target declares.

**Message:** `{version}` is newer than every line `[releases.{target}]` declares, and the newest is `{newest}`

**Fix:** Correct the number to a declared line, such as the newest, or, for a line that's planned, add it to `planned` in `[releases.<target>]`. Work for the planned line is marked with `planned` in place of its number, as `<target> planned`, and `ascribe versions release` writes the number when the line ships.

**Read more:** [`[releases.<target>]`](../content-model/content.md#71-releasestarget) in the `ascribe.toml` reference.

### ASC245 `available-planned-line`
@id {permalink=true}: ASC245
@summary: `planned` for a target with no planned line, or more than one.

Error · file level · next step: write · [SPEC §4.4]({repo}/blob/main/SPEC.md#44-available)

**When:** `planned` for a target with no planned line, or more than one.

**Message:** `{target}` has no planned line, so `planned` names no version; declare one in `[releases.{target}] planned`, or write the version

**Fix:** Write the version in place of `planned`, or, for work in a line that hasn't shipped, declare that line in `[releases.<target>] planned`. `planned` names a target's planned line, so it needs exactly one: with two, write the version of the one the content is in.

**Read more:** [The planned line](../writing/directives.md#the-planned-line) in the directive reference.

## `@steps`

### ASC033 `steps-not-ordered-list`
@id {permalink=true}: ASC033
@summary: Bound block isn't an ordered list.

Error · file level · next step: write · [SPEC §4.6]({repo}/blob/main/SPEC.md#46-steps)

**When:** Bound block isn't an ordered list.

**Message:** `@steps` must be followed by an ordered list (`1.`, `2.`, …), but the next block is \{found}

**Fix:** Put an ordered list directly below `@steps`.

**Read more:** [`@steps`](../writing/directives.md#steps) in the directive reference.

## `@details`

### ASC034 `details-title-missing`
@id {permalink=true}: ASC034
@summary: Missing title.

Error · file level · next step: write · [SPEC §4.7]({repo}/blob/main/SPEC.md#47-details)

**When:** Missing title.

**Message:** `@details` needs a title line directly above it: the text readers see while the content is collapsed

**Fix:** Add a title line directly above `@details`: `.Show the full configuration`.

**Read more:** [`@details`](../writing/directives.md#details) in the directive reference.

## Project widget

### ASC035 `widget-schema`
@id {permalink=true}: ASC035
@summary: Violates its declared schema.

Error · file level · next step: write · [SPEC §6]({repo}/blob/main/SPEC.md#6-project-widgets)

**When:** Violates its declared schema.

**Message:** `@{name}` doesn't match its declaration in ascribe.toml: \{detail}

**Fix:** Change the widget's use to match its declaration in `[widgets]`, or change the declaration.

**Read more:** [Project widgets](../writing/directives.md#project-widgets) in the directive reference.
