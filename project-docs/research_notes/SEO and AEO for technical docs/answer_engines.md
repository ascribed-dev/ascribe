# Answer engine optimization (AEO / GEO) for technical documentation

Researched 2026-10-10. Every claim carries the date of its source where known. Three evidence classes are used throughout:

- **[OFFICIAL]** the operator's own documentation or a named employee's public statement.
- **[MEASURED]** a study with a stated method and sample; funder named.
- **[ADVICE]** vendor or agency guidance with no published evidence.

"Not confirmed at primary" means I saw the claim only in a search summary or secondary write-up and did not read the original page. Two things I observed directly while researching are marked **[OBSERVED]**; they have no URL.

## 1. The retrieval paths: where each product gets content, which user agents, what controls exist, and whether they run JavaScript

### Takeaway
Every major operator now splits its bots three ways: a training crawler, a search-index crawler, and a user-triggered fetcher; robots.txt governs the first two, and the user-triggered fetchers are documented as "generally" or "may" ignore it (OpenAI, Perplexity, Google) or as honoring it (Anthropic). The only measured study of JavaScript execution (Vercel/MERJ, December 2024) found that no AI crawler except Google's and Apple's renders JavaScript, so a statically rendered docs site is already on the right side of the largest technical divide.

### Cited Findings

**OpenAI / ChatGPT**
- [OFFICIAL] OpenAI documents four agents: `OAI-SearchBot` (search index; opted-out sites are excluded from ChatGPT search answers), `GPTBot` (training), `ChatGPT-User` (user-initiated actions in ChatGPT and Custom GPTs; "robots.txt rules may not apply", and it is "not used to determine whether content may appear in Search"), and `OAI-AdsBot` (ad landing-page checks). IP ranges are published as JSON per bot. A robots.txt change takes "~24 hours" to take effect for search. The page says nothing about JavaScript. — [OpenAI, Overview of OpenAI crawlers](https://developers.openai.com/api/docs/bots) (read 2026-10-10)
- [OFFICIAL] OpenAI's publishers FAQ says ChatGPT adds `utm_source=chatgpt.com` to referral URLs. I could not read the page directly (HTTP 403 to an automated fetch); the sentence is quoted in the page's own URL text fragment as indexed. — [OpenAI Publishers and Developers FAQ](https://help.openai.com/en/articles/12627856-publishers-and-developers-faq)
- [Reported, not confirmed by OpenAI] The Information reported that OpenAI used Google results scraped through SerpApi for some real-time queries; OpenAI's stated position is its own crawler, Bing, and licensed publisher data. — [Search Engine Land](https://searchengineland.com/openai-chatgpt-serpapi-google-search-results-461226) (not confirmed at primary)
- [MEASURED, Ahrefs] In 1.4 million ChatGPT prompts, 88.46% of cited sources were typed "search" (search-index pages) and 1.93% "reddit"; ChatGPT classes sources as search, news, reddit, youtube, academia. — [Ahrefs, Why ChatGPT Cites One Page Over Another](https://ahrefs.com/blog/why-chatgpt-cites-pages/) (seen via search summary; not confirmed at primary)

**Google: AI Overviews, AI Mode, Gemini**
- [OFFICIAL] AI Overviews and AI Mode draw on the ordinary Search index. A page must be "indexed and eligible to be shown in Google Search with a snippet"; there are "no additional requirements to appear in AI Overviews or AI Mode, nor other special optimizations necessary." Both may use "query fan-out", issuing several related searches. Page last updated 2025-12-10. — [Google Search Central, AI features and your website](https://developers.google.com/search/docs/appearance/ai-features)
- [OFFICIAL] Controls for what Search (including its AI features) shows: `nosnippet`, `data-nosnippet`, `max-snippet`, `noindex`. `Google-Extended` limits training and grounding "in other Google systems". Recrawl after a change "can take anywhere from several days to several months." — [same page](https://developers.google.com/search/docs/appearance/ai-features)
- [OFFICIAL] `Google-Extended` is a robots.txt token only, with no separate HTTP user agent; it governs use for training future Gemini models and for grounding (Gemini apps, Vertex AI); it "does not impact a site's inclusion in Google Search" and is not a ranking signal. `Google-CloudVertexBot` crawls on site owners' request for Vertex AI Agents. — [Google common crawlers](https://developers.google.com/search/docs/crawling-indexing/google-common-crawlers)
- [OFFICIAL] Google's user-triggered fetchers "generally ignore robots.txt rules". They include `Google-Agent` (agents on Google infrastructure browsing at a user's request) and `Google-GeminiNotebook` (formerly `Google-NotebookLM`, old token supported until August 2026). — [Google user-triggered fetchers](https://developers.google.com/search/docs/crawling-indexing/google-user-triggered-fetchers)
- Consequence stated by Google's own structure: there is no way to stay in Google Search while opting out of AI Overviews except the snippet controls, which also shrink the ordinary snippet. — [AI features page](https://developers.google.com/search/docs/appearance/ai-features)

**Microsoft Copilot / Bing**
- [OFFICIAL] Copilot answers are grounded on the Bing index; Bing Webmaster Tools reports "grounding queries", the phrases the AI used when retrieving cited content, across Microsoft Copilot, AI summaries in Bing, and select partner integrations. — [Bing Webmaster Blog, 2026-02-10](https://blogs.bing.com/webmaster/February-2026/Introducing-AI-Performance-in-Bing-Webmaster-Tools-Public-Preview)
- [OFFICIAL, individual draft] Microsoft authored an IETF draft on display-based preferences for search and AI crawler usage (`draft-madhavan-aipref-displaybasedpref`, -02 March 2026, not a working-group document). — [IETF datatracker](https://datatracker.ietf.org/doc/draft-madhavan-aipref-displaybasedpref)
- [Secondary] Microsoft retired the public Bing Search API in August 2025, which pushed third parties to other indexes. — [cloro.dev](https://cloro.dev/blog/brave-search-api-vs-serp-api/) (vendor source; not confirmed at Microsoft)

**Perplexity**
- [OFFICIAL] `PerplexityBot` surfaces and links sites in Perplexity search results and "is not used to crawl content for AI foundation models". `Perplexity-User` makes user-requested visits; "Since a user requested the fetch, this fetcher generally ignores robots.txt rules." IP ranges published as JSON. — [Perplexity crawlers](https://docs.perplexity.ai/guides/bots)
- [Secondary] Perplexity runs its own crawler and index (described as hundreds of billions of pages on Vespa). — [The AI Engineer](https://theaiengineer.substack.com/p/how-perplexity-built-their-search) (not confirmed at primary)

**Anthropic / Claude**
- [OFFICIAL] `ClaudeBot` (training), `Claude-User` (fetches at a user's request), `Claude-SearchBot` (indexes to improve search results). All honor robots.txt, including the non-standard `Crawl-delay`. Blocking `Claude-User` "prevents our system from retrieving your content in response to a user query"; blocking `Claude-SearchBot` prevents indexing for search. IP list at `claude.com/crawling/bots.json`; robots.txt must be set per subdomain. — [Anthropic support article](https://support.claude.com/en/articles/8896518-does-anthropic-crawl-data-from-the-web-and-how-can-site-owners-block-the-crawler)
- [Secondary, strong circumstantial] Claude's web search is backed by Brave Search: Brave appears on Anthropic's subprocessor list (added 2025-03-19, still present 2026-07-02 per the source). Anthropic has made no formal statement. — [cloro.dev](https://cloro.dev/blog/brave-search-api-vs-serp-api/) (vendor source)
- [OFFICIAL] The API's web fetch tool takes `max_content_tokens`; content over the limit is truncated; the limit applies to text, not PDFs. — [Claude Platform docs, web fetch tool](https://platform.claude.com/docs/en/agents-and-tools/tool-use/web-fetch-tool)

**Apple, Common Crawl, ByteDance**
- [OFFICIAL] `Applebot-Extended` does not crawl; it is a robots.txt token controlling whether data Applebot collected may train Apple's models. Pages that disallow it can still appear in search. — [Apple, About Applebot](https://support.apple.com/en-us/119829)
- [Reported] ByteDance documents that `Bytespider` follows robots.txt; Fortune (October 2024) and TollBit (first half of 2026) report it reaching disallowed pages. — [ppc.land](https://ppc.land/bytespider/) (secondary)

**Coding agents**
- [OFFICIAL] Codex: web search is on by default for local tasks and served from a **cache of pre-indexed results**; `--search` or `web_search = "live"` enables live browsing; modes are `disabled | cached | indexed | live`. — [OpenAI Codex config reference](https://developers.openai.com/codex/config-reference), [Codex CLI features](https://developers.openai.com/codex/cli/features)
- [OFFICIAL] VS Code Copilot: `#fetch` retrieves a page; since VS Code 1.100 (April 2025) it converts the page to Markdown and includes the whole page where possible; the 1.99 notes (March 2025) say JavaScript is disabled in the fetch window and authenticated pages cannot be fetched. — [VS Code 1.100 notes](https://code.visualstudio.com/updates/v1_100), [VS Code 1.99 notes](https://code.visualstudio.com/updates/v1_99), [Add context to chat](https://code.visualstudio.com/docs/copilot/chat/copilot-chat-context)
- [Forum, Cursor staff] Cursor `@Docs` crawls a public docs URL on Cursor's servers, shares one index across users, converts HTML to Markdown, removes repeated navigation by n-gram deduplication, and chunks at about 500 tokens. Client-rendered sites and sites whose only index is a left-hand navigation panel are reported to index zero or one page. — [Cursor forum: how docs crawling works](https://forum.cursor.com/t/how-does-docs-crawling-work/264), [Indexed 0 pages](https://forum.cursor.com/t/indexed-0-pages/53990) (old thread; current pipeline unverified)
- [OBSERVED 2026-10-10] Claude Code's WebFetch, as presented to this agent: converts the page to Markdown, has a small fast model answer a prompt against it, returns cross-host redirects instead of following them, caches for 15 minutes, and reads 100,000 characters per call (a 223,074-character Google blog page returned only the first 100,000, which did not include the article body).

**JavaScript**
- [MEASURED, Vercel + MERJ, December 2024; Vercel is a hosting vendor] From nextjs.org and Vercel network traffic: GPTBot, ClaudeBot, and PerplexityBot fetch JavaScript files (11.50% of ChatGPT's requests, 23.84% of Claude's) but do not execute them; "none of the major AI crawlers currently render JavaScript". Gemini uses Googlebot's infrastructure and renders fully; AppleBot renders. Monthly volumes: Googlebot 4.5B, GPTBot 569M, Claude 370M, AppleBot 314M, PerplexityBot 24.4M. 404 rates: ChatGPT 34.82%, Claude 34.16%, Googlebot 8.22%. — [Vercel, The rise of the AI crawler](https://vercel.com/blog/the-rise-of-the-ai-crawler)

**Meta tags and `noai`**
- `noai` / `noimageai` are not standardized; Google's robots specification does not list them; no major AI crawler has committed to honoring them. — [Originality.ai dashboard](https://originality.ai/blog/noai-noimageai-adoption-dashboard) (secondary)

### Inferences
- For a static docs site, the things that decide reachability are: allow `OAI-SearchBot`, `Claude-SearchBot`, `PerplexityBot`, `Googlebot`, `bingbot` in robots.txt; serve content in the initial HTML; return real 404s. Everything else in this section is a consequence of an index someone else runs.
- Three different indexes matter for docs (Google's, Bing's, Brave's), plus OpenAI's and Perplexity's own crawls. Being in Bing and Brave is a prerequisite for Copilot and Claude that classic Google-only SEO does not cover.
- The 34% 404 rate for AI fetchers in 2024 and Mintlify's 2026 404 benchmark (section 3) describe the same failure: agents guess URLs. Stable URLs, redirects for moved pages, and a machine-readable map reduce it.

### Gaps
- No operator documents whether its user-triggered fetcher executes JavaScript. The Vercel/MERJ study is nearly two years old and I found no replication dated 2026.
- I found no official documentation for Cursor's `@Web`, Windsurf's web tools, or which search index GitHub Copilot's web search uses.
- Whether Gemini (the app) grounds on anything beyond the Google Search index is not stated beyond the Google-Extended definition.
- I could not confirm CCBot's or Meta's (`meta-externalagent`) current documentation.

## 2. What gets cited: published studies

### Takeaway
The peer-reviewed result (GEO, KDD 2024) shows that text already in the model's context is cited more when it contains statistics, quotations, and source citations; a 2026 survey concludes that nothing has been shown to cause a stable, cross-platform gain in being retrieved in the first place. Industry studies agree only on direction: classic rank still predicts citation but less each year, cited pages skew newer, and the same prompt returns a different source list almost every time. I found no transparent study specific to developer or documentation queries.

### Cited Findings

**Academic**
- [MEASURED, academic: Princeton, Georgia Tech, Allen Institute, IIT Delhi authors] GEO introduces GEO-bench and reports that content changes can raise visibility in generative-engine responses by "up to 40%", with effect varying by domain. Accepted at KDD 2024. — [Aggarwal et al., arXiv:2311.09735](https://arxiv.org/abs/2311.09735)
- [MEASURED, single-author survey, 2026-07-15] The original GEO gains hold where the source is already in a fixed context; they do not establish organic discoverability. The most reproducible levers are topical relevance and position in the context. Generic heuristics transfer poorly; citation-oriented rewrites can impair retrieval; commercial audits show low source overlap and large run-to-run variance. "No reviewed technique shows a stable, longitudinal, cross-platform causal effect" on discoverability or downstream behavior. — [Martinez, arXiv:2607.14035](https://arxiv.org/abs/2607.14035)
- [MEASURED, academic preprint, September 2025] AI search skews toward earned media over brand-owned and social content, and the services differ in domain diversity, freshness, and sensitivity to phrasing. No independent replication found. — [arXiv:2509.08919](https://arxiv.org/abs/2509.08919) (abstract seen via search summary)

**Overlap with classic rankings (Google AI Overviews)**
- [MEASURED, Ahrefs, self-funded, tool vendor; 2025-07-21] 1.9M citations from 1M AI Overviews, top three citations each: 76.10% of cited pages rank in the top 10, 9.50% in positions 11 to 100, 14.40% outside the top 100. Pages cited from outside the top 10 ranked for fewer keywords, which "doesn't neatly fit" the fan-out explanation. — [Ahrefs](https://ahrefs.com/blog/search-rankings-ai-citations)
- [MEASURED, Ahrefs, early 2026] About 863,000 keywords and 4M AI Overview URLs: 38% of cited pages rank in the top 10, 31.2% in 11 to 100, 31.0% outside the top 100. Part of the drop reflects better citation detection in Ahrefs' tooling. — [DesignRush summary](https://news.designrush.com/ai-overview-citations-drop-ahrefs) (not confirmed at primary)
- [MEASURED, Semrush, self-funded, tool vendor; July 2025] Overlap with Google's top 10: Perplexity 91% of domains and 82% of URLs; AI Overviews 86% and 67%; AI Mode 54% and 35%. — [Semrush AI Mode comparison study](https://semrush.com/blog/ai-mode-comparison-study) (not confirmed at primary)
- [MEASURED, BrightEdge, Surfer, Moz; vendors] Lower figures: BrightEdge about 17% of AI Overview citations in the top 10; Surfer, 10,000 keywords, 67.82% of cited pages not in the top 10 for the query or its fan-outs; Moz, 40,000 queries, 88% of AI Mode citations outside the top 10. — [DesignRush summary](https://news.designrush.com/ai-overview-citations-drop-ahrefs), [arXiv search summary of Moz](https://arxiv.org/abs/2607.14035) (none confirmed at primary)

**Freshness**
- [MEASURED, Ahrefs, July 2025] 17M citations across 7 AI platforms: cited URLs average 1,064 days old against 1,432 for organic results (25.7% newer); ChatGPT cites the newest pages, Google AI Overviews the oldest. Correlational. — [Ahrefs](https://ahrefs.com/blog/do-ai-assistants-prefer-to-cite-fresh-content)
- [OFFICIAL, individual statement] Fabrice Canel (Microsoft), commenting in 2025, said generative AI values fresh content, partly as a check on training data, and pointed publishers to IndexNow. — [Search Engine Land](https://searchengineland.com/microsoft-bing-copilot-use-schema-for-its-llms-453455)

**Page structure**
- [MEASURED, vendor; reported by Search Engine Land, 2026] Pages whose headings matched the query most closely were cited 41.0% of the time against about 30% for weaker matches; heading structure otherwise helped only slightly; 4 to 10 subheadings did best; pages over 5,000 words were cited less than pages under 500. — [Search Engine Land](https://searchengineland.com/chatgpt-citations-ranking-precision-length-study-474538) (not confirmed at primary)
- [MEASURED, SE Ranking, tool vendor] 216,524 pages: sections of 120 to 180 words between headings averaged 4.6 ChatGPT citations against 2.7 for sections under 50 words. — [Loganix summary](https://loganix.com/the-anatomy-of-an-ai-citable-page/) (secondary; method not seen)
- [MEASURED, Ahrefs] Content length and citation: r = 0.04. — [same summary](https://loganix.com/the-anatomy-of-an-ai-citable-page/) (secondary)
- [MEASURED, AirOps, vendor, March 2026] 548,534 retrieved pages; ChatGPT left 85% of retrieved pages uncited. — [AirOps](https://airops.com/blog/chatgpt-content-structure) (not confirmed at primary)

**Stability of answers**
- [MEASURED, SparkToro with Gumshoe.ai, an AI-tracking startup; fieldwork November to December 2025] 600 volunteers ran 12 prompts 2,961 times across ChatGPT, Claude, and Google AI Overviews / AI Mode. Under 1 in 100 runs returned the same list of brands; under 1 in 1,000 the same list in the same order. The pool of brands was fairly stable (the top four appeared in 55 to 77% of 994 responses). The authors conclude that position is not a usable metric but appearance rate over many runs may be. The post discloses the conflict of interest. — [SparkToro](https://sparktoro.com/blog/new-research-ais-are-highly-inconsistent-when-recommending-brands-or-products-marketers-should-take-care-when-tracking-ai-visibility/)

**Developer and documentation queries**
- [ADVICE / unverified vendor claim] xSeek, a citation tracker, says Stack Overflow is the dominant citation for programming queries and that GitHub is cited both as documentation and as a forum. No method, sample, or dates published. — [xSeek](https://www.xseek.io/sources/perplexity/stackoverflow)

### Inferences
- The two-stage picture is consistent across sources: retrieval is decided by an ordinary search index (so classic SEO decides whether a page is a candidate), and citation among retrieved pages is influenced by the text itself. A docs toolchain can affect the second stage through what it emits and the first only through crawlability and indexability.
- Query fan-out means a page is retrieved for sub-questions, not the user's question. Docs pages that answer one narrow question each are natural targets, but that is an inference, not a measured result.
- Every structure study above is correlational and vendor-run; the heading-match result is the only one with a plausible mechanism (retrieval matches query text to passage text).

### Gaps
- No study I found isolates developer or documentation queries with a stated method and sample. This is the largest gap for the reader.
- I did not confirm the Seer Interactive, Profound, Authoritas, BrightEdge, or Kevin Indig studies at primary sources; secondary figures for Seer conflict (35%/91% versus 120%/41% CTR lifts) and should not be quoted. — [ppc.land](https://ppc.land/23-factors-that-actually-get-your-content-cited-by-ai-search-engines/)
- GEO-bench's size and the per-technique gains are in the paper body, which I did not read.

## 3. `llms.txt`, `llms-full.txt`, Markdown pages, content negotiation, and the Web Documentation Delivery Spec

### Takeaway
No AI search product reads `llms.txt` to decide what to cite, and Google says so in its documentation; the file's measured readers are coding agents, which fetch it when pointed at it and never speculatively. The one controlled benchmark (Mintlify, July 2026, vendor-run, open source) found that a link to `llms.txt` on each Markdown page cut agents' dead-URL fetches from 2.23 to 0.11 per task without changing accuracy, which is evidence for exactly what this toolchain already ships.

### Cited Findings

**Specification and status**
- `llms.txt` was proposed by Jeremy Howard on 2024-09-03 and is still "a proposal"; the page was last modified 2026-08-10. Required: an H1. Then a blockquote summary, free Markdown without headings, and H2 sections of `[name](url): notes` links; an `## Optional` section marks links an agent may skip. It also proposes a Markdown version of each page at the same URL plus `.md` (`index.html.md` for directory URLs). The page does not define `llms-full.txt`. — [llmstxt.org](https://llmstxt.org/)

**Operators' statements**
- [OFFICIAL, Google, guide published 2026-05-15, note added June 2026] Google Search does not use `llms.txt`; it neither helps nor harms; keeping one is fine if other services use it. The same guide says chunking content for AI is unnecessary and that structured data is not required for generative AI search. — [Google's guide to optimizing for generative AI features](https://developers.google.com/search/docs/fundamentals/ai-optimization-guide), [announcement](https://developers.google.com/search/blog/2026/05/a-new-resource-for-optimizing) (content seen via search summary and Search Engine Roundtable: [seroundtable](https://www.seroundtable.com/google-optimize-generative-ai-mythbusting-41335.html))
- [OFFICIAL] "You don't need to create new machine readable files, AI text files, or markup to appear in these features." — [Google, AI features and your website](https://developers.google.com/search/docs/appearance/ai-features)
- [OFFICIAL, individual] John Mueller, June 2025: "FWIW no AI system currently uses llms.txt." In 2026 he called it "purely speculative for now" and, per Ahrefs, "a temporary crutch, perhaps to save some tokens" for coding tools reading developer docs. — [Search Engine Roundtable](https://seroundtable.com/google-ai-llms-txt-39607.html), [Search Engine Journal](https://www.searchenginejournal.com/google-says-llms-txt-is-purely-speculative-for-now/577576/), [Ahrefs](https://ahrefs.com/blog/llmstxt-study/)
- OpenAI and Anthropic: I found no official statement from either that any of their crawlers or products reads `llms.txt`. Both publish the file on their own docs. — absence noted; see [Ahrefs](https://ahrefs.com/blog/llmstxt-study/)

**Server-log studies**
- [MEASURED, Ahrefs, self-funded; published 2026-06-15] 137,210 domains with traffic in May 2026 (Ahrefs Web Analytics customers, so adoption is an upper bound): 28% publish `llms.txt`; 97% of those files got zero requests in the month; about 22,000 requests in total went to the other 3%. By requester: SEO audit tools 21.7%, general crawlers 13.1%, AI agents and agentic infrastructure 10.5%, AI training crawlers 5.3% (GPTBot alone 4.51%, ClaudeBot 0.8%), AI assistants 2.5%, AI retrieval bots 1.1% (OAI-SearchBot 0.74%). Claude Code out-fetched every AI retrieval bot, assistant, and training crawler except GPTBot. AI bots made zero requests for `llms.txt` on domains that have none: "Agents fetch llms.txt when directed, not speculatively." — [Ahrefs](https://ahrefs.com/blog/llmstxt-study/)
- [MEASURED, smaller, secondary] A 12-week test on 83 sites logged OpenAI's crawler reading `llms.txt` 7 times against 3,990 robots.txt fetches, ClaudeBot 9 against 3,120, PerplexityBot 0 against 775; Meta's crawler read it more than robots.txt. Another log study found 0 verified AI-lab hits in 1,227 requests across 900 domains. — [ppc.land](https://ppc.land/llms-txt-adoption-rises-8-8x-but-97-of-files-get-zero-ai-requests/), [Digital Applied](https://www.digitalapplied.com/blog/llms-txt-in-practice-adoption-evidence-2026) (attribution of each figure to its original study not confirmed)

**Adoption and content**
- [MEASURED, Common Crawl, non-profit; 2026-08-31, July 2026 crawl] 5.17M `/llms.txt` and 1.76M `/llms-full.txt` URLs sampled: 11.72% of `/llms.txt` URLs returned a text body; of `/llms-full.txt`, 11.66% returned 200 and only 7.01% of those had a text body. Of 584,107 files analyzed: 49.90% have the full shape (H1, blockquote, link sections); 22.56% contain no links; 68.27% are templated; Wix generates 41.34%. 6.59% include policy language (rate limits, copyright, citation demands) that the format gives no force: "`llms.txt` is not an access-control mechanism." — [Common Crawl](https://commoncrawl.org/blog/a-content-analysis-of-llms-txt-files-from-the-july-2026-crawl-archive)
- [MEASURED, various] Adoption depends on the sample: 28% of Ahrefs' panel, about 10% of 300,000 domains (SE Ranking), under 0.005% of all sites (Sistrix); Originality.ai counted 4,088 files in June 2025 and 36,120 in May 2026. — [ppc.land](https://ppc.land/llms-txt-adoption-rises-8-8x-but-97-of-files-get-zero-ai-requests/) (secondary)
- Chrome's Lighthouse has an `llms.txt` audit; it made 22 of the requests in Ahrefs' sample. — [Ahrefs](https://ahrefs.com/blog/llmstxt-study/)

**Markdown pages and content negotiation**
- [MEASURED, Mintlify, docs-hosting vendor; 2026-07-17; code at github.com/mintlify/url-discovery-bench] 2,400 runs: 20 Mintlify-hosted sites, 5 questions each, 3 runs, 4 formats, 2 agents (Claude Code and Codex). Average 404s per task: HTML 2.23, plain Markdown 1.42, Markdown with a link to `llms.txt` 0.11. Inlining the whole index into each page removed the same 404s but cost more tokens. Accuracy stayed in the mid-to-high 90s in every arm: "The win was efficiency, not correctness." "Markdown itself wasn't the fix, the map was." Replicated on three more models (for example GPT-5.6: 6.79 to 0.02). The origin of the problem was Mintlify's own Markdown output dropping the navigation, leaving agents to guess URLs. Per-arm accuracy, tokens, and cost are not published. — [Mintlify](https://mintlify.com/blog/llms-txt-agent-benchmark)
- [MEASURED, Mintlify, 2026-07-29] On Mintlify-hosted docs, agents were 66% of measured web traffic in July 2026 (about 213M agent requests against 105M human page loads), up from 15.2% in January; measurement was widened on 2026-06-08, so the rise is partly an artifact. Of agent requests on machine routes, direct `.md` URLs rose from 25.1% (February) to 54.4% (July), `Accept: text/markdown` negotiation fell from 25.2% to 8.3%, and plain text fell from 47.2% to 34.5%. Hosted MCP servers took 1.42M tool calls in July (search 56.57%, filesystem retrieval 43.30%). — [Mintlify, State of docs traffic](https://mintlify.com/blog/state-of-docs-traffic), [Mintlify analytics docs](https://www.mintlify.com/docs/analytics/traffic)
- [OFFICIAL, Cloudflare] "Markdown for Agents" exists: when a request's `Accept` includes `text/markdown`, Cloudflare converts the origin's HTML to Markdown at the edge, adds `Vary: Accept`, and sets `Content-Signal: ai-train=yes, search=yes, ai-input=yes` unless the origin sets its own. Cap: 6 MiB of decompressed HTML. Pro, Business, Enterprise, and SSL for SaaS, at no cost. Docs page last updated 2026-10-09. — [Cloudflare docs](https://developers.cloudflare.com/fundamentals/reference/markdown-for-agents/)
- [Vendor claim] Cloudflare's example: one blog post is 16,180 tokens as HTML and 3,150 as Markdown. Mueller called serving Markdown to bots "a stupid idea" days before the launch. — [Search Engine Journal](https://www.searchenginejournal.com/cloudflares-new-markdown-for-ai-bots-what-you-need-to-know/567339/)
- Claude Code's WebFetch did not send a Markdown-preferring `Accept` header as of an open feature request. — [claudeissues mirror of issue 60856](https://claudeissues.com/issue/60856-webfetch-websearch-support-content-negotiation-accept-header-default-to-accept-t) (third-party mirror; current state unverified)
- Hugging Face serves Markdown automatically when Cursor or Claude Code fetch a docs page. — [Hugging Face changelog](https://huggingface.co/changelog/docs-llms-txt)

**Web Documentation Delivery Spec and `afdocs`**
- The spec defines 28 checks in 7 categories for how a docs site serves agents: content discoverability (7: index exists, is valid, fits one fetch, links resolve, links point to Markdown, directives in HTML and in Markdown point to `llms.txt`), Markdown availability (2: `.md` URLs, `Accept` negotiation), page size (6: client-side rendering detection, Markdown size, HTML size before and after conversion, where content starts, single-fetch completeness), content structure (5: tabbed-content blowup, header quality, code-fence validity, portable links, embedded data), URL stability (2: soft 404s, redirects), observability (3: `llms.txt` coverage, Markdown/HTML parity, cache headers), authentication (3). Thresholds: `llms.txt` under 50,000 characters, pages under 50,000 characters. CC BY 4.0. Companion specs on content and on repository-local docs are planned. `afdocs` is the CLI: `npx afdocs check <url>`. — [agentdocsspec.com](https://agentdocsspec.com/), [github.com/agent-ecosystem/afdocs](https://github.com/agent-ecosystem/afdocs)
- The spec's stated evidence base is its author's observation of agents (cited as 10+ hours validating 578 coding patterns with Claude) and observed failures: truncation, receiving CSS instead of content, cross-host redirects not followed, agents unaware of `llms.txt`. It calls `llms.txt` the most effective discovery mechanism observed. — [agentdocsspec.com](https://agentdocsspec.com/)
- Third-party scorers built on it treat 100,000 characters as the point where "most agents" truncate and recommend section-level indexes under 50,000 characters. — [Mintlify score page](https://www.mintlify.com/score/claude-code.md), [Fern agent score](https://buildwithfern.com/agent-score/company/claude-api)

### Inferences
- `llms.txt` is a navigation aid for an agent that is already on the site, not a discovery or ranking signal. The pointer on each page is the part with evidence; the file at the root alone is not found by anything that matters.
- The benchmark's "accuracy unchanged" result means the benefit is fewer wasted fetches and tokens. That is still worth having, but it should not be sold as better answers.
- `.md` URLs are overtaking content negotiation in real traffic (54% against 8% on Mintlify), so `.md` URLs are the primary mechanism and `Accept` negotiation the secondary; a static host that cannot negotiate loses little.
- The measurable size targets are concrete: keep `llms.txt` and each page under 50,000 characters; split the index by section when it would exceed that. My own fetch tool's 100,000-character window matches the scorers' truncation threshold.
- Candidates worth building that have evidence or a spec check behind them: an index split by section when large; Markdown/HTML parity checked in the build; a check that every `llms.txt` link resolves and points to Markdown; flattening tabbed content in the Markdown output; a size warning per page.

### Gaps
- No measured evidence that `llms-full.txt` is read by anything or improves outcomes; Ahrefs did not measure it, Mintlify's reports do not mention it, and Common Crawl found it mostly absent.
- The Mintlify benchmark covers only Mintlify-hosted sites and two agents; no independent replication.
- I could not read the Web Documentation Delivery Spec's per-agent truncation table (it lives on a background article and the platforms page, which my searches did not retrieve), nor its version or author statement from the landing page.

## 4. Docs-specific delivery: MCP servers, `AGENTS.md`, bundled docs, and what agents' fetch tools do to a page

### Takeaway
The best-evidenced intervention is the cheapest: a compressed docs index placed in `AGENTS.md` that points to docs files the agent can read locally took Vercel's Next.js evals from 53% to 100%, while an on-demand skill was never invoked in 56% of cases. The one independent test of a docs MCP server (Context7 on Terminal-Bench 2.0) found no measurable benefit, because the agent called it in only 6 of 89 tasks.

### Cited Findings

**`AGENTS.md` and docs bundled with the project**
- [MEASURED, Vercel, framework vendor; 2026-01-27] Evals on Next.js 16 APIs absent from training data (`'use cache'`, `connection()`, `forbidden()`, and others), scored on build, lint, and test. Pass rates: no docs 53%; skill, default 53%; skill with explicit instructions 79%; docs index in `AGENTS.md` 100%. The skill was not invoked in 56% of cases without explicit instructions. The index was compressed from about 40 KB to 8 KB (pipe-delimited paths to doc files) with no loss. The injected instruction: "Prefer retrieval-led reasoning over pre-training-led reasoning for any Next.js tasks." The number of evals and the agent and model are not stated. — [Vercel](https://vercel.com/blog/agents-md-outperforms-skills-in-our-agent-evals)
- Vercel's recommendation to framework authors: ship an `AGENTS.md` snippet for users, and structure docs so an agent can read specific files on demand. — [same post](https://vercel.com/blog/agents-md-outperforms-skills-in-our-agent-evals)

**MCP servers for documentation**
- [MEASURED, Margin Lab, independent eval shop; 2026-04-10] Codex with GPT-5.4 on Terminal-Bench 2.0 (89 tasks, one run per configuration): 63/89 without Context7, 64/89 with it plus an `AGENTS.md` directive to use it. The agent called Context7 in 6 of 89 tasks; outcomes in those 6 were identical. "A single data point with one agent, one MCP, and one benchmark." — [Margin Lab](https://marginlab.ai/blog/do-mcps-improve-coding-agents-part-1/)
- [OFFICIAL, product facts] Mintlify hosts an MCP server for each docs site at `/mcp`; GitBook at `/~gitbook/mcp` (HTTP transport, published content only); Cloudflare's docs server is `https://docs.mcp.cloudflare.com/mcp`; Microsoft Learn's is remote, unauthenticated, and returns up to 10 chunks or a full page, with a daily full index refresh; AWS has a managed Knowledge MCP server (generally available October 2025, no account needed). None publishes an accuracy evaluation. — [Mintlify](https://mintlify.com/docs/ai/model-context-protocol), [GitBook](https://gitbook.com/docs/ai-and-search/mcp-servers-for-published-docs), [Cloudflare](https://developers.cloudflare.com/agents/model-context-protocol/mcp-servers-for-cloudflare/), [Microsoft Learn](https://learn.microsoft.com/training/support/mcp), [AWS Insider](https://awsinsider.net/articles/2025/10/03/aws-knowledge-mcp-server-ga-offers-ai-agents-real-time-aws-documentation-and-context.aspx)
- [MEASURED, Mintlify] Usage exists: 1.42M tool calls to hosted docs MCP servers in July 2026; MCP clients expose no conversation identifier, so calls cannot be grouped into tasks. — [Mintlify](https://mintlify.com/blog/state-of-docs-traffic)
- [MEASURED, academic, 2026] The wording of an MCP tool's description changes whether a model selects it. — [arXiv:2602.18914](https://arxiv.org/abs/2602.18914) (seen via search summary)
- Docusaurus has no official `llms.txt` plugin; several community plugins generate `llms.txt` and Markdown pages. — [docusaurus-plugin-llms](https://github.com/rachfop/docusaurus-plugin-llms), [SignalWire plugin](https://github.com/signalwire/docs/pull/307)

**What fetch tools do to a page**
- Claude API web fetch: truncates text at `max_content_tokens` (documented example 100,000); PDFs exempt. — [Claude Platform docs](https://platform.claude.com/docs/en/agents-and-tools/tool-use/web-fetch-tool)
- Claude Code WebFetch: HTML is converted to Markdown, truncated at about 100 KB, and summarized by a small model whose output is all the main model sees; a changelog entry (v2.1.117) moved truncation before conversion; the truncation is not in the official docs. — [Growtika](https://growtika.com/blog/claude-source-code-leak-geo-findings), [claudeissues mirror, docs issue 51783](https://claudeissues.com/issue/51783-docs-webfetch-docs-missing-large-html-truncation-behavior), [issue 22937](https://claudeissues.com/issue/22937-webfetch-tool-truncates-large-documentation-pages-preventing-access-to-critical) (third-party sources; consistent with the **[OBSERVED]** behavior in section 1)
- VS Code Copilot `#fetch`: converts to Markdown, includes the whole page "where possible", caches, asks for confirmation. — [VS Code 1.100 notes](https://code.visualstudio.com/updates/v1_100)
- Codex: answers from a search cache by default, not from a live fetch. — [Codex config reference](https://developers.openai.com/codex/config-reference)
- Cursor `@Docs`: about 500-token chunks after deduplicating navigation. — [Cursor forum](https://forum.cursor.com/t/how-does-docs-crawling-work/264)
- [Reported] A study of MCP middleware telemetry across Claude Code, Cursor, Codex, Copilot, and Aider found zero agent-initiated pagination requests after an over-budget tool response, and that truncation is silent. — [Daniel Vaughan's Codex blog](https://codex.danielvaughan.com/2026/08/29/agents-dont-paginate-first-chunk-selection-tool-response-codex-cli/) (secondary; original paper not located)
- Inline CSS ahead of content can make a short page invisible to an agent, since the fetch window fills before the content starts. — [agentdocsspec.com](https://agentdocsspec.com/)

### Inferences
- Both measured results point the same way: agents use what is already in front of them and under-use what they must decide to call. An index in `AGENTS.md` and a pointer on each page are passive; a skill or MCP server is active and often skipped.
- For a docs toolchain, the build-worthy item is therefore an emitted, compressed docs index sized for `AGENTS.md` (Vercel's was 8 KB) that maps topics to Markdown files, plus a way to ship the Markdown with the package so the files are local. An MCP server is the least-evidenced item on the list.
- Because fetch tools keep only the first window and agents do not paginate, the order of content matters: the answer and the page's own content must come before navigation, styles, and embedded data. In Markdown output this is automatic; in HTML it argues for content early in the document.
- A summarizing fetch tool means the main model may never see exact text. Verbatim details (flags, parameter names) survive better when they are in short, plainly labelled passages, but that is inference.

### Gaps
- Vercel's eval sizes and model are unpublished; it is a single first-party result on one framework.
- No evaluation at all for Mintlify, GitBook, Microsoft Learn, AWS, or Cloudflare docs MCP servers' effect on answers.
- No operator-published truncation sizes for Cursor, Copilot, Windsurf, or Codex fetches.
- No measurement of docs bundled in packages (for example in `node_modules`) as distinct from Vercel's `AGENTS.md` index.

## 5. Content properties: which have evidence

### Takeaway
Almost every content rule in circulation is advice. The properties with any measurement behind them are: statistics, quotations, and citations in the text (academic, in-context only); headings that match the query's wording (vendor, correlational); recency (vendor, correlational); and page size and content position (mechanical, follows from truncation). Google states that chunking pages for AI is unnecessary.

### Cited Findings

| Property | Evidence class | Source |
|---|---|---|
| Statistics, quotations, cited sources in the text | MEASURED, academic; effect shown only once the page is in context | [GEO, arXiv:2311.09735](https://arxiv.org/abs/2311.09735); limits in [arXiv:2607.14035](https://arxiv.org/abs/2607.14035) |
| Topical relevance; position in the context | MEASURED, the "most reproducible levers" | [arXiv:2607.14035](https://arxiv.org/abs/2607.14035) |
| Headings worded like the question | MEASURED, vendor, correlational (41.0% against about 30% cited) | [Search Engine Land](https://searchengineland.com/chatgpt-citations-ranking-precision-length-study-474538) (secondary) |
| Section length of 120 to 180 words | MEASURED, vendor, correlational; method not seen | [Loganix summary of SE Ranking](https://loganix.com/the-anatomy-of-an-ai-citable-page/) |
| Page length | MEASURED, no effect (r = 0.04) or negative above 5,000 words | [Loganix summary of Ahrefs](https://loganix.com/the-anatomy-of-an-ai-citable-page/), [Search Engine Land](https://searchengineland.com/chatgpt-citations-ranking-precision-length-study-474538) |
| Recency / updated dates | MEASURED, vendor, correlational (cited URLs 25.7% newer) | [Ahrefs](https://ahrefs.com/blog/do-ai-assistants-prefer-to-cite-fresh-content) |
| Small pages; content early in the document | Mechanical: follows from documented truncation | [Claude web fetch docs](https://platform.claude.com/docs/en/agents-and-tools/tool-use/web-fetch-tool), [agentdocsspec.com](https://agentdocsspec.com/) |
| Stable URLs, redirects, real 404s | MEASURED indirectly: AI fetchers hit 404s on 34% of requests; a URL map cut agent 404s by about 95% | [Vercel/MERJ](https://vercel.com/blog/the-rise-of-the-ai-crawler), [Mintlify](https://mintlify.com/blog/llms-txt-agent-benchmark) |
| Structured, complete API documentation | MEASURED, academic: raises executable code from 42.55% to at most 66.36% | [arXiv:2604.09515](https://arxiv.org/abs/2604.09515) |
| Chunking a page into small pieces | OFFICIAL: Google says not needed | [Google AI optimization guide](https://developers.google.com/search/docs/fundamentals/ai-optimization-guide) |
| Rewriting content specially for AI | OFFICIAL: Google lists it as unnecessary; academic: citation-oriented rewrites can impair retrieval | [seroundtable](https://www.seroundtable.com/google-optimize-generative-ai-mythbusting-41335.html), [arXiv:2607.14035](https://arxiv.org/abs/2607.14035) |
| Tabbed content flattened in Markdown; valid code fences; portable links | Spec check, author's observation; no study | [agentdocsspec.com](https://agentdocsspec.com/) |
| Self-contained sections, answer-first paragraphs, tables, definitions, complete runnable code, consistent terminology, one place per fact | ADVICE only | for example [stackmatix](https://www.stackmatix.com/blog/developer-docs-aeo-citations) |

- Google's content guidance for AI features is one phrase: "non-commodity" content that offers something beyond common knowledge. — [Google AI optimization guide](https://developers.google.com/search/docs/fundamentals/ai-optimization-guide) (via search summary)
- A clean heading hierarchy as a 2.8x predictor of citation is claimed by one vendor study (Nobori.ai, 50,431 citations, 13 weeks). — [Loganix summary](https://loganix.com/the-anatomy-of-an-ai-citable-page/) (secondary; method not seen)

### Inferences
- The advice-only properties are not wrong for lacking a study; several have a mechanism. Retrieval returns passages, so a section that needs its neighbors to make sense loses information when lifted alone; chunk boundaries (about 500 tokens in Cursor) fall on headings at best. That argues for self-contained sections without proving it.
- "One place per fact" and consistent terminology reduce contradiction between retrieved passages, which matters given the context-memory conflict result in section 6, but nothing measures it.
- A toolchain can check the mechanical properties (size, content start, parity, link validity, code-fence validity) and cannot check the rhetorical ones. The mechanical ones are also the ones with evidence.

### Gaps
- No controlled study of answer-first paragraphs, tables, definition lists, anchors, or runnable code samples on citation or on agent task success.
- No study of stable heading anchors and deep links in AI citations (whether products cite to a fragment or to the page).

## 6. Correctness problems specific to docs: versions, deprecated APIs, hallucinated parameters

### Takeaway
Academic work measures the problem well and the cures poorly: models default to the API they were trained on, and supplying documentation helps but does not fully override memory (executable output rises from about 43% to at most 66%). I found no measurement of version labels in text, metadata, or URLs as a mitigation; the supporting evidence is indirect, from Vercel's result that an explicit "prefer retrieval over training" instruction plus version-matched docs reached 100%.

### Cited Findings
- [MEASURED, academic, June 2024] Seven LLMs, 145 API mappings from eight Python libraries, 28,125 completion prompts: models use deprecated APIs; causes traced to model, prompt, and library factors; two light fixes proposed (replace the API after generation; insert a prompt). A secondary source gives deprecated usage of 25 to 38% across models; not confirmed in the paper. — [Wang et al., arXiv:2406.09834](https://arxiv.org/abs/2406.09834)
- [MEASURED, academic, 2026-04-10] 270 real API updates from eight Python libraries, 11 models from four families: without comprehensive documentation only 42.55% of generated examples run in the target environment; structured documentation and larger models raise this to at most 66.36%; outdated patterns persist "even when API update specifications are provided" (context-memory conflict); self-reflection adds about 11%. — [Ashik et al., arXiv:2604.09515](https://arxiv.org/abs/2604.09515)
- [MEASURED, academic] GitChameleon 2.0: state-of-the-art models reach about 48 to 51% on version-conditioned generation. VersiCode is a reusable benchmark for version-specific generation. — cited in [arXiv:2604.09515](https://arxiv.org/abs/2604.09515); [VersiCode, arXiv:2406.07411](https://arxiv.org/abs/2406.07411)
- [MEASURED, Vercel] On APIs newer than the training data, baseline 53%; version-matched docs index plus the retrieval-over-training instruction 100%. — [Vercel](https://vercel.com/blog/agents-md-outperforms-skills-in-our-agent-evals)
- [Tool paper] DocAware: version-specific documentation retrieval raised F1 from 70.9% to 86.8% on code review across five JavaScript libraries. — reported in search results; original not located, cite with caution: [arXiv:2604.09515 related work](https://arxiv.org/abs/2604.09515)
- [OFFICIAL, product claims] Context7 positions itself as serving "version-specific documentation"; Microsoft Learn's MCP refreshes its index daily; GitBook's MCP serves the latest published version only. — [Context7 overview](https://www.mintlify.com/upstash/context7/mcp/overview), [Microsoft Learn](https://learn.microsoft.com/training/support/mcp), [GitBook](https://gitbook.com/docs/ai-and-search/mcp-servers-for-published-docs)
- Codex's default cached search means an agent may answer from an index snapshot older than the live docs. — [Codex config reference](https://developers.openai.com/codex/config-reference)
- Vercel/MERJ warned in 2024 that freshness in assistants is inconsistent and some rely on cached or training data. — [Vercel](https://vercel.com/blog/the-rise-of-the-ai-crawler)

### Inferences
- Since memory beats context when they conflict, a page that only states the new API is weaker than one that also names the old form and says it is removed or replaced as of a version. Deprecation and migration notes placed next to the current API are the docs-side analog of the papers' "insert prompt" fix. This is inference; nothing measures it for docs pages.
- A per-version index (one `llms.txt` per version, or version in the URL and in the page's first lines) lets an agent match docs to the installed version. A single index that mixes versions invites the mixing failure. Shipping the docs index with the package ties version to version by construction, which is what Vercel's approach does.
- With GitBook-style "latest only" delivery, users on older versions get wrong answers confidently; a toolchain that emits versioned Markdown avoids that.
- Mixing two products' docs is a retrieval-relevance failure; the only measured lever is topical relevance, which suggests product and version names in headings and first sentences, not only in site chrome that Markdown output drops.

### Gaps
- No study of version labels in body text versus metadata versus URL, or of `rel=canonical` across versions, on AI answer correctness.
- No published vendor data on how often AI answers cite an outdated docs version.
- No study of hallucinated parameters as a function of reference-page structure.

## 7. Structured data and AI

### Takeaway
Google says no schema.org markup is needed for its AI features and warns against over-focusing on it; Microsoft said in March 2025 that schema helps its LLMs understand content. The one controlled test (Ahrefs, May 2026) found adding JSON-LD to 1,885 pages produced no significant gain in citations on any platform.

### Cited Findings
- [OFFICIAL, Google, updated 2025-12-10] "There's also no special schema.org structured data that you need to add." Structured data should match visible text. — [AI features and your website](https://developers.google.com/search/docs/appearance/ai-features)
- [OFFICIAL, Google, May to July 2026] Structured data "isn't required for generative AI search"; keep using it for rich-result eligibility; the mythbusting list includes over-focusing on structured data for AI. — [Google AI optimization guide](https://developers.google.com/search/docs/fundamentals/ai-optimization-guide), [Search Engine Land](https://searchengineland.com/google-publishes-guide-on-optimizing-for-generative-ai-features-477671)
- [OFFICIAL, individual] Ryan Levering (Google), Search Central Live New York, March 2025: "A lot of our systems run much better with structured data. It's computationally cheaper." — [analyzify summary](https://analyzify.com/hub/structured-data-ai-visibility) (secondary)
- [OFFICIAL, individual] Fabrice Canel (Microsoft), SMX Munich, March 2025: schema markup helps Microsoft's LLMs understand content. I found no 2026 update from Microsoft. — [Search Engine Land](https://searchengineland.com/microsoft-bing-copilot-use-schema-for-its-llms-453455)
- [MEASURED, Ahrefs, self-funded; May 2026] Cited pages were almost three times as likely to have JSON-LD (correlation). In a matched test of 1,885 pages that added JSON-LD against control pages that did not: AI Mode +2.4% and ChatGPT +2.2% (neither significant), AI Overviews −4.6% (significant, not attributed to schema). — [Ahrefs](https://ahrefs.com/blog/schema-ai-citations/), [Search Engine Journal](https://searchenginejournal.com/schema-markup-didnt-move-ai-citations-in-ahrefs-test/574568) (seen via search summary)
- [MEASURED, small] searchVIU, October 2025: none of five AI systems extracted a price present only in JSON-LD. Otterly: 6 of 7 AI platforms could not retrieve or interpret schema when queried directly. — [geotoolbox summary](https://geotoolbox.ai/blog/schema-markup-for-ai.md) (secondary)
- Circulating figures such as "300% higher accuracy" or "30% higher visibility" from schema have no traceable source. — [same summary](https://geotoolbox.ai/blog/schema-markup-for-ai.md)

### Inferences
- User-triggered fetchers convert HTML to Markdown, which discards JSON-LD entirely. Anything an agent must know (version, date, product) has to be in visible text; metadata alone does not reach it. This agrees with the searchVIU result.
- Structured data remains a classic-search matter (rich results, entity understanding in the index), which the other researcher covers; as an AEO lever it has a negative controlled result.

### Gaps
- No controlled test for `TechArticle`, `APIReference`, `SoftwareSourceCode`, or `FAQPage` on documentation specifically.
- No Bing or Microsoft statement in 2026 found.

## 8. Measurement: seeing AI traffic and citations

### Takeaway
Three first-party sources now exist: Bing Webmaster Tools' AI Performance report (citations, cited pages, grounding queries; no clicks), Google Search Console's generative AI report (impressions only, AI Overviews and AI Mode combined, partial rollout), and server logs filtered by documented user agents. Third-party "AI visibility" trackers measure a distribution, not a position; a single-run rank is noise.

### Cited Findings

**First-party reports**
- [OFFICIAL, Microsoft; public preview 2026-02-10] AI Performance in Bing Webmaster Tools reports total citations, average cited pages per day, page-level citation counts, and grounding queries (a sample), across Copilot, AI summaries in Bing, and select partners. Citation counts do not indicate ranking or placement. No click data. — [Bing Webmaster Blog](https://blogs.bing.com/webmaster/February-2026/Introducing-AI-Performance-in-Bing-Webmaster-Tools-Public-Preview)
- [OFFICIAL, Microsoft; June 2026 preview] Added Intents, Topics, Citation Share (the site's share of citations for a grounding query, "observational", not a ranking), and Compare. — [Bing blog](https://blogs.bing.com/search/June-2026/New-AI-Visibility-Insights-in-Bing-Webmaster-Tools-Intents-Topics-Citation-Share-Compare)
- CSV export exists; no API (on the backlog per Fabrice Canel). One publisher saw about 48,000 citations and 14 Copilot clicks. — [Search Influence](https://www.searchinfluence.com/blog/bing-ai-performance-report-copilot-citations/) (secondary)
- [OFFICIAL, Google; June 2026] Search Console added Search Generative AI performance reports: impressions in generative AI features, with a Search view (AI Overviews and AI Mode together) and a Discover view; rolling out to a subset of sites; the same data remains inside the overall Performance report. — [Google Search Central blog](https://developers.google.com/search/blog/2026/06/gen-ai-performance-reports) (title and date confirmed; body read via secondary coverage: [ppc.land](https://ppc.land/google-finally-gives-search-console-its-own-generative-ai-visibility-reports/))
- [OFFICIAL, Google] AI feature traffic is counted in the Performance report under the "Web" search type. Google cautions that "no third-party tool has access to our internal ranking or AI systems." — [AI features page](https://developers.google.com/search/docs/appearance/ai-features), [AI optimization guide](https://developers.google.com/search/docs/fundamentals/ai-optimization-guide)

**Referrers**
- [OFFICIAL] ChatGPT appends `utm_source=chatgpt.com` to outbound links. — [OpenAI Publishers FAQ](https://help.openai.com/en/articles/12627856-publishers-and-developers-faq)
- [ADVICE / vendor claims] Mobile apps, in-app browsers, and copy-paste strip the referrer, so analytics counts are a floor; a figure of "over 70%" of AI referrals arriving without a referrer circulates from one vendor report and is untraced. — [anglera](https://www.anglera.com/blog/ai-referral-traffic-measurement), [authoritytech](https://authoritytech.io/curated/ai-search-traffic-invisible-attribution-analytics-measurement-2026) (secondary)
- [MEASURED, Cloudflare Radar via third parties] Crawl-to-refer ratios, July 2026: Anthropic 1,917:1, OpenAI 251:1, Google 4.7:1. The ratio lumps an operator's training crawler with its user fetcher, so it overstates. chatgpt.com supplied 0.913% of referrals against Google's 88.08%. — [ecorpit summary](https://ecorpit.com/ai-crawler-crawl-to-referral-ratio-2026-geo-decision/) (not confirmed at Cloudflare Radar; figures differ between secondary sources)

**Logs**
- Every operator in section 1 publishes its user-agent tokens and IP ranges, so log lines can be classified as training, search-index, or user-triggered, and verified by IP. — [OpenAI](https://developers.openai.com/api/docs/bots), [Anthropic](https://support.claude.com/en/articles/8896518-does-anthropic-crawl-data-from-the-web-and-how-can-site-owners-block-the-crawler), [Perplexity](https://docs.perplexity.ai/guides/bots), [Google](https://developers.google.com/search/docs/crawling-indexing/google-user-triggered-fetchers)
- [MEASURED, Mintlify] Limits of log attribution: services with browser user agents count as humans; a known signature cannot show whether the request came from the product's own search or an intermediary (Parallel, Exa); a client switching to Markdown routes raises "agent share" with no change in tasks; total requests fall as agents get more accurate, so "wasted work" (404s per task) is the better metric. Claude-User traffic falls almost 60% on weekends. — [Mintlify](https://mintlify.com/blog/state-of-docs-traffic)
- Ahrefs separated agents from crawlers by user agent in logs, including a `Claude-Code` signature. — [Ahrefs](https://ahrefs.com/blog/llmstxt-study/)

**Third-party trackers**
- [MEASURED, SparkToro/Gumshoe] Identical prompts return the same brand list under 1% of the time; a tracker that runs a prompt once and reports a position reports noise. Appearance rate over many runs is more stable. — [SparkToro](https://sparktoro.com/blog/new-research-ais-are-highly-inconsistent-when-recommending-brands-or-products-marketers-should-take-care-when-tracking-ai-visibility/)
- [MEASURED, academic survey] Commercial audits show "low source overlap, substantial run-to-run variability". — [arXiv:2607.14035](https://arxiv.org/abs/2607.14035)

### Inferences
- What a static host can measure without a server: (1) requests to `.md` URLs and to `llms.txt` by user agent, if the host exposes access logs or a CDN log export; (2) referrers and the `utm_source=chatgpt.com` parameter in client-side analytics, as a floor; (3) Bing's AI Performance export and Search Console's report, both off-site. If the host exposes no logs at all, items 2 and 3 are all there is.
- Because `.md` URLs are fetched almost only by agents, their request count is a cleaner agent signal than user-agent sniffing on HTML pages. A toolchain that emits `.md` pages has already created the measurement surface.
- Bing's grounding queries are the only first-party view of what an AI actually searched for to reach a docs page; they are the closest thing to fan-out queries anyone publishes.
- A docs team's own test is feasible and better than a tracker: a fixed set of product questions run many times per product, scoring whether the answer is correct for the current version. That measures the thing that matters (correctness), which citation counts do not.

### Gaps
- I could not read the body of Google's June 2026 Search Console announcement (the page exceeded my fetch window), so the absence of clicks and queries in the report rests on secondary coverage.
- No first-party citation report exists from OpenAI, Anthropic, or Perplexity that I could find.
- Which static hosts (GitHub Pages, Cloudflare Pages, Netlify) expose which logs was not researched here.

## 9. Access and cost: crawl volume, blocking, and what blocking does to visibility

### Takeaway
Docs hosts have published real costs (Read the Docs: one crawler took 73 TB in a month), and Cloudflare now blocks training and agent bots by default on ad-bearing pages for new and free-plan sites. For a docs site that wants to be used by agents, the risk has inverted: a default bot rule or WAF challenge can silently block `Claude-User`, `ChatGPT-User`, and coding agents.

### Cited Findings
- [MEASURED, Read the Docs, docs host; 2024-07-25] One AI crawler downloaded 73 TB of zipped HTML in May 2024, almost 10 TB in one day, costing over $5,000 in bandwidth; a bug made it re-download the same files. — [Read the Docs](https://about.readthedocs.com/blog/2024/07/ai-crawlers-abuse/), via [Simon Willison](https://simonwillison.net/2024/Jul/25/ai-crawlers-need-to-be-more-respectful)
- [MEASURED, Wikimedia Foundation; April 2025] Bandwidth for multimedia downloads rose 50% since January 2024, attributed to scrapers; 65% of the most expensive traffic came from bots, which were about 35% of pageviews. — [TechCrunch](https://techcrunch.com/2025/04/02/ai-crawlers-cause-wikimedia-commons-bandwidth-demands-to-surge-50)
- [MEASURED, Vercel/MERJ] AI crawlers together were about 28% of Googlebot's volume on Vercel's network in late 2024. — [Vercel](https://vercel.com/blog/the-rise-of-the-ai-crawler)
- [MEASURED, Cloudflare Radar via third party, Q1 2026] 89.4% of AI crawler traffic was training or mixed-purpose, 8% search, 2.2% user-triggered. — [softwareseni summary](https://www.softwareseni.com/the-numbers-behind-ai-crawling-what-cloudflare-radar-reveals-about-who-takes-and-who-gives-back) (not confirmed at primary)
- [OFFICIAL, Cloudflare] 2025-07-01: new domains block known AI crawlers by default; pay per crawl introduced. — [Cloudflare](https://blog.cloudflare.com/content-independence-day-no-ai-crawl-without-compensation/), [pay per crawl](https://blog.cloudflare.com/introducing-pay-per-crawl/)
- [OFFICIAL, Cloudflare; 2026-07-01] All customers can manage AI bots by behavior in three classes: Search, Agent, Training. From 2026-09-15, new domains (and, per TechCrunch, all existing free customers) block Training and Agent on pages that display ads; Search stays allowed. Rules apply by the most restrictive class, so multi-purpose crawlers such as Googlebot, Applebot, and BingBot are blocked for customers who block Training. Pay per crawl became Pay Per Use (beta; partners Ceramic.ai and You.com). — [Cloudflare](https://blog.cloudflare.com/content-independence-day-ai-options/), [Pay Per Use](https://blog.cloudflare.com/pay-per-use/), [changelog](https://developers.cloudflare.com/changelog/post/2026-07-01-ai-traffic-options/), [TechCrunch](https://techcrunch.com/2026/07/01/cloudflares-new-policy-pushes-ai-companies-to-pay-for-publishers-content/)
- Anthropic honors `Crawl-delay`; OpenAI and Anthropic warn that blocking by IP is less reliable than robots.txt. — [Anthropic](https://support.claude.com/en/articles/8896518-does-anthropic-crawl-data-from-the-web-and-how-can-site-owners-block-the-crawler)
- [MEASURED, effect of blocking; news publishers, not docs]
  - BuzzStream (link-building vendor, March 2026): about 4M citations over 3,600 prompts; among top-50 news sites, 70.6% of those blocking `ChatGPT-User`, 82.4% blocking `OAI-SearchBot`, and 88.2% blocking `GPTBot` still appeared in citations. — [ppc.land](https://ppc.land/blocking-ai-crawlers-doesnt-stop-citations-new-data-shows-why/)
  - cloro (data vendor, July 2026): 1,058 domains; median ChatGPT citations per Google ranking were 0.003 for GPTBot-blocking domains against 0.417 for others; correlational and provider-specific. — [cloro](https://cloro.dev/research/ai-crawler-blocks/)
  - Goodie (AEO vendor, July 2026): 31M citations, 105 publishers; blocking works against labs that honor it and not against Grok, AI Overviews, or DeepSeek. — [Goodie](https://higoodie.com/blog/publishers-and-ai-search-study/)
  - Wharton and Rutgers researchers: about a 7% fall in weekly human visits within six weeks of blocking. — [ppc.land](https://ppc.land/blocking-ai-crawlers-doesnt-stop-citations-new-data-shows-why/) (secondary)
- The Web Documentation Delivery Spec has a check for bot protection interfering with automated fetches. — [agentdocsspec.com](https://agentdocsspec.com/)
- [OBSERVED 2026-10-10] help.openai.com returned HTTP 403 to this agent's fetch: an operator's own help center blocked an AI agent reading its bot documentation.

### Inferences
- The three studies of blocking disagree because they measure different things (appearing at all versus citation rate) on news sites. For docs, the relevant fact is from operators' own documentation: blocking `OAI-SearchBot` or `Claude-SearchBot` removes the site from that product's search answers.
- Blocking training crawlers (`GPTBot`, `ClaudeBot`, `Google-Extended`) is the choice with a real trade-off for docs: it saves bandwidth but removes the docs from future models' memory, and section 6 shows memory beats context. A project that wants agents to write correct code for it has a reason to allow training crawls.
- A docs toolchain can emit a robots.txt with the documented tokens grouped by purpose, and its documentation should warn that a CDN's default AI rules can override it. On Cloudflare the September 2026 default targets ad-bearing pages, so most docs sites are unaffected unless they are on a free plan with ads or have opted to block Training.
- Serving `.md` pages cuts bytes per agent request several-fold (Cloudflare's 16,180 to 3,150 tokens example), which is also the cheapest rate-limit.

### Gaps
- No 2025 or 2026 crawl-volume figures from Read the Docs, GitBook, or Docusaurus-hosted sites found; Mintlify's are in section 3.
- No study of blocking's effect on docs sites specifically.
- I did not confirm how the 2026-09-15 Cloudflare rollout went.

## 10. The standards in flux

### Takeaway
As of October 2026 nothing beyond RFC 9309 robots.txt and each operator's own user-agent tokens is honored by a major AI operator. The IETF AI Preferences drafts are still working-group documents past their August 2026 milestone; Cloudflare's Content Signals are deployed on millions of robots.txt files and, per Google's John Mueller, used by no crawler.

### Cited Findings
- [IETF, datatracker read 2026-10-10] Working-group documents: `draft-ietf-aipref-vocab-08` (2026-09-13) and `draft-ietf-aipref-attach-05` (2026-08-18), both intended Proposed Standard, neither yet with the IESG. Seven related individual drafts exist, among them `draft-illyes-aipref-cbcp-04` (a Google author) and drafts on compliance, exclusions, and parameters. — [IETF aipref documents](https://datatracker.ietf.org/wg/aipref/documents/)
- The attachment draft defines a `Content-Usage` HTTP header and updates RFC 9309 so robots.txt can carry usage preferences; the working group's milestone to submit to the IESG was August 2026. — [draft-ietf-aipref-attach](https://datatracker.ietf.org/doc/draft-ietf-aipref-attach/)
- Cloudflare's Content Signals (`ai-train`, `search`, `ai-input`) were submitted as an individual draft in October 2025 and are the default in Cloudflare-managed robots.txt since 2025-09-24, on a reported 3.8M+ domains. — [draft-romm-aipref-contentsignals-00](https://datatracker.ietf.org/doc/html/draft-romm-aipref-contentsignals-00), [Lumar](https://agentic-readiness.lumar.io/docs/access/content-signal.md) (domain count from a third-party tracker)
- [OFFICIAL, individual; 2026-07-06] John Mueller on Reddit: the content-signal directive has "no effects whatsoever for any crawler or LLM" to his knowledge and adds bloat to robots.txt. Informal, hedged. — [Search Engine Roundtable](https://seroundtable.com/google-cloudflare-content-signals-41631.html)
- Cloudflare's Markdown for Agents sends `Content-Signal: ai-train=yes, search=yes, ai-input=yes` by default on converted responses. — [Cloudflare docs](https://developers.cloudflare.com/fundamentals/reference/markdown-for-agents/)
- Really Simple Licensing (RSL) 1.0 was released 2025-12-10; publishers including Reddit, Yahoo, Medium, and O'Reilly support it; Cloudflare, Akamai, and Fastly can enforce it at the network level; no major model developer has committed to honor it (as of a June 2026 report). — [Creative Commons](https://creativecommons.org/2025/12/10/integrating-choices-in-open-standards/), [rslstandard.org](https://rslstandard.org/), [TechTarget](https://www.techtarget.com/it-strategy/feature/What-CIOs-need-to-know-about-the-RSL-protocol)
- TDM Reservation Protocol is a W3C Community Group report (not a W3C standard), tied to Article 4 of the EU DSM Directive; latest vocabulary revision February 2024, final report 2024-05-10. No AI operator commitment found. — [W3C TDMRep](https://www.w3.org/ns/tdmrep), [community group](https://www.w3.org/groups/cg/tdmrep)
- `noai` / `noimageai`: not standardized, not honored by commitment. — [Originality.ai](https://originality.ai/blog/noai-noimageai-adoption-dashboard)
- Mueller pointed site owners toward WebMCP rather than `llms.txt` as the agent-facing direction. — [Search Engine Journal](https://www.searchenginejournal.com/google-says-llms-txt-is-purely-speculative-for-now/577576/)

### Inferences
- A toolchain should not bake any of these in as a contract yet. A pass-through for custom robots.txt lines and custom response headers covers `Content-Signal` and a future `Content-Usage` without committing to a vocabulary that is still at draft -08.
- The durable pattern across operators is the three-way split (train / search / user-triggered). AIPREF's vocabulary and Cloudflare's classes both formalize it, so a configuration model with those three purposes will likely map onto whatever ships.

### Gaps
- I did not read the current text of `draft-ietf-aipref-vocab-08`, so its category names are not recorded here.
- No information found on WebMCP's status, or on agent-facing manifests beyond `llms.txt` and MCP server discovery.
- Whether any operator has stated it will honor `Content-Usage` once published: none found.
