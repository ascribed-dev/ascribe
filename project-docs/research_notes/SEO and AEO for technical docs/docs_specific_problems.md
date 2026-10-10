# Search and discoverability problems specific to technical documentation

Researched 2026-10-10. How to read the citations:

- **[opened]** means I fetched the page and read the claim there.
- **[summary]** means the claim comes from a search-result summary of that URL, and I did not open the page. Treat these as leads to confirm.
- Dates are the date of the source. A practice described from a 2010s mailing list is historical unless a later source confirms it.
- "No source found" in a Gaps list means I searched and found nothing citable. It does not mean nothing exists.

The largest caveat on the whole file: first-person accounts with measured outcomes are rare. Most of what docs teams have published is the problem and the remedy, with no before-and-after number. Where a result is reported, I say what kind (a number, internal feedback, or one person's check).

## Versioned docs in search: old versions outranking current ones, wrong-version landings, duplicates

### Takeaway
Every large project with versioned docs has documented old versions outranking current ones, and the same four remedies recur (canonical to current, `noindex` or robots.txt on old versions, a banner, a `/current/` or `/stable/` alias). The documented mistakes are more instructive than the successes: canonicals that point at pages that don't exist, canonicals that make supported old versions unfindable, and build steps that overwrite the canonical. Almost nobody published a measured result.

### Cited Findings

**PostgreSQL (2010 to 2021): the best-documented case, including a failure**
- 2010, pgsql-docs: a user reports that older versions "always" come up before the latest, because Google ranks by inbound links and old pages have more; the workaround is to replace the version in the URL with `current`. [summary] — [postgresql.org message 4C4A0DEB](https://www.postgresql.org/message-id/4C4A0DEB.9060501@emolecules.com)
- 2017, pgsql-docs: Conor McNally asks why there is no canonical URL, since a search like "postgres select syntax" returns "a pretty random selection of duplicate pages". Magnus Hagander objects that the versions are not duplicates and that a canonical to current would make old versions unsearchable. [summary] — [postgrespro.com thread 2329723](https://www.postgrespro.com/list/thread-id/2329723)
- 2019-05-12, pgsql-www, "patch to point search engines at current version of docs": Adam Sah reports, "I've confirmed that rel=canonical will make it impossible to find old versions in Google." Jonathan Katz notes the drawback that "this could end up causing pages that existed in older, but supported, versions to go missing from search engines." Alternatives listed: a browser extension, a keyboard shortcut; a cookie-based redirect is marked not recommended because Google may penalize it. [opened] — [postgresql.org message CALWCfd+nj](https://postgresql.org/message-id/CALWCfd%2Bnj%3D4j%3DhWdd_EgFhTX%3D8ZgEbUWZrQCnJU5h-N-xFMMCw%40mail.gmail.com)
- 2020-11-18, pgsql-www, "Fixing Google Search on the docs (redux)": Dave Page reports that "the vast majority of inbound traffic to the docs, hits the 9.1 version" (no figure given; 9.1 reached end of life in 2016), and that the team had "tried various remedies, clearly none of which are working." He proposes a trial of a couple of months blocking versioned docs URLs in robots.txt. [opened] — [postgresql.org message CA+OCxoyVwmmZ](https://postgresql.org/message-id/CA%2BOCxoyVwmmZkWUJCez2hCqa89iGv%3Dvq58NF1yQkTg9gtpkn%3Dg%40mail.gmail.com)
- 2020-11-19, same thread: Dave Page's revised proposal is three rules: remove all docs URLs not under `/current/` from the sitemap; put a canonical on every docs page pointing to the `/current/` version; "where a page has been removed entirely, mark the most recent version of it as the canonical one instead." [opened] — [postgresql.org message CA+OCxoxet](https://postgresql.org/message-id/CA%2BOCxoxet%2BzWmWB5b2sLjvUHwNngPnyu8STQ4frYoMtY--MNMA%40mail.gmail.com)
- Same thread: Andres Freund says Google no longer honors sitemap priority hints, and suggests not applying a canonical where no newer version of the page exists. [summary] — [postgresql.org message 20201119195033](https://www.postgresql.org/message-id/20201119195033.wuuhzza7tm2jtzuu%40alap3.anarazel.de)
- Outcome: a third-party browser add-on's description says "in 2021 a huge improvement was made" so that search engines treat `current` as canonical. This is not a PostgreSQL source and I did not confirm it on postgresql.org. [summary] — [pg_docs_bot listing](https://firefox.marketplace.browsertotal.com/ko/firefox/addon/pg_docs_bot/)
- Unsupported versions carry a banner: "This documentation is for an unsupported version of PostgreSQL," with a link to the same page in the current version. [summary] — [postgresql.org/docs/7.3](https://www.postgresql.org/docs/7.3/index.html)

**Python (2013 to 2022)**
- 2013 and 2019: list posts report that Google leads to Python 2 pages first ("anytime I do a google search I notice that the first result is from Python 2 docs", July 2019). [summary] — [docs list, 2013](https://www.python.org/pipermail/docs/2013-July/015195.html); [python-list, 2019](https://www.python.org/pipermail/python-list/2019-July/892113.html)
- Issue 26355 (2017) proposed canonical URLs of the form `/3/<path>`. The stated obstacle: at build time Sphinx cannot know whether the page exists in the target version. [summary] — [docs list, issue26355](https://mail.python.org/pipermail/docs/2017-February/029941.html)
- Issue 35435 (December 2018): maintainers preferred keeping old docs online with a link to the new page over hiding them ("hiding old pages from search engines is lying to them"), and proposed rebuilding old releases so they carry a canonical from `/3.4/` to `/3/`, already done for 3.5. [summary] — [docs list, issue35435](https://mail.python.org/pipermail/docs/2018-December/038477.html)
- Documented mistake, 2021-10-04 to 2022-05-29: the 3.11 docs declared a canonical under `/3/`, which then pointed at 3.9, so the canonical for the 3.11 "What's New" page was a 404. Reporter: "search engines and social networking link scrapers redirect to canonical links by default." Fix (python/docsbuild-scripts@40ce6d1, 2022-05-23): the build knows which version `/3/` is and omits the canonical for versions newer than it. [opened] — [python/pythondotorg #1880](https://github.com/python/pythondotorg/issues/1880)
- End-of-life versions show a red banner; one final rebuild is triggered when a version becomes unsupported so the "outdated" warning is added. [summary] — [python/docsbuild-scripts #216](https://github.com/python/docsbuild-scripts/issues/216); [python/cpython #120795](https://github.com/python/cpython/issues/120795)
- Issue 42486 (2020): a user reports "python shuffle list" does not show `docs.python.org/3/library/random.html` on the first page of Google; closed as out of scope. [summary] — [docs list, issue42486](https://mail.python.org/archives/list/docs@python.org/thread/62DFNQOY7BVK6MTHHHYD5AYL66MW2MAK/)

**Django**
- As of 2022-01-20, Django's older pages declare the newest release as canonical: the 2.2 page carried `<link rel="canonical" href="https://docs.djangoproject.com/en/4.0/topics/db/">`. Simon Willison checked this by hand; no ranking result reported. [opened] — [til.simonwillison.net](https://til.simonwillison.net/readthedocs/documentation-seo-canonical)
- PostgreSQL's 2020 thread cites Django as the model, with the observation that quick test searches surfaced Django 3.1 pages first. That is one person's spot check, not a measurement. [summary] — [postgresql.org thread](https://postgresql.org/message-id/CA%2BOCxoyVwmmZkWUJCez2hCqa89iGv%3Dvq58NF1yQkTg9gtpkn%3Dg%40mail.gmail.com)
- Ticket #18922: development-version docs in search results confuse newcomers, who then file invalid bugs against the version they have installed. [summary] — [Django ticket 18922](https://code.djangoproject.com/ticket/18922)
- Ticket #22273 (wontfix): docs for unsupported versions are deliberately not published; a reader builds them from source. [summary] — [Django ticket 22273](https://code.djangoproject.com/ticket/22273)

**Read the Docs**
- Read the Docs sets a canonical on every version pointing to the project's default version (usually `latest` or `stable`) on the primary domain; the Datasette 0.50 page carried a canonical to `/en/stable/`. [opened] — [til.simonwillison.net](https://til.simonwillison.net/readthedocs/documentation-seo-canonical)
- Its SEO guide advises robots.txt to keep unsupported versions out of search while leaving them online, redirects for moved pages across all versions, and updating internal links as well as adding redirects; it warns that "Google and Bing will show pages that were previously indexed that now give a 404." The guide gives no measured results. Advice only. [opened] — [Read the Docs SEO guide](https://docs.readthedocs.com/platform/stable/guides/technical-docs-seo-guide.html)
- Its canonical guide calls the canonical "not a perfect solution". [summary] — [Read the Docs canonical guide (mirror)](https://readthedoc-test00.readthedocs.io/en/latest/guides/canonical.html)

**GitLab**
- Issue #915 asks for `noindex` on all archived versions: "We need to stop google from indexing old docs." The page I read shows no dates and no outcome. [opened] — [gitlab-docs #915](https://gitlab.com/gitlab-org/gitlab-docs/-/issues/915)
- Documented mistake, issue #1568: a post-build find-and-replace that rewrote URLs for each versioned site also rewrote every canonical URL, so canonicals on versioned docs were wrong. [summary] — [gitlab-docs #1568](https://gitlab.com/gitlab-org/gitlab-docs/-/issues/1568)
- Documented mistake, issue #1132: each versioned path served its own robots.txt, but only the one at the domain root counts; a rule excluding versions had been added and later removed. [summary] — [gitlab-docs #1132](https://gitlab.com/gitlab-org/gitlab-docs/-/issues/1132)
- Epic 8937 planned a separate archives site on a subdomain for unsupported versions. [summary] — [gitlab-org epic 8937](https://gitlab.com/groups/gitlab-org/-/epics/8937)

**Camunda: `noindex` plus canonical interaction, with a reported fix**
- December 2021: users report that the docs search (powered by Google) and Google findability had regressed. October 2022 root cause: "Google declared canonicals from versions we remove from indexing," which made those pages nearly unfindable. June 2023: after adding a proper sitemap for the latest version and requesting a recrawl, the team reports, on internal feedback from Support, that "both embedded search and search from Google into the Camunda Platform 7 docs works as intended." No metrics. Elapsed time from report to reported fix: about 18 months. [opened] — [Camunda forum thread 32044](https://forum.camunda.io/t/camunda-docs-repo-search-is-not-working-as-expected/32044)

**OpenStack**
- 2023-11-03: a search for a Keystone config option returns a Pike (2017) page first. The unsupported-release banner links to the docs root, not to the same page in the current release; the author hand-edits the URL to `latest`, "which works sometimes." Proposed: `noindex` or removing old docs. The replies were not visible to me. [opened] — [openstack-discuss thread](https://lists.openstack.org/archives/list/openstack-discuss@lists.openstack.org/thread/Q3AJ5C54IZSGBJ76JTVXTWVTFNXDRD7N/)

**Kubernetes**
- The site publishes the current version and the four previous ones; older versions live on `vX-YY.docs.kubernetes.io` as static snapshots with a banner saying the version is no longer maintained. [summary] — [v1-32 docs](https://v1-32.docs.kubernetes.io/docs/home/supported-doc-versions/)
- A maintainer on the Kubernetes forum says the v1.20 and v1.21 sites are no longer published and what remains is "a cached artifact that hasn't been cleaned up yet." [summary] — [discuss.kubernetes.io thread 24250](https://discuss.kubernetes.io/t/https-v1-21-docs-kubernetes-io-and-https-v1-20-docs-kubernetes-io-certificate-issue/24250)

**React, Angular, Docusaurus, Elastic, Microsoft**
- React (March 2023) moved the old docs to `legacy.reactjs.org` and redirected existing links there; legacy pages carry a "no longer updated" banner. A later issue reports a link that redirects to a deprecated legacy page with no replacement on react.dev. [summary] — [react.dev launch post (ja)](https://ja.react.dev/blog/2023/03/16/introducing-react-dev); [create-react-app #13537](https://redirect.github.com/facebook/create-react-app/issues/13537)
- Angular's old site says it is the archived documentation for v17 and points to angular.dev. A Cursor forum thread reports an AI tool still using angular.io links. [summary] — [angular.io/docs](https://angular.io/docs); [Cursor forum thread 46222](https://forum.cursor.com/t/angular-still-using-old-angular-io-docs-and-links/46222)
- Docusaurus's versioning page offers an `unmaintained` banner and advises keeping versions "below 10". On 2026-10-10 that page says nothing about `noindex` or canonicals. [opened] — [Docusaurus versioning](https://docusaurus.io/docs/versioning)
- Meshtastic documents a Docusaurus trap: the version-copy command copies pages verbatim, so root-absolute links in a frozen version still resolve to the live version; they rewrite links in each snapshot and exclude snapshots from the sitemap. [summary] — [Meshtastic docs versioning](https://meshtastic.org/docs/development/documentation/versioning/)
- Elastic's 2025 docs rebuild keeps 8.19 and earlier at `elastic.co/guide` and maps legacy URL patterns to versions in a `legacy-url-mappings.yml` file; a redirect's source and target must both be in the new docs. [summary] — [Elastic docs-builder legacy URL mappings](https://docs-v3-preview.elastic.dev/elastic/docs-builder/pull/3826/documentation/catalog/legacy-url-mappings.md)
- Microsoft, historical: a 2010 MSDN post says a version selector was added because people were "confused by finding help topics for the wrong version of VS and not realizing it." [summary] — [devblogs.microsoft.com](https://devblogs.microsoft.com/bharry/new-msdn-version-selector/)

### Inferences
- The failures all come from one missing fact at build time: whether the same page exists in the target version. Python's 404 canonical, PostgreSQL's "mark the most recent version of it as the canonical one," and OpenStack's banner that links to the root are the same gap. A toolchain that knows every page's versions can compute, per page, "newest version in which this page exists" and use it for the canonical and for the banner link.
- Canonical-to-current and "old versions stay findable" conflict. PostgreSQL confirmed the first destroys the second. A build can make the choice explicit per version (supported versions self-canonical, unsupported ones canonical to current or `noindex`), which is what Python ended up near.
- `noindex` and canonical should never be set on the same page pointing in different directions (Camunda), and canonicals should be checked after every post-build rewrite (GitLab). Both are detectable in the built output.
- Version-relative links inside a frozen version (Meshtastic) are a build-time check: a link in version N should resolve inside version N.

### Gaps
- No project published a before-and-after number for any remedy. PostgreSQL's "vast majority hits 9.1" is the only analytics statement, and it has no figure.
- I could not confirm at a primary source what PostgreSQL shipped in 2021, whether GitLab shipped `noindex` on archives, or the current canonical behaviour of Kubernetes, MongoDB, Docker, Microsoft Learn, Angular, or React. MongoDB and Docker: no source found.
- Microsoft Learn's `?view=` moniker and its canonical handling: no primary source found.

## How developers reach documentation, and the shift from search to AI (2023 to 2026)

### Takeaway
Surveys still put documentation first as a learning resource, but by 2026 two docs hosts report that AI agents make about as many requests as browsers, and one framework reports docs traffic down about 40% since early 2023. The traffic figures are vendor samples that count different things and should not be added together.

### Cited Findings
- Stack Overflow Developer Survey 2025 (over 49,000 responses, 177 countries): 84% use or plan to use AI tools (76% the year before); 47.1% use them daily; 3.1% "highly trust" AI output and 46% distrust it; 66% name "AI solutions that are almost right, but not quite" as a frustration. [opened] — [survey.stackoverflow.co/2025](https://survey.stackoverflow.co/2025/)
- Same survey, from the press release: top resources for learning to code are technical documentation (68%), online resources (59%), and Stack Overflow (51%), all lower than the year before; AI tools for learning rose to 44% from 37%. [summary] — [Stack Overflow press release](https://stackoverflow.co/company/press/archive/stack-overflow-2025-developer-survey/)
- Stack Overflow question volume: about 3,862 questions in December 2025, a 78% drop on the year (DevClass, January 2026); about 1,304 in July 2026 against 207,000 in March 2014. Sources conflict on the December figures (36Kr gives 3,312 for December 2025 and 17,935 for December 2024). [summary] — [DevClass](https://devclass.com/2026/01/05/dramatic-drop-in-stack-overflow-questions-as-devs-look-elsewhere-for-help); [36Kr](https://eu.36kr.com/en/p/3940270853110913); [KuCoin news](https://www.kucoin.com/news/flash/stack-overflow-s-monthly-questions-drop-to-1-304-in-july-2026-down-99-from-2014)
- The decline began before generative AI; one dashboard notes the measure is questions posted to one site, not all information seeking. [summary] — [Dror Poleg dashboard](https://data.drorpoleg.com/charts/stack-overflow-questions/)
- Tailwind, January 2026: Adam Wathan wrote in a GitHub comment that "Traffic to our docs is down about 40% from early 2023 despite Tailwind being more popular than ever," that revenue was down almost 80%, and that three of four engineers were laid off; the docs were the only route to the paid products. The comment was on a pull request proposing an `/llms.txt` endpoint (opened 2025-11-18), which he declined. I read this through news coverage, not the comment itself. [summary] — [DevClass](https://www.devclass.com/ai-ml/2026/01/08/tailwind-labs-lays-off-75-percent-of-engineers-thanks-to-brutal-impact-of-ai/4079571); [eWeek](https://www.eweek.com/news/tailwind-labs-lays-off-engineers-due-to-ai/)
- Mintlify, published 2026-04-03, 30 days of Cloudflare data across Mintlify-hosted sites, about 790 million requests classified by user agent: AI agents 45.3% (357.6M), browsers 45.8% (361.7M). Claude Code 199.4M and Cursor 142.3M are 95.6% of identified agent traffic; ChatGPT 1.8M. Codex sends no identifying user agent, so the agent share is undercounted. Vendor data; number of sites not stated. [opened] — [Mintlify, state of agent traffic](https://www.mintlify.com/blog/state-of-ai)
- Mintlify's midyear report says agents are 66% of measured traffic. [summary] — [Mintlify, state of docs traffic](https://index.mintlify.com/blog/state-of-docs-traffic)
- GitBook, one week (2026-04-27 to 2026-05-03), sites with more than 100 page views in April, 61.2M page views: humans 36.2%, AI agents 38.9%, crawlers 24.1%. Of agent traffic, ChatGPT is 54% and coding agents combined are under 7%; about 22% is unidentified. AI share of readers was under 10% in January 2025 and about 41% in December 2025. Vendor data. [opened] — [GitBook, April 2026](https://gitbook.com/blog/ai-docs-data-april-2026)
- The two hosts disagree on who the agents are: coding agents dominate on Mintlify (95.6%), ChatGPT dominates on GitBook (54%). [opened] — [Mintlify](https://www.mintlify.com/blog/state-of-ai); [GitBook](https://gitbook.com/blog/ai-docs-data-april-2026)
- State of Docs 2026 (GitBook, byline 2026-03-18, over 1,100 respondents; a survey of docs practitioners, not readers): 70% of teams factor AI into information architecture decisions; 57% don't track leads from docs. [opened] — [GitBook, State of Docs 2026](https://www.gitbook.com/blog/state-of-docs-2026)
- Secondhand from that report, not confirmed at the report: direct navigation 66%, in-product links 54%, AI search 35% (46% at enterprise companies) as ways readers reach docs. [summary] — [docsio.co](https://docsio.co/blog/how-to-write-documentation)

### Inferences
- "Traffic" now measures two different readers. A page view from a browser and a fetch from a coding agent are both demand for the page, but only the first can see a banner, a tab, or a version picker.
- Agents arrive at a URL someone or something gave them, often from training data. That makes stale URLs and old-version URLs more durable than they were with search alone (the Angular and Cursor report above is one instance).
- Tailwind's account shows that discovery of anything other than the answer (a product, an upgrade notice, a deprecation) cannot depend on a human visiting the page.

### Gaps
- No MDN traffic data for 2025 or 2026 found. No JetBrains or SlashData figure on docs discovery channels found. No published Google research on how developers reach its docs found.
- I did not open the State of Docs report itself; the channel percentages are unconfirmed.
- No docs site has published search-referral versus AI-referral numbers over time. GitBook's post has no referral data.

## Error messages and codes as queries

### Takeaway
The evidence that readers search for exact error strings is mostly indirect: teams that built error indexes say so, and browser and framework vendors designed links from the error into the docs to shorten the loop. A stable code with a page or anchor per code is the consistent practice; I found no measured result for it.

### Cited Findings
- Mozilla added a "Learn More" link to console errors (fixed in Firefox 49, 2016) with the rationale that "often the message gets copied, put into google, and answered on Stackoverflow." [summary] — [Mozilla bug 1179876](https://bugzilla.mozilla.org/show_bug.cgi?id=1179876)
- An academic survey found explanations of exceptions and error messages among developers' most frequent web search tasks. I did not open the paper. [summary] — [ANU repository, "What do developers search for on the web?"](https://openresearch-repository.anu.edu.au/items/9e8a059f-5f85-4938-a896-8e57d68012b6)
- Rust: one index page lists every compiler error code; each code's explanation is a file `error_codes/EXXXX.md` in a format set by RFC 1567; removed codes are kept with a note, never deleted; `rustc --explain` prints the same text offline. [summary] — [Rust error codes index](https://doc.rust-lang.org/error_codes); [rustc_error_codes source](https://doc.rust-lang.org/beta/nightly-rustc/src/rustc_error_codes/lib.rs.html)
- React: the production build strips messages and prints a link; each code has a page (for example `react.dev/errors/481`) that shows the full text. [summary] — [react.dev/errors](https://react.dev/errors)
- Angular (v11.1, early 2021) added `NG`-prefixed codes so framework errors are distinguishable from TypeScript and browser errors, with runtime codes starting with 0 and compiler codes 1 to 9, and an index of pages per code. The stated aim was searchability. The set of codes differs between versions (the v18 index lacks entries on the current one). [summary] — [Angular blog](https://blog.angular.dev/angular-debugging-guides-dfe0ef915036) (returned 403 to me); [InfoWorld](https://www.infoworld.com/article/2262101/angular-adds-error-codes-debugging-guides.html); [angular.dev/errors](https://angular.dev/errors)
- Scala 3 opened a discussion proposing "a rust-style error index", evidence the pattern is copied. [summary] — [scala/scala3 discussion 18679](https://github.com/scala/scala3/discussions/18679)

### Inferences
- A code is a better search key than the message: it survives rewording, minification, and translation. The page for a code should still contain the exact message text, because readers who paste the message include it.
- "Never delete a code" (Rust) is the error-page form of "URLs don't move". A toolchain that owns the diagnostics list can generate one page or anchor per code, fail the build when a code's page disappears, and keep the code's page across versions even when the code is retired.
- Error codes vary by version (Angular). An error page needs the versions the code applies to, which a version-aware build already has.

### Gaps
- No study found that measures the share of developer queries that are pasted error strings.
- No published results from Stripe, AWS, Microsoft, Kubernetes, or TypeScript on error pages and search. No source found for each.
- Next.js's `/docs/messages` pages: not confirmed.

## Reference documentation

### Takeaway
I found advice and product defaults, not evidence. Nobody has published how a one-page reference performs against page-per-endpoint in search or in AI answers.

### Cited Findings
- Fern renders an API reference as one long page by default and offers `paginated: true` for a page per endpoint. Product documentation, no outcome data. [summary] — [fern-api/docs pull 415](https://github.com/fern-api/docs/pull/415/files)
- GitBook recommends a page per operation for larger APIs. Advice. [summary] — [GitBook OpenAPI layouts](https://gitbook.com/docs/api-references/guides/openapi-layouts)
- Redocly's advice: client-rendered reference sites depend on the crawler running JavaScript, which "is not perfect"; sitemaps matter because docs hierarchies are deep. Vendor advice. [summary] — [Redocly, SEO best practices](https://redocly.com/blog/seo-best-practices-documentation)
- An agent-readiness audit flags pages whose Markdown exceeds 50,000 characters as likely to be truncated or summarized by agent platforms, and pages over 100,000 as failing. Third-party audit tool, early development (spec v0.1.0, 2026-02-22). [summary] — [afdocs on Socket](https://socket.dev/npm/package/afdocs/overview/0.4.0); [Mintlify score report](https://www.mintlify.com/score/tabstack.md)
- The same audit found pages where the content starts past 50% of the converted output, so "agents may never see the documentation content." [summary] — [Mintlify score report](https://www.mintlify.com/score/tabstack.md)

### Inferences
- Page size is the one reference-page property with a published threshold, and it comes from agents, not search engines. A build can report each page's rendered Markdown size and warn above a limit.
- A fragment (`#symbol`) is not a separate indexable URL. One long page gives every symbol an anchor but one search result; page-per-symbol gives each a result but risks thin near-duplicates. I found no data that settles it.

### Gaps
- No first-person account of switching reference layout with a result. No data on how generated reference pages are indexed or cited. No source found.

## Variants: tabs, collapsed sections, code-sample switchers

### Takeaway
The clearest published evidence is on the agent side: converting a tabbed page to Markdown naively loses or garbles the hidden panels, and Cloudflare changed both its conversion and its writing rules because of it. I found no first-person data on whether tabbed content is found in web search.

### Cited Findings
- Cloudflare (style guide, last updated 2026-08-20): tab "panels are hidden until the tab itself is clicked"; running the HTML through an HTML-to-Markdown converter leaves "references to the panels `id`, usually handled by JavaScript, ... visible but non-functional." Its converter now transforms `Tabs` "into standard unordered lists." [opened] — [Cloudflare AI consumability](https://developers.cloudflare.com/style-guide/how-we-docs/ai-consumability/index.md)
- Cloudflare's writing rules: "The primary answer or core instruction should always appear in the main content flow," not "exclusively inside a tab or collapsible section"; "Use Details for supplementary information, not for the primary answer"; tabs for platform-specific variations only after the general concept. [opened] — [Cloudflare AI consumability](https://developers.cloudflare.com/style-guide/how-we-docs/ai-consumability/index.md)
- Cloudflare's measured saving: the Workers Get Started page is 15,229 tokens as HTML and 2,110 as Markdown, 7.22 times fewer. [opened] — [Cloudflare AI consumability](https://developers.cloudflare.com/style-guide/how-we-docs/ai-consumability/index.md)
- Stripe serves every docs page as Markdown at the URL plus `.md`. A search summary of an earlier version of Stripe's page gives as a reason that the Markdown "includes content hidden in tabs"; the page as I read it on 2026-10-10 no longer has that sentence. [opened for `.md`; summary for the tabs reason] — [Stripe, building with LLMs](https://docs.stripe.com/building-with-llms)
- The afdocs audit has a "Tabbed Content Serialization" check; one report found 33 tab groups on 7 of 15 sampled pages, all serializing under 50,000 characters. [summary] — [Mintlify score report](https://www.mintlify.com/score/tabstack.md)
- A vendor notes that selecting all and copying a docs page takes only the open tab. Vendor pitch. [summary] — [Product Hunt, Page2AI](https://www.producthunt.com/posts/1218817)

### Inferences
- A toolchain that knows a page's variants can write all of them into the Markdown output with a label per variant, which is the fix Cloudflare built after the fact. Serializing all variants multiplies page size, so it interacts with the size limits in the reference section.
- "Is the only statement of this instruction inside a tab or a collapsed section?" is checkable at build time.
- Giving a variant its own URL is the only way a search result or a citation can point at it. No evidence found on whether teams that do this gain anything.

### Gaps
- No docs team has published whether content in non-default tabs is indexed or ranked. No source found. Search engines' official positions are covered by another researcher.
- No measurement of agents picking the wrong variant from a serialized page.

## Code examples

### Takeaway
Research shows developers go to examples first, and that models reproduce deprecated APIs. Evidence on what makes an example get used correctly by an AI is absent.

### Cited Findings
- Meng, Steinhardt, and Schubert (2019), observation study: the most accessed content is the API reference, then the recipes page; code-oriented developers look for an example to adapt and stop reading once it works. Guidelines drawn: provide clean, working examples and connect concepts to code. [summary] — [paper PDF](https://mangold-international.com/resources/meng-et-al-how-developers-use-api-documentation-2019.pdf); [A List Apart discussion](https://alistapart.com/article/ten-extras-for-great-api-documentation/)
- "LLMs Meet Library Evolution" (ICSE 2025): seven models, 145 API mappings from eight Python libraries, 28,125 completion prompts; the first evaluation of deprecated API use in code completion. A decoding-time replacement fixed over 85% of deprecated uses; adding a prompt did not work well enough. [summary] — [arXiv 2406.09834](https://arxiv.org/abs/2406.09834v3)
- Context7's own guide says models write for the version that dominated training data, and that a version pin scopes retrieved docs to a release. Vendor claim. [summary] — [Context7](https://context7.com/versioned-library-documentation)
- An audit vendor claims copied docs pages lose code-block language tags. Vendor pitch. [summary] — [Product Hunt, Page2AI](https://www.producthunt.com/posts/1218817)

### Inferences
- The deprecated-API result is a version problem: the model has examples from many versions and no label on any. An example that states its version in the text next to it (not only in the URL or a picker) carries the label into training data and retrieval.
- A build that knows each example's language and the versions it applies to can keep the language tag and a version line in every output format.

### Gaps
- No evidence found on whether code blocks rank in search or appear in search features, or on complete versus excerpt, imports, or placeholders affecting AI use. No source found.

## Terminology and intent

### Takeaway
Advice only. The mismatch is described everywhere and measured nowhere I could find.

### Cited Findings
- Read the Docs' search analytics guidance: a query with no results suggests either missing content or a term to add to existing pages; a query with results for a page that exists can mean the page is hard to reach by navigation. [summary] — [Read the Docs search analytics guide (mirror)](https://readripman.readthedocs.io/en/latest/guides/search-analytics.html)
- A vendor example of the mismatch: a reader searches "customizing fonts"; the page is titled "Typography overrides". [summary] — [docsio.co](https://docsio.co/docs/guides/analytics/search-zero-results)
- Microsoft's rename guidance (Azure AD to Entra ID): for the first year, write "formerly Azure Active Directory" after the new name or "Azure Active Directory is now Microsoft Entra ID". [summary] — [Microsoft Learn, how to rename Azure AD](https://learn.microsoft.com/en-ca/entra/fundamentals/how-to-rename-azure-ad)

### Inferences
- An alias has to appear in the page text to be found by search or by a model. An alias that exists only in a site-search synonym list helps on-site search and nothing else.

### Gaps
- No evidence found that glossaries, "also known as" lines, or aliases improve findability. No comparison of task pages and reference pages for search or AI answers. No source found.

## Restructures and renames

### Takeaway
I found no docs-specific migration account with traffic numbers. General figures come from SEO vendors. The docs-specific accounts describe mechanics and loose ends.

### Cited Findings
- Vendor figures for site migrations in general: a 5 to 15% temporary dip is normal; 30 to 40% drops follow incomplete redirects and take 2 to 3 months to recover; one cited study of 892 domain migrations found 17% never recovered. Not docs-specific; SEO vendors with an interest. [summary] — [Attract Group](https://attractgroup.com/blog/website-redesign-seo/); [Search Engine Journal](https://www.searchenginejournal.com/site-migration-traffic-drop/)
- Google Search Central's own account of moving its documentation site (November 2021): mapping URLs took far longer than planned, and each localized site had its own URL pattern needing separate redirects. I could not read the article body; this is from a search summary. [summary] — [Google Search Central blog](https://developers.google.com/search/blog/2021/11/one-year-of-search-central?hl=en)
- Microsoft's Azure AD rename guidance: scan for all synonyms ("Azure Active Directory", "Azure AD", "AAD"); don't make breaking changes; the replacement script's output must be reviewed by hand; do not rename "Active Directory", a different product. [summary] — [Microsoft Learn](https://learn.microsoft.com/en-ca/entra/fundamentals/how-to-rename-azure-ad)
- After the rename, a third party notes that old instructions, screenshots, Microsoft URLs, APIs, and labels may still say Azure AD, and keeps the old label where an API, command, or field still uses it. [summary] — [FileWave KB](https://kb.filewave.com/books/identity-provider-idp-integration/page/azure-active-directory-is-now-microsoft-entra-id)
- React's move left at least one old link redirecting to a deprecated page with no successor. [summary] — [create-react-app #13537](https://redirect.github.com/facebook/create-react-app/issues/13537)
- Read the Docs: add redirects and also update internal links; old URLs stay in search indexes "for a while." Advice. [opened] — [Read the Docs SEO guide](https://docs.readthedocs.com/platform/stable/guides/technical-docs-seo-guide.html)

### Inferences
- The loose ends are the pages with no successor. A build can list every URL in the previous release that has neither a page nor a redirect in this one, and refuse a redirect whose target does not exist.
- A product rename leaves the old name in identifiers that cannot change. Old-name mentions need to be marked as intended so a rename check does not flag them every time.

### Gaps
- No docs team's traffic numbers before and after a migration, and no recovery time. No source found.

## Multiple products and domains

### Takeaway
No docs-specific evidence found.

### Cited Findings
- General case studies report gains from moving a subdomain into a subfolder (49% more organic traffic in a quarter for an education site), with other changes made at the same time; Ahrefs describes a case where traffic only moved. None is a docs site. [summary] — [Go Fish Digital](https://gofishdigital.com/case-study/online-trading-academy-smart-investment-seo); [Ahrefs](https://ahrefs.com/blog/subdomain-vs-subfolder)
- GitLab chose a separate subdomain for archived versions so they don't slow the main site's deploys, an operational reason and not a search one. [summary] — [gitlab-org epic 8937](https://gitlab.com/groups/gitlab-org/-/epics/8937)

### Inferences
- A separate host for retired versions has a side effect worth noting: robots.txt applies per host, so one file can exclude the whole archive, which avoids the per-path mistake GitLab recorded.

### Gaps
- No account of a docs site moving between subdomain and subfolder, or of several products sharing one docs site, with a search result. No source found.

## Translations

### Takeaway
The documented problem is stale translations, and the documented remedies are recording the source revision per translated page, a banner, and retiring locales that nobody maintains.

### Cited Findings
- MDN, 2021-03-25: kept 11 locales and stopped displaying 21, many with "very few pages, a high percentage of which are out-of-date or otherwise flawed"; estimated loss "less than 2%" of total traffic; removed articles' URLs redirect to their en-US equivalents; retirement is one-way. This is the plan as announced. [opened] — [Mozilla Hacks](https://hacks.mozilla.org/2021/03/mdn-localization-in-march-tier-1-locales-unfrozen-and-future-plans/)
- MDN later archived Polish and German because work on them had stalled. [summary] — [mdn discussion 147](https://github.com/orgs/mdn/discussions/147)
- MDN: translated pages kept legacy compatibility tables, so non-English readers saw outdated data; the fix was a macro that reads the shared data source. [summary] — [Mozilla Discourse](https://discourse.mozilla.org/t/help-wanted-replacing-legacy-compat-tables-with-bcd-powered-tables-in-locales/37847)
- OpenTelemetry: each translated page records the English commit it was translated from (`default_lang_commit`); `npm run check:i18n` lists drifted pages; a drifted page gets a `drifted_from_default` field and shows an "outdated" banner; the link checker skips drifted pages. Example output: "DRIFTED files: 361 out of 990" (illustrative). [opened] — [OpenTelemetry localization](https://opentelemetry.io/docs/contributing/localization/)
- Read the Docs names incomplete translations as a case where the canonical should point at the original. Advice. [opened] — [Read the Docs SEO guide](https://docs.readthedocs.com/platform/stable/guides/technical-docs-seo-guide.html)

### Inferences
- Data that is the same in every language (compatibility tables, API signatures, code) should come from one source so it cannot go stale in a translation. MDN's fix is that rule applied late.
- Drift is computable from the source revision. What to do with a drifted page in search (index it, `noindex` it, canonical to the source language) is a choice no source here has measured.

### Gaps
- No evidence on how stale or English-fallback pages behave in search results, or on `hreflang` for partly translated docs. Kubernetes's behaviour: not confirmed. No source found.

## Freshness

### Takeaway
One team has written down that deprecated pages should be kept from AI, and admits it cannot tell whether that works. Otherwise the practice is banners. No evidence found that dates affect readers.

### Cited Findings
- Cloudflare applies `noindex` by frontmatter to content such as deprecated features, because those pages "are no longer advisable to follow and could potentially confuse AI outputs," and adds that "it's unclear whether all AI crawlers will respect these directives." [opened] — [Cloudflare AI consumability](https://developers.cloudflare.com/style-guide/how-we-docs/ai-consumability/index.md)
- Banners on unmaintained content: PostgreSQL, Python (red for end of life), Kubernetes, React legacy, Angular v17, Elastic 6.8 ("no longer updated"). [summary] — [postgresql.org/docs/7.3](https://www.postgresql.org/docs/7.3/index.html); [python/cpython #120795](https://github.com/python/cpython/issues/120795); [Elastic 6.8 guide](https://www.elastic.co/guide/en/kibana/6.8/redirects.html)
- Retiring by removal: Django does not publish unsupported versions; Kubernetes stops publishing old version sites, which can linger as cached artifacts. [summary] — [Django ticket 22273](https://code.djangoproject.com/ticket/22273); [discuss.kubernetes.io](https://discuss.kubernetes.io/t/https-v1-21-docs-kubernetes-io-and-https-v1-20-docs-kubernetes-io-certificate-issue/24250)
- Elastic debated whether a removed feature's page should redirect to the last version that had it or to a new page saying the feature is unsupported in 9.x; open at the time. [summary] — [elastic/docs-content pull 1062](https://github.com/elastic/docs-content/pull/1062)

### Inferences
- A banner is rendered for humans. Unless the notice is in the page text of every output format, an agent reading Markdown does not see it. A build that knows a page is deprecated or belongs to an unsupported version can write that as the first line of the Markdown too.
- Elastic's question names the two tombstone designs: send the reader to the last version that had the page, or keep a page that says the thing is gone. PostgreSQL's "most recent version of it" rule is the first.

### Gaps
- No evidence found on "last updated" dates and reader trust, search, or AI answers; none on 410 versus redirect versus tombstone for docs. No source found.

## Changelogs and release notes

### Takeaway
Nothing found on their role in search or AI answers.

### Cited Findings
- The one use recorded: the deprecated-API study built its 145 API mappings by hand from documentation and changelogs, which shows changelogs are where deprecations are recorded and that extracting them took manual work. [summary] — [arXiv 2406.09834](https://arxiv.org/abs/2406.09834v3)

### Inferences
- If a toolchain already knows which versions each page and symbol applies to, a per-version "what changed" list is derivable, and would be the structured form of what those researchers assembled by hand.

### Gaps
- No source found on changelogs in search or AI answers for "what changed in version X".

## Site search

### Takeaway
The practice (read the zero-result queries, then write, rename, or add terms) is documented by tools. I found no docs team's account of what it changed and what happened.

### Cited Findings
- Read the Docs shows top queries for the past 30 days with result counts, and offers a CSV of all queries. [summary] — [Read the Docs search analytics guide (mirror)](https://readripman.readthedocs.io/en/latest/guides/search-analytics.html)
- One practitioner report (OpenROAD, 2023) lists its zero-result queries by frequency as topics not covered. [summary] — [UCSC OSPO report](https://ospo.ucsc.edu/report/osre23/ucsd/openroad/20230727-luarss/)
- Docusaurus with Algolia filters results to the current version and language by default; without it a query returns one result per version. One project had to re-run the crawler after each version release to avoid empty results. [summary] — [Docusaurus search](https://docusaurus.io/docs/search); [fishjam-cloud/documentation pull 280](https://github.com/fishjam-cloud/documentation/pull/280)
- Camunda's embedded search was Google's, so its site search broke when its Google indexing broke. [opened] — [Camunda forum thread 32044](https://forum.camunda.io/t/camunda-docs-repo-search-is-not-working-as-expected/32044)
- Vendor guidance: ignore queries below a threshold (5 hits suggested); separate vocabulary mismatches from content gaps. Advice. [summary] — [docsio.co](https://docsio.co/docs/guides/analytics/search-zero-results)

### Inferences
- Site search has the versioned-docs problem in small: results need a version filter, and the index must be rebuilt when the version list changes. A build that produces the index has both facts.

### Gaps
- No first-person account with outcomes. No source found.

## Measuring documentation discoverability

### Takeaway
Published material says page views are a weak measure and that many teams measure little. I found no team's account of what it stopped measuring.

### Cited Findings
- State of Docs 2026: 57% of teams don't track leads from documentation. [opened] — [GitBook, State of Docs 2026](https://www.gitbook.com/blog/state-of-docs-2026)
- A GitBook article citing the report: 49% of teams track no internal docs metrics. [summary] — [GitBook, business impact metrics](https://www.gitbook.com/blog/documentation-business-impact-metrics)
- Mintlify's guidance: views "could be driven by bot traffic or repeat visitors"; low traffic with high feedback scores suggests a good page that is hard to discover. Vendor advice. [summary] — [Mintlify, improving docs](https://mintlify.com/docs/guides/improving-docs)
- Redocly argues teams rely on page views that don't reflect value, and proposes acquisition, adoption, and deflection. Vendor advice. [summary] — [Redocly, via Plushcap](https://www.plushcap.com/content/redocly/blog/redocly-key-metrics-for-docs)
- Classifying requests by user agent undercounts agents: Codex sends no identifying header (Mintlify), and about 22% of agent traffic on GitBook is unidentified. [opened] — [Mintlify](https://www.mintlify.com/blog/state-of-ai); [GitBook](https://gitbook.com/blog/ai-docs-data-april-2026)
- Tailwind: usage "growing faster than it ever has" while docs traffic fell about 40%, so docs traffic stopped tracking adoption. [summary] — [DevClass](https://www.devclass.com/ai-ml/2026/01/08/tailwind-labs-lays-off-75-percent-of-engineers-thanks-to-brutal-impact-of-ai/4079571)
- PostgreSQL used which version receives inbound search traffic as its measure of the versioning problem. [opened] — [postgresql.org message](https://postgresql.org/message-id/CA%2BOCxoyVwmmZkWUJCez2hCqa89iGv%3Dvq58NF1yQkTg9gtpkn%3Dg%40mail.gmail.com)

### Inferences
- The one discoverability metric with a first-person use is "share of search landings on the current version" (PostgreSQL). It needs only the landing URL and the version it belongs to, which a version-aware build can supply as a URL-to-version map.
- Several of the problems above have a build-time measure that needs no analytics: pages whose canonical target doesn't exist, old-release URLs with no successor, pages over the agent size limit, instructions that exist only inside a tab, drifted translations.

### Gaps
- No dashboards or "we stopped measuring X" accounts from Stripe, Twilio, Cloudflare, Microsoft, Google, GitLab, GitHub, Kubernetes, DigitalOcean, HashiCorp, or Vercel found. I did not find relevant Write the Docs, API the Docs, or DevRelCon talks in the searches I ran; that is a limit of this pass, not evidence they don't exist.
