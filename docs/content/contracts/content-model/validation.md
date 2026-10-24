---
type: reference
title: "`ascribe.toml` format contract: validation rules"
description: "Every rule a content model loader enforces, and the map of SPEC §7.2 to the contract."
---

Part of the [`ascribe.toml` format contract](../content-model.md), whose [contents](../content-model.md#contents) list every section. Each rule's fix, for someone writing `ascribe.toml`, is in the diagnostics reference: [from the file to widgets](../../checks/diagnostics/content-model.md), and [from the consumer to `[checks]`](../../checks/diagnostics/content-model-builds.md).

## 24. Validation rules

A loader MUST enforce every rule below when it loads `ascribe.toml`, and report each violation at the span of the offending key or value. Every rule is an error unless marked **warning**. A model with errors doesn't load, and no document is checked against it; warnings don't stop loading.

- **Slugs** are stable identifiers, for tests and for the diagnostics registry, [`tests/conformance/diagnostics.toml`]({repo}/blob/main/tests/conformance/diagnostics.toml), which gives each one a code. Only `model-name-multiple-roles` corresponds to a row of SPEC §8.2 ("Content model"); the rest are loader rules this reference adds.
- **Messages** are templates. `{name}` is a placeholder. Where a rule has several messages, each covers one case of it.
- Rules about documents, such as a page matching no content type or an unknown frontmatter key, aren't loader rules.
- **Filesystem rules** (`model-content-root-missing`, `model-source-path-missing`, `model-source-outside-repository`, and `model-listing-folder` for a folder that isn't there) need the project directory. A loader given only the file's text, such as an unsaved editor buffer or a unit test, skips them; `ascribe check` and the language server always run them.

### 24.1 File and structure

| Slug | Rule | Message |
|---|---|---|
| `model-toml-syntax` | The file is valid TOML 1.0. | `` ascribe.toml isn't valid TOML: {detail} `` |
| `model-unknown-key` | Every key is one this reference defines for its table, except in tables keyed by names the project chooses. The message names the processor's version, `{version}`, since a key a later version added is unknown to an earlier one. | `` unknown key `{key}` in `[{table}]`; this is ascribe {version}, and the project may need a newer one ``<br>`` unknown key `{key}` in `[{table}]`; did you mean `{suggestion}`? This is ascribe {version}, and the project may need a newer one ``<br>`` `{key}` isn't a key of `[{table}]`: a page says what type it is in its frontmatter, with `type: {type}` ``, for `files` and `default` in a type's table |
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
| `model-type-file` | Each file a type's `template` and `guide` name is there, with that exact name, inside the folder `ascribe.toml` is in once symbolic links are followed, and outside the content root and the output directory. | `` `{path}`, the {key} of content type `{type}`, doesn't exist ``<br>`` `{path}`, the {key} of content type `{type}`, doesn't exist; `{actual}` differs from it only in case ``<br>`` `{path}`, the {key} of content type `{type}`, is inside the content root `{content}`, where it would be read as a page ``<br>`` `{path}`, the {key} of content type `{type}`, is inside the output directory `{output}`, which a build writes and removes ``<br>`` `{path}`, the {key} of content type `{type}`, is outside the folder ascribe.toml is in ``<br>`` `{path}`, the {key} of content type `{type}`, leads through a symbolic link to a file outside the folder ascribe.toml is in `` |
| `model-type-template` | A type's template can be read, and reads as a page: its frontmatter is valid YAML, and its Ascribe syntax has no error. | `` can't read `{path}`, the template of content type `{type}`: {reason} ``<br>`` `{path}`, the template of content type `{type}`, isn't a page Ascribe can read: on its line {line}, {detail} `` |
| `model-field-reserved` | No content type, `[frontmatter]`, or fragment schema declares `type`, `available`, `variant`, `intended`, `permalink`, `listing`, `replaces`, or `nav-label`; no content type or `[frontmatter]` declares `slug`, `formatted`, or `source`; and a `title` they declare is a required `string`. | `` `{field}` is reserved by the Ascribe spec and every page accepts it; remove it from `[{table}]` ``<br>`` `{field}` is reserved by the Ascribe spec, and fragments can't use it ``<br>`` `{field}` is reserved by the Ascribe spec and every fragment accepts it; remove it from `[fragments.frontmatter]` ``<br>`` `slug` is reserved by the astro profile, which uses it as a page's URL id; remove it from `[{table}]` ``<br>`` `formatted` is reserved by the site output, which writes the formatted form of `inline` fields under it; remove it from `[{table}]` ``<br>`` `source` is reserved by the site output, which writes each page's path from the content root under it; remove it from `[{table}]` ``<br>`` every page type has `title`, the page's title, as a required string, so `[{table}]` can't make it "{found}"; declare it as "string", or leave it out `` |
| `model-type-syntax` | Every field and attribute type parses under §5.1, and is allowed where it's used: no `date`, `list`, or `object` for attributes; no `set` for fields; `object`, `list(object)`, and bare `enum` only in table form. | `` "{type}" isn't a valid {kind} type: {detail} ``, for example `` "strng?" isn't a valid field type: expected string, number, boolean, date, enum(…), or list(…) `` |
| `model-type-fields` | `fields` is present exactly when the type is `object` or `list(object)`, optional or not. | `` field `{field}` is an object, so it needs `fields` ``<br>`` `fields` is only allowed on object fields, and `{field}` is "{type}" `` |
| `model-enum-values` | An enumeration has at least one value and no duplicates, and its values come from exactly one of `enum(…)` and `values`. `values` appears only with bare `enum`. | `` `{field}` has an empty enumeration ``<br>`` "{value}" appears twice in the enumeration for `{field}` ``<br>`` `{field}`: list enumeration values in enum(…) or in `values`, not both ``<br>`` `{field}` is a bare enum, so it needs `values` `` |
| `model-set-token` | Every value of a `set(enum)` attribute is a token (SPEC §3.3). | `` "{value}" can't be in a value set: members can't contain spaces or any of , \| { } = " `` |
| `model-default-type` | A `default` has the declared type, and an enumeration default is one of its values. | `` default for `{field}` must be {type}, but it's {found} ``<br>`` default "{value}" for `{field}` isn't one of: {values} `` |
| `model-phrases-field-type` | `phrases = true` is set only on `string` and `list(string)` fields. | `` phrases = true only works on string and list(string) fields, and `{field}` is "{type}" `` |
| `model-inline-field` | `inline` is `"code"`, and is set only on a page type's top-level `string` fields, its own or `[frontmatter]`'s (§5.3). | `` inline = "code" only works on string fields, and `{field}` is "{type}" ``<br>`` inline must be "code", not "{value}" ``<br>`` inline = "code" only works on a page type's top-level fields, its own or `[frontmatter]`'s, and `{field}` is in {place} `` |
| `model-pattern-syntax` | Every pattern parses under §1.3, doesn't start with `/`, and has no `..` segment. | `` "{pattern}" isn't a valid pattern: {detail} ``<br>`` pattern "{pattern}" is already relative to the content root; remove the leading / ``<br>`` pattern "{pattern}" can't contain .. `` |
| `model-field-role` | `role` is `"description"`, on a `string` field, `"review-date"`, on a `date` field, or `"owner"`, on a `string` or `enum` field; only on a page type's top-level fields, its own or `[frontmatter]`'s; and a table has at most one field with each role (§5.4). | `` role = "{role}" only works on a {expected} field, and `{field}` is "{type}" ``<br>`` role must be "description", "review-date", or "owner", not "{value}" ``<br>`` role only works on a page type's top-level fields, its own or `[frontmatter]`'s, and `{field}` is in {place} ``<br>`` `{other}` already has role = "{role}"; a content type has one field with each role `` |
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
| `model-phrase-line` | A phrase that's a target's line names a target that has a live line in `[releases]`. | `` phrase `{key}` is the line of `{target}`, which has no live line in `[releases]` `` |
| `model-releases-target` | Each `[releases.<target>]` table is named after a versioned dimension value, or a name no dimension has, which is a value of the implicit `release` dimension while no declared dimension takes that name; only such a target takes `label`. | `` `[releases.{target}]`: `{target}` is in no declared dimension, so it would be a value of the implicit dimension `release`, but `[dimensions.release]` takes that name; add `{target}` to a dimension's `values`, or rename `[dimensions.release]` ``<br>`` `[releases.{target}]`: `{target}` is versionless, so it has no releases ``<br>`` `[releases.{target}]`: `{target}` is a dimension; name one of its versioned values: {values} ``<br>`` `[releases.{target}] label`: `{target}` is a value of `[dimensions.{dimension}]`, which labels it; set its label in that dimension's `labels` `` |
| `model-releases-line` | Every line of a target is a valid version, listed once, with as many parts as the target's other lines. | `` `[releases.{target}]` mixes lines of different lengths, `{a}` and `{b}`; write every line of a target with the same number of parts ``<br>`` `[releases.{target}]` lists the line `{line}` twice ``<br>`` `[releases.{target}]`: "{line}" isn't a valid version, such as "3.4" `` |
| `model-releases-order` | A line is in one of `planned`, `live`, and `archived`; every planned line is newer than every live line, and every archived line older. | `` `[releases.{target}]`: the planned line `{line}` isn't newer than every live line, and the newest is `{live}` ``<br>`` `[releases.{target}]`: the archived line `{line}` isn't older than every live line, and the oldest is `{live}` ``<br>`` `[releases.{target}]` lists `{line}` in both `{a}` and `{b}`; a line is in one list `` |
| `model-releases-ends` | Every line in `[releases.<target>.ends]` is a live line. | `` `[releases.{target}.ends]`: `{line}` isn't a live line; the live lines are {live} ``<br>`` `[releases.{target}.ends]`: `{line}` isn't a live line, and `{target}` has none `` |
| `model-glossary-page` | Each path in `[glossary] pages` is a page under the content root, not a fragment, with its name's case as on disk. Whether it's there is checked with the project's files, as a type's `template` is. | `` `{path}` in `[glossary] pages` doesn't exist ``<br>`` `{path}` in `[glossary] pages` is a fragment; list the page that includes it `` |

`[versions] scheme` is covered by `model-invalid-value`, and `[versions] current`, which isn't a key, by `model-versions-current`. A `[glossary] terms` table is `model-unknown-key`. Terms are read from pages, so their rules are checks of the project: two terms with the same text or the same id are `glossary-term-duplicate`, a term with no definition `glossary-definition-missing`, and `@term` on a page `pages` doesn't list `term-off-glossary-page`. Image attribute types are covered by the rules in §24.3.

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
| `model-consumer-permalink-path` | `permalink-path` starts with `/`. | `` permalink-path must start with "/", such as "/go/" `` |
| `model-consumer-agents-site` | `agents = true` comes with `site`. | `` agents = true needs `site`: every link in llms.txt and the Markdown pages is an absolute URL `` |
| `model-build-name-case` | Build names are unique ignoring case. | `` builds `{a}` and `{b}` differ only in case, so they'd share an output directory on some file systems `` |
| `model-build-variants` | `variants` is `"switch"` or a non-empty table of dimension selections, each a value or a non-empty array of values. | `` variants must be "switch" or a selection such as { deployment = "cloud" } ``<br>`` build `{build}` selects nothing; write variants = "switch" `` |
| `model-build-unknown-dimension` | Every dimension a selection names is declared. | `` build `{build}` selects dimension `{dimension}`, which isn't declared `` |
| `model-build-unknown-value` | Every selected value is a value of its dimension. | `` build `{build}`: `{value}` isn't a value of `{dimension}`; values: {values} `` |
| `model-build-availability` | `availability` is `"badge"` or a table with exactly one key, `filter`. | `` availability must be "badge" or { filter = "<target> <version>" } `` |
| `model-build-filter-target` | The filter's target is a declared dimension value. | `` build `{build}` filters for `{target}`, which isn't a declared dimension value ``<br>`` build `{build}` filters for `{target}`, which is a dimension; filter for one of its values: {values} `` |
| `model-build-filter-version` | The filter has a version exactly when its target is versioned, and the version follows SPEC Appendix A's `version` rule. | `` build `{build}` filters for `{target}`, which is versioned, so it needs a version, such as "{target} 3.3" ``<br>`` build `{build}`: `{target}` is versionless, so the filter can't name a version ``<br>`` build `{build}`: "{version}" isn't a valid version `` |
| `model-build-versions` | A versioned build's target has a live line in `[releases]`, and the build has no filter. | `` build `{build}` is versioned by `{target}`, which has no live line; list its lines in `[releases.{target}]`, such as `live = ["3.4"]` ``<br>`` build `{build}` is versioned by `{target}` and filtered too; a filtered build is already one version of one target, so a versioned build's availability is `badge` `` |
| `model-build-filter-excluded` (**warning**) | A build's filter target isn't a value its own selection drops. | `` build `{build}` filters for `{target}`, but its selection keeps only {dimension} = {values}, so pages marked for `{target}` are dropped `` |
| `model-editor-build-unknown` | `editor.build` names a declared build. | `` editor.build is `{build}`, which isn't a declared build; builds: {builds} `` |
| `model-editor-build-required` | When there are several builds and none is named `site`, `editor.build` is set. | `` there are several builds and none is named site; set [editor] build to the one the editor should check `` |

### 24.8 Sources

| Slug | Rule | Message |
|---|---|---|
| `model-source-path-missing` | A source's `path` exists and is a directory. | `` the folder of source `{source}`, `{path}`, doesn't exist ``<br>`` the folder of source `{source}`, `{path}`, isn't a directory `` |
| `model-source-outside-repository` | A source's folder is inside the git repository the project is in, when it's in one; with none, in the language server and `ascribe mcp`, inside what the user opened (variant `opened`). | `` the folder of source `{source}`, `{path}`, is outside the git repository the project is in `` |
| `model-source-remote` | A source has `path` or `git`, not both; `branch` only with `git`; a URL `git` can be given; a branch name `git` accepts; and, with `git`, copies outside the content root. | `` source `{source}` has both `path` and `git`: a source is a folder in this repository or another repository, not both ``<br>`` source `{source}` needs `path`, a folder in this repository, or `git`, another repository's URL ``<br>`` source `{source}` has `branch` but no `git`: a branch is only for a source in another repository ``<br>`` `{url}` isn't a URL Ascribe can give `git`: use `https://`, `http://`, `ssh://`, `git://`, `file://`, or `user@host:path` ``<br>`` `{branch}` isn't a branch name `git` accepts ``<br>`` the copies of source `{source}` go in `sources/{source}/`, which is inside the content root `{content}`, where they'd be read as pages; give the content root a folder of its own `` |

### 24.9 Checks

| Slug | Rule | Message |
|---|---|---|
| `model-check-not-configurable` | `[checks]` names only checks the diagnostics registry marks configurable. | `` `{check}` can't be set in [checks]: it reports whether the project is valid, which a project can't lower or turn off `` |
| `model-intended-check` | `[[intended]]` names a review check that's acknowledged at a content model entry. | `` `{check}` isn't a check `` |
| `model-intended-entry` | `[[intended]]` names exactly one declared phrase, feature, or image. | `` an acknowledgement in [[intended]] names what it's about with one of `phrase`, `feature`, or `image` `` |
| `model-checks-vale` | `[checks.vale]` has exactly one of `preset` and `config`, and `off` goes with `preset` and names its rules. | `` `[checks.vale]` has both `preset` and `config`; pick one: Ascribe's preset, or the project's own `.vale.ini` `` |

### 24.10 Permalinks

| Slug | Rule | Message |
|---|---|---|
| `model-permalinks` | A name in `[permalinks]` is retired or an alias, not both; an alias leads, through aliases, to a name that isn't one; a retired permalink's reason isn't the quick fix's placeholder. | `` `{name}` is both retired and an alias; a name is one thing `` |

### 24.11 Crawlers

| Slug | Rule | Message |
|---|---|---|
| `model-crawlers-line` | Each of `[crawlers] lines` is one line, with no line break. | `` `{key}` has a line break; each string in `lines` is one line of robots.txt `` |
| `model-crawlers-named-twice` | A crawler is named in one group: not by a disallowed purpose and a group of `lines`, nor by two groups of `lines`. | `` `{key}` names `{token}`, which a group on line {line} names already; a crawler obeys only one group, so write everything for it in one `` |

### 24.12 Listing

| Slug | Rule | Message |
|---|---|---|
| `model-listing-folder` | A key of `[listing.folders]` is a folder's path from the content root, ending in `/`, of a folder that exists by its exact name. | `` `{folder}` in `[listing.folders]` doesn't exist under the content root ``<br>`` `{folder}` in `[listing.folders]` isn't a folder ``<br>`` `{folder}` in `[listing.folders]` isn't a folder's path; write it from the content root, with `/` between its parts and at its end, such as `internal/` `` |
| `model-listing-type` | A key of `[listing.types]` is a content type. | `` `{type}` in `[listing.types]` isn't a content type; use one of: {values} `` |
| `model-listing-looser` | No rule is looser than a folder's it's under, or than `default`. | `` `{rule}` is looser than `{wider}` on line {line}, which it's under; a narrower rule can only restrict ``<br>`` `{rule}` is looser than the site's `{wider}` on line {line}; with that default, no folder or type can be listed more openly `` |

`model-listing-looser` holds `older-lines` to `default` too.

---

## 25. Map of SPEC §7.2 to this file

Every declaration SPEC §7.2 lists, and every setting SPEC §9.3 and §9.5 need, maps to one section:

| SPEC declaration | Section of `ascribe.toml` | Here |
|---|---|---|
| Spec version (§11) | `spec` | [§2](../content-model.md#2-top-level-spec) |
| Content root (§2.2); output directory (§9.4) | `[project]` | [§3](../content-model.md#3-project) |
| Content types: page schemas and the fragment schema (§2.1, §2.2) | `[frontmatter]`, `[types.<name>]`, `[fragments.frontmatter]` | [§4](../content-model.md#4-typesname-and-fragments) |
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

