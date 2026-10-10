# Paid tier and business models for an open-source docs tool (evidence as of October 2026)

Notes on method and reliability, which apply to every section:

- All prices and pages were viewed on 3 October 2026 unless another date is given. Prices are in US dollars.
- "Fetched" means the page itself was retrieved and read. "Search summary" means the figure came from a search-engine summary of the listed page and was not confirmed by reading the page; treat those as less certain.
- Vendor pricing pages are primary sources for price but are self-interested for everything else. Several comparison articles cited here are published by competitors (Scalar, Fern, Ferndesk, Docsie, docsio.co, documentation.ai) and are flagged where used.
- A correction to the background brief: Doctave's announcement is dated 31 August 2026, and the shutdown date it gives is 14 September 2026, not 31 August (see the Doctave section).

## 1. Current pricing and packaging of docs platforms (October 2026)

### Takeaway
Hosted docs platforms cluster in two shapes: a free tier plus a flat $150 to $450 a month step per site (Mintlify, Fern, Scalar, Theneo, ReadMe, GitBook Ultimate), or low per-seat pricing with SSO and analytics held back for a higher tier (Redocly at $10 and $24 a seat). Preview deployments are gated at Mintlify but included in every plan at Read the Docs and (before its closure) Doctave; AI is almost always metered in credits or sold as an add-on, and SAML SSO is almost always enterprise-only.

### Cited Findings

Mintlify (fetched; the page's numerals rendered garbled, so prices are cross-checked against third parties)
- Starter is free with 5 editor seats, custom domain, web editor, API playground and MCP server; no AI assistant and no preview deployments. Pro has unlimited editor seats and adds agent, assistant, automations, preview deployments and admin APIs. Enterprise is "Contact us" and adds SSO, SCIM, RBAC, SLA and advanced insights — [Mintlify pricing](https://www.mintlify.com/pricing)
- AI metering on Pro: 10,000 credits a month included, "$0.01 per credit for overages"; an Assistant answer costs 25 credits and is "Free when the Assistant can't answer"; an automation (doc update) costs 250 credits per update and is "Free when nothing needs updating". At list overage price that is $0.25 per answer and $2.50 per automated doc update — [Mintlify pricing](https://www.mintlify.com/pricing)
- Pro is $450 a month billed annually, per deployment, as of September 2026; "since 8 September only two things spend [credits]: AI Assistant answers at 25 credits each, and automation runs that update your docs at 250"; the 14-day trial includes up to 5,000 AI credits (search summary of competitor and third-party write-ups; self-interested sources) — [Fern, "Mintlify reviews, pricing, and alternatives (September 2026)"](https://buildwithfern.com/post/mintlify-reviews-pricing-alternatives); [Ferndesk, Mintlify pricing](https://ferndesk.com/blog/mintlify-pricing)
- Conflict: one aggregator headline still lists "$150/mo Pro" for Mintlify, which appears to be an older price; the vendor page and September 2026 write-ups agree on $450 — [Automation Atlas](https://automationatlas.io/tools/mintlify/)

GitBook (fetched)
- Free: $0 per site, 1 user, change requests included, basic commenting, GitBook Agent limited to "10 message limit per week", no custom domain. Premium: "$65 per site/month" plus "$12 per user/month", includes "Review & approve edits", custom domain, AI search and writing, and "500 successful answers included" for the Assistant. Ultimate: "$249 per site/month" plus "$12 per user/month", adds AI Insights, external sources and unlimited Agent. Enterprise: custom, and the only tier with "SAML SSO for your team" — [GitBook pricing](https://www.gitbook.com/pricing)

Fern (primary page did not render; search summary only)
- Hobby is free with 2 members, 250 AI credits and a custom domain; Team is $150 a month billed annually with 5 members, 1,000 AI credits, password protection and PDF export; Enterprise is custom with SSO, RBAC and self-hosting. Ask Fern costs two credits per message. SDK generation is priced separately from $250 a month per SDK — [Fern pricing](https://buildwithfern.com/pricing); [apis.io plan listing](https://plans.apis.io/plans/fern/fern-plans-pricing/)

ReadMe (fetched)
- Starter is "$0 /month, free for everyone" with 1 admin and custom domain. Pro is "$250 /month, billed annually", up to 5 admins, "$20/additional admin", and includes "Branching and reviews". Enterprise is annual-only and adds "SSO, OAuth, & more", audit logs and "Branch review permissions". AI: an "Ask AI" add-on costs "$150/mo"; "Docs Audit" and "GitHub AI Writer" are enterprise features — [ReadMe pricing](https://readme.com/pricing)

Redocly (fetched)
- Pro is $10 per seat per month with 1 project, 100 pages, custom domain and "Reunite (Git integration, editor, visual reviews)". Enterprise is $24 per seat per month and adds SSO, guest SSO, RBAC, AI search, analytics and MCP servers. Enterprise+ is custom and billed yearly, and is where security questionnaires, procurement forms, legal reviews and pay-by-invoice sit. Seats count only owner, member and committer roles; there are no charges for readers or page views; extra pages cost $0.12 a month each — [Redocly pricing](https://redocly.com/pricing)

Read the Docs (fetched)
- Community is free for open-source projects, with public docs and ad-supported hosting. Business plans are Basic $50 a month (2 concurrent builds, shared domain, SSO with GitHub and GitLab), Advanced $150 a month (4 concurrent builds, 5 custom domains, 30 days of analytics) and Pro $250 a month (6 concurrent builds, 15 custom domains, Google SSO, 90 days of analytics). Pull request previews, versioning, search and CDN hosting are listed for all plans, including the free one — [Read the Docs pricing](https://about.readthedocs.com/pricing/)

Scalar (search summary of vendor pages)
- Free includes hosted docs, the API client, up to 3 APIs and 1 SDK. Pro is $150 a month ($125 billed yearly) flat with 5 editor seats, not per seat. Business is $600 a month ($500 billed yearly) — [Scalar pricing](https://scalar.com/pricing); [Scalar vs Redocly](https://scalar.com/resources/compare/redocly)

Bump.sh, Theneo, Archbee, Document360, Stoplight (search summaries; mostly third-party listings)
- Bump.sh: Basic $50 a month (10 API docs, 3 users), Pro $250 a month (30 docs, 5 users), custom enterprise — [G2 Bump.sh pricing](https://www.g2.com/products/bump-sh/pricing)
- Theneo: free Starter (1 public project, 20 team members); Business $150 a month ($120 annual) per workspace; Growth $450 a month ($400 annual) per workspace — [documentation.ai Theneo review](https://documentation.ai/blog/theneo-review)
- Archbee: Growing from $80 a month, Scaling from $350 a month; a competitor claims real cost reaches $150 to $230 a month once AI, analytics and API add-ons are included (Docsie is a competitor) — [Archbee pricing](https://www.archbee.com/pricing); [Docsie comparison](https://www.docsie.io/blog/articles/archbee-vs-readme-pricing-comparison-2026/)
- Document360: discontinued its free tier in November 2024 and moved to quote-only pricing; one third-party estimate puts two projects on Business at $599 to $999 a month — [HappySupport, Document360 pricing](https://www.happysupport.ai/en/blog/document360-pricing)
- Stoplight: pricing page live under SmartBear copyright as of 26 September 2026, with Basic at $44 a month billed annually (3 users) and Startup at $113 a month (8 users) (Scalar is a competitor) — [Scalar, Stoplight alternatives](https://scalar.com/alternatives/stoplight)

Doctave, for comparison (fetched; now closed)
- Startup $99 a month (5 users, 1 custom domain, extra users $9), Growth $399 a month (10 users, extra users $19, versioning), Scale from $1,000 a month (25 users, SAML SSO, RBAC, authenticated readers). "Preview environments" were in all plans — [Doctave pricing](https://www.doctave.com/pricing)

Market-level observation
- A summary of 2026 tooling write-ups says pricing "has bifurcated with generous free tiers while mid-market has drifted toward $200-450/month per project" (search summary; the underlying page is a vendor blog) — [docsio.co, docs as code](https://docsio.co/blog/docs-as-code)

### Inferences
- The prevailing price anchor for a "serious" hosted docs tier is $150 to $450 a month per site. Redocly ($10 a seat) and Read the Docs ($50 a month) are the low end; both have much larger installed bases to amortise over.
- Preview deployments are not reliably a paid differentiator: Read the Docs gives pull request previews to free open-source projects, Doctave included them at $99, and Redocly bundles "visual reviews" at $10 a seat. Mintlify gating previews at $450 is one vendor's packaging choice, not a market norm.
- AI is the main metered unit. Two metering patterns exist: per successful answer (Mintlify, GitBook) and per doc-update run (Mintlify at 250 credits). Both vendors waive the charge when the AI produces nothing, which suggests buyers resist paying for null results.
- SAML SSO is uniformly the enterprise gate (GitBook, Mintlify, ReadMe, Fern, Doctave Scale, Chromatic); Redocly is the exception in offering it at a published $24 a seat.

### Gaps
- Fern's pricing page could not be read directly; figures are from a search summary.
- No reliable October 2026 pricing was found for Docusaurus-adjacent hosts as a category; Docusaurus sites are typically deployed on general hosts (Vercel, Netlify, Cloudflare), covered in section 4.
- Read the Docs for Business is understood to cover private repositories, but the fetched page text did not state it explicitly, so it is not asserted above.
- Whether Mintlify's Starter plan is free for commercial use without limits on pages or traffic was not confirmed.

## 2. Doctave and other docs-tool shutdowns, pivots, acquisitions and funding (2023 to 2026)

### Takeaway
Doctave gave no reason for closing and only two weeks' public notice; it was an unfunded Helsinki company founded in 2020 that charged $99 to $1,000+ a month and left behind an MIT-style open-source generator. Across the wider field, independent developer-tool companies are being absorbed by platforms (Fern by Postman, Astro and VoidZero by Cloudflare, Astral by OpenAI, Stoplight by SmartBear, Optic by Atlassian), and in several cases the paid hosted product was shut after acquisition.

### Cited Findings

Doctave
- The announcement is dated 31 August 2026 and gives the shutdown date as 14 September 2026, when hosted sites stop being served, the dashboard becomes unavailable and builds and deploys stop. This differs from the brief's "shut down on 31 August 2026" — [Doctave is shutting down](https://www.doctave.com/blog/doctave-is-shutting-down)
- The only stated explanation is: "After several years of building documentation tooling, we've made the difficult decision to shut Doctave down." No commercial reason, customer count, refund terms or next steps for the team are given; the post says the team "informed all our customers individually" — [Doctave is shutting down](https://www.doctave.com/blog/doctave-is-shutting-down)
- Migration path: an open-source static site generator, Docapella, "mostly compatible with existing Doctave projects"; users rename `doctave.yaml` to `docapella.yaml`. Server-dependent features such as analytics are unavailable — [Doctave is shutting down](https://www.doctave.com/blog/doctave-is-shutting-down); [Docapella](https://docapella.com/)
- Doctave is described by Tracxn as an unfunded company based in Helsinki, founded in 2020 by Niklas Begley, with Anton Rautio as CTO; it was a docs-as-code SaaS built on Rust (aggregator profile; search summary) — [Tracxn Doctave profile](https://tracxn.com/d/companies/doctave/__nmYVztop-XTERV0jDUFXOH201Fy93-HwHCMFzCK6NC8); [I'd Rather Be Writing Q&A with Niklas Begley](https://idratherbewriting.com/blog/doctave-qa-niklas-begley)
- The original open-source Doctave generator is MIT licensed — [Doctave on GitHub](https://github.com/Doctave/doctave)
- Pricing at closure: $99, $399 and from $1,000 a month, plus a migration service from $500 per project — [Doctave pricing](https://www.doctave.com/pricing)

Acquisitions and funding
- Fern: raised a $9M Series A led by Bessemer (total $13M) in April 2025; Postman acquired Fern on 8 January 2026, with the whole team joining; Fern reported more than 200 customers at acquisition (search summary) — [Fern Series A post](https://buildwithfern.com/post/series-a); [Pulse 2.0 on Postman and Fern](https://pulse2.com/postman-acquires-fern-to-expand-api-documentation-and-sdk-capabilities/)
- Mintlify: raised a $45M Series B at a $500M valuation in April 2026, co-led by Andreessen Horowitz and Salesforce Ventures, total funding $67M; claims 20,000+ companies. Sacra estimates $10M ARR at end of 2025, up from $1M at end of 2024 (estimate, not audited) — [FinSMEs](https://www.finsmes.com/2026/04/mintlify-raises-45m-in-series-b-funding-at-500m-valuation.html); [StockAnalysis Mintlify profile](https://stockanalysis.com/private/mintlify/)
- GitBook: third-party estimates of revenue are inconsistent ($3.9M for 2025 from one aggregator, $2M from another) and headcount is put at 44 to 52 in mid-2026. These are low-reliability aggregator figures — [GetLatka GitBook](https://getlatka.com/companies/gitbook.com); [Tracxn GitBook](https://tracxn.com/d/companies/gitbook/__XgkPQ8J_wnxo4YUrt2nzCR2GJNRfsh0dGTAP_gsEIj0)
- Stoplight: acquired by SmartBear, announced 22 August 2023; in October 2025 the APIs You Won't Hate newsletter wrote that "work on Stoplight appears to be slowing"; no end-of-life announcement as of 26 September 2026 (via a competitor's page) — [Stoplight joins SmartBear](https://blog.stoplight.io/stoplight-joins-smartbear-a-cto-perspective); [Scalar, Stoplight alternatives](https://scalar.com/alternatives/stoplight)
- Optic (API drift and breaking-change detection): acquired by Atlassian on 25 April 2024; the GitHub repository was archived on 12 January 2026 after the last release in August 2025; useoptic.com no longer resolves and no integration into Atlassian Compass is evident (authors of these posts sell alternatives) — [MarketScreener](https://www.marketscreener.com/quote/stock/ATLASSIAN-CORPORATION-25531314/news/Atlassian-Corporation-completed-the-acquisition-of-Optic-46544272/); [DEV, "Optic Is Dead"](https://dev.to/flarecanary/optic-is-dead-what-now-for-api-drift-detection-2kb8)
- Astro: Cloudflare announced on 16 January 2026 that it was acquiring The Astro Technology Company; all full-time employees became Cloudflare employees; Astro stays MIT licensed with open governance — [Astro, joining Cloudflare](https://astro.build/blog/joining-cloudflare/); [The New Stack](https://thenewstack.io/cloudflare-acquires-team-behind-open-source-framework-astro/)
- Astral (Ruff, uv): OpenAI announced the acquisition on 19 March 2026; on 17 June 2026 Astral announced it was winding down pyx, its paid hosted registry launched in beta in August 2025, and open-sourcing the GPU index underneath it — [OpenAI to acquire Astral](https://openai.com/index/openai-to-acquire-astral/); [pydevtools on pyx](https://pydevtools.com/blog/astral-winds-down-pyx-open-sources-gpu-packaging/)
- VoidZero (Vite): raised a $12.5M Series A led by Accel in October 2025; Cloudflare announced its acquisition on 4 June 2026. Evan You said the company "experimented with a mixed licensing model for Vite+, but it didn't feel right" and that "monetizing tooling, especially open-source software, has proven to be quite challenging" — [VoidZero Series A](https://voidzero.dev/posts/announcing-series-a); [TechTimes](https://www.techtimes.com/articles/317907/20260606/cloudflare-buys-voidzero-vites-130m-weekly-users-get-new-vendor-neutrality-pledge.htm)
- TinaCMS: acquired by SSW (an Australian consultancy) in May 2024; before the deal the team was "a skeleton crew of just four developers"; the self-hosted backend was later open-sourced under Apache 2.0 — [Tina joins SSW](https://tina.io/blog/Tina-Joins-SSW); [TinaCMS is now fully open source](https://tina.io/blog/Tinacms-is-now-fully-open-source)
- Swimm: now positions around AI documentation and reverse engineering of legacy and COBOL mainframe code for large enterprises, with sales-led, unpublished pricing (search summary of review sites) — [Swimm](https://swimm.io/); [CheckThat Swimm profile](https://checkthat.ai/brands/swimm)
- Document360 removed its free tier in November 2024 — [HappySupport](https://www.happysupport.ai/en/blog/document360-pricing)

### Inferences
- Doctave is the closest structural analogue to a hosted Ascribe tier (Rust, docs-as-code, Git workflow, previews, unfunded small team) and ran roughly six years before closing. Its silence on reasons means the closure cannot be attributed to pricing, competition or founder choice from public evidence.
- The pattern of 2024 to 2026 exits is acquisition by a platform with a larger revenue base, followed in some cases by removal of the paid product (pyx) or abandonment of the tool (Optic). None of the acquired tool makers cited here had published evidence of a self-sustaining paid tier before being acquired.
- Mintlify is the outlier with venture-scale growth claims; its position rests on a hosted platform with sales, not an add-on to a free local tool.

### Gaps
- Doctave: no public statement of reasons, customer numbers, revenue or what the founders do next was found.
- No reliable information was found on ReadMe corporate changes, Archbee or Docsie ownership changes, or Swimm layoffs in 2025 to 2026.
- GitBook's actual revenue and profitability are not public; aggregator figures conflict.
- Deal terms for Fern, Astro, Astral and VoidZero were not disclosed.

## 3. Open-core developer tools with a free local tool and paid hosted collaboration

### Takeaway
The split that survives is a free local tool plus a hosted service whose cost scales with something the customer cannot easily self-host (Chromatic snapshots, Buf registry, Nx Cloud compute, Read the Docs builds). Attempts by framework or tool makers to add a generic hosted product alongside a free tool have repeatedly been wound down (Astro Studio, pyx, Vite+ licensing, Vercel's paid remote cache), and sponsorware, the main non-hosted alternative, has been abandoned by its best-known docs practitioners.

### Cited Findings

Astro Studio (hosted database for Astro DB)
- "Astro Studio never reached a point where we felt we had found product-market fit." "Thousands of Astro developers tried the platform, but few stuck around." Larger platforms such as Turso and Supabase "offered similar features". The post says the goal of building "a profitable business to support the continued development and growth of Astro" failed. New databases stopped on 1 October 2024; existing ones became inaccessible after 1 March 2025 — [Goodbye Studio, Hello DB](https://astro.build/blog/goodbye-astro-studio/)

Astral, VoidZero
- pyx "was the revenue answer for a venture-funded company giving away uv and Ruff"; it was shut after OpenAI's backing removed the commercial pressure (commentary source, not Astral itself) — [pydevtools](https://pydevtools.com/blog/astral-winds-down-pyx-open-sources-gpu-packaging/)
- VoidZero dropped mixed licensing for Vite+ before its acquisition — [TechTimes](https://www.techtimes.com/articles/317907/20260606/cloudflare-buys-voidzero-vites-130m-weekly-users-get-new-vendor-neutrality-pledge.htm)

Storybook and Chromatic (visual review as the paid service; fetched)
- Free: 5,000 snapshots a month. Starter: $179 a month for 35,000 snapshots. Pro: $399 a month for 85,000. Overage on Starter is $0.008 per snapshot. Collaborators and users are unlimited on every tier, including Free. SSO, roles and team access control are Enterprise only. There is a "Free for open source" programme by application — [Chromatic pricing](https://www.chromatic.com/pricing)

Git-based CMSs
- Tina Cloud: Free ($0, 2 users); Team $24 per project per month ($290 a year, 3 users, up to 10); Team Plus $41 a month ($490 a year, 5 users, adds editorial workflow); Business $249 a month ($2,990 a year, 20 users); Enterprise adds SSO. Extra seats are $9 to $18 per user per year. AI features are "coming soon" — [Tina pricing](https://tina.io/pricing)
- Keystatic Cloud: free for up to 3 users per team; Pro from $10 a month, plus $5 per user per month beyond 3; Pro unlocks Cloud Images and experimental multiplayer editing (search summary) — [Keystatic Cloud docs](https://keystatic.com/docs/cloud); [DEV, Keystatic pricing 2026](https://dev.to/nayankyada/keystatic-pricing-2026-free-tier-limits-cloud-costs-when-to-upgrade-5f07)
- Decap CMS (community continuation of Netlify CMS, no paid tier): still releasing, with 3.16.1 on 8 September 2026, at a modest cadence (search summary) — [Decap CMS blog](https://decapcms.org/blog/)
- Sanity: Free plan includes 20 seats and unlimited free viewers; Growth is $15 per seat per month with comments, tasks and AI Assist; enterprise is custom (search summary of third-party guides) — [Roboto Studio, Sanity pricing](https://robotostudio.com/blog/sanity-cms-pricing-which-plan-is-right-for-you)

Build and registry services
- Nx Cloud: Hobby plan free with 50,000 credits a month and 5 contributors; Team plan is usage-based with $29 of included credit and $5.50 per 10,000 credits beyond (search summary) — [Nx Cloud pricing](https://nx.dev/pricing)
- Turborepo: Vercel made Remote Cache free on all plans in December 2024, including for teams that do not host on Vercel, "resulting in immediate savings for over 43,000 existing teams" — [Vercel changelog](https://vercel.com/changelog/free-vercel-remote-cache)
- Buf: Community plan free (1 private repository, unlimited public); Teams $0.50 per type per month; Pro $5 per type per month with a $3,000 a month minimum, adding SAML/OIDC SSO, a private instance and audit logging (search summary) — [Buf pricing](https://buf.build/pricing)

Docs-adjacent tools
- Read the Docs: two revenue sources, Read the Docs for Business and ad-funded Community hosting via EthicalAds; advertising "remains the single largest source of funding for Read the Docs and one that scales as our costs scale" (statement dates from 2019 to 2020; current split not found) — [EthicalAds, ad funding at Read the Docs](https://www.ethicalads.io/blog/2019/06/ad-funding-at-read-the-docs-and-whats-next-for-ethical-advertising/); [About Read the Docs](https://docs.readthedocs.io/en/stable/about/index.html)
- Vale: Vale Server was a commercial desktop app sold as a one-off $40 purchase; a third-party 2026 guide describes Vale Studio as a hosted UI with team management on top of the free CLI (single low-reliability source for the Studio claim) — [Introducing Vale Server](https://jdkato.medium.com/introducing-vale-server-36264789d89); [docsio.co Vale guide](https://docsio.co/blog/vale-linter)
- Zensical (successor to Material for MkDocs): the generator is MIT licensed and free; revenue comes from Zensical Spark memberships (Community $9 a month or $89 a year; Organization $399 a month or $3,999 a year; Private $7,499 a year; Strategic $14,999 a year; Strategic 20 $24,999 a year; support add-on $199 a month) and from Zensical Studio, an authoring product with "project-wide refactoring" (Studio free; Studio Pro $19 a month or $179 a year; Studio Team $39 a month or $399 a year per seat). Studio beta is free until 5 November 2026 — [Zensical Studio pricing](https://zensical.org/studio/pricing/); [Material for MkDocs, Insiders now free](https://squidfunk.github.io/mkdocs-material/blog/2025/11/11/insiders-now-free-for-everyone/)
- Biome: funded by Open Collective and GitHub Sponsors, sponsors such as Depot, and an enterprise support programme in which companies contract a core contributor; no hosted paid product (search summary) — [Biome roadmap 2026](https://biomejs.dev/blog/roadmap-2026/)
- Prettier: funded by donations; Open Collective shows Meta Open Source at $49,000 since November 2024 and GitHub Sponsors at about $27,300 since February 2024; it raised $110k in total and redistributed $75k, including a $22,500 bounty paid to Biome contributors in 2023 (search summary) — [Prettier on Open Collective](https://opencollective.com/prettier); [Prettier bounty post](https://prettier.io/blog/2023/11/27/20k-bounty-was-claimed/)
- Redocly: free open-source CLI and Redoc alongside a per-seat hosted product from $10 a seat — [Redocly pricing](https://redocly.com/pricing)
- Sentry and Graphite: see sections 4 and 6.

### Inferences
- Surviving paid tiers charge for a metered, stateful or compute-heavy service (snapshots, builds, registries, hosted content APIs). Chromatic is the nearest model to "review as the paid service": it meters the artefact (snapshots), not the people, and keeps reviewers free.
- Git-based CMS cloud tiers show how low the price ceiling can be when the hosted part is thin: Keystatic Cloud at $10 a month and Tina Cloud at $24 a month per project, with Tina needing an acquirer after running on four developers.
- Zensical is a current, direct test of a different split for a docs tool: free generator, paid authoring tool (Studio) and paid influence and support (Spark). It launched in late 2025 and no results are public yet.
- Features that were once paid have been made free by better-capitalised owners (Turborepo remote cache, pyx's GPU index). A solo maintainer's paid feature can be undercut the same way.

### Gaps
- No revenue figures were found for Chromatic, Tina Cloud, Keystatic Cloud, Nx Cloud, Buf, Redocly or Zensical Spark and Studio.
- The reason Vale Server was discontinued, and the current status and pricing of Vale Studio, could not be confirmed from a primary source.
- Changesets has no paid tier that was found; no monetisation evidence was located.
- Read the Docs' current revenue mix between ads and Business subscriptions is not public in the sources found.

## 4. Review as a paid product

### Takeaway
Across preview and review tools, reviewers are almost always free and unlimited; vendors charge the people who author changes (or the compute consumed) and treat reviewer access as a growth feature. Evidence that teams pay for review tooling specifically for docs is thin: one small product (DraftView) targets exactly this and is free at small scale, and no revenue or adoption figures for it were found.

### Cited Findings
- Netlify: "With any plan, you can add unlimited free Reviewer roles to your team"; reviewers can annotate deploy previews with screenshots, recordings and comments through the Netlify Drawer — [Netlify Drawer docs](https://docs.netlify.com/deploy/review-deploys/netlify-drawer-for-feedback/overview/); [Netlify reviewer quickstart](https://docs.netlify.com/deploy/review-deploys/netlify-drawer-for-feedback/netlify-reviewer-quickstart/)
- Netlify pricing (fetched): Free $0 with 300 credits; Personal $9 a month; Pro $20 a month with 3,000 credits and unlimited team members; "Unlimited deploy previews" from the Free tier; SSO only on Enterprise. A production deploy costs 15 credits (about $0.10) and bandwidth 20 credits per GB — [Netlify pricing](https://www.netlify.com/pricing/)
- Vercel (fetched): Hobby $0; Pro $20 per developer seat per month with $20 of included usage credit; viewer seats are "Unlimited" on Pro; comments on preview deployments are available on all plans; SSO on Pro is a "$300 / month" add-on; custom preview deployment URLs are a "$100 / month" add-on — [Vercel pricing](https://vercel.com/pricing)
- Vercel comments require each commenter to sign in with their own account (search summary) — [Vercel knowledge base, preview deployments for review](https://vercel.com/kb/guide/preview-deployments-retail-campaign-review)
- Chromatic: unlimited collaborators on every plan; price scales with snapshots ($179 a month for 35,000) — [Chromatic pricing](https://www.chromatic.com/pricing)
- Reviewable (fetched): free forever for public repositories and personal private repositories; Team $8 and Business $16 per contributor per month billed annually; a contributor is "anyone in your GitHub organization who creates a pull request" and "This does not include admins, reviewers, or people using the reporting features"; SSO/SAML is on Business — [Reviewable pricing](https://www.reviewable.io/pricing)
- Graphite (fetched): Hobby free for personal repositories; Starter "$20 per user/month, billed annually"; Team "$40 per user/month" with unlimited AI reviews; SAML/SSO and audit log only on Enterprise. The pricing page header says "Cursor Cloud Agents are now in Graphite" — [Graphite pricing](https://graphite.com/pricing)
- CodeRabbit (fetched): Essentials $24, Team $48 and Advanced $72 per developer per month billed annually; "free reviews forever for public repositories"; only "developers who create pull requests" are charged; usage beyond included limits costs "$0.25 per reviewed file" and the agent "$0.40 per agent minute" — [CodeRabbit pricing](https://www.coderabbit.ai/pricing)
- Redocly bundles "visual reviews" in Reunite at $10 per seat and does not charge for readers; GitBook includes change requests in the free plan and "Review & approve edits" from Premium; ReadMe puts "Branching and reviews" in the $250 Pro plan — [Redocly pricing](https://redocly.com/pricing); [GitBook pricing](https://www.gitbook.com/pricing); [ReadMe pricing](https://readme.com/pricing)
- DraftView (docs-specific): a "visual review layer for docs-as-code projects"; the author generates a password-protected review link for a pull request, and reviewers without GitHub accounts see rendered docs, leave inline comments and suggest edits that sync back as pull requests. It is "free for public repositories", "free forever at small-team scale", and "you pay when your team grows, only for members who actually review that month" (vendor's own description; search summary) — [Introducing DraftView](https://www.docswriter.com/posts/introducing-draftview); [DraftView](https://www.draftview.app/)
- DraftView's current site positions it as "Human Review for AI-Generated Doc PRs" — [DraftView](https://www.draftview.app/)
- PullFlow (code review in Slack) is "always free for open source projects, startups, and small dev teams" (vendor claim) — [PullFlow](https://www.pullflow.com/)

### Inferences
- A paid tier that charges per reviewer would run against the norm set by Netlify, Vercel, Chromatic, Reviewable, Sanity and Redocly. The conventional billable units are the author or committer seat, the site or project, or metered compute.
- DraftView is the only product found that bills on active reviewers, and it softens that with a free small-team tier and free public repositories.
- The headline capability in the brief (rendered pull-request changes that reviewers comment on without developer accounts) already exists in three forms: free in general hosts (Netlify's free reviewers; Vercel comments, which need a Vercel account), bundled in docs platforms (Redocly, GitBook, ReadMe), and as a dedicated small product (DraftView). This is evidence of demand and also of low price expectations.
- AI code reviewers have established a per-author price band of $24 to $72 a month and free use on public repositories; no docs-specific AI reviewer pricing was found outside the docs platforms' credit schemes.

### Gaps
- DraftView's actual price list, maker, launch date and customer numbers could not be read from its site; the pricing page was not retrieved.
- No survey or first-hand founder account was found that quantifies teams paying for review tooling specifically for documentation.
- Vercel's exact rules on whether external commenters need a paid or free seat were taken from a search summary, not the docs page.

## 5. Cost to run a GitHub App plus hosted preview and review service

### Takeaway
Raw infrastructure for a small preview-and-comment service is cheap, on the order of $5 to $50 a month at low volume on Cloudflare or Fly.io list prices; the material costs are compliance and support. A SOC 2 Type II first year is quoted at $25,000 to $50,000 for a small startup, and enterprise docs buyers expect SSO, security questionnaires and procurement paperwork, which competitors reserve for custom-priced tiers.

### Cited Findings

Cloudflare (fetched)
- Workers Free: 100,000 requests a day, 10 ms CPU per invocation. Workers Paid: $5 a month minimum, 10 million requests a month included then $0.30 per million, 30 million CPU-ms included then $0.02 per million — [Cloudflare Workers pricing](https://developers.cloudflare.com/workers/platform/pricing/)
- D1: free tier of 5 million rows read a day, 100,000 rows written a day and 5 GB; paid includes 25 billion rows read and 50 million rows written a month, then $0.001 per million rows read, $1.00 per million rows written and $0.75 per GB-month — [Cloudflare Workers pricing](https://developers.cloudflare.com/workers/platform/pricing/)
- R2: free tier of 10 GB-month, 1 million Class A and 10 million Class B operations a month; paid $0.015 per GB-month, $4.50 per million Class A, $0.36 per million Class B; egress is free — [Cloudflare R2 pricing](https://developers.cloudflare.com/r2/pricing/)
- Durable Objects on the paid plan: 1 million requests a month included then $0.15 per million; 400,000 GB-s included then $12.50 per million GB-s. KV: $0.50 per million reads, $5.00 per million writes — [Cloudflare Workers pricing](https://developers.cloudflare.com/workers/platform/pricing/)

Fly.io (fetched)
- Smallest machine (shared-cpu-1x, 256 MB) is $2.19 a month in the cheapest regions; volumes $0.15 per GB-month; outbound transfer $0.02 per GB in North America and Europe; managed Postgres Basic from $38 a month plus $0.28 per GB-month; support plans are Standard $29, Premium $199 and Enterprise from $2,500 a month — [Fly.io pricing](https://docs.fly.io/about/pricing)

Vercel and Netlify as the host (fetched)
- Vercel Pro is $20 per developer seat with $20 usage credit; SSO add-on $300 a month. Netlify Pro is $20 a month with 3,000 credits; a production deploy costs 15 credits — [Vercel pricing](https://vercel.com/pricing); [Netlify pricing](https://www.netlify.com/pricing/)

Unit-cost benchmarks from vendors who resell similar work
- Chromatic charges $0.008 per extra snapshot; CodeRabbit charges $0.25 per reviewed file and $0.40 per agent minute; Mintlify charges an effective $0.25 per AI answer and $2.50 per automated doc update at overage rates — [Chromatic pricing](https://www.chromatic.com/pricing); [CodeRabbit pricing](https://www.coderabbit.ai/pricing); [Mintlify pricing](https://www.mintlify.com/pricing)

Compliance and enterprise expectations
- SOC 2: "most startups spend between $25,000 and $50,000" for total first-year certification; Type II audit fees for small to midsize companies are $12,000 to $20,000; compliance platforms (Vanta, Drata, Secureframe) charge $7,500 to $30,000 a year at startup size; penetration tests $5,000 to $25,000; Drata models about $28,000 first-year for a 25-person startup. These figures come from compliance vendors, who are self-interested — [Drata, SOC 2 cost](https://drata.com/learn/soc-2/cost); [Sector Post, SOC 2 audit costs](https://www.thesectorpost.com/compliance/soc2/audit-costs)
- Another vendor source puts a first Type II audit at $35,000 to $80,000 all in, and claims 83% of enterprise buyers require SOC 2 from SaaS vendors (vendor marketing; methodology not seen) — [buildmvpfast, SOC 2 readiness](https://www.buildmvpfast.com/blog/soc2-readiness-checklist-startups-enterprise-2026); [Konfirmity, SOC 2 for SaaS](https://www.konfirmity.com/blog/soc-2-for-saas)
- Type II requires a 3 to 12 month observation period (same sources) — [buildmvpfast](https://www.buildmvpfast.com/blog/soc2-readiness-checklist-startups-enterprise-2026)
- Redocly lists "security questionnaires", "procurement forms", "legal reviews" and "pay-by-invoice" as features of its custom-priced Enterprise+ tier, billed yearly — [Redocly pricing](https://redocly.com/pricing)
- GitHub Marketplace: GitHub keeps a 5% transaction fee (developers keep 95%); paid plans require a verified-publisher organisation, and a GitHub App needs at least 100 installations to list; paid apps must handle purchase, upgrade, downgrade, cancellation and trial events — [GitHub blog, Marketplace fees](https://github.blog/news-insights/company-news/github-reduces-marketplace-transaction-fees-revamps-technology-partner-program/); [GitHub Docs, requirements for listing an app](https://docs.github.com/en/apps/github-marketplace/creating-apps-for-github-marketplace/requirements-for-listing-an-app)
- Support burden, first-hand: a sponsorware maintainer wrote that "The more popular your projects get, the more user-support you have to do...And with this funding model, user-support stays free" — [pawamoy, sunsetting the sponsorware strategy](https://pawamoy.github.io/posts/sunsetting-the-sponsorware-strategy/)

### Inferences
- Illustrative arithmetic from the list prices above (an inference, not a quoted figure): a service that stores built preview sites in R2, serves them and a comment API from Workers, and keeps comments in D1 would fit inside the $5 Workers Paid minimum plus cents of R2 storage for tens of repositories; 100 GB of stored previews is $1.50 a month and egress is free. Build compute is the variable part and can be pushed to the customer's own CI (the pull request already builds there), which is how Chromatic and Nx Cloud keep the vendor side to storage and coordination.
- AI drift checks are the only component with meaningful marginal cost; market prices for comparable AI units ($0.25 per answer or file, $2.50 per doc update) indicate what buyers already pay per unit, not what it costs to serve.
- Hosting private repository content makes the service a data processor for customer source material, which is what triggers security review. Competitors price that overhead into custom enterprise tiers; a one-person vendor has no such tier to absorb it.
- The GitHub Marketplace's 100-installation threshold and verified-organisation requirement mean a new GitHub App cannot charge through the Marketplace from day one; billing would need to be direct.

### Gaps
- No first-hand cost breakdown from a small docs-preview or review service was found.
- The "SSO tax" as a named phenomenon was not sourced; the evidence here is the pricing pages that gate SSO.
- Model API costs for AI drift checks were out of scope and not priced here.
- Support-hours data for small hosted developer tools was not found beyond the qualitative account above.

## 6. Solo-maintainer and small-team monetisation evidence

### Takeaway
Sponsorware can fund a single maintainer at a high level in the best case (Material for MkDocs is reported at about $16,000 a month when it closed its sponsor account; Caleb Porzio passed $1M cumulative), but the docs ecosystem's two best-known sponsorware programmes were both ended in 2025 on grounds of operational overhead, and the larger sums elsewhere came mainly from paid content and logos rather than gated features. Source-available licences draw public criticism, and the VS Code Marketplace has no payment mechanism.

### Cited Findings

Sponsorware and sponsors
- Material for MkDocs: all Insiders features became free with version 9.7.0 on 11 November 2025; the project entered maintenance mode with critical fixes for at least 12 months; the Insiders repository was scheduled for deletion on 1 May 2026; the team's effort moved to Zensical — [Insiders now free for everyone](https://squidfunk.github.io/mkdocs-material/blog/2025/11/11/insiders-now-free-for-everyone/)
- A third-party blogger who tracked the programme reports $16,000 in monthly sponsorships when the GitHub Sponsors account closed (as of 5 November 2025), and relays that Martin Donath said on a podcast that "managing of subscribers through GitHub Sponsors at their given scale has not been easy". The official post gives no revenue figure — [Duerrenberger, Material for MkDocs is no more](https://duerrenberger.dev/blog/2025/11/06/material-for-mkdocs-is-no-more-long-live-zensical/)
- Earlier milestones: the project passed $100,000 a year in sponsorship; September 2022 tier prices were $15, $35 and $125 a month (search summary) — [Duerrenberger, sponsor journey](https://duerrenberger.dev/blog/2024/02/13/material-for-mkdocs-github-sponsor-journey/); [mkdocs-material issue 3986](https://github.com/squidfunk/mkdocs-material/issues/3986)
- Stated reasons for leaving MkDocs include "limitations of our core dependency, MkDocs ... deeply rooted in its architecture" (relayed by the same blogger) — [Duerrenberger](https://duerrenberger.dev/blog/2025/11/06/material-for-mkdocs-is-no-more-long-live-zensical/)
- mkdocstrings (pawamoy): the Insiders programme ran about 2.5 years and reached "a bit more than a thousand dollar in monthly sponsorships". Reasons for ending it: "twice as much repositories (public and private versions), which is OK when you maintain only one, but not when you maintain 10 or more"; support stays free while demand grows; and "I've always loved the programming side of things, and less the marketing one". The author joined the Zensical team — [pawamoy, sunsetting the sponsorware strategy](https://pawamoy.github.io/posts/sunsetting-the-sponsorware-strategy/)
- Caleb Porzio: over $1M cumulative on GitHub Sponsors, of which about $725k came from premium Livewire screencasts and $200k from company logos; only about $5k came from small "buy me coffee" sponsors — [Caleb Porzio, $1M on GitHub Sponsors](https://calebporzio.com/i-just-cracked-1-million-on-github-sponsors-heres-my-playbook)
- Sindre Sorhus: estimated at roughly $27k a year from GitHub Sponsors, about $10k from Open Collective and $6k from Patreon, against packages downloaded nearly 2 billion times a month (third-party estimate; search summary) — [Changelog, a proposal for open source sustainability](https://changelog.com/posts/a-proposal-for-open-source-sustainability)
- Evan You / VoidZero: took venture funding ($12.5M Series A, October 2025), tried mixed licensing for Vite+, and sold to Cloudflare in June 2026 — [VoidZero Series A](https://voidzero.dev/posts/announcing-series-a); [TechTimes](https://www.techtimes.com/articles/317907/20260606/cloudflare-buys-voidzero-vites-130m-weekly-users-get-new-vendor-neutrality-pledge.htm)

Licences
- Sentry moved from open source to the Business Source License and then, in November 2023, to its own Functional Source License, which converts to Apache 2.0 or MIT after two years and bars competing commercial use; Sentry branded the approach "Fair Source" in 2024 — [TechCrunch on FSL](https://techcrunch.com/2023/11/20/with-functional-source-license-sentry-wants-to-grant-developers-freedom-without-harmful-free-riding/); [Sentry is now Fair Source](https://blog.sentry.io/sentry-is-now-fair-source)
- Criticism: Thierry Carrez of the Open Infrastructure Foundation and OSI called it "proprietary gatekeeping wrapped in open-washed clothing"; supporters such as GitButler's Scott Chacon defend it as protecting investment — [TechCrunch on fair source](https://techcrunch.com/2024/09/22/some-startups-are-going-fair-source-to-avoid-the-pitfalls-of-open-source-licensing/)
- TinaCMS went the other way and open-sourced its self-hosted backend under Apache 2.0 — [TinaCMS is now fully open source](https://tina.io/blog/Tinacms-is-now-fully-open-source)

Marketplaces
- The VS Code Marketplace has no paid-extension or payment support; the long-standing request is open as microsoft/vscode issue 111800. The workaround is a free listing with features unlocked by a licence key sold elsewhere (second source is a payments vendor) — [microsoft/vscode issue 111800](https://github.com/microsoft/vscode/issues/111800); [Dodo Payments guide](https://dodopayments.com/blogs/sell-vscode-extensions)
- GitHub Marketplace terms are in section 5 — [GitHub Docs](https://docs.github.com/en/apps/github-marketplace/creating-apps-for-github-marketplace/requirements-for-listing-an-app)

Other models
- Zensical replaces feature gating with paid membership and support tiers ($9 a month to $24,999 a year) and a paid authoring tool — [Zensical Studio pricing](https://zensical.org/studio/pricing/)
- Biome sells contracted core-contributor time as enterprise support — [Biome roadmap 2026](https://biomejs.dev/blog/roadmap-2026/)

### Inferences
- The $16,000 a month Material for MkDocs figure came after roughly five years and on top of one of the most widely used docs themes; mkdocstrings, also popular, reached about $1,000 a month in 2.5 years. A 0.1.x tool with a small user base sits far below either starting point.
- Both docs sponsorware operators cited administration and split repositories as costs. Withholding features from the open-source edition has a maintenance cost independent of community reaction.
- Porzio's breakdown shows the money came from paid educational content and corporate logos, not from donations.
- No evidence was found of a solo maintainer sustaining a paid hosted service with enterprise customers; the solo-maintainer successes found are content, sponsorship or support models.

### Gaps
- The $16,000 a month Material for MkDocs figure is third-party; no official final figure or sponsor count was found.
- No data was found on Zensical Spark or Studio uptake.
- No quantified community-reaction data (forks, user loss) for BSL or FSL adopters was collected here; Elastic's licence history was not researched.
- No revenue data was found for VS Code extensions sold through licence keys.

## 7. Market size signals

### Takeaway
The best current survey (State of Docs 2026, 1,131 respondents, run by GitBook) reports 45% of teams publishing with dedicated documentation tools and 21% with open-source platforms or Git repositories; analyst market-size figures exist but are inconsistent and methodologically opaque. No count of docs-as-code users was found.

### Cited Findings
- State of Docs 2026 was published on 28 August 2026 by GitBook, with 1,131 respondents (2.5 times the previous year) and 30+ interviews. Respondents: technical writers 35%, leadership or decision-makers 21%, engineers 15%, customer experience 7%, operations 6%, support 5%, developer relations 5%, marketing 4%. Regions: Europe and Middle East 37%, North America 33%, Asia-Pacific 19%. GitBook is a vendor in this market — [State of Docs 2026, introduction and demographics](https://www.stateofdocs.com/2026/introduction-and-demographics)
- Publishing tools: dedicated documentation tools 45%, open-source platforms or Git repositories 21%, website publishing platforms 9% — [State of Docs 2026, docs tooling](https://www.stateofdocs.com/2026/docs-tooling)
- Conflict: a secondary article summarising the same report says "31% of respondents use Git repositories as their primary documentation tool, while 45% use dedicated HAT tools". The 31% may refer to a different question (authoring or storage rather than publishing); the report page itself gives 21% for publishing — [Dr.Explain summary](https://www.drexplain.com/press/articles/documentation_as_infrastructure_key_takeaways_from_state_of_docs_2026/); [State of Docs 2026, docs tooling](https://www.stateofdocs.com/2026/docs-tooling)
- 30% cite keeping docs in sync with the product as their biggest challenge; 76% of documentation professionals use AI regularly; 62% cite hallucinations as the top concern; 56% of regular AI users spend less time writing and more time editing and reviewing; 44% have AI guidelines — [State of Docs 2026](https://www.stateofdocs.com/2026)
- About 70% of teams factor AI into information-architecture decisions, up from 31% in 2025 — [State of Docs 2026, docs tooling](https://www.stateofdocs.com/2026/docs-tooling)
- 80% of decision-makers review documentation before buying; 88% rate docs as important to purchase decisions; 57% do not track leads from documentation; 51% say docs are important or essential for closing deals — [State of Docs 2026, purchase decisions](https://www.stateofdocs.com/2026/purchase-decisions-and-business-impact)
- Analyst sizing (paywalled market-report vendors; methods not visible): "Software Documentation Tool Market" $4.71B in 2025 to $10B by 2035 at 7.8% CAGR; another report gives $6.32B in 2024 to $12.45B by 2033; API documentation tools are put at over $3.1B in 2026 — [WiseGuy Reports](https://www.wiseguyreports.com/reports/software-documentation-tool-market); [Verified Market Reports](https://www.verifiedmarketreports.com/product/software-documentation-tools-market/); [Dataintelo](https://dataintelo.com/report/api-documentation-tools-market)
- Vendor scale claims: Mintlify says it powers docs for more than 20,000 companies; Fern reported 200+ customers at acquisition; Vercel cited 43,000 teams using Remote Cache — [FinSMEs](https://www.finsmes.com/2026/04/mintlify-raises-45m-in-series-b-funding-at-500m-valuation.html); [Pulse 2.0](https://pulse2.com/postman-acquires-fern-to-expand-api-documentation-and-sdk-capabilities/); [Vercel changelog](https://vercel.com/changelog/free-vercel-remote-cache)

### Inferences
- About one in five State of Docs respondents publishes with open-source platforms or Git repositories, the population a tool like Ascribe addresses; roughly twice as many use dedicated tools. The sample is recruited by GitBook and skews toward its audience, so the open-source share may be understated.
- "Keeping docs in sync with product" as the top challenge for 30%, and the shift toward editing and reviewing AI output, are the survey findings closest to demand for drift checks and review surfaces. They indicate a problem, not a willingness to pay.
- The analyst figures disagree by more than a billion dollars for nearby years and mix help-authoring, knowledge-base and API tools; they do not size docs-as-code tooling.

### Gaps
- No count of docs-as-code users or of sites built with Docusaurus, MkDocs, Sphinx, Astro Starlight or similar was found.
- The State of Docs chapter on team structure returned a 404 at the URLs tried, so team-size and ownership figures were not captured.
- No State of Docs figure on tooling budgets or spend was found on the pages read.
- Write the Docs' 2025 salary survey results exist, but tooling and team-size breakdowns were not retrieved — [Write the Docs surveys](https://www.writethedocs.org/surveys/)

## 8. Buyer and persona

### Takeaway
Direct evidence on who holds the budget for docs tooling is weak. The available signals are that survey respondents are mostly technical writers with a fifth in leadership, that vendors price for small editor teams (3 to 5 seats included) with procurement features reserved for enterprise, and one secondary claim that dedicated docs teams with their own tooling budget are becoming rarer.

### Cited Findings
- State of Docs 2026 respondents: 35% technical writers, 21% leadership or decision-makers, 15% engineers, 5% developer relations; the largest company-size groups were small companies (up to 50 employees) and enterprises (301+) — [State of Docs 2026, introduction and demographics](https://www.stateofdocs.com/2026/introduction-and-demographics)
- A search summary of 2026 commentary states that "the standalone documentation department with its own tooling budget is disappearing, with fewer than a quarter of developer-tools companies still maintaining a centralized docs team as of Q1 2026". The originating page and method were not verified; treat as unconfirmed — [GitDoc blog, technical writing trends 2026](https://gitdoc.ai/blog/technical-writing-trends-2026)
- Packaging as a proxy for team size: included seats are 5 editors (Mintlify Starter), 5 members (Fern Team), 5 admins (ReadMe Pro), 5 editor seats (Scalar Pro), 5 users (Doctave Startup), 3 users (Bump.sh Basic, Tina Team, Keystatic free) — [Mintlify pricing](https://www.mintlify.com/pricing); [ReadMe pricing](https://readme.com/pricing); [Scalar pricing](https://scalar.com/pricing); [Doctave pricing](https://www.doctave.com/pricing); [Tina pricing](https://tina.io/pricing)
- Enterprise purchasing requirements appear as paid features: procurement forms, security questionnaires, legal review and invoicing at Redocly Enterprise+; SAML SSO at the top tier nearly everywhere — [Redocly pricing](https://redocly.com/pricing); [GitBook pricing](https://www.gitbook.com/pricing)
- Review and AI tools bill the engineering organisation per pull-request author ($8 to $72 a month), which places those purchases with engineering managers rather than docs leads — [Reviewable pricing](https://www.reviewable.io/pricing); [CodeRabbit pricing](https://www.coderabbit.ai/pricing); [Graphite pricing](https://graphite.com/pricing)
- Willingness-to-pay signals from operators: Astro found that "Thousands of Astro developers tried the platform, but few stuck around"; Evan You called monetising open-source tooling "quite challenging"; a sponsorware maintainer reached about $1,000 a month after 2.5 years — [Astro, goodbye Studio](https://astro.build/blog/goodbye-astro-studio/); [TechTimes](https://www.techtimes.com/articles/317907/20260606/cloudflare-buys-voidzero-vites-130m-weekly-users-get-new-vendor-neutrality-pledge.htm); [pawamoy](https://pawamoy.github.io/posts/sunsetting-the-sponsorware-strategy/)
- Counter-signal: Mintlify's estimated growth from $1M to $10M ARR during 2025 shows companies do pay hundreds of dollars a month for hosted docs with AI features (Sacra estimate) — [StockAnalysis Mintlify profile](https://stockanalysis.com/private/mintlify/)
- Docs matter commercially to buyers of the documented product: 80% of decision-makers review docs before buying, yet 57% of teams do not track leads from docs, which limits a docs lead's ability to justify spend — [State of Docs 2026, purchase decisions](https://www.stateofdocs.com/2026/purchase-decisions-and-business-impact)

### Inferences
- Vendors converge on a first paid tier sized for about five editors, implying the typical paying docs team is small and that per-seat revenue from editors is limited; this is consistent with flat per-site pricing being common.
- If docs tooling budgets are moving from docs teams to engineering or developer-experience functions, the purchase competes with engineering tools priced per author, and the buyer will expect SSO and security review.
- Evidence of willingness to pay is strongest for full hosted platforms and weakest for add-on services beside a free tool.

### Gaps
- No survey data was found on who signs off docs tooling purchases, typical docs tooling budgets, or team sizes; the relevant State of Docs chapter could not be retrieved.
- No founder account quantifying conversion from a free docs tool to a paid hosted tier was found.
- The claim about disappearing docs budgets is from an unverified secondary source.
