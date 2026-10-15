# Survey and report data on docs-as-code pain points

Research date: 3 October 2026.

Method caveats that apply to everything below:

- Pages were read through a fetch tool that summarizes content. Figures were cross-checked against a second fetch or a search snippet where possible, but chart-level data (full option lists behind charts) was often not retrievable. Anything marked "not retrievable" is a gap, not a zero.
- Publisher conflicts of interest: State of Docs is run by GitBook (sells a docs platform that competes with docs-as-code tooling). Postman sells an API platform with documentation features. Stack Overflow sells a knowledge-base product (Stack Internal, formerly Stack Overflow for Teams). DORA is a Google Cloud program. Write the Docs and Tom Johnson (idratherbewriting.com) are community/individual sources with no competing product.
- Almost no survey isolates docs-as-code users as a segment. The only survey found that is specifically about the docs-as-code population is Tom Johnson's 2020 developer documentation survey, which is six years old.

## What are the full ranked lists of workflow challenges in State of Docs 2025 and 2026, what is the runner-up to "keeping docs in sync", and what do the reports say about docs-as-code, Git, review, and tooling satisfaction?

### Takeaway
Keeping docs current is the top-ranked challenge in both editions (30% of all respondents in 2026 as a single biggest workflow challenge; 56% of API-docs teams in 2025), but neither report publishes, in retrievable text, the full ranked list or names the runner-up, and neither reports on docs-as-code, pull-request review, or tool satisfaction as such. The 2026 report's only Git-related figure is that 21% publish with open-source platforms or Git repos versus 45% on dedicated documentation tools.

### Cited Findings

State of Docs 2026 (GitBook; 1,131 respondents; "more than 30 in-depth interviews"; fielded 2026, exact field dates and recruitment method not stated on the pages read)

- Respondent roles: technical writers 35%, leadership/decision-makers 21%, engineers 15%, customer experience 7%, operations 6%, support 5%, developer relations 5%, marketing 4%. Regions: Europe and Middle East 37%, North America 33%, Asia-Pacific 19%, South America 6%, Africa 6%. Company size described as spread fairly evenly, with small (freelance to 50) and enterprise (301+) the two largest segments — [State of Docs 2026, Introduction and demographics](https://www.stateofdocs.com/2026/introduction-and-demographics)
- 30% cite keeping docs in sync with the product as their single biggest workflow challenge, described as "nearly double the runner-up". The runner-up is not named and has no percentage in the page text (implies roughly 15 to 17%, but that is an inference) — [State of Docs 2026, Docs and product](https://www.stateofdocs.com/2026/docs-and-product)
- By role: 43% of engineers versus 26% of technical writers name sync as their biggest challenge. Report's explanation: "engineers see the product changing and know the docs are falling behind, while writers juggle multiple priorities" — [State of Docs 2026, Docs and product](https://www.stateofdocs.com/2026/docs-and-product)
- Processes for keeping docs updated: 67% update docs based on product changes; 39% use a customer feedback loop; 21% have no formal process at all — [State of Docs 2026, Docs and product](https://www.stateofdocs.com/2026/docs-and-product)
- How users reach docs: direct navigation 66%, in-product links 54%, traditional search engines 45%, AI-powered search 35% (25% at micro-companies, 46% at enterprises), coding AI assistants 18%, MCP servers 16%; 40% offer in-product help. Planned-versus-current gap: AI assistants +9 points, MCP servers +8 points — [State of Docs 2026, Docs and product](https://www.stateofdocs.com/2026/docs-and-product)
- Publishing platform: dedicated documentation tools 45%; open-source platforms / Git repos 21%; website publishing platforms 9%. This is the closest thing in the report to a docs-as-code adoption figure — [State of Docs 2026, Docs tooling](https://www.stateofdocs.com/2026/docs-tooling)
- Information architecture: self-created guidelines 53% (+4 points year over year), intuition 32% (-4), established frameworks such as Diataxis or DITA 28% (+4). 70% factor AI into IA decisions at some level; 46% considering AI but not as primary focus (+10); 20% not thinking about AI for IA (-11) — [State of Docs 2026, Docs tooling](https://www.stateofdocs.com/2026/docs-tooling)
- Team structure: 35.6% centralized team; 22% no formal documentation team; 16% hybrid. Reporting line: Product 27%, CEO/leadership 20%, Engineering 13%, Support 9%, Marketing 5% — [State of Docs 2026, Docs team structure](https://www.stateofdocs.com/2026/docs-team-structure)
- Who is responsible for docs (multi-select as summarized): technical writers 55%, Product 44%, Engineering 39%, Customer support 29%. The exact question wording was not retrievable, so treat the 55% as "technical writers named as leading docs", not "55% of technical writers" — [State of Docs 2026, Docs team structure](https://www.stateofdocs.com/2026/docs-team-structure)
- Organizations with no formal docs team lag on AI: 60% adopted AI-powered features (vs 79 to 81% with formal teams), 70% use AI regularly for creation (vs 79 to 80%), 26% have AI guidelines (vs 47 to 49%) — [State of Docs 2026, Docs team structure](https://www.stateofdocs.com/2026/docs-team-structure)
- Measurement: 25% track no documentation metrics; 35% track no internal process metrics; only 11 to 12% track business-outcome metrics; 57% do not track leads from docs; 49% do not track internal docs metrics (55% in 2025); 32% of medium companies (51 to 300) track nothing versus 18% of enterprises; 74% rate docs somewhat or very effective for self-service troubleshooting. The report calls this "the biggest missed opportunity in documentation" — [State of Docs 2026, Measuring docs success](https://www.stateofdocs.com/2026/measuring-docs-success)
- 50% say docs matter for closing deals — [State of Docs 2026, landing page](https://www.stateofdocs.com/2026)

State of Docs 2025 (GitBook; 444 respondents; first edition)

- 444 respondents: technical writers, managers, engineers, support, designers, marketers, developer advocates; mostly North America and Europe; most at small businesses or large enterprises. No role percentages on the page — [State of Docs 2025, Introduction and basic stats](https://www.stateofdocs.com/2025/introduction-basic-stats)
- 56% said keeping everything up to date is their biggest challenge related to API docs. Scope is teams with API documentation, not all respondents — [State of Docs 2025, Documentation tooling and API docs](https://www.stateofdocs.com/2025/documentation-tooling-and-api-docs)
- GitBook's own summary generalizes this to "more than half of the survey respondents said their biggest challenge is keeping docs up to date" and says they want tools that update automatically. This wording is broader than the chapter's API-docs scope; treat the chapter as authoritative — [GitBook blog, State of Docs 2025 highlights](https://www.gitbook.com/blog/state-of-docs-report-2025)
- Tooling: version control platforms (GitHub/GitLab) are "by far the most common" tool teams rely on; no percentage given — [State of Docs 2025, Documentation tooling and API docs](https://www.stateofdocs.com/2025/documentation-tooling-and-api-docs)
- About 75% say docs are at least somewhat centralized; about half have everything in a single platform; about 25% say docs are spread across multiple platforms — [State of Docs 2025, Documentation tooling and API docs](https://www.stateofdocs.com/2025/documentation-tooling-and-api-docs)
- 77% use homegrown methods to structure information; fewer than 1 in 4 use a framework such as Diataxis — [State of Docs 2025, Documentation tooling and API docs](https://www.stateofdocs.com/2025/documentation-tooling-and-api-docs)
- API docs: almost 74% of companies offering APIs use OpenAPI; about 80% say API docs are more important than before (over 50% "much more"); 40% say engineers handle API documentation alone without technical writers — [State of Docs 2025, Documentation tooling and API docs](https://www.stateofdocs.com/2025/documentation-tooling-and-api-docs)
- 87% of large companies have one or more technical writers; 46% of large companies have decentralized or hybrid docs teams. The report describes a "documentation debt" phase where "engineering teams grow faster than docs teams", most visible at mid-sized companies. This is qualitative; no debt figure is given — [State of Docs 2025, Documentation team structure](https://www.stateofdocs.com/2025/documentation-team-structure)
- 54% believe documentation generates as many or more leads than marketing sites — [State of Docs 2025, landing page](https://www.stateofdocs.com/2025)

### Inferences
- The sync problem is the best-evidenced pain point in State of Docs, and it is felt more strongly by engineers (43%) than writers (26%). For a docs-as-code product, that suggests the engineer persona is the one most motivated by drift detection.
- "Nearly double the runner-up" puts the second-place challenge at roughly 15 to 17%, so the remaining 70% of responses are split across several smaller challenges; no other single workflow problem dominates.
- The 2025 "56%" and 2026 "30%" are not comparable: different question (API-docs challenge versus single biggest workflow challenge) and different base.
- State of Docs does not treat docs-as-code as a category. Its 21% "open-source platforms / Git repos" is a publishing-platform answer and likely understates Git use, since many dedicated tools (GitBook, Mintlify, ReadMe, Fern) sync with Git.

### Gaps
- Full ranked list of workflow challenges for 2026, including the named runner-up: not retrievable from the page text. It is probably in a chart. Needs a manual read of https://www.stateofdocs.com/2026/docs-and-product.
- No State of Docs figures found on pull-request review, review turnaround, approval bottlenecks, or satisfaction with specific tools.
- 2026 chapter "Purchase decisions and business impact" and 2025 chapters "Purchase decisions" and "Metrics and measurement" were not read.
- Exact field dates and recruitment method for both editions were not found. The fetch tool reported a publication date of "August 28, 2026" for both the 2025 and 2026 pages, which cannot be right for the 2025 edition; treat that date as unverified.

## What do Write the Docs, Tom Johnson's survey, Stack Overflow, DORA, the GitHub Open Source Survey, Postman, and academic studies say about documentation pain points?

### Takeaway
Across independent sources the same problem recurs: documentation that is outdated, inconsistent, or incomplete (93% of open-source respondents in 2017; 55% of API teams in 2025). For writers specifically, the only docs-as-code-era survey (2020) names technical knowledge, lack of time, and getting engineers to review as the biggest challenges. Write the Docs' salary survey has no tooling or pain-point questions.

### Cited Findings

Tom Johnson, developer documentation trends survey (idratherbewriting.com; fielded January to March 2020; 405 completed of 855 started; promoted via his blog, LinkedIn, Twitter; respondents are people writing docs for developers, 37% US, 15% India, 5% Germany, 5% UK; 39% identify with Write the Docs, 14% with STC). OLDER DATA (2020).

- Biggest challenges named by respondents (free-text, no percentages): technical knowledge requirements, insufficient time/bandwidth, obtaining engineer reviews, addressing both novice and advanced audiences — [Developer documentation trends, survey results](https://idratherbewriting.com/learnapidoc/docapis_trends.html)
- Review process: 25% review docs with code review tools, 19% in-person meetings, 14% collaborative annotation tools — [Developer documentation trends, survey results](https://idratherbewriting.com/learnapidoc/docapis_trends.html)
- Engineer contributions: 31% via pull requests, 31% through wikis, 22% by direct repository access — [Developer documentation trends, survey results](https://idratherbewriting.com/learnapidoc/docapis_trends.html)
- Who creates reference docs: 36% engineers only, 26% engineers and writers together, 6% writers only — [Developer documentation trends, survey results](https://idratherbewriting.com/learnapidoc/docapis_trends.html)
- Team shape: 34% lone writers, 31% in teams of 2 to 4; 75% satisfied with their job — [Developer documentation trends, survey results](https://idratherbewriting.com/learnapidoc/docapis_trends.html)
- A separate 2019 Tom Johnson survey of engineers who write docs reportedly found 80% prefer to treat docs like software (Git, text editors, Markdown, static site generators). Seen only in a search snippet of his post; not verified on the page. OLDER DATA (2019) — [Results from survey about engineers who write documentation](https://idratherbewriting.com/2019/03/21/trends-with-engineers-writing-docs/)

Write the Docs Documentation Salary Survey 2024 (Write the Docs community; 779 documentarians in 55 countries; 6th annual)

- The survey contains no questions on tools, docs-as-code, or workflow pain points. It covers pay, employment type, location, and satisfaction — [WTD Salary Survey 2024 results](https://www.writethedocs.org/surveys/salary-survey/2024/)
- Primary role: technical writer 84.7%, editor 2.6%, DocOps 2.3%, project/product manager 2.1%. 68.8% say documentation is their whole official job description — [WTD Salary Survey 2024 results](https://www.writethedocs.org/surveys/salary-survey/2024/)
- Satisfaction: 71.3% satisfied or very satisfied with employment overall; 71.8% of employees satisfied with salary; 51.5% of employees report job stability "around the same" as the prior year — [WTD Salary Survey 2024 results](https://www.writethedocs.org/surveys/salary-survey/2024/)

Stack Overflow Developer Survey (developers, not writers)

- 2024: 61% of respondents spend more than 30 minutes a day searching for answers or solutions; one in four spend at least 60 minutes — [Stack Overflow Developer Survey 2024, professional developers](https://survey.stackoverflow.co/2024/professional-developers)
- 2024: 45% encounter knowledge silos frequently; 53% agree or strongly agree that waiting on answers disrupts their workflow. Figures seen via Stack Overflow's own marketing summary for its Teams product — [Stack Overflow, Insights from the 2024 Developer Survey](https://stackoverflow.co/internal/resources/your-developers-deserve-better-insights-from-the-2024-developer-survey/)
- 2024: technical documentation is the top online learning resource at 84%; API and SDK documents are the preferred documentation type for 90% — [Stack Overflow Developer Survey 2024](https://survey.stackoverflow.co/2024/)
- 2025 (more than 49,000 developers, 177 countries): technical documentation remains the top learning resource at 68%, lower than the prior year — [Stack Overflow blog, 2025 Developer Survey results](https://stackoverflow.blog/2025/12/29/developers-remain-willing-but-reluctant-to-use-ai-the-2025-developer-survey-results-are-here/)

DORA (Google Cloud; State of DevOps reports; respondents are software delivery professionals)

- 2023: high-quality documentation leads to 25% higher team performance relative to low-quality documentation; trunk-based development is estimated to have 12.8x more impact on organizational performance when documentation quality is high; documentation amplifies the effect of continuous integration by 2.4x, continuous delivery by 2.7x, and reliability practices by 1.4x. Figures seen in search snippets of the report, not read in the PDF directly — [DORA Accelerate State of DevOps Report 2023 (PDF)](https://dora.dev/research/2023/dora-report/2023-dora-accelerate-state-of-devops-report.pdf)
- 2022 (OLDER DATA): lift to organizational performance from each technical capability with below-average versus above-average documentation: continuous delivery 63% vs 656%; continuous integration 34% vs 750%; loosely coupled teams 46% vs 313%; SRE 79% vs 343%; supply chain security 37% vs 451%; trunk-based development 36% vs 1525%; version control 27% vs 278%. Documentation quality was measured with eight metrics covering clarity, findability, and reliability — [DORA, Documentation quality capability](https://dora.dev/capabilities/documentation-quality/)
- DORA's stated obstacle: "Documentation needs to be actively created and maintained, which takes work" — [DORA, Documentation quality capability](https://dora.dev/capabilities/documentation-quality/)

GitHub Open Source Survey 2017 (GitHub with academic and industry collaborators; 5,500 randomly sampled respondents from over 3,800 GitHub repositories plus 500+ from a non-random sample of other communities). OLDER DATA (2017).

- "Incomplete or outdated documentation is a pervasive problem, observed by 93% of respondents", yet 60% of contributors rarely or never contribute to documentation — [Open Source Survey 2017](https://opensourcesurvey.org/2017/)
- About 25% of the open-source community reads and writes English less than "very well" — [Open Source Survey 2017](https://opensourcesurvey.org/2017/)

Postman State of the API 2025 (Postman; over 5,700 developers, architects, executives; 73% in engineering/software development; 43% Asia-Pacific, 30% North America)

- 93% of API teams face collaboration blockers. Inconsistent or outdated documentation is the top one at 55%; 34% cannot find existing APIs (discovery) — [Postman State of the API 2025](https://www.postman.com/state-of-api/2025/)
- The second-ranked blocker is reported inconsistently: one reading of the report page gives "duplicate efforts 43%"; a secondary summary gives "inconsistent definitions 43%" and "duplicated efforts 35%". Unresolved; check the PDF — [Postman State of the API 2025](https://www.postman.com/state-of-api/2025/); contradicted by [Medium summary of the 2025 report](https://medium.com/@pulasthinarada/api-strategy-is-ai-strategy-key-takeaways-from-the-2025-state-of-the-postman-api-report-5316ab2eb636)
- Postman frames the root cause as documentation scattered across chat, internal docs, email, and wikis. Seen in a secondary summary only — [Medium summary of the 2025 report](https://medium.com/@pulasthinarada/api-strategy-is-ai-strategy-key-takeaways-from-the-2025-state-of-the-postman-api-report-5316ab2eb636)

Academic

- Aghajani et al., "Software Documentation Issues Unveiled" (ICSE 2019): 878 documentation-related artifacts mined from mailing lists, Stack Overflow, issue trackers, and pull requests, yielding a taxonomy of 162 documentation issue types; common problems are outdated, incomplete, and inconsistent information. OLDER DATA (2019) — [ICSE 2019 listing](https://2019.icse-conferences.org/details/icse-2019-Technical-Papers/49/Software-Documentation-Issues-Unveiled)
- Aghajani et al., "Software Documentation: The Practitioners' Perspective" (ICSE 2020): two surveys with 146 practitioners in total; only a small subset of the 162 issue types are considered important by practitioners. OLDER DATA (2020) — [Paper PDF](https://homepages.dcc.ufmg.br/~figueiredo/disciplinas/papers/icse20aghajani.pdf)
- A 2026 arXiv paper, "Who Writes the Docs in SE 3.0? Agent vs. Human Documentation Pull Requests", exists and is directly relevant to AI-authored docs PRs. Not read; no figures extracted — [arXiv 2601.20171](https://arxiv.org/abs/2601.20171)

### Inferences
- Technical writers and engineers report different pain. Writers (Tom Johnson 2020): subject-matter depth, time, and getting engineers to review. Engineers and developers (State of Docs 2026, Postman 2025, GitHub 2017): docs that are stale, inconsistent, or hard to find.
- "Obtaining engineer reviews" is the only review-related pain point found in any survey, and it has no percentage attached. Review friction is plausible but weakly quantified.
- DORA measures the payoff of good documentation, not the pain of producing it. It supports a business case, not a pain-point ranking.

### Gaps
- Category percentages from the Aghajani taxonomy (share of issues about up-to-dateness, completeness, correctness, process, tools) and the per-issue importance ratings from the 2020 practitioner survey: both PDFs could not be parsed with the tools available. No numbers are reported here to avoid guessing.
- Stack Overflow 2024 knowledge-silo figures were confirmed only on Stack Overflow's product-marketing page, not on the survey results page itself.
- DORA 2024 and 2025 documentation findings were not read at source. A claim that a 25% increase in AI adoption is associated with a 7.5% increase in documentation quality (DORA 2024) was seen only on vendor blogs (Swimm, which sells a code-documentation tool: https://swimm.io/blog/heres-what-the-2024-dora-report-has-to-say-about-code-documentation). Unverified against the primary report.
- No ACM SIGDOC papers with survey data on docs-as-code were found in the searches run.
- Write the Docs has no tooling or pain-point survey that I could find; only the salary survey.
- GitHub Octoverse was not searched for documentation-specific figures.

## How widely adopted is docs-as-code, which tools are used, and how satisfied are users?

### Takeaway
Among people who write developer documentation, docs-as-code was already the majority approach in 2020 (56% follow it, 22% somewhat; 67% manage content in Git). In the broader 2026 documentation population, only 21% publish with open-source platforms or Git repos, against 45% on dedicated documentation tools. No survey found reports satisfaction scores or market share for Docusaurus, MkDocs, Sphinx, Hugo, Mintlify, GitBook, Fern, or ReadMe.

### Cited Findings
- 2020, developer-docs writers (n=405): 56% follow a docs-as-code approach, 22% somewhat, 20% do not. OLDER DATA (2020) — [Developer documentation trends, survey results](https://idratherbewriting.com/learnapidoc/docapis_trends.html)
- 2020: 67% manage content in Git, 8% in a CMS, 5% in a CCMS; source format Markdown 37%, HTML 15%, XML 15% — [Developer documentation trends, survey results](https://idratherbewriting.com/learnapidoc/docapis_trends.html)
- 2020: primary authoring tool is a static site generator (Jekyll, Hugo, Gatsby, Sphinx) for 22%, wikis 14%, XML tools 11%, help authoring tools 8%, FrameMaker 3%. Editors: VS Code 25%, Notepad++ 19%, Atom 14% — [Developer documentation trends, survey results](https://idratherbewriting.com/learnapidoc/docapis_trends.html)
- 2020: 48% publish with CI/CD, 33% do not, 15% plan to. Hosting: own infrastructure 31%, GitHub Pages 15%, GitLab 10% — [Developer documentation trends, survey results](https://idratherbewriting.com/learnapidoc/docapis_trends.html)
- 2020 raw tool mention counts (search snippet of the results slides, not verified on page): Confluence 46, Flare 35, Oxygen XML 34, Hugo 27, MS Word 26, Jekyll 25 — [Developer documentation trends, results slides](https://idratherbewriting.com/learnapidoc/slides/devdoctrends_results.html)
- 2026, all documentation roles (n=1,131): dedicated documentation tools 45%, open-source platforms / Git repos 21%, website publishing platforms 9% — [State of Docs 2026, Docs tooling](https://www.stateofdocs.com/2026/docs-tooling)
- 2025 (n=444): version control platforms (GitHub/GitLab) are "by far the most common" tool docs teams rely on; no percentage — [State of Docs 2025, Documentation tooling and API docs](https://www.stateofdocs.com/2025/documentation-tooling-and-api-docs)
- 2025: almost 74% of companies with APIs use OpenAPI (2020 Tom Johnson: 47% use OpenAPI specs) — [State of Docs 2025, Documentation tooling and API docs](https://www.stateofdocs.com/2025/documentation-tooling-and-api-docs); [Developer documentation trends, survey results](https://idratherbewriting.com/learnapidoc/docapis_trends.html)

### Inferences
- The two adoption figures (56 to 78% in 2020 versus 21% in 2026) measure different populations and different questions. Tom Johnson's audience skews to developer documentation and Write the Docs; State of Docs includes support, marketing, operations, and leadership, and is run by a hosted-platform vendor whose audience likely skews toward hosted tools.
- A defensible statement: docs-as-code is the norm for developer-facing documentation and a minority approach across documentation as a whole.
- The 2020 static site generator list (Jekyll, Hugo, Gatsby, Sphinx) predates the rise of Docusaurus, Mintlify, and Fern, so it says nothing about the current tool mix.

### Gaps
- No primary survey found with per-tool usage or satisfaction for Docusaurus, MkDocs, Sphinx, Hugo, Mintlify, GitBook, Fern, or ReadMe. State of Docs 2026's tooling chapter did not expose per-tool numbers in retrievable text.
- No post-2020 survey isolating docs-as-code practitioners was found. Tom Johnson's survey has not been repeated as far as the searches showed.
- No satisfaction or net-promoter-style data on docs-as-code workflows (Git, pull requests, CI builds) was found for technical writers.

## Is there quantified data on time lost, review turnaround, contributor drop-off, or documentation debt?

### Takeaway
Very little. The quantified figures are about developer time spent searching for answers (61% spend more than 30 minutes a day, 2024) and open-source contributor behavior (60% rarely or never contribute to docs, 2017). No survey found quantifies docs review turnaround, pull-request abandonment by docs contributors, or documentation debt.

### Cited Findings
- 61% of developers spend more than 30 minutes a day searching for answers or solutions; one in four spend at least 60 minutes (2024) — [Stack Overflow Developer Survey 2024, professional developers](https://survey.stackoverflow.co/2024/professional-developers)
- 53% agree or strongly agree that waiting on answers disrupts their workflow; 45% encounter knowledge silos frequently (2024; vendor marketing page for Stack Overflow's knowledge product) — [Stack Overflow, Insights from the 2024 Developer Survey](https://stackoverflow.co/internal/resources/your-developers-deserve-better-insights-from-the-2024-developer-survey/)
- 60% of open-source contributors rarely or never contribute to documentation, while 93% observe incomplete or outdated documentation (2017, OLDER DATA) — [Open Source Survey 2017](https://opensourcesurvey.org/2017/)
- 21% of open-source respondents who experienced or witnessed negative behavior stopped contributing to a project (2017; about negative interactions generally, not documentation workflow) — [Open Source Survey 2017](https://opensourcesurvey.org/2017/)
- 21% of documentation teams have no formal process for keeping docs updated (2026) — [State of Docs 2026, Docs and product](https://www.stateofdocs.com/2026/docs-and-product)
- 40% say engineers handle API documentation alone, without technical writers (2025) — [State of Docs 2025, Documentation tooling and API docs](https://www.stateofdocs.com/2025/documentation-tooling-and-api-docs)
- "Documentation debt" appears as a qualitative label for the phase where "engineering teams grow faster than docs teams"; no measurement (2025) — [State of Docs 2025, Documentation team structure](https://www.stateofdocs.com/2025/documentation-team-structure)
- "Obtaining engineer reviews" is listed among writers' biggest challenges, without a percentage (2020) — [Developer documentation trends, survey results](https://idratherbewriting.com/learnapidoc/docapis_trends.html)
- Time saved, the inverse metric: 78% say AI makes documentation faster and 35% claim time savings of 50% or more; technical writers report among the smallest gains, with 31% at 50% or more (2026) — [State of Docs 2026, AI and documentation creation](https://www.stateofdocs.com/2026/ai-and-documentation-creation)

### Inferences
- The Stack Overflow search-time figure measures time looking for answers in general, not time lost to bad documentation. Using it as a documentation-cost figure overstates what it shows.
- The absence of review-turnaround and contributor drop-off data is itself a finding: these are commonly asserted docs-as-code pain points with no survey quantification behind them.

### Gaps
- No primary figures found for: docs pull-request review turnaround, share of docs PRs abandoned, hours per week writers spend on tooling or build problems, or a measured size of documentation debt.
- Aggregator pages surfaced in search (rockstardeveloperuniversity.com, happysupport.ai, docsio.co, fastdoc.io, rasepi.com) compile documentation statistics. None were used; their figures were not traced to primary sources.

## How has AI changed the pain points in 2025 and 2026?

### Takeaway
AI use in documentation went from 60% (2025) to 76% (2026), and the work has moved from drafting to checking: 43% spend more time fact-checking and 43% more time editing AI output, while hallucination is the top concern at 62%. Docs are now also read by machines: 18% report users reaching docs through coding assistants and 16% through MCP servers, and 70% factor AI into information architecture (the report compares this to 31% the year before).

### Cited Findings

Adoption and the shift from writing to reviewing

- 2025: 60% use generative AI in documentation workflows, 31% often; 25 to 30% do not use AI at all (both figures appear in the chapter) — [State of Docs 2025, AI and the future of documentation](https://www.stateofdocs.com/2025/ai-and-the-future-of-documentation)
- 2026: 76% use AI regularly (always, often, or occasionally) for documentation creation; 11% never; 75% expect to increase usage; 64% believe AI will be "extremely impactful" — [State of Docs 2026, AI and documentation creation](https://www.stateofdocs.com/2026/ai-and-documentation-creation)
- 2026 use cases: drafting 62%, brainstorming 58%, proofreading 58%, style guide matching 50%, writing complete documents 25%. Tools: general-purpose LLMs 73%, code assistants 39%, documentation-specific AI tools 26% — [State of Docs 2026, AI and documentation creation](https://www.stateofdocs.com/2026/ai-and-documentation-creation)
- 2026: 56% of heavy AI users report "less writing, more editing" versus 10% of non-users. (The landing page states this as 56% of regular AI users.) — [State of Docs 2026, Docs and professional development](https://www.stateofdocs.com/2026/docs-and-professional-development)
- 2026: 43% spend more time on fact-checking and validation; 43% spend more time editing AI-generated content; 27% spend more time communicating with the wider team. Time freed: first drafts 44%, formatting/styling 35% — [State of Docs 2026, Docs and professional development](https://www.stateofdocs.com/2026/docs-and-professional-development)
- 2026: 33% report doing more documentation work than before; 28% say AI changed their day-to-day significantly; 26% report minimal role change — [State of Docs 2026, Docs and professional development](https://www.stateofdocs.com/2026/docs-and-professional-development)
- 2026 new skills needed: AI/prompt engineering 50%, information architecture 38%, content strategy 36%, developer tools 35%, API knowledge 25%, coding 24%, none 9% — [State of Docs 2026, Docs and professional development](https://www.stateofdocs.com/2026/docs-and-professional-development)
- 2026: 62% cite hallucinations as their primary AI concern — [State of Docs 2026, AI and documentation creation](https://www.stateofdocs.com/2026/ai-and-documentation-creation)
- 2026 governance: 44% have formal or informal AI guidelines; 22% have no plans to create any; among "always" users 55% have guidelines and 15% have none and no plans — [State of Docs 2026, AI and documentation creation](https://www.stateofdocs.com/2026/ai-and-documentation-creation)
- Developer-side parallel (Stack Overflow 2025, 49,000+ developers): 45% name "AI solutions that are almost right, but not quite" as their top frustration; 66% say they spend more time fixing almost-right AI-generated code; 46% do not trust the accuracy of AI output (31% the year before); 3% "highly trust" it. These are about code, not docs — [Stack Overflow press release, 2025 Developer Survey](https://stackoverflow.co/company/press/archive/stack-overflow-2025-developer-survey/)

Docs read by AI and agents

- 2026: users reach docs via AI-powered search 35%, coding AI assistants 18%, MCP servers 16% — [State of Docs 2026, Docs and product](https://www.stateofdocs.com/2026/docs-and-product)
- 2026: 70% factor AI into information architecture decisions (landing page: up from 31%); 59% say teams should consider AI/LLMs when formatting docs — [State of Docs 2026, landing page](https://www.stateofdocs.com/2026); [State of Docs 2026, AI and documentation consumption](https://www.stateofdocs.com/2026/ai-and-documentation-consumption)
- 2026 optimization for AI readers: 30% add explicit context, 30% make pages self-contained, 26% invest in structured data — [State of Docs 2026, AI and documentation consumption](https://www.stateofdocs.com/2026/ai-and-documentation-consumption)
- 2026 shipped AI features: conversational AI interface 38%, AI-enhanced search 36%, none 26% (41% of organizations without a formal docs team have shipped none, versus 19 to 21% with centralized or hybrid teams). Planned: chatbots 32%, AI search 29%, MCP servers 25%, no plans 15% — [State of Docs 2026, AI and documentation consumption](https://www.stateofdocs.com/2026/ai-and-documentation-consumption)
- 2026: 67% say AI has made their docs better (40% slightly, 27% much), 7% worse; 47% of those with AI features agree users find information faster — [State of Docs 2026, AI and documentation consumption](https://www.stateofdocs.com/2026/ai-and-documentation-consumption)
- 2026 security: 56% comfortable with external AI integrations, 16% blocked or very cautious; top concerns data privacy 51%, compliance 36%, data retention 35% — [State of Docs 2026, AI and documentation consumption](https://www.stateofdocs.com/2026/ai-and-documentation-consumption)
- 2026 year-over-year items reported in the consumption chapter: context-aware assistance 51% to 62%; chat-based assistance 51% to 59%; AI as primary creation tool 19% to 35%; 41% believe docs should adapt to users in real time. The question wording behind these was not retrievable — [State of Docs 2026, AI and documentation consumption](https://www.stateofdocs.com/2026/ai-and-documentation-consumption)
- 2025: 87% believe AI will be at least somewhat impactful (nearly half "huge"); 42% expect docs to intelligently adapt to user needs; 25% believe docs will be written mainly for AI and LLMs to read — [State of Docs 2025, AI and the future of documentation](https://www.stateofdocs.com/2025/ai-and-the-future-of-documentation)
- Postman 2025: 24% actively design APIs with AI agents in mind, 13% design equally for humans and machines, 60% design primarily for humans; 89% of developers use generative AI tools — [Postman State of the API 2025](https://www.postman.com/state-of-api/2025/)

### Inferences
- The bottleneck has moved from producing text to verifying it. Fact-checking and editing AI output (43% each) are now larger time sinks than they were, and hallucination (62%) is the leading worry. Review tooling matters more, not less.
- Writers gain least from AI on speed (31% at 50% or more savings versus 35% overall), consistent with their work being verification-heavy.
- Sync remains the top workflow challenge in 2026 despite 76% AI adoption, which suggests AI drafting has not solved drift.
- Machine readers are a real but still minority channel (16 to 18%), with planned investment (MCP servers 25%) running ahead of current use.

### Gaps
- No data found on how AI-generated docs pull requests affect review load or quality, apart from the unread arXiv paper noted above.
- No figures found on llms.txt adoption specifically.
- The 31%-to-70% year-over-year IA comparison comes from the landing page; the 2025 chapter text read here did not expose a 31% figure, so the baseline is unverified.

## How do the pain points rank by weight of evidence?

### Takeaway
Stale or out-of-sync documentation is the only pain point confirmed by multiple independent surveys with large samples. Everything specific to the docs-as-code workflow itself (Git friction, pull-request review, build tooling) is either unquantified or rests on one 2020 survey.

### Cited Findings
Ranked from strongest to weakest evidence:

1. Docs out of date or out of sync with the product. 30% single biggest workflow challenge, 43% of engineers, 26% of writers (2026, n=1,131); 56% of API-docs teams (2025, n=444); 55% of API teams cite inconsistent or outdated docs (2025, n=5,700+); 93% observe incomplete or outdated docs (2017, n=5,500+) — [State of Docs 2026](https://www.stateofdocs.com/2026/docs-and-product); [State of Docs 2025](https://www.stateofdocs.com/2025/documentation-tooling-and-api-docs); [Postman 2025](https://www.postman.com/state-of-api/2025/); [Open Source Survey 2017](https://opensourcesurvey.org/2017/)
2. Verifying AI output. 62% name hallucinations as top concern; 43% spend more time fact-checking; 43% more time editing AI content (2026) — [State of Docs 2026, AI and documentation creation](https://www.stateofdocs.com/2026/ai-and-documentation-creation); [State of Docs 2026, Docs and professional development](https://www.stateofdocs.com/2026/docs-and-professional-development)
3. No process or ownership. 21% have no formal update process; 22% have no formal docs team; 40% of API docs written by engineers alone; 60% of open-source contributors rarely or never touch docs — [State of Docs 2026, Docs and product](https://www.stateofdocs.com/2026/docs-and-product); [State of Docs 2026, Docs team structure](https://www.stateofdocs.com/2026/docs-team-structure); [State of Docs 2025](https://www.stateofdocs.com/2025/documentation-tooling-and-api-docs); [Open Source Survey 2017](https://opensourcesurvey.org/2017/)
4. Scattered and hard-to-find docs. About 25% have docs spread across multiple platforms (2025); 34% cannot find existing APIs (2025); 45% hit knowledge silos frequently (2024) — [State of Docs 2025](https://www.stateofdocs.com/2025/documentation-tooling-and-api-docs); [Postman 2025](https://www.postman.com/state-of-api/2025/); [Stack Overflow 2024 insights](https://stackoverflow.co/internal/resources/your-developers-deserve-better-insights-from-the-2024-developer-survey/)
5. Inability to measure docs value. 25% track no metrics; 57% do not track leads; 11 to 12% track business outcomes (2026) — [State of Docs 2026, Measuring docs success](https://www.stateofdocs.com/2026/measuring-docs-success)
6. Writer-specific: technical depth, lack of time, getting engineer reviews. Named as biggest challenges, no percentages (2020, n=405) — [Developer documentation trends, survey results](https://idratherbewriting.com/learnapidoc/docapis_trends.html)
7. Structure without a framework. 53% rely on self-created guidelines, 32% on intuition (2026); 77% homegrown methods (2025) — [State of Docs 2026, Docs tooling](https://www.stateofdocs.com/2026/docs-tooling); [State of Docs 2025](https://www.stateofdocs.com/2025/documentation-tooling-and-api-docs)

### Inferences
- Items 1 to 5 are about documentation in general. None is specific to the docs-as-code workflow, and docs-as-code users are not broken out in any of them.
- The frequently repeated claims that Git and Markdown are a barrier for non-engineer contributors, and that pull-request review is slow for docs, have no survey quantification in the sources found. They should be presented as practitioner opinion, not data.
- The engineer/writer split in item 1 is the only role-separated pain-point figure found in recent data.

### Gaps
- No survey found that ranks pain points specifically among docs-as-code users.
- No quantified evidence found for tooling-level complaints (build failures, broken links, preview environments, merge conflicts, Markdown/MDX syntax errors).
- The runner-up workflow challenge in State of Docs 2026 remains unidentified.
