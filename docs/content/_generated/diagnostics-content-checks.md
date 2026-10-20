## Pages
@memo: Generated from tests/conformance/diagnostics.toml by tests/conformance/tests/docs.rs; edit the registry, then run `ASCRIBE_BLESS=1 cargo test -p ascribe-conformance --test docs` to rewrite it.

### ASC149 `page-size`
@id {permalink=true}: ASC149

Advice · page level · next step: write · configurable in `[checks]` · [SPEC §9.4]({repo}/blob/main/SPEC.md#94-outputs)

**When:** A page's Markdown, as a build writes it for agents to read, is as long as the limit or longer: 50,000 characters, or `[checks.page-size] limit`. A page over the limit in several builds is one problem, with its size in the build where it's longest.

**Message:** this page's Markdown is \{size} characters in build `{build}`, and an agent reads under \{limit} in one piece; split the page, or move long tables and code below the prose

**Fix:** Split the page into pages that each answer one question, or move long tables, reference lists, and code below the prose, where an agent that stops reading early loses the least. The limit is the Web Documentation Delivery Spec's: a page passes under 50,000 characters of Markdown. Set another with `[checks.page-size] limit`. It's the spec's `page-size-markdown` check, made from the build before a deploy; `ascribe report agents` names this check for it, and for the spec's other checks of a page's size.

**Read more:** [`[checks]`](../content-model/checks.md#19-checks) in the `ascribe.toml` reference.

### ASC150 `page-description-missing`
@id {permalink=true}: ASC150

Advice · file level · next step: write · configurable in `[checks]` · [SPEC §7.2]({repo}/blob/main/SPEC.md#72-declarations)

**When:** A page's content type marks a field `role = "description"`, and the page leaves the field out or empty. A required field, whose absence is `frontmatter-missing-field`, and a field with a default aren't looked at.

**Message:** this page has no `{field}`; write a sentence saying what the page is for, which search results and agents show with its title

**Fix:** Add the field the content type marks with `role = "description"` to the page's frontmatter: a sentence or two saying what the page covers and who it's for.

**Read more:** [Field roles](../content-model/index.md#54-field-roles) in the `ascribe.toml` reference.

### ASC151 `heading-level-skipped`
@id {permalink=true}: ASC151

Advice · page level · next step: fix · configurable in `[checks]` · [SPEC §9.4]({repo}/blob/main/SPEC.md#94-outputs)

**When:** A heading is more than one level below the heading before it, or is at level 1, which the page's title is. Headings from includes count at the levels they're written with, and after a skip the next heading is compared with the one that skipped, so a skipped section is reported once.

**Message:** this heading is level \{level}, after a level-\{previous} heading; make it level \{expected}, so the outline has no gap

**Fix:** Make each heading one level below the heading it belongs under. A page's title is its level-1 heading, so its own headings start at level 2. The quick fix changes the one heading; the headings under it may need to move up too.

**Read more:** [`[checks]`](../content-model/checks.md#19-checks) in the `ascribe.toml` reference.

### ASC152 `code-language-missing`
@id {permalink=true}: ASC152

Advice · file level · next step: choose · configurable in `[checks]` · [SPEC §1.4]({repo}/blob/main/SPEC.md#14-relationship-to-commonmark)

**When:** A fenced code block has no language after its opening fence: nothing, or only attributes, such as `phrases=true`. An indented code block isn't looked at.

**Message:** this code block has no language; write it after the opening fence, or `text` for output

**Fix:** Write the block's language right after the opening fence, such as `sh`, `toml`, or `json`. Use `text` for program output, logs, and anything else that isn't code. A highlighter colors the block by it, and an agent knows what the block is.

**Read more:** [`[checks]`](../content-model/checks.md#19-checks) in the `ascribe.toml` reference.

### ASC153 `review-overdue`
@id {permalink=true}: ASC153

Advice · file level · next step: review · configurable in `[checks]` · acknowledged in the page's `intended` frontmatter · [SPEC §7.2]({repo}/blob/main/SPEC.md#72-declarations)

**When:** A page's content type marks a field `role = "review-date"`, and the date the page gives it is before the day the check runs as: today in UTC, or `ASCRIBE_TODAY`.

**Message:** this page was due for review on \{date}; check it still holds, then move `{field}` to the next review

**Fix:** Read the page against what it describes and correct what has changed, then set its review date to when it's next due. If the page is right as it is, moving the date is the whole fix. The check compares with today's date in UTC (or `ASCRIBE_TODAY`), so it can start reporting without a change to the project.

**Read more:** [Field roles](../content-model/index.md#54-field-roles) in the `ascribe.toml` reference.

### ASC192 `title-heading-differs`
@id {permalink=true}: ASC192

Advice · page level · next step: review · configurable in `[checks]` · acknowledged in the page's `intended` frontmatter · [SPEC §2.1]({repo}/blob/main/SPEC.md#21-files)

**When:** A page writes its own level-1 heading, and neither it nor the page's title contains the other, ignoring case. A page without a level-1 heading, which most pages are, since the layout draws the title as one, isn't looked at.

**Message:** the title, `{title}`, says something the page's own top heading, `{heading}`, doesn't; a search engine may show either one as the page's name, so a reader can be promised one page and land on what reads like another

**Fix:** Make the two agree: change the title or the heading so one says what the other does, or remove the heading and let the layout draw the title, as it does on most pages. When they differ on purpose, as a landing page's title is often the site's name and its heading a tagline, acknowledge it in the page's `intended` frontmatter, saying why.

**Read more:** [`[checks]`](../content-model/checks.md#19-checks) in the `ascribe.toml` reference.

### ASC195 `template-placeholder-left`
@id {permalink=true}: ASC195

Advice · file level · next step: write · configurable in `[checks]` · [SPEC §7.2]({repo}/blob/main/SPEC.md#72-declarations)

**When:** A page of a content type that names a template still has one of the template's placeholders, exactly as the template has it but for how its lines are wrapped, or a frontmatter field with the template's value. A memo the page keeps from the template, its instructions to the writer, isn't reported. A placeholder's opening brace is escaped on the page as in the template, so a phrase of the same name, or a placeholder quoted in inline code or a code block, isn't one. A page of a type with no template isn't looked at, and a placeholder changed by a word isn't found.

**Message:** this page still has its template's placeholder `{placeholder}`; write what it asks for in its place

**Fix:** Write the page's own text where the template's placeholder is, the text in braces. The template's instructions are memos, which are never published: follow each, then delete it or keep it. A field the template fills in, such as `title`, gets the page's own value. The check finds only what was never touched: a placeholder with a word changed isn't reported, so read the page through before publishing it.

**Read more:** [Page types](../content-model/index.md#41-page-types) in the `ascribe.toml` reference.

### ASC198 `description-link`
@id {permalink=true}: ASC198

Advice · file level · next step: write · configurable in `[checks]` · [SPEC §9.4]({repo}/blob/main/SPEC.md#94-outputs)

**When:** A page's description, the field its type marks `role = "description"`, holds a Markdown link.

**Message:** a description is plain text, so `{link}` isn't a link: a search result and a link preview show its brackets and destination, and `ascribe check` and `ascribe rename` don't see it

**Fix:** Keep the link's text and remove the brackets and the destination; for a description that isn't quoted, the editor's quick fix does it. Link to the page from the page's body instead, where it's a link that's checked and kept up to date.

**Read more:** [Field roles](../content-model/index.md#54-field-roles) in the `ascribe.toml` reference.

### ASC220 `link-into-older-line`
@id {permalink=true}: ASC220

Advice · page level · next step: write · configurable in `[checks]` · [SPEC §9.3]({repo}/blob/main/SPEC.md#93-build-modes)

**When:** A link in the current line of a versioned build names a page the current line doesn't publish, and nothing takes its place there, so it leads to the page in an older line.

**Message:** `{path}` isn't in the current line, so this link leads to it in \{line}; the current line usually needs a page in its place

**Fix:** Give the current line a page in its place: mark the page so the current line has it, or write the page that takes its place and give it `replaces`. When the link is meant to lead to an older line, as on a page about migrating from it, leave it.

**Read more:** [Links between lines](../writing/directives.md#links-between-lines) in the directive reference.

### ASC221 `link-fragment-dropped`
@id {permalink=true}: ASC221

Advice · page level · next step: write · configurable in `[checks]` · [SPEC §9.3]({repo}/blob/main/SPEC.md#93-build-modes)

**When:** A link to a heading leads, in a line of a versioned build, to a page that takes its target's place or to its target in another line, and the page it lands on has no heading with that id.

**Message:** in `{build}`, this link leads to `{page}`, which has no heading `{id}`, so it goes to the top of the page

**Fix:** Give the heading on the page it lands on the same id, with `@id`, or link to the page without `#` and the id.

**Read more:** [Links between lines](../writing/directives.md#links-between-lines) in the directive reference.

### ASC223 `html-comment`
@id {permalink=true}: ASC223

Advice · file level · next step: review · configurable in `[checks]` · acknowledged with `@intended` above the block, or under the heading for a heading · [SPEC §4.10]({repo}/blob/main/SPEC.md#410-memo)

**When:** A page or a fragment has an HTML comment, as a block or inside a paragraph. A comment in a code block or a memo isn't one, and Vale's own comments, such as `<!-- vale off -->`, aren't reported.

**Message:** an HTML comment is in the published page's source; a memo isn't

**Fix:** An HTML comment isn't private: it's in the source of every published page, and in the JSON output. Write it as a `@memo` instead, which keeps the same text beside the same block and out of every output: the editor's quick fix, **Rewrite as a memo**, does it for a comment that's a block of its own. A comment written for another tool, which has to stay a comment, is acknowledged with `@intended {check=html-comment}` above its block.

**Read more:** [`@memo`](../writing/directives.md#memo) in the directive reference.

### ASC224 `memo-empty`
@id {permalink=true}: ASC224

Advice · file level · next step: fix · configurable in `[checks]` · [SPEC §4.10]({repo}/blob/main/SPEC.md#410-memo)

**When:** A container memo has nothing between its opening line and its `@end`, or only blank lines.

**Message:** this memo is empty

**Fix:** A memo that says nothing is almost always a slip: write what it was meant to say between `@memo:` and `@end`, or remove it. The fix removes it, which changes no output, since no output shows a memo.

**Read more:** [`@memo`](../writing/directives.md#memo) in the directive reference.

### ASC229 `template-section-empty`
@id {permalink=true}: ASC229

Advice · file level · next step: write · configurable in `[checks]` · [SPEC §7.2]({repo}/blob/main/SPEC.md#72-declarations)

**When:** A page of a content type that names a template has a section the template gives nothing under its heading but memos, with the same heading at the same level, and nothing in the page's section, its subsections included, but memos, directives bound to its heading such as `@id`, and the subsections' headings. A section the template gives any other content is `template-placeholder-left`'s, and a page of a type with no template isn't looked at.

**Message:** the template's section `{heading}` is still empty; write it, or remove the heading if this page doesn't need it

**Fix:** Write the section's text under its heading. The template gives it only memos, its instructions to the writer, which are never published, so left as it is the section builds as a heading with nothing under it. Follow each memo, then delete it or keep it. If the page doesn't need the section, remove its heading and its memos.

**Read more:** [Page types](../content-model/index.md#41-page-types) in the `ascribe.toml` reference.

### ASC237 `nav-label-long`
@id {permalink=true}: ASC237

Advice · file level · next step: write · configurable in `[checks]` · [SPEC §2.1]({repo}/blob/main/SPEC.md#21-files)

**When:** A page's `nav-label` is longer than its `title`, each counted in characters as a reader sees it.

**Message:** this `nav-label` is \{length} characters, longer than the \{title}-character title it stands for in the sidebar

**Fix:** A `nav-label` is a page's short name in a list, a sidebar or previous and next, where the title is too long to read at a glance. Shorten it, or remove it, and the title is the page's name there too.

**Read more:** [A page's label](../navigation/index.md#a-page-s-label) in the navigation reference.

### ASC241 `glossary-definition-long`
@id {permalink=true}: ASC241

Advice · page level · next step: write · configurable in `[checks]` · [SPEC §5.4]({repo}/blob/main/SPEC.md#54-glossary-terms)

**When:** A term's definition, as a build derives it, is longer than the limit `description-too-long` measures a description against.

**Message:** the definition of the term "\{term}" is over \{limit} characters in build `{build}`, too long for a link's title or a hover

**Fix:** End the first paragraph after the definition, in a sentence or two, and put the rest in a second paragraph. The first paragraph is what a link to the term shows as its title and the editor shows on hover; the rest of the section is still on the glossary page. The length is counted on the definition as each build derives it, with phrases substituted and formatting dropped.

**Read more:** [`[glossary]`](../content-model/content.md#12-glossary) in the `ascribe.toml` reference.

## Across the project

### ASC154 `page-orphan`
@id {permalink=true}: ASC154

Advice · page level · next step: review · configurable in `[checks]` · acknowledged in the page's `intended` frontmatter · [SPEC §5.2]({repo}/blob/main/SPEC.md#52-links)

**When:** No other page the build publishes links to the page or includes it. Index pages (`index.md`), and a build's only page, aren't reported. The message says what reaches the page: only the sidebar, or, for a page that's `unlisted`, nothing but its address.

**Message:** no other page links to this page or includes it, so readers reach it only through the sidebar

**Fix:** Link to the page from a page about its topic, or include it in one, so a reader of that page finds it. A page in the navigation is still in the sidebar, so this is a hint: acknowledge one that needs no link in its `intended` frontmatter, or turn the check off in [`[checks]`](../content-model/checks.md#19-checks) (`page-orphan = "off"`). A page that's `listing: unlisted` is in no sidebar, so only someone given its address reaches it: link to it, or take `listing: unlisted` away to put it in the sidebar.

**Read more:** [Links](../writing/directives.md#links) in the directive reference.

### ASC155 `fragment-unused`
@id {permalink=true}: ASC155

Advice · file level · next step: review · configurable in `[checks]` · acknowledged in the page's `intended` frontmatter · [SPEC §4.2]({repo}/blob/main/SPEC.md#42-include)

**When:** No file includes the fragment.

**Message:** no file includes this fragment, so nothing publishes it

**Fix:** Include the fragment where it belongs with `@include`, or delete it when nothing needs it any more.

**Read more:** [`@include`](../writing/directives.md#include) in the directive reference.

### ASC156 `phrase-unused`
@id {permalink=true}: ASC156

Advice · file level · next step: review · configurable in `[checks]` · acknowledged in `[[intended]]` · [SPEC §5.1]({repo}/blob/main/SPEC.md#51-phrases)

**When:** No file writes the phrase, in its text or in a frontmatter field that takes phrases.

**Message:** no file uses the phrase `{key}`

**Fix:** Write the phrase where its value belongs, or remove it from `[phrases]`. A phrase is used wherever a page or a fragment writes its key in braces, in the text or in a frontmatter field that takes phrases.

**Read more:** [`[phrases]`](../content-model/content.md#phrases) in the `ascribe.toml` reference.

### ASC157 `feature-unused`
@id {permalink=true}: ASC157

Advice · file level · next step: review · configurable in `[checks]` · acknowledged in `[[intended]]` · [SPEC §4.4]({repo}/blob/main/SPEC.md#44-available)

**When:** No availability spec names the feature: no `@available`, table row's `available`, or page's `available`.

**Message:** no file uses the feature `{key}`

**Fix:** Mark the content the feature covers with its key (`@available: <key>`, or `available: <key>` in a page's frontmatter), or remove the feature from `[features]` once nothing needs it.

**Read more:** [`[features.<key>]`](../content-model/content.md#9-featureskey) in the `ascribe.toml` reference.

### ASC158 `glossary-term-unused`
@id {permalink=true}: ASC158

Advice · file level · next step: review · configurable in `[checks]` · acknowledged with `@intended` above the block, or under the heading for a heading · [SPEC §5.4]({repo}/blob/main/SPEC.md#54-glossary-terms)

**When:** The term, and each of its aliases, appears in no other page's prose, and no link names its heading.

**Message:** the glossary term `{term}` appears in no other file's prose, and no link names its heading, so it's never linked

**Fix:** Check the term and its `aliases` against how the pages write it: occurrences match whole words, in prose only, and ignore case unless the term is case-sensitive. Remove the term when the pages don't need it. A term with `match = "marked"` is used only where a link names its heading: link the occurrences that mean it, as `[build](glossary.md#build)`. A term used only inside a phrase's value is reported too, since phrase values aren't searched: acknowledge it with [`@intended`](../writing/directives.md#intended) at the top of the term's section, under its heading.

**Read more:** [Terms](../content-model/content.md#122-terms) in the `ascribe.toml` reference.

### ASC159 `image-unused`
@id {permalink=true}: ASC159

Advice · file level · next step: review · configurable in `[checks]` · acknowledged in `[[intended]]` · [SPEC §5.3]({repo}/blob/main/SPEC.md#53-images)

**When:** An image file under the content root that no file shows or links to: no Markdown image or link, no `src`, `srcset`, `href`, or `poster` in raw HTML, and no frontmatter string names it.

**Message:** no file shows or links to `{path}`

**Fix:** Delete the image when nothing needs it. A frontmatter string counts when it names the image from the file or from the content root, as `cover: img/card.png` does. An image used only by the site's own code is reported too, since Ascribe doesn't read it: turn the check off in [`[checks]`](../content-model/checks.md#19-checks) when your site uses images that way.

**Read more:** [Images](../writing/directives.md#images) in the directive reference.

### ASC160 `image-large`
@id {permalink=true}: ASC160

Advice · file level · next step: review · configurable in `[checks]` · acknowledged in `[[intended]]` · [SPEC §5.3]({repo}/blob/main/SPEC.md#53-images)

**When:** An image file under the content root is larger than the limit: 500 KB, or `[checks.image-large] limit`.

**Message:** `{path}` is \{size}, over the \{limit} limit for an image

**Fix:** Make the image smaller: save it at the size it's shown, or compress it (a screenshot as an optimized PNG, a photo as JPEG or WebP). The limit is 500 KB; set another in `ascribe.toml`, such as `[checks.image-large] limit = "1 MB"`.

**Read more:** [`[checks]`](../content-model/checks.md#19-checks) in the `ascribe.toml` reference.

### ASC161 `title-duplicate`
@id {permalink=true}: ASC161

Advice · page level · next step: write · configurable in `[checks]` · [SPEC §2.2]({repo}/blob/main/SPEC.md#22-pages-and-fragments)

**When:** Two pages the build publishes have the same title, ignoring case.

**Message:** `{title}` is also the title of \{others}, so readers can't tell the pages apart in search results or a list of pages

**Fix:** Give each page a title that says what sets it apart. When one page replaces the other, remove the old one, or publish only one of them in each build with `available`.

**Read more:** [`[checks]`](../content-model/checks.md#19-checks) in the `ascribe.toml` reference.

### ASC172 `availability-left-behind`
@id {permalink=true}: ASC172

Advice · file level · next step: review · configurable in `[checks]` · acknowledged in `[[intended]]` · [SPEC §4.4]({repo}/blob/main/SPEC.md#44-available)

**When:** A feature's availability names a version for every target, each target lists its lines in `[releases]`, and no version is later than its target's current line.

**Message:** feature `{key}` names releases up to \{version}, and the current line of `{target}` is \{current}, so what it marks has shipped

**Fix:** When the feature stands for what the next release adds, as one named `next` does, write the shipped release's spec at the content that shipped in it (`@available: ascribe 0.2.0`), and point the feature at the next planned line. When it's a history worth keeping by name, acknowledge it in [`[[intended]]`](../content-model/checks.md#intended-acknowledgements). After each release, move its line from `planned` to `live` in [`[releases.<target>]`](../content-model/content.md#71-releasestarget).

**Read more:** [`[releases.<target>]`](../content-model/content.md#71-releasestarget) in the `ascribe.toml` reference.

### ASC191 `description-duplicate`
@id {permalink=true}: ASC191

Advice · page level · next step: write · configurable in `[checks]` · [SPEC §9.4]({repo}/blob/main/SPEC.md#94-outputs)

**When:** Two pages the build publishes have the same description, ignoring case.

**Message:** this is also the description of \{others}, so a search result, a link preview, and `llms.txt` say the same of each page, and a reader can't tell which one they want

**Fix:** Give each page a description that says what it's for and what sets it apart from the others. When one page replaces the other, remove the old one, or publish only one of them in each build with `available`.

**Read more:** [Field roles](../content-model/index.md#54-field-roles) in the `ascribe.toml` reference.

### ASC211 `available-line-undeclared`
@id {permalink=true}: ASC211

Advice · file level · next step: choose · configurable in `[checks]` · [SPEC §4.4]({repo}/blob/main/SPEC.md#44-available)

**When:** A version is in a line between two of its target's declared lines, and that line isn't declared.

**Message:** `{version}` is in the line `{line}`, which `[releases.{target}]` doesn't declare; declare it, or correct the number to `{older}` or `{newer}`

**Fix:** Declare the line in `[releases.<target>]`, in the list where it falls: `live` for a line readers are on, `planned` for one newer than every live line that hasn't shipped, or, for one older than every live line that shipped and that nothing publishes now, `archived` with no address, as `"3.3" = {}`. Or correct the number to a declared line, such as the one either side of it.

**Read more:** [`[releases.<target>]`](../content-model/content.md#71-releasestarget) in the `ascribe.toml` reference.

### ASC212 `available-same-in-every-line`
@id {permalink=true}: ASC212

Advice · file level · next step: write · configurable in `[checks]` · [SPEC §4.4]({repo}/blob/main/SPEC.md#44-available)

**When:** Every build that publishes a mark's content is versioned by its target, or filtered for it at a version in the window or after it, and every version the mark names for the target is at or before the oldest live line, so no output shows it.

**Message:** `{entry}` names versions every live line of `{target}` has, the oldest being `{oldest}`, so no output shows them; write `{simpler}`

**Fix:** Write the mark without its versions, keeping its state: `self-managed 3.1` becomes `self-managed`, and `self-managed deprecated 3.2` becomes `self-managed deprecated`. Every live line has the content, and a line's output leaves out a version at or before the oldest live line, so nothing a reader sees changes.

**Read more:** [`[releases.<target>]`](../content-model/content.md#71-releasestarget) in the `ascribe.toml` reference.

### ASC213 `available-in-no-line`
@id {permalink=true}: ASC213

Warning · file level · next step: write · configurable in `[checks]` · [SPEC §4.4]({repo}/blob/main/SPEC.md#44-available)

**When:** Every build that publishes a mark's content is versioned by its target, or filtered for it at a version in the window or after it, and the content is in no live or planned line of the target.

**Message:** `{entry}` leaves the content out of every live and planned line of `{target}`, the oldest being `{oldest}`, so no output has it

**Fix:** Remove the content, which no line's output publishes, or, if a line still has it, correct the mark. A mark that makes content unavailable from a version before the oldest live line, such as `self-managed (ga 3.0, removed 3.2)` with 3.4 the oldest, leaves it in no line.

**Read more:** [`[releases.<target>]`](../content-model/content.md#71-releasestarget) in the `ascribe.toml` reference.

### ASC214 `release-line-ended`
@id {permalink=true}: ASC214

Advice · file level · next step: write · configurable in `[checks]` · [SPEC §7.2]({repo}/blob/main/SPEC.md#72-declarations)

**When:** A live line's date in `[releases.<target>.ends]` is before the day the check runs as: today in UTC, or `ASCRIBE_TODAY`.

**Message:** support for `{target}` \{line} ended on \{date}, and the line is still live

**Fix:** Move the line from `live` to `archived` in `[releases.<target>]`, with its date as `ended` and, when it's still readable somewhere, its `url`; or, if its support goes on, change its date in `ends`.

**Read more:** [`[releases.<target>]`](../content-model/content.md#71-releasestarget) in the `ascribe.toml` reference.

### ASC227 `memo-outdated`
@id {permalink=true}: ASC227

Advice · file level · next step: write · configurable in `[checks]` · [SPEC §7.7]({repo}/blob/main/SPEC.md#77-memo-files)

**When:** No block of what a memo file's `about` names holds its `quote`.

**Message:** no block of `{about}` has the words this memo quotes, so it's on the \{place} and no block

**Fix:** The words were changed since the memo was written. Read the memo against the page as it is: if it still holds, quote the words it's about now; if it doesn't, change it or delete it. A quote is found in one block, with any run of spaces and line breaks read as one space.

**Read more:** [Placed, outdated, and detached](../writing/memo-files.md#placed-outdated-and-detached) in the memo files reference.

### ASC228 `memo-long`
@id {permalink=true}: ASC228

Advice · file level · next step: write · configurable in `[checks]` · [SPEC §7.7]({repo}/blob/main/SPEC.md#77-memo-files)

**When:** A memo file's body is longer than the limit: 2,000 characters, or `[checks.memo-long] limit`.

**Message:** this memo is \{length} characters, over the \{limit} limit

**Fix:** A memo is read by whoever is about to change the page, so it says why in as few words as it can. Cut what the page itself says, and split a memo about several things into one about each. The limit is 2,000 characters; set another in `ascribe.toml`, such as `[checks.memo-long] limit = 4000`.

**Read more:** [Write a memo file](../writing/memo-files.md#write-a-memo-file) in the memo files reference.

### ASC235 `nav-page-unplaced`
@id {permalink=true}: ASC235

Advice · file level · next step: fix · configurable in `[checks]` · [SPEC §7.8]({repo}/blob/main/SPEC.md#78-navigation)

**When:** `ascribe.nav` lists `pages` for a page's section and doesn't name the page. A section with no `pages` has the default order, and its pages aren't reported, nor is a section's own page, nor an `unlisted` page, which no sidebar shows.

**Message:** `{folder}`'s `pages` in ascribe.nav doesn't name this page, so it comes after the pages it does, by title

**Fix:** Add the page's file name to the section's `pages` in `ascribe.nav`, where it should come. The fix appends the section's pages that aren't named, in the order they come in now, so the navigation is the same after it; move the names to where you want them.

**Read more:** [The file](../navigation/index.md#the-file) in the navigation reference.

### ASC236 `nav-section-unplaced`
@id {permalink=true}: ASC236

Advice · file level · next step: fix · configurable in `[checks]` · [SPEC §7.8]({repo}/blob/main/SPEC.md#78-navigation)

**When:** `ascribe.nav` lists one or more of a section's sibling sections and not the section, which is reported on its first page. A folder none of whose sections the file lists has the default order, and isn't reported, nor is a section below the content root whose own page beside its folder places it among its parent's pages, since a table wouldn't move it.

**Message:** ascribe.nav lists other sections in `{parent}` and not `{folder}`, so it comes after them, in path order

**Fix:** Add a `[[sections]]` table with the section's `folder` to `ascribe.nav`, where the section should come. The fix appends a table for each section that isn't listed, in the order they come in now, so the navigation is the same after it; move the tables to where you want them. A file that writes `sections = [...]` inline gets no fix: add each section to its list.

**Read more:** [The file](../navigation/index.md#the-file) in the navigation reference.

## Outputs

### ASC163 `description-too-long`
@id {permalink=true}: ASC163

Advice · file level · next step: write · configurable in `[checks]` · [SPEC §9.4]({repo}/blob/main/SPEC.md#94-outputs)

**When:** The project publishes `llms.txt` (`[consumer] agents = true`), and a page's description, or a content type's default for its description field, is longer than 300 characters as `llms.txt` writes it.

**Message:** this description is \{length} characters, and its line in llms.txt reads best under \{limit}; shorten it to one sentence

**Fix:** Shorten the description to one sentence that says what the page is for. It's the page's line in `llms.txt`, which an agent reads to choose a page, so the whole index should stay short.

**Read more:** [Field roles](../content-model/index.md#54-field-roles) in the `ascribe.toml` reference.

### ASC164 `llms-section-large`
@id {permalink=true}: ASC164

Advice · page level · next step: write · configurable in `[checks]` · [SPEC §9.4]({repo}/blob/main/SPEC.md#94-outputs)

**When:** The project publishes `llms.txt` (`[consumer] agents = true`), and one of a build's `llms.txt` files is longer than 50,000 characters even with the index split by folder. Reported at the page the file is about.

**Message:** \{file} is \{size} characters in build `{build}`, over the \{limit} an agent reads in one fetch; move some of its pages into folders of their own, or shorten their descriptions

**Fix:** Each folder of pages gets its own `llms.txt` once the index is too long for one file, so a file this long is one folder, or the pages outside any folder, with more pages than an agent reads in one fetch. Move some of them into subfolders of their own, or shorten their descriptions. It's the delivery spec's `llms-txt-size` check, made from the build before a deploy; `ascribe report agents` names this check for it.

**Read more:** [`[consumer]`](../content-model/output.md#consumer) in the `ascribe.toml` reference.

### ASC184 `listing-disallowed`
@id {permalink=true}: ASC184

Advice · file level · next step: fix · configurable in `[checks]` · [SPEC §2.1]({repo}/blob/main/SPEC.md#21-files)

**When:** A page is `unlisted`, and what the project says in `[crawlers]` keeps a search crawler from fetching it: its own lines, for every crawler or in a group that names the crawler, or a disallowed purpose.

**Message:** this page is unlisted, and `{line}` in `[crawlers] lines` disallows its address, so a crawler never fetches it to read that it isn't to be listed; a search engine can still list the address alone

**Fix:** Remove one of the two. A crawler reads `noindex` only on a page it may fetch, so to keep a page out of search results, let crawlers fetch it: remove the `Disallow` line, or narrow it so it doesn't cover the page, or allow the purpose. If the page only needs to stay out of what crawlers fetch, remove `listing: unlisted`. The editor's quick fix removes `listing`.

**Read more:** [`[crawlers]`](../content-model/checks.md#21-crawlers) in the `ascribe.toml` reference.

### ASC190 `crawlers-disallowed`
@id {permalink=true}: ASC190

Advice · file level · next step: choose · configurable in `[checks]` · [SPEC §7.2]({repo}/blob/main/SPEC.md#72-declarations)

**When:** `[crawlers]` disallows `search` or `on-request`, and the site's pages are listed unless they say otherwise.

**Message:** `{purpose} = "disallow"` keeps AI search products from indexing the site, so its pages aren't in their answers, while the pages are listed for every other search engine

**Fix:** Choose what the site wants. To be in AI search answers, and fetched when a person asks, `allow` the purpose or remove it. To keep it, turn this check off in `[checks]`, saying so in a comment: the choice is the project's, and this only says what it costs. A purpose's group asks only the crawlers Ascribe knows, and some of them say they may not follow `robots.txt`.

**Read more:** [`[crawlers]`](../content-model/checks.md#21-crawlers) in the `ascribe.toml` reference.

## Prose, through Vale

### ASC165 `prose`
@id {permalink=true}: ASC165

Advice · file level · next step: write · configurable in `[checks]` · [SPEC §2.1]({repo}/blob/main/SPEC.md#21-files)

**When:** The project has `[checks.vale]`, and Vale reports an alert about the prose of a page or a fragment, or of a phrase's text. Reported where the alert is in the source, or at the phrase.

**Message:** \{rule}: \{message}

**Fix:** Vale reported this about the prose, under the rule the message starts with. Change the text, or take the replacement the editor offers when the rule gives one; for text from a phrase, change the phrase's value in `ascribe.toml`. A rule that's wrong for the project is turned off in Vale's configuration, or, for the `quiet` preset, in `[checks.vale] off`; one place is quieted with Vale's own comments (`<!-- vale Rule = NO -->`). See [Prose, through Vale](../checks/vale.md).

**Read more:** [`[checks.vale]`](../content-model/checks.md#checksvale) in the `ascribe.toml` reference.

### ASC166 `prose-not-checked`
@id {permalink=true}: ASC166

Advice · file level · next step: outside · configurable in `[checks]` · [SPEC §2.1]({repo}/blob/main/SPEC.md#21-files)

**When:** The project has `[checks.vale]`, and Vale couldn't be run, failed, or didn't finish in time, or the tool checking the project doesn't start it, as the agents' tools don't. Reported once, at `[checks.vale]`.

**Message:** the prose wasn't checked: `{command}` couldn't be run (\{reason}). Install Vale, or set `[checks.vale] command` to where it is

**Fix:** Install [Vale](https://vale.sh) 3.16 or later so that the command `[checks.vale] command` names (`vale` by default) runs, or fix what Vale's own message says about its configuration. Nothing else is checked differently while it can't run. A project that checks prose only in CI can turn this off on other machines with `prose-not-checked = "off"` in `[checks]`. The agents' tools never start Vale; the agents' hook never starts a `command` that's a program in the project, and the editor asks before it does. Run `ascribe check --vale` in a terminal to check the prose then.

**Read more:** [`[checks.vale]`](../content-model/checks.md#checksvale) in the `ascribe.toml` reference.

## External links

### ASC168 `link-external-broken`
@id {permalink=true}: ASC168

Advice · file level · next step: review · configurable in `[checks]` · acknowledged with `@intended` above the block, or under the heading for a heading · [SPEC §5.2]({repo}/blob/main/SPEC.md#52-links)

**When:** `ascribe report links`: the link checker found that an external link answers with an error, doesn't answer in time, or can't be reached. A host `[checks.links] ignore` names isn't checked.

**Message:** `{url}` answered \{status}, so the link is broken for readers

**Fix:** Open the address. When the page is gone, link to where its content went, or remove the link. A site that turns link checkers away, or one that's down for a while, may be fine as it is: acknowledge the link with `@intended {check=link-external-broken}: <why>` directly above its block, or leave the whole host out with `[checks.links] ignore`. Only `ascribe report links` checks external links; `ascribe check` never reaches the network.

**Read more:** [`[checks.links]`](../content-model/checks.md#checkslinks) in the `ascribe.toml` reference.

### ASC169 `link-external-moved`
@id {permalink=true}: ASC169

Advice · file level · next step: choose · configurable in `[checks]` · [SPEC §5.2]({repo}/blob/main/SPEC.md#52-links)

**When:** `ascribe report links`: an external link's address redirects permanently (`301` or `308`) to another one that answers. A site's root that redirects to another of its own pages, a landing page, hasn't moved.

**Message:** `{url}` moved permanently to `{to}`; link to the new address

**Fix:** Link to the address it redirects to; the fix writes it. Check first that it's the page you meant: some sites send every old address to their home page, and then the link needs a new target, not this one. Only `ascribe report links` checks external links.

**Read more:** [`[checks.links]`](../content-model/checks.md#checkslinks) in the `ascribe.toml` reference.

## The published site

### ASC170 `delivery-hosting`
@id {permalink=true}: ASC170

Advice · file level · next step: outside · configurable in `[checks]` · [SPEC §9.5]({repo}/blob/main/SPEC.md#95-consumer-profile)

**When:** `ascribe report agents`: the published site fails, or warns on, a check of the Web Documentation Delivery Spec that its hosting decides, such as status codes, caching, or content negotiation.

**Message:** the site fails the delivery spec's `{check}` check: \{result}. \{setting}

**Fix:** Change the setting the message names where the site is hosted; no page needs editing. [Serve the Markdown versions](../astro/serve-markdown.md) says what to set on the hosts it names. Only `ascribe report agents` checks a published site.

**Read more:** [`[consumer]`](../content-model/output.md#consumer) in the `ascribe.toml` reference.

### ASC171 `delivery-output`
@id {permalink=true}: ASC171

Advice · file level · next step: outside · configurable in `[checks]` · [SPEC §9.5]({repo}/blob/main/SPEC.md#95-consumer-profile)

**When:** `ascribe report agents`: the published site fails, or warns on, a check of the Web Documentation Delivery Spec that `llms.txt`, the Markdown pages, or the pointer on each page should pass, which Ascribe writes with `[consumer] agents = true`.

**Message:** the site fails the delivery spec's `{check}` check, which what Ascribe writes should pass: \{result}. Report it at \{issues}

**Fix:** First check that the site publishes what the build wrote, as it wrote it: `llms.txt` and the Markdown pages from the plain output, and each page's pointer from the site output. When it does, this is a bug in Ascribe: report it at the issue tracker the message names, with the report's output. No page needs editing.

**Read more:** [`[consumer]`](../content-model/output.md#consumer) in the `ascribe.toml` reference.

## Tested examples

### ASC199 `example-stale`
@id {permalink=true}: ASC199

Advice · file level · next step: outside · configurable in `[checks]` · [SPEC §4.8]({repo}/blob/main/SPEC.md#48-snippet)

**When:** A `@snippet` shows a file that its source's `ascribe.examples.toml` makes an example, and the file has changed since the observation that lists it was made. For a source in another repository, the config, the observation, and the file are the copies `ascribe sources fetch` made. Reported once for each snippet of the file, whichever region it shows.

**Message:** `{path}` changed after its last test run, so the page doesn't say what it was tested with; run `ascribe observe` in `{folder}` to test it again

**Fix:** Run `ascribe observe` in the folder named, where `ascribe.examples.toml` is, and commit the observation it writes. It runs the folder's test command, and writes the observation only when the command passes. An observation covers a whole file: an edit anywhere in it, even outside the region a page shows, needs a new test run. For a source in another repository, that's done in the code's repository, and `ascribe sources update` copies the new observation here; nothing in this repository runs the test.

**Read more:** [`@snippet`](../writing/directives.md#snippet) in the directive reference.

### ASC200 `example-unobserved`
@id {permalink=true}: ASC200

Advice · file level · next step: outside · configurable in `[checks]` · [SPEC §4.8]({repo}/blob/main/SPEC.md#48-snippet)

**When:** A `@snippet` shows a file that its source's `ascribe.examples.toml` makes an example, and no observation lists the file. For a source in another repository, the config and the observation are the copies `ascribe sources fetch` made. Reported once for each snippet of the file.

**Message:** `{path}` is an example, and no test run has covered it, so the page doesn't say what it was tested with; run `ascribe observe` in `{folder}`

**Fix:** Run `ascribe observe` in the folder named, where `ascribe.examples.toml` is, and commit the observation it writes. When the file isn't meant to be an example, add it to the config's `ignore`, or narrow its `include`. For a source in another repository, both are done in the code's repository, and `ascribe sources update` copies the result here.

**Read more:** [`@snippet`](../writing/directives.md#snippet) in the directive reference.

### ASC201 `examples-config-invalid`
@id {permalink=true}: ASC201

Warning · file level · next step: outside · configurable in `[checks]` · [SPEC §7.3]({repo}/blob/main/SPEC.md#73-sources)

**When:** A source has an `ascribe.examples.toml`, or an observation beside it, that can't be read: in its folder, or, for a source in another repository, in its copies. Reported at the source in ascribe.toml, once for each problem in each file.

**Message:** the examples of source `{source}` can't be checked: `{file}` can't be read: \{detail}

**Fix:** Fix the file named, in the source's folder: `ascribe.examples.toml` is written by the code's owners, and an observation by `ascribe observe`, which writes it again when it's run there. Until it can be read, no snippet of the source says what it was tested with.

**Read more:** [`[sources.<name>]`](../content-model/output.md#18-sourcesname) in the `ascribe.toml` reference.

### ASC202 `observation-file-missing`
@id {permalink=true}: ASC202

Advice · file level · next step: outside · configurable in `[checks]` · [SPEC §7.3]({repo}/blob/main/SPEC.md#73-sources)

**When:** An observation beside the `ascribe.examples.toml` of a source in the same repository lists a file that isn't in the source's folder. Reported at the file's line in the observation, once for each file.

**Message:** `{observation}` lists `{path}`, which isn't in `{folder}`; run `{command}` there to write the observation again

**Fix:** The example was usually deleted or renamed without a second test run. Run the command named, in the folder named, and commit the observation it writes, which lists the examples as they are. When nothing writes the observation any more, delete its file.

**Read more:** [`[sources.<name>]`](../content-model/output.md#18-sourcesname) in the `ascribe.toml` reference.
