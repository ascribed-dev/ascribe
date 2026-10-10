# Documentation versioning: practice at named organizations and projects (as of 9 October 2026)

Method note. "Observed 2026-10-09" means I fetched the live page that day and read its HTML (canonical link, robots meta tag, banner text, version list). Those observations are primary but are a single sample of one page per site. Everything else is from the cited page as fetched or from search-result summaries; the latter are marked "(search summary, not opened)".

## 1. How common is each need: current only; current plus one or two; many maintained versions; long-lived archives? Any survey data?

### Takeaway
No survey measures this: the State of Docs reports (2025, 2026) publish no versioning figure, and I found no Write the Docs data. Among the named projects the pattern is clear and repeated: a small set of maintained versions (typically 2 to 5, tied to the product's support window) plus a much longer frozen archive that nobody edits; SaaS products and several tool vendors keep one version only, and Docusaurus's own guidance says most sites should not version.

### Cited Findings

Versions kept live versus archived, by project:

| Project | Live / maintained docs versions | Archive | Source |
|---|---|---|---|
| Kubernetes | Current + 4 previous (v1.37 at kubernetes.io; v1.36 to v1.33 at `v1-NN.docs.kubernetes.io`) | Older subdomains; each is "a static snapshot" | [Supported doc versions](https://kubernetes.io/docs/home/supported-doc-versions/) |
| Django | Docs fixes go to `main` and "if easily backported" the latest stable branch only | Version switcher lists 18 versions, 1.8 to 6.1 (observed 2026-10-09) | [Release process](https://docs.djangoproject.com/en/dev/internals/release-process/); [docs home](https://docs.djangoproject.com/en/6.1/) |
| Python | 5 supported branches (3.15, 3.14 bugfix; 3.13, 3.12, 3.11 security) + `main` | 13 end-of-life branches listed (2.6, 2.7, 3.0 to 3.10); their docs stay online | [Devguide: versions](https://devguide.python.org/versions/) |
| PostgreSQL | 5 supported majors (18, 17, 16, 15, 14) + 19 beta + devel | 24 unsupported versions in the manuals archive, 6.3 to 13 (observed 2026-10-09) | [Docs index](https://www.postgresql.org/docs/); [archive](https://www.postgresql.org/docs/manuals/archive/); [versioning policy](https://www.postgresql.org/support/versioning/) |
| Node.js | API docs per release line | `nodejs.org/docs/` lists `latest-v4.x` through `latest-v26.x`; individual releases back to v0.10.48 still served (observed 2026-10-09) | [nodejs.org/docs](https://nodejs.org/docs/); [v0.10.48 fs page](https://nodejs.org/docs/v0.10.48/api/fs.html) |
| Rust | Std docs per release | `doc.rust-lang.org/1.0.0/std/` and `/1.80.0/std/` both return 200 (observed 2026-10-09) | [1.0.0 std](https://doc.rust-lang.org/1.0.0/std/); [1.80.0 std](https://doc.rust-lang.org/1.80.0/std/) |
| Go | Each major release supported until two newer majors exist | Release history page | [Go release history](https://go.dev/doc/devel/release) (search summary, not opened) |
| GitHub Docs (Enterprise Server) | 4 supported GHES releases, from one source | "Frozen" snapshot of each deprecated release kept "in perpetuity", not linked from the site | [Versioning documentation](https://docs.github.com/en/contributing/writing-for-github-docs/versioning-documentation) |
| GitLab | Supported versions online at archives.docs.gitlab.com; default at docs.gitlab.com | 106 versions (19.4 down to 10.3) as Docker images for offline or self-hosted browsing | [GitLab Docs archives](https://docs.gitlab.com/archives/) |
| Elastic | One cumulative set for all of 9.x (since April 2025) | 8.19 and earlier remain at elastic.co/guide in the old AsciiDoc system | [Versioning and availability](https://www.elastic.co/docs/get-started/versioning-availability) |
| MongoDB | Supported versions + `/docs/upcoming/` | "Legacy documentation" for EOL versions, which MongoDB "does not maintain or update" | [Legacy docs](https://www.mongodb.com/docs/legacy/) |
| Microsoft Learn | Version selector (monikers) per product | `learn.microsoft.com/previous-versions/` directory covering about 25 product families, flagged `is_archived: true` | [Previous versions](https://learn.microsoft.com/en-us/previous-versions/) |
| React | Current at react.dev | 18.react.dev, 17.react.dev, 16.react.dev, 15.react.dev, legacy.reactjs.org (observed 2026-10-09) | [React versions](https://react.dev/versions) |
| Next.js | Current unversioned at `/docs`; `/docs/15`, `/docs/14` | 14 is past end of life but still served with an EOL notice | [Next.js 14 docs](https://nextjs.org/docs/14/getting-started/installation) |
| React Native | 0.60 to 0.73 in the repo in August 2023; 0.67 and below hidden from the dropdown | Proposal to move old ones to immutable deployments | [react-native-website #3819](https://github.com/facebook/react-native-website/issues/3819) |

- Docusaurus guidance: "Most of the time, you don't need versioning as it will just increase your build time"; it is "best suited for websites with high-traffic and rapid changes to documentation between versions"; "try to keep the number of your versions below 10"; old versions can be "archived versions that will never be rebuilt" linked as external immutable URLs. — [Docusaurus: Versioning](https://docusaurus.io/docs/versioning)
- State of Docs 2025 drew on 450+ docs professionals; State of Docs 2026 on 1,100+. The published highlights cover AI use and metrics, not versioning. — [State of Docs 2025 highlights](https://www.gitbook.com/blog/state-of-docs-report-2025); [State of Docs 2026 highlights](https://www.gitbook.com/blog/state-of-docs-2026) (both search summaries, not opened)
- Kubernetes states that doc availability and support are different things: "The availability of documentation for a Kubernetes version is separate from whether that release is currently supported." — [Supported doc versions](https://kubernetes.io/docs/home/supported-doc-versions/)
- PostgreSQL supports a major version "for 5 years after its initial release"; version 14 reaches end of life on 12 November 2026. — [Versioning policy](https://www.postgresql.org/support/versioning/)
- Python: security-only after two years, "Five years after a release, support ends. The release cycle is frozen; no further changes are allowed." 3.10 reached end of life on 2026-10-01. — [Devguide: versions](https://devguide.python.org/versions/)
- Django LTS releases get security and data-loss fixes "typically three years". — [Release process](https://docs.djangoproject.com/en/dev/internals/release-process/)
- Stripe versions the API by date-plus-name (the docs' samples use `2026-09-30.endive`) and publishes one docs set with a changelog and an upgrade guide, not a docs site per version. — [Upgrade your integration](https://docs.stripe.com/upgrades)
- A small project that added versioned docs gave the common reason for needing a second version: the site "has always published a single version built from docs/ on main", so readers on a released version had no matching docs. — [zalando-incubator/agentic-identity-broker PR #64](https://github.com/zalando-incubator/agentic-identity-broker/pull/64) (search summary, not opened)
- Atlassian publishes Server / Data Center docs as one Confluence space per product version (URL pattern `confluence.atlassian.com/bitbucketserver078/`, `bitbucketserver088/`, `bitbucketserver0816/`, `bitbucketserver101/`). — [Bitbucket 7.8 release notes](https://confluence.atlassian.com/bitbucketserver078/bitbucket-server-release-notes-1037995044.html); [Bitbucket 10.1 releases](https://confluence.atlassian.com/bitbucketserver101/releases-1721010328.html) (search summary, not opened)
- HashiCorp's unified docs repository keeps versioned products in one folder per version; a fix that applies to several releases is made in each version folder separately. — [web-unified-docs versioning guide (third-party mirror)](https://www.mintlify.com/hashicorp/web-unified-docs/content-guides/versioning) (search summary, not opened; mirror, not the primary repository)

### Inferences
- Three clusters appear, matching the three tiers under design: (a) one version only (SaaS and API products such as Stripe; most small projects per Docusaurus's advice); (b) a short maintained window of 2 to 5 versions that follows the product's support policy (Kubernetes 5, PostgreSQL 5, Python 5, GitHub 4, Django effectively 2 for docs fixes); (c) a frozen archive that is far longer than the maintained window (PostgreSQL 24 archived versus 5 live; Python 13 versus 5; GitLab 106 images; Django 18 listed versus 1 to 2 receiving fixes).
- "Many maintained versions" in the sense of many versions actively edited is rare. Even projects that show many versions edit few of them; the rest are snapshots.
- The long archive is a need mainly of installed software with slow upgraders (databases, languages, self-managed enterprise servers). It costs little because it is static.

### Gaps
- No survey figure on the share of docs teams that version. Neither State of Docs report's highlights mention one; I did not open the full reports, so a figure inside them cannot be ruled out.
- No Write the Docs survey or talk with numbers was found.
- AWS, Cloudflare, Twilio, Grafana's count of live versions, Red Hat, and Vue's current layout were not confirmed at primary sources in this pass. Atlassian and HashiCorp rest on search summaries only.
- Go's doc-versioning layout (pkg.go.dev per module version) was not confirmed.

## 2. After a major rewrite, what happens to the old docs? How often is it "freeze the old site" rather than conditional content?

### Takeaway
For frontend frameworks and for Elastic's change of docs system, the answer is uniformly "freeze the old site on its own hostname and put a notice on it"; I found no case of a major rewrite being handled with conditional content in one source. Conditional single-source content is used for something different: many minor versions of a product whose docs change little between them (GitHub Enterprise Server, Elastic 9.x, Microsoft Learn monikers).

### Cited Findings
- React: the new site launched 16 March 2023; the old docs were archived at legacy.reactjs.org, old links redirect, and legacy pages say "This site is no longer updated." Canonical on a legacy page points to itself, not react.dev (observed 2026-10-09). — [Introducing react.dev (fr)](https://fr.react.dev/blog/2023/03/16/introducing-react-dev) (search summary; date taken from the French translation); [legacy page](https://legacy.reactjs.org/docs/hooks-intro.html)
- React keeps one frozen site per old major: 18.react.dev, 17.react.dev, 16.react.dev, 15.react.dev (observed 2026-10-09). — [React versions](https://react.dev/versions)
- Angular: the old domain is frozen as `v17.angular.io` with the banner "This site is no longer updated... This is the archived documentation for Angular v17"; its canonical link points to the angular.dev home page, not to the equivalent page. `v18.angular.dev/overview` exists and its canonical points to `angular.dev/overview` (observed 2026-10-09). — [v17.angular.io page](https://v17.angular.io/guide/component-overview); [v18.angular.dev](https://v18.angular.dev/overview)
- Vue: Vue 2 reached end of life on 31 December 2023; v2.vuejs.org stays up with "Vue 2 has reached EOL and is no longer actively maintained." (observed 2026-10-09). The v2 docs live in their own repository, separate from the v3 docs repository. — [Vue 2 EOL](https://v2.vuejs.org/eol/); [v2 guide](https://v2.vuejs.org/v2/guide/); [vuejs/v2.vuejs.org](https://github.com/vuejs/v2.vuejs.org/pull/2957/files) (search summary)
- Tailwind: `v3.tailwindcss.com` and `v2.tailwindcss.com` both return 200; the v3 site carries a "v4.0 is here" link (observed 2026-10-09). — [v3 docs](https://v3.tailwindcss.com/docs/installation); [v2 docs](https://v2.tailwindcss.com/docs)
- Svelte: the Svelte 5 docs point readers who want Svelte 3/4 syntax to `v4.svelte.dev`, which returns 200 (observed 2026-10-09). — [Svelte legacy overview](https://svelte.dev/docs/svelte/legacy-overview); [v4.svelte.dev](https://v4.svelte.dev/docs/introduction)
- Next.js differs: majors are path prefixes on the same site (`/docs/14`, `/docs/15`), the 14 pages say "Next.js 14 is outside of our support policy", and the canonical link on a v14 page points to the unversioned URL (observed 2026-10-09). — [Next.js 14 docs](https://nextjs.org/docs/14/getting-started/installation)
- Elastic split at the change of docs system: the Markdown system (April 2025, with 9.0) holds 9.x and later; "Documentation for Elastic Stack 8.19.0 and earlier is available at elastic.co/guide" in the old AsciiDoc system. A 7.17 page there carries `noindex,nofollow` and a "no longer updated" notice (observed 2026-10-09). — [Versioning and availability](https://www.elastic.co/docs/get-started/versioning-availability); [7.17 reference](https://www.elastic.co/guide/en/elasticsearch/reference/7.17/index.html); [elastic/docs](https://github.com/elastic/docs) (search summary)
- Microsoft: Azure DevOps moved content for old versions (TFS 2013 onward), deprecated cloud content, legacy features and 2012 to 2017 release notes to a separate archive site; completed April 2022, announced 12 July 2022; reached by a "Previous versions" link in the version selector. — [Azure DevOps blog](https://devblogs.microsoft.com/devops/content-archived-for-azure-devops-previous-versions/)
- Conditional single source for minors: GitHub Docs versions are "single-source to avoid repetition", using `{% ifversion %}` and feature flags in `data/features/`. — [Versioning documentation](https://docs.github.com/en/contributing/writing-for-github-docs/versioning-documentation)
- Conditional single source for minors: Elastic 9.x, "the same page stays valid over time and shows version-related evolutions", with mandatory page-level `applies_to`. — [Write cumulative documentation](https://www.elastic.co/docs/contribute-docs/how-to/cumulative-docs)

### Inferences
- Of the six frontend projects checked, five freeze the old major on a separate hostname (React, Angular, Vue, Tailwind, Svelte) and one uses a path prefix on the same site (Next.js). None merges two majors into conditional pages.
- Elastic and GitHub both pair the two mechanisms: conditionals inside a major or a support window, and a frozen snapshot at the boundary. This is the strongest evidence that the two are complements, not alternatives.
- A frozen old major usually coincides with a new site design, a new domain, or a new docs toolchain, so the old site is frozen with its old generator. A toolchain's "archive" tier therefore has to accept output it did not build.

### Gaps
- No count across a broad sample; the claim rests on the named projects.
- Whether v4.svelte.dev shows a notice was not established (my pattern search found none, which does not prove absence).
- Whether Tailwind and Svelte set canonical links to the new site was not detected in the pages fetched; treat as unknown.

## 3. What triggers retiring a docs version, who does it, and what work does it involve?

### Takeaway
The trigger is almost always the product's end-of-support date, not a docs decision, and the work is: snapshot the built site, add a banner, drop the version from the switcher, delete its conditionals from the source, add redirects, and tell search engines. GitHub's runbook is the most detailed public account: 16 steps, a 20 to 30 minute scrape, human sign-off gates, and by 2026 a coding agent driving the steps.

### Cited Findings
- GitHub's GHES deprecation runbook, in order: confirm the date with the release controller; create a repository `github/docs-ghes-X` for the archive; scrape the rendered site (the full scrape "takes 20-30 minutes") and publish it via GitHub Pages; move the version from `supported` to `deprecatedWithFunctionalRedirects`; run codemods over pipelines, content and data to strip conditionals; hand-clean leftovers (blank lines, table pipes, empty `else` branches, and `{% ifversion ghes < <new-oldest> %}` blocks "which are always false after the deprecation"); add redirects by hand where a feature is gone on every version; fix tests; tag `enterprise-X-deprecation` so the archive can be re-scraped later; mark the OpenAPI description deprecated; purge the Fastly cache. — [deprecation-steps.md](https://github.com/github/docs/blob/main/src/ghes-releases/lib/deprecation-steps.md)
- Who at GitHub: a coding agent runs scripts and drafts pull requests (labelled `llm-generated`); a human approves at marked gates, reviews the diff and deploys; the content team reviews the pull request. The codemod can "rewrite the whole frontmatter" of changed files, so the diff needs review. — [deprecation-steps.md](https://github.com/github/docs/blob/main/src/ghes-releases/lib/deprecation-steps.md)
- GitHub keeps deprecated releases as a "frozen" snapshot "in perpetuity"; they are not linked from the site but remain reachable by URL. — [Versioning documentation](https://docs.github.com/en/contributing/writing-for-github-docs/versioning-documentation)
- Python: the docs server rebuilds feature and bugfix branches by cron, security branches less often, and end-of-life branches not at all; "Manual rebuilds are needed to add the end-of-life banner for newly end-of-life branches." Each branch pins its own Sphinx version (for example 2.3.1 for 3.7, 3.4.3 for 3.10, 8.x for 3.12 and later). — [python/docsbuild-scripts](https://github.com/python/docsbuild-scripts)
- Python deindexes by robots.txt: it disallows `/2.0/` through `/2.7/` and `/3.0/` through `/3.10/`, that is every end-of-life version including 3.10, which went end of life eight days before the check (observed 2026-10-09). — [docs.python.org/robots.txt](https://docs.python.org/robots.txt)
- PostgreSQL: an unsupported version's page says "This documentation is for an unsupported version of PostgreSQL.", carries `<meta name="robots" content="nofollow">`, and its canonical link points to `/docs/current/<same page>` (observed 2026-10-09 on version 13). robots.txt disallows only `/docs/devel/`. — [PG 13 SELECT page](https://www.postgresql.org/docs/13/sql-select.html); [robots.txt](https://www.postgresql.org/robots.txt)
- MongoDB: an EOL version's page carries `noindex, nosnippet` and a notice that the version is "no longer supported" with an upgrade link (observed 2026-10-09 on v5.0). Policy: "MongoDB does not maintain or update legacy documentation." — [v5.0 page](https://www.mongodb.com/docs/v5.0/tutorial/getting-started/); [Legacy docs](https://www.mongodb.com/docs/legacy/)
- Kubernetes: a version leaving `main` becomes a `release-1.NN` branch served at its own subdomain with the banner "no longer actively maintained. The version you are currently viewing is a static snapshot." The repository has `release-1.4` through `release-1.36` (33 branches) and `dev-1.38` for the next release (observed 2026-10-09). The v1.33 page is `index, follow`. — [v1.33 pods page](https://v1-33.docs.kubernetes.io/docs/concepts/workloads/pods/); [kubernetes/website branches](https://github.com/kubernetes/website/branches)
- Kubernetes release docs are run by a per-release docs lead on the release team; feature docs go to `dev-1.NN` as placeholder pull requests, and "If the feature does need documentation but the PR is not created, the feature may be removed from the milestone." The release calendar has a Docs Freeze (week 12 in the 1.30 cycle). — [Documenting a feature for a release](https://kubernetes.io/docs/contribute/new-content/new-features/); [sig-release 1.30](https://git.k8s.io/sig-release/releases/release-1.30/README.md) (the calendar is a search summary)
- Elastic's rule inside the cumulative set: "No information should be removed for supported product versions, unless it was never accurate." — [Write cumulative documentation](https://www.elastic.co/docs/contribute-docs/how-to/cumulative-docs)
- Docker historically kept an archive (v17.06, v18.03, v18.09 at `docs.docker.com/vNN.NN/`, plus Docker Hub images to run old docs offline, with unsupported versions reachable only by branch or image tag). Today `https://docs.docker.com/v18.09/` redirects to the docs home page (observed 2026-10-09). — [docsarchive.md (fork of the old repo)](https://github.com/docker/docker.github.io-1/blob/master/docsarchive.md) (search summary); [docs.docker.com/v18.09/](https://docs.docker.com/v18.09/)
- OpenChoreo prunes to the latest four versions "to cut repo size and build times"; removal needed a manual config edit the script could not do. — [openchoreo.github.io PR #887](https://github.com/openchoreo/openchoreo.github.io/pull/887) (search summary, not opened)
- React Native's archiving proposal (11 August 2023) defines archiving as storing a version "permanently to a standalone immutable deployment"; steps: build from current `main`, deploy static output, get a permalink, add it to the versions page, delete the version from `versions.json` and the versioned folders. The issue closed on 12 June 2024 as stale with no recorded decision; one objection was that a Netlify deploy URL looks temporary for a permanent archive. — [react-native-website #3819](https://github.com/facebook/react-native-website/issues/3819)

### Inferences
- Retirement has two separable parts that different tools handle badly together: producing an immutable copy of the output (cheap), and removing the version from the living source (expensive when the source is conditional). GitHub's runbook is long mainly because of the second part.
- A conditional model needs a retirement command that deletes always-false and always-true branches; GitHub built codemods for it and still cleans up by hand.
- Search-engine treatment at retirement is inconsistent across projects: robots.txt disallow (Python), `noindex` (MongoDB, Elastic), canonical to current (PostgreSQL, Django, Node.js), or nothing (Kubernetes snapshot is `index, follow`).
- An archive must be re-buildable or patchable in rare cases (GitHub keeps a tag for re-scraping; Python must do a one-off rebuild to add the banner). A banner injected at serve time or by an overlay would avoid rebuilding with an old toolchain.

### Gaps
- Why Docker removed its versioned archive, and when, is not documented in any source I found; a Docker forum thread asking for the criteria has no visible answer ([forum thread](https://forums.docker.com/t/whats-the-criteria-to-remove-docker-versions-from-the-archive-list/43835), search summary).
- Kubernetes's written steps for cutting a release branch and subdomain were not found. Subdomains older than v1.33 (v1-28, v1-20, v1-10) did not respond from my machine on 2026-10-09; I could not tell whether they are retired or the request failed.
- Microsoft Learn's rules for retiring a moniker were not found at a primary source.

## 4. How are fixes backported across maintained versions, with what tooling, and at what cost?

### Takeaway
Branch-per-version projects backport with a label plus a bot that opens a cherry-pick pull request per target branch, and limit docs backports by policy to one or two branches; frozen versions get none. The repeated complaint is duplication and drift, which is the reason Elastic gave for abandoning a docs set per minor version.

### Cited Findings
- Django: "Documentation fixes generally will be more freely backported to the last release branch", because up-to-date docs matter and the regression risk is low; in practice "main, and, if easily backported, to the latest stable branch". — [Release process](https://docs.djangoproject.com/en/dev/internals/release-process/)
- Grafana: add a `backport <BRANCH>` label; the bot (Grot) opens a pull request per label after merge and, when it cannot, comments with manual instructions. Policy: typos and copy edits go to the latest version only; incorrect information goes to all supported versions. Branches are named `v9.0.x`. — [Grafana Writers' Toolkit: backport changes](https://grafana.com/docs/writers-toolkit/review/backport-changes/)
- Python uses a `cherry-picker` tool and backport labels for maintenance branches. — [python/cherry-picker](https://github.com/python/cherry-picker) (search summary, not opened)
- TYPO3 docs: labels trigger one backport pull request per version, only if the cherry-pick applies cleanly; otherwise a `backport-failed` label is set and the cherry-pick is manual. — [TYPO3: backport changes](https://docs.typo3.org/m/typo3/docs-how-to-document/main/en-us/Maintainers/BackportChanges.html#backport-changes) (search summary, not opened)
- Elastic's stated reasons for cumulative docs: a "single 'source of truth'" for each feature, and it "avoids 'drift' between multiple similar sets of documentation"; benefits listed for readers are fewer duplicate pages, a feature's full history in one place, and simpler search and navigation. Under the old system "we build a separate copy of each book for each of those branches". — [Write cumulative documentation](https://www.elastic.co/docs/contribute-docs/how-to/cumulative-docs); [Versioning and availability](https://www.elastic.co/docs/get-started/versioning-availability); [elastic/docs](https://github.com/elastic/docs) (last one is a search summary)
- GitHub: because content is single-source, there is no backport; but changing an already archived version means checking out the deprecation tag, re-running the scrape for the page, uploading the files and purging the CDN. — [deprecation-steps.md](https://github.com/github/docs/blob/main/src/ghes-releases/lib/deprecation-steps.md)
- HashiCorp's folder-per-version repository: a fix for several releases is edited in each version folder. — [web-unified-docs versioning guide (third-party mirror)](https://www.mintlify.com/hashicorp/web-unified-docs/content-guides/versioning) (search summary, not opened)
- Docusaurus maintainers' position, as summarized: backporting docs between versions is the site owner's job and does not always make sense. — [docusaurus #8373](https://github.com/facebook/docusaurus/issues/8373) (search summary, not opened)
- Apache Iceberg: release docs are copied into a per-version docs branch by hand, and the resulting diff is unreadable. — [apache/iceberg #8151](https://github.com/apache/iceberg/issues/8151) (search summary, not opened)
- Red Hat Quay: docs files carry branch-specific content, so the same conflict recurs on every cherry-pick to a release branch. — [quay/quay #7481](https://github.com/quay/quay/issues/7481) (search summary, not opened)

### Inferences
- The observed backport depth for docs is shallow: latest stable only (Django), or "all supported" only for factual errors (Grafana). Policies explicitly exclude typos and copy edits from older versions.
- A toolchain's opt-in tier of "one or two live versions" matches what teams are willing to backport to. Beyond two, teams either stop editing (archive) or switch to single-source conditionals (GitHub, Elastic).
- The bot pattern (label, automatic cherry-pick pull request, manual fallback on conflict) is the standard tool and lives in the forge, not in the docs generator.

### Gaps
- No team published a measured cost (hours, pull-request counts, conflict rate) for docs backports.
- Elastic has not, in the sources I reached, published results (traffic, contribution rate, defect counts) after the 2025 change, nor a blog post on it; the reasons above are from its contributor docs. The assignment's premise of "published results" is unconfirmed.
- GitLab's backport and version-text policy page required a login when fetched ([development/documentation/versions](https://docs.gitlab.com/development/documentation/versions/)); not confirmed.

## 5. What reader-facing problems are documented?

### Takeaway
The best-documented problem is search engines sending readers to old versions, because old pages have accumulated more inbound links; projects respond with banners, canonical links, robots rules and `noindex`, and at least one canonical-link scheme has itself produced 404s. Cross-version links and switchers that keep the reader on "the same page in another version" exist at PostgreSQL and Elastic; I found fewer first-hand reports of switcher 404s than expected.

### Cited Findings
- Python (2018, issue 35435): old pages rank because they are "linked from many articles and sources"; Julien Palard preferred linking old pages to new ones over hiding them, calling hiding old pages from search engines effectively lying to them. A 2016 suggestion was for 3.x pages to declare `/3/` as canonical. — [bugs.python.org msg331376](https://bugs.python.org/msg331376); [docs list, issue35435](https://mail.python.org/pipermail/docs/2018-December/038471.html) (search summaries, not opened)
- Python canonical 404: an issue reported that the canonical link on 3.11 pages pointed to `/3/...`, which then resolved to 3.9 where the page did not exist, so the canonical returned 404; the proposed fix was for newer editions to omit the canonical link. The search summary dates it 2018, which cannot be right for 3.11; treat the date as unknown. — [python/pythondotorg #1880](https://github.com/python/pythondotorg/issues/1880) (search summary, not opened)
- Python today: every version's page, including 3.5 and 3.9, sets canonical to `https://docs.python.org/3/<page>`; 2.7 sets canonical to `/2/<page>`; EOL pages say the document is "for an old version of Python that is no longer supported" (observed 2026-10-09). — [3.9 os page](https://docs.python.org/3.9/library/os.html); [2.7 os page](https://docs.python.org/2.7/library/os.html)
- OpenStack (March 2020): a user reported that older release docs "always come up first". — [openstack-discuss](https://lists.openstack.org/pipermail/openstack-discuss/2020-March/013338.html) (search summary, not opened)
- Azure DevOps archived old-version content in 2022 to "Improve search results for currently supported versions", reduce the reading time of articles covering many versions, and reduce confusion about which article applies to the installed version. — [Azure DevOps blog](https://devblogs.microsoft.com/devops/content-archived-for-azure-devops-previous-versions/)
- Read the Docs: without canonical URLs "it's easy for outdated documentation to be the top search result"; a warning banner alone does not stop search engines sending readers to the page; hidden versions are added to robots.txt; its built-in banner for versions below stable is off by default. — [RTD canonical URLs](https://github.com/vroncevic/readthedocs.org/blob/master/docs/guides/canonical.rst); [RTD deprecating content](https://github.com/readthedocs/readthedocs.org/blob/f53b93631114650686528f3d3f2afe8b51837aa9/docs/user/guides/deprecating-content.rst) (search summaries of repository copies, not opened)
- PostgreSQL: each page lists the same page in supported and unsupported versions, and canonical points to `/docs/current/<same page>` (observed 2026-10-09). — [PG 16 SELECT page](https://www.postgresql.org/docs/16/sql-select.html)
- Django: every version's page sets canonical to the same page in the current stable (6.1); a 1.8 page says it is for "an insecure version of Django that is no longer supported. Please upgrade to a newer release!" (observed 2026-10-09). — [Django 1.8 queries page](https://docs.djangoproject.com/en/1.8/topics/db/queries/)
- Node.js: old API pages set canonical to the unversioned `nodejs.org/api/<page>`; the v0.10.48 page still declares it with `http://` (observed 2026-10-09). — [latest-v18.x fs](https://nodejs.org/docs/latest-v18.x/api/fs.html); [v0.10.48 fs](https://nodejs.org/docs/v0.10.48/api/fs.html)
- Angular's archived v17 site sets canonical to the angular.dev home page for every page, so the old page's equivalent is not signalled (observed 2026-10-09). — [v17.angular.io page](https://v17.angular.io/guide/component-overview)
- Readers needing an old version that the site no longer shows: a 2017 thread describes having to read raw Markdown on a release branch to get Angular 4.1 docs. — [Ionic forum](https://forum.ionicframework.com/t/how-to-access-old-4-1-x-angular-docs/99779) (search summary, not opened)
- Reader identification of the version: Elastic uses availability badges in page headers, section headers and inline, with a per-page version dropdown; Kubernetes prints "You are viewing documentation for Kubernetes version: v1.33" on snapshots; GitLab added version-specific search to online archives in 15.6 and to offline archives in 16.6. — [Elastic versioning](https://www.elastic.co/docs/get-started/versioning-availability); [Kubernetes v1.33 snapshot](https://v1-33.docs.kubernetes.io/docs/home/supported-doc-versions/); [GitLab archives](https://docs.gitlab.com/archives/)
- Hacker News discussion (2025): one side, "versioned docs = major maintenance headache"; the other, old versions are needed by people who cannot upgrade. — [HN item 44109895](https://news.ycombinator.com/item?id=44109895) (search summary, not opened)

### Inferences
- A canonical link from an old page to "the same path in current" is the most common control (Python, PostgreSQL, Django, Node.js, Next.js), and it fails exactly when the page was renamed or removed: the canonical then points at a 404. A toolchain that knows page identity across versions could emit a correct canonical or none.
- Projects disagree on whether old versions should be findable by search at all: Python blocks crawling of EOL versions, MongoDB and Elastic use `noindex`, PostgreSQL and Django keep them indexed with a canonical, Kubernetes leaves snapshots indexable. There is no settled practice; this should be a setting, with a default.
- Version identity shown on the page itself (banner on old, badge on content) is universal among the projects checked; a switcher alone is not treated as enough.

### Gaps
- No first-hand project report of a version switcher landing on a 404 was found in this pass, though the canonical-404 issue is the same failure in another place.
- No published measurement of how often readers land on the wrong version.
- Broken links between versions: no project-published account found.

## 6. What do AI coding agents and LLM-based search do with versioned docs, and what guidance or evidence exists?

### Takeaway
The published evidence is thin and mostly first-party: agents default to whatever version dominated their training data, and the fix teams are converging on is to hand the agent docs that match the installed version (bundled in the package, or a per-version `llms.txt`). Vercel's Next.js is the only named project found with numbers.

### Cited Findings
- Next.js bundles the docs for the installed version at `node_modules/next/dist/docs/` and generates an `AGENTS.md` whose purpose is to redirect agents from stale training data to those version-matched docs; `create-next-app` writes `AGENTS.md` and `CLAUDE.md` by default (`--agents-md`). Requires 16.2 canary or later; earlier versions download a copy into the project. — [Next.js: AI agents guide](https://nextjs.org/docs/app/guides/ai-agents) (search summary, not opened)
- Vercel reported a 100% pass rate with `AGENTS.md` against 79% for the best skill-based setup in its own evaluations; results are published at nextjs.org/evals. First-party; method and task set not checked. — [Next.js: AI agents guide](https://nextjs.org/docs/app/guides/ai-agents); commentary in [Next.js 16.2 AGENTS.md hands-on](https://rabinarayanpatra.com/blogs/nextjs-16-2-agents-md-next-browser) (both search summaries)
- Next.js serves a per-version `llms.txt`: `nextjs.org/docs/14/llms.txt` exists alongside `nextjs.org/llms.txt`. — [nextjs.org/docs/14/llms.txt](https://nextjs.org/docs/14/llms.txt) (appeared in search results; not opened)
- Svelte serves `llms.txt` per docs page section (`svelte.dev/docs/svelte/legacy-overview/llms.txt`). — [Svelte llms.txt](https://svelte.dev/docs/svelte/legacy-overview/llms.txt) (appeared in search results; not opened)
- MongoDB docs state that an `llms.txt` index exists and that appending `.md` to any page URL returns Markdown. — [MongoDB upcoming release notes](https://mongodb.com/docs/upcoming/release-notes) (search summary, not opened)
- Stripe's docs open with agent-directed instructions ("Start here: Integrate with Stripe using skills and plugins", `stripe agent setup`), offer `.md` versions of pages, and pin code samples to a dated API version. — [Upgrade your integration](https://docs.stripe.com/upgrades)
- sitespeed.io: agents learned the tool from years of posts about older releases and confidently write removed flags; the project added one curated `llms.txt` and a guide mapping old patterns to current ones. — [sitespeed.io PR #4901](https://github.com/sitespeedio/sitespeed.io/pull/4901) (search summary, not opened)
- Fern's vendor guidance: separate `llms.txt` per version prevents token bloat but requires the tool to know which version the developer targets; one unified file with version sections suits APIs that are mostly stable across versions. — [Fern: API docs for AI agents](https://buildwithfern.com/post/optimizing-api-docs-ai-agents-llms-txt-guide) (search summary, not opened)
- A Docusaurus-based project planned `llms.txt` and `llms-full.txt` per cut version plus a README line in each package pointing to the `llms.txt` for its docs version. — [reforged-ts #186](https://github.com/phmilk/reforged-ts/issues/186) (search summary, not opened)
- Context7 documents a way to specify a library version when retrieving docs for an agent. — [Context7: specifying versions](https://www.mintlify.com/upstash/context7/guides/specifying-versions) (appeared in search results; not opened)
- GitHub uses a coding agent to run its docs-version deprecation runbook under human gates. — [deprecation-steps.md](https://github.com/github/docs/blob/main/src/ghes-releases/lib/deprecation-steps.md)

### Inferences
- The agent problem is the search-engine problem with a worse failure: the agent does not show the reader a banner. Signals that work for agents are those in the text it reads (version stated in the page body or front matter of the Markdown output) and docs located by the installed version, not by a site switcher.
- Shipping version-matched docs with the package removes the need for many hosted versions for agent use; hosted old versions then serve humans only.
- For the three tiers: an unversioned site needs only a version stamp in its machine-readable outputs; a site with live versions needs per-version `llms.txt` and Markdown; archives should be excluded from, or clearly labelled in, machine-readable indexes.

### Gaps
- No independent study of agents choosing the right docs version was found; Vercel's numbers are first-party and I did not read the method.
- No published guidance from Kubernetes, Python, PostgreSQL, Django, Elastic or GitHub on agents and versioned docs was found.
- Whether archived or EOL versions are excluded from any project's `llms.txt` was not checked.

## 7. What published numbers exist: versions kept, repository or build size, build times, traffic share of old versions?

### Takeaway
Counts of versions are easy to find; build and size figures are rare and small-scale; I found no published traffic share for old versions except a single usage figure quoted in the React Native proposal.

### Cited Findings
- Kubernetes: 5 versions served as current plus four previous; 33 `release-1.NN` branches (1.4 to 1.36) in the website repository (observed 2026-10-09). — [Supported doc versions](https://kubernetes.io/docs/home/supported-doc-versions/); [kubernetes/website branches](https://github.com/kubernetes/website/branches)
- PostgreSQL: 5 supported + 24 archived manual versions (6.3 to 13); one PDF per version of 13.6 to 15.3 MB in A4 and US sizes. — [Docs index](https://www.postgresql.org/docs/); [archive](https://www.postgresql.org/docs/manuals/archive/)
- Python: 5 supported and 13 end-of-life branches listed; docs built for 18 language codes; building all maintained versions and translations "can take a few hours". — [Devguide: versions](https://devguide.python.org/versions/); [python/docsbuild-scripts](https://github.com/python/docsbuild-scripts)
- Django: 18 versions in the switcher (1.8 to 6.1) (observed 2026-10-09). — [Django docs home](https://docs.djangoproject.com/en/6.1/)
- GitLab: 106 archived versions as Docker images (19.4 to 10.3). — [GitLab Docs archives](https://docs.gitlab.com/archives/)
- GitHub: 4 supported GHES releases; archiving one takes a 20 to 30 minute scrape. — [Versioning documentation](https://docs.github.com/en/contributing/writing-for-github-docs/versioning-documentation); [deprecation-steps.md](https://github.com/github/docs/blob/main/src/ghes-releases/lib/deprecation-steps.md)
- Node.js: 23 `latest-vN.x` directories (v4 to v26) (observed 2026-10-09). — [nodejs.org/docs](https://nodejs.org/docs/)
- React: 4 frozen major-version sites plus legacy.reactjs.org (observed 2026-10-09). — [React versions](https://react.dev/versions)
- React Native (August 2023): versions 0.60 to 0.73 in the repository; the proposal cites version 0.64 as having "over 5% usage" as a reason to keep its docs reachable; it gives no build-time figure. — [react-native-website #3819](https://github.com/facebook/react-native-website/issues/3819)
- One measured build cost for a Docusaurus-style site: 26 s and 16.6 MiB for one version against 45 s and 20.4 MiB for two. — [zalando-incubator/agentic-identity-broker PR #64](https://github.com/zalando-incubator/agentic-identity-broker/pull/64) (search summary, not opened)
- Docusaurus rule of thumb: fewer than 10 versions. — [Docusaurus: Versioning](https://docusaurus.io/docs/versioning)
- Support windows that set the maintained count: PostgreSQL 5 years per major; Python 5 years (2 bugfix + 3 security); Django LTS typically 3 years; Kubernetes release branches maintained about 12 months. — [PostgreSQL versioning](https://www.postgresql.org/support/versioning/); [Devguide: versions](https://devguide.python.org/versions/); [Django release process](https://docs.djangoproject.com/en/dev/internals/release-process/); [Kubernetes releases (v1.33 snapshot)](https://v1-33.docs.kubernetes.io/releases/release/) (last one is a search summary)

### Inferences
- The number of maintained docs versions is the product's support window divided by its release interval (PostgreSQL 5 years / 1 year = 5; Python the same; Kubernetes shows 5 though only about 3 are supported). Docs teams do not choose it independently.
- Archive size grows without bound by design (Kubernetes 33 branches, GitLab 106 images, PostgreSQL back to 1998), which is why every large project moves archives out of the live build.

### Gaps
- No project published the share of traffic going to old versions. A targeted search returned nothing.
- No repository-size or build-time figures were found for Kubernetes, Django, PostgreSQL, Elastic (before or after 2025), GitHub or Microsoft Learn.
- Elastic's count of doc sets or branches under the old system was not found.
- Red Hat, AWS, Cloudflare, Twilio, Atlassian and HashiCorp numbers were not obtained.
