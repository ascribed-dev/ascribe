# Presenting changes to prose and rendered documents (state as of October 2026)

Notes on sourcing. Each finding is tagged where it matters:

- **[fetched]** means the page itself was retrieved and read during this research.
- **[snippet]** means the claim comes from a search-result summary of that page; the page was not opened (or returned a 404 / redirect when fetched). Treat these as leads to verify before quoting.
- "Documented behaviour" is the vendor's or spec's own statement. "Opinion" is a practitioner or contributor view.

## 1. How do document tools (Word, Google Docs, Notion, GitBook, Mintlify, Wikipedia visual diff, Draftable) present prose changes, and what do they call their view modes?

### Takeaway
Every mainstream tool offers the same three states under different names: a marked-up view, a clean "after" view, and a clean "before" view. Word additionally has a fourth, low-noise state (Simple Markup: clean text plus a margin bar), and the universal inline convention is underline or colour for insertions and strikethrough for deletions.

### Cited Findings

**Microsoft Word (documented behaviour)**
- Four display modes, with Microsoft's wording: Simple Markup "displays tracked changes with a red line in the margin"; All Markup "displays tracked changes with different colors of text and lines for each reviewer"; No Markup "hides the markup to show the document with changes incorporated"; Original "displays the original document without tracked changes and comments showing." [fetched] — [Microsoft Support: Track changes in Word](https://support.microsoft.com/en-us/office/track-changes-in-word-197ba630-0f5f-4a8e-9a77-3712475e806a)
- "Deletions are marked with a strikethrough, and additions are marked with an underline." Different reviewers get different colours. [fetched] — [Microsoft Support: Track changes in Word](https://support.microsoft.com/en-us/office/track-changes-in-word-197ba630-0f5f-4a8e-9a77-3712475e806a)
- Revisions can instead be shown as balloons: "If you choose to show revisions as balloons, they display in the margins of the document." [fetched] — [Microsoft Support: Track changes in Word](https://support.microsoft.com/en-us/office/track-changes-in-word-197ba630-0f5f-4a8e-9a77-3712475e806a)
- Original mode does not remove pending changes; unaccepted changes remain in the document. [snippet] — [Microsoft Support: Track changes in Word](https://support.microsoft.com/en-us/office/track-changes-in-word-197ba630-0f5f-4a8e-9a77-3712475e806a)

**Google Docs (documented behaviour)**
- In suggesting mode: "You'll see your change in a new color. Anything you delete will be crossed out." [fetched] — [Google Docs Editors Help: Suggest edits](https://support.google.com/docs/answer/6033474)
- Review is under Tools, then "Review suggested edits"; "To preview what your document will look like with or without the changes, click the Down arrow and choose an option." Bulk actions are "Accept all" or "Reject all". [fetched] — [Google Docs Editors Help: Suggest edits](https://support.google.com/docs/answer/6033474)
- The three options in that drop-down are labelled "Show Suggested Edits", "Preview 'Accept All'", and "Preview 'Reject All'". [snippet, third-party tutorial rather than Google] — [The Write Life: track changes in Google Docs](https://thewritelife.com/google-docs-adds-track-changes-editing-heres-use/)
- Document modes are named Editing, Suggesting, Viewing. [snippet] — [PCWorld: How to track changes in Google Docs](https://www.pcworld.com/article/606677/how-to-track-changes-in-google-docs.html)

**Notion**
- Version history lets users "review highlighted changes", see who made each change, and preview what will be affected before restoring. It tracks text edits, blocks added or deleted, some non-text changes (images, to-dos, callouts, colours) and some simple table and database changes. [snippet] — [Notion Help: Delete & restore content](https://www.notion.com/help/duplicate-delete-and-restore-content)
- Added text is shown in green; deleted or replaced text in red strikethrough. [snippet, third-party blog, not Notion] — [Sparxno: Notion page version history](https://www.sparxno.com/blog/notion-page-version-history)

**GitBook**
- "Diff view" opens from the Changes tab of a change request and highlights every page and block that has been edited. [snippet] — [GitBook Docs: Change requests](https://gitbook.com/docs/collaboration/change-requests)
- Two table-of-contents scopes: "Show all pages" (default; changed and unchanged pages, for seeing edits in the context of the whole space) and "Show only changed pages" (for focus in large spaces). [snippet] — [GitBook Docs: Change requests](https://gitbook.com/docs/collaboration/change-requests)
- Default layout is a split view with "before" on the left and "after" on the right; a diff-mode button switches to inline single-column. [snippet] — [GitBook Docs: Change requests](https://gitbook.com/docs/collaboration/change-requests)
- July 2025 changelog: diff view now shows when a page's title or description changed, and the Changes section uses a file explorer so reviewers can see everything a change request touched and jump to it. [snippet] — [GitBook changelog, 15 July 2025](https://gitbook.com/docs/changelog/july-2025/15-july-variables-redesigned-search-better-diff-view-and-more)

**Mintlify (documented behaviour)**
- In the web editor, clicking a changed file in the publish menu opens it in diff view against the published version; exit by clicking "Viewing changes" in the top bar. [fetched] — [Mintlify Docs: Review changes](https://www.mintlify.com/docs/editor/review)
- "Visual mode shows a visual diff and source mode shows a text diff." [fetched] — [Mintlify Docs: Review changes](https://www.mintlify.com/docs/editor/review)
- "Files that can't display a diff, such as images or deleted files, appear in the list but aren't clickable." [fetched] — [Mintlify Docs: Review changes](https://www.mintlify.com/docs/editor/review)
- Hidden pages in the branch diff can be opened from the publish menu even though they are not in site navigation. [fetched] — [Mintlify Docs: Review changes](https://www.mintlify.com/docs/editor/review)
- Pull requests get a preview deployment: a temporary URL rendering the changes as published, rebuilt on each save. This is a clean "after" view, not a diff. [snippet] — [Mintlify Docs: Review changes](https://www.mintlify.com/docs/editor/review#live-preview)

**Wikipedia / MediaWiki visual diff (documented behaviour)**
- Visual diffs have been available since 2017, both when previewing changes before saving and from the history page's standard diff display. [snippet] — [MediaWiki: VisualEditor/Diffs](https://www.mediawiki.org/wiki/VisualEditor/Diffs)
- Users "will see additions, removals, new links, and formatting highlighted"; a toggle button switches "between visual and wikitext diffs". [fetched] — [MediaWiki: VisualEditor/Diffs](https://www.mediawiki.org/wiki/VisualEditor/Diffs)
- Changes with no visible text footprint are described in the margin: "Other changes, such as changing the size of an image, are described in notes on the side." [fetched] — [MediaWiki: VisualEditor/Diffs](https://www.mediawiki.org/wiki/VisualEditor/Diffs)
- Stated limitations: it does not show changes to "invisible page metadata, such as categories or TOC keywords", has trouble with complex table modifications, and is not available on undo diff pages or edit conflicts. [fetched] — [MediaWiki: VisualEditor/Diffs](https://www.mediawiki.org/wiki/VisualEditor/Diffs)

**Draftable**
- Side-by-side view with synchronised scrolling; a switch to "single page view" shows all changes as a redline. [snippet] — [Draftable: Compare](https://www.draftable.com/compare)
- A change list shows every change together; clicking one jumps to that point in the document. [snippet] — [Draftable: Compare](https://www.draftable.com/compare)
- Redline output uses distinct formatting and colours for deleted, inserted and moved text; moved-text detection is an optional setting. [snippet] — [Draftable Help: Redline comparison settings](https://help.draftable.com/hc/en-us/articles/16549612535705-Redline-comparison-settings)
- Draftable Legal can view and export "changed pages only". [snippet, title only] — [Draftable Help: changed pages only](https://help.draftable.com/hc/en-us/articles/20294100319385-How-to-view-and-export-changed-pages-only-in-Draftable-Legal)

### View-mode label comparison (compiled from the findings above)

| Tool | Marked-up view | Clean "after" | Clean "before" | Extra |
|---|---|---|---|---|
| Word | All Markup | No Markup | Original | Simple Markup (clean + margin line) |
| Google Docs | Show suggested edits | Preview "Accept all" | Preview "Reject all" | |
| GitBook | Diff view, inline | right pane of split view | left pane of split view | all pages / only changed pages |
| Mintlify | Visual diff / source diff | preview deployment | published version | |
| MediaWiki | Visual / Wikitext toggle | | | side notes for invisible changes |
| Draftable | single page (redline) | side-by-side right | side-by-side left | change list |

### Inferences
- A three-way toggle labelled "Changes / As it will be / As it was" maps directly onto Word's All Markup / No Markup / Original and Google's Show / Preview accept all / Preview reject all. The plain-language labels are arguably clearer than Word's: "No Markup" and "Original" are both clean views and Word's names do not say which is before and which is after.
- Word's Simple Markup is a precedent for a fourth, low-noise state: final text plus a margin bar only. A tool that already draws a bar on every changed block could offer this cheaply, as an option on the "As it will be" view.
- MediaWiki's side notes are the established answer for changes that leave no visible trace in rendered output (attributes, image size, link targets). GitHub uses tooltips for the same job (see question 2).
- GitBook's "only changed pages" filter and Draftable's "changed pages only" are precedents for scoping a multi-page review to what changed while keeping the full-site context available.

### Gaps
- GitBook's exact marking of added, edited and deleted blocks (colours, icons, labels) could not be confirmed: every GitBook docs URL returned a 404 or an unfollowed redirect to the fetch tool. All GitBook findings are snippet-level.
- Word's Microsoft Support page, as fetched, does not describe moves or the "changed lines" margin-bar setting in detail. Moves are covered in question 3 from secondary sources.
- Notion's own description of how additions and deletions are coloured was not fetched; the green / red-strikethrough claim rests on a third-party blog.
- No documentation found on git-based CMSs (Decap, TinaCMS, Keystatic and similar) offering a rendered prose diff. Not searched specifically; unknown rather than confirmed absent.

## 2. How do GitHub rich diff, GitLab, htmldiff/daisydiff and similar render changes in rendered HTML, and what are the known limitations?

### Takeaway
GitHub's rendered prose diff is documented as green and red backgrounds on the rendered page plus tooltips for invisible attribute changes, but it cannot take review comments and is documented (and reported by users) to fail on complex changes. Algorithmic HTML diffing has structural weaknesses: it sees only visible content and can produce awkward or fragmented ins/del output.

### Cited Findings

**GitHub rich diff (documented behaviour)**
- Supported for formats handled by github/markup: "Markdown, AsciiDoc, Textile, ReStructuredText, Rdoc, Org, Creole, MediaWiki, Pod". [fetched] — [GitHub Docs: Working with non-code files](https://docs.github.com/en/repositories/working-with-files/using-files/working-with-non-code-files)
- Two views: "The source view shows the raw text that has been typed, while the rendered view shows how that text would look once it's rendered on GitHub." [fetched] — [GitHub Docs: Working with non-code files](https://docs.github.com/en/repositories/working-with-files/using-files/working-with-non-code-files)
- Added content gets "a green background" and removed text "a red background". [fetched] — [GitHub Docs: Working with non-code files](https://docs.github.com/en/repositories/working-with-files/using-files/working-with-non-code-files)
- Invisible changes: "we provide a tooltip describing changes to attributes that, unlike words, would not otherwise be visible in the rendered document." [fetched] — [GitHub Docs: Working with non-code files](https://docs.github.com/en/repositories/working-with-files/using-files/working-with-non-code-files)
- Limitation, commenting: "Commit comments can only be added to files within the *source* view, on a line-by-line basis." [fetched] — [GitHub Docs: Working with non-code files](https://docs.github.com/en/repositories/working-with-files/using-files/working-with-non-code-files)
- Limitation, failure on complex changes: GitHub may fail to generate a rendered view for complex documents with extensive changes, in which case "you can still use the source view to analyze and comment on changes." Documents with embedded HTML should be reviewed "in both the rendered and source views for completeness." [fetched] — [GitHub Docs: Working with non-code files](https://docs.github.com/en/repositories/working-with-files/using-files/working-with-non-code-files)

**GitHub rich diff (user reports, opinion)**
- Community discussion opened 16 January 2023 reports that rich diff on modified existing Markdown files shows the error "we're unable to render the document prior to diffing", while it works for newly created files. Another user confirmed on 24 June 2024; one commenter called it a years-long issue that is "intermittently unreliable". No GitHub staff response is recorded. [fetched] — [GitHub Community Discussion #44477](https://github.com/orgs/community/discussions/44477)

**GitLab**
- Merge request diffs offer Inline mode ("often better for changes to single lines") and Side-by-side mode ("often better for changes affecting large numbers of sequential lines"), chosen from Preferences. [fetched] — [GitLab Docs: Changes in merge requests](https://docs.gitlab.com/user/project/merge_requests/changes/)
- Generated files (lock files, minified JS/CSS, node_modules and others) are collapsed by default, configurable with the `gitlab-generated` attribute in `.gitattributes`. [fetched] — [GitLab Docs: Changes in merge requests](https://docs.gitlab.com/user/project/merge_requests/changes/)
- A rendered Markdown diff in merge requests is tracked as a feature request, "Provide option to render markdown changes in merge request diffs". [snippet] — [GitLab issue 17000](https://gitlab.com/gitlab-org/gitlab/-/issues/17000)
- A related issue, "Option to toggle between rendered and raw diff", concerns rendered diffs such as Jupyter notebooks; some users objected that they want to review the actual source because conversion can produce misleading diffs. [snippet, opinion] — [GitLab issue 346764](https://gitlab.com/gitlab-org/gitlab/-/issues/346764)

**HTML diff libraries**
- DaisyDiff only detects visible HTML changes; `title`, `meta` and `script` nodes are stripped before diffing. [snippet] — [DaisyDiff mailing list, issue 51](https://groups.google.com/g/daisydiff/c/pTUBLByiTgk)
- htmldiff-js (port of htmldiff.net) states its algorithm "isn't perfect"; for example, when a new tag ends in the same string as the previous tag, it produces two separate change tags around the common string. [snippet] — [dfoverdx/htmldiff-js](https://github.com/dfoverdx/htmldiff-js)
- Injecting diff tags into HTML requires syntax awareness to avoid placing tags inside attributes or other invalid positions. [snippet, opinion from an issue thread] — [dandavison/delta issue 1703](https://github.com/dandavison/delta/issues/1703)
- One project argued for diffing the source (DocBook) rather than the HTML because rendered diffs mangle formatting, spacing and line breaking. [snippet, opinion] — [int19h/cll issue 127](https://github.com/int19h/cll/issues/127)
- GOV.UK maintains an HTML diffing library, nokodiff. [snippet, title only] — [GOV.UK developer docs: nokodiff](https://docs.publishing.service.gov.uk/repos/nokodiff.html)
- GitBook published an open-source React component for rich diffs between Markdown/HTML documents. [snippet; historical, maintenance status not checked] — [GitbookIO/react-rich-diff](https://github.com/GitbookIO/react-rich-diff)

**git word diff (documented behaviour)**
- `--word-diff=plain` shows words as `[-removed-]` and `{+added+}`; `color` mode highlights "using only colors". [fetched] — [git-diff documentation](https://git-scm.com/docs/git-diff)
- Git's word diff is computed inside the line diff's hunks: it "operates by taking the same line-by-line diff that is produced without the option and computing word-by-word changes within each hunk. This may produce a larger diff than a dedicated word-diff tool would." [fetched] — [git-diff documentation](https://git-scm.com/docs/git-diff)
- `--word-diff-regex` defines what counts as a word; `--word-diff-regex=.` gives character-level diffs. [fetched] — [git-diff documentation](https://git-scm.com/docs/git-diff)

### Inferences
- GitHub's two documented weaknesses (no commenting in the rendered view, and failure on complex changes with fallback to source) are the clearest gaps a dedicated docs-PR review tool can fill. The tool should define an explicit fallback for blocks it cannot diff at word level, for example showing the block as removed plus added, and never fail the whole page.
- A two-level model (decide block status first, then word-diff only inside blocks classed as "changed") avoids the worst htmldiff failure modes, because ins/del never has to wrap or cross block boundaries.
- Git's `[-removed-]{+added+}` plain mode is a precedent for non-colour textual delimiters around changed words, relevant to copy-paste and to accessibility (question 6).
- Tooltips (GitHub) and side notes (MediaWiki) are the two established ways to surface changes that do not alter rendered text, such as link targets, image sources and alt text. Tooltips are hover-only and so weaker for keyboard and touch users; side notes are always visible.

### Gaps
- No primary source found on diff-match-patch's semantic cleanup behaviour or on "unreadable interleaved del/ins" as a named problem; not searched specifically. The interleaving concern is widely discussed but uncited here.
- GitLab's current (2026) support for rendered Markdown diffs in merge requests was not confirmed. The fetched changes page mentions no rendered toggle for Markdown; the two issues were seen only as search results and their status is unknown.
- No GitHub statement found on why rich diff fails or whether it is being fixed.

## 3. Moved content: how is it shown, and what guidance exists on showing origin and destination?

### Takeaway
Tools that detect moves mark both ends: the origin with a deletion-like style and the destination with an insertion-like style, in a colour distinct from ordinary insert/delete, and the better ones link the two ends so a click jumps between them. Wikimedia's design discussion is the richest source of guidance and warns that showing moved content twice in full gets noisy.

### Cited Findings

**Word**
- With "Track Moves" on, moved material appears in green double strikethrough at the old position and green double underline at the new position. [snippet, forum answer] — [WordBanter: green double-strikethrough and double-underline](https://www.wordbanter.com/showthread.php?t=130918)
- Advanced Track Changes Options has separate "Moved from" (default double strikethrough) and "Moved to" (default double underline) format settings. [snippet] — [Word Tips: Changing how changes are noted](https://word.tips.net/T000040_Changing_How_Changes_are_Noted_in_Word.html)
- Only cut and paste counts as a move; copy and paste does not. [snippet, forum answer] — [microsoft.public.word.docmanagement thread](https://groups.google.com/g/microsoft.public.word.docmanagement/c/Rq2du-ii_SY)
- Word's object model has distinct constants for move marks. [snippet, title only] — [Microsoft Learn: WdMoveFromTextMark](https://learn.microsoft.com/en-us/OFFICE/VBA/API/word.wdmovefromtextmark)

**MediaWiki wikitext diff (two-column)**
- Before the 2018 change, "whenever a part of a text was simply moved to another position on the page, it was displayed as if it had been removed and some other text had been added." The improved diff marks moved chunks with small arrows and highlights changes made inside moved paragraphs. Detecting moves "isn't trivial" and had performance implications. [fetched] — [Wikimedia Diff blog, 24 Aug 2018](https://diff.wikimedia.org/2018/08/24/wikimedia-diff-comparison-improved/)
- Moved paragraphs on the left get a yellow border and a right-facing arrow; on the right a blue border and a left-facing arrow; clicking an arrow jumps between the moved-from and moved-to locations. [snippet] — [Wikipedia Help:Diff](https://en.wikipedia.org/wiki/Help:Diff)

**MediaWiki visual diff**
- If an edit swaps two paragraphs, with or without edits inside them, the visual diff identifies that this is what happened, whereas a wikitext diff shows a deletion plus an unrelated addition. [fetched] — [MediaWiki: VisualEditor/Diffs](https://www.mediawiki.org/wiki/VisualEditor/Diffs)
- Design critique (opinion, Wikimedia designer Pginer-WMF, 2017): "the current triangles are not obvious to understand where a paragraph was removed from and added to." [fetched] — [Phabricator T169325](https://phabricator.wikimedia.org/T169325)
- Proposals in the same task: replace triangles with arrows showing movement direction; "move markers" with a text excerpt, blue colour coding, acting as links that scroll to the destination; hover to highlight both ends; show only an up/down indicator at the destination without rendering the origin. [fetched] — [Phabricator T169325](https://phabricator.wikimedia.org/T169325)
- Concern (opinion, developer Tchanders): whether "moved content should be shown twice in its entirety", since it complicates diffs with several moves; suggestion to mark only one paragraph per swap. Another developer noted "excess moves" being shown as a separate bug. [fetched] — [Phabricator T169325](https://phabricator.wikimedia.org/T169325)
- Subtask "Visual diff: Show moves in the sidebar" (T270561) was closed as resolved on 24 December 2020; the parent task remains open and parked. [fetched] — [Phabricator T169325](https://phabricator.wikimedia.org/T169325)

**git**
- `--color-moved` colours moved lines differently from added/removed. `plain` mode catches any moved line but "is not very useful in a review to determine if a block of code was moved without permutation." [fetched] — [git-diff documentation](https://git-scm.com/docs/git-diff)
- `blocks` mode only treats text as moved when the block has "at least 20 alphanumeric characters"; `zebra` alternates two colours so adjacent moved blocks can be told apart; `dimmed-zebra` dims the uninteresting interior and keeps the block borders prominent. [fetched] — [git-diff documentation](https://git-scm.com/docs/git-diff)

**Other**
- Draftable uses a distinct colour and format for moved text, and move detection is an optional setting. [snippet] — [Draftable Help: Redline comparison settings](https://help.draftable.com/hc/en-us/articles/16549612535705-Redline-comparison-settings)
- A third-party tool, prosediff, describes following "text that moved even when it was edited" and aligning each paragraph with the one it came from. [snippet, project README] — [raffaelemancuso/prosediff](https://github.com/raffaelemancuso/prosediff)

### Inferences
- Consensus pattern: (a) a visual treatment for moves distinct from add and remove (Word green double lines; MediaWiki yellow/blue borders with arrows; git separate moved colours), (b) both ends marked, (c) a link between the ends. A bar plus a "moved from" link at the destination covers (a) and (c) but only the destination end; the practice in Word and MediaWiki is to leave a marker at the origin too.
- Given Wikimedia's worry about showing moved text twice, the low-noise version is a one-line stub at the origin ("moved to ...", linked) and full content at the destination. This matches the "move markers with excerpt" proposal and reuses a collapsed-removed-block style.
- Edits inside a moved block should still get word-level highlights. MediaWiki shipped this explicitly in 2018 and prosediff advertises it.
- Git's 20-character minimum and Wikimedia's "excess moves" bug both show that move detection needs a size threshold; short blocks (one-line paragraphs, list items, headings) produce false-positive moves.
- Direction ("moved up" / "moved down") was proposed at Wikimedia as useful low-cost information for the reader.
- Cross-page moves are not addressed by any source found; all precedents are within one document.

### Gaps
- GitHub's lack of move detection in pull request diffs is common knowledge but no primary GitHub source stating it was found.
- Microsoft's own support article describing move formatting was not located; Word move details come from forums and a tips site.
- The current shipped appearance of moves in the MediaWiki visual diff (after the 2020 sidebar work) was not verified against a live example.
- No formal usability study on move presentation was found.

## 4. Removed content: inline struck-through, collapsed, or hidden with a marker? How much to show?

### Takeaway
Tools split into two camps: show deletions inline struck through (Word All Markup, Google Docs, Notion, MediaWiki visual diff, GitHub rich diff) or hide the deleted text and leave only a marker (Word Simple Markup's margin line, Word balloons). No source found gives explicit guidance on how much deleted text to show; collapsing removed blocks to one expandable line is a reasonable middle position without direct precedent in the sources read.

### Cited Findings
- Word All Markup shows deletions inline with strikethrough; Simple Markup hides inline markup and shows only "a red line in the margin". [fetched] — [Microsoft Support: Track changes in Word](https://support.microsoft.com/en-us/office/track-changes-in-word-197ba630-0f5f-4a8e-9a77-3712475e806a)
- Word can move revisions out of the text into margin balloons. [fetched] — [Microsoft Support: Track changes in Word](https://support.microsoft.com/en-us/office/track-changes-in-word-197ba630-0f5f-4a8e-9a77-3712475e806a)
- Google Docs: "Anything you delete will be crossed out", inline. [fetched] — [Google Docs Editors Help: Suggest edits](https://support.google.com/docs/answer/6033474)
- GitHub rich diff shows removed text inline with a red background. [fetched] — [GitHub Docs: Working with non-code files](https://docs.github.com/en/repositories/working-with-files/using-files/working-with-non-code-files)
- Mintlify lists deleted files in the changed-file list but they are not clickable and show no diff. [fetched] — [Mintlify Docs: Review changes](https://www.mintlify.com/docs/editor/review)
- GitLab collapses low-value files by default and lets the reader expand unchanged context in steps ("Previous 20 lines" / "Next 20 lines"). [fetched] — [GitLab Docs: Changes in merge requests](https://docs.gitlab.com/user/project/merge_requests/changes/)
- Practitioner advice (opinion): always read a clean copy after reviewing tracked changes, because markup hides errors in the resulting text. [snippet, title and summary] — [Sophie Michals, Medium](https://sophiemichals.medium.com/editing-tip-always-read-a-clean-copy-after-tracking-changes-a79f9fa1b21d)
- For removed text, screen readers by default mostly do not announce that it is deleted, so struck-through text may be read as if current (see question 6). [fetched] — [MDN: del element](https://developer.mozilla.org/en-US/docs/Web/HTML/Reference/Elements/del)

### Inferences
- Collapsing a wholly removed block to one struck-through line with an expander sits between Word's two modes: more information than a Simple Markup margin line, less interruption than All Markup's full struck paragraph. Progressive disclosure of this kind is established for unchanged or low-value content (GitLab), though the sources do not show it applied to deleted prose.
- Word-level deletions inside a changed block should stay inline, since every tool surveyed does that; collapsing is for whole removed blocks only.
- Mintlify's non-clickable deleted files are a weak spot worth avoiding: a deleted page is a meaningful change a docs reviewer may want to read.
- The "read a clean copy" advice supports making the clean after view a first-class mode: deleted text left inline interrupts reading flow, and reviewers need to check the result reads correctly.
- Because assistive technology may read deleted text as live text, collapsed-by-default removed blocks also reduce the risk of a screen reader user mistaking removed content for current content, provided the collapsed stub carries a text label such as "Removed".

### Gaps
- No explicit guideline (vendor, NN/g, GOV.UK or academic) found on how much deleted text to display, or on thresholds for collapsing.
- No source found comparing reviewer performance with inline versus collapsed deletions.

## 5. Change navigation: next/previous, counts, change lists, and jumping into hidden content

### Takeaway
Next/Previous stepping plus a count and a clickable list of changes is the standard set (Word, Draftable, GitBook). No source read documents how tools handle stepping into content hidden in tabs or collapsed sections.

### Cited Findings
- Word: "select Tracking, then select Next to view the next tracked change" and likewise Previous. [fetched] — [Microsoft Support: Track changes in Word](https://support.microsoft.com/en-us/office/track-changes-in-word-197ba630-0f5f-4a8e-9a77-3712475e806a)
- Word Reviewing Pane: "The summary section at the top of the Reviewing Pane displays the exact number of visible tracked changes and comments that remain in your document." [fetched] — [Microsoft Support: Track changes in Word](https://support.microsoft.com/en-us/office/track-changes-in-word-197ba630-0f5f-4a8e-9a77-3712475e806a)
- Google Docs: the review box has up and down arrows for stepping through individual suggested edits. [snippet, third-party tutorial; Google's own help page as fetched does not mention this] — [Docswrite: accept all changes in Google Docs](https://blog.docswrite.com/how-to-accept-all-changes-in-google-docs-a-step-by-step-guide)
- Draftable: all changes in one list; clicking a change goes to that point in the document. [snippet] — [Draftable: Compare](https://www.draftable.com/compare)
- GitBook: file explorer in the Changes section lets reviewers see everything a change request touched and jump to it; table of contents can be filtered to changed pages only. [snippet] — [GitBook changelog, 15 July 2025](https://gitbook.com/docs/changelog/july-2025/15-july-variables-redesigned-search-better-diff-view-and-more); [GitBook Docs: Change requests](https://gitbook.com/docs/collaboration/change-requests)
- GitLab: "Show file browser" gives a tree or list of changed files, toggled with the F key. [fetched] — [GitLab Docs: Changes in merge requests](https://docs.gitlab.com/user/project/merge_requests/changes/)
- MediaWiki: clicking a move arrow jumps between the two ends of a move. [snippet] — [Wikipedia Help:Diff](https://en.wikipedia.org/wiki/Help:Diff)
- A third-party guide exists specifically on navigating Word tracked changes. [snippet, title only] — [Apt Words: Navigating tracked changes](https://aptwords.co.uk/navigating-tracked-changes/)

### Inferences
- Word's count is of *visible* changes remaining, so the number follows the current filter. A "change 3 of 17" indicator should likewise be explicit about its scope (this page or the whole pull request).
- Two navigation levels are the norm: across files/pages (GitBook file explorer, GitLab file browser) and within a page (Word Next/Previous). In a multi-page docs review, Next at the last change on a page should have defined behaviour: stop, wrap, or continue to the next changed page.
- A change list (Word Reviewing Pane, Draftable) doubles as an accessible alternative to visual scanning: a text list of changes with types and links works for screen reader users regardless of ins/del support.
- For content hidden in tabs, accordions or collapsed removed blocks, the consistent behaviour would be for Next/Previous to reveal the container and then scroll and focus; a collapsed removed block should count as one stop. This is a design inference with no source behind it.

### Gaps
- No documentation found on how any tool navigates to changes inside hidden UI (tabs, accordions, collapsed sections). This looks like an unaddressed area; treat as a design decision to test.
- Keyboard shortcuts for next/previous change in Word and Google Docs were not captured from primary sources.
- Whether GitHub's rich diff offers per-change navigation was not confirmed; the docs page read does not mention any.

## 6. Accessibility: colour, ins/del semantics, ARIA roles, palettes and contrast

### Takeaway
Colour alone fails WCAG 1.4.1, so each change type needs a non-colour cue (underline, strikethrough, bar style, text label). The semantic `ins`/`del` elements are correct to use but screen reader support is inconsistent and disputed between sources, so text labels and a change list are the dependable channel.

### Cited Findings

**WCAG 1.4.1 Use of Color (normative text and W3C guidance)**
- "Color is not used as the only visual means of conveying information, indicating an action, prompting a response, or distinguishing a visual element." [fetched] — [W3C: Understanding SC 1.4.1](https://www.w3.org/WAI/WCAG22/Understanding/use-of-color.html)
- "If content is conveyed through colors differing in hue AND lightness with 3:1 contrast ratio or greater, this counts as additional visual distinction." But where perceiving a specific colour matters (green valid, red invalid), an additional indicator is still needed. [fetched; wording as returned by the fetch tool, check against the page before quoting] — [W3C: Understanding SC 1.4.1](https://www.w3.org/WAI/WCAG22/Understanding/use-of-color.html)
- Sufficient techniques include G14 (colour information also available in text) and G182 (additional visual cues when text colour differences convey information). [fetched] — [W3C: Understanding SC 1.4.1](https://www.w3.org/WAI/WCAG22/Understanding/use-of-color.html)
- Applied to diffs by a Wikimedia designer (opinion, Volker_E): marking formatting changes with background colour alone could "fail for certain visually impaired users"; dotted underlines suggested as a non-colour cue. [fetched] — [Phabricator T169325](https://phabricator.wikimedia.org/T169325)

**HTML ins/del and ARIA roles**
- `del` has the implicit ARIA role `deletion`; attributes `cite` (URI explaining the change) and `datetime`. [fetched] — [MDN: del element](https://developer.mozilla.org/en-US/docs/Web/HTML/Reference/Elements/del)
- WAI-ARIA 1.2 is a W3C Recommendation published 6 June 2023 and defines `deletion` as content "marked as removed or content that is being suggested for removal", with a related `insertion` role. [fetched, definition text partly truncated] — [WAI-ARIA 1.2](https://www.w3.org/TR/wai-aria-1.2/)
- MDN also documents an ARIA `suggestion` role. [snippet, title only] — [MDN: ARIA suggestion role](https://developer.mozilla.org/en-US/docs/Web/Accessibility/ARIA/Reference/Roles/suggestion_role)

**Screen reader support (sources conflict)**
- MDN: "The presence of the del element is not announced by most screen reading technology in its default configuration." [fetched] — [MDN: del element](https://developer.mozilla.org/en-US/docs/Web/HTML/Reference/Elements/del)
- Vispero/TPGi (26 January 2023, updated 7 September 2023): "JAWS and NVDA convey ins, del and mark semantics. There is no reliance on the visual style." Also: "NVDA supports role=insertion and role=deletion JAWS does not." NVDA announces the `s` element as "deleted"; JAWS gives no indication. [fetched] — [Vispero: Screen readers support for text level HTML semantics](https://vispero.com/resources/screen-readers-support-for-text-level-html-semantics/)
- Same article: VoiceOver on iOS conveys ins and del; Narrator conveys no HTML text-level semantics. [snippet] — [Vispero: Screen readers support for text level HTML semantics](https://vispero.com/resources/screen-readers-support-for-text-level-html-semantics/)
- Adrian Roselli's timeline: NVDA added `del`/`ins` support in Chrome in September 2018; JAWS 2020.2003.13 added support in March 2020; JAWS 2020.2004.66 rolled it back in May 2020 "in response to customer feedback"; NVDA added `mark` in September 2020. [fetched] — [Adrian Roselli: Tweaking Text Level Styles](https://adrianroselli.com/2017/12/tweaking-text-level-styles.html)
- Conflict: MDN says most screen readers do not announce `del` by default; Vispero (2023) says JAWS and NVDA do; Roselli records JAWS removing the feature in 2020 after user complaints. The sources disagree and none is dated after 2023. — [MDN](https://developer.mozilla.org/en-US/docs/Web/HTML/Reference/Elements/del); [Vispero](https://vispero.com/resources/screen-readers-support-for-text-level-html-semantics/); [Roselli](https://adrianroselli.com/2017/12/tweaking-text-level-styles.html)

**Hidden-text workaround and its cost**
- Technique: visually hidden `::before`/`::after` generated content such as " [deletion start] " / " [deletion end] " and " [insertion start] " / " [insertion end] ". [fetched] — [MDN: del element](https://developer.mozilla.org/en-US/docs/Web/HTML/Reference/Elements/del); [Adrian Roselli](https://adrianroselli.com/2017/12/tweaking-text-level-styles.html)
- MDN caution: "Some people who use screen readers deliberately disable announcing content that creates extra verbosity. Because of this, it is important to not abuse this technique and only apply it in situations where not knowing content has been deleted would adversely affect understanding." [fetched] — [MDN: del element](https://developer.mozilla.org/en-US/docs/Web/HTML/Reference/Elements/del)
- Roselli caution (opinion): "Because inserting hidden text makes for really verbose experiences for non-macOS / non-Narrator, probably don't do it." He advises testing with real users. [fetched] — [Adrian Roselli](https://adrianroselli.com/2017/12/tweaking-text-level-styles.html)
- High contrast: Roselli uses system colours in Windows High Contrast Mode (`Highlight`/`HighlightText` for `mark`, `Highlight` for the `ins` border) via the legacy `-ms-high-contrast` media query. [fetched; historical syntax, the standard successor is `forced-colors`, which this source does not cover] — [Adrian Roselli](https://adrianroselli.com/2017/12/tweaking-text-level-styles.html)

**Colour-blind palettes**
- GitHub released light and dark Protanopia & Deuteranopia themes in public beta on 19 April 2022; the theme "swaps colors like red and green across GitHub.com for orange and blue". [snippet] — [GitHub Changelog, 19 April 2022](https://github.blog/changelog/2022-04-19-protanopia-deuteranopia-colorblind-themes-beta/)
- A separate Tritanopia theme (blue/yellow colour blindness) entered beta the same day. [snippet] — [GitHub Changelog: Tritanopia theme](https://github.blog/changelog/2022-04-19-tritanopia-colorblind-theme-beta/)
- GitHub's stated rationale: colour blindness could affect up to 8% of the world's population, mostly on the red/green spectrum. [snippet] — [GitHub Changelog, 19 April 2022](https://github.blog/changelog/2022-04-19-protanopia-deuteranopia-colorblind-themes-beta/)
- An earlier single colourblind scheme was criticised as inaccessible for people with tritanopia, which led to separate themes. [snippet, title only] — [GitHub Community Discussion #6385](https://github.com/orgs/community/discussions/6385)

### Inferences
- A design where added = solid bar + text label, changed = dashed bar, removed = strikethrough, and word-level insert vs delete = underline vs strikethrough already meets the intent of 1.4.1 if every distinction survives in greyscale. The check to make: are word-level inserts distinguishable from surrounding unchanged text without colour? A background tint alone is not enough; an underline (the Word convention) or similar is needed.
- Solid versus dashed bar is a non-colour cue but a subtle one. A text label on changed, removed and moved blocks as well as added ones (technique G14) is more robust and gives screen readers something to announce.
- Use real `ins` and `del` elements for word-level changes: this is correct semantics, costs nothing, and is announced by NVDA and iOS VoiceOver per Vispero. Do not depend on it, given JAWS's uncertain state and Narrator's lack of support. The dependable channel is block-level visible text labels plus a navigable change list or summary.
- The MDN and Roselli warnings argue against wrapping every word-level change in hidden "[insertion start]" text; in a heavily edited paragraph it becomes unusably verbose. A better trade is one per-block label ("Changed") and letting the user read before and after text via the view toggle. The "As it will be" and "As it was" views are themselves an accessibility feature: they give screen reader users two clean, linear readings.
- Following GitHub, a blue/orange alternative to green/red is the established colour-blind-safe diff palette, with a different treatment needed for tritanopia. If non-colour cues are complete, an alternative palette is a comfort feature and not a conformance requirement.
- Support `forced-colors` mode: background highlights are removed by the browser there, so underline, strikethrough and borders must carry the meaning.

### Gaps
- Contrast requirements for highlight backgrounds in light and dark themes were not researched to a source. The relevant criteria would be WCAG 1.4.3 (text contrast against the tinted background) and 1.4.11 (non-text contrast for bars and borders), but their understanding documents were not fetched, so no thresholds are cited here.
- No GitHub publication on accessible diff markup (beyond colour themes) was found; GitHub's diff accessibility engineering work is not covered.
- No screen reader test data newer than September 2023 was found; the 2026 state of JAWS, VoiceOver macOS and Narrator for ins/del and the ARIA roles is unverified.
- The full ARIA 1.2 definition text for `insertion`, and whether naming is prohibited on these roles, was truncated in the fetch.

## 7. UX research and guidance on prose diff granularity (word vs sentence vs block)

### Takeaway
No formal research or style guidance on preferred granularity for prose diffs was found; the available evidence is tool convention (word-level inside changed paragraphs) and a few practitioner observations.

### Cited Findings
- MediaWiki's documentation page asserts a user preference for the visual diff layout and that it helps users understand their changes better. [snippet; the fetched page text did not contain formal research findings, so the basis for the claim is unverified] — [MediaWiki: VisualEditor/Diffs](https://www.mediawiki.org/wiki/VisualEditor/Diffs)
- Wikimedia Germany's move-detection work originated from a community wishlist item, "Show text changes when moving text chunks", indicating editor demand for move-aware prose diffs. [snippet] — [Meta-Wiki: WMDE Technical Wishes](https://meta.wikimedia.org/wiki/WMDE_Technical_Wishes/Show_text_changes_when_moving_text_chunks)
- Wikimedia's Community Tech team kept notes on an improved diff compare screen. [snippet, title only] — [Meta-Wiki: Improved diff compare screen notes](https://meta.wikimedia.org/wiki/Community_Tech/Improved_diff_compare_screen/Notes)
- GitLab's own guidance on layout: inline mode "is often better for changes to single lines"; side-by-side "is often better for changes affecting large numbers of sequential lines." [fetched] — [GitLab Docs: Changes in merge requests](https://docs.gitlab.com/user/project/merge_requests/changes/)
- A search summary of scholarly-publishing material reports that reviewers find tracked changes time-consuming to work with when revisions are major. [snippet; the specific source for this statement was not identified in the results, treat as unverified] — [PMC: Your article needs revision?](https://www.ncbi.nlm.nih.gov/pmc/articles/PMC8495623/)
- Tool convention: Word, Google Docs, Notion and GitHub rich diff all mark changes at word or phrase level inline. — [Microsoft Support](https://support.microsoft.com/en-us/office/track-changes-in-word-197ba630-0f5f-4a8e-9a77-3712475e806a); [Google Docs Help](https://support.google.com/docs/answer/6033474); [GitHub Docs](https://docs.github.com/en/repositories/working-with-files/using-files/working-with-non-code-files)
- Third-party prose diff tools describe a two-level approach: block-level alignment plus word- or character-level highlighting inside matched blocks. [snippet, project READMEs] — [PiotrTrzpil/markdown-diff-viewer](https://github.com/PiotrTrzpil/markdown-diff-viewer); [raffaelemancuso/prosediff](https://github.com/raffaelemancuso/prosediff)

### Inferences
- The de facto standard is two-level: classify blocks (added, removed, changed, moved), then show word-level changes within changed blocks. The reviewed design matches it.
- GitLab's inline versus side-by-side advice suggests a rule for heavily rewritten paragraphs: when most of a paragraph changed, interleaved word-level marks become hard to read and showing the block as old-then-new is clearer. A change-ratio threshold for falling back from word-level marks to whole-block replacement is a reasonable design choice, though no source gives a number.
- If major revisions do make tracked changes slow to review, that supports clean "As it will be" reading as a primary mode, with markup as something to consult.

### Gaps
- No NN/g article, GOV.UK guidance or academic study on prose diff readability or on word-level versus sentence-level granularity was found. One search returned only unrelated readability-formula literature. This is a real gap in the public evidence, at least as far as two queries could establish.
- GOV.UK's nokodiff documentation and the Wikimedia Community Tech notes may contain relevant design rationale but were not read.
- No source found for a quantitative threshold at which word-level marking should give way to block replacement.
