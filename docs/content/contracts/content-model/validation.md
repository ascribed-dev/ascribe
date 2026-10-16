---
type: reference
title: "`ascribe.toml` format contract: validation rules"
description: "Every rule a content model loader enforces, the map of SPEC §7.2 to the contract, and the decisions behind it."
---

Part of the [`ascribe.toml` format contract](../content-model.md), whose [contents](../content-model.md#contents) list every section.

## 24. Validation rules

A loader MUST enforce every rule below when it loads `ascribe.toml`, and report each violation at the span of the offending key or value. Every rule is an error unless marked **warning**. A model with errors doesn't load, and no document is checked against it; warnings don't stop loading.

- **Slugs** are stable identifiers, for tests and for the diagnostics registry, [`tests/conformance/diagnostics.toml`]({repo}/blob/main/tests/conformance/diagnostics.toml), which gives each one a code. Only `model-name-multiple-roles` corresponds to a row of SPEC §8.2 ("Content model"); the rest are loader rules this reference adds.
- **Messages** are templates. `{name}` is a placeholder. Where a rule has several messages, each covers one case of it.
- Rules about documents, such as a page matching no content type or an unknown frontmatter key, aren't loader rules.
- **Filesystem rules** (`model-content-root-missing`, `model-glossary-link`, `model-source-path-missing`, `model-source-outside-repository`, and `model-listing-folder` for a folder that isn't there) need the project directory. A loader given only the file's text, such as an unsaved editor buffer or a unit test, skips them; `ascribe check` and the language server always run them.

### 24.1 File and structure

| Slug | Rule | Message |
|---|---|---|
| `model-toml-syntax` | The file is valid TOML 1.0. | `` ascribe.toml isn't valid TOML: {detail} `` |
| `model-unknown-key` | Every key is one this reference defines for its table, except in tables keyed by names the project chooses. | `` unknown key `{key}` in `[{table}]` ``<br>`` unknown key `{key}` in `[{table}]`; did you mean `{suggestion}`? ``<br>`` `{key}` isn't a key of `[{table}]`: a page says what type it is in its frontmatter, with `type: {type}` ``, for `files` and `default` in a type's table |
| `model-missing-key` | Every required key is present. | `` `[{table}]` is missing the required key `{key}` `` |
| `model-wrong-type` | Every value has the TOML type its key requires. | `` `{key}` must be {expected}, but it's {found} ``<br>`` `{key}` must be a string, but it's a number; quote it: {key} = "{value}" `` |
| `model-invalid-value` | A key with a fixed set of values has one of them. | `` `{key}` can't be "{value}"; use one of: {values} `` |
| `model-spec-unsupported` | `spec` is a version this processor implements. | `` ascribe.toml targets spec version "{spec}", but this processor implements {supported} `` |
| `model-invalid-name` | Every name follows its grammar (§1.2). | `` `{name}` isn't a valid {role} name: {rule} ``, for example `` `Deployment` isn't a valid dimension name: use a lowercase letter, then lowercase letters, digits, or hyphens `` |
| `model-empty-text` | Every `label`, `name`, `term`, `definition`, and `description` is non-empty. | `` `{key}` can't be empty `` |

### 24.2 Project

| Slug | Rule | Message |
|---|---|---|
| `model-path-absolute` | `content-root`, `output-dir`, and each source's `path` are relative paths. | `` `{key}` must be a path relative to ascribe.toml, not an absolute path `` |
| `model-content-root-missing` | The content root exists and is a directory. | `` content root `{path}` doesn't exist ``<br>`` content root `{path}` isn't a directory `` |
| `model-output-overlaps-content` | The output directory isn't inside the content root, the content root isn't inside the output directory, and they differ (derived from the spec). | `` output directory `{output}` is inside content root `{content}`; move it outside, or builds will read their own output as source ``<br>`` content root `{content}` is inside output directory `{output}`; builds could delete source files as stale output ``<br>`` output directory and content root are both `{path}` `` |

### 24.3 Content types, fragments, and fields

| Slug | Rule | Message |
|---|---|---|
| `model-type-title` | Every page type declares `title` as a required `string`. | `` content type `{type}` must declare title = "string": it's the page title, used for empty link text ``<br>`` content type `{type}`: `title` must be a required string, not "{found}" `` |
| `model-type-file` | Each file a type's `template` and `guide` name is there, with that exact name, inside the folder `ascribe.toml` is in once symbolic links are followed, and outside the content root and the output directory. | `` `{path}`, the {key} of content type `{type}`, doesn't exist ``<br>`` `{path}`, the {key} of content type `{type}`, doesn't exist; `{actual}` differs from it only in case ``<br>`` `{path}`, the {key} of content type `{type}`, is inside the content root `{content}`, where it would be read as a page ``<br>`` `{path}`, the {key} of content type `{type}`, is inside the output directory `{output}`, which a build writes and removes ``<br>`` `{path}`, the {key} of content type `{type}`, is outside the folder ascribe.toml is in ``<br>`` `{path}`, the {key} of content type `{type}`, leads through a symbolic link to a file outside the folder ascribe.toml is in `` |
| `model-type-template` | A type's template can be read, and reads as a page: its frontmatter is valid YAML, and its Ascribe syntax has no error. | `` can't read `{path}`, the template of content type `{type}`: {reason} ``<br>`` `{path}`, the template of content type `{type}`, isn't a page Ascribe can read: on its line {line}, {detail} `` |
| `model-field-reserved` | No content type or fragment schema declares `type`, `available`, `variant`, `permalink`, or `listing`, and no content type declares `slug`, `formatted`, or `source`. | `` `{field}` is reserved by the Ascribe spec and every page accepts it; remove it from `[types.{type}.frontmatter]` ``<br>`` `{field}` is reserved by the Ascribe spec, and fragments can't use it ``<br>`` `slug` is reserved by the astro profile, which uses it as a page's URL id; remove it from `[types.{type}.frontmatter]` ``<br>`` `formatted` is reserved by the site output, which writes the formatted form of `inline` fields under it; remove it from `[types.{type}.frontmatter]` ``<br>`` `source` is reserved by the site output, which writes each page's path from the content root under it; remove it from `[types.{type}.frontmatter]` `` |
| `model-type-syntax` | Every field and attribute type parses under §5.1, and is allowed where it's used: no `date`, `list`, or `object` for attributes; no `set` for fields; `object`, `list(object)`, and bare `enum` only in table form. | `` "{type}" isn't a valid {kind} type: {detail} ``, for example `` "strng?" isn't a valid field type: expected string, number, boolean, date, enum(…), or list(…) `` |
| `model-type-fields` | `fields` is present exactly when the type is `object` or `list(object)`, optional or not. | `` field `{field}` is an object, so it needs `fields` ``<br>`` `fields` is only allowed on object fields, and `{field}` is "{type}" `` |
| `model-enum-values` | An enumeration has at least one value and no duplicates, and its values come from exactly one of `enum(…)` and `values`. `values` appears only with bare `enum`. | `` `{field}` has an empty enumeration ``<br>`` "{value}" appears twice in the enumeration for `{field}` ``<br>`` `{field}`: list enumeration values in enum(…) or in `values`, not both ``<br>`` `{field}` is a bare enum, so it needs `values` `` |
| `model-set-token` | Every value of a `set(enum)` attribute is a token (SPEC §3.3). | `` "{value}" can't be in a value set: members can't contain spaces or any of , \| { } = " `` |
| `model-default-type` | A `default` has the declared type, and an enumeration default is one of its values. | `` default for `{field}` must be {type}, but it's {found} ``<br>`` default "{value}" for `{field}` isn't one of: {values} `` |
| `model-phrases-field-type` | `phrases = true` is set only on `string` and `list(string)` fields. | `` phrases = true only works on string and list(string) fields, and `{field}` is "{type}" `` |
| `model-inline-field` | `inline` is `"code"`, and is set only on a content type's top-level `string` fields (§5.3). | `` inline = "code" only works on string fields, and `{field}` is "{type}" ``<br>`` inline must be "code", not "{value}" ``<br>`` inline = "code" only works on a content type's top-level fields, and `{field}` is in {place} `` |
| `model-pattern-syntax` | Every pattern parses under §1.3, doesn't start with `/`, and has no `..` segment. | `` "{pattern}" isn't a valid pattern: {detail} ``<br>`` pattern "{pattern}" is already relative to the content root; remove the leading / ``<br>`` pattern "{pattern}" can't contain .. `` |
| `model-field-role` | `role` is `"description"`, on a `string` field, `"review-date"`, on a `date` field, or `"owner"`, on a `string` or `enum` field; only on a content type's top-level fields; and a type has at most one field with each role (§5.4). | `` role = "{role}" only works on a {expected} field, and `{field}` is "{type}" ``<br>`` role must be "description", "review-date", or "owner", not "{value}" ``<br>`` role only works on a content type's top-level fields, and `{field}` is in {place} ``<br>`` `{other}` already has role = "{role}"; a content type has one field with each role `` |
| `model-attribute-reserved` | No image or widget attribute key is one HTML already gives a meaning on that element (SPEC §7.2): `src`, `alt`, or `title` on images; `heading` or `primary` on widgets; and on both, HTML's global attributes (such as `id`, `class`, `style`, and `title`), any key starting with `aria-`, and HTML's event-handler attributes (such as `onclick`, `onload`, and `onerror`). The lists are explicit, in `ascribe_core::reserved`, so keys that merely begin with `on`, such as `online` or `only-if`, are allowed. | `` `{key}` can't be an image attribute: HTML already uses it on the <img> element ``<br>`` `{key}` can't be an attribute of widget `{name}`: the site output already uses it on the widget's element `` |

### 24.4 Dimensions, names, lifecycle, notes, and features

| Slug | Rule | Message |
|---|---|---|
| `model-name-multiple-roles` | A name is used in at most one of these roles: dimension name, dimension value, lifecycle state (including built-in states), feature key (SPEC §7.2, §8.2). | `` `{name}` is used as both {role-a} and {role-b}; a name can have only one role, so availability specs stay unambiguous ``, for example `` `beta` is used as both a dimension value (in dimensions.channel) and a lifecycle state (built in); a name can have only one role, so availability specs stay unambiguous `` |
| `model-name-case` (**warning**) | No two names in those roles differ only in case. | `` `{a}` and `{b}` differ only in case; names are case-sensitive, so they're easy to confuse `` |
| `model-dimension-empty` | `values` has at least one value. | `` dimension `{dimension}` has no values `` |
| `model-dimension-value-duplicate` | No value appears twice in one dimension. | `` `{value}` appears twice in dimensions.{dimension}.values `` |
| `model-dimension-value-shared` | No value belongs to two dimensions. | `` `{value}` is a value of both `{a}` and `{b}`; a value can belong to only one dimension `` |
| `model-label-undeclared` | Every key in a dimension's `labels` is one of its values. | `` dimensions.{dimension}.labels has a label for `{value}`, which isn't one of its values: {values} `` |
| `model-versionless-undeclared` | Every entry in a dimension's `versionless` is one of its values. | `` dimensions.{dimension}.versionless lists `{value}`, which isn't one of its values: {values} `` |
| `model-lifecycle-available-required` | A new lifecycle state sets `available`. | `` new lifecycle state `{state}` must set available = true or available = false `` |
| `model-lifecycle-ga-unavailable` | `ga` counts as available. | `` `ga` must count as available: content with no lifecycle state is ga `` |
| `model-note-label-required` | A new note type sets `label`. | `` new note type `{type}` needs a label, such as label = "{Type}" `` |
| `model-availability-syntax` | A feature's `available` parses as an availability spec (SPEC Appendix A `availability`). | `` feature `{key}`: "{spec}" isn't a valid availability spec: {detail} `` |
| `model-availability-unknown-name` | Every target in a feature's spec is a declared dimension value or dimension name, and every state is a declared lifecycle state. | `` feature `{key}`: `{name}` isn't a declared dimension value or dimension name ``<br>`` feature `{key}`: `{name}` isn't a declared lifecycle state `` |
| `model-availability-versionless` | A versionless target, or a dimension name, has no versions in a feature's spec (SPEC §4.4). | `` feature `{key}`: `{target}` is versionless, so it takes a state but no version ``<br>`` feature `{key}`: `{target}` is a dimension name, so it takes a state but no version; name one of its values, such as `{example}`, to give a version `` |
| `model-availability-history-order` | A history in a feature's spec is in chronological order under the version scheme. | `` feature `{key}`: the history for `{target}` must be in chronological order, but {later} comes before {earlier} `` |
| `model-feature-nested` | A feature's `available` isn't itself a feature key. | `` feature `{key}`: available must be an availability spec, not another feature (`{other}`) `` |

### 24.5 Versions, phrases, glossary, and images

| Slug | Rule | Message |
|---|---|---|
| `model-phrase-value-type` | Every phrase value is a string. | `` phrase `{key}` must be a quoted string, but it's {found}; write {key} = "{value}" `` |
| `model-glossary-duplicate-term` | No two terms or aliases are the same text, ignoring case when either is case-insensitive. | `` "{text}" is declared by both glossary terms `{a}` and `{b}` `` |
| `model-glossary-link` | A term's `link` file exists under the content root and isn't a fragment. | `` glossary term `{id}` links to {path}, which doesn't exist ``<br>`` glossary term `{id}` links to {path}, which is a fragment; link to a page that includes it `` |

`[versions] scheme` is covered by `model-invalid-value`, and `[versions] current`, which isn't a key, by `model-versions-current`. Image attribute types are covered by the rules in §24.3.

### 24.6 Widgets

| Slug | Rule | Message |
|---|---|---|
| `model-widget-reserved-name` | The name doesn't start with `ascribe-` and isn't one of HTML's reserved custom-element names. | `` widget name `{name}` is reserved: names starting with ascribe- belong to Ascribe's element library ``<br>`` widget name `{name}` is reserved by HTML and can't be a custom element `` |
| `model-widget-forms` | `forms` is non-empty, has no duplicates, and contains only `"line"` and `"container"`. | `` forms must be ["line"], ["container"], or ["line", "container"] `` |
| `model-widget-binding` | `binding` is present when `forms` includes `"line"`, and absent otherwise. | `` widget `{name}` has a line form, so it needs a binding: "self", "heading", "block", or "heading-or-block" ``<br>`` widget `{name}` is container-only, and a container holds its own content; remove binding `` |
| `model-widget-container-primary` | With container form, the primary isn't required; a container-only widget's primary is `"none"`. | `` widget `{name}` has a container form, whose opener has no primary, so its primary can't be required; use "{kind}?" ``<br>`` widget `{name}` is container-only, so it can't take a primary `` |
| `model-widget-groupable-form` | A groupable widget is container-only. | `` widget `{name}` is groupable, so it must be container-only: forms = ["container"] `` |
| `model-widget-plain-content` | `plain-content` is set only on widgets that wrap content: container form, or binding `block` or `heading-or-block`. | `` widget `{name}` doesn't wrap content, so plain-content has no effect; remove it `` |

`primary`, `title`, and `binding` values are covered by `model-invalid-value`; attribute types by §24.3.

### 24.7 Consumer, builds, and editor

| Slug | Rule | Message |
|---|---|---|
| `model-consumer-unsupported` | Every consumer key has a value the profile supports (for example, `html = false` with `astro`). | `` the {profile} profile doesn't support {key} = {value}; use {values} `` |
| `model-consumer-site` | `site` is an absolute `http` or `https` URL with no path, query, or fragment. | `` site must be an origin such as "https://docs.example.com"; put any path in base-path `` |
| `model-consumer-base-path` | `base-path` starts with `/`. | `` base-path must start with "/", such as "/docs/" `` |
| `model-consumer-agents-site` | `agents = true` comes with `site`. | `` agents = true needs `site`: every link in llms.txt and the Markdown pages is an absolute URL `` |
| `model-build-name-case` | Build names are unique ignoring case. | `` builds `{a}` and `{b}` differ only in case, so they'd share an output directory on some file systems `` |
| `model-build-variants` | `variants` is `"switch"` or a non-empty table of dimension selections, each a value or a non-empty array of values. | `` variants must be "switch" or a selection such as { deployment = "cloud" } ``<br>`` build `{build}` selects nothing; write variants = "switch" `` |
| `model-build-unknown-dimension` | Every dimension a selection names is declared. | `` build `{build}` selects dimension `{dimension}`, which isn't declared `` |
| `model-build-unknown-value` | Every selected value is a value of its dimension. | `` build `{build}`: `{value}` isn't a value of `{dimension}`; values: {values} `` |
| `model-build-availability` | `availability` is `"badge"` or a table with exactly one key, `filter`. | `` availability must be "badge" or { filter = "<target> <version>" } `` |
| `model-build-filter-target` | The filter's target is a declared dimension value. | `` build `{build}` filters for `{target}`, which isn't a declared dimension value ``<br>`` build `{build}` filters for `{target}`, which is a dimension; filter for one of its values: {values} `` |
| `model-build-filter-version` | The filter has a version exactly when its target is versioned, and the version follows SPEC Appendix A's `version` rule. | `` build `{build}` filters for `{target}`, which is versioned, so it needs a version, such as "{target} 3.3" ``<br>`` build `{build}`: `{target}` is versionless, so the filter can't name a version ``<br>`` build `{build}`: "{version}" isn't a valid version `` |
| `model-build-filter-excluded` (**warning**) | A build's filter target isn't a value its own selection drops. | `` build `{build}` filters for `{target}`, but its selection keeps only {dimension} = {values}, so pages marked for `{target}` are dropped `` |
| `model-editor-build-unknown` | `editor.build` names a declared build. | `` editor.build is `{build}`, which isn't a declared build; builds: {builds} `` |
| `model-editor-build-required` | When there are several builds and none is named `site`, `editor.build` is set. | `` there are several builds and none is named site; set [editor] build to the one the editor should check `` |

### 24.8 Sources
@available: ascribe 0.2.0

| Slug | Rule | Message |
|---|---|---|
| `model-source-path-missing` | A source's `path` exists and is a directory. | `` the folder of source `{source}`, `{path}`, doesn't exist ``<br>`` the folder of source `{source}`, `{path}`, isn't a directory `` |
| `model-source-outside-repository` | A source's folder is inside the git repository the project is in, when it's in one; with none, in the language server and `ascribe mcp`, inside what the user opened (variant `opened`). | `` the folder of source `{source}`, `{path}`, is outside the git repository the project is in `` |
| `model-source-remote` | A source has `path` or `git`, not both; `branch` only with `git`; a URL `git` can be given; a branch name `git` accepts; and, with `git`, copies outside the content root. | `` source `{source}` has both `path` and `git`: a source is a folder in this repository or another repository, not both ``<br>`` source `{source}` needs `path`, a folder in this repository, or `git`, another repository's URL ``<br>`` source `{source}` has `branch` but no `git`: a branch is only for a source in another repository ``<br>`` `{url}` isn't a URL Ascribe can give `git`: use `https://`, `http://`, `ssh://`, `git://`, `file://`, or `user@host:path` ``<br>`` `{branch}` isn't a branch name `git` accepts ``<br>`` the copies of source `{source}` go in `sources/{source}/`, which is inside the content root `{content}`, where they'd be read as pages; give the content root a folder of its own `` |

### 24.9 Checks
@available: ascribe 0.3.0

| Slug | Rule | Message |
|---|---|---|
| `model-check-not-configurable` | `[checks]` names only checks the diagnostics registry marks configurable. | `` `{check}` can't be set in [checks]: it reports whether the project is valid, which a project can't lower or turn off `` |
| `model-intended-check` | `[[intended]]` names a review check that's acknowledged at a content model entry. | `` `{check}` isn't a check `` |
| `model-intended-entry` | `[[intended]]` names exactly one declared phrase, feature, glossary term, or image. | `` an acknowledgement in [[intended]] names what it's about with one of `phrase`, `feature`, `term`, or `image` `` |
| `model-checks-vale` | `[checks.vale]` has exactly one of `preset` and `config`, and `off` goes with `preset` and names its rules. | `` `[checks.vale]` has both `preset` and `config`; pick one: Ascribe's preset, or the project's own `.vale.ini` `` |

### 24.10 Permalinks
@available: ascribe 0.3.0

| Slug | Rule | Message |
|---|---|---|
| `model-permalinks` | A name in `[permalinks]` is retired or an alias, not both; an alias leads, through aliases, to a name that isn't one; a retired permalink's reason isn't the quick fix's placeholder. | `` `{name}` is both retired and an alias; a name is one thing `` |

### 24.11 Crawlers
@available: ascribe 0.3.0

| Slug | Rule | Message |
|---|---|---|
| `model-crawlers-line` | Each of `[crawlers] lines` is one line, with no line break. | `` `{key}` has a line break; each string in `lines` is one line of robots.txt `` |
| `model-crawlers-named-twice` | A crawler is named in one group: not by a disallowed purpose and a group of `lines`, nor by two groups of `lines`. | `` `{key}` names `{token}`, which a group on line {line} names already; a crawler obeys only one group, so write everything for it in one `` |

### 24.12 Listing
@available: ascribe 0.3.0

| Slug | Rule | Message |
|---|---|---|
| `model-listing-folder` | A key of `[listing.folders]` is a folder's path from the content root, ending in `/`, of a folder that exists by its exact name. | `` `{folder}` in `[listing.folders]` doesn't exist under the content root ``<br>`` `{folder}` in `[listing.folders]` isn't a folder ``<br>`` `{folder}` in `[listing.folders]` isn't a folder's path; write it from the content root, with `/` between its parts and at its end, such as `internal/` `` |
| `model-listing-type` | A key of `[listing.types]` is a content type. | `` `{type}` in `[listing.types]` isn't a content type; use one of: {values} `` |
| `model-listing-looser` | No rule is looser than a folder's it's under, or than `default`. | `` `{rule}` is looser than `{wider}` on line {line}, which it's under; a narrower rule can only restrict ``<br>`` `{rule}` is looser than the site's `{wider}` on line {line}; with that default, no folder or type can be listed more openly `` |

@available: next
`model-listing-looser` holds `older-lines` to `default` too.

---

## 25. Map of SPEC §7.2 to this file

Every declaration SPEC §7.2 lists, and every setting SPEC §9.3 and §9.5 need, maps to one section:

| SPEC declaration | Section of `ascribe.toml` | Here |
|---|---|---|
| Spec version (§11) | `spec` | [§2](../content-model.md#2-top-level-spec) |
| Content root (§2.2); output directory (§9.4) | `[project]` | [§3](../content-model.md#3-project) |
| Content types: page schemas and the fragment schema (§2.1, §2.2) | `[types.<name>]`, `[fragments.frontmatter]` | [§4](../content-model.md#4-typesname-and-fragments) |
| Field roles (§7.2) | The `role` field option | [§5.4](../content-model.md#54-field-roles) |
| Fragment patterns (§2.2) | `[fragments] patterns` | [§4.3](../content-model.md#43-fragments) |
| Field types (§7.2) | Field type syntax | [§5](../content-model.md#5-field-and-attribute-types) |
| Directive schemas for project widgets (§3, §6) | `[widgets.<name>]` | [§14](content.md#14-widgetsname) |
| Dimensions: names, values, display labels, versionless values (§4.3, §4.4) | `[dimensions.<name>]` | [§6](content.md#6-dimensionsname) |
| Version scheme (§4.4) | `[versions]` | [§7](content.md#7-versions) |
| Lifecycle states (§4.4) | `[lifecycle.<state>]` | [§8](content.md#8-lifecyclestate) |
| Features registry (§4.4) | `[features.<key>]` | [§9](content.md#9-featureskey) |
| Note types (§4.5) | `[notes.<type>]` | [§10](content.md#10-notestype) |
| Phrases registry; phrases in frontmatter (§5.1) | `[phrases]`; the `phrases` field option | [§11](content.md#11-phrases) |
| Glossary (§5.4) | `[glossary]` | [§12](content.md#12-glossary) |
| Image attributes (§5.3) | `[images.attributes]` | [§13](content.md#13-images) |
| Consumer profile (§9.5) | `[consumer]` | [§15](output.md#15-consumer) |
| Builds (§9.3) | `[builds.<name>]` | [§16](output.md#16-buildsname) |
| Name roles (§7.2, last paragraph) | Validation rule `model-name-multiple-roles` | [§24](#24-validation-rules) |
| The editor's default build | `[editor]` | [§17](output.md#17-editor) |
| Sources: the folders outside the content a snippet may read (§4.8, §7.3) | `[sources.<name>]` | [§18](output.md#18-sourcesname) |
| Acknowledgements of a content model entry's or an image's problems (§4.9) | `[[intended]]` | [§19.1](checks.md#191-intended) |
| Retired permalinks and aliases (§7.6) | `[permalinks]` | [§20](checks.md#20-permalinks) |
| The project's own `robots.txt` lines (§7.2, §9.4) | `[crawlers]` | [§21](checks.md#21-crawlers) |
| How the site's, a folder's, or a type's pages are listed (§2.1, §7.2) | `[listing]` | [§22](checks.md#22-listing) |

---

## 26. Decisions

Each item settles a gap in SPEC.md. All 21 were decided on 2026-09-28 as recommended below, except item 12, which was settled when the site-render and asset contracts were written. Items 1, 2, 3, 4, and 6 are now also stated in SPEC.md (§2.1, §5.2, §7.2, §8.2). Each item keeps the alternatives that were considered.

1. **Frontmatter diagnostics (SPEC §8.2).** §8.1 says file-level validation covers frontmatter, but §8.2 has no rows for it. *Decision:* add file-level error rows: unknown frontmatter key; missing required field; value doesn't match the field's type; reserved key (`available`, `variant`) on a fragment; page matches more than one content type; page matches no content type and there's no default.
2. **Assigning content types to pages (SPEC §7.2).** The spec doesn't say how a page gets its type. ~~*Decision:* `files` patterns per type plus at most one `default = true` type; a page matching several types is an error, with no precedence.~~ Replaced by [decision #450]({repo}/blob/main/project-docs/decisions/450-page-type-key.md): a page names its type with the reserved frontmatter key `type`. *Considered:* first match in file order (TOML tables are formally unordered); most specific pattern (hard to define); a frontmatter `type` key (would need a new reserved key).
3. **Page titles (SPEC §5.2).** Empty link text uses "the page title", but the spec doesn't say where it comes from. *Decision:* the frontmatter `title`, which every page type must declare as a required string. *Considered:* fall back to the first level-1 heading.
4. **Reserved keys in fragments (SPEC §2.1, §4.3, §4.4).** `available` and `variant` are defined for pages. *Decision:* fragments can't use them in spec 0.1; use `@available` inside the fragment. *Considered:* fragment `available` applies to everything the fragment contributes, like a section spec.
5. **Version scheme name (SPEC §4.4).** *Decision:* call the one scheme `numeric`: dotted numbers of any length, compared numerically with missing components as 0. The grammar has no pre-release syntax, so "semver" would overpromise. *Considered:* call it `semver` and cap versions at three components, which needs a new document diagnostic.
6. **Name rules beyond the one-role rule (SPEC §4.3, §4.4, §7.2).** *Decision:* (a) a dimension value belongs to only one dimension, since `cloud` in a spec must mean one thing; (b) dimension names follow the `key` rule, since they're attribute keys; (c) warn on names that differ only in case. (a) and (b) are stated in SPEC §7.2.
7. **Glossary (SPEC §5.4).** The spec gives no format or matching rules. *Decision:* terms with a required plain-text `definition` and an optional `link`; occurrences link to `link`; terms without it aren't linked in site or plain output. Whole-word matching, longest term wins, prose only (not headings, link text, or code), `first` per resolved page by default. No term element is needed: in the site output an occurrence is an ordinary link whose title is the definition, marked with `data-ascribe-term` (see the [element contract]({repo}/blob/main/packages/elements/CONTRACT.md)).
8. **Widget plain fallback (SPEC §6, §9.4).** *Decision:* a static CommonMark string with phrases substituted and no attribute interpolation (no behavior); a widget that wraps content keeps that content in plain output unless `plain-content = "drop"`, since silently losing content is worse than showing it. The spec's "or nothing" then applies to the widget itself, not its content.
9. **Widget schema constraints (SPEC §3.5, §3.6, §6).** *Decision:* groupable widgets are container-only; widgets with container form have no required primary; names starting with `ascribe-` and HTML's reserved custom-element names are rejected.
10. **Absolute links in plain output (SPEC §9.4).** Plain-markdown links are "absolute URLs", which needs the site's origin. *Decision:* optional `[consumer] site`; without it, links are root-relative and `ascribe build` warns.
11. **HTML passthrough (SPEC §9.5).** The site output depends on raw HTML (custom elements). *Decision:* keep the key, but the `astro` profile accepts only `true` until a profile needs `false`, rather than defining a degraded site output now.
12. **Heading ids, image attributes, and asset placement (SPEC §9.4, §9.5).** *Decision:* no keys. The profile named by `profile` fixes all three, and the `astro` profile has one way to do each: a `<ascribe-attributes>` marker for heading ids and image attributes ([site-render contract](../site-render.md)), and mirrored asset copies with relative image references ([asset contract](../assets.md)). A key that accepts one value says nothing, and since unknown keys are errors, adding a key when a second profile needs a choice breaks no existing file, while removing one later would. *Considered:* the keys first drafted, `heading-ids` (`"attribute"` or `"html"`), `image-attributes` (`"attribute"` or `"html"`), `assets` (`"beside-page"` or `"directory"`), and `assets-dir`. Their alternatives were dropped: an `{#id}` attribute block is rewritten by Astro's default smartypants and GFM processing before a plugin sees it (quotes, `--` in ids), which the marker avoids; a raw HTML heading loses the consumer's inline processing and, in Astro, its table-of-contents entry; a raw `<img>` bypasses Astro's image processing; and a shared asset directory needs hashed names to avoid collisions, which mirroring avoids by construction. With them went the loader rule `model-consumer-assets-dir`.
13. **Filter builds on versioned targets (SPEC §9.3).** A filter is "given a target and, for versioned targets, a version". *Decision:* the version is required for versioned targets, and not allowed for versionless ones. *Considered:* a versioned target with no version means "at every version", but then the "state in effect" is undefined.
14. **Project defaults.** *Decision:* `content-root = "docs"`, `output-dir = ".ascribe/build"`; paths relative to `ascribe.toml`, `..` allowed, absolute paths rejected (keeps projects portable). The content root can't be `"."` by default, because the output directory couldn't then sit outside it.
15. **YAML flavor for frontmatter.** *Decision:* the YAML 1.2 core schema (`yes` is a string, `3.10` is a number), and `date` fields accept `YYYY-MM-DD` scalars, quoted or not.
16. **The editor's default build.** *Decision:* the only build, else the build named `site`, else required.
17. **Built-in lifecycle states and note types.** *Decision:* built-ins can be relabeled, and states' `available` flags changed, but not removed; `ga` must stay available; new states must set `available` explicitly; new note types need a `label`; built-in states take part in the one-role rule.
18. **Implicit type and build.** *Decision:* with no `[types]`, one implicit type `page` with a required `title`, whose pages name no type; with no `[builds]`, one build `site` (`switch`, `badge`). Declaring any type or build removes the implicit one.
19. **Features referring to features (SPEC §4.4).** *Decision:* not allowed, so there are no chains or cycles.
20. **Phrases in frontmatter (SPEC §5.1).** *Decision:* opt in per field with `phrases = true`, on `string` and `list(string)` fields only; off by default.
21. **Spec version matching (SPEC §11).** *Decision:* `spec` is a quoted string that must exactly equal a version the processor implements; `"0.1"` for now. Revisit compatibility ranges when 0.2 exists.
22. **Code in a page title (SPEC §7.2).** *Decision:* opt in per field with `inline = "code"`, on a content type's top-level `string` fields, and code spans only (§5.3). Every output carries the plain text and the formatted form. *Considered:* reading code spans in every title, which would change the meaning of every existing title with a backtick and make `title` the one string field that isn't plain text; and links or emphasis in titles, which go wrong where a title becomes the text of another link.
23. **Fields the checks read (SPEC §7.2).** A check of a page's description or its review date needs to know which field holds it, and a project names its fields. *Decision:* a `role` on the field (§5.4), so the type that has the field says what it is, and a type without one opts out of the check. *Considered:* a `field` setting in each check's `[checks]` table, which names one field for every type, so two types with different names for their description can't both be checked, and a type without the field is reported as missing it; and fixed names, such as `description`, which take a name from every project.
