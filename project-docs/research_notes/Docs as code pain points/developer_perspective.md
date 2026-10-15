# Docs-as-code pain points: the developer and engineer perspective

Research date: 3 October 2026. Scope: developers and engineers (not professional technical writers) working with docs in the repo.

Reading notes for the report writer:

- **Quote fidelity.** Quotes were extracted from the linked pages by a fetch tool that summarizes pages. They are reproduced here as returned. Check each against the primary page before publishing it as a verbatim quote. Hacker News quotes in particular may be truncated mid-sentence.
- **Source labels.** Each finding is tagged `[first-hand]`, `[academic]`, `[survey]`, `[vendor]` (sells a docs or drift product), `[project]` (open-source project's own docs or issue tracker), or `[secondary]` (aggregator or search-result summary, primary not opened).
- **Coverage.** About 25 searches and fetches were run. Several suggested sources were not reached (see Gaps in each section). Absence of a finding here is not evidence that none exists.

## 1. Why does documentation drift happen even when docs sit next to code?

### Takeaway
Developers describe drift as an incentive and process problem, not a location problem: co-locating docs makes a same-PR update possible but nothing makes it mandatory, so docs are skipped unless they are an explicit work item. Academic measurements confirm drift is common in repos where the docs already live beside the code (28.9% of the top 1,000 GitHub projects had at least one outdated code reference in README or wiki).

### Cited Findings

Quantitative evidence

- `[survey]` State of Docs 2026: 30% of respondents name keeping docs in sync with the product as their single biggest workflow challenge; among engineers it is 43%, among technical writers 26%. 67% update docs based on product changes, 39% use a customer feedback loop, and 21% have no formal process for keeping docs in sync. — [State of Docs 2026, Docs and product](https://www.stateofdocs.com/2026/docs-and-product). Sample size and survey dates were not captured from this page. The report is, to my knowledge, published by GitBook (a docs-platform vendor); the publisher was not confirmed in this session.
- `[academic]` 28.9% of the top 1,000 most popular GitHub projects contained at least one outdated reference to source code in their documentation (README and wiki). — [Tan, Wagner, Treude, "Wait, wasn't that code here before? Detecting Outdated Software Documentation", arXiv 2307.04291, 10 July 2023](https://arxiv.org/html/2307.04291)
- `[academic]` Across 3,000+ GitHub projects, "most projects contain at least one outdated code element reference at some point in their history". — [Tan, Wagner, Treude, "Detecting Outdated Code Element References in Software Repository Documentation", arXiv 2212.01479, 2 December 2022; published in Empirical Software Engineering](https://arxiv.org/abs/2212.01479)
- `[secondary]` Aghajani et al., "Software Documentation Issues Unveiled" (ICSE 2019, older material): a search summary reported that up-to-dateness problems account for 39% of documentation content issues. This figure came from a search-result summary only and was not verified against the paper. — [IEEE Xplore record](https://ieeexplore.ieee.org/abstract/document/8811931)

Developer accounts of cause (Ask HN, "How do you manage the drift between implemented code and documentation", 10 May 2024, submitted by Sheeny96) — [thread](https://news.ycombinator.com/item?id=40317113) `[first-hand]`

- Docs as an unfunded work item: "All work has a cost including documentation. You either make updated documentation an explicit work item or it will, at best, get done ad hoc, at worst, it will not be done." — lasereyes136
- Cost-benefit skepticism: "The cost of keeping docs up to date is higher than navigating it, and as IDEs, paradigms, and AI improves, the cost of understanding code goes down." — muzani
- Giving up on parallel docs: "Accept it and make that explicit; make the code clear enough to be self-documenting. Trying to maintain documentation in parallel to a codebase is a fool's errand." — jjgreen
- Docs as a point-in-time artifact: "The documentation is not the definition of a feature. It was the spec the feature was written to, at the time work began." — codingdave

Ownership and definition of done

- `[first-hand]` Moving docs into the repo changes who owns them: "There's an implicit ownership change, from having technical writers own the documentation, to including it as part of the commit." — 8note, [HN thread "It's time to move your docs in the repo", approx. March 2026, 116 points](https://news.ycombinator.com/item?id=47380231)
- `[survey]` Sean Huck (Adyen), quoted in State of Docs 2026: "One of our big goals is to be included in every product team's definition of done." The phrasing ("goal") indicates docs are not yet in the definition of done for every team there. — [State of Docs 2026](https://www.stateofdocs.com/2026/docs-and-product)
- `[first-hand]` Squarespace Domains engineering listed its pre-docs-as-code problems as code and docs living on separate platforms, outdated documentation lacking context, confusing ownership (single-author tools causing friction when people changed teams), and waterfall-style doc delivery. — [Rafael Peixinho, Squarespace Engineering Blog, 10 October 2025](https://engineering.squarespace.com/blog/2025/making-documentation-simpler-and-practical-our-docs-as-code-journey)
- `[first-hand, technical writer]` Sarah Moir argues docs as code is adopted as tooling without the process: "All too often, however, I find that this philosophy is adopted as a **set of tools**, and the processes and integration are ignored." and "Even if you manage to define best practices that your team is committed to following, there isn't a way to force your documentation contributors to adhere to all of these best practices." — [Sarah Moir, "Docs as code is a broken promise", 10 April 2024](https://thisisimportant.net/posts/docs-as-code-broken-promise/)

Dissent on co-location itself

- `[first-hand]` "The only people who insists _all_ doc must live in the same repo as the code are the ones who does not value documentation." — xorcist, [HN, approx. March 2026](https://news.ycombinator.com/item?id=47380231)
- `[first-hand]` "It is NOT easier to author, comment on, label, view history of, move without breaking links, etc. markdown docs vs Confluence." — xixao, [same thread](https://news.ycombinator.com/item?id=47380231)

### Inferences
- The engineer/writer gap in State of Docs (43% vs 26%) is consistent with engineers being the ones who see the code change and know the docs are now wrong, while lacking a required step that makes them fix it. The report itself offers a similar reading, per the search summary.
- Co-location removes a logistical barrier (separate tool, separate review) but leaves the incentive barrier untouched. The DOCER numbers are a direct measurement of drift in docs that are already co-located (README files).
- The recurring developer framing is "explicit work item or it does not happen", which points at definition of done and review enforcement, not authoring tools.

### Gaps
- No quantitative data was found on how often PRs that change behavior also touch docs, or on how often a "docs updated" PR-template checkbox is ticked without a docs change.
- "Software Engineering at Google" (2020), chapter 10 on documentation, was not fetched; no quotes from it are included.
- Stack Overflow Developer Survey 2025 was searched but returned nothing specific to docs maintenance. The only docs figure found: technical documentation is the top learning resource at 68% — [Stack Overflow 2025 Developer Survey](https://survey.stackoverflow.co/2025/).
- Deadline pressure as a cause appears in vendor explainers (for example [endrel.com](https://endrel.com/blog/why-api-docs-go-out-of-sync-and-how-to-fix-it)) but no first-hand engineering account quantifying it was found.

## 2. What mechanisms do teams use to keep docs in sync, and how well do they work?

### Takeaway
The mechanisms developers trust most are those that make docs a build artifact (generated reference, docs generated from tests, CI checks on docstrings); social mechanisms (PR-template checkboxes, reviewer vigilance) are widely recommended but nobody in the sources reports evidence that they work. Automated drift detectors exist and find real problems, but they have narrow coverage (code-element references) and false positives; independent user reports on commercial AI doc-update bots were not found, only vendor and competitor material.

### Cited Findings

Same-PR and review-based mechanisms

- `[first-hand]` "If you have pull request templates, add a checklist item: 'Update documentation'. Reviewers should check the docs and review it as part of pull requests." — ponyous, [Ask HN, 10 May 2024](https://news.ycombinator.com/item?id=40317113)
- `[first-hand]` "Make a branch to change the behavior... document it in the same branch, merge it together." — tacostakohashi, [same thread](https://news.ycombinator.com/item?id=40317113)
- `[first-hand]` Hard enforcement reported as the main benefit of in-repo docs: "The biggest win for me with docs-in-repo isn't the AI angle, it's that pull requests can't land without updating the relevant docs." — redgridtactical, [HN, approx. March 2026](https://news.ycombinator.com/item?id=47380231). The comment does not say how this is enforced.
- `[first-hand]` Spec-first inversion: "You could enforce the process so that you only make code updates based on the documentation...This may be cumbersome to some, but I find it to be effective." — interbased, [Ask HN, 10 May 2024](https://news.ycombinator.com/item?id=40317113)

Generated and tested docs

- `[first-hand]` "where I work we have 2 systems for generating documentation based off of code. 1. OpenAPI specs. 2. I built a system that has a step in CI/CD that checks the docstrings." — Mockapapella, [Ask HN, 10 May 2024](https://news.ycombinator.com/item?id=40317113)
- `[first-hand, tool author]` Docs generated from executable tests: "You write the StrictYAML based tests structured according to your domain language/interactions...then you apply...to generate the markdown/HTML/whatever docs...never goes out of date by definition." — hitchstory (author of the tool being described), [same thread](https://news.ycombinator.com/item?id=40317113)
- `[survey]` Manny Silva (Skyflow, Doc Detective), quoted in State of Docs 2026: "Documentation is creating a contract with the user that tells them how what they're using is supposed to work." Silva maintains Doc Detective, a docs-testing tool. — [State of Docs 2026](https://www.stateofdocs.com/2026/docs-and-product)
- `[vendor]` A Promptless article draft cites an APIContext 2024 white paper for the claim that 75% of production APIs have endpoints that do not conform to their specification, which would mean generated reference docs can be wrong even when generation is automated. The primary white paper was not opened. — [Promptless/promptless.ai PR #1102, 2 October 2026](https://github.com/Promptless/promptless.ai/pull/1102)

Drift-detection tools

- `[academic]` DOCER extracts code-element references from README and wiki pages with regular expressions and flags references no longer present in source; packaged as a GitHub Actions workflow that runs on pull requests. The authors opened issues on 15 projects. True positives led to fixes in google/cctz and google/hs-portray. A documented false positive: google/clif, where removed CMake flags were still relevant to users with multiple Python versions. False positives are suppressed through a `.DOCER_exclude` file. — [Tan, Wagner, Treude, arXiv 2307.04291, 10 July 2023](https://arxiv.org/html/2307.04291)
- `[project]` Small open-source detectors exist: `drift` (AST-based pairing of doc blocks with code anchors, VS Code extension) and `docs-drift` (CLI and GitHub Action validating code examples in Markdown). No adoption or effectiveness data found. — [pallaprolus/drift](https://github.com/pallaprolus/drift), [georg-nikola/docs-drift](https://github.com/georg-nikola/docs-drift)
- `[academic]` READU (arXiv 2607.15780, 2026) targets just-in-time detection and repair of README bugs; it appeared in search results but was not opened. — [arXiv 2607.15780](https://arxiv.org/html/2607.15780v1)

AI doc-update approaches

- `[first-hand]` DIY LLM check, proposed in 2024: "Use GitHub hooks, and for each change in the source code identify the relevant documentation...send the code and the documentation to an LLM with a prompt the boils down to 'is this documentation correct, if not suggest corrections.'" — mlhpdx, [Ask HN, 10 May 2024](https://news.ycombinator.com/item?id=40317113)
- `[first-hand]` Coding agent updates docs in the same change: "Add xyz optional param to our API and claude adds the code + updates the documentation." — themanmaran, [HN, approx. March 2026](https://news.ycombinator.com/item?id=47380231)
- `[survey]` Mirna Wong (dbt Labs): "We're using agentic workflows so engineers don't have to remember to tell us when something needs docs." — [State of Docs 2026](https://www.stateofdocs.com/2026/docs-and-product)
- `[vendor/secondary]` A HappySupport roundup (HappySupport sells a competing product) characterizes Swimm, Promptless, Mintlify's Workflows agent and HappySupport as maintenance tools that detect outdated content; says Promptless proposes diff-shaped updates through the normal review process, has an entry price of $500 per month and fewer integrations; and calls Swimm's staleness detection, which can run in CI, the most mature of the four. None of this is independent user testimony. — [HappySupport, "Best AI Documentation Tools"](https://www.happysupport.ai/en/blog/best-ai-documentation-tools)
- `[vendor]` Mintlify describes its agent as monitoring the codebase and proposing documentation updates when changes ship. — [Mintlify, "How to stop documentation drift"](https://www.mintlify.com/library/how-to-stop-documentation-drift). A third-party review by Ferndesk (a competitor) cites a $300/month price in its title; contents not opened. — [Ferndesk Mintlify review](https://ferndesk.com/blog/mintlify-review)
- `[vendor]` Augment Code's guide states the structural cause as: code changes have an enforced merge path while doc updates require a separate manual step, and proposes merge-triggered agents that draft corrections and route them through human review. — [Augment Code guide](https://www.augmentcode.com/guides/self-updating-documentation-docs-agents-sync)

### Inferences
- Mechanisms fall on a spectrum from social (checkbox, reviewer memory) to structural (docs derived from code or tests). Developers in the sources express confidence only in the structural end; the social end is recommended without evidence.
- Structural mechanisms cover reference material and code samples. Conceptual docs, tutorials and how-to guides have no equivalent, so they remain dependent on review discipline or on AI-proposed updates.
- Regex or AST detectors catch deleted or renamed identifiers. They cannot catch behavior changes behind a stable name, which is the harder drift case. The google/clif false positive shows that "no longer in the code" does not always mean "wrong in the docs".
- AI doc-update bots relocate the problem to review: someone still has to judge the proposed diff. No source measured accept rates or error rates for those proposals.

### Gaps
- No independent, first-hand user reports were found for Promptless, Mintlify's agent, Dosu, Swimm or DocuWriter (effectiveness, false positives, noise, cost). Dosu and DocuWriter did not surface in results at all. Everything found on these tools is vendor or competitor content.
- No data on CODEOWNERS for docs paths, link-checker effectiveness, or doctest adoption rates.
- No measured before/after drift rates for any mechanism.
- Engineering-blog accounts from GitLab, Stripe, Cloudflare, Datadog and Spotify were not reached.

## 3. What friction do developers report with the docs toolchain itself?

### Takeaway
The best-documented toolchain pain in 2023 to 2026 is upstream churn: the Docusaurus v2 to v3 move (MDX v1 to v3) broke content that previously compiled, and the MkDocs ecosystem went through a maintainer breakdown, Material for MkDocs entering maintenance mode (November 2025) and an MkDocs 2.0 announcement that removes the plugin system (January 2026), forcing downstream projects such as Backstage TechDocs to plan migrations. Evidence on slow or flaky CI builds was thin.

### Cited Findings

Docusaurus and MDX

- `[project]` Docusaurus's own guidance: the transition from MDX v1 to MDX v3 is the main challenge to adopting v3; some documents that compiled under v2 fail under v3 and others render differently. `{` now opens a JavaScript expression and `<` a JSX tag, and MDX fails on invalid content in either. The project ships `npx docusaurus-mdx-checker` to list failing files. — [Docusaurus, "Preparing your site for Docusaurus v3"](https://docusaurus.io/blog/preparing-your-site-for-docusaurus-v3); [Upgrading to Docusaurus v3](https://docusaurus.io/docs/migration/v3)
- `[first-hand]` User reports in the official upgrade-support discussion (opened by maintainer slorber, 22 September 2023) — [facebook/docusaurus Discussion #9336](https://github.com/facebook/docusaurus/discussions/9336):
  - "A critical plugin I rely on breaks on v3 but with no errors so I'm at a loss to work out what needs to change." — homotechsual, 29 November 2023 (plugin: docusaurus-theme-github-codeblock)
  - "Admonitions don't render in block quotes in MDX files. We don't know whether it's due to the MDX upgrade or a change in Docusaurus." — mderriey, 21 November 2023
  - "I've tried it once, and I've got a lot of errors. It seems impossible to solve." — RRQM, 17 February 2024
  - johnnyreilly (October 2023) reported React hydration errors and having to rework custom rehype plugins.
- `[project]` Third-party plugin breakage: "MDX compilation fails with Docusaurus v3". — [PaloAltoNetworks/docusaurus-openapi-docs issue #591](https://github.com/PaloAltoNetworks/docusaurus-openapi-docs/issues/591)
- `[secondary]` Additional migration cautions: remark/rehype/unified packages are ESM-only so CommonJS `require()` plugin loading breaks; swizzled theme components may need to be deleted and re-swizzled; Node 18 minimum. — [PocketLantern brief](https://pocketlantern.dev/briefs/docusaurus-2-to-3-after-mdx3-and-node-18-minimum-2026)

MkDocs, Material for MkDocs, Zensical

- `[first-hand]` Timeline from Florian Maas, "The Slow Collapse of MkDocs" (22 March 2026, edited through 25 March 2026) — [fpgmaas.com](https://fpgmaas.com/blog/collapse-of-mkdocs/):
  - 20 April 2024: MkDocs 1.6.0; maintainer @oprypin had stepped down on 6 April 2024 after a public conflict with @squidfunk (Material for MkDocs).
  - 30 August 2024: MkDocs 1.6.1, the final release to date as of the article; about 18 months without meaningful development followed.
  - 11 November 2025: Material for MkDocs enters maintenance mode; Zensical announced.
  - 21 January 2026: MkDocs v2 announced with the plugin system removed.
  - 15 March 2026: ProperDocs launched as an MkDocs 1.x replacement (21 stars one week after launch; Zensical had 3,700+ stars).
  - 90,000+ GitHub projects depend on MkDocs.
  - Quoted in the article: "Before MkDocs Material, MkDocs was a toy. The version 2 announcements suggest MkDocs is going back to its roots — becoming a toy again" (@twardoch); "Is this project being actively maintained, or has it been abandoned?" (@facelessuser, July 2025); "I then slowly stopped triaging issues, answering questions, etc.. So, essentially, nothing much happened since last year" (@pawamoy).
  - Material for MkDocs now displays a build-time warning: "MkDocs 2.0 introduces backward-incompatible changes".
- `[secondary]` Maintenance-mode terms: version 9.7.0 is the final feature release and makes former sponsor-only Insiders features available to all; critical bugs and security issues fixed for at least 12 months; no new features. Zensical is a from-scratch rewrite by the same team that reads existing `mkdocs.yml` and claims 5x faster rebuilds (vendor/project claim, not independently measured). — [Material for MkDocs changelog](https://squidfunk.github.io/mkdocs-material/changelog/); [Material for MkDocs blog](https://squidfunk.github.io/mkdocs-material/blog/). Date note: search results give 11 November 2025 for 9.7.0, while a third-party post about the Zensical announcement is dated 6 November 2025 ([duerrenberger.dev](https://duerrenberger.dev/blog/2025/11/06/material-for-mkdocs-is-no-more-long-live-zensical/)); the announcement and the 9.7.0 release may be a few days apart.
- `[project]` Downstream migration work created for unrelated engineering teams: [renovatebot/renovate Discussion #39232](https://github.com/renovatebot/renovate/discussions/39232), [ddev/ddev issue #7840](https://github.com/ddev/ddev/issues/7840), [kedro-org/kedro issue #5267](https://github.com/kedro-org/kedro/issues/5267), [osmfoundation/welcome-mat issue #197](https://github.com/osmfoundation/welcome-mat/issues/197).

Backstage TechDocs

- `[project]` Issue opened 12 February 2026 by bhupatikrish (now closed): "While this is not a breaking bug in the current release, it represents a significant architectural risk. The uncertainty regarding the underlying build engine creates a hurdle for internal adoption." and "With the upstream project being maintained only till end of 2026, it might be ideal to discuss this topic now so the team and community have enough time to address this issue." — [backstage/backstage issue #32815](https://github.com/backstage/backstage/issues/32815)
- `[project]` Follow-up RFCs: [#33990, Exploring Zensical as the next TechDocs documentation engine](https://github.com/backstage/backstage/issues/33990) and [#34329, future of mkdocs in techdocs](https://github.com/backstage/backstage/issues/34329). Contents not opened.
- `[project]` TechDocs is slow on first request because docs are generated on demand, and with multiple backends docs may be generated once per backend. — [Backstage TechDocs troubleshooting](https://backstage.io/docs/features/techdocs/troubleshooting/); see also [issue #13615](https://github.com/backstage/backstage/issues/13615)

Workflow overhead and previews

- `[first-hand, technical writer]` "If you require a branch, pull request, and all build checks to pass to fix a typo, the time it takes to fix a typo could easily triple." — [Sarah Moir, 10 April 2024](https://thisisimportant.net/posts/docs-as-code-broken-promise/)
- `[first-hand, technical writer]` "To take advantage of these capabilities, you often need to build the tools and checks yourself, or get your documentation platform team to build them for you." — [Sarah Moir, 10 April 2024](https://thisisimportant.net/posts/docs-as-code-broken-promise/)
- `[first-hand, technical writer]` "Documentation reviews in a pull request can be confusing. Just like reviewing UI code is difficult if you only have access to the source and not a staging environment." — [Sarah Moir, 10 April 2024](https://thisisimportant.net/posts/docs-as-code-broken-promise/)
- `[first-hand]` Squarespace's setup needed both local preview (TechDocs CLI) and branch-based remote staging to make review workable; the post discloses no build times or adoption numbers and admits no failure modes. — [Squarespace Engineering Blog, 10 October 2025](https://engineering.squarespace.com/blog/2025/making-documentation-simpler-and-practical-our-docs-as-code-journey)

### Inferences
- A docs site built on a static site generator is a software dependency with its own upgrade and supply-chain risk. The MkDocs episode shows that a governance failure in a docs tool can impose migration work on tens of thousands of projects whose engineers never chose to care about the tool.
- MDX's strictness turns prose into code that can fail to compile, so ordinary characters (`{`, `<`) in a docs edit can break a build. That raises the cost of a casual docs fix for a developer.
- Preview deployments are a prerequisite for meaningful docs review in PRs, and teams have to build or configure them; it is not a default.

### Gaps
- No first-hand numbers on docs build duration or flakiness in CI were found.
- Sphinx and Hugo upgrade or breaking-change pain was not researched.
- Versioned-docs pain and monorepo versus separate docs repo trade-offs: no sources reached.
- Comments on Backstage issue #32815 were not returned by the fetch, so the community reaction is not captured.

## 4. How do developers experience collaborating with technical writers in PRs?

### Takeaway
Evidence from the developer side is sparse. What was found is mostly from writers and vendors: review of raw markup without a rendered preview produces confused feedback, Git is a barrier for non-developer contributors, and moving docs into the repo implicitly shifts ownership from writers to whoever makes the commit.

### Cited Findings
- `[first-hand]` Ownership shift: "There's an implicit ownership change, from having technical writers own the documentation, to including it as part of the commit." — 8note, [HN, approx. March 2026](https://news.ycombinator.com/item?id=47380231)
- `[first-hand]` AI as the thing that aligned roles on docs priority: "LLMs did was get PMs on the same page as TWs, devs, and support toward prioritizing it." — starkparker (quote appears truncated at the start), [same thread](https://news.ycombinator.com/item?id=47380231)
- `[first-hand]` "Developers are discovering docs and accessibility only now due to AI." — theletterf, [same thread](https://news.ycombinator.com/item?id=47380231)
- `[first-hand, technical writer]` Git as a barrier: "Git isn't simple—it's easy to get into an unexpected state with your cloned Git repo, local branch, or disastrous merge conflict resolution decisions." — [Sarah Moir, 10 April 2024](https://thisisimportant.net/posts/docs-as-code-broken-promise/)
- `[first-hand, technical writer]` Uncontrolled authoring environments: different contributors "can use different tools with different settings and functionality", leading to "inconsistent content quality and style." — [Sarah Moir, 10 April 2024](https://thisisimportant.net/posts/docs-as-code-broken-promise/)
- `[vendor]` ClickHelp (sells a non-Git documentation tool) states that the most common docs-as-code challenges arise when contributors are unfamiliar with Git workflows, naming technical writers, product managers and subject-matter experts. — [ClickHelp blog](https://clickhelp.com/clickhelp-technical-writing-blog/when-docs-as-code-reaches-its-limits-and-what-teams-do-next/)
- `[survey]` Writers depend on engineers to be told about changes; dbt Labs is automating that notification ("so engineers don't have to remember to tell us"). — [State of Docs 2026](https://www.stateofdocs.com/2026/docs-and-product)
- `[first-hand]` Squarespace frames a single PR approval flow as reducing collaboration friction: "If you can do all of your tasks in one workflow with one approval process, it encourages doing" (quote truncated in extraction). — [Squarespace Engineering Blog, 10 October 2025](https://engineering.squarespace.com/blog/2025/making-documentation-simpler-and-practical-our-docs-as-code-journey)

### Inferences
- The developer-writer handoff has two failure directions: writers are not told about changes (so they cannot update docs), and when docs move into the repo, developers inherit ownership without inheriting writing support.
- Style linting in CI (for example Vale) is commonly proposed as a way to take style comments out of human review, but no source here reports how developers experience it.

### Gaps
- No first-hand developer complaints were found about writer review latency, style nitpicks, or ownership disputes in PRs. This may be a search gap rather than an absence; it should not be reported as "developers do not complain about this".
- No data on writers lacking repo access or context.
- No quantitative data on docs-PR review turnaround times.

## 5. How has AI changed the developer side in 2025 and 2026?

### Takeaway
AI has created a second docs corpus inside the repo (AGENTS.md, CLAUDE.md, rules files) that drifts like any other doc but is consumed by an agent that acts on it. Early research finds stale references in roughly a quarter of sampled repositories and questions whether these files help at all, while developers report AI-generated docs and tutorials degrading in quality.

### Cited Findings

Context files as a new maintenance burden

- `[academic]` "Context rot": applying an existing README/wiki consistency checker to a statistically representative sample of 356 repositories "identifies stale code element references in 23.0% of repositories". The authors argue existing documentation-consistency tooling is an immediate starting point for detecting it. — [Treude and Baltes, "Context Rot in AI-Assisted Software Development", arXiv 2606.09090, 8 June 2026](https://arxiv.org/abs/2606.09090). A search snippet of the full text reports a dataset of 9,470 context files from 4,463 GitHub repositories; that figure was not verified against the paper body — [HTML version](https://arxiv.org/html/2606.09090v1).
- `[academic]` Study of 2,303 agent context files from 1,925 repositories (Claude Code 922, OpenAI Codex 694, GitHub Copilot 687). Files are modified in multiple commits in 67.4% (Claude Code), 59.7% (Copilot) and 59.2% (Codex) of cases. Median interval between commits: 24.1 hours (Claude Code), 22.0 hours (Codex), 70.7 hours (Copilot). Changes are mostly additive: Claude Code median 57.0 words added per commit, with deletions under 15.0 words across all tools. Median length 335.5 to 535.0 words. Flesch Reading Ease 16.6 for Claude Code files (very difficult). Content prevalence: testing 75.0%, implementation details 69.9%, architecture 67.7%, development process 63.3%, build and run 62.3%, security 14.5%, performance 14.5%. — [Chatlatanagulchai et al., "Agent READMEs: An Empirical Study of Context Files for Agentic Coding", arXiv 2511.12884, 17 November 2025](https://arxiv.org/html/2511.12884v1)
- `[academic]` ETH Zurich evaluation: "providing context files does not generally improve task success rates, while increasing inference cost by over 20% on average", across LLMs and agents and for both LLM-generated and developer-committed files; agents follow the instructions, but "repository overviews, although popular and recommended by model providers, are not helpful." — [Gloaguen et al., "Evaluating AGENTS.md", arXiv 2602.11988, 12 February 2026, revised 29 September 2026](https://arxiv.org/abs/2602.11988). Secondary coverage gives -3% success for LLM-generated files and +4% for human-written files; those two numbers were not confirmed in the abstract — [Medium summary](https://medium.com/@reliabledataengineering/claude-md-dont-work-eth-zurich-study-shows-context-files-reduce-success-rates-by-3-1518cac80929).
- `[secondary]` Further 2026 studies on context files surfaced but were not opened: [arXiv 2602.14690](https://arxiv.org/html/2602.14690v4), [arXiv 2605.10039](https://arxiv.org/pdf/2605.10039), [arXiv 2607.27250](https://arxiv.org/pdf/2607.27250).

Stale docs consumed by agents

- `[vendor]` Promptless (sells a doc-update product) in an article draft dated 2 October 2026: "AI coding agents don't do this. They take documentation at face value and use it as a specification."; "Rules-file rot is a distinct and underappreciated form of documentation drift."; "They drift like any other docs, and their reader implements whatever they say." — [Promptless/promptless.ai PR #1102](https://github.com/Promptless/promptless.ai/pull/1102). The PR lists Claude Sonnet 4.6 as co-author, so the article is itself partly AI-written.
- `[vendor]` The same draft attributes to Mintlify's State of Knowledge 2026 a claim that agent traffic to documentation sites exceeds human traffic by nearly 2:1. Mintlify is a docs-platform vendor; the primary was not opened. — [Mintlify State of Knowledge 2026](https://www.mintlify.com/state-of-knowledge/2026)
- Conflict to flag: the Promptless draft says 66% of developers cite "AI solutions that are almost right, but not quite" as their top AI pain point. Search summaries of the Stack Overflow 2025 survey give 45% for that frustration, and 66% for developers spending more time fixing almost-right AI-generated code. The Promptless figure appears to conflate the two. — [Promptless PR #1102](https://github.com/Promptless/promptless.ai/pull/1102); [Stack Overflow 2025 survey, AI section](https://survey.stackoverflow.co/2025/ai)
- `[survey]` State of Docs 2026: 18% of users access docs through coding AI assistants and 16% through MCP servers; 35% discover docs through AI-powered search (25% at micro-companies, 46% at enterprises). — [State of Docs 2026](https://www.stateofdocs.com/2026/docs-and-product)

Developer sentiment on AI and docs

- `[first-hand]` AI raised the priority of long-standing practice: "These were all best practices before LLMs existed and they remain so even now." — susam; "About 95% of the work needed to make LLMs happy is just general purpose better engineering." — forrestthewoods; "AI means that you cannot defer software design until you've written half code; you cannot defer documentation to random notes." — ronsor. — [HN, approx. March 2026](https://news.ycombinator.com/item?id=47380231); linked article: [dein.fr, 13 March 2026](https://www.dein.fr/posts/2026-03-13-its-time-to-move-your-docs-in-the-repo)
- `[academic]` Qualitative study of 1,154 Reddit and Hacker News posts on AI slop; 1,603 codings across 978 posts, 15 codes in three clusters (Review Friction, Quality Degradation, Forces and Consequences). Framed as a tragedy of the commons where individual productivity gains push costs onto reviewers and maintainers. Developer quotes in the paper: "I'm starting to see documentation and tutorials missing key information and code samples needed to be able to implement something now" and "The DevRel field was absolutely gutted in the layoffs starting in 2022. […] They were the ones maintaining docs and code examples and demo repos". — [Baltes, Cheong, Treude, "An Endless Stream of AI Slop", arXiv 2603.27249v3, 13 June 2026](https://arxiv.org/html/2603.27249v3)
- `[secondary]` DORA: search summaries report that in the 2024 report a 25% increase in AI adoption is associated with a 7.5% increase in documentation quality, and that the 2025 report ties AI effectiveness to data and documentation that is high quality and accessible to AI. Neither figure was verified in the DORA primary. — secondary summaries: [DX on DORA 2024](https://getdx.com/blog/2024-dora-report/), [RedMonk on DORA 2025, 18 December 2025](https://redmonk.com/rstephens/2025/12/18/dora2025/); primary to check: [dora.dev 2024 report](https://dora.dev/research/2024/dora-report/)

### Inferences
- Context files show the usual drift mechanics in compressed form: frequent small additive commits with few deletions suggest instructions accumulate and outdated ones are rarely pruned. The 23.0% stale-reference rate is close to the 28.9% README rate measured in 2023 with the same family of tools, although the samples and populations differ and the two numbers should not be treated as a trend.
- The ETH result and the context-rot result point the same direction for different reasons: descriptive overview content in context files adds cost without improving outcomes, and it is also the content most exposed to drift.
- The claim that "agents implement stale docs" is mainly advanced by vendors that sell drift products. The academic work supports the narrower claims that context files do go stale and that agents follow the instructions in them; no study found here measures defects caused by stale docs.
- AI cuts both ways for drift: coding agents can update docs in the same change at near-zero marginal effort, and they also produce more docs volume that someone has to keep true.

### Gaps
- No research was found that measures the downstream defect rate when an agent consumes stale documentation.
- llms.txt maintenance burden: nothing found.
- No first-hand engineering account of managing AGENTS.md/CLAUDE.md drift in a real team was reached.
- DORA figures need verification against the primary reports.

## 6. What do open-source maintainers say about docs contributions and docs maintenance burden?

### Takeaway
In 2026 the dominant maintainer complaint is the volume of low-value AI-generated contributions, with README and docs edits named as a typical example; the cost of reviewing has not fallen along with the cost of producing. Separately, the MkDocs episode shows docs tooling itself suffering maintainer burnout that then lands on every downstream project.

### Cited Findings
- `[first-hand]` scikit-learn maintainers (Adrin Jalali, scikit-learn core maintainer and VP of Labs at Probabl; Cailean Osborne), 24 February 2026 — [Probabl blog](https://blog.probabl.ai/maintaining-open-source-age-of-gen-ai). Probabl is the company behind scikit-learn, not a docs vendor.
  - "While the cost of writing and contributing code has shrunk thanks to AI, the cost of reviewing and maintaining code hasn't."
  - Low-value PRs include "Redundant contributions to README files and ones that claim significant performance gains while failing basic linting tests."
  - "Almost every second issue on our main repo gets at least one such message, in many cases multiple ones."
  - "A contribution should be worth more to the project than the time it takes to review it."
  - "Maintainers from many open source communities have been deliberating what to do. Some reject all AI-generated contributions, saying they have never seen useful ones."
- `[academic]` Reviewer burden is the dominant theme in developer discussion of AI slop; one team in the data received 30 pull requests daily across 6 reviewers. — [Baltes, Cheong, Treude, arXiv 2603.27249v3, 13 June 2026](https://arxiv.org/html/2603.27249v3)
- `[secondary]` GitHub has acknowledged the problem and was reported in February 2026 to be weighing options including disabling pull requests or limiting them to trusted collaborators. — [Open Source For You, February 2026](https://www.opensourceforu.com/2026/02/github-weighs-pull-request-kill-switch-as-ai-slop-floods-open-source/). Search summaries also reported a "1 out of 10 AI PRs is legitimate" figure and that the Jazzband collective shut down citing AI-generated spam; neither was traced to a primary source and both should be treated as unverified.
- `[academic]` Related 2026 papers surfaced but not opened: ["AI Slop is DDoSing Open Source", arXiv 2607.04003](https://arxiv.org/pdf/2607.04003); ["To Ban or not to Ban? How Open Source Projects Govern GenAI Contributions", arXiv 2603.26487](https://arxiv.org/pdf/2603.26487).
- `[first-hand]` Docs-tool maintainer burnout: "Maintaining MkDocs has been really lonely...after 7 years of doing exactly nothing for MkDocs, @lovelydinosaur stepped in without context" (@oprypin, quoted by Maas). — [Florian Maas, 22 March 2026](https://fpgmaas.com/blog/collapse-of-mkdocs/)
- `[academic]` Maintainer response to automated drift reports was mixed: of issues filed on 15 projects by the DOCER authors, some led to fixes (google/cctz, google/hs-portray) and some were false positives (google/clif). — [arXiv 2307.04291, 10 July 2023](https://arxiv.org/html/2307.04291)

### Inferences
- Docs used to be the recommended "good first contribution". Low-effort AI README edits have made docs PRs a review liability for maintainers, which may reduce willingness to accept outside docs contributions.
- Maintainers now face docs burden from three directions: keeping their own docs true, triaging AI-generated docs PRs, and absorbing churn from docs toolchain dependencies.

### Gaps
- No data on what share of AI-generated PRs are docs-only.
- No maintainer accounts on the ongoing cost of keeping docs current independent of AI (pre-2025 baseline) were collected.
- The "1 in 10" and Jazzband claims need primary sources before use.
