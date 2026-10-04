# Performance implications of adding drift checking, prose linting, code-sample extraction, OpenAPI processing, link checking and rendered diffing to Ascribe (as of October 2026)

Conventions used in these notes:

- "Own author" means the figure is published by the tool's maintainers or vendor. "Third party" means a user, issue reporter or independent writer.
- "(snippet)" means the figure was surfaced in a search-result summary of the cited page and was not re-read on the page itself. Treat these as lower confidence.
- "(measured 2026-10-03)" means I read the number directly from the GitHub Releases API or npm registry on that date. These are download or unpacked sizes, not benchmark results.
- Figures from different benchmarks are listed separately and must not be combined into one comparison.

## 1. Baselines: published build-time figures for documentation generators at scale

### Takeaway
JavaScript-based generators (Docusaurus, VitePress, Astro/Starlight) have well-documented scaling failures in the thousands-to-tens-of-thousands of pages range: builds of tens of minutes to hours and out-of-memory crashes at 4 to 10 GB heaps. Rust/Rspack-based bundling cut Docusaurus builds 2 to 4 times, but static-site generation, not Markdown parsing, remained the dominant cost. Versions and locales multiply the page count directly.

### Cited Findings

Docusaurus

- Third party, 1 May 2025: a site of about 11,000 Markdown files took 1 hour 15 minutes on a Linux machine with 16 cores and 64 GB RAM, and ran out of memory on local Windows and Mac machines — [Docusaurus discussion #11140](https://github.com/facebook/docusaurus/discussions/11140)
- Own author (maintainer slorber), 2 May 2025, macOS M3, same site: "Bundling with rspack" 762.87 s, "SSG > Generate static files" 1632.71 s, total 2417.55 s (about 40 minutes); data load was 5 s. Parsing and loading was therefore a negligible share; bundling and SSG were the cost — [Docusaurus discussion #11140](https://github.com/facebook/docusaurus/discussions/11140)
- Own author, 14 May 2025, Mac M3 Pro, same site after fixes: cold bundler cache 480 s total (170 s bundling, 270 s SSG, 40 s other); warm bundler cache 350 s total (40 s bundling, 270 s SSG, 40 s other) — [Docusaurus discussion #11140](https://github.com/facebook/docusaurus/discussions/11140)
- Third party, 14 April 2026: a commenter in the same thread reported that `faster: true` reduced a build to "5 seconds maximum". This is a one-line user claim with no site size given and is inconsistent with the maintainer's own 350 to 480 s measurements on the 11,000-file site; treat as not comparable — [Docusaurus discussion #11140](https://github.com/facebook/docusaurus/discussions/11140)
- Own author, 4 November 2024 (Docusaurus 3.6, "Docusaurus Faster": Rspack, SWC, Lightning CSS): production builds React Native website 3.04x faster, Babel website 3.27x faster, Lexical website 2x faster; general expectation "2 to 4 times faster". No hardware stated. No memory figures given, only "consumes less memory overall"; a memory leak affecting i18n sites was fixed — [Docusaurus 3.6 release post](https://docusaurus.io/blog/releases/3.6)
- Own author (PR by slorber on the Babel site): cold build 8.062 s with Faster versus 26.333 s without, "3.27 ± 0.14 times faster" (snippet) — [babel/website PR #2997](https://github.com/babel/website/pull/2997)
- Own author, 26 May 2025 (Docusaurus 3.8), MacBook Pro M3: SSG worker threads make static site generation "~2×" faster on average; Rspack persistent cache makes bundling "~2-5×" faster on rebuilds. React Native website: cold build 31 s (3.8x faster than baseline), warm rebuild 17 s (7x). Docusaurus.io: cold build 42 s (3.5x), warm rebuild 24 s (6.1x). Disabling the `concatenateModule` optimisation gave one large site a 4x faster cold build and 16x faster rebuild — [Docusaurus 3.8 release post](https://docusaurus.io/blog/releases/3.8)
- Third party, 27 July 2020 (Docusaurus v2 era): 2 versions plus "next" at 360 .mdx files each took 26 minutes and peaked at 10 GB RAM, needing `--max_old_space_size=16000`; one version plus next took about 18 minutes and 5 GB. Another user reported 3,100 Markdown files taking 20+ minutes with frequent OOM. `--no-minify` cut one user's build from 30+ minutes to 3 minutes (June 2022) — [Docusaurus discussion #3132](https://github.com/facebook/docusaurus/discussions/3132)
- Own author (slorber, 8 August 2022) on multiplication: "Once a version becomes unmaintained, I suggest 'archiving it' so it's not anymore inside your production SPA and does not increase the build time"; recommends splitting versions/locales into separate sub-sites — [Docusaurus discussion #3132](https://github.com/facebook/docusaurus/discussions/3132)
- Other reported cases exist as issue titles: "After updating to v3.1, large repo build takes 3 hours" and "Very Large Docusaurus (3K Markdown Files) Fails to Build" (titles only; not read) — [Docusaurus issue #9754](https://github.com/facebook/docusaurus/issues/9754), [Docusaurus discussion #10788](https://github.com/facebook/docusaurus/discussions/10788)

MkDocs, Material for MkDocs, Zensical

- Third party: `mkdocs serve` needs a full build before becoming interactive, "ranging from 30 to 40 minutes for large projects" (snippet) — [MkDocs issue #3695](https://github.com/mkdocs/mkdocs/issues/3695)
- Third party: single large topics took up to 30 seconds to render, attributed to large numbers of code blocks (snippet) — [mkdocs-material issue #3643](https://github.com/squidfunk/mkdocs-material/issues/3643)
- Own author, 5 November 2025 (Zensical announcement): "repeated builds – especially when serving the site – are already 4 to 5x faster, as only changed files need to be rebuilt"; initial builds "can sometimes be slower than with MkDocs"; goal is "docs-as-code workflows with tens of thousands of pages"; parallelisation is limited because Zensical still used Python Markdown at that date; no absolute build times or page counts published — [Zensical announcement, Material for MkDocs blog](https://squidfunk.github.io/mkdocs-material/blog/2025/11/05/zensical/)
- Own author (roadmap, snippet): an experimental Rust parser compatible with Python Markdown benchmarks "more than 50x faster than Python Markdown"; no corpus or hardware given in the snippet — [Zensical roadmap](https://zensical.org/roadmap/)
- Zensical was at version 0.0.67 on 30 September 2026 (snippet), i.e. still pre-1.0 — [Zensical releases](https://github.com/zensical/zensical/releases/)

Sphinx

- Third party, 24 January 2026: Godot engine docs, about 1,652 .rst files (about 34 MB), take around 1 hour to build with Sphinx 8.1.3 on macOS arm64. cProfile showed about one third of time in `_copy_except__document` (toctree handling) and about one third in docutils `publish`. No parallel flags or hardware detail given — [Sphinx issue #14277](https://github.com/sphinx-doc/sphinx/issues/14277)
- Third party (snippet): OpenStack nova release-notes job that "typically took 52m to 1h" fell to 15m with parallel sphinx-build — [zuul-jobs commit b892aad548](https://opendev.org/zuul/zuul-jobs/commit/b892aad548c1feeba3452a378dbc638231236d27)
- Own author of Sphinx-Needs (snippet): for a 500-page project the `-j` benefit is about 40% of build time with 8 cores — [Sphinx-Needs performance docs](https://sphinx-needs.readthedocs.io/en/latest/performance/script.html)
- Third party (snippet): version regressions took one build from about 30 s to about 1500 s (Sphinx 3.0.1) — [Sphinx issue #7479](https://github.com/sphinx-doc/sphinx/issues/7479)

Hugo

- Third party, Hugo forum (snippets, not verified on page): a 10,000-page multilingual site built in 155,920 ms; a 124,287-page site took 1,391,936 ms (about 23 minutes) with minification; a 90,000-page site reached 4.62 minutes after optimisation — [Hugo forum: generating a rather large website](https://discourse.gohugo.io/t/generating-a-rather-large-website/19402), [Hugo forum: optimize build time for large websites](https://discourse.gohugo.io/t/how-to-optimize-hugo-build-time-for-large-websites/54633)
- Third party, 19-21 July 2025: a user with 120,000-page sites reported that 80,000 pages with `--minify` was too much for an M1 Mac laptop and that `--minify` adds 2 to 4 minutes; the maintainer (bep) stated rendering takes the bulk of time and memory on big sites; remedies were render segments (`--renderSegments`) and external minification — [Hugo forum: sites over 100,000 pages](https://discourse.gohugo.io/t/hugo-sites-over-100-000-pages-issues-remedies/55369)
- Third party, 28 October 2020: a cross-generator test (Hugo, Eleventy, Jekyll, Gatsby, Next, Nuxt from 1 to 64,000 files) found Hugo fastest at every size; the author deliberately reports relative rather than absolute values because of machine variance — [CSS-Tricks: Comparing Static Site Generator Build Times](https://css-tricks.com/comparing-static-site-generator-build-times/)

Astro and Starlight

- Third party, 2026: a large Starlight site (Astro v7.0.6, Starlight v0.41.3, Node v24.18.0, macOS arm64) with heavy KaTeX content ran out of memory at a heap of about 4,093 to 4,109 MB during the content-sync phase, before page generation; cause identified as the `glob()` loader eagerly rendering every Markdown/MDX entry and holding rendered HTML in the content store — [Astro issue #17301](https://github.com/withastro/astro/issues/17301)
- Third party (snippet): after Astro 5.1.7 / Starlight 0.31, content sync became markedly slower and could exhaust memory with many or large files — [Astro issue #13050](https://github.com/withastro/astro/issues/13050)
- Own author (2021 post, snippet): a build optimisation improved build times "by up to 75%" for 10,000+ page sites — [Astro blog: Scaling Astro to 10,000+ Pages](https://astro.build/blog/experimental-static-build/)

VitePress

- Third party, 1 March 2026: about 26,000 pages (20 locale directories at about 1,300 routes each) crashed with heap out of memory during "rendering pages" even with an 8 GB heap (last recorded 7,997.8 MB); VitePress 2.0.0-alpha.16, Node v22.21.1, Windows 11; no maintainer response — [VitePress issue #5134](https://github.com/vuejs/vitepress/issues/5134)
- Third party, 26 December 2023: a "very large" site with local search enabled took 4 hours 27 minutes (16,008 s) to build; files were indexed twice per build — [VitePress issue #3377](https://github.com/vuejs/vitepress/issues/3377)

Mintlify-style hosted builds

- Vendor/third-party pages (snippets only): changes are "typically live within 2-3 minutes"; for large docs a redeploy "could take up to 20 minutes" — [Mintlify quickstart](https://www.mintlify.com/docs/quickstart), [Hackmamba Mintlify migration guide](https://hackmamba.io/technical-documentation/mintlify-documentation-migration-guide/)

### Inferences
- In the one Docusaurus case with a phase breakdown (11,000 files), content loading was 5 s out of about 2,400 s. The slowness users complain about is bundling and per-page HTML generation in a JavaScript runtime, not Markdown parsing. A native tool that does not bundle a single-page application avoids the dominant cost category altogether, so Docusaurus numbers are an upper-bound pain reference rather than a like-for-like baseline for `ascribe build`.
- The recurring failure mode is memory, not only time: 4 GB (Astro default heap), 8 GB (VitePress) and 10 GB (Docusaurus v2) ceilings appear repeatedly. Holding the fully rendered output of every page in memory at once (Astro content store) is the specific anti-pattern named by a reporter. Ascribe's "resolved snapshot of the whole project" should be checked against the same risk if rendered HTML or expanded OpenAPI content is retained per page.
- Versions and locales multiply linearly at minimum: the VitePress case is literally 20 locales x 1,300 pages; Docusaurus maintainers advise archiving versions out of the build.

### Gaps
- No published pages-per-second figures were found for mdBook, Zola, Starlight or VitePress at documentation scale from the tools' own authors.
- No absolute Zensical build times (pages, seconds, hardware) are published; only relative "4 to 5x" rebuild claims.
- Mintlify publishes no build-time benchmarks; only anecdotal deploy durations in snippets.
- Hugo forum figures are from snippets and lack hardware; the tool's own documentation gives no benchmark table that I located.

## 2. Rust tooling precedents: reported speedups and credited techniques

### Takeaway
Rust rewrites in adjacent domains report 10x to 100x speedups over interpreted-language incumbents on cold whole-repository runs, and credit per-file parallelism, single-pass traversal, hand-written parsers, on-disk caching and (for semantic tools) salsa-style incremental computation. Almost all headline figures are the tool's own.

### Cited Findings
- Ruff (own author): "10-100x faster" than existing linters and formatters; headline chart is "Linting the CPython codebase from scratch"; built-in caching "to avoid re-analyzing unchanged files" — [Ruff docs](https://docs.astral.sh/ruff/)
- Ruff testimonials on the vendor's page (third-party statements hosted by the author): pylint "about 2.5 minutes" versus Ruff "0.4 seconds" on a 250,000-line module's codebase (Nick Schrock); flake8 "~20s" versus Ruff "~0.2s" for a whole repository (Bryan Van de Ven) — [Ruff docs](https://docs.astral.sh/ruff/)
- Ruff v0.4.0 (own author, snippet): hand-written recursive-descent parser is ">2x faster", giving "a 20-40% speedup for all linting and formatting invocations" — [Ruff v0.4.0 post](https://astral.sh/blog/ruff-v0.4.0)
- Ruff techniques as described by third-party write-ups (snippets): one AST per file, all rules in a single traversal, files processed in parallel, results cached so reruns only lint changed files — [LWN: Ruff, a fast Python linter](https://lwn.net/Articles/930487/), [Talk Python episode 400](https://talkpython.fm/episodes/show/400/ruff-the-fast-rust-based-python-linter)
- ty (own author, 16 December 2025, Apple M4): cold check of home-assistant/core 2.19 s versus Pyrefly 5.32 s, Pyright 19.62 s, mypy 45.66 s; "Without caching, ty is consistently between 10x and 60x faster than mypy and Pyright"; architecture "built around" incrementality using salsa — [Astral: ty announcement](https://astral.sh/blog/ty)
- oxc (own author; no hardware on the page): parser 3x faster than swc and 5x faster than Biome (noting Biome produces a CST); oxlint 50x to 100x faster than ESLint depending on core count; oxfmt 3x faster than Biome and 35x faster than Prettier; transformer 4x faster than swc with 20% less memory — [Oxc benchmarks](https://oxc.rs/docs/guide/benchmarks)
- Biome v2 (own author, 17 June 2025): the type-aware `noFloatingPromises` rule catches "about 75%" of cases typescript-eslint would, "at a fraction of the performance impact"; the post acknowledges "many users choose Biome for its speed" while introducing a project-scanning phase, and gives no timing or memory figures for multi-file analysis — [Biome v2 announcement](https://biomejs.dev/blog/biome-v2/)
- rumdl (own author; benchmark last run February 2026, methodology reviewed August 2026; hyperfine, cold start, cache disabled, Rust Book repository, 478 Markdown files): rumdl 217 ms; mado 77 ms; pymarkdown 240 ms; remark-lint 671 ms; markdownlint-cli2 2.2 s; markdownlint-cli 2.7 s; mdformat 4.0 s; Prettier 4.8 s. The page states results are "directional" and that "tool capabilities and workloads are not identical"; no hardware stated in the extract — [rumdl benchmarks](https://rumdl.dev/benchmarks/)
- Zensical (own author, 5 November 2025): credits a "differential build engine" (ZRX) with "differential builds, caching, and data flow orchestration" for 4 to 5x faster repeated builds — [Zensical announcement](https://squidfunk.github.io/mkdocs-material/blog/2025/11/05/zensical/)
- typst (third-party summary of project architecture docs, snippet): "All Typst language features must accommodate for incremental compilation"; the `comemo` memoisation library does the dependency tracking — [typst architecture notes mirror](https://huggingface.co/spaces/ecyht2/typst-docs-rag/blame/main/sources/docs/dev/architecture.md)
- lychee (own author): "Fast, async, stream-based link checker"; `--threads` defaults to the number of cores; network concurrency defaults to 128 — [lychee README](https://github.com/lycheeverse/lychee)

### Inferences
- Note that in rumdl's own benchmark another Rust linter (mado, 77 ms) is about 2.8x faster than rumdl (217 ms) and a Python tool (pymarkdown, 240 ms) is nearly level with it on a 478-file corpus. At a few hundred files, process start-up and rule count dominate; "Rust" alone does not fix a figure.
- Biome v2 is the closest precedent for what Ascribe is considering: a fast single-file tool adding whole-project analysis. Its maintainers publicly flagged the speed risk and published no numbers, which suggests the cost is workload-dependent and hard to headline.
- The techniques that recur across independent projects are: (1) per-file work that is embarrassingly parallel, (2) one parse and one traversal per file with all checks attached, (3) a content-hash cache on disk for CLI reruns, and (4) a query/memoisation layer for cross-file facts.

### Gaps
- No source located that attributes a measured share of speedup to arena allocation specifically; the oxc benchmark page reviewed does not mention technique.
- No published benchmark for Zensical's Rust core in absolute terms.
- typst's own incremental-compilation timings were not retrieved.

## 3. Incremental checking in language servers and acceptable latency

### Takeaway
The best-documented design is rust-analyzer's: a salsa query database, immutable snapshots, cancellation on every change, per-file syntax trees, and an explicit invariant that local edits do not invalidate global data. The only hard incremental latency numbers found are Astral's: single-digit milliseconds for ty versus hundreds of milliseconds to seconds for competitors. No official LSP or VS Code latency budget for diagnostics was found.

### Cited Findings
- rust-analyzer (own author): "The core invariant we maintain is 'typing inside a function's body never invalidates global derived data'" — [rust-analyzer architecture](https://rust-analyzer.github.io/book/contributing/architecture.html)
- rust-analyzer (own author): "Syntax tree is built for a single file. This is to enable parallel parsing of all files." and "Parsing never fails, the parser produces `(T, Vec<Error>)` rather than `Result<T, Error>`" — [rust-analyzer architecture](https://rust-analyzer.github.io/book/contributing/architecture.html)
- rust-analyzer (own author) on cancellation: "When applying a change, salsa bumps this counter and waits until all other threads using salsa finish. If a thread does salsa-based computation and notices that the counter is incremented, it panics with a special value." — [rust-analyzer architecture](https://rust-analyzer.github.io/book/contributing/architecture.html)
- rust-analyzer (own author) on state: "`AnalysisHost` is a state to which you can transactionally `apply_change`. `Analysis` is an immutable snapshot of the state." — [rust-analyzer architecture](https://rust-analyzer.github.io/book/contributing/architecture.html)
- ty language server (own author, 16 December 2025, Apple M4, PyTorch repository, recompute diagnostics after an edit): the post's text gives ty 4.7 ms versus Pyright 386 ms and Pyrefly 2.38 s; a chart on the same page was extracted as 4.5 ms, 370.5 ms and 2.60 s. The two sets differ slightly within the same source — [Astral: ty announcement](https://astral.sh/blog/ty)
- VS Code Markdown language server (own author, 16 August 2022): when implementing link diagnostics in-process on a large Markdown workspace (vscode-docs), "I kept accidentally blocking the extension host for a few hundred milliseconds"; moving to a separate language-server process was the fix — [VS Code blog: Introducing the Markdown Language Server](https://code.visualstudio.com/blogs/2022/08/16/markdown-language-server)
- vale-ls default behaviour (own author): `lintOnChange` true; `debounceMs` 300, "How long typing has to settle before `lintOnChange` runs Vale" — [Vale LSP guide](https://docs.vale.sh/guides/lsp)
- Third-party issue (snippet): typing in large notes becomes slow when a Markdown LSP uses full-document synchronisation, giving O(lines) work per keystroke — [QOwnNotes issue #3739](https://github.com/pbek/QOwnNotes/issues/3739)
- Third-party comparison page (snippet): markdown-oxide uses regular expressions rather than an AST "as a deliberate tradeoff for indexing speed" and is "fast in the common case" with no published benchmark — [IWE comparison page](https://iwe.md/docs/concepts/comparison/)
- A non-authoritative blog on inline code completion (not diagnostics) puts "feels instant" at roughly 100 to 150 ms from last keystroke to first visible result (snippet) — [Goatfied: latency budgets for ghost text](https://www.goatfied.com/blog/latency-budgets-for-ghost-text-experience)

### Inferences
- The only shipped default debounce found for a prose/docs server is vale-ls at 300 ms. That, plus the "few hundred milliseconds is a problem when it blocks" statement from the VS Code team, brackets the practical range: diagnostics computed off the UI thread a few hundred milliseconds after typing stops are in line with existing docs tooling; single-digit milliseconds is what the fastest salsa-based servers achieve.
- rust-analyzer's invariant translates to docs as: editing prose inside a page body should not invalidate project-global facts (navigation, anchors index, snippet index, OpenAPI model). Any new capability that makes every page depend on a large shared input (an OpenAPI spec, a source-tree scan) needs that input to be a separately memoised query, or each keystroke re-runs it.
- Ascribe's current "resolved snapshot of the whole project, re-check on edits" matches the snapshot half of rust-analyzer's design; whether it matches the fine-grained invalidation half was not established here.

### Gaps
- No normative latency budget for diagnostics exists in the LSP specification or VS Code extension guidance that I could find; the 100 to 150 ms figure above is for completions and from a non-authoritative source.
- No published performance numbers for marksman or markdown-oxide.
- No published incremental-latency figures for Ruff's own (lint) server or for Biome's daemon were found.

## 4. Vale and Rust-native prose/Markdown linters

### Takeaway
Vale's own headline is 2,827 Markdown pages with 82 rules in under 20 seconds; its editor integration is a Rust wrapper that runs a separately installed Vale binary after a 300 ms debounce. Rust-native alternatives publish per-document latency (Harper, 10 ms) or whole-repo cold runs (rumdl, 217 ms for 478 files), but these measure different things and are all self-reported.

### Cited Findings
- Vale (own author): GitLab documentation, 2,827 Markdown pages, 82 rules, "Start to finish <20s". No hardware or methodology given. Described as "One Go binary. Parallel checks. No separate runtime." — [vale.sh](https://vale.sh/)
- Vale adoption context (own author): 4,194 repositories using the Vale Action; "111 of 156 checked repos run Vale in CI" — [vale.sh](https://vale.sh/)
- vale-ls (own author): "implements the Language Server Protocol around a local installation of Vale"; can install Vale into a `vale_bin` folder beside itself; defaults `lintOnChange` true, `debounceMs` 300 — [Vale LSP guide](https://docs.vale.sh/guides/lsp)
- vale-ls is written in Rust (Cargo project) while Vale is Go, so the server and the linter are separate executables — [vale-ls repository](https://github.com/vale-cli/vale-ls)
- Third-party blog (snippet): proselint took 50 s on a blog's posts; Vale "with more checks" took 1.5 s on the same content — [bulimov.me: Code-like linting for prose](https://bulimov.me/post/2022/01/16/prose-lint/)
- Third-party guide (snippet, unverified and unattributed methodology): "On a corpus of 1,500 Markdown files, Vale completes in under 2 seconds while comparable tools take 10-30 seconds" — [docsio: Vale linter guide](https://docsio.co/blog/vale-linter)
- Third-party report (snippet): in one project a full Vale run took 20 seconds, but regular expressions in the vocabulary configuration slowed it to over ten minutes. The search result did not make clear which page this came from; it is listed here as an unverified pathological-case report — [search-surfaced; closest candidate GitLab Vale documentation tests page](https://docs.gitlab.com/development/documentation/testing/vale/)
- Harper (own author): suggestion time Harper 10 ms, LanguageTool 650 ms, Grammarly 4000 ms; no methodology, document size or hardware is given in the table — [Harper COMPARISON.md](https://github.com/Automattic/harper/blob/master/COMPARISON.md)
- rumdl (own author, February 2026): 217 ms for 478 files cold, versus markdownlint-cli2 2.2 s — [rumdl benchmarks](https://rumdl.dev/benchmarks/)
- typos keeps dated benchmark run files in its repository (2019-10-24 to 2021-05-21); the numbers inside were not retrieved — [typos benchsuite runs](https://github.com/crate-ci/typos/blob/master/benchsuite/runs/)

### Inferences
- Vale's stated figure works out to under about 7 ms per page across all cores (20 s / 2,827 pages); the source gives it as an upper bound, so the real per-page figure is not known. It is fast enough that Vale-in-CI as an external step is not a CI-minutes problem at a few thousand pages.
- The editor path is where shelling out costs: vale-ls is structurally a process-per-lint design with a 300 ms debounce in front of it. An in-process linter avoids process start-up and re-reading configuration and styles on each run, which is the architectural difference Harper's 10 ms claim relies on.
- The "20 seconds to over ten minutes" report, if accurate, shows that user-supplied regular expressions are the main performance hazard of a rule-driven prose linter, independent of implementation language. Rust's `regex` crate guarantees linear-time matching, but that was not established from a source here.

### Gaps
- No measurement was found of Vale's per-invocation start-up cost, or of spawn-per-file versus long-running-process cost. vale-ls documentation does not state how often or how it invokes the binary beyond "runs Vale" after the debounce.
- No independent benchmark comparing Vale with Harper, typos or rumdl on a common corpus was found.
- Harper's memory footprint claim ("1/50th of LanguageTool") appeared only in third-party snippets and is not in the comparison table I read.

## 5. Link checking

### Takeaway
External link checking is bounded by remote servers, not by local compute: lychee defaults to 128 concurrent requests but throttles to 10 per host with a 50 ms interval and a 20 s timeout, and GitHub links are rate-limited per token. Real projects report 10 to 14 minute CI runs with flaky outcomes and are moving external checks to scheduled jobs.

### Cited Findings
- lychee defaults (own author): `--max-concurrency` 128; `--host-concurrency` 10; `--host-request-interval` 50 ms; `--timeout` 20 s; `--max-retries` 3; `--retry-wait-time` 1 s; `--max-redirects` 10; `--threads` equals core count; cache is off by default ("lychee will not store any data on disk"), stored in `.lycheecache` when enabled, `--max-cache-age` 1 day — [lychee README](https://github.com/lycheeverse/lychee)
- lychee rate-limit guidance (own author): on HTTP 429 lychee retries with backoff, and many concurrent requests to one site raise the chance of being limited; mitigations are `--max-retries 0`, lowering `--max-concurrency`, accepting 429 (`--accept 200,429`), `--cache`, or excluding hosts. With a token GitHub allows "1,000 requests per hour per repository" (15,000 for enterprise) — [lychee rate limits](https://lychee.cli.rs/troubleshooting/rate-limits/)
- Third party (OWASP Cheat Sheet Series): recent link-check runs took "10m08s" and "13m53s" for 2,432 external file/URL pairs (2,223 distinct URLs) across 127 sheets; one run performed "3,035 initial link checks plus 210 checks from whole-file retries"; a push run failed while the PR run on the identical commit passed; the known-failures baseline holds 351 exceptions including 226 recorded 403s and 45 recorded 404s. Proposal: check only new or changed external links on PRs, run the full audit weekly, cache results up to 24 hours, start at 8 concurrent requests overall and 1 to 2 per host, and classify 403/429/timeouts as "unverified" rather than broken — [OWASP CheatSheetSeries issue #2488](https://github.com/OWASP/CheatSheetSeries/issues/2488)
- Third party (snippet): a full CI run scanning 576 links takes about 1 minute; source page not identified in the result — [search-surfaced; candidate tech-tales.blog lychee post](https://tech-tales.blog/en/posts/2026/lychee-link-checker/)
- lychee-action documentation shows scheduled (cron) workflows with `fail: false` as a standard pattern (snippet) — [lychee-action](https://github.com/lycheeverse/lychee-action)

### Inferences
- From the OWASP case: about 2,400 links in 10 to 14 minutes is roughly 3 to 4 links per second end to end, despite a tool capable of 128 concurrent requests. Per-host throttling, timeouts (20 s each by default) and retries dominate. This is why external checks are kept out of the edit loop and usually out of blocking PR checks.
- Internal link and anchor checking is a different problem (pure local graph lookup against the project snapshot) and belongs in the per-edit path; the VS Code Markdown team's "few hundred milliseconds" stalls came from doing even this naively in-process on a large workspace.
- A 20 s timeout per dead host means a handful of dead domains adds minutes regardless of local speed; a persisted cache with a 1-day default age is the main lever lychee offers.

### Gaps
- lychee publishes no throughput benchmark (links per second) of its own.
- No source measured memory use of link checking at scale.

## 6. Code-sample extraction, file watching and sample testing (also relevant to drift checking)

### Takeaway
Scanning a source tree for markers is cheap at ripgrep-class throughput (tens of milliseconds over the Linux kernel tree on a fast desktop). Watching that tree is constrained by Linux inotify limits, which editors already hit on large workspaces. Actually compiling or running samples is the expensive part: reported doctest times are tens of seconds to minutes and are dominated by compilation.

### Cited Findings
- ripgrep (own author; Intel i9-12900K 5.2 GHz; Linux kernel source after `make defconfig && make -j8`; pattern `[A-Z]+_SUSPEND` with `-w`): ripgrep 0.082 s; git grep 0.273 s; ag 0.443 s; ugrep 0.639 s; ack 2.935 s. Ignoring gitignore and restricting to C files: ripgrep 0.063 s versus GNU grep 0.674 s — [ripgrep README](https://github.com/BurntSushi/ripgrep)
- ripgrep (own author, same hardware; single file of about 13 GB cached in memory, pattern `Sherlock [A-Z]\w+`): 1.042 s. Pattern with no literal to optimise on (`[A-Za-z]{30}`): 15.569 s — a roughly 15x slowdown from pattern shape alone — [ripgrep README](https://github.com/BurntSushi/ripgrep)
- Third-party descriptions put that kernel corpus at about 75,000 files and about 900 MB (snippet; not stated in the README extract) — [codeant.ai ripgrep vs grep](https://codeant.ai/blogs/ripgrep-vs-grep-performance)
- inotify limits (snippets): each watched directory consumes a watch; a common default `max_user_watches` is 8,192; kernels since 5.11 scale the default up to 1,048,576 depending on RAM; the limit is per user, so it is shared across every process that user runs — [Watchexec: Linux inotify limits](https://watchexec.github.io/docs/inotify-limits.html)
- VS Code's recommended value is `fs.inotify.max_user_watches=524288`, and exceeding the limit produces "ENOSPC: System limit for number of file watchers reached" (snippets) — [support.tools: max_user_watches for VS Code](https://support.tools/post/increasing-max-user-watches-for-large-workspaces-in-vscode/)
- Rust 2024 edition merged doctests (own author, Rust project): before the 2024 edition rustdoc compiled each documentation code block as a separate executable; merging them into one binary "greatly reduces" total time because "most of the time in doctests is spent in compilation" (snippets) — [Rust Edition Guide: rustdoc doctests](https://doc.rust-lang.org/nightly/edition-guide/rust-2024/rustdoc-doctests.html), [Rust project goal: merged doctests](https://rust-lang.github.io/rust-project-goals/2024h2/merged-doctests.html)
- Third-party measurements of that change (snippets): 69 doctests 61.85 s to 6.29 s; 1,125 doctests across four crates 190.74 s to 12.55 s; another project 224 s to 42 s locally and 178 s to 58 s in CI — [TeNeT PR #1911](https://github.com/Ryo-wtnb11/TeNeT/pull/1911), [tensor4all-rs issue #769](https://github.com/tensor4all/tensor4all-rs/issues/769), [openquant PR #200](https://github.com/Open-Quant/openquant/pull/200)

### Inferences
- The per-sample compile figures before merging (61.85 s / 69, i.e. just under a second each; 190.74 s / 1,125, i.e. about 0.17 s each) show that "one process or compile per snippet" is the cost model to avoid; batching snippets into one compilation unit was worth 5x to 15x in the cited cases. The same shape applies to any language whose samples must be compiled.
- Marker extraction itself is not a CI-time concern, but it changes the language server's dependency set: the server now depends on files outside the docs project, which means (a) more inotify watches against a per-user limit already shared with VS Code and with every other Ascribe server process, and (b) a need to respect ignore files to avoid walking `node_modules`/`target`.
- Drift checking that compares docs against source files inherits exactly this cost profile: cheap scan, non-trivial watch set.

### Gaps
- No published throughput or scale-limit figures were found for the Rust `notify` crate or for Watchman at monorepo scale in this pass.
- No published figures were found for the cost of running extracted documentation samples in languages other than Rust (for example Python or TypeScript snippet test tools).
- No source was found that measures "drift checking" as a distinct capability in any comparable tool.

## 7. OpenAPI: parse, render and diff cost

### Takeaway
Real-world specs are 4 to 10 MB, and the cost is in resolution and typed deserialisation rather than raw parsing: a Rust tool reached 5.67 GB and about 155 s on Stripe's 7.7 MB spec by expanding `$ref`s without sharing, and Go's typed deserialisation of the Kubernetes spec was 18x slower than untyped. Browser renderers degrade badly at about 2 MB.

### Cited Findings
- Spec sizes (GitHub file views, snippets): Stripe `spec3.json` 7.66 MB and `spec3.sdk.json` 9.59 MB; GitHub `api.github.com.yaml` 9.44 MB; Kubernetes `swagger.json` 4.27 MB — [stripe/openapi spec3.json](https://github.com/stripe/openapi/blob/master/openapi/spec3.json), [github/rest-api-description](https://github.com/github/rest-api-description/blob/main/descriptions/api.github.com/api.github.com.yaml), [kubernetes swagger.json](https://github.com/kubernetes/kubernetes/blob/master/api/openapi-spec/swagger.json)
- Third party, 21-22 September 2026 (Barbacane, a Rust-based OpenAPI compiler): parsing `stripe.json` (7.7 MB) peaked at 5.67 GB and ran about 155 s before the hosted CI runner was reclaimed (exit 143). Cause: "$ref" resolution duplicated subgraphs — "a modestly sized component that reaches a large subgraph is duplicated once per reference"; Stripe's `error` schema was expanded 594 times — [barbacane issue #221](https://github.com/barbacane-dev/barbacane/issues/221)
- Third party, 6 September 2022 (Intel Core i7-8559U 2.70 GHz, macOS): deserialising Kubernetes `swagger.json` into the typed `spec.Swagger` took 529,833,040 ns (about 529 ms) versus 29,475,772 ns (about 29 ms) into `map[string]interface{}`; cause was repeated re-deserialisation for vendor extensions at each nesting level — [kube-openapi issue #315](https://github.com/kubernetes/kube-openapi/issues/315)
- Third party (snippet): about 900 ms to JSON-parse the Kubernetes swagger file and about 20 ms to un-gzip it from data embedded in the binary — [kustomize issue #3670](https://github.com/kubernetes-sigs/kustomize/issues/3670)
- Redoc, third party, 11 May 2021: a 1.86 MB spec rendered a page that was "basically unusable", with interactions taking more than 1 s — [Redoc issue #1615](https://github.com/Redocly/redoc/issues/1615)
- Redoc, third party, 7 November 2024: a spec of 190,000+ lines consumed about 1.2 GB of browser memory with a First Contentful Paint of 1.2 minutes; reporter requested virtualisation; no maintainer reply visible — [Redoc issue #2615](https://github.com/Redocly/redoc/issues/2615)
- Redoc, third party (snippet): a 33,300-line swagger file caused "JavaScript heap out of memory" during doc generation — [Redoc issue #1357](https://github.com/Redocly/redoc/issues/1357)
- docusaurus-openapi-docs (own author, snippet): large schemas produced heavy MDX with embedded JSON that slowed MDX compilation; fixed by `externalJsonProps` (default true), which writes large JSON props to separate files "bypassing MDX AST processing entirely" — [docusaurus-openapi-docs repository](https://github.com/PaloAltoNetworks/docusaurus-openapi-docs)
- oasdiff (own author): detects "755 distinct changes"; no timing figures published — [oasdiff](https://www.oasdiff.com/)

### Inferences
- The Barbacane case is the most directly relevant warning for Ascribe: a Rust implementation with a typed model still hit 5.67 GB because references were inlined. Keeping `$ref`s as shared handles (interned or arena-indexed) rather than expanding them is the difference between megabytes and gigabytes on Stripe-class specs.
- Since each Ascribe language server holds a whole-project snapshot, a resolved OpenAPI model is held once per server process; with one process per docs project in a workspace, a shared spec referenced by several projects would be parsed and held several times.
- Generating one page per operation from a 5 to 10 MB spec is also a page-count multiplier (and interacts with versions and locales as in section 1).
- The docusaurus-openapi-docs fix (keep the big JSON out of the Markdown/MDX pipeline) generalises: do not route spec content through the Markdown parser or the typed content model as inline data.

### Gaps
- No published timing for OpenAPI diff (oasdiff or others) on large specs was found.
- No published parse-time benchmark for Rust OpenAPI crates on the Stripe, GitHub or Kubernetes specs was found beyond the Barbacane failure report.
- Swagger UI large-spec issues exist by title ([swagger-ui issue #2710](https://github.com/swagger-api/swagger-ui/issues/2710)) but were not read for figures.
- Spec file sizes are from search snippets of GitHub file views and may have changed since.

## 8. Rendered diffing: algorithm complexity and practical limits

### Takeaway
Line/sequence diffs are cheap and near-linear on similar inputs; tree diffs are quadratic or worse and every serious tool ships hard limits and a fallback. difftastic's own documentation says it "scales relatively poorly on files with a large number of changes", caps the search at 3,000,000 graph vertices and 1,000,000 bytes, and falls back to a line diff.

### Cited Findings
- Myers: O(ND) time, where N is the sum of sequence lengths and D the edit distance, so cost grows with how different the inputs are (snippet) — [jsdiff: Myers' diff algorithm](https://www.jsdiff.com/docs/myers-diff-algorithm.html)
- Histogram extends patience diff to "support low-occurrence common elements" (snippet) — [git diff-options documentation](https://git-scm.com/docs/diff-options/2.6.7)
- imara-diff (own author's README, via a fork, snippet): histogram "outperforms Myers algorithm by 10% - 100% across a wide variety of workloads" — [imara-diff README (fork)](https://github.com/cosgroveb/imara-diff)
- A search snippet reports average runtimes in Git's implementation of Myers 0.101 ms, histogram 0.115 ms, patience 0.128 ms; the snippet did not make clear which of two academic sources this is from, and the abstract I read does not contain it. Unverified; this conflicts in direction with the imara-diff claim above (different implementations and workloads) — [arXiv:2507.22071](https://arxiv.org/pdf/2507.22071); possibly [Empirical Software Engineering: How different are different diff algorithms in Git?](https://link.springer.com/article/10.1007/s10664-019-09772-z)
- Third party (bachelor's thesis, Niels Glodny, submitted 16 July 2025): "The histogram diff algorithm has pathological cases where a single-line change can cause the entire rest of the file to be marked as changed." — [arXiv:2507.22071](https://arxiv.org/abs/2507.22071)
- Zhang-Shasha tree edit distance: O(m²n²) time in general, O(n⁴) time and O(n²) space for similar-sized trees, O(n² log² n) for trees of logarithmic depth; RTED and APTED achieve O(n³) time and O(n²) space (snippets) — [RTED paper, VLDB 2012](https://vldb.org/pvldb/vol5/p334_mateuszpawlik_vldb2012.pdf), [Tree Edit Distance, University of Salzburg](https://tree-edit-distance.dbresearch.uni-salzburg.at/)
- GumTree: worst-case O(n²) with n = max(|T1|, |T2|) (snippet) — [TU Delft: Evaluating Stable Tree Differencing with GumTree and HyperDiff](https://pure.tudelft.nl/admin/files/247281336/Evaluating_Stable_Tree_Differencing_with_Gumtree_and_HyperDiff.pdf)
- difftastic (own author): "Difftastic treats diff calculations as a route finding problem on a directed acyclic graph" solved with Dijkstra; "Constructing the whole graph would require exponential memory relative to the number of syntax nodes", so neighbours are generated lazily — [difftastic manual: diffing](https://difftastic.wilfred.me.uk/diffing.html)
- difftastic (own author, snippets): graph size is O(L x R) in the item counts of the two sides, reaching several million vertices; it "scales relatively poorly on files with a large number of changes, and can use a lot of memory"; it discards obviously unchanged regions first and "if the graph is just too big, difftastic falls back to a conventional line-oriented diff" — [difftastic README](https://github.com/Wilfred/difftastic), [Wilfred Hughes: Difftastic, the Fantastic Diff](https://www.wilfred.me.uk/blog/2022/09/06/difftastic-the-fantastic-diff/)
- difftastic limits (man page, snippet): `DFT_GRAPH_LIMIT` default 3,000,000 vertices; `DFT_BYTE_LIMIT` default 1,000,000 bytes, above which structural parsing is skipped — [difft man page](https://github.com/Wilfred/difftastic/blob/master/difft.1.md)
- difftastic documented hard cases (own author): 18 listed, including moved content in or out of delimiters, reordering within lists, "middle insertions", sliders, reflowed comments and small changes inside large strings; unordered data is called out as "NP-hard" and "MAX SNP-hard"; any parse error triggers fallback to a line diff — [difftastic manual: tricky cases](https://difftastic.wilfred.me.uk/tricky_cases.html)
- Third party: a difftastic process "died of signal 9 after loading big JSON" (issue title only) — [difftastic issue #316](https://github.com/Wilfred/difftastic/issues/316)
- Third party: an optimisation write-up is titled "How do I boost difftastic by 4x", noting vertex construction as the bottleneck (snippet) — [QuarticCat: optimize difftastic](https://blog.quarticcat.com/posts/optimize-difftastic/)
- Python's `difflib.HtmlDiff`/`ndiff` has "dreadful performance" when most lines differ, because it searches for intra-line similarity across non-adjacent lines (snippet) — [Python issue 6931](https://bugs.python.org/issue6931)

### Inferences
- difftastic's tricky cases map closely onto documentation edits: reflowed paragraphs, reordered list items, content moved into or out of a container (for example wrapping steps in an admonition or tab), and small edits inside long text nodes. A rendered-content differ should expect exactly these to be both the common case and the expensive or low-quality case.
- Every production tree differ examined bounds its work and degrades to a sequence diff. A rendered-diff feature needs the same: a node or byte budget per page with a block-level sequence-diff fallback, because worst-case cost is quadratic in page size and grows with the amount changed.
- Rendered diffing is naturally a CI or review-surface operation over two builds rather than a per-keystroke check; nothing found suggests any tool runs tree diffs in the edit loop.

### Gaps
- No published benchmark of HTML-specific diff tools (node counts versus time) was found.
- The git diff algorithm runtime figures could not be tied to a verified source.
- difftastic limit defaults are from a search snippet of the man page, not read directly.

## 9. Multi-process cost: many language servers and monorepo handling

### Takeaway
VS Code documents both models (one multiplexed server using `workspaceFolders`, or one server instance per folder) without mandating either. The cautionary data point is rust-analyzer run one-per-root: 22 to 39 GB RSS per instance and 60+ GB total across about 82 workspaces. Per-process baseline cost for small native servers is not published.

### Cited Findings
- VS Code (own author): multi-root "can be supported using either a single language server or by using multiple language servers—one for each workspace"; the official `lsp-multi-server-sample` starts one server instance per workspace folder, and one-per-folder "makes it easier to keep the list of dependencies inside of a workspace/project separated" (snippets) — [VS Code wiki: Adopting Multi Root Workspace APIs](https://github.com/microsoft/vscode/wiki/Adopting-Multi-Root-Workspace-APIs), [VS Code Language Server Extension Guide](https://code.visualstudio.com/api/language-extensions/language-server-extension-guide)
- LSP `InitializeParams` carries `workspaceFolders`, enabling one server to handle several folders (snippet) — [VS Code wiki: Adopting Multi Root Workspace APIs](https://github.com/microsoft/vscode/wiki/Adopting-Multi-Root-Workspace-APIs)
- Third party, 26 June 2026 (Linux x86_64, 64 GB RAM, rust-analyzer v1.95.0, repository with about 82 independent Cargo workspaces plus git worktrees): an integration that spawned one rust-analyzer per workspace root saw individual instances at 22 GB to 39 GB RSS and 60+ GB total, OOM-crashing the session. Requested fixes: deduplicate instances per logical project, add a `maxInstances` cap or shared-server mode, and shut down idle instances — [claude-plugins-official issue #3417](https://github.com/anthropics/claude-plugins-official/issues/3417)
- Third party: in VS Code, when a Cargo workspace is split into several folder views, rust-analyzer duplicates the loaded workspace for each folder, causing excessive memory use (snippet) — [rust-analyzer issue #14571](https://github.com/rust-lang/rust-analyzer/issues/14571)
- Third party: a single rust-analyzer server using more than 2.5 GB (snippet) — [rust-analyzer discussion #19423](https://github.com/rust-lang/rust-analyzer/discussions/19423)
- Biome v2 advertises monorepo support (nested configuration) from a single tool process; no memory or latency figures published — [Biome v2 announcement](https://biomejs.dev/blog/biome-v2/)
- The inotify watch limit is per user, not per process (snippet), so watches are a shared budget across all server processes — [Watchexec: Linux inotify limits](https://watchexec.github.io/docs/inotify-limits.html)

### Inferences
- With one process per docs project, anything loaded per process is multiplied by the project count: the resolved project snapshot, any OpenAPI model, prose-lint dictionaries and compiled rule sets, and the file-watch set. The rust-analyzer case shows duplication of shared inputs (the same dependency graph loaded N times) is what turns an acceptable per-process figure into an OOM. For Ascribe the analogous shared inputs would be a common OpenAPI spec, shared style packages, and a shared source tree scanned for snippets.
- The remedies requested of the rust-analyzer integration (deduplicate by logical project, cap instances, reclaim idle ones) are the standard mitigations short of moving to one multiplexed server.

### Gaps
- No measurement was found of start-up time or baseline RSS for lightweight native language servers (Ruff server, Biome, rumdl, Harper, marksman) per process. Ascribe's own figures would have to be measured.
- No source compares one-server-per-folder against a multiplexed server quantitatively.

## 10. WebAssembly: size and performance of Rust tools in the browser

### Takeaway
Rust tools compiled to WebAssembly ship as large payloads: about 11 MB (Ruff), about 47 MB (Biome) and about 25 to 28 MB (typst compiler) unpacked. A bare Markdown parser can be tiny (markdown-wasm, 0.17 MB unpacked, C-based). No size or speed figure for comrak or pulldown-cmark compiled to WebAssembly was found.

### Cited Findings
- `@astral-sh/ruff-wasm-web` 0.16.10: 10.98 MB unpacked, 6 files (measured 2026-10-03) — [npm registry: @astral-sh/ruff-wasm-web](https://registry.npmjs.org/@astral-sh/ruff-wasm-web/latest)
- `@biomejs/wasm-web` 2.5.15: 47.33 MB unpacked, 4 files; `@biomejs/wasm-bundler` 47.32 MB (measured 2026-10-03) — [npm registry: @biomejs/wasm-web](https://registry.npmjs.org/@biomejs/wasm-web/latest)
- `harper.js` 2.10.0: 75.19 MB unpacked across 17 files (measured 2026-10-03). The package may bundle more than one build variant; the size of the single `.wasm` file was not checked — [npm registry: harper.js](https://registry.npmjs.org/harper.js/latest)
- `@myriaddreamin/typst-ts-web-compiler` 0.7.0: 28.4 MB unpacked; `@myriaddreamin/typst-ts-renderer` 0.7.0: 1.09 MB unpacked (measured 2026-10-03) — [npm registry: typst-ts-web-compiler](https://registry.npmjs.org/@myriaddreamin/typst-ts-web-compiler/latest), [npm registry: typst-ts-renderer](https://registry.npmjs.org/@myriaddreamin/typst-ts-renderer/latest)
- Third party: one web app got its typst WASM compiler to about 24.6 MB uncompressed and enforces a 25,000,000-byte build limit (snippet) — [typsmthng PR #18](https://github.com/aaditagrawal/typsmthng/pull/18)
- Third party: a Go wrapper around typst WASM allocates about 48 MB of linear memory per compile call because each call instantiates a fresh module (snippet) — [typst-go-wasm](https://pkg.go.dev/github.com/varunbpatil/typst-go-wasm)
- `markdown-wasm` 1.2.0 (C md4c compiled to WebAssembly, not Rust): 0.17 MB unpacked; for comparison `markdown-it` 15.0.2 (JavaScript) is 1.97 MB unpacked (measured 2026-10-03) — [npm registry: markdown-wasm](https://registry.npmjs.org/markdown-wasm/latest), [npm registry: markdown-it](https://registry.npmjs.org/markdown-it/latest)
- Native relative speed, third party (snippet, old): pulldown-cmark ran faster than cmark, and comrak at about 1.9x the runtime of cmark — [Rust users forum: comrak release](https://users.rust-lang.org/t/release-comrak-commonmark-gfm-compatible-markdown-parser/10340)
- comrak's repository includes a benchmark target comparing comrak, cmark-gfm, pulldown-cmark and markdown-it.rs; results are not published in the README extract (snippet) — [comrak repository](https://github.com/kivikakk/comrak)

### Inferences
- The split typst.ts makes (28.4 MB compiler versus 1.09 MB renderer) is the relevant pattern for a browser review surface: ship a small renderer or diff viewer to the browser and keep the full checker on the server or in CI, rather than shipping the whole tool.
- A WebAssembly build of Ascribe would carry whatever is linked in. The Ruff-to-Biome spread (11 MB to 47 MB) suggests feature breadth, not the language, sets the size; adding prose-lint dictionaries, an OpenAPI stack and rule regexes pushes toward the upper end. An HTTP client for link checking would not be usable as-is in a browser sandbox in any case.
- Unpacked npm size is not transfer size; these `.wasm` files compress, and compressed sizes were not measured.

### Gaps
- No measured size or throughput for comrak or pulldown-cmark compiled to WebAssembly was found.
- No published performance comparison of the Ruff or Biome playgrounds against their native builds was found.
- Compressed (gzip/brotli) transfer sizes for the packages above were not measured.

## 11. Binary size and distribution

### Takeaway
Comparable single-binary tools download at about 7 to 17 MB compressed for Linux x64; Biome is the outlier at about 67 MB uncompressed per platform package. No source attributes size to specific dependency classes (TLS, regex, OpenAPI), so the effect of adding those to Ascribe has to be measured.

### Cited Findings
All sizes below are for Linux x86-64 release assets or npm platform packages, read on 2026-10-03.

- Ruff 0.16.10 (released 2026-10-01): `ruff-x86_64-unknown-linux-gnu.tar.gz` 10.0 MB; musl 10.35 MB (compressed archives) — [GitHub API: astral-sh/ruff latest release](https://api.github.com/repos/astral-sh/ruff/releases/latest)
- Biome 2.5.15 (released 2026-09-30): `biome-linux-x64` 66.66 MB; `biome-darwin-x64` 61.28 MB; `biome-win32-x64.exe` 79.01 MB (uncompressed executables). npm `@biomejs/cli-linux-x64` 66.66 MB unpacked — [GitHub API: biomejs/biome latest release](https://api.github.com/repos/biomejs/biome/releases/latest), [npm registry: @biomejs/cli-linux-x64](https://registry.npmjs.org/@biomejs/cli-linux-x64/latest)
- lychee v0.24.2 (released 2026-05-01; includes an HTTP client with TLS): 7.63 MB gnu, 7.71 MB musl (compressed) — [GitHub API: lycheeverse/lychee latest release](https://api.github.com/repos/lycheeverse/lychee/releases/latest)
- Vale v3.24.0 (Go; released 2026-10-01): `vale_3.24.0_Linux_64-bit.tar.gz` 13.05 MB (compressed) — [GitHub API: vale-cli/vale latest release](https://api.github.com/repos/vale-cli/vale/releases/latest)
- vale-ls v0.6.0 (released 2026-10-01): 4.1 MB zip — [GitHub API: vale-cli/vale-ls latest release](https://api.github.com/repos/vale-cli/vale-ls/releases/latest)
- Harper v2.12.0 (released 2026-10-01): `harper-ls` 14.45 MB, `harper-cli` 13.24 MB (compressed); VS Code extension package for linux-x64 14.83 MB — [GitHub API: Automattic/harper latest release](https://api.github.com/repos/Automattic/harper/releases/latest)
- rumdl v0.2.78 (released 2026-09-29): 7.09 MB compressed; npm `@rumdl/cli-linux-x64` 16.9 MB unpacked; the `rumdl` meta-package is 0.01 MB — [GitHub API: rvben/rumdl latest release](https://api.github.com/repos/rvben/rumdl/releases/latest), [npm registry: @rumdl/cli-linux-x64](https://registry.npmjs.org/@rumdl/cli-linux-x64/latest)
- typos v1.50.3: 8.21 MB compressed (musl) — [GitHub API: crate-ci/typos latest release](https://api.github.com/repos/crate-ci/typos/releases/latest)
- difftastic 0.71.0 (bundles many tree-sitter grammars): 11.68 MB compressed — [GitHub API: Wilfred/difftastic latest release](https://api.github.com/repos/Wilfred/difftastic/releases/latest)
- mdBook v0.5.4: 4.82 MB compressed; Zola v0.23.6: 16.48 MB compressed — [GitHub API: rust-lang/mdBook latest release](https://api.github.com/repos/rust-lang/mdBook/releases/latest), [GitHub API: getzola/zola latest release](https://api.github.com/repos/getzola/zola/releases/latest)
- oasdiff v1.33.0 (Go): 6.82 MB compressed — [GitHub API: oasdiff/oasdiff latest release](https://api.github.com/repos/oasdiff/oasdiff/releases/latest)
- Other npm platform packages for scale: `@esbuild/linux-x64` 0.28.2 is 11.43 MB unpacked; `@rollup/rollup-linux-x64-gnu` 4.64.0 is 2.15 MB; `@oxlint/linux-x64-gnu` 1.43.0 is 12.7 MB (this scoped package's version lags the `oxlint` package at 1.86.0, so it may be a superseded name) — [npm registry: @esbuild/linux-x64](https://registry.npmjs.org/@esbuild/linux-x64/latest), [npm registry: @rollup/rollup-linux-x64-gnu](https://registry.npmjs.org/@rollup/rollup-linux-x64-gnu/latest), [npm registry: @oxlint/linux-x64-gnu](https://registry.npmjs.org/@oxlint/linux-x64-gnu/latest)
- oxc (own author): states the swc package is 37 MB and that oxc's transformer package is 35 MB smaller — [Oxc benchmarks](https://oxc.rs/docs/guide/benchmarks)

### Inferences
- For rumdl, the only tool here with both figures, 7.09 MB compressed corresponds to 16.9 MB unpacked on npm, about 2.4x. Applying that ratio to other tools would be an estimate and is not done here.
- lychee demonstrates that an async HTTP client with TLS fits in a 7.6 MB compressed binary; Harper demonstrates that an embedded English dictionary and rule set lands at about 13 to 14 MB compressed. These are whole-tool sizes, not marginal costs.
- npm distribution cost is per platform: each supported OS/architecture gets its own platform package of the full binary size, so any size increase is paid once per target in the release pipeline, though each user downloads only one.

### Gaps
- No source quantifies the marginal binary-size cost of adding an OpenAPI stack, a regex-heavy linter or a TLS client to an existing Rust binary.
- Uncompressed sizes of the Ruff, lychee, Vale and Harper executables were not measured (only compressed archives).
- The current size of the Ascribe binary was not part of this research.

## 12. CI cost

### Takeaway
From 1 January 2026 a standard GitHub-hosted Linux 2-core runner costs $0.006 per minute, with 2,000 (Free) to 50,000 (Enterprise Cloud) included minutes per month for private repositories; standard runners are free for public repositories. Jobs are billed in whole minutes, so short docs checks are dominated by rounding and set-up rather than by the tool's run time.

### Cited Findings
- Per-minute rates, standard GitHub-hosted runners (own author): Linux 1-core x64 $0.002; Linux 2-core x64 $0.006; Linux 2-core arm64 $0.005; Windows 2-core $0.010; macOS 3-4 core $0.062. Larger runners: Linux 4-core to 96-core $0.012 to $0.252; macOS 12-core $0.077; macOS M2 Pro 5-core $0.102 — [GitHub Docs: Actions runner pricing](https://docs.github.com/en/billing/reference/actions-runner-pricing)
- Rounding (own author): "GitHub rounds the minutes and partial minutes each job uses up to the nearest whole minute"; "included minutes cannot be used for larger runners"; larger runners "are not free for public repositories" — [GitHub Docs: Actions runner pricing](https://docs.github.com/en/billing/reference/actions-runner-pricing)
- Included allowances for private repositories (own author): Free 2,000 minutes and 500 MB artifact storage; Pro 3,000 minutes and 1 GB; Team 3,000 minutes and 2 GB; Enterprise Cloud 50,000 minutes and 50 GB; 10 GB cache storage on each. "The use of standard GitHub-hosted runners is free in public repositories." — [GitHub Docs: GitHub Actions billing](https://docs.github.com/en/billing/concepts/product-billing/github-actions)
- 2026 changes (third-party reporting, snippets): hosted-runner prices were cut by up to 39% effective 1 January 2026; a $0.002-per-minute platform charge for self-hosted runners announced 16 December 2025 for 1 March 2026 was postponed within 48 hours and, as of September 2026 reporting, had not taken effect — [bex.co: GitHub shelved its self-hosted runner charge](https://bex.co/blog/2026/09/19/github-self-hosted-runner-charge-reversal), [SAMexpert: GitHub Actions pricing backlash](https://samexpert.com/github-actions-pricing-backlash-2026/)
- Docs CI job durations reported in sources above: OWASP link checks 10m08s and 13m53s — [OWASP CheatSheetSeries issue #2488](https://github.com/OWASP/CheatSheetSeries/issues/2488); Vale on GitLab docs under 20 s for the lint itself — [vale.sh](https://vale.sh/); a Rust project's doctest CI step 178 s before and 58 s after merged doctests (snippet) — [openquant PR #200](https://github.com/Open-Quant/openquant/pull/200); OpenStack release-notes Sphinx job 52m to 1h before and 15m after parallelising (snippet) — [zuul-jobs commit](https://opendev.org/zuul/zuul-jobs/commit/b892aad548c1feeba3452a378dbc638231236d27)
- Hosted-runner resource limits bite on heavy steps: the Barbacane OpenAPI job was "reclaimed" (exit 143) at 5.67 GB peak, and Docusaurus users report Actions failing without added swap — [barbacane issue #221](https://github.com/barbacane-dev/barbacane/issues/221), [Docusaurus discussion #3132](https://github.com/facebook/docusaurus/discussions/3132)

### Inferences
- At $0.006 per minute with whole-minute rounding, a docs check that takes 5 seconds and one that takes 55 seconds cost the same $0.006 per job on a private repository. Local checks (prose lint, snippet extraction, internal links, OpenAPI parsing if memory-safe) are effectively free in CI minutes; the capabilities that move the bill are those that add minutes: external link checking (10 to 14 minutes in the OWASP case, about $0.06 to $0.08 per run at the Linux 2-core rate) and compiling or running code samples.
- Because most open-source docs repositories are public and standard runners are free there, wall-clock PR latency and flakiness are likely to matter more to Ascribe's users than dollars.
- Memory is a harder CI limit than time for the candidate capabilities: the observed failures on hosted runners were OOM or reclaim, not time-outs.

### Gaps
- No survey of "typical" docs CI job duration was found; the figures above are individual cases.
- The self-hosted charge status is from third-party reporting, not a GitHub changelog entry read directly.
- The memory available on standard GitHub-hosted runners in 2026 was not retrieved.

## Cross-cutting summary by candidate capability

### Takeaway
On the evidence found, the six capabilities fall into three cost classes: cheap and local (prose linting, snippet-marker extraction, drift checks against local files), cheap in time but memory-hazardous (OpenAPI processing), and inherently slow or unbounded (external link checking, sample execution, tree-based rendered diffing). Only the first class has precedent for running in the edit loop.

### Cited Findings
- Prose linting in the edit loop has precedent with a 300 ms debounce (vale-ls) and a claimed 10 ms per suggestion pass (Harper, own author, no methodology) — [Vale LSP guide](https://docs.vale.sh/guides/lsp), [Harper COMPARISON.md](https://github.com/Automattic/harper/blob/master/COMPARISON.md)
- Source scanning at 0.082 s over the Linux kernel tree (own author, i9-12900K) — [ripgrep README](https://github.com/BurntSushi/ripgrep)
- OpenAPI: 5.67 GB and about 155 s on a 7.7 MB spec in a Rust tool when references are expanded without sharing — [barbacane issue #221](https://github.com/barbacane-dev/barbacane/issues/221)
- External links: 10 to 14 minutes for about 2,400 links with flaky results — [OWASP CheatSheetSeries issue #2488](https://github.com/OWASP/CheatSheetSeries/issues/2488)
- Structural diff: quadratic graph, 3,000,000-vertex and 1,000,000-byte caps with line-diff fallback (snippet for the defaults) — [difft man page](https://github.com/Wilfred/difftastic/blob/master/difft.1.md)
- Sample compilation: 61.85 s for 69 Rust doctests before batching, 6.29 s after (snippet) — [TeNeT PR #1911](https://github.com/Ryo-wtnb11/TeNeT/pull/1911)

### Inferences
- Edit-loop candidates: prose linting (in-process), internal link and anchor checks, snippet-marker resolution against an index, drift checks against already-indexed local files.
- On-save or CI candidates: OpenAPI parsing and page generation (memoised, references shared, never per keystroke), spec diffing.
- CI-only or scheduled candidates: external link checking (scheduled, cached, changed-links-only on PRs), compiling or running samples (batched), rendered diffing between two builds (bounded, with fallback).
- Per-process multiplication applies to everything loaded into a language server; anything shared across docs projects in a workspace (spec, styles, source index, file watches) is paid once per server under the current one-process-per-project design.

### Gaps
- No tool was found that publishes costs for this combination of capabilities in one binary, so interaction effects (for example OpenAPI-generated pages feeding prose linting and link checking) are unmeasured.
- "Drift checking" has no published benchmark in any comparable tool; its cost here is inferred from scanning and watching figures only.
