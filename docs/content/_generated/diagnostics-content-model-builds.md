## The consumer, builds, and the editor
@memo: Generated from tests/conformance/diagnostics.toml by tests/conformance/tests/docs.rs; edit the registry, then run `ASCRIBE_BLESS=1 cargo test -p ascribe-conformance --test docs` to rewrite it.

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

**Read more:** [`[editor]`](../content-model/checks.md#23-editor) in the `ascribe.toml` reference.

### ASC118 `model-editor-build-required`
@id {permalink=true}: ASC118
@summary: The content model declares several builds, none named `site`, and `[editor] build` doesn't name one.

Error · file level · next step: choose · [SPEC §10]({repo}/blob/main/SPEC.md#10-authoring-environment)

**When:** The content model declares several builds, none named `site`, and `[editor] build` doesn't name one.

**Message:** there are several builds and none is named site; set [editor] build to the one the editor should check

**Fix:** Set `[editor] build` to the build the editor should check, or name one of the builds `site`.

**Read more:** [`[editor]`](../content-model/checks.md#23-editor) in the `ascribe.toml` reference.

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

## `[crawlers]`

### ASC183 `model-crawlers-line`
@id {permalink=true}: ASC183
@summary: A string in `[crawlers] lines` has a line break.

Error · file level · next step: write · [SPEC §7.2]({repo}/blob/main/SPEC.md#72-declarations)

**When:** A string in `[crawlers] lines` has a line break.

**Message:** `{key}` has a line break; each string in `lines` is one line of robots.txt

**Fix:** Split the string into one string per line: `lines = ["User-agent: *", "Disallow: /drafts/"]`.

**Read more:** [`[crawlers]`](../content-model/output.md#18-crawlers) in the `ascribe.toml` reference.

### ASC189 `model-crawlers-named-twice`
@id {permalink=true}: ASC189
@summary: `[crawlers]` names one crawler in two groups.

Error · file level · next step: write · [SPEC §7.2]({repo}/blob/main/SPEC.md#72-declarations)

**When:** `[crawlers]` names one crawler in two groups.

**Message:** `{key}` names `{token}`, which a group on line \{line} names already; a crawler obeys only one group, so write everything for it in one

**Fix:** Name each crawler in one group. Move the lines for it into the first group that names it and remove the second `User-agent` line. For a crawler a disallowed purpose covers, the purpose's group already says `Disallow: /`: remove the group from `lines`, or allow the purpose and write the crawler's group whole in `lines`.

**Read more:** [`[crawlers]`](../content-model/output.md#18-crawlers) in the `ascribe.toml` reference.

## `[listing]`

### ASC185 `model-listing-folder`
@id {permalink=true}: ASC185
@summary: A folder in `[listing.folders]` doesn't exist, isn't a folder, or isn't written as a folder's path.

Error · file level · next step: write · [SPEC §7.2]({repo}/blob/main/SPEC.md#72-declarations)

**When:** A folder in `[listing.folders]` doesn't exist, isn't a folder, or isn't written as a folder's path.

**Message:** `{folder}` in `[listing.folders]` doesn't exist under the content root

**Fix:** Write the folder's path from the content root, ending in `/`, with its name's case as it is on disk: `"internal/" = "unlisted"`. A rule is for a folder and everything under it; patterns aren't read.

**Read more:** [`[listing]`](../content-model/output.md#17-listing) in the `ascribe.toml` reference.

### ASC186 `model-listing-type`
@id {permalink=true}: ASC186
@summary: `[listing.types]` names something that isn't a content type.

Error · file level · next step: choose · [SPEC §7.2]({repo}/blob/main/SPEC.md#72-declarations)

**When:** `[listing.types]` names something that isn't a content type.

**Message:** `{type}` in `[listing.types]` isn't a content type; use one of: \{values}

**Fix:** Name a content type the project declares in `[types]`, or `page` when it declares none.

**Read more:** [`[listing]`](../content-model/output.md#17-listing) in the `ascribe.toml` reference.

### ASC187 `model-listing-looser`
@id {permalink=true}: ASC187
@summary: A rule of `[listing]` is looser than a wider rule it's under.

Error · file level · next step: write · [SPEC §7.2]({repo}/blob/main/SPEC.md#72-declarations)

**When:** A rule of `[listing]` is looser than a wider rule it's under.

**Message:** `{rule}` is looser than `{wider}` on line \{line}, which it's under; a narrower rule can only restrict

**Fix:** Remove the narrower rule, or make it as strict as the one it's under, or stricter: `listed`, then `unquoted`, then `unlisted`. To list some pages more openly than the rest, loosen the wider rule, and restrict the others with rules of their own.

**Read more:** [`[listing]`](../content-model/output.md#17-listing) in the `ascribe.toml` reference.

## `[apis]`

### ASC259 `model-api-outside-repository`
@id {permalink=true}: ASC259
@summary: An API's spec is outside the project's repository, or, with no repository, outside the folder that was opened.

Error · file level · next step: write · [SPEC §7.11]({repo}/blob/main/SPEC.md#711-apis)

**When:** An API's spec is outside the project's repository, or, with no repository, outside the folder that was opened.

**Message:** the spec of API `{api}`, `{path}`, is outside the git repository the project is in

**Fix:** Give the API a `spec` inside the project's repository: a spec in another repository isn't read, so commit a copy of it, or the file your pipeline bundles. With no repository, the editor and `ascribe mcp` read a spec only inside the folder they were opened on: open the folder that contains it.

**Read more:** [`[apis.<name>]`](../content-model/sources.md#21-apisname) in the `ascribe.toml` reference.

### ASC260 `model-api-reference`
@id {permalink=true}: ASC260
@summary: An API's `reference` is an address pattern with a placeholder other than `{operationId}` and `{tag}`, or with no `{operationId}`.

Error · file level · next step: write · [SPEC §7.11]({repo}/blob/main/SPEC.md#711-apis)

**When:** An API's `reference` is an address pattern with a placeholder other than `{operationId}` and `{tag}`, or with no `{operationId}`.

**Message:** the reference of API `{api}` has no `{operationId}`, so every operation's address would be the same

**Fix:** Write the address pattern as your renderer addresses an operation, with `{operationId}` where the operation's id goes and `{tag}` for its first tag, such as `"/api/#/operations/{operationId}"` for Stoplight Elements. For Redoc, write `{ renderer = "redoc", at = "/api/" }` instead.

**Read more:** [`[apis.<name>]`](../content-model/sources.md#21-apisname) in the `ascribe.toml` reference.

## `[checks]`

### ASC142 `model-check-not-configurable`
@id {permalink=true}: ASC142
@summary: `[checks]` sets the level of a check about whether the project is valid.

Error · file level · next step: write · [SPEC §8.2]({repo}/blob/main/SPEC.md#82-diagnostics)

**When:** `[checks]` sets the level of a check about whether the project is valid.

**Message:** `{check}` can't be set in [checks]: it reports whether the project is valid, which a project can't lower or turn off

**Fix:** Remove the check from `[checks]`. Only checks about the content's quality, which the [diagnostics reference](../checks/diagnostics.md) marks as configurable, can be set there; a diagnostic about whether the project is valid is always reported at its own severity.

**Read more:** [`[checks]`](../content-model/checks.md#22-checks) in the `ascribe.toml` reference.

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
