# Writer sentiment: first-hand pain points of docs-as-code

Research date: 3 October 2026. Scope: practitioner and community accounts beyond the already-known Moir, Johnson, Ferri-Benedetti (2023-24 posts) and Shaheen pieces.

**How to read the quotes in these notes.** Pages were read through a fetch tool that summarises with a small model, so:

- "Quoted" items were returned as verbatim text by the fetch tool. They are very likely exact, but I could not eyeball the raw page. Spot-check before publishing any of them. Where the tool inserted "..." inside a quote, the elision is the tool's, not the author's.
- "Snippet only" items come from search-engine summaries. Treat the wording as paraphrase, not as a quote.
- Write the Docs (WTD) newsletter text is the newsletter editors' summary of Slack threads. Slack participants are anonymous in the newsletter; inner quotation marks are the newsletter's own.

**What I could not read.** Reddit r/technicalwriting (old.reddit.com fetch refused; `site:reddit.com` searches returned no Reddit threads), LinkedIn posts (none surfaced), the SAP Community post (HTTP 403), and WTD conference talk videos (YouTube only, no transcripts fetched). The Reddit and LinkedIn parts of the brief are therefore unanswered; see Gaps.

**Sources actually read (16), used for the frequency counts below.**

| ID | Source | Date | Type |
|---|---|---|---|
| S1 | WTD newsletter, "Is Docs-as-Code Worth It?" | May 2024 | Community Slack summary |
| S2 | WTD newsletter, "Docs with Code Or Just as Code?" | Oct 2023 | Community Slack summary |
| S3 | WTD newsletter, "Choosing a CCMS or a static site generator" | Jun 2023 | Community Slack summary |
| S4 | WTD newsletter, "Where to keep content for reuse in docs-as-code" | Mar 2025 | Community Slack summary |
| S5 | WTD newsletter, "Docs-as-code and its discontents: Versioning" | Mar 2018 (older) | Community Slack summary |
| S6 | WTD newsletter, "Migrating from many sources to docs-as-code" | Jul 2026 | Community Slack summary |
| S7 | WTD newsletter, "To buy or build in the age of AI" | Apr 2026 | Community Slack summary |
| S8 | Hacker News thread on "Docs as code (2017)", 99 comments | Jul 2024 | Mostly developers, a few doc practitioners |
| S9 | Hacker News thread on "What docs-as-code means", 113 comments | Oct 2024 | Mostly developers |
| S10 | Andrew Owen, "Docs as code doesn't have to mean Markdown and Git" | 19 Oct 2023 | Practitioner blog |
| S11 | Scott Abel, "Why Your Docs-as-Code Toolchain is Holding You Back" | 5 May 2025 | **Vendor-aligned** (promotes Heretto CCMS and DITA) |
| S12 | Mike Howes Q&A on I'd Rather Be Writing, "From DITA to docs-as-code and Docusaurus" | May 2023 | Practitioner interview |
| S13 | Fabrizio Ferri-Benedetti, "What I've learned designing agentic workflows for docs" | 11 May 2026 | Practitioner blog |
| S14 | Fabrizio Ferri-Benedetti, "New habits for tech writers in the age of LLMs" | 28 Feb 2026 | Practitioner blog |
| S15 | JetBrains Writerside blog, "Collaborating on Docs" | Nov 2023 | **Vendor** (Writerside); snippet only |
| S16 | Tom Johnson, "Discoveries and realizations while walking down the Docs-as-Code path" | 23 Aug 2017 (older) | Practitioner blog; snippet only |

## What do writers on Reddit, Hacker News, Write the Docs, LinkedIn and personal blogs say are the worst parts of docs-as-code?

### Takeaway
Across the 16 sources read, the most-repeated complaints are Git as a barrier to contribution and the hidden, ongoing cost of maintaining a self-built toolchain, followed by what plain-text markup gives up compared with CCMS tools and by pull-request friction for small changes. The same sources carry a consistent counter-view: the alternatives do not get people contributing either, and several writers say they lost little by leaving DITA.

### Cited Findings

**Theme ranking (count of the 16 sources read in which the theme appears as a complaint).** Counts are small and the sample excludes Reddit and LinkedIn, so treat the order as indicative, not statistical.

| Rank | Theme | Sources | Count |
|---|---|---|---|
| 1 | Git complexity and fear; Git as barrier to casual contributors | S1, S8, S9, S10, S11, S15, S16 | 7 |
| 2 | Toolchain maintenance and hidden cost; writer becomes docs engineer; single-maintainer risk | S1, S3, S7, S11, S13, S14 | 6 |
| 3 | Lost CCMS capability: reuse, conditions, structure, translation memory, PDF, multi-version | S1, S3, S4, S5, S10 | 5 |
| 4 | PR and review friction: PR for every typo, developer reviews, diff not equal to rendered output | S2, S8, S9, S10 | 4 |
| 4 | Authoring experience: no spellcheck in IDEs, diagrams hard in Markdown, relative paths, weaker live collaboration than Google Docs or Notion | S2, S4, S9, S10 | 4 |
| 4 | AI-era issues (2025-26): agent workflow cost and reliability, build-your-own temptation, AI unreliable for conversion | S6, S7, S13, S14 | 4 |
| 7 | Versioned docs and coupling docs to code releases | S5, S8, S9 | 3 |
| 8 | Onboarding and migration pain (change resistance, messy conversions) | S6, S11 | 2 |

- WTD Slack's busiest topic in spring 2024 "began in the #docs-as-code channel with a question about what bugs people about a docs-as-code approach." The newsletter summary: "Many of the main problems people discussed had to do with barriers to contributing to docs." (Quoted) — [WTD newsletter, May 2024](https://www.writethedocs.org/blog/newsletter-may-2024/)
- The same summary on cost: "Using free and open-source software means your initial monetary investment is low, but they require a lot of maintenance." (Quoted) — [WTD newsletter, May 2024](https://www.writethedocs.org/blog/newsletter-may-2024/)
- A writer who moved to docs in the same repo as product code "were feeling frustrated"; drawbacks listed were "changes feeling slower and less flexible, requiring PRs for everything, even small typos," and "it felt like there were roadblocks everywhere." (Quoted) — [WTD newsletter, Oct 2023](https://www.writethedocs.org/blog/newsletter-october-2023/)
- Andrew Owen (independent writer, 19 Oct 2023) argues "Docs are not code" and that "Plain text (Markdown, reStructuredText, and to a lesser extent Asciidoc) throws away the key developments in tech writing". (Quoted) — [andrewowen.net](https://andrewowen.net/blog/docs-as-code-doesnt-have-to-mean-markdown-and-git/)
- HN user MilStdJunkie (9 Jul 2024), self-described "pretty die-hard enthusiast for this approach", lists limits: "code is formal language, and docs are natural language"; "the units of change are much, much smaller in a repo of code vs a corpus of documents"; and the review preview is never "IDENTICAL to the format as it's delivered". (Quoted fragments) — [HN comment 40920842](https://news.ycombinator.com/item?id=40920842)
- HN user xorcist (21 Oct 2024): "Documentation isn't code. It may all be text, and share other similarities, but it's something fundamentally different." (Quoted) — [HN comment 41909377](https://news.ycombinator.com/item?id=41909377)
- HN user Nathanba (21 Oct 2024): "the second you put docs into version control next to code it's no longer low friction enough." (Quoted) — [HN comment 41900038](https://news.ycombinator.com/item?id=41900038)
- Scott Abel (5 May 2025; **vendor-aligned**, the piece promotes Heretto and DITA): "Writers troubleshoot pipelines more than they do writing content."; "Writer fatigue. Writers aren't DevOps engineers. They shouldn't have to be." (Quoted) — [The Content Wrangler](https://www.thecontentwrangler.com/p/why-your-docs-as-code-toolchain-is)

**Counter-view: what writers like, and complaints others dismiss**

- Contribution is hard everywhere, not only in Git: "people noted that it can be hard getting people to contribute even in systems that don't require you to learn Git, such as Confluence or even Google Docs." (Quoted) — [WTD newsletter, May 2024](https://www.writethedocs.org/blog/newsletter-may-2024/)
- Git fear is solvable with tooling: "Some suggested that Git GUIs can accomplish most of what people want, leaving them free to focus on the actual docs themselves." The thread closed on "docs-as-code isn't for everyone or every situation." (Quoted) — [WTD newsletter, May 2024](https://www.writethedocs.org/blog/newsletter-may-2024/)
- PRs give safety and accountability: benefits named were "increased accountability for changes with PRs" and that "Some also felt safer knowing any mistakes weren't theirs alone and that the CI checks and reviews helped keep entire pages from breaking." (Quoted) — [WTD newsletter, Oct 2023](https://www.writethedocs.org/blog/newsletter-october-2023/)
- Mike Howes, asked what he lost leaving DITA (May 2023): "Not much really. For a small team, Markdown works much better for us." and "SMEs love that they can simply edit the source and open a merge request (MR), instead creating a Jira ticket". (Quoted) — [I'd Rather Be Writing](https://idratherbewriting.com/blog/docusaurus-questions-and-answers-howes)
- MilStdJunkie (9 Jul 2024) on relative complexity: "DaC is several times...less complicated than standing up a S1000D, a DITA, or even a DocBook publishing system" (Quoted; elision is the fetch tool's) — [HN comment 40922709](https://news.ycombinator.com/item?id=40922709)
- HN user gofreddygo (10 Jul 2024): "After doing this a couple times, it's a no brainer. The benefits are significant, the effort minimal." (Quoted) — [HN comment 40924762](https://news.ycombinator.com/item?id=40924762)
- HN user ElevenLathe (21 Oct 2024) on docs built with code: "They literally can't get out of sync without us noticing (the build breaks)." (Quoted) — [HN comment 41899541](https://news.ycombinator.com/item?id=41899541)
- HN user consteval (21 Oct 2024) dismisses the Git barrier: "I think therefore everyone should know how to navigate their codebase and use Git, at least a little." (Quoted) — [HN comment 41907976](https://news.ycombinator.com/item?id=41907976)
- HN user simonw (10 Jul 2024) dismisses the "coupling limits what gets written" complaint: "I don't think I've ever seen a project argue so passionately for 'all documentation lives in the same repo' that people were put off writing books or tutorials" (Quoted) — [HN comment 40923268](https://news.ycombinator.com/item?id=40923268)

### Inferences
- The two top themes are linked: Git is the visible symptom, and the unfunded toolchain behind it is the structural cause that community threads return to.
- HN commenters are mostly developers. Their praise (docs stay in sync, build breaks on drift) is about engineering outcomes, while WTD complaints are about writer workflow. The two groups are judging different things.
- The strongest anti-docs-as-code wording in this sample comes from a vendor-aligned source (S11). Community sources are more measured ("isn't for everyone").

### Gaps
- No Reddit r/technicalwriting thread could be read. old.reddit.com was refused by the fetch tool and four differently-worded searches returned no Reddit results. Reddit sentiment is entirely missing from the ranking.
- No LinkedIn posts were found or read.
- WTD conference talks (for example "One AWS team's move to docs as code: what worked, what didn't, what's next", https://www.youtube.com/watch?v=Cxuo3udElcE) are video only; content not reviewed.
- Frequency counts rest on 16 sources, two of them snippet-only. They cannot support percentages.

## What specific Git and GitHub/GitLab situations cause the most trouble?

### Takeaway
The situations named are divergent edits to the same file, PRs whose scope sprawls, a PR requirement for trivial fixes, and serving multiple doc versions from a static site generator. I found little first-hand detail on rebasing, release branching or accidental publishes.

### Cited Findings
- Fear of Git deters contribution: "Git was universally acknowledged as something that seems complicated and might scare people away from suggesting improvements." and "Few people want to learn Git, they just want to get things done." (Quoted) — [WTD newsletter, May 2024](https://www.writethedocs.org/blog/newsletter-may-2024/)
- Divergent edits: HN user fucalost (9 Jul 2024): "When the same document is edited by two separate individuals and diverges, it is a nightmare to reconcile the two" (Quoted) — [HN comment 40921092](https://news.ycombinator.com/item?id=40921092)
- PR scope: MilStdJunkie (9 Jul 2024): "PR in a docs as code arrangement can be frickin' terrifying...you have to have a pretty good handle on controlling the scope of change" (Quoted; elision is the fetch tool's) — [HN comment 40920842](https://news.ycombinator.com/item?id=40920842)
- PRs for typos, plus developer review and linting on doc-only changes: "requiring PRs for everything, even small typos. Each change also required reviews from developers and lots of linting when app code wasn't touched." (Quoted) — [WTD newsletter, Oct 2023](https://www.writethedocs.org/blog/newsletter-october-2023/)
- No file locking: Andrew Owen (Oct 2023) notes "locks, which are a feature of Subversion but absent from Git". (Quoted fragment) — [andrewowen.net](https://andrewowen.net/blog/docs-as-code-doesnt-have-to-mean-markdown-and-git/)
- Versioned docs (2018, older): WTD Slack consensus was "SSGs are not the optimal solution if you need to serve multiple versions of your content." Git submodules were floated as a workaround; Sphinx, DocFX, Antora and MadCap Flare were recommended instead. (Quoted) — [WTD newsletter, Mar 2018](https://www.writethedocs.org/blog/newsletter-march-2018/)
- Submodules are disliked: HN user consteval (21 Oct 2024): "submodules kind of suck ass, so I don't know if devs would be keen for that." (Quoted) — [HN comment 41907976](https://news.ycombinator.com/item?id=41907976)
- Coupling docs to code versions: HN user xorcist (9 Jul 2024): "The most useful documentation describes all versions of the software, and should be only loosely coupled with it" (Quoted) — [HN comment 40922565](https://news.ycombinator.com/item?id=40922565)
- Snippet only (2017, older): Tom Johnson found smaller repositories mean fewer merge conflicts, because with many writers branching and merging in one repo it is easier for someone to break things. — [I'd Rather Be Writing, 2017](https://idratherbewriting.com/2017/08/23/content-architecture-and-repo-sizes/)
- Snippet only (**vendor**, Nov 2023): JetBrains writers say that having to collaborate on the same file, pull and rebase daily, and resolve conflicts may signal organisational and content-management problems. — [JetBrains Writerside blog](https://blog.jetbrains.com/writerside/2023/11/collaborating-on-docs-best-practices-and-strategies-from-jetbrains-writers/)
- Mitigations writers offered: open PRs early, keep PRs small, batch tiny fixes, pair-review on a call with the docs running locally, and run CI only on touched files. Summary line: "if you're going to use docs-as-code, it's best to optimize for the code tools and processes available to you." (Quoted) — [WTD newsletter, Oct 2023](https://www.writethedocs.org/blog/newsletter-october-2023/)

### Inferences
- The Git complaints are less about Git commands and more about process weight: every change, however small, pays the full branch, PR, review and CI cost.
- Versioning complaints are old (2018) but I found nothing saying they are solved; Docusaurus-style built-in versioning was not discussed in the sources read.

### Gaps
- No first-hand accounts found of rebasing trouble, release-branch strategy, long-lived branches, or accidental publishes. These are the situations most likely to appear on Reddit, which I could not read.
- The two merge-conflict sources (S15, S16) are snippet-only and one is a vendor.

## What do writers say about authoring experience?

### Takeaway
Complaints centre on the gap between what the writer or reviewer sees (raw markup, a diff) and what ships, on developer editors lacking writer basics, and on reuse syntax adding learning load. I found almost nothing first-hand on local build setup, preview speed or Vale.

### Cited Findings
- Review format versus output: MilStdJunkie (9 Jul 2024): "the review format will never be completely equivalent to the deliverable. The build process will always stand in the way" (Quoted) — [HN comment 40920842](https://news.ycombinator.com/item?id=40920842)
- Editors: Andrew Owen (Oct 2023): "Most software integrated development environments (IDEs) don't even have a default spellchecker" (Quoted) — [andrewowen.net](https://andrewowen.net/blog/docs-as-code-doesnt-have-to-mean-markdown-and-git/)
- Review back-and-forth: Owen says that in documentation review "there tends to be much more back and forth" than in code review. (Quoted fragment) — [andrewowen.net](https://andrewowen.net/blog/docs-as-code-doesnt-have-to-mean-markdown-and-git/)
- Custom syntax is a tax: "any syntax used to add features to docs, such as content reuse, adds another thing people have to learn before they can contribute." (Quoted) — [WTD newsletter, May 2024](https://www.writethedocs.org/blog/newsletter-may-2024/)
- Collaboration: "collaboration seemed more difficult than in tools like Google Docs and Notion." (Quoted) — [WTD newsletter, Oct 2023](https://www.writethedocs.org/blog/newsletter-october-2023/)
- Linting noise: "lots of linting when app code wasn't touched"; writers asked for CI that runs "only ... when needed (not linting untouched code)". (Quoted; second phrase is from the newsletter's suggestion list) — [WTD newsletter, Oct 2023](https://www.writethedocs.org/blog/newsletter-october-2023/)
- Diagrams: HN user smokel (20 Oct 2024): "diagrams are typically more useful than text. This still requires some manual effort which is difficult to achieve with Markdown." (Quoted) — [HN comment 41897119](https://news.ycombinator.com/item?id=41897119)
- Partials and paths: arguments for keeping reusable snippets in a separate directory included "relief from fear that the snippets to include would be built as complete pages, and not having to worry about relative paths." (Quoted) — [WTD newsletter, Mar 2025](https://www.writethedocs.org/blog/newsletter-march-2025/)
- Counter-view: Mike Howes (May 2023) praised speed: "Docusaurus build times, in addition to the short deployment times to GitHub Pages, really impressed us" (Quoted) — [I'd Rather Be Writing](https://idratherbewriting.com/blog/docusaurus-questions-and-answers-howes)

### Inferences
- The recurring authoring complaint is a fidelity gap: authors and reviewers work on a representation that differs from the published page. That is the same root as the already-known raw-diff review complaint.

### Gaps
- No first-hand quotes found on WYSIWYG preference, local environment setup (Node, Python, Ruby), slow previews, broken builds, or Vale specifically. Vale appears only in Ferri-Benedetti's 2026 post as a deterministic first pass before an AI reviewer (see AI section).
- No material on editor differences (VS Code versus others).

## What do writers who moved from DITA, Flare, Paligo, Confluence or other tools say they lost?

### Takeaway
Writers who value CCMS features name reuse, conditional text, variables, structured authoring, style enforcement, translation memory and out-of-the-box PDF and multi-version output as the losses, and say they can be rebuilt on a static site generator only with engineering time. At least one small-team migrator says he lost almost nothing because the team had not been using reuse well.

### Cited Findings
- Andrew Owen's list of what dedicated tools provide and plain text lacks: "Content reuse (single sourcing, conditional text, variables and so on)."; "Structured authoring"; "Style enforcement"; "Separation of content from layout"; "Translation memory". For enterprise documentation "you should demand an XML-based solution". (Quoted fragments, 19 Oct 2023) — [andrewowen.net](https://andrewowen.net/blog/docs-as-code-doesnt-have-to-mean-markdown-and-git/)
- CCMS does PDF and single sourcing natively: CCMS tools "can make PDFs and web-based documents and work with single sources from the start. Of course, you can also do this with static site generators, but you have to spend time and effort creating the build steps and themes." (Quoted) — [WTD newsletter, Jun 2023](https://www.writethedocs.org/blog/newsletter-june-2023/)
- Hidden cost of free tools: "Static site generators, for example, often need theme development, build process upkeep, and document conversions, which can take a lot of time and resources." (Quoted) — [WTD newsletter, Jun 2023](https://www.writethedocs.org/blog/newsletter-june-2023/)
- The other side of that trade: "Paligo and other CCMSs demand a specific understanding of XML or the tool itself", so an SSG "might be a better choice if you are a single writer or have users who are not tech-savvy." (Quoted) — [WTD newsletter, Jun 2023](https://www.writethedocs.org/blog/newsletter-june-2023/)
- Reuse has no settled convention in docs-as-code: a March 2025 Slack thread debated where transcluded snippets should even live, ending with "there is no single right way to approach any project like this." (Quoted) — [WTD newsletter, Mar 2025](https://www.writethedocs.org/blog/newsletter-march-2025/)
- Multi-version output: 2018 consensus that static site generators are not optimal for serving multiple versions; MadCap Flare was among the recommended alternatives (older). — [WTD newsletter, Mar 2018](https://www.writethedocs.org/blog/newsletter-march-2018/)
- Counter-view, DITA to Docusaurus: Mike Howes (May 2023): "One of the main strengths of structured authoring is reuse, but we weren't really using it effectively" (Quoted) — [I'd Rather Be Writing](https://idratherbewriting.com/blog/docusaurus-questions-and-answers-howes)
- Migration from Confluence, SharePoint and Zendesk (Jul 2026): "automated conversions across multiple sources are never straightforward," and "people resist change, so explain the big picture and benefits and offer education." (Quoted) — [WTD newsletter, Jul 2026](https://www.writethedocs.org/blog/newsletter-july-2026/)
- Snippet only, page returned 403: an SAP Community post, "From DITA to Doc-as-Code: A Technical Writer's Honest Account" (about Aug 2026), reportedly says DITA reuse updates everywhere instantly while docs-as-code needs other approaches, and that translation needs deliberate engineering to integrate localisation services and protect Markdown syntax. Wording and author unverified. — [SAP Community](https://community.sap.com/t5/supply-chain-management-blog-posts-by-sap/from-dita-to-doc-as-code-a-technical-writer-s-honest-account/ba-p/14467149)
- An ACM SIGDOC 2024 paper covers an enterprise DITA-to-Markdown migration and structured authoring inside docs-as-code; surfaced in search but not read. — [ACM SIGDOC '24](https://dl.acm.org/doi/fullHtml/10.1145/3641237.3691677)

### Inferences
- What is "lost" depends on team size and whether translation, PDF and multi-product variants are real requirements. Small API-docs teams report little loss; enterprise and regulated-output writers report a lot.
- The losses are not absolute: sources agree the features can be rebuilt, which converts the loss into the toolchain-maintenance burden ranked second above.

### Gaps
- No first-hand accounts read from writers leaving MadCap Flare or Paligo specifically; a targeted search returned only vendor and comparison pages.
- No practitioner quotes found on translation workflow beyond Owen's "Translation memory" item and the unverified SAP snippet.
- SAP post and ACM paper unread; both look valuable for a follow-up.

## What do writers say about their standing and workload?

### Takeaway
Writers describe absorbing tooling work that nobody funds, with systems that depend on one person and engineering time for docs treated as secondary. By 2026 a leading practitioner frames self-reliance as the expected posture.

### Cited Findings
- Docs infrastructure is second priority: "if product development is a priority for your company, then allocating any engineering resources to a documentation platform will be considered secondary." (Quoted) — [WTD newsletter, Apr 2026](https://www.writethedocs.org/blog/newsletter-april-2026/)
- Process becomes the product: a WTD reader raised the idea that "you want the docs to be the product, but that product shouldn't be your processes or docs website." (Quoted; newsletter attributes it to an article titled "The pros and cons of using Markdown") — [WTD newsletter, May 2024](https://www.writethedocs.org/blog/newsletter-may-2024/)
- Fabrizio Ferri-Benedetti (28 Feb 2026): "the era of waiting for engineering to fix our tools or hand us context is over" (Quoted) — [passo.uno](https://passo.uno/new-habits-tech-writers-ai-age/)
- **Vendor-aligned** (Scott Abel, 5 May 2025, promoting Heretto): "You rely on a single person to maintain the system."; "If new writers need a crash course in Git, Makefiles, and CI jobs to fix a typo, the barrier is too high."; "Onboarding new contributors is a slog." (Quoted) — [The Content Wrangler](https://www.thecontentwrangler.com/p/why-your-docs-as-code-toolchain-is)
- Standing among developers: HN user xtiansimon (20 Oct 2024): "When I'm working collaboratively and I hear people tell me what I'm doing is not important, I have to believe there is some truth to it." (Quoted; context of the comment not checked) — [HN comment 41898512](https://news.ycombinator.com/item?id=41898512)
- Developer view of why writers are needed: HN user bluGill (9 Jul 2024): "Developers are too close to the code to write effective documentation for it. They will go into great detail about things that nobody else cares about" (Quoted) — [HN comment 40921035](https://news.ycombinator.com/item?id=40921035)
- Upside for writer standing: HN user florianmartens (21 Oct 2024): "technical writers have a nice interface to contribute to them (and hold devs accountable!)." (Quoted) — [HN comment 41904235](https://news.ycombinator.com/item?id=41904235)
- SME contribution stays shallow even when the workflow works: Mike Howes (May 2023): "it's been slow to get users to provide more substantial content, such as best practices, use cases" (Quoted) — [I'd Rather Be Writing](https://idratherbewriting.com/blog/docusaurus-questions-and-answers-howes)

### Inferences
- The "more engineer contributions" promise is only partly met: engineers fix small things through merge requests, while substantive content still falls to writers.
- Writers who enjoy tooling describe the docs-engineer role as empowerment (Ferri-Benedetti); writers who do not describe it as fatigue (Abel's framing). The complaint is about the role being unchosen and unfunded.

### Gaps
- Nothing found on hiring expectations (job ads requiring Git, code or CI skills) or on pay and career effects. Reddit career threads would be the natural source.
- No first-hand account of a writer maintaining a theme or CI pipeline day to day beyond the general statements above.

## Which complaints have changed or appeared in 2025 and 2026 with AI tools?

### Takeaway
In 2025-26 material the new concerns are the cost, timing and reliability of agent workflows in doc repos, the temptation to have AI build a custom docs platform that someone must then maintain, and the extra artefacts writers now produce for machine readers. I found no first-hand writer complaint about a flood of AI-generated doc PRs; that specific grievance is unconfirmed.

### Cited Findings
- Build-versus-buy reopened by AI (Apr 2026): "unless you are proficient in the code developed by the AI agent, you need to have engineering resources to check and test the code and to maintain the platform." Outcome: "most contributors expressed concern about the long-term costs outweighing apparent gains." (Quoted) — [WTD newsletter, Apr 2026](https://www.writethedocs.org/blog/newsletter-april-2026/)
- Same issue, on replacing writers: "AI agents are currently incapable of producing product docs without the help of documentarians." (Quoted, from the section "Docs without dedicated documentarians?") — [WTD newsletter, Apr 2026](https://www.writethedocs.org/blog/newsletter-april-2026/)
- AI is weak for migration (Jul 2026): "AI isn't the best tool for converting documentation." Deterministic tools such as Pandoc were preferred; llms.txt, Markdown exports and a local MCP server were suggested for LLM delivery. (Quoted sentence; remainder summarised) — [WTD newsletter, Jul 2026](https://www.writethedocs.org/blog/newsletter-july-2026/)
- Agent workflows in a docs repo, Ferri-Benedetti (11 May 2026): "The main drawback with AWs is that their intricate security design frequently backfires"; "A repo-wide sweep on every commit would be expensive and slow"; "when and how we're running agentic workflows becomes a non-trivial matter"; "few people have tried this before". (Quoted) — [passo.uno](https://passo.uno/agentic-workflows-for-docs/)
- Same post, positive: "Mixing dumb robots with smart ones...helps reduce false positives while ensuring complete scan coverage" (a deterministic Vale pass first, then an agent review). (Quoted; elision is the fetch tool's) — [passo.uno](https://passo.uno/agentic-workflows-for-docs/)
- Writing for machine readers is now part of the job, Ferri-Benedetti (28 Feb 2026): "Context is the new content, and curation is the skill that makes it useful"; "All that is docs. It doesn't matter that it's going to be consumed by AI"; "any LLM can put together plausible docs with some context and a simple prompt". (Quoted) — [passo.uno](https://passo.uno/new-habits-tech-writers-ai-age/)
- AI also strengthens the case for docs-as-code: Ferri-Benedetti (28 Feb 2026) says "file diffs are a better source of truth than engineer notes". (Quoted) — [passo.uno](https://passo.uno/new-habits-tech-writers-ai-age/)
- Snippet only: Ferri-Benedetti and Tom Johnson's 2025-26 podcast and posts describe writers becoming "context curators", and discuss the uncertain future of llms.txt. — [I'd Rather Be Writing podcast page](https://idratherbewriting.com/blog/podcast-fabri-tom-sept-episode-1); [passo.uno](https://passo.uno/from-tech-writers-to-ai-context-curators/)
- **Vendor** claims, not writer sentiment: Fern (Jan 2026) says agents are reliable for mechanical edits and less reliable for judgment calls, and that reviewers should read conceptual explanations closely. — [Fern blog](https://buildwithfern.com/post/technical-writing-ai-agents-devin-cursor-claude-code)

### Inferences
- AI shifts the top-ranked themes more than it adds new ones: agents lower the Git and tooling barrier for writers, and move the cost to workflow design, review and maintenance of generated code.
- Plain-text-in-Git is being reframed as an advantage for AI (diffs and Markdown as agent input), which weakens the 2024 "broken promise" argument in practitioner writing.

### Gaps
- No first-hand writer account found of review burden from AI-generated doc PRs, of agents making unwanted edits, or of complaints about llms.txt upkeep. Searches returned vendor content and academic papers on code PRs only.
- Podcast and "context curators" posts were not fetched; only search summaries were seen.
- 2025-26 Reddit and LinkedIn discussion, where such complaints would likely appear, was unreachable.
