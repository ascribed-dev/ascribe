# Redirects: search engines, AI crawlers and assistants, and versioned documentation

Researched 2026-10-23. Google pages give their own "last updated" dates, which are listed with them. Findings marked "observed 2026-10-23" come from `curl -sIL` requests made on that date to the URL cited. Each one records the status codes, `Location` headers, `<link rel="canonical">` and `<meta name="robots">` that the server sent then. Sites can change these at any time.

## 1. Search engines (Google, Bing): redirect types, signal transfer, how long to keep redirects, chains, 404/410, soft 404s, client-side redirects, canonical vs redirect, sitemaps, Change of Address

### Takeaway
Google treats a server-side 301 or 308 (and an instant meta refresh) as a strong signal that the target is canonical. It treats a 302, 303 or 307 (and a delayed refresh) as a weak signal that keeps the source URL in results. Google says to keep redirects "generally at least 1 year" and for users "indefinitely". It says to redirect to the final URL in one hop (it follows at most 10), to return 404 or 410 for content that is gone, and never to send many old URLs to the home page, which it may treat as a soft 404. Bing's guidance is older and thinner but agrees.

### Cited Findings
**Redirect types and how strong a signal each is**
- Permanent: HTTP 301 and 308. Googlebot follows them and treats the target as canonical. "Use permanent redirects when you're sure that the redirect won't be reverted." Server-side redirects are preferred. — [Google, Redirects and Google Search (updated 2026-04-14)](https://developers.google.com/search/docs/crawling-indexing/301-redirects)
- Temporary: HTTP 302, 303 and 307 are followed, but the target is not treated as canonical. The source page usually stays in results, though the target can still be indexed if other canonical signals point to it. — [Google, Redirects (2026-04-14)](https://developers.google.com/search/docs/crawling-indexing/301-redirects)
- Google uses a 301 or 308 as a "strong" signal and a 302, 303 or 307 as a "weak" one. 307 is handled like 302, and 308 like 301. Google ignores the body of the redirecting response and processes the final target. — [Google, How HTTP status codes affect Google's crawlers (updated 2026-02-04)](https://developers.google.com/search/docs/crawling-indexing/http-network-errors)
- A meta refresh with a 0-second delay is treated as permanent. A delayed meta refresh, or an HTTP `Refresh` header with a delay, is treated as temporary. — [Google, Redirects (2026-04-14)](https://developers.google.com/search/docs/crawling-indexing/301-redirects)
- Google says to use a JavaScript `location` redirect only if neither a server-side redirect nor a meta refresh is possible, because "rendering may fail for various reasons" and Google might never see the redirect. A "crypto redirect" (a visible link with a short explanation) may not be recognized at all. — [Google, Redirects (2026-04-14)](https://developers.google.com/search/docs/crawling-indexing/301-redirects)
- Google keeps both URLs. The one that isn't canonical becomes an "alternate name" and may still appear in results for a while after a move, which "fades" without any action. — [Google, Redirects (2026-04-14)](https://developers.google.com/search/docs/crawling-indexing/301-redirects)
- Older guidance: in 2010 Mueller said a server-side 301 is preferred over meta refresh or JavaScript (reported by Search Engine Roundtable). Current docs instead accept an instant meta refresh as permanent. — [SER, "Google Strongly Recommends 301 Redirect Over Meta Refresh"](https://seroundtable.com/archives/016431.html)

**How long to keep redirects**
- Keep redirects "for as long as possible, generally at least 1 year". This gives Google time to transfer signals and recrawl links that point to the old URLs. For users, the page suggests keeping them indefinitely while you update your own links and high-volume external links. — [Google, How to move a site (updated 2026-08-20)](https://developers.google.com/search/docs/crawling-indexing/site-move-with-url-changes)
- Mueller (Ask Googlebot video): "To be certain that a redirect has been seen a few times, we recommend keeping the redirect in place for at least one year." — [Search Engine Journal](https://www.searchenginejournal.com/google-keep-301-redirects-in-place-for-a-year/428998/)
- Mueller (2019 tweet, as reported): keeping redirects is usually cheap; "at least 1 year" is the minimum because by then the web has moved on, and redirects that are eventually dropped or lost are "generally fine too". — [SEJ coverage via search results](https://www.searchenginejournal.com/google-keep-301-redirects-in-place-for-a-year/); I didn't open the original tweet.
- Claims that the target "fully inherits" the old URL's signals after a year (for example AirOps' glossary) go further than Google's wording supports. — [AirOps glossary](https://www.airops.com/glossary/301-redirect)

**Redirect chains**
- Googlebot follows up to 10 hops. Google recommends redirecting "to the final destination directly". If you can't, keep chains to ideally no more than 3 hops and fewer than 5, because chains add latency and some browsers handle long ones badly. — [Google, How to move a site (2026-08-20)](https://developers.google.com/search/docs/crawling-indexing/site-move-with-url-changes)
- Google's crawlers follow 10 hops by default for general web content, but Google's Inspection Tools don't follow redirects, and other products may use other limits. — [Google, HTTP status codes (2026-02-04)](https://developers.google.com/search/docs/crawling-indexing/http-network-errors)
- Microsoft Learn's authoring guide asks authors to collapse chains: if A redirects to B and B later redirects to C, both entries should point to C. — [Microsoft Learn, Sort redirects](https://learn.microsoft.com/contribute/docs-authoring/sort-redirects)

**Removed pages: 404 vs 410, soft 404s, redirecting to the home page**
- Content that isn't moving should "correctly return an HTTP 404 or 410". Several old pages merged into one new page may all redirect to it. Sending many old URLs to one unrelated page, such as the home page, "can confuse users and might be treated as a soft 404 error." — [Google, How to move a site (2026-08-20)](https://developers.google.com/search/docs/crawling-indexing/site-move-with-url-changes)
- "All 4xx errors, except 429, are treated the same." A URL that was indexed is removed from the index. A 404 on a newly found URL isn't processed, and Google crawls it less and less often. — [Google, HTTP status codes (2026-02-04)](https://developers.google.com/search/docs/crawling-indexing/http-network-errors)
- A 2xx response is reported as a soft 404 when its content looks like an error page or is empty. — [Google, HTTP status codes (2026-02-04)](https://developers.google.com/search/docs/crawling-indexing/http-network-errors)
- Mueller has said Google sometimes confirms a 404 before dropping a URL and "tend[s] to do that faster with a 410". He put the difference at "a couple days or so" and said that in the long run a 404 is the same as a 410. — [Search Engine Roundtable](https://seroundtable.com/404-410-google-15225.html); [SER, content removal](https://seroundtable.com/google-content-removal-16851.html). This is older SER reporting, and current Google docs make no distinction between the two.

**Canonical tags vs redirects, robots.txt, noindex**
- Google ranks canonicalization signals: redirects are a "strong signal that the target … should become canonical", `rel="canonical"` is also strong, and sitemap inclusion is "a weak signal". Using several together raises the chance the preferred URL is chosen. — [Google, Consolidate duplicate URLs (updated 2026-07-10)](https://developers.google.com/search/docs/crawling-indexing/consolidate-duplicate-urls)
- Don't use robots.txt to canonicalize: "Google may still index URLs that are disallowed in robots.txt without their content." Don't use `noindex` to pick a canonical within a site either; use `rel="canonical"`. Google decides which pages are duplicates from how similar their content is. — [Google, Consolidate duplicate URLs (2026-07-10)](https://developers.google.com/search/docs/crawling-indexing/consolidate-duplicate-urls)
- After a move, every new URL should carry a canonical that points to itself, and canonical tags must use the new URLs. — [Google, How to move a site (2026-08-20)](https://developers.google.com/search/docs/crawling-indexing/site-move-with-url-changes)

**Sitemaps after a move, and the Change of Address tool**
- While watching a move, submit both the old-URL sitemap and the new one. Search Console warnings that the old URLs redirect are "normal and you can ignore these warnings." Afterwards, submit the new sitemap and remove the old one. — [Google, How to move a site (2026-08-20)](https://developers.google.com/search/docs/crawling-indexing/site-move-with-url-changes)
- "Pick a canonical URL for each of your pages and submit them in a sitemap"; every URL in a sitemap is treated as a suggested canonical. — [Google, Consolidate duplicate URLs (2026-07-10)](https://developers.google.com/search/docs/crawling-indexing/consolidate-duplicate-urls)
- Change of Address is for moving to another domain or subdomain. It isn't needed for HTTP to HTTPS, for www vs non-www, or for moving paths within a domain. — [Google, How to move a site (2026-08-20)](https://developers.google.com/search/docs/crawling-indexing/site-move-with-url-changes)

**Bing**
- Bing's archived Site Move help (2014) says 301 redirects are "sufficient for Bing to understand that you want the new URLs to be indexed", and that the Site Move tool only speeds this up. — [Bing help, archived 2014](https://webarchive.library.unt.edu/web/20140924024129mp_/http://www.bing.com/webmaster/help/how-to-use-the-site-move-tool-bb8f5112)
- Bing's 2013 blog advised keeping redirects in place for at least several months. — [Bing Webmaster blog, 2013-03-15](https://blogs.bing.com/webmaster/2013/03/15/bing-wmt-launches-site-move-tool)
- A 2021 Microsoft forum reply says the Site Move tool is no longer in the new Bing Webmaster console. — [Microsoft Tech Community](https://techcommunity.microsoft.com/discussions/bing/domain-move-tool/2144750)
- IndexNow (used by Bing, Yandex and others): "You should submit redirected URLs and pages that return HTTP 404 or HTTP 410 status codes." — [IndexNow FAQ](https://www.indexnow.org/faq)
- A third-party rules site says Bing removed a line penalizing meta refresh from its guidelines in May 2023. I couldn't confirm this from Bing. — [PowerMapper SortSite rule](https://www.powermapper.com/products/sortsite/rules/seometarefresh/)

### Inferences
- A docs toolchain's default for a moved page should be a server-side 301 or 308 straight to the final URL. Where the host can't send HTTP redirects (a static host with no redirect config), the fallback is a 0-second meta refresh plus a `rel="canonical"` to the target. JavaScript alone is the last resort. To keep chains to one hop, the toolchain can collapse A→B→C into A→C when it builds the redirects.
- Redirects should be kept indefinitely by default. Google's one year is a minimum for search signals, not a time after which removing them is advised. For a static docs site a redirect costs almost nothing, and the old links outside the site (question and answer sites, issues, product UIs) never stop being followed.
- A removed page with no successor should return a real 404, or 410 if the host supports it. It should not redirect to the home page or a section index. The difference between 410 and 404 is small for Google.
- After a reorganization, the sitemap should list only the new canonical URLs. Submitting the old URLs' sitemap for a while is something Google allows during a domain move, not something docs need for moves inside a site.

### Gaps
- I didn't confirm when Google's hop limit was set at 10. Older Google help is often quoted as saying 5 hops per crawl attempt, but I didn't find a dated primary source for that.
- I didn't find Bing's current Webmaster Guidelines page, which renders client-side and returned no text. Bing's position on 302 vs 301, meta refresh, and how long to keep redirects is supported only by sources from 2013 and 2014 and by secondhand reports.
- I didn't find a primary source on how PageRank or other signals pass through a 3xx today. Gary Illyes said in 2016 that 30x redirects no longer lose PageRank, but I didn't verify this.

## 2. AI crawlers and answer engines: do they follow redirects, how do they treat 301 vs 302 vs meta refresh, what happens to citations of an old URL, and what agent-docs specs say

### Takeaway
No AI vendor (OpenAI, Anthropic, Perplexity, Google for Gemini) publishes how its crawlers or user agents handle redirects. The best evidence is indirect. Vercel's 2024 log study shows AI crawlers hitting 404s about 34% of the time (Googlebot about 8%) and redirects 14% of the time (ChatGPT). It also shows none of the major AI crawlers runs JavaScript, so JavaScript redirects are invisible to them. Ahrefs (2025) found AI assistants send users to 404 pages 2.87 times as often as Google. Its causes are old URLs moved without redirects, and plausible URLs the assistant made up. The Web Documentation Delivery Spec (agentdocsspec.com) says outright to use same-host HTTP 301 or 302 redirects. It warns against cross-host redirects, which some agents such as Claude Code won't follow automatically, and fails JavaScript redirects.

### Cited Findings
**What crawlers do (measured)**
- Vercel and MERJ (published 2024-12-17, from nextjs.org and Vercel network logs): 404 share of fetches was ChatGPT 34.82%, Claude 34.16%, Googlebot 8.22%. Redirect share was ChatGPT 14.36%, Googlebot 1.49%; no figure is given for Claude. — [Vercel, The rise of the AI crawler](https://vercel.com/blog/the-rise-of-the-ai-crawler)
- None of the major AI crawlers renders JavaScript, including GPTBot, OAI-SearchBot, ChatGPT-User and ClaudeBot. ChatGPT fetches JavaScript files in 11.50% of requests and Claude in 23.84%, but neither runs them. Gemini uses Googlebot's infrastructure and renders, and AppleBot also renders. — [Vercel (2024-12-17)](https://vercel.com/blog/the-rise-of-the-ai-crawler)
- Vercel's recommendation: "Efficient URL management matters more than ever", which it spells out as "maintaining proper redirects, keeping sitemaps up to date, and using consistent URL patterns". — [Vercel (2024-12-17)](https://vercel.com/blog/the-rise-of-the-ai-crawler)
- Volume: in one month GPTBot made 569M fetches and Claude 370M, against Googlebot's 4.5B; together the two are about 20% of Googlebot's. — [Vercel (2024-12-17)](https://vercel.com/blog/the-rise-of-the-ai-crawler)

**Vendor documentation (what the bots are; none mentions redirects)**
- OpenAI documents three agents. OAI-SearchBot finds pages for ChatGPT search, follows robots.txt, and picks up robots.txt changes in about 24 hours. GPTBot collects training data and follows robots.txt. ChatGPT-User fetches pages when a user asks; because it is user-initiated, "robots.txt rules may not apply". The page says nothing about redirects, canonical URLs or sitemaps. — [OpenAI, Overview of OpenAI crawlers](https://developers.openai.com/api/docs/bots). (The old address, platform.openai.com/docs/bots, now returns a cross-host 301 to this one, observed 2026-10-23.)
- Anthropic documents three bots. ClaudeBot collects training data. Claude-User fetches pages for a user's request. Claude-SearchBot indexes pages to improve search results. All honor robots.txt, which has to be set per subdomain, and support Crawl-delay. Nothing is said about redirects. — [Anthropic, Does Anthropic crawl data from the web](https://support.claude.com/en/articles/8896518)
- Google-Extended is a robots.txt token, not a separate crawler. Gemini gets pages through Googlebot's infrastructure (per Vercel above), so Google's documented redirect handling (Q1) applies to it. — [Vercel (2024-12-17)](https://vercel.com/blog/the-rise-of-the-ai-crawler)
- One vendor blog claims GPTBot, ClaudeBot and PerplexityBot each handle HTTP redirects differently and follow 3 to 5 hops in practice. It gives no method I could check, so treat it as unverified. — [CaptainDNS blog](https://www.captaindns.com/en/blog/ai-crawlers-redirects-handling-gptbot-claudebot-perplexitybot)

**Agent-docs specifications**
- Web Documentation Delivery Spec v0.6.0 (Draft, 2026-09-13), "Start Here" item 5: "Don't break your URLs. If you must move content, use same-host HTTP redirects. Avoid cross-host redirects, JavaScript redirects, and soft 404s." — [agentdocsspec.com/spec/web](https://agentdocsspec.com/spec/web/)
- Its `redirect-behavior` check (URL Stability, Medium severity) passes when every redirect is a same-host HTTP 301 or 302. It warns on cross-host HTTP redirects ("Claude Code, for example, doesn't automatically follow cross-host redirects", a guard against open-redirect attacks). It fails on JavaScript redirects ("JavaScript redirects don't work at all because agents don't execute JavaScript"). The check looks for `window.location` and meta refresh as signs of client-side redirects. — [agentdocsspec.com, URL stability](https://agentdocsspec.com/spec/web/url-stability/)
- Its `http-status-codes` check fails pages that serve "not found" content with status 200: "Soft 404s … are worse than real 404s for agents", because an agent may pull information out of the error page. — [agentdocsspec.com, URL stability](https://agentdocsspec.com/spec/web/url-stability/)
- llmstxt.org says nothing about redirects or moved content; a text search of the page found no such guidance. — [llmstxt.org](https://llmstxt.org/)
- This session observed the same thing in practice: Claude Code's WebFetch returned the cross-host 301 from platform.openai.com to developers.openai.com to the caller instead of following it, and needed a second call. This matches the spec's warning. — [OpenAI bots URL](https://platform.openai.com/docs/bots)

**Citations of old or invented URLs**
- Ahrefs (published 2025-09-02) checked 16M URLs cited by ChatGPT, Perplexity, Copilot, Gemini, Claude and Mistral, and found AI assistants send visitors to 404 pages 2.87 times as often as Google Search. Clicked URLs returning 404: ChatGPT 1.01%, Claude 0.58%, Copilot 0.34%, Perplexity 0.31%, Gemini 0.21%, Mistral 0.12%, Google 0.15%. Cited URLs returning 404: ChatGPT 2.38%, Google 0.84%. — [Ahrefs, How often do AI assistants hallucinate links](https://ahrefs.com/blog/how-often-do-ai-assistants-hallucinate-links); [SEJ coverage](https://www.searchenginejournal.com/ai-search-sends-users-to-404-pages-nearly-3x-more-than-google/555267/)
- Ahrefs gives two causes. Some cited URLs were valid once but later deleted or moved without a redirect. Others were made up to fit a site's URL pattern, such as `/blog/newsletter/`. Assistants that use Google's or Bing's index (Gemini, Perplexity, Copilot) had lower rates. — [Ahrefs (2025-09-02)](https://ahrefs.com/blog/how-often-do-ai-assistants-hallucinate-links)

### Inferences
- Trained models remember URLs from the moment their training data was collected, and keep citing them for the life of the model, often years. A docs site therefore gets requests for old URLs long after search engines have caught up. That argues for keeping redirects indefinitely, not for one year.
- Of the redirect methods, only a server-side HTTP redirect works for every kind of client: search engines, AI crawlers, and agents fetching on request. A meta refresh is seen by Google, but its effect on AI crawlers is undocumented. Agentdocsspec counts it as a client-side redirect to look for, not a pass. A JavaScript redirect is invisible to every AI crawler except Gemini.
- Keep redirects on the same host. A docs site that moves to another domain, or sends `/docs/x` to `docs.example.com/x`, risks agents like Claude Code stopping at the first hop.
- Some of the 404s Ahrefs counts are made-up URLs that follow a site's pattern. A toolchain could lower the cost of those by giving the 404 page a real 404 status, links to similar pages, and a search link. Redirecting guesses to the home page would be a soft 404, which agentdocsspec scores as worse.
- Neither agentdocsspec nor any vendor tells 301 apart from 302 for agents; both pass the agentdocsspec check. The choice between them matters only to search engines (Q1).

### Gaps
- No primary source says how GPTBot, ClaudeBot, PerplexityBot, OAI-SearchBot, Claude-SearchBot or Perplexity-User treat 301 vs 302 or meta refresh, how many hops they follow, or whether they update a stored URL when it redirects. Settling it would take a controlled log test.
- No vendor says whether an answer engine's citations follow a redirect (show the new URL) or keep the URL that was cited.
- I found no AI-specific redirect guidance in llms.txt, and none from Perplexity.

## 3. Link rot: how often links break, and the cost of broken inbound links

### Takeaway
Link rot is large and grows with age. Pew (2024) found 38% of pages from 2013 gone a decade later, and most of the losses were single pages removed from sites that still exist, which is exactly what a docs reorganization does. On Stack Overflow, 14.2% of links were broken in 2019 and only 5.8% of posts with broken links were ever fixed. Inbound links are almost never repaired by whoever wrote them, so the site that moved the page is the only place the break can be fixed.

### Cited Findings
- Pew Research Center, "When Online Content Disappears" (2024-05-17): 38% of pages that existed in 2013 were no longer accessible in October 2023, against about 8% of pages from 2023, and about a quarter of all pages from 2013 to 2023. The sample was drawn from Common Crawl. — [Pew report PDF](https://pewresearch.org/wp-content/uploads/sites/20/2024/05/pl_2024.05.17_link-rot_report.pdf); [infoDOCKET summary](https://www.infodocket.com/2024/05/17/new-report-from-pew-research-when-online-content-disappears-link-rot-and-digital-decay-on-government-news-and-other-webpages)
- Pew: most losses were individual pages removed from sites that otherwise still worked, not whole sites going offline. — [infoDOCKET summary of Pew](https://www.infodocket.com/2024/05/17/new-report-from-pew-research-when-online-content-disappears-link-rot-and-digital-decay-on-government-news-and-other-webpages)
- Gigazine reports Pew's sample as 999,899 URLs. — [Gigazine](https://gigazine.net/gsc_news/en/20240520-online-contents-disappears)
- "Broken External Links on Stack Overflow" (arXiv 2020, data from June 2019): 14.2% of links on Stack Overflow were broken, and only 5.8% of posts with broken links ever had them fixed. Broken links appear regardless of a post's score. Most pointed to GitHub and other user-maintained sites, and web-technology tags had more of them. — [arXiv 2010.04892](https://arxiv.org/abs/2010.04892)
- Zhang et al., "An Empirical Study of Obsolete Answers on Stack Overflow" (IEEE TSE): 58.4% of obsolete answers were already out of date when posted, and only 20.5% were ever updated. — [Replication repo](https://github.com/SAILResearch/replication-obsolete_answers_SO/)
- Zittrain, Albert and Lessig (Harvard Law Review Forum, 2014): more than 70% of URLs in the Harvard Law Review and other journals, and 50% of URLs in U.S. Supreme Court opinions, no longer lead to the material cited. Their response was Perma.cc. — [Paper PDF](https://Gwern.net/doc/cs/linkrot/archiving/2014-zittrain.pdf); [ABA Journal](https://www.abajournal.com/news/article/link_rot_int_scotus_decisions_documented_half_dont_go_to_original_material)
- AI assistants add a new source of broken inbound links: 1.01% to 2.38% of URLs cited by ChatGPT return 404 (Ahrefs 2025; see Q2). — [Ahrefs](https://ahrefs.com/blog/how-often-do-ai-assistants-hallucinate-links)
- Microsoft Learn requires a redirect whenever an article is deleted. Its build warns when a redirection rule is removed or changed, telling authors not to remove it so links don't break. — [Microsoft Learn, Sort redirects](https://learn.microsoft.com/contribute/docs-authoring/sort-redirects); [PowerShell-Docs PR #10370](https://github.com/MicrosoftDocs/PowerShell-Docs/pull/10370)

### Inferences
- The low fix rate on Stack Overflow (5.8%) means a docs team can't count on people outside it updating links. A redirect at the source is the only fix that reaches every old link.
- Docs URLs are pasted into places that can't be edited or searched: product UIs and error messages, CLI help, released package READMEs, issues, and chat logs. Old releases keep shipping their links for as long as they're installed, which again argues for keeping redirects forever.

### Gaps
- I found no study measuring link rot for documentation sites specifically, such as how many links into docs.* break per year or how many come from GitHub issues or product UIs.
- I found no study that puts a cost (support tickets, traffic lost, conversion) on broken inbound docs links. The often-quoted New York Times link-rot figure (about 25% of deep links, 2021) wasn't checked.

## 4. Versioned docs: "latest" or "stable" aliases, versioned URLs, moved pages across versions, retired versions, canonical vs noindex

### Takeaway
Large docs sites use one of three patterns. (a) Version folders plus an alias, with old versions' canonical tags pointing at the alias or current version's copy of the same page: Python, Django, Microsoft Learn, Read the Docs. (b) One unversioned, cumulative set of docs, with old versioned trees kept but marked `noindex`: Elastic since April 2025. (c) Retired versions redirected page by page to the current docs (MongoDB), or removed so they return 404 (Kubernetes). Aliases are usually served with a temporary redirect (302) to a concrete version, or directly as their own URL (Python's `/3/`). Old versions typically keep serving content with a cross-version canonical rather than `noindex`. Python also blocks end-of-life versions in robots.txt.

### Cited Findings
**Python (docs.python.org)**
- `/` returns a 302 to `/3/`. `/3/…` is a stable alias that serves content directly, with a canonical pointing to itself. — observed 2026-10-23, [docs.python.org](https://docs.python.org/)
- Old versions are served with a canonical to the `/3/` copy: `/3.8/library/asyncio.html` has a canonical of `https://docs.python.org/3/library/asyncio.html`, and `/3.8/` points to `/3/index.html`. — observed 2026-10-23, [3.8 asyncio](https://docs.python.org/3.8/library/asyncio.html)
- robots.txt disallows `/dev`, `/release` and versions past end of life (`/2/`, `/2.0/` through `/2.7/`, `/3.0/`–`/3.2/` and others), under the comment "Prevent development and old documentation from showing up in search results." — observed 2026-10-23, [docs.python.org/robots.txt](https://docs.python.org/robots.txt)
- Removed modules keep a page in the current docs. `/3/library/distutils.html`, `imp.html` and `asynchat.html` return 200 with a stub filed under "Removed Modules" in the 3.15 docs, rather than 404 or a redirect. — observed 2026-10-23, [distutils](https://docs.python.org/3/library/distutils.html)
- A page missing from an old version returns 404: `/2/library/asyncio.html` returns 404. — observed 2026-10-23

**Django (docs.djangoproject.com)**
- `/en/stable/…` returns a 302 to the current release (`/en/6.1/…`). The page there has a canonical to itself and `robots: ALL`. — observed 2026-10-23, [stable URLs page](https://docs.djangoproject.com/en/stable/topics/http/urls/)
- Old and development versions (`/en/1.8/…`, `/en/2.2/…`, `/en/dev/`) serve with a canonical to the same path under `/en/6.1/`, not `noindex`. — observed 2026-10-23, [1.8 URLs page](https://docs.djangoproject.com/en/1.8/topics/http/urls/)
- A page removed in the current release returns 404 after the `stable` redirect: `/en/stable/ref/contrib/gis/geoip/` → 302 → `/en/6.1/ref/contrib/gis/geoip/` → 404. — observed 2026-10-23
- Django's robots.txt disallows untranslated sections of translated docs (for example `/el/6.1/ref`), not old versions. — observed 2026-10-23, [robots.txt](https://docs.djangoproject.com/robots.txt)

**Kubernetes (kubernetes.io)**
- "This website contains documentation for the current version of Kubernetes and the four previous versions". The latest is on kubernetes.io, and each older version on a subdomain such as `v1-36.docs.kubernetes.io` … `v1-33.docs.kubernetes.io`. Keeping docs online is separate from whether a release is supported. — [Kubernetes, Available documentation versions](https://kubernetes.io/docs/home/supported-doc-versions/)
- An older version still listed (`v1-33.docs.kubernetes.io/docs/concepts/overview/`) returns 200 with `robots: index, follow` and no canonical tag. Versions that have dropped out (`v1-30`, `v1-20`) return 404 for the same page path; they aren't redirected. — observed 2026-10-23
- kubernetes.io's robots.txt disallows only `/legacy/`, `/v1.0/`, `/v1.1/` and 404 pages. — observed 2026-10-23, [robots.txt](https://kubernetes.io/robots.txt)

**MongoDB (mongodb.com/docs)**
- `/docs/manual/` is the unversioned current alias, with a canonical to itself and `robots: index`. — observed 2026-10-23, [manual CRUD](https://www.mongodb.com/docs/manual/crud/)
- Retired versions redirect page by page to the current alias: `/docs/v5.0/crud/` and `/docs/v4.4/crud/` each 301 → `/docs/manual/crud` → 301 → `/docs/manual/crud/`. That is a two-hop chain, the second hop only adding a trailing slash. — observed 2026-10-23

**Elastic (elastic.co)**
- In April 2025 Elastic moved to a new site at elastic.co/docs, built with docs-builder from Markdown. "This documentation is cumulative: a new set of docs is not published for every minor release. Instead, each page stays valid over time and incorporates version-specific changes." — [Elastic, Contribute to Elastic documentation](https://www.elastic.co/docs/contribute-docs/)
- Old `current` alias URLs redirect page by page to the new information architecture: `/guide/en/elasticsearch/reference/current/query-dsl.html` → 301 → `/docs/explore-analyze/query-filter/languages/querydsl`, and `…/current/index.html` → 301 → `/docs/reference/elasticsearch`. — observed 2026-10-23
- Pinned old-version pages remain but are hidden from search: `/guide/en/elasticsearch/reference/7.17/index.html` and `/8.18/index.html` return 200 with `robots: noindex,nofollow` and no canonical. — observed 2026-10-23

**Microsoft Learn**
- Versions are a query parameter, `?view=`. The unversioned `/dotnet/api/system.string` gets a 301 to `?view=net-10.0`. An older view (`?view=net-6.0`) serves content with a canonical to `?view=net-10.0`. — observed 2026-10-23, [System.String](https://learn.microsoft.com/en-us/dotnet/api/system.string?view=net-6.0)
- When a page doesn't exist in the requested version, Learn falls back to the nearest version that has it: `system.web.ui.page?view=net-8.0` → 301 → `?view=netframework-4.8.1&viewFallbackFrom=net-8.0`, with a canonical to the fallback view. — observed 2026-10-23
- Retired content moves under `/previous-versions/` and stays indexable (`ROBOTS: INDEX,FOLLOW`, canonical to itself). — observed 2026-10-23, [previous-versions example](https://learn.microsoft.com/en-us/previous-versions/windows/desktop/legacy/)
- Redirects live in `.openpublishing.redirection.json`. Preserving the document ID on a redirect moves page views and rankings to the target, and is meant for renames. — [Microsoft Mixed Reality contributing guide](https://learn.microsoft.com/en-gb/windows/mixed-reality/contributing); [Sort redirects](https://learn.microsoft.com/contribute/docs-authoring/sort-redirects)

**Read the Docs (platform behaviour shared by thousands of projects)**
- The root URL and `/<lang>/` redirect to the default version (usually `latest` or `stable`). The `/page/<path>` prefix redirects any page to the default version, and is recommended for links from outside because the root redirect doesn't carry the page path. — [Read the Docs, Redirects](https://docs.readthedocs.com/platform/stable/user-defined-redirects.html)
- Observed: `docs.readthedocs.io/` → 302 → `docs.readthedocs.com/platform/` → 302 → `/platform/stable/` (a cross-host chain after Read the Docs' own domain move), and `/page/versions.html` → 302 → `/platform/stable/versions.html`. — observed 2026-10-23
- "Page Redirects let you redirect a page across all versions"; Exact Redirects match the full URL including language and version. By default redirects apply only to pages that don't exist; "Forced redirects" apply to existing pages too. To retire a version, use an exact redirect such as `/en/2.0/*` → `/en/3.0/:splat`, and the old version "must be disabled" or the redirect forced. — [Read the Docs, Redirects](https://docs.readthedocs.com/platform/stable/user-defined-redirects.html)
- The canonical URL is set from the project's default version and canonical domain, so that search engines send users to the right version and domain. Without canonical URLs, "outdated documentation" can rank highly. — [Read the Docs, Canonical URLs](https://docs.readthedocs.com/platform/stable/canonical-urls.html)

### Inferences
- The most common default across these sites is: an alias URL (`stable`, `3`, `manual`, unversioned) that is the canonical target; old versions served with a `rel=canonical` to the same path under the alias; a temporary redirect only from the bare root or alias entry point to a concrete version. Using a 302 for the alias is correct under Google's definitions (Q1), because the alias's target changes with every release.
- A cross-version canonical is only a hint. Google picks canonicals by content similarity (Q1), so a 3.8 page whose content differs a lot from the current one may be indexed on its own anyway. That is one reason Elastic uses `noindex` and Python uses robots.txt for end-of-life versions. Python's approach has a known weakness: robots.txt-blocked URLs can be indexed without content, and their canonical tags can't be read.
- Pages that moved within the current version and pages missing from an old version are different problems:
  - Within a version, a moved page should get a 301 from its old path to its new one (Read the Docs' page redirect "across all versions", Microsoft Learn's redirection file).
  - A page absent from an old version can fall back to the nearest version that has it (Microsoft Learn's `viewFallbackFrom`) or return 404 (Django, Python).
- There are three precedents for retired versions:
  - MongoDB's page-by-page 301 to the current alias keeps old inbound links working, but can land readers on content for a different version.
  - Kubernetes' 404 drops them entirely.
  - Elastic's and Python's "keep but hide" keeps them readable for users and out of search.
  For a toolchain whose goal is "findable and citable", keeping retired versions with `noindex`, or redirecting them page by page to the alias where the page still exists, both beat 404. Python's stub pages for removed modules are a fourth option for content that is gone: a real page saying where it went.

### Gaps
- I didn't confirm whether Kubernetes' retired version subdomains redirect at the root (only page paths were tested), or what Kubernetes' written policy is for versions leaving the five-version window.
- I found no engineering blog post from Python, Django, MongoDB or Elastic explaining their choice of canonical, noindex or redirect for old versions. The findings above are observed behavior, not stated rationale.
- Read the Docs' redirect docs don't say whether its redirects use 301 or 302. The root and `/page/` redirects observed were 302.
- I didn't confirm whether a page missing from an old Django version, but present in the current one, keeps a canonical to a current page.

## 5. Removing content: redirect to a related page vs 404/410, and redirecting everything to the home page

### Takeaway
Redirect only when a real successor exists: the same content moved, or several pages merged into one. Otherwise return 404 or 410. Google says sending many URLs to an unrelated page such as the home page may be treated as a soft 404, so the old URL loses its signals and still misleads users. Agentdocsspec scores soft 404s as worse than real 404s for agents.

### Cited Findings
- Redirect URLs whose content moved. When pages are merged, redirect the old URLs to the merged page. Content not being moved should "correctly return an HTTP 404 or 410". — [Google, How to move a site (2026-08-20)](https://developers.google.com/search/docs/crawling-indexing/site-move-with-url-changes)
- Redirecting many old URLs to an irrelevant destination such as the home page "can confuse users and might be treated as a soft 404 error." — [Google, How to move a site (2026-08-20)](https://developers.google.com/search/docs/crawling-indexing/site-move-with-url-changes)
- A 200 response whose content is an error, an empty page or an error message is reported as a soft 404. — [Google, HTTP status codes (2026-02-04)](https://developers.google.com/search/docs/crawling-indexing/http-network-errors)
- Google treats 404 and 410 the same over time; 410 may drop the URL "a couple days" sooner. — [SER](https://seroundtable.com/404-410-google-15225.html)
- "Soft 404s (200 status with 'page not found' content) are worse than real 404s for agents", because an agent may extract information from the error page. — [agentdocsspec.com, URL stability](https://agentdocsspec.com/spec/web/url-stability/)
- Microsoft Learn requires a redirect whenever an article is deleted, so that references to it still resolve. The page I read doesn't say how to choose the target. — [Microsoft Learn, Sort redirects](https://learn.microsoft.com/contribute/docs-authoring/sort-redirects)
- Python keeps a stub page (200) for each removed standard-library module under "Removed Modules" (observed 2026-10-23 for distutils, imp and asynchat). It explains the removal rather than returning 404 or redirecting. — [docs.python.org distutils](https://docs.python.org/3/library/distutils.html)
- Submitting redirected and 404 or 410 URLs through IndexNow tells Bing and other engines about the change sooner. — [IndexNow FAQ](https://www.indexnow.org/faq)

### Inferences
- A docs toolchain's choices, in order of preference:
  1. 301 to the successor page when one exists. That includes a split page sending readers to the part that holds most of the old content.
  2. A real page with a 200 status explaining the removal and linking to alternatives, when readers need to be told something. This is Python's "Removed Modules" pattern. It is a real page with content, so it isn't a soft 404, as long as it says more than "not found".
  3. 404 or 410 when there is nothing to say.
  It should never send unmatched URLs to the home page or a section index in bulk.
- A build that removes a page that was previously published can be made to require an explicit decision: a successor, a removal notice, or "gone". Microsoft Learn's build enforces this; it warns when a redirection rule is removed.
- A 410 needs host support that static hosts often lack. Since Google treats 404 and 410 almost the same, a toolchain can default to 404 and offer 410 where the host can send it.

### Gaps
- I found no measurement of how much ranking or traffic a site loses when removed pages are bulk-redirected to the home page, compared with returning 404.
- I found no guidance from AI vendors on whether a 410 makes their crawlers or indexes drop a URL faster than a 404.
