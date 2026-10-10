<!-- Generated from tests/conformance/diagnostics.toml by tests/conformance/tests/docs.rs. Edit the registry, then run `ASCRIBE_BLESS=1 cargo test -p ascribe-conformance --test docs`. -->

### The file

#### ASC060 `model-toml-syntax`
@id {permalink=true}: ASC060

Error · file level · next step: write · [SPEC §7.1]({repo}/blob/main/SPEC.md#71-role)

**Message:** ascribe.toml isn't valid TOML: \{detail}

**Fix:** Fix the TOML syntax at the place the message names.

#### ASC061 `model-unknown-key`
@id {permalink=true}: ASC061

Error · file level · next step: choose · [SPEC §7.1]({repo}/blob/main/SPEC.md#71-role)

**Message:** unknown key `{key}` in `[{table}]`

**Fix:** Fix the key's spelling, or remove it.

#### ASC062 `model-missing-key`
@id {permalink=true}: ASC062

Error · file level · next step: write · [SPEC §7.1]({repo}/blob/main/SPEC.md#71-role)

**Message:** `[{table}]` is missing the required key `{key}`

**Fix:** Add the key.

#### ASC063 `model-wrong-type`
@id {permalink=true}: ASC063

Error · file level · next step: write · [SPEC §7.1]({repo}/blob/main/SPEC.md#71-role)

**Message:** `{key}` must be \{expected}, but it's \{found}

**Fix:** Give the key a value of the expected type. Strings are always quoted in TOML.

#### ASC064 `model-invalid-value`
@id {permalink=true}: ASC064

Error · file level · next step: choose · [SPEC §7.1]({repo}/blob/main/SPEC.md#71-role)

**Message:** `{key}` can't be "\{value}"; use one of: \{values}

**Fix:** Use one of the values the message lists.

#### ASC065 `model-spec-unsupported`
@id {permalink=true}: ASC065

Error · file level · next step: choose · [SPEC §11]({repo}/blob/main/SPEC.md#11-versioning)

**Message:** ascribe.toml targets spec version "\{spec}", but this processor implements \{supported}

**Fix:** Set `spec = "0.1"`, the specification version this release implements, or update Ascribe.

#### ASC066 `model-invalid-name`
@id {permalink=true}: ASC066

Error · file level · next step: write · [SPEC §7.2]({repo}/blob/main/SPEC.md#72-declarations)

**Message:** `{name}` isn't a valid \{role} name: \{rule}

**Fix:** Rename it following the rule the message gives.

#### ASC067 `model-empty-text`
@id {permalink=true}: ASC067

Error · file level · next step: write · [SPEC §7.2]({repo}/blob/main/SPEC.md#72-declarations)

**Message:** `{key}` can't be empty

**Fix:** Give the key a value, or remove it.

### `[project]`

#### ASC068 `model-path-absolute`
@id {permalink=true}: ASC068

Error · file level · next step: write · [SPEC §2.2]({repo}/blob/main/SPEC.md#22-pages-and-fragments)

**Message:** `{key}` must be a path relative to ascribe.toml, not an absolute path

**Fix:** Write the path relative to the directory of `ascribe.toml`.

#### ASC069 `model-content-root-missing`
@id {permalink=true}: ASC069

Error · file level · next step: write · [SPEC §2.2]({repo}/blob/main/SPEC.md#22-pages-and-fragments)

**Message:** content root `{path}` doesn't exist

**Fix:** Create the directory, or correct `[project] content-root`.

#### ASC070 `model-output-overlaps-content`
@id {permalink=true}: ASC070

Error · file level · next step: write · [SPEC §9.4]({repo}/blob/main/SPEC.md#94-outputs)

**Message:** output directory `{output}` is inside content root `{content}`; move it outside, or builds will read their own output as source

**Fix:** Keep the output directory and the content root apart: neither inside the other, and not the same. The default output directory, `.ascribe/build`, is outside the default content root, `docs`.

### `[sources]`

#### ASC133 `model-source-path-missing`
@id {permalink=true}: ASC133

Error · file level · next step: write · [SPEC §7.3]({repo}/blob/main/SPEC.md#73-sources)

**Message:** the folder of source `{source}`, `{path}`, doesn't exist

**Fix:** Fix `path`, which is relative to the folder ascribe.toml is in, or create the folder.

#### ASC134 `model-source-outside-repository`
@id {permalink=true}: ASC134

Error · file level · next step: write · [SPEC §7.3]({repo}/blob/main/SPEC.md#73-sources)

**Message:** the folder of source `{source}`, `{path}`, is outside the git repository the project is in

**Fix:** Give the source a `path` inside the project's repository, or, for code in another repository, give it `git` instead.

#### ASC135 `model-source-remote`
@id {permalink=true}: ASC135

Error · file level · next step: write · [SPEC §7.4]({repo}/blob/main/SPEC.md#74-sources-in-another-repository)

**Message:** source `{source}` has both `path` and `git`: a source is a folder in this repository or another repository, not both

**Fix:** Give each source either `path`, a folder in this repository, or `git`, the URL of another repository, with `branch` only beside `git`. A source with `git` keeps its copies in `sources/<name>/`, which must be outside the content root.

### Content types, fields, and attributes

#### ASC071 `model-type-multiple-defaults`
@id {permalink=true}: ASC071

Error · file level · next step: choose · [SPEC §7.2]({repo}/blob/main/SPEC.md#72-declarations)

**Message:** only one content type can be the default, but `{a}` and `{b}` both set default = true

**Fix:** Keep `default = true` on one content type only.

#### ASC072 `model-type-unreachable`
@id {permalink=true}: ASC072

Error · file level · next step: write · [SPEC §7.2]({repo}/blob/main/SPEC.md#72-declarations)

**Message:** content type `{type}` has no `files` and isn't the default, so no page can use it

**Fix:** Give the type `files` patterns, make it the default type, or remove it.

#### ASC073 `model-type-title`
@id {permalink=true}: ASC073

Error · file level · next step: write · [SPEC §7.2]({repo}/blob/main/SPEC.md#72-declarations)

**Message:** content type `{type}` must declare title = "string": it's the page title, used for empty link text

**Fix:** Declare `title = "string"` in the type's `frontmatter`.

#### ASC074 `model-field-reserved`
@id {permalink=true}: ASC074

Error · file level · next step: write · [SPEC §2.1]({repo}/blob/main/SPEC.md#21-files)

**Message:** `{field}` is reserved by the Ascribe spec and every page accepts it; remove it from `[types.{type}.frontmatter]`

**Fix:** Remove the field from the type. Every page accepts it already, with the meaning the specification gives it.

#### ASC075 `model-type-syntax`
@id {permalink=true}: ASC075

Error · file level · next step: write · [SPEC §7.2]({repo}/blob/main/SPEC.md#72-declarations)

**Message:** "\{type}" isn't a valid \{kind} type: \{detail}

**Fix:** Fix the type. The types are `string`, `number`, `boolean`, `date`, `enum(a, b)`, `list(T)`, and, for attributes, `set(T)`; a `?` at the end makes one optional.

#### ASC076 `model-type-fields`
@id {permalink=true}: ASC076

Error · file level · next step: write · [SPEC §7.2]({repo}/blob/main/SPEC.md#72-declarations)

**Message:** field `{field}` is an object, so it needs `fields`

**Fix:** Give an `object` field its `fields`, or remove `fields` from a field that isn't an object.

#### ASC077 `model-enum-values`
@id {permalink=true}: ASC077

Error · file level · next step: write · [SPEC §7.2]({repo}/blob/main/SPEC.md#72-declarations)

**Message:** `{field}` has an empty enumeration

**Fix:** List the enumeration's values once each, in `enum(a, b)` or in `values`, not both.

#### ASC078 `model-set-token`
@id {permalink=true}: ASC078

Error · file level · next step: write · [SPEC §3.3]({repo}/blob/main/SPEC.md#33-attributes)

**Message:** "\{value}" can't be in a value set: members can't contain spaces or any of , | \{ } = "

**Fix:** Remove spaces and the characters `,` `|` `{` `}` `=` `"` from the value, or make the attribute a single value instead of a set.

#### ASC079 `model-default-type`
@id {permalink=true}: ASC079

Error · file level · next step: write · [SPEC §7.2]({repo}/blob/main/SPEC.md#72-declarations)

**Message:** default for `{field}` must be \{type}, but it's \{found}

**Fix:** Give a default of the field's type.

#### ASC080 `model-phrases-field-type`
@id {permalink=true}: ASC080

Error · file level · next step: write · [SPEC §5.1]({repo}/blob/main/SPEC.md#51-phrases)

**Message:** phrases = true only works on string and list(string) fields, and `{field}` is "\{type}"

**Fix:** Remove `phrases = true`, or make the field a `string` or `list(string)`.

#### ASC081 `model-pattern-syntax`
@id {permalink=true}: ASC081

Error · file level · next step: write · [SPEC §2.2]({repo}/blob/main/SPEC.md#22-pages-and-fragments)

**Message:** "\{pattern}" isn't a valid pattern: \{detail}

**Fix:** Fix the pattern. Patterns are relative to the content root, with no leading `/` and no `..` segment.

#### ASC119 `model-attribute-reserved`
@id {permalink=true}: ASC119

Error · file level · next step: write · [SPEC §7.2]({repo}/blob/main/SPEC.md#72-declarations)

**Message:** `{key}` can't be an image attribute: HTML already uses it on the \<img> element

**Fix:** Rename the attribute.

#### ASC141 `model-inline-field`
@id {permalink=true}: ASC141

Error · file level · next step: write · [SPEC §7.2]({repo}/blob/main/SPEC.md#72-declarations)

**Message:** inline = "code" only works on string fields, and `{field}` is "\{type}"

**Fix:** Remove `inline`, or set it to "code" on a string field of a content type's `frontmatter`.

#### ASC148 `model-field-role`
@id {permalink=true}: ASC148

Error · file level · next step: write · [SPEC §7.2]({repo}/blob/main/SPEC.md#72-declarations)

**Message:** role = "\{role}" only works on a \{expected} field, and `{field}` is "\{type}"

**Fix:** Give `role = "description"` to one string field of a content type's `frontmatter`, `role = "review-date"` to one date field, and `role = "owner"` to one string or enum field, or remove `role`. A type has at most one field with each role, and only a page type's top-level fields have roles.

### Dimensions, names, lifecycle states, notes, and features

#### ASC082 `model-name-multiple-roles`
@id {permalink=true}: ASC082

Error · file level · next step: write · [SPEC §7.2]({repo}/blob/main/SPEC.md#72-declarations)

**Message:** `{name}` is used as both \{role-a} and \{role-b}; a name can have only one role, so availability specs stay unambiguous

**Fix:** Rename one of them, so that each name has one role.

#### ASC083 `model-name-case`
@id {permalink=true}: ASC083

Warning · file level · next step: write · [SPEC §7.2]({repo}/blob/main/SPEC.md#72-declarations)

**Message:** `{a}` and `{b}` differ only in case; names are case-sensitive, so they're easy to confuse

**Fix:** Rename one of them, so they differ by more than case.

#### ASC084 `model-dimension-empty`
@id {permalink=true}: ASC084

Error · file level · next step: write · [SPEC §7.2]({repo}/blob/main/SPEC.md#72-declarations)

**Message:** dimension `{dimension}` has no values

**Fix:** Add the dimension's values, or remove the dimension.

#### ASC085 `model-dimension-value-duplicate`
@id {permalink=true}: ASC085

Error · file level · next step: write · [SPEC §7.2]({repo}/blob/main/SPEC.md#72-declarations)

**Message:** `{value}` appears twice in dimensions.\{dimension}.values

**Fix:** Remove the repeated value.

#### ASC086 `model-dimension-value-shared`
@id {permalink=true}: ASC086

Error · file level · next step: write · [SPEC §7.2]({repo}/blob/main/SPEC.md#72-declarations)

**Message:** `{value}` is a value of both `{a}` and `{b}`; a value can belong to only one dimension

**Fix:** Rename the value in one of the dimensions.

#### ASC087 `model-label-undeclared`
@id {permalink=true}: ASC087

Error · file level · next step: choose · [SPEC §7.2]({repo}/blob/main/SPEC.md#72-declarations)

**Message:** dimensions.\{dimension}.labels has a label for `{value}`, which isn't one of its values: \{values}

**Fix:** Correct the label's key, or add the value to the dimension's `values`.

#### ASC088 `model-versionless-undeclared`
@id {permalink=true}: ASC088

Error · file level · next step: choose · [SPEC §7.2]({repo}/blob/main/SPEC.md#72-declarations)

**Message:** dimensions.\{dimension}.versionless lists `{value}`, which isn't one of its values: \{values}

**Fix:** Correct the entry, or add the value to the dimension's `values`.

#### ASC089 `model-lifecycle-available-required`
@id {permalink=true}: ASC089

Error · file level · next step: choose · [SPEC §7.2]({repo}/blob/main/SPEC.md#72-declarations)

**Message:** new lifecycle state `{state}` must set available = true or available = false

**Fix:** Add `available = true` or `available = false` to the state.

#### ASC090 `model-lifecycle-ga-unavailable`
@id {permalink=true}: ASC090

Error · file level · next step: write · [SPEC §7.2]({repo}/blob/main/SPEC.md#72-declarations)

**Message:** `ga` must count as available: content with no lifecycle state is ga

**Fix:** Remove `available = false` from `[lifecycle.ga]`.

#### ASC091 `model-note-label-required`
@id {permalink=true}: ASC091

Error · file level · next step: write · [SPEC §4.5]({repo}/blob/main/SPEC.md#45-note)

**Message:** new note type `{type}` needs a label, such as label = "\{Type}"

**Fix:** Give the note type a `label`.

#### ASC092 `model-availability-syntax`
@id {permalink=true}: ASC092

Error · file level · next step: write · [SPEC §4.4]({repo}/blob/main/SPEC.md#44-available)

**Message:** feature `{key}`: "\{spec}" isn't a valid availability spec: \{detail}

**Fix:** Fix the feature's `available` spec. Its syntax is in the [directive reference](../reference/directives.md#availability-specs).

#### ASC093 `model-availability-unknown-name`
@id {permalink=true}: ASC093

Error · file level · next step: choose · [SPEC §4.4]({repo}/blob/main/SPEC.md#44-available)

**Message:** feature `{key}`: `{name}` isn't a declared dimension value or dimension name

**Fix:** Use a declared dimension value, dimension, or lifecycle state, or declare the name.

#### ASC094 `model-availability-versionless`
@id {permalink=true}: ASC094

Error · file level · next step: write · [SPEC §4.4]({repo}/blob/main/SPEC.md#44-available)

**Message:** feature `{key}`: `{target}` is versionless, so it takes a state but no version

**Fix:** Remove the version. A versionless target, or a dimension name, takes a state but no version.

#### ASC095 `model-availability-history-order`
@id {permalink=true}: ASC095

Error · file level · next step: write · [SPEC §4.4]({repo}/blob/main/SPEC.md#44-available)

**Message:** feature `{key}`: the history for `{target}` must be in chronological order, but \{later} comes before \{earlier}

**Fix:** List the target's states oldest first.

#### ASC096 `model-feature-nested`
@id {permalink=true}: ASC096

Error · file level · next step: write · [SPEC §4.4]({repo}/blob/main/SPEC.md#44-available)

**Message:** feature `{key}`: available must be an availability spec, not another feature (`{other}`)

**Fix:** Write the feature's availability out in full. One feature can't refer to another.

### Phrases and the glossary

#### ASC097 `model-phrase-value-type`
@id {permalink=true}: ASC097

Error · file level · next step: write · [SPEC §5.1]({repo}/blob/main/SPEC.md#51-phrases)

**Message:** phrase `{key}` must be a quoted string, but it's \{found}; write \{key} = "\{value}"

**Fix:** Quote the value.

#### ASC098 `model-glossary-duplicate-term`
@id {permalink=true}: ASC098

Error · file level · next step: write · [SPEC §5.4]({repo}/blob/main/SPEC.md#54-glossary-terms)

**Message:** "\{text}" is declared by both glossary terms `{a}` and `{b}`

**Fix:** Remove the text from one of the terms, or make the terms distinct.

#### ASC099 `model-glossary-link`
@id {permalink=true}: ASC099

Error · file level · next step: choose · [SPEC §5.4]({repo}/blob/main/SPEC.md#54-glossary-terms)

**Message:** glossary term `{id}` links to \{path}, which doesn't exist

**Fix:** Fix the link, or link to a page that includes the fragment.

### Widgets

#### ASC100 `model-widget-reserved-name`
@id {permalink=true}: ASC100

Error · file level · next step: write · [SPEC §6]({repo}/blob/main/SPEC.md#6-project-widgets)

**Message:** widget name `{name}` is reserved: names starting with ascribe- belong to Ascribe's element library

**Fix:** Rename the widget, for example with your project's name as its prefix: `quill-aside`.

#### ASC101 `model-widget-forms`
@id {permalink=true}: ASC101

Error · file level · next step: choose · [SPEC §6]({repo}/blob/main/SPEC.md#6-project-widgets)

**Message:** forms must be ["line"], ["container"], or ["line", "container"]

**Fix:** Set `forms` to `["line"]`, `["container"]`, or `["line", "container"]`.

#### ASC102 `model-widget-binding`
@id {permalink=true}: ASC102

Error · file level · next step: choose · [SPEC §6]({repo}/blob/main/SPEC.md#6-project-widgets)

**Message:** widget `{name}` has a line form, so it needs a binding: "self", "heading", "block", or "heading-or-block"

**Fix:** Give a widget with a line form a `binding`. Remove `binding` from a container-only widget.

#### ASC103 `model-widget-container-primary`
@id {permalink=true}: ASC103

Error · file level · next step: write · [SPEC §6]({repo}/blob/main/SPEC.md#6-project-widgets)

**Message:** widget `{name}` has a container form, whose opener has no primary, so its primary can't be required; use "\{kind}?"

**Fix:** Make the primary optional (`"text?"`), or remove the widget's container form. A container-only widget takes no primary.

#### ASC104 `model-widget-groupable-form`
@id {permalink=true}: ASC104

Error · file level · next step: write · [SPEC §6]({repo}/blob/main/SPEC.md#6-project-widgets)

**Message:** widget `{name}` is groupable, so it must be container-only: forms = ["container"]

**Fix:** Set `forms = ["container"]`, or remove `groupable`.

#### ASC105 `model-widget-plain-content`
@id {permalink=true}: ASC105

Error · file level · next step: write · [SPEC §6]({repo}/blob/main/SPEC.md#6-project-widgets)

**Message:** widget `{name}` doesn't wrap content, so plain-content has no effect; remove it

**Fix:** Remove `plain-content`.

### The consumer, builds, and the editor

#### ASC106 `model-consumer-unsupported`
@id {permalink=true}: ASC106

Error · file level · next step: choose · [SPEC §9.5]({repo}/blob/main/SPEC.md#95-consumer-profile)

**Message:** the \{profile} profile doesn't support \{key} = \{value}; use \{values}

**Fix:** Use one of the values the message lists.

#### ASC107 `model-consumer-site`
@id {permalink=true}: ASC107

Error · file level · next step: write · [SPEC §9.5]({repo}/blob/main/SPEC.md#95-consumer-profile)

**Message:** site must be an origin such as "https://docs.example.com"; put any path in base-path

**Fix:** Set `site` to the origin alone, such as `"https://docs.example.com"`, and put any path in `base-path`.

#### ASC108 `model-consumer-base-path`
@id {permalink=true}: ASC108

Error · file level · next step: write · [SPEC §9.5]({repo}/blob/main/SPEC.md#95-consumer-profile)

**Message:** base-path must start with "/", such as "/docs/"

**Fix:** Start `base-path` with `/`: `"/docs/"`.

#### ASC109 `model-build-name-case`
@id {permalink=true}: ASC109

Error · file level · next step: write · [SPEC §9.3]({repo}/blob/main/SPEC.md#93-build-modes)

**Message:** builds `{a}` and `{b}` differ only in case, so they'd share an output directory on some file systems

**Fix:** Rename one of the builds.

#### ASC110 `model-build-variants`
@id {permalink=true}: ASC110

Error · file level · next step: write · [SPEC §9.3]({repo}/blob/main/SPEC.md#93-build-modes)

**Message:** variants must be "switch" or a selection such as \{ deployment = "cloud" }

**Fix:** Set `variants` to `"switch"`, or to a selection that names at least one dimension.

#### ASC111 `model-build-unknown-dimension`
@id {permalink=true}: ASC111

Error · file level · next step: choose · [SPEC §9.3]({repo}/blob/main/SPEC.md#93-build-modes)

**Message:** build `{build}` selects dimension `{dimension}`, which isn't declared

**Fix:** Correct the dimension's name, or declare the dimension in `[dimensions]`.

#### ASC112 `model-build-unknown-value`
@id {permalink=true}: ASC112

Error · file level · next step: choose · [SPEC §9.3]({repo}/blob/main/SPEC.md#93-build-modes)

**Message:** build `{build}`: `{value}` isn't a value of `{dimension}`; values: \{values}

**Fix:** Use one of the values the message lists.

#### ASC113 `model-build-availability`
@id {permalink=true}: ASC113

Error · file level · next step: write · [SPEC §9.3]({repo}/blob/main/SPEC.md#93-build-modes)

**Message:** availability must be "badge" or \{ filter = "\<target> \<version>" }

**Fix:** Set `availability` to `"badge"`, or to `{ filter = "<target> <version>" }`.

#### ASC114 `model-build-filter-target`
@id {permalink=true}: ASC114

Error · file level · next step: choose · [SPEC §9.3]({repo}/blob/main/SPEC.md#93-build-modes)

**Message:** build `{build}` filters for `{target}`, which isn't a declared dimension value

**Fix:** Filter for a declared dimension value, not a dimension name.

#### ASC115 `model-build-filter-version`
@id {permalink=true}: ASC115

Error · file level · next step: write · [SPEC §9.3]({repo}/blob/main/SPEC.md#93-build-modes)

**Message:** build `{build}` filters for `{target}`, which is versioned, so it needs a version, such as "\{target} 3.3"

**Fix:** Give a versioned target a version (`"self-managed 3.3"`), and a versionless target none.

#### ASC116 `model-build-filter-excluded`
@id {permalink=true}: ASC116

Warning · file level · next step: write · [SPEC §9.3]({repo}/blob/main/SPEC.md#93-build-modes)

**Message:** build `{build}` filters for `{target}`, but its selection keeps only \{dimension} = \{values}, so pages marked for `{target}` are dropped

**Fix:** Make the build's `variants` selection keep the target it filters for, or filter for a target it keeps.

#### ASC117 `model-editor-build-unknown`
@id {permalink=true}: ASC117

Error · file level · next step: choose · [SPEC §10]({repo}/blob/main/SPEC.md#10-authoring-environment)

**Message:** editor.build is `{build}`, which isn't a declared build; builds: \{builds}

**Fix:** Set `[editor] build` to one of the builds the message lists.

#### ASC118 `model-editor-build-required`
@id {permalink=true}: ASC118

Error · file level · next step: choose · [SPEC §10]({repo}/blob/main/SPEC.md#10-authoring-environment)

**Message:** there are several builds and none is named site; set [editor] build to the one the editor should check

**Fix:** Set `[editor] build` to the build the editor should check, or name one of the builds `site`.

#### ASC162 `model-consumer-agents-site`
@id {permalink=true}: ASC162

Error · file level · next step: write · [SPEC §9.5]({repo}/blob/main/SPEC.md#95-consumer-profile)

**Message:** agents = true needs `site`: every link in llms.txt and the Markdown pages is an absolute URL

**Fix:** Set `site` in `[consumer]` to the published site's origin, such as `"https://docs.example.com"`, or remove `agents`.

#### ASC174 `model-consumer-permalink-path`
@id {permalink=true}: ASC174

Error · file level · next step: write · [SPEC §9.5]({repo}/blob/main/SPEC.md#95-consumer-profile)

**Message:** permalink-path must start with "/", such as "/go/"

**Fix:** Start `permalink-path` with `/`: `"/go/"`. It's a path from the site's root, not from `base-path`.

### `[permalinks]`

#### ASC178 `model-permalinks`
@id {permalink=true}: ASC178

Error · file level · next step: write · [SPEC §7.6]({repo}/blob/main/SPEC.md#76-retired-permalinks-and-aliases)

**Message:** `{name}` is both retired and an alias; a name is one thing

**Fix:** Give each name in `[permalinks]` one meaning: retired, in `[permalinks.retired]`, or an alias, in `[permalinks.aliases]`. A retired permalink can still lead somewhere: give it `to`. Make each alias lead, perhaps through other aliases, to a permalink a heading or page declares. Write the reason a permalink was retired in place of the placeholder the editor's quick fix writes: a reader whose link led there sees it.

### `[checks]`

#### ASC142 `model-check-not-configurable`
@id {permalink=true}: ASC142

Error · file level · next step: write · [SPEC §8.2]({repo}/blob/main/SPEC.md#82-diagnostics)

**Message:** `{check}` can't be set in [checks]: it reports whether the project is valid, which a project can't lower or turn off

**Fix:** Remove the check from `[checks]`. Only checks about the content's quality, which the [diagnostics reference](../reference/diagnostics.md) marks as configurable, can be set there; a diagnostic about whether the project is valid is always reported at its own severity.

#### ASC146 `model-intended-check`
@id {permalink=true}: ASC146

Error · file level · next step: choose · [SPEC §4.9]({repo}/blob/main/SPEC.md#49-intended)

**Message:** `{check}` isn't a check

**Fix:** Name a check whose next step is review and whose problems are about a phrase, feature, glossary term, or image, by its name in the [diagnostics reference](../reference/diagnostics.md). A page's or a block's problem is acknowledged in the page itself.

#### ASC147 `model-intended-entry`
@id {permalink=true}: ASC147

Error · file level · next step: choose · [SPEC §4.9]({repo}/blob/main/SPEC.md#49-intended)

**Message:** an acknowledgement in [[intended]] names what it's about with one of `phrase`, `feature`, `term`, or `image`

**Fix:** Give the acknowledgement exactly one of `phrase`, `feature`, `term` (a glossary term's id), or `image` (a path under the content root), naming something the content model declares, and a reason of your own in place of the placeholder the editor's quick fix writes.

#### ASC167 `model-checks-vale`
@id {permalink=true}: ASC167

Error · file level · next step: write · [SPEC §8.2]({repo}/blob/main/SPEC.md#82-diagnostics)

**Message:** `[checks.vale]` has both `preset` and `config`; pick one: Ascribe's preset, or the project's own `.vale.ini`

**Fix:** Give `[checks.vale]` one of `preset = "quiet"` or `config = "<path to .vale.ini>"`. `off` lists rules of the preset to turn off, by their names in alerts (`Ascribe.Repeated`), and is only for a preset.
