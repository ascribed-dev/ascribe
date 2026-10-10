<!-- Generated from tests/conformance/diagnostics.toml by tests/conformance/tests/docs.rs. Edit the registry, then run `ASCRIBE_BLESS=1 cargo test -p ascribe-conformance --test docs`. -->

### Pages

#### ASC144 `page-size`

Advice · page level · next step: write · configurable in `[checks]` · [SPEC §9.4]({repo}/blob/main/SPEC.md#94-outputs)

**Message:** this page's Markdown is \{size} characters in build `{build}`, and an agent reads under \{limit} in one piece; split the page, or move long tables and code below the prose

**Fix:** Split the page into pages that each answer one question, or move long tables, reference lists, and code below the prose, where an agent that stops reading early loses the least. The limit is the Web Documentation Delivery Spec's: a page passes under 50,000 characters of Markdown. Set another with `[checks.page-size] limit`.

#### ASC145 `page-description-missing`

Advice · file level · next step: write · configurable in `[checks]` · [SPEC §7.2]({repo}/blob/main/SPEC.md#72-declarations)

**Message:** this page has no `{field}`; write a sentence saying what the page is for, which search results and agents show with its title

**Fix:** Add the field the content type marks with `role = "description"` to the page's frontmatter: a sentence or two saying what the page covers and who it's for.

#### ASC146 `heading-level-skipped`

Advice · page level · next step: fix · configurable in `[checks]` · [SPEC §9.4]({repo}/blob/main/SPEC.md#94-outputs)

**Message:** this heading is level \{level}, after a level-\{previous} heading; make it level \{expected}, so the outline has no gap

**Fix:** Make each heading one level below the heading it belongs under. A page's title is its level-1 heading, so its own headings start at level 2. The quick fix changes the one heading; the headings under it may need to move up too.

#### ASC147 `code-language-missing`

Advice · file level · next step: choose · configurable in `[checks]` · [SPEC §1.4]({repo}/blob/main/SPEC.md#14-relationship-to-commonmark)

**Message:** this code block has no language; write it after the opening fence, or `text` for output

**Fix:** Write the block's language right after the opening fence, such as `sh`, `toml`, or `json`. Use `text` for program output, logs, and anything else that isn't code. A highlighter colors the block by it, and an agent knows what the block is.

#### ASC148 `review-overdue`

Advice · file level · next step: review · configurable in `[checks]` · [SPEC §7.2]({repo}/blob/main/SPEC.md#72-declarations)

**Message:** this page was due for review on \{date}; check it still holds, then move `{field}` to the next review

**Fix:** Read the page against what it describes and correct what has changed, then set its review date to when it's next due. If the page is right as it is, moving the date is the whole fix. The check compares with today's date, so it can start reporting without a change to the project.
