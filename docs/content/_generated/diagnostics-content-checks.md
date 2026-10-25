## Pages
@memo: Generated from tests/conformance/diagnostics.toml by tests/conformance/tests/docs.rs; edit the registry, then run `ASCRIBE_BLESS=1 cargo test -p ascribe-conformance --test docs` to rewrite it.

### ASC149 `page-size`
@id {permalink=true}: ASC149
@summary: A page's Markdown, as a build writes it for agents to read, is as long as the limit or longer: 50,000 characters, or `[checks.page-size] limit`. A page over the limit in several builds is one problem, with its size in the build where it's longest.

Advice · page level · next step: write · configurable in `[checks]` · [SPEC §9.4]({repo}/blob/main/SPEC.md#94-outputs)

**When:** A page's Markdown, as a build writes it for agents to read, is as long as the limit or longer: 50,000 characters, or `[checks.page-size] limit`. A page over the limit in several builds is one problem, with its size in the build where it's longest.

**Message:** this page's Markdown is \{size} characters in build `{build}`, and an agent reads under \{limit} in one piece; split the page, or move long tables and code below the prose

**Fix:** Split the page into pages that each answer one question, or move long tables, reference lists, and code below the prose, where an agent that stops reading early loses the least. The problem names the page's largest parts, up to five, largest first, each with its size: its sections, at their headings, and its includes, at their `@include` lines. The limit is the Web Documentation Delivery Spec's: a page passes under 50,000 characters of Markdown. Set another with `[checks.page-size] limit`. It's the spec's `page-size-markdown` check, made from the build before a deploy; `ascribe report agents` names this check for it, and for the spec's other checks of a page's size.

**Read more:** [`[checks]`](../content-model/checks.md#19-checks) in the `ascribe.toml` reference.

### ASC150 `page-description-missing`
@id {permalink=true}: ASC150
@summary: A page's content type marks a field `role = "description"`, and the page leaves the field out or empty. A required field, whose absence is `frontmatter-missing-field`, and a field with a default aren't looked at.

Advice · file level · next step: write · configurable in `[checks]` · [SPEC §7.2]({repo}/blob/main/SPEC.md#72-declarations)

**When:** A page's content type marks a field `role = "description"`, and the page leaves the field out or empty. A required field, whose absence is `frontmatter-missing-field`, and a field with a default aren't looked at.

**Message:** this page has no `{field}`; write a sentence saying what the page is for, which search results and agents show with its title

**Fix:** Add the field the content type marks with `role = "description"` to the page's frontmatter: a sentence or two saying what the page covers and who it's for.

**Read more:** [Field roles](../content-model/index.md#54-field-roles) in the `ascribe.toml` reference.

### ASC151 `heading-level-skipped`
@id {permalink=true}: ASC151
@summary: A heading is more than one level below the heading before it, or is at level 1, which the page's title is. Headings from includes count at the levels they're written with, and after a skip the next heading is compared with the one that skipped, so a skipped section is reported once.

Advice · page level · next step: fix · configurable in `[checks]` · [SPEC §9.4]({repo}/blob/main/SPEC.md#94-outputs)

**When:** A heading is more than one level below the heading before it, or is at level 1, which the page's title is. Headings from includes count at the levels they're written with, and after a skip the next heading is compared with the one that skipped, so a skipped section is reported once.

**Message:** this heading is level \{level}, after a level-\{previous} heading; make it level \{expected}, so the outline has no gap

**Fix:** Make each heading one level below the heading it belongs under. A page's title is its level-1 heading, so its own headings start at level 2. The quick fix changes the one heading; the headings under it may need to move up too.

**Read more:** [`[checks]`](../content-model/checks.md#19-checks) in the `ascribe.toml` reference.

### ASC152 `code-language-missing`
@id {permalink=true}: ASC152
@summary: A fenced code block has no language after its opening fence: nothing, or only attributes, such as `phrases=true`. An indented code block isn't looked at.

Advice · file level · next step: choose · configurable in `[checks]` · [SPEC §1.4]({repo}/blob/main/SPEC.md#14-relationship-to-commonmark)

**When:** A fenced code block has no language after its opening fence: nothing, or only attributes, such as `phrases=true`. An indented code block isn't looked at.

**Message:** this code block has no language; write it after the opening fence, or `text` for output

**Fix:** Write the block's language right after the opening fence, such as `sh`, `toml`, or `json`. Use `text` for program output, logs, and anything else that isn't code. A highlighter colors the block by it, and an agent knows what the block is.

**Read more:** [`[checks]`](../content-model/checks.md#19-checks) in the `ascribe.toml` reference.

### ASC153 `review-overdue`
@id {permalink=true}: ASC153
@summary: A page's content type marks a field `role = "review-date"`, and the date the page gives it is before the day the check runs as: today in UTC, or `ASCRIBE_TODAY`.

Advice · file level · next step: review · configurable in `[checks]` · acknowledged in the page's `intended` frontmatter · [SPEC §7.2]({repo}/blob/main/SPEC.md#72-declarations)

**When:** A page's content type marks a field `role = "review-date"`, and the date the page gives it is before the day the check runs as: today in UTC, or `ASCRIBE_TODAY`.

**Message:** this page was due for review on \{date}; check it still holds, then move `{field}` to the next review

**Fix:** Read the page against what it describes and correct what has changed, then set its review date to when it's next due. If the page is right as it is, moving the date is the whole fix. The check compares with today's date in UTC (or `ASCRIBE_TODAY`), so it can start reporting without a change to the project.

**Read more:** [Field roles](../content-model/index.md#54-field-roles) in the `ascribe.toml` reference.

### ASC192 `title-heading-differs`
@id {permalink=true}: ASC192
@summary: A page writes its own level-1 heading, and neither it nor the page's title contains the other, ignoring case. A page without a level-1 heading, which most pages are, since the layout draws the title as one, isn't looked at.

Advice · page level · next step: review · configurable in `[checks]` · acknowledged in the page's `intended` frontmatter · [SPEC §2.1]({repo}/blob/main/SPEC.md#21-files)

**When:** A page writes its own level-1 heading, and neither it nor the page's title contains the other, ignoring case. A page without a level-1 heading, which most pages are, since the layout draws the title as one, isn't looked at.

**Message:** the title, `{title}`, says something the page's own top heading, `{heading}`, doesn't; a search engine may show either one as the page's name, so a reader can be promised one page and land on what reads like another

**Fix:** Make the two agree: change the title or the heading so one says what the other does, or remove the heading and let the layout draw the title, as it does on most pages. When they differ on purpose, as a landing page's title is often the site's name and its heading a tagline, acknowledge it in the page's `intended` frontmatter, saying why.

**Read more:** [`[checks]`](../content-model/checks.md#19-checks) in the `ascribe.toml` reference.

### ASC195 `template-placeholder-left`
@id {permalink=true}: ASC195
@summary: A page of a content type that names a template still has one of the template's placeholders, or a frontmatter field with the template's value.

Advice · file level · next step: write · configurable in `[checks]` · [SPEC §7.2]({repo}/blob/main/SPEC.md#72-declarations)

**When:** A page of a content type that names a template still has one of the template's placeholders, exactly as the template has it but for how its lines are wrapped, or a frontmatter field with the template's value. A memo the page keeps from the template, its instructions to the writer, isn't reported. A placeholder's opening brace is escaped on the page as in the template, so a phrase of the same name, or a placeholder quoted in inline code or a code block, isn't one. A page of a type with no template isn't looked at, and a placeholder changed by a word isn't found.

**Message:** this page still has its template's placeholder `{placeholder}`; write what it asks for in its place

**Fix:** Write the page's own text where the template's placeholder is, the text in braces. The template's instructions are memos, which are never published: follow each, then delete it or keep it. A field the template fills in, such as `title`, gets the page's own value. The check finds only what was never touched: a placeholder with a word changed isn't reported, so read the page through before publishing it.

**Read more:** [Page types](../content-model/index.md#41-page-types) in the `ascribe.toml` reference.

### ASC198 `description-link`
@id {permalink=true}: ASC198
@summary: A page's description, the field its type marks `role = "description"`, holds a Markdown link.

Advice · file level · next step: write · configurable in `[checks]` · [SPEC §9.4]({repo}/blob/main/SPEC.md#94-outputs)

**When:** A page's description, the field its type marks `role = "description"`, holds a Markdown link.

**Message:** a description is plain text, so `{link}` isn't a link: a search result and a link preview show its brackets and destination, and `ascribe check` and `ascribe rename` don't see it

**Fix:** Keep the link's text and remove the brackets and the destination; for a description that isn't quoted, the editor's quick fix does it. Link to the page from the page's body instead, where it's a link that's checked and kept up to date.

**Read more:** [Field roles](../content-model/index.md#54-field-roles) in the `ascribe.toml` reference.

### ASC220 `link-into-older-line`
@id {permalink=true}: ASC220
@summary: A link in the current line of a versioned build names a page the current line doesn't publish, and nothing takes its place there, so it leads to the page in an older line.

Advice · page level · next step: write · configurable in `[checks]` · [SPEC §9.3]({repo}/blob/main/SPEC.md#93-build-modes)

**When:** A link in the current line of a versioned build names a page the current line doesn't publish, and nothing takes its place there, so it leads to the page in an older line.

**Message:** `{path}` isn't in the current line, so this link leads to it in \{line}; the current line usually needs a page in its place

**Fix:** Give the current line a page in its place: mark the page so the current line has it, or write the page that takes its place and give it `replaces`. When the link is meant to lead to an older line, as on a page about migrating from it, leave it.

**Read more:** [Links between lines](../writing/inline.md#links-between-lines) in the phrases and links reference.

### ASC221 `link-fragment-dropped`
@id {permalink=true}: ASC221
@summary: A link to a heading leads, in a line of a versioned build, to a page that takes its target's place or to its target in another line, and the page it lands on has no heading with that id.

Advice · page level · next step: write · configurable in `[checks]` · [SPEC §9.3]({repo}/blob/main/SPEC.md#93-build-modes)

**When:** A link to a heading leads, in a line of a versioned build, to a page that takes its target's place or to its target in another line, and the page it lands on has no heading with that id.

**Message:** in `{build}`, this link leads to `{page}`, which has no heading `{id}`, so it goes to the top of the page

**Fix:** Give the heading on the page it lands on the same id, with `@id`, or link to the page without `#` and the id.

**Read more:** [Links between lines](../writing/inline.md#links-between-lines) in the phrases and links reference.

### ASC223 `html-comment`
@id {permalink=true}: ASC223
@summary: A page or a fragment has an HTML comment, as a block or inside a paragraph. A comment in a code block or a memo isn't one, and Vale's own comments, such as `<!-- vale off -->`, aren't reported.

Advice · file level · next step: review · configurable in `[checks]` · acknowledged with `@intended` above the block, or under the heading for a heading · [SPEC §4.10]({repo}/blob/main/SPEC.md#410-memo)

**When:** A page or a fragment has an HTML comment, as a block or inside a paragraph. A comment in a code block or a memo isn't one, and Vale's own comments, such as `<!-- vale off -->`, aren't reported.

**Message:** an HTML comment is in the published page's source; a memo isn't

**Fix:** An HTML comment isn't private: it's in the source of every published page, and in the JSON output. Write it as a `@memo` instead, which keeps the same text beside the same block and out of every output: the editor's quick fix, **Rewrite as a memo**, does it for a comment that's a block of its own. A comment written for another tool, which has to stay a comment, is acknowledged with `@intended {check=html-comment}` above its block.

**Read more:** [`@memo`](../writing/directives.md#memo) in the directive reference.

### ASC224 `memo-empty`
@id {permalink=true}: ASC224
@summary: A container memo has nothing between its opening line and its `@end`, or only blank lines.

Advice · file level · next step: fix · configurable in `[checks]` · [SPEC §4.10]({repo}/blob/main/SPEC.md#410-memo)

**When:** A container memo has nothing between its opening line and its `@end`, or only blank lines.

**Message:** this memo is empty

**Fix:** A memo that says nothing is almost always a slip: write what it was meant to say between `@memo:` and `@end`, or remove it. The fix removes it, which changes no output, since no output shows a memo.

**Read more:** [`@memo`](../writing/directives.md#memo) in the directive reference.

### ASC229 `template-section-empty`
@id {permalink=true}: ASC229
@summary: A page of a content type that names a template has a section the template gives only memos, and the page's section still has nothing but memos and headings.

Advice · file level · next step: write · configurable in `[checks]` · [SPEC §7.2]({repo}/blob/main/SPEC.md#72-declarations)

**When:** A page of a content type that names a template has a section the template gives nothing under its heading but memos, with the same heading at the same level, and nothing in the page's section, its subsections included, but memos, directives bound to its heading such as `@id`, and the subsections' headings. A section the template gives any other content is `template-placeholder-left`'s, and a page of a type with no template isn't looked at.

**Message:** the template's section `{heading}` is still empty; write it, or remove the heading if this page doesn't need it

**Fix:** Write the section's text under its heading. The template gives it only memos, its instructions to the writer, which are never published, so left as it is the section builds as a heading with nothing under it. Follow each memo, then delete it or keep it. If the page doesn't need the section, remove its heading and its memos.

**Read more:** [Page types](../content-model/index.md#41-page-types) in the `ascribe.toml` reference.

### ASC237 `nav-label-long`
@id {permalink=true}: ASC237
@summary: A page's `nav-label` is longer than its `title`, each counted in characters as a reader sees it.

Advice · file level · next step: write · configurable in `[checks]` · [SPEC §2.1]({repo}/blob/main/SPEC.md#21-files)

**When:** A page's `nav-label` is longer than its `title`, each counted in characters as a reader sees it.

**Message:** this `nav-label` is \{length} characters, longer than the \{title}-character title it stands for in the sidebar

**Fix:** A `nav-label` is a page's short name in a list, a sidebar or previous and next, where the title is too long to read at a glance. Shorten it, or remove it, and the title is the page's name there too.

**Read more:** [A page's label](../navigation/index.md#a-page-s-label) in the navigation reference.

### ASC241 `glossary-definition-long`
@id {permalink=true}: ASC241
@summary: A term's definition, as a build derives it, is longer than the limit `description-too-long` measures a description against.

Advice · page level · next step: write · configurable in `[checks]` · [SPEC §5.4]({repo}/blob/main/SPEC.md#54-glossary-terms)

**When:** A term's definition, as a build derives it, is longer than the limit `description-too-long` measures a description against.

**Message:** the definition of the term "\{term}" is over \{limit} characters in build `{build}`, too long for a link's title or a hover

**Fix:** End the first paragraph after the definition, in a sentence or two, and put the rest in a second paragraph. The first paragraph is what a link to the term shows as its title and the editor shows on hover; the rest of the section is still on the glossary page. The length is counted on the definition as each build derives it, with phrases substituted and formatting dropped.

**Read more:** [`[glossary]`](../content-model/content.md#12-glossary) in the `ascribe.toml` reference.

### ASC252 `summary-missing`
@id {permalink=true}: ASC252
@summary: A heading's `@id` marks it a permalink, and its section has no `@summary`, once includes are expanded. A page's own permalink has its description, which `page-description-missing` reports; a permalink in a fragment is reported once, in the fragment.

Advice · page level · next step: write · configurable in `[checks]` · [SPEC §4.12]({repo}/blob/main/SPEC.md#412-summary)

**When:** A heading's `@id` marks it a permalink, and its section has no `@summary`, once includes are expanded. A page's own permalink has its description, which `page-description-missing` reports; a permalink in a fragment is reported once, in the fragment.

**Message:** the permalink `{name}` has no summary, so a product showing its help has none; add a `@summary` to its section

**Fix:** Give the section a summary with `@summary`: a sentence or two a product shows in a tooltip or beside an error. Put `@summary` on the line above a paragraph of the section that reads well alone, so it's written once and shown in both places, or write `@summary: <text>` directly under the heading and its `@id` when nothing on the page reads well alone. A project that ships embedded help can raise this check to a warning, so every permalink needs one: `[checks] summary-missing = "warning"`.

**Read more:** [`@summary`](../writing/directives.md#summary) in the directive reference.

### ASC253 `summary-long`
@id {permalink=true}: ASC253
@summary: A section's summary, as plain text on one line, is as long as the limit or longer: 300 characters, or `[checks.summary-long] limit`. A summary in a fragment is reported once, in the fragment.

Advice · page level · next step: write · configurable in `[checks]` · [SPEC §4.12]({repo}/blob/main/SPEC.md#412-summary)

**When:** A section's summary, as plain text on one line, is as long as the limit or longer: 300 characters, or `[checks.summary-long] limit`. A summary in a fragment is reported once, in the fragment.

**Message:** this summary is \{length} characters, and a product shows one under \{limit}; shorten it to a sentence or two

**Fix:** Shorten the summary to a sentence or two that say what the section is for. A product shows it in a tooltip or beside an error, where there's room for little more. Its length is counted as `description-too-long` counts a description's: as plain text on one line, with phrases replaced and formatting dropped. The limit is 300 characters; set another in `ascribe.toml`, such as `[checks.summary-long] limit = 200`.

**Read more:** [`@summary`](../writing/directives.md#summary) in the directive reference.

## Across the project

### ASC154 `page-orphan`
@id {permalink=true}: ASC154
@summary: No other page the build publishes links to the page or includes it. Index pages (`index.md`), and a build's only page, aren't reported. The message says what reaches the page: only the sidebar, or, for a page that's `unlisted`, nothing but its address.

Advice · page level · next step: review · configurable in `[checks]` · acknowledged in the page's `intended` frontmatter · [SPEC §5.2]({repo}/blob/main/SPEC.md#52-links)

**When:** No other page the build publishes links to the page or includes it. Index pages (`index.md`), and a build's only page, aren't reported. The message says what reaches the page: only the sidebar, or, for a page that's `unlisted`, nothing but its address.

**Message:** no other page links to this page or includes it, so readers reach it only through the sidebar

**Fix:** Link to the page from a page about its topic, or include it in one, so a reader of that page finds it. A page in the navigation is still in the sidebar, so this is a hint: acknowledge one that needs no link in its `intended` frontmatter, or turn the check off in [`[checks]`](../content-model/checks.md#19-checks) (`page-orphan = "off"`). A page that's `listing: unlisted` is in no sidebar, so only someone given its address reaches it: link to it, or take `listing: unlisted` away to put it in the sidebar.

**Read more:** [Links](../writing/inline.md#links) in the phrases and links reference.

### ASC155 `fragment-unused`
@id {permalink=true}: ASC155
@summary: No file includes the fragment.

Advice · file level · next step: review · configurable in `[checks]` · acknowledged in the page's `intended` frontmatter · [SPEC §4.2]({repo}/blob/main/SPEC.md#42-include)

**When:** No file includes the fragment.

**Message:** no file includes this fragment, so nothing publishes it

**Fix:** Include the fragment where it belongs with `@include`, or delete it when nothing needs it any more.

**Read more:** [`@include`](../writing/directives.md#include) in the directive reference.

### ASC156 `phrase-unused`
@id {permalink=true}: ASC156
@summary: No file writes the phrase, in its text or in a frontmatter field that takes phrases.

Advice · file level · next step: review · configurable in `[checks]` · acknowledged in `[[intended]]` · [SPEC §5.1]({repo}/blob/main/SPEC.md#51-phrases)

**When:** No file writes the phrase, in its text or in a frontmatter field that takes phrases.

**Message:** no file uses the phrase `{key}`

**Fix:** Write the phrase where its value belongs, or remove it from `[phrases]`. A phrase is used wherever a page or a fragment writes its key in braces, in the text or in a frontmatter field that takes phrases.

**Read more:** [`[phrases]`](../content-model/content.md#phrases) in the `ascribe.toml` reference.

### ASC157 `feature-unused`
@id {permalink=true}: ASC157
@summary: No availability spec names the feature: no `@available`, table row's `available`, or page's `available`.

Advice · file level · next step: review · configurable in `[checks]` · acknowledged in `[[intended]]` · [SPEC §4.4]({repo}/blob/main/SPEC.md#44-available)

**When:** No availability spec names the feature: no `@available`, table row's `available`, or page's `available`.

**Message:** no file uses the feature `{key}`

**Fix:** Mark the content the feature covers with its key (`@available: <key>`, or `available: <key>` in a page's frontmatter), or remove the feature from `[features]` once nothing needs it.

**Read more:** [`[features.<key>]`](../content-model/content.md#9-featureskey) in the `ascribe.toml` reference.

### ASC158 `glossary-term-unused`
@id {permalink=true}: ASC158
@summary: The term, and each of its aliases, appears in no other page's prose, and no link names its heading.

Advice · file level · next step: review · configurable in `[checks]` · acknowledged with `@intended` above the block, or under the heading for a heading · [SPEC §5.4]({repo}/blob/main/SPEC.md#54-glossary-terms)

**When:** The term, and each of its aliases, appears in no other page's prose, and no link names its heading.

**Message:** the glossary term `{term}` appears in no other file's prose, and no link names its heading, so it's never linked

**Fix:** Check the term and its `aliases` against how the pages write it: occurrences match whole words, in prose only, and ignore case unless the term is case-sensitive. Remove the term when the pages don't need it. A term with `match = "marked"` is used only where a link names its heading: link the occurrences that mean it, as `[build](glossary.md#build)`. A term used only inside a phrase's value is reported too, since phrase values aren't searched: acknowledge it with [`@intended`](../writing/directives.md#intended) at the top of the term's section, under its heading.

**Read more:** [Terms](../content-model/content.md#122-terms) in the `ascribe.toml` reference.

### ASC159 `image-unused`
@id {permalink=true}: ASC159
@summary: An image file under the content root that no file shows or links to: no Markdown image or link, no `src`, `srcset`, `href`, or `poster` in raw HTML, and no frontmatter string names it.

Advice · file level · next step: review · configurable in `[checks]` · acknowledged in `[[intended]]` · [SPEC §5.3]({repo}/blob/main/SPEC.md#53-images)

**When:** An image file under the content root that no file shows or links to: no Markdown image or link, no `src`, `srcset`, `href`, or `poster` in raw HTML, and no frontmatter string names it.

**Message:** no file shows or links to `{path}`

**Fix:** Delete the image when nothing needs it. A frontmatter string counts when it names the image from the file or from the content root, as `cover: img/card.png` does. An image used only by the site's own code is reported too, since Ascribe doesn't read it: turn the check off in [`[checks]`](../content-model/checks.md#19-checks) when your site uses images that way.

**Read more:** [Images](../writing/inline.md#images) in the phrases and links reference.

### ASC160 `image-large`
@id {permalink=true}: ASC160
@summary: An image file under the content root is larger than the limit: 500 KB, or `[checks.image-large] limit`.

Advice · file level · next step: review · configurable in `[checks]` · acknowledged in `[[intended]]` · [SPEC §5.3]({repo}/blob/main/SPEC.md#53-images)

**When:** An image file under the content root is larger than the limit: 500 KB, or `[checks.image-large] limit`.

**Message:** `{path}` is \{size}, over the \{limit} limit for an image

**Fix:** Make the image smaller: save it at the size it's shown, or compress it (a screenshot as an optimized PNG, a photo as JPEG or WebP). The limit is 500 KB; set another in `ascribe.toml`, such as `[checks.image-large] limit = "1 MB"`.

**Read more:** [`[checks]`](../content-model/checks.md#19-checks) in the `ascribe.toml` reference.

### ASC161 `title-duplicate`
@id {permalink=true}: ASC161
@summary: Two pages the build publishes have the same title, ignoring case.

Advice · page level · next step: write · configurable in `[checks]` · [SPEC §2.2]({repo}/blob/main/SPEC.md#22-pages-and-fragments)

**When:** Two pages the build publishes have the same title, ignoring case.

**Message:** `{title}` is also the title of \{others}, so readers can't tell the pages apart in search results or a list of pages

**Fix:** Give each page a title that says what sets it apart. When one page replaces the other, remove the old one, or publish only one of them in each build with `available`.

**Read more:** [`[checks]`](../content-model/checks.md#19-checks) in the `ascribe.toml` reference.

### ASC172 `availability-left-behind`
@id {permalink=true}: ASC172
@summary: A feature's availability names a version for every target, each target lists its lines in `[releases]`, and no version is later than its target's current line.

Advice · file level · next step: review · configurable in `[checks]` · acknowledged in `[[intended]]` · [SPEC §4.4]({repo}/blob/main/SPEC.md#44-available)

**When:** A feature's availability names a version for every target, each target lists its lines in `[releases]`, and no version is later than its target's current line.

**Message:** feature `{key}` names releases up to \{version}, and the current line of `{target}` is \{current}, so what it marks has shipped

**Fix:** When the feature stood for what a release would add, write the shipped release's spec at the content that shipped in it (`@available: ascribe 0.2.0`), and mark what the next release adds `<target> planned` instead of keeping a feature for it. When it's a history worth keeping by name, acknowledge it in [`[[intended]]`](../content-model/checks.md#intended-acknowledgements). After each release, move its line from `planned` to `live` in [`[releases.<target>]`](../content-model/content.md#71-releasestarget).

**Read more:** [`[releases.<target>]`](../content-model/content.md#71-releasestarget) in the `ascribe.toml` reference.

### ASC191 `description-duplicate`
@id {permalink=true}: ASC191
@summary: Two pages the build publishes have the same description, ignoring case.

Advice · page level · next step: write · configurable in `[checks]` · [SPEC §9.4]({repo}/blob/main/SPEC.md#94-outputs)

**When:** Two pages the build publishes have the same description, ignoring case.

**Message:** this is also the description of \{others}, so a search result, a link preview, and `llms.txt` say the same of each page, and a reader can't tell which one they want

**Fix:** Give each page a description that says what it's for and what sets it apart from the others. When one page replaces the other, remove the old one, or publish only one of them in each build with `available`.

**Read more:** [Field roles](../content-model/index.md#54-field-roles) in the `ascribe.toml` reference.

### ASC211 `available-line-undeclared`
@id {permalink=true}: ASC211
@summary: A version is in a line between two of its target's declared lines, and that line isn't declared.

Advice · file level · next step: choose · configurable in `[checks]` · [SPEC §4.4]({repo}/blob/main/SPEC.md#44-available)

**When:** A version is in a line between two of its target's declared lines, and that line isn't declared.

**Message:** `{version}` is in the line `{line}`, which `[releases.{target}]` doesn't declare; declare it, or correct the number to `{older}` or `{newer}`

**Fix:** Declare the line in `[releases.<target>]`, in the list where it falls: `live` for a line readers are on, `planned` for one newer than every live line that hasn't shipped, or, for one older than every live line that shipped and that nothing publishes now, `archived` with no address, as `"3.3" = {}`. Or correct the number to a declared line, such as the one either side of it.

**Read more:** [`[releases.<target>]`](../content-model/content.md#71-releasestarget) in the `ascribe.toml` reference.

### ASC212 `available-same-in-every-line`
@id {permalink=true}: ASC212
@summary: Every live line has the content a mark is on, since every version it names is at or before the oldest live line, and no output shows the versions.

Advice · file level · next step: write · configurable in `[checks]` · [SPEC §4.4]({repo}/blob/main/SPEC.md#44-available)

**When:** Every version a mark names for its target is at or before the oldest live line and its last state counts as available, so every live line has the content, and no build filtered for the target at a version before the window publishes it: no output shows the versions, or a build that badges availability shows them and they're all before the oldest live line, so the badge says only that the content is older than every live line.

**Message:** `{entry}` names versions every live line of `{target}` has, the oldest being `{oldest}`, so no output shows them; write `{simpler}`

**Fix:** Write the mark without its versions, keeping its state: `self-managed 3.1` becomes `self-managed`, and `self-managed deprecated 3.2` becomes `self-managed deprecated`. Every live line has the content, and a line's output leaves out a version at or before the oldest live line, so where no output shows the versions nothing a reader sees changes. When the mark would be its target alone, generally available, the target is the only value of its dimension, no build filters for another target, and the mark names nothing else, the fix removes the mark instead: the `@available` line, the page's `available` key, or the row's `{available=…}`. Content with no mark reads as the scope around it, so a mark is removed only when no mark around it says something, as a page that's deprecated or a section in preview does, and never in a fragment, which reads as where it's included; there, as everywhere else, the fix rewrites it. In a build that badges availability, which is reported only for versions before the oldest live line, the badge's version goes, or the whole badge with the mark, which changes the page, so the fix isn't safe; a project that keeps such badges, as "new in 0.2", turns the check off in `[checks]`.

**Read more:** [`[releases.<target>]`](../content-model/content.md#71-releasestarget) in the `ascribe.toml` reference.

### ASC213 `available-in-no-line`
@id {permalink=true}: ASC213
@summary: Every build that publishes a mark's content is versioned by its target, or filtered for it at a version in the window or after it, and the content is in no live or planned line of the target.

Warning · file level · next step: write · configurable in `[checks]` · [SPEC §4.4]({repo}/blob/main/SPEC.md#44-available)

**When:** Every build that publishes a mark's content is versioned by its target, or filtered for it at a version in the window or after it, and the content is in no live or planned line of the target.

**Message:** `{entry}` leaves the content out of every live and planned line of `{target}`, the oldest being `{oldest}`, so no output has it

**Fix:** Remove the content, which no line's output publishes, or, if a line still has it, correct the mark. A mark that makes content unavailable from a version before the oldest live line, such as `self-managed (ga 3.0, removed 3.2)` with 3.4 the oldest, leaves it in no line.

**Read more:** [`[releases.<target>]`](../content-model/content.md#71-releasestarget) in the `ascribe.toml` reference.

### ASC214 `release-line-ended`
@id {permalink=true}: ASC214
@summary: A live line's date in `[releases.<target>.ends]` is before the day the check runs as: today in UTC, or `ASCRIBE_TODAY`.

Advice · file level · next step: write · configurable in `[checks]` · [SPEC §7.2]({repo}/blob/main/SPEC.md#72-declarations)

**When:** A live line's date in `[releases.<target>.ends]` is before the day the check runs as: today in UTC, or `ASCRIBE_TODAY`.

**Message:** support for `{target}` \{line} ended on \{date}, and the line is still live

**Fix:** Move the line from `live` to `archived` in `[releases.<target>]`, with its date as `ended` and, when it's still readable somewhere, its `url`; or, if its support goes on, change its date in `ends`.

**Read more:** [`[releases.<target>]`](../content-model/content.md#71-releasestarget) in the `ascribe.toml` reference.

### ASC227 `memo-outdated`
@id {permalink=true}: ASC227
@summary: No block of what a memo file's `about` names holds its `quote`.

Advice · file level · next step: write · configurable in `[checks]` · [SPEC §7.7]({repo}/blob/main/SPEC.md#77-memo-files)

**When:** No block of what a memo file's `about` names holds its `quote`.

**Message:** no block of `{about}` has the words this memo quotes, so it's on the \{place} and no block

**Fix:** The words were changed since the memo was written. Read the memo against the page as it is: if it still holds, quote the words it's about now; if it doesn't, change it or delete it. A quote is found in one block, with any run of spaces and line breaks read as one space.

**Read more:** [Placed, outdated, and detached](../writing/memo-files.md#placed-outdated-and-detached) in the memo files reference.

### ASC228 `memo-long`
@id {permalink=true}: ASC228
@summary: A memo file's body is longer than the limit: 2,000 characters, or `[checks.memo-long] limit`.

Advice · file level · next step: write · configurable in `[checks]` · [SPEC §7.7]({repo}/blob/main/SPEC.md#77-memo-files)

**When:** A memo file's body is longer than the limit: 2,000 characters, or `[checks.memo-long] limit`.

**Message:** this memo is \{length} characters, over the \{limit} limit

**Fix:** A memo is read by whoever is about to change the page, so it says why in as few words as it can. Cut what the page itself says, and split a memo about several things into one about each. The limit is 2,000 characters; set another in `ascribe.toml`, such as `[checks.memo-long] limit = 4000`.

**Read more:** [Write a memo file](../writing/memo-files.md#write-a-memo-file) in the memo files reference.

### ASC235 `nav-page-unplaced`
@id {permalink=true}: ASC235
@summary: `ascribe.nav` lists `pages` for a page's section and doesn't name the page. A section with no `pages` has the default order, and its pages aren't reported, nor is a section's own page, nor an `unlisted` page, which no sidebar shows.

Advice · file level · next step: fix · configurable in `[checks]` · [SPEC §7.8]({repo}/blob/main/SPEC.md#78-navigation)

**When:** `ascribe.nav` lists `pages` for a page's section and doesn't name the page. A section with no `pages` has the default order, and its pages aren't reported, nor is a section's own page, nor an `unlisted` page, which no sidebar shows.

**Message:** `{folder}`'s `pages` in ascribe.nav doesn't name this page, so it comes after the pages it does, by title

**Fix:** Add the page's file name to the section's `pages` in `ascribe.nav`, where it should come. The fix appends the section's pages that aren't named, in the order they come in now, so the navigation is the same after it; move the names to where you want them.

**Read more:** [The file](../navigation/index.md#the-file) in the navigation reference.

### ASC236 `nav-section-unplaced`
@id {permalink=true}: ASC236
@summary: `ascribe.nav` lists some of a section's sibling sections and not the section, which is reported on its first page.

Advice · file level · next step: fix · configurable in `[checks]` · [SPEC §7.8]({repo}/blob/main/SPEC.md#78-navigation)

**When:** `ascribe.nav` lists one or more of a section's sibling sections and not the section, which is reported on its first page. A folder none of whose sections the file lists has the default order, and isn't reported, nor is a section below the content root whose own page beside its folder places it among its parent's pages, since a table wouldn't move it.

**Message:** ascribe.nav lists other sections in `{parent}` and not `{folder}`, so it comes after them, in path order

**Fix:** Add a `[[sections]]` table with the section's `folder` to `ascribe.nav`, where the section should come. The fix appends a table for each section that isn't listed, in the order they come in now, so the navigation is the same after it; move the tables to where you want them. A file that writes `sections = [...]` inline gets no fix: add each section to its list.

**Read more:** [The file](../navigation/index.md#the-file) in the navigation reference.

### ASC243 `baseline-entry-unmatched`
@id {permalink=true}: ASC243
@summary: `ascribe.baseline` has an entry that no problem of the project matches any more: the problem was fixed, or the text it was about changed. Reported at the entry's line, when `ascribe check` reads the baseline and checks the whole project for every build.

Advice · file level · next step: outside · configurable in `[checks]` · [SPEC §2.1]({repo}/blob/main/SPEC.md#21-files)

**When:** `ascribe.baseline` has an entry that no problem of the project matches any more: the problem was fixed, or the text it was about changed. Reported at the entry's line, when `ascribe check` reads the baseline and checks the whole project for every build.

**Message:** ascribe.baseline holds back `{check}` in `{file}`, and nothing there matches it any more; run `ascribe baseline prune`

**Fix:** Run `ascribe baseline prune`, which removes every entry that matches nothing, and commit ascribe.baseline. Left in the file, the entry would hold back a later problem of the same check whose text happens to be the same. See [The baseline](../checks/baseline.md).

**Read more:** [`[checks]`](../content-model/checks.md#19-checks) in the `ascribe.toml` reference.

### ASC244 `frontmatter-field-repeated`
@id {permalink=true}: ASC244
@summary: A type's own `frontmatter` declares a field exactly as `[frontmatter]` does, key for key, so the type's copy changes nothing. Reported at the type's field.

Advice · file level · next step: fix · configurable in `[checks]` · [SPEC §7.2]({repo}/blob/main/SPEC.md#72-declarations)

**When:** A type's own `frontmatter` declares a field exactly as `[frontmatter]` does, key for key, so the type's copy changes nothing. Reported at the type's field.

**Message:** the type `{type}` declares `{field}` exactly as `[frontmatter]` does, so this copy changes nothing; delete it, and the type has `[frontmatter]`'s

**Fix:** Delete the type's field, and the type has the one in `[frontmatter]`, which is the same. The fix deletes its line, with the comment lines directly above it, when it's written as `key = value` under the table's header; one written another way is deleted by hand. A type's field that differs from the shared one in any key replaces it whole, for that type, and isn't reported: keep it when the type needs it to differ.

**Read more:** [Page types](../content-model/index.md#41-page-types) in the `ascribe.toml` reference.

### ASC257 `addresses-base-unreadable`
@id {permalink=true}: ASC257
@summary: The project has `ascribe.addresses`, and `ascribe check` can't read the base it compares the addresses with: there's no git repository, or the base isn't there, as in a shallow clone. A base with no project published nothing, so that's advice, not a warning. Reported once, at `ascribe.addresses`.

Warning · file level · next step: outside · configurable in `[checks]` · [SPEC §7.10]({repo}/blob/main/SPEC.md#710-the-addresses-file)

**When:** The project has `ascribe.addresses`, and `ascribe check` can't read the base it compares the addresses with: there's no git repository, or the base isn't there, as in a shallow clone. A base with no project published nothing, so that's advice, not a warning. Reported once, at `ascribe.addresses`.

**Message:** the addresses weren't compared with the base: \{reason}

**Fix:** Give the check a base it can read. In CI, fetch the history (`fetch-depth: 0` on actions/checkout), and pass the branch a pull request targets, `--base origin/$GITHUB_BASE_REF`, or, on a push, the commit before it, `--base ${{ github.event.before }} --base-exact`. A base with no project, as a migration's first pull request has, published nothing, so there's nothing to compare: it's reported, as advice, so a check that compared nothing never passes silently. See [The base](../publishing/redirects.md#the-base).

**Read more:** [The base](../publishing/redirects.md#the-base) in the guide to keeping addresses.

### ASC282 `api-operation-unreferenced`
@id {permalink=true}: ASC282
@summary: An operation of a declared spec, not marked `deprecated`, that no link names, by its id or by its method and path.

Advice · file level · next step: write · configurable in `[checks]` · [SPEC §7.11]({repo}/blob/main/SPEC.md#711-apis)

**When:** An operation of a declared spec, not marked `deprecated`, that no link names, by its id or by its method and path.

**Message:** no page links to `{operation}` (\{method} \{route}) in \{path}

**Fix:** Link to the operation from the guide that covers it, by its `operationId`, `openapi.yaml#getUser`, or by its method and path when it has none, `openapi.yaml#delete/users/{id}`. An operation a guide doesn't need to cover can be marked `deprecated` in the spec when it's on its way out, which leaves it out; for an API whose reference is enough on its own, turn the check off in [`[checks]`](../content-model/checks.md#19-checks).

**Read more:** [`[apis.<name>]`](../content-model/checks.md#23-apisname) in the `ascribe.toml` reference.
