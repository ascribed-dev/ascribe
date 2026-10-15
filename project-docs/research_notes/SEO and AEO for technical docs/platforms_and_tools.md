# Platforms and tools: what docs teams get for SEO and AEO (as of October 2026)

Research date: 2026-10-10. Every claim carries the URL it was read at. "Read 2026-10" means the page was fetched during this research and carries no ship date of its own. "Search snippet" means the claim came from a search-result summary of the cited page and the page itself was not opened; treat those as unconfirmed at the primary source. Several vendor pages (GitBook, Fern) embed "agent instructions" asking an AI reader to call a query endpoint; those were not followed, and are recorded below only as a feature observation.

## Metadata: titles, descriptions, canonical URLs, Open Graph and Twitter tags, social images

### Takeaway
Hosted platforms (Mintlify, Fern, GitBook, docs.page) emit a complete head (title with site suffix, description, canonical, `og:*`, `twitter:*`) and a generated social image with no configuration; open-source generators split into those that do most of it (Docusaurus, Starlight, jekyll-seo-tag) and those that leave canonical, Open Graph, or the site URL to the author (VitePress, Sphinx, MkDocs, mdBook). No tool read here reports missing or duplicate titles and descriptions at build time; that job is left to crawlers (Screaming Frog, Lighthouse, Semrush).

### Cited Findings

Platform summary (default / configurable / reported / paid), from the sources cited in the sections below:

| Tool | Default with no configuration | Configurable | Reported back to the team | Paid |
|---|---|---|---|---|
| Mintlify | Full head, canonical, generated 1200x630 OG image, JSON-LD graph, sitemap, robots.txt with Content-Signal, llms.txt, llms-full.txt, `.md` pages, `Accept: text/markdown`, skill.md, hosted MCP | Frontmatter and `docs.json` (`seo.metatags`, `seo.paths`, `seo.indexing`, `seo.trailingSlash`, `redirects`, `markdown.instructions`, `<Visibility>`) | Traffic split human vs agent, search, feedback, assistant transcripts, CSV, S3 stream; `mint broken-links`; public agent score | Analytics and CI checks need Pro or Enterprise; WebMCP on Pro and Enterprise |
| GitBook | Title, description, OG tags, canonical, sitemap, 301 on page move, llms.txt, llms-full.txt, `.md` pages, MCP server | Page-level and site-level indexing, canonical and alternate URL per page, redirects, adaptive content by `visitor.type` | Traffic, search keywords, page ratings, Agents & MCP report, Broken links report with one-click redirects, AI insights | Assistant sidebar, MCP connectors, adaptive content on Ultimate; search-box AI on Essential and Ultimate |
| Fern | Meta tags, canonical, dynamic OG image, TechArticle and BreadcrumbList JSON-LD, sitemap with content-hash `lastmod`, robots.txt, llms.txt at every level, `.md` pages | Frontmatter (`headline`, `canonical-url`, `noindex`, `nofollow`, `jsonld`), `docs.yml` `metadata`, `redirects`, `<llms-only>`/`<llms-ignore>`, custom llms.txt | Web Analytics tab: traffic by LLM provider, bot vs human per page; `fern check` rules; `fern docs link check` | Plan gating not stated on pages read |
| ReadMe | llms.txt on by default | Markdown for Agents, AI Dropdown, MCP toggles under AI settings | Discoverability grade against the Agent-Friendly Docs spec; Docs Audit score out of 10 | Docs Audit scheduling, export, history on Enterprise; doc search over MCP and full Ask AI need the AI add-on |
| Redocly (Realm, Revel) | llms.txt and clean Markdown unless `llmstxt.hide`; canonical and sitemap once `siteUrl` is set | `seo` block (title, description, image, keywords, `jsonLd`, `meta`), front matter `seo.priority` | Not found | Pro, Enterprise, Enterprise+ |
| Read the Docs | Sitemap, robots.txt hiding Hidden versions, canonical URL passed as an environment variable | Redirects, custom robots.txt and sitemap.xml, addons | Search analytics (queries with result counts), traffic analytics (top pages, 404s) | Retention and redirect limits vary by plan |
| Docusaurus | Canonical, description from first line, sitemap, broken-link build failure | Front matter, `headTags`, per-version `noIndex`, sitemap `lastmod` | Build errors and warnings only | Free |

Hosted platforms:
- Mintlify generates for every page: charset, `og:type` website, site name, large-summary Twitter card, title as "Page Title - Site Name", `og:title`/`twitter:title` falling back to the page title, `og:description`/`twitter:description` falling back to `description`, `canonical` built from the page URL with `og:url` matching, and `og:image`/`twitter:image` at 1200 by 630. Read 2026-10 — [Mintlify SEO](https://www.mintlify.com/docs/optimize/seo)
- Mintlify generates an OG image for every page from the logo, page title, description, and primary color; `thumbnails.background`, a per-page `thumbnail` object, or a static `og:image` change or replace it; `mint dev` serves previews at `/_mintlify/og/<page-path>.png` — [Mintlify SEO](https://www.mintlify.com/docs/optimize/seo)
- Mintlify frontmatter accepts `title`, `description`, `canonical`, `keywords`, `og:*`, `twitter:*`, `noindex`, `robots`; `docs.json` has `seo.metatags` (site-wide), `seo.paths` (folder-scoped, shipped 2026-09-25), `seo.trailingSlash` (shipped 2026-07-16), `seo.organization` — [Mintlify SEO](https://www.mintlify.com/docs/optimize/seo); [Mintlify changelog](https://www.mintlify.com/docs/changelog)
- Mintlify added canonical and `og:url` tags on 2025-10-11, and fixed duplicate OpenGraph meta tags on 2025-08-02 — [Mintlify changelog](https://www.mintlify.com/docs/changelog)
- Fern with nothing set: `<title>` is the page title with the site name appended; `og:title`, `og:description`, `og:url` fall back to page values; canonical is the page URL under the instance host; `twitter:card` is `summary_large_image`; a dynamic OG image is generated per page (`og:dynamic` is true when no site-wide image is set) showing logo, section, description, URL, and gradient. Read 2026-10 — [Fern SEO metadata](https://buildwithfern.com/learn/docs/seo/setting-seo-metadata.md)
- Fern frontmatter fields: `headline` (sets `<title>`), `canonical-url`, `keywords`, `og:*`, `twitter:*`, `noindex`, `nofollow`, `jsonld`, `jsonld:breadcrumb`; `docs.yml` `metadata` has `canonical-host` and `og:dynamic:*` styling options — [Fern SEO metadata](https://buildwithfern.com/learn/docs/seo/setting-seo-metadata.md)
- GitBook generates the HTML title and `og:title` from page and project titles, the meta description and `og:description` from page descriptions, and manages canonical URLs automatically. Read 2026-10 — [GitBook help: SEO](https://gitbook.com/docs/help-center/published-documentation/publishing/how-does-gitbook-handle-seo)
- GitBook lets a page set canonical and alternate URLs in the page options menu, including pointing at another GitBook page or an external URL, for versioned docs built with variants (search snippet) — [GitBook SEO](https://gitbook.com/docs/publish/seo)
- docs.page: `name` and `description` in `docs.json` are site defaults; page `title`, `description`, `image` override; if `socialPreview` is unset it generates a card from title, description, and logo. Read 2026-10 — [docs.page SEO](https://use.docs.page/customize/seo)
- Redocly `seo` config: `title` (priority: global, then page headings, then front matter), `description` (up to 150 characters, reused for `og:description` and `twitter:description`), `siteUrl`, `image`, `keywords`, `lang`, `jsonLd`, `meta`, `projectTitle` (suffix "| projectTitle"). Read 2026-10 — [Redocly seo config](https://redocly.com/docs/realm/config/seo)
- Document360 sets meta titles manually per article in a "GEO/SEO" tab, and its Eddy AI generates descriptions singly or in bulk (articles under 200 words are not eligible; workspace language must be English for bulk); descriptions feed its llms.txt. Page dated 2026-08-25 — [Document360 AI & search description](https://docs.document360.com/docs/seo-descriptions)
- Archbee has per-document "SEO meta controls": meta title, description, preview image, a custom URL key, and a URL alias (search snippet) — [Archbee redirects](https://docs.archbee.com/how-to-manage-urls-for-redirects)

Open-source generators:
- Docusaurus adds a canonical link to each Markdown page automatically; front matter `title`, `description`, `keywords`, `image`; `description` defaults to the first line of content and feeds meta and `og:description`; i18n adds `hreflang` alternates. Read 2026-10 — [Docusaurus SEO](https://docusaurus.io/docs/seo); [plugin-content-docs](https://docusaurus.io/docs/api/plugins/@docusaurus/plugin-content-docs)
- Docusaurus `titleDelimiter` defaults to `|` — [Docusaurus config](https://docusaurus.io/docs/api/docusaurus-config)
- Starlight requires `title`; `description` is optional (the docs show extending the schema to require it); site `description` is the fallback for the meta description; `titleDelimiter` defaults to `|`; `head` exists site-wide and per page — [Starlight configuration](https://starlight.astro.build/reference/configuration/); [Starlight frontmatter](https://starlight.astro.build/reference/frontmatter/)
- VitePress defaults: `title` "VitePress", `description` "A VitePress site", title is first `<h1>` plus suffix, `titleTemplate` customizes or removes the suffix; the reference documents no default canonical or `og:` tags and shows adding them through `transformHead` or `transformPageData` — [VitePress site config](https://vitepress.dev/reference/site-config)
- MkDocs emits a canonical link only when `site_url` is set (default `null`) and a description meta tag only when `site_description` is set — [MkDocs configuration](https://www.mkdocs.org/user-guide/configuration/)
- Material for MkDocs reads `title` and `description` front matter into the HTML title, meta tags, and social cards; its built-in `social` plugin generates a card per page locally (needs image-processing dependencies), with layouts and a debug grid — [Material reference](https://squidfunk.github.io/mkdocs-material/reference/); [Material social plugin](https://squidfunk.github.io/mkdocs-material/plugins/social/)
- Zensical: `site_url` sets the canonical URL, `site_description` is the fallback description, `site_author` is written to the head — [Zensical basics](https://zensical.org/docs/setup/basics/)
- Sphinx emits a canonical link only when `html_baseurl` is set (default empty); `html_title` defaults to "project release documentation" — [Sphinx configuration](https://www.sphinx-doc.org/en/master/usage/configuration.html)
- sphinxext-opengraph: generates a description from the first 200 characters (`ogp_description_length`), `ogp_enable_meta_description` defaults to true, `ogp_social_cards` generates a PNG per page (needs matplotlib), `ogp_canonical_url` exists for versioned docs, and on Read the Docs the site URL is read from `READTHEDOCS_CANONICAL_URL` — [sphinxext-opengraph](https://sphinxext-opengraph.readthedocs.io/en/latest/)
- Hugo's embedded `opengraph.html` emits `og:url`, `og:site_name`, `og:title`, `og:description`, `og:locale`, `og:type`, and for pages `article:published_time`, `article:modified_time`, and tags; `twitter_cards.html` emits `summary_large_image` when an image exists and `summary` otherwise — [Hugo embedded templates](https://gohugo.io/templates/embedded/)
- Docsy's meta description falls back: page `description`, then Hugo's summary for non-index pages, then the site description — [Docsy feedback and SEO](https://www.docsy.dev/docs/content/feedback/)
- jekyll-seo-tag adds title with site title appended, description, canonical URL, next and previous URLs, JSON-LD site and post metadata, Open Graph tags, and Twitter Summary Card metadata; it states that it does not analyze site structure or content — [jekyll-seo-tag](https://github.com/jekyll/jekyll-seo-tag)
- Antora sets `page.canonicalUrl` only when `site.url` is an absolute URL — [Antora site URL](https://docs.antora.org/antora/latest/playbook/site-url/)
- mdBook's HTML renderer reference documents `site-url`, `cname`, `input-404`, redirects, and search, and mentions no canonical tag, Open Graph tags, or sitemap — [mdBook renderers](https://rust-lang.github.io/mdBook/format/configuration/renderers.html)

Automatic social-card generation: Mintlify (default), Fern (default), docs.page (default), Material for MkDocs (`social` plugin), sphinxext-opengraph (`ogp_social_cards`) — sources as above.

### Inferences
- A generated head with canonical, Open Graph, and Twitter tags is table stakes on hosted platforms; a generated social image is close to it (three of the hosted platforms read do it by default).
- A build-time report of pages with missing, duplicate, or over-length titles and descriptions is something none of the docs tools read here ships, though the data is all in the build. The nearest are Document360's filter by "description availability" and ReadMe's Docs Audit.
- Title fallbacks that silently produce "VitePress", "Documentation", or a first line of body text are a source of duplicate titles that the tools do not flag.

### Gaps
- Starlight's default head tags (canonical, `og:title`, `og:url`, `twitter:card`) and Nextra's head and Open Graph handling were not confirmed at a primary source; the pages fetched did not list them.
- Whether GitBook generates an OG image automatically was not stated on the pages read.
- ReadMe's metadata page returned 404 at the URL tried; Stoplight, Zendesk Guide, and Bump.sh metadata controls were not found.

## Structured data (JSON-LD)

### Takeaway
Structured data on docs sites became a default only in 2026: Mintlify shipped an auto-generated graph in July 2026 and Fern emits TechArticle and BreadcrumbList on every indexable page. Open-source generators mostly emit none unless a plugin or template adds it.

### Cited Findings
- Mintlify shipped auto-generated Schema.org JSON-LD on 2026-07-10. Each indexable page emits a connected `@graph` with stable `@id`s: `Organization`, `WebSite`, `WebPage` (with description and modification dates), `BreadcrumbList` (from navigation), and `TechArticle`, or `APIReference` for pages with `api`, `openapi`, or `asyncapi` frontmatter. Pages with `noindex: true` get none — [Mintlify changelog](https://www.mintlify.com/docs/changelog); [Mintlify SEO](https://www.mintlify.com/docs/optimize/seo)
- A Mintlify fix on 2026-07-16 stopped `BreadcrumbList` JSON-LD from emitting empty items — [Mintlify changelog](https://www.mintlify.com/docs/changelog)
- Fern adds `TechArticle` to every indexable page (title, headline, description, canonical URL, language, site name) and `BreadcrumbList` for nested pages (top-level pages get none); `jsonld: false` suppresses it, `jsonld:breadcrumb` replaces the breadcrumb, and `noindex` pages get none — [Fern SEO metadata](https://buildwithfern.com/learn/docs/seo/setting-seo-metadata.md)
- Redocly has a `seo.jsonLd` object the author fills in; no automatic types are documented — [Redocly seo config](https://redocly.com/docs/realm/config/seo)
- Docusaurus's SEO page shows `Organization` only as a user-supplied `headTags` example and says blogs get rich results from publish date, authors, and image metadata — [Docusaurus SEO](https://docusaurus.io/docs/seo)
- A third-party Docusaurus plugin's README says Docusaurus generates microdata for `BreadcrumbList` on docs pages, which the plugin replaces with JSON-LD (search snippet; not confirmed in Docusaurus's own docs) — [docusaurus-plugin-structured-data](https://github.com/mat4m0/docusaurus-plugin-structured-data)
- jekyll-seo-tag emits JSON-LD site and post metadata; the publisher property appears only when `logo` is set — [jekyll-seo-tag](https://github.com/jekyll/jekyll-seo-tag); [usage](https://github.com/jekyll/jekyll-seo-tag/blob/master/docs/usage.md)
- Hugo's embedded `schema.html` emits microdata `meta` elements (`name`, `description`, `datePublished`, `dateModified`, `wordCount`, `image`, `keywords`), not JSON-LD — [Hugo embedded templates](https://gohugo.io/templates/embedded/)
- Lighthouse's `structured-data` audit is manual, with weight 0 — [Lighthouse default config](https://raw.githubusercontent.com/GoogleChrome/lighthouse/main/core/config/default-config.js)
- Cloudflare's Markdown for Agents preserves a page's JSON-LD in a `json` code block at the end of the converted Markdown — [Cloudflare Markdown for Agents](https://developers.cloudflare.com/fundamentals/reference/markdown-for-agents/)

### Inferences
- `BreadcrumbList` plus `TechArticle` with a modification date is the emerging default set; `FAQPage` was not found as a default anywhere.
- All inputs to these types (navigation position, title, description, canonical URL, language, last-modified date) are known at build time, so emitting and validating them needs no crawl.

### Gaps
- GitBook, ReadMe, Starlight, VitePress, Nextra, MkDocs, Material, Sphinx, Antora, and mdBook: no documentation of default JSON-LD was found; absence was not verified by inspecting rendered output.

## Sitemaps and robots: generation, `lastmod`, robots.txt, `noindex` controls

### Takeaway
Every hosted platform and most generators produce a sitemap; `lastmod` is off, absent, or git-dependent in most open-source tools, and Fern is the only one read that derives it from a content hash. `noindex` reaches from page to project on hosted platforms, and in 2026 the same switches began to govern llms.txt and robots.txt Content-Signal lines.

### Cited Findings
Sitemaps:
- Mintlify generates `sitemap.xml` and `robots.txt` automatically; only pages in navigation are indexed by default; hidden pages are excluded from the sitemap, search, assistant context, and MCP results; a custom file at the project root overrides the generated one — [Mintlify SEO](https://www.mintlify.com/docs/optimize/seo)
- Fern's sitemap carries a `<lastmod>` per entry that changes only when content changes: on each publish it hashes each page's normalized Markdown (SHA-256), ignoring whitespace and capitalization, and updates only changed pages; it is separate from the visible `last-updated` frontmatter — [Fern SEO overview](https://buildwithfern.com/learn/docs/seo/overview)
- GitBook generates a sitemap for any published space not set to unlisted; its own pages disagree on the path (`/sitemap.xml` versus `/sitemap-pages.xml`) (search snippet) — [GitBook SEO](https://gitbook.com/docs/publish/seo); [GitBook help: SEO](https://gitbook.com/docs/help-center/published-documentation/publishing/how-does-gitbook-handle-seo)
- Redocly generates `/sitemap.xml` only when `siteUrl` is set; it excludes non-public pages, ignored pages, and pages with `excludeFromSearch: true`; default priority 0.5, non-default versions 0.3, overridable with `seo.priority` — [Redocly seo config](https://redocly.com/docs/realm/config/seo)
- docs.page's sitemap lists every page file regardless of sidebar visibility or per-page `noindex`, with priority 1.0 for index pages, 0.8 for others, and `changefreq: weekly` — [docs.page SEO](https://use.docs.page/customize/seo)
- Read the Docs generates a sitemap automatically covering public, non-hidden versions, sorted by version number, with last-updated times and translations; a custom `sitemap.xml` in the default version's output is served as-is — [RTD sitemaps](https://docs.readthedocs.com/platform/stable/reference/sitemaps.html)
- Docusaurus's sitemap plugin ships in the classic preset: `lastmod` defaults to `null` (off; `'date'` or `'datetime'` turn it on), `changefreq` `'weekly'`, `priority` 0.5, `ignorePatterns`, `createSitemapItems`; `<lastmod>` is output only for routes whose plugin supplies `sourceFilePath` or `lastUpdatedAt`; it filters pages with a `noindex` robots meta — [Docusaurus sitemap plugin](https://docusaurus.io/docs/api/plugins/@docusaurus/plugin-sitemap); [Docusaurus SEO](https://docusaurus.io/docs/seo)
- Starlight has built-in sitemap generation once `site` is set in the Astro config — [Starlight customization](https://starlight.astro.build/guides/customization/)
- VitePress generates a sitemap only when `sitemap.hostname` is configured; `<lastmod>` requires `lastUpdated`; `transformItems` edits entries — [VitePress sitemap](https://vitepress.dev/guide/sitemap-generation)
- sphinx-sitemap: `sitemap_show_lastmod` is off unless set, uses the last git commit date, and does not support shallow clones; multiple versions need one sitemap each combined by hand into a sitemap index; `sitemap_excludes` takes wildcards — [sphinx-sitemap](https://sphinx-sitemap.readthedocs.io/en/latest/advanced-configuration.html)
- Hugo generates `sitemap.xml` by default, a sitemap index for multilingual sites, and honors `sitemap.disable: true` in front matter — [Hugo sitemap templates](https://gohugo.io/templates/sitemap/)
- Antora generates sitemap files only when `site.url` is absolute, and `robots.txt` only when `site.robots` is also defined — [Antora site URL](https://docs.antora.org/antora/latest/playbook/site-url/)
- mdBook: a sitemap pull request (#1618) exists and commenters report indexing problems without one; a third-party `mdbook-sitemap-generator` fills the gap (search snippet; merge status not checked) — [mdBook PR 1618](https://github.com/rust-lang/mdBook/pull/1618)
- Mintlify fixes: sitemap normalizes `/index` paths (2025-10-24); hidden pages stopped appearing in the sitemap (2026-04-10); self-hosted export writes `sitemap.xml` (2026-06-19) — [Mintlify changelog](https://www.mintlify.com/docs/changelog)

robots.txt:
- Mintlify's default robots.txt carries `Content-Signal: ai-train=yes, search=yes, ai-input=yes` for all user agents (shipped 2026-05-29); a custom robots.txt is served without it — [Mintlify SEO](https://www.mintlify.com/docs/optimize/seo); [Mintlify changelog](https://www.mintlify.com/docs/changelog)
- Fern serves a default robots.txt allowing all crawlers and pointing to the sitemap; a custom one can express Cloudflare Content Signals — [Fern SEO overview](https://buildwithfern.com/learn/docs/seo/overview)
- Read the Docs generates a robots.txt that hides versions set to Hidden and allows the rest; a custom file must live in the default version — [RTD robots.txt](https://docs.readthedocs.com/platform/stable/reference/robots.html)
- Docusaurus and Redocly ship no generated robots.txt; the author adds a static file — [Docusaurus SEO](https://docusaurus.io/docs/seo); [Redocly seo config](https://redocly.com/docs/realm/config/seo)
- docs.page's robots.txt allows all paths and points to the sitemap by default — [docs.page SEO](https://use.docs.page/customize/seo)

`noindex` scopes:
- Mintlify: per page (`noindex: true`; `hidden: true` is treated as `noindex`, since 2026-01-23), site-wide (`seo.metatags.robots`), and project-wide ("Don't index project", shipped 2026-06-12), which adds `noindex, nofollow` everywhere, empties the sitemap, returns 404 for llms.txt, and serves `Disallow: /` with `Content-Signal` all set to no; `seo.indexing: "all"` indexes hidden pages (2026-04-24) — [Mintlify SEO](https://www.mintlify.com/docs/optimize/seo); [Mintlify changelog](https://www.mintlify.com/docs/changelog)
- Mintlify: search engines ignore preview deployments (2025-10-11); raw `.md` pages send `x-robots-tag: noindex` (2025-08-30, after Google indexed them); static assets and favicons send `X-Robots-Tag: noindex` (2026-07-16) — [Mintlify changelog](https://www.mintlify.com/docs/changelog)
- GitBook: site-level indexing switch (on by default), and two page-level controls, one for GitBook's own search and Ask AI and one for external search engines and crawlers, with settings inherited down a section (search snippet) — [GitBook SEO](https://gitbook.com/docs/publish/seo)
- GitBook's page options menu states that hidden pages are "still indexed by search, search engines, and LLMs unless indexing is turned off" (2026-09-30); hidden pages are included in `llms-full.txt` and remain available through MCP — [GitBook changelog](https://gitbook.com/docs/changelog); [GitBook LLM-ready docs](https://gitbook.com/docs/publishing-documentation/llm-ready-docs)
- Fern: `noindex: true` keeps a page out of search engines and out of the LLM endpoints while leaving it in the sidebar; `nofollow` is separate — [Fern customize LLM output](https://buildwithfern.com/learn/docs/ai-features/customize-llm-output)
- Docusaurus: site `noIndex` (default false; no sitemap is generated when true), per-version `noIndex`, `unlisted` pages (not indexed, left out of sitemaps, reachable by link), and `draft` pages (development only) — [Docusaurus config](https://docusaurus.io/docs/api/docusaurus-config); [plugin-content-docs](https://docusaurus.io/docs/api/plugins/@docusaurus/plugin-content-docs)
- docs.page: `seo.noindex` (default false) disallows all paths in robots.txt, drops the sitemap line, and adds the meta tag to every page; page `noindex: true` adds only the meta tag — [docs.page SEO](https://use.docs.page/customize/seo)
- Redocly has no noindex switch; the author adds a robots meta tag through `seo.meta` or an `X-Robots-Tag` response header — [Redocly seo config](https://redocly.com/docs/realm/config/seo)
- Starlight `draft: true` pages are left out of production builds; `pagefind: false` removes a page from on-site search only — [Starlight frontmatter](https://starlight.astro.build/reference/frontmatter/)
- MkDocs `draft_docs` are served in `mkdocs serve` and excluded from builds; `exclude_docs` and `not_in_nav` control inclusion and navigation warnings — [MkDocs configuration](https://www.mkdocs.org/user-guide/configuration/)

### Inferences
- `noindex` is no longer only about search engines: Mintlify and Fern tie the same flag to llms.txt and structured data, while GitBook keeps hidden pages visible to LLMs unless indexing is switched off. Tools disagree on whether "hidden" implies "not for agents".
- docs.page listing `noindex` pages in its sitemap is an inconsistency of exactly the kind a static check catches (a URL both in the sitemap and marked `noindex`); Screaming Frog reports the same thing from a crawl.
- Git-derived `lastmod` breaks on shallow clones (Sphinx, Starlight, Docusaurus all warn about it); Fern's content hash avoids the dependency.

### Gaps
- Whether Starlight, MkDocs, Material, Nextra, and Hugo sitemaps carry `lastmod` by default was not confirmed on the pages read. MkDocs's configuration page does not describe its sitemap at all.
- ReadMe, Document360, Archbee, Bump.sh, Stoplight, and Zendesk Guide: sitemap and robots.txt behavior not found at a primary source.

## Versions and SEO: canonical across versions, hiding old versions, banners, redirects

### Takeaway
Two models exist: canonical-to-latest computed by the tool (Antora, Redocly, Fern) and per-version noindex or hiding set by the team (Docusaurus, Read the Docs, GitBook, Mintlify). No tool read here reports which old-version pages lack a counterpart in the latest version.

### Cited Findings
- Antora: "The canonical URL is the absolute URL of the newest, non-prerelease version of a page"; a page points at itself only if it is the newest non-prerelease version; prerelease pages point back to the latest release; the default UI outputs the link tag — [Antora site URL](https://docs.antora.org/antora/latest/playbook/site-url/)
- Redocly marks only default versions of versioned pages canonical and gives non-default versions sitemap priority 0.3 — [Redocly seo config](https://redocly.com/docs/realm/config/seo)
- Fern redirects broken versioned links to the default version and manages canonical URLs, and discourages hand-written redirects for version routing (search snippet) — [Fern redirects](https://buildwithfern.com/learn/docs/seo/redirects)
- Docusaurus per-version options: `noIndex` and `banner` (`none`, `unreleased`, `unmaintained`) — [plugin-content-docs](https://docusaurus.io/docs/api/plugins/@docusaurus/plugin-content-docs)
- Mintlify recommends a per-page frontmatter `canonical` pointing older versions at the latest equivalent; its llms.txt lists only the default language and default version — [Mintlify SEO](https://www.mintlify.com/docs/optimize/seo); [Mintlify llms.txt](https://www.mintlify.com/docs/ai/llmstxt)
- GitBook: canonical and alternate URLs can be set per page so the current variant is canonical (search snippet) — [GitBook SEO](https://gitbook.com/docs/publish/seo)
- Read the Docs builds the canonical URL from the default version and the canonical domain and passes it to the build as `READTHEDOCS_CANONICAL_URL`; the tool has to use it (Sphinx `html_baseurl`, MkDocs `site_url`) — [RTD canonical URLs](https://docs.readthedocs.com/platform/stable/canonical-urls.html)
- Read the Docs: Hidden versions are disallowed in the default robots.txt and left out of the sitemap; addons include a non-stable notification and a latest-version notification; a `/page/` prefix redirects to the default version — [RTD robots.txt](https://docs.readthedocs.com/platform/stable/reference/robots.html); [RTD addons](https://docs.readthedocs.com/platform/stable/addons.html); [RTD redirects](https://docs.readthedocs.com/platform/stable/user-defined-redirects.html)
- sphinxext-opengraph has `ogp_canonical_url` "useful for versioned docs"; sphinx-sitemap needs one sitemap per version combined manually — [sphinxext-opengraph](https://sphinxext-opengraph.readthedocs.io/en/latest/); [sphinx-sitemap](https://sphinx-sitemap.readthedocs.io/en/latest/advanced-configuration.html)

### Inferences
- Antora's rule (canonical is the same page in the newest non-prerelease version, computed) is the most complete built-in behavior; everywhere else the team sets it by hand or by a per-version switch.
- Which versions a page exists in is static knowledge, so "this old page has no successor to point its canonical at" is checkable at build time; nobody reports it.

### Gaps
- Material for MkDocs with `mike`, Starlight, VitePress, and Nextra version handling were not researched at a primary source.
- How ReadMe treats canonical URLs across versions was not found.

## Redirects: how declared, whether checked, whether created on move

### Takeaway
Redirects are a config list on docs-as-code platforms and a dashboard elsewhere; only GitBook creates them automatically when a page moves, and in 2026 Mintlify, Fern, and GitBook each added a check or report tying redirects to broken URLs.

### Cited Findings
- GitBook automatically creates an HTTP 301 redirect from the old URL when a page is moved or renamed (search snippet) — [GitBook SEO](https://gitbook.com/docs/publish/seo)
- GitBook 2026 changes: deleting a published page warns that links may break (03-30); external redirect destinations (03-04); permanent redirects using HTTP 308 (09-16); redirects apply to translated pages and match case-insensitively (09-22) — [GitBook changelog](https://gitbook.com/docs/changelog)
- GitBook's Broken links report (2026-10-06) "finds pages visitors reach that return a 404 and suggests one-click redirects"; its Broken URLs report groups URL variants into one row (2026-08-06) — [GitBook changelog](https://gitbook.com/docs/changelog)
- Mintlify: a `redirects` array in `docs.json` with `source`, `destination`, `permanent` (308 by default, 307 when false; status codes changed 2026-03-13), `:slug*` and partial wildcards; not created automatically; `mint broken-links` checks that redirect destinations exist (2026-04-24) — [Mintlify redirects and broken links](https://www.mintlify.com/docs/create/broken-links); [Mintlify changelog](https://www.mintlify.com/docs/changelog)
- Fern: `redirects` in `docs.yml` with regex parameters; its own pages disagree on the default status (302 on the English page, 308 on a translated page); a `missing-redirects` rule in `fern check` warns when a published page is removed without a redirect, with severity set under `check.rules` (search snippet) — [Fern redirects](https://buildwithfern.com/learn/docs/seo/redirects); [Fern changelog 2026-04-02](https://buildwithfern.com/learn/docs/changelog/2026/4/2.md)
- Read the Docs: page, exact, clean-URL-to-HTML, and HTML-to-clean-URL redirects; suffix wildcard with `:splat`; applied only where no page exists unless "Force Redirect"; first match wins; an infinite redirect returns 404; 100 per project on Community; not applied to pull request previews — [RTD redirects](https://docs.readthedocs.com/platform/stable/user-defined-redirects.html)
- Archbee: a URL alias field per document redirects an old path to the new URL key (search snippet) — [Archbee URL alias](https://www.archbee.com/docs/url-redirect)
- mdBook: `[output.html.redirect]` generates HTML redirect pages; fragment redirects need JavaScript — [mdBook renderers](https://rust-lang.github.io/mdBook/format/configuration/renderers.html)
- Antora: `page-aliases` feed redirects and canonical URLs; the redirect facility is set in the playbook — [Antora site URL](https://docs.antora.org/antora/latest/playbook/site-url/)
- The Agent-Friendly Documentation Spec's `redirect-behavior` check wants redirects to stay same-host and avoid JavaScript redirects — [Agent-Friendly Documentation Spec](https://agentdocsspec.com/spec/web/)

### Inferences
- "A removed or moved published page needs a redirect" is now a named check in Fern and a report in GitBook; both need knowledge of the previously published set of URLs, which a build can keep as a file.
- Static-host redirect pages (mdBook, and by extension meta-refresh pages) fail the Agent-Friendly spec's no-JavaScript-redirect rule for fragments.

### Gaps
- Docusaurus's client-redirects plugin, Starlight's Astro redirects, Hugo `aliases`, VitePress, Material's `mkdocs-redirects`, and Zensical's redirects page were not read; Zensical lacked redirects parity as of a January 2026 issue (search snippet) — [Zensical compatibility](https://zensical.org/compatibility/features/)
- Redirect chain and loop detection: only Mintlify's warning against circular pairs and Read the Docs' 404-on-loop were found; no tool was found that reports chains at build time.

## Links: broken-link detection at build time

### Takeaway
Internal link checking at build is standard and usually fails the build; anchors are checked less strictly, and external links are left to a separate command or tool.

### Cited Findings
- Docusaurus: `onBrokenLinks` throws by default (build only); `onBrokenAnchors` warns by default; `markdown.hooks.onBrokenMarkdownLinks` warns by default (the top-level option is deprecated since v3.9); `onDuplicateRoutes` warns — [Docusaurus config](https://docusaurus.io/docs/api/docusaurus-config)
- VitePress: `ignoreDeadLinks` defaults to false, so dead links fail the build; it accepts `'localhostLinks'`, strings, regular expressions, or functions — [VitePress site config](https://vitepress.dev/reference/site-config)
- MkDocs 1.6 `validation` defaults: `links.not_found` warn, `links.anchors` info, `links.absolute_links` info, `links.unrecognized_links` info, `nav.omitted_files` info, `nav.not_found` warn; `strict` (default false) turns warnings into failures — [MkDocs configuration](https://www.mkdocs.org/user-guide/configuration/)
- Sphinx: `nitpicky` (default false) warns on every unresolved reference; the `linkcheck` builder checks external links and writes `output.txt`; unlinked pages produce an orphan warning, and `fail_on_warnings` on Read the Docs fails the build — [Sphinx configuration](https://www.sphinx-doc.org/en/master/usage/configuration.html); [Sphinx builders](https://www.sphinx-doc.org/en/master/usage/builders/index.html); [RTD SEO guide](https://docs.readthedocs.com/platform/stable/guides/technical-docs-seo-guide.html)
- Mintlify: `mint broken-links` CLI; a CI check on pull requests (Pro and Enterprise, GitHub) that covers internal links only, set to Warning or Blocking; CI checks for broken links shipped February 2025 — [Mintlify CI checks](https://www.mintlify.com/docs/deploy/ci); [Mintlify changelog](https://www.mintlify.com/docs/changelog)
- Fern: the `broken-links` rule in `fern check` validates internal links against local navigation and runs in CI and `fern docs dev`; `fern docs link check` (announced 2026-05-14) scans the published site and reports 404 and 403 URLs with their source pages, including external URLs (search snippet) — [Fern changelog 2026-05-14](https://buildwithfern.com/learn/docs/changelog/2026/5/14)
- ReadMe's Docs Audit reports broken links per page alongside linter results — [ReadMe Docs Audit](https://docs.readme.com/main/docs/docs-audit)
- Document360 has "Link status" analytics for broken or redirected links and a "Page not found" report (search snippet) — [Document360 analytics](https://docs.document360.com/docs/analytics-overview)
- Read the Docs traffic analytics lists pages returning 404 — [RTD traffic analytics](https://docs.readthedocs.com/platform/stable/traffic-analytics.html)
- mdbook-linkcheck is a separate backend that validates links, with web links optional (search snippet) — [mdbook-linkcheck](https://docs.rs/mdbook-linkcheck/)
- Lighthouse's `crawlable-anchors` and `link-text` audits check that links are crawlable and descriptive — [Lighthouse default config](https://raw.githubusercontent.com/GoogleChrome/lighthouse/main/core/config/default-config.js)

### Inferences
- Anchor checking is the weak spot: Docusaurus warns, MkDocs logs at info, Mintlify's page does not say. A tool that fails the build on broken anchors by default is stricter than the field.
- Hosted platforms report 404s from real traffic (GitBook, Read the Docs, Document360); static tools can only check what the build knows.

### Gaps
- Starlight has no built-in link check on the pages read; the community `starlight-links-validator` plugin was not confirmed at its source.
- lychee's offline and fragment behavior and linkinator's details came only from mirrors and secondary summaries and are unconfirmed.

## Search and analytics the team can see, including AI-specific reporting

### Takeaway
Agent analytics shipped across the hosted platforms in 2026: Mintlify splits human from agent traffic and breaks agents down by provider; GitBook's "Agents & MCP" report (announced 2026-10-06) counts agent reads by surface; Fern reports traffic by LLM provider. None of the open-source generators offers any of it, and Read the Docs' analytics do not distinguish agents.

### Cited Findings
- Mintlify analytics (Pro and Enterprise) views: Traffic (visitors, page views, referrals, human versus agent traffic, popular pages), Assistant (usage, topics, satisfaction, transcripts), Search (volume, frequent queries, click-through rates), User engagements, Savings, Feedback; CSV export; S3 streaming on Enterprise (2026-07-24). Read 2026-10 — [Mintlify analytics](https://www.mintlify.com/docs/optimize/analytics); [Mintlify changelog](https://www.mintlify.com/docs/changelog)
- Mintlify identifies agent visitors by IP address and user agent; the count approximates distinct sources, not sessions; the breakdown groups traffic by provider (ChatGPT, Claude, Cursor) with HTTP clients, Browser, and Other; from 2026-06-08 it recognizes more agents and counts agent HTML page views in addition to Markdown requests (search snippet) — [Mintlify traffic analytics](https://www.mintlify.com/docs/analytics/traffic)
- Mintlify's agent analytics blog says the team can see which agents visited, which pages they request most, and what searches they run through MCP (search snippet; date not captured) — [Mintlify blog: agent analytics](https://mintlify.com/blog/agent-analytics)
- Mintlify dated items: Markdown view requests always count as agent traffic (2026-07-31); "top agents" widget accepts a page filter (2026-09-18); Claude Code user agents classified (2026-06-12); agents can read analytics through the admin MCP (2026-05-29); assistant questions grouped into categories with sentiment (2026-07-02); hosted MCP feedback tool for agents (2026-07-02); conversation insights for all sites with agent and assistant (2026-01-09); assistant query bucketing (2025-12-03); `mint analytics` CLI added 2026-04-10 and removed 2026-06-19 — [Mintlify changelog](https://www.mintlify.com/docs/changelog)
- GitBook's Agents & MCP report: headline metrics are agent page reads, agents' share of all site reads, and MCP calls; a people-versus-agents chart over time; reads counted on four surfaces (Markdown `.md`, llms.txt, MCP, site page views and RSS); MCP cards for tools called, search queries run, and pages read; a ranking of clients (unidentified ones grouped) and of pages read by agents. Read 2026-10 — [GitBook Agents & MCP](https://gitbook.com/docs/analytics/insights/agents-and-mcp)
- GitBook dated items: Agents & MCP report announced 2026-10-06; AI insights dashboard for Assistant questions and content gaps (2026-03-31); detection of Claude, Claude Code, and Gemini visits improved (2026-04-28); AI insights questions filterable by channel (2026-05-19); MCP `sendFeedback` tool for agents (2026-07-29) — [GitBook changelog](https://gitbook.com/docs/changelog)
- GitBook page ratings (sad, neutral, happy) appear under Analyze, Traffic, Page feedback; insights cover page traffic and search keywords — [GitBook site settings](https://gitbook.com/docs/publishing-documentation/site-settings); [GitBook help: SEO](https://gitbook.com/docs/help-center/published-documentation/publishing/how-does-gitbook-handle-seo)
- Fern's Web Analytics tab shows traffic by LLM provider (Claude, ChatGPT, Cursor given as examples) and page-level bot versus human breakdowns for Markdown and llms.txt — [Fern llms.txt analytics](https://buildwithfern.com/learn/docs/ai-features/llms-txt-analytics)
- Document360: search analytics with total, successful, and no-result searches and a No Result Searches keyword list; an MCP analytics dashboard with call volumes, success and failure rates, tool usage, active users, client distribution, and search activity (search snippet) — [Document360 search analytics](https://docs.document360.com/docs/analytics-search.md); [Document360 MCP analytics](https://docs.document360.com/docs/mcp-server-analytics)
- Read the Docs search analytics: top queries of the past 30 days with result count and search count per query (zero results visible by count), daily totals, CSV download; 90 days retained on Community — [RTD search analytics](https://docs.readthedocs.com/platform/stable/search-analytics.html)
- Read the Docs traffic analytics: page views and 404 pages, top pages of the last month; the page does not say whether bots or agents are separated — [RTD traffic analytics](https://docs.readthedocs.com/platform/stable/traffic-analytics.html)
- Docsy's "Was this page helpful?" widget sends `page_helpful` events to Google Analytics (since 0.8.0) — [Docsy feedback](https://www.docsy.dev/docs/content/feedback/)
- Nextra's feedback link opens a prefilled GitHub issue — [Nextra theme configuration](https://nextra.site/docs/docs-theme/theme-configuration)
- Bing Webmaster Tools' AI Performance report (public preview February 2026) shows total citations, average cited pages per day, a sample of grounding queries, and per-URL citation frequency, with no click data and CSV export only (search snippet from third-party write-ups; no Microsoft page was read) — [Search Influence](https://www.searchinfluence.com/blog/bing-ai-performance-report-copilot-citations/)
- Google Search Console: branded-queries filter announced 2025-11-20; AI Mode data is merged into Web Search totals since June 2025; a separate generative AI report was rolling out to a subset of sites in 2026 (search snippet from secondary sources) — [Google Search Central blog](https://developers.google.com/search/blog/2025/11/search-console-branded-filter); [ppc.land](https://ppc.land/google-updates-search-console-performance-analysis-guidance/)
- AirOps offers "Vercel Agent Analytics", classifying AI bot visits from a Vercel log drain (search snippet); no native Vercel AI-traffic dashboard was found — [AirOps](https://docs.airops.com/insights/settings/vercel-agent-analytics)

### Inferences
- The shared shape of an agent dashboard is: share of reads from agents, breakdown by client, pages agents read most, and the surface used (Markdown, llms.txt, MCP, HTML). All of it needs request logs, so a static toolchain can only offer it through a host or a log format.
- Counting rules are unsettled: Mintlify changed its agent definition in June 2026 and its visitor counting in September 2026, so period comparisons break. Any number published about "agent share" depends on these definitions.
- AI referral traffic (humans arriving from ChatGPT and similar) as a named metric was not found in any docs platform's documentation read here; Mintlify lists "referrals" generally.

### Gaps
- ReadMe's analytics for AI or agent traffic: no primary page found.
- Whether Mintlify's search view lists zero-result queries was not stated on the page read.
- Cloudflare AI Crawl Control's analytics were not read; the Markdown for Agents page only names it as the settings location.
- Archbee, Bump.sh, Stoplight, and Zendesk analytics not researched.

## AEO features: who ships what, since when, default on or off

### Takeaway
By October 2026 llms.txt, per-page Markdown at `.md` URLs, and a hosted MCP server are standard on hosted docs platforms and on by default; the differentiators are content negotiation, human-only and agent-only content blocks, discovery headers, generated skills, and agent instructions. Open-source generators rely on community plugins, with Docsy the exception.

### Cited Findings
llms.txt and llms-full.txt:
- Mintlify: `/llms.txt` and `/llms-full.txt` hosted automatically on all sites since November 2024; custom files since 2025-08-16; OpenAPI and AsyncAPI specs listed since 2026-02-27; blockquote summaries since 2026-04-17; per-section indexes under `/_llms/` when the file exceeds 100,000 characters (2026-08-21 and 08-28); also served at `/.well-known/llms.txt`; only the default language and version are listed — [Mintlify changelog](https://www.mintlify.com/docs/changelog); [Mintlify llms.txt](https://www.mintlify.com/docs/ai/llmstxt)
- Mintlify sends a `Link` header advertising `rel="llms-txt"` and `rel="llms-full-txt"`, the MCP server card, agent card, and agent skills index, plus `X-Llms-Txt: /llms.txt`, on every response including 404s — [Mintlify llms.txt](https://www.mintlify.com/docs/ai/llmstxt)
- GitBook: `/llms.txt` lists every published page with a Markdown version; `/llms-full.txt` holds all content and includes hidden pages; it is paginated, each part linking to the previous and next (2026-09-30) — [GitBook LLM-ready docs](https://gitbook.com/docs/publishing-documentation/llm-ready-docs); [GitBook changelog](https://gitbook.com/docs/changelog)
- Fern: llms.txt generated automatically at every level of the hierarchy (`/llms.txt`, `/docs/llms.txt`), with descriptions from `description` then `subtitle`, and links to OpenAPI and AsyncAPI specs; Fern does not generate `llms-full.txt` (the key is deprecated) because full-site files exceeded context windows and saw little use; query parameters `lang`, `excludeSpec`, `excludeChangelog` filter the output — [Fern llms.txt](https://buildwithfern.com/learn/docs/ai-features/llms-txt); [Fern customize LLM output](https://buildwithfern.com/learn/docs/ai-features/customize-llm-output)
- ReadMe: llms.txt "enabled on all projects by default", available on all plans, excludes hidden pages; a custom file in the repo root stops automatic updates. Page updated 2026-09-28 — [ReadMe llms.txt](https://docs.readme.com/main/docs/llmstxt)
- Redocly: llms.txt and clean Markdown versions generated unless `llmstxt.hide: true`; `sections` with include and exclude globs — [Redocly seo config](https://redocly.com/docs/realm/config/seo)
- docs.page: llms.txt and llms-full.txt built per request from the repository with no setup — [docs.page llms.txt](https://use.docs.page/features/llms-txt)
- Bump.sh: enabling Ask AI generates llms.txt, makes every page available as `.md`, and exposes an MCP server at `/mcp` (search snippet; Markdown announced December 2025) — [Bump.sh Ask AI](https://docs.bump.sh/help/documentation-experience/ask-ai-capabilities/)
- Document360 publishes an llms.txt that includes articles with descriptions — [Document360 AI & search description](https://docs.document360.com/docs/seo-descriptions)
- Docsy: opt-in agent support (llms.txt, Markdown page output, alternate links, a "View Markdown" link), first in release 0.15.0 (search snippet for the release number) — [Docsy agent support](https://www.docsy.dev/docs/content/agent-support/)
- Community plugins: `vitepress-plugin-llms` (llms.txt, llms-full.txt, a `.md` per page, copy and download buttons, `<llm-only>` and `<llm-exclude>` tags; used by Vite, Vue.js, Vitest, Rolldown) — [vitepress-plugin-llms](https://github.com/okineadev/vitepress-plugin-llms); `starlight-llms-txt` — [starlight-llms-txt](https://github.com/delucis/starlight-llms-txt); `mkdocs-llmstxt` 0.5.0 (2025-11-20, in maintenance mode) (search snippet) — [mkdocs-llmstxt](https://github.com/pawamoy/mkdocs-llmstxt); several Docusaurus plugins, with no official one (search snippet) — [lekoarts.de](https://www.lekoarts.de/how-to-add-llms-txt-to-docusaurus/)
- A Zendesk Marketplace app (KnowledgeBridge) generates llms.txt for a Help Center; no native Zendesk feature was found (search snippet) — [Zendesk Marketplace](https://www.zendesk.com/marketplace/apps/support/1259972/knowledgebridge/)

Markdown pages and content negotiation:
- Mintlify: `.md` on any page URL since February 2025; `Accept: text/markdown` since 2026-01-23 (`text/plain` also works); every HTML page links to its Markdown alternate (2026-06-26); Markdown 404s carry recovery guidance and links to llms.txt and up to three related pages (2026-09-11) — [Mintlify changelog](https://www.mintlify.com/docs/changelog); [Mintlify Markdown export](https://www.mintlify.com/docs/ai/markdown-export)
- GitBook: `.md` on any published page; published sites serve Markdown to ChatGPT Search's crawler (2026-09-30) and expose AI agent detection and robots directives in Markdown (2026-08-21); Markdown routes accept `?ask=` — [GitBook LLM-ready docs](https://gitbook.com/docs/publishing-documentation/llm-ready-docs); [GitBook changelog](https://gitbook.com/docs/changelog)
- Fern: `.md` and `.mdx` on any page URL; API reference pages return parameters and responses as a readable list — [Fern llms.txt](https://buildwithfern.com/learn/docs/ai-features/llms-txt)
- ReadMe "Markdown for Agents": `Accept: text/markdown` returns Markdown, enabled under AI, Discoverability; a missing `.md` page returns a Markdown 404 with `agentTools` front matter; all plans. Page updated 2026-09-25 — [ReadMe AI-ready docs](https://docs.readme.com/main/docs/ai-ready-docs); [ReadMe llms.txt](https://docs.readme.com/main/docs/llmstxt)
- Cloudflare Markdown for Agents converts HTML to Markdown at the edge on `Accept: text/markdown`, opt-in per zone, on Pro, Business, and Enterprise at no cost, with a default `Content-Signal` of `ai-train=yes, search=yes, ai-input=yes`. Page updated 2026-10-09 — [Cloudflare Markdown for Agents](https://developers.cloudflare.com/fundamentals/reference/markdown-for-agents/)
- Vercel's own docs serve Markdown by `Accept: text/markdown` or `.md` and publish `llms.txt` and `sitemap.md`; Vercel ships a template for the pattern (search snippet) — [Vercel agent resources](https://vercel.com/docs/agent-resources)

Buttons and menus:
- Mintlify contextual menu: Cursor and VS Code (July 2025), Grok (2026-01-30), a copyable MCP install command (2026-02-13), Google AI Studio, Devin, Windsurf (2026-03-20), per-page menus in frontmatter (2026-07-16); Cmd+C copies the page as Markdown — [Mintlify changelog](https://www.mintlify.com/docs/changelog); [Mintlify Markdown export](https://www.mintlify.com/docs/ai/markdown-export)
- ReadMe AI Dropdown: opens the page in ChatGPT or Claude or copies it as Markdown; all plans — [ReadMe AI-ready docs](https://docs.readme.com/main/docs/ai-ready-docs)
- Nextra: "Copy Documentation" dropdown added in v4.4.0, with `theme.copyPage` to disable in v4.6.0; an open bug says it copies source, not rendered Markdown (search snippet) — [Nextra issue 4791](https://goodfirstissue.org/shuding/nextra/issues/4791)
- docs.page has a "View markdown" item in its page action menu — [docs.page llms.txt](https://use.docs.page/features/llms-txt)

MCP servers:
- Mintlify: hosted MCP servers since July 2025; `get_page` tool (2026-04-03); filesystem tool (2026-04-17); MCP server card at well-known paths (2026-05-29); works on authenticated sites (2026-09-25); WebMCP registration in the browser on Pro and Enterprise (2026-09-25) — [Mintlify changelog](https://www.mintlify.com/docs/changelog)
- GitBook: MCP server at `/~gitbook/mcp` on every published site; server card (2026-09-22); `askQuestion` tool (2026-07-29); one-click connectors (2026-08-06) — [GitBook LLM-ready docs](https://gitbook.com/docs/publishing-documentation/llm-ready-docs); [GitBook changelog](https://gitbook.com/docs/changelog)
- ReadMe: project MCP with OpenAPI tools on all plans; doc search and fetch need the AI add-on — [ReadMe AI-ready docs](https://docs.readme.com/main/docs/ai-ready-docs)
- docs.page: a per-repository MCP endpoint — [docs.page llms.txt](https://use.docs.page/features/llms-txt)
- Document360: an MCP server with publishing and workflow tools in v12.5.2 (search snippet) — [Document360 release note](https://docs.document360.com/docs/june-2026-1252)

Skills, AGENTS.md, and agent instructions:
- Mintlify: auto-generated `install.md` (2026-01-16) and `skill.md` (2026-01-23), installable with `npx skills add` (2026-01-30); `.mintlify/AGENTS.md` and `Assistant.md` (2026-02-20); agent-skills discovery at `/.well-known/agent-skills/` (2026-03-27); multiple `SKILL.md` files from `.mintlify/skills/` (2026-04-10); `markdown.instructions` rendered as an "Agent Instructions" block in every Markdown page, llms.txt, and llms-full.txt (2026-06-26) — [Mintlify changelog](https://www.mintlify.com/docs/changelog); [Mintlify Markdown export](https://www.mintlify.com/docs/ai/markdown-export)
- Fern: a default directive can be prepended to every page served to agents; Fern publishes agent skills for using its docs — [Fern llms.txt](https://buildwithfern.com/learn/docs/ai-features/llms-txt)
- Observed while reading: GitBook and Fern "page not found" responses and some GitBook pages carry instructions addressed to AI agents, pointing at a question endpoint or at the llms.txt index — [GitBook SEO (404)](https://gitbook.com/docs/publishing-documentation/search-engine-optimization-seo); [Fern analytics (404)](https://buildwithfern.com/learn/docs/ai-features/analytics)

Human-only and agent-only content:
- Mintlify `<Visibility for="humans">` and `<Visibility for="agents">` (2026-04-17; in the editor 2026-07-16); agent content is included in llms-full.txt — [Mintlify Markdown export](https://www.mintlify.com/docs/ai/markdown-export); [Mintlify changelog](https://www.mintlify.com/docs/changelog)
- Fern `<llms-only>` (served in llms.txt and `.md`, hidden on the page, in search, and from Ask Fern) and `<llms-ignore>` (the reverse) — [Fern customize LLM output](https://buildwithfern.com/learn/docs/ai-features/customize-llm-output)
- GitBook adaptive content can branch on `visitor.type` (`human` or `agent`) (2026-08-14; adaptive content is an Ultimate feature) — [GitBook changelog](https://gitbook.com/docs/changelog); [GitBook site settings](https://gitbook.com/docs/publishing-documentation/site-settings)
- `vitepress-plugin-llms` has `<llm-only>` and `<llm-exclude>` — [vitepress-plugin-llms](https://github.com/okineadev/vitepress-plugin-llms)

### Inferences
- Expected in late 2026: llms.txt on by default, `.md` URL per page, a copy-as-Markdown control, an MCP endpoint. Differentiating: `Accept` negotiation, discovery headers and an HTML alternate link, Markdown 404s, per-section llms.txt, agent-only and human-only blocks, generated skills.
- Vendors disagree on llms-full.txt: Mintlify and GitBook ship it (GitBook paginates it), Fern deprecated it. Size limits (Mintlify's 100,000 characters, the spec's fetch limit) are pushing everyone toward nested indexes.
- A static site can produce llms.txt, `.md` files, alternate links, and agent-only blocks at build time; content negotiation, response headers, and MCP need a server or edge layer (Cloudflare's product fills exactly that for static hosts).

### Gaps
- Read the Docs: no llms.txt, Markdown, or AI-crawler feature was found in its docs or by search.
- Stoplight: no primary source found for any AEO feature.
- Archbee's llms.txt and MCP: only a third-party directory listing was found.
- Zensical and Material for MkDocs: no llms.txt entry was found in the roadmap snippets read.
- Launch dates for GitBook's llms.txt, `.md`, and MCP, and for Fern's and ReadMe's features, were not captured (GitBook's changelog page was only read back to March 2026).

## Audits and scores: SEO checks, docs health, AI readiness

### Takeaway
"Agent readiness" scoring converged on one open spec in 2026: the Agent-Friendly Documentation Spec and its `afdocs` checker underlie Mintlify's agent score, Fern's Agent Score leaderboard, and ReadMe's Discoverability grade, and Lighthouse added an experimental Agentic Browsing category. All of them test a live URL; none runs on a build without a network.

### Cited Findings
- Agent-Friendly Documentation Spec v0.6.0 (draft, 2026-09-13) defines 28 checks in 7 categories: content discoverability (`llms-txt-exists`, `-valid`, `-size`, `-links-resolve`, `-links-markdown`, `-directive-html`, `-directive-md`), Markdown availability (`markdown-url-support`, `content-negotiation`), page size (`rendering-strategy`, `page-size-markdown`, `page-size-html`, `page-size-transfer`, `content-start-position`, `single-fetch-completeness`), content structure (`tabbed-content-serialization`, `section-header-quality`, `markdown-code-fence-validity`, `markdown-link-portability`, `embedded-data-serialization`), URL stability (`http-status-codes`, `redirect-behavior`), observability (`llms-txt-coverage`, `markdown-content-parity`, `cache-header-hygiene`), authentication (`auth-gate-detection`, `auth-alternative-access`, `bot-protection-interference`) — [Agent-Friendly Documentation Spec](https://agentdocsspec.com/spec/web/)
- `afdocs` runs as `npx afdocs check <url>` against a live site, needs Node 22 or later, is MIT-licensed and at 0.x, prints a scorecard with a 0 to 100 score, letter grade, and per-check PASS, WARN, FAIL with fixes, and documents CI integration; requests default to a 200 ms delay — [afdocs repository](https://github.com/agent-ecosystem/afdocs)
- afdocs scoring: results are Pass, Warn, Fail, or Skip; weights Critical 10, High 7, Medium 4, Low 2; multi-page checks score proportionally over a sample; `--checks` runs a subset — [afdocs checks](https://afdocs.dev/checks/)
- Mintlify agent score, published 2026-04-27 at mintlify.com/score: 29 checks, the spec's seven categories plus three Mintlify checks (full content discoverability, agent skill discoverability, MCP server discoverability); CI use is not mentioned — [Mintlify blog: agent score](https://mintlify.com/blog/agent-score)
- Fern Agent Score ("Lighthouse for AI agents"): 22 checks, 247 companies scored, 189 at 80 or above, and the claim that 73% of top API docs lack an llms.txt. Read 2026-10; launch date not stated — [Fern Agent Score](https://buildwithfern.com/agent-score)
- ReadMe's Discoverability settings page grades a project against the Agent-Friendly Docs spec (all plans) — [ReadMe AI-ready docs](https://docs.readme.com/main/docs/ai-ready-docs)
- ReadMe Docs Audit runs the project's Linter rules over every page, reports errors, warnings, broken links, and an average score out of 10 with per-page scores; run on demand from the dashboard; scheduling, CSV export, and history are Enterprise; Pro gets a preview; no CI integration mentioned. Page updated 2026-09-15 — [ReadMe Docs Audit](https://docs.readme.com/main/docs/docs-audit)
- Lighthouse's SEO category in the current default config: `is-crawlable` (weight about 4.04), `document-title`, `meta-description`, `http-status-code`, `link-text`, `crawlable-anchors`, `robots-txt`, `image-alt`, `hreflang`, `canonical` (weight 1 each), and `structured-data` (manual, weight 0). Read 2026-10 — [Lighthouse default config](https://raw.githubusercontent.com/GoogleChrome/lighthouse/main/core/config/default-config.js)
- What the Lighthouse SEO audits check, by name: page is not blocked from indexing; document has a title; document has a meta description; page returns a successful HTTP status; links have descriptive text; links are crawlable; robots.txt is valid; images have alt text; hreflang is valid; rel=canonical is valid — [Lighthouse SEO audits](https://developer.chrome.com/docs/lighthouse/seo/); [Lighthouse default config](https://raw.githubusercontent.com/GoogleChrome/lighthouse/main/core/config/default-config.js)
- The developer.chrome.com SEO page still lists `plugins` and `tap-targets`, which are not in the current default config's SEO category — [Lighthouse SEO audits](https://developer.chrome.com/docs/lighthouse/seo/); contradicted by [Lighthouse default config](https://raw.githubusercontent.com/GoogleChrome/lighthouse/main/core/config/default-config.js)
- Lighthouse's default config has an `agentic-browsing` category: `agent-accessibility-tree`, `webmcp-form-coverage`, `webmcp-registered-tools`, `webmcp-schema-validity`, `cumulative-layout-shift`, `llms-txt`, `ard-schema` — [Lighthouse default config](https://raw.githubusercontent.com/GoogleChrome/lighthouse/main/core/config/default-config.js)
- Secondary sources date the Agentic Browsing category to Lighthouse 13.3 (5 to 7 May 2026, sources differ), call it experimental, and say a 404 for llms.txt is treated as not applicable (search snippet) — [ppc.land](https://ppc.land/google-adds-llms-txt-to-lighthouse-as-agentic-web-standards-heat-up/)
- Semrush Site Audit added an "AI Search" issue category on 2025-07-01 with four checks: llms.txt not found, links with non-descriptive anchor text, pages with only one incoming internal link, links with no anchor text; available on all subscription levels — [Semrush news](https://semrush.com/news/411622-optimize-for-ai-search-with-site-audit)
- Screaming Frog reports over 300 issues, warnings, and opportunities across tabs: response codes, security, URL, page titles (missing, duplicate, multiple, over 60 characters), meta description (missing, duplicate, multiple, over 155 characters), H1 and H2 (missing, multiple, duplicate, non-sequential), content (exact and near duplicates, low content, soft 404), images, canonicals, pagination, directives, hreflang, JavaScript, links (no internal outlinks, high crawl depth), structured data, sitemaps (orphan URLs, URLs not in sitemap), PageSpeed, mobile, accessibility — [Screaming Frog issues](https://www.screamingfrog.co.uk/seo-spider/issues/)
- Sitebulb describes 300+ prioritized "Hints" with importance from Critical to Insight (search snippet) — [Sitebulb hints](https://sitebulb.com/hints/)
- Unlighthouse's `unlighthouse-ci` audits every page it discovers through the sitemap, robots.txt, and links, and exits 1 when a category score falls below a budget (search snippet) — [Unlighthouse CI](https://unlighthouse.dev/integrations/ci)
- pa11y-ci runs against each `<loc>` of a sitemap with `--sitemap`, and can rewrite hostnames for local testing (search snippet) — [pa11y-ci](https://npmjs.com/package/@kschaefer/pa11y-ci)
- Mintlify has a `mint a11y` command (2025-10-03) and `mint validate` (2026-01-16) — [Mintlify changelog](https://www.mintlify.com/docs/changelog)
- Third-party scores of vendor docs vary by run and check set: Mintlify's docs scored 100 on Fern's leaderboard on 2026-06-22 with afdocs v0.18.7, Bump.sh's developer docs 86 on Mintlify's tool on 2026-05-11, Zendesk's developer docs 59 on Fern's on 2026-04-03 (search snippets) — [Fern Agent Score: Mintlify](https://buildwithfern.com/agent-score/company/mintlify-com-docs); [Mintlify score: Bump](https://www.mintlify.com/score/bump.md); [Fern Agent Score: Zendesk](https://buildwithfern.com/agent-score/company/zendesk)

### Inferences
- No docs tool ships an SEO or agent-readiness score that runs in CI on the build itself. afdocs runs in CI but fetches a deployed URL; ReadMe's audit is dashboard-only; Mintlify's and Fern's scores are web forms.
- The three vendor scores use 22, 28, and 29 checks from the same spec, so scores are not comparable across tools, and vendors score each other's docs publicly; the score is marketing as well as measurement.
- A docs toolchain that knows its own output could answer many spec checks before deploy (see the last section), which is a position none of the vendors occupies.

### Gaps
- Ahrefs Site Audit's issue categories and whether it has an llms.txt or AI check: no primary source found.
- Semrush's "AI Search Health" score: only third-party descriptions were found; the official page read does not mention it.
- Sitebulb's hint categories and any AI-specific hints were not confirmed.
- Lighthouse's Agentic Browsing audits have no developer.chrome.com page that was read; release version and date come from secondary sources.

## Content controls bearing on SEO and AEO

### Takeaway
Drafts, hidden pages, and last-updated dates exist everywhere but mean different things in each tool; description is optional in every tool read, and none requires it by default.

### Cited Findings
- Description is optional by default: Starlight (only `title` required; schema can be extended to require it) — [Starlight frontmatter](https://starlight.astro.build/reference/frontmatter/); Mintlify lists pages without a description in llms.txt with none, using the first paragraph truncated at 300 characters otherwise — [Mintlify llms.txt](https://www.mintlify.com/docs/ai/llmstxt); Docusaurus falls back to the first line of content — [plugin-content-docs](https://docusaurus.io/docs/api/plugins/@docusaurus/plugin-content-docs)
- Last-updated dates: Mintlify "last modified" timestamps enabled in `docs.json` (2025-12-08) and modification dates in `WebPage` JSON-LD — [Mintlify changelog](https://www.mintlify.com/docs/changelog); Docusaurus `showLastUpdateTime` and `showLastUpdateAuthor` default false, need full git history, and can be overridden by `last_update` front matter — [plugin-content-docs](https://docusaurus.io/docs/api/plugins/@docusaurus/plugin-content-docs); Starlight `lastUpdated` default false, from git, "may not be accurate" on shallow clones, overridable by a date in front matter — [Starlight configuration](https://starlight.astro.build/reference/configuration/); VitePress `lastUpdated` default false, from git — [VitePress site config](https://vitepress.dev/reference/site-config); Sphinx `html_last_updated_fmt` default None (build time, not git) — [Sphinx configuration](https://www.sphinx-doc.org/en/master/usage/configuration.html); Fern `last-updated` frontmatter shows a footer date separate from sitemap `lastmod` — [Fern SEO overview](https://buildwithfern.com/learn/docs/seo/overview); Docsy shows a last-modified date linked to the commit — [Docsy adding content](https://www.docsy.dev/docs/adding-content/)
- Hidden and draft semantics: Mintlify hidden pages are out of sitemap, search, assistant, and MCP, and get `noindex` — [Mintlify SEO](https://www.mintlify.com/docs/optimize/seo); GitBook hidden pages stay indexed by search engines and LLMs unless indexing is off — [GitBook changelog](https://gitbook.com/docs/changelog); Docusaurus `unlisted` versus `draft` — [plugin-content-docs](https://docusaurus.io/docs/api/plugins/@docusaurus/plugin-content-docs); Mintlify `.mintignore` excludes files from publishing and search indexing (2025-12-01) — [Mintlify changelog](https://www.mintlify.com/docs/changelog)
- Title templates and separators: Docusaurus `titleDelimiter` `|`; Starlight `titleDelimiter` `|`; VitePress `titleTemplate` with `:title`; Redocly `projectTitle` suffix; Mintlify "Page Title - Site Name"; Fern `headline` overrides the `<title>` — sources as cited under Metadata.
- Heading anchors: Docusaurus `markdown.anchors.maintainCase` defaults to false (lowercased ids) — [Docusaurus config](https://docusaurus.io/docs/api/docusaurus-config); Starlight `markdown.headingLinks` default true — [Starlight configuration](https://starlight.astro.build/reference/configuration/); Sphinx `html_permalinks` default true — [Sphinx configuration](https://www.sphinx-doc.org/en/master/usage/configuration.html); Docsy 0.14.0 added heading aliases and in-page targets (search snippet) — [Docsy blog index](https://www.docsy.dev/blog/index.md)
- Trailing slashes: Docusaurus `trailingSlash` default undefined; Mintlify `seo.trailingSlash`; VitePress `cleanUrls` default false; MkDocs `use_directory_urls` default true — sources as cited above.
- Page status: Material front matter `status` (`new`, `deprecated`, or custom) — [Material reference](https://squidfunk.github.io/mkdocs-material/reference/)
- Hugo's Open Graph template emits `article:published_time` and `article:modified_time` from `date` and `lastmod` — [Hugo embedded templates](https://gohugo.io/templates/embedded/)
- The Agent-Friendly spec's `section-header-quality` check asks that section headers be descriptive, not generic — [Agent-Friendly Documentation Spec](https://agentdocsspec.com/spec/web/)

### Inferences
- "Description required" is available nowhere as a default and only by schema extension in Starlight; a content model that can require it per page type is ahead of the field.
- Heading-anchor stability (an anchor kept when a heading is reworded) has almost no tooling; Docsy's heading aliases are the only instance found.

### Gaps
- Authorship and reading-time features were not researched beyond Docusaurus's last-update author.
- Material's `git-revision-date-localized` and `git-committers` plugins were not read.

## What users ask for and do not get

### Takeaway
The clearest unmet demand is in open-source generators: built-in llms.txt and Markdown output (Docusaurus, Nextra), a sitemap (mdBook), and parity of SEO plugins during the Material to Zensical move. On hosted platforms the changelogs show the complaints after the fact, as fixes.

### Cited Findings
- Docusaurus has no official llms.txt plugin; a 2026 write-up links a feature request and describes wanting `.md` URLs and `text/markdown` negotiation; at least four community plugins exist (search snippet) — [lekoarts.de](https://www.lekoarts.de/how-to-add-llms-txt-to-docusaurus/); [docusaurus-plugin-llms](https://github.com/rachfop/docusaurus-plugin-llms)
- Nextra: an open issue "Automated generation of llms.txt" with 16 reactions, and an open bug (2025-09-14) that the copy button copies MDX source rather than rendered Markdown, exposing source the author wanted private (search snippet) — [Nextra issue 4791](https://goodfirstissue.org/shuding/nextra/issues/4791)
- mdBook: sitemap pull request #1618 open with users asking for movement and reporting Google indexing problems; the author cites limited maintainer time (search snippet) — [mdBook PR 1618](https://github.com/rust-lang/mdBook/pull/1618)
- MkDocs has been unmaintained since August 2024, Material for MkDocs entered maintenance mode on 2025-11-05 with at least 12 months of critical fixes, and Zensical supports only a subset of plugins — [Material blog: Zensical](https://squidfunk.github.io/mkdocs-material/blog/2025/11/05/zensical/)
- A January 2026 issue noted Zensical lacked parity for the blog, macros, RSS, redirects, and social cards plugins (search snippet) — [Zensical compatibility](https://zensical.org/compatibility/features/)
- Problems visible as later fixes in Mintlify's changelog: Google indexing raw `.md` files (fixed 2025-08-30); `.md` links unreachable for ChatGPT (2025-10-03); hidden pages in the sitemap (2026-04-10); nested llms.txt 404s (2026-07-10); invalid `charset`, `canonical`, and `og:*` tags (2026-07-02); empty breadcrumb items in JSON-LD (2026-07-16); MDX comments leaking into `.md` output (2026-08-07) — [Mintlify changelog](https://www.mintlify.com/docs/changelog)
- GitBook's Broken links report needed false-positive fixes for private repositories, CNAME hosts, and placeholder hostnames (2026-09-16) — [GitBook changelog](https://gitbook.com/docs/changelog)
- Fern's Agent Score reports llms.txt coverage gaps on large sites, for example 1257 of 1495 sitemap pages on Vercel's docs, and partial content negotiation (search snippet) — [Fern Agent Score: Vercel](https://buildwithfern.com/agent-score/company/vercel-com-docs)
- Bing's AI Performance report has no API (Microsoft said it is on the backlog) and no click data (search snippet) — [Search Influence](https://www.searchinfluence.com/blog/bing-ai-performance-report-copilot-citations/)
- Read the Docs' canonical URL is passed as an environment variable that the documentation tool must be configured to use — [RTD canonical URLs](https://docs.readthedocs.com/platform/stable/canonical-urls.html)

### Inferences
- The recurring bug class on platforms that do ship AEO features is disagreement between outputs: sitemap versus hidden, HTML versus Markdown, llms.txt versus sitemap, comments or source leaking into agent output. These are consistency checks over one site model, and each was found in production, not by a build check.
- Markdown export that leaks source (Nextra) or MDX comments (Mintlify) shows that "serve the source file" is not the same as "serve the page as Markdown"; a compiler that renders to Markdown as an output has an advantage.

### Gaps
- Issue trackers and feature boards were not read directly (GitHub issue pages, Mintlify and GitBook community forums, ReadMe's feature board). Demand evidence here is from changelogs and search snippets; reaction counts and current status are unverified.
- No evidence was gathered on what users of Read the Docs, Antora, Sphinx, Hugo, or Starlight ask for.

## Which checks are deterministic and could run at build time with no crawl and no network

### Takeaway
Most of what crawler-based SEO audits and the agent-readiness spec report about titles, descriptions, headings, canonicals, sitemap membership, internal links, redirects, and llms.txt is decidable from a static description of the site; what is not decidable is anything about HTTP behavior (status codes, headers, negotiation, compression, bot protection) and anything about external URLs.

### Cited Findings
- Of the Agent-Friendly spec's 28 checks, those that concern file contents can be read off a build: `llms-txt-valid`, `llms-txt-size`, `llms-txt-links-markdown`, `llms-txt-directive-html`, `llms-txt-directive-md`, `page-size-markdown`, `page-size-html`, `markdown-code-fence-validity`, `markdown-link-portability`; those needing live responses include `content-negotiation`, `page-size-transfer`, `http-status-codes`, `redirect-behavior`, `cache-header-hygiene`, `auth-gate-detection`, `auth-alternative-access`, `bot-protection-interference`. The static or live classification is this research's assessment of the spec's check definitions, not the spec's own — [Agent-Friendly Documentation Spec](https://agentdocsspec.com/spec/web/)
- afdocs itself runs only against a URL — [afdocs repository](https://github.com/agent-ecosystem/afdocs)
- Lighthouse SEO audits whose inputs are in the built HTML: `document-title`, `meta-description`, `link-text`, `crawlable-anchors`, `image-alt`, `hreflang`, `canonical`, `is-crawlable` (the meta robots part), `robots-txt` validity; `http-status-code` needs a server — [Lighthouse default config](https://raw.githubusercontent.com/GoogleChrome/lighthouse/main/core/config/default-config.js)
- Lighthouse's `llms-txt` audit checks basic conformance, such as starting with an H1 and containing at least one Markdown link (search snippet) — [letsdatascience.com](https://letsdatascience.com/news/lighthouse-flags-llmstxt-when-links-lack-markdown-d74a1479)
- Screaming Frog issue types that depend only on page content and the URL set: missing, duplicate, multiple, or over-length titles and meta descriptions; missing, multiple, duplicate, or non-sequential H1 and H2; exact duplicates and low-content pages; missing alt text; missing, conflicting, or non-indexable canonicals; noindex and nofollow directives; missing hreflang return links; pages without internal outlinks; high crawl depth; orphan URLs; URLs not in the sitemap; uppercase, underscore, or space in URLs — [Screaming Frog issues](https://www.screamingfrog.co.uk/seo-spider/issues/)
- Semrush's AI Search checks (llms.txt present, non-descriptive anchor text, pages with one incoming internal link, links with no anchor text) all depend only on site content and the link graph — [Semrush news](https://semrush.com/news/411622-optimize-for-ai-search-with-site-audit)
- Build-time checks that tools already run with no network: Docusaurus broken links, anchors, and duplicate routes; MkDocs link, anchor, and nav validation; VitePress dead links; Sphinx nitpicky references and orphan pages; Fern `broken-links` and `missing-redirects` rules; Mintlify redirect-destination existence — [Docusaurus config](https://docusaurus.io/docs/api/docusaurus-config); [MkDocs configuration](https://www.mkdocs.org/user-guide/configuration/); [VitePress site config](https://vitepress.dev/reference/site-config); [RTD SEO guide](https://docs.readthedocs.com/platform/stable/guides/technical-docs-seo-guide.html); [Fern changelog 2026-04-02](https://buildwithfern.com/learn/docs/changelog/2026/4/2.md); [Mintlify changelog](https://www.mintlify.com/docs/changelog)
- Checks that need a network or a deployed site, per the tools that run them: external link checking (Sphinx `linkcheck`, `fern docs link check`), 404s from real traffic (GitBook, Read the Docs), and all analytics — [Sphinx builders](https://www.sphinx-doc.org/en/master/usage/builders/index.html); [Fern changelog 2026-05-14](https://buildwithfern.com/learn/docs/changelog/2026/5/14); [GitBook changelog](https://gitbook.com/docs/changelog); [RTD traffic analytics](https://docs.readthedocs.com/platform/stable/traffic-analytics.html)

### Inferences
Deterministic from a static site description (routes, titles, descriptions, headings, links, redirects, versions, flags), with no crawl:
- Title: missing, duplicate across routes, over a length, identical to the site name, or produced by a fallback.
- Description: missing, duplicate, too short or too long, identical to the first sentence by fallback.
- Headings: exactly one H1, no skipped levels, duplicate heading text within a page, generic headings ("Overview", "Introduction") repeated across pages, duplicate anchor ids.
- Canonical: present, absolute, points at an existing indexable route, no canonical chain, old-version pages have a target in the current version.
- Indexing consistency: a `noindex`, draft, or hidden page that is in the sitemap, in llms.txt, or linked from navigation; an indexable page missing from the sitemap; an indexable page with no inbound internal link (orphan) or only one.
- Links: broken internal links and anchors, links through a redirect, empty or non-descriptive link text, images without alt text.
- Redirects: destination exists, no chains, no loops, no redirect whose source is a live page, a removed or moved route (against the previous build's route list) without a redirect.
- Versions and languages: hreflang return links, per-version canonical and noindex consistency, version-only pages.
- llms.txt and Markdown outputs: file exists, has an H1 and a summary, size under a limit, every link resolves to a built Markdown file, coverage against the indexable route set, Markdown page size, valid code fences, portable (absolute or resolvable) links in Markdown, HTML and Markdown parity when both come from the same source.
- Structured data: JSON-LD well-formed, required properties present, breadcrumb items non-empty and matching navigation.
- Social images: one exists per page at the expected size; `og:url` equals the canonical.

Not decidable without a deployment: HTTP status codes and soft 404s, response headers (`Link`, `X-Robots-Tag`, cache headers, Content-Signal), content negotiation, transfer size after compression, redirect status codes as served by the host, bot protection and authentication gates, external link health, rendering of client-side content in a real browser, and anything measured from traffic.

### Gaps
- The static or live split above is an assessment, not something any source states; the spec does not classify its own checks this way.
- Whether afdocs can be pointed at a local static server to cover the header-independent checks in CI before deploy was not confirmed (its CI page was not read).
