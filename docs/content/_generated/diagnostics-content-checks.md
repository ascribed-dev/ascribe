<!-- Generated from tests/conformance/diagnostics.toml by tests/conformance/tests/docs.rs. Edit the registry, then run `ASCRIBE_BLESS=1 cargo test -p ascribe-conformance --test docs`. -->

### Across the project

#### ASC148 `page-orphan`

Advice · page level · next step: review · configurable in `[checks]` · acknowledged in the page's `intended` frontmatter · [SPEC §5.2]({repo}/blob/main/SPEC.md#52-links)

**When:** No other page the build publishes links to the page or includes it. Index pages (`index.md`), and a build's only page, aren't reported.

**Message:** no other page links to this page or includes it, so readers reach it only through the site's own navigation or search, if those list it

**Fix:** Link to the page from a page about its topic, or include it in one. A site whose own navigation, such as a sidebar, lists the page still reaches it, so this is a hint: acknowledge one such page in its `intended` frontmatter, or turn the check off in [`[checks]`](../reference/content-model.md#19-checks) (`page-orphan = "off"`) when your navigation lists every page.

#### ASC149 `fragment-unused`

Advice · file level · next step: review · configurable in `[checks]` · acknowledged in the page's `intended` frontmatter · [SPEC §4.2]({repo}/blob/main/SPEC.md#42-include)

**When:** No file includes the fragment.

**Message:** no file includes this fragment, so nothing publishes it

**Fix:** Include the fragment where it belongs with `@include`, or delete it when nothing needs it any more.

#### ASC150 `phrase-unused`

Advice · file level · next step: review · configurable in `[checks]` · acknowledged in `[[intended]]` · [SPEC §5.1]({repo}/blob/main/SPEC.md#51-phrases)

**When:** No file writes the phrase, in its text or in a frontmatter field that takes phrases.

**Message:** no file uses the phrase `{key}`

**Fix:** Write the phrase where its value belongs, or remove it from `[phrases]`. A phrase is used wherever a page or a fragment writes its key in braces, in the text or in a frontmatter field that takes phrases.

#### ASC151 `feature-unused`

Advice · file level · next step: review · configurable in `[checks]` · acknowledged in `[[intended]]` · [SPEC §4.4]({repo}/blob/main/SPEC.md#44-available)

**When:** No availability spec names the feature: no `@available`, table row's `available`, or page's `available`.

**Message:** no file uses the feature `{key}`

**Fix:** Mark the content the feature covers with its key (`@available: <key>`, or `available: <key>` in a page's frontmatter), or remove the feature from `[features]` once nothing needs it.

#### ASC152 `glossary-term-unused`

Advice · file level · next step: review · configurable in `[checks]` · acknowledged in `[[intended]]` · [SPEC §5.4]({repo}/blob/main/SPEC.md#54-glossary-terms)

**When:** The term, and each of its aliases, appears in no file's prose. A term with `match = "marked"` isn't reported.

**Message:** the glossary term `{term}` appears in no file's prose, so it's never linked

**Fix:** Check the term and its `aliases` against how the pages write it: occurrences match whole words, in prose only, and ignore case unless the term is case-sensitive. Remove the term when the pages don't need it. A term with `match = "marked"` isn't reported, since its uses are links to its page.

#### ASC153 `image-unused`

Advice · file level · next step: review · configurable in `[checks]` · acknowledged in `[[intended]]` · [SPEC §5.3]({repo}/blob/main/SPEC.md#53-images)

**When:** An image file under the content root that no file shows or links to.

**Message:** no file shows or links to `{path}`

**Fix:** Delete the image when nothing needs it. An image used only from raw HTML, or by the site's own code, is reported too, since Ascribe doesn't read either: turn the check off in [`[checks]`](../reference/content-model.md#19-checks) when your site uses images that way.

#### ASC154 `image-large`

Advice · file level · next step: review · configurable in `[checks]` · acknowledged in `[[intended]]` · [SPEC §5.3]({repo}/blob/main/SPEC.md#53-images)

**When:** An image file under the content root is larger than the limit: 500 KB, or `[checks.image-large] limit`.

**Message:** `{path}` is \{size}, over the \{limit} limit for an image

**Fix:** Make the image smaller: save it at the size it's shown, or compress it (a screenshot as an optimized PNG, a photo as JPEG or WebP). The limit is 500 KB; set another in `ascribe.toml`, such as `[checks.image-large] limit = "1 MB"`.

#### ASC155 `title-duplicate`

Advice · page level · next step: write · configurable in `[checks]` · [SPEC §2.2]({repo}/blob/main/SPEC.md#22-pages-and-fragments)

**When:** Two pages the build publishes have the same title, ignoring case.

**Message:** `{title}` is also the title of \{others}, so readers can't tell the pages apart in search results or a list of pages

**Fix:** Give each page a title that says what sets it apart. When one page replaces the other, remove the old one, or publish only one of them in each build with `available`.
