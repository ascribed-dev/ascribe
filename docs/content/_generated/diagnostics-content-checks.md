## Pages
@memo: Generated from crates/ascribe-core/data/diagnostics.toml by tests/conformance/tests/docs.rs; edit the registry, then run `ASCRIBE_BLESS=1 cargo test -p ascribe-conformance --test docs` to rewrite it.

### ASC149 `page-size`
@id {permalink=true}: ASC149
@summary: A page's Markdown, as a build writes it for agents to read, is as long as the limit or longer: 50,000 characters, or `[checks.page-size] limit`. A page over the limit in several builds is one problem, with its size in the build where it's longest.

[Advice](../checks/diagnostics.md#severity) · [page level](../checks/diagnostics.md#file-level-and-page-level) · next step: [write](../checks/diagnostics.md#what-to-do-next) · [configurable](../checks/diagnostics.md#setting-a-checks-level) in `[checks]` · [SPEC §9.4]({repo}/blob/main/SPEC.md#94-outputs)

**When:** A page's Markdown, as a build writes it for agents to read, is as long as the limit or longer: 50,000 characters, or `[checks.page-size] limit`. A page over the limit in several builds is one problem, with its size in the build where it's longest.

**Message:** this page's Markdown is \{size} characters in build `{build}`, and an agent reads under \{limit} in one piece; split the page, or move long tables and code below the prose

**Fix:** Split the page into pages that each answer one question, or move long tables, reference lists, and code below the prose, where an agent that stops reading early loses the least. The problem names the page's largest parts, up to five, largest first, each with its size: its sections, at their headings, and its includes, at their `@include` lines. The limit is the Web Documentation Delivery Spec's: a page passes under 50,000 characters of Markdown. Set another with `[checks.page-size] limit`. It's the spec's `page-size-markdown` check, made from the build before a deploy; `ascribe report delivery` names this check for it, and for the spec's other checks of a page's size.

**Read more:** [`[checks]`](../content-model/checks.md#22-checks) in the `ascribe.toml` reference.

### ASC150 `page-description-missing`
@id {permalink=true}: ASC150
@summary: A page's content type marks a field `role = "description"`, and the page leaves the field out or empty. A required field, whose absence is `frontmatter-missing-field`, and a field with a default aren't looked at.

[Advice](../checks/diagnostics.md#severity) · [file level](../checks/diagnostics.md#file-level-and-page-level) · next step: [write](../checks/diagnostics.md#what-to-do-next) · [configurable](../checks/diagnostics.md#setting-a-checks-level) in `[checks]` · [SPEC §7.2]({repo}/blob/main/SPEC.md#72-declarations)

**When:** A page's content type marks a field `role = "description"`, and the page leaves the field out or empty. A required field, whose absence is `frontmatter-missing-field`, and a field with a default aren't looked at.

**Message:** this page has no `{field}`; write a sentence saying what the page is for, which search results and agents show with its title

**Fix:** Add the field the content type marks with `role = "description"` to the page's frontmatter: a sentence or two saying what the page covers and who it's for.

**Read more:** [Field roles](../content-model/index.md#54-field-roles) in the `ascribe.toml` reference.

### ASC151 `heading-level-skipped`
@id {permalink=true}: ASC151
@summary: A heading is more than one level below the heading before it, or is at level 1, which the page's title is. Headings from includes count at the levels they're written with, and after a skip the next heading is compared with the one that skipped, so a skipped section is reported once.

[Advice](../checks/diagnostics.md#severity) · [page level](../checks/diagnostics.md#file-level-and-page-level) · next step: [fix](../checks/diagnostics.md#what-to-do-next) · [configurable](../checks/diagnostics.md#setting-a-checks-level) in `[checks]` · [SPEC §9.4]({repo}/blob/main/SPEC.md#94-outputs)

**When:** A heading is more than one level below the heading before it, or is at level 1, which the page's title is. Headings from includes count at the levels they're written with, and after a skip the next heading is compared with the one that skipped, so a skipped section is reported once.

**Message:** this heading is level \{level}, after a level-\{previous} heading; make it level \{expected}, so the outline has no gap

**Fix:** Make each heading one level below the heading it belongs under. A page's title is its level-1 heading, so its own headings start at level 2. The quick fix changes the one heading; the headings under it may need to move up too.

**Read more:** [`[checks]`](../content-model/checks.md#22-checks) in the `ascribe.toml` reference.

### ASC152 `code-language-missing`
@id {permalink=true}: ASC152
@summary: A fenced code block has no language after its opening fence: nothing, or a pair of SPEC §3.3.1 first, such as `phrases=true` or `title="x.ts"`, which a renderer such as comrak or Astro takes as the language. An indented code block isn't looked at.

[Advice](../checks/diagnostics.md#severity) · [file level](../checks/diagnostics.md#file-level-and-page-level) · next step: [choose](../checks/diagnostics.md#what-to-do-next) · [configurable](../checks/diagnostics.md#setting-a-checks-level) in `[checks]` · Ascribe's own, not in SPEC

**When:** A fenced code block has no language after its opening fence: nothing, or a pair of SPEC §3.3.1 first, such as `phrases=true` or `title="x.ts"`, which a renderer such as comrak or Astro takes as the language. An indented code block isn't looked at.

**Message:** this code block has no language; write it after the opening fence, or `text` for output

**Fix:** Write the block's language right after the opening fence, such as `sh`, `toml`, or `json`. Use `text` for program output, logs, and anything else that isn't code. A highlighter colors the block by it, and an agent knows what the block is.

**Read more:** [`[checks]`](../content-model/checks.md#22-checks) in the `ascribe.toml` reference.

### ASC153 `recheck-overdue`
@id {permalink=true}: ASC153
@summary: A page's content type marks a field `role = "recheck-date"` (or `"review-date"`, its old name), and the date the page gives it is before the day the check runs as: today in UTC, or `ASCRIBE_TODAY`.

[Advice](../checks/diagnostics.md#severity) · [file level](../checks/diagnostics.md#file-level-and-page-level) · next step: [review](../checks/diagnostics.md#what-to-do-next) · [configurable](../checks/diagnostics.md#setting-a-checks-level) in `[checks]` · [acknowledged](../glossary.md#acknowledgement) in the page's `intended` frontmatter · [SPEC §7.2]({repo}/blob/main/SPEC.md#72-declarations)

**When:** A page's content type marks a field `role = "recheck-date"` (or `"review-date"`, its old name), and the date the page gives it is before the day the check runs as: today in UTC, or `ASCRIBE_TODAY`.

**Message:** this page was due for a recheck on \{date}; check it still holds, then move `{field}` to the next recheck

**Fix:** Read the page against what it describes and correct what has changed, then set its recheck date to when it's next due. If the page is right as it is, moving the date is the whole fix. The check compares with today's date in UTC (or `ASCRIBE_TODAY`), so it can start reporting without a change to the project.

**Read more:** [Field roles](../content-model/index.md#54-field-roles) in the `ascribe.toml` reference.

### ASC192 `title-heading-differs`
@id {permalink=true}: ASC192
@summary: A page writes its own level-1 heading, and neither it nor the page's title contains the other, ignoring case. A page without a level-1 heading, which most pages are, since the layout draws the title as one, isn't looked at.

[Advice](../checks/diagnostics.md#severity) · [page level](../checks/diagnostics.md#file-level-and-page-level) · next step: [review](../checks/diagnostics.md#what-to-do-next) · [configurable](../checks/diagnostics.md#setting-a-checks-level) in `[checks]` · [acknowledged](../glossary.md#acknowledgement) in the page's `intended` frontmatter · [SPEC §2.1]({repo}/blob/main/SPEC.md#21-files)

**When:** A page writes its own level-1 heading, and neither it nor the page's title contains the other, ignoring case. A page without a level-1 heading, which most pages are, since the layout draws the title as one, isn't looked at.

**Message:** the title, `{title}`, says something the page's own top heading, `{heading}`, doesn't; a search engine may show either one as the page's name, so a reader can be promised one page and land on what reads like another

**Fix:** Make the two agree: change the title or the heading so one says what the other does, or remove the heading and let the layout draw the title, as it does on most pages. When they differ on purpose, as a landing page's title is often the site's name and its heading a tagline, acknowledge it in the page's `intended` frontmatter, saying why.

**Read more:** [`[checks]`](../content-model/checks.md#22-checks) in the `ascribe.toml` reference.

### ASC195 `template-placeholder-left`
@id {permalink=true}: ASC195
@summary: A draft of a content type that names a template still has one of the template's placeholders, or a frontmatter field with the template's value.

[Advice](../checks/diagnostics.md#severity) · [file level](../checks/diagnostics.md#file-level-and-page-level) · next step: [write](../checks/diagnostics.md#what-to-do-next) · [configurable](../checks/diagnostics.md#setting-a-checks-level) in `[checks]` · Ascribe's own, not in SPEC

**When:** A draft (a page with `draft: true`) of a content type that names a template still has one of the template's placeholders, exactly as the template has it but for how its lines are wrapped, or a frontmatter field with the template's value. In a page that publishes it's `template-placeholder-published`, an error. A memo the page keeps from the template, its instructions to the writer, isn't reported. A placeholder's opening brace is escaped on the page as in the template, so a phrase of the same name, or a placeholder quoted in inline code or a code block, isn't one. A page of a type with no template isn't looked at, and a placeholder changed by a word isn't found.

**Message:** this page still has its template's placeholder `{placeholder}`; write what it asks for in its place

**Fix:** Write the page's own text where the template's placeholder is, the text in braces. The template's instructions are memos, which are never published: follow each, then delete it or keep it. A field the template fills in, such as `title`, gets the page's own value. The check finds only what was never touched: a placeholder with a word changed isn't reported, so read the page through before publishing it.

**Read more:** [Page types](../content-model/index.md#41-page-types) in the `ascribe.toml` reference.

### ASC198 `description-link`
@id {permalink=true}: ASC198
@summary: A page's description, the field its type marks `role = "description"`, holds a Markdown link.

[Advice](../checks/diagnostics.md#severity) · [file level](../checks/diagnostics.md#file-level-and-page-level) · next step: [write](../checks/diagnostics.md#what-to-do-next) · [configurable](../checks/diagnostics.md#setting-a-checks-level) in `[checks]` · [SPEC §9.4]({repo}/blob/main/SPEC.md#94-outputs)

**When:** A page's description, the field its type marks `role = "description"`, holds a Markdown link.

**Message:** a description is plain text, so `{link}` isn't a link: a search result and a link preview show its brackets and destination, and `ascribe check` and `ascribe rename` don't see it

**Fix:** Keep the link's text and remove the brackets and the destination; for a description that isn't quoted, the editor's quick fix does it. Link to the page from the page's body instead, where it's a link that's checked and kept up to date.

**Read more:** [Field roles](../content-model/index.md#54-field-roles) in the `ascribe.toml` reference.

### ASC220 `link-into-older-line`
@id {permalink=true}: ASC220
@summary: A link in the current line of a versioned build names a page the current line doesn't publish, and nothing takes its place there, so it leads to the page in an older line.

[Advice](../checks/diagnostics.md#severity) · [page level](../checks/diagnostics.md#file-level-and-page-level) · next step: [write](../checks/diagnostics.md#what-to-do-next) · [configurable](../checks/diagnostics.md#setting-a-checks-level) in `[checks]` · [SPEC §9.3]({repo}/blob/main/SPEC.md#93-build-modes)

**When:** A link in the current line of a versioned build names a page the current line doesn't publish, and nothing takes its place there, so it leads to the page in an older line.

**Message:** `{path}` isn't in the current line, so this link leads to it in \{line}; the current line usually needs a page in its place

**Fix:** Give the current line a page in its place: mark the page so the current line has it, or write the page that takes its place and give it `replaces`. When the link is meant to lead to an older line, as on a page about migrating from it, leave it.

**Read more:** [Links between lines](../writing/inline.md#links-between-lines) in the phrases and links reference.

### ASC221 `link-fragment-dropped`
@id {permalink=true}: ASC221
@summary: A link to a heading leads, in a line of a versioned build, to a page that takes its target's place or to its target in another line, and the page it lands on has no heading with that id.

[Advice](../checks/diagnostics.md#severity) · [page level](../checks/diagnostics.md#file-level-and-page-level) · next step: [write](../checks/diagnostics.md#what-to-do-next) · [configurable](../checks/diagnostics.md#setting-a-checks-level) in `[checks]` · [SPEC §9.3]({repo}/blob/main/SPEC.md#93-build-modes)

**When:** A link to a heading leads, in a line of a versioned build, to a page that takes its target's place or to its target in another line, and the page it lands on has no heading with that id.

**Message:** in `{build}`, this link leads to `{page}`, which has no heading `{id}`, so it goes to the top of the page

**Fix:** Give the heading on the page it lands on the same id, with `@id`, or link to the page without `#` and the id.

**Read more:** [Links between lines](../writing/inline.md#links-between-lines) in the phrases and links reference.

### ASC223 `html-comment`
@id {permalink=true}: ASC223
@summary: A page or a fragment has an HTML comment, as a block or inside a paragraph. A comment in a code block or a memo isn't one, and Vale's own comments, such as `<!-- vale off -->`, aren't reported.

[Advice](../checks/diagnostics.md#severity) · [file level](../checks/diagnostics.md#file-level-and-page-level) · next step: [review](../checks/diagnostics.md#what-to-do-next) · [configurable](../checks/diagnostics.md#setting-a-checks-level) in `[checks]` · [acknowledged](../glossary.md#acknowledgement) with `@intended` above the block, or under the heading for a heading · [SPEC §4.10]({repo}/blob/main/SPEC.md#410-memo)

**When:** A page or a fragment has an HTML comment, as a block or inside a paragraph. A comment in a code block or a memo isn't one, and Vale's own comments, such as `<!-- vale off -->`, aren't reported.

**Message:** an HTML comment is in the published page's source; a memo isn't

**Fix:** An HTML comment isn't private: it's in the source of every published page, and in the JSON output. Write it as a `@memo` instead, which keeps the same text beside the same block and out of every output: the editor's quick fix, **Rewrite as a memo**, does it for a comment that's a block of its own. A comment written for another tool, which has to stay a comment, is acknowledged with `@intended {check=html-comment}` above its block.

**Read more:** [`@memo`](../writing/directives.md#memo) in the directive reference.

### ASC224 `memo-empty`
@id {permalink=true}: ASC224
@summary: A container memo has nothing between its opening line and its `@end`, or only blank lines.

[Advice](../checks/diagnostics.md#severity) · [file level](../checks/diagnostics.md#file-level-and-page-level) · next step: [fix](../checks/diagnostics.md#what-to-do-next) · [configurable](../checks/diagnostics.md#setting-a-checks-level) in `[checks]` · [SPEC §4.10]({repo}/blob/main/SPEC.md#410-memo)

**When:** A container memo has nothing between its opening line and its `@end`, or only blank lines.

**Message:** this memo is empty

**Fix:** A memo that says nothing is almost always a slip: write what it was meant to say between `@memo:` and `@end`, or remove it. The fix removes it, which changes no output, since no output shows a memo.

**Read more:** [`@memo`](../writing/directives.md#memo) in the directive reference.

### ASC229 `template-section-empty`
@id {permalink=true}: ASC229
@summary: A page of a content type that names a template has a section the template gives only memos, and the page's section still has nothing but memos and headings.

[Advice](../checks/diagnostics.md#severity) · [file level](../checks/diagnostics.md#file-level-and-page-level) · next step: [write](../checks/diagnostics.md#what-to-do-next) · [configurable](../checks/diagnostics.md#setting-a-checks-level) in `[checks]` · Ascribe's own, not in SPEC

**When:** A page of a content type that names a template has a section the template gives nothing under its heading but memos, with the same heading at the same level, and nothing in the page's section, its subsections included, but memos, directives bound to its heading such as `@id`, and the subsections' headings. A section the template gives any other content is `template-placeholder-left`'s, and a page of a type with no template isn't looked at.

**Message:** the template's section `{heading}` is still empty; write it, or remove the heading if this page doesn't need it

**Fix:** Write the section's text under its heading. The template gives it only memos, its instructions to the writer, which are never published, so left as it is the section builds as a heading with nothing under it. Follow each memo, then delete it or keep it. If the page doesn't need the section, remove its heading and its memos.

**Read more:** [Page types](../content-model/index.md#41-page-types) in the `ascribe.toml` reference.

### ASC237 `nav-label-long`
@id {permalink=true}: ASC237
@summary: A page's `nav-label` is longer than its `title`, each counted in characters as a reader sees it.

[Advice](../checks/diagnostics.md#severity) · [file level](../checks/diagnostics.md#file-level-and-page-level) · next step: [write](../checks/diagnostics.md#what-to-do-next) · [configurable](../checks/diagnostics.md#setting-a-checks-level) in `[checks]` · [SPEC §2.1]({repo}/blob/main/SPEC.md#21-files)

**When:** A page's `nav-label` is longer than its `title`, each counted in characters as a reader sees it.

**Message:** this `nav-label` is \{length} characters, longer than the \{title}-character title it stands for in the sidebar

**Fix:** A `nav-label` is a page's short name in a list, a sidebar or previous and next, where the title is too long to read at a glance. Shorten it, or remove it, and the title is the page's name there too.

**Read more:** [A page's label](../navigation/index.md#a-page-s-label) in the navigation reference.

### ASC241 `glossary-definition-long`
@id {permalink=true}: ASC241
@summary: A term's definition, as a build derives it, is longer than the limit `description-too-long` measures a description against.

[Advice](../checks/diagnostics.md#severity) · [page level](../checks/diagnostics.md#file-level-and-page-level) · next step: [write](../checks/diagnostics.md#what-to-do-next) · [configurable](../checks/diagnostics.md#setting-a-checks-level) in `[checks]` · [SPEC §5.4]({repo}/blob/main/SPEC.md#54-glossary-terms)

**When:** A term's definition, as a build derives it, is longer than the limit `description-too-long` measures a description against.

**Message:** the definition of the term "\{term}" is over \{limit} characters in build `{build}`, too long for a link's title or a hover

**Fix:** End the first paragraph after the definition, in a sentence or two, and put the rest in a second paragraph. The first paragraph is what a link to the term shows as its title and the editor shows on hover; the rest of the section is still on the glossary page. The length is counted on the definition as each build derives it, with phrases substituted and formatting dropped.

**Read more:** [`[glossary]`](../content-model/content.md#8-glossary) in the `ascribe.toml` reference.

### ASC252 `summary-missing`
@id {permalink=true}: ASC252
@summary: A heading's `@id` marks it a permalink, and its section has no `@summary`, once includes are expanded. A page's own permalink has its description, which `page-description-missing` reports; a permalink in a fragment is reported once, in the fragment.

[Advice, off by default](../checks/diagnostics.md#severity) · [page level](../checks/diagnostics.md#file-level-and-page-level) · next step: [write](../checks/diagnostics.md#what-to-do-next) · [configurable](../checks/diagnostics.md#setting-a-checks-level) in `[checks]` · [SPEC §4.12]({repo}/blob/main/SPEC.md#412-summary)

**When:** A heading's `@id` marks it a permalink, and its section has no `@summary`, once includes are expanded. A page's own permalink has its description, which `page-description-missing` reports; a permalink in a fragment is reported once, in the fragment.

**Message:** the permalink `{name}` has no summary, so a product showing its help has none; add a `@summary` to its section

**Fix:** Give the section a summary with `@summary`: a sentence or two a product shows in a tooltip or beside an error. Put `@summary` on the line above a paragraph of the section that reads well alone, so it's written once and shown in both places, or write `@summary: <text>` directly under the heading and its `@id` when nothing on the page reads well alone. The check is off unless the project turns it on, as one that ships embedded help does: `[checks] summary-missing = "warning"` makes every permalink need a summary.

**Read more:** [`@summary`](../writing/directives.md#summary) in the directive reference.

### ASC253 `summary-long`
@id {permalink=true}: ASC253
@summary: A section's summary, as plain text on one line, is as long as the limit or longer: 300 characters, or `[checks.summary-long] limit`. A summary in a fragment is reported once, in the fragment.

[Advice, off by default](../checks/diagnostics.md#severity) · [page level](../checks/diagnostics.md#file-level-and-page-level) · next step: [write](../checks/diagnostics.md#what-to-do-next) · [configurable](../checks/diagnostics.md#setting-a-checks-level) in `[checks]` · [SPEC §4.12]({repo}/blob/main/SPEC.md#412-summary)

**When:** A section's summary, as plain text on one line, is as long as the limit or longer: 300 characters, or `[checks.summary-long] limit`. A summary in a fragment is reported once, in the fragment.

**Message:** this summary is \{length} characters, and a product shows one under \{limit}; shorten it to a sentence or two

**Fix:** Shorten the summary to a sentence or two that say what the section is for. A product shows it in a tooltip or beside an error, where there's room for little more. Its length is counted as `description-too-long` counts a description's: as plain text on one line, with phrases replaced and formatting dropped. The check is off unless the project names it in `[checks]`; the limit is 300 characters, and `[checks.summary-long] limit = 200` turns it on with another.

**Read more:** [`@summary`](../writing/directives.md#summary) in the directive reference.

### ASC286 `template-placeholder-published`
@id {permalink=true}: ASC286
@summary: A page that publishes, of a content type that names a template, still has one of the template's placeholders, or a frontmatter field with the template's value.

[Error](../checks/diagnostics.md#severity) · [file level](../checks/diagnostics.md#file-level-and-page-level) · next step: [write](../checks/diagnostics.md#what-to-do-next) · [configurable](../checks/diagnostics.md#setting-a-checks-level) in `[checks]` · Ascribe's own, not in SPEC

**When:** A page that publishes, one without `draft: true`, of a content type that names a template still has one of the template's placeholders, or a frontmatter field with the template's value, found as `template-placeholder-left` finds them. In a draft, it's that advice instead, so a page being written isn't covered in errors.

**Message:** this page publishes with its template's placeholder `{placeholder}`; write what it asks for in its place, or mark the page `draft: true` until it's ready

**Fix:** Write the page's own text where the template's placeholder is, the text in braces, and the page's own value in each field the template fills in, such as `title`. A placeholder in a page that publishes is never meant: readers see the template's words. While the page isn't ready, mark it `draft: true` in its frontmatter: no build publishes it, and what's left to fill in is advice. `ascribe new` starts a page that way.

**Read more:** [Page types](../content-model/index.md#41-page-types) in the `ascribe.toml` reference.

### ASC287 `github-alert`
@id {permalink=true}: ASC287
@summary: A block quote's first line is one of GitHub's alert markers, `[!NOTE]`, `[!TIP]`, `[!IMPORTANT]`, `[!WARNING]`, or `[!CAUTION]`, in any case. A quote that mentions a marker later isn't looked at.

[Warning](../checks/diagnostics.md#severity) · [file level](../checks/diagnostics.md#file-level-and-page-level) · next step: [fix](../checks/diagnostics.md#what-to-do-next) · [configurable](../checks/diagnostics.md#setting-a-checks-level) in `[checks]` · [SPEC §4.5]({repo}/blob/main/SPEC.md#45-note)

**When:** A block quote's first line is one of GitHub's alert markers, `[!NOTE]`, `[!TIP]`, `[!IMPORTANT]`, `[!WARNING]`, or `[!CAUTION]`, in any case. A quote that mentions a marker later isn't looked at.

**Message:** `{marker}` is GitHub's alert syntax, which Ascribe shows as a plain block quote that starts with `{marker}`; write the quote as `{note}`

**Fix:** Write the callout as a `@note` of the same type: GitHub's `NOTE`, `TIP`, `IMPORTANT`, `WARNING`, and `CAUTION` are `@note`'s types `note`, `tip`, `important`, `warning`, and `caution`. The fix does it, keeping the quote's content: `@note {type=tip}: …` for one paragraph, or `@note {type=tip}:` and `@end` around several blocks. Every output then shows it as a callout: the site as a note, and the Markdown output as a block quote that starts with its type, which reads the same on GitHub and anywhere else.

**Read more:** [`@note`](../writing/directives.md#note) in the directive reference.

### ASC288 `phrase-text-written`
@id {permalink=true}: ASC288
@summary: A page or a fragment writes out a declared phrase's value in its text, where it could write the phrase.

[Advice](../checks/diagnostics.md#severity) · [file level](../checks/diagnostics.md#file-level-and-page-level) · next step: [review](../checks/diagnostics.md#what-to-do-next) · [configurable](../checks/diagnostics.md#setting-a-checks-level) in `[checks]` · [acknowledged](../glossary.md#acknowledgement) with `@intended` above the block, or under the heading for a heading · [SPEC §5.1]({repo}/blob/main/SPEC.md#51-phrases)

**When:** A page or a fragment writes out a declared phrase's value in its text, where it could write the phrase. A value matches only as a whole word, or a whole version, with its case as written, and a value of one character or of digits alone isn't looked for; when one value is inside another, only the longer is reported. Code, frontmatter, link destinations, the label of a reference link, and a heading marked `@term` aren't looked in.

**Message:** `{text}` is the value of the phrase `{key}`; write `{key}`, so the page changes with the value

**Fix:** Write the phrase in braces, as `{product}`, where the page means its value, so the page changes when `ascribe.toml` does: a version, a URL, or a name written out goes stale at the next release. The fix does it, and is safe for a value that can't be an ordinary word: one with a space or a symbol in it, such as a version or a URL. A heading that takes the phrase needs an `@id` too, so its link stays the same when the value changes. Where the text means what it says whatever the value becomes, as a changelog's version does, acknowledge it with `@intended {check=phrase-text-written}` above its block.

**Read more:** [Phrases](../writing/inline.md#phrases) in the phrases and links reference.
