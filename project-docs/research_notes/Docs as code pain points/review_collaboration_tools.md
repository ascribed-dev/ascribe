# Review, collaboration and editing-experience pain points in docs-as-code, and the tooling market's response

Research date: 3 October 2026. All pages were fetched on that date unless noted.

Reading notes for the report writer:

- **[VENDOR]** marks a claim that comes from a company selling a product in this space. Use these for features and prices only, not as evidence of how painful the alternative is.
- **[SNIPPET]** marks text that came from a search-result summary and was not confirmed on the page itself. Treat the wording as approximate and do not present it as a verbatim quote.
- Pages were read through a summarising fetch tool. Quotes in quotation marks were returned as verbatim by that tool; where the tool truncated a quote I say so.
- Coverage is uneven. Reddit r/technicalwriting, G2 and Capterra returned nothing usable, and several named tools were not researched at all. See the Gaps subsections.

## 1. What specifically is wrong with reviewing documentation in GitHub/GitLab pull requests

### Takeaway
The best-evidenced defect is structural: GitHub's rendered ("rich diff") view and its commenting system are two separate things, so a reviewer can either read the rendered page or comment, not both. GitHub has not responded to user requests to fix this in 2025-2026, and at least four independent browser extensions were built in 2026 to patch it.

### Cited Findings

Rendered view and comments are disconnected on GitHub:

- Ahmed Sabbour, 23 March 2026: "The source diff is cluttered with formatting syntax, and the rich diff view strips away line numbers, comment indicators, and any connection back to specific source lines." — [sabbour.me](https://sabbour.me/2026/03/23/markdown-rich-review-for-github-prs.html)
- Same post: "You see the final output, but when you spot something to comment on, you have to manually find the same line in the source diff." He calls this a "hunt-and-scroll process" that "slows down reviews, especially for longer documents." — [sabbour.me](https://sabbour.me/2026/03/23/markdown-rich-review-for-github-prs.html)
- GitHub Community discussion #160981, opened by mrohan-sq on 29 May 2025: inline comments on Markdown files appear in the source diff but not in the rich diff; file-level comments do appear. The poster asked for a sidebar with highlighted references, similar to Google Docs. A replier (chess-king-dot, 29 May 2025) explained that inline comments are tied to source line numbers, which do not map consistently to the rich diff. No GitHub staff reply, only an automated acknowledgement. — [GitHub Community #160981](https://github.com/orgs/community/discussions/160981)
- GitHub Community discussion #186730, opened by ckumick-va on 9 February 2026, 47 upvotes at time of fetch: "Reviewers would find it easier to review Markdown if they could see and comment on the diff of the formatted Markdown inside the Pull Request without leaving to View File mode." No GitHub staff reply. — [GitHub Community #186730](https://github.com/orgs/community/discussions/186730)
- In that thread, FrankLedo (8 April 2026) described the workflow that breaks: "engineers write design docs as .md files, open a draft PR, and share the PR link for review". Two different people (chienyuanchang on 29 May 2026, jdolitsky on 2 September 2026) each posted a browser extension they had built as a workaround. — [GitHub Community #186730](https://github.com/orgs/community/discussions/186730)

Reviewers misread prose diffs:

- Sarah Moir, "Docs as code is a broken promise", 10 April 2024: "Documentation reviews in a pull request can be confusing." — [thisisimportant.net](https://thisisimportant.net/posts/docs-as-code-broken-promise/)
- Moved content read as deletion, same post: "An engineer might see that you deleted lines from a file, not realizing that you deleted them because they were in the wrong place, not because the information was wrong." — [thisisimportant.net](https://thisisimportant.net/posts/docs-as-code-broken-promise/)
- No rendered view, same post: "if you can't provide your reviewers with a staged or preview version of the content, you might get a technical review full of confused comments instead of helpful feedback." — [thisisimportant.net](https://thisisimportant.net/posts/docs-as-code-broken-promise/)
- Same post: "Parsing a documentation pull request and providing helpful comments requires the reviewer experience the content the same way that a future customer will." — [thisisimportant.net](https://thisisimportant.net/posts/docs-as-code-broken-promise/)

Line-based diffs on wrapped paragraphs:

- Hacker News thread "Writing one sentence per line" (item 31808093; the thread date was not returned by the fetch, the item ID places it in mid-2022). User luhn: "One sentence per line means diffs will operate per-sentence, rather than per-paragraph." User michaelmior: "diffs make _way_ more sense when there is only one sentence per line." — [Hacker News](https://news.ycombinator.com/item?id=31808093)
- [SNIPPET] A 2026 open-source PR describes the underlying problem: with hard-wrapped Markdown, editing one sentence reflows the surrounding lines, so the diff shows a whole paragraph instead of the sentence that changed. — [the-hcma/home-warden PR #166](https://github.com/the-hcma/home-warden/pull/166)

Git itself as a barrier during review:

- Tom Johnson, responding to Moir (date not returned by the fetch): "capturing a snapshot between release branches exactly right so that reviewers could see a diff in the Review Board tool...definitely wasn't easy". — [idratherbewriting.com](https://idratherbewriting.com/blog/thoughts-on-docs-as-code-promise)
- Same post: "Git is far too complex to be all that practical for a team of writers of varying technical levels", and he reports "one new writer admitting that she was afraid to make a change for fear of making a mistake with Git". He still prefers docs-as-code, valuing diffs and Markdown over proprietary alternatives. — [idratherbewriting.com](https://idratherbewriting.com/blog/thoughts-on-docs-as-code-promise)

### Inferences
- The rich-diff/comment split looks like the most concrete, verifiable and currently unfixed defect. Evidence: two unanswered GitHub feature requests a year apart, and multiple independently built extensions in 2026.
- Browser extensions are a weak fix for non-engineers. Each reviewer has to install one, and Sabbour's extension navigates the reviewer back to the source diff to comment, so the comment is still written against raw markup.
- The complaints cluster around one root cause: the review unit is a source line, while the thing a prose reviewer thinks about is a sentence, paragraph or rendered page.

### Gaps
- No first-hand quote found on GitHub "suggestion" blocks breaking markup (for example a suggestion spanning a fenced code block or table). Not confirmed either way.
- No first-hand quote found on the lack of a tracked-changes or comment-resolution model, beyond the Google Docs comparison in discussion #160981.
- No data found on review latency for docs PRs. The only figure is a vendor recommendation of a 48-hour review SLA (DraftView, see section 5), which is not a measurement.
- Nothing GitLab-specific was found. All evidence above is about GitHub.
- Reddit r/technicalwriting returned no usable results; the one search returned unrelated github/docs PRs.

## 2. How teams work around this today, and what each workaround costs

### Takeaway
Three workarounds are documented: preview deployments, copying into Google Docs, and sentence-per-line source formatting. Each moves the cost rather than removing it: previews need engineering setup, Google Docs creates a second copy, and sentence-per-line asks writers to change how they write.

### Cited Findings

Preview deployments:

- Moir treats a staged or preview build as the precondition for a useful technical review (quote in section 1), and names the cost: "to take advantage of these capabilities, you often need to build the tools and checks yourself, or get your documentation platform team to build them for you." — [thisisimportant.net](https://thisisimportant.net/posts/docs-as-code-broken-promise/), 10 April 2024
- Same post: "documentation teams are often lucky to get one engineer to work on the documentation site itself, let alone help enable any special docs as code workflows." — [thisisimportant.net](https://thisisimportant.net/posts/docs-as-code-broken-promise/)
- The State of Docs 2026 report repeats this point: docs-as-code often ends up as tool adoption without process investment, and docs teams are lucky to get one engineer. [SNIPPET], and the report is produced by GitBook, a vendor. — [State of Docs 2026](https://www.stateofdocs.com/2026/docs-tooling), page dated 28 August 2026

Review in Google Docs, then paste back:

- [VENDOR] DraftView, 25 March 2026: "Copying docs into Google Docs for review...creates version drift and forces manual reconciliation afterward". — [DraftView blog](https://www.draftview.app/blog/docs-as-code-review-workflow-complete-guide)
- [VENDOR] Same article lists what non-technical reviewers do when sent a Markdown diff: ignore the PR, leave vague comments such as "looks fine", ask for a Google Docs copy, or send feedback by email or Slack outside the PR. — [DraftView blog](https://www.draftview.app/blog/docs-as-code-review-workflow-complete-guide)

Semantic line breaks (one sentence per line):

- Benefit: sentence-level diffs (luhn and michaelmior quotes in section 1). — [Hacker News](https://news.ycombinator.com/item?id=31808093)
- Cost, user ketzu: "I hate to change for my tools, I rather have my tools support my workflow." — [Hacker News](https://news.ycombinator.com/item?id=31808093)
- Alternatives raised in the same thread: fragmede, "If you're using Git, you can do _git diff --word-diff_ to enable word mode, which is computationally more expensive."; spiffytech, "I recently switched my git diff viewer to difftastic, which does semantic diffing."; _tom_, "You could write normally and then split on sentence boundaries with any NLP tool." — [Hacker News](https://news.ycombinator.com/item?id=31808093)
- Adoption signal in 2026: a dedicated reflow tool exists (jbeda/mdreflow, sentence-per-line by default), and several open-source repos merged PRs enforcing the convention in CI. [SNIPPET] — [mdreflow](https://github.com/jbeda/mdreflow)

Rich diff plus browser extensions:

- Third-party extensions that add comments to GitHub's rendered Markdown view. — [chienyuanchang/rich-diff-comments](https://github.com/chienyuanchang/rich-diff-comments), [bkonold/rich-diff-comments](https://github.com/bkonold/rich-diff-comments), [sabbour.me](https://sabbour.me/2026/03/23/markdown-rich-review-for-github-prs.html)

### Inferences
- Word-diff and difftastic are local command-line fixes. They do not change what a reviewer sees in the GitHub web interface, which is where non-engineers would review.
- Sentence-per-line helps the diff but not the reviewer's reading experience; the source still is not the rendered page.
- The Google Docs detour is the only workaround that gives non-engineers a model they already know, and its cost (two copies, manual reconciliation) is asserted only by a vendor in what I found.

### Gaps
- No independent, first-hand account found of the Google Docs round trip and its costs. This is widely believed but I could only source it to a vendor.
- No evidence found on screenshots-in-PRs as a workaround.
- No cost data (build minutes, setup hours) for preview deployments on docs sites.

## 3. What non-engineer reviewers experience, and what fraction of contributors are non-writers

### Takeaway
I found no statistic for the share of docs contributors who are non-writers. The qualitative picture is consistent across practitioner sources: ownership is spread across PMs, designers, engineers and legal, and the working advice in 2026 is to offer several contribution paths rather than force everyone through a PR.

### Cited Findings
- State of Docs 2026 (produced by GitBook; page dated 28 August 2026; 1,131 respondents per a third-party summary [SNIPPET]): "Documentation ownership actually gets distributed across PMs, designers, engineers, and legal — and why that matters more than most teams admit." — [State of Docs 2026, contributors](https://www.stateofdocs.com/2026/contributors)
- Manny Silva (Skyflow, Doc Detective), in the same report: "I like to offer multiple contribution paths — WYSIWYG editors, docs-as-code PRs, AI writing agents in Slack..." (truncated by the fetch tool). — [State of Docs 2026, docs tooling](https://www.stateofdocs.com/2026/docs-tooling)
- Tooling mix in State of Docs 2026: 45% use dedicated documentation tools, 21% use open-source platforms or Git repos, 9% use website publishing platforms. — [State of Docs 2026, docs tooling](https://www.stateofdocs.com/2026/docs-tooling)
- Moir on the shared-environment gap: "Unlike a typical content management system (CMS) like WordPress or Drupal, or software like MadCap Flare, the writing environment isn't shared — only the content is the same." — [thisisimportant.net](https://thisisimportant.net/posts/docs-as-code-broken-promise/), 10 April 2024
- Moir on Git as a barrier: "To do docs as code, writers need to learn how to use and troubleshoot Git. And Git isn't simple." and "it's easy to get into an unexpected state ... often the best troubleshooting is starting over — and that's not a great experience!" (two quotes joined; the ellipsis is mine). — [thisisimportant.net](https://thisisimportant.net/posts/docs-as-code-broken-promise/)
- Moir on alternatives: "There are newer CMSes on the market, like ReadMe, Heretto, Paligo, and others... it seems like the writing experience is much simpler, and there is a way to write Markdown without needing to use developer level tools." — [thisisimportant.net](https://thisisimportant.net/posts/docs-as-code-broken-promise/)
- [VENDOR] DraftView: "Requiring all reviewers to use GitHub...guarantees delays". — [DraftView blog](https://www.draftview.app/blog/docs-as-code-review-workflow-complete-guide), 25 March 2026

### Inferences
- The 21% "open-source platforms/Git repos" figure understates Git-backed docs, because many of the "dedicated documentation tools" in the 45% group (GitBook, Mintlify, Fern, ReadMe) sync to Git. The survey categories do not separate these.
- Git fear is reported for writers, not just SMEs. If writers hesitate, occasional contributors are likely worse off, but I found no direct evidence for PMs, legal, support or marketing specifically.

### Gaps
- **No figure found for the fraction of docs contributors who are non-writers.** The State of Docs 2026 contributors page returned no percentages through the fetch tool; the numbers may be in charts the tool could not read. The State of Docs 2025 URLs I tried returned 404. Someone should open these pages in a browser.
- No first-hand accounts from PMs, legal reviewers, support or marketing describing their own experience. Everything above is writers describing other people.
- Survey size of 1,131 comes from a third-party summary (happysupport.ai), not from the report page itself.

## 4. Tools offering visual or WYSIWYG editing on top of Git-backed Markdown/MDX

### Takeaway
Every hosted docs platform now ships a browser editor that commits to Git, and they share the same limit: the visual editor understands the vendor's own component set, not arbitrary MDX. The two-way sync is where the documented problems sit (formatting rewritten on save, PRs and in-app review being separate systems, pushes blocked by branch protection).

### Cited Findings

**GitBook (block editor plus Git Sync)**

- Pricing [VENDOR], pricing page: Free is $0 per site with 1 user; Premium is "$65 per site/month" plus "$12 per user/month" (annual billing); Ultimate is "$249 per site/month" plus "$12 per user/month"; Enterprise is custom. Git Sync and change requests are on all plans; "Review & approve edits" is not on Free. — [GitBook pricing](https://www.gitbook.com/pricing)
- Review happens in two unconnected systems [VENDOR docs]: "Creating a pull request in GitHub or GitLab doesn't create a change request in GitBook, and creating a change request in GitBook doesn't create a pull request in your repository." — [GitBook Git Sync troubleshooting](https://gitbook.com/docs/docs-as-code/git-sync/troubleshooting.md)
- Other documented limits [VENDOR docs]: 100MB per file; READMEs created in the GitBook UI can create "duplicate README files in your repository"; pages must be listed in `SUMMARY.md` to appear. — [GitBook Git Sync troubleshooting](https://gitbook.com/docs/docs-as-code/git-sync/troubleshooting.md)
- Formatting churn [SNIPPET, from GitBook's own docs]: GitBook normalises Markdown on sync, for example changing `-` list markers to `*`, and will change non-GitBook-flavoured markup back at the next sync. GitBook publishes a repository documenting how each block is translated to Markdown. — [GitBook content configuration](https://gitbook.com/docs/getting-started/git-sync/content-configuration), [GitbookIO/git-sync-normalization](https://github.com/GitbookIO/git-sync-normalization)
- User complaint, Mila Kowalski, 23 March 2026: "No direct Markdown export... the 'Markdown' is full of proprietary block formats". She also reports the free tier dropping from 3 users to 1 and paying "3 × $65/month = $195/month" in per-site fees. **Caveat: the post is a migration story that ends by recommending Theneo, a competitor. Treat it as possibly promotional.** Most of its complaints concern OpenAPI handling, not prose review. — [dev.to](https://dev.to/mjkloski/we-used-gitbook-for-two-years-heres-the-honest-post-mortem-of-why-we-left-52fm)

**Mintlify (web editor)**

- Pricing [VENDOR], pricing page: Starter $0 with 5 editor seats, web editor included; Pro $450/month with unlimited editor seats, adding the agent, assistant, automations and preview deployments, with 10,000 AI credits a month (25 credits per assistant answer, 250 per automation update); Enterprise custom. — [Mintlify pricing](https://www.mintlify.com/pricing)
- Preview deployments are a Pro feature, so they are not on the free tier. [VENDOR] — [Mintlify pricing](https://www.mintlify.com/pricing)
- Git round trip [VENDOR docs]: "When you publish, the editor commits your changes to your repository." On feature branches, "your edits commit to the branch automatically and open a pull request for review." Incoming pushes: "The editor merges non-conflicting changes and flags anything that needs your attention." — [Mintlify editor docs](https://www.mintlify.com/docs/editor)
- Collaboration [VENDOR docs]: "Multiple people can edit the same page at once, with live cursors showing who is working where. Comments and suggestions are visible to everyone." — [Mintlify editor docs](https://www.mintlify.com/docs/editor)
- Documented limit [VENDOR docs]: "Source mode is unavailable while suggesting mode is on". — [Mintlify editor docs](https://www.mintlify.com/docs/editor)
- Pricing changed in 2026 from seat-based tiers to the structure above. [SNIPPET, from competitor Fern's comparison page] — [buildwithfern.com](https://buildwithfern.com/post/mintlify-reviews-pricing-alternatives)

**Fern (Fern Editor)**

- What it does [VENDOR docs]: "a visual WYSIWYG editor that lets team members update documentation without code, markdown, or Git access". "every edit creates a pull request (a merge request on GitLab), preserving Git history, code review, CI checks, and branch protections." Contributors do not need GitHub accounts; an admin connects the repo. — [Fern Editor docs](https://buildwithfern.com/learn/docs/content/visual-editor)
- Limits [VENDOR docs]: asides, frames, icons and sticky table headers are not yet supported ("coming soon"). "Fern Editor supports modern Chromium browsers on desktop. Mobile editing and support for other browsers are coming soon." The fetch summary also states the editor cannot edit custom React or custom MDX components, only Fern's built-in library; that sentence was the tool's paraphrase, not a quote, so confirm before relying on it. — [Fern Editor docs](https://buildwithfern.com/learn/docs/content/visual-editor)
- Pricing [VENDOR, SNIPPET]: editor included in all plans, no per-seat pricing; free for 10 people on a custom domain. Not confirmed on a pricing page. — [Fern Editor announcement](https://buildwithfern.com/post/fern-editor)

**ReadMe (Refactored, bi-directional sync)**

- What it does [VENDOR docs]: "Changes sync automatically between ReadMe and Git creating a single source of truth." Supports GitHub Cloud, GitHub Enterprise Server (Enterprise plan only), GitLab and Bitbucket. Requires at least the Starter plan. — [ReadMe bi-directional sync docs](https://docs.readme.com/main/docs/bi-directional-sync)
- Limits [VENDOR docs]: "The repository you're syncing to must be empty—no commits or files (e.g., README.md)—before connecting to ReadMe." Files need `title` and `summary` frontmatter; file names must match URL slugs; branch names must match version names. ReadMe does not bypass branch protection, so protected branches can make sync fail unless ReadMe is added to the bypass list or direct pushes are allowed. — [ReadMe bi-directional sync docs](https://docs.readme.com/main/docs/bi-directional-sync)
- Pricing [VENDOR]: "Starter is free"; "Pro is $250 a month billed annually"; "$250 flat to 5 admins, $20 each after". — [ReadMe blog](https://readme.com/blog/is-readme-worth-it)
- [SNIPPET, not confirmed on the page] Upgrading to Refactored is described as irreversible, and the Suggested Edits feature is reported as not supported after upgrading. If true this is notable, since it removes a review feature in the move to Git sync. Verify at the upgrade guide. — [ReadMe upgrade guide](https://docs.readme.com/main/docs/upgrade-to-readme-refactored)

**TinaCMS (open-source, Git-backed, visual editing)**

- Pricing [VENDOR], pricing page: Free $0 with 2 users; Team $24/month per project with 3 users included, up to 10; Team Plus $41/month per project with 5 included, up to 20; Business "$249/ project / month" with 20 included; Enterprise custom. "Editorial Workflow" starts at Team Plus. — [Tina pricing](https://tina.io/pricing)
- Formatting churn, historical: issue #1098 (AGMETEOR, 5 May 2020), "Tina CMS automatically makes changes to the MDX files and changes information", specifically date formats and added quotes in frontmatter, on open and without edits. Closed as wontfix. This is six years old and on version 0.19.0. — [tinacms/tinacms #1098](https://github.com/tinacms/tinacms/issues/1098)
- Formatting churn, recent [SNIPPET]: a TinaCMS PR describes the risk that if its printer lays an object out differently, the next save rewrites props across every affected file and the customer's Git history absorbs the churn; Tina built a custom printer to match Prettier's output. Date not confirmed. — [tinacms PR mirror](https://github.com/amishakov/tinacms/pull/358)
- [SNIPPET, third-party reviews] Visual editing needs frontend instrumentation (the `useTina` hook and React components); the editor can feel slow on large pages with many components. — [Lucky Media review](https://www.luckymedia.dev/insights/tina-cms)

**Decap CMS and Keystatic (open-source, Git-backed)**

- [SNIPPET, third-party agency review] Decap's branch-based editorial workflow is described as beta and limited for non-technical draft-and-review cycles; MDX support is limited; native GitHub mode requires collaborators to have repo write access and a GitHub sign-in. — [Lucky Media, Decap review](https://www.luckymedia.dev/insights/decap-cms)
- [SNIPPET, third-party agency review] Keystatic's admin UI is described as usable by a non-technical editor within an hour once a developer has configured it. — [Lucky Media, Keystatic review](https://www.luckymedia.dev/insights/keystatic)

**Redocly (Reunite)**

- [SNIPPET, vendor and third-party] Reunite provides hosting, an editor and "visual reviews". Pro is $10 per seat per month (1 project, 100 pages); Enterprise is $24 per seat per month. Prices come from review sites, not Redocly's pricing page. — [Redocly blog](https://redocly.com/blog/premium-vs-open-source), [documentation.ai review](https://documentation.ai/blog/redocly-review)

**Doctave**

- Shut down. Announced 31 August 2026 by Niklas Begley; hosted sites, dashboard and builds stopped on 14 September 2026. No business reason given. Customers were pointed to Docapella, an open-source static site generator from the same team (rename `doctave.yaml` to `docapella.yaml`). "Your content is yours, and it's already in your repository". — [Doctave blog](https://www.doctave.com/blog/doctave-is-shutting-down)

### Inferences
- The common ceiling is custom components. Fern's editor is limited to its own component set; Tina needs each component instrumented; GitBook's blocks are proprietary. A team with its own MDX components gets a visual editor only for the parts the vendor already knows.
- Two-way sync produces three distinct documented costs: normalisation churn (GitBook, Tina), a review split across two systems (GitBook change requests vs PRs), and conflict with branch protection (ReadMe pushes directly to the synced branch). Fern avoids the last two by making every edit a PR, at the cost of sending the reviewer back to a PR.
- These editors are tied to the vendor's hosting. None of the hosted ones is offered as an editor for a Docusaurus, Astro Starlight or MkDocs site. Tina, Decap and Keystatic are the framework-independent options, and they are general CMSs configured by a developer, not docs review tools.
- Doctave's closure, weeks before this research, shows a hosted docs-as-code platform failing while its content stayed portable. That is a point for Git-native storage and a warning about the size of the market for a standalone platform.
- Pricing has moved from per-seat to per-site or flat fees with metered AI (Mintlify, GitBook). Occasional reviewers no longer cost a seat on Mintlify Pro or Fern, which removes one historical objection to inviting SMEs.

### Gaps
- **Not researched at all:** Archbee, Document360, ClickHelp, Front Matter CMS, GitDoc, Markdoc tooling, Sanity/Contentful-based docs, Docusaurus-adjacent editors, VS Code Markdown WYSIWYG and comment extensions. I ran out of budget; no claims should be made about them from these notes.
- No G2, Capterra or Reddit complaints were retrieved for any tool. User-complaint evidence here is thin: one possibly promotional dev.to post, one six-year-old GitHub issue, and third-party agency reviews.
- No evidence found, positive or negative, on how the Mintlify or Fern editors handle formatting churn when saving a file they did not create.
- Keystatic and Decap pricing not confirmed (both are open source; hosted options were not checked).
- Fern and Redocly prices are not confirmed against their own pricing pages.

## 5. Tools that address review specifically

### Takeaway
Preview-comment tools from hosting providers are mature and free for reviewers, but they comment on a deployed page, not on a change: they do not show what changed. Only one product found, DraftView, targets rendered prose review of a pull request directly, and I could not confirm its pricing or mechanics beyond its own marketing.

### Cited Findings

**Vercel Comments** [VENDOR docs, last updated 19 August 2026]

- "Comments are available on all plans" and are "enabled by default" on all preview deployments, "free of charge. The only requirement is that all users must have a Vercel account." — [Vercel Comments docs](https://vercel.com/docs/comments)
- Reviewers "click on the page or highlight text to place your comment." Threads can be linked to Slack. — [Vercel Comments docs](https://vercel.com/docs/comments)
- "Anyone in your Vercel team can leave comments on your previews by default. On Pro and Enterprise plans, you can invite external users to view your deployment and leave comments." — [Vercel Comments docs](https://vercel.com/docs/comments)
- When a new deployment is built, a modal prompts the viewer to refresh. The page does not say what happens to a comment whose anchored text has changed. — [Vercel Comments docs](https://vercel.com/docs/comments)
- Vercel used the feature on its own documentation ("Using Vercel comments to improve the Next.js 13 documentation"). Title only; I did not read the post. — [Vercel blog](https://vercel.com/blog/using-vercel-comments-to-improve-the-next-js-13-documentation)

**Netlify Drawer** [VENDOR docs]

- "site visitors can take screenshots and add visual or text-based annotations, create screen recordings, and share comments." — [Netlify Drawer docs](https://docs.netlify.com/deploy/review-deploys/netlify-drawer-for-feedback/overview/)
- Two-way PR sync: "Any comment added using the Netlify Drawer on a Deploy Preview is automatically posted in the corresponding pull/merge request at your Git provider and vice versa, so everyone can work in context." — [Netlify Drawer docs](https://docs.netlify.com/deploy/review-deploys/netlify-drawer-for-feedback/overview/)
- Free reviewers: "An unlimited number of cross-functional stakeholders can review Deploy Previews and branch deploys for free when they log in to the Netlify Drawer in the Reviewer role." — [Netlify Drawer docs](https://docs.netlify.com/deploy/review-deploys/netlify-drawer-for-feedback/overview/)
- Issues can be opened in GitHub, GitLab, Bitbucket, Jira, Trello, Azure DevOps, Linear and Shortcut. Limit: "Branch deploys do not support the conversation pane since there is no synced pull/merge request." — [Netlify Drawer docs](https://docs.netlify.com/deploy/review-deploys/netlify-drawer-for-feedback/overview/)

**DraftView** [VENDOR, article dated 25 March 2026]

- Renders a GitHub pull request as a visual review page with a "Google Docs-style interface"; reviewer suggestions sync back to GitHub as native Suggested Changes the writer can accept or reject. Supports Markdown, MDX and AsciiDoc. A GitHub account is required for reviewers. — [DraftView blog](https://www.draftview.app/blog/docs-as-code-review-workflow-complete-guide)
- The vendor's own guidance concedes limits: keep PRs small because large multi-file changes are not reviewed well, and set a 48-hour review SLA. — [DraftView blog](https://www.draftview.app/blog/docs-as-code-review-workflow-complete-guide)

**GitHub rich diff and extensions**

- Covered in section 1. Extensions exist for Chrome and Edge that add comment, reply and resolve to the rendered view. — [chienyuanchang/rich-diff-comments](https://github.com/chienyuanchang/rich-diff-comments), [bkonold/rich-diff-comments](https://github.com/bkonold/rich-diff-comments) (the latter also covers Azure DevOps)

**In-platform review**

- GitBook change requests (review and approval from Premium up) and Mintlify suggesting mode, both described in section 4. Both live inside the vendor's editor, and GitBook's does not connect to the PR. — [GitBook pricing](https://www.gitbook.com/pricing), [Mintlify editor docs](https://www.mintlify.com/docs/editor)

### Inferences
- Vercel and Netlify comments answer "is this page right?" but not "what changed?". A reviewer on a preview sees the whole new page with no change marks, so they must already know what to look at. This is the inverse of the PR problem, where the reviewer sees only the change with no rendering.
- Vercel comments require a Vercel account and Netlify's reviewer role requires a Netlify login. DraftView requires a GitHub account. Every option found asks the non-engineer to hold an account on a developer platform.
- Netlify syncs preview comments into the PR; Vercel's docs page does not claim this. If a docs team hosts elsewhere (GitHub Pages, Cloudflare Pages, self-hosted), neither tool is available.
- A suggestion that becomes a GitHub Suggested Change (DraftView's model) still inherits GitHub's line-based mechanics. Whether that holds up on tables, MDX components and reflowed paragraphs is unverified.

### Gaps
- **DraftView pricing not confirmed.** The pricing URL returned an unrelated assistant-style page with no prices. No independent user reviews found.
- Pullpo and Reviewable were not researched. GitDoc appeared in search results as a "document review platform" but was not read.
- No independent user reports found on Vercel or Netlify comments for docs review specifically.
- Not confirmed whether Vercel comments post back to the GitHub PR or can block a merge.
- Cloudflare Pages preview collaboration was not checked.

## 6. Gaps practitioners say remain unsolved in 2025-2026, and how AI is changing the review burden

### Takeaway
AI is raising the volume of doc changes that need review while the review surface has not improved, so the bottleneck is moving further toward review. Practitioner guidance in late 2025 is to hold AI contributions to the same review process as human ones, which presumes a review process that works.

### Cited Findings

AI and review volume:

- Fabrizio Ferri Benedetti (passo.uno; author per site attribution), 10 November 2025: "The dam of AI-written doc contributions might be about to break." — [passo.uno](https://passo.uno/ai-docs-policy-contributions/)
- Same post, on what reviewers already face in code: "posts wondering how to review a vibe-coded pull request consisting of nine thousand new lines of code". — [passo.uno](https://passo.uno/ai-docs-policy-contributions/)
- Same post, on policy: "Docs made using LLMs or written with substantial AI intervention should go through the same review process as the rest." and "You, as the human initiator and overseer of AI work, are responsible for its output." Human reviewers should "focus on what matters, which is style, structure, usefulness, impact, and so on" after automated tests run. — [passo.uno](https://passo.uno/ai-docs-policy-contributions/)
- [SNIPPET, secondary summary of a 2026 paper, not read] A study of 33,596 agent-authored pull requests reportedly found 61.38% received no recorded human review. This is about code PRs, not docs, and I did not verify it against the paper. — [arXiv 2601.15195](https://arxiv.org/html/2601.15195) (the search result that surfaced this figure may be describing a different paper; verify before citing)
- [VENDOR, SNIPPET] Fern, January 2026: agents are described as reliable for mechanical tasks such as updating parameter names and fixing formatting, less reliable for judgement about what readers need; writers should treat agents as junior contributors and review their PRs. — [Fern blog](https://buildwithfern.com/post/technical-writing-ai-agents-devin-cursor-claude-code)

AI as a contribution path and as a product line:

- Manny Silva lists "AI writing agents in Slack" as a contribution path alongside WYSIWYG editors and PRs. — [State of Docs 2026](https://www.stateofdocs.com/2026/docs-tooling), 28 August 2026
- Mirna Wong (dbt Labs), same report: "We set up a GitHub workflow that analyzes an engineer's code changes..." (truncated by the fetch tool; the workflow evidently drafts or flags doc updates from code changes). — [State of Docs 2026](https://www.stateofdocs.com/2026/docs-tooling)
- Vendors now meter AI doc updates as a paid unit: Mintlify charges 250 credits per automation update, with 10,000 credits a month on Pro; GitBook puts "GitBook Agent" on Ultimate, with 10 messages a week on Free. [VENDOR] — [Mintlify pricing](https://www.mintlify.com/pricing), [GitBook pricing](https://www.gitbook.com/pricing)
- State of Docs 2026: 70% factor AI into information architecture decisions. — [State of Docs 2026](https://www.stateofdocs.com/2026/docs-tooling)

Gaps that are still open, by evidence:

- Commenting on rendered Markdown in a GitHub PR: requested May 2025 and February 2026, unanswered by GitHub, being patched by extensions as late as September 2026. — [GitHub Community #186730](https://github.com/orgs/community/discussions/186730), [#160981](https://github.com/orgs/community/discussions/160981)
- Process and tooling investment: "Docs as code is a workflow with processes _and_ tools, and therefore requires investment, maintenance, and a decent amount of custom tooling to get true value out of it." (Moir, April 2024), echoed in State of Docs 2026. — [thisisimportant.net](https://thisisimportant.net/posts/docs-as-code-broken-promise/)
- Platform review and Git review are separate on at least one major platform (GitBook change requests vs PRs). — [GitBook Git Sync troubleshooting](https://gitbook.com/docs/docs-as-code/git-sync/troubleshooting.md)

### Inferences
Pain points that look poorly served, in rough order of evidence strength:

1. **A rendered, change-aware review surface for a PR, usable on any site generator.** GitHub shows changes without usable rendering-plus-comments; preview tools show rendering without changes; platform editors show both only inside their own hosting. DraftView is the one product found aimed at this, and it is unverified beyond its marketing.
2. **Review by people with no developer-platform account.** Every review tool found needs a GitHub, Vercel or Netlify login. Fern removes the account requirement for editing, not for reviewing a PR.
3. **Visual editing of a team's own MDX components** on a self-hosted framework, without formatting churn in the resulting diff.
4. **One review thread.** Comments end up in the PR, in a preview tool, in a platform's change request, and in Slack or email. Netlify's two-way PR sync is the only bridge found.
5. **Review capacity for AI-authored doc changes.** Tools are generating more doc PRs; none of the sources describes a tool that makes a human's review of them faster, as opposed to adding an AI reviewer.

The supply side has consolidated around hosted platforms with flat pricing and metered AI. A team that wants to stay on an open-source generator gets the least help.

### Gaps
- No data found on how much AI has increased docs PR volume. The passo.uno line is a prediction ("might be about to break"), not a measurement.
- AI review bots for prose (as opposed to code) were not researched; no user reports found.
- No 2026 practitioner post was found that states the unsolved gaps directly. The strongest critique (Moir) is from April 2024, and the list above is my synthesis.
- Write the Docs conference talks and technicalwriting.dev were not searched.
- State of Docs 2026 is produced by GitBook. Its framing of "multiple contribution paths" matches GitBook's product, so treat its emphasis with that in mind.
