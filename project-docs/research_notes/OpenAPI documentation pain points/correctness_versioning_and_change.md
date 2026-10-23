# Keeping OpenAPI-based documentation correct: examples, testing, coverage, versioning, and change tracking

Researched 2026-10-22. Vendor claims are labeled "(vendor)". Sources a competitor wrote about a rival are labeled "(competitor)". Dates are given where a source is older than 2025.

## Examples: how request and response examples are validated, in the spec and outside it

### Takeaway
Examples inside the spec are well covered: Spectral and Redocly lint them against their schemas by default, openapi-examples-validator checks them in CI, and Schemathesis replays them against a live server. Examples outside the spec are another matter. JSON in Markdown guides, tutorials, and SDK READMEs is checked by almost nothing. The one tool found for it, skeemas-markdown-validation, needs a schema linked by hand above each block and doesn't read OpenAPI. That gap is the opening for a compiler that validates prose examples against the spec's schemas.

### Cited Findings
- Spectral's `oas3-valid-media-example` says "Examples must be valid against their defined schema. This rule is applied to Media Type objects." `oas3-valid-schema-example` does the same for Schema objects. Both are in the recommended ruleset at severity `error`, and the OAS2 rules (`oas2-valid-media-example`, `oas2-valid-schema-example`) match them. The docs say errors usually come from type mismatches and from properties the schema doesn't include. — [Spectral OpenAPI rules reference](https://raw.githubusercontent.com/stoplightio/spectral/develop/docs/reference/openapi-rules.md)
- Spectral's `oas3-examples-value-or-externalValue` (warn) checks only that an example has `value` or `externalValue` and not both. The rules page doesn't say whether the validation rules follow `externalValue`. — [Spectral OpenAPI rules reference](https://raw.githubusercontent.com/stoplightio/spectral/develop/docs/reference/openapi-rules.md)
- Redocly's `no-invalid-media-type-examples` "[d]isallow[s] invalid media type examples by ensuring they comply with the corresponding schema definitions." It covers `example` and `examples`, plus `dataValue` in OAS 3.2 ("validated against the schema the same way `value` is"). Its default severity is `warn`. Its option `allowAdditionalProperties` defaults to `false`, so an example with extra properties fails. It supports OAS 3.0, 3.1, and 3.2 but not 2.0, and the page says nothing about `externalValue`. — [Redocly rule docs](https://redocly.com/docs/cli/rules/oas/no-invalid-media-type-examples)
- openapi-examples-validator "validates embedded JSON-examples in OpenAPI-specs (v2 and v3 are supported)". Examples kept outside the spec can be checked with `-s` (schema path) and `-e` (example file), or with a mapping file (`-m`) from schema paths to example files. `-n` rejects undeclared properties and `-o` ignores chosen formats. It needs Node 18 or later. — [openapi-examples-validator README (v6.0.3)](https://unpkg.com/openapi-examples-validator@6.0.3/README.md)
  - The mapping-file mode is the nearest existing pattern to "validate an example file against a named schema". The user has to keep the schema-path mapping up to date by hand.
- Schemathesis's `examples` phase "uses example and examples from your schema, filling missing parts with generated data". It sends them to the running API and runs checks such as `response_schema_conformance` and `status_code_conformance` on the responses. All checks are on by default. The other phases are `coverage` (boundary values for every constraint), `fuzzing`, and `stateful`. — [Schemathesis CLI reference](https://schemathesis.readthedocs.io/en/stable/reference/cli/); [Schemathesis checks reference](https://schemathesis.readthedocs.io/en/stable/reference/checks/)
- Outside the spec, skeemas-markdown-validation validates JSON code blocks in Markdown against a schema linked above each block, and a drifted example fails the test run. It uses skeemas' own schema format and doesn't read an OpenAPI document. — [skeemas-markdown-validation on GitHub](https://github.com/Prestaul/skeemas-markdown-validation)
- Redocly's own work on Markdown in OpenAPI lints the Markdown in `description` fields (formatting) and doesn't validate JSON examples in prose. — [Redocly blog: "Lint OpenAPI, lint Markdown. Now lint Markdown in OpenAPI."](https://redocly.com/blog/lint-markdown)
- A web search aimed at "validate JSON blocks in Markdown against OpenAPI schemas" turned up no ready-made tool. The pattern people describe is a homemade CI script that pulls out the fenced blocks and runs Ajv on them. — search summary, see [skeemas-markdown-validation](https://github.com/Prestaul/skeemas-markdown-validation) and [JSON Schema blog: Validating OpenAPI and JSON Schema](https://json-schema.org/blog/posts/validating-openapi-and-json-schema) (the second checks the OpenAPI document against its meta-schema, not examples in prose)
- Prism's validation proxy (`prism proxy`) forwards real traffic and reports request and response violations against the spec in an `sl-violations` header, giving location, severity, code, and message. It checks traffic, not documentation examples. — [Stoplight blog (vendor)](https://blog.stoplight.io/ensure-api-excellence-with-free-data-validation-tools-like-prism); [jvt.me walkthrough (2022)](https://www.jvt.me/posts/2022/03/04/openapi-local-prism/)
- Known Prism bug: issue #2527 reports that Prism rewrote a backend's 403 into a 401 security violation even though the spec defined the 403. — [stoplightio/prism#2527](https://github.com/stoplightio/prism/issues/2527)

### Inferences
- In-spec examples are a solved problem, though Redocly ships the rule at `warn` and Spectral at `error`. A team that wants examples in prose checked has to build it.
- The useful jump past skeemas-style tools is to work out the schema from context: the operation and status a page is about, or a `$ref` named in a directive. Hand-linking each block is the friction that keeps today's approaches from spreading.
- Redocly's `allowAdditionalProperties: false` default is a sensible precedent: extra keys in an example are usually drift, such as a renamed field.

### Gaps
- I found no measurement of how common stale examples are, in the spec or in prose. The evidence is anecdotal (see the HN quotes under Contract testing).
- Not confirmed: whether Spectral, Redocly, or openapi-examples-validator fetch and validate `externalValue` targets.
- I found no tool that checks request examples in SDK READMEs or tutorials against the spec. SDK READMEs that generators produce are presumably correct when generated, but nothing confirmed that they are tested.

## Code samples: generated versus hand-written, testing, and drift

### Takeaway
The common practice is to generate samples per operation from the SDK generator and inject them into the spec as `x-codeSamples` through an overlay. That keeps samples in step with the SDK but doesn't run them. Hand-written samples in guides are untested unless the team builds that. The market moved in 2026: Anthropic bought Stainless and shut down its hosted generator.

### Cited Findings
- `x-codeSamples` (formerly `x-code-samples`) "is a widely accepted spec extension that enables the addition of custom code samples". Each entry has `lang`, an optional `label`, and `source`. Docs tools "will render the usage snippet in the right-hand panel", including Mintlify, ReadMe, Redocly, and Stoplight. — [Speakeasy guide to x-codeSamples (vendor)](https://www.speakeasy.com/openapi/guides/x-codesamples.md)
- Speakeasy's guide names the drift problem: hand-written samples "may not correspond to a usage snippet from an existing SDK." Speakeasy generates a sample "for every `operationId`" and writes them to an overlay that a CI workflow regenerates. The guide doesn't describe running or testing the samples. — [Speakeasy guide to x-codeSamples (vendor)](https://www.speakeasy.com/openapi/guides/x-codesamples.md); [Speakeasy: generating code samples (vendor)](https://www.speakeasy.com/docs/sdks/sdk-docs/code-samples/generate-code-samples)
- Speakeasy says the overlay "is pushed to a registry, so it can be applied to the spec without changing the original source file", and it works with Scalar, Bump.sh, Mintlify, Redocly, and ReadMe. — [Speakeasy: generating code samples (vendor)](https://speakeasy.com/docs/code-samples/generate-code-samples); [Bump.sh blog on Speakeasy code samples (vendor)](https://bump.sh/blog/bump-speakeasy-api-docs-sdks/)
- Fern argues that Speakeasy's split model brings drift back "at the vendor boundary": samples, reference, guides, and SDK come from two vendors on separate release schedules. This is a competitor's view. — [Fern blog (competitor)](https://buildwithfern.com/post/speakeasy-reviews-pricing-alternatives)
- Anthropic acquired Stainless in mid-May 2026. Stainless had generated every official Anthropic SDK. It told TechCrunch it "will wind down all hosted Stainless products, including its SDK generator". Customers keep the SDKs they have generated. The price is reported at over $300M and wasn't confirmed. — [TechCrunch, 2026-05-18](https://techcrunch.com/2026/05/18/anthropic-has-acquired-the-dev-tools-startup-used-by-openai-google-and-cloudflare/); [InfoWorld](https://infoworld.com/article/4172947/anthropic-acquires-stainless-to-strengthen-claudes-developer-tooling.html); [Let's Data Science](https://letsdatascience.com/news/anthropic-acquires-stainless-shuts-shared-sdk-generator-b4e8a569)

### Inferences
- Generated samples are right about syntax and the SDK's shape, but nothing found runs them against the API. They are "not stale relative to the SDK", not "proven to work".
- Because samples arrive as an overlay, the spec a docs compiler reads may or may not include them, depending on whether the overlay was applied. A compiler that reads `x-codeSamples` should say which overlay it applied.
- The Stainless shutdown means some API teams in late 2026 are moving SDK generators, a moment when sample drift is likely.

### Gaps
- Not researched for lack of budget: httpsnippet (the HAR-to-snippet library behind many "curl / Python / Node" tabs), OpenAPI Generator's sample output, and Fern's snippet testing. I found no source showing any of these runs the generated samples.
- I found no source on teams testing hand-written guide snippets against a live or mock API (doctest-style) for REST APIs.

## Contract and docs testing: what Dredd, Schemathesis, Prism, Portman, Postman, and Optic prove about the docs

### Takeaway
These tools prove the spec matches the implementation. None proves the prose matches the spec. Dredd, the original "docs are tests" tool, was archived in November 2024. Optic was bought by Atlassian in April 2024 and folded into Compass. Schemathesis is the active open-source tool. Practitioners still describe drift as unsolved and lean on process: PR checklists, docs in the same repo, docs generated from tests.

### Cited Findings
- The `apiaryio/dredd` repository was "archived by the owner on Nov 8, 2024" and is read-only. OpenAPI 3 support was still marked "experimental". — [apiaryio/dredd on GitHub](https://github.com/apiaryio/dredd)
- Dredd's docs describe it as "a language-agnostic command-line tool for validating API description document against backend implementation of the API". — [Dredd documentation](https://dredd.readthedocs.io/)
- A maintained fork, `stacklych/dredd`, targets OpenAPI and drops API Blueprint and OpenAPI 2. — [Docker Hub stacklych/dredd](https://hub.docker.com/r/stacklych/dredd/)
- Atlassian announced it had acquired Optic in April 2024 (PitchBook gives 2024-04-25) and folded it into Compass. — [Atlassian blog](https://www.atlassian.com/blog/announcements/optic-acquisition); [PitchBook](https://pitchbook.com/profiles/company/232093-63)
- The npm package `@useoptic/optic` (MIT) says "Optic Labs is now part of Atlassian". An Atlassian Community answer from January 2025 says Optic's LintGPT "is no longer publicly available". — [npm @useoptic/optic](https://npmjs.com/package/@useoptic/optic); [Atlassian Community, Jan 2025](https://community.atlassian.com/forums/Compass-questions/LintGPT/qaq-p/2912984)
- APIs You Won't Hate covered the Optic acquisition under the heading "Killing Your Products" (newsletter, 2024-01-05 by its URL slug). — [APIs You Won't Hate newsletter](https://apisyouwonthate.com/newsletter/newsletter-1-5-24/)
- Schemathesis's coverage phase also sends HTTP methods the spec doesn't document. The `unsupported_method` check fires in those cases, which tests that the spec covers every method the server answers. — [Schemathesis checks reference](https://schemathesis.readthedocs.io/en/stable/reference/checks/)
- Specmatic (vendor) builds an "API coverage report" from the spec and the running app, flagging endpoints the app serves that the spec lacks and spec endpoints the app doesn't implement, and can fail CI. Karate Labs (vendor) also reports contract coverage and calls to endpoints missing from the spec. — [Specmatic (vendor)](https://specmatic.io/demonstration/detect-mismatches-between-your-api-specifications-and-implementation-specmatic-api-coverage-report/); [Karate Labs API Coverage (vendor)](https://karatelabs.io/api-coverage)
- Practitioners on "Ask HN: How do you manage the drift between implemented code and documentation" (2024-05-10):
  - lasereyes136: "If you expect the team to create documentation without explicit time to do so, expect it to not get done."
  - ponyous: "If you have pull request templates, add a checklist item: 'Update documentation'."
  - tacostakohashi: "documentation should live in the same repo as the code it relates to."
  - hitchstory, on docs generated from tests: "since it is generated from a test, never goes out of date by definition."
  - mlhpdx: "Use GitHub hooks, and for each change in the source code identify the relevant documentation" (an LLM check).
  - muzani: "tests are the best kind of doc."
  - [HN item 40317113](https://news.ycombinator.com/item?id=40317113)
- An older HN comment (2019): "a separate openapi spec that is not enforced can quickly become outdated, then an auto-generated from code is better." — [HN item 21290458](https://news.ycombinator.com/item?id=21290458)
- In the OpenAPI v4 (Moonwalk) thread (2023), a commenter describes the reverse: the spec is written, found insufficient, the code is changed, and the spec is left out of date. — [HN item 36144674](https://news.ycombinator.com/item?id=36144674)

### Inferences
- The tools form a chain: implementation to spec (Schemathesis, Prism, Specmatic, Dredd) and spec version to spec version (oasdiff, below). The missing link is spec to prose: guides, tutorials, and changelog text. That link is the docs compiler's territory.
- Optic, the tool closest to "docs-aware change review", went closed-source inside Compass. That leaves open-source room for spec-change review that knows about the docs.
- Practitioners' answers are mostly process, not tooling, which suggests no tool they trust covers the prose.

### Gaps
- Portman (Postman contract tests generated from OpenAPI) and Postman's contract testing weren't researched for lack of budget. I have no cited findings on them.
- Not confirmed whether Optic's open-source CLI is still maintained after the acquisition, or whether its GitHub repository is archived.

## Coverage: reporting operations, parameters, or error codes that no page documents

### Takeaway
"Coverage" in the API tools market means spec against implementation (which endpoints the spec misses) or tests against spec. I found no tool that reports which operations, parameters, or error codes no guide or tutorial mentions. Docs completeness is measured with lint rules on the spec itself, such as a description on each operation.

### Cited Findings
- Spectral's recommended ruleset includes `operation-description` (warn) and `operation-tag-defined` ("Operation tags should be defined in global tags", warn). These check completeness inside the spec only. — [Spectral OpenAPI rules reference](https://raw.githubusercontent.com/stoplightio/spectral/develop/docs/reference/openapi-rules.md)
- Specmatic and Karate (vendors) report coverage between spec and implementation, and between tests and spec. — [Specmatic (vendor)](https://specmatic.io/demonstration/detect-mismatches-between-your-api-specifications-and-implementation-specmatic-api-coverage-report/); [Karate (vendor)](https://karatelabs.io/api-coverage)
- Theneo's Elva (vendor) builds an API catalog from repository routes and writes OpenAPI for endpoints that have no spec. Its figures come from its own demo. — [Theneo blog (vendor)](https://www.theneo.io/blog/undocumented-api-endpoints)
- An arXiv paper (APISENSOR, 2026) found endpoints in Dify's network traffic that were missing from its API Reference. That is evidence that reference docs miss endpoints in real projects. — [arXiv 2603.23852](https://arxiv.org/pdf/2603.23852)
- DigitalAPI (vendor) describes an audit step that sorts APIs into documented, partly documented, and undocumented. Here "documented" means having a reference page. — [DigitalAPI blog (vendor)](https://www.digitalapi.ai/blogs/api-documentation-portal-how-to-build)
- oasdiff's `checks changelog coverage` "maps every possible edit to the checks that cover it". That is coverage of the diff tool's own rules, not of the docs. — [oasdiff README](https://github.com/oasdiff/oasdiff)

### Inferences
- A report of "operations, parameters, or error codes that no page other than the reference mentions" appears to be new. Nothing found does it.
- It works only if guides refer to operations in a form a machine can read (operationId directives, links to reference anchors). Plain prose mentions can't be counted reliably.

### Gaps
- I found no survey or practitioner account of how teams measure docs completeness beyond the reference.

## Versioning: multiple API versions in docs, availability extensions, and rendering

### Takeaway
The big public APIs use date-based versions: Stripe's rolling dates, now monthly versions plus twice-yearly named major releases, and GitHub's calendar versions in a header. Both build docs and changelogs from version metadata, not by hand. Stability and availability are marked with vendor `x-` extensions; no standard exists. Redocly's `x-badges` and oasdiff's `x-stability-level` are the cited examples. Each renderer or tool reads its own extension.

### Cited Findings
- Stripe (Brandur Leach, 2017-08-05): "we implement versioning with rolling versions that are named with the date they're released." Each backwards-incompatible change is a version change module containing documentation, a transformation, and the affected resource types. Responses are produced at the current version and transformed backwards. "Our API changelog is programmatically generated and receives updates as soon as our services are deployed." The API reference flags fields that changed since the logged-in user's pinned version. — [Stripe blog, 2017](https://stripe.com/blog/api-versioning)
- Stripe's process since 2024-09-30.acacia: monthly API versions with no breaking changes, and about twice a year a named major release (Acacia, Basil, and so on) that can break integrations. SDKs use semver, with minor versions for monthly releases and major versions for major releases. Cached doc pages disagree on the current version (2026-08-26.dahlia, 2026-09-30.endive). — [Stripe blog: new API release process](https://stripe.com/blog/introducing-stripes-new-api-release-process); [Stripe docs: API versions](https://docs.stripe.com/api-versions); [Stripe SDK versioning](https://docs.stripe.com/sdks/versioning)
- GitHub REST picks the version with the `X-GitHub-Api-Version` header, and the current docs show `2026-03-10` as the example. Requests without the header get `2022-11-28`. Versions are supported for 24 months after a newer one ships. Before a version closes, responses carry a `Deprecation` header with the closing date. Additive changes (new optional parameters, response fields, operations) go into all supported versions without a new version number. An unsupported version returns 410 in the current docs and 400 in older Enterprise Server docs. — [GitHub docs: API versions](https://docs.github.com/en/rest/about-the-rest-api/api-versions); [GHES 3.14 version](https://docs.github.com/en/enterprise-server@3.14/rest/about-the-rest-api/api-versions)
- Redocly's `x-badges` goes on operations, as an array of `{name, position, color}`. Badges show in the operation's header and the navigation, and Redocly's example is a "Beta" badge. Redoc 0.132.0 extended `x-badges` to parameters and schema properties. — [Redocly x-badges](https://redocly.com/docs/realm/content/api-docs/openapi-extensions/x-badges); [Redoc changelog](https://cdn.jsdelivr.net/npm/@redocly/redoc@0.132.0/CHANGELOG.md)
- oasdiff's `x-stability-level` takes draft, alpha, beta, or stable. At the default threshold (beta), "endpoints marked `draft` or `alpha` are excluded from breaking-change detection". Endpoints without the extension are "treated as `stable`". It reports `api-stability-decreased` and `api-stability-increased`, on properties too. — [oasdiff STABILITY.md](https://raw.githubusercontent.com/oasdiff/oasdiff/main/docs/STABILITY.md)
- Fern filters docs by audience: elements tagged in the spec are filtered per docs instance in `docs.yml`, down to single properties, and instance audiences control "which products and versions appear in each documentation instance". `x-fern-ignore` removes an element from every output. — [Fern docs: audiences (vendor)](https://buildwithfern.com/learn/docs/api-references/audiences)

### Inferences
- Every extension a team uses (`x-badges`, `x-stability-level`, `x-fern-availability`, `x-sunset`) is read by a different tool. A team that wants both a "Beta" badge in the docs and stability-aware diffing marks the same fact twice. A docs compiler could accept one source and project it onto each.
- Stripe's model, where a change record carries its own documentation and the changelog and per-version docs are built from it, is the main precedent for treating version changes as data. Stripe built it in-house, and I found no general-purpose tool that does the same.
- GitHub's rule that additive changes land in every supported version means a per-version doc build can't be "one spec per version" without copying those additions into each one.

### Gaps
- Not researched for lack of budget: Twilio, Adyen (per-version specs), Shopify (quarterly date versions), and Plaid. No cited findings.
- Not checked: how Scalar, Mintlify, ReadMe, or Bump.sh render deprecated or beta operations, or whether any reads `x-stability-level`. Docs renderers in general belong to another researcher.
- I found no standard OpenAPI field for plans or tiers (feature availability by plan), and no tool that renders one.

## Change tracking: breaking-change detection and changelogs

### Takeaway
oasdiff is the active open-source standard for diffing OpenAPI documents. It detects breaking changes, writes changelogs in several formats, has a GitHub Action, and offers a paid hosted service for approving changes in a PR. Bump.sh offers PR diff comments and a hosted changelog. Optic, the main rival in this space, went into Atlassian. Writers still have to turn a structural diff into a changelog people read, and Stripe's generated changelog shows the alternative: the change carries its own description.

### Cited Findings
- oasdiff ("Command-line tool to compare and detect breaking changes in OpenAPI specs") has these commands:
  - `breaking`: changes that break existing clients.
  - `changelog`: every change that can affect consumers, written for people to read.
  - `diff`: the full machine-readable diff, including documentation-only edits.
  - `summary`: counts of changes.
  
  Its other features:
  - Output in html, json, markdown, markup, text, or yaml, with customizable HTML and Markdown changelog templates.
  - Localization in en, ru, pt-br, and es.
  - Change fingerprints that give a change a stable ID across commits, and source locations for each change.
  - A JSON Schema for its breaking and changelog output.
  - Customizable rules, `allOf` merging, and endpoint matching across renamed path parameters.
  - A GitHub Action, and a hosted PR workflow for teams (paid) where each change is approved or rejected.
  - An MCP server at `api.oasdiff.com/mcp`.
  
  The repository has about 1.4k stars. — [oasdiff README](https://github.com/oasdiff/oasdiff)
- Bump.sh's GitHub Action generates a hosted API reference "with changelog and diff" from OpenAPI or AsyncAPI, comments the API diff on pull requests, and has `fail_on_breaking`. — [bump-sh/github-action](https://github.com/bump-sh/github-action)
- Bump.sh's own public changelog sorts each change as breaking or not. For example, it notes that removing a body property is breaking unless the property was deprecated first. — [developers.bump.sh/changes](https://developers.bump.sh/changes?page=3)
- Stripe generates its changelog from version change modules ("programmatically generated … as soon as our services are deployed"). — [Stripe blog, 2017](https://stripe.com/blog/api-versioning)
- GitHub tells integrators to read the breaking-changes changelog for the target version before changing the header. — [GitHub docs: API versions](https://docs.github.com/en/rest/about-the-rest-api/api-versions)

### Inferences
- A diff tool can say that `customer.email` became required, but not why or what to do about it. The "why" text is what writers add by hand. A compiler that pairs oasdiff's change IDs (fingerprints) with prose entries the writer keeps could check that every breaking change has a written entry.
- oasdiff's stable fingerprints and JSON Schema make it a workable input for a docs compiler, rather than reimplementing OpenAPI diffing.

### Gaps
- I found no cited practitioner complaints about writing API changelogs by hand (noise from doc-only diffs, missing context). Searches didn't surface them within budget.
- `openapi-diff` (OpenAPITools) and Redocly's diff features weren't researched.

## Deprecation and sunset documentation practices

### Takeaway
There are three layers, and none links to the others:
1. `deprecated: true` in OpenAPI, a design-time flag with no date.
2. HTTP headers: the `Deprecation` header (RFC 9745, a date) and `Sunset` (RFC 8594, a date), plus `Link rel="deprecation"` pointing at migration notes.
3. Tool-specific extensions such as oasdiff's `x-sunset`, which enforces grace periods in CI.

Docs must state the dates and migration path themselves.

### Cited Findings
- RFC 9745's `Deprecation` header carries a structured-field Date that "may be in the future (the resource context will be deprecated at that date) or in the past". It is not a boolean; one secondary blog wrongly calls it one. — [RFC 9745 diff](https://ftp.nic.ad.jp/rfc/authors/rfc9745-rfcdiff.html); [Zuplo example](https://zuplo.com/examples/http-deprecation)
- RFC 8594's `Sunset` header gives the date after which a resource is expected to stop responding, in HTTP-date format. Guides say the Sunset date should come after the Deprecation date, and pair both with `Link: <…>; rel="deprecation"`. — [Zuplo example](https://zuplo.com/examples/http-deprecation); [Gravitee guide](https://www.gravitee.io/corpus/gen-1889/api/api-deprecation-headers-and-sunset-communication-strategies.html)
- An IETF draft (draft-rmili-httpapi-deprecation-manifest) notes that OpenAPI's deprecation keywords "carry no date, and they describe the contract rather than a particular deployment of it". It proposes an `application/deprecations+json` manifest for deprecating individual fields in bodies, which the headers can't express. — [IETF datatracker](https://datatracker.ietf.org/doc/draft-rmili-httpapi-deprecation-manifest/)
- oasdiff's `x-sunset` gives the removal date and is optional by default. Moving it earlier is breaking. `--deprecation-days-stable` and `--deprecation-days-beta` make `x-sunset` mandatory and enforce a minimum grace period. Removing a deprecated endpoint before its sunset date is breaking; on or after it, or with no sunset set, it isn't. — [oasdiff DEPRECATION.md](https://raw.githubusercontent.com/oasdiff/oasdiff/main/docs/DEPRECATION.md)
- GitHub sends a `Deprecation` header with the closing date before an API version stops being supported. — [GitHub docs: API versions](https://docs.github.com/en/rest/about-the-rest-api/api-versions)
- API Evangelist (2026-08-14) writes about using OpenAPI Overlays for deprecation and migration. I didn't read the post in full, so I can't summarize its proposal. — [API Evangelist](https://apievangelist.com/2026/08/14/openapi-overlays-for-deprecation-and-migration-choreography/)

### Inferences
- The same deprecation can be stated in four places that must agree: `deprecated: true`, `x-sunset`, the runtime headers, and the docs' prose and changelog. Nothing found checks that they agree. A docs compiler could hold the date once and check or produce the rest.
- Deprecating a single field is badly served: OpenAPI's `deprecated` on a schema property has no date, and the headers work per resource.

### Gaps
- I found no practitioner data on how often deprecations ship without dates or migration guides.
- Not confirmed how docs renderers show `x-sunset` or Deprecation and Sunset dates, if at all.
