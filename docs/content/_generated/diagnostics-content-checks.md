<!-- Generated from tests/conformance/diagnostics.toml by tests/conformance/tests/docs.rs. Edit the registry, then run `ASCRIBE_BLESS=1 cargo test -p ascribe-conformance --test docs`. -->

## Pages

### ASC149 `page-size`
@id {permalink=true}: ASC149

Advice · page level · next step: write · configurable in `[checks]` · [SPEC §9.4]({repo}/blob/main/SPEC.md#94-outputs)

**Message:** this page's Markdown is \{size} characters in build `{build}`, and an agent reads under \{limit} in one piece; split the page, or move long tables and code below the prose

**Fix:** Split the page into pages that each answer one question, or move long tables, reference lists, and code below the prose, where an agent that stops reading early loses the least. The limit is the Web Documentation Delivery Spec's: a page passes under 50,000 characters of Markdown. Set another with `[checks.page-size] limit`. It's the spec's `page-size-markdown` check, made from the build before a deploy; `ascribe report agents` names this check for it, and for the spec's other checks of a page's size.

**Read more:** [`[checks]`](../reference/content-model/checks.md#19-checks) in the `ascribe.toml` reference.

### ASC150 `page-description-missing`
@id {permalink=true}: ASC150

Advice · file level · next step: write · configurable in `[checks]` · [SPEC §7.2]({repo}/blob/main/SPEC.md#72-declarations)

**Message:** this page has no `{field}`; write a sentence saying what the page is for, which search results and agents show with its title

**Fix:** Add the field the content type marks with `role = "description"` to the page's frontmatter: a sentence or two saying what the page covers and who it's for.

**Read more:** [Field roles](../reference/content-model.md#54-field-roles) in the `ascribe.toml` reference.

### ASC151 `heading-level-skipped`
@id {permalink=true}: ASC151

Advice · page level · next step: fix · configurable in `[checks]` · [SPEC §9.4]({repo}/blob/main/SPEC.md#94-outputs)

**Message:** this heading is level \{level}, after a level-\{previous} heading; make it level \{expected}, so the outline has no gap

**Fix:** Make each heading one level below the heading it belongs under. A page's title is its level-1 heading, so its own headings start at level 2. The quick fix changes the one heading; the headings under it may need to move up too.

**Read more:** [`[checks]`](../reference/content-model/checks.md#19-checks) in the `ascribe.toml` reference.

### ASC152 `code-language-missing`
@id {permalink=true}: ASC152

Advice · file level · next step: choose · configurable in `[checks]` · [SPEC §1.4]({repo}/blob/main/SPEC.md#14-relationship-to-commonmark)

**Message:** this code block has no language; write it after the opening fence, or `text` for output

**Fix:** Write the block's language right after the opening fence, such as `sh`, `toml`, or `json`. Use `text` for program output, logs, and anything else that isn't code. A highlighter colors the block by it, and an agent knows what the block is.

**Read more:** [`[checks]`](../reference/content-model/checks.md#19-checks) in the `ascribe.toml` reference.

### ASC153 `review-overdue`
@id {permalink=true}: ASC153

Advice · file level · next step: review · configurable in `[checks]` · acknowledged in the page's `intended` frontmatter · [SPEC §7.2]({repo}/blob/main/SPEC.md#72-declarations)

**Message:** this page was due for review on \{date}; check it still holds, then move `{field}` to the next review

**Fix:** Read the page against what it describes and correct what has changed, then set its review date to when it's next due. If the page is right as it is, moving the date is the whole fix. The check compares with today's date in UTC (or `ASCRIBE_TODAY`), so it can start reporting without a change to the project.

**Read more:** [Field roles](../reference/content-model.md#54-field-roles) in the `ascribe.toml` reference.

### ASC192 `title-heading-differs`
@id {permalink=true}: ASC192

Advice · page level · next step: review · configurable in `[checks]` · acknowledged in the page's `intended` frontmatter · [SPEC §2.1]({repo}/blob/main/SPEC.md#21-files)

**When:** A page writes its own level-1 heading, and neither it nor the page's title contains the other, ignoring case. A page without a level-1 heading, which most pages are, since the layout draws the title as one, isn't looked at.

**Message:** the title, `{title}`, says something the page's own top heading, `{heading}`, doesn't; a search engine may show either one as the page's name, so a reader can be promised one page and land on what reads like another

**Fix:** Make the two agree: change the title or the heading so one says what the other does, or remove the heading and let the layout draw the title, as it does on most pages. When they differ on purpose, as a landing page's title is often the site's name and its heading a tagline, acknowledge it in the page's `intended` frontmatter, saying why.

**Read more:** [`[checks]`](../reference/content-model/checks.md#19-checks) in the `ascribe.toml` reference.

### ASC195 `template-placeholder-left`
@id {permalink=true}: ASC195

Advice · file level · next step: write · configurable in `[checks]` · [SPEC §7.2]({repo}/blob/main/SPEC.md#72-declarations)

**When:** A page of a content type that names a template still has one of the template's placeholders, exactly as the template has it but for how its lines are wrapped; a paragraph the template quotes as an instruction; or a frontmatter field with the template's value. A page of a type with no template isn't looked at, and a placeholder changed by a word isn't found.

**Message:** this page still has its template's placeholder `{placeholder}`; write what it asks for in its place

**Fix:** Write the page's own text where the template's placeholder is, the text in braces, and remove each instruction the template quotes once you've followed it. A field the template fills in, such as `title`, gets the page's own value. The check finds only what was never touched: a placeholder with a word changed isn't reported, so read the page through before publishing it.

**Read more:** [Page types](../reference/content-model.md#41-page-types) in the `ascribe.toml` reference.

## Across the project

### ASC154 `page-orphan`
@id {permalink=true}: ASC154

Advice · page level · next step: review · configurable in `[checks]` · acknowledged in the page's `intended` frontmatter · [SPEC §5.2]({repo}/blob/main/SPEC.md#52-links)

**When:** No other page the build publishes links to the page or includes it. Index pages (`index.md`), and a build's only page, aren't reported.

**Message:** no other page links to this page or includes it, so readers reach it only through the site's own navigation or search, if those list it

**Fix:** Link to the page from a page about its topic, or include it in one. A site whose own navigation, such as a sidebar, lists the page still reaches it, so this is a hint: acknowledge one such page in its `intended` frontmatter, or turn the check off in [`[checks]`](../reference/content-model/checks.md#19-checks) (`page-orphan = "off"`) when your navigation lists every page.

**Read more:** [Links](../reference/directives.md#links) in the directive reference.

### ASC155 `fragment-unused`
@id {permalink=true}: ASC155

Advice · file level · next step: review · configurable in `[checks]` · acknowledged in the page's `intended` frontmatter · [SPEC §4.2]({repo}/blob/main/SPEC.md#42-include)

**When:** No file includes the fragment.

**Message:** no file includes this fragment, so nothing publishes it

**Fix:** Include the fragment where it belongs with `@include`, or delete it when nothing needs it any more.

**Read more:** [`@include`](../reference/directives.md#include) in the directive reference.

### ASC156 `phrase-unused`
@id {permalink=true}: ASC156

Advice · file level · next step: review · configurable in `[checks]` · acknowledged in `[[intended]]` · [SPEC §5.1]({repo}/blob/main/SPEC.md#51-phrases)

**When:** No file writes the phrase, in its text or in a frontmatter field that takes phrases.

**Message:** no file uses the phrase `{key}`

**Fix:** Write the phrase where its value belongs, or remove it from `[phrases]`. A phrase is used wherever a page or a fragment writes its key in braces, in the text or in a frontmatter field that takes phrases.

**Read more:** [`[phrases]`](../reference/content-model/content.md#phrases) in the `ascribe.toml` reference.

### ASC157 `feature-unused`
@id {permalink=true}: ASC157

Advice · file level · next step: review · configurable in `[checks]` · acknowledged in `[[intended]]` · [SPEC §4.4]({repo}/blob/main/SPEC.md#44-available)

**When:** No availability spec names the feature: no `@available`, table row's `available`, or page's `available`.

**Message:** no file uses the feature `{key}`

**Fix:** Mark the content the feature covers with its key (`@available: <key>`, or `available: <key>` in a page's frontmatter), or remove the feature from `[features]` once nothing needs it.

**Read more:** [`[features.<key>]`](../reference/content-model/content.md#9-featureskey) in the `ascribe.toml` reference.

### ASC158 `glossary-term-unused`
@id {permalink=true}: ASC158

Advice · file level · next step: review · configurable in `[checks]` · acknowledged in `[[intended]]` · [SPEC §5.4]({repo}/blob/main/SPEC.md#54-glossary-terms)

**When:** The term, and each of its aliases, appears in no file's prose. A term with `match = "marked"` isn't reported.

**Message:** the glossary term `{term}` appears in no file's prose, so it's never linked

**Fix:** Check the term and its `aliases` against how the pages write it: occurrences match whole words, in prose only, and ignore case unless the term is case-sensitive. Remove the term when the pages don't need it. A term with `match = "marked"` isn't reported, since its uses are links to its page. A term used only inside a phrase's value is reported too, since phrase values aren't searched: acknowledge it in [`[[intended]]`](../reference/content-model/checks.md#intended-acknowledgements).

**Read more:** [Terms](../reference/content-model/content.md#122-terms) in the `ascribe.toml` reference.

### ASC159 `image-unused`
@id {permalink=true}: ASC159

Advice · file level · next step: review · configurable in `[checks]` · acknowledged in `[[intended]]` · [SPEC §5.3]({repo}/blob/main/SPEC.md#53-images)

**When:** An image file under the content root that no file shows or links to: no Markdown image or link, no `src`, `srcset`, `href`, or `poster` in raw HTML, and no frontmatter string names it.

**Message:** no file shows or links to `{path}`

**Fix:** Delete the image when nothing needs it. A frontmatter string counts when it names the image from the file or from the content root, as `cover: img/card.png` does. An image used only by the site's own code is reported too, since Ascribe doesn't read it: turn the check off in [`[checks]`](../reference/content-model/checks.md#19-checks) when your site uses images that way.

**Read more:** [Images](../reference/directives.md#images) in the directive reference.

### ASC160 `image-large`
@id {permalink=true}: ASC160

Advice · file level · next step: review · configurable in `[checks]` · acknowledged in `[[intended]]` · [SPEC §5.3]({repo}/blob/main/SPEC.md#53-images)

**When:** An image file under the content root is larger than the limit: 500 KB, or `[checks.image-large] limit`.

**Message:** `{path}` is \{size}, over the \{limit} limit for an image

**Fix:** Make the image smaller: save it at the size it's shown, or compress it (a screenshot as an optimized PNG, a photo as JPEG or WebP). The limit is 500 KB; set another in `ascribe.toml`, such as `[checks.image-large] limit = "1 MB"`.

**Read more:** [`[checks]`](../reference/content-model/checks.md#19-checks) in the `ascribe.toml` reference.

### ASC161 `title-duplicate`
@id {permalink=true}: ASC161

Advice · page level · next step: write · configurable in `[checks]` · [SPEC §2.2]({repo}/blob/main/SPEC.md#22-pages-and-fragments)

**When:** Two pages the build publishes have the same title, ignoring case.

**Message:** `{title}` is also the title of \{others}, so readers can't tell the pages apart in search results or a list of pages

**Fix:** Give each page a title that says what sets it apart. When one page replaces the other, remove the old one, or publish only one of them in each build with `available`.

**Read more:** [`[checks]`](../reference/content-model/checks.md#19-checks) in the `ascribe.toml` reference.

### ASC172 `availability-left-behind`
@id {permalink=true}: ASC172

Advice · file level · next step: review · configurable in `[checks]` · acknowledged in `[[intended]]` · [SPEC §4.4]({repo}/blob/main/SPEC.md#44-available)

**When:** `[versions] current` is set, and a feature's availability names a version for every target, none of them later than the current release.

**Message:** feature `{key}` names releases up to \{version}, and the current release is \{current}, so what it marks has shipped

**Fix:** When the feature stands for what the next release adds, as one named `next` does, write the shipped release's spec at the content that shipped in it (`@available: ascribe 0.2.0`), and point the feature at the release after it, or at a state such as `unreleased`. When it's a history worth keeping by name, acknowledge it in [`[[intended]]`](../reference/content-model/checks.md#intended-acknowledgements). After each release, move [`[versions] current`](../reference/content-model/content.md#7-versions) to it.

**Read more:** [`[versions]`](../reference/content-model/content.md#7-versions) in the `ascribe.toml` reference.

### ASC191 `description-duplicate`
@id {permalink=true}: ASC191

Advice · page level · next step: write · configurable in `[checks]` · [SPEC §9.4]({repo}/blob/main/SPEC.md#94-outputs)

**When:** Two pages the build publishes have the same description, ignoring case.

**Message:** this is also the description of \{others}, so a search result, a link preview, and `llms.txt` say the same of each page, and a reader can't tell which one they want

**Fix:** Give each page a description that says what it's for and what sets it apart from the others. When one page replaces the other, remove the old one, or publish only one of them in each build with `available`.

**Read more:** [Field roles](../reference/content-model.md#54-field-roles) in the `ascribe.toml` reference.

## Outputs

### ASC163 `description-too-long`
@id {permalink=true}: ASC163

Advice · file level · next step: write · configurable in `[checks]` · [SPEC §9.4]({repo}/blob/main/SPEC.md#94-outputs)

**Message:** this description is \{length} characters, and its line in llms.txt reads best under \{limit}; shorten it to one sentence

**Fix:** Shorten the description to one sentence that says what the page is for. It's the page's line in `llms.txt`, which an agent reads to choose a page, so the whole index should stay short.

**Read more:** [Field roles](../reference/content-model.md#54-field-roles) in the `ascribe.toml` reference.

### ASC164 `llms-section-large`
@id {permalink=true}: ASC164

Advice · page level · next step: write · configurable in `[checks]` · [SPEC §9.4]({repo}/blob/main/SPEC.md#94-outputs)

**Message:** \{file} is \{size} characters in build `{build}`, over the \{limit} an agent reads in one fetch; move some of its pages into folders of their own, or shorten their descriptions

**Fix:** Each folder of pages gets its own `llms.txt` once the index is too long for one file, so a file this long is one folder, or the pages outside any folder, with more pages than an agent reads in one fetch. Move some of them into subfolders of their own, or shorten their descriptions. It's the delivery spec's `llms-txt-size` check, made from the build before a deploy; `ascribe report agents` names this check for it.

**Read more:** [`[consumer]`](../reference/content-model/output.md#consumer) in the `ascribe.toml` reference.

### ASC184 `listing-disallowed`
@id {permalink=true}: ASC184

Advice · file level · next step: fix · configurable in `[checks]` · [SPEC §2.1]({repo}/blob/main/SPEC.md#21-files)

**When:** A page is `unlisted`, and what the project says in `[crawlers]` keeps a search crawler from fetching it: its own lines, for every crawler or in a group that names the crawler, or a disallowed purpose.

**Message:** this page is unlisted, and `{line}` in `[crawlers] lines` disallows its address, so a crawler never fetches it to read that it isn't to be listed; a search engine can still list the address alone

**Fix:** Remove one of the two. A crawler reads `noindex` only on a page it may fetch, so to keep a page out of search results, let crawlers fetch it: remove the `Disallow` line, or narrow it so it doesn't cover the page, or allow the purpose. If the page only needs to stay out of what crawlers fetch, remove `listing: unlisted`. The editor's quick fix removes `listing`.

**Read more:** [`[crawlers]`](../reference/content-model/checks.md#21-crawlers) in the `ascribe.toml` reference.

### ASC190 `crawlers-disallowed`
@id {permalink=true}: ASC190

Advice · file level · next step: choose · configurable in `[checks]` · [SPEC §7.2]({repo}/blob/main/SPEC.md#72-declarations)

**When:** `[crawlers]` disallows `search` or `on-request`, and the site's pages are listed unless they say otherwise.

**Message:** `{purpose} = "disallow"` keeps AI search products from indexing the site, so its pages aren't in their answers, while the pages are listed for every other search engine

**Fix:** Choose what the site wants. To be in AI search answers, and fetched when a person asks, `allow` the purpose or remove it. To keep it, turn this check off in `[checks]`, saying so in a comment: the choice is the project's, and this only says what it costs. A purpose's group asks only the crawlers Ascribe knows, and some of them say they may not follow `robots.txt`.

**Read more:** [`[crawlers]`](../reference/content-model/checks.md#21-crawlers) in the `ascribe.toml` reference.

## Prose, through Vale

### ASC165 `prose`
@id {permalink=true}: ASC165

Advice · file level · next step: write · configurable in `[checks]` · [SPEC §2.1]({repo}/blob/main/SPEC.md#21-files)

**Message:** \{rule}: \{message}

**Fix:** Vale reported this about the prose, under the rule the message starts with. Change the text, or take the replacement the editor offers when the rule gives one; for text from a phrase, change the phrase's value in `ascribe.toml`. A rule that's wrong for the project is turned off in Vale's configuration, or, for the `quiet` preset, in `[checks.vale] off`; one place is quieted with Vale's own comments (`<!-- vale Rule = NO -->`). See [Prose, through Vale](../reference/../guides/vale.md).

**Read more:** [`[checks.vale]`](../reference/content-model/checks.md#checksvale) in the `ascribe.toml` reference.

### ASC166 `prose-not-checked`
@id {permalink=true}: ASC166

Advice · file level · next step: outside · configurable in `[checks]` · [SPEC §2.1]({repo}/blob/main/SPEC.md#21-files)

**Message:** the prose wasn't checked: `{command}` couldn't be run (\{reason}). Install Vale, or set `[checks.vale] command` to where it is

**Fix:** Install [Vale](https://vale.sh) 3.16 or later so that the command `[checks.vale] command` names (`vale` by default) runs, or fix what Vale's own message says about its configuration. Nothing else is checked differently while it can't run. A project that checks prose only in CI can turn this off on other machines with `prose-not-checked = "off"` in `[checks]`. The agents' tools never start Vale; the agents' hook never starts a `command` that's a program in the project, and the editor asks before it does. Run `ascribe check --vale` in a terminal to check the prose then.

**Read more:** [`[checks.vale]`](../reference/content-model/checks.md#checksvale) in the `ascribe.toml` reference.

## External links

### ASC168 `link-external-broken`
@id {permalink=true}: ASC168

Advice · file level · next step: review · configurable in `[checks]` · acknowledged with `@intended` above the block · [SPEC §5.2]({repo}/blob/main/SPEC.md#52-links)

**When:** `ascribe report links`: the link checker found that an external link answers with an error, doesn't answer in time, or can't be reached. A host `[checks.links] ignore` names isn't checked.

**Message:** `{url}` answered \{status}, so the link is broken for readers

**Fix:** Open the address. When the page is gone, link to where its content went, or remove the link. A site that turns link checkers away, or one that's down for a while, may be fine as it is: acknowledge the link with `@intended {check=link-external-broken}: <why>` directly above its block, or leave the whole host out with `[checks.links] ignore`. Only `ascribe report links` checks external links; `ascribe check` never reaches the network.

**Read more:** [`[checks.links]`](../reference/content-model/checks.md#checkslinks) in the `ascribe.toml` reference.

### ASC169 `link-external-moved`
@id {permalink=true}: ASC169

Advice · file level · next step: choose · configurable in `[checks]` · [SPEC §5.2]({repo}/blob/main/SPEC.md#52-links)

**When:** `ascribe report links`: an external link's address redirects permanently (`301` or `308`) to another one that answers. A site's root that redirects to another of its own pages, a landing page, hasn't moved.

**Message:** `{url}` moved permanently to `{to}`; link to the new address

**Fix:** Link to the address it redirects to; the fix writes it. Check first that it's the page you meant: some sites send every old address to their home page, and then the link needs a new target, not this one. Only `ascribe report links` checks external links.

**Read more:** [`[checks.links]`](../reference/content-model/checks.md#checkslinks) in the `ascribe.toml` reference.

## The published site

### ASC170 `delivery-hosting`
@id {permalink=true}: ASC170

Advice · file level · next step: outside · configurable in `[checks]` · [SPEC §9.5]({repo}/blob/main/SPEC.md#95-consumer-profile)

**When:** `ascribe report agents`: the published site fails, or warns on, a check of the Web Documentation Delivery Spec that its hosting decides, such as status codes, caching, or content negotiation.

**Message:** the site fails the delivery spec's `{check}` check: \{result}. \{setting}

**Fix:** Change the setting the message names where the site is hosted; no page needs editing. The [Astro guide](../reference/../guides/astro.md#what-your-host-does) says what to set on the hosts it names. Only `ascribe report agents` checks a published site.

**Read more:** [`[consumer]`](../reference/content-model/output.md#consumer) in the `ascribe.toml` reference.

### ASC171 `delivery-output`
@id {permalink=true}: ASC171

Advice · file level · next step: outside · configurable in `[checks]` · [SPEC §9.5]({repo}/blob/main/SPEC.md#95-consumer-profile)

**When:** `ascribe report agents`: the published site fails, or warns on, a check of the Web Documentation Delivery Spec that `llms.txt`, the Markdown pages, or the pointer on each page should pass, which Ascribe writes with `[consumer] agents = true`.

**Message:** the site fails the delivery spec's `{check}` check, which what Ascribe writes should pass: \{result}. Report it at \{issues}

**Fix:** First check that the site publishes what the build wrote, as it wrote it: `llms.txt` and the Markdown pages from the plain output, and each page's pointer from the site output. When it does, this is a bug in Ascribe: report it at the issue tracker the message names, with the report's output. No page needs editing.

**Read more:** [`[consumer]`](../reference/content-model/output.md#consumer) in the `ascribe.toml` reference.
