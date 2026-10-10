# Pending / batched review flow and review-state communication (as of October 2026)

Method note for the report writer: pages were read through a fetch tool that summarises content, so strings shown in quotation marks are as returned by that tool and should be treated as close-to-verbatim rather than guaranteed character-exact. Where a claim rests only on a search-result snippet or an issue title (not a fetched page), it is marked "(snippet only)". Anything I could not source is in Gaps, not in Cited Findings.

## GitHub: batched review flow, labels, redesign, complaints, GraphQL

### Takeaway
GitHub's batch is entered with "Start a review" (later comments use "Add review comment"), comments carry a "Pending" badge and are visible only to the reviewer, and the batch is released through "Review changes" / "Submit review" with Comment / Approve / Request changes. The single most documented failure is people never submitting: the "Pending" label is widely misread as "waiting on the other party", and the 2026 Files changed redesign added pending comments to the submission panel but also introduced a bug where pending comments vanish from view.

### Cited Findings
- Documented labels: the first comment uses "Start a review"; once a review exists the button is "Add review comment". — [GitHub Docs, Reviewing proposed changes](https://docs.github.com/en/pull-requests/collaborating-with-pull-requests/reviewing-changes-in-pull-requests/reviewing-proposed-changes-in-a-pull-request)
- Documented visibility statement: "Before you submit your review, your line comments are pending and only visible to you." Pending comments can be edited "anytime before you submit your review." — [GitHub Docs](https://docs.github.com/en/pull-requests/collaborating-with-pull-requests/reviewing-changes-in-pull-requests/reviewing-proposed-changes-in-a-pull-request)
- Submit path: click "Review changes", choose "Comment" ("leave general feedback without explicitly approving the changes or requesting additional changes"), "Approve" ("submit your feedback and approve merging the changes proposed in the pull request") or "Request changes" ("submit feedback that must be addressed before the pull request can be merged"), then "Submit review". — [GitHub Docs](https://docs.github.com/en/pull-requests/collaborating-with-pull-requests/reviewing-changes-in-pull-requests/reviewing-proposed-changes-in-a-pull-request)
- Discard path: the docs' route to abandon a pending review is to "click Submit review, then click Discard review" — i.e. Discard lives inside the submit surface, not as a peer button beside it. — [GitHub Docs](https://docs.github.com/en/pull-requests/collaborating-with-pull-requests/reviewing-changes-in-pull-requests/reviewing-proposed-changes-in-a-pull-request)
- Redesign timeline: the improved "Files changed" page entered public preview on 26 June 2025 and became the default for all users on 22 January 2026; an opt-out to the classic experience was still available at that point. — [Changelog 2025-06-26](https://github.blog/changelog/2025-06-26-improved-pull-request-files-changed-experience-now-in-public-preview/); [Changelog 2026-01-22](https://github.blog/changelog/2026-01-22-improved-pull-request-files-changed-page-on-by-default/)
- Redesign change relevant to batching: "Pending comments are now shown in the review submission panel, so you can double-check your feedback before submitting." The same release notes fixing "multiple bugs where users could not add, edit, delete, reply to, or jump to comments and outdated threads during an active review." — [Changelog 2026-01-22](https://github.blog/changelog/2026-01-22-improved-pull-request-files-changed-page-on-by-default/)
- Redesign also allows commenting on any line of a changed file, not only lines near a change (snippet only). — [Changelog 2025-06-26](https://github.blog/changelog/2025-06-26-improved-pull-request-files-changed-experience-now-in-public-preview/)
- Later 2026 additions: a Comments panel on Files changed that shows general PR comments and lets you add new ones without leaving the page (19 Feb 2026), and side-by-side code and comments (19 Mar 2026) (snippet only). — [Changelog 2026-02-19](https://github.blog/changelog/2026-02-19-access-all-pull-request-comments-without-leaving-the-new-files-changed-page/); [Changelog 2026-03-19](https://github.blog/changelog/2026-03-19-view-code-and-comments-side-by-side-in-pull-request-files-changed-page/)
- User complaint, forgotten reviews: discussion "Make it clear that 'PENDING' comments have to be submitted" (opened 20 Jan 2022, 423 upvotes when read). The author spent "2 months to figure out that the big, yellow 'PENDING' decorator" meant unsent, not awaiting review. Other participants report comments invisible for "a few days", "many days", and "I just realized it today after 2 months of being neglected"; comments were "outdated by the time they are submitted". Suggested relabels: "Draft", "Unsubmitted", "Not yet submitted", plus "only visible to you" wording. No GitHub staff reply was visible in the thread, which was still receiving posts in 2026. — [community discussion #10369](https://github.com/orgs/community/discussions/10369)
- Related open requests: "Make un-submitted reviews/comments more obvious or visible" and "Make it easier to view pending comments on viewed files" (titles only, not fetched). — [discussion #16280](https://github.com/orgs/community/discussions/16280); [discussion #12662](https://github.com/orgs/community/discussions/12662)
- User complaint, lost-looking drafts in the redesign: reported 10 March 2026 — pending comments disappear from Files changed after soft navigation (switching tabs without reload); "When I eventually submit the review, the comments are actually posted correctly." Only an automated acknowledgement; no fix or workaround recorded. — [community discussion #189088](https://github.com/orgs/community/discussions/189088)
- GraphQL: `addPullRequestReviewThread` — "Adds a new thread to a pending Pull Request Review." `addPullRequestReviewThreadReply` — "Adds a reply to an existing Pull Request Review Thread." `addPullRequestReview` adds a review; `deletePullRequestReview` removes one. — [GitHub GraphQL reference, Pulls](https://docs.github.com/en/graphql/reference/pulls)
- GraphQL enums: `PullRequestReviewEvent` = APPROVE ("Submit feedback and approve merging these changes."), COMMENT ("Submit general feedback without explicit approval."), REQUEST_CHANGES ("Submit feedback that must be addressed before merging."), DISMISS. `PullRequestReviewState` includes PENDING — "A review that has not yet been submitted." — [GitHub GraphQL reference, Pulls](https://docs.github.com/en/graphql/reference/pulls)
- GraphQL deprecation: `addPullRequestReviewComment` is deprecated in favour of `addPullRequestReviewThread` / `addPullRequestReviewThreadReply`. — [GitHub GraphQL reference, Pulls](https://docs.github.com/en/graphql/reference/pulls)
- Replies can be pending (secondary sources, snippet only): `AddPullRequestReviewThreadReplyInput` takes `pullRequestReviewThreadId` (required), `body`, and optional `pullRequestReviewId` — "the Node ID of the pending review to which the reply will belong"; third-party tooling documents the sequence `addPullRequestReview` (pending) -> `addPullRequestReviewThreadReply` -> `submitPullRequestReview`. — [agynio/gh-pr-review](https://github.com/agynio/gh-pr-review); [dgaramos/dr-agents #380](https://github.com/dgaramos/dr-agents/issues/380)
- Third-party observation (issue title only): comments in a pending, unsubmitted review "are invisible to the REST list and refuse REST replies". — [seasonedcc/seasoned-skills #451](https://github.com/seasonedcc/seasoned-skills/issues/451)
- Third-party bug report: `addPullRequestReviewThread` returning null in some repositories (title only). — [community discussion #182548](https://github.com/orgs/community/discussions/182548)

### Inferences
- A bar that says "N pending" inherits GitHub's most-complained-about word. The community's own proposed fixes ("Draft", "Not yet submitted", "only visible to you") are a ready-made copy direction for the extension's bar and per-comment badge.
- Because replies made while a review is pending also join the batch, a user who "just replies" to an author can silently fail to communicate; the extension's "Add to review" on replies reproduces this risk and needs the same visibility statement at the reply site, not only on the bar.
- GitHub itself moved toward showing the pending list inside the submit panel in 2026; a submit dialog that shows only a count and a summary box is behind the current GitHub baseline.
- GitHub places Discard inside the submit surface. Putting "Discard" as a sibling of "Submit review…" on a persistent bar makes the destructive action more reachable than upstream does.

### Gaps
- Whether GitHub shows a confirmation when discarding a pending review: not stated in the docs page read; unverified.
- Documented behaviour of resolve/unresolve while a review is pending (whether it is immediate and independent of the batch): not found in a primary source. The GraphQL page fetched did not return `resolveReviewThread`, `unresolveReviewThread` or `submitPullRequestReview` text, so their descriptions and input fields are unverified here.
- The `pullRequestReviewId` field on the reply input was seen only in secondary sources; confirm against the GraphQL input-object reference before relying on it.
- "Add single comment" label and the exact wording "Finish your review": the current docs page returned "Start a review", "Add review comment", "Review changes" and "Submit review"; "Add single comment" and "Finish your review" were not in the returned text and may be historical or simply omitted by the summariser. Unverified.
- Exact labels in the redesigned submission panel (2026) were not captured.
- No source found on whether GitHub warns on navigation away with a pending review, or reminds users later.

## VS Code GitHub Pull Requests extension

### Takeaway
The extension uses a checkout-based "Review Mode" and submits the review from the pull request Description view; it prompts to pull when the local branch is out of date, controlled by `githubPullRequests.pullBranch`. Its issue tracker shows the same hazards the design under review faces: pending-comment loss/misplacement and a poor experience when the checkout is stale.

### Cited Findings
- Flow as documented: select a PR to open its Description; "commit or stash your current changes, then select Checkout"; the extension "checks out the pull request branch and enters Review Mode"; the "Changes in Pull Request" view lists files and commits; "Submit your comments or review from the pull request Description"; "Exit Review Mode" returns to the previous branch. — [VS Code docs, Working with GitHub](https://code.visualstudio.com/docs/sourcecontrol/github)
- Out-of-date branch: since extension 0.50.0, when the local branch is out of date a prompt to pull shows when "Refresh" on the PR overview is clicked; `githubPullRequests.pullBranch` configures whether to be prompted to pull when a change in the PR is detected and gained an `always` option (snippet only, from the changelog). — [extension changelog on Open VSX](https://open-vsx.org/extension/GitHub/vscode-pull-request-github/changes)
- Setting reliability: an issue reports the "Pull Branch" setting not being honoured (title only). — [issue #5307](https://github.com/microsoft/vscode-pull-request-github/issues/5307)
- Local edits shift anchors: 0.104.0 fixed "Review/Comment Suggestions are offset by one line if you make local changes first" (snippet only). — [extension changelog on Open VSX](https://open-vsx.org/extension/GitHub/vscode-pull-request-github/changes)
- Stale checkout: issue titled "PR comment experience is *really* bad (when checked out branch is stale?)" (title only). — [issue #1864](https://github.com/microsoft/vscode-pull-request-github/issues/1864)
- New commits not reflected: "PR not updating with new commits" (title only; snippet says refresh did not fetch new commits, causing conversation about already-fixed code). — [issue #5001](https://github.com/microsoft/vscode-pull-request-github/issues/5001)
- Pending-comment integrity: "Lost, overridden or misplaced pending comments" (opened 12 May 2025, extension 0.110.0, open, assigned, labelled as probable bug): "comment contents are swapped/overridden or moved around in the file"; comments vanish without syncing or move between files; reporter links it to delayed/out-of-order sync. — [issue #6950](https://github.com/microsoft/vscode-pull-request-github/issues/6950)
- Label confusion mirrors GitHub's: "Comments created with 'Add a Comment' say 'pending'" (title only). — [issue #2822](https://github.com/microsoft/vscode-pull-request-github/issues/2822)
- Historical: "Adding comments to a review creates multiple reviews" and "Disable option to Start Review when no comment text" (titles only; old issues). — [issue #1343](https://github.com/microsoft/vscode-pull-request-github/issues/1343); [issue #1079](https://github.com/microsoft/vscode-pull-request-github/issues/1079)
- Refresh still a sore point: "The 'Refresh Pull Request Comments' command still does not work" (title only). — [issue #8565](https://github.com/microsoft/vscode-pull-request-github/issues/8565)

### Inferences
- The incumbent handles "behind" with a pull prompt rather than a passive notice; an extension that only shows a notice should at least offer the pull action in the notice.
- The incumbent has no documented guard for commenting on unpushed/locally modified lines beyond fixing offsets; refusing comments on unpushed lines is stricter than the incumbent and needs a clear reason string, because users coming from GHPR will not expect a refusal.
- Optimistic local display of pending comments with asynchronous sync is where #6950-style corruption comes from; serialising writes and showing per-comment sync state would address it.

### Gaps
- Exact button labels in the comment widget ("Start Review", "Add Review Comment", "Add Comment") and the Description view submit buttons were not confirmed from a primary source; the VS Code docs page read does not name them.
- Full enumerated values and default of `githubPullRequests.pullBranch` not confirmed from package.json.
- No source found describing an "ahead of PR head / unpushed commits" notice in GHPR.
- Only titles were read for most issues; their status and maintainer responses are unverified.

## GitLab

### Takeaway
GitLab offers both paths at every comment box ("Start a review" vs "Add comment now", then "Add to review"), surfaces the batch through a "Your review" control that opens a drawer listing pending comments, and combines submit with Approve / Comment / Request changes. The drawer design is recent (17.11 behind a flag, default in 18.1).

### Cited Findings
- "Start a review" saves the comment unpublished, shown as "Pending", and sets the reviewer's status to "Reviewer started review"; "Add comment now" publishes immediately. Subsequent comments use "Add to review". — [GitLab Docs, Merge request reviews](https://docs.gitlab.com/user/project/merge_requests/reviews/)
- "Your review" (upper right) opens the review drawer showing all pending comments, editable before submission. — [GitLab Docs](https://docs.gitlab.com/user/project/merge_requests/reviews/)
- Submit outcomes: "Approve", "Comment", "Request changes". — [GitLab Docs](https://docs.gitlab.com/user/project/merge_requests/reviews/)
- "Discard review" removes all unpublished comments. — [GitLab Docs](https://docs.gitlab.com/user/project/merge_requests/reviews/)
- Thread state inside a pending comment: a pending comment can carry a "Resolve thread" / "Reopen thread" choice. — [GitLab Docs](https://docs.gitlab.com/user/project/merge_requests/reviews/)
- "Add summary" (AI-generated) is available on Premium/Ultimate. — [GitLab Docs](https://docs.gitlab.com/user/project/merge_requests/reviews/)
- Version history: improved review drawer introduced in 17.11 behind `improved_review_experience`, default in 18.1; an approval-submission shortcut arrived in 18.6. — [GitLab Docs](https://docs.gitlab.com/user/project/merge_requests/reviews/)
- Open pitfall: issue "Submit pending review comments when using 'approve'" (title only) — approving outside the review flow can leave pending comments unsent. — [gitlab issue 562579](https://gitlab.com/gitlab-org/gitlab/-/issues/562579)

### Inferences
- GitLab batches the resolve decision with the comment (resolution publishes with the review), whereas the extension resolves immediately; either is defensible, but the extension should say that resolve is immediate because GitLab and Reviewable users will expect it to be batched.
- GitLab exposes "status: reviewer started review" to others, a partial answer to GitHub discussion #4427's request; the GitHub API offers no equivalent, so the extension cannot signal this.

### Gaps
- Whether "Discard review" has a confirmation, and exactly where the pending count is shown, were not captured.

## Gerrit, Phabricator/Phorge, Reviewable, Azure DevOps, Bitbucket

### Takeaway
Gerrit and Reviewable make drafts the only mode (there is no "comment now"), which removes the mode confusion; both publish through one explicit action ("Reply" / "Publish") that previews what will be sent. Reviewable additionally tells other participants that you have unsent drafts. Bitbucket Cloud adopted the GitHub-style opt-in batch ("Start review" / "Finish review") in 2023.

### Cited Findings
- Gerrit: "Comments are first saved as drafts, and you can revisit the drafts as you read through code review." They are published "by clicking the 'Reply'"; the Reply dialog shows the "inline draft comments that will be published". — [Gerrit Review UI docs](https://gerrit-review.googlesource.com/Documentation/user-review-ui.html)
- Gerrit comment state: "Comments can be unresolved (something should be changed) or resolved (informational)." Quick replies "Done" (addressed in a later patchset) and "Ack" (acknowledged without change) mark a thread resolved. — [Gerrit Review UI docs](https://gerrit-review.googlesource.com/Documentation/user-review-ui.html)
- Gerrit attention set: "For every change Gerrit maintains an 'Attention Set' with users that are currently expected to act on the change." Replying removes the replier and adds others by rule (owner replying on an unresolved thread adds all thread participants; a non-owner adds the owner). The reply dialog has a section, expandable via "MODIFY", to adjust who is added. Users in the set are marked with a chevron. — [Gerrit attention set docs](https://gerrit-review.googlesource.com/Documentation/user-attention-set.html); [Gerrit Review UI docs](https://gerrit-review.googlesource.com/Documentation/user-review-ui.html)
- Reviewable visibility of drafts: "While no one can read your drafts, other participants will be able to see that you have unsent drafts pending and how long ago you last touched the review." — [Reviewable docs, Discussions](https://docs.reviewable.io/discussions.html)
- Reviewable batches state changes too: publishing sends draft comments "along with disposition changes or pending acknowledgements"; disposition changes need publishing to take effect even with no comment; marking as read is buffered until publish. — [Reviewable docs, Discussions](https://docs.reviewable.io/discussions.html)
- Reviewable dispositions: Discussing, Satisfied, Blocking, Working, Informing; the primary acknowledge button's label varies (Resolve, Done, Accept, Conclude, Retract). — [Reviewable docs, Discussions](https://docs.reviewable.io/discussions.html)
- Bitbucket Cloud: "Start review" begins a session; comments and tasks added are marked pending and "visible only to you until you submit them"; add more with "Add comment"; "Finish review" submits and can approve or request changes; participants get a single email notification summarising the feedback (snippet only). — [Atlassian blog, Batched comments](https://www.atlassian.com/blog/bitbucket/batch-comments); [Bitbucket Cloud docs, Review a pull request](https://support.atlassian.com/bitbucket-cloud/docs/review-a-pull-request/)

### Inferences
- Naming of the releasing action varies: GitHub "Submit review", GitLab "Your review" then submit, Bitbucket "Finish review", Gerrit "Reply"/send, Reviewable "Publish". "Submit review…" matches GitHub, which is the right vocabulary for a GitHub-backed extension.
- Every tool that batches previews the batch at submit time (Gerrit reply dialog, GitLab drawer, GitHub since Jan 2026). A list of the pending comments in the submit UI is table stakes.
- One-notification-per-review is a stated benefit of batching (Bitbucket); it is a reason to keep replies in the batch by default, but argues for an explicit "send now" escape hatch like GitLab's "Add comment now".

### Gaps
- Gerrit: no documentation found on discarding drafts or on a draft count indicator.
- Phabricator/Phorge: the Phorge inline-comments page returned HTTP 403; no findings. Unverified.
- Azure DevOps: not researched in the calls available; no cited finding on its (non-batched) comment model or vote dropdown.
- Reviewable: Publish button counts and its approve/request-changes dropdown were not on the page read.
- Bitbucket details come from search snippets, not a fetched page.

## Safeguards and conventions

### Takeaway
Across tools the consistent safeguards are: an explicit "only visible to you" statement, a preview of pending items at submit, and a single releasing action. Evidence of failure concentrates on unclear "Pending" wording and on drafts that silently never get sent.

### Cited Findings
- Explicit visibility wording is used by GitHub ("pending and only visible to you") and Bitbucket ("visible only to you until you submit them"). — [GitHub Docs](https://docs.github.com/en/pull-requests/collaborating-with-pull-requests/reviewing-changes-in-pull-requests/reviewing-proposed-changes-in-a-pull-request); [Atlassian blog](https://www.atlassian.com/blog/bitbucket/batch-comments)
- Users asked GitHub to replace "Pending" with "Draft"/"Unsubmitted"/"Not yet submitted" and to add tooltips. — [discussion #10369](https://github.com/orgs/community/discussions/10369)
- Users asked that others be told a review has been started (title only). — [discussion #4427](https://github.com/orgs/community/discussions/4427); Reviewable already does this — [Reviewable docs](https://docs.reviewable.io/discussions.html)
- Pending comments shown at submit: GitHub (Jan 2026), GitLab drawer, Gerrit reply dialog. — [Changelog 2026-01-22](https://github.blog/changelog/2026-01-22-improved-pull-request-files-changed-page-on-by-default/); [GitLab Docs](https://docs.gitlab.com/user/project/merge_requests/reviews/); [Gerrit Review UI docs](https://gerrit-review.googlesource.com/Documentation/user-review-ui.html)
- What bypasses the batch differs by tool: GitLab offers "Add comment now" beside "Start a review"; Reviewable batches even dispositions and read-marks; GitLab lets a pending comment carry a resolve/reopen choice. — [GitLab Docs](https://docs.gitlab.com/user/project/merge_requests/reviews/); [Reviewable docs](https://docs.reviewable.io/discussions.html)

### Inferences
- There is no cross-tool convention on whether resolve is batched, so the extension must state its rule in the UI.
- A persistent bar with a count addresses "forgotten review" only while the editor is open on that PR; reminders on exit/branch switch/window close would address the days-to-months cases users report.

### Gaps
- No primary source found for any tool warning on navigation with unsent drafts, or for undo after discard.
- No source found on whether reactions bypass the pending batch on GitHub.

## Communicating "local checkout differs from PR head" and "can't comment on this line"

### Takeaway
Documented precedent is thin. VS Code GHPR prompts to pull when the branch is out of date; JetBrains documents a checkout-driven Review mode but nothing found on divergence messaging. GitHub's web redesign widened commentable lines to any line of a changed file.

### Cited Findings
- GHPR prompts to pull an out-of-date local branch on Refresh, governed by `githubPullRequests.pullBranch` (snippet only). — [extension changelog](https://open-vsx.org/extension/GitHub/vscode-pull-request-github/changes)
- GHPR has had line-offset defects when local changes exist (fixed in 0.104.0, snippet only) and complaints about stale checkouts (title only). — [extension changelog](https://open-vsx.org/extension/GitHub/vscode-pull-request-github/changes); [issue #1864](https://github.com/microsoft/vscode-pull-request-github/issues/1864)
- JetBrains IDEs: checking out the PR branch starts "Review mode", where changes and comments are shown in the diff and directly in the editor (snippet only). — [IntelliJ IDEA help](https://www.jetbrains.com/help/idea/work-with-github-pull-requests.html)
- GitHub web (new Files changed) permits comments on any line of a changed file, not just lines around a change (snippet only). — [Changelog 2025-06-26](https://github.blog/changelog/2025-06-26-improved-pull-request-files-changed-experience-now-in-public-preview/)

### Inferences
- Since GitHub now accepts comments on any line of a changed file, refusals should be limited to lines that genuinely do not exist at the PR head (unpushed or locally edited), and the message should name the remedy (push, or pull to match the head).
- Behind and ahead need different remedies (pull vs push); separate notices with a one-click action follow the GHPR precedent.

### Gaps
- No sources gathered for GitHub CLI, Graphite, GitHub Desktop or JetBrains Space on divergence messaging; no source for JetBrains' handling of an out-of-sync branch or its pending-review labels.
- No source found for signed-out messaging patterns in these tools.

## General UX guidance: confirmation vs undo, batching

### Takeaway
NN/g supports a confirmation for discarding a pending review because it destroys the user's work, provided the dialog is specific and its buttons name the outcome; it also favours undo where feasible.

### Cited Findings
- "Use a confirmation dialog before committing to actions with serious consequences — such as destroying users' work or costing large amounts of money." — [NN/g, Confirmation Dialogs Can Prevent User Errors — If Not Overused (Nielsen, 2018)](https://www.nngroup.com/articles/confirmation-dialog/)
- "Do not use confirmation dialogs for routine actions… if you cry wolf too many times, people will stop paying attention to the question." — [NN/g](https://www.nngroup.com/articles/confirmation-dialog/)
- "Be specific and inform users about the consequence of their action. Do not ask 'Are you sure you want to do this?'" — [NN/g](https://www.nngroup.com/articles/confirmation-dialog/)
- "Instead of Yes/No answers, provide response options that summarize what will happen for each possible response," e.g. "Delete file" / "Keep file". — [NN/g](https://www.nngroup.com/articles/confirmation-dialog/)
- For particularly dangerous operations, "require a nonstandard action from the user to confirm"; the article also recommends undo as a recovery mechanism. — [NN/g](https://www.nngroup.com/articles/confirmation-dialog/)
- Related NN/g pieces on error prevention and on placing consequential options next to benign ones (titles only, not fetched). — [NN/g, Preventing User Errors](https://www.nngroup.com/articles/user-mistakes/); [NN/g, Dangerous UX: Consequential Options Close to Benign Options](https://www.nngroup.com/articles/proximity-consequential-options/)

### Inferences
- Discard should confirm with a specific message including the count ("Discard 4 unsent comments?") and outcome-named buttons ("Discard comments" / "Keep review"). Submit with Comment should not confirm (routine); the dialog itself is the confirmation.
- "Discard" adjacent to "Submit review…" on one bar is the pattern the NN/g proximity article (title only) warns about; separation or demotion into the submit dialog (as GitHub does) is the safer layout.
- Undo for discard is hard with GitHub's API because deleting the pending review is server-side; a local snapshot offered as "Restore" would be the nearest equivalent.

### Gaps
- Material Design, Apple HIG and GitHub Primer guidance were not fetched; no cited findings for them.
- No source gathered on staging/batching patterns as a general UX topic.
