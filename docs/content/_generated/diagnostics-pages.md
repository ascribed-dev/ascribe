## Links
@memo: Generated from crates/ascribe-core/data/diagnostics.toml by tests/conformance/tests/docs.rs; edit the registry, then run `ASCRIBE_BLESS=1 cargo test -p ascribe-conformance --test docs` to rewrite it.

### ASC036 `link-target-missing`
@id {permalink=true}: ASC036
@summary: Target file doesn't exist.

[Error](../checks/diagnostics.md#severity) · [file level](../checks/diagnostics.md#file-level-and-page-level) · next step: [choose](../checks/diagnostics.md#what-to-do-next) · [SPEC §5.2]({repo}/blob/main/SPEC.md#52-links)

**When:** Target file doesn't exist.

**Message:** `{path}` doesn't exist

**Fix:** Fix the path, or create the file. Names must match exactly, including case, and the file must be inside the project and not in a `.git` folder.

**Read more:** [Links](../writing/inline.md#links) in the phrases and links reference.

### ASC037 `link-id-missing`
@id {permalink=true}: ASC037
@summary: Target id doesn't exist in the target file.

[Error](../checks/diagnostics.md#severity) · [page level](../checks/diagnostics.md#file-level-and-page-level) · next step: [choose](../checks/diagnostics.md#what-to-do-next) · [SPEC §5.2]({repo}/blob/main/SPEC.md#52-links)

**When:** Target id doesn't exist in the target file.

**Message:** `{path}` has no heading with the id `{id}`

**Fix:** Fix the `#id`, or give the heading you mean that id with `@id`.

**Read more:** [Links](../writing/inline.md#links) in the phrases and links reference.

### ASC038 `link-to-fragment`
@id {permalink=true}: ASC038
@summary: Target is a fragment.

[Error](../checks/diagnostics.md#severity) · [file level](../checks/diagnostics.md#file-level-and-page-level) · next step: [choose](../checks/diagnostics.md#what-to-do-next) · [SPEC §4.2]({repo}/blob/main/SPEC.md#42-include)

**When:** Target is a fragment.

**Message:** `{path}` is a fragment, which isn't published on its own; link to a page that includes it

**Fix:** Link to a page that includes the fragment.

**Read more:** [Links](../writing/inline.md#links) in the phrases and links reference.

### ASC040 `link-id-removed`
@id {permalink=true}: ASC040
@summary: Target id is removed by a build.

[Error](../checks/diagnostics.md#severity) · [page level](../checks/diagnostics.md#file-level-and-page-level) · next step: [write](../checks/diagnostics.md#what-to-do-next) · [SPEC §9.3]({repo}/blob/main/SPEC.md#93-build-modes)

**When:** Target id is removed by a build.

**Message:** build `{build}` removes the heading `{id}` from `{path}`, so this link would be broken in that build

**Fix:** Put the link in a `@variant` arm that the same build removes, or keep the heading in that build.

**Read more:** [Links](../writing/inline.md#links) in the phrases and links reference.

### ASC041 `link-route`
@id {permalink=true}: ASC041
@summary: Destination is a route rather than a file path.

[Warning](../checks/diagnostics.md#severity) · [file level](../checks/diagnostics.md#file-level-and-page-level) · next step: [choose](../checks/diagnostics.md#what-to-do-next) · [SPEC §5.2]({repo}/blob/main/SPEC.md#52-links)

**When:** Destination is a route rather than a file path.

**Message:** this looks like the published route of `{page}`; link to the file instead: `{suggestion}`

**Fix:** Link to the page's file, not its URL: Ascribe writes each output's URLs. The editor's quick fix makes the change.

**Read more:** [Links](../writing/inline.md#links) in the phrases and links reference.

### ASC121 `link-page-dropped`
@id {permalink=true}: ASC121
@summary: Target page isn't published by a build.

[Error](../checks/diagnostics.md#severity) · [page level](../checks/diagnostics.md#file-level-and-page-level) · next step: [write](../checks/diagnostics.md#what-to-do-next) · [SPEC §5.2]({repo}/blob/main/SPEC.md#52-links)

**When:** Target page isn't published by a build.

**Message:** build `{build}` doesn't publish `{path}`, so this link would be broken in that build; move the link into a `@variant` arm the build removes

**Fix:** Put the link in a `@variant` arm that the same build removes, or publish the page in that build. A link to a draft waits until the draft is published: take it out, or remove the page's `draft: true`.

**Read more:** [Links](../writing/inline.md#links) in the phrases and links reference.

### ASC263 `link-operation-missing`
@id {permalink=true}: ASC263
@summary: Target is an operation the declared spec doesn't have.

[Error](../checks/diagnostics.md#severity) · [file level](../checks/diagnostics.md#file-level-and-page-level) · next step: [choose](../checks/diagnostics.md#what-to-do-next) · [SPEC §5.2]({repo}/blob/main/SPEC.md#52-links)

**When:** Target is an operation the declared spec doesn't have.

**Message:** `{path}` has no operation `{id}`

**Fix:** Fix the `#` part: the operation's `operationId` as the spec writes it, or, for an operation with no id, its method in lowercase and its path, `delete/users/{id}`. Or link to the operation that took its place. A spec generated from code can change an id when the code changes: setting ids in the code keeps them, and the links to them.

**Read more:** [Links](../writing/inline.md#links) in the phrases and links reference.

### ASC264 `link-spec-outside-project`
@id {permalink=true}: ASC264
@summary: Target is a declared spec outside the project's folder, with no operation.

[Error](../checks/diagnostics.md#severity) · [file level](../checks/diagnostics.md#file-level-and-page-level) · next step: [write](../checks/diagnostics.md#what-to-do-next) · [SPEC §9.4]({repo}/blob/main/SPEC.md#94-outputs)

**When:** Target is a declared spec outside the project's folder, with no operation.

**Message:** `{path}` is the spec of API `{api}`, outside the project's folder, which no output can copy; link to one of its operations, or to the published reference

**Fix:** Add `#` and an operation's `operationId` to link to that operation in the published reference, or link to the reference's page itself. A spec outside the project's folder is read for its operations, and never copied into the outputs.

**Read more:** [Links](../writing/inline.md#links) in the phrases and links reference.

### ASC274 `link-operation-form-unsupported`
@id {permalink=true}: ASC274
@summary: Target is an operation with no id, by its method and path, and the API's reference is an address pattern.

[Error](../checks/diagnostics.md#severity) · [file level](../checks/diagnostics.md#file-level-and-page-level) · next step: [write](../checks/diagnostics.md#what-to-do-next) · [SPEC §5.2]({repo}/blob/main/SPEC.md#52-links)

**When:** Target is an operation with no id, by its method and path, and the API's reference is an address pattern.

**Message:** `#{operation}` names an operation with no id by its method and path, and the reference of API `{api}` is an address pattern, which has no address for it; a link by method and path needs `renderer = "redoc"`

**Fix:** Give the operation an `operationId` in the spec, or in the code it's generated from, and link to it by its id: every renderer addresses an operation by its id. Or publish the reference with Redoc 3, `{ renderer = "redoc", at = "/api/" }`, which has an address for an operation with no id.

**Read more:** [Links](../writing/inline.md#links) in the phrases and links reference.

### ASC275 `link-operation-by-path`
@id {permalink=true}: ASC275
@summary: Target is an operation that has an id, by its method and path.

[Advice](../checks/diagnostics.md#severity) · [file level](../checks/diagnostics.md#file-level-and-page-level) · next step: [fix](../checks/diagnostics.md#what-to-do-next) · [SPEC §5.2]({repo}/blob/main/SPEC.md#52-links)

**When:** Target is an operation that has an id, by its method and path.

**Message:** `#{operation}` names the operation `{id}` of `{path}` by its method and path; link to it by its id, `#{id}`

**Fix:** Link to the operation by its `operationId`, which every renderer addresses it by and which stays the operation's when its path changes. The fix writes it.

**Read more:** [Links](../writing/inline.md#links) in the phrases and links reference.

### ASC276 `link-operation-deprecated`
@id {permalink=true}: ASC276
@summary: Target is an operation the spec marks deprecated.

[Advice](../checks/diagnostics.md#severity) · [file level](../checks/diagnostics.md#file-level-and-page-level) · next step: [write](../checks/diagnostics.md#what-to-do-next) · [SPEC §5.2]({repo}/blob/main/SPEC.md#52-links)

**When:** Target is an operation the spec marks deprecated.

**Message:** `#{operation}` names an operation `{path}` marks deprecated

**Fix:** Link to the operation that replaces it, if the spec names one, or say in the text that it's deprecated. A deprecated operation still works, and its reference marks it; a guide that teaches it sends readers to something going away.

**Read more:** [Links](../writing/inline.md#links) in the phrases and links reference.

### ASC281 `operation-link-not-operation`
@id {permalink=true}: ASC281
@summary: An `@operation`'s link isn't to an operation of a declared API.

[Error](../checks/diagnostics.md#severity) · [file level](../checks/diagnostics.md#file-level-and-page-level) · next step: [write](../checks/diagnostics.md#what-to-do-next) · [SPEC §4.13]({repo}/blob/main/SPEC.md#413-operation)

**When:** An `@operation`'s link isn't to an operation of a declared API.

**Message:** `{link}` isn't a link to an operation; `@operation` takes a declared API's spec, `#`, and an operation's `operationId`

**Fix:** Link to an operation of an API `[apis]` declares: the spec's path from this page, `#`, and the operation's `operationId`, or, for one with no id, its method in lowercase and its path, `../api/openapi.yaml#delete/users/{id}`. To link to a page or a file, write a link in the text instead.

**Read more:** [`@operation`](../writing/directives.md#operation) in the directive reference.

## Images

### ASC042 `image-source-missing`
@id {permalink=true}: ASC042
@summary: Local source doesn't exist.

[Error](../checks/diagnostics.md#severity) · [file level](../checks/diagnostics.md#file-level-and-page-level) · next step: [choose](../checks/diagnostics.md#what-to-do-next) · [SPEC §5.3]({repo}/blob/main/SPEC.md#53-images)

**When:** Local source doesn't exist.

**Message:** the image `{path}` doesn't exist

**Fix:** Fix the path, or add the image. Names must match exactly, including case, and the file must be inside the project and not in a `.git` folder.

**Read more:** [Images](../writing/inline.md#images) in the phrases and links reference.

### ASC043 `image-alt-missing`
@id {permalink=true}: ASC043
@summary: Missing alt text.

[Warning](../checks/diagnostics.md#severity) · [file level](../checks/diagnostics.md#file-level-and-page-level) · next step: [write](../checks/diagnostics.md#what-to-do-next) · [SPEC §5.3]({repo}/blob/main/SPEC.md#53-images)

**When:** Missing alt text.

**Message:** this image has no alt text; describe it between the brackets for readers who can't see it

**Fix:** Describe the image between the brackets: `![The settings page](settings.png)`.

**Read more:** [Images](../writing/inline.md#images) in the phrases and links reference.

### ASC059 `image-attribute-missing`
@id {permalink=true}: ASC059
@summary: Required image attribute missing.

[Error](../checks/diagnostics.md#severity) · [file level](../checks/diagnostics.md#file-level-and-page-level) · next step: [write](../checks/diagnostics.md#what-to-do-next) · [SPEC §5.3]({repo}/blob/main/SPEC.md#53-images)

**When:** Required image attribute missing.

**Message:** this image is missing the required attribute `{key}`

**Fix:** Add the attribute in the block after the image: `![Alt](image.png){width=600}`.

**Read more:** [Images](../writing/inline.md#images) in the phrases and links reference.

## Phrases

### ASC044 `phrase-undeclared`
@id {permalink=true}: ASC044
@summary: `{key}` in prose whose key isn't declared.

[Warning](../checks/diagnostics.md#severity) · [file level](../checks/diagnostics.md#file-level-and-page-level) · next step: [choose](../checks/diagnostics.md#what-to-do-next) · [SPEC §5.1]({repo}/blob/main/SPEC.md#51-phrases)

**When:** `{key}` in prose whose key isn't declared.

**Message:** `{key}` isn't a declared phrase, so its braces are literal text; declare it in [phrases], or write `\{` to keep it literal

**Fix:** Declare the key in `[phrases]` if it's meant as a phrase. Otherwise write `\{` to keep the braces as text.

**Read more:** [Phrases](../writing/inline.md#phrases) in the phrases and links reference.

### ASC126 `phrase-double-braces`
@id {permalink=true}: ASC126
@summary: A declared `{key}` directly between two more braces (`{{key}}`), usually a substitution left over from another tool.

[Warning](../checks/diagnostics.md#severity) · [file level](../checks/diagnostics.md#file-level-and-page-level) · next step: [fix](../checks/diagnostics.md#what-to-do-next) · [SPEC §5.1]({repo}/blob/main/SPEC.md#51-phrases)

**When:** A declared `{key}` directly between two more braces (`{{key}}`), usually a substitution left over from another tool.

**Message:** `{key}` is a declared phrase between two more braces, so its value appears between literal braces; remove the outer braces, or write `\{` to keep them

**Fix:** Remove the outer braces, or write `\{` for a brace that's meant.

**Read more:** [Phrases](../writing/inline.md#phrases) in the phrases and links reference.

## Headings

### ASC045 `heading-phrase-without-id`
@id {permalink=true}: ASC045
@summary: No `@id`, and the heading contains a phrase.

[Warning](../checks/diagnostics.md#severity) · [file level](../checks/diagnostics.md#file-level-and-page-level) · next step: [fix](../checks/diagnostics.md#what-to-do-next) · [SPEC §5.5]({repo}/blob/main/SPEC.md#55-heading-ids)

**When:** No `@id`, and the heading contains a phrase.

**Message:** this heading contains a phrase, so its id changes whenever the phrase's value does; give it a stable id with `@id`

**Fix:** Give the heading a stable id: an `@id` line directly under it.

**Read more:** [Heading ids](../writing/inline.md#heading-ids) in the phrases and links reference.

### ASC046 `heading-duplicate-without-id`
@id {permalink=true}: ASC046
@summary: No `@id`, and the heading's slug is the same as another heading's on the page, so its id is numbered.

[Warning](../checks/diagnostics.md#severity) · [page level](../checks/diagnostics.md#file-level-and-page-level) · next step: [fix](../checks/diagnostics.md#what-to-do-next) · [SPEC §5.5]({repo}/blob/main/SPEC.md#55-heading-ids)

**When:** No `@id`, and the heading's slug is the same as another heading's on the page, so its id is numbered.

**Message:** this heading's text gives the same slug as another heading on the page, so its id is numbered and can change when headings move; give it a stable id with `@id`

**Fix:** Give the heading a stable id: an `@id` line directly under it.

**Read more:** [Heading ids](../writing/inline.md#heading-ids) in the phrases and links reference.

### ASC124 `heading-empty-slug`
@id {permalink=true}: ASC124
@summary: No `@id`, and the heading's slug is empty (its text is only punctuation or emoji).

[Warning](../checks/diagnostics.md#severity) · [file level](../checks/diagnostics.md#file-level-and-page-level) · next step: [write](../checks/diagnostics.md#what-to-do-next) · [SPEC §5.5]({repo}/blob/main/SPEC.md#55-heading-ids)

**When:** No `@id`, and the heading's slug is empty (its text is only punctuation or emoji).

**Message:** this heading's text gives it an empty id, so nothing can link to it reliably; give it an `@id`

**Fix:** Give the heading an id: an `@id` line directly under it.

**Read more:** [Heading ids](../writing/inline.md#heading-ids) in the phrases and links reference.

## Frontmatter

### ASC047 `frontmatter-unknown-key`
@id {permalink=true}: ASC047
@summary: Key the file's content type or the fragment schema doesn't declare, other than a reserved key on a page or `intended` on a fragment.

[Error](../checks/diagnostics.md#severity) · [file level](../checks/diagnostics.md#file-level-and-page-level) · next step: [choose](../checks/diagnostics.md#what-to-do-next) · [SPEC §7.2]({repo}/blob/main/SPEC.md#72-declarations)

**When:** Key the file's content type or the fragment schema doesn't declare, other than a reserved key on a page or `intended` on a fragment.

**Message:** `{key}` isn't a field of the content type `{type}`

**Fix:** Fix the key's spelling or remove it, or declare the field in the content type's `frontmatter` (`[fragments.frontmatter]` for a fragment).

**Read more:** [Frontmatter values](../content-model/index.md#42-frontmatter-values) in the `ascribe.toml` reference.

### ASC048 `frontmatter-missing-field`
@id {permalink=true}: ASC048
@summary: Required field missing.

[Error](../checks/diagnostics.md#severity) · [file level](../checks/diagnostics.md#file-level-and-page-level) · next step: [write](../checks/diagnostics.md#what-to-do-next) · [SPEC §7.2]({repo}/blob/main/SPEC.md#72-declarations)

**When:** Required field missing.

**Message:** the required field `{field}` is missing; the content type `{type}` needs it

**Fix:** Add the field to the page's frontmatter, or give it a default in the content type. A page with no `title` that opens with a level-1 heading has a fix that makes the heading its title.

**Read more:** [Frontmatter values](../content-model/index.md#42-frontmatter-values) in the `ascribe.toml` reference.

### ASC049 `frontmatter-type-mismatch`
@id {permalink=true}: ASC049
@summary: Value doesn't match the field's declared type.

[Error](../checks/diagnostics.md#severity) · [file level](../checks/diagnostics.md#file-level-and-page-level) · next step: [choose](../checks/diagnostics.md#what-to-do-next) · [SPEC §7.2]({repo}/blob/main/SPEC.md#72-declarations)

**When:** Value doesn't match the field's declared type.

**Message:** `{field}` must be \{expected}, but it's \{found}

**Fix:** Give a value of the field's type. Quote a string that YAML would read as something else: `version: "3.10"`.

**Read more:** [Frontmatter values](../content-model/index.md#42-frontmatter-values) in the `ascribe.toml` reference.

### ASC050 `frontmatter-reserved-in-fragment`
@id {permalink=true}: ASC050
@summary: Reserved key (`available`, `variant`, `permalink`, `listing`, `replaces`, `type`, `nav-label`) on a fragment.

[Error](../checks/diagnostics.md#severity) · [file level](../checks/diagnostics.md#file-level-and-page-level) · next step: [write](../checks/diagnostics.md#what-to-do-next) · [SPEC §2.2]({repo}/blob/main/SPEC.md#22-pages-and-fragments)

**When:** Reserved key (`available`, `variant`, `permalink`, `listing`, `replaces`, `type`, `nav-label`) on a fragment.

**Message:** `{key}` is reserved for pages, and fragments can't use it

**Fix:** Remove the key from the fragment's frontmatter. Use `@available` or `@variant` in the fragment's content, or set the key on the pages that include it.

**Read more:** [Fragments](../content-model/index.md#43-fragments) in the `ascribe.toml` reference.

### ASC122 `frontmatter-syntax`
@id {permalink=true}: ASC122
@summary: Frontmatter that isn't valid YAML.

[Error](../checks/diagnostics.md#severity) · [file level](../checks/diagnostics.md#file-level-and-page-level) · next step: [write](../checks/diagnostics.md#what-to-do-next) · [SPEC §2.1]({repo}/blob/main/SPEC.md#21-files)

**When:** Frontmatter that isn't valid YAML.

**Message:** the frontmatter isn't valid YAML: \{detail}

**Fix:** Fix the YAML between the `---` lines.

**Read more:** [Files](../writing/frontmatter.md#files) in the frontmatter reference.

### ASC188 `listing-loosened`
@id {permalink=true}: ASC188
@summary: `listing` looser than a rule of the content model the page is under.

[Error](../checks/diagnostics.md#severity) · [file level](../checks/diagnostics.md#file-level-and-page-level) · next step: [fix](../checks/diagnostics.md#what-to-do-next) · [SPEC §2.1]({repo}/blob/main/SPEC.md#21-files)

**When:** `listing` looser than a rule of the content model the page is under.

**Message:** this page is `{listing}`, looser than `{rule}` on line \{line} of ascribe.toml, which it's under; a page can only be as strict as its rule, or stricter

**Fix:** Remove the page's `listing`, which the rule already sets, or make it the rule's value or a stricter one: `listed`, then `unquoted`, then `unlisted`. To list the page more openly, change the rule in `[listing]`: a page can only restrict what it's under. The editor's quick fix removes `listing`.

**Read more:** [`[listing]`](../content-model/output.md#17-listing) in the `ascribe.toml` reference.

### ASC196 `content-type-missing`
@id {permalink=true}: ASC196
@summary: Page names no content type, in a project that declares types.

[Error](../checks/diagnostics.md#severity) · [file level](../checks/diagnostics.md#file-level-and-page-level) · next step: [choose](../checks/diagnostics.md#what-to-do-next) · [SPEC §7.2]({repo}/blob/main/SPEC.md#72-declarations)

**When:** Page names no content type, in a project that declares types.

**Message:** this page doesn't say what type it is; give it `type:` in its frontmatter, with one of: \{values}

**Fix:** Add `type:` to the page's frontmatter, naming one of the content types `ascribe.toml` declares, such as `type: how-to`. A page's type says which fields its frontmatter has, and which template and guide it's written from. In the editor, a quick fix adds each type.

**Read more:** [Page types](../content-model/index.md#41-page-types) in the `ascribe.toml` reference.

### ASC197 `content-type-unknown`
@id {permalink=true}: ASC197
@summary: Page's `type` names no declared content type.

[Error](../checks/diagnostics.md#severity) · [file level](../checks/diagnostics.md#file-level-and-page-level) · next step: [choose](../checks/diagnostics.md#what-to-do-next) · [SPEC §7.2]({repo}/blob/main/SPEC.md#72-declarations)

**When:** Page's `type` names no declared content type.

**Message:** `{type}` isn't a content type; the types are: \{values}

**Fix:** Name one of the content types `ascribe.toml` declares, spelled as it is there, or declare the type in `[types.<name>]`. A project that declares no types has no `type` on its pages: remove it.

**Read more:** [Page types](../content-model/index.md#41-page-types) in the `ascribe.toml` reference.

### ASC216 `replaces-not-a-page`
@id {permalink=true}: ASC216
@summary: `replaces` that doesn't name a page.

[Error](../checks/diagnostics.md#severity) · [file level](../checks/diagnostics.md#file-level-and-page-level) · next step: [choose](../checks/diagnostics.md#what-to-do-next) · [SPEC §2.1]({repo}/blob/main/SPEC.md#21-files)

**When:** `replaces` that doesn't name a page.

**Message:** `replaces` names `{path}`, which doesn't exist

**Fix:** Write the path of the page this one takes the place of, relative to this page or from the content root with a leading `/`, as a link to it is written: `replaces: install-legacy.md`.

**Read more:** [A page that replaces another](../writing/frontmatter.md#a-page-that-replaces-another) in the frontmatter reference.

### ASC217 `replaces-cycle`
@id {permalink=true}: ASC217
@summary: A page that `replaces` itself, or a chain of `replaces` that comes back to where it started.

[Error](../checks/diagnostics.md#severity) · [page level](../checks/diagnostics.md#file-level-and-page-level) · next step: [write](../checks/diagnostics.md#what-to-do-next) · [SPEC §2.1]({repo}/blob/main/SPEC.md#21-files)

**When:** A page that `replaces` itself, or a chain of `replaces` that comes back to where it started.

**Message:** `replaces` comes back to this page: \{chain}; a page replaces an older page, and the chain ends

**Fix:** Keep `replaces` on the newer page alone, naming the page it takes the place of. A page replaced in turn names nothing that comes after it.

**Read more:** [A page that replaces another](../writing/frontmatter.md#a-page-that-replaces-another) in the frontmatter reference.

### ASC218 `replaces-overlap`
@id {permalink=true}: ASC218
@summary: A page and the page it `replaces`, both published by one line of a versioned build.

[Error](../checks/diagnostics.md#severity) · [page level](../checks/diagnostics.md#file-level-and-page-level) · next step: [write](../checks/diagnostics.md#what-to-do-next) · [SPEC §9.3]({repo}/blob/main/SPEC.md#93-build-modes)

**When:** A page and the page it `replaces`, both published by one line of a versioned build.

**Message:** `{build}` publishes both `{path}` and `{replaced}`, which it replaces; give the two pages availability that no line shares

**Fix:** Mark the newer page with the line it starts in, `available: self-managed 3.5`, and the page it replaces with where it ends, `available: self-managed (ga 3.0, removed 3.5)`, so each line publishes one of them.

**Read more:** [A page that replaces another](../writing/frontmatter.md#a-page-that-replaces-another) in the frontmatter reference.

### ASC219 `replaces-same-page`
@id {permalink=true}: ASC219
@summary: Two pages that `replaces` the same page, both published by one line of a versioned build.

[Error](../checks/diagnostics.md#severity) · [page level](../checks/diagnostics.md#file-level-and-page-level) · next step: [write](../checks/diagnostics.md#what-to-do-next) · [SPEC §9.3]({repo}/blob/main/SPEC.md#93-build-modes)

**When:** Two pages that `replaces` the same page, both published by one line of a versioned build.

**Message:** `{build}` publishes both `{path}` and `{other}`, which each replace `{replaced}`; one page takes its place in a line

**Fix:** Keep `replaces` on one of the two pages, or give them availability that no line shares, so a line has one page in the old page's place. When one page is split in two, the other takes no `replaces`; link to it from the first.

**Read more:** [A page that replaces another](../writing/frontmatter.md#a-page-that-replaces-another) in the frontmatter reference.

### ASC222 `listing-loosened-in-line`
@id {permalink=true}: ASC222
@summary: `listing` looser than `[listing] older-lines`, on a page an older line of a versioned build publishes.

[Error](../checks/diagnostics.md#severity) · [page level](../checks/diagnostics.md#file-level-and-page-level) · next step: [write](../checks/diagnostics.md#what-to-do-next) · [SPEC §2.1]({repo}/blob/main/SPEC.md#21-files)

**When:** `listing` looser than `[listing] older-lines`, on a page an older line of a versioned build publishes.

**Message:** this page is `{listing}`, and `{rule}` on line \{line} of ascribe.toml makes every page of an older line at least `{strict}`, so it can't be `{listing}` there

**Fix:** Remove the page's `listing`, which the rule already sets in an older line, or make it the rule's value or a stricter one: `listed`, then `unquoted`, then `unlisted`. To list an older line's pages more openly, change `older-lines` in `[listing]`.

**Read more:** [`[listing]`](../content-model/output.md#17-listing) in the `ascribe.toml` reference.

## Lists

### ASC052 `list-ended-by-directive`
@id {permalink=true}: ASC052
@summary: Unindented directive line ends a list.

[Warning](../checks/diagnostics.md#severity) · [file level](../checks/diagnostics.md#file-level-and-page-level) · next step: [write](../checks/diagnostics.md#what-to-do-next) · [SPEC §3.9]({repo}/blob/main/SPEC.md#39-directives-inside-lists-and-blockquotes)

**When:** Unindented directive line ends a list.

**Message:** this unindented `@{name}` ends the list above it; indent it to the list item's content to keep it in the item

**Fix:** Indent the directive to the list item's content, to keep it in the item. To end the list there, put a blank line before the directive.

**Read more:** [Lists and block quotes](../writing/syntax.md#lists-and-block-quotes) in the directive syntax reference.

### ASC053 `directive-indented-code`
@id {permalink=true}: ASC053
@summary: Directive line over-indented into an indented code block.

[Warning](../checks/diagnostics.md#severity) · [file level](../checks/diagnostics.md#file-level-and-page-level) · next step: [write](../checks/diagnostics.md#what-to-do-next) · [SPEC §3.9]({repo}/blob/main/SPEC.md#39-directives-inside-lists-and-blockquotes)

**When:** Directive line over-indented into an indented code block.

**Message:** this `@{name}` is indented four or more spaces past its container's content, so it's part of an indented code block, not a directive

**Fix:** Indent the directive less, to its container's content. If it's meant as code, put it in a fenced code block.

**Read more:** [Lists and block quotes](../writing/syntax.md#lists-and-block-quotes) in the directive syntax reference.

### ASC054 `steps-numbering-continued`
@id {permalink=true}: ASC054
@summary: An ordered list continues the numbering of a list bound by `@steps` right after it ends (usually an unindented directive split the list).

[Warning](../checks/diagnostics.md#severity) · [file level](../checks/diagnostics.md#file-level-and-page-level) · next step: [write](../checks/diagnostics.md#what-to-do-next) · [SPEC §4.6]({repo}/blob/main/SPEC.md#46-steps)

**When:** An ordered list continues the numbering of a list bound by `@steps` right after it ends (usually an unindented directive split the list).

**Message:** this list continues the numbering of the `@steps` list above it, which usually means an unindented line split that list; indent the line to keep one list

**Fix:** Indent the line that splits the list, so the steps stay one list.

**Read more:** [`@steps`](../writing/directives.md#steps) in the directive reference.
