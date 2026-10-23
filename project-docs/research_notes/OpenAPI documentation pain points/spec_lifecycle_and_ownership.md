# OpenAPI spec lifecycle and ownership: where specs come from, who owns what, how they are checked, and how they drift

Researched 2026-10-22. Sources are dated where the date matters. Material from vendor blogs (Speakeasy, Bump.sh, Redocly, Postman) is marked as such; these vendors sell tooling for the problems they describe.

## Design-first vs code-first: how common each is, and what annotation-generated specs cost documentation

### Takeaway
No survey I could find splits teams into design-first and code-first. The big surveys measure "API-first", which is a broader organisational stance. What can be established is which spec version each framework emits by default. Many popular generators still default to OpenAPI 3.0, and some to Swagger 2.0 (October 2026). Their output is known to lack long-form descriptions and examples, gets nullability wrong in 3.1, handles polymorphism (`oneOf`) badly, and produces mechanical `operationId`s. A docs tool should expect specs that are structurally valid but thin in prose, and that are regenerated on every build.

### Cited Findings
- Postman 2025 State of the API (more than 5,700 respondents, 7th annual report): 82% of organisations have adopted some level of API-first, and 25% are fully API-first. 58% of respondents say they "document APIs" (create or update specifications). Only 17% use contract testing. 55% struggle with inconsistent documentation. 93% of API teams face collaboration blockers. The report gives no split between design-first and code-first, and no breakdown by spec format or OpenAPI version. — [Postman State of the API 2025](https://www.postman.com/state-of-api/2025/)
- Nordic APIs reports that the share of organisations claiming some API-first adoption rose from 66% (2023) to 74% (2024) to 82% (2025). Sources disagree on whether the "12% increase" for fully API-first organisations is percentage points or relative. — [Nordic APIs, Deep Dive into State of the API 2025](https://nordicapis.com/a-deep-dive-into-the-state-of-the-api-2025/)
- SmartBear's State of Quality API 2023 surveyed more than 1,100 respondents (late 2022 to early 2023), and 51% named API standardisation as their top issue. I could not extract any design-first figure from the PDF. — [Nordic APIs summary of SmartBear 2023](https://nordicapis.com/10-insights-from-the-state-of-software-quality-2023-report/); [SmartBear report PDF](https://assets.smartbear.com/m/5276eb5353c29969/original/State-of-Quality-API-2023-Report.pdf)
- Tom Johnson (I'd Rather Be Writing, page dated 2026-10-21): "It's common that engineers generate the OpenAPI spec." His own survey found answers "trending toward engineers or engineers plus tech writers working together." He objects that with code-first, "the spec can no longer serve as the blueprint for creating the API, and instead becomes an afterthought." He warns that if teams auto-generate and writers have no access to the source code, "your developers will be writing your reference docs." On his own projects, "developers haven't been that familiar with OpenAPI, so I usually create the OpenAPI specification document manually." — [I'd Rather Be Writing, Introduction to the OpenAPI specification](https://idratherbewriting.com/learnapidoc/pubapis_openapi_intro.html)
- Phil Sturgeon (Bump.sh blog, 2024-01-11) says generated output is "going to be missing long form descriptions" and is "probably going to be missing examples too". Annotation tools are often "stuck on older less useful versions of OpenAPI". Comments "could still completely fail to accurately describe the code below". Traffic sniffing "will generally only get things 90% right". — [Bump.sh, Code-first: How to Generate OpenAPI from Code](https://bump.sh/blog/code-first-openapi/) (vendor blog)
- Sturgeon (2020-03-30) on annotations: "Do not confuse proximity with accuracy." Annotations don't verify that the code behaves as described, so they still need contract testing. Combining design-first with annotated code creates multiple sources of truth. — [APIs You Won't Hate, There's No Reason to Write OpenAPI By Hand](https://apisyouwonthate.com/blog/theres-no-reason-to-write-openapi-by-hand/) (older, 2020)
- Default OpenAPI version emitted by each framework (API Evangelist, data snapshot 2026-10-08):
  - 3.1: FastAPI, Litestar, Django Ninja, springdoc, Huma, Quarkus, Elysia, utoipa, Scramble, ASP.NET Core on .NET 10.
  - 3.0: Swashbuckle, NSwag, NestJS, Hono, drf-spectacular, Micronaut, swagger-php, rswag, OpenApiSpex.
  - Swagger 2.0: fastify-swagger, swaggo, tsoa, grape-swagger.
  - 3.2: only ASP.NET Core on .NET 11.
  — [API Evangelist, A Year After OpenAPI 3.2, Most API Tooling Cannot Read It](https://apievangelist.com/2026/10/09/a-year-after-openapi-3-2-most-api-tooling-cannot-read-it/)
- Nullability with springdoc and OpenAPI 3.1 (issue opened 2025-11-14): springdoc emits no nullable marker, neither `nullable: true` nor `type: ["string","null"]`, even with `@Nullable` or `@Schema(nullable = true)`. `requiredMode = NOT_REQUIRED` had no effect. The maintainers pointed upstream to swagger-core #5001 and closed the issue as "invalid" on 2025-12-06. — [springdoc-openapi #3138](https://github.com/springdoc/springdoc-openapi/issues/3138)
- springdoc 1.6.9: putting `oneOf` on a controller body parameter doesn't create the schemas. The workaround is to annotate a shared superclass or interface. — [springdoc-openapi #1568](https://github.com/springdoc/springdoc-openapi/issues/1568) (via search summary)
- springdoc can mark a property required even when its annotations conflict (`@Null` alongside `@NotNull`). — [springdoc-openapi #3110](https://github.com/springdoc/springdoc-openapi/issues/3110) (via search summary)
- The .NET 10 native OpenAPI generator writes a nullable enum as a `oneOf` with a null-type branch instead of `nullable: true`. Most popular client generators then read it as a polymorphic or composite object. — [Webline Global, How to Fix .NET 10 OpenAPI Nullable Enum Client Generation](https://www.weblineglobal.com/blog/dotnet-10-openapi-nullable-enum-fix/)
- `operationId` quality in FastAPI: by default FastAPI builds the operationId from "the function name, the path, and the HTTP method", which gives client methods like `createItemItemsPost`. The docs recommend overriding `generate_unique_id_function`, and post-processing `openapi.json` to strip tag prefixes before generating clients. — [FastAPI docs, Generate Clients](https://fastapi.tiangolo.com/advanced/generate-clients/)
- Validator errors in real specs (ApiNotes, 2026-04-23; dataset described inconsistently as 7,039 "errors" or "specs"), ranked by occurrence:
  1. `nullable` deprecated, use a type array: 24,539
  2. must match exactly one schema in `oneOf`: 19,772
  3. must have required property `$ref`: 19,136
  4. additional properties: 7,232
  5. enum mismatch: 6,026
  6. type errors: 3,857
  7. broken `$ref`: 2,800
  8. missing required fields (schema, content, description): 805

  It doesn't say whether the specs were generated or hand-written. — [ApiNotes, I analyzed 7,039 OpenAPI spec errors](https://apinotes.io/blog/common-openapi-spec-errors-and-how-to-fix-them)
- Retrofitting annotations across a large legacy codebase is a long project, and a generated spec is only as accurate as its annotations are complete. — [DEV Community, Generate OpenAPI 3.2 from existing code](https://dev.to/jeff_pdc/generate-openapi-32-from-existing-code-express-fastapi-go-and-spring-compared-323e)

### Inferences
- The most common spec reaching a docs tool is likely a CI-generated 3.0 or 3.1 file with mechanical operationIds and sparse descriptions. Nullability is often encoded the 3.0 way (`nullable`) even in documents labelled 3.1, which the ApiNotes counts suggest. Nullable-as-`oneOf` will show up as false polymorphism in rendered docs.
- Regeneration on every build means any prose added to the generated file directly is lost. The docs layer has to treat the generated spec as read-only input.
- "API-first" in survey headlines should not be read as "design-first spec with writer-quality prose".

### Gaps
- No survey data splits design-first from code-first (Postman, SmartBear and the search results all lacked it). The SmartBear 2023 PDF could not be text-extracted in this session.
- I found no GitHub-issue evidence specific to drf-spectacular, Swashbuckle/NSwag, tsoa or NestJS polymorphism or description problems. Only springdoc and .NET 10 are sourced here.
- Criteo's engineering post "Can you trust your OpenAPI spec?" returned 403 and was not read.

## Description ownership: who writes `description` and `summary`, how writers edit and review them, and conflicts on regeneration

### Takeaway
Engineers typically own the spec, and in code-first shops the descriptions live in annotations or docstrings, where writers have to work in source code or not at all. The documented workarounds are three: writers edit the generated file and the diff is saved as an overlay; prose is kept in separate overlay files applied in CI; or the prose sits outside the spec in a user guide, which duplicates content. Each introduces a second source of truth that can drift from the first.

### Cited Findings
- Engineers commonly generate the spec. "Tech writers can play a key role in collaborating with the API team in producing the spec", and in spec-first work they can shape endpoint names and consistency. Writers "probably need to take the lead to guide engineers." Reference output lacks room for tutorials, auth and rate limits, so "you still have to provide a user guide". Keeping endpoint descriptions in both the spec and a guide risks the two falling out of sync. — [I'd Rather Be Writing](https://idratherbewriting.com/learnapidoc/pubapis_openapi_intro.html)
- Sturgeon (2024-04-08) presents overlays as the fix for technical writers working on code-generated specs. Writers add descriptions and guides "instead of having your changes overridden" on regeneration, and the overlay is applied in CI before publishing. Writers shouldn't have to learn every annotation framework. — [APIs You Won't Hate, Use OpenAPI Overlays Today](https://apisyouwonthate.com/blog/use-openapi-overlays-today/)
- Lorna Mitchell (2025-01-09) describes a workflow:
  1. Edit the generated OpenAPI file directly, adding rich descriptions, links and multiple examples.
  2. Run `speakeasy overlay compare` to produce an overlay from the diff.
  3. Commit the overlay and apply it in CI before the docs build, then lint the result.

  The problem it addresses is "maintaining rich and extensive content such as Markdown descriptions and examples in codebase annotations". Overlays grow as edits accumulate. — [lornajane.net, Save edits to OpenAPI as an Overlay](https://lornajane.net/posts/2025/save-edits-to-openapi-as-an-overlay)
- A SoftServe technical-communication team describes a custom Node.js and Gemini (LLM) merge that regenerates the spec from code while preserving manual edits. This is evidence that writers are building bespoke merge tooling. — [Medium/SoftServe, How to use AI to update OpenAPI files from code without losing manual edits](https://medium.com/softserve-technical-communication/how-to-use-ai-to-update-openapi-files-from-code-without-losing-manual-edits-aa18f344fe9e)
- OpenAPI Initiative best practices warn against keeping both a hand-written description and code annotations, because "newcomers to the project will not know which one is actually in use and mistakes will be made". They recommend a CI check that keeps the two consistent. — [learn.openapis.org Best Practices](https://learn.openapis.org/best-practices.html) (quote via search snippet)
- One guide advises keeping hand-written endpoint prose to a minimum, because "hand-written endpoint prose is the part that drifts". — [datadef.io, Keep API Docs in Sync With Code](https://datadef.io/guides/en/keep-api-docs-in-sync-with-code) (vendor-ish guide)
- GitLab's merge request to "Regenerate API file, document how it was done" shows a large project having to write down how regeneration is performed. — [GitLab MR 179892](https://gitlab.com/gitlab-org/gitlab/-/merge_requests/179892) (title only; contents not read)

### Inferences
- In code-first shops, a writer's edit path is either a code pull request against annotations (which means review by engineers, framework-specific syntax, and length limits in practice) or an out-of-band file such as an overlay, a Markdown sidecar or a docs-tool override. The out-of-band file is keyed by JSONPath or operationId, so it breaks silently when operationIds or paths change. FastAPI's default IDs, for example, embed the function name and path.
- Where prose lives outside the spec, review happens in two places: the code pull request and the docs pull request. Neither reviewer sees the combined result unless CI renders it.

### Gaps
- I found no Reddit (r/technicalwriting, r/ExperiencedDevs) or Write the Docs threads with first-person writer accounts of edit access or review. Search returned nothing relevant.
- I found no data on how often overlay targets break after regeneration.

## Linting and style: Spectral, Redocly CLI, Vacuum, and API style guides

### Takeaway
Linters can enforce documentation presence: an operation description or summary, parameter descriptions, tag descriptions, examples that are present and valid against the schema, and info contact and license. They cannot judge whether the prose is good. Practitioners complain that rulesets copied from public style guides are noisy and don't match how their team works, that a first run produces hundreds of findings that need waivers, and that Spectral is slow on large specs, lags on 3.2, and is described as dormant since SmartBear acquired Stoplight.

### Cited Findings
- Redocly CLI built-in rules relevant to docs (the page doesn't say which are in the recommended set):
  - presence: `operation-description`, `operation-summary`, `operation-operationId`, `parameter-description`, `tag-description`, `info-contact`, `info-license`
  - examples: `scalar-property-missing-example`, `no-invalid-media-type-examples`, `no-invalid-schema-examples`, `no-invalid-parameter-examples`
  - tags: `tags-alphabetical`, `operation-singular-tag`

  Severity is configurable as `error`, `warn` or `off`. — [Redocly CLI built-in rules](https://redocly.com/docs/cli/rules/built-in-rules)
- Kin Lane (API Evangelist, 2024-09-18): Spectral rules found in public design guides "trigger a lot of false positives" and "aren't aligned with the way your teams operate". Rules are "very atomic, technical, and empty of any strategic, operational, and business alignment". "You hit walls pretty fast trying to lint OpenAPIs using Spectral." He suspects people don't want to learn OpenAPI, JSON Schema, JSONPath or Spectral. — [API Evangelist, Why That Spectral Rule Is Not As Effective](https://apievangelist.com/2024/09/18/why-that-spectral-rule-is-not-as-effective-in-governing-your-apis-as-you-would-like/)
- A real CI adoption: Spectral with the OWASP ruleset "reported 365 findings, 135 of them errors" on its first run. Each was fixed or waived in `.spectral.yaml` "with the reason beside it". — [alrayyes/Hush-Hush PR #619](https://github.com/alrayyes/Hush-Hush/pull/619)
- Stoplight's docs admit that some Spectral OAS rules "may identify false positives" when the document is a library of reusable components referenced by other specs. — [Stoplight Spectral OpenAPI Rules](https://docs.stoplight.io/docs/spectral/4dec24461f3af-open-api-rules)
- Speakeasy (vendor, 2026-02-09; it built its own linter):
  - Spectral took 10.0 s on LaunchDarkly (48K lines), 29.97 s on DigitalOcean (73K) and 30.34 s on Plaid (81K), with peak RSS of 151 to 870 MB.
  - Spectral lacks built-in rules such as `oas3-example-missing`, and "silently ignores" 3.2 constructs. The post calls Spectral "largely dormant" since SmartBear acquired Stoplight.
  - Vacuum is faster (about 3 s on DigitalOcean) but "uses significantly more memory" (734 MB peak).

  — [Speakeasy, Why we built our own OpenAPI linter](https://www.speakeasy.com/blog/why-we-built-our-own-openapi-linter)
- API Evangelist's October 2026 test: Spectral "validates 3.2 against the 3.0 schema and fails it". — [API Evangelist, 2026-10-09](https://apievangelist.com/2026/10/09/a-year-after-openapi-3-2-most-api-tooling-cannot-read-it/)
- Vacuum claims to lint a 5 MB spec in about 230 ms (its own claim) and is compatible with Spectral rulesets. — [vacuum on pkg.go.dev](https://pkg.go.dev/github.com/daveshanley/vacuum@v0.9.7); [API Evangelist conversation with Dave Shanley, 2025-01-28](https://apievangelist.com/2025/01/28/api-evangelist-conversaton-with-dave-shanley-princess-beef-heavy-industries/index.md)
- Style guides:
  - Zalando's RESTful API Guidelines cover versioning, property naming and formats, and are enforced by Zalando's own linter Zally or a Spectral ruleset.
  - Adidas published one of the first style guides with a Spectral ruleset.
  - Microsoft's Azure API Style Guide ships a Spectral ruleset whose maintainers say findings "should be evaluated in the context of your service, and using your judgement".

  — [Zalando guidelines](https://opensource.zalando.com/restful-api-guidelines/); [Zally](https://providers.apievangelist.com/providers/zally/); [Azure API Style Guide](https://github.com/Azure/azure-api-style-guide); [Spectral README ruleset list](https://cdn.jsdelivr.net/gh/stoplightio/spectral@develop/README.md)
- Recommended rollout from a practitioner guide: start with `extends: ['spectral:oas']` and `--fail-severity=error`, add custom rules at `warn`, and tighten later. — [qaskills, Spectral OpenAPI Linting Guide 2026](https://qaskills.sh/blog/spectral-openapi-linting-guide-2026)

### Inferences
- Lint rules check that descriptions and examples exist, not that they are accurate or well written. A spec can pass `operation-description` with "Gets the item."
- Because waivers live in linter config, a docs tool receiving a spec cannot assume any documentation rule was enforced upstream.

### Gaps
- I found no measured false-positive rates for any linter. Speakeasy notes they "matter" but doesn't measure them.
- I didn't verify which Redocly rules the `recommended` config turns on, or at what severity.
- Google AIP linting (api-linter) targets protobuf, not OpenAPI, and was not researched.

## Overlays, $ref splitting and bundling, and large specs

### Takeaway
Overlay 1.0.0 shipped on 2024-10-17. Overlay 1.1.0 (adding a `copy` action) was completed in early 2026. Several tools now apply overlays: Bump.sh, Speakeasy, Redocly, Mintlify, libopenapi, openapi-format and openapi-overlays-js. Kin Lane still calls the tooling "early compared to core OpenAPI". The pain points are JSONPath, whose implementations are incompatible and which is hard for non-engineers, the unclear semantics of `update`, and fragile targets. Multi-file specs also hurt: bundlers are abandoned or have been taken in-house, tools can't resolve `$ref`, refs are circular, and large specs run to tens of thousands of lines and thousands of files.

### Cited Findings
- Overlay Specification 1.0.0 was released 2024-10-17 and announced 2024-10-22. Its use cases include updating descriptions (Markdown). — [Overlay v1.0.0](https://spec.openapis.org/overlay/v1.0.0.html); [OAI announcement](https://www.openapis.org/blog/2024/10/22/announcing-overlay-specification)
- Overlay 1.1.0 is now the "latest" version. It adds a `copy` property to the Action Object, which copies or moves an element. Upgrading mainly means changing the version number. The OAI's February 2026 newsletter marked it complete. — [Overlay latest (v1.1.0)](https://spec.openapis.org/overlay/latest); [Upgrading Overlay 1.0 to 1.1](https://learn.openapis.org/upgrading/overlay-v1.0-to-v1.1.html); [OAI Arazzo/Overlay news](https://www.openapis.org/tag/arazzo)
- Tooling (Kin Lane, 2026-06-25):
  - openapi-overlays-js (Lorna Mitchell) is the reference library, with no CLI and few contributors.
  - `bump overlay` is "probably the most-used entry point".
  - Others: openapi-format, overlays-cli (a prototype by Josh Ponelat), Speakeasy's openapi-overlay and openapi CLI, and libopenapi.
  - Redocly reaches similar goals through decorators rather than native overlays (as of that post).
  - Tooling is "still early compared to core OpenAPI".

  — [API Evangelist, OpenAPI Overlays Tooling Support](https://apievangelist.com/2026/06/25/openapi-overlays-tooling-support/)
- In an October 2026 test, Overlay 1.0 was applied by Bump.sh, Mintlify, Redocly, Speakeasy, libopenapi, openapi-format and openapi-overlays-js. — [API Evangelist, 2026-10-09](https://apievangelist.com/2026/10/09/a-year-after-openapi-3-2-most-api-tooling-cannot-read-it/)
- Problems Sturgeon reported in April 2024, before 1.0:
  - JSONPath "has several incompatible implementations", so tools should align with RFC 9535.
  - JSONPath is hard for non-technical users.
  - "update is unclear in its intention when it comes to trying to add things", which is like PUT vs PATCH. There is a proposal for `add` and `replace`.
  - Speakeasy's CLI was harder to install in GitHub Actions.

  — [APIs You Won't Hate, Use OpenAPI Overlays Today](https://apisyouwonthate.com/blog/use-openapi-overlays-today/)
- OpenSearch has an RFC to adopt overlays for distribution-specific filtering of its API spec. This is evidence of real use for audience variants, not prose. — [opensearch-api-specification #1183](https://github.com/opensearch-project/opensearch-api-specification/issues/1183)
- Speakeasy uses overlays heavily before SDK generation: stripping internal endpoints, injecting examples, and serving multiple audiences from one source. — [Speakeasy, Overlays best practices](https://www.speakeasy.com/openapi/overlays) (vendor)
- Bundling:
  - "Bundling has been struggling lately", with the most popular tools "mostly being abandoned or taken in-house".
  - JSON-pointer-style refs produced by bundlers are valid but confusing to humans, and break tools that build model names from `$ref`.
  - Heavy splitting risks circular references.

  — [APIs You Won't Hate, Popular OpenAPI Bundling Tools Compared](https://apisyouwonthate.com/blog/openapi-bundling-tools-compared-blog-post/)
- Splitting makes life hard "when tools aren't able to resolve the $refs themselves". — [Jamie Tanna, Bundling Multi-File OpenAPI Documents (2022, older)](https://jvt.me/posts/2022/02/10/bundle-openapi)
- Redocly CLI offers `split` (one file into many), `bundle` (many into one), and an experimental `join` (OpenAPI 3.x only). — [Redocly CLI file management](https://redocly.com/docs/cli/file-management)
- A project working with Datadog's v2 API spec describes it as about 70,000 lines, splitting into more than 3,000 schema files. — [Datadog MCP openapi-splitting doc](https://glama.ai/mcp/servers/@Believe-SA/datadog-mcp/blob/3245a6b1fa4eeaeaf3c17fda983b9dd8efffbe63/docs/openapi-splitting.md)
- Public specs from LaunchDarkly, DigitalOcean and Plaid are 48K, 73K and 81K lines. — [Speakeasy linter post](https://www.speakeasy.com/blog/why-we-built-our-own-openapi-linter)

### Inferences
- A docs tool may receive a bundled single file, a split multi-file tree, or a generated base file plus one or more overlays. It needs to say which it resolves itself and in what order overlays apply. Overlay 1.0 and 1.1 documents will coexist.
- An overlay is a second source of truth targeted by JSONPath. When the base spec regenerates, a target that no longer matches fails silently unless the tool reports a zero-match action.

### Gaps
- I found no adoption numbers for overlays, such as share of teams or download counts.
- I found no first-hand reports of overlay targets breaking after regeneration, though this is the obvious failure mode.
- I did not confirm whether Redocly's native overlay support, reported in October 2026, has replaced decorators.

## Spec/code drift: how often specs disagree with the implementation, how it is detected, and how it shows up in docs

### Takeaway
Academic checks of public specs find mismatches in nearly all of them: about 99% of more than 1,000 APIs.guru specs had some mismatch, and 40% of 91 produced errors against the live API (older studies, 2018 to 2022). Only 17% of teams use contract testing (Postman 2025). The detection tools are Schemathesis (property-based testing), the Prism validation proxy, Pact (consumer-driven), and traffic inference. The two best-known traffic-inference tools have been absorbed: Akita into Postman in 2023, and Optic into Atlassian, with its repository archived on 2026-01-12. Linting and spec diffing compare documents to documents and catch no implementation drift.

### Cited Findings
- Kim et al. (ISSTA 2022), as reported by a 2026 arXiv paper: "almost 99% of the 1,000+ specifications they checked on APIs.guru contained some mismatch." Ed-douibi et al. (EDOC 2018) "tested 91 OpenAPI specifications from APIs.guru and found that 40% produced errors when a correct client calls a real API", with errors "split between faults in the specifications themselves and the implementation". These are secondhand; the originals were not read. The 99% figure may originate in the Schemathesis authors' work rather than Kim et al. — [Mazhar, Wang, Mäntylä, Fault Injection in OpenAPI Specifications (arXiv 2607.12101, 2026-07-13)](https://arxiv.org/html/2607.12101v1)
- About 29.8% of 500 public APIs omit explanations of error messages. That is an omission, not a mismatch. — (via search summary of [RESTRuler / related arXiv work](https://arxiv.org/pdf/2402.13710); not verified in the original)
- Postman 2025: only 17% use contract testing. — [Postman State of the API 2025](https://www.postman.com/state-of-api/2025/)
- One project running Schemathesis in CI found real drift between API and spec on 4 of 7 check categories, after earlier CI failures had hidden it. It needed an experimental flag to load an OpenAPI 3.1.0 spec. — [xavisavvy/scrum-monsters #200](https://github.com/xavisavvy/scrum-monsters/issues/200)
- "Linting and diffing only compare documents to documents; neither catches implementation drift." Prism can sit in front of a live API and validate request and response traffic against the spec. — [spec-coding.dev, Contract Testing Plan](https://spec-coding.dev/blog/contract-testing-plan-from-openapi-to-ci) (practitioner blog)
- Optic ("OpenAPI linting, diffing and testing", which could generate a spec from test traffic) was archived on 2026-01-12. "Optic Labs is now part of Atlassian." — [opticdev/optic](https://github.com/opticdev/optic)
- Postman acquired Akita Software, which inferred APIs passively from traffic using BPF, in July 2023, to fold it into Postman's platform. — [BusinessWire, 2023-07-19](https://www.businesswire.com/news/home/20230719991128/en/Postman-Acquires-Akita-Software-to-Help-Build-the-Future-of-API-Observability)
- With annotations, "Developers can forget to make the changes, and developers can make mistakes." Traffic sniffing gets "90% right". — [APIs You Won't Hate (2020)](https://apisyouwonthate.com/blog/theres-no-reason-to-write-openapi-by-hand/); [Bump.sh (2024)](https://bump.sh/blog/code-first-openapi/)
- Fern (vendor, August 2026) frames "schema drift" as SDKs and docs diverging, and argues for generating both from one spec. — [Fern, Stopping schema drift](https://buildwithfern.com/post/stopping-schema-drift-coupling-sdks-documentation-claude) (vendor)
- Specmatic markets an "API coverage report" to detect mismatches between specs and implementation. — [Specmatic](https://specmatic.io/demonstration/detect-mismatches-between-your-api-specifications-and-implementation-specmatic-api-coverage-report/) (vendor)

### Inferences
- In docs, drift shows up as response schemas missing fields the API returns, required or nullable flags that are wrong (compare the springdoc nullable bug), undocumented error responses, and examples that don't validate. Generated specs drift less in shape, but are no more accurate in descriptions, which are free text in annotations.
- A docs compiler can catch drift inside the spec (examples against schemas, broken `$ref`s, overlay targets), but not drift between the spec and the running API without traffic or tests.

### Gaps
- I found no recent (2024 to 2026) industry survey quantifying drift in private or internal specs.
- I didn't verify whether Optic's capabilities continue inside an Atlassian product.

## OpenAPI version churn: Swagger 2.0, 3.0, 3.1, 3.2, and Arazzo

### Takeaway
By October 2026, a year after OpenAPI 3.2 (September 2025), most tools still top out at 3.1. Swagger 2.0 is still read by about as many tools as 3.1, and is the default output of tsoa, swaggo, fastify-swagger and grape-swagger. Spectral, ReadMe, Fern and the JavaScript swagger-parser reject or fail 3.2, and Redoc silently omits 3.2 operations. A docs tool will receive all four versions, and the 3.0-to-3.1 nullability change is the most common source of invalid documents.

### Cited Findings
- OpenAPI 3.2.0 was released around 2025-09-19, with the OAI blog post dated 2025-09-23. It is backward-compatible with 3.1. New features:
  - hierarchical tags (`summary`, `parent`, `kind`), replacing Redocly's `x-tagGroups`
  - the `QUERY` method and `additionalOperations`
  - streaming and sequential media types via `itemSchema` and `prefixEncoding` (SSE, JSON Lines)
  - the OAuth device authorization flow

  — [Redocly, OpenAPI 3.2](https://redocly.com/blog/openapi-3-2); [Nordic APIs, What's New in 3.2](https://nordicapis.com/whats-new-in-openapi-specification-v3-2-0/); [Speakeasy OpenAPI release notes](https://www.speakeasy.com/openapi/release-notes); [learn.openapis.org Upgrading 3.1 to 3.2](https://learn.openapis.org/upgrading/v3.1-to-v3.2.html)
- A 3.2.1 patch spec exists. — [OpenAPI Specification v3.2.1](https://spec.openapis.org/oas/v3.2.1.html)
- API Evangelist, data snapshot 2026-10-08: 137 products reviewed, 50 run hands-on.
  - Reading, across 95 products whose versions could be established: Swagger 2.0 74, OpenAPI 3.0 91, 3.1 76, 3.2 36.
  - Newest version read: 3.1 for 40 products, 3.2 for 36, 3.0 for 16.
  - 3.2 status across 133 products: 33 supported (only about 15 say so themselves), 18 partial, 30 unsupported, 52 unknown.
  - Supporting 3.2: Swagger UI, Scalar, Bump.sh, Mintlify, Kiota, libopenapi, drf-spectacular and utoipa, among others.
  - Partial: Redoc builds pages but omits 3.2 operations. Postman's converter supports 3.2 but Spec Hub's docs stop at 3.1. Generated clients from openapi-typescript, orval, hey-api and oapi-codegen lack `QUERY`.
  - Unsupported: ReadMe ("OpenAPI 3.2 is currently unsupported"), Spectral, Fern, the JavaScript swagger-parser (and swagger-cli and orval downstream of it), Azure APIM and AWS API Gateway.
  - The article's counts are internally inconsistent in places (45 vs 56 hosted vendors).

  — [API Evangelist, A Year After OpenAPI 3.2, Most API Tooling Cannot Read It](https://apievangelist.com/2026/10/09/a-year-after-openapi-3-2-most-api-tooling-cannot-read-it/)
- The Java swagger-parser models were still on 3.1 while the JS tooling had shipped 3.2. — [swagger-parser #2248](https://github.com/swagger-api/swagger-parser/issues/2248)
- SmartBear's Swagger Studio lists 3.2 support as beta. — [SmartBear Swagger Studio 3.2 support](https://support.smartbear.com/swagger/studio/docs/en/openapi-3-2-support.html)
- `nullable` was deprecated in 3.1 in favour of JSON Schema type arrays. It is the most frequent validator error seen (24,539 occurrences), followed by `oneOf` failures (19,772). — [ApiNotes, 2026-04-23](https://apinotes.io/blog/common-openapi-spec-errors-and-how-to-fix-them)
- Schemathesis needed an experimental flag to load a 3.1.0 spec in one CI setup. — [scrum-monsters #200](https://github.com/xavisavvy/scrum-monsters/issues/200)
- The official swagger-codegen-cli did not support 3.1.0 as of the cited post. — [Jamie Tanna (2022)](https://jvt.me/posts/2022/02/10/bundle-openapi) (older)
- A tool exists to upgrade a spec to 3.2 automatically. — [APIs You Won't Hate, Automatically Upgrade to OpenAPI v3.2](https://apisyouwonthate.com/blog/automaticly-upgrade-to-openapi-v3-2/)
- Arazzo 1.0 (workflows) is consumed by Redocly, Speakeasy, Spectral, Stoplight, Bump.sh and libopenapi. No gateway, portal or MCP host was found supporting it as of October 2026. — [API Evangelist, 2026-10-09](https://apievangelist.com/2026/10/09/a-year-after-openapi-3-2-most-api-tooling-cannot-read-it/)
- Guidance: don't pick 3.2 just because it is the latest if a required tool supports only 3.0 or 3.1. — (via search summary; [Mintlify, OpenAPI vs Swagger](https://www.mintlify.com/library/openapi-vs-swagger))

### Inferences
- The version a team writes is constrained by the weakest tool in its pipeline: linter, SDK generator, gateway or docs renderer. Spectral failing 3.2 and ReadMe rejecting it keep many teams on 3.1 or 3.0 even when their framework can emit 3.2.
- Documents labelled 3.1 often contain 3.0 idioms (`nullable`, `example` vs `examples`), because frameworks or hand edits mix them. A tolerant reader with clear diagnostics will be more useful than a strict one.
- 3.2's hierarchical tags affect navigation directly. Until renderers support them, teams will keep using `x-tagGroups` and other vendor extensions.

### Gaps
- I found no reliable share-of-specs-by-version figure for real-world specs (as opposed to tool support).
- I found no first-hand practitioner reports of using Arazzo in documentation.
