## The file
@memo: Generated from tests/conformance/diagnostics.toml by tests/conformance/tests/docs.rs; edit the registry, then run `ASCRIBE_BLESS=1 cargo test -p ascribe-conformance --test docs` to rewrite it.

### ASC060 `model-toml-syntax`
@id {permalink=true}: ASC060
@summary: `ascribe.toml` isn't valid TOML.

Error · file level · next step: write · [SPEC §7.1]({repo}/blob/main/SPEC.md#71-role)

**When:** `ascribe.toml` isn't valid TOML.

**Message:** ascribe.toml isn't valid TOML: \{detail}

**Fix:** Fix the TOML syntax at the place the message names.

**Read more:** [The file](../content-model/index.md#11-the-file) in the `ascribe.toml` reference.

### ASC061 `model-unknown-key`
@id {permalink=true}: ASC061
@summary: A table of `ascribe.toml` has a key this version of Ascribe doesn't know.

Error · file level · next step: choose · [SPEC §7.1]({repo}/blob/main/SPEC.md#71-role)

**When:** A table of `ascribe.toml` has a key this version of Ascribe doesn't know.

**Message:** unknown key `{key}` in `[{table}]`; this is ascribe \{version}, and the project may need a newer one

**Fix:** Fix the key's spelling, or remove it. A key added in a later version of Ascribe is unknown to an older one: when the project was written for a newer Ascribe, install it. A content type has no `files` or `default`: each page names its type in its frontmatter, as `type: how-to`, so remove them and add `type:` to the type's pages. The glossary has no `terms`: write each term as a heading marked `@term` on a glossary page, with its definition as the paragraph under it, and list the page in `[glossary] pages`. `[[intended]]` has no `term`: acknowledge a term's problem with `@intended` under its heading.

**Read more:** [The file](../content-model/index.md#11-the-file) in the `ascribe.toml` reference.

### ASC062 `model-missing-key`
@id {permalink=true}: ASC062
@summary: A table of `ascribe.toml` is missing a key it requires.

Error · file level · next step: write · [SPEC §7.1]({repo}/blob/main/SPEC.md#71-role)

**When:** A table of `ascribe.toml` is missing a key it requires.

**Message:** `[{table}]` is missing the required key `{key}`

**Fix:** Add the key.

**Read more:** [The file](../content-model/index.md#11-the-file) in the `ascribe.toml` reference.

### ASC063 `model-wrong-type`
@id {permalink=true}: ASC063
@summary: A key's value is of the wrong type, such as a number where a string is required.

Error · file level · next step: write · [SPEC §7.1]({repo}/blob/main/SPEC.md#71-role)

**When:** A key's value is of the wrong type, such as a number where a string is required.

**Message:** `{key}` must be \{expected}, but it's \{found}

**Fix:** Give the key a value of the expected type. Strings are always quoted in TOML.

**Read more:** [The file](../content-model/index.md#11-the-file) in the `ascribe.toml` reference.

### ASC064 `model-invalid-value`
@id {permalink=true}: ASC064
@summary: A key's value isn't one of the values the key allows.

Error · file level · next step: choose · [SPEC §7.1]({repo}/blob/main/SPEC.md#71-role)

**When:** A key's value isn't one of the values the key allows.

**Message:** `{key}` can't be "\{value}"; use one of: \{values}

**Fix:** Use one of the values the message lists.

**Read more:** [The file](../content-model/index.md#11-the-file) in the `ascribe.toml` reference.

### ASC065 `model-spec-unsupported`
@id {permalink=true}: ASC065
@summary: `spec` names a specification version this version of Ascribe doesn't implement.

Error · file level · next step: choose · [SPEC §11]({repo}/blob/main/SPEC.md#11-versioning)

**When:** `spec` names a specification version this version of Ascribe doesn't implement.

**Message:** ascribe.toml targets spec version "\{spec}", but this processor implements \{supported}

**Fix:** Set `spec = "0.1"`, the specification version this release implements, or update Ascribe.

**Read more:** [Top level: `spec`](../content-model/index.md#2-top-level-spec) in the `ascribe.toml` reference.

### ASC066 `model-invalid-name`
@id {permalink=true}: ASC066
@summary: A name the content model declares, such as a dimension, a value, or a content type, breaks the rules for that kind of name.

Error · file level · next step: write · [SPEC §7.2]({repo}/blob/main/SPEC.md#72-declarations)

**When:** A name the content model declares, such as a dimension, a value, or a content type, breaks the rules for that kind of name.

**Message:** `{name}` isn't a valid \{role} name: \{rule}

**Fix:** Rename it following the rule the message gives.

**Read more:** [Names](../content-model/index.md#12-names) in the `ascribe.toml` reference.

### ASC067 `model-empty-text`
@id {permalink=true}: ASC067
@summary: A key that holds text is empty.

Error · file level · next step: write · [SPEC §7.2]({repo}/blob/main/SPEC.md#72-declarations)

**When:** A key that holds text is empty.

**Message:** `{key}` can't be empty

**Fix:** Give the key a value, or remove it.

**Read more:** [The file](../content-model/index.md#11-the-file) in the `ascribe.toml` reference.

### ASC246 `model-from-file`
@id {permalink=true}: ASC246
@summary: The file `[from]` takes from doesn't exist, can't be read, or is outside the project's repository or the folder that was opened.

Error · file level · next step: write · [SPEC §7.9]({repo}/blob/main/SPEC.md#79-taking-from-another-project)

**When:** The file `[from]` takes from doesn't exist, can't be read, or is outside the project's repository or the folder that was opened.

**Message:** `{path}`, the file `[from]` takes from, doesn't exist

**Fix:** Fix the path after `from`, which is relative to the folder ascribe.toml is in, so it names the other project's `ascribe.toml`, inside the project's repository. With no repository, the editor and `ascribe mcp` read the file only inside the folder they were opened on: open the folder that contains it.

**Read more:** [Taking from another project](../content-model/index.md#21-taking-from-another-project) in the `ascribe.toml` reference.

### ASC247 `model-from-take`
@id {permalink=true}: ASC247
@summary: `take` names a key the other file doesn't have, one of this project's own, one taken twice, or something that isn't a dotted path of keys.

Error · file level · next step: write · [SPEC §7.9]({repo}/blob/main/SPEC.md#79-taking-from-another-project)

**When:** `take` names a key the other file doesn't have, one of this project's own, one taken twice, or something that isn't a dotted path of keys.

**Message:** `{file}` has no `{key}` to take

**Fix:** Name a table or a key the other file has, as a dotted path of keys: `releases` takes the whole table, `consumer.site` one key. Take each once, and not a table and a key in it. `spec`, `[project]`, and `[from]` are each project's own, so write them in this file.

**Read more:** [Taking from another project](../content-model/index.md#21-taking-from-another-project) in the `ascribe.toml` reference.

### ASC248 `model-from-written`
@id {permalink=true}: ASC248
@summary: A key is taken with `[from]` and written in `ascribe.toml` too.

Error · file level · next step: write · [SPEC §7.9]({repo}/blob/main/SPEC.md#79-taking-from-another-project)

**When:** A key is taken with `[from]` and written in `ascribe.toml` too.

**Message:** `{key}` is taken from `{file}` and written here too; a key has one home

**Fix:** Delete the key here, to use the other file's, or remove it from `take`, to keep this one. A table that's taken can't have a key of its own here: take the keys it shares instead, one by one.

**Read more:** [Taking from another project](../content-model/index.md#21-taking-from-another-project) in the `ascribe.toml` reference.

## `[project]`

### ASC068 `model-path-absolute`
@id {permalink=true}: ASC068
@summary: A path in `ascribe.toml` is absolute, where it must be relative to `ascribe.toml`.

Error · file level · next step: write · [SPEC §2.2]({repo}/blob/main/SPEC.md#22-pages-and-fragments)

**When:** A path in `ascribe.toml` is absolute, where it must be relative to `ascribe.toml`.

**Message:** `{key}` must be a path relative to ascribe.toml, not an absolute path

**Fix:** Write the path relative to the directory of `ascribe.toml`.

**Read more:** [Paths and patterns](../content-model/index.md#13-paths-and-patterns) in the `ascribe.toml` reference.

### ASC069 `model-content-root-missing`
@id {permalink=true}: ASC069
@summary: The content root doesn't exist, or isn't a directory.

Error · file level · next step: write · [SPEC §2.2]({repo}/blob/main/SPEC.md#22-pages-and-fragments)

**When:** The content root doesn't exist, or isn't a directory.

**Message:** content root `{path}` doesn't exist

**Fix:** Create the directory, or correct `[project] content-root`.

**Read more:** [`[project]`](../content-model/index.md#3-project) in the `ascribe.toml` reference.

### ASC070 `model-output-overlaps-content`
@id {permalink=true}: ASC070
@summary: The output directory and the content root are the same, or one is inside the other.

Error · file level · next step: write · [SPEC §9.4]({repo}/blob/main/SPEC.md#94-outputs)

**When:** The output directory and the content root are the same, or one is inside the other.

**Message:** output directory `{output}` is inside content root `{content}`; move it outside, or builds will read their own output as source

**Fix:** Keep the output directory and the content root apart: neither inside the other, and not the same. The default output directory, `.ascribe/build`, is outside the default content root, `docs`.

**Read more:** [`[project]`](../content-model/index.md#3-project) in the `ascribe.toml` reference.

## `[sources]`

### ASC133 `model-source-path-missing`
@id {permalink=true}: ASC133
@summary: A source's `path` doesn't exist, or isn't a folder.

Error · file level · next step: write · [SPEC §7.3]({repo}/blob/main/SPEC.md#73-sources)

**When:** A source's `path` doesn't exist, or isn't a folder.

**Message:** the folder of source `{source}`, `{path}`, doesn't exist

**Fix:** Fix `path`, which is relative to the folder ascribe.toml is in, or create the folder.

**Read more:** [`[sources.<name>]`](../content-model/output.md#18-sourcesname) in the `ascribe.toml` reference.

### ASC134 `model-source-outside-repository`
@id {permalink=true}: ASC134
@summary: A source's folder is outside the project's repository, or, with no repository, outside the folder that was opened.

Error · file level · next step: write · [SPEC §7.3]({repo}/blob/main/SPEC.md#73-sources)

**When:** A source's folder is outside the project's repository, or, with no repository, outside the folder that was opened.

**Message:** the folder of source `{source}`, `{path}`, is outside the git repository the project is in

**Fix:** Give the source a `path` inside the project's repository, or, for code in another repository, give it `git` instead. With no repository, the editor and `ascribe mcp` read a source only inside the folder they were opened on: open the folder that contains the source.

**Read more:** [`[sources.<name>]`](../content-model/output.md#18-sourcesname) in the `ascribe.toml` reference.

### ASC135 `model-source-remote`
@id {permalink=true}: ASC135
@summary: A source has both `path` and `git`, or neither, a `branch` without `git`, a URL or a branch `git` can't take, or copies that would go inside the content root.

Error · file level · next step: write · [SPEC §7.4]({repo}/blob/main/SPEC.md#74-sources-in-another-repository)

**When:** A source has both `path` and `git`, or neither, a `branch` without `git`, a URL or a branch `git` can't take, or copies that would go inside the content root.

**Message:** source `{source}` has both `path` and `git`: a source is a folder in this repository or another repository, not both

**Fix:** Give each source either `path`, a folder in this repository, or `git`, the URL of another repository, with `branch` only beside `git`. A source with `git` keeps its copies in `sources/<name>/`, which must be outside the content root.

**Read more:** [A source in another repository](../content-model/output.md#a-source-in-another-repository) in the `ascribe.toml` reference.

## Content types, fields, and attributes

### ASC074 `model-field-reserved`
@id {permalink=true}: ASC074
@summary: A content type, or the fragments' frontmatter, declares a field the specification or an output reserves.

Error · file level · next step: write · [SPEC §2.1]({repo}/blob/main/SPEC.md#21-files)

**When:** A content type, or the fragments' frontmatter, declares a field the specification or an output reserves.

**Message:** `{field}` is reserved by the Ascribe spec and every page accepts it; remove it from `[{table}]`

**Fix:** Remove the field from the type. Every page accepts it already, with the meaning the specification gives it. Every page type has `title` as a required string: declare it only to set `inline` or `phrases`.

**Read more:** [Page types](../content-model/index.md#41-page-types) in the `ascribe.toml` reference.

### ASC075 `model-type-syntax`
@id {permalink=true}: ASC075
@summary: A field's or an attribute's type isn't one Ascribe knows, or isn't written correctly.

Error · file level · next step: write · [SPEC §7.2]({repo}/blob/main/SPEC.md#72-declarations)

**When:** A field's or an attribute's type isn't one Ascribe knows, or isn't written correctly.

**Message:** "\{type}" isn't a valid \{kind} type: \{detail}

**Fix:** Fix the type. The types are `string`, `number`, `boolean`, `date`, `enum(a, b)`, `list(T)`, and, for attributes, `set(T)`; a `?` at the end makes one optional.

**Read more:** [Short form](../content-model/index.md#51-short-form) in the `ascribe.toml` reference.

### ASC076 `model-type-fields`
@id {permalink=true}: ASC076
@summary: An `object` field has no `fields`, or a field that isn't an object has them.

Error · file level · next step: write · [SPEC §7.2]({repo}/blob/main/SPEC.md#72-declarations)

**When:** An `object` field has no `fields`, or a field that isn't an object has them.

**Message:** field `{field}` is an object, so it needs `fields`

**Fix:** Give an `object` field its `fields`, or remove `fields` from a field that isn't an object.

**Read more:** [Table form](../content-model/index.md#52-table-form) in the `ascribe.toml` reference.

### ASC077 `model-enum-values`
@id {permalink=true}: ASC077
@summary: An enumeration is empty, lists a value twice, gives its values in both places, or gives none.

Error · file level · next step: write · [SPEC §7.2]({repo}/blob/main/SPEC.md#72-declarations)

**When:** An enumeration is empty, lists a value twice, gives its values in both places, or gives none.

**Message:** `{field}` has an empty enumeration

**Fix:** List the enumeration's values once each, in `enum(a, b)` or in `values`, not both.

**Read more:** [Short form](../content-model/index.md#51-short-form) in the `ascribe.toml` reference.

### ASC078 `model-set-token`
@id {permalink=true}: ASC078
@summary: A value of a set-valued attribute has a space or a reserved character.

Error · file level · next step: write · [SPEC §3.3]({repo}/blob/main/SPEC.md#33-attributes)

**When:** A value of a set-valued attribute has a space or a reserved character.

**Message:** "\{value}" can't be in a value set: members can't contain spaces or any of , | \{ } = "

**Fix:** Remove spaces and the characters `,` `|` `{` `}` `=` `"` from the value, or make the attribute a single value instead of a set.

**Read more:** [Short form](../content-model/index.md#51-short-form) in the `ascribe.toml` reference.

### ASC079 `model-default-type`
@id {permalink=true}: ASC079
@summary: A field's default isn't of the field's type, or isn't one of its values.

Error · file level · next step: write · [SPEC §7.2]({repo}/blob/main/SPEC.md#72-declarations)

**When:** A field's default isn't of the field's type, or isn't one of its values.

**Message:** default for `{field}` must be \{type}, but it's \{found}

**Fix:** Give a default of the field's type.

**Read more:** [Table form](../content-model/index.md#52-table-form) in the `ascribe.toml` reference.

### ASC080 `model-phrases-field-type`
@id {permalink=true}: ASC080
@summary: `phrases = true` is on a field that isn't a `string` or a `list(string)`.

Error · file level · next step: write · [SPEC §5.1]({repo}/blob/main/SPEC.md#51-phrases)

**When:** `phrases = true` is on a field that isn't a `string` or a `list(string)`.

**Message:** phrases = true only works on string and list(string) fields, and `{field}` is "\{type}"

**Fix:** Remove `phrases = true`, or make the field a `string` or `list(string)`.

**Read more:** [`[phrases]`](../content-model/content.md#phrases) in the `ascribe.toml` reference.

### ASC081 `model-pattern-syntax`
@id {permalink=true}: ASC081
@summary: A pattern isn't valid, starts with `/`, or has a `..` segment.

Error · file level · next step: write · [SPEC §2.2]({repo}/blob/main/SPEC.md#22-pages-and-fragments)

**When:** A pattern isn't valid, starts with `/`, or has a `..` segment.

**Message:** "\{pattern}" isn't a valid pattern: \{detail}

**Fix:** Fix the pattern. Patterns are relative to the content root, with no leading `/` and no `..` segment.

**Read more:** [Paths and patterns](../content-model/index.md#13-paths-and-patterns) in the `ascribe.toml` reference.

### ASC119 `model-attribute-reserved`
@id {permalink=true}: ASC119
@summary: An image's or a widget's attribute has a name that HTML or the site output already uses on its element.

Error · file level · next step: write · [SPEC §7.2]({repo}/blob/main/SPEC.md#72-declarations)

**When:** An image's or a widget's attribute has a name that HTML or the site output already uses on its element.

**Message:** `{key}` can't be an image attribute: HTML already uses it on the \<img> element

**Fix:** Rename the attribute.

**Read more:** [`[images]`](../content-model/content.md#13-images) in the `ascribe.toml` reference.

### ASC141 `model-inline-field`
@id {permalink=true}: ASC141
@summary: `inline` is on a field that isn't a page type's top-level string field, or isn't `"code"`.

Error · file level · next step: write · [SPEC §7.2]({repo}/blob/main/SPEC.md#72-declarations)

**When:** `inline` is on a field that isn't a page type's top-level string field, or isn't `"code"`.

**Message:** inline = "code" only works on string fields, and `{field}` is "\{type}"

**Fix:** Remove `inline`, or set it to "code" on a string field of a content type's `frontmatter`.

**Read more:** [Code in a field](../content-model/index.md#53-code-in-a-field) in the `ascribe.toml` reference.

### ASC148 `model-field-role`
@id {permalink=true}: ASC148
@summary: A field's `role` isn't one Ascribe knows, is on a field of the wrong type or below a page type's top level, or is on two fields of one type.

Error · file level · next step: write · [SPEC §7.2]({repo}/blob/main/SPEC.md#72-declarations)

**When:** A field's `role` isn't one Ascribe knows, is on a field of the wrong type or below a page type's top level, or is on two fields of one type.

**Message:** role = "\{role}" only works on a \{expected} field, and `{field}` is "\{type}"

**Fix:** Give `role = "description"` to one string field of a content type's `frontmatter`, `role = "review-date"` to one date field, and `role = "owner"` to one string or enum field, or remove `role`. A type has at most one field with each role, and only a page type's top-level fields have roles.

**Read more:** [Field roles](../content-model/index.md#54-field-roles) in the `ascribe.toml` reference.

### ASC193 `model-type-file`
@id {permalink=true}: ASC193
@summary: A content type's `template` or `guide` doesn't exist, or is inside the content root or the output directory, or outside the project's folder.

Error · file level · next step: write · [SPEC §7.2]({repo}/blob/main/SPEC.md#72-declarations)

**When:** A content type's `template` or `guide` doesn't exist, or is inside the content root or the output directory, or outside the project's folder.

**Message:** `{path}`, the \{key} of content type `{type}`, doesn't exist

**Fix:** Name a file by its path from the folder ascribe.toml is in, with its name's case as it is on disk, outside the content root and the output directory: `template = "templates/how-to.md"`. A type's template and guide are read by people and tools starting a page, and aren't pages themselves.

**Read more:** [Page types](../content-model/index.md#41-page-types) in the `ascribe.toml` reference.

### ASC194 `model-type-template`
@id {permalink=true}: ASC194
@summary: A content type's template can't be read, or isn't a page Ascribe can read.

Error · file level · next step: write · [SPEC §7.2]({repo}/blob/main/SPEC.md#72-declarations)

**When:** A content type's template can't be read, or isn't a page Ascribe can read.

**Message:** can't read `{path}`, the template of content type `{type}`: \{reason}

**Fix:** Fix the template so it reads as a page: valid YAML between its `---` lines, and no error in its Ascribe syntax. A template is read when the content model loads, so a problem in it is reported once, here, and not on every page made from it. A type that names no `template` and no `guide` takes them from `templates/templates.lock`, which `ascribe templates eject` writes: put back a file it gives (`ascribe templates status` says what's changed), or name the type's own `template` and `guide`.

**Read more:** [Page types](../content-model/index.md#41-page-types) in the `ascribe.toml` reference.

## Dimensions, names, lifecycle states, notes, and features

### ASC082 `model-name-multiple-roles`
@id {permalink=true}: ASC082

Error · file level · next step: write · [SPEC §7.2]({repo}/blob/main/SPEC.md#72-declarations)

**Message:** `{name}` is used as both \{role-a} and \{role-b}; a name can have only one role, so availability specs stay unambiguous

**Fix:** Rename one of them, so that each name has one role.

**Read more:** [Names](../content-model/index.md#12-names) in the `ascribe.toml` reference.

### ASC083 `model-name-case`
@id {permalink=true}: ASC083
@summary: Two names the content model declares differ only in case.

Warning · file level · next step: write · [SPEC §7.2]({repo}/blob/main/SPEC.md#72-declarations)

**When:** Two names the content model declares differ only in case.

**Message:** `{a}` and `{b}` differ only in case; names are case-sensitive, so they're easy to confuse

**Fix:** Rename one of them, so they differ by more than case.

**Read more:** [Names](../content-model/index.md#12-names) in the `ascribe.toml` reference.

### ASC084 `model-dimension-empty`
@id {permalink=true}: ASC084
@summary: A dimension has no values.

Error · file level · next step: write · [SPEC §7.2]({repo}/blob/main/SPEC.md#72-declarations)

**When:** A dimension has no values.

**Message:** dimension `{dimension}` has no values

**Fix:** Add the dimension's values, or remove the dimension.

**Read more:** [`[dimensions.<name>]`](../content-model/content.md#dimensions) in the `ascribe.toml` reference.

### ASC085 `model-dimension-value-duplicate`
@id {permalink=true}: ASC085
@summary: A dimension lists a value twice.

Error · file level · next step: write · [SPEC §7.2]({repo}/blob/main/SPEC.md#72-declarations)

**When:** A dimension lists a value twice.

**Message:** `{value}` appears twice in dimensions.\{dimension}.values

**Fix:** Remove the repeated value.

**Read more:** [`[dimensions.<name>]`](../content-model/content.md#dimensions) in the `ascribe.toml` reference.

### ASC086 `model-dimension-value-shared`
@id {permalink=true}: ASC086
@summary: A value is in more than one dimension.

Error · file level · next step: write · [SPEC §7.2]({repo}/blob/main/SPEC.md#72-declarations)

**When:** A value is in more than one dimension.

**Message:** `{value}` is a value of both `{a}` and `{b}`; a value can belong to only one dimension

**Fix:** Rename the value in one of the dimensions.

**Read more:** [`[dimensions.<name>]`](../content-model/content.md#dimensions) in the `ascribe.toml` reference.

### ASC087 `model-label-undeclared`
@id {permalink=true}: ASC087
@summary: A dimension's `labels` gives a label for a value the dimension doesn't have.

Error · file level · next step: choose · [SPEC §7.2]({repo}/blob/main/SPEC.md#72-declarations)

**When:** A dimension's `labels` gives a label for a value the dimension doesn't have.

**Message:** dimensions.\{dimension}.labels has a label for `{value}`, which isn't one of its values: \{values}

**Fix:** Correct the label's key, or add the value to the dimension's `values`.

**Read more:** [`[dimensions.<name>]`](../content-model/content.md#dimensions) in the `ascribe.toml` reference.

### ASC088 `model-versionless-undeclared`
@id {permalink=true}: ASC088
@summary: A dimension's `versionless` lists a value the dimension doesn't have.

Error · file level · next step: choose · [SPEC §7.2]({repo}/blob/main/SPEC.md#72-declarations)

**When:** A dimension's `versionless` lists a value the dimension doesn't have.

**Message:** dimensions.\{dimension}.versionless lists `{value}`, which isn't one of its values: \{values}

**Fix:** Correct the entry, or add the value to the dimension's `values`.

**Read more:** [`[dimensions.<name>]`](../content-model/content.md#dimensions) in the `ascribe.toml` reference.

### ASC089 `model-lifecycle-available-required`
@id {permalink=true}: ASC089
@summary: A lifecycle state the project adds doesn't say whether it counts as available.

Error · file level · next step: choose · [SPEC §7.2]({repo}/blob/main/SPEC.md#72-declarations)

**When:** A lifecycle state the project adds doesn't say whether it counts as available.

**Message:** new lifecycle state `{state}` must set available = true or available = false

**Fix:** Add `available = true` or `available = false` to the state.

**Read more:** [`[lifecycle.<state>]`](../content-model/content.md#8-lifecyclestate) in the `ascribe.toml` reference.

### ASC090 `model-lifecycle-ga-unavailable`
@id {permalink=true}: ASC090
@summary: `[lifecycle.ga]` doesn't count as available.

Error · file level · next step: write · [SPEC §7.2]({repo}/blob/main/SPEC.md#72-declarations)

**When:** `[lifecycle.ga]` doesn't count as available.

**Message:** `ga` must count as available: content with no lifecycle state is ga

**Fix:** Remove `available = false` from `[lifecycle.ga]`.

**Read more:** [`[lifecycle.<state>]`](../content-model/content.md#8-lifecyclestate) in the `ascribe.toml` reference.

### ASC091 `model-note-label-required`
@id {permalink=true}: ASC091
@summary: A note type the project adds has no `label`.

Error · file level · next step: write · [SPEC §4.5]({repo}/blob/main/SPEC.md#45-note)

**When:** A note type the project adds has no `label`.

**Message:** new note type `{type}` needs a label, such as label = "\{Type}"

**Fix:** Give the note type a `label`.

**Read more:** [`[notes.<type>]`](../content-model/content.md#10-notestype) in the `ascribe.toml` reference.

### ASC092 `model-availability-syntax`
@id {permalink=true}: ASC092
@summary: A feature's `available` isn't a valid availability spec.

Error · file level · next step: write · [SPEC §4.4]({repo}/blob/main/SPEC.md#44-available)

**When:** A feature's `available` isn't a valid availability spec.

**Message:** feature `{key}`: "\{spec}" isn't a valid availability spec: \{detail}

**Fix:** Fix the feature's `available` spec. Its syntax is in the [directive reference](../writing/directives.md#availability-specs).

**Read more:** [`[features.<key>]`](../content-model/content.md#9-featureskey) in the `ascribe.toml` reference.

### ASC093 `model-availability-unknown-name`
@id {permalink=true}: ASC093
@summary: A feature's `available` names a target or a state the content model doesn't declare.

Error · file level · next step: choose · [SPEC §4.4]({repo}/blob/main/SPEC.md#44-available)

**When:** A feature's `available` names a target or a state the content model doesn't declare.

**Message:** feature `{key}`: `{name}` isn't a declared dimension value or dimension name

**Fix:** Use a declared dimension value, dimension, or lifecycle state, or declare the name.

**Read more:** [`[features.<key>]`](../content-model/content.md#9-featureskey) in the `ascribe.toml` reference.

### ASC094 `model-availability-versionless`
@id {permalink=true}: ASC094
@summary: A feature's `available` gives a version to a versionless target or a dimension name.

Error · file level · next step: write · [SPEC §4.4]({repo}/blob/main/SPEC.md#44-available)

**When:** A feature's `available` gives a version to a versionless target or a dimension name.

**Message:** feature `{key}`: `{target}` is versionless, so it takes a state but no version

**Fix:** Remove the version. A versionless target, or a dimension name, takes a state but no version.

**Read more:** [`[features.<key>]`](../content-model/content.md#9-featureskey) in the `ascribe.toml` reference.

### ASC095 `model-availability-history-order`
@id {permalink=true}: ASC095
@summary: A feature's history for a target isn't in chronological order.

Error · file level · next step: write · [SPEC §4.4]({repo}/blob/main/SPEC.md#44-available)

**When:** A feature's history for a target isn't in chronological order.

**Message:** feature `{key}`: the history for `{target}` must be in chronological order, but \{later} comes before \{earlier}

**Fix:** List the target's states oldest first.

**Read more:** [`[features.<key>]`](../content-model/content.md#9-featureskey) in the `ascribe.toml` reference.

### ASC096 `model-feature-nested`
@id {permalink=true}: ASC096
@summary: A feature's `available` names another feature.

Error · file level · next step: write · [SPEC §4.4]({repo}/blob/main/SPEC.md#44-available)

**When:** A feature's `available` names another feature.

**Message:** feature `{key}`: available must be an availability spec, not another feature (`{other}`)

**Fix:** Write the feature's availability out in full. One feature can't refer to another.

**Read more:** [`[features.<key>]`](../content-model/content.md#9-featureskey) in the `ascribe.toml` reference.

## `[releases]`

### ASC204 `model-releases-target`
@id {permalink=true}: ASC204
@summary: A `[releases]` table names a target that's versionless, a dimension, or a name `[dimensions.release]` takes, or labels a target its dimension labels.

Error · file level · next step: choose · [SPEC §7.2]({repo}/blob/main/SPEC.md#72-declarations)

**When:** A `[releases]` table names a target that's versionless, a dimension, or a name `[dimensions.release]` takes, or labels a target its dimension labels.

**Message:** `[releases.{target}]`: `{target}` is in no declared dimension, so it would be a value of the implicit dimension `release`, but `[dimensions.release]` takes that name; add `{target}` to a dimension's `values`, or rename `[dimensions.release]`

**Fix:** Name the table after the target a mark gives a version to, such as `[releases.self-managed]`: a versioned value of a declared dimension, or a name no dimension has, which makes it a value of the implicit dimension `release`. A versionless value has no versions to list. While a target is left to the implicit dimension, no declared dimension can be named `release`.

**Read more:** [`[releases.<target>]`](../content-model/content.md#71-releasestarget) in the `ascribe.toml` reference.

### ASC205 `model-releases-line`
@id {permalink=true}: ASC205
@summary: A target's lines in `[releases]` aren't valid versions, are listed twice, or have different numbers of parts.

Error · file level · next step: write · [SPEC §7.2]({repo}/blob/main/SPEC.md#72-declarations)

**When:** A target's lines in `[releases]` aren't valid versions, are listed twice, or have different numbers of parts.

**Message:** `[releases.{target}]` mixes lines of different lengths, `{a}` and `{b}`; write every line of a target with the same number of parts

**Fix:** Write each line as a version, once, with as many parts as the target's other lines: `["3.4", "3.5"]` for a line for each minor release, or `["0.2.0"]` for one for each patch release. A line holds every version that starts with it.

**Read more:** [`[releases.<target>]`](../content-model/content.md#71-releasestarget) in the `ascribe.toml` reference.

### ASC207 `model-releases-order`
@id {permalink=true}: ASC207
@summary: A target's planned line isn't newer than every live line, an archived line isn't older than every live one, or a line is in two lists.

Error · file level · next step: write · [SPEC §7.2]({repo}/blob/main/SPEC.md#72-declarations)

**When:** A target's planned line isn't newer than every live line, an archived line isn't older than every live one, or a line is in two lists.

**Message:** `[releases.{target}]`: the planned line `{line}` isn't newer than every live line, and the newest is `{live}`

**Fix:** Keep each line in one list, in the order the lists mean: `planned` lines are newer than every `live` line, and `archived` lines older. A line that has shipped moves from `planned` to `live`, and one that has left the window from `live` to `archived`.

**Read more:** [`[releases.<target>]`](../content-model/content.md#71-releasestarget) in the `ascribe.toml` reference.

### ASC208 `model-releases-ends`
@id {permalink=true}: ASC208
@summary: `[releases.<target>.ends]` gives an end date to a line that isn't live.

Error · file level · next step: choose · [SPEC §7.2]({repo}/blob/main/SPEC.md#72-declarations)

**When:** `[releases.<target>.ends]` gives an end date to a line that isn't live.

**Message:** `[releases.{target}.ends]`: `{line}` isn't a live line; the live lines are \{live}

**Fix:** Give an end date to a line in `live` only. A line whose support has ended and that has left the window goes in `archived`, with its date as `ended`.

**Read more:** [`[releases.<target>]`](../content-model/content.md#71-releasestarget) in the `ascribe.toml` reference.

### ASC209 `model-versions-current`
@id {permalink=true}: ASC209
@summary: `ascribe.toml` has `[versions] current`, which is no longer a key.

Error · file level · next step: write · [SPEC §7.2]({repo}/blob/main/SPEC.md#72-declarations)

**When:** `ascribe.toml` has `[versions] current`, which is no longer a key.

**Message:** `[versions] current` is no longer a key; a target's current release is the newest line in `[releases.<target>] live`

**Fix:** Remove `current` from `[versions]`, and list the release in `[releases.<target>]`, as `live = ["3.4"]`, for the target it's a release of. Each target has its own current line, and a check that compared with `current`, `availability-left-behind`, compares each target's versions with that target's current line.

**Read more:** [`[releases.<target>]`](../content-model/content.md#71-releasestarget) in the `ascribe.toml` reference.

## Phrases and the glossary

### ASC097 `model-phrase-value-type`
@id {permalink=true}: ASC097
@summary: A phrase's value isn't a string, or a table naming a target's line.

Error · file level · next step: write · [SPEC §5.1]({repo}/blob/main/SPEC.md#51-phrases)

**When:** A phrase's value isn't a string, or a table naming a target's line.

**Message:** phrase `{key}` must be a quoted string, but it's \{found}; write \{key} = "\{value}"

**Fix:** Quote the value. A phrase whose value is a target's current release line is a table with one key, `line`: `release = { line = "self-managed" }`.

**Read more:** [`[phrases]`](../content-model/content.md#phrases) in the `ascribe.toml` reference.

### ASC215 `model-phrase-line`
@id {permalink=true}: ASC215
@summary: A phrase is the line of a target that has no live line.

Error · file level · next step: choose · [SPEC §5.1]({repo}/blob/main/SPEC.md#51-phrases)

**When:** A phrase is the line of a target that has no live line.

**Message:** phrase `{key}` is the line of `{target}`, which has no live line in `[releases]`

**Fix:** Name a target whose lines `[releases.<target>]` lists, with at least one in `live`, or list the target's lines there. The phrase is the target's current line: the newest live one.

**Read more:** [`[phrases]`](../content-model/content.md#phrases) in the `ascribe.toml` reference.

### ASC238 `model-glossary-page`
@id {permalink=true}: ASC238
@summary: A path in `[glossary] pages` doesn't exist, or is a fragment.

Error · file level · next step: choose · [SPEC §7.2]({repo}/blob/main/SPEC.md#72-declarations)

**When:** A path in `[glossary] pages` doesn't exist, or is a fragment.

**Message:** `{path}` in `[glossary] pages` doesn't exist

**Fix:** Each path in `pages` is a page, from the content root, with its name's case as it is on disk: fix the path, or, for a fragment, list a page that includes it.

**Read more:** [`[glossary]`](../content-model/content.md#12-glossary) in the `ascribe.toml` reference.

## Widgets

### ASC100 `model-widget-reserved-name`
@id {permalink=true}: ASC100
@summary: A widget's name is reserved by Ascribe's element library or by HTML.

Error · file level · next step: write · [SPEC §6]({repo}/blob/main/SPEC.md#6-project-widgets)

**When:** A widget's name is reserved by Ascribe's element library or by HTML.

**Message:** widget name `{name}` is reserved: names starting with ascribe- belong to Ascribe's element library

**Fix:** Rename the widget, for example with your project's name as its prefix: `quill-aside`.

**Read more:** [`[widgets.<name>]`](../content-model/content.md#14-widgetsname) in the `ascribe.toml` reference.

### ASC101 `model-widget-forms`
@id {permalink=true}: ASC101
@summary: A widget's `forms` isn't one of the lists it allows.

Error · file level · next step: choose · [SPEC §6]({repo}/blob/main/SPEC.md#6-project-widgets)

**When:** A widget's `forms` isn't one of the lists it allows.

**Message:** forms must be ["line"], ["container"], or ["line", "container"]

**Fix:** Set `forms` to `["line"]`, `["container"]`, or `["line", "container"]`.

**Read more:** [`[widgets.<name>]`](../content-model/content.md#14-widgetsname) in the `ascribe.toml` reference.

### ASC102 `model-widget-binding`
@id {permalink=true}: ASC102
@summary: A widget with a line form has no `binding`, or a container-only widget has one.

Error · file level · next step: choose · [SPEC §6]({repo}/blob/main/SPEC.md#6-project-widgets)

**When:** A widget with a line form has no `binding`, or a container-only widget has one.

**Message:** widget `{name}` has a line form, so it needs a binding: "self", "heading", "block", or "heading-or-block"

**Fix:** Give a widget with a line form a `binding`. Remove `binding` from a container-only widget.

**Read more:** [`[widgets.<name>]`](../content-model/content.md#14-widgetsname) in the `ascribe.toml` reference.

### ASC103 `model-widget-container-primary`
@id {permalink=true}: ASC103
@summary: A widget with a container form requires a primary, or a container-only widget takes one.

Error · file level · next step: write · [SPEC §6]({repo}/blob/main/SPEC.md#6-project-widgets)

**When:** A widget with a container form requires a primary, or a container-only widget takes one.

**Message:** widget `{name}` has a container form, whose opener has no primary, so its primary can't be required; use "\{kind}?"

**Fix:** Make the primary optional (`"text?"`), or remove the widget's container form. A container-only widget takes no primary.

**Read more:** [`[widgets.<name>]`](../content-model/content.md#14-widgetsname) in the `ascribe.toml` reference.

### ASC104 `model-widget-groupable-form`
@id {permalink=true}: ASC104
@summary: A groupable widget has a line form.

Error · file level · next step: write · [SPEC §6]({repo}/blob/main/SPEC.md#6-project-widgets)

**When:** A groupable widget has a line form.

**Message:** widget `{name}` is groupable, so it must be container-only: forms = ["container"]

**Fix:** Set `forms = ["container"]`, or remove `groupable`.

**Read more:** [`[widgets.<name>]`](../content-model/content.md#14-widgetsname) in the `ascribe.toml` reference.

### ASC105 `model-widget-plain-content`
@id {permalink=true}: ASC105
@summary: A widget that doesn't wrap content sets `plain-content`.

Error · file level · next step: write · [SPEC §6]({repo}/blob/main/SPEC.md#6-project-widgets)

**When:** A widget that doesn't wrap content sets `plain-content`.

**Message:** widget `{name}` doesn't wrap content, so plain-content has no effect; remove it

**Fix:** Remove `plain-content`.

**Read more:** [`[widgets.<name>]`](../content-model/content.md#14-widgetsname) in the `ascribe.toml` reference.

## The consumer, builds, and the editor

### ASC106 `model-consumer-unsupported`
@id {permalink=true}: ASC106
@summary: A `[consumer]` key has a value the consumer profile doesn't support.

Error · file level · next step: choose · [SPEC §9.5]({repo}/blob/main/SPEC.md#95-consumer-profile)

**When:** A `[consumer]` key has a value the consumer profile doesn't support.

**Message:** the \{profile} profile doesn't support \{key} = \{value}; use \{values}

**Fix:** Use one of the values the message lists.

**Read more:** [`[consumer]`](../content-model/output.md#consumer) in the `ascribe.toml` reference.

### ASC107 `model-consumer-site`
@id {permalink=true}: ASC107
@summary: `[consumer] site` isn't an origin alone, such as `https://docs.example.com`.

Error · file level · next step: write · [SPEC §9.5]({repo}/blob/main/SPEC.md#95-consumer-profile)

**When:** `[consumer] site` isn't an origin alone, such as `https://docs.example.com`.

**Message:** site must be an origin such as "https://docs.example.com"; put any path in base-path

**Fix:** Set `site` to the origin alone, such as `"https://docs.example.com"`, and put any path in `base-path`.

**Read more:** [`[consumer]`](../content-model/output.md#consumer) in the `ascribe.toml` reference.

### ASC108 `model-consumer-base-path`
@id {permalink=true}: ASC108
@summary: `[consumer] base-path` doesn't start with `/`.

Error · file level · next step: write · [SPEC §9.5]({repo}/blob/main/SPEC.md#95-consumer-profile)

**When:** `[consumer] base-path` doesn't start with `/`.

**Message:** base-path must start with "/", such as "/docs/"

**Fix:** Start `base-path` with `/`: `"/docs/"`.

**Read more:** [`[consumer]`](../content-model/output.md#consumer) in the `ascribe.toml` reference.

### ASC109 `model-build-name-case`
@id {permalink=true}: ASC109
@summary: Two builds' names differ only in case.

Error · file level · next step: write · [SPEC §9.3]({repo}/blob/main/SPEC.md#93-build-modes)

**When:** Two builds' names differ only in case.

**Message:** builds `{a}` and `{b}` differ only in case, so they'd share an output directory on some file systems

**Fix:** Rename one of the builds.

**Read more:** [`[builds.<name>]`](../content-model/output.md#16-buildsname) in the `ascribe.toml` reference.

### ASC110 `model-build-variants`
@id {permalink=true}: ASC110
@summary: A build's `variants` isn't `"switch"` or a selection that names a dimension.

Error · file level · next step: write · [SPEC §9.3]({repo}/blob/main/SPEC.md#93-build-modes)

**When:** A build's `variants` isn't `"switch"` or a selection that names a dimension.

**Message:** variants must be "switch" or a selection such as \{ deployment = "cloud" }

**Fix:** Set `variants` to `"switch"`, or to a selection that names at least one dimension.

**Read more:** [`[builds.<name>]`](../content-model/output.md#16-buildsname) in the `ascribe.toml` reference.

### ASC111 `model-build-unknown-dimension`
@id {permalink=true}: ASC111
@summary: A build's selection names a dimension the content model doesn't declare.

Error · file level · next step: choose · [SPEC §9.3]({repo}/blob/main/SPEC.md#93-build-modes)

**When:** A build's selection names a dimension the content model doesn't declare.

**Message:** build `{build}` selects dimension `{dimension}`, which isn't declared

**Fix:** Correct the dimension's name, or declare the dimension in `[dimensions]`.

**Read more:** [`[builds.<name>]`](../content-model/output.md#16-buildsname) in the `ascribe.toml` reference.

### ASC112 `model-build-unknown-value`
@id {permalink=true}: ASC112
@summary: A build's selection names a value its dimension doesn't have.

Error · file level · next step: choose · [SPEC §9.3]({repo}/blob/main/SPEC.md#93-build-modes)

**When:** A build's selection names a value its dimension doesn't have.

**Message:** build `{build}`: `{value}` isn't a value of `{dimension}`; values: \{values}

**Fix:** Use one of the values the message lists.

**Read more:** [`[builds.<name>]`](../content-model/output.md#16-buildsname) in the `ascribe.toml` reference.

### ASC113 `model-build-availability`
@id {permalink=true}: ASC113
@summary: A build's `availability` isn't `"badge"` or a filter.

Error · file level · next step: write · [SPEC §9.3]({repo}/blob/main/SPEC.md#93-build-modes)

**When:** A build's `availability` isn't `"badge"` or a filter.

**Message:** availability must be "badge" or \{ filter = "\<target> \<version>" }

**Fix:** Set `availability` to `"badge"`, or to `{ filter = "<target> <version>" }`.

**Read more:** [`[builds.<name>]`](../content-model/output.md#16-buildsname) in the `ascribe.toml` reference.

### ASC114 `model-build-filter-target`
@id {permalink=true}: ASC114
@summary: A build's filter names something other than a declared dimension value.

Error · file level · next step: choose · [SPEC §9.3]({repo}/blob/main/SPEC.md#93-build-modes)

**When:** A build's filter names something other than a declared dimension value.

**Message:** build `{build}` filters for `{target}`, which isn't a declared dimension value

**Fix:** Filter for a declared dimension value, not a dimension name.

**Read more:** [`[builds.<name>]`](../content-model/output.md#16-buildsname) in the `ascribe.toml` reference.

### ASC115 `model-build-filter-version`
@id {permalink=true}: ASC115
@summary: A build's filter gives a versioned target no version, a versionless one a version, or a version that isn't valid.

Error · file level · next step: write · [SPEC §9.3]({repo}/blob/main/SPEC.md#93-build-modes)

**When:** A build's filter gives a versioned target no version, a versionless one a version, or a version that isn't valid.

**Message:** build `{build}` filters for `{target}`, which is versioned, so it needs a version, such as "\{target} 3.3"

**Fix:** Give a versioned target a version (`"self-managed 3.3"`), and a versionless target none.

**Read more:** [`[builds.<name>]`](../content-model/output.md#16-buildsname) in the `ascribe.toml` reference.

### ASC116 `model-build-filter-excluded`
@id {permalink=true}: ASC116
@summary: A build filters for a target its own selection of variants leaves out.

Warning · file level · next step: write · [SPEC §9.3]({repo}/blob/main/SPEC.md#93-build-modes)

**When:** A build filters for a target its own selection of variants leaves out.

**Message:** build `{build}` filters for `{target}`, but its selection keeps only \{dimension} = \{values}, so pages marked for `{target}` are dropped

**Fix:** Make the build's `variants` selection keep the target it filters for, or filter for a target it keeps.

**Read more:** [`[builds.<name>]`](../content-model/output.md#16-buildsname) in the `ascribe.toml` reference.

### ASC117 `model-editor-build-unknown`
@id {permalink=true}: ASC117
@summary: `[editor] build` names a build the content model doesn't declare.

Error · file level · next step: choose · [SPEC §10]({repo}/blob/main/SPEC.md#10-authoring-environment)

**When:** `[editor] build` names a build the content model doesn't declare.

**Message:** editor.build is `{build}`, which isn't a declared build; builds: \{builds}

**Fix:** Set `[editor] build` to one of the builds the message lists.

**Read more:** [`[editor]`](../content-model/output.md#17-editor) in the `ascribe.toml` reference.

### ASC118 `model-editor-build-required`
@id {permalink=true}: ASC118
@summary: The content model declares several builds, none named `site`, and `[editor] build` doesn't name one.

Error · file level · next step: choose · [SPEC §10]({repo}/blob/main/SPEC.md#10-authoring-environment)

**When:** The content model declares several builds, none named `site`, and `[editor] build` doesn't name one.

**Message:** there are several builds and none is named site; set [editor] build to the one the editor should check

**Fix:** Set `[editor] build` to the build the editor should check, or name one of the builds `site`.

**Read more:** [`[editor]`](../content-model/output.md#17-editor) in the `ascribe.toml` reference.

### ASC162 `model-consumer-agents-site`
@id {permalink=true}: ASC162
@summary: `[consumer] agents = true` without `[consumer] site`.

Error · file level · next step: write · [SPEC §9.5]({repo}/blob/main/SPEC.md#95-consumer-profile)

**When:** `[consumer] agents = true` without `[consumer] site`.

**Message:** agents = true needs `site`: every link in llms.txt and the Markdown pages is an absolute URL

**Fix:** Set `site` in `[consumer]` to the published site's origin, such as `"https://docs.example.com"`, or remove `agents`.

**Read more:** [`[consumer]`](../content-model/output.md#consumer) in the `ascribe.toml` reference.

### ASC174 `model-consumer-permalink-path`
@id {permalink=true}: ASC174
@summary: `[consumer] permalink-path` doesn't start with `/`.

Error · file level · next step: write · [SPEC §9.5]({repo}/blob/main/SPEC.md#95-consumer-profile)

**When:** `[consumer] permalink-path` doesn't start with `/`.

**Message:** permalink-path must start with "/", such as "/go/"

**Fix:** Start `permalink-path` with `/`: `"/go/"`. It's a path from the site's root, not from `base-path`.

**Read more:** [`[consumer]`](../content-model/output.md#consumer) in the `ascribe.toml` reference.

### ASC206 `model-build-versions`
@id {permalink=true}: ASC206
@summary: A versioned build's target has no live line, or the build is filtered too.

Error · file level · next step: write · [SPEC §9.3]({repo}/blob/main/SPEC.md#93-build-modes)

**When:** A versioned build's target has no live line, or the build is filtered too.

**Message:** build `{build}` is versioned by `{target}`, which has no live line; list its lines in `[releases.{target}]`, such as `live = ["3.4"]`

**Fix:** Give the target its lines in `[releases.<target>]`, with `live`, or remove `versions` from the build. A build is either versioned, with one output for each live line, or filtered for one version; not both.

**Read more:** [`[builds.<name>]`](../content-model/output.md#16-buildsname) in the `ascribe.toml` reference.

## `[permalinks]`

### ASC178 `model-permalinks`
@id {permalink=true}: ASC178
@summary: A name in `[permalinks]` is both retired and an alias, an alias leads back to itself, or a retired permalink's reason is still a placeholder.

Error · file level · next step: write · [SPEC §7.6]({repo}/blob/main/SPEC.md#76-retired-permalinks-and-aliases)

**When:** A name in `[permalinks]` is both retired and an alias, an alias leads back to itself, or a retired permalink's reason is still a placeholder.

**Message:** `{name}` is both retired and an alias; a name is one thing

**Fix:** Give each name in `[permalinks]` one meaning: retired, in `[permalinks.retired]`, or an alias, in `[permalinks.aliases]`. A retired permalink can still lead somewhere: give it `to`. Make each alias lead, perhaps through other aliases, to a permalink a heading or page declares. Write the reason a permalink was retired in place of the placeholder the editor's quick fix writes: a reader whose link led there sees it.

**Read more:** [`[permalinks]`](../content-model/checks.md#20-permalinks) in the `ascribe.toml` reference.

## `[crawlers]`

### ASC183 `model-crawlers-line`
@id {permalink=true}: ASC183
@summary: A string in `[crawlers] lines` has a line break.

Error · file level · next step: write · [SPEC §7.2]({repo}/blob/main/SPEC.md#72-declarations)

**When:** A string in `[crawlers] lines` has a line break.

**Message:** `{key}` has a line break; each string in `lines` is one line of robots.txt

**Fix:** Split the string into one string per line: `lines = ["User-agent: *", "Disallow: /drafts/"]`.

**Read more:** [`[crawlers]`](../content-model/checks.md#21-crawlers) in the `ascribe.toml` reference.

### ASC189 `model-crawlers-named-twice`
@id {permalink=true}: ASC189
@summary: `[crawlers]` names one crawler in two groups.

Error · file level · next step: write · [SPEC §7.2]({repo}/blob/main/SPEC.md#72-declarations)

**When:** `[crawlers]` names one crawler in two groups.

**Message:** `{key}` names `{token}`, which a group on line \{line} names already; a crawler obeys only one group, so write everything for it in one

**Fix:** Name each crawler in one group. Move the lines for it into the first group that names it and remove the second `User-agent` line. For a crawler a disallowed purpose covers, the purpose's group already says `Disallow: /`: remove the group from `lines`, or allow the purpose and write the crawler's group whole in `lines`.

**Read more:** [`[crawlers]`](../content-model/checks.md#21-crawlers) in the `ascribe.toml` reference.

## `[listing]`

### ASC185 `model-listing-folder`
@id {permalink=true}: ASC185
@summary: A folder in `[listing.folders]` doesn't exist, isn't a folder, or isn't written as a folder's path.

Error · file level · next step: write · [SPEC §7.2]({repo}/blob/main/SPEC.md#72-declarations)

**When:** A folder in `[listing.folders]` doesn't exist, isn't a folder, or isn't written as a folder's path.

**Message:** `{folder}` in `[listing.folders]` doesn't exist under the content root

**Fix:** Write the folder's path from the content root, ending in `/`, with its name's case as it is on disk: `"internal/" = "unlisted"`. A rule is for a folder and everything under it; patterns aren't read.

**Read more:** [`[listing]`](../content-model/checks.md#22-listing) in the `ascribe.toml` reference.

### ASC186 `model-listing-type`
@id {permalink=true}: ASC186
@summary: `[listing.types]` names something that isn't a content type.

Error · file level · next step: choose · [SPEC §7.2]({repo}/blob/main/SPEC.md#72-declarations)

**When:** `[listing.types]` names something that isn't a content type.

**Message:** `{type}` in `[listing.types]` isn't a content type; use one of: \{values}

**Fix:** Name a content type the project declares in `[types]`, or `page` when it declares none.

**Read more:** [`[listing]`](../content-model/checks.md#22-listing) in the `ascribe.toml` reference.

### ASC187 `model-listing-looser`
@id {permalink=true}: ASC187
@summary: A rule of `[listing]` is looser than a wider rule it's under.

Error · file level · next step: write · [SPEC §7.2]({repo}/blob/main/SPEC.md#72-declarations)

**When:** A rule of `[listing]` is looser than a wider rule it's under.

**Message:** `{rule}` is looser than `{wider}` on line \{line}, which it's under; a narrower rule can only restrict

**Fix:** Remove the narrower rule, or make it as strict as the one it's under, or stricter: `listed`, then `unquoted`, then `unlisted`. To list some pages more openly than the rest, loosen the wider rule, and restrict the others with rules of their own.

**Read more:** [`[listing]`](../content-model/checks.md#22-listing) in the `ascribe.toml` reference.

## `[checks]`

### ASC142 `model-check-not-configurable`
@id {permalink=true}: ASC142
@summary: `[checks]` sets the level of a check about whether the project is valid.

Error · file level · next step: write · [SPEC §8.2]({repo}/blob/main/SPEC.md#82-diagnostics)

**When:** `[checks]` sets the level of a check about whether the project is valid.

**Message:** `{check}` can't be set in [checks]: it reports whether the project is valid, which a project can't lower or turn off

**Fix:** Remove the check from `[checks]`. Only checks about the content's quality, which the [diagnostics reference](../checks/diagnostics.md) marks as configurable, can be set there; a diagnostic about whether the project is valid is always reported at its own severity.

**Read more:** [`[checks]`](../content-model/checks.md#19-checks) in the `ascribe.toml` reference.

### ASC146 `model-intended-check`
@id {permalink=true}: ASC146
@summary: An acknowledgement in `[[intended]]` names something that isn't a check, a check that isn't a review check, or one whose problems are acknowledged in the pages.

Error · file level · next step: choose · [SPEC §4.9]({repo}/blob/main/SPEC.md#49-intended)

**When:** An acknowledgement in `[[intended]]` names something that isn't a check, a check that isn't a review check, or one whose problems are acknowledged in the pages.

**Message:** `{check}` isn't a check

**Fix:** Name a check whose next step is review and whose problems are about a phrase, feature, glossary term, or image, by its name in the [diagnostics reference](../checks/diagnostics.md). A page's or a block's problem is acknowledged in the page itself.

**Read more:** [`[[intended]]`: acknowledgements](../content-model/checks.md#intended-acknowledgements) in the `ascribe.toml` reference.

### ASC147 `model-intended-entry`
@id {permalink=true}: ASC147
@summary: An acknowledgement in `[[intended]]` doesn't name exactly one thing the content model declares, or its reason is still a placeholder.

Error · file level · next step: choose · [SPEC §4.9]({repo}/blob/main/SPEC.md#49-intended)

**When:** An acknowledgement in `[[intended]]` doesn't name exactly one thing the content model declares, or its reason is still a placeholder.

**Message:** an acknowledgement in [[intended]] names what it's about with one of `phrase`, `feature`, or `image`

**Fix:** Give the acknowledgement exactly one of `phrase`, `feature`, or `image` (a path under the content root), naming something the content model declares, and a reason of your own in place of the placeholder the editor's quick fix writes.

**Read more:** [`[[intended]]`: acknowledgements](../content-model/checks.md#intended-acknowledgements) in the `ascribe.toml` reference.

### ASC167 `model-checks-vale`
@id {permalink=true}: ASC167
@summary: `[checks.vale]` has both `preset` and `config`, or neither, or turns off a rule with `config` or one its preset doesn't have.

Error · file level · next step: write · [SPEC §8.2]({repo}/blob/main/SPEC.md#82-diagnostics)

**When:** `[checks.vale]` has both `preset` and `config`, or neither, or turns off a rule with `config` or one its preset doesn't have.

**Message:** `[checks.vale]` has both `preset` and `config`; pick one: Ascribe's preset, or the project's own `.vale.ini`

**Fix:** Give `[checks.vale]` one of `preset = "quiet"` or `config = "<path to .vale.ini>"`. `off` lists rules of the preset to turn off, by their names in alerts (`Ascribe.Repeated`), and is only for a preset.

**Read more:** [`[checks.vale]`](../content-model/checks.md#checksvale) in the `ascribe.toml` reference.
