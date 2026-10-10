<!-- Generated from tests/conformance/diagnostics.toml by tests/conformance/tests/docs.rs. Edit the registry, then run `ASCRIBE_BLESS=1 cargo test -p ascribe-conformance --test docs`. -->

### Attributes

#### ASC001 `attribute-unknown-key`

Error · file level · next step: choose · [SPEC §3.3]({repo}/blob/main/SPEC.md#33-attributes)

**When:** Unknown key for the directive, image, or table row.

**Message:** `@{name}` has no attribute `{key}`; its attributes are: \{keys}

**Fix:** Fix the key's spelling, or remove it. To accept a new image attribute, declare it in `[images.attributes]`; for a project widget, in its `attributes`.

#### ASC002 `attribute-type-mismatch`

Error · file level · next step: choose · [SPEC §3.3]({repo}/blob/main/SPEC.md#33-attributes)

**When:** Value doesn't match the key's declared type.

**Message:** `{key}` must be \{expected}, but it's "\{value}"

**Fix:** Give a value of the key's declared type: one of the enumeration's values, `true` or `false` for a boolean, or a plain number such as `600` for a number.

#### ASC003 `attribute-bare-key`

Error · file level · next step: write · [SPEC §3.3]({repo}/blob/main/SPEC.md#33-attributes)

**When:** Bare key without a value.

**Message:** `{key}` needs a value, such as `{key}=<value>`; booleans are written out too: `{key}=true`

**Fix:** Give the key a value: `key=value`. Booleans are written out too: `open=true`.

#### ASC004 `attribute-unquoted-reserved`

Error · file level · next step: fix · [SPEC §3.3]({repo}/blob/main/SPEC.md#33-attributes)

**When:** Unquoted value containing a reserved character.

**Message:** the value of `{key}` contains `{character}`, so it must be quoted: `{key}="{value}"`

**Fix:** Quote the value: `label="Using other images"`.

#### ASC055 `attribute-syntax`

Error · file level · next step: write · [SPEC §3.3]({repo}/blob/main/SPEC.md#33-attributes)

**When:** Attribute block that doesn't parse (such as an unclosed quote or brace, or `=` with no value).

**Message:** this attribute block isn't valid: \{detail}

**Fix:** Fix the attribute block: close every quote and brace, give every `=` a value, and separate pairs with commas.

#### ASC056 `attribute-duplicate-key`

Error · file level · next step: write · [SPEC §3.3]({repo}/blob/main/SPEC.md#33-attributes)

**When:** The same key given more than once.

**Message:** `{key}` is given more than once; give each attribute once

**Fix:** Give the key once. For several values of a set-valued key, write a value set: `platform=cloud|on-prem`.

### Directives

#### ASC005 `directive-unknown`

Warning · file level · next step: choose · [SPEC §3.2]({repo}/blob/main/SPEC.md#32-recognition)

**When:** Directive-shaped line (`@word` followed by `{`, `:`, or end of line) with an unknown name.

**Message:** `@{name}` isn't a known directive, so this line is text; write `\@{name}` if that's what you mean

**Fix:** Correct the directive's name, or declare a project widget with that name in `ascribe.toml`. If the line is meant as text, write `\@` at its start.

#### ASC006 `directive-primary`

Error · file level · next step: write · [SPEC §3.4]({repo}/blob/main/SPEC.md#34-the-primary)

**When:** Primary given to a directive that takes none, or a required primary missing.

**Message:** `@{name}` doesn't take a primary; remove the text after its colon

**Fix:** Remove the text after the colon, or add the primary the directive needs, such as the path in `@include: setup.md`.

#### ASC120 `directive-extra-text`

Error · file level · next step: fix · [SPEC §3.1]({repo}/blob/main/SPEC.md#31-syntax)

**When:** Text on a directive line that fits no part of it: after the name or attributes, after `@end`, or after an identifier primary.

**Message:** `@{name}`'s primary is a single word, so `{extra}` isn't part of it; remove it

**Fix:** Remove the extra text. Text a directive should show belongs in its primary, after the colon, or in its content.

### Container

#### ASC007 `container-unclosed`

Error · file level · next step: write · [SPEC §3.5]({repo}/blob/main/SPEC.md#35-forms)

**When:** Container not closed before its enclosing block ends.

**Message:** the `@{name}` container opened here isn't closed before \{end}; add `@end` where it should end

**Fix:** Add an `@end` line where the container's content ends, indented like its opener.

#### ASC008 `container-colon-unexpected`

Error · file level · next step: fix · [SPEC §3.5]({repo}/blob/main/SPEC.md#35-forms)

**When:** Trailing `:` on a directive with no container form.

**Message:** `@{name}` has no container form, so its line can't end in a colon; remove the trailing `:`

**Fix:** Remove the trailing colon. The directive applies to what its binding says, such as the block after it.

#### ASC009 `container-colon-missing`

Error · file level · next step: fix · [SPEC §3.5]({repo}/blob/main/SPEC.md#35-forms)

**When:** Container-only directive without a trailing `:`.

**Message:** `@{name}` is always a container; end its line with a colon to open it

**Fix:** End the opener's line with a colon, and close the container with `@end`.

#### ASC010 `container-open-at-arm`

Error · file level · next step: write · [SPEC §3.6]({repo}/blob/main/SPEC.md#36-groups)

**When:** Container still open when the next arm of its group begins (reported at that arm's opener).

**Message:** the `@{name}` container opened on line \{line} is still open where this `@{group}` arm begins; close it with `@end` first

**Fix:** Close the inner container with `@end` before the next arm's opener.

#### ASC011 `end-unmatched`

Error · file level · next step: write · [SPEC §3.5]({repo}/blob/main/SPEC.md#35-forms)

**When:** End line with no open container.

**Message:** this `@end` has no open container to close; if a directive above should be a container, end its line with a colon

**Fix:** Remove the `@end`, or make the directive it should close a container by ending that directive's line with a colon.

#### ASC012 `end-indent-mismatch`

Error · file level · next step: write · [SPEC §3.9]({repo}/blob/main/SPEC.md#39-directives-inside-lists-and-blockquotes)

**When:** End line indented differently from its opener.

**Message:** this `@end` is indented differently from the `@{name}` it would close; indent them the same

**Fix:** Indent the `@end` exactly as its opener is indented.

#### ASC013 `container-nesting-deep`

Warning · file level · next step: write · [SPEC §3.10]({repo}/blob/main/SPEC.md#310-nesting)

**When:** Nesting deeper than two levels.

**Message:** containers are nested \{depth} levels deep here; more than two is hard to follow, so flatten them, for example with a following-block form

**Fix:** Flatten the structure: use a directive's following-block form instead of a container, or split the content into sections.

### Binding

#### ASC014 `binding-no-block`

Error · file level · next step: write · [SPEC §3.8]({repo}/blob/main/SPEC.md#38-binding)

**When:** Following-block directive with no following block in its container.

**Message:** `@{name}` applies to the block after it, but nothing follows it in this \{container}

**Fix:** Put the block the directive applies to directly below it, in the same container, or remove the directive.

#### ASC015 `binding-heading`

Error · file level · next step: write · [SPEC §3.8]({repo}/blob/main/SPEC.md#38-binding)

**When:** Following-block directive bound to a heading.

**Message:** `@{name}` applies to the next block, and the next block here is a heading; a directive can't be bound to a heading

**Fix:** Move the directive below the heading, directly above the block it describes.

#### ASC016 `binding-blank-line`

Warning · file level · next step: fix · [SPEC §3.8]({repo}/blob/main/SPEC.md#38-binding)

**When:** Blank line between a following-block directive and its block.

**Message:** a blank line separates `@{name}` from the block it applies to; remove it so the directive touches its block

**Fix:** Remove the blank line, so the directive touches its block. `ascribe fmt` does this.

#### ASC017 `binding-not-section-top`

Error · file level · next step: write · [SPEC §3.8]({repo}/blob/main/SPEC.md#38-binding)

**When:** Heading-bound directive that isn't at the top of its section.

**Message:** `@{name}` applies to its heading's section, so it must come directly under the heading, before any other content

**Fix:** Move the directive directly under its heading, before any other content of the section.

### Title

#### ASC018 `title-not-accepted`

Warning · file level · next step: write · [SPEC §3.7]({repo}/blob/main/SPEC.md#37-titles)

**When:** Title given to a directive that doesn't accept one.

**Message:** this line looks like a title, but `@{name}` below it doesn't take one, so it's text; if that's intended, start it with `\.` to silence this

**Fix:** Remove the title line, or start it with `\.` if it's text that begins with a dot.

#### ASC019 `title-dot-space`

Warning · file level · next step: write · [SPEC §3.7]({repo}/blob/main/SPEC.md#37-titles)

**When:** A `. ` line (dot and space) directly above a directive that accepts a title.

**Message:** a title line has no space after its dot; write `.{title}` to make this the title of the `@{name}` below

**Fix:** Remove the space after the dot (`.Title`) to make the line a title, or separate it from the directive with a blank line if it's text.

### `@id`

#### ASC020 `id-duplicate`

Error · page level · next step: write · [SPEC §4.1]({repo}/blob/main/SPEC.md#41-id)

**When:** Duplicate id on a page, including ids from included content.

**Message:** the id `{id}` is used more than once on this page; every id on a page must be unique

**Fix:** Give one of the headings a different `@id`. When the second id comes from an included fragment, change it in the page or in the fragment, or include the fragment once.

#### ASC058 `id-invalid`

Error · file level · next step: write · [SPEC §4.1]({repo}/blob/main/SPEC.md#41-id)

**When:** Id containing characters other than letters, digits, hyphens, underscores, and periods.

**Message:** `{id}` isn't a valid id: use only letters, digits, hyphens, underscores, and periods

**Fix:** Use only letters, digits, hyphens, underscores, and periods in the id.

### `@include`

#### ASC021 `include-target-missing`

Error · file level · next step: choose · [SPEC §4.2]({repo}/blob/main/SPEC.md#42-include)

**When:** Target file doesn't exist.

**Message:** the included file `{path}` doesn't exist

**Fix:** Fix the path. It's relative to the including file, or to the content root when it starts with `/`, and the file must be a Markdown source file inside the content root.

#### ASC022 `include-id-missing`

Error · page level · next step: choose · [SPEC §4.2]({repo}/blob/main/SPEC.md#42-include)

**When:** Target id doesn't exist in the target file.

**Message:** `{path}` has no heading with the id `{id}`

**Fix:** Fix the `#id`, or give the heading you want to include that id with `@id`.

#### ASC023 `include-cycle`

Error · page level · next step: write · [SPEC §4.2]({repo}/blob/main/SPEC.md#42-include)

**When:** Include cycle.

**Message:** including `{path}` here creates a cycle: \{cycle}

**Fix:** Remove one of the includes in the cycle. A section can't include itself, directly or through other files.

#### ASC125 `include-heading-without-id`

Warning · file level · next step: write · [SPEC §4.2]({repo}/blob/main/SPEC.md#42-include)

**When:** `{heading=false}` without an `#id`, which has no effect.

**Message:** `heading=false` only applies to an include of a section (`file.md#id`), so it does nothing here; remove it, or name the section

**Fix:** Remove `heading=false`, or include a section: `file.md#id`.

### `@variant`

#### ASC024 `variant-no-arm-survives`

Warning · page level · next step: write · [SPEC §9.3]({repo}/blob/main/SPEC.md#93-build-modes)

**When:** No arm of a group survives a build's selection.

**Message:** build `{build}` removes every arm of this `@variant` group, so none of its content is published in that build

**Fix:** Add an arm for the build's selection, or check the build's `variants` in `ascribe.toml`. If the content is meant to be absent from that build, the warning can be ignored.

#### ASC025 `variant-unknown`

Error · file level · next step: choose · [SPEC §4.3]({repo}/blob/main/SPEC.md#43-variant)

**When:** Unknown dimension or value.

**Message:** `{dimension}` isn't a declared dimension; the dimensions are: \{dimensions}

**Fix:** Use a dimension and value declared in `[dimensions]`, or declare them there.

#### ASC026 `variant-mixed-arms`

Error · file level · next step: write · [SPEC §4.3]({repo}/blob/main/SPEC.md#43-variant)

**When:** Group mixes labeled and dimensional arms.

**Message:** this group mixes labeled and dimensional arms; give every arm attributes, or every arm a title

**Fix:** Make every arm of the group dimensional (attributes, such as `{pm=npm}`) or every arm labeled (a title line).

#### ASC027 `variant-arm-kind`

Error · file level · next step: write · [SPEC §4.3]({repo}/blob/main/SPEC.md#43-variant)

**When:** Arm has both a title and attributes, or neither.

**Message:** this `@variant` arm has both a title and attributes; use one or the other

**Fix:** Give the arm either attributes or a title line: one, not both and not neither.

#### ASC028 `variant-no-shared-dimension`

Error · file level · next step: write · [SPEC §4.3]({repo}/blob/main/SPEC.md#43-variant)

**When:** Dimensional arms share no dimension key.

**Message:** the arms of this group have no dimension in common; every arm must name at least one of the same dimensions

**Fix:** Name at least one dimension that every arm of the group shares.

### `@available`

#### ASC029 `available-unknown`

Error · file level · next step: choose · [SPEC §4.4]({repo}/blob/main/SPEC.md#44-available)

**When:** Unknown target or state.

**Message:** `{name}` isn't a declared dimension value, dimension, or feature key

**Fix:** Use a declared dimension value, dimension, feature key, or lifecycle state, or declare the name in `ascribe.toml`.

#### ASC030 `available-history-order`

Error · file level · next step: write · [SPEC §4.4]({repo}/blob/main/SPEC.md#44-available)

**When:** History out of chronological order.

**Message:** the history for `{target}` must be in chronological order, but \{later} comes before \{earlier}

**Fix:** List the target's states oldest first: `self-managed (preview 3.3, ga 3.5)`.

#### ASC031 `available-versionless`

Error · file level · next step: write · [SPEC §4.4]({repo}/blob/main/SPEC.md#44-available)

**When:** Versions given for a versionless target.

**Message:** `{target}` is versionless, so it takes a state but no version

**Fix:** Remove the version. A versionless target, or a dimension name, takes a state but no version: `cloud beta`.

#### ASC032 `available-exceeds-scope`

Error · page level · next step: write · [SPEC §4.4]({repo}/blob/main/SPEC.md#44-available)

**When:** Spec exceeds its enclosing scope.

**Message:** this availability includes `{target}`, which the enclosing \{scope} doesn't

**Fix:** Narrow this availability to what the enclosing page or section allows, or widen the enclosing one.

#### ASC057 `available-syntax`

Error · file level · next step: write · [SPEC §4.4]({repo}/blob/main/SPEC.md#44-available)

**When:** Spec that doesn't parse, in a directive or in `available` frontmatter.

**Message:** "\{spec}" isn't a valid availability spec: \{detail}

**Fix:** Fix the availability spec. Its syntax is in the [directive reference](../reference/directives.md#availability-specs).

### `@steps`

#### ASC033 `steps-not-ordered-list`

Error · file level · next step: write · [SPEC §4.6]({repo}/blob/main/SPEC.md#46-steps)

**When:** Bound block isn't an ordered list.

**Message:** `@steps` must be followed by an ordered list (`1.`, `2.`, …), but the next block is \{found}

**Fix:** Put an ordered list directly below `@steps`.

### `@details`

#### ASC034 `details-title-missing`

Error · file level · next step: write · [SPEC §4.7]({repo}/blob/main/SPEC.md#47-details)

**When:** Missing title.

**Message:** `@details` needs a title line directly above it: the text readers see while the content is collapsed

**Fix:** Add a title line directly above `@details`: `.Show the full configuration`.

### Project widget

#### ASC035 `widget-schema`

Error · file level · next step: write · [SPEC §6]({repo}/blob/main/SPEC.md#6-project-widgets)

**When:** Violates its declared schema.

**Message:** `@{name}` doesn't match its declaration in ascribe.toml: \{detail}

**Fix:** Change the widget's use to match its declaration in `[widgets]`, or change the declaration.

### Links

#### ASC036 `link-target-missing`

Error · file level · next step: choose · [SPEC §5.2]({repo}/blob/main/SPEC.md#52-links)

**When:** Target file doesn't exist.

**Message:** `{path}` doesn't exist

**Fix:** Fix the path, or create the file. Names must match exactly, including case, and the file must be inside the project.

#### ASC037 `link-id-missing`

Error · page level · next step: choose · [SPEC §5.2]({repo}/blob/main/SPEC.md#52-links)

**When:** Target id doesn't exist in the target file.

**Message:** `{path}` has no heading with the id `{id}`

**Fix:** Fix the `#id`, or give the heading you mean that id with `@id`.

#### ASC038 `link-to-fragment`

Error · file level · next step: choose · [SPEC §4.2]({repo}/blob/main/SPEC.md#42-include)

**When:** Target is a fragment.

**Message:** `{path}` is a fragment, which isn't published on its own; link to a page that includes it

**Fix:** Link to a page that includes the fragment.

#### ASC040 `link-id-removed`

Error · page level · next step: write · [SPEC §9.3]({repo}/blob/main/SPEC.md#93-build-modes)

**When:** Target id is removed by a build.

**Message:** build `{build}` removes the heading `{id}` from `{path}`, so this link would be broken in that build

**Fix:** Put the link in a `@variant` arm that the same build removes, or keep the heading in that build.

#### ASC041 `link-route`

Warning · file level · next step: choose · [SPEC §5.2]({repo}/blob/main/SPEC.md#52-links)

**When:** Destination is a route rather than a file path.

**Message:** this looks like the published route of `{page}`; link to the file instead: `{suggestion}`

**Fix:** Link to the page's file, not its URL: Ascribe writes each output's URLs. The editor's quick fix makes the change.

#### ASC121 `link-page-dropped`

Error · page level · next step: write · [SPEC §5.2]({repo}/blob/main/SPEC.md#52-links)

**When:** Target page isn't published by a build.

**Message:** build `{build}` doesn't publish `{path}`, so this link would be broken in that build; move the link into a `@variant` arm the build removes

**Fix:** Put the link in a `@variant` arm that the same build removes, or publish the page in that build.

### Images

#### ASC042 `image-source-missing`

Error · file level · next step: choose · [SPEC §5.3]({repo}/blob/main/SPEC.md#53-images)

**When:** Local source doesn't exist.

**Message:** the image `{path}` doesn't exist

**Fix:** Fix the path, or add the image. Names must match exactly, including case, and the file must be inside the project.

#### ASC043 `image-alt-missing`

Warning · file level · next step: write · [SPEC §5.3]({repo}/blob/main/SPEC.md#53-images)

**When:** Missing alt text.

**Message:** this image has no alt text; describe it between the brackets for readers who can't see it

**Fix:** Describe the image between the brackets: `![The settings page](../reference/settings.png)`.

#### ASC059 `image-attribute-missing`

Error · file level · next step: write · [SPEC §5.3]({repo}/blob/main/SPEC.md#53-images)

**When:** Required image attribute missing.

**Message:** this image is missing the required attribute `{key}`

**Fix:** Add the attribute in the block after the image: `![Alt](../reference/image.png){width=600}`.

### Phrases

#### ASC044 `phrase-undeclared`

Warning · file level · next step: choose · [SPEC §5.1]({repo}/blob/main/SPEC.md#51-phrases)

**When:** `{key}` in prose whose key isn't declared.

**Message:** `{key}` isn't a declared phrase, so its braces are literal text; declare it in [phrases], or write `\{` to keep it literal

**Fix:** Declare the key in `[phrases]` if it's meant as a phrase. Otherwise write `\{` to keep the braces as text.

#### ASC126 `phrase-double-braces`

Warning · file level · next step: fix · [SPEC §5.1]({repo}/blob/main/SPEC.md#51-phrases)

**When:** A declared `{key}` directly between two more braces (`{{key}}`), usually a substitution left over from another tool.

**Message:** `{key}` is a declared phrase between two more braces, so its value appears between literal braces; remove the outer braces, or write `\{` to keep them

**Fix:** Remove the outer braces, or write `\{` for a brace that's meant.

### Headings

#### ASC045 `heading-phrase-without-id`

Warning · file level · next step: fix · [SPEC §5.5]({repo}/blob/main/SPEC.md#55-heading-ids)

**When:** No `@id`, and the heading contains a phrase.

**Message:** this heading contains a phrase, so its id changes whenever the phrase's value does; give it a stable id with `@id`

**Fix:** Give the heading a stable id: an `@id` line directly under it.

#### ASC046 `heading-duplicate-without-id`

Warning · page level · next step: fix · [SPEC §5.5]({repo}/blob/main/SPEC.md#55-heading-ids)

**When:** No `@id`, and the heading's slug is the same as another heading's on the page, so its id is numbered.

**Message:** this heading's text gives the same slug as another heading on the page, so its id is numbered and can change when headings move; give it a stable id with `@id`

**Fix:** Give the heading a stable id: an `@id` line directly under it.

#### ASC124 `heading-empty-slug`

Warning · file level · next step: write · [SPEC §5.5]({repo}/blob/main/SPEC.md#55-heading-ids)

**When:** No `@id`, and the heading's slug is empty (its text is only punctuation or emoji).

**Message:** this heading's text gives it an empty id, so nothing can link to it reliably; give it an `@id`

**Fix:** Give the heading an id: an `@id` line directly under it.

### Frontmatter

#### ASC047 `frontmatter-unknown-key`

Error · file level · next step: choose · [SPEC §7.2]({repo}/blob/main/SPEC.md#72-declarations)

**When:** Key the file's content type or the fragment schema doesn't declare, other than a reserved key on a page or `intended` on a fragment.

**Message:** `{key}` isn't a field of the content type `{type}`

**Fix:** Fix the key's spelling or remove it, or declare the field in the content type's `frontmatter` (`[fragments.frontmatter]` for a fragment).

#### ASC048 `frontmatter-missing-field`

Error · file level · next step: write · [SPEC §7.2]({repo}/blob/main/SPEC.md#72-declarations)

**When:** Required field missing.

**Message:** the required field `{field}` is missing; the content type `{type}` needs it

**Fix:** Add the field to the page's frontmatter, or give it a default in the content type.

#### ASC049 `frontmatter-type-mismatch`

Error · file level · next step: choose · [SPEC §7.2]({repo}/blob/main/SPEC.md#72-declarations)

**When:** Value doesn't match the field's declared type.

**Message:** `{field}` must be \{expected}, but it's \{found}

**Fix:** Give a value of the field's type. Quote a string that YAML would read as something else: `version: "3.10"`.

#### ASC050 `frontmatter-reserved-in-fragment`

Error · file level · next step: write · [SPEC §2.2]({repo}/blob/main/SPEC.md#22-pages-and-fragments)

**When:** Reserved key (`available`, `variant`) on a fragment.

**Message:** `{key}` is reserved for pages, and fragments can't use it

**Fix:** Remove the key from the fragment's frontmatter. Use `@available` or `@variant` in the fragment's content, or set the key on the pages that include it.

#### ASC051 `content-type-unresolved`

Error · file level · next step: write · [SPEC §7.2]({repo}/blob/main/SPEC.md#72-declarations)

**When:** Page matches more than one content type, or matches none and there's no default type.

**Message:** this page matches the content types \{types}, but a page can match only one; make their `files` patterns exclusive

**Fix:** Make the content types' `files` patterns match each page once, or mark one type `default = true` for the pages no pattern matches.

#### ASC122 `frontmatter-syntax`

Error · file level · next step: write · [SPEC §2.1]({repo}/blob/main/SPEC.md#21-files)

**When:** Frontmatter that isn't valid YAML.

**Message:** the frontmatter isn't valid YAML: \{detail}

**Fix:** Fix the YAML between the `---` lines.

### Lists

#### ASC052 `list-ended-by-directive`

Warning · file level · next step: write · [SPEC §3.9]({repo}/blob/main/SPEC.md#39-directives-inside-lists-and-blockquotes)

**When:** Unindented directive line ends a list.

**Message:** this unindented `@{name}` ends the list above it; indent it to the list item's content to keep it in the item

**Fix:** Indent the directive to the list item's content, to keep it in the item. To end the list there, put a blank line before the directive.

#### ASC053 `directive-indented-code`

Warning · file level · next step: write · [SPEC §3.9]({repo}/blob/main/SPEC.md#39-directives-inside-lists-and-blockquotes)

**When:** Directive line over-indented into an indented code block.

**Message:** this `@{name}` is indented four or more spaces past its container's content, so it's part of an indented code block, not a directive

**Fix:** Indent the directive less, to its container's content. If it's meant as code, put it in a fenced code block.

#### ASC054 `steps-numbering-continued`

Warning · file level · next step: write · [SPEC §4.6]({repo}/blob/main/SPEC.md#46-steps)

**When:** An ordered list continues the numbering of a list bound by `@steps` right after it ends (usually an unindented directive split the list).

**Message:** this list continues the numbering of the `@steps` list above it, which usually means an unindented line split that list; indent the line to keep one list

**Fix:** Indent the line that splits the list, so the steps stay one list.

### Files

#### ASC123 `source-unreadable`

Error · file level · next step: write · [SPEC §2.1]({repo}/blob/main/SPEC.md#21-files)

**When:** Source file that can't be read, or isn't valid UTF-8.

**Message:** this file can't be read: \{reason}

**Fix:** Check the file's permissions, and save it as UTF-8.

### `@snippet`

#### ASC127 `snippet-address`

Error · file level · next step: write · [SPEC §4.8]({repo}/blob/main/SPEC.md#48-snippet)

**When:** Address that isn't `<source>:<path>`, optionally with `#<region>`.

**Message:** `{address}` isn't a snippet address: \{detail}; write `<source>:<path>`, optionally with `#<region>`

**Fix:** Write the address as `<source>:<path>`, with the path relative to the source's folder and no `..`, and add `#<region>` for a region. A file outside the project's folder can only be named through a source: declare one in `[sources.<name>]`.

#### ASC128 `snippet-source-unknown`

Error · file level · next step: choose · [SPEC §4.8]({repo}/blob/main/SPEC.md#48-snippet)

**When:** Source the content model doesn't declare.

**Message:** there's no source named `{source}`; the declared sources are: \{sources}

**Fix:** Fix the source's name, or declare it in ascribe.toml: `[sources.<name>]` with a `path`.

#### ASC129 `snippet-file-missing`

Error · file level · next step: choose · [SPEC §4.8]({repo}/blob/main/SPEC.md#48-snippet)

**When:** File doesn't exist, or its source doesn't include it.

**Message:** `{path}` doesn't exist in source `{source}`

**Fix:** Fix the path, which is relative to the source's folder. For a file the source doesn't include, add a pattern that matches it to the source's `include`, or take it out of `ignore`. For a link, name the file it leads to through a source that includes it. For a source in another repository, run `ascribe sources fetch` to copy the file. When the file isn't in the repository at the pinned commit, because it was moved or deleted there, `fetch` can't copy it: name the file where it is now, or change the code and move the pin with `ascribe sources update`.

#### ASC130 `snippet-file-not-text`

Error · file level · next step: write · [SPEC §4.8]({repo}/blob/main/SPEC.md#48-snippet)

**When:** File isn't text.

**Message:** `{path}` isn't text (\{reason}), so it can't be a snippet

**Fix:** Take the snippet from a text file: UTF-8, with no NUL characters.

#### ASC131 `snippet-region-missing`

Error · file level · next step: choose · [SPEC §4.8]({repo}/blob/main/SPEC.md#48-snippet)

**When:** Region doesn't exist in the file.

**Message:** `{path}` has no region `{region}`; its regions are: \{regions}

**Fix:** Fix the region's name, or mark the region in the file with `:snippet-start: <name>` and `:snippet-end:` comments. A file whose extension isn't in the comment table can only be used whole.

#### ASC132 `snippet-tags`

Error · file level · next step: write · [SPEC §4.8]({repo}/blob/main/SPEC.md#48-snippet)

**When:** The file's tags are unbalanced, name a region twice, or use a reserved tag.

**Message:** `{path}` can't be used: `{tag}` on line \{line} is never closed

**Fix:** Fix the tags in the code file: give every `-start` tag its `-end`, give each region its own name, and remove tags Ascribe reserves for later (`state`, `replace`, `uncomment`, `emphasize`).

### Source copies

#### ASC136 `lock-invalid`

Error · file level · next step: write · [SPEC §7.4]({repo}/blob/main/SPEC.md#74-sources-in-another-repository)

**When:** `ascribe.lock` isn't valid TOML, or doesn't have the shape §7.4 gives it.

**Message:** ascribe.lock can't be read: \{detail}

**Fix:** ascribe.lock is written by `ascribe sources fetch` and `ascribe sources update`. Undo the change made to it by hand, or delete it and run `ascribe sources fetch`.

#### ASC137 `lock-source-unknown`

Error · file level · next step: write · [SPEC §7.4]({repo}/blob/main/SPEC.md#74-sources-in-another-repository)

**When:** `ascribe.lock` pins a source the content model doesn't declare with `git`, or pins it with another `git`.

**Message:** ascribe.lock pins source `{source}`, which ascribe.toml doesn't declare; run `ascribe sources fetch` to remove it

**Fix:** Run `ascribe sources fetch` to make the lock match ascribe.toml, or `ascribe sources update <name>` for a source whose `git` changed.

#### ASC138 `source-copy-changed`

Error · file level · next step: write · [SPEC §7.4]({repo}/blob/main/SPEC.md#74-sources-in-another-repository)

**When:** A file `ascribe.lock` lists is missing from the source's folder, or its hash differs.

**Message:** `{path}` isn't the file ascribe.lock pins: it was changed here. Change the code in source `{source}`'s repository and run `ascribe sources update {source}`, or run `ascribe sources fetch` to undo the change

**Fix:** A copy is the file as it is in its repository at the pinned commit. Change the code where it lives, then run `ascribe sources update <name>`; `ascribe sources fetch` puts back a copy that was changed or deleted here.

#### ASC139 `source-copy-unlocked`

Error · file level · next step: write · [SPEC §7.4]({repo}/blob/main/SPEC.md#74-sources-in-another-repository)

**When:** A file in a source's copies folder that `ascribe.lock` doesn't list.

**Message:** `{path}` is in source `{source}`'s copies, but ascribe.lock doesn't list it, so there's no telling which commit it's from; run `ascribe sources fetch`

**Fix:** Run `ascribe sources fetch`: it copies the files snippets use at the pinned commit and records them, and removes the rest.

#### ASC140 `source-copy-unused`

Warning · file level · next step: write · [SPEC §7.4]({repo}/blob/main/SPEC.md#74-sources-in-another-repository)

**When:** A copy that no snippet uses.

**Message:** no snippet uses `{path}` any more; `ascribe sources fetch` removes it

**Fix:** Run `ascribe sources fetch`, which removes the copies no snippet uses.

### `@intended`

#### ASC143 `intended-check`

Error · file level · next step: choose · [SPEC §4.9]({repo}/blob/main/SPEC.md#49-intended)

**When:** Names no check, a check that doesn't exist, a check that isn't a review check, or a check reported somewhere else.

**Message:** `{check}` isn't a check

**Fix:** Name the check whose problem is intended, by its name in the [diagnostics reference](../reference/diagnostics.md): a check whose next step is review, acknowledged where it reports its problems. A page's problem goes in the page's `intended` frontmatter, a block's with `@intended` above the block, and a content model entry's in `[[intended]]` in `ascribe.toml`. Any other problem isn't acknowledged; fix it, or set its level in `[checks]` if it's configurable.

#### ASC144 `intended-entry`

Error · file level · next step: write · [SPEC §4.9]({repo}/blob/main/SPEC.md#49-intended)

**When:** An `intended` frontmatter entry that isn't a mapping with a `check` and a reason, or an acknowledgement whose reason is still a processor's placeholder.

**Message:** `intended` is a list of acknowledgements, each with a `check` and a `reason`

**Fix:** Write `intended` as a list, each entry with the `check` it acknowledges and a `reason` that says why the problem is intended:

```yaml
intended:
  - check: page-orphan
    reason: Linked from the site's sidebar.
```

The editor's quick fix writes a placeholder reason for you to replace: until you do, the acknowledgement is reported.

#### ASC145 `intended-unused`

Advice · page level · next step: fix · [SPEC §4.9]({repo}/blob/main/SPEC.md#49-intended)

**When:** An acknowledgement that matches no problem in any build.

**Message:** nothing here needs acknowledging any more: `{check}` reports no problem here in any build

**Fix:** Remove the acknowledgement: what it excused has gone. `ascribe check` reports this only when it checks every build, since a problem may appear in one build alone.
