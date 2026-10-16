# VS Code extension UI for reviewing changes, collaboration, testing, and reports

Scope: how extensions and VS Code itself present change review, comments, history, test results, and reports, judged for Ascribe (`ascribe diff` page-level rendered diff and HTML report; tested examples as checked command output; reports on links, problems, overdue pages, agent-readiness). Researched 2026-10-16. Where a source is older than 2024 it is marked.

## 1. How do PR/MR extensions, GitLens, Git Graph, the SCM view, timeline, and the multi-diff editor present changes, reviews, and comments? How does the Comments API work and who uses it beyond PRs?

### Takeaway
The shape that has stuck is: a tree view lists what changed (PRs, then files in a PR), a webview "Description"/overview page holds the summary and the submit-review actions, and the real reading happens in VS Code's own diff editor with Comments API threads in the gutter. VS Code now has a built-in multi-file "changes" editor that any extension can open with the `vscode.changes` command and a list of (label, original, modified) triples. That is the cheapest way to show "every page this branch changes" in one scrollable view. Heavy graph and history views (GitLens Commit Graph, Git Graph) are webviews, and GitLens puts several of its graph and history features behind a paid tier for private repos, which users resent.

### Cited Findings
**GitHub Pull Requests and Issues (Microsoft)**
- Adds an activity-bar viewlet that "shows a list of pull requests and issues". The tree is grouped by configurable queries (`githubPullRequests.queries`, defaults "Waiting For My Review", "Assigned To Me", "Created By Me"). Issues have the same kind of setting, `githubIssues.queries`. — [vscode-pull-request-github README](https://raw.githubusercontent.com/microsoft/vscode-pull-request-github/main/README.md)
- Selecting a PR opens its **Description**, which is also where comments and reviews are submitted. Its action bar adds assignees, reviewers, labels, and a milestone. **Checkout** switches to the PR branch and enters **Review Mode**, and **Exit Review Mode** goes back. The **Changes in Pull Request** view lists changed files and commits. Selecting a file shows its diff and lets you comment, and **Open File** opens the working copy with full IntelliSense. — [VS Code docs: Working with GitHub](https://code.visualstudio.com/docs/sourcecontrol/github)
- Issue features: hover cards for `#`/`@` references, completions, "Start Working on Issue" (creates a branch), and Code Actions that create issues from TODO/BUG/FIXME/ISSUE/HACK comments. A Copilot "Code Review" option and AI-generated PR titles and descriptions are now in the PR view. — [VS Code docs: Working with GitHub](https://code.visualstudio.com/docs/sourcecontrol/github)
- The PR extension was the first major user of the Comments API and moved from a provider model to the controller model in 2019 (old). — [PR #1168 "Adopt new commenting api"](https://github.com/microsoft/vscode-pull-request-github/pull/1168)

**GitLab Workflow (official GitLab extension)**
- Adds a sidebar panel for issues, merge requests, and pipelines. It shows the status of the branch's latest CI/CD pipeline and job output. You can "review and create discussions from the diff, and resolve and reopen these discussions", with diff comments shown inline. — [GitLab docs: GitLab Workflow extension](https://docs.gitlab.com/editor_extensions/visual_studio_code/index.html)
- A 2026 third-party article says approval is a command-palette action and that batched reviews are not supported. This is a secondary source and unverified. — [DEV Community, How to Review Pull Requests in VS Code (2026)](https://dev.to/rahulxsingh/how-to-review-pull-requests-in-vs-code-2026-3fga)
- GitLab's team reported that VS Code's API for creating and editing comments was undocumented, so they copied the GitHub PR extension's implementation (old issue). — [gitlab-vscode-extension issue #269](https://gitlab.com/gitlab-org/gitlab-vscode-extension/-/issues/269)

**GitLens (GitKraken)**
- The free "Community" tier includes inline blame (an end-of-line decoration plus the status bar), CodeLens above files and blocks, blame hovers, and the Inspect view (a side-bar view). The Commit Graph, Worktrees, and Visual History (file and repo timelines) are free on public repos, but the graph needs a free GitKraken account, and private repos need Pro. Launchpad (a PR triage list) is Pro. The AI features (Review Changes, Compose Commits, Explain Changes, commit messages) are Pro. Cloud Patches are a preview that needs an account. — [GitLens README](https://raw.githubusercontent.com/gitkraken/vscode-gitlens/main/README.md); [GitKraken help: Commit Graph](https://help.gitkraken.com/gitlens/gl-commit-graph/)
- When the Commit Graph was in preview (2022, old), users complained that it was hard to use without a context menu and that a private repo was wrongly treated as private, which blocked the graph. The maintainer said it was "fully accessible to all users (no account needed) for local and public repos", with only private repos needing a paid account. — [GitLens Discussion #2158](https://github.com/gitkraken/vscode-gitlens/discussions/2158)
- A third-party overview says the private-repo paywall is a recurring complaint, says it led to a community fork of v11.7.0 ("GitLess"), and cites Hacker News criticism of the upsell since v12. These claims are unverified secondhand. — [beri.net GitLens overview](https://www.beri.net/tools/gitlens)

**Git Graph (mhutchie)**
- Shows a graph of local and remote branches, refs, and uncommitted changes, with Git actions in right-click menus. It shows commit details and file changes, compares two commits, tracks code review, and has a find widget, branch filtering, issue linking, and PR creation. The repository has 313 open issues and 21 open PRs, and its GitHub page shows no recent release. — [vscode-git-graph on GitHub](https://github.com/mhutchie/vscode-git-graph)
- GitKraken publishes "GitLens vs Git Graph" comparison posts, a sign that Git Graph is the free alternative people choose. — [GitKraken blog (2026)](https://gitkraken.com/blog/gitlens-vs-vs-code-git-graph-use-cases-from-reddit-2026)

**Built-in SCM view and multi-diff editor**
- v1.85 (Nov 2023) shipped the multi-diff editor as a preview, behind `multiDiffEditor.experimental.enabled`: "The multi diff editor lets you view changes in multiple files in one scrollable view." It covered local, staged, incoming/outgoing, and pull request changes, and was "not yet feature complete". The same release added the SCM Incoming/Outgoing section with an "All Changes" entry (`scm.showIncomingChanges`/`scm.showOutgoingChanges`). Community PRs added Collapse All and Expand All. — [VS Code 1.85 release notes](https://code.visualstudio.com/updates/v1_85)
- The public built-in command is `vscode.changes`: "Opens a list of resources in the changes editor to compare their contents", with arguments `title` ("Human readable title for the changes editor") and `resourceList`. `vscode.diff` opens a single left/right diff with a title and options. — [VS Code Built-in Commands](https://code.visualstudio.com/api/references/commands)
- In practice extensions call `executeCommand("vscode.changes", title, [[label, originalUri, modifiedUri], ...])`. Either side may be undefined for an added or deleted file. Known bugs: phantom repeated file headers after scrolling a diff of about 30 files (reported against 1.129.x), and the right side not honoring readonly. — [vscode issue #326702](https://github.com/microsoft/vscode/issues/326702); [vscode issue #199251](https://github.com/microsoft/vscode/issues/199251)

**Comments API**
- The model is `createCommentController(id, label)` → `CommentThread`s at a range. `commentingRangeProvider` decides where users may comment ("Without it, users cannot leave any comments"). There is an optional `reactionHandler`, `CommentThread.canReply`, `CommentThreadCollapsibleState` (Collapsed/Expanded), `CommentThreadState` (Unresolved/Resolved), and `CommentMode` (Editing/Preview). — [vscode.d.ts (main)](https://raw.githubusercontent.com/microsoft/vscode/main/src/vscode-dts/vscode.d.ts); [VS Code API reference](https://code.visualstudio.com/api/references/vscode-api)
- Non-PR users: "Review Comment Renderer" lets users leave GitHub-style inline review comments on a local codebase and export them as Markdown for a coding agent. It is now deprecated in favor of "Review Relay". "Code Notes - Review Comments" is similar. — [Review Comment Renderer (Marketplace)](https://marketplace.visualstudio.com/items?itemName=statiolake.vscode-comment-renderer); [Code Notes (Marketplace)](https://marketplace.visualstudio.com/items?itemName=jellydn.vscode-code-notes)
- Limitation: every provider's comments land in the one built-in Comments panel. "this is intrinsic to the Comments API and cannot be hidden." — [Review Comment Renderer (Marketplace)](https://marketplace.visualstudio.com/items?itemName=statiolake.vscode-comment-renderer)
- An old API issue: comment threads could not start in edit mode. — [vscode issue #92892](https://github.com/microsoft/vscode/issues/92892)

### Inferences
- For `ascribe diff` in the editor, the lowest-cost native shape is a tree of changed pages (added, changed, removed, with the counts the HTML report has), plus a "View all changes" action that calls `vscode.changes` with source-file triples. A rendered view could reuse the built-in Markdown-preview diff (see section 4) or virtual-document URIs of rendered output. This mirrors SCM's "All Changes" and the PR extension's Changes tree without a webview.
- The Comments API suits "page review notes" (local review comments an agent can read), as Review Comment Renderer shows. But its comments are mixed into the global Comments panel with GitHub PR threads, so a docs-review controller should label its threads clearly.
- GitLens's history suggests that putting core review views behind an account or paywall draws lasting resentment. Ascribe's views should stay free and work offline.

### Gaps
- I did not verify whether the GitHub PR extension opens the multi-diff editor ("View all changes") or shows status checks in its Description page. The official doc page doesn't say.
- I did not fetch first-party detail on the Timeline view (file history and local history entries) or the GitLens Inspect and Visual File History UIs beyond the README tier table.
- Git Graph's last release date and maintenance status are not on its repo page as fetched. My training says it has been unmaintained since about 2021, but I did not verify this here.
- `vscode.changes` is a command, not a typed API. I found no documented stable `multiDiff` API in `vscode.d.ts` (a grep found none).

## 2. How do the Testing API and Test Explorer, Jest, Playwright Test, and Vitest show results? Could checked examples or doc checks map to the Testing API? Who uses it for non-unit-test checks?

### Takeaway
The Testing API gives a lot for little code: a tree in Test Explorer, gutter run and status icons, inline failure overlays, and an expected-versus-actual diff in the peek view when a `TestMessage` carries `expectedOutput`/`actualOutput` (`TestMessage.diff`). It also offers "outdated" marking (`invalidateTestResults`), continuous run, coverage, and publish-only controllers for results produced elsewhere. Ascribe's tested examples (command output compared against a recorded file) map almost exactly to `TestMessage.diff`, and `testing/message/content` menus can add an "Accept new output" (bless) button. I found no well-known extension that uses the Testing API for linters or docs. Linters stay in Problems.

### Cited Findings
**Testing API mechanics**
- The pieces are `vscode.tests.createTestController(id, label)` and `createTestItem(id, label, uri)` with a `range` and nested `children`. `resolveHandler` handles lazy discovery, and `canResolveChildren` marks items that can be resolved on demand. Run profiles of kind Run, Debug, or Coverage are made with `createRunProfile(label, kind, runHandler, isDefault?, tag?, supportsContinuousRun?)`. `TestTag` limits which items a profile can run. `run.appendOutput` takes ANSI text with CRLF line endings. Publish-only controllers call `createTestRun(new TestRunRequest(), name, false)` with no run profile, and `persist=false` means the results are not kept. — [Testing API guide](https://code.visualstudio.com/api/extension-guides/testing); [vscode.d.ts](https://raw.githubusercontent.com/microsoft/vscode/main/src/vscode-dts/vscode.d.ts)
- The run states are `enqueued`, `started`, `skipped`, `passed`, `failed`, and `errored`. `errored` "indicates a test that couldn't be executed at all, from a compilation error for example", as distinct from `failed`. — [vscode.d.ts](https://raw.githubusercontent.com/microsoft/vscode/main/src/vscode-dts/vscode.d.ts)
- `TestMessage` has `message` (string or MarkdownString), `expectedOutput`/`actualOutput` ("If given with … a diff view will be shown"), `location`, `stackTrace`, and `contextValue`. The `contextValue` feeds the `testing/message/context` menu (results tree) and `testing/message/content` ("a prominent button overlaying editor content where the message is displayed"). The API's own example of the latter uses `"when": "testMessage == canApplyRichDiff"`. `static diff(message, expected, actual)` "will present as a diff in the editor." — [vscode.d.ts](https://raw.githubusercontent.com/microsoft/vscode/main/src/vscode-dts/vscode.d.ts)
- `TestController.invalidateTestResults(items?)` "Marks an item's results as being outdated … commonly called when code or configuration changes". The same logic "may be used to drive continuous test runs". `refreshHandler` backs a Refresh button. — [vscode.d.ts](https://raw.githubusercontent.com/microsoft/vscode/main/src/vscode-dts/vscode.d.ts)
- Coverage works through `run.addCoverage(new FileCoverage(uri, statementCoverage))` and `loadDetailedCoverage`, which returns `StatementCoverage`/`DeclarationCoverage`. `FileCoverage.includesTests` gives per-test coverage. — [Testing API guide](https://code.visualstudio.com/api/extension-guides/testing); [vscode.d.ts](https://raw.githubusercontent.com/microsoft/vscode/main/src/vscode-dts/vscode.d.ts)

**User-facing Testing UI**
- Test Explorer (beaker icon) has a tree, run and debug controls on each node, a filter by status or current file, `@` status search, and Refresh Tests. The gutter has a play control that becomes a pass or fail status, and a failing test's message appears as an editor overlay. "Show Output" opens Test Results. A Copilot "Fix Test Failure" sparkle now appears. Coverage appears in a Test Coverage view (percentages and color bars), as gutter colors with execution-count hovers, with "Show Inline Coverage", as Explorer percentages, in diff editors, and in a coverage toolbar. Settings include `testing.countBadge`, `testing.gutterEnabled`, `testing.defaultGutterClickAction`, `testing.coverageBarThresholds`, `testing.displayedCoveragePercent`, `testing.showCoverageInExplorer`, and `testing.coverageToolbarEnabled`. — [VS Code docs: Testing](https://code.visualstudio.com/docs/debugtest/testing)
- `testing.openTesting` was renamed `testing.automaticallyOpenTestResults` in December 2024. — [vscode-jest README](https://raw.githubusercontent.com/jest-community/vscode-jest/master/README.md)
- The old third-party Test Explorer UI (hbenl) is deprecated, superseded by the native UI. — [Test Explorer UI (Marketplace)](https://marketplace.visualstudio.com/items?itemName=hbenl.vscode-test-explorer)

**Jest (vscode-jest, v6.4.x)**
- Fully integrated with native testing. Failures show inline at `expect` calls and in the problem inspector. Status bar items `Jest (folder)` and `Jest-WS` show pass, fail, and unknown totals. Snapshots can be updated from Test Explorer (for the workspace, a folder, a file, or one test) and from the gutter (update or view the snapshot for one block). Since v6.3.0 there is "Run with Coverage", and the extension still has its own coverage overlay (`coverageColors`, Default and Gutter formatters). — [vscode-jest README](https://raw.githubusercontent.com/jest-community/vscode-jest/master/README.md)
- Noise and performance: the README says "we have definitely seen sluggish performance for some projects/users". It says watch or on-save can run "90% of all the tests in the project" after one added test. v6.1.0 replaced `autoRun` with `runMode` (watch, on-save, on-demand, deferred) and a per-session chooser, because "A static runMode config in 'settings.json' is simply not sufficient." — [vscode-jest README](https://raw.githubusercontent.com/jest-community/vscode-jest/master/README.md)

**Playwright Test**
- The gutter play button turns into a green check or red X, with run time next to the test name. The Test Results panel summarizes passed, failed, and skipped tests. Inline errors show expected versus received values and a full call log. A Playwright sidebar holds projects (multi-select), **Show Browser**, **Show Trace Viewer** (opens a trace automatically after a run, with a timeline, DOM snapshots, network, and console), **Pick locator**, and **Record new**. — [Playwright docs: Getting started - VS Code](https://playwright.dev/docs/getting-started-vscode)

**Vitest (official `vitest.explorer`)**
- Results in the editor. Coverage needs VS Code ≥ 1.88 and runs from the Testing view or per test from the gutter context menu. It has a Trace View for Vitest browser-mode traces. — [vitest-dev/vscode](https://github.com/vitest-dev/vscode)
- Before native coverage (2023, old), users combined it with Coverage Gutters and lcov. — [vitest-dev/vscode issue #136](https://github.com/vitest-dev/vscode/issues/136)
- The third-party "Jest / Vitest Runner" offers a one-command snapshot update and native Test Explorer items. — [Jest / Vitest Runner (Marketplace)](https://marketplace.visualstudio.com/items?itemName=firsttris.vscode-jest-runner)

**Non-unit-test use of the Testing API**
- Deno's extension publishes `deno test` results through the Testing API, but only with the "run" capability. That is still a test runner. — [vscode_deno testing_api.md](https://github.com/denoland/manual/blob/v1.22.0/vscode_deno/testing_api.md)
- Searches for linting, docs, contract, Robot Framework, OPA, or dbt extensions that use the Testing API for checks found nothing. — (search; no source found)

### Inferences
- Tested examples fit the Testing API well. Each `ascribe.examples.toml` entry or each checked-output file becomes a TestItem with a `range` at its place on the page. A mismatch becomes `TestMessage.diff("output changed", recorded, observed)`, which opens the native expected/actual diff in the peek view. An "Accept observed output" button goes on `testing/message/content` through `contextValue`, the same pattern as Jest's snapshot update and the API's own `canApplyRichDiff` example. "Couldn't run" maps to `errored`, and an edit to a page or its project calls `invalidateTestResults`.
- Doc checks (links, problems) should stay as diagnostics in Problems. That is what users expect from linters (SonarQube, markdownlint, built-in Markdown link validation), and no precedent turned up for linters as tests. Using the Testing API for them would also bring count badges and noise.
- Jest's experience argues for on-demand by default, with continuous run as an opt-in for checks that run commands, since running everything on save makes users complain about sluggishness.

### Gaps
- I did not confirm from a primary source how the peek view renders `expectedOutput`/`actualOutput` (inline peek diff versus full diff editor) or when `testing.automaticallyOpenPeekView` applies. The user docs page does not cover it.
- I found no extension using the Testing API for docs, link, or contract checks. That absence is a result in itself, but the search was not exhaustive (I did not check Pester, Bicep, Terraform test, or Robot Framework directly).

## 3. How do extensions present reports and dashboards (SonarQube for IDE, Snyk, Code Climate, Coverage Gutters, Import Cost, WakaTime, link checkers)? Webview vs tree vs Problems.

### Takeaway
Mature analyzers use three layers. Squiggles and Problems panel entries carry the actionable item. A dedicated tree view adds grouping, filters, and "focus on new code". A webview appears only for long-form explanation (rule description, issue detail). Whole-project dashboards are mostly left to the server or web UI. "Focus on new code", which shows only what the change introduced, is the key filter, and it is what makes a report fit for review.

### Cited Findings
**SonarQube for IDE (formerly SonarLint)**
- Shows issues in four places: editor squiggles (with a "…" for hints), the **PROBLEMS** panel, the **SONARQUBE** panel (all findings, including hotspots and taint vulnerabilities), and the **SonarQube Rule Description** webview. The webview explains the rule with noncompliant and compliant examples, "sometimes as a diff", and lists coding attributes and severity. Secondary locations and flows go in a **SONARQUBE ISSUE LOCATIONS** view that appears only when needed. The panel has **Filter Findings** (All, Current, or Open files; fix availability; severity) and **Focus on New Code** (the eye icon). — [Sonar docs: Investigating issues](https://docs.sonarsource.com/sonarqube-for-vs-code/using/investigating-issues)
- Standalone, new code means code changed in the last 30 days. In connected mode it inherits the server's new-code definition, but the "Reference branch" definition is unsupported, and users ask for it because server and IDE disagree. Focus defaults to overall code. — [Sonar docs: New code](https://docs.sonarsource.com/sonarqube-for-ide/vs-code/concepts/new-code/); [Sonar Community request](https://community.sonarsource.com/t/focus-on-new-code-support-for-projects-using-reference-branch-in-connected-mode/178377)

**Snyk**
- Has inline issue highlighting "categorized by type and severity" across Open Source, Code, IaC, and Secrets, plus fix suggestions. — [Snyk docs: VS Code extension](https://docs.snyk.io/developer-tools/snyk-ide-plugins-and-extensions/visual-studio-code-extension)
- The extension downloads and runs the Snyk CLI, and its troubleshooting centers on CLI download failures and debug logs (which "may write your code to the IDE log files"). — [Snyk docs: Troubleshooting VS Code](https://docs.snyk.io/developer-tools/snyk-ide-plugins-and-extensions/visual-studio-code-extension/troubleshooting-for-visual-studio-code-extension)

**Coverage Gutters**
- Reads lcov or xml. A "watch" mode tracks coverage changes and applies coverage automatically, and a caching layer makes "speedy rendering even in large files". Gutter coverage conflicts with breakpoints, so users switch to line coverage. — [vscode-coverage-gutters](https://github.com/ryanluker/vscode-coverage-gutters)
- Native Testing API coverage (section 2) now covers what this extension did, through the Test Coverage view and gutter. — [VS Code docs: Testing](https://code.visualstudio.com/docs/debugtest/testing)

**Link checking**
- Built-in Markdown link validation is off by default (`markdown.validate.enabled`). It has toggles for file links, fragment links, cross-file header links, and reference links, plus `markdown.validate.ignoredLinks` ("useful for links to files that exist only after publishing"). It is local only: "there is no checking of external http(s) links." Invalid links are reported as warnings or errors (diagnostics). — [VS Code docs: Markdown](https://code.visualstudio.com/docs/languages/markdown)
- LinkCheckMD (Microsoft, older) puts results in an output column, not the Problems panel, and notes that checking broken links is "more of an art than a science". — [linkcheckermd README](https://github.com/mlafleur/linkcheckermd)
- Zensical Studio (a docs tool) reports broken links in a Problems panel across the project. — [Zensical: links](https://zensical.org/studio/write/links/)

### Inferences
- For Ascribe's link, problem, and overdue reports, the precedent is diagnostics in Problems for anything that has a location (broken link, problem, overdue frontmatter date), plus one tree view grouping by report (Links, Problems, Overdue, Agent-readiness) with filters and a "changed by this branch only" toggle modeled on Sonar's Focus on New Code. The full HTML report fits as an "Open report" action that opens a webview or the browser, not as the main surface.
- An `ignoredLinks`-style setting for "exists only after publish" links is a known need. Ascribe's link check, which resolves against the content model, is strictly better here and could say so.

### Gaps
- I did not research Code Climate, Import Cost, or WakaTime UIs, or user complaints about them (no fetches made). From memory, which is unverified: Import Cost shows size as an inline decoration, and WakaTime shows a status-bar timer that links to a web dashboard.
- I found no primary-source complaints about Snyk's webview or performance. The search returned nothing relevant.
- I did not confirm whether Snyk has a "net new issues" (delta) filter like Sonar's.

## 4. How do visual or rendered-preview diff features work (rendered Markdown diff, image diff, notebook diff)?

### Takeaway
VS Code's built-in Markdown preview can now render diffs: an opt-in that shows the rendered document side by side, scroll-synced, with changed words highlighted, enabled through `workbench.diffEditorAssociations`. Several third-party "rendered Markdown diff" extensions also exist. The richest, Rich Markdown Diff, has SCM, Explorer, and Git Graph entry points and exports a standalone HTML diff. Notebooks show that rendered-output diff is wanted but hard: nbdime does it in a browser, and it is slow on large notebooks.

### Cited Findings
- Built-in: "The Markdown preview can also render diffs, showing the rendered document with changes highlighted". "This behavior is opt-in." Open a single diff with **View: Reopen Editor With… → Markdown Preview**, or set `workbench.diffEditorAssociations` (which "takes precedence over `workbench.editorAssociations` for diff views"). Side-by-side previews stay scroll-synced, and the exact modified words get stronger highlighting. — [VS Code docs: Markdown](https://code.visualstudio.com/docs/languages/markdown)
- Rich Markdown Diff (phine-apps): compares "rendered HTML side-by-side or inline instead of raw source code", side by side by default with an inline toggle (Alt+F7). It diffs tables, nested lists, footnotes, and math and code blocks structurally, and color-codes Mermaid flowchart nodes and edges. Entry points: two `.md` files in Explorer, a **Show Markdown Diff** icon on modified files in the SCM view, the editor title in Git history or Git Graph diffs, Compare with Clipboard, and an "Open With… Markdown Visual Conflict Resolver" for conflicts. **Export Rendered Diff to HTML** writes a standalone file for offline viewing. — [rich-markdown-diff on GitHub](https://github.com/phine-apps/rich-markdown-diff)
- The Marketplace listing shows Rich Markdown Diff with about 808 installs. Other rendered-diff extensions are tiny: Markdown Diff Preview (xpenghans2) has about 16 installs and compares against HEAD; Markdown Diff Visualiser does word-level red and green, side by side; Markdown Format Diff and Markdown Diff (xiunen) diff the working tree against HEAD. Install counts come from search snippets and may be stale. — [Rich Markdown Diff (Marketplace)](https://marketplace.visualstudio.com/items?itemName=phine-apps.rich-markdown-diff); [Markdown Diff Preview](https://marketplace.visualstudio.com/items?itemName=xpenghans2.md-diff-preview); [Markdown Diff Visualiser](https://marketplace.visualstudio.com/items?itemName=Arjuntic.markdown-diff-visualiser); [Markdown Format Diff](https://marketplace.visualstudio.com/items?itemName=machshev1969.markdown-format-diff); [Markdown Diff](https://marketplace.visualstudio.com/items?itemName=xiunen.markdown-diff)
- Notebooks: `nbdiff-web` "shows you a rich rendered diff of notebooks", including images and outputs, in a browser. VS Code users asked for nbdime integration in the editor (the issue is old, opened 2019). — [nbdime](https://github.com/jupyter/nbdime); [vscode issue #71611](https://github.com/microsoft/vscode/issues/71611)
- nbdime diffs "can be very slow for large notebooks". GitHub's rich notebook diff uses nbdime and has a 5-second render limit. Raw diffs of image output (base64) are "impossible to review". — [ReviewNB blog](https://blog.reviewnb.com/large-notebook-diff-not-rendering/)

### Inferences
- Since VS Code can now render a Markdown diff itself, Ascribe's rendered page diff in the editor has two routes. (a) Hand VS Code the source pair and let the built-in Markdown-preview diff render it. This is cheap, but it uses CommonMark rendering, not Ascribe's directives. (b) Serve the rendered page HTML for each side and show it in a webview, or as the HTML report's page diff. Only (b) shows what readers will see. Rich Markdown Diff's success with SCM-row and diff-title entry points, plus "export to HTML", matches `ascribe diff`'s HTML report almost exactly.
- The nbdime experience says to keep the rendered diff per page and on demand (open it when a page is selected), not to render every changed page eagerly.

### Gaps
- I did not establish when the built-in Markdown preview diff shipped (which VS Code release), or whether it supports extensions' markdown-it plugins (which would let Ascribe directives render in it).
- I did not verify VS Code's built-in notebook diff (rendered output diff) or image diff behavior from a primary source. My training says VS Code has a notebook diff editor with output diffs and shows images side by side in diffs, but I have not confirmed that here.
- I did not confirm whether Rich Markdown Diff uses a webview or a custom editor.

## 5. What do users complain about (performance, webview heaviness, paywalls, noise)?

### Takeaway
The recurring complaints are paywalls on views that used to be free (GitLens), automatic runs that are too slow or too noisy (Jest watch and on-save), diff and webview correctness and scale problems (the multi-diff editor past about 30 files, nbdime on large notebooks), new-code definitions that don't match the server (Sonar), and one shared global Comments panel that one extension can't keep to itself.

### Cited Findings
- Paywall: GitLens's Commit Graph, Visual History, and Worktrees need Pro for private repos and a GitKraken account even for public ones. Users saw early friction (2022) and later, by secondhand reports, a fork and Hacker News criticism. — [GitLens README](https://raw.githubusercontent.com/gitkraken/vscode-gitlens/main/README.md); [Discussion #2158](https://github.com/gitkraken/vscode-gitlens/discussions/2158); [beri.net (secondhand)](https://www.beri.net/tools/gitlens)
- Noise and performance from automatic runs: vscode-jest admits sluggishness and over-triggering, and its fix was runMode with an on-demand option and a session chooser. — [vscode-jest README](https://raw.githubusercontent.com/jest-community/vscode-jest/master/README.md)
- Scale and correctness of diff views: the multi-diff editor shows phantom headers after scrolling a diff of about 30 files, and readonly is not honored. — [vscode #326702](https://github.com/microsoft/vscode/issues/326702); [vscode #199251](https://github.com/microsoft/vscode/issues/199251)
- Rendered diffs are slow on large inputs and need render time limits. — [ReviewNB blog](https://blog.reviewnb.com/large-notebook-diff-not-rendering/)
- Mismatched definitions of "new": Sonar's IDE can't use the server's reference-branch definition, so the IDE's "new code" differs from the server's. — [Sonar Community](https://community.sonarsource.com/t/focus-on-new-code-support-for-projects-using-reference-branch-in-connected-mode/178377)
- The shared Comments panel can't be hidden per provider. — [Review Comment Renderer](https://marketplace.visualstudio.com/items?itemName=statiolake.vscode-comment-renderer)
- Undocumented APIs push extensions to copy each other (GitLab copied GitHub PR's comments code). — [gitlab-vscode-extension #269](https://gitlab.com/gitlab-org/gitlab-vscode-extension/-/issues/269)
- Gutter conflicts: coverage gutters hide breakpoints. — [vscode-coverage-gutters](https://github.com/ryanluker/vscode-coverage-gutters)

### Inferences
- For Ascribe: keep every review and report view free and local, and run checks on demand by default with continuous run as an opt-in. Compute the base for "changed by this branch" the same way the CLI and CI do, so the editor and `ascribe diff` never disagree, which is the trap Sonar fell into. Keep any rendered diff lazy and per page. Avoid gutter decorations that compete with breakpoints and test icons.

### Gaps
- I found no primary-source complaints about webview heaviness in particular extensions (SonarQube, Snyk, GitLens graph) in this pass. Searches returned nothing usable.
- I did not survey complaints about WakaTime, Import Cost, or Code Climate.
