# Archiving old documentation versions, and the repository and build cost of versions

Research date: 2026-10-09. Three kinds of evidence are used and labelled:

- **[primary]**: read at the project's own documentation, repository, or issue tracker on the research date.
- **[observed]**: my own check on 2026-10-09 (an HTTP request to the live site, or a measurement in a scratch git repository). The method is given with the finding.
- **[secondary]** or **[snippet]**: taken from a third party's summary or from a search-result excerpt; the primary source was not opened. Treat these as leads, not confirmed facts.

## Which archive forms are used most by real projects, and what do those projects say about maintaining them?

### Takeaway
The dominant form is a frozen static site kept online at its own subdomain or path with a banner ("no longer maintained", "static snapshot"), usually built once and never rebuilt. Downloadable bundles (HTML zip, EPUB, PDF), container images, and immutable host-side deploy snapshots are the common secondary forms; web-archive formats (WARC/WACZ, ZIM) are almost never the form a docs project itself publishes.

### Cited Findings

#### Frozen static site on a subdomain or path
- Kubernetes serves the current release at `kubernetes.io` and the four previous minors each on a subdomain of the form `v1-<minor>.docs.kubernetes.io` (v1.37 current; v1.36, v1.35, v1.34, v1.33 older). The page says "the availability of documentation for a Kubernetes version is separate from whether that release is currently supported." [primary] — [Kubernetes: Available Documentation Versions](https://kubernetes.io/docs/home/supported-doc-versions/)
- The v1.33 subdomain is served by Netlify and carries the notice text "no longer actively maintained. The version you are currently viewing is a static snapshot. For up-to-date information, see the [latest version]". [observed: `curl` of the page, 2026-10-09] — [v1-33.docs.kubernetes.io](https://v1-33.docs.kubernetes.io/docs/home/)
- Docusaurus lists its own versions in three tiers: current (3.10.2), "Past versions (Not maintained anymore)" still built into the main site (3.9.2 back to 3.0.1, and `/docs/2.x`), and archived versions that are links out to other hosts: 2.0.1 to 2.3.1 at `docusaurus-archive-october-2023.netlify.app`, and 1.x at `v1.docusaurus.io`. [primary] — [Docusaurus versions](https://docusaurus.io/versions)
- Docusaurus documents the method: Jamstack hosts such as Netlify save each production build under an immutable URL, and "You can include archived versions that will never be rebuilt as external links to these immutable URLs." It says Jest and Docusaurus use this "to keep the number of actively built versions low." [primary] — [Docusaurus: Versioning](https://docusaurus.io/docs/versioning)
- The Docusaurus maintainer's standing advice (27 Jul 2020, repeated 8 Aug 2022) is to "take older versions and publish them as standalone sites"; once a version is unmaintained, archiving it means it is no longer inside the production single-page app and does not add build time. Docusaurus's own site does this with a `versionsArchived.json` file and a dynamic config. On 31 Aug 2022 he floated a `docusaurus version:archive` command but wanted more user examples first. [primary] — [Docusaurus discussion #3132](https://github.com/facebook/docusaurus/discussions/3132)
- A Puppeteer maintainer reported in the same thread (27 Aug 2022) that archiving this way "was hard on GitHub Pages", so they linked to the Markdown source of old versions instead of a hosted site, and asked for lightweight versioning without rebuilding. [primary] — [Docusaurus discussion #3132](https://github.com/facebook/docusaurus/discussions/3132)
- Spectro Cloud keeps three active versions in the build and maps older ones through an `archiveVersions.json` file to an external archive domain hosted on Netlify via branch deploys (post dated 10 Apr 2024; 435+ Markdown pages at the time). [primary] — [Spectro Cloud blog](https://www.spectrocloud.com/blog/when-docs-and-a-dinosaur-git-along-enabling-versioning-in-docusaurus)
- React moved its previous documentation to `legacy.reactjs.org` (served by Vercel). [observed 2026-10-09: `legacy.reactjs.org/docs/getting-started.html` now answers HTTP 308 to `https://react.dev/learn`, so at least that old page is no longer readable at its archived address] — [legacy.reactjs.org](https://legacy.reactjs.org/docs/getting-started.html)
- Angular keeps old majors at version subdomains: `v17.angular.io/docs` answers HTTP 200 and declares `<link rel="canonical" href="https://angular.dev">`. [observed 2026-10-09] — [v17.angular.io](https://v17.angular.io/docs)
- Vue keeps old majors at `v2.vuejs.org`, `v1.vuejs.org`, `012.vuejs.org`; `v2.vuejs.org` is served by Netlify and carries "no longer actively maintained" text. [observed 2026-10-09 for v2; hostnames for v1 and 0.12 from search results] — [v2.vuejs.org](https://v2.vuejs.org/v2/guide/), [v1.vuejs.org](https://v1.vuejs.org/examples)
- Python keeps every release's documentation online under `https://docs.python.org/release/<full version>/`, listed from 3.15 down to 1.4, with the statement "Some previous versions of the documentation remain available online." [primary] — [python.org: documentation by version](https://www.python.org/doc/versions/)
- Django keeps unsupported versions on the same host and path scheme (`/en/1.8/`) with the banner "This document is for an insecure version of Django that is no longer supported. Please upgrade to a newer release!" and a version switcher that still lists 1.8 through 6.1 and dev. [primary] — [Django 1.8 docs](https://docs.djangoproject.com/en/1.8/)

#### Container image
- GitLab publishes each docs version as a container image, from 10.3 to 19.4, run with `docker run -it --rm -p 4000:4000 <image>:<version>`. Image paths changed three times: `registry.gitlab.com/gitlab-org/gitlab-docs` (10.3 to 15.5), `.../gitlab-docs/archives` (15.6 to 17.8), `.../technical-writing/docs-gitlab-com/archives` (17.9 to 19.4). "You'll need to have Docker installed to access them." [primary] — [GitLab Docs archives](https://docs.gitlab.com/archives/)
- Supported GitLab versions are additionally hosted online at a separate site, `archives.docs.gitlab.com`. [primary] — [GitLab Docs archives](https://docs.gitlab.com/archives/)
- GitLab's reason for a separate archives project: the main site deploys on every change, so a bigger image slows every deploy and costs bandwidth; the archives project deploys about once a month, so size, time, and bandwidth matter less. The archive site is built by gathering the archived versions into one Docker image, which a CI job pushes to GitLab Pages. The epic notes the cost: "Since we'll be gathering all the archived versions in one Docker image, its size would get bigger over time", and includes a task to raise the Pages and artifact size limits. No sizes are given. [primary] — [GitLab epic 8937](https://gitlab.com/groups/gitlab-org/-/epics/8937)

#### Built output committed to a branch (mike)
- mike's model is an archive by construction: "mike works by creating a new Git commit on your `gh-pages` branch every time you deploy a new version of your docs." Once a version is built "you should never need to touch that version again", so you "never have to worry about breaking changes in MkDocs". Redeploying a version replaces only that version's files. `mike delete` removes a version; aliases are symlinks by default, or redirects or copies. [primary] — [mike README](https://github.com/jimporter/mike)

#### Downloadable bundles (zip, tarball, EPUB, PDF)
- Read the Docs builds PDF, ePub, and zipped HTML by default alongside the site on each build; for Sphinx "All output formats are built mostly lossless from the documentation source"; for MkDocs and others, "Most of the extensions export the HTML outputs as another format (for instance PDF) through a conversion process." [primary] — [Read the Docs: offline formats](https://docs.readthedocs.com/platform/stable/downloadable-documentation.html)
- Python offers per-version archives as HTML (zip, tar.bz2), plain text, Texinfo, and EPUB; as of 3.15 "No pre-built PDFs are provided" and readers are told to run `make dist-pdf` themselves. [primary] — [Python docs download page](https://docs.python.org/3/download.html)
- Sizes for the Python 3.14 docset: HTML tar.bz2 8,458,087 bytes; HTML zip 12,920,275; EPUB 9,153,668; plain text tar.bz2 3,329,828; Texinfo zip 13,024,672; PDF (A4) zip 21,613,232. The HTML, text, Texinfo, and EPUB files are dated October 2026; the PDF files are dated 01-Oct-2025, that is, they stopped being regenerated. [observed: directory listing, 2026-10-09] — [docs.python.org/3.14/archives/](https://docs.python.org/3.14/archives/)
- Django's old-version pages link an HTML zip on Django's own media host and a PDF and EPUB on Read the Docs' media host ("Offline (Django 1.8)"). [primary] — [Django 1.8 docs](https://docs.djangoproject.com/en/1.8/)

#### Whole-docset export from the generator
- Antora Assembler merges an Antora component version into one document following the navigation and hands it to a converter. Exporters: `@antora/pdf-extension`, `@antora/epub-extension`, `@antora/html-single-extension`, and a custom exporter extension. It runs once "for each selected component version" during normal site generation and publishes the result as an export in the site. [primary] — [Antora Assembler](https://docs.antora.org/assembler/latest/)

#### Web-archive formats
- WACZ packages WARC data plus indexes, a page list, and metadata into a ZIP. Version 1.1.1 is the stable specification; 1.2.0 is a draft. It is designed to be hosted "by simply serving it up at a given URL as a static document, possibly from cloud object storage, or a CDN", read by HTTP Range requests without specialised server software, whereas plain WARC replay "currently requires complex server infrastructure (e.g. a Wayback Machine)". [snippet of the primary specification] — [WACZ 1.1.1](https://specs.webrecorder.net/wacz/1.1.1/), [Webrecorder: WACZ 1.0 announcement](https://webrecorder.net/blog/2021-01-18-wacz-format-1-0)
- Kiwix's Zimit crawls a site in a browser and converts the WARC output to a single ZIM file with warc2zim; by default only URLs on the main page's domain and subdomains are included; since warc2zim 2.0.0 the ZIM no longer needs service workers or HTTPS to render. [snippet] — [warc2zim on PyPI](https://pypi.org/project/warc2zim)
- ArchiveBox states its aim as content "viewable with common software in 50 - 100 years without needing to run ArchiveBox or other specialized software". [snippet] — [ArchiveBox documentation](https://docs.archivebox.io/_/downloads/en/dev/pdf/)

#### Where archives are stored: limits that decide the choice
- GitHub: files over 50 MiB warn, over 100 MiB are blocked; repositories "ideally" under 1 GB and "strongly recommended" under 5 GB. [primary] — [GitHub: About large files](https://docs.github.com/en/repositories/working-with-files/managing-large-files/about-large-files-on-github)
- GitHub Pages: published site at most 1 GB; source repository recommended at most 1 GB; soft limits of 100 GB bandwidth a month and 10 builds an hour (the build limit does not apply to a custom Actions workflow); deployments time out after 10 minutes. [primary] — [GitHub Pages limits](https://docs.github.com/en/pages/getting-started-with-github-pages/github-pages-limits)
- GitHub release assets: up to 1,000 assets per release, each under 2 GiB, and "There is no limit on the total size of a release, nor bandwidth usage." [primary] — [GitHub: About releases](https://docs.github.com/en/repositories/releasing-projects-on-github/about-releases)
- GitLab.com: repository size limit 10 GB including LFS; maximum push 5 GiB; Pages site at most 1 GB; CI artifacts at most 1 GB compressed; generic package registry files up to 5 GB. [primary] — [GitLab.com settings](https://docs.gitlab.com/user/gitlab_com/)

### Inferences
- Ranked by how often it appears among the projects checked: (1) frozen site on a subdomain or path (Kubernetes, Angular, Vue, React, Python, Django, Docusaurus, Spectro Cloud); (2) downloadable HTML/EPUB/PDF bundle (Python, Django, everything on Read the Docs); (3) built output in a branch (mike users); (4) container image (GitLab, alone among those checked); (5) a web-archive file (none of the projects checked publishes one).
- Every project that speaks about maintenance says the same thing: the archive is built once and not rebuilt. mike says so as a design principle, Docusaurus says "never be rebuilt", Kubernetes says "static snapshot".
- An archive that depends on a host feature is only as durable as that host: Docusaurus's 2.x archive lives at a Netlify deploy URL named for the month it was frozen, and GitLab's image path has moved three times. A self-contained file a project can re-host anywhere (zip, single HTML, PDF) avoids this.
- Release assets are the least constrained first-party storage on GitHub (2 GiB a file, no total or bandwidth limit), while Pages and the repository itself are both held to about 1 GB. An archive that must not live in git fits release assets or object storage better than a branch.
- A whole-docset export per version, as Antora Assembler does, is the closest existing feature to "archive as a product of the build": the same content model yields a site, a single HTML file, a PDF, and an EPUB.

### Gaps
- No primary statement was found for how React, Angular, or Vue build and deploy their archived sites (whether each is a pinned branch redeployed or a one-time upload). The hosting providers were read from response headers only.
- SingleFile, MHTML, HTTrack, `wget --mirror`, Browsertrix, and IIPC guidance were not researched at primary sources; no documentation project was found that publishes its archive in any of them.
- The claim that WACZ is used by the Internet Archive and Library of Congress comes from a vendor page (PageCrawl) and was not confirmed.
- No size was found for any GitLab docs archive image, and no project was found that publishes docs archives through a package registry (npm, PyPI) as its stated archive channel.
- Whether projects deliberately rely on the Internet Archive as their archive of record was not found stated anywhere.

## What breaks first in a frozen site over time, according to published accounts?

### Takeaway
No published postmortem of an archived docs site was found; the evidence is piecemeal. What it shows is that the first failures are in what the frozen pages depend on outside themselves: the old URL itself (redirected away), hosted search, third-party scripts, and the build toolchain if anyone tries to rebuild.

### Cited Findings

#### The address, and search-engine signals
- React's archived page `legacy.reactjs.org/docs/getting-started.html` now redirects (HTTP 308) to `react.dev/learn`; the archived content is not served at that address. [observed 2026-10-09] — [legacy.reactjs.org](https://legacy.reactjs.org/docs/getting-started.html)
- Canonical handling differs by project. Django 1.8 pages declare `<link rel="canonical" href="https://docs.djangoproject.com/en/6.1/">` (the site root of the current version, not the equivalent page) together with `<meta name="ROBOTS" content="ALL">`. Python 3.8 pages declare a canonical of `https://docs.python.org/3/index.html` and Python 2.7 pages declare `https://docs.python.org/2/index.html`. Angular v17 declares `https://angular.dev`. Docusaurus's `/docs/2.x` declares itself as canonical. [observed 2026-10-09] — [Django 1.8](https://docs.djangoproject.com/en/1.8/), [Python 3.8](https://docs.python.org/3.8/), [Python 2.7](https://docs.python.org/2.7/), [v17.angular.io](https://v17.angular.io/docs), [Docusaurus 2.x](https://docusaurus.io/docs/2.x/)
- Read the Docs separates "hidden" from "inactive". Hidden versions "are not listed on the flyout menu on the docs site and are not shown in search results", are listed as `Disallow: /path/to/version/` in the default `robots.txt`, and stay reachable: "Hiding a version doesn't make it private, any user with a link to its docs can still see it." Inactive versions "have their documentation content deleted and builds cannot be triggered." [primary] — [Read the Docs: Versions](https://docs.readthedocs.com/platform/stable/versions.html)
- Read the Docs shows a notice on any version other than `stable` saying readers "may be reading an outdated version of the documentation"; admins can configure it. [primary] — [Read the Docs: Versions](https://docs.readthedocs.com/platform/stable/versions.html)
- mike has a `canonical_version` option, "useful for telling search engines what pages to prefer", and rewrites `site_url` for the version being built. [primary] — [mike README](https://github.com/jimporter/mike)
- Community positions conflict. An OpenStack thread lists `noindex` on old versions and removal as options and notes that removal produces 404s for existing search results; a PostgreSQL thread argues that blocking indexing of everything but the current version hurts people who need old versions and suggests blocking only unsupported ones; a Docusaurus discussion added a `noIndex` option per version while a maintainer suggested canonical links instead because Google's guidance on versioned docs is unclear. [snippet] — [OpenStack list thread](https://lists.openstack.org/archives/list/openstack-discuss@lists.openstack.org/thread/Q3AJ5C54IZSGBJ76JTVXTWVTFNXDRD7N/), [PostgreSQL list message](https://www.postgresql.org/message-id/CABUevEzmy02nUWdisHNCoM9-19vWE1xATinWfFgW6n-iFd3qUQ%40mail.gmail.com), [Docusaurus discussion #7960](https://github.com/facebook/docusaurus/discussions/7960)

#### Search inside the archive
- Archived sites still ship hosted search and analytics code: `v1.docusaurus.io`, `v2.vuejs.org`, and Docusaurus `/docs/2.x` reference Algolia DocSearch; `v17.angular.io`, `v1.docusaurus.io`, and Docusaurus `/docs/2.x` reference Google Analytics or Tag Manager. [observed 2026-10-09: string match in the served HTML; whether each still functions was not tested] — [v1.docusaurus.io](https://v1.docusaurus.io/docs/en/installation), [v2.vuejs.org](https://v2.vuejs.org/v2/guide/), [v17.angular.io](https://v17.angular.io/docs)
- A DocSearch configuration issue records search that "stopped working" for a project's external API reference pages; the configuration repository was later archived, which limits the fix path. [snippet] — [algolia/docsearch-configs #739](https://github.com/algolia/docsearch-configs/issues/739)
- GitLab had to build search for its archives separately: "Version-specific search was introduced in 15.6 for the online archives" and "Offline search also works in self-hosted environments as of 16.6." [primary] — [GitLab Docs archives](https://docs.gitlab.com/archives/)
- Pagefind's claim for a search that ships with the static output: full-text search of a 10,000-page site in under 300 kB of network transfer including the library, because the index is split into chunks fetched per query. [snippet of the project's own site] — [pagefind.app](https://www.pagefind.app)
- One site with 489 articles reported a Pagefind index of about 5 MB on disk including the WebAssembly runtime. [snippet] — [raymii.org](https://raymii.org/s/blog/Site_update_self_hosted_search_via_pagefind.html)
- Pagefind can merge several separately built indexes at query time in the browser (`mergeIndex`), with per-index weighting; cross-domain use needs CORS headers. [snippet of primary docs] — [Pagefind: Searching multiple sites](https://pagefind.app/docs/multisite/)
- Hugo forum users reported Pagefind failing to build at roughly 250,000 to 300,000 pages on their machines. [snippet, anecdotal] — [Hugo forum](https://discourse.gohugo.io/t/scaling-pagefind-to-over-1-million-pages/56109)

#### External assets
- A tool that rebuilds static snapshots from the Wayback Machine had fetched missing assets from the live internet and saved whatever came back; because an archived domain "may belong to someone else today", a parking page was saved as `app.js`. The fix limited live fetches to a short allowlist (Google Fonts, `code.jquery.com`, Squarespace CDN hosts). [snippet] — [GeiserX/Wayback-Archive PR #55](https://github.com/GeiserX/Wayback-Archive/pull/55)

#### The build toolchain (why output is snapshotted, not source)
- Read the Docs' reproducible-builds guide: "An update in a dependency can break your builds when least expected, or make your docs look different from your local version"; transitive dependencies also need pinning, and the OS and Python version should be fixed in the build config. [snippet of primary docs] — [Read the Docs: reproducible builds](https://docs.readthedocs.io/page/guides/reproducible-builds.html)
- CPython keeps a `requirements-oldest-sphinx.txt` that pins the oldest supported Sphinx with its whole dependency tree, taken from a freeze of a known-good environment; an older Sphinx 3.2 setup also had to hold back docutils, Jinja2, and MarkupSafe because Sphinx 3.2 is incompatible with their newer releases. [snippet] — [CPython Doc/requirements-oldest-sphinx.txt (mirror)](https://chromium.googlesource.com/external/github.com/python/cpython/+/6eaddc10e972273c1aed8b88c538e65e4773496e/Doc/requirements-oldest-sphinx.txt)
- AiiDA found that Sphinx 7.4.0 broke its docs build and failed on incremental builds where an output directory from an older Sphinx existed; it pinned to a minor version. [snippet] — [aiidateam/aiida-core PR #6527](https://github.com/aiidateam/aiida-core/pull/6527)
- Trusted Firmware pinned Sphinx below 7.0.0 because of incompatibility with older `sphinx-rtd-theme`. [snippet] — [TF-A tests commit](https://review.trustedfirmware.org/plugins/gitiles/TF-A/tf-a-tests/+/be38d47c53d5a2fdcf720567a73a3a3ed44ee07b)
- A SymPy maintainer's position: building docs "is more of a development activity so we don't really need to support older versions of anything." [snippet] — [sympy/sympy PR #25543](https://github.com/sympy/sympy/pull/25543)
- mike's design rests on the same point: built versions stay in the branch so that later MkDocs releases cannot break them. [primary] — [mike README](https://github.com/jimporter/mike)
- Python's 3.14 PDF archive stopped being regenerated in October 2025 while the HTML, EPUB, and text archives continued. [observed 2026-10-09] — [docs.python.org/3.14/archives/](https://docs.python.org/3.14/archives/)

#### Compliance reasons to keep old versions
- EU electronic instructions for use for medical devices (Implementing Regulation (EU) 2021/2226, amended by (EU) 2025/1234, in force 16 July 2025): secondary summaries say manufacturers must retain all electronic versions with their publication dates and make superseded versions available, on request, for the retention period; one summary says the amendment removed an obligation to publish all historical versions on the website; another says historical versions must still be kept on the website (Art. 5(13)). The summaries conflict; retention periods of 10 years, and 15 years for implantables and devices without an expiry date, are cited from a draft-stage summary. **[secondary; not confirmed at EUR-Lex]** — [NSF summary](https://www.nsf.org/life-science-regulatory-news/eu-expands-application-of-electronic-instructions-for-use-for-medical-devices), [Efor analysis](https://efor-group.com/en/evolution-european-regulatory-framework-electronic-instructions-use-eifu-of-medical-devices-analysis-of-regulation-eu-2025-1234-amending-regulation-eu-2021-2226/), [Sidley, May 2021](https://www.sidley.com/en/insights/newsupdates/2021/05/digitalization-instructions-for-use-eu-issues-updated-framework-for-eifus-of-medical-devices), [consolidated text on EUR-Lex](https://eur-lex.europa.eu/eli/reg_impl/2021/2226/2025-07-16/eng/pdf)
- Kubernetes states the separation explicitly: documentation availability for a version "is separate from whether that release is currently supported." [primary] — [Kubernetes: Available Documentation Versions](https://kubernetes.io/docs/home/supported-doc-versions/)

#### AI agents and archived docs
- Vendor write-ups report that assistants trained on older material produce answers from outdated documentation, and that supplying current Markdown and an `llms.txt` reduces but does not remove the errors, which tend to be near misses such as an old import path or deprecated argument. Blockdaemon and ReadMe describe version metadata in their generated `llms.txt`. These are vendors describing their own products, not independent tests. [snippet] — [MotherDuck](https://motherduck.com/blog/fix-outdated-llm-documentation-duckdb/), [Zyte](https://dev.zyte.com/blog/how-we-put-dev-docs-in-ai-spotlight/), [Blockdaemon](https://docs.blockdaemon.com/docs/ai-optimization), [ReadMe](https://docs.readme.com/main/docs/LLMstxt)

### Inferences
- Order of failure suggested by the evidence: (1) the archive's own address, when the owner later redirects or retires the host (React); (2) hosted search, whose index and credentials live with a third party; (3) analytics and other third-party scripts, which keep running or keep failing unnoticed; (4) assets on domains that change hands; (5) the ability to rebuild, which is gone within a few generator majors unless the whole environment was pinned.
- A frozen page's `canonical` goes stale in a specific way: Django and Python point every old page at the current version's index, not at the equivalent page, which avoids maintaining a per-page mapping that would need updating each release. An archive should either omit the canonical, point it at itself, or write one stable target at freeze time; anything that must be rewritten later contradicts "frozen".
- `noindex`, `robots.txt`, sitemap exclusion, and exclusion from site search are properties of the live site's configuration as much as of the archive. Read the Docs applies them from outside (the robots file and the flyout) without touching the built version, which is the model that keeps the snapshot immutable.
- A banner baked into the snapshot at freeze time cannot name the current version without going stale. Kubernetes and Django word theirs without a version number ("see the latest version", "upgrade to a newer release"), which works indefinitely.
- An archive that should outlive its host needs search that ships with it. Pagefind-style chunked client-side indexes are the published option that scales to a 10,000-page docset; GitLab's two-year gap between online and offline search for archives shows this is not free if left for later.
- For a regulated publisher the requirement, on the available summaries, is retention with dates and retrieval on request, not public hosting. A dated, immutable, self-contained file per version satisfies that better than a live site.
- No source found says that `noindex` or `robots.txt` reliably controls what AI crawlers or coding assistants read. Labelling the version in the page text itself is the only measure that travels with the content.

### Gaps
- No published postmortem of a frozen documentation site decaying over years was found.
- No study or project statement was found on accessibility of frozen output (for example, an archive falling behind WCAG revisions). Not covered by any source read.
- No primary guidance from an AI vendor or the `llms.txt` proposal on marking archived or superseded documentation was found.
- Whether the Kubernetes and GitLab archive sites send `noindex` (meta or `X-Robots-Tag`) was not determined; a pattern match on the served HTML found no robots meta tag on either, which is weak evidence of absence.
- No US source (for example FDA labelling or 21 CFR Part 11) or contractual support-window source was researched. The EU summaries conflict and need the consolidated regulation read directly.
- No link-rot measurement specific to documentation sites was found.

## Is a whole-docset PDF or single-file HTML practical at 100, 1,000, and 10,000 pages? Any published sizes or timings?

### Takeaway
Published numbers show whole-docset PDF is routine at the scale of a large reference manual (the Python docs: about 22 MB zipped; the Linux kernel docs: 5 to 11 minutes), with failures coming from individual oversized blocks, not page count. No published measurement was found for a 10,000-source-page docset as one PDF or one HTML file.

### Cited Findings
- Python 3.14's complete documentation as PDF (A4, several PDFs in one archive) is 21.6 MB zipped; the same docset is 12.9 MB as zipped multi-page HTML and 9.2 MB as one EPUB. [observed 2026-10-09] — [docs.python.org/3.14/archives/](https://docs.python.org/3.14/archives/)
- Linux kernel documentation: a 2025 patch to build PDF files in parallel reports the PDF build taking about 5 minutes on a Ryzen 9 machine with 32 threads, against about 11 minutes serial. [snippet] — [LKML, Sept 2025](https://lkml.iu.edu/hypermail/linux/kernel/2509.0/01042.html)
- Sphinx's own documentation as PDF (2018): about 57 seconds, of which about 48 seconds was Sphinx and 9 seconds pdflatex; the LaTeX writer's footnote renumbering was the slow step. [snippet; issue number is one of the two linked] — [sphinx-doc/sphinx #4803](https://redirect.github.com/sphinx-doc/sphinx/issues/4803), [#4362](https://redirect.github.com/sphinx-doc/sphinx/issues/4362)
- LaTeX failures come from single large blocks: a verbatim block of more than 4,300 lines exhausted TeX's main memory in the kernel docs (2016); BIND 9 hit the same with a block of more than twenty thousand lines, and raising `extra_mem_bot` made the build hang, so each release's notes were split into separate blocks; OpenStack Nova's PDF failed with "Dimension too large" on a large sample policy file after Sphinx 3.1.0 and the file was kept in HTML only. [snippet] — [LKML 2016](https://lkml.iu.edu/hypermail/linux/kernel/1611.0/00975.html), [BIND 9 MR 9266](https://gitlab.isc.org/isc-projects/bind9/-/merge_requests/9266), [Nova commit](https://opendev.org/openstack/nova/commit/b2f07a4959169fa42a263e6cae27d4b859808d00)
- OpenStack's technical committee discussed (14 Mar 2025) PDF docs jobs timing out for projects with large documentation. [snippet] — [OpenStack TC IRC log](https://meetings.opendev.org/irclogs/%23openstack-tc/%23openstack-tc.2025-03-14.log.html)
- Typst: Typst's own site cites Zerodha's figure that a 2,000-page, table-heavy document compiles in about 1 minute against about 18 minutes with lualatex. Vendor-reported; the Zerodha post (14 Feb 2024) was not opened to confirm. [snippet] — [Typst pricing page](https://typst.app/pricing/), [Zerodha tech blog](https://zerodha.tech/blog/1-5-million-pdfs-in-25-minutes/)
- Prince: a forum user reported more than 10 minutes for a 1,200-page PDF that was mostly one large table, on a small Windows cloud instance. [snippet; anecdotal, and which of the two forum threads holds it was not confirmed] — [Prince forum topic 3227](https://www.princexml.com/forum/topic/3227?post=15783), [topic 2003](https://princexml.com/forum/topic/2003/memory-consumption-of-prince-vs-dompdf)
- WeasyPrint's documentation is quoted as saying "Tables are known to be slow, especially when they are rendered on multiple pages." [secondary quotation] — [mkdocs-with-pdf on PyPI](https://pypi.org/project/mkdocs-with-pdf) (plugin that depends on WeasyPrint; its example log shows "Converting 10 articles to PDF took 7.1s")
- Chromium-based conversion, vendor figures for their own products: about 800 pages from a 13 MB HTML page in 23 seconds at about 500 MB peak memory (ABCpdf); a 1,000+ page conversion in 6.72 seconds at 813 MB (Syncfusion). [snippet, vendor data] — [ABCpdf 16](https://websupergoo.com/abcpdf-16.htm), [Syncfusion benchmarks](https://help.syncfusion.com/document-processing/pdf/conversions/html-to-pdf/net/performance-metrics)
- Antora Assembler's documented limits are about source constructs, not size: Markdown-style headings unsupported; a `//` comment in a table cell loses block images; an unterminated delimited block on one page disrupts every later page in the assembly; xref shorthand for page references unsupported; inline macros spanning lines are left unprocessed; attributes set in the document body do not affect reference resolution. [primary] — [Antora Assembler: Known issues](https://docs.antora.org/assembler/latest/known-issues/)
- `mkdocs-print-site-plugin` produces one combined page and leaves PDF creation to the browser's print dialog; it must be last in the plugin list. `mkdocs-with-pdf` renders with WeasyPrint and notes "WeasyPrint and Google Chrome are not fully compatible"; a fork replaces WeasyPrint with Chrome. [snippet] — [mkdocs-print-site-plugin](https://pypi.org/project/mkdocs-print-site-plugin), [mkdocs-with-pdf](https://pypi.org/project/mkdocs-with-pdf), [mkdocs-with-pdf-browser](https://pypi.org/project/mkdocs-with-pdf-browser)
- The Ragas project keeps PDF generation in a separate MkDocs config that inherits the main one, and does not enable it in its Read the Docs build. [snippet] — [Ragas: PDF export](https://docs.ragas.io/en/latest/community/pdf_export/)
- mdBook generates a `print.html` containing the whole book for the browser's print function; `mdbook-pdf` drives headless Chrome over the DevTools protocol and needs a browser installed. [snippet] — [mdBook printing notes](https://mdbook.code-maven.com/printing), [mdbook-pdf image](https://hub.docker.com/r/hollowman6/mdbook-pdf)
- Docusaurus has no first-party PDF. Third-party tools crawl the rendered site: `docusaurus-prince-pdf` (needs Prince; its listing notes a watermark and licence cost and that the method "doesn't work well" with Docusaurus sites), `docs-to-pdf` (Puppeteer; follows a pagination selector), `docusaurus-pdf` ("not intended to be used during build process"). [snippet] — [docusaurus-prince-pdf](https://github.com/signcl/docusaurs-prince-pdf), [docs-to-pdf](https://www.npmjs.com/package/docs-to-pdf), [docusaurus-pdf README](https://cdn.jsdelivr.net/npm/docusaurus-pdf@1.2.0/README.md)
- Interactive content: Docusaurus renders all tabs eagerly at build unless the lazy option is set, so hidden tab content is in the HTML. Fumadocs' PDF guide says content in accordions and tabs is invisible to print and recommends overriding those components for a print build. Confluence has an open bug that native PDF export does not export all tabs. [snippet] — [Docusaurus: Tabs](https://docusaurus.io/docs/markdown-features/tabs), [Fumadocs: Export PDF](https://www.fumadocs.dev/docs/guides/export-pdf), [Atlassian CONFCLOUD-84964](https://jira.atlassian.com/browse/CONFCLOUD-84964)
- Read the Docs builds PDF, ePub, and zipped HTML on every commit for Sphinx projects. [primary] — [Read the Docs: offline formats](https://docs.readthedocs.com/platform/stable/downloadable-documentation.html)

### Inferences
- At 100 source pages, every route works (browser print of a combined page, WeasyPrint, LaTeX, Typst); the choice is about fidelity, not feasibility.
- At about 1,000 source pages (thousands of printed pages), a native typesetting route (LaTeX, Typst) or a Chromium conversion finishes in seconds to minutes on the published numbers; WeasyPrint and Prince have reported slowness on long tables; the browser print dialog on one enormous page is the weakest route, though no measurement of it was found.
- At 10,000 source pages nothing published confirms a single PDF or single HTML file is workable. The Python docs split their PDF into several documents rather than one, and Antora Assembler's unit is one component version, which suggests splitting by section or book is the practised answer.
- A whole-docset PDF costs roughly the same order of bytes as the zipped HTML (Python: 21.6 MB against 12.9 MB), so size is not the obstacle to storing one per version outside git.
- PDF is the first output a project drops when maintenance tightens: Python stopped publishing pre-built PDFs while keeping HTML, EPUB, and text. An archive feature that treats print as primary should expect the same pressure.
- The recurring technical failure is one outsized block (a long literal, a wide table), not total length. A linter that flags such blocks for the print output would address the failures actually reported.
- Tabs, accordions, and other state-dependent content need an explicit print rule (expand all, in order). Three unrelated tools document the same omission.

### Gaps
- No published size or timing for a single self-contained HTML file of a whole docset (Antora's html-single exporter, SingleFile, or otherwise) was found.
- No controlled comparison of WeasyPrint, Prince, Paged.js, and Typst on the same long document was found; no Paged.js performance data at all.
- No numbers for Asciidoctor PDF, rinohtype, Pandoc, or the Sphinx Typst builders were found.
- No page count for the Python or kernel PDFs was collected, so bytes or seconds per page cannot be derived.
- The Zerodha and Prince figures were not confirmed at their primary sources.
- Link handling in merged PDFs (cross-page references becoming internal links) is documented by Antora Assembler in outline only; no comparison across tools was found.

## What does a copied version actually cost in git, given blob deduplication and delta compression? Where does the cost really come from (working tree, builds, search, review noise)?

### Takeaway
In git's object store a copied folder is nearly free: unchanged files are the same blob, and edited copies delta-compress against their originals once packed. The cost that grows linearly is everything outside the object store: the working tree, the file count every tool walks, the build, the search index, and edits landing in the wrong copy.

### Cited Findings

#### How git stores copies
- Git is "a content-addressable filesystem"; a blob's key is the SHA-1 of a header (`blob <size>\0`) plus the content; the blob does not hold a file name ("you aren't storing the filename in your system — just the content"); names live in tree objects, each entry being "the SHA-1 hash of a blob or subtree with its associated mode, type, and filename." Storage is "a single file per piece of content, named with the SHA-1 checksum of the content and its header." [primary] — [Pro Git: Git Objects](https://git-scm.com/book/en/v2/Git-Internals-Git-Objects)
- Loose objects hold each version whole, zlib-compressed: adding one line to a 22 KB, 400-line file "stored that new content as a completely new object"; the two versions were each about 7 KB on disk. [primary] — [Pro Git: Packfiles](https://git-scm.com/book/en/v2/Git-Internals-Packfiles)
- Packing stores deltas: "When Git packs objects, it looks for files that are named and sized similarly, and stores just the deltas from one version of the file to the next." In the example, loose objects of about 15 KB became a 7 KB packfile; the newer 22,054-byte version is stored whole and the older one as a delta (shown as 9 bytes in the book's listing). Git packs "if you have too many loose objects around, if you run the `git gc` command manually, or if you push to a remote server." [primary] — [Pro Git: Packfiles](https://git-scm.com/book/en/v2/Git-Internals-Packfiles)
- How delta candidates are found: "The objects are first internally sorted by type, size and optionally names and compared against the other objects within --window"; "The default value for --window is 10 and --depth is 50." Objects are grouped by a "name hash" of the path; the default hash (version 1) "depends primarily on the final 16 bytes of the path. If there are many paths in the repo that have the same final 16 bytes and differ only by parent directory, then this name-hash may lead to too many collisions and cause poor results." Name-hash version 2 and `--path-walk` exist to improve this case. Delta compression is skipped for objects above `core.bigFileThreshold`. [primary] — [git-pack-objects](https://git-scm.com/docs/git-pack-objects)
- Measurement in a scratch repository (git 2.50.1, 54 files, 780 KB working tree, taken from this project's `docs/content`): [observed 2026-10-09]

  | State | `.git/objects` | Working tree |
  |---|---|---|
  | One copy, packed | 316 KB | 780 KB |
  | Plus one identical copy in `versioned/v1` | 320 KB (3 new objects, all trees) | 1,560 KB |
  | Plus nine more copies, every text file changed by one appended line; before packing | 3,500 KB | 8,580 KB |
  | Same, after default `git gc` | 408 KB | 8,580 KB |
  | Same, after `git repack -a -d -f --window=250` | 364 KB | 8,580 KB |

  Eleven copies cost 1.29 times the packed size of one (1.15 times with a wider window) and 11 times the working tree; before packing, the edited copies cost full size. Source: this research, no URL.

#### Where the cost is reported
- Spectro Cloud's reasons for not using `versioned_docs` copies (435+ pages, post of 10 Apr 2024): a large codebase; slower git interactions "especially fresh clones", which slows CI; and several version folders making it "more likely" that authors edit the wrong version. No sizes are given. Versioned content also slowed local development enough that they configured the dev server to run without it. [primary] — [Spectro Cloud blog](https://www.spectrocloud.com/blog/when-docs-and-a-dinosaur-git-along-enabling-versioning-in-docusaurus)
- Docusaurus's own guidance: versioning "will just increase your build time"; "try to keep the number of your versions below 10"; old versions become "a lot of obsolete versioned documentation that nobody even reads anymore"; do not cut a version per patch release because it only duplicates files; for dev and deploy previews "limit to 2 or 3 versions ... to improve startup and build time" with `onlyIncludeVersions`. [primary] — [Docusaurus: Versioning](https://docusaurus.io/docs/versioning)
- Build memory follows versioning, not just page count: a site with two versions of about 360 MDX files each plus `next` took about 26 minutes and "upwards of 10gb" of RAM; removing one version changed memory little; removing versioning entirely dropped the peak to about 5 GB (27 Jul 2020). [primary] — [Docusaurus discussion #3132](https://github.com/facebook/docusaurus/discussions/3132)
- Per-file git lookups scale with file count: on an 11,000-document site with about 3,000 commits, loading the site took 7 seconds with last-update metadata disabled and 25 seconds with it enabled (12 May 2025). [primary] — [Docusaurus discussion #3132](https://github.com/facebook/docusaurus/discussions/3132)
- Binary assets are what makes a docs repository large: one user's repository exceeded 5 GB and pruning old image history gave roughly a 3 times build speed-up (9 May 2025); another site had 544 images totalling 130 MB in `static/img` (28 Jul 2020). [primary] — [Docusaurus discussion #3132](https://github.com/facebook/docusaurus/discussions/3132)
- Consensys' docs template notes that every version in history is kept in `versioned_docs` and removing old ones is a manual task. [snippet] — [Consensys docs template](https://docs-template.consensys.io/configure/versioning)
- GitHub's thresholds make the binary case concrete: warning at 50 MiB a file, block at 100 MiB, repositories recommended under 1 GB. [primary] — [GitHub: About large files](https://docs.github.com/en/repositories/working-with-files/managing-large-files/about-large-files-on-github)

### Inferences
- "A copy per version bloats the repository" is, for text, wrong about `.git` and right about the checkout. History and clone transfer grow by a small fraction per copied version; the working tree, and everything that reads it, grows by a whole version each time.
- Three conditions make copies cost real history: (1) binary assets copied and then changed, which do not delta well and are skipped above `core.bigFileThreshold`; (2) many files with the same name in sibling directories (`index.md`, `README.md` per version), which is exactly the name-hash collision case the git documentation warns about, mitigated by a wider window, name-hash v2, or `--path-walk`; (3) unpacked local repositories between `gc` runs, where every edited copy is a full blob.
- The costs projects actually report are, in order of how often they appear: build time and memory; local development speed; clone and CI time (asserted, not measured); wrong-version edits. No project in the sources read reported a measured `.git` size attributable to text copies.
- Review noise is the unmeasured cost: cutting a version adds a diff as large as the docset, and a fix applied to N versions is N file changes. No source quantified this; Spectro Cloud's "edit the wrong version" is the only statement of it.
- A design that stores no per-version copy in git therefore mainly saves working-tree size, tool walk time, and authoring mistakes, not repository bytes. The argument for it should be made on those grounds.
- The honest comparison with branch-per-version: git storage is equivalent (same blobs, same deltas); the branch model keeps the working tree at one version and moves the cost to backports and multi-branch builds.

### Gaps
- No project published a before-and-after `.git` size for adding or removing `versioned_docs` copies.
- No measurement of review or pull-request noise from copied versions was found.
- The scratch measurement used 54 files; it shows the mechanism, not behaviour at 10,000 files, where name-hash collisions across version directories could reduce delta quality. Not tested.
- Git LFS behaviour for versioned image assets (whether LFS storage deduplicates identical files across version folders, and its quotas) was not researched.

## What numbers exist for build time or repository growth per added version?

### Takeaway
Few numbers isolate the marginal cost of one version. The firm ones are for whole sites (minutes to hours, 5 to 14 GB of memory at thousands of pages in Docusaurus) and for built output committed to a branch (50 to 100 MB a deploy; a `gh-pages` branch that is 95 to 99.6 percent of the repository).

### Cited Findings

#### Build time and memory (source copies)
- Two versions of about 360 MDX files plus `next`: about 26 minutes, more than 10 GB RAM, `--max_old_space_size=16000` needed; without versioning, peak about 5 GB (krillboi, 27 Jul 2020). [primary] — [Docusaurus discussion #3132](https://github.com/facebook/docusaurus/discussions/3132)
- Puppeteer's versioned docs ran out of memory on GitHub Actions or took "many hours" (OrKoN, 3 Aug 2022; no figures). [primary] — [Docusaurus discussion #3132](https://github.com/facebook/docusaurus/discussions/3132)
- About 3,100 Markdown files: production build "20+ minutes" and often out of memory; 22 minutes cut to about 8 by switching to esbuild (madelson, Sept 2022). Another user: about 60 minutes on average (27 Sep 2022). A user planning 10,000+ files estimated "60+ GB Ram" (24 Dec 2024). [primary] — [Docusaurus discussion #3132](https://github.com/facebook/docusaurus/discussions/3132)
- About 11,000 Markdown files with an OpenAPI plugin: about 1 h 15 min on a 16-core, 64 GB Linux machine, and out of memory on local machines (1 May 2025). The maintainer's benchmark of that site on Docusaurus 3.0: 2,417 seconds total, peak about 13.8 GB, of which bundling 12 minutes and static generation 27 minutes; on a later release, 480 seconds cold and 350 seconds with a warm bundler cache (14 May 2025). [primary] — [Docusaurus discussion #11140](https://github.com/facebook/docusaurus/discussions/11140)
- A regression shows how fragile large builds are: after upgrading from 3.0.1 to 3.1, a two-locale site of about 2,300 Markdown files went from under 30 minutes to 3 h 30 min; another went from 8 to 9 minutes to 17 to 20; another from about 45 minutes to over 2 hours. Memory was about 7 GB, previously up to 14 GB, at which the reporter's CI machines crashed. Fixed in PR #9778 (24 Jan 2024), after which the first site built in about 11 minutes. [primary] — [Docusaurus issue #9754](https://github.com/facebook/docusaurus/issues/9754)
- Docusaurus Faster (Rust-based tooling, 4 Nov 2024): React Native's site built 3.04 times faster, Babel's 3.27 times, Lexical's 2 times; no memory figures. [primary] — [Docusaurus 3.6 release post](https://docusaurus.io/blog/releases/3.6)
- Docusaurus 3.8 reports roughly 2 to 5 times faster rebuilds from a persistent bundler cache. [snippet] — [Docusaurus 3.8 release post](https://docusaurus.io/blog/releases/3.8)
- A site with 131 versions and about 50,000 files reported a 5 MB route registry and high build time and memory (2021); a later user said "the versions alone take 50+ min to build." **[snippet; the thread was not opened, and the fetch of #3132 did not contain these figures]** — likely [Docusaurus discussion #6066](https://github.com/facebook/docusaurus/discussions/6066) or [issue #7256](https://github.com/facebook/docusaurus/issues/7256)

#### Repository growth (built output in a branch)
- QuantEcon `lecture-python.myst` (issue opened 16 Oct 2025): repository 4.2 GB, of which `.git` 3.9 GB and working directory about 300 MB. The `gh-pages` branch held 193 deployments since December 2020, 7.56 GB uncompressed and 99.6 percent of the total; other branches about 30 MB. The current `gh-pages` tree alone is 95 MB. "Each deployment adds roughly 50-100 MB." Breakdown of the 7.56 GB: notebooks 1.70 GB, images 1.55 GB, HTML/JS/CSS 1.41 GB, search indices and other 2.90 GB. Proposed fix: make `gh-pages` an orphan branch with one commit, expected to bring the repository to about 100 MB and a fresh clone from 5 to 10 minutes down to about 30 seconds. [primary] — [QuantEcon/lecture-python.myst #647](https://github.com/QuantEcon/lecture-python.myst/issues/647)
- NVIDIA IsaacLab (PR opened 16 Jul 2026, merged 19 Jul 2026): `gh-pages` held "approximately 656 MiB of compressed Git data", about 95 percent of the clone payload; deleting it was expected to take a clone "from approximately 694 MiB to 84 MiB". The nightly job had committed the full multi-version site on every deploy; it now uploads the site with `actions/upload-pages-artifact` and deploys with `actions/deploy-pages`, with URLs and multi-version behaviour unchanged. [primary] — [isaac-sim/IsaacLab PR #6571](https://github.com/isaac-sim/IsaacLab/pull/6571)
- mike commits to `gh-pages` on every deploy; its README does not discuss branch growth or squashing history. [primary] — [mike README](https://github.com/jimporter/mike)
- A published workaround prunes `gh-pages` by squashing its history on a schedule in GitHub Actions, prompted by non-reproducible binary files making the repository grow continuously. [snippet] — [elis.nu, July 2024](https://elis.nu/blog/2024/07/prune-gh-pages-branches/)

#### Search index
- In QuantEcon's `gh-pages` history, "search indices and other" was the largest category at 2.90 GB of 7.56 GB. [primary] — [QuantEcon/lecture-python.myst #647](https://github.com/QuantEcon/lecture-python.myst/issues/647)
- One project measured a static in-browser search index at 1.3 MiB gzipped, downloaded whole on first search and rebuilt on every deploy. [snippet] — [sister-software/mailwoman #2434](https://github.com/sister-software/mailwoman/issues/2434)

### Inferences
- From QuantEcon's figures, built output costs about 39 MB of uncompressed history per deploy on average (7.56 GB over 193) against a 95 MB live tree, and about 20 MB of packed `.git` per deploy (3.9 GB over 193). Built output regenerated on each deploy compresses far worse than source: hashes in file names, minified bundles, search indexes, and notebooks change wholesale.
- The contrast with the source-copy measurement is the main quantitative point available: ten near-duplicate source copies added 29 percent to a packed repository, while 193 near-duplicate built copies multiplied one by about 40.
- Build time is not linear in versions in any clean way in the published data; the one controlled observation (krillboi) found memory roughly doubling between "no versioning" and "two versions plus next", and barely moving when one version was removed. This suggests a fixed overhead from the versioning machinery plus a per-page cost, but one data point from 2020 cannot carry a model.
- Bundler-era regressions (a 7-fold slowdown from one minor upgrade) are a cost of rebuilding old versions with new tools that never appears in a per-version estimate. A version that is never rebuilt is immune.
- Search indexes are a large and overlooked share of stored output. Per-version indexes that are built once and merged at query time (as Pagefind supports) avoid both rebuilding and re-storing them.

### Gaps
- No source gives build minutes per added version with page count held constant, for any generator.
- No numbers were found for Sphinx, MkDocs, Antora, or Hugo multi-version builds, or for Read the Docs build minutes consumed per version.
- No CI cost in currency was found.
- No clone-time measurement for a `versioned_docs` repository was found; Spectro Cloud asserts slower clones without figures.
- The 131-version, 50,000-file data point is unverified.

## Which cost-reduction techniques are proven in practice?

### Takeaway
The techniques with named users and stated results are: stop rebuilding old versions (archive them as separate immutable deployments), keep built output out of git history (deploy from artifacts, or squash the output branch), and limit which versions a given build includes. Git-side techniques (partial clone, sparse checkout) are documented and sound but no docs project was found reporting results from them.

### Cited Findings

#### Do not rebuild old versions
- Docusaurus and Jest link archived versions to immutable host deploy URLs "that will never be rebuilt", specifically "to keep the number of actively built versions low." [primary] — [Docusaurus: Versioning](https://docusaurus.io/docs/versioning)
- Spectro Cloud builds three versions and links the rest to an archive domain. [primary] — [Spectro Cloud blog](https://www.spectrocloud.com/blog/when-docs-and-a-dinosaur-git-along-enabling-versioning-in-docusaurus)
- GitLab deploys archives from a separate project about monthly instead of with every docs change. [primary] — [GitLab epic 8937](https://gitlab.com/groups/gitlab-org/-/epics/8937)
- mike deploys one version at a time and leaves the others' files untouched. [primary] — [mike README](https://github.com/jimporter/mike)

#### Limit what a build includes
- `onlyIncludeVersions` restricts a Docusaurus build to a subset; recommended at 2 or 3 versions for development and previews. [primary] — [Docusaurus: Versioning](https://docusaurus.io/docs/versioning)
- Spectro Cloud's dev server runs with no generated versions at all. [primary] — [Spectro Cloud blog](https://www.spectrocloud.com/blog/when-docs-and-a-dinosaur-git-along-enabling-versioning-in-docusaurus)
- Read the Docs builds each version separately and lets a version be deactivated, which deletes its built content and stops builds. [primary] — [Read the Docs: Versions](https://docs.readthedocs.com/platform/stable/versions.html)

#### Keep built output out of git history
- IsaacLab replaced commits to `gh-pages` with a Pages artifact upload, removing about 95 percent of clone payload (694 MiB to 84 MiB expected) with no change to the multi-version site. [primary] — [isaac-sim/IsaacLab PR #6571](https://github.com/isaac-sim/IsaacLab/pull/6571)
- QuantEcon planned to reset `gh-pages` to a single orphan commit (4.2 GB to about 100 MB expected). [primary] — [QuantEcon/lecture-python.myst #647](https://github.com/QuantEcon/lecture-python.myst/issues/647)
- Scheduled squashing of `gh-pages` in CI. [snippet] — [elis.nu](https://elis.nu/blog/2024/07/prune-gh-pages-branches/)
- Release assets carry no total size or bandwidth limit on GitHub (2 GiB a file, 1,000 files a release). [primary] — [GitHub: About releases](https://docs.github.com/en/repositories/releasing-projects-on-github/about-releases)

#### Faster and cached builds
- Rust-based bundling gave 2 to 3.3 times faster production builds on three named sites; a warm bundler cache took one 11,000-page build from 480 to 350 seconds, with static generation (270 seconds) unaffected by the cache. [primary] — [Docusaurus 3.6](https://docusaurus.io/blog/releases/3.6), [Docusaurus discussion #11140](https://github.com/facebook/docusaurus/discussions/11140)
- Results are not uniform: one team reported slower builds after moving to 3.9 with the faster option, and another found the Node version the largest factor. [snippet] — [Docusaurus discussion #11664](https://github.com/facebook/docusaurus/discussions/11664)
- Turning off per-file last-update lookups cut site load from 25 to 7 seconds on an 11,000-document site. [primary] — [Docusaurus discussion #3132](https://github.com/facebook/docusaurus/discussions/3132)
- The Linux kernel halved PDF build time (about 11 to about 5 minutes) by building PDFs in parallel. [snippet] — [LKML, Sept 2025](https://lkml.iu.edu/hypermail/linux/kernel/2509.0/01042.html)

#### Git-side techniques
- Partial clone lets git "function without having a complete copy of the repository", avoiding downloads "in advance during clone and fetch operations"; missing objects are fetched on demand from a promisor remote. Stated costs: "Dynamic object fetching tends to be slow as objects are fetched one at a time", and the user must be online with the remote available. Checkout pre-fetches the blobs it needs in one batch. [primary] — [Git: partial clone](https://git-scm.com/docs/partial-clone)
- GitHub's guidance (Derrick Stolee, 21 Dec 2020, updated 28 Apr 2021): blobless clones (`--filter=blob:none`) suit large repositories with many large historical files; treeless clones (`--filter=tree:0`) and shallow clones (`--depth=1`) suit throwaway CI builds and are discouraged for daily work; "Never fetch from a shallow clone!" The post contains no measurements. [primary] — [GitHub blog](https://github.blog/open-source/git/get-up-to-speed-with-partial-clone-and-shallow-clone/)
- Sparse checkout changes "the working tree from having all tracked files present to only having a subset of those files"; cone mode, the default, selects whole directories; the command is still marked experimental, and commands such as `git grep` return results limited by it. [primary] — [git-sparse-checkout](https://git-scm.com/docs/git-sparse-checkout)
- Packing can be tuned for same-named files in many directories: a wider `--window`, name-hash version 2, or `--path-walk`. [primary] — [git-pack-objects](https://git-scm.com/docs/git-pack-objects)

#### Branch per version: what it moves the cost to
- Spectro Cloud's branch-per-minor model needs a script that fetches every version branch, checks each out, runs the versioning command, stages the output, and merges the per-branch `versions.json` files before one final build; backports use a label-driven GitHub Action that opens a backport pull request per target branch, and "For merge conflicts, the commit is cherry-picked into the version branch and the conflict is resolved manually." [primary] — [Spectro Cloud blog](https://www.spectrocloud.com/blog/when-docs-and-a-dinosaur-git-along-enabling-versioning-in-docusaurus)
- Kubernetes' contributor guide warns that generated files differ between release branches and the main branch, so a cherry-picked change can apply cleanly yet produce different generated output. [snippet] — [Kubernetes: Contributing to the upstream code](https://v1-35.docs.kubernetes.io/docs/contribute/generate-ref-docs/contribute-upstream/)
- Kyverno's website keeps docs per release branch, with all changes going to the main branch first and then to older branches. [snippet] — [Kyverno website repository (fork)](https://github.com/posquit0/website)

### Inferences
- Ranked by evidence: (1) freeze and stop rebuilding old versions: used by Docusaurus, Jest, Spectro Cloud, GitLab, Kubernetes, and every mike user; (2) keep built output out of git history: two projects with measured 95 to 99 percent reductions; (3) restrict versions per build: documented and used; (4) faster bundlers and caches: measured 2 to 3 times, with exceptions; (5) partial clone, sparse checkout, shallow CI clones: sound per git's documentation, unreported by docs projects.
- Partial clone and sparse checkout address the two costs the scratch measurement separated: partial clone the history (already small for text), sparse checkout the working tree (the part that actually grows). Sparse checkout is the more relevant of the two for a copy-per-version layout, but it is experimental and changes what `git grep` sees, which is a poor fit for authors.
- The same-content-different-version problem has two published answers and nothing in between: copies in the tree (Docusaurus) or branches (Spectro Cloud, Kubernetes, Kyverno, mike). Both keep a full source per version. Storing only the built snapshot of a published version, outside git, with the source recoverable from a tag, is consistent with what every project does once a version is unmaintained: nobody edits it, and nobody rebuilds it.
- Content-addressed storage of built output is the technique the git measurements point to but no docs project was found describing: QuantEcon's history shows near-identical trees stored 193 times. A store keyed by file hash would hold unchanged pages and assets once across versions, the way git already does for source.
- A git tag plus a build artifact stored outside the repository gives the archive two independent recoveries: the artifact for reading, the tag for the source. The reproducibility evidence says to rely on the first and treat the second as best effort.

### Gaps
- No documentation project was found reporting results from partial clone, sparse checkout, or Git LFS.
- No project was found describing content-addressed or deduplicated storage of built docs output across versions.
- No source was found on single-source filtering (one source, versions produced by conditions) with measured build or repository cost; other researchers cover the technique itself.
- No measurements of backport effort (pull requests per fix, conflict rate) on a branch-per-version docs repository were found.
- Incremental builds that rebuild only changed versions were not found as a named feature of any generator checked; what exists is building versions separately (Read the Docs, mike) or excluding them (`onlyIncludeVersions`).
- GitHub's per-file release limit appears as "under 2 GiB" on the releases page, while the large-files page says the limit is that of the plan's Git LFS file size; the two were not reconciled.
