## Files
@memo: Generated from crates/ascribe-core/data/diagnostics.toml by tests/conformance/tests/docs.rs; edit the registry, then run `ASCRIBE_BLESS=1 cargo test -p ascribe-conformance --test docs` to rewrite it.

### ASC123 `source-unreadable`
@id {permalink=true}: ASC123
@summary: Source file that can't be read, or isn't valid UTF-8.

[Error](../checks/diagnostics.md#severity) · [file level](../checks/diagnostics.md#file-level-and-page-level) · next step: [write](../checks/diagnostics.md#what-to-do-next) · [SPEC §2.1]({repo}/blob/main/SPEC.md#21-files)

**When:** Source file that can't be read, or isn't valid UTF-8.

**Message:** this file can't be read: \{reason}

**Fix:** Check the file's permissions, and save it as UTF-8.

**Read more:** [Files](../writing/frontmatter.md#files) in the frontmatter reference.

## `@snippet`

### ASC127 `snippet-address`
@id {permalink=true}: ASC127
@summary: Address that isn't `<source>:<path>`, optionally with `#<region>`.

[Error](../checks/diagnostics.md#severity) · [file level](../checks/diagnostics.md#file-level-and-page-level) · next step: [write](../checks/diagnostics.md#what-to-do-next) · [SPEC §4.8]({repo}/blob/main/SPEC.md#48-snippet)

**When:** Address that isn't `<source>:<path>`, optionally with `#<region>`.

**Message:** `{address}` isn't a snippet address: \{detail}; write `<source>:<path>`, optionally with `#<region>`

**Fix:** Write the address as `<source>:<path>`, with the path relative to the source's folder and no `..`, and add `#<region>` for a region. A file outside the project's folder can only be named through a source: declare one in `[sources.<name>]`.

**Read more:** [`@snippet`](../writing/directives.md#snippet) in the directive reference.

### ASC128 `snippet-source-unknown`
@id {permalink=true}: ASC128
@summary: Source the content model doesn't declare.

[Error](../checks/diagnostics.md#severity) · [file level](../checks/diagnostics.md#file-level-and-page-level) · next step: [choose](../checks/diagnostics.md#what-to-do-next) · [SPEC §4.8]({repo}/blob/main/SPEC.md#48-snippet)

**When:** Source the content model doesn't declare.

**Message:** there's no source named `{source}`; the declared sources are: \{sources}

**Fix:** Fix the source's name, or declare it in ascribe.toml: `[sources.<name>]` with a `path`.

**Read more:** [`@snippet`](../writing/directives.md#snippet) in the directive reference.

### ASC129 `snippet-file-missing`
@id {permalink=true}: ASC129
@summary: File doesn't exist, or its source doesn't include it.

[Error](../checks/diagnostics.md#severity) · [file level](../checks/diagnostics.md#file-level-and-page-level) · next step: [choose](../checks/diagnostics.md#what-to-do-next) · [SPEC §4.8]({repo}/blob/main/SPEC.md#48-snippet)

**When:** File doesn't exist, or its source doesn't include it.

**Message:** `{path}` doesn't exist in source `{source}`

**Fix:** Fix the path. It's relative to the source's folder, so it doesn't repeat it: with `path = "src"`, `src/config.js` is `<source>:config.js`. When the message says the source's patterns leave the file out, it's a link, or it's in another repository, the reference page has the steps.

**Read more:** [`@snippet`](../writing/directives.md#snippet) in the directive reference.

### ASC130 `snippet-file-not-text`
@id {permalink=true}: ASC130
@summary: File isn't text.

[Error](../checks/diagnostics.md#severity) · [file level](../checks/diagnostics.md#file-level-and-page-level) · next step: [write](../checks/diagnostics.md#what-to-do-next) · [SPEC §4.8]({repo}/blob/main/SPEC.md#48-snippet)

**When:** File isn't text.

**Message:** `{path}` isn't text (\{reason}), so it can't be a snippet

**Fix:** Take the snippet from a text file: UTF-8, with no NUL characters.

**Read more:** [`@snippet`](../writing/directives.md#snippet) in the directive reference.

### ASC131 `snippet-region-missing`
@id {permalink=true}: ASC131
@summary: Region doesn't exist in the file.

[Error](../checks/diagnostics.md#severity) · [file level](../checks/diagnostics.md#file-level-and-page-level) · next step: [choose](../checks/diagnostics.md#what-to-do-next) · [SPEC §4.8]({repo}/blob/main/SPEC.md#48-snippet)

**When:** Region doesn't exist in the file.

**Message:** `{path}` has no region `{region}`; its regions are: \{regions}

**Fix:** Fix the region's name, or mark the region in the file with `:snippet-start: <name>` and `:snippet-end:` comments. A file whose name and extension are in neither comment table can only be used whole.

**Read more:** [`@snippet`](../writing/directives.md#snippet) in the directive reference.

### ASC132 `snippet-tags`
@id {permalink=true}: ASC132
@summary: The file's tags are unbalanced, name a region twice, or use a reserved tag.

[Error](../checks/diagnostics.md#severity) · [file level](../checks/diagnostics.md#file-level-and-page-level) · next step: [write](../checks/diagnostics.md#what-to-do-next) · [SPEC §4.8]({repo}/blob/main/SPEC.md#48-snippet)

**When:** The file's tags are unbalanced, name a region twice, or use a reserved tag.

**Message:** `{path}` can't be used: `{tag}` on line \{line} is never closed

**Fix:** Fix the tags in the code file: give every `-start` tag its `-end`, give each region its own name, and remove tags Ascribe reserves for later (`state`, `replace`, `uncomment`, `emphasize`).

**Read more:** [`@snippet`](../writing/directives.md#snippet) in the directive reference.

## Source copies

### ASC136 `lock-invalid`
@id {permalink=true}: ASC136
@summary: `ascribe.lock` isn't valid TOML, or doesn't have the shape §7.4 gives it.

[Error](../checks/diagnostics.md#severity) · [file level](../checks/diagnostics.md#file-level-and-page-level) · next step: [write](../checks/diagnostics.md#what-to-do-next) · [SPEC §7.4]({repo}/blob/main/SPEC.md#74-sources-in-another-repository)

**When:** `ascribe.lock` isn't valid TOML, or doesn't have the shape §7.4 gives it.

**Message:** ascribe.lock can't be read: \{detail}

**Fix:** ascribe.lock is written by `ascribe sources fetch` and `ascribe sources update`. Undo the change made to it by hand, or delete it and run `ascribe sources fetch`.

**Read more:** [A source in another repository](../content-model/sources.md#a-source-in-another-repository) in the `ascribe.toml` reference.

### ASC137 `lock-source-unknown`
@id {permalink=true}: ASC137
@summary: `ascribe.lock` pins a source the content model doesn't declare with `git`, or pins it with another `git`.

[Error](../checks/diagnostics.md#severity) · [file level](../checks/diagnostics.md#file-level-and-page-level) · next step: [write](../checks/diagnostics.md#what-to-do-next) · [SPEC §7.4]({repo}/blob/main/SPEC.md#74-sources-in-another-repository)

**When:** `ascribe.lock` pins a source the content model doesn't declare with `git`, or pins it with another `git`.

**Message:** ascribe.lock pins source `{source}`, which ascribe.toml doesn't declare; run `ascribe sources fetch` to remove it

**Fix:** Run `ascribe sources fetch` to make the lock match ascribe.toml, or `ascribe sources update <name>` for a source whose `git` changed.

**Read more:** [A source in another repository](../content-model/sources.md#a-source-in-another-repository) in the `ascribe.toml` reference.

### ASC138 `source-copy-changed`
@id {permalink=true}: ASC138
@summary: A file `ascribe.lock` lists is missing from the source's folder, or its hash differs.

[Error](../checks/diagnostics.md#severity) · [file level](../checks/diagnostics.md#file-level-and-page-level) · next step: [write](../checks/diagnostics.md#what-to-do-next) · [SPEC §7.4]({repo}/blob/main/SPEC.md#74-sources-in-another-repository)

**When:** A file `ascribe.lock` lists is missing from the source's folder, or its hash differs.

**Message:** `{path}` isn't the file ascribe.lock pins: it was changed here. Change the code in source `{source}`'s repository and run `ascribe sources update {source}`, or run `ascribe sources fetch` to undo the change

**Fix:** A copy is the file as it is in its repository at the pinned commit. Change the code where it lives, then run `ascribe sources update <name>`; `ascribe sources fetch` puts back a copy that was changed or deleted here.

**Read more:** [A source in another repository](../content-model/sources.md#a-source-in-another-repository) in the `ascribe.toml` reference.

### ASC139 `source-copy-unlocked`
@id {permalink=true}: ASC139
@summary: A file in a source's copies folder that `ascribe.lock` doesn't list.

[Error](../checks/diagnostics.md#severity) · [file level](../checks/diagnostics.md#file-level-and-page-level) · next step: [write](../checks/diagnostics.md#what-to-do-next) · [SPEC §7.4]({repo}/blob/main/SPEC.md#74-sources-in-another-repository)

**When:** A file in a source's copies folder that `ascribe.lock` doesn't list.

**Message:** `{path}` is in source `{source}`'s copies, but ascribe.lock doesn't list it, so there's no telling which commit it's from; run `ascribe sources fetch`

**Fix:** Run `ascribe sources fetch`: it copies the files snippets use at the pinned commit and records them, and removes the rest.

**Read more:** [A source in another repository](../content-model/sources.md#a-source-in-another-repository) in the `ascribe.toml` reference.

### ASC140 `source-copy-unused`
@id {permalink=true}: ASC140
@summary: A copy that no snippet uses.

[Warning](../checks/diagnostics.md#severity) · [file level](../checks/diagnostics.md#file-level-and-page-level) · next step: [write](../checks/diagnostics.md#what-to-do-next) · [SPEC §7.4]({repo}/blob/main/SPEC.md#74-sources-in-another-repository)

**When:** A copy that no snippet uses.

**Message:** no snippet uses `{path}` any more; `ascribe sources fetch` removes it

**Fix:** Run `ascribe sources fetch`, which removes the copies no snippet uses.

**Read more:** [A source in another repository](../content-model/sources.md#a-source-in-another-repository) in the `ascribe.toml` reference.

## `@intended`

### ASC143 `intended-check`
@id {permalink=true}: ASC143
@summary: Names no check, a check that doesn't exist, a check that isn't a review check, or a check reported somewhere else.

[Error](../checks/diagnostics.md#severity) · [file level](../checks/diagnostics.md#file-level-and-page-level) · next step: [choose](../checks/diagnostics.md#what-to-do-next) · [SPEC §4.9]({repo}/blob/main/SPEC.md#49-intended)

**When:** Names no check, a check that doesn't exist, a check that isn't a review check, or a check reported somewhere else.

**Message:** `{check}` isn't a check

**Fix:** Name the check whose problem is intended, by its name in the [diagnostics reference](../checks/diagnostics.md): a check whose next step is review, acknowledged where it reports its problems. A page's problem goes in the page's `intended` frontmatter, a block's with `@intended` above the block (a heading's under the heading), and a content model entry's in `[[intended]]` in `ascribe.toml`. Any other problem isn't acknowledged; fix it, or set its level in `[checks]` if it's configurable.

**Read more:** [`@intended`](../writing/directives.md#intended) in the directive reference.

### ASC144 `intended-entry`
@id {permalink=true}: ASC144
@summary: An `intended` frontmatter entry that isn't a mapping with a `check` and a reason, or an acknowledgement whose reason is still a processor's placeholder.

[Error](../checks/diagnostics.md#severity) · [file level](../checks/diagnostics.md#file-level-and-page-level) · next step: [write](../checks/diagnostics.md#what-to-do-next) · [SPEC §4.9]({repo}/blob/main/SPEC.md#49-intended)

**When:** An `intended` frontmatter entry that isn't a mapping with a `check` and a reason, or an acknowledgement whose reason is still a processor's placeholder.

**Message:** `intended` is a list of acknowledgements, each with a `check` and a `reason`

**Fix:** Write `intended` as a list, each entry with the `check` it acknowledges and a `reason` that says why the problem is intended:

```yaml
intended:
  - check: page-orphan
    reason: Linked from the site's sidebar.
```

The editor's quick fix writes a placeholder reason for you to replace: until you do, the acknowledgement is reported.

**Read more:** [`@intended`](../writing/directives.md#intended) in the directive reference.

### ASC145 `intended-unused`
@id {permalink=true}: ASC145
@summary: An acknowledgement that matches no problem in any build.

[Advice](../checks/diagnostics.md#severity) · [page level](../checks/diagnostics.md#file-level-and-page-level) · next step: [fix](../checks/diagnostics.md#what-to-do-next) · [SPEC §4.9]({repo}/blob/main/SPEC.md#49-intended)

**When:** An acknowledgement that matches no problem in any build.

**Message:** nothing here needs acknowledging any more: `{check}` reports no problem here in any build

**Fix:** Remove the acknowledgement: what it excused has gone. `ascribe check` reports this only when it checks every build, since a problem may appear in one build alone.

**Read more:** [`@intended`](../writing/directives.md#intended) in the directive reference.

## Permalinks list

### ASC175 `permalinks-list-invalid`
@id {permalink=true}: ASC175
@summary: `ascribe.permalinks` isn't valid TOML, or doesn't have the shape §7.5 gives it.

[Error](../checks/diagnostics.md#severity) · [file level](../checks/diagnostics.md#file-level-and-page-level) · next step: [write](../checks/diagnostics.md#what-to-do-next) · [SPEC §7.5]({repo}/blob/main/SPEC.md#75-the-permalinks-list)

**When:** `ascribe.permalinks` isn't valid TOML, or doesn't have the shape §7.5 gives it.

**Message:** ascribe.permalinks can't be read: \{detail}

**Fix:** ascribe.permalinks is written by `ascribe permalinks sync`. Undo the change made to it by hand, or, for a list from a newer version of Ascribe, update Ascribe. Don't delete the file to quiet this: it's how a removed permalink is noticed, and a new one written by `sync` doesn't know the names the content no longer has.

**Read more:** [Permalinks](../writing/directives.md#permalinks) in the directive reference.

### ASC176 `permalinks-list-behind`
@id {permalink=true}: ASC176
@summary: A permalink the content declares, or a name `ascribe.addresses` moves or retires, that `ascribe.permalinks` doesn't list as it is.

[Warning](../checks/diagnostics.md#severity) · [file level](../checks/diagnostics.md#file-level-and-page-level) · next step: [fix](../checks/diagnostics.md#what-to-do-next) · [SPEC §7.5]({repo}/blob/main/SPEC.md#75-the-permalinks-list)

**When:** A permalink the content declares, or a name `ascribe.addresses` moves or retires, that `ascribe.permalinks` doesn't list as it is.

**Message:** the permalink `{name}` isn't in ascribe.permalinks yet; run `ascribe permalinks sync`

**Fix:** Run `ascribe permalinks sync`, which adds each new permalink to ascribe.permalinks, records where each is declared now, and records each name `ascribe.addresses` moves or retires as it has it, and commit the file with the change. In the editor, the quick fix does the same.

**Read more:** [Permalinks](../writing/directives.md#permalinks) in the directive reference.

### ASC177 `permalink-lost`
@id {permalink=true}: ASC177
@summary: A name `ascribe.permalinks` lists that the project no longer has: not declared, and not moved or retired in `ascribe.addresses`.

[Warning](../checks/diagnostics.md#severity) · [file level](../checks/diagnostics.md#file-level-and-page-level) · next step: [write](../checks/diagnostics.md#what-to-do-next) · [SPEC §7.5]({repo}/blob/main/SPEC.md#75-the-permalinks-list)

**When:** A name `ascribe.permalinks` lists that the project no longer has: not declared, and not moved or retired in `ascribe.addresses`.

**Message:** `{name}` was a permalink, in `{was}`, and nothing declares it now or says where it went. Links to it from outside these docs, such as from a product, aren't found by this check

**Fix:** Something outside the docs may link to the name. When what it named is still in the docs, put it back: `@id {permalink=true}: <name>` under the heading that now covers it, or `permalink: <name>` in a page's frontmatter. When it moved, record where: `ascribe permalinks retire <name> --to <page>` writes `"go:<name>" = "<page>"` in `ascribe.addresses`' `[moved]`, and its address leads there; a page and `#` and a heading's id, or `go:` and another permalink's name, are both places it can lead. When it's gone, retire it: `ascribe permalinks retire <name> --reason "…"` writes it in `[retired]`, and its address shows the reason. In the editor, the quick fix writes the entry for you to give a reason. A permalink can move to another page or heading and keep working; run `ascribe permalinks sync` after moving it. Running `sync` doesn't quiet this: it keeps every name that's gone.

**Read more:** [Permalinks](../writing/directives.md#permalinks) in the directive reference.

## Memo files

### ASC225 `memo-invalid`
@id {permalink=true}: ASC225
@summary: A memo file can't be read, its frontmatter isn't valid, or its `about` or `quote` isn't one a memo can have; or a `memos/` folder is inside the content root.

[Error](../checks/diagnostics.md#severity) · [file level](../checks/diagnostics.md#file-level-and-page-level) · next step: [write](../checks/diagnostics.md#what-to-do-next) · [SPEC §7.7]({repo}/blob/main/SPEC.md#77-memo-files)

**When:** A memo file that can't be read, has no frontmatter or frontmatter that isn't valid YAML, has no `about`, has an `about` that isn't written as a target, has a key other than `about` and `quote`, or has a `quote` with an `about` that names a content model entry other than a glossary term; or a `memos/` folder inside the content root.

**Message:** this memo file doesn't start with frontmatter, so it's about nothing; start it with `---`, a line `about:` naming what it's about, and `---`

**Fix:** A memo file starts with frontmatter: `about`, which names what the memo is about the way `ascribe refs` names a target (`page.md`, `page.md#id`, a glossary term as `term:<id>`, or an entry such as `phrase:product`), and `quote`, which you can leave out, with words of the block it's about. Nothing else goes in the frontmatter. The folder `memos/` is beside `ascribe.toml`, outside the content root.

**Read more:** [Write a memo file](../writing/memo-files.md#write-a-memo-file) in the memo files reference.

### ASC226 `memo-detached`
@id {permalink=true}: ASC226
@summary: A memo file's `about` that names nothing in the project.

[Warning](../checks/diagnostics.md#severity) · [file level](../checks/diagnostics.md#file-level-and-page-level) · next step: [choose](../checks/diagnostics.md#what-to-do-next) · [SPEC §7.7]({repo}/blob/main/SPEC.md#77-memo-files)

**When:** A memo file's `about` that names nothing in the project.

**Message:** `{about}` isn't a page or fragment of the project, so this memo is about nothing

**Fix:** Point `about` at what the memo is about now. A rename, with `ascribe rename` or the editor, rewrites the `about` of the memos about what it renames, but a page moved or a heading's id changed by hand leaves them detached, so give them the new name. If what it was about is gone, the memo is too: delete the file.

**Read more:** [Placed, outdated, and detached](../writing/memo-files.md#placed-outdated-and-detached) in the memo files reference.

## Navigation

### ASC230 `nav-invalid`
@id {permalink=true}: ASC230
@summary: `ascribe.nav` isn't valid TOML, or doesn't have the shape §7.8 gives it.

[Error](../checks/diagnostics.md#severity) · [file level](../checks/diagnostics.md#file-level-and-page-level) · next step: [write](../checks/diagnostics.md#what-to-do-next) · [SPEC §7.8]({repo}/blob/main/SPEC.md#78-navigation)

**When:** `ascribe.nav` isn't valid TOML, or doesn't have the shape §7.8 gives it.

**Message:** ascribe.nav can't be read: \{detail}

**Fix:** Correct the file to the shape the reference gives it: `version = 1`, and a `[[sections]]` table for each section listed, with `folder`, a path from the content root ending in `/`, or `""` for the root, and optionally `pages`, a list of file names in that folder, and `label`. Until it reads, every sidebar has the default order, as if the project had no file.

**Read more:** [The file](../navigation/index.md#the-file) in the navigation reference.

### ASC231 `nav-page-unknown`
@id {permalink=true}: ASC231
@summary: A `pages` entry in `ascribe.nav` that isn't a page in its section's folder, or that names the section's own `index.md`.

[Error](../checks/diagnostics.md#severity) · [file level](../checks/diagnostics.md#file-level-and-page-level) · next step: [choose](../checks/diagnostics.md#what-to-do-next) · [SPEC §7.8]({repo}/blob/main/SPEC.md#78-navigation)

**When:** A `pages` entry in `ascribe.nav` that isn't a page in its section's folder, or that names the section's own `index.md`.

**Message:** `{name}` isn't a page in `{folder}`; its pages are: \{values}

**Fix:** Write the file name of a page in the section's folder, as it is on disk, case included, or remove the entry. A page in a nested folder is listed in that folder's own section, and a section's `index.md` comes first without being listed. The fix offered renames the entry to the closest page's name; check it's the one you meant. Until the name matches a page, the entry is passed over, and the pages the list leaves out come after the ones it names.

**Read more:** [The file](../navigation/index.md#the-file) in the navigation reference.

### ASC232 `nav-folder-unknown`
@id {permalink=true}: ASC232
@summary: A `folder` in `ascribe.nav` that isn't a section.

[Error](../checks/diagnostics.md#severity) · [file level](../checks/diagnostics.md#file-level-and-page-level) · next step: [choose](../checks/diagnostics.md#what-to-do-next) · [SPEC §7.8]({repo}/blob/main/SPEC.md#78-navigation)

**When:** A `folder` in `ascribe.nav` that isn't a section.

**Message:** `{folder}` isn't a section: no page is in it or in a folder under it

**Fix:** Write the path of a folder with a page in it or under it, from the content root, ending in `/`, as it is on disk, case included, or remove the section's table. A folder whose name starts with `_` holds fragments, and isn't a section. The fix offered changes the folder to the closest section's; check it's the one you meant. Until the folder matches a section, its table is passed over.

**Read more:** [The file](../navigation/index.md#the-file) in the navigation reference.

### ASC233 `nav-duplicate`
@id {permalink=true}: ASC233
@summary: A folder `ascribe.nav` lists twice, or a page a section's `pages` names twice.

[Error](../checks/diagnostics.md#severity) · [file level](../checks/diagnostics.md#file-level-and-page-level) · next step: [fix](../checks/diagnostics.md#what-to-do-next) · [SPEC §7.8]({repo}/blob/main/SPEC.md#78-navigation)

**When:** A folder `ascribe.nav` lists twice, or a page a section's `pages` names twice.

**Message:** the section `{folder}` is listed already, earlier in the file; its first table is the one that counts

**Fix:** Remove the second, keeping what you need from it in the first: a section's table, `pages`, and `label` are read from its first table only, and a page takes its first place in the list. The fix removes the second table, or the second entry in `pages`.

**Read more:** [The file](../navigation/index.md#the-file) in the navigation reference.

### ASC234 `section-page-both`
@id {permalink=true}: ASC234
@summary: A folder with an `index.md` and a page beside it with the folder's name.

[Error](../checks/diagnostics.md#severity) · [file level](../checks/diagnostics.md#file-level-and-page-level) · next step: [choose](../checks/diagnostics.md#what-to-do-next) · [SPEC §7.8]({repo}/blob/main/SPEC.md#78-navigation)

**When:** A folder with an `index.md` and a page beside it with the folder's name.

**Message:** `{folder}` has this `index.md` and `{page}` beside it, and a section has one own page; keep one of them

**Fix:** A section's own page is its `index.md` or the page beside its folder with its name, not both. Keep the one you want first in the section: move the other's content into it and delete it, or rename the other so it's a page of its own. Until then, the `index.md` is the section's own page, and the page beside the folder is one of its parent's pages.

**Read more:** [Sections and pages](../navigation/index.md#sections-and-pages) in the navigation reference.

## Glossary

### ASC239 `glossary-term-duplicate`
@id {permalink=true}: ASC239
@summary: Two terms, or a term and an alias, with the same text, or two terms with the same id.

[Error](../checks/diagnostics.md#severity) · [file level](../checks/diagnostics.md#file-level-and-page-level) · next step: [write](../checks/diagnostics.md#what-to-do-next) · [SPEC §5.4]({repo}/blob/main/SPEC.md#54-glossary-terms)

**When:** Two terms, or a term and an alias, with the same text, or two terms with the same id.

**Message:** "\{text}" is already the glossary term `{first}`, on \{page}

**Fix:** Remove the text from one of the terms' headings or `aliases`, or make the terms distinct. Give one of two terms with the same id another with `@id`. Text is compared ignoring case unless both terms are case-sensitive.

**Read more:** [`[glossary]`](../content-model/content.md#8-glossary) in the `ascribe.toml` reference.

### ASC240 `glossary-definition-missing`
@id {permalink=true}: ASC240
@summary: A term whose section doesn't start with a paragraph.

[Error](../checks/diagnostics.md#severity) · [file level](../checks/diagnostics.md#file-level-and-page-level) · next step: [write](../checks/diagnostics.md#what-to-do-next) · [SPEC §5.4]({repo}/blob/main/SPEC.md#54-glossary-terms)

**When:** A term whose section doesn't start with a paragraph.

**Message:** the term "\{term}" has no definition: its section doesn't start with a paragraph

**Fix:** Write the definition as the first paragraph under the term's heading, after its `@term` and any other directives that bind the heading. A link's title and the editor's hover show that paragraph, so a section that opens with a note, a list, a code block, or a subheading leaves them nothing to show.

**Read more:** [`[glossary]`](../content-model/content.md#8-glossary) in the `ascribe.toml` reference.

### ASC242 `term-off-glossary-page`
@id {permalink=true}: ASC242
@summary: `@term` on a heading that no page the glossary lists shows.

[Error](../checks/diagnostics.md#severity) · [file level](../checks/diagnostics.md#file-level-and-page-level) · next step: [choose](../checks/diagnostics.md#what-to-do-next) · [SPEC §4.11]({repo}/blob/main/SPEC.md#411-term)

**When:** `@term` on a heading that no page the glossary lists shows.

**Message:** `@term` is on `{path}`, which isn't a page `[glossary] pages` lists, so this heading isn't a term

**Fix:** List the page in `[glossary] pages` in ascribe.toml, or move the term to a glossary page, or remove `@term`. Listing the page links its terms on every other page, so the fix that adds it is offered in the editor and never applied by `ascribe check --fix`. A fragment is listed through a page that includes it.

**Read more:** [`[glossary]`](../content-model/content.md#8-glossary) in the `ascribe.toml` reference.

## `@summary`

### ASC249 `summary-not-paragraph`
@id {permalink=true}: ASC249
@summary: Following form whose block isn't a paragraph.

[Error](../checks/diagnostics.md#severity) · [file level](../checks/diagnostics.md#file-level-and-page-level) · next step: [write](../checks/diagnostics.md#what-to-do-next) · [SPEC §4.12]({repo}/blob/main/SPEC.md#412-summary)

**When:** Following form whose block isn't a paragraph.

**Message:** `@summary` with no text must be followed by a paragraph, but the next block is \{found}

**Fix:** Put the paragraph that summarizes the section directly below `@summary`. When nothing on the page reads well alone, as for a lead-in to a list, give the summary its own text instead: `@summary: <text>`, directly under the heading.

**Read more:** [`@summary`](../writing/directives.md#summary) in the directive reference.

### ASC250 `summary-repeated`
@id {permalink=true}: ASC250
@summary: A second summary in one section, including summaries from included content.

[Error](../checks/diagnostics.md#severity) · [page level](../checks/diagnostics.md#file-level-and-page-level) · next step: [write](../checks/diagnostics.md#what-to-do-next) · [SPEC §4.12]({repo}/blob/main/SPEC.md#412-summary)

**When:** A second summary in one section, including summaries from included content.

**Message:** the section `{heading}` already has a summary; a section has one

**Fix:** Remove one of the summaries. A summary in an included file is the summary of the section it lands in, so a page that includes one under a heading with its own has two: keep the one that reads well alone.

**Read more:** [`@summary`](../writing/directives.md#summary) in the directive reference.

### ASC251 `summary-in-variant`
@id {permalink=true}: ASC251
@summary: Summary inside a `@variant` arm, including one an include brings into an arm.

[Error](../checks/diagnostics.md#severity) · [page level](../checks/diagnostics.md#file-level-and-page-level) · next step: [write](../checks/diagnostics.md#what-to-do-next) · [SPEC §4.12]({repo}/blob/main/SPEC.md#412-summary)

**When:** Summary inside a `@variant` arm, including one an include brings into an arm.

**Message:** `@summary` is inside a `@variant` arm, and a summary can't differ by arm

**Fix:** Move the summary out of the group, to the top of its section, with text that holds for every arm.

**Read more:** [`@summary`](../writing/directives.md#summary) in the directive reference.
