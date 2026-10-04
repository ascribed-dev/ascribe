# Review of the review mockup

A review of [`mockup.html`](mockup.html) for whoever revises it next. It checks the mockup against the [Review plan](README.md) and its phase files, against how it renders, and against research into how document and code-review tools handle the same problems ([report](../../reports/Review%20and%20diff%20UI%20practices.md)).

The mockup's structure is right and should stay: the thread column, markers and a panel at narrow widths, the detached list at the top, collapsed resolved threads, the three show modes, and both ends of a move. What needs work is the site preview's panel, the wording around unsent comments, and a handful of states and labels.

## How it was checked

- Rendered at 1440px and 390px, in light and dark.
- States exercised: reviewing, before review starts, unpushed commits, anchors didn't arrive.
- Try buttons exercised: site preview, hidden tab, removed text, comment where GitHub can't anchor.
- **Not checked:** not signed in, checkout behind, not an Ascribe page, the "As it will be" and "As it was" modes, stepping through changes, and keyboard-only use. Check these when revising.

Line numbers refer to `mockup.html` as of this review and will drift.

## Fix in the mockup

These need no change to the plan.

### 1. The site preview's panel covers the page

The toolbar app's panel (`.app`, `.apppanel`, built around line 766) is always open, pinned to the bottom, and can't be collapsed or moved. It covers the bottom third of the page and sits over the thread column. With the submit form open it covers about half.

A fixed bottom panel that can't be moved is the case WCAG 2.4.11 (Focus Not Obscured) names. Vercel's preview toolbar, the nearest precedent, starts dormant, drags to either side, and can be hidden.

- Start the panel collapsed to the toolbar pill, and open it from the pill.
- Keep the open panel to the controls row. Don't put the submit form in it unchanged; see item 4.
- Add bottom scroll padding to the page while the panel is open.

### 2. Site looks the same as Page at desktop width

The site's nav appears only when the preview pane is at least 880px wide (line 718). At 1440px the pane is 788px, so the nav never shows, and only the orange header strip distinguishes Site from Page. Phase 7's point is the page in the site's own layout. Lower the threshold, or drop the thread column to markers in Site so the nav fits.

### 3. Count markers cover text at narrow widths

The marker pill (`.marker`, line 149) is absolutely positioned at the top right of the block and sits on the end of the first line ("lantern -①", "control①"). Reserve space for it (right padding on blocks that have one) or move it into the margin. It is also under the 24px minimum target size (WCAG 2.5.8); so are the ↑ and ↓ buttons.

### 4. The submit form doesn't show what will be sent

The form (`pendingBar`, line 694) has the three choices and a summary box. GitHub (since January 2026), GitLab and Gerrit all list the unsent comments at submit time so the reviewer can check them. List each one with a link that jumps to its block.

### 5. Discard deletes everything in one click

**Discard** sits beside **Submit review…** on the bar and calls `discard()` (line 910) with no confirmation. GitHub puts Discard inside the submit surface. Move it into the submit form, and confirm with the count and outcome-named buttons: "Discard 3 unsent comments?" with **Discard comments** and **Keep review**.

### 6. Detached threads are labelled "Outdated"

The detached thread carries an **Outdated** badge (line 497). These are two states: outdated means the block exists but its text changed, and detached means no block was found. Phase 5 keeps them separate.

- Give detached threads their own label, and show the original text the comment was on (GitHub supplies the diff hunk).
- Add an example of an outdated thread that is still placed beside its block, with the label and a way to see the original text.

### 7. Changed blocks have no text label

Added, Removed and Moved blocks carry a label (lines 599 to 611). Changed blocks rely on a dashed bar alone. Screen reader support for `ins` and `del` is disputed, so the visible label is the dependable signal. Add "Changed".

### 8. Change navigation has no position

The ↑ and ↓ buttons (`stepChange`, line 895) wrap silently within the page. Word and GitBook show a position and continue to the next changed page.

- Show "Change 3 of 9", and say that the count is for this page.
- At the last change, offer the next changed page instead of wrapping.

### 9. The review header is too tall and wraps

The counts wrap ("· 1 / moved") at 1440px, and the legend (line 683) is a permanent second row. Shorten the counts (for example "9 changes" with the breakdown on hover and focus), and once item 7 is done the legend can go, since every mark will carry its own label.

### 10. Smaller fixes

- **"Before review starts" shows "GitHub: not signed in"** in the status bar (line 841). Review being off and being signed out are separate states.
- **The "ahead" notice has no Push action** (line 690). The "behind" notice has **Pull**. The button there now ("Show a block I can't comment on") is a demo control; move it to the Try row.
- **"Show" next to "Show in preview"** on a collapsed resolved thread in the source pane (lines 501 to 503) reads as a duplicate. Use "Expand".
- **Thread headers wrap in the 250px column.** On the removed-text thread, "Resolve" drops to its own line.
- **Resolve acts at once, and nothing says so.** GitLab and Reviewable batch it, so reviewers can't assume. Add a short note or tooltip.
- **The source on hover (`.where`, line 151) appears on hover and click only.** Show it on keyboard focus too.

## Needs a decision before the mockup changes

These touch the plan's settled decisions or add something no phase builds. The README says not to reopen a decision inside a phase, so raise them with the plan's owner; don't pick silently.

### A. The word "pending"

The bar says "1 pending comment" and comments carry a **Pending** badge (line 499). This is GitHub's own word and its most complained-about one: a GitHub community discussion with 423 upvotes describes reviews left unsent for days to months because people read "Pending" as "waiting on the other person". The relabels users ask for there are "Draft" or "Not yet submitted", with "only visible to you".

The mockup already says "not sent yet" on the comment and "Nothing is sent until you submit the review" in the composer. Recommendation: use that wording on the bar and badge too ("1 unsent comment · only you can see it"). Phase 6 and README decision 7 use "pending", so the docs would change with it.

### B. A way to send one reply now

Every reply goes into the batch (line 519). A reviewer who "just replies" has told nobody anything until they submit. GitLab offers "Add comment now" beside "Start a review" at each box.

This conflicts with README decision 7 ("Nothing is posted until the reviewer submits"). Recommendation: allow it for replies to existing threads only, and keep new threads batched.

### C. A list of all threads

Every tool surveyed pairs the in-context view with a filterable list of all threads (open, resolved, detached). The mockup has only the in-context view, and no phase builds a list. Recommendation: add it to phase 6 as a panel with a filter, which also gives keyboard and screen reader users one place to reach every thread.

### D. Where the controls live in VS Code

The mockup puts the show modes, navigation and refresh in a header inside the preview. VS Code's guidelines prefer native editor title actions over controls drawn in a webview. Phase 4 specifies a header in the preview. Worth settling before phase 4 is built.

## Left open on purpose

The research found no evidence either way on these. Keep them as they are and test with real reviewers.

- **The connector line.** No surveyed product draws one, and Word removed its own. The mockup already highlights both ends on hover, which is what the others rely on. The line helps most when a thread has been pushed away from its block by the ones above it.
- **Collapsed removed blocks.** No direct precedent. The "Removed" label and **Show** control are the right safeguards.
- **Changes with nothing to see** (a link's target, an image's alt text). The mockup has no example. GitHub uses tooltips and MediaWiki uses notes at the side; notes work for keyboard and touch.

## Already right, keep

- Threads in a margin column at wide widths, markers that open a panel at narrow widths.
- Detached threads listed at the top of the page with what they were on.
- Resolved threads collapsed in place, with reopen.
- "Changes / As it will be / As it was": every mainstream tool has these three views, and these labels are plainer than Word's.
- A move shown as a one-line linked stub at the origin and the full paragraph at the destination.
- A thread inside a closed tab or details says where it is and offers **Show it**.
- The refused composer on unpushed lines says why and what to do.
- The conversation fallback notice, with no Resolve on those comments.
- The states for before review starts, not signed in, anchors missing, and not an Ascribe page.

## Sources

The research report is at [`reports/Review and diff UI practices.md`](../../reports/Review%20and%20diff%20UI%20practices.md), with links for each claim. It rests on vendor documentation, W3C criteria and public issue threads; it found no controlled studies, and it flags the details that came from search snippets only.
