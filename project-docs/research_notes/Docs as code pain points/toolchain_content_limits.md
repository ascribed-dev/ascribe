# Toolchain, markup and content-management limits of docs-as-code at scale

Notes compiled 3 October 2026. Scope excludes review/collaboration with non-technical contributors and docs-to-code drift.

Evidence labels used below:
- **[first-hand]** a team or maintainer describing their own site or project
- **[practitioner]** an individual writer's opinion or analysis
- **[vendor]** a company that sells a competing or adjacent product; what it sells is stated
- **[snippet]** taken from a search-result summary; the page itself was not opened, so treat wording and numbers as unverified
- Material from before 2023 carries its year.

## Ranking of limits by frequency of report and severity of consequence

### Takeaway
The limits that are both most often reported and most damaging are platform ones: generator abandonment or breaking rewrites (MkDocs, 2024 to 2026) and build time and memory growth on large or versioned sites (Docusaurus). Markup fragility (MDX) and weak reuse are reported just as often but tend to cost time rather than stop publication. This ranking is my judgement from the sources gathered; no source ranks them and no survey with counts was found.

### Cited Findings
Ranked list, with the evidence each rank rests on (details and links in the sections that follow):

1. **Generator or dependency abandonment and breaking major versions.** Severity: forces unplanned migration of the whole site. Evidence: MkDocs has had no release since 1.6.1 on 30 August 2024; 90,000+ GitHub projects depend on it; MkDocs 2.0 drops the plugin system (about 300 plugins); Material for MkDocs entered maintenance mode in November 2025 — [Florian Maas, 22 Mar 2026](https://fpgmaas.com/blog/collapse-of-mkdocs/); [Material for MkDocs blog, 18 Feb 2026](https://squidfunk.github.io/mkdocs-material/blog/2026/02/18/mkdocs-2.0/)
2. **Build time and memory growth, multiplied by versions and locales.** Severity: builds fail outright (out of memory) or take over an hour. Evidence: about 11,000 Markdown files taking about 1 h 15 min on a 16-core, 64 GB Linux machine and crashing on laptops — [Docusaurus discussion #11140, 1 May 2025](https://github.com/facebook/docusaurus/discussions/11140); 26-minute builds needing 16 GB of Node heap for about 360 MDX files times three versions (2020) — [Docusaurus discussion #3132](https://github.com/facebook/docusaurus/discussions/3132)
3. **MDX fragility.** Severity: one stray `{` or `<` fails the build for the whole site; upgrades between MDX major versions break legacy content in bulk. Evidence: Docusaurus v3 migration reports — [Docusaurus issue #9538](https://github.com/facebook/docusaurus/issues/9538); [MDX troubleshooting](https://mdxjs.com/docs/troubleshooting-mdx/)
4. **Content reuse, variables and conditional content with no standard mechanism.** Severity: copy-paste that diverges; each generator's include syntax is non-portable. Evidence: SIGDOC 2024 industry paper on a DITA-to-Markdown migration — [Berger, SIGDOC '24](https://dl.acm.org/doi/fullHtml/10.1145/3641237.3691677); vendor claims — [Author-it](https://www.author-it.com/resources/docs-as-code-vs-ccms)
5. **Migration cost between generators.** Severity: weeks of engineering time per migration, recurring every few years. Evidence: Cloudflare moved Gatsby to Hugo in 2021 (4,850 files, about three weeks) and Hugo to Astro in 2024-25 (8,060 files, six weeks) — [Cloudflare blog, 8 Jan 2025](https://blog.cloudflare.com/open-source-all-the-way-down-upgrading-our-developer-documentation/)
6. **Versioning (folder copies or release branches).** Severity: feeds rank 2 directly, plus backport labour. Evidence: Docusaurus maintainers' own advice to archive old versions — [snippet, Docusaurus discussions](https://github.com/facebook/docusaurus/discussions/10523)
7. **Localization tooling that does not understand the markup.** Severity: lost translations and failed builds. Evidence: Crowdin's MDX handling — [Docusaurus discussion #8821, 24 Mar 2023](https://github.com/facebook/docusaurus/discussions/8821)
8. **Quality automation overhead** (prose linting false positives, link checking, redirects, image bloat). Severity: friction and noise rather than outage. Evidence: [GitLab Vale docs](https://docs.gitlab.com/development/documentation/testing/vale)
9. **Outputs beyond the website** (PDF, LLM-facing endpoints). Severity: unmet requirement rather than breakage; mostly solved with third-party scrapers. Evidence thin and mostly vendor-sourced.

### Inferences
- Ranks 1, 2, 5 and 6 compound: versioning inflates build size, slow builds motivate migration, and migration is what abandonment forces.
- Ranks 3 and 7 share a root cause: MDX is a programming-language superset of Markdown, so third-party tools (translation platforms, linters) parse it as something else.

### Gaps
- No quantitative survey ranking these pain points was found. The ordering reflects how many independent first-hand reports I located, which is biased toward projects with public GitHub trackers (Docusaurus, MkDocs).
- Hugo, Sphinx, Antora and Starlight issue trackers were not searched in depth, so their absence from the top ranks is not evidence they scale better.

## Markup: flavour fragmentation, MDX fragility, AsciiDoc/rST trade-offs, portability, and Markdoc/MyST/Djot

### Takeaway
MDX is the most frequently reported markup failure point: it turns prose characters into syntax and breaks legacy content on each major version. The alternatives intended to fix this (Markdoc, MyST, Djot) exist but none has become a cross-tool standard; Markdown itself remains unstandardised.

### Cited Findings
- MDX troubleshooting guidance states that a brace is parsed as a JavaScript expression and must be escaped as `\{` if a literal is wanted — [MDX docs](https://mdxjs.com/docs/troubleshooting-mdx/) [snippet]
- A Docusaurus v3 issue is titled "MDX v3 LaTeX: Escaping Curly Braces", showing that maths content collides with MDX expression syntax — [Docusaurus issue #9538](https://github.com/facebook/docusaurus/issues/9538) [snippet]
- A downstream project reported its "Docusaurus build server crashing due to strict MDX compilation errors in documentation files", caused by unclosed tags, raw `<` or `>` read as JSX, and unescaped braces; raw tags such as `<excludes>` or `<BigInteger>` in prose had to be wrapped in inline code — [carlos-emr/carlos issue #2734](https://github.com/carlos-emr/carlos/issues/2734) [first-hand, snippet]
- Generated API reference content also broke: "MDX compilation fails with Docusaurus v3" in the OpenAPI docs plugin — [PaloAltoNetworks/docusaurus-openapi-docs issue #591](https://github.com/PaloAltoNetworks/docusaurus-openapi-docs/issues/591) [snippet]
- Mitigations offered by Docusaurus: `markdown.format: 'detect'` to parse `.md` as CommonMark and reserve MDX for `.mdx`, and `npx docusaurus-mdx-checker` to find non-compliant files before upgrading — [Docusaurus issue #9538 and related](https://github.com/facebook/docusaurus/issues/9538) [snippet]
- Stripe's Markdoc FAQ frames the split as "docs as code" (MDX, arbitrary JavaScript) versus "docs as data" (Markdoc, strict separation of code and content), and says Markdoc was built so contributors could edit without edits needing the code-review scrutiny that embedded JavaScript requires — [Markdoc FAQ](https://markdoc.dev/docs/faq) [first-hand from Stripe, snippet; 2022]
- Hacker News reaction (2022) included "I don't understand how this is fundamentally different than MDX" — [HN item 31341348](https://news.ycombinator.com/item?id=31341348) [snippet; 2022]
- Fabrizio Ferri-Benedetti's review (11 May 2022) says Markdoc lets writers "focus on content, without having to worry that much about JSX oddities", but that it is a framework needing development effort to integrate, not a turnkey site, and that Stripe's quality comes from giving "enough technical resources to your technical writers" — [passo.uno, 2022](https://passo.uno/markdoc-review/) [practitioner]
- The same author described a "Tower of Babel state of affairs" of Markdown, AsciiDoc and reStructuredText mixed within single repositories, joined by "glorified pandoc piping acting as duct tape", and listed the lack of a standard lightweight markup for technical documentation as the first of three missing standards — [passo.uno, 1 Nov 2021](https://passo.uno/docs-as-code-tools-open-standards/) [practitioner; 2021]
- Djot (by the author of Pandoc and CommonMark) had not reached a 1.0 spec per the 2026 search summary — [jgm/djot](https://github.com/jgm/djot) [snippet]
- MyST remains tied to the Python ecosystem (markdown-it-py parser plus Sphinx extension); an open mystmd issue asks to make the spec tests "less unique to markdown-it, and easier to reuse outside of mystmd's parser" — [MyST-Parser](https://github.com/executablebooks/MyST-Parser); [mystmd issue #3078](https://github.com/jupyter-book/mystmd/issues/3078) [snippet]
- CommonMark itself has not reached 1.0, and a 2026 commentary says dialects and extensions are growing, not consolidating — [dev.to, "Why Are We Still Using Markdown in 2026?"](https://dev.to/castillodk/why-are-we-still-using-markdown-in-2026-15db) [practitioner, snippet]

### Inferences
- Component lock-in follows from MDX's design: content that imports a site's React components cannot be rendered by another generator without rewriting those components, which is part of why Cloudflare's migration spent 14 of its roughly 37 working days on components (see Build and platform).
- Markdoc, MyST and Djot each solved the problem for one ecosystem (Stripe/React, Sphinx/Jupyter, Pandoc) and none is supported across Docusaurus, MkDocs, Hugo and Starlight together, so choosing one is itself a lock-in decision.

### Gaps
- No first-hand 2023-2026 account of a team adopting Markdoc or Djot at scale for docs was found; adoption levels are unconfirmed.
- AsciiDoc and reStructuredText adoption trade-offs were not researched from primary sources in this pass (no Antora or Sphinx tracker material was opened). The SIGDOC 2024 paper covers structured authoring languages, but its full text returned HTTP 403, so only the abstract is used.
- Whether MDX compile errors remain as common under MDX 3 with `format: 'detect'` as under the v2-to-v3 transition is not established.

## Content reuse and structure: snippets, includes, variables, conditions, single-sourcing versus DITA or a CCMS

### Takeaway
Reuse in docs-as-code is done through generator-specific includes, shortcodes or components rather than a markup standard, so it is neither portable nor tracked. The best neutral source is one SIGDOC 2024 industry paper; most other claims about where this breaks come from CCMS vendors.

### Cited Findings
- "Implementing Structured Authoring Practices in a Docs-as-Code Framework" by Arthur Berger (Solo.io), SIGDOC '24, October 2024, is an industry insight paper based on "an enterprise migration from DITA to Markdown as well as a startup technical writing team's selection of content development tools". It covers version control, structured authoring languages, static site generators, deployment options and content maintenance, and frames its contribution as showing "the possibilities and constraints of taking a docs-as-code approach to structured authoring" — [ACM Digital Library](https://dl.acm.org/doi/fullHtml/10.1145/3641237.3691677) [first-hand; abstract only, via snippet]
- passo.uno (2021) lists content reuse, references and tables as Markdown's specific weaknesses and points to Lightweight DITA and MyST as candidate answers — [passo.uno, 1 Nov 2021](https://passo.uno/docs-as-code-tools-open-standards/) [practitioner; 2021]
- Markdoc's feature set (variables, conditional rendering, custom tags) is described as mirroring what MDX and AsciiDoc already offer, in Markdown-like syntax — [passo.uno, 2022](https://passo.uno/markdoc-review/) [practitioner; 2022]
- In Backstage TechDocs, cross-service aggregation relies on a plugin-specific `!include` syntax from mkdocs-monorepo-plugin — [Roadie TechDocs docs](https://roadie.io/docs/catalog/techdocs/troubleshooting/) [vendor: Roadie sells hosted Backstage; snippet]
- **[vendor]** Author-it (sells a CCMS) claims that in docs-as-code "reuse across products becomes copy-paste that drifts", translation "has no workflow that tracks changes so you re-translate whole files", and commit history "only approximates" a compliance audit trail; it recommends keeping docs-as-code for developer reference and moving product documentation to a CCMS — [Author-it](https://www.author-it.com/resources/docs-as-code-vs-ccms) [snippet]
- **[vendor]** ClickHelp (sells a cloud documentation platform) lists the limits it sees: table editing is cumbersome without visual editing, "extensive cross-linking" and multilingual content need "custom configuration and plugins", "large portals may still require several minutes for full deployments", analytics need external platforms, and role-based access needs "additional infrastructure such as edge functions". It names no customers and gives no numbers — [ClickHelp, 5 Mar 2026, updated 28 Jul 2026](https://clickhelp.com/clickhelp-technical-writing-blog/when-docs-as-code-reaches-its-limits-and-what-teams-do-next/)

### Inferences
- The vendor claims are directionally consistent with the practitioner sources but are unquantified and self-interested; the specific claim that whole files must be re-translated is contradicted in part by Crowdin's string-level deduplication (see Localization), which exists but is fragile.
- Because includes are implemented per generator (Hugo shortcodes, MkDocs plugins, MDX imports, AsciiDoc `include::`), reuse is the part of a corpus most likely to need manual rewriting in a migration.

### Gaps
- The body of the SIGDOC paper (specific workarounds, tool comparisons, any numbers) could not be read; a writer with ACM access should check it before quoting beyond the abstract.
- No first-hand account was found of a team measuring duplicated content or reuse-related errors in a Markdown corpus.
- No first-hand account of a team moving back from docs-as-code to DITA or a CCMS was found; the "teams move structured docs to a CCMS" pattern is vendor-asserted only.

## Versioning and multi-product or multi-repo aggregation

### Takeaway
Folder-copy versioning (Docusaurus) multiplies build cost by versions times locales, and maintainers themselves recommend archiving old versions; branch-per-release versioning (mike, sphinx-multiversion) moves the cost to backport labour. Aggregators such as Backstage TechDocs inherit the limits and the maintenance risk of the generator underneath.

### Cited Findings
- Docusaurus maintainers' guidance: each new version increases build time; docs times versions times locales drives large builds; old versions should be archived to standalone deployments, and very large sites can be split into multiple single-instance sites linked together — [Docusaurus discussion #10523](https://github.com/facebook/docusaurus/discussions/10523) [first-hand maintainer, snippet]
- TouchGFX docs (2020): two versions plus "next", about 360 MDX files each; 26-minute build; "we have to use the `--max_old_space_size=16000` option on the node command to get it to finish, else it runs out of memory"; with versioning removed, peak memory fell to about 5 GB — [Docusaurus discussion #3132, reporter krillboi](https://github.com/facebook/docusaurus/discussions/3132) [first-hand; 2020]
- Puppeteer docs (2022): versioned docs caused out-of-memory failures in GitHub Actions — [Docusaurus discussion #3132, reporter OrKoN](https://github.com/facebook/docusaurus/discussions/3132) [first-hand; 2022]
- mike for MkDocs manages versions by automating a `gh-pages` branch; a 2026 comparison says the workflow "takes some learning and isn't as deeply integrated" as Sphinx with Read the Docs, which builds each Git tag as a version — [docsio.co, Sphinx vs MkDocs](https://docsio.co/blog/sphinx-vs-mkdocs) [vendor: Docsio sells a docs product; snippet]
- Projects using release branches write an explicit backport rule: doc fixes that matter to users of a released version need a second pull request into the release branch — [dhilipkumars/axiom issue #114](https://github.com/dhilipkumars/axiom/issues/114) [first-hand, snippet]
- ScyllaDB's Sphinx theme docs advise building from release branches, not tags, so documentation changes can be backported without moving tag references — [ScyllaDB Sphinx theme](https://sphinx-theme.scylladb.com/stable/configuration/multiversion.html) [first-hand, snippet]
- Backstage TechDocs: a user raised "TechDocs Evolution: Addressing the Sustainability of the MkDocs Ecosystem" on 12 February 2026, writing that "maintenance on the core mkdocs repository appears to have slowed significantly" and asking for engine-agnostic TechDocs or first-class Zensical support. The issue is closed; no maintainer decision was visible on the page — [backstage/backstage issue #32815](https://github.com/backstage/backstage/issues/32815) [first-hand user]
- Roadie reported that the TechDocs backend consumed "a disproportionate amount of CPU and memory" for tenants with large sites, repositories holding several doc sets, and frequent rebuilds, and split TechDocs out of its monolithic Backstage deployment — [Roadie blog](https://roadie.io/blog/splitting-techdocs-out-of-our-monolithic-backstage-deployment/) [first-hand operational account from a vendor that sells hosted Backstage; snippet, date not confirmed]
- TechDocs needs Docker where docs are generated, which may not be viable inside a Kubernetes pod, and the first request for a doc set is slow because it is built on demand — [Backstage TechDocs troubleshooting](https://backstage.io/docs/features/techdocs/troubleshooting/) [snippet]

### Inferences
- The two versioning models trade one cost for another: copies in the tree make every build pay for every version, while branches keep builds small but make every cross-version fix a cherry-pick.
- TechDocs shows a second-order risk: an organisation that standardised internal docs on Backstage is exposed to the MkDocs situation without having chosen MkDocs directly.

### Gaps
- No first-hand Antora account (multi-repo aggregation, playbook maintenance, build time) was found in this pass.
- No source was found on git submodule pain specific to docs aggregation.
- Kubernetes SIG Docs, GitLab and Microsoft Learn versioning practices were not researched.
- Whether Backstage has decided on a post-MkDocs engine is unconfirmed.

## Localization

### Takeaway
The strongest first-hand evidence is that translation platforms parse MDX as something it is not, which corrupts markup and loses translations. Evidence on string segmentation and translation drift specifically is thin.

### Cited Findings
- Docusaurus maintainer Sébastien Lorber (24 March 2023) reported that Crowdin's new MDX support targeted MDX v1 although v2 had been out since January 2022; newly uploaded `.mdx` files were parsed differently from older ones, which defeated Crowdin's duplicate-string deduplication, so the Docusaurus 2.4.0 docs lost translations that 2.3.1 had. An upload containing empty Markdown table cells returned "Error from server: <Code: 500, Message: Internal Server Error>". The workaround was forcing the Markdown parser for `.mdx` files with `type: 'md'` in the Crowdin config — [Docusaurus discussion #8821](https://github.com/facebook/docusaurus/discussions/8821) [first-hand]
- Docusaurus's own Crowdin guide warns that Crowdin treats JSX as embedded HTML and can alter it on download, producing a site that fails to build; simple string props survive, object or array props are likely to fail — [Docusaurus i18n Crowdin docs](https://docusaurus.io/docs/i18n/crowdin) [first-hand, snippet]
- The same guidance notes Crowdin updates its MDX parser regularly, causing subtle breakage, and recommends pinning the parser version — [Docusaurus i18n Crowdin docs](https://docusaurus.io/docs/i18n/crowdin) [snippet]
- Configuration drift between translation tooling and docs: a project issue notes its localization guidance "describes Crowdin mappings that crowdin.yml does not have" — [z-shell/wiki issue #929](https://github.com/z-shell/wiki/issues/929) [first-hand, snippet]
- Locales multiply build cost alongside versions (see Versioning) — [Docusaurus discussion #10523](https://github.com/facebook/docusaurus/discussions/10523) [snippet]
- **[vendor]** Author-it claims docs-as-code translation forces re-translating whole files because no workflow tracks changes — [Author-it](https://www.author-it.com/resources/docs-as-code-vs-ccms) [snippet]

### Inferences
- Translated MDX is a build-breaking input written by people (translators) who cannot run the build, which makes MDX fragility worse in localized sites than in the source language.

### Gaps
- No first-hand sources were found on Transifex or Lokalise with Markdown docs, on segmentation quality, or on measured translation lag behind the source language.
- The Crowdin MDX account is from 2023; whether Crowdin's MDX parsing has since caught up with MDX 3 is unconfirmed.

## Quality automation: links, linting, code samples, anchors, redirects, stale and bloating images

### Takeaway
Teams do automate quality checks, but each check carries a maintenance cost of its own: Vale needs curated exception lists and rule governance, and images inflate Git history unless moved to LFS, which then breaks some hosting and CI setups. First-hand evidence here is weaker than for build and platform issues.

### Cited Findings
- GitLab's Vale documentation defines three levels (error, warning, suggestion). Adding an error-level rule requires first fixing every existing occurrence in the docs. For new warnings or suggestions, maintainers must weigh "how many more warnings or suggestions it creates" and "how often an author might ignore it because it's acceptable in the context" — [GitLab docs](https://docs.gitlab.com/development/documentation/testing/vale) [first-hand]
- GitLab also documents that rules with the `raw` scope generally cannot be disabled inline, and suggests "tweaking the formatting around the change" as the workaround for a false positive; valid words flagged by the spelling check go on an exceptions list — [GitLab docs](https://docs.gitlab.com/development/documentation/testing/vale) [first-hand]
- A 2026 vendor guide advises starting Vale with a small rule set (terminology and vocabulary only) "so writers can trust the results" and estimates Vale "starts paying for itself somewhere around 20 pages" — [docsio.co](https://docsio.co/blog/vale-linter) [vendor: Docsio sells a docs product; snippet; the 20-page figure is an unsupported estimate]
- A docs team reports lychee link-checks a roughly 200-page repo in under 30 seconds, fast enough to run on every pull request — [dev.to, EkLine](https://dev.to/ekline/5-github-actions-that-save-our-docs-team-hours-every-week-jkc) [vendor: EkLine sells a docs review tool; snippet]
- Image bloat: Git stores each regenerated screenshot as a full copy; one write-up estimates thirty screenshots updated weekly add 156 MB of history a year, and reports a docs repo going from 412 MB to 28 MB after moving images to Git LFS — [dev.to, omachala](https://dev.to/omachala/your-screenshot-automation-is-bloating-your-git-repo-3lgc) [practitioner promoting a screenshot tool; snippet; figures unverified]
- LFS side effects: without `lfs: true` on checkout, CI receives 130-byte pointer files in place of images — [dev.to, omachala](https://dev.to/omachala/your-screenshot-automation-is-bloating-your-git-repo-3lgc) [snippet]; and LFS-tracked PNGs showed as broken images on GitHub Pages builds — [mkdocs/mkdocs issue #2577](https://github.com/mkdocs/mkdocs/issues/2577) [first-hand, snippet; 2021]
- Open-source projects have filed issues to move heavy docs assets out of the repo or into LFS for repository size — [janhq/jan issue #7666](https://github.com/janhq/jan/issues/7666); [splattner/openzev issue #723](https://github.com/splattner/openzev/issues/723) [first-hand, snippet]

### Inferences
- GitLab's rule that an error-level check cannot be added until all existing violations are fixed shows why lint coverage grows slowly on a large legacy corpus: the cost of a new rule scales with corpus size.
- The quantitative claims in this area come mostly from tool vendors and should not be quoted as neutral measurements.

### Gaps
- No first-hand sources were found on redirect-map management at scale, broken anchors after heading renames, external link-check flakiness (rate limiting, false failures), or automated testing of code samples. These were asked about and remain unanswered.
- No markdownlint-specific false-positive or setup-cost account was found.

## Build and platform: build time, upgrades, MkDocs and Material for MkDocs, migrations, search, hosted lock-in

### Takeaway
As of October 2026 the MkDocs ecosystem is fractured: MkDocs 1.x is unmaintained, MkDocs 2.0 is an incompatible rewrite, Material for MkDocs is in maintenance mode, and users must choose between Zensical, ProperDocs and MaterialX. Separately, Docusaurus sites in the thousands of pages report builds from tens of minutes to over an hour, and large teams such as Cloudflare re-platform roughly every three to four years at a cost of weeks.

### Cited Findings
**MkDocs and Material for MkDocs status (fast-moving; verified against sources dated 2026)**
- Timeline per Florian Maas: maintainer conflict became public on 25 February 2024; the then-maintainer (@oprypin) stepped down on 6 April 2024 and the original author returned; MkDocs 1.6.1 on 30 August 2024 is the last release; Material for MkDocs entered maintenance mode and Zensical was announced in November 2025; on 9 March 2026 @oprypin took control of the PyPI package and backed down within six hours; ProperDocs launched on 15 March 2026 — [Florian Maas, 22 Mar 2026](https://fpgmaas.com/blog/collapse-of-mkdocs/) [practitioner analysis]
- Numbers from the same post: 90,000+ GitHub projects depend on MkDocs; Zensical had about 3,700 stars and ProperDocs 21 stars one week after launch; MkDocs v2 development showed no activity after 19 February 2026 (as of 22 March 2026) — [Florian Maas, 22 Mar 2026](https://fpgmaas.com/blog/collapse-of-mkdocs/)
- Material for MkDocs team (Martin Donath, Alex Voss, Kathi), post dated 18 February 2026 and shown as updated 2 October 2026: MkDocs 1.x has had "no releases in the past 18 months"; "MkDocs 2.0 won't have a plugin system", affecting about 300 plugins; navigation is passed to themes "as pre-rendered HTML rather than structured data"; configuration moves from YAML to TOML with no migration path; "MkDocs 2.0 is incompatible with Material for MkDocs"; the rewrite was initially unlicensed and later MIT — [Material for MkDocs blog](https://squidfunk.github.io/mkdocs-material/blog/2026/02/18/mkdocs-2.0/) [first-hand from the theme's authors, who also build the competing Zensical]
- Material for MkDocs 9.7.0 is the final feature release, made all former sponsor-only (Insiders) features free, and comes with a commitment to critical bug and security fixes for at least 12 months — [Material for MkDocs changelog](https://squidfunk.github.io/mkdocs-material/changelog/) [snippet]
- Zensical is described by its authors as a from-scratch, MIT-licensed generator that reads existing `mkdocs.yml` files and rebuilds 5x faster — [Material for MkDocs blog](https://squidfunk.github.io/mkdocs-material/blog/) [first-hand vendor-style claim, snippet; speed figure not independently verified]
- ProperDocs presents itself as the continuation of MkDocs 1.x by the last active maintainer and publishes a warning about "MkDocs 2.0" — [ProperDocs discussion #33](https://github.com/orgs/ProperDocs/discussions/33) [first-hand, snippet]
- Downstream reaction: migration issues opened in Renovate, DDEV, Kedro, OSM Foundation, TRaSH-Guides, Privacy Guides and others; one is titled "The docs site builds with MkDocs, which loses support on 2026-11-05" and another "Docs are built on MkDocs 1.x + Material, which has no supported path forward" — [renovate discussion #39232](https://github.com/renovatebot/renovate/discussions/39232); [ddev issue #7840](https://github.com/ddev/ddev/issues/7840); [kedro issue #5267](https://github.com/kedro-org/kedro/issues/5267); [portolan-cli issue #892](https://github.com/portolan-sdi/portolan-cli/issues/892); [griptape-nodes-engine issue #5630](https://github.com/griptape-ai/griptape-nodes-engine/issues/5630) [first-hand, titles only via snippet]
- Date conflict to note: one source gives 11 November 2025 for both the 9.7.0 release and the maintenance-mode announcement ([Florian Maas](https://fpgmaas.com/blog/collapse-of-mkdocs/)), while a blog post about the announcement is dated 6 November 2025 ([duerrenberger.dev](https://duerrenberger.dev/blog/2025/11/06/material-for-mkdocs-is-no-more-long-live-zensical/)) and a downstream issue cites 5 November 2026 as end of support ([portolan-cli issue #892](https://github.com/portolan-sdi/portolan-cli/issues/892)). The announcement was most likely about 5 November 2025 with 9.7.0 following on 11 November; the exact day is unconfirmed.

**Build time and memory**
- SailPoint-affiliated reporter @tyler-mairose-sp, 1 May 2025: about 11,000 `.md`/`.mdx` files including OpenAPI-generated pages; "The site does build, on a Linux machine (16 cores & 64GB of RAM) after about an hour and 15 minutes"; production builds crash with out-of-memory on Windows and Mac laptops. Maintainer profiling on an M3 Mac: data loading 5 s, bundling with Rspack 762.87 s, static generation 1,632.71 s. Suggested fixes: disable `concatenateModules` and parallel code splitting (about 3% larger JS), then SSG worker threads and Rspack persistent cache from v3.8 — [Docusaurus discussion #11140](https://github.com/facebook/docusaurus/discussions/11140) [first-hand; the reporter's employer is inferred from the handle and not confirmed]
- The same thread's summary included an April 2026 comment that `faster: true` brought a build to "5sec max"; this almost certainly refers to a different, smaller site and should not be read as the resolution for the 11,000-file site — [Docusaurus discussion #11140](https://github.com/facebook/docusaurus/discussions/11140) [unverified]
- An internal API reference site of about 3,100 Markdown files took 20+ minutes with frequent out-of-memory failures; switching to esbuild-loader cut 22 minutes to 8 (2022). Another site went from 30+ minutes to 3 with `--no-minify` (2022) — [Docusaurus discussion #3132](https://github.com/facebook/docusaurus/discussions/3132) [first-hand; 2022]
- A site of about 1,500 `.mdx` files used roughly 4 to 8 GB during build — [Docusaurus issue #7410](https://github.com/facebook/docusaurus/issues/7410) [first-hand, snippet; 2022]
- Further 2025 threads exist titled "Problems when building very large website with Docusaurus" and "I tried to build a large site... and did not get very far" — [discussion #10895](https://github.com/facebook/docusaurus/discussions/10895); [discussion #11259](https://github.com/facebook/docusaurus/discussions/11259) [titles only; not opened]

**Migration cost**
- Cloudflare (8 January 2025): developers.cloudflare.com, 4,000+ pages with dozens of pull requests merged daily, moved from Hugo to Astro. The site "had outgrown the workflow for contributors" and faced "scalability and high maintenance costs for user experience improvements". Cost: six weeks total (10 days evaluation, 14 days component migration, 5 days testing, 8-hour code freeze), 8,060 files changed. The earlier Gatsby-to-Hugo move in 2021 changed 4,850 files and took about three weeks — [Cloudflare blog](https://blog.cloudflare.com/open-source-all-the-way-down-upgrading-our-developer-documentation/) [first-hand; the post does not name specific Hugo defects]
- After the move, a Cloudflare docs engineer reported improving Astro dev startup by 316% and page reload by 344%, having hit a build performance edge case from the page count — [kian.org.uk](https://kian.org.uk/making-cloudflare-docs-faster-by-300-percent/) [first-hand, snippet]
- A 2024 account of upgrading a blog to Docusaurus 3.0 and a 2026 pull request jumping from 2.0.0-beta.5 to 3.10.2 show that major upgrades are deferred for years and then done in one step — [thedaxshepherd.com, 22 Feb 2024](https://thedaxshepherd.com/2024/02/22/upgrading-blog-docusaurus-3-0/); [mapillary-python-sdk PR #186](https://github.com/mapillary/mapillary-python-sdk/pull/186) [first-hand, snippets]

**Hosted platforms**
- One open-source project described Mintlify as highly abstracted, quick to start on but with no way to optimise further, and considered moving to Docusaurus — [TraceMachina/nativelink issue #453](https://github.com/TraceMachina/nativelink/issues/453) [first-hand, snippet; date not confirmed]
- Migration stories published by the platforms run in the opposite direction (for example Hedera from GitBook to Mintlify "in about a third of the expected time") — [Mintlify wall of love](https://www.mintlify.com/wall-of-love) [vendor: Mintlify sells hosted docs; testimonial]

### Inferences
- Cloudflare re-platformed in 2021 and again in 2024-25. If that cadence is typical for large docs sites, generator migration is a recurring operating cost, not a one-off.
- Docusaurus build cost is dominated by bundling and static generation, not by reading content, so the cost comes from the React/MDX architecture; that is consistent with Zensical and Starlight marketing themselves on rebuild speed.
- The Material team's account of MkDocs 2.0 should be read knowing they ship a competing generator; the factual points (no plugin system, TOML config) are corroborated by downstream issue titles and by ProperDocs, but were not checked against MkDocs's own announcement.

### Gaps
- MkDocs's own statement about 2.0 was not read; the description here comes from Material for MkDocs, Florian Maas and ProperDocs.
- Current state of MaterialX, and whether Zensical has reached feature parity with Material for MkDocs plugins, is unconfirmed.
- Search quality (Algolia DocSearch limits, local search on large sites) was not researched; no sources.
- No first-hand account of leaving Mintlify, GitBook, Fern or ReadMe with costs or lock-in specifics was found; pricing complaints appeared only in unsourced snippets.
- Hugo, Sphinx and Starlight build-time data at scale were not gathered beyond the Cloudflare case.

## Output beyond the website: PDF, in-app help, API reference, LLM readers

### Takeaway
PDF is not built into the common generators and is handled by third-party scrapers, some of them abandoned. LLM-facing output (llms.txt, Markdown endpoints, MCP servers) is presented as a 2026 requirement almost entirely by vendors; neutral first-hand evidence of teams struggling with it was not found.

### Cited Findings
- Docusaurus PDF output relies on third-party tools that crawl the rendered HTML: docusaurus-prince-pdf (requires Prince XML) and docs-to-pdf, which is a fork of the no-longer-maintained mr-pdf — [signcl/docusaurus-prince-pdf](https://github.com/signcl/docusaurus-prince-pdf); [jean-humann/docs-to-pdf](https://github.com/jean-humann/docs-to-pdf) [snippet]
- API reference generated from OpenAPI into MDX broke on the Docusaurus v3 upgrade ("MDX compilation fails with Docusaurus v3"), and OpenAPI-generated pages were part of the 11,000-file site with 75-minute builds — [docusaurus-openapi-docs issue #591](https://github.com/PaloAltoNetworks/docusaurus-openapi-docs/issues/591); [Docusaurus discussion #11140](https://github.com/facebook/docusaurus/discussions/11140) [first-hand]
- Cloudflare publishes a "Docs for agents" section in its developer docs — [Cloudflare docs](https://developers.cloudflare.com/docs-for-agents/) [first-hand, title only via snippet]
- **[vendor]** GitBook (sells hosted docs) asserts that in 2026 documentation tools "need to support llms.txt output, MCP server support, and structured Markdown delivery" — [GitBook blog](https://www.gitbook.com/blog/best-ai-documentation-tools) [snippet]
- **[vendor]** Scalar (sells API docs tooling) distinguishes llms.txt for bulk ingestion from a docs MCP server for live lookup during coding sessions — [Scalar](https://scalar.com/learn/mcp/mcp-api-documentation) [snippet]
- **[vendor]** Author-it claims "Markdown's lack of semantic structure limits reliable AI retrieval" — [Author-it](https://www.author-it.com/resources/docs-as-code-vs-ccms) [snippet; unsupported assertion from a CCMS seller]

### Inferences
- Generated API reference is a scale amplifier: it adds thousands of machine-written MDX pages that hit both the MDX strictness and the build-time limits.
- The Author-it claim sits awkwardly with the llms.txt and Markdown-endpoint trend, in which plain Markdown is the preferred format for LLM consumption; the two vendor camps contradict each other and neither offers evidence.

### Gaps
- No first-hand team account of implementing llms.txt, per-page Markdown endpoints or a docs MCP server on a self-built static site, including build or maintenance cost, was found.
- No sources on in-app help delivery from docs-as-code content.
- No data on how often customers still require PDF; the "regulatory and contract" rationale came from an unsourced snippet.
- General MCP adoption statistics surfaced in search (downloads, production-use percentages) came from low-quality aggregators and are omitted.

## Who maintains the toolchain, and at what cost

### Takeaway
Sources agree that upkeep falls to writers or to a small number of docs engineers, and that well-run setups (Stripe, Cloudflare) are the product of dedicated engineering investment. Cost is reported in engineer-weeks per migration; no source gives an ongoing budget or headcount figure.

### Cited Findings
- Fabrizio Ferri-Benedetti (2021): docs-as-code is "a giant standing on feet of clay, on the fragile toolchains that we use to create our documentation"; teams keep "Rube Goldberg machines" alive, fighting "a daily battle to keep a ragtag bunch of open source utilities working together"; scaling without "dedicating writers to the task of keeping pipelines operational" needs better tools — [passo.uno, 1 Nov 2021](https://passo.uno/docs-as-code-tools-open-standards/) [practitioner; 2021]
- The same author elsewhere: writers publish "using broken tools that often lack maintainers" — [passo.uno](https://passo.uno/tech-writing-depth-issue/) [practitioner, snippet; date not confirmed]
- On Stripe: its docs quality reflects "the kind of magic that happens when you provide enough technical resources to your technical writers and trust them to do the right thing", and Markdoc "requires dedicated development effort" to adopt — [passo.uno, 11 May 2022](https://passo.uno/markdoc-review/) [practitioner; 2022]
- Cloudflare's migration cost six weeks of a dedicated team's time, the second re-platforming in under four years — [Cloudflare blog, 8 Jan 2025](https://blog.cloudflare.com/open-source-all-the-way-down-upgrading-our-developer-documentation/) [first-hand]
- The MkDocs episode shows the upstream version of the same problem: a tool with 90,000+ dependent projects went 18 months without a release under a single returning author — [Florian Maas, 22 Mar 2026](https://fpgmaas.com/blog/collapse-of-mkdocs/); [Material for MkDocs blog](https://squidfunk.github.io/mkdocs-material/blog/2026/02/18/mkdocs-2.0/)
- Roadie had to re-architect its deployment because TechDocs generation starved the rest of the backend — [Roadie blog](https://roadie.io/blog/splitting-techdocs-out-of-our-monolithic-backstage-deployment/) [vendor first-hand, snippet]

### Inferences
- The maintenance burden has two layers that teams tend to notice separately: the in-house pipeline (CI, linters, plugins, custom components) and the upstream open-source projects, which are themselves often run by one or two people.
- The teams cited as successes are those with engineers assigned to docs infrastructure, which suggests that the limit is staffing as much as technology.

### Gaps
- No source gives ongoing maintenance cost as hours, headcount or money; only one-off migration durations are quantified.
- Write the Docs talks, r/technicalwriting threads, idratherbewriting.com and technicalwriting.dev were not searched in this pass and could hold first-hand cost accounts.
