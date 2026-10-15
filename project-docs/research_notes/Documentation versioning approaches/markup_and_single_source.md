# Markup-level and single-source mechanisms for expressing documentation versions

Research date: 2026-10-09. Scope: content-level mechanisms only (not site generators' version publishing, organizational practice, or archiving).

Confidence markers used below:

- **[fetched]** the page was read this session and the claim comes from it.
- **[snippet]** the claim comes from a search-result summary of the cited page, not a full read.
- **[unverified]** the claim is from prior knowledge; the URL is where it should be confirmed, but it was not read this session.

## Which mechanisms can express a version range or a "since" and "until", and which only named conditions? Which understand version ordering?

### Takeaway
Three families exist. (1) Ordered-version systems with real range syntax: Elastic `applies_to`, GitHub Docs `versions`/`ifversion`, Microsoft Learn monikers, Kubernetes feature-gate data, MDN browser-compat-data. (2) Named-condition systems with no ordering: DITA profiling, DocBook profiling, Flare conditions, Paligo filters, AsciiDoc `ifdef`, Sphinx `only`, Markdoc `if`. (3) Label-only "since" annotations whose version is a free string nobody compares: Sphinx `versionadded` and relatives, GitLab History, Javadoc `@since`. AsciiDoc `ifeval` sits awkwardly between: it has `<`/`>=` but compares numbers, not versions.

### Cited Findings

#### Elastic docs-builder `applies_to` (ordered, ranges, lifecycle, several products)
- General form is `<key>: <lifecycle> [version], <lifecycle> [version], ...`; lifecycle is mandatory, version optional. [fetched] — [docs-builder applies_to syntax](https://elastic.github.io/docs-builder/syntax/applies/)
- Three placements: page frontmatter (`applies_to:` YAML, mandatory on every page), section (a ```` ```{applies_to} ```` block directly after a heading), and inline role (``{applies_to}`stack: ga 9.1` ``). A `{preview}` shorthand role also exists. [fetched] — [docs-builder applies_to syntax](https://elastic.github.io/docs-builder/syntax/applies/)
- Keys: `stack`; `serverless` (sub-keys `security`, `elasticsearch`, `observability`, `vectordb`); `deployment` (sub-keys `ece`, `eck`, `self`, `ess`); `product` and product sub-keys such as `apm_agent_*`, `edot_*`, `curator`, `ecctl`, each with its own version line. [fetched] — [docs-builder applies_to syntax](https://elastic.github.io/docs-builder/syntax/applies/)
- Lifecycles: `experimental`, `preview`, `beta`, `ga`, `deprecated`, `removed`, `unavailable`. [fetched] — [docs-builder applies_to syntax](https://elastic.github.io/docs-builder/syntax/applies/)
- Version specifiers: `x.x` or `x.x+` means "from this version on" (`ga 9.2` and `ga 9.2+` are identical); `x.x-y.y` is an inclusive range; `=x.x` is exactly that version. [fetched] — [docs-builder applies_to syntax](https://elastic.github.io/docs-builder/syntax/applies/)
- Several lifecycles combine with commas: `stack: ga 9.2+, beta 9.0-9.1`. With bare versions the builder infers ranges: `stack: preview 9.0, beta 9.1, ga 9.3` is read as `preview =9.0, beta 9.1-9.2, ga 9.3+`. [fetched] — [docs-builder applies_to syntax](https://elastic.github.io/docs-builder/syntax/applies/)
- "Until" is expressed as a lifecycle, not an upper bound on availability: `ga 9.3, removed 9.5`. `unavailable` is the negation ("not in this context") and guidelines say to use it sparingly. [fetched] — [applies_to syntax](https://elastic.github.io/docs-builder/syntax/applies/); [Elastic cumulative docs guidelines](https://www.elastic.co/docs/contribute-docs/how-to/cumulative-docs/guidelines)
- Unversioned products (Serverless) take a lifecycle with no version; date-based tagging is not accepted. [fetched] — [Elastic cumulative docs guidelines](https://www.elastic.co/docs/contribute-docs/how-to/cumulative-docs/guidelines)

#### GitHub Docs: `versions` frontmatter and Liquid `ifversion` (ordered, ranges, several products, named features)
- Page frontmatter `versions:` takes keys `fpt`, `ghec`, `ghes`, and `feature`. `'*'` means all releases of that product; values are range expressions such as `ghes: '>=2.20'` or `ghes: '>=3.1 <3.3'`. [fetched] — [Versioning documentation, GitHub Docs](https://docs.github.com/en/contributing/writing-for-github-docs/versioning-documentation)
- Inline: `{% ifversion fpt %}`, `{% ifversion ghes > 3.0 %}`, `{% ifversion not ghes %}`, `{% ifversion ghes > 2.21 and ghes < 3.1 %}`, `{% ifversion fpt or ghes > 2.21 %}`, with `elsif` and `else`. Supported comparison operators are `=`, `>`, `<`, `!=`; `==`, `>=`, `<=` are not supported, so "since 3.1" is written `ghes > 3.0`. [fetched] — [Versioning documentation, GitHub Docs](https://docs.github.com/en/contributing/writing-for-github-docs/versioning-documentation)
- Feature-based versioning: `data/features/<name>.yml` holds a `versions:` block (for example `ghes: '>3.1'`); content writes `{% ifversion <name> %}` and frontmatter may carry `feature: '<name>'` (one feature only). When the feature reaches more products, only the YAML file changes. [fetched] — [Versioning documentation, GitHub Docs](https://docs.github.com/en/contributing/writing-for-github-docs/versioning-documentation)
- No lifecycle vocabulary (beta, deprecated) is part of the versioning syntax itself. [fetched, by absence] — [Versioning documentation, GitHub Docs](https://docs.github.com/en/contributing/writing-for-github-docs/versioning-documentation)

#### Microsoft Learn monikers (ordered, ranges, several products)
- Real files in `MicrosoftDocs/azure-devops-docs` set page-level `monikerRange: '<= azure-devops'` in YAML and wrap sections in `:::moniker range="azure-devops"` ... `:::moniker-end`, including `< azure-devops` and a single named moniker such as `azure-devops-2022`. [snippet] — [install-extension.md](https://github.com/MicrosoftDocs/azure-devops-docs/blob/main/docs/marketplace/install-extension.md); [access-levels.md](https://github.com/MicrosoftDocs/azure-devops-docs/blob/main/docs/organizations/security/access-levels.md); [link-type-reference.md](https://github.com/MicrosoftDocs/azure-devops-docs/blob/main/docs/boards/queries/link-type-reference.md)
- The authoring-tool proposal lists three operators for ranges: equals, greater-than-or-equal, less-than-or-equal. [snippet] — [vscode-docs-authoring issue 362](https://github.com/microsoft/vscode-docs-authoring/issues/362)
- Rendered Learn pages expose `moniker_range_name` and `monikers: []` in page metadata, showing the range is resolved to an explicit moniker list at build time. [fetched] — [Markdown reference for Microsoft Learn](https://learn.microsoft.com/en-us/contribute/content/markdown-reference)
- The public Markdown reference (dated 2021-11-09, updated 2024-03-05) does **not** document monikers at all. [fetched] — [Markdown reference for Microsoft Learn](https://learn.microsoft.com/en-us/contribute/content/markdown-reference)

#### Kubernetes: `feature-state` shortcode and feature-gate data (ordered stages with from/to)
- `{{< feature-state state="stable" >}}` renders "Feature state: Stable since Kubernetes v1.37"; `for_k8s_version="v1.10"` overrides the version; valid states are `alpha`, `beta`, `deprecated`, `stable`. [fetched] — [Custom Hugo Shortcodes, Kubernetes](https://kubernetes.io/docs/contribute/style/hugo-shortcodes/)
- `{{< feature-state feature_gate_name="NodeSwap" >}}` reads the state from the gate's data file instead of repeating it in the page. [fetched] — [Custom Hugo Shortcodes, Kubernetes](https://kubernetes.io/docs/contribute/style/hugo-shortcodes/)
- Each gate is one Markdown file under `content/en/docs/reference/command-line-tools-reference/feature-gates/` whose front matter lists `stages:` with `stage` (alpha/beta/stable/deprecated), `defaultValue`, `fromVersion`, and optional `toVersion`. On graduation a new stage is appended and `toVersion` is added to the previous one; earlier stages are kept. `removed: true` moves the gate to the "Feature Gates (removed)" page. [fetched] — [Documenting a feature for a release, Kubernetes](https://kubernetes.io/docs/contribute/new-content/new-features/)

#### MDN browser-compat-data (structured, several products each with versions, since and until)
- Each support statement requires `version_added` (`false`, a version string, `"preview"`, or a ranged value such as `"≤79"`), and may carry `version_removed`, `flags`, `prefix`, `alternative_name`, `partial_implementation` (requires a note), and `notes`. [fetched] — [compat-data-schema.md](https://github.com/mdn/browser-compat-data/blob/main/schemas/compat-data-schema.md)
- A feature's `status` has `experimental` (deprecated field; Baseline preferred), `standard_track`, and `deprecated` booleans. [fetched] — [compat-data-schema.md](https://github.com/mdn/browser-compat-data/blob/main/schemas/compat-data-schema.md)
- A browser may declare `"mirror"` to inherit its upstream browser's data, which is derivation across products rather than restating. [fetched] — [compat-data-schema.md](https://github.com/mdn/browser-compat-data/blob/main/schemas/compat-data-schema.md)

#### AsciiDoc `ifdef` / `ifndef` / `ifeval` (named conditions; numeric comparison only)
- `ifdef::attr[]` ... `endif::[]` includes content when the attribute is set; `ifndef` when it is not. Comma means OR (`ifdef::a,b[]`), plus means AND (`ifdef::a+b[]`); "The two combinators cannot be combined in the same expression." A single-line form exists: `ifdef::revnumber[This document has a version number of {revnumber}.]`. [fetched] — [ifdef and ifndef, Asciidoctor](https://docs.asciidoctor.org/asciidoc/latest/directives/ifdef-ifndef/)
- `ifeval::[{sectnumlevels} == 3]` supports `==`, `!=`, `<`, `<=`, `>`, `>=`, following Ruby operator rules. Value types are number, quoted string, and boolean; an unquoted value containing a period is coerced to a float. Mismatched types make the comparison fail and the content is skipped. `ifeval` has no named `endif`. [fetched] — [ifeval, Asciidoctor](https://docs.asciidoctor.org/asciidoc/latest/directives/ifeval/)
- The page says nothing about version strings. [fetched, by absence] — [ifeval, Asciidoctor](https://docs.asciidoctor.org/asciidoc/latest/directives/ifeval/)
- Include tags: source regions are marked `tag::name[]` / `end::name[]` and selected with `include::file[tag=x]` or `tags=a;b`; `*` selects all tagged regions, `**` all lines, `!name` negates, and nested tags can be excluded (`foo;!bar`). [fetched] — [Include tagged regions, Asciidoctor](https://docs.asciidoctor.org/asciidoc/latest/directives/include-tagged-regions/)

#### DITA profiling (named values on typed attributes; no ordering)
- Conditional attributes: `@product`, `@platform`, `@audience`, `@deliveryTarget` (replaces deprecated `@print`), `@otherprops`, `@props` (specializable into new attributes), and `@rev`, which is "used only for flagging". [fetched] — [Conditional processing attributes, DITA 1.3](https://dita-lang.org/1.3/dita/archspec/base/conditional-processing-attributes)
- DITA 1.3 let `@audience`, `@platform`, `@product`, `@otherprops` accept grouped values with the `@props` syntax. [snippet] — [Changes from DITA 1.2 to 1.3](https://help.adobe.com/en_US/framemaker/using/using-framemaker/dita-1.3-source/non-normative/new-in-1.3.html)
- Evaluation logic: each attribute is evaluated independently; an attribute evaluates to exclude "only when all the values in that attribute evaluate to 'exclude'"; "If any single attribute evaluates to exclude, the element is excluded." So values within an attribute are OR, attributes are AND. [fetched] — [Filtering, DITA 1.3](https://dita-lang.org/1.3/dita/archspec/base/filtering)
- No comparison or range operator exists; a "version" is just another token in `@product` or a specialized attribute. [fetched, by absence] — [Filtering, DITA 1.3](https://dita-lang.org/1.3/dita/archspec/base/filtering)

#### Sphinx / reStructuredText (free-string "since" labels; unordered tags)
- `.. versionadded:: version [explanation]`, `versionchanged`, `deprecated`, and `versionremoved` (added in Sphinx 7.3). Sphinx 9.0 renamed them `version-added`, `version-changed`, `version-deprecated`, `version-removed`, keeping the old names as aliases. They render as "Added in version 2.5: ...", "Changed in version ...", "Deprecated since version ...", "Removed in version ...". [fetched] — [Sphinx directives](https://www.sphinx-doc.org/en/master/usage/restructuredtext/directives.html)
- `.. only:: <expression>` includes content if a tag expression is true, for example `.. only:: html and draft`; boolean expressions with parentheses are supported. Tags are set with `--tag` or in `conf.py`; undefined tags are false. [fetched] — [Sphinx directives](https://www.sphinx-doc.org/en/master/usage/restructuredtext/directives.html)
- CPython adds `deprecated-removed` (deprecation version plus a specific removal version), `soft-deprecated`, `availability` (platforms, with negation: `Unix, not WASI, not Android.`), and `impl-detail`. [fetched] — [Python devguide, reStructuredText markup](https://devguide.python.org/documentation/markup/)
- `sphinx.ext.ifconfig` includes content when a Python expression over config values is true, which would allow arbitrary comparisons. [unverified] — [sphinx.ext.ifconfig](https://www.sphinx-doc.org/en/master/usage/extensions/ifconfig.html)

#### Markdoc (named variables, equality only)
- Variables are `{% $variable %}` with dot paths, supplied through `config.variables` at transform time; partials can receive variables (`{% partial variables={sdk: "Ruby", version: 3} file="header.md" /%}`). [fetched] — [Markdoc variables](https://markdoc.dev/docs/variables)
- Six built-in functions: `equals`, `and`, `or`, `not`, `default`, `debug`. `equals` is strict equality on primitives; there is no greater-than or less-than. Custom functions (the docs show `includes`) are registered in `config.functions`. Example: `{% if and(not($a), or($b, $c)) %}`. [fetched] — [Markdoc functions](https://markdoc.dev/docs/functions)

#### GitLab (prose labels in fixed shortcodes)
- `{{< details >}}` holds `- Tier:`, `- Offering:`, `- Status:` (Beta, Experiment, or Limited availability; GA has no status line). `{{< history >}}` holds one bullet per change, such as `- [Introduced](link) in GitLab 16.3.` [fetched] — [GitLab docs: availability details](https://docs.gitlab.com/development/documentation/styleguide/availability_details/)

#### Commercial single-sourcing tools
- MadCap Flare: content carries condition tags (grouped in condition tag sets); each target has include/exclude settings, with a Basic and an Advanced expression editor. By default content with any of its tags is included unless excluded, and an exclude on one tag removes content that carries two tags. [snippet] — [Flare 2026: Creating Basic and Advanced Tag Expressions](https://help.madcapsoftware.com/flare2026/Content/Flare/Step2-Authoring/Conditions/Other-Activities/Creating-Basic-Advanced-Tag-Expressions.htm); [Flare 2026: Associating Conditions With Targets](https://help.madcapsoftware.com/flare2026/Content/Flare/Step2-Authoring/Conditions/Process/Associating-Conditions-Targets.htm)
- Paligo: filter attributes come from DocBook plus Paligo's own, including `xinfo:product` and `xinfo:version`. Unmarked content is always included; elements with a different value for the selected attribute are excluded; several chosen values act as OR. [snippet] — [Paligo filter attributes](https://docs.paligo.net/en/filter-attributes.html); [Paligo: How Filtering Works](https://docs.paligo.net/en/how-filtering-works.html)
- DocBook profiling uses effectivity attributes (`arch`, `condition`, `os`, `revision`, `userlevel`, `vendor`, and others) matched against `profile.*` stylesheet parameters. [unverified; host unreachable this session] — [DocBook XSL: Profiling](http://www.sagehill.net/docbookxsl/Profiling.html)

#### Code-level annotations
- Rust: `#[stable(feature = "foo", since = "1.420.69")]`, `#[unstable(feature = "foo", issue = "1234")]`, `#[deprecated(since = "1.38.0", note = "...")]`. [fetched] — [rustc dev guide: stability attributes](https://rustc-dev-guide.rust-lang.org/stability.html)
- Javadoc `@since`, Go's `// Deprecated:` paragraph, and OpenAPI's boolean `deprecated` field are single-point or boolean markers with no range. [unverified] — [Javadoc doc comment spec](https://docs.oracle.com/en/java/javase/21/docs/specs/javadoc/doc-comment-spec.html); [Go wiki: Deprecated](https://go.dev/wiki/Deprecated); [OpenAPI 3.1.0](https://spec.openapis.org/oas/v3.1.0)

### Inferences
- Because Asciidoctor coerces unquoted dotted values to floats, `ifeval::[{version} >= 1.9]` would treat 1.10 as 1.1 and get it wrong, and a three-part version would not be a float at all. AsciiDoc therefore has comparison syntax but no version ordering. This follows from the coercion rule; the Asciidoctor page does not state it.
- Only Elastic's syntax expresses a whole lifecycle history (preview, then beta, then GA, then removed) in one annotation. Kubernetes holds the same shape, but in a data file per feature gate rather than in the prose.
- GitHub's syntax separates "which versions" (ordered, ranged) from "which feature" (a named indirection onto a versions block), which is the only mechanism found where a named condition is itself defined in terms of ordered ranges.
- DITA, Flare, Paligo, and DocBook cannot say "since 3.1" without enumerating every later version, or inventing a token that means it.

### Gaps
- Microsoft's authoritative moniker syntax (full operator list, `||`, nesting rules) could not be confirmed at a primary source. The public contributor Markdown reference omits it, and the docfx v3 spec URL returned 404. Evidence here is from real repository files only.
- Whether Stripe uses Markdoc conditionals for its dated API versions is not confirmed; the Stripe Markdoc post does not mention API versions.
- DocBook profiling, Author-it, and Heretto specifics were not read at a primary source this session.
- MDX conditionals and Docusaurus admonitions were not researched; neither has a version construct in its language that I could cite.

## Which treat availability as something shown to the reader (a badge), which as a build filter, and which do both from one annotation?

### Takeaway
Most mechanisms do one or the other. Filters: AsciiDoc conditionals, Sphinx `only`, Markdoc `if`, GitHub `ifversion`, Flare, Paligo, moniker zones. Labels: Sphinx version directives, Kubernetes `feature-state`, GitLab History, Elastic `applies_to`, Rust stability, BCD tables. DITA is the one long-standing standard where a single attribute can either filter or flag, chosen by the DITAVAL profile at build time. Elastic deliberately chose label-only: nothing is filtered, every version reads the same page.

### Cited Findings
- Elastic: "the same page stays valid over time and shows version-related evolutions"; applicability is shown as badges such as "Elastic Stack: Generally available since 9.1". The source describes no version dropdown and no filtering. [fetched] — [Elastic: cumulative docs](https://www.elastic.co/docs/contribute-docs/how-to/cumulative-docs)
- Elastic: "We don't maintain versioned branches or separate copies of a page for each release. Instead, every page covers all supported versions". [snippet] — [docs-builder applies_to syntax](https://elastic.github.io/docs-builder/syntax/applies/)
- Elastic badges are computed at build time from the lifecycle plus a release status; popovers read "Generally available since X.X", "Preview in X.X", "Planned for removal". [fetched] — [applies_to syntax](https://elastic.github.io/docs-builder/syntax/applies/); [Elastic: cumulative docs](https://www.elastic.co/docs/contribute-docs/how-to/cumulative-docs)
- GitHub Docs: `versions` frontmatter decides which versions get the page at all and `ifversion` removes body content per version; the reader picks a version in the version picker. Nothing in the versioning page describes a rendered badge. [fetched] — [Versioning documentation, GitHub Docs](https://docs.github.com/en/contributing/writing-for-github-docs/versioning-documentation)
- DITA: a DITAVAL profile assigns each attribute value an action. `@audience`, `@platform`, `@product`, `@otherprops`, `@props`, `@deliveryTarget` can be filtered or flagged; `@rev` can only be flagged. Flagging can "highlight specific content on output". [fetched] — [Conditional processing attributes, DITA 1.3](https://dita-lang.org/1.3/dita/archspec/base/conditional-processing-attributes)
- DITAVAL structure: a `val` root with `prop` and `revprop` children, which can carry start and end flags. [snippet] — [DITAVAL elements, DITA 1.3](https://docs.oasis-open.org/dita/dita/v1.3/os/part1-base/langRef/containers/ditaval-elements.html)
- Sphinx version directives only render a labelled paragraph; `only` only filters, and "is designed to control only content of document. It could not control sections, labels and so on." [fetched] — [Sphinx directives](https://www.sphinx-doc.org/en/master/usage/restructuredtext/directives.html)
- Kubernetes `feature-state` renders a line of text and, with `feature_gate_name`, a "More information about this feature" block from the data file; it does not filter. [fetched] — [Custom Hugo Shortcodes, Kubernetes](https://kubernetes.io/docs/contribute/style/hugo-shortcodes/)
- Kubernetes `{{< version-check >}}` compares the page parameter `min-kubernetes-server-version` with the site `version` and renders a notice: a page-level "requires at least" label. [fetched] — [Custom Hugo Shortcodes, Kubernetes](https://kubernetes.io/docs/contribute/style/hugo-shortcodes/)
- GitLab: details and history blocks are reader-facing labels; the guide says not to use badges inline and to write inline availability as plain text such as "Ultimate only." [fetched] — [GitLab docs: availability details](https://docs.gitlab.com/development/documentation/styleguide/availability_details/)
- Markdoc `if` filters at transform time from variables; Stripe uses it for "conditionally displaying content based on their location or the Stripe features they use". [fetched] — [How Stripe builds interactive docs with Markdoc (2022-09-13)](https://stripe.dev/blog/markdoc)
- Paligo: marking content only declares what can be filtered; the filter is applied in publishing settings, and a preview shows the effect for a chosen variant. [snippet] — [Paligo: Mark Up Elements for Filtering](https://docs.paligo.net/en/mark-up-elements-for-filtering.html); [Paligo: Preview the filter effects](https://docs.paligo.net/en/preview-the-filter-effects.html)

### Inferences
- DITA's design, in which the content states a fact ("this is for product X") and a separate profile decides whether that fact filters or flags, is the closest precedent for one annotation driving both a badge and a build filter. It lacks version ordering, so it cannot do so for "since".
- No mechanism found derives both a reader-facing "since 9.1" badge and a per-version filtered build from one ordered annotation. Elastic has the ordered annotation and the badge but no filter; GitHub has the ordered annotation and the filter but no badge; Microsoft's monikers filter and drive a selector.
- Label-only systems never lose content silently. Filter-only systems never show the reader that a paragraph is version-specific unless the author also writes it in prose.

### Gaps
- Whether Microsoft Learn renders any per-zone "applies to" label from monikers, as opposed to only filtering by the selected version, was not confirmed.
- How DITA-OT renders flags in practice (images, colors, change bars) was not read beyond the spec snippet.

## How do they declare the list of versions, the current one, and unreleased ones?

### Takeaway
The systems that understand order all keep one central registry outside the content: Elastic's `versions.yml`, GitHub's `lib/enterprise-server-releases.ts`, Antora's `antora.yml` per component version, Kubernetes's `hugo.toml` params, BCD's `browsers/*.json`. Systems of named conditions either declare nothing (AsciiDoc attributes, Sphinx tags) or declare a controlled vocabulary with no order (DITA subject scheme, Flare condition tag sets). Unreleased versions are handled by a placeholder token (Python `next`, Rust `CURRENT_RUSTC_VERSION`), a rendering state (Elastic "Planned"), or a prerelease flag (Antora).

### Cited Findings
- Elastic: `config/versions.yml` in docs-builder "tracks the latest released version of each product and the earliest version documented in Docs V3" and "drives our dynamic rendering logic". Each badge combines a lifecycle with a release status (prerelease or post-release). [fetched] — [Elastic: cumulative docs](https://www.elastic.co/docs/contribute-docs/how-to/cumulative-docs); [versions.yml](https://github.com/elastic/docs-builder/blob/main/config/versions.yml)
- Elastic unreleased rendering: a `ga`/`preview`/`beta` annotation for an unreleased version shows "Planned"; `deprecated` shows "Deprecation planned"; `removed` shows "Removal planned". A range whose end is unreleased is shown as `x.x+`. Guidelines: "Avoid using version numbers in prose adjacent to `applies_to` badge", because the badge may show "Planned". [fetched] — [applies_to syntax](https://elastic.github.io/docs-builder/syntax/applies/); [cumulative docs guidelines](https://www.elastic.co/docs/contribute-docs/how-to/cumulative-docs/guidelines)
- Elastic scope: cumulative docs start at Stack 9.0, ECE 4.0, ECK 3.0; earlier majors stay in the old AsciiDoc system. [fetched] — [Elastic: cumulative docs](https://www.elastic.co/docs/contribute-docs/how-to/cumulative-docs)
- GitHub: the authoritative GHES release list is `lib/enterprise-server-releases.ts`; version names look like `enterprise-server@<release>` with short name `ghes`; four GHES releases are supported at a time. [fetched] — [Versioning documentation, GitHub Docs](https://docs.github.com/en/contributing/writing-for-github-docs/versioning-documentation)
- Antora: the `version` key in `antora.yml` declares a component version; `true` takes the git refname; `~` makes it versionless. [snippet] — [Antora: Version Key](https://docs.antora.org/antora/3.2/component-version-key/)
- Antora ordering: versionless first, then named versions in reverse alphabetical order, then semantic versions descending. The latest version is the first non-prerelease in that order. `display_version` does not affect sorting. [fetched] — [Antora: How Component Versions are Sorted](https://docs.antora.org/antora/latest/how-component-versions-are-sorted/)
- Antora lets `antora.yml` assign AsciiDoc attributes per component version, which is how per-version values reach `ifdef`/`ifeval` and `{attribute}` substitution. [unverified] — [Antora: Assign Attributes to a Component Version](https://docs.antora.org/antora/latest/component-attributes/)
- Kubernetes: `hugo.toml` defines `version` and `latest` site params; shortcodes `{{< param "version" >}}`, `{{< latest-version >}}`, `{{< latest-semver >}}` print them. An archived docs set can have `version` v1.19 while `latest` is v1.20. [fetched] — [Custom Hugo Shortcodes, Kubernetes](https://kubernetes.io/docs/contribute/style/hugo-shortcodes/)
- Python: unreleased features use the literal `next` as the version (`.. versionadded:: next`); "When a release is made, the release manager will change the `next` to the just-released version" using `update_version_next.py`. `deprecated-removed` requires a specific removal version, not `next`. [fetched] — [Python devguide, reStructuredText markup](https://devguide.python.org/documentation/markup/)
- Rust: stabilization PRs write `#[stable(since = "CURRENT_RUSTC_VERSION")]`, a placeholder later replaced with the real version. [fetched] — [rustc dev guide: stability attributes](https://rustc-dev-guide.rust-lang.org/stability.html)
- Django: docs for unreleased features must be marked because "documentation readers are using the latest release, not the development version". [fetched] — [Django: Writing documentation](https://docs.djangoproject.com/en/dev/internals/contributing/writing-documentation/)
- DITA: a subject scheme map defines controlled values as a `subjectdef` hierarchy and binds them to an attribute; `schemeref` merges a base scheme into an extending one. Authoring tools "SHOULD use these lists of controlled values to provide lists from which authors can select values". [snippet] — [Defining controlled values for attributes, DITA 1.3](https://docs.oasis-open.org/dita/dita/v1.3/os/part1-base/archSpec/base/controlled-values-for-attributes.html); [Subject scheme maps](https://dita-lang.org/1.3/dita/archspec/base/subjectschema)
- BCD: `version_added: "preview"` denotes support in the current beta or preview channel. [fetched] — [compat-data-schema.md](https://github.com/mdn/browser-compat-data/blob/main/schemas/compat-data-schema.md)
- BCD declares each browser's releases, with dates and statuses, in `browsers/<browser>.json`. [unverified] — [browsers-schema.md](https://github.com/mdn/browser-compat-data/blob/main/schemas/browsers-schema.md)
- Sphinx tags have no registry: any Python identifier may be passed with `--tag`, and undefined tags are simply false. [fetched] — [Sphinx directives](https://www.sphinx-doc.org/en/master/usage/restructuredtext/directives.html)

### Inferences
- A placeholder for "the next release" appears independently in Python (`next`), Rust (`CURRENT_RUSTC_VERSION`), and Elastic ("Planned" rendering). All three exist because features are documented before the version number is certain, and GitHub's guide states the same hazard: changes may not ship in the release originally planned.
- Elastic's approach differs in kind: the author writes the intended version, and the registry decides at build time whether to show it, so no rewrite step is needed at release.
- Antora's sort (named before semantic, reverse alphabetical) shows the cost of inferring order from names. Systems with an explicit ordered registry avoid that.

### Gaps
- The exact schema of Elastic's `versions.yml` and of Microsoft's moniker definitions was not read.
- How Flare, Paligo, Author-it, and Heretto let an administrator declare an ordered list of versions, if at all, was not confirmed.

## What validation exists? What are the documented failure modes: content that silently disappears, conditions nobody understands, dead conditions left after a version retires, combinatorial explosion of attributes?

### Takeaway
Validation is strongest where versions are a registry: Elastic rejects overlapping or contradictory lifecycle sets at build, and GitHub lints version names and unsupported versions as errors. Named-condition systems default to permissive behavior that hides mistakes: DITA includes any value with no rule, Sphinx treats an unknown tag as false, AsciiDoc treats an unset attribute as unset. No tool found checks for content that is visible in no version.

### Cited Findings

#### Validation that exists
- Elastic build rules: one version per lifecycle (`ga 9.2, ga 9.3` invalid); one open-ended `+` per key (`ga 9.2+, beta 9.0+` invalid); range order (`preview 9.2-9.0` invalid); no overlaps, with ranges inclusive (`ga 9.2+, beta 9.0-9.2` invalid). Page-level `applies_to` is mandatory. [fetched] — [applies_to syntax](https://elastic.github.io/docs-builder/syntax/applies/)
- GitHub content linter rules, all severity error: GHD019 "Liquid `ifversion` tags should be used instead of `if` tags when the argument is a valid version"; GHD020 "Liquid `ifversion` tags should contain valid version names as arguments"; GHD022 "Liquid `ifversion`, `elsif`, and `else` tags should be valid and not contain unsupported versions"; GHD016 conditional arguments must not be quoted; GHD040 "Tables must use the correct liquid versioning format"; GHD006 no hardcoded old-version internal links; GHD014 data references must exist. [fetched] — [Using the content linter, GitHub Docs](https://docs.github.com/en/contributing/collaborating-on-github-docs/using-the-content-linter)
- DITA: "Conditional processing code should provide a report of any attribute values encountered in content that do not have an action associated with them." [snippet] — [DITAVAL elements, DITA 1.3](https://docs.oasis-open.org/dita/dita/v1.3/os/part1-base/langRef/containers/ditaval-elements.html)
- DITA subject scheme binding "restricts the permissible values for the attribute to those that are contained in the set of controlled values". [snippet] — [Defining controlled values for attributes, DITA 1.3](https://docs.oasis-open.org/dita/dita/v1.3/os/part1-base/archSpec/base/controlled-values-for-attributes.html)
- Markdoc: every tag and node has a schema; Stripe runs "the Markdoc validator in our continuous integration system to ensure correctness at build time" and has a VS Code extension showing errors while typing. [fetched] — [How Stripe builds interactive docs with Markdoc](https://stripe.dev/blog/markdoc)
- BCD lint rules include: desktop browsers mandatory, `partial_implementation` requires a note, `spec_url` required when `standard_track` is true, ranged versions only for releases two or more years old. [fetched] — [compat-data-schema.md](https://github.com/mdn/browser-compat-data/blob/main/schemas/compat-data-schema.md)
- Rust: `deprecated(since)` is checked against the current compiler version (`deprecated_in_future` lint); crates using `stable`/`unstable` must enable `staged_api`; a deprecated item must also carry `stable` or `unstable`. [fetched] — [rustc dev guide: stability attributes](https://rustc-dev-guide.rust-lang.org/stability.html)
- Flare has a "Viewing Unused Conditions" view that lists and deletes unused tags, and MadCap Analyzer reports undefined condition tags. [snippet] — [Flare: Viewing Unused Conditions](https://help.madcapsoftware.com/flare2024/Content/Flare/Conditions/Other-Activities/Viewing-Unused-Condition-Tags1.htm)

#### Documented failure modes
- Permissive defaults. DITA: "By default, values in conditional processing attributes that are not defined in a DITAVAL profile evaluate to 'include'." Sphinx: undefined tags are false. [fetched] — [Filtering, DITA 1.3](https://dita-lang.org/1.3/dita/archspec/base/filtering); [Sphinx directives](https://www.sphinx-doc.org/en/master/usage/restructuredtext/directives.html)
- Silent type failure in AsciiDoc: if the two sides of `ifeval` have different types, "the comparison fails and the content is skipped". [fetched] — [ifeval, Asciidoctor](https://docs.asciidoctor.org/asciidoc/latest/directives/ifeval/)
- Negation that changes meaning over time. GitHub: adding a new version changes how `not` and `else` evaluate; the guide tells writers to check an article's frontmatter when editing and to verify in rendered output across versions, since `not`/`else` make output depend on each article's frontmatter. [fetched] — [Versioning documentation, GitHub Docs](https://docs.github.com/en/contributing/writing-for-github-docs/versioning-documentation)
- Reuse plus conditions. GitHub: reusables "can include Liquid conditionals", and the versioning guide notes a reusable can render differently depending on which article references it. [fetched] — [Creating reusable content, GitHub Docs](https://docs.github.com/en/contributing/writing-for-github-docs/creating-reusable-content); [Versioning documentation](https://docs.github.com/en/contributing/writing-for-github-docs/versioning-documentation)
- Liquid layout breakage: whitespace control (`{%- ifversion fpt %}`) is needed so conditionals do not break lists, and tables need a dedicated versioning format enforced by GHD040. [fetched] — [Versioning documentation](https://docs.github.com/en/contributing/writing-for-github-docs/versioning-documentation); [content linter](https://docs.github.com/en/contributing/collaborating-on-github-docs/using-the-content-linter)
- Flare double-tag conflict: content with two tags is removed by an exclude rule on one of them even when the other is wanted. [snippet] — [Flare troubleshooting: Condition Tags](https://help.madcapsoftware.com/flare2025/Content/Flare/Resources/Troubleshooting/TS-Condition-Tags1.htm)
- Flare naming hazards: generic tag names, names that are substrings of others, and product names in tags, which age badly. The same article used a project with thirteen conditions as its example of scale. [snippet] — [MadCap blog: Tips and tricks using conditions](https://www.madcapsoftware.com/blog/tips-and-tricks-using-conditions-in-madcap-flare/)
- DITA limits reported by Oxygen: `xml:lang` cannot be used for profiling, and an entire CALS table column cannot easily be filtered. [snippet] — [Oxygen XML blog: Small Problems with the DITA Standard](https://oxygenxmlblog.netlify.com/topics/dita-standard-problems)
- Sphinx `only` cannot control sections or labels. [fetched] — [Sphinx directives](https://www.sphinx-doc.org/en/master/usage/restructuredtext/directives.html)
- Stripe on unrestricted logic in content: "Content authoring effectively became software development", which is why Markdoc omitted loops and variable assignment to keep "the content ... fully stateless". [fetched] — [How Stripe builds interactive docs with Markdoc](https://stripe.dev/blog/markdoc)

#### Cleanup of old conditions
- Django: "we only keep these annotations around for two releases", and they are written as self-contained blocks so "we can delete the lines containing `.. versionchanged:: A.B` and its indented description without editing any other lines". Version numbers are not to appear in prose outside those blocks. [fetched] — [Django: Writing documentation](https://docs.djangoproject.com/en/dev/internals/contributing/writing-documentation/)
- GitLab: remove history items and inline text that refer to unsupported versions; GitLab supports the current major and two previous majors; removal merge requests are merged during the milestone of the new major. Feature-flag history is removed only when every event for the flag is in unsupported versions. [fetched] — [GitLab docs: availability details](https://docs.gitlab.com/development/documentation/styleguide/availability_details/)
- GitHub: deprecating a GHES release is a checklist: move the number to the deprecated array in the releases file, run scripts to remove static files and redirects, then "Remove the outdated Liquid markup and frontmatter" on a separate topic branch for review. [snippet] — [github/docs PR 9140](https://github.com/github/docs/pull/9140/files)
- Kubernetes: stages are appended, not replaced, and removed gates move to a separate page with `removed: true`. [fetched] — [Documenting a feature for a release, Kubernetes](https://kubernetes.io/docs/contribute/new-content/new-features/)
- Elastic: "No information should be removed for supported product versions, unless it was never accurate." The guidelines give no rule for retiring old annotations. [fetched] — [Elastic: cumulative docs](https://www.elastic.co/docs/contribute-docs/how-to/cumulative-docs); [guidelines](https://www.elastic.co/docs/contribute-docs/how-to/cumulative-docs/guidelines)
- Python's devguide states no policy for removing old `versionadded`/`versionchanged`. [fetched, by absence] — [Python devguide, reStructuredText markup](https://devguide.python.org/documentation/markup/)

### Inferences
- The check "this content is visible in no declared version" was found in none of the sources. Elastic's overlap and order checks are the nearest thing, and they are checks on one annotation, not on the interaction of nested ones (a section inside a page whose ranges do not intersect).
- Cleanup is mechanical only where the registry is ordered: GitHub can strip conditions for a retired release because `ghes < 3.9` is decidable once 3.9 is the oldest. A named condition gives a tool nothing to reason about.
- Django's rule (self-contained annotation blocks, no version numbers in prose) is an authoring convention that makes deletion safe without tooling; Elastic's "no version numbers next to badges" rule serves a different purpose but has the same effect of keeping the version in one place.
- Negation and `else` are the constructs whose meaning shifts when the version list grows; systems with only positive "since/range" statements avoid that class of error.

### Gaps
- No source quantified combinatorial explosion of profiling attributes in DITA, Flare, or Paligo; the search surfaced only general complaints. The critique is widely repeated in the technical-writing community but I found no primary, citable study.
- Whether docs-builder reports an error when a section-level `applies_to` contradicts the page-level one was not documented on the pages read.
- The current GitHub Docs deprecation procedure (2026) was seen only through an older pull request; the script names may have changed.
- Asciidoctor's behavior for a missing include tag (warning or silent) was not confirmed.

## What have teams published about conditional single-sourcing at scale, for and against?

### Takeaway
Published first-party rationale is thin and mostly lives in contributor guides, not engineering posts. Elastic gives the clearest argument for one cumulative source. GitHub's guide is the clearest account of the costs of conditionals at scale. Stripe's Markdoc post argues for constraining logic in content. Antora's documentation recommends branches for versions.

### Cited Findings
- Elastic, for cumulative: one source of truth per feature supports "consistency, accuracy, and maintainability" and avoids drift between similar doc sets; readers arriving from outside land on a page more likely to apply to them and can compare versions on one page. [fetched] — [Elastic: cumulative docs](https://www.elastic.co/docs/contribute-docs/how-to/cumulative-docs)
- Elastic on the old model: separate docs sets for every minor version "resulted in fragmentation and unnecessary duplication"; the move was bundled with AsciiDoc to Markdown and a new information architecture. [snippet; the migration page returned 404 when fetched directly] — [docs-builder migration: New versioning (PR preview)](https://docs-v3-preview.elastic.dev/elastic/docs-builder/pull/2927/migration/versioning)
- Elastic contributor rules surfaced in repository automation: all 9.x versions are documented on the same page and published from main; do not remove information that applies to an older version; do not add version-specific information without an `applies_to` tag. [snippet] — [elastic/elasticsearch PR 136943](https://github.com/elastic/elasticsearch/pull/136943)
- Elastic cautions within its own model: tag only when applicability changes from what the page already established; do not tag typo or structure changes; "For updates, remember they might be older than you think"; do not assume a default product or deployment type. [fetched] — [Elastic cumulative docs guidelines](https://www.elastic.co/docs/contribute-docs/how-to/cumulative-docs/guidelines)
- GitHub, costs acknowledged in its own guide: `not`/`else` depend on per-article frontmatter; reusables render differently by referencing article; a change may not ship in the planned release; versioning must be verified in rendered output for each version. [fetched] — [Versioning documentation, GitHub Docs](https://docs.github.com/en/contributing/writing-for-github-docs/versioning-documentation)
- GitHub's mitigation is indirection: feature-based versioning moves the range out of the prose into one YAML file per feature. [fetched] — [Versioning documentation, GitHub Docs](https://docs.github.com/en/contributing/writing-for-github-docs/versioning-documentation)
- Stripe (2022-09-13): mixing ERB, Ruby, HTML, and Markdown made personalization possible but hurt maintainability; Markdoc imposes "prescriptive rails for content extensibility". [fetched] — [How Stripe builds interactive docs with Markdoc](https://stripe.dev/blog/markdoc)
- Antora recommends storing each documentation version in a branch, as software version lines are kept, and notes tags cannot be updated afterwards. [snippet] — [Antora: content branches](https://docs.antora.org/antora/3.2/playbook/content-branches/)
- Kubernetes keeps per-release docs sets (a `dev-1.38` branch receives feature docs) and uses in-content annotations only for feature maturity, not to merge releases into one source. [fetched] — [Documenting a feature for a release, Kubernetes](https://kubernetes.io/docs/contribute/new-content/new-features/)
- Django and GitLab both pair in-content version notes with a retention window (two releases; three majors). [fetched] — [Django: Writing documentation](https://docs.djangoproject.com/en/dev/internals/contributing/writing-documentation/); [GitLab docs: availability details](https://docs.gitlab.com/development/documentation/styleguide/availability_details/)

### Inferences
- Elastic and GitHub chose opposite reader models from similar annotations: Elastic shows everything with badges, GitHub filters per selected version. Elastic's "no information removed" rule and GitHub's "verify every version" rule are the respective prices.
- Every team that keeps version facts in content also publishes a rule limiting how long, or how often, they are written (Elastic: only when applicability changes; Django: two releases; GitLab: supported versions only). Annotation volume is treated as the main scaling risk.
- Kubernetes and Antora show that in-content annotations and per-version branches are routinely combined: branches for the version line, annotations for maturity or small differences.

### Gaps
- No Elastic engineering blog post on the move was found; the rationale is in contributor documentation only, and the guidelines page is described as still in development.
- No first-party Microsoft write-up of monikers at scale was found.
- No GitHub engineering blog post specifically on Liquid versioning at scale was found.
- No Antora or Asciidoctor page explicitly weighing conditionals against branches was found; only the branch recommendation.
- Write the Docs, tcworld, and OASIS DITA Adoption Committee material on profiling at scale was not located this session.

## Is there any published comparison of annotation-in-one-source against a copy per version, with reasons and outcomes?

### Takeaway
No neutral, evidence-based comparison was found. What exists is one-sided rationale from teams that switched (Elastic) or stayed (Antora's branch recommendation), with reasons but no measured outcomes.

### Cited Findings
- Elastic states reasons for moving to one source (single source of truth, less drift, readers see all versions on one page) but the pages read give no before-and-after measurements. [fetched] — [Elastic: cumulative docs](https://www.elastic.co/docs/contribute-docs/how-to/cumulative-docs)
- Elastic kept a copy-per-version boundary at majors: 8.x and earlier remain in the old per-version AsciiDoc system, and cumulative pages start at 9.0. [fetched] — [Elastic: cumulative docs](https://www.elastic.co/docs/contribute-docs/how-to/cumulative-docs)
- GitHub combines both: live conditionals for the supported releases, and frozen snapshots, reachable by URL, for releases that have closed down. [fetched] — [Versioning documentation, GitHub Docs](https://docs.github.com/en/contributing/writing-for-github-docs/versioning-documentation)
- Antora's documentation describes branches as well suited to managing multiple versions of the same content. [snippet] — [Antora: content branches](https://docs.antora.org/antora/3.2/playbook/content-branches/)
- DITA 1.3 added branch filtering so one map can publish several filtered copies of the same branch: two peer `ditavalref` elements produce two copies, each filtered by its own DITAVAL. Before 1.3, conditions applied only to a root map. [snippet] — [Using a single ditavalref, DITA 1.3](https://docs.oasis-open.org/dita/dita/v1.3/os/part1-base/archSpec/base/branch-filtering-single-set.html); [Changes from DITA 1.2 to 1.3](https://help.adobe.com/en_US/framemaker/using/using-framemaker/dita-1.3-source/non-normative/new-in-1.3.html)

### Inferences
- Both large adopters of one-source versioning bound it: Elastic by major version, GitHub by support window. Neither annotates indefinitely; old versions end as frozen copies. The practical pattern is annotation within a window and a copy beyond it.
- DITA branch filtering is the single-source equivalent of "several versions side by side in one output", produced from conditions rather than copies.

### Gaps
- No published comparison with outcome data (maintenance effort, error rates, reader success) was found. This is a real absence in the sources searched, not only a search miss, but the search was limited to roughly forty calls.
- Elastic's results since the 2025 launch are not reported in the pages read.
