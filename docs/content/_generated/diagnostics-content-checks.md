<!-- Generated from tests/conformance/diagnostics.toml by tests/conformance/tests/docs.rs. Edit the registry, then run `ASCRIBE_BLESS=1 cargo test -p ascribe-conformance --test docs`. -->

### Pages

#### ASC149 `page-size`

Advice · page level · next step: write · configurable in `[checks]` · [SPEC §9.4]({repo}/blob/main/SPEC.md#94-outputs)

**Message:** this page's Markdown is \{size} characters in build `{build}`, and an agent reads under \{limit} in one piece; split the page, or move long tables and code below the prose

**Fix:** Split the page into pages that each answer one question, or move long tables, reference lists, and code below the prose, where an agent that stops reading early loses the least. The limit is the Web Documentation Delivery Spec's: a page passes under 50,000 characters of Markdown. Set another with `[checks.page-size] limit`.

#### ASC150 `page-description-missing`

Advice · file level · next step: write · configurable in `[checks]` · [SPEC §7.2]({repo}/blob/main/SPEC.md#72-declarations)

**Message:** this page has no `{field}`; write a sentence saying what the page is for, which search results and agents show with its title

**Fix:** Add the field the content type marks with `role = "description"` to the page's frontmatter: a sentence or two saying what the page covers and who it's for.

#### ASC151 `heading-level-skipped`

Advice · page level · next step: fix · configurable in `[checks]` · [SPEC §9.4]({repo}/blob/main/SPEC.md#94-outputs)

**Message:** this heading is level \{level}, after a level-\{previous} heading; make it level \{expected}, so the outline has no gap

**Fix:** Make each heading one level below the heading it belongs under. A page's title is its level-1 heading, so its own headings start at level 2. The quick fix changes the one heading; the headings under it may need to move up too.

#### ASC152 `code-language-missing`

Advice · file level · next step: choose · configurable in `[checks]` · [SPEC §1.4]({repo}/blob/main/SPEC.md#14-relationship-to-commonmark)

**Message:** this code block has no language; write it after the opening fence, or `text` for output

**Fix:** Write the block's language right after the opening fence, such as `sh`, `toml`, or `json`. Use `text` for program output, logs, and anything else that isn't code. A highlighter colors the block by it, and an agent knows what the block is.

#### ASC153 `review-overdue`

Advice · file level · next step: review · configurable in `[checks]` · acknowledged in the page's `intended` frontmatter · [SPEC §7.2]({repo}/blob/main/SPEC.md#72-declarations)

**Message:** this page was due for review on \{date}; check it still holds, then move `{field}` to the next review

**Fix:** Read the page against what it describes and correct what has changed, then set its review date to when it's next due. If the page is right as it is, moving the date is the whole fix. The check compares with today's date in UTC (or `ASCRIBE_TODAY`), so it can start reporting without a change to the project.

### Across the project

#### ASC154 `page-orphan`

Advice · page level · next step: review · configurable in `[checks]` · acknowledged in the page's `intended` frontmatter · [SPEC §5.2]({repo}/blob/main/SPEC.md#52-links)

**When:** No other page the build publishes links to the page or includes it. Index pages (`index.md`), and a build's only page, aren't reported.

**Message:** no other page links to this page or includes it, so readers reach it only through the site's own navigation or search, if those list it

**Fix:** Link to the page from a page about its topic, or include it in one. A site whose own navigation, such as a sidebar, lists the page still reaches it, so this is a hint: acknowledge one such page in its `intended` frontmatter, or turn the check off in [`[checks]`](../reference/content-model.md#19-checks) (`page-orphan = "off"`) when your navigation lists every page.

#### ASC155 `fragment-unused`

Advice · file level · next step: review · configurable in `[checks]` · acknowledged in the page's `intended` frontmatter · [SPEC §4.2]({repo}/blob/main/SPEC.md#42-include)

**When:** No file includes the fragment.

**Message:** no file includes this fragment, so nothing publishes it

**Fix:** Include the fragment where it belongs with `@include`, or delete it when nothing needs it any more.

#### ASC156 `phrase-unused`

Advice · file level · next step: review · configurable in `[checks]` · acknowledged in `[[intended]]` · [SPEC §5.1]({repo}/blob/main/SPEC.md#51-phrases)

**When:** No file writes the phrase, in its text or in a frontmatter field that takes phrases.

**Message:** no file uses the phrase `{key}`

**Fix:** Write the phrase where its value belongs, or remove it from `[phrases]`. A phrase is used wherever a page or a fragment writes its key in braces, in the text or in a frontmatter field that takes phrases.

#### ASC157 `feature-unused`

Advice · file level · next step: review · configurable in `[checks]` · acknowledged in `[[intended]]` · [SPEC §4.4]({repo}/blob/main/SPEC.md#44-available)

**When:** No availability spec names the feature: no `@available`, table row's `available`, or page's `available`.

**Message:** no file uses the feature `{key}`

**Fix:** Mark the content the feature covers with its key (`@available: <key>`, or `available: <key>` in a page's frontmatter), or remove the feature from `[features]` once nothing needs it.

#### ASC158 `glossary-term-unused`

Advice · file level · next step: review · configurable in `[checks]` · acknowledged in `[[intended]]` · [SPEC §5.4]({repo}/blob/main/SPEC.md#54-glossary-terms)

**When:** The term, and each of its aliases, appears in no file's prose. A term with `match = "marked"` isn't reported.

**Message:** the glossary term `{term}` appears in no file's prose, so it's never linked

**Fix:** Check the term and its `aliases` against how the pages write it: occurrences match whole words, in prose only, and ignore case unless the term is case-sensitive. Remove the term when the pages don't need it. A term with `match = "marked"` isn't reported, since its uses are links to its page. A term used only inside a phrase's value is reported too, since phrase values aren't searched: acknowledge it in [`[[intended]]`](../reference/content-model.md#intended-acknowledgements).

#### ASC159 `image-unused`

Advice · file level · next step: review · configurable in `[checks]` · acknowledged in `[[intended]]` · [SPEC §5.3]({repo}/blob/main/SPEC.md#53-images)

**When:** An image file under the content root that no file shows or links to: no Markdown image or link, no `src`, `srcset`, `href`, or `poster` in raw HTML, and no frontmatter string names it.

**Message:** no file shows or links to `{path}`

**Fix:** Delete the image when nothing needs it. A frontmatter string counts when it names the image from the file or from the content root, as `cover: img/card.png` does. An image used only by the site's own code is reported too, since Ascribe doesn't read it: turn the check off in [`[checks]`](../reference/content-model.md#19-checks) when your site uses images that way.

#### ASC160 `image-large`

Advice · file level · next step: review · configurable in `[checks]` · acknowledged in `[[intended]]` · [SPEC §5.3]({repo}/blob/main/SPEC.md#53-images)

**When:** An image file under the content root is larger than the limit: 500 KB, or `[checks.image-large] limit`.

**Message:** `{path}` is \{size}, over the \{limit} limit for an image

**Fix:** Make the image smaller: save it at the size it's shown, or compress it (a screenshot as an optimized PNG, a photo as JPEG or WebP). The limit is 500 KB; set another in `ascribe.toml`, such as `[checks.image-large] limit = "1 MB"`.

#### ASC161 `title-duplicate`

Advice · page level · next step: write · configurable in `[checks]` · [SPEC §2.2]({repo}/blob/main/SPEC.md#22-pages-and-fragments)

**When:** Two pages the build publishes have the same title, ignoring case.

**Message:** `{title}` is also the title of \{others}, so readers can't tell the pages apart in search results or a list of pages

**Fix:** Give each page a title that says what sets it apart. When one page replaces the other, remove the old one, or publish only one of them in each build with `available`.

#### ASC172 `availability-left-behind`

Advice · file level · next step: review · configurable in `[checks]` · acknowledged in `[[intended]]` · [SPEC §4.4]({repo}/blob/main/SPEC.md#44-available)

**When:** `[versions] current` is set, and a feature's availability names a version for every target, none of them later than the current release.

**Message:** feature `{key}` names releases up to \{version}, and the current release is \{current}, so what it marks has shipped

**Fix:** When the feature stands for what the next release adds, as one named `next` does, write the shipped release's spec at the content that shipped in it (`@available: ascribe 0.2.0`), and point the feature at the release after it, or at a state such as `unreleased`. When it's a history worth keeping by name, acknowledge it in [`[[intended]]`](../reference/content-model.md#intended-acknowledgements). After each release, move [`[versions] current`](../reference/content-model.md#7-versions) to it.

### Outputs

#### ASC163 `description-too-long`

Advice · file level · next step: write · configurable in `[checks]` · [SPEC §9.4]({repo}/blob/main/SPEC.md#94-outputs)

**Message:** this description is \{length} characters, and its line in llms.txt reads best under \{limit}; shorten it to one sentence

**Fix:** Shorten the description to one sentence that says what the page is for. It's the page's line in `llms.txt`, which an agent reads to choose a page, so the whole index should stay short.

#### ASC164 `llms-section-large`

Advice · page level · next step: write · configurable in `[checks]` · [SPEC §9.4]({repo}/blob/main/SPEC.md#94-outputs)

**Message:** \{file} is \{size} characters in build `{build}`, over the \{limit} an agent reads in one fetch; move some of its pages into folders of their own, or shorten their descriptions

**Fix:** Each folder of pages gets its own `llms.txt` once the index is too long for one file, so a file this long is one folder, or the pages outside any folder, with more pages than an agent reads in one fetch. Move some of them into subfolders of their own, or shorten their descriptions.

### Prose, through Vale

#### ASC165 `prose`

Advice · file level · next step: write · configurable in `[checks]` · [SPEC §2.1]({repo}/blob/main/SPEC.md#21-files)

**Message:** \{rule}: \{message}

**Fix:** Vale reported this about the prose, under the rule the message starts with. Change the text, or take the replacement the editor offers when the rule gives one; for text from a phrase, change the phrase's value in `ascribe.toml`. A rule that's wrong for the project is turned off in Vale's configuration, or, for the `quiet` preset, in `[checks.vale] off`; one place is quieted with Vale's own comments (`<!-- vale Rule = NO -->`). See [Prose, through Vale](../reference/../guides/vale.md).

#### ASC166 `prose-not-checked`

Advice · file level · next step: outside · configurable in `[checks]` · [SPEC §2.1]({repo}/blob/main/SPEC.md#21-files)

**Message:** the prose wasn't checked: `{command}` couldn't be run (\{reason}). Install Vale, or set `[checks.vale] command` to where it is

**Fix:** Install [Vale](https://vale.sh) 3.16 or later so that the command `[checks.vale] command` names (`vale` by default) runs, or fix what Vale's own message says about its configuration. Nothing else is checked differently while it can't run. A project that checks prose only in CI can turn this off on other machines with `prose-not-checked = "off"` in `[checks]`.

### External links

#### ASC168 `link-external-broken`

Advice · file level · next step: review · configurable in `[checks]` · acknowledged with `@intended` above the block · [SPEC §5.2]({repo}/blob/main/SPEC.md#52-links)

**When:** `ascribe report links`: the link checker found that an external link answers with an error, doesn't answer in time, or can't be reached. A host `[checks.links] ignore` names isn't checked.

**Message:** `{url}` answered \{status}, so the link is broken for readers

**Fix:** Open the address. When the page is gone, link to where its content went, or remove the link. A site that turns link checkers away, or one that's down for a while, may be fine as it is: acknowledge the link with `@intended {check=link-external-broken}: <why>` directly above its block, or leave the whole host out with `[checks.links] ignore`. Only `ascribe report links` checks external links; `ascribe check` never reaches the network.

#### ASC169 `link-external-moved`

Advice · file level · next step: choose · configurable in `[checks]` · [SPEC §5.2]({repo}/blob/main/SPEC.md#52-links)

**When:** `ascribe report links`: an external link's address redirects permanently (`301` or `308`) to another one that answers. A site's root that redirects to another of its own pages, a landing page, hasn't moved.

**Message:** `{url}` moved permanently to `{to}`; link to the new address

**Fix:** Link to the address it redirects to; the fix writes it. Check first that it's the page you meant: some sites send every old address to their home page, and then the link needs a new target, not this one. Only `ascribe report links` checks external links.

### The published site

#### ASC170 `delivery-hosting`

Advice · file level · next step: outside · configurable in `[checks]` · [SPEC §9.5]({repo}/blob/main/SPEC.md#95-consumer-profile)

**When:** `ascribe report agents`: the published site fails, or warns on, a check of the Web Documentation Delivery Spec that its hosting decides, such as status codes, caching, or content negotiation.

**Message:** the site fails the delivery spec's `{check}` check: \{result}. \{setting}

**Fix:** Change the setting the message names where the site is hosted; no page needs editing. The [Astro guide](../reference/../guides/astro.md#what-your-host-does) says what to set on the hosts it names. Only `ascribe report agents` checks a published site.

#### ASC171 `delivery-output`

Advice · file level · next step: outside · configurable in `[checks]` · [SPEC §9.5]({repo}/blob/main/SPEC.md#95-consumer-profile)

**When:** `ascribe report agents`: the published site fails, or warns on, a check of the Web Documentation Delivery Spec that `llms.txt`, the Markdown pages, or the pointer on each page should pass, which Ascribe writes with `[consumer] agents = true`.

**Message:** the site fails the delivery spec's `{check}` check, which what Ascribe writes should pass: \{result}. Report it at \{issues}

**Fix:** First check that the site publishes what the build wrote, as it wrote it: `llms.txt` and the Markdown pages from the plain output, and each page's pointer from the site output. When it does, this is a bug in Ascribe: report it at the issue tracker the message names, with the report's output. No page needs editing.
