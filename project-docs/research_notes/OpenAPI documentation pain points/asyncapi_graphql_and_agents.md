# Adjacent areas: AsyncAPI and GraphQL documentation, and how AI agents consume API specs and docs

Researched 2026-10-22. Most sources in this area are vendor blogs and comparison pages written by competitors (Fern, Mintlify, Scalar, Speakeasy, Stainless, Redocly, Bump.sh). They are labelled **(vendor)** below. Agent tooling (MCP generation, llms.txt, Code Mode) changes from month to month, so claims marked **(fast-moving)** should be checked again before the report is published.

---

# PART A: AsyncAPI and GraphQL

## A1. AsyncAPI 3.x: state of docs tooling, adoption, and pain points

### Takeaway
AsyncAPI docs tooling works but trails OpenAPI. The 3.0 spec (December 2023) broke compatibility, and parts of the ecosystem still haven't caught up almost three years later: Bump.sh, for example, still lists support only "up to 2.6". Most OpenAPI documentation pains carry over unchanged: generated reference pages with thin descriptions, prose that has to be written separately, and versioning. AsyncAPI adds its own: channels and operations are separate objects, channel addresses differ from channel IDs, protocol bindings vary by broker, and connection lifecycle, auth and error handling sit outside the spec's reference model.

### Cited Findings
- **Spec timeline.** AsyncAPI 3.0.0 was released on 2023-12-05 as a breaking change, with a migration guide — [asyncapi/spec releases](https://github.com/asyncapi/spec/releases); [3.0.0 reference](https://www.asyncapi.com/docs/reference/specification/v3.0.0).
- **Tooling was the known bottleneck.** Before the release, maintainers wrote that 3.0 meant adapting all of their tools. Parsers came first and the rest of the tooling followed — [AsyncAPI blog, "AsyncAPI Spec 3.0 is coming!"](https://www.asyncapi.com/blog/async-api-spec-3.0-release).
- **Early adoption.** A March 2024 community update called adoption "amazing". It named Microcks as the first CNCF Sandbox open-source tool to officially support v3 — [AsyncAPI Community Update, March 2024](https://www.asyncapi.com/blog/2024-march-summary).
- **Migration is still rough.** The official converter's docs label 2.x-to-3.x conversion as work in progress. They also note that in 3.0 a channel's identifier is no longer its address — [asyncapi/converter-js](https://github.com/asyncapi/converter-js). Scalar ships an upgrader that takes AsyncAPI 1.x, 2.x or 3.0 documents to 3.1.0 **(vendor)** — [Scalar AsyncAPI Upgrader](https://scalar.com/tools/asyncapi-upgrader/getting-started). That page is the only source found that names a 3.1.0 version; check it against the spec repo.
- **Official HTML docs: the generator and html-template.**
  - `@asyncapi/html-template` produces static docs and uses the AsyncAPI React component internally — [asyncapi/html-template](https://github.com/asyncapi/html-template).
  - Its latest release is v3.5.6 (14 March 2026). The v3.5 line pairs with Generator v3 and CLI v5, and older template versions need different generator versions. That version matrix is a usability cost in itself — [html-template README](https://github.com/asyncapi/html-template).
  - Patch releases in 2026 bumped `@asyncapi/react-component` to 3.0.3 (March 2026) — [html-template releases](https://github.com/asyncapi/html-template/releases).
  - A dedicated issue tracked v3 support in html-template, and it linked to a matching React component issue (asyncapi-react#733) — [html-template #430](https://github.com/asyncapi/html-template/issues/430). Its current status was not confirmed.
- **Older rendering gaps.** A 2021 unification proposal listed things the HTML template didn't render, including `correlationId` and whether a schema is circular. Bindings and extensions rendering was still being brought over to the React component at the time — [html-template #51](https://github.com/asyncapi/html-template/issues/51); [April 2021 at AsyncAPI](https://www.asyncapi.com/blog/april-2021-at-asyncapi). These are dated (2021) and not specific to v3.
- **Stale official tool listing.** The asyncapi.com tools page entries were about 1,232 days old when found by search — [asyncapi.com/tools](https://www.asyncapi.com/tools) (staleness as reported in search metadata).
- **EventCatalog** **(vendor, open-source project)**:
  - It imports AsyncAPI specs and builds a catalog of messages, channels, services and domains. It adds architecture diagrams, schema browsing and Markdown enrichment that is "kept in sync when specs are re-imported".
  - Docs are versioned automatically when specs change.
  - It reports 33,000+ catalogs created (a banner on the same page says 24,000+), about 2.9k GitHub stars, and named users PayFit, Nike, Ticketmaster, GOV.UK and AWS. These are self-reported and the page contradicts itself — [EventCatalog AsyncAPI](https://www.eventcatalog.dev/specifications/asyncapi).
  - Its model is organised around domains, services and messages, not around a single spec file. That is a different shape from an API reference.
- **Bump.sh** **(vendor)** supports OpenAPI 2.0–3.2, but AsyncAPI only "up to version 2.6". It does not support GraphQL ("may come later") — [Bump.sh FAQ](https://docs.bump.sh/help/faq/). OpenAPI and AsyncAPI docs can be published in the same portal — [Bump.sh vs RapiDoc](https://bump.sh/compare/rapidoc/).
- **Redocly** **(vendor)** renders AsyncAPI descriptions, with 3.0.0 reported as supported — [Redocly: Add AsyncAPI descriptions](https://redocly.com/docs/realm/content/api-docs/add-asyncapi-docs). Fern's July 2026 guide calls Redocly's AsyncAPI support "early stage" **(competitor claim)** — [Fern AsyncAPI guide](https://buildwithfern.com/post/asyncapi-documentation-generator).
- **AsyncAPI-specific gaps, according to Fern (vendor, July 2026)** — [Fern AsyncAPI guide](https://buildwithfern.com/post/asyncapi-documentation-generator):
  - It names the Generator, EventCatalog, Bump.sh and Modelina as the main open-source options. In Fern's view they cover rendering well and client generation poorly.
  - Generated docs usually omit connection lifecycle, auth and error handling, which need hand-written prose.
  - Only the search summary of this page was read; the full page did not load.
- **Maturity gap (weak source).** A 2026 comparison blog claims AsyncAPI tooling is "5–10x less mature" than OpenAPI's, and that portals such as ReadMe and Redocly support OpenAPI better than AsyncAPI — [misar.blog](https://www.misar.blog/compare/openapi-vs-asyncapi). The source is unsupported and vendor-style, so use it only as colour.
- **Older listings.** Widdershins and api2html appear in a Nordic APIs list of AsyncAPI generators — [Nordic APIs](https://nordicapis.com/8-asyncapi-documentation-generators/). Both are older tools whose maintenance wasn't checked.

### Inferences
- **OpenAPI pains that recur in AsyncAPI:**
  - Generated reference with thin descriptions: the same generator-first workflow, and the same reliance on description fields.
  - The narrative/reference split: lifecycle, auth and errors need prose, per Fern.
  - Versioning: EventCatalog's automatic versioning on re-import is a selling point, which implies others don't do it.
  - Examples: message examples live in the spec, just as OpenAPI's do.
- **Pains new to AsyncAPI:**
  - Two-sided semantics: whether the application sends or receives, which 3.0 models as an operation `action`.
  - Channel IDs vs addresses (new in 3.0).
  - Protocol bindings that vary by broker (Kafka, AMQP, MQTT, WebSocket).
  - A system-level view (which service publishes or consumes which message) that matters more than the per-endpoint view. EventCatalog's domain/service/message model exists because of this.
- **Version lag is a design risk.** Even established vendors (Bump.sh) lag the spec by years. A compiler that adds AsyncAPI should target 3.x directly and offer, or point to, a 2.x upgrade path, not support both.

### Gaps
- No independent AsyncAPI adoption figures were found. Postman's State of the API AsyncAPI numbers did not surface, and no download counts were checked.
- No first-hand writer or engineer complaints (GitHub discussions, Reddit, HN) about AsyncAPI docs were found in the searches run. The pain points above come from vendors and maintainers.
- The current status of v3 bindings rendering in the React component, and of html-template issue #430, wasn't confirmed.
- The AsyncAPI React component was not examined beyond its dependency version.

## A2. GraphQL: schema as documentation, tooling, and pain points

### Takeaway
In GraphQL the schema, with its `description` strings and `@deprecated`, is the reference, and introspection makes it browsable for free (GraphiQL, Voyager, SpectaQL, Magidoc, graphql-markdown, Redocly). What's missing is everything outside the schema: task guides, auth, rate limits, errors, and example operations. No documentation generator found validates the example queries in a guide against the schema. Tools that validate operations exist (graphql-eslint, GraphQL Inspector, graphql-validator), but they target code files, not Markdown.

### Cited Findings
- **Introspection covers the "what"; guides are still needed for the "how".**
  - GraphQL.org: interactive tools like GraphiQL let developers explore the fields and types an API exposes. Guides are still needed to tie the general concepts to a particular use case — [graphql.org best practices](https://graphql.org/learn/best-practices/).
  - A secondary guide calls over-reliance on schema-first documentation a common pitfall. It describes the schema as "the foundation, not the entire edifice" — [adhdecode](https://adhdecode.com/api-architecture/api-documentation/api-documentation-graphql/) (low-authority blog).
- **The usual real-world structure: a reference split by kind of element, plus separate guides.** GitHub's GraphQL docs split the schema reference by category (queries, mutations, objects, and so on) and keep guides for authentication and rate limits separate — [GitHub GraphQL docs](https://docs.github.com/en/graphql); [About the GraphQL API](https://docs.github.com/en/graphql/overview/about-the-graphql-api).
- **Authoring guidance.** Totara's guidance says to document every schema element: queries, mutations, arguments, enums, scalars, interfaces and unions — [Totara GraphQL authoring best practices](https://totara.atlassian.net/wiki/spaces/DEV/pages/121187530/GraphQL+documentation+authoring+best+practices).
- **Schema evolution and deprecation are hard.** Hygraph's example: changing a string `author` field into an object breaks clients that expect the original type **(vendor)** — [Hygraph, GraphQL pain points](https://hygraph.com/blog/graphql-pain-points).
- **SpectaQL.** It builds static HTML docs from a live endpoint, an introspection result file, or SDL files. Examples come either from static metadata or from a custom generator plugin, so they are supplied by hand rather than checked — [anvilco/spectaql](https://github.com/anvilco/spectaql). One team preferred pointing it at local schema files over introspecting a deployed API — [Atomic Object](https://spin.atomicobject.com/graphql-api-documentation/).
- **Magidoc** generates static docs from any GraphQL schema and can add custom pages written in Markdown — [magidoc README](https://github.com/magidoc-org/magidoc/blob/main/README.md).
- **graphql-markdown.**
  - It writes Markdown and MDX files that are committed alongside an existing docs site (Docusaurus), unlike SpectaQL and GraphDoc, which produce standalone sites — [graphql-markdown.dev](https://graphql-markdown.dev/docs/advanced/examples) (project's own framing).
  - It has an examples feature, but the search did not show that examples are validated.
- **Redocly** **(vendor)** turns GraphQL schema files into reference docs showing queries, mutations and types. It lists GraphQL APIs in a catalog alongside OpenAPI and AsyncAPI APIs — [Redocly: Add GraphQL descriptions](https://redocly.com/docs/realm/content/api-docs/add-graphql-docs).
- **Tools exist to validate operations against a schema, but not in Markdown:**
  - graphql-eslint works on `.graphql` files, `gql` template literals and `/* GraphQL */` comments. With a schema loaded, it validates operations fully — [graphql-eslint docs](https://the-guild.dev/graphql/eslint/docs); [Introducing GraphQL-ESLint](https://the-guild.dev/blog/introducing-graphql-eslint).
  - Its old `validate-against-schema` rule was removed. Maintainers point users to GraphQL Inspector's `validate` instead — [graphql-eslint discussion #456](https://github.com/graphql-hive/graphql-eslint/discussions/456).
  - creditkarma/graphql-validator is a CLI that parses each query in a file glob and validates it against the schema — [graphql-validator](https://github.com/creditkarma/graphql-validator).
  - None of these documents linting fenced `graphql` blocks in Markdown — same sources.
- **Agents push GraphQL toward curated operations too** (see Part B). Apollo MCP Server exposes either predefined operations (from `.graphql` files, operation collections, or persisted query manifests) or introspection tools. The introspection tools are `introspect` and `execute`, plus a `search` tool that "can result in fewer tool calls than introspect" when the wanted type is deep in the schema **(vendor; fast-moving: Apollo's doc versions disagree on the exact tool set)** — [Apollo MCP Server: Define tools](https://www.apollographql.com/docs/apollo-mcp-server/define-tools).

### Inferences
- **GraphQL pains, mapped to OpenAPI:**
  - **No endpoints to organise around.** Reference generators fall back to organising by kind of schema element (Query, Mutation, Object, Enum…), as GitHub's docs do, which is not organised by task. The narrative/reference split is sharper than in OpenAPI: guides are the only place task structure lives.
  - **Mutations vs guides.** A mutation's reference entry is an argument list. The workflow lives in a guide, which embeds operations as code blocks.
  - **Examples.** OpenAPI examples sit in the spec next to the operation. GraphQL example operations live in guide prose, outside the schema, so they drift unchecked. This is the clearest opening for a docs-as-code compiler: validate fenced `graphql` blocks against the project's SDL at build time, which no tool found does.
  - **Deprecations.** `@deprecated(reason:)` is machine-readable, so a compiler could flag guide examples that use deprecated fields.
  - **Schema size.** Large schemas such as GitHub's and Shopify's make per-type pages numerous. The Apollo `search` tool shows that even agents struggle to navigate deep type hierarchies.
- **Recurring theme.** Thin descriptions are the same problem as in OpenAPI, because generated reference pages are only as good as the `description` strings, which are often written by engineers in code-first schema builders.

### Gaps
- No direct evidence was found on whether teams actually validate example queries in guides against the schema (surveys, or docs repos doing it). Large docs teams such as GitHub and Shopify might do this in internal tooling; their docs repos weren't inspected.
- GraphiQL's Doc Explorer, GraphQL Voyager, Hive's explorer and Apollo Studio's explorer were not researched individually. Their role (interactive browsing of the schema) is described only generally by graphql.org.
- No first-hand practitioner complaints (HN, Reddit) were gathered.

## A3. Is there a common abstraction across OpenAPI, AsyncAPI, GraphQL, and gRPC/protobuf docs?

### Takeaway
The vendors that cover several formats (Redocly, Fern, Bump.sh, partly EventCatalog) share a catalog and portal layer across formats, but each format gets its own reference renderer. No tool found renders all of them through one reference model. The layer worth generalising is a catalog of APIs, page placement and navigation, prose that links into the reference, and checked examples. The reference model itself should stay specific to each format.

### Cited Findings
- **Redocly** **(vendor)** gives OpenAPI, AsyncAPI and GraphQL separate "add descriptions" pages, each rendered separately, and unifies them in a catalog — [Redocly GraphQL](https://redocly.com/docs/realm/content/api-docs/add-graphql-docs); [Redocly AsyncAPI](https://redocly.com/docs/realm/content/api-docs/add-asyncapi-docs). A third-party comparison says it also handles SOAP — [GitBook roundup](https://www.gitbook.com/blog/best-docs-as-code-platforms-api-teams) **(competitor)**.
- **Fern** **(vendor)** lists OpenAPI, gRPC/protobuf, AsyncAPI, OpenRPC and its own Fern Definition as first-class inputs — [Fern multi-protocol post, July 2026](https://buildwithfern.com/post/multi-protocol-api-platforms-rest-grpc). Its docs say protobuf can generate documentation, and that gRPC SDK generation is only for .NET/C# — [Fern: What is gRPC?](https://buildwithfern.com/learn/api-definitions/grpc/overview). Whether Fern supports GraphQL is unconfirmed: one third-party list claims it, Fern's own docs don't — [GitBook roundup](https://www.gitbook.com/blog/best-api-docs-sdk-generation-tools).
- **Bump.sh** covers OpenAPI and AsyncAPI only, and explicitly not GraphQL — [Bump.sh FAQ](https://docs.bump.sh/help/faq/).
- **EventCatalog** focuses on AsyncAPI and events. The AsyncAPI page doesn't mention OpenAPI or GraphQL, and xRegistry support is "coming soon" — [EventCatalog AsyncAPI](https://www.eventcatalog.dev/specifications/asyncapi).

### Inferences
- **The formats differ in their main unit:** an operation on a path and method (OpenAPI), a channel with operations and messages (AsyncAPI), a type graph with root fields (GraphQL), and a service with RPC methods and messages (protobuf). A shared abstraction that fits all four would likely be the lowest common denominator, roughly "a named operation with input and output schemas and a description".
  - That fits OpenAPI, gRPC and AsyncAPI operations reasonably well.
  - It fits GraphQL poorly, because GraphQL's useful documentation unit is the type, and the operation is chosen by the client.
- **The concerns that do generalise across formats:**
  1. Referencing an API element from prose by a stable identifier.
  2. Checking that references and examples resolve and validate against the source schema.
  3. Placing generated reference pages in the site's navigation.
  4. Description-quality lints, such as an empty description or a summary that repeats the name.
  5. Versioning and deprecation display.
  6. Agent-facing exports (Part B).

### Gaps
- The searches found no open-source tool that renders all four formats. Whether Scalar, ReadMe or Mintlify support AsyncAPI, GraphQL or protobuf wasn't checked.
- Protobuf docs tooling (Buf Schema Registry docs, protoc-gen-doc) wasn't researched.

---

# PART B: How AI agents consume API specs and docs

## B1. llms.txt, llms-full.txt, and Markdown exports for API docs

### Takeaway
Docs platforms have made llms.txt and per-page Markdown (`.md` URLs) standard: Mintlify, Fern, Scalar, and Redocly behind a toggle. Measured requests from AI crawlers for llms.txt are close to zero, though. The more defensible value is clean Markdown that agents and coding tools fetch on demand, and passive context such as an AGENTS.md docs index. How API reference is represented inside llms.txt (one link per operation, or per tag, or a link to the raw spec) is not standardised. **(fast-moving)**

### Cited Findings
- **Platform offerings (all from vendor comparison pages; check each against its own docs):**
  - **Mintlify:** every page is available as Markdown by appending `.md` to its URL. It generates `llms.txt`, `llms-full.txt` and `skill.md`, and hosts a search MCP server — [Mintlify, best llms.txt platforms](https://www.mintlify.com/library/best-llms-txt-platforms) **(vendor)**. Speakeasy, a competitor, says Mintlify's OpenAPI integration is less robust, with "opaque error messages, and incorrect validation failures" — [Speakeasy, Choosing a docs vendor](https://www.speakeasy.com/blog/choosing-a-docs-vendor) **(competitor)**.
  - **Fern:** llms.txt at the root and in each navigation section, plus `.md` page URLs. Sources disagree on whether Fern produces llms-full.txt: one says it deliberately skips it and has agents fetch pages one at a time, another says it auto-generates both — [Context7 platform list](https://context7.com/llms-txt-documentation-platforms); [Mintlify](https://www.mintlify.com/library/best-llms-txt-platforms). **Conflict unresolved.**
  - **Scalar:** reported to generate llms.txt and expose a docs MCP server, with an MIT-licensed renderer — [Scalar, Best API documentation tools 2026](https://scalar.com/library/best-api-documentation-tools-2026) **(vendor)**.
  - **Redocly:** llms.txt generation behind a toggle. Its Markdown support is reported as limited, and the company has publicly questioned whether any LLM provider reads llms.txt unprompted — [Mintlify, Redocly alternatives](https://www.mintlify.com/library/redocly-alternatives) **(competitor; secondhand)**.
  - **Bump.sh:** public docs with "Ask AI" enabled expose an MCP server at the docs URL plus `/mcp` — [Bump.sh FAQ](https://docs.bump.sh/help/faq/) **(vendor)**.
- **Crawler evidence against llms.txt (secondary sources; the primary reports were not opened):**
  - An Ahrefs log study of about 137,000 domains (May 2026) reportedly found that about 97% of published llms.txt files got zero requests in a month. Most requests that did happen came from SEO audit tools, and retrieval bots such as OAI-SearchBot and PerplexityBot made up about 1% — [geoly.ai](https://www.geoly.ai/blog/does-llms-txt-work); [ariashaw.com](https://ariashaw.com/does-llms-txt-actually-work).
  - A single-site log over 14 days across 18 AI crawler families recorded 723 robots.txt fetches and zero llms.txt fetches — [Saaslinks server-log study](https://saaslinks.net/blog/llms-txt-server-log-study).
  - A separate 12-day study recorded about 1,000 AI-crawler requests and none for llms.txt — [AWRSHIFT](https://wp.awrshift.com/llms-txt-in-practice/).
  - Google's John Mueller is reported as saying AI services haven't said they use the file, and that server logs show they don't check for it — [geoly.ai](https://www.geoly.ai/blog/does-llms-txt-work).
  - **Adoption:** 8.7% of the top 1,000 sites published an llms.txt as of June 2026 — [geoly.ai](https://www.geoly.ai/blog/does-llms-txt-work). SE Ranking reportedly found about 10% of roughly 300,000 domains, with no correlation between having the file and being cited by AI — [digitortoise](https://digitortoise.com/llms-txt-explained/).
  - **Outlier, contradicted:** a Worknido post claims GPTBot fetched llms.txt on about 60% of implementing sites, citing unnamed "public server log submissions" — [Worknido](https://www.worknido.com/blog/does-llmstxt-actually-work-evidence-from-2026-server-logs). It is unverifiable and contradicts the larger datasets.
- **Crawlers aren't the main consumer.** Coding agents fetch URLs a developer or tool points them at, so crawler logs don't measure agent use; the studies above measure only crawlers.
- **Passive context beats on-demand docs in one vendor eval** **(vendor-run, January 2026)** — [Vercel, "AGENTS.md outperforms skills in our agent evals"](https://vercel.com/blog/agents-md-outperforms-skills-in-our-agent-evals):
  - A compressed 8 KB docs index embedded in AGENTS.md reached a 100% pass rate on Next.js 16 API tasks.
  - Skills peaked at 79%, and only with explicit instructions to use them.
  - Without those instructions, skills matched having no docs at all (about 53% baseline), because the skill was never invoked in 56% of cases.
  - The eval is small (19 tasks, one framework) and run by the vendor.

### Inferences
- **For a compiler, the cheap and defensible outputs are:**
  - A Markdown rendering of each reference page, including operation pages generated from OpenAPI.
  - An index file. Whether to call it llms.txt matters less than its content.
  - A link to the raw spec.
- llms.txt itself is cheap to emit, but don't claim it drives agent traffic; the logs say it doesn't.
- **The Vercel result points toward a compact index** (operation name, one-line purpose, link) as the most useful agent artefact: something small enough to sit in context permanently. That argues for good `summary` fields on every operation.

### Gaps
- No source showed concretely how platforms represent API reference inside llms.txt (one line per operation? per tag? a link to openapi.json?) or llms-full.txt (does it inline every operation?). Inspecting live files (for example `docs.stripe.com/llms.txt` or a Mintlify customer's) would answer this. That wasn't done.
- No platform's documentation on `Accept: text/markdown` content negotiation was confirmed; Fern's agent-prep post didn't load.
- The llmstxt.org proposal text wasn't fetched in this pass.

## B2. MCP servers generated from OpenAPI, and what they reveal about spec quality

### Takeaway
Generators (Speakeasy/Gram, Stainless, Scalar, Fern, Mintlify, open-source converters) started with one MCP tool per operation, and every practitioner source reports the same failure. Too many tools, near-duplicate human-oriented descriptions, missing or unstable operationIds, enums that exist only in prose, and deep schemas all degrade tool selection. The field has moved two ways. One is curation: pruning, vendor extensions or overlays with descriptions written for the LLM, and hand-built workflow tools. The other is "search + execute" (Code Mode), where the spec becomes data the agent queries in a sandbox instead of a tool list. Either way, the spec's descriptions, summaries, schemas and examples are now read by models as well as people. **(fast-moving)**

### Cited Findings
- **Speakeasy** **(vendor; Georges Haidar, 2025-06-24, "50+ production MCP servers")** — [Speakeasy blog](https://www.speakeasy.com/blog/generating-mcp-from-openapi-lessons-from-50-production-servers):
  - Generators produce about one tool per operation, so 200 endpoints means about 200 tools, which "can overwhelm" a model. GitHub's API, with more than 500 operations, is the example.
  - Vague, near-duplicate descriptions cause the wrong tool to be chosen. Its example is three user endpoints described only as "Returns the user profile", "Fetches user details" and "Get user info".
  - Descriptions written for human readers can be too long and waste context tokens.
  - Its fix is an `x-speakeasy-mcp` extension, applicable through Overlays without editing the source spec, with these fields:
    - `disabled`: prune the operation.
    - `name` and `description`: an LLM-specific name and description, so human prose can stay.
    - `scopes`: for example `read`/`write`, to limit which tools a server exposes.
  - Also prune health and inspection endpoints.
- **Neon** **(API vendor writing about its own MCP server; David Gomes, 2025-05-01)** — [Neon blog](https://neon.com/blog/autogenerating-mcp-servers-openai-schemas):
  - The GitHub API has over 600 operations.
  - Models pick the wrong tool even among a few similar ones.
  - LLMs handle complex JSON payloads and ambiguous optional parameters poorly.
  - REST is resource-centric, while agents need task- and workflow-level tools.
  - Recommendations: generate, then prune; rewrite descriptions to say when and why to use a tool, with examples; add higher-level workflow tools.
- **Further practitioner and vendor guidance, from search summaries (pages not opened in full)** — [Truto](https://truto.one/blog/openapi-to-mcp-how-mcp-servers-auto-generate-tools-from-api-docs/); [Scalar, OpenAPI to MCP mapping](https://scalar.com/learn/mcp/openapi-to-mcp-server); [hackteam.io, "Stop converting OpenAPI specs into MCP servers"](https://hackteam.io/blog/stop-converting-openapi-specs-mcp-servers/); [DEV, how many MCP tools is too many](https://dev.to/merlonix/how-many-mcp-tools-is-too-many-tool-count-selection-overload-7gc):
  - Selection accuracy degrades past about 25–50 tools at once.
  - Combined tag and method filters cut a 200-tool dump to 12–30 tools per server.
  - Many vendor specs omit or duplicate `operationId`. Generators fall back to names built from method and path (for example `get_project`) and must keep them stable across releases, and must sanitise names to fit MCP's rules on length, characters and uniqueness.
  - A status field with twelve enum values that exist only in prose gets called with invented values, so enums belong in the schema.
- **Cloudflare Code Mode** **(vendor; 2026-02-20)** — [Cloudflare blog](https://blog.cloudflare.com/code-mode-mcp/):
  - Two tools, `search()` and `execute()`, cover the whole Cloudflare API (over 2,500 endpoints) in about 1,000 tokens. The page says the per-endpoint alternative would cost 1.17 million tokens in one place and "over 2 million" in another, inconsistently. Figures were measured with tiktoken.
  - `search()` hands the model's JavaScript a `spec` object holding the full OpenAPI spec with `$ref`s pre-resolved. The code filters by path, tag or product inside a Workers isolate, so the spec never enters the context window.
  - In the worked example, finding an enum meant walking `allOf` chains: deep schema nesting is a cost even when the agent reads the spec through code.
  - "No collection of hand-maintained servers could keep up" with the API.
  - InfoQ covered the launch in April 2026 — [InfoQ](https://www.infoq.com/news/2026/04/cloudflare-code-mode-mcp-server/). A later write-up says the server now lists four tools (`docs`, `search`, `execute`, `whoami`) — per a [search summary](https://www.matthewswong.com/en/blog/cloudflare-code-mode-mcp-two-tools-1000-tokens/), not verified.
- **Stainless** **(vendor)**:
  - It generates an MCP server from OpenAPI alongside its TypeScript SDK, published as a separate npm package — [Stainless blog](https://www.stainless.com/blog/generate-mcp-servers-from-openapi-specs).
  - It offers a dynamic mode for large APIs with three meta-tools (list, get schema, invoke) — [Stainless MCP product page](https://www.stainless.com/products/mcp).
  - A secondhand review describes a code mode (`search_docs` + `execute`, running TypeScript against the SDK in a sandbox) with "94–97% task accuracy" and "3x fewer tool calls" — [hunted.space](https://hunted.space/product/stainless-2). These figures are vendor-reported and secondhand, and sources disagree on whether the sandbox is Deno or Cloudflare Workers.
- **Scalar** **(vendor)** describes a hosted MCP server built on the same few-tools search-and-execute design, rather than one tool per endpoint — [Scalar, Generate an MCP server from OpenAPI](https://scalar.com/learn/mcp/generate-mcp-server-from-openapi).
- **Bump.sh** builds MCP servers from **workflow** documents (Arazzo or its own "Flower"), not from operations directly — [Bump.sh FAQ](https://docs.bump.sh/help/faq/). This implicitly concedes that operations are the wrong unit for agents.
- **The same pattern in GraphQL:** Apollo MCP Server prefers predefined operations, with introspect/search/execute as the fallback **(vendor)** — [Apollo docs](https://www.apollographql.com/docs/apollo-mcp-server/define-tools).

### Inferences
- **What MCP generation reveals about specs:**
  1. `summary` and `description` are often near-duplicates across similar operations, so nothing tells similar operations apart.
  2. Descriptions say what an operation does, not when to use it instead of the alternatives.
  3. operationIds are missing, duplicated or unstable.
  4. Constraints such as enums, formats and defaults sit in prose instead of the schema.
  5. Schemas are deeply nested through `allOf` and `$ref`.
  6. Nothing in the spec groups operations into tasks. Arazzo is the emerging standard for that.
- **These are checkable at build time.** A docs compiler could lint for them (duplicate summaries, a missing operationId, enum-like prose, an empty description) and so serve human readers and agents with one rule set.
- **The two directions.** An LLM-specific description field (Speakeasy's extension) and the Code Mode approach (the spec as queryable data) both work around the spec rather than improve it. The underlying demand is a spec that is accurate, fully resolved, and has distinct, task-oriented summaries.

### Gaps
- Fern's and Mintlify's MCP generators weren't researched in depth. Search results showed only that Mintlify hosts a docs-search MCP server, not one generated from operations.
- FastMCP's `from_openapi` and the open-source openapi-mcp-generator weren't examined directly.
- The MCP registry wasn't examined to count how many listed servers come from OpenAPI.
- No independent (non-vendor) measurement of tool-selection accuracy against tool count was opened. The 25–50 figure is from a DEV blog summary.

## B3. Do agents do better with the raw spec, rendered docs, or Markdown?

### Takeaway
No controlled, independent evaluation was found comparing agents' task accuracy given a raw OpenAPI spec, rendered HTML docs, or Markdown. The evidence is about token cost, not accuracy:
- Markdown is far cheaper than HTML (vendor claim of about 90% fewer tokens).
- Compact formats are 85–89% cheaper than OpenAPI YAML or JSON (one arXiv paper, with no accuracy results).
- Code Mode avoids loading the spec at all.

Two papers explicitly list this comparison as unstudied.

### Cited Findings
- **LAPIS** (arXiv:2602.18541, Daniel García García, submitted 2026-02-20) — [arXiv](https://arxiv.org/abs/2602.18541):
  - It argues OpenAPI carries substantial token overhead as LLM context, and proposes a compact format convertible from OpenAPI 3.x. The format adds centralised error definitions, webhook trigger conditions, structured rate limits and operation flow declarations.
  - It reports 85.5% fewer tokens than OpenAPI YAML and 88.6% fewer than JSON (cl100k_base). The test set was five specs, including GitHub (1,080 endpoints), Twilio (197) and DigitalOcean (545).
  - It reports no task-accuracy results and lists accuracy against OpenAPI context as future work — [arXiv PDF](https://arxiv.org/pdf/2602.18541).
- **"Architecture as Capability Equalizer for Coding Agents"** (arXiv 2608.21747) states that no prior work had compared such formats as input for code generation — [arXiv PDF](https://arxiv.org/pdf/2608.21747) (as summarised in search; the paper was not read in full).
- **"Beyond Browsing: API-Based Web Agents"** (arXiv 2410.16464, October 2024) gave agents OpenAPI YAML for some sites and README-style docs for others, but didn't compare the two — [arXiv](https://arxiv.org/pdf/2410.16464) (as summarised in search).
- **Vendor claims without independent backing** **(Fern, 2026)** — [Fern, Prepare APIs for AI agents (March 2026)](https://buildwithfern.com/post/prepare-apis-documentation-ai-agent-consumption); [Fern, llms.txt guide (May 2026)](https://buildwithfern.com/post/optimizing-api-docs-ai-agents-llms-txt-guide) (search summaries; pages didn't load):
  - Markdown cuts token use by about 90% compared with HTML.
  - Realistic examples in OpenAPI schemas improve the accuracy of generated code.
- **Code Mode results** (Cloudflare, Stainless) show that agents can use a raw spec well when they query it through code rather than read it — [Cloudflare](https://blog.cloudflare.com/code-mode-mcp/). Stainless's figure of "94–97% accuracy" is vendor-reported and secondhand — [hunted.space](https://hunted.space/product/stainless-2). A 2026 roundup attributes a 15–20% retry rate on code execution, compared with direct tool calls, to Speakeasy's reporting — [fast.io roundup](https://www.fast.io/resources/best-mcp-server-generator-tools.md) (attribution muddled in the source).
- **On-demand docs vs passive context.** Vercel's eval (above) is the closest thing to a controlled comparison of how docs are delivered: an AGENTS.md index scored 100%, skills peaked at 79%, and the baseline was 53%. It concerns framework docs, not API specs, and is vendor-run — [Vercel](https://vercel.com/blog/agents-md-outperforms-skills-in-our-agent-evals).

### Inferences
- **What the evidence supports so far:**
  - The raw spec is best treated as data to be queried (by Code Mode or SDK generation), not text to be read.
  - Markdown suits task guides and per-operation pages an agent fetches on demand.
  - A compact index suits passive context.
  - HTML is the worst choice of the three on cost.
- The report should say plainly that no accuracy study separates these options.

### Gaps
- No independent study measuring agent accuracy across raw spec, Markdown and HTML for API tasks was found. This is the main gap in Part B.
- The full Fern posts didn't load, so their methodology (if any) for the 90% figure is unknown.

## B4. What new demands do agents place on spec authors?

### Takeaway
Agents want what good human docs always needed, made explicit and machine-readable:
- distinct summaries that say when to use an operation;
- stable operationIds;
- constraints (enums, defaults, formats) in the schema, not in prose;
- realistic examples;
- documented errors;
- grouping into tasks or workflows.

Descriptions written for humans are often too long or too vague for selecting a tool, which is why vendors added fields for LLM-specific descriptions.

### Cited Findings
- **Descriptions should say when and why to use an operation, with examples, expected workflows and guidance on common parameters** — [Neon](https://neon.com/blog/autogenerating-mcp-servers-openai-schemas). Precise, distinct descriptions; human-length prose costs context — [Speakeasy](https://www.speakeasy.com/blog/generating-mcp-from-openapi-lessons-from-50-production-servers) **(vendor)**.
- **Constraints belong in the schema.** Agents need explicit types, defaults, constraints and examples, not just parameter names — [LogRocket](https://blog.logrocket.com/how-write-agent-friendly-api-documentation/). Prose-only enums lead to invented values — [Truto](https://truto.one/blog/openapi-to-mcp-how-mcp-servers-auto-generate-tools-from-api-docs/) **(vendor; search summary)**.
- **Stable, valid operationIds**, or derived names kept stable across releases — [Truto](https://truto.one/blog/openapi-to-mcp-how-mcp-servers-auto-generate-tools-from-api-docs/); [Scalar](https://scalar.com/learn/mcp/openapi-to-mcp-server) **(vendor; search summary)**.
- **A stale spec makes agents fail with confidence** — [Theneo](https://www.theneo.io/blog/why-api-specs-break-ai-agents-mcp-launch) **(vendor)**.
- **Errors, rate limits and flows.** LAPIS adds centralised error definitions, structured rate limits and operation flow declarations because OpenAPI expresses them poorly for LLMs — [arXiv 2602.18541](https://arxiv.org/abs/2602.18541). Fern says generated AsyncAPI docs omit errors, auth and connection lifecycle — [Fern AsyncAPI guide](https://buildwithfern.com/post/asyncapi-documentation-generator) **(vendor)**.
- **Workflows as first-class artefacts.** Bump.sh builds MCP servers from Arazzo/Flower workflows — [Bump.sh FAQ](https://docs.bump.sh/help/faq/). Neon recommends higher-level workflow tools — [Neon](https://neon.com/blog/autogenerating-mcp-servers-openai-schemas).
- **Examples.** Fern claims realistic examples improve the accuracy of generated code **(vendor, unverified)** — [Fern llms.txt guide](https://buildwithfern.com/post/optimizing-api-docs-ai-agents-llms-txt-guide).

### Inferences
- Nearly every demand above is also an existing pain point in human OpenAPI docs: thin descriptions, missing examples, undocumented errors, no task-level narrative. Agents raise the cost of these defects; they don't change what the defects are. That supports one quality model (lints plus examples checked at build time) serving both audiences, not a separate "agent mode".
- **The one genuinely new demand:** a short, distinct, selection-oriented summary per operation, written so it can tell similar operations apart. It's worth a dedicated lint, such as duplicate or near-duplicate summaries within a tag.
- Arazzo is the emerging standard for the task layer that OpenAPI lacks. It and GraphQL's predefined operations both play the role a guide's code examples play for humans.

### Gaps
- No quantitative evidence ties particular spec improvements (better descriptions, examples, error schemas) to measured agent success. All the guidance is practitioner or vendor experience.
- No source covered how agents use documented error responses specifically, such as whether retry and recovery improves.
