<!-- Generated from tests/conformance/diagnostics.toml by tests/conformance/tests/docs.rs. Edit the registry, then run `ASCRIBE_BLESS=1 cargo test -p ascribe-conformance --test docs`. -->

## Links

### ASC036 `link-target-missing`
@id {permalink=true}: ASC036

Error · file level · next step: choose · [SPEC §5.2]({repo}/blob/main/SPEC.md#52-links)

**When:** Target file doesn't exist.

**Message:** `{path}` doesn't exist

**Fix:** Fix the path, or create the file. Names must match exactly, including case, and the file must be inside the project.

**Read more:** [Links](../reference/directives.md#links) in the directive reference.

### ASC037 `link-id-missing`
@id {permalink=true}: ASC037

Error · page level · next step: choose · [SPEC §5.2]({repo}/blob/main/SPEC.md#52-links)

**When:** Target id doesn't exist in the target file.

**Message:** `{path}` has no heading with the id `{id}`

**Fix:** Fix the `#id`, or give the heading you mean that id with `@id`.

**Read more:** [Links](../reference/directives.md#links) in the directive reference.

### ASC038 `link-to-fragment`
@id {permalink=true}: ASC038

Error · file level · next step: choose · [SPEC §4.2]({repo}/blob/main/SPEC.md#42-include)

**When:** Target is a fragment.

**Message:** `{path}` is a fragment, which isn't published on its own; link to a page that includes it

**Fix:** Link to a page that includes the fragment.

**Read more:** [Links](../reference/directives.md#links) in the directive reference.

### ASC040 `link-id-removed`
@id {permalink=true}: ASC040

Error · page level · next step: write · [SPEC §9.3]({repo}/blob/main/SPEC.md#93-build-modes)

**When:** Target id is removed by a build.

**Message:** build `{build}` removes the heading `{id}` from `{path}`, so this link would be broken in that build

**Fix:** Put the link in a `@variant` arm that the same build removes, or keep the heading in that build.

**Read more:** [Links](../reference/directives.md#links) in the directive reference.

### ASC041 `link-route`
@id {permalink=true}: ASC041

Warning · file level · next step: choose · [SPEC §5.2]({repo}/blob/main/SPEC.md#52-links)

**When:** Destination is a route rather than a file path.

**Message:** this looks like the published route of `{page}`; link to the file instead: `{suggestion}`

**Fix:** Link to the page's file, not its URL: Ascribe writes each output's URLs. The editor's quick fix makes the change.

**Read more:** [Links](../reference/directives.md#links) in the directive reference.

### ASC121 `link-page-dropped`
@id {permalink=true}: ASC121

Error · page level · next step: write · [SPEC §5.2]({repo}/blob/main/SPEC.md#52-links)

**When:** Target page isn't published by a build.

**Message:** build `{build}` doesn't publish `{path}`, so this link would be broken in that build; move the link into a `@variant` arm the build removes

**Fix:** Put the link in a `@variant` arm that the same build removes, or publish the page in that build.

**Read more:** [Links](../reference/directives.md#links) in the directive reference.

## Images

### ASC042 `image-source-missing`
@id {permalink=true}: ASC042

Error · file level · next step: choose · [SPEC §5.3]({repo}/blob/main/SPEC.md#53-images)

**When:** Local source doesn't exist.

**Message:** the image `{path}` doesn't exist

**Fix:** Fix the path, or add the image. Names must match exactly, including case, and the file must be inside the project.

**Read more:** [Images](../reference/directives.md#images) in the directive reference.

### ASC043 `image-alt-missing`
@id {permalink=true}: ASC043

Warning · file level · next step: write · [SPEC §5.3]({repo}/blob/main/SPEC.md#53-images)

**When:** Missing alt text.

**Message:** this image has no alt text; describe it between the brackets for readers who can't see it

**Fix:** Describe the image between the brackets: `![The settings page](../reference/settings.png)`.

**Read more:** [Images](../reference/directives.md#images) in the directive reference.

### ASC059 `image-attribute-missing`
@id {permalink=true}: ASC059

Error · file level · next step: write · [SPEC §5.3]({repo}/blob/main/SPEC.md#53-images)

**When:** Required image attribute missing.

**Message:** this image is missing the required attribute `{key}`

**Fix:** Add the attribute in the block after the image: `![Alt](../reference/image.png){width=600}`.

**Read more:** [Images](../reference/directives.md#images) in the directive reference.

## Phrases

### ASC044 `phrase-undeclared`
@id {permalink=true}: ASC044

Warning · file level · next step: choose · [SPEC §5.1]({repo}/blob/main/SPEC.md#51-phrases)

**When:** `{key}` in prose whose key isn't declared.

**Message:** `{key}` isn't a declared phrase, so its braces are literal text; declare it in [phrases], or write `\{` to keep it literal

**Fix:** Declare the key in `[phrases]` if it's meant as a phrase. Otherwise write `\{` to keep the braces as text.

**Read more:** [Phrases](../reference/directives.md#phrases) in the directive reference.

### ASC126 `phrase-double-braces`
@id {permalink=true}: ASC126

Warning · file level · next step: fix · [SPEC §5.1]({repo}/blob/main/SPEC.md#51-phrases)

**When:** A declared `{key}` directly between two more braces (`{{key}}`), usually a substitution left over from another tool.

**Message:** `{key}` is a declared phrase between two more braces, so its value appears between literal braces; remove the outer braces, or write `\{` to keep them

**Fix:** Remove the outer braces, or write `\{` for a brace that's meant.

**Read more:** [Phrases](../reference/directives.md#phrases) in the directive reference.

## Headings

### ASC045 `heading-phrase-without-id`
@id {permalink=true}: ASC045

Warning · file level · next step: fix · [SPEC §5.5]({repo}/blob/main/SPEC.md#55-heading-ids)

**When:** No `@id`, and the heading contains a phrase.

**Message:** this heading contains a phrase, so its id changes whenever the phrase's value does; give it a stable id with `@id`

**Fix:** Give the heading a stable id: an `@id` line directly under it.

**Read more:** [Heading ids](../reference/directives.md#heading-ids) in the directive reference.

### ASC046 `heading-duplicate-without-id`
@id {permalink=true}: ASC046

Warning · page level · next step: fix · [SPEC §5.5]({repo}/blob/main/SPEC.md#55-heading-ids)

**When:** No `@id`, and the heading's slug is the same as another heading's on the page, so its id is numbered.

**Message:** this heading's text gives the same slug as another heading on the page, so its id is numbered and can change when headings move; give it a stable id with `@id`

**Fix:** Give the heading a stable id: an `@id` line directly under it.

**Read more:** [Heading ids](../reference/directives.md#heading-ids) in the directive reference.

### ASC124 `heading-empty-slug`
@id {permalink=true}: ASC124

Warning · file level · next step: write · [SPEC §5.5]({repo}/blob/main/SPEC.md#55-heading-ids)

**When:** No `@id`, and the heading's slug is empty (its text is only punctuation or emoji).

**Message:** this heading's text gives it an empty id, so nothing can link to it reliably; give it an `@id`

**Fix:** Give the heading an id: an `@id` line directly under it.

**Read more:** [Heading ids](../reference/directives.md#heading-ids) in the directive reference.

## Frontmatter

### ASC047 `frontmatter-unknown-key`
@id {permalink=true}: ASC047

Error · file level · next step: choose · [SPEC §7.2]({repo}/blob/main/SPEC.md#72-declarations)

**When:** Key the file's content type or the fragment schema doesn't declare, other than a reserved key on a page or `intended` on a fragment.

**Message:** `{key}` isn't a field of the content type `{type}`

**Fix:** Fix the key's spelling or remove it, or declare the field in the content type's `frontmatter` (`[fragments.frontmatter]` for a fragment).

**Read more:** [Frontmatter values](../reference/content-model.md#42-frontmatter-values) in the `ascribe.toml` reference.

### ASC048 `frontmatter-missing-field`
@id {permalink=true}: ASC048

Error · file level · next step: write · [SPEC §7.2]({repo}/blob/main/SPEC.md#72-declarations)

**When:** Required field missing.

**Message:** the required field `{field}` is missing; the content type `{type}` needs it

**Fix:** Add the field to the page's frontmatter, or give it a default in the content type.

**Read more:** [Frontmatter values](../reference/content-model.md#42-frontmatter-values) in the `ascribe.toml` reference.

### ASC049 `frontmatter-type-mismatch`
@id {permalink=true}: ASC049

Error · file level · next step: choose · [SPEC §7.2]({repo}/blob/main/SPEC.md#72-declarations)

**When:** Value doesn't match the field's declared type.

**Message:** `{field}` must be \{expected}, but it's \{found}

**Fix:** Give a value of the field's type. Quote a string that YAML would read as something else: `version: "3.10"`.

**Read more:** [Frontmatter values](../reference/content-model.md#42-frontmatter-values) in the `ascribe.toml` reference.

### ASC050 `frontmatter-reserved-in-fragment`
@id {permalink=true}: ASC050

Error · file level · next step: write · [SPEC §2.2]({repo}/blob/main/SPEC.md#22-pages-and-fragments)

**When:** Reserved key (`available`, `variant`, `permalink`, `listing`, `type`) on a fragment.

**Message:** `{key}` is reserved for pages, and fragments can't use it

**Fix:** Remove the key from the fragment's frontmatter. Use `@available` or `@variant` in the fragment's content, or set the key on the pages that include it.

**Read more:** [Fragments](../reference/content-model.md#43-fragments) in the `ascribe.toml` reference.

### ASC122 `frontmatter-syntax`
@id {permalink=true}: ASC122

Error · file level · next step: write · [SPEC §2.1]({repo}/blob/main/SPEC.md#21-files)

**When:** Frontmatter that isn't valid YAML.

**Message:** the frontmatter isn't valid YAML: \{detail}

**Fix:** Fix the YAML between the `---` lines.

**Read more:** [Files](../reference/directives.md#files) in the directive reference.

### ASC188 `listing-loosened`
@id {permalink=true}: ASC188

Error · file level · next step: fix · [SPEC §2.1]({repo}/blob/main/SPEC.md#21-files)

**When:** `listing` looser than a rule of the content model the page is under.

**Message:** this page is `{listing}`, looser than `{rule}` on line \{line} of ascribe.toml, which it's under; a page can only be as strict as its rule, or stricter

**Fix:** Remove the page's `listing`, which the rule already sets, or make it the rule's value or a stricter one: `listed`, then `unquoted`, then `unlisted`. To list the page more openly, change the rule in `[listing]`: a page can only restrict what it's under. The editor's quick fix removes `listing`.

**Read more:** [`[listing]`](../reference/content-model/checks.md#22-listing) in the `ascribe.toml` reference.

### ASC196 `content-type-missing`
@id {permalink=true}: ASC196

Error · file level · next step: choose · [SPEC §7.2]({repo}/blob/main/SPEC.md#72-declarations)

**When:** Page names no content type, in a project that declares types.

**Message:** this page doesn't say what type it is; give it `type:` in its frontmatter, with one of: \{values}

**Fix:** Add `type:` to the page's frontmatter, naming one of the content types `ascribe.toml` declares, such as `type: how-to`. A page's type says which fields its frontmatter has, and which template and guide it's written from. In the editor, a quick fix adds each type.

**Read more:** [Page types](../reference/content-model.md#41-page-types) in the `ascribe.toml` reference.

### ASC197 `content-type-unknown`
@id {permalink=true}: ASC197

Error · file level · next step: choose · [SPEC §7.2]({repo}/blob/main/SPEC.md#72-declarations)

**When:** Page's `type` names no declared content type.

**Message:** `{type}` isn't a content type; the types are: \{values}

**Fix:** Name one of the content types `ascribe.toml` declares, spelled as it is there, or declare the type in `[types.<name>]`. A project that declares no types has no `type` on its pages: remove it.

**Read more:** [Page types](../reference/content-model.md#41-page-types) in the `ascribe.toml` reference.

## Lists

### ASC052 `list-ended-by-directive`
@id {permalink=true}: ASC052

Warning · file level · next step: write · [SPEC §3.9]({repo}/blob/main/SPEC.md#39-directives-inside-lists-and-blockquotes)

**When:** Unindented directive line ends a list.

**Message:** this unindented `@{name}` ends the list above it; indent it to the list item's content to keep it in the item

**Fix:** Indent the directive to the list item's content, to keep it in the item. To end the list there, put a blank line before the directive.

**Read more:** [Lists and block quotes](../reference/directives.md#lists-and-block-quotes) in the directive reference.

### ASC053 `directive-indented-code`
@id {permalink=true}: ASC053

Warning · file level · next step: write · [SPEC §3.9]({repo}/blob/main/SPEC.md#39-directives-inside-lists-and-blockquotes)

**When:** Directive line over-indented into an indented code block.

**Message:** this `@{name}` is indented four or more spaces past its container's content, so it's part of an indented code block, not a directive

**Fix:** Indent the directive less, to its container's content. If it's meant as code, put it in a fenced code block.

**Read more:** [Lists and block quotes](../reference/directives.md#lists-and-block-quotes) in the directive reference.

### ASC054 `steps-numbering-continued`
@id {permalink=true}: ASC054

Warning · file level · next step: write · [SPEC §4.6]({repo}/blob/main/SPEC.md#46-steps)

**When:** An ordered list continues the numbering of a list bound by `@steps` right after it ends (usually an unindented directive split the list).

**Message:** this list continues the numbering of the `@steps` list above it, which usually means an unindented line split that list; indent the line to keep one list

**Fix:** Indent the line that splits the list, so the steps stay one list.

**Read more:** [`@steps`](../reference/directives.md#steps) in the directive reference.

## Files

### ASC123 `source-unreadable`
@id {permalink=true}: ASC123

Error · file level · next step: write · [SPEC §2.1]({repo}/blob/main/SPEC.md#21-files)

**When:** Source file that can't be read, or isn't valid UTF-8.

**Message:** this file can't be read: \{reason}

**Fix:** Check the file's permissions, and save it as UTF-8.

**Read more:** [Files](../reference/directives.md#files) in the directive reference.

## `@snippet`

### ASC127 `snippet-address`
@id {permalink=true}: ASC127

Error · file level · next step: write · [SPEC §4.8]({repo}/blob/main/SPEC.md#48-snippet)

**When:** Address that isn't `<source>:<path>`, optionally with `#<region>`.

**Message:** `{address}` isn't a snippet address: \{detail}; write `<source>:<path>`, optionally with `#<region>`

**Fix:** Write the address as `<source>:<path>`, with the path relative to the source's folder and no `..`, and add `#<region>` for a region. A file outside the project's folder can only be named through a source: declare one in `[sources.<name>]`.

**Read more:** [`@snippet`](../reference/directives.md#snippet) in the directive reference.

### ASC128 `snippet-source-unknown`
@id {permalink=true}: ASC128

Error · file level · next step: choose · [SPEC §4.8]({repo}/blob/main/SPEC.md#48-snippet)

**When:** Source the content model doesn't declare.

**Message:** there's no source named `{source}`; the declared sources are: \{sources}

**Fix:** Fix the source's name, or declare it in ascribe.toml: `[sources.<name>]` with a `path`.

**Read more:** [`@snippet`](../reference/directives.md#snippet) in the directive reference.

### ASC129 `snippet-file-missing`
@id {permalink=true}: ASC129

Error · file level · next step: choose · [SPEC §4.8]({repo}/blob/main/SPEC.md#48-snippet)

**When:** File doesn't exist, or its source doesn't include it.

**Message:** `{path}` doesn't exist in source `{source}`

**Fix:** Fix the path, which is relative to the source's folder. For a file the source doesn't include, add a pattern that matches it to the source's `include`, or take it out of `ignore`. For a link, name the file it leads to through a source that includes it. For a source in another repository, run `ascribe sources fetch` to copy the file. When the file isn't in the repository at the pinned commit, because it was moved or deleted there, `fetch` can't copy it: name the file where it is now, or change the code and move the pin with `ascribe sources update`.

**Read more:** [`@snippet`](../reference/directives.md#snippet) in the directive reference.

### ASC130 `snippet-file-not-text`
@id {permalink=true}: ASC130

Error · file level · next step: write · [SPEC §4.8]({repo}/blob/main/SPEC.md#48-snippet)

**When:** File isn't text.

**Message:** `{path}` isn't text (\{reason}), so it can't be a snippet

**Fix:** Take the snippet from a text file: UTF-8, with no NUL characters.

**Read more:** [`@snippet`](../reference/directives.md#snippet) in the directive reference.

### ASC131 `snippet-region-missing`
@id {permalink=true}: ASC131

Error · file level · next step: choose · [SPEC §4.8]({repo}/blob/main/SPEC.md#48-snippet)

**When:** Region doesn't exist in the file.

**Message:** `{path}` has no region `{region}`; its regions are: \{regions}

**Fix:** Fix the region's name, or mark the region in the file with `:snippet-start: <name>` and `:snippet-end:` comments. A file whose extension isn't in the comment table can only be used whole.

**Read more:** [`@snippet`](../reference/directives.md#snippet) in the directive reference.

### ASC132 `snippet-tags`
@id {permalink=true}: ASC132

Error · file level · next step: write · [SPEC §4.8]({repo}/blob/main/SPEC.md#48-snippet)

**When:** The file's tags are unbalanced, name a region twice, or use a reserved tag.

**Message:** `{path}` can't be used: `{tag}` on line \{line} is never closed

**Fix:** Fix the tags in the code file: give every `-start` tag its `-end`, give each region its own name, and remove tags Ascribe reserves for later (`state`, `replace`, `uncomment`, `emphasize`).

**Read more:** [`@snippet`](../reference/directives.md#snippet) in the directive reference.

## Source copies

### ASC136 `lock-invalid`
@id {permalink=true}: ASC136

Error · file level · next step: write · [SPEC §7.4]({repo}/blob/main/SPEC.md#74-sources-in-another-repository)

**When:** `ascribe.lock` isn't valid TOML, or doesn't have the shape §7.4 gives it.

**Message:** ascribe.lock can't be read: \{detail}

**Fix:** ascribe.lock is written by `ascribe sources fetch` and `ascribe sources update`. Undo the change made to it by hand, or delete it and run `ascribe sources fetch`.

**Read more:** [A source in another repository](../reference/content-model/output.md#a-source-in-another-repository) in the `ascribe.toml` reference.

### ASC137 `lock-source-unknown`
@id {permalink=true}: ASC137

Error · file level · next step: write · [SPEC §7.4]({repo}/blob/main/SPEC.md#74-sources-in-another-repository)

**When:** `ascribe.lock` pins a source the content model doesn't declare with `git`, or pins it with another `git`.

**Message:** ascribe.lock pins source `{source}`, which ascribe.toml doesn't declare; run `ascribe sources fetch` to remove it

**Fix:** Run `ascribe sources fetch` to make the lock match ascribe.toml, or `ascribe sources update <name>` for a source whose `git` changed.

**Read more:** [A source in another repository](../reference/content-model/output.md#a-source-in-another-repository) in the `ascribe.toml` reference.

### ASC138 `source-copy-changed`
@id {permalink=true}: ASC138

Error · file level · next step: write · [SPEC §7.4]({repo}/blob/main/SPEC.md#74-sources-in-another-repository)

**When:** A file `ascribe.lock` lists is missing from the source's folder, or its hash differs.

**Message:** `{path}` isn't the file ascribe.lock pins: it was changed here. Change the code in source `{source}`'s repository and run `ascribe sources update {source}`, or run `ascribe sources fetch` to undo the change

**Fix:** A copy is the file as it is in its repository at the pinned commit. Change the code where it lives, then run `ascribe sources update <name>`; `ascribe sources fetch` puts back a copy that was changed or deleted here.

**Read more:** [A source in another repository](../reference/content-model/output.md#a-source-in-another-repository) in the `ascribe.toml` reference.

### ASC139 `source-copy-unlocked`
@id {permalink=true}: ASC139

Error · file level · next step: write · [SPEC §7.4]({repo}/blob/main/SPEC.md#74-sources-in-another-repository)

**When:** A file in a source's copies folder that `ascribe.lock` doesn't list.

**Message:** `{path}` is in source `{source}`'s copies, but ascribe.lock doesn't list it, so there's no telling which commit it's from; run `ascribe sources fetch`

**Fix:** Run `ascribe sources fetch`: it copies the files snippets use at the pinned commit and records them, and removes the rest.

**Read more:** [A source in another repository](../reference/content-model/output.md#a-source-in-another-repository) in the `ascribe.toml` reference.

### ASC140 `source-copy-unused`
@id {permalink=true}: ASC140

Warning · file level · next step: write · [SPEC §7.4]({repo}/blob/main/SPEC.md#74-sources-in-another-repository)

**When:** A copy that no snippet uses.

**Message:** no snippet uses `{path}` any more; `ascribe sources fetch` removes it

**Fix:** Run `ascribe sources fetch`, which removes the copies no snippet uses.

**Read more:** [A source in another repository](../reference/content-model/output.md#a-source-in-another-repository) in the `ascribe.toml` reference.

## `@intended`

### ASC143 `intended-check`
@id {permalink=true}: ASC143

Error · file level · next step: choose · [SPEC §4.9]({repo}/blob/main/SPEC.md#49-intended)

**When:** Names no check, a check that doesn't exist, a check that isn't a review check, or a check reported somewhere else.

**Message:** `{check}` isn't a check

**Fix:** Name the check whose problem is intended, by its name in the [diagnostics reference](../reference/diagnostics.md): a check whose next step is review, acknowledged where it reports its problems. A page's problem goes in the page's `intended` frontmatter, a block's with `@intended` above the block, and a content model entry's in `[[intended]]` in `ascribe.toml`. Any other problem isn't acknowledged; fix it, or set its level in `[checks]` if it's configurable.

**Read more:** [`@intended`](../reference/directives.md#intended) in the directive reference.

### ASC144 `intended-entry`
@id {permalink=true}: ASC144

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

**Read more:** [`@intended`](../reference/directives.md#intended) in the directive reference.

### ASC145 `intended-unused`
@id {permalink=true}: ASC145

Advice · page level · next step: fix · [SPEC §4.9]({repo}/blob/main/SPEC.md#49-intended)

**When:** An acknowledgement that matches no problem in any build.

**Message:** nothing here needs acknowledging any more: `{check}` reports no problem here in any build

**Fix:** Remove the acknowledgement: what it excused has gone. `ascribe check` reports this only when it checks every build, since a problem may appear in one build alone.

**Read more:** [`@intended`](../reference/directives.md#intended) in the directive reference.

## Permalinks list

### ASC175 `permalinks-list-invalid`
@id {permalink=true}: ASC175

Error · file level · next step: write · [SPEC §7.5]({repo}/blob/main/SPEC.md#75-the-permalinks-list)

**When:** `ascribe.permalinks` isn't valid TOML, or doesn't have the shape §7.5 gives it.

**Message:** ascribe.permalinks can't be read: \{detail}

**Fix:** ascribe.permalinks is written by `ascribe permalinks sync`. Undo the change made to it by hand, or, for a list from a newer version of Ascribe, update Ascribe. Don't delete the file to quiet this: it's how a removed permalink is noticed, and a new one written by `sync` doesn't know the names the content no longer has.

**Read more:** [Permalinks](../reference/directives.md#permalinks) in the directive reference.

### ASC176 `permalinks-list-behind`
@id {permalink=true}: ASC176

Warning · file level · next step: fix · [SPEC §7.5]({repo}/blob/main/SPEC.md#75-the-permalinks-list)

**When:** A permalink the content declares, an alias, or a retired permalink that `ascribe.permalinks` doesn't list as it is.

**Message:** the permalink `{name}` isn't in ascribe.permalinks yet; run `ascribe permalinks sync`

**Fix:** Run `ascribe permalinks sync`, which adds each new permalink to ascribe.permalinks, records where each is declared now, and records each alias and retired permalink as `ascribe.toml` has it, and commit the file with the change. In the editor, the quick fix does the same.

**Read more:** [Permalinks](../reference/directives.md#permalinks) in the directive reference.

### ASC177 `permalink-removed`
@id {permalink=true}: ASC177

Warning · file level · next step: write · [SPEC §7.5]({repo}/blob/main/SPEC.md#75-the-permalinks-list)

**When:** A name `ascribe.permalinks` lists that the project no longer has: not declared, an alias, or retired.

**Message:** `{name}` was a permalink, in `{was}`, and nothing declares it now. Links to it from outside these docs, such as from a product, aren't found by this check

**Fix:** Something outside the docs may link to the name. When what it named is still in the docs, put it back: `@id {permalink=true}: <name>` under the heading that now covers it, `permalink: <name>` in a page's frontmatter, or an alias in `[permalinks.aliases]` that leads to the permalink that covers it now. When it's gone, retire it: `ascribe permalinks retire <name> --reason "…"`, with `--to` for a page that covers it now, writes it in `[permalinks.retired]`, and its address shows the reason. In the editor, the quick fix writes the entry for you to give a reason. A permalink can move to another page or heading and keep working; run `ascribe permalinks sync` after moving it. Running `sync` doesn't quiet this: it keeps every name that's gone.

**Read more:** [Permalinks](../reference/directives.md#permalinks) in the directive reference.

## Retired permalinks and aliases

### ASC179 `permalink-alias-target-missing`
@id {permalink=true}: ASC179

Error · file level · next step: write · [SPEC §7.6]({repo}/blob/main/SPEC.md#76-retired-permalinks-and-aliases)

**When:** An alias that doesn't lead to a permalink the content declares.

**Message:** the alias `{name}` leads to `{target}`, which no heading or page declares

**Fix:** Point the alias at a permalink a heading's `@id {permalink=true}` or a page's `permalink` declares, or at another alias that leads to one. When what it named is gone, retire the alias instead: move it to `[permalinks.retired]`, with a reason.

**Read more:** [`[permalinks]`](../reference/content-model/checks.md#20-permalinks) in the `ascribe.toml` reference.

### ASC180 `permalink-name-conflict`
@id {permalink=true}: ASC180

Error · file level · next step: write · [SPEC §7.6]({repo}/blob/main/SPEC.md#76-retired-permalinks-and-aliases)

**When:** A name `ascribe.toml` retires or aliases that the content declares too.

**Message:** `{name}` is retired here, and `{place}` declares it as a permalink; a name is one thing

**Fix:** When the permalink is back, delete its entry from `[permalinks.retired]` or `[permalinks.aliases]`. When it's going, remove the `@id {permalink=true}` or the page's `permalink` that still declares it: a name retired on purpose is retired once what declared it has gone.

**Read more:** [`[permalinks]`](../reference/content-model/checks.md#20-permalinks) in the `ascribe.toml` reference.

### ASC181 `permalink-retired-target-missing`
@id {permalink=true}: ASC181

Error · file level · next step: write · [SPEC §7.6]({repo}/blob/main/SPEC.md#76-retired-permalinks-and-aliases)

**When:** A retired permalink's `to` that doesn't lead to a page, or to a heading on it.

**Message:** `{name}` is retired to `{to}`, and there's no page `{path}`

**Fix:** Write `to` as a link from the content root is written: a page's path, such as `guides/security.md`, and `#` and a heading's id, such as `guides/security.md#rotate-keys`. Leave `to` out to show the reason alone.

**Read more:** [`[permalinks]`](../reference/content-model/checks.md#20-permalinks) in the `ascribe.toml` reference.
