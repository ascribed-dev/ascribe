# Proposal: OpenAPI

A proposal, not a plan. It says what would be built and in what order, so the idea can be judged before anything is designed in detail. It picks up section 3 of the [brainstorm](../brainstorm.md#3-openapi), and draws on the research in [OpenAPI documentation pain points](../reports/OpenAPI%20documentation%20pain%20points.md).

## The goal

A writer links from a guide to an operation in the project's OpenAPI spec, and the link is checked like a link to a heading: completed in the editor, shown on hover, and an error when the spec no longer has the operation. A JSON example in a page that claims to be an operation's request or response is checked against that operation's schema. A report says which operations no page mentions.

Ascribe reads the spec and checks the pages against it. It doesn't draw the API reference. A renderer the team already uses (Scalar, Redoc, starlight-openapi, or a hosted portal) publishes the reference pages, and Ascribe's links land on them.

## Why checks and not a renderer

The research found the market split in two.

- **Rendering a spec is crowded and mostly solved.** Every tool surveyed draws a reference, and most embed an operation in an authored page. The hard parts are expensive and never finished: `oneOf`, `allOf` and discriminators (Swagger UI's most upvoted issue, open since 2017), nullability written two ways, four spec versions, try-it consoles, auth, code samples, and specs of 80,000 lines. Vendors with whole teams lag a year behind OpenAPI 3.2. In Ascribe the cost doubles, since `render_site_html` and the Astro plugin must agree, and `@ascribed/elements`' contract would grow with every schema shape.
- **The joins between the spec and the pages are unchecked.** Links from prose into operations break silently in every tool but Fern. Examples in guides are validated by nothing. No tool reports operations no guide mentions. These are what Ascribe already does for pages: links, completion, hover, rename, diagnostics, and reports of unused things.

So the brainstorm's level 1 is the core of this proposal, level 2 is built in a thin form, and level 3 is left to the renderers.

## What exists

- **Links to files that aren't pages** (SPEC §5.2, the [assets contract](../../docs/content/contracts/assets.md)). A link to `api/openapi.yaml` is an asset: checked for existence with exact case, copied into every output, and rewritten. Its fragment is kept and never checked, so `[Get a user](../api/openapi.yaml#getUser)` passes today and opens the raw YAML.
- **One resolver for every link.** `resolve_reference` (`crates/ascribe-resolve/src/references.rs`) serves check, the build, the language server, and `ascribe link`, and returns a `Resolution`. A new kind of target is a new variant there, and every surface picks it up.
- **The boundary.** A link's target may be inside the content root or the project root (`Layout::is_allowed`, SPEC §9.4's asset rule). With `ascribe.toml` at the repository's root, `api/openapi.yaml` is in reach; with `ascribe.toml` inside `docs/`, it isn't. The one reach outside is `[sources]`: a folder `ascribe.toml` names, in the same git repository, whose code pages may read, read through `FileSystem` and from git at a base revision as the pages are.
- **The editor's link features.** Completion after `#` offers a page's headings (`crates/ascribe-lsp/src/complete.rs`); hover previews the target (`hover.rs`); go to definition can already land inside a file that isn't Markdown, as a snippet's region does (`examples.rs`).
- **Reports of unused things.** `fragment-unused`, `image-unused`, and the others are advice in `crates/ascribe-check/src/across.rs`, counted by `Project::uses`, and listed by `ascribe report inventory`.
- **YAML and JSON parsers** (`serde_yaml_ng`, `serde_json`) are workspace dependencies. Nothing validates against a JSON Schema: `jsonschema` isn't a dependency.

Nothing in the code reads OpenAPI.

## Settled

Answered by the maintainer on 24 October 2026.

1. **`spec` may name a file outside the project's folder,** in the same git repository, as `[sources]` may name a folder of code. A layout with `docs/ascribe.toml` and `api/openapi.yaml` is the common one, and a copy of the spec inside `docs/` would drift from the spec the API publishes, which is what the checks are for. A link reaches a spec outside the folder by its relative path, `../../api/openapi.yaml#getUser`, as the one exception to SPEC §9.4's rule that a reference resolves inside the project or content root, and only with a fragment: a link to it with no fragment is an error saying to link the published reference instead, since a file from outside the project has no place in the outputs to be copied to. A spec in another repository is still left out.

## What the spikes found

Two spikes on 24 October 2026, before any code: the thread's seam, a renderer's address for an operation, and the corpus phase 1 was to measure.

### Renderers' addresses

One small spec, with tags `Users`, `Billing & Invoices`, and `Café`, an operation with two tags, one with no tag, one with no `operationId`, and an id with a dot (`Health.Check_v2`), rendered by each renderer at its current version and at the version of a year before. Each address was read from the rendered page and then loaded fresh, to see whether it lands on the operation.

| Renderer | An operation's address | As a template | Across the year |
|---|---|---|---|
| Redoc 2.5 and 3.0 | 3.0's own is `#/users/getuser`, 2.5's `#tag/Users/operation/getUser`; both accept `#operation/getUser` and land | `#operation/{operationId}` | Stable through that form; 3.0 also accepts 2.5's |
| Swagger UI 5.29 and 5.34 | `#/Users/getUser`: the first tag as written, percent-encoded, matched with case | `#/{tag}/{operationId}` | Stable |
| Stoplight Elements 9.0 | `#/operations/getUser` | `#/operations/{operationId}` | Stable |
| starlight-openapi 0.21 and 0.27 | `/api/operations/getuser/`, a page: the id slugged by github-slugger (`healthcheck_v2`) | Needs that slug of the id | Stable |
| Docusaurus OpenAPI docs 4.5 and 5.2 | A page named by lodash's `kebabCase` of the id (`get-user`, `health-check-v-2`) | Needs that slug, a different one | Stable |
| Scalar 1.38 and 1.76 | `#tag/users/GET/users/{id}`: a slug of the tag, the method, and the path; no form by `operationId`, and none without the tag | No: the tag's slug and the method's case are Scalar's | **Broke:** 1.38 wrote `#tag/users/get/…` and `billing--invoices`, 1.76 writes `GET` and `billing-invoices`, and 1.76 sends 1.38's addresses to the introduction |

So a template of the id as written serves three of six, two more need a slug of the id by their own rule, and Scalar can't be served by a template, or by anything that doesn't track its releases. An operation with no `operationId` gets a different made-up address in each: Redoc's JSON pointer under the tag, Swagger UI's `delete_users__id_`, Elements' `#/paths/users-id/delete`, starlight-openapi's `usersid` (with no method, so two operations on one path collide), and Docusaurus's slug of the summary.

### The corpus

A seeded sample of 300 APIs from APIs.guru (298 downloaded), and specs generated with each generator's defaults from the same small API (a users resource with five operations and a nested list, and a health check):

- **Generated specs:** FastAPI writes every id, mechanically (`get_user_users__user_id__get`). NestJS writes every id as `UsersController_findOne`. springdoc writes the method's name (`findOne`), and a second method of the same name becomes `findAll_1`, which one depending on scan order, so an id can change meaning between builds. **.NET writes none**: neither .NET 9's own generator nor Swashbuckle 10 gives an operation an `operationId` by default.
- **Published specs:** 83% of specs have an id on every operation, 10% on none, 6% on some. Leaving out Microsoft Graph, which is 22,361 of the 30,739 operations, 8.3% of operations have no id. No spec used an id twice, which APIs.guru's curation may account for.
- **What an id holds:** `/` in 1,998 ids across 13 specs (GitHub's `activity/list-org-events-for-authenticated-user`), and spaces in 38, so a link's fragment is percent-decoded before it's matched.
- **Reading it:** every spec parsed; none has a path item that's a `$ref`. Parsing the largest is fast enough for `check` and for the editor's re-read: GitHub's 3 MB spec in 12 ms as JSON and 72 ms as YAML, Graph's 57 MB YAML in 1.4 s. Reading only `paths` saves little in YAML, where the parser is the cost.

APIs.guru is a directory of public APIs, cleaned and converted to JSON, so it under-counts what internal, generated specs look like; the generated specs are what the defaults write, and teams often change them.

### What they change

- **The address can't be one template.** The proposal's fallback, named profiles, is what the evidence asks for: open question 2.
- **An operation with no id is common enough to need a link form,** since .NET's defaults write none: open question 1.
- **An id that changes meaning,** as springdoc's suffix does, is the one case `link-operation-missing` can't catch. The docs say to set ids explicitly, and a later check could compare a link's method and path with the base's.

## The design

### The spec is declared

```toml
[apis.public]
spec = "api/openapi.yaml"
reference = "/api/#tag/{tag}/operation/{operationId}"
```

| Key | Meaning |
|---|---|
| `spec` | The spec's file, from the project root. One file, in YAML or JSON. |
| `reference` | The address of an operation's page in the published reference, as a template. Relative to `[consumer] base-path`, or absolute when the reference is published elsewhere. Its placeholders are the operation's `{operationId}`, `{tag}` (its first), `{method}`, and `{path}`. |

The section is `[apis]`, not `[openapi]`, so an AsyncAPI spec can be declared beside it later, with a `format` key added then.

### Reading the spec, loosely

Ascribe reads the part of the spec its checks need: each operation's method, path, `operationId`, `summary`, `tags`, and `deprecated`, and, from phase 3, its request and response schemas and the components they reference. It reads that part the same way in Swagger 2.0 and OpenAPI 3.0, 3.1, and 3.2, since `paths` hasn't changed shape across them, and it ignores everything else. It doesn't validate the spec; Spectral and Redocly do that.

This is what keeps Ascribe out of the version lag the research found. A reader that models the whole spec has to be rewritten for each version; one that reads `paths` and skips the rest reads 3.2 today. It also accepts the documents that strict readers reject: a spec labelled 3.1 that still writes `nullable: true` makes the most common validator error in one analysis of published specs, and its links are as good as any.

Ascribe reads the spec as readers see it: the file the team's pipeline publishes, after bundling and overlays. It doesn't apply overlays or follow `$ref`s into other files. A team whose spec is split across files, or patched by an overlay, points `spec` at the bundled result their renderer is given. A spec generated from code has to be on disk when Ascribe runs, as it does for the renderer.

The spec is read through `FileSystem::read_file`, inside the project's folder or outside it in the same git repository (Settled 1), and from git at a base revision for `ascribe diff` and review, as a source's code is. The editor watches it where it is.

### A link to an operation

```markdown
To fetch one user, call [Get a user](../api/openapi.yaml#getUser).
```

A link to a declared spec with a fragment is a link to an operation: the fragment is the `operationId`. A link to the spec without a fragment is an asset, as today, for "download the spec", when the spec is inside the project's folder; outside it, that link is an error (Settled 1). A link to an undeclared YAML file is an asset, as today.

- **Checked.** An `operationId` the spec doesn't have is an error, `link-operation-missing`, with the closest ids suggested, at the level `link-id-missing` has. A spec that can't be read or parsed is an error on its declaration.
- **Written as the renderer's address.** The site output writes the `reference` template filled in for the operation; the plain output writes it absolute when `[consumer] site` is set or the template is absolute; the JSON output gives the link's target as an operation: the API's name, the `operationId`, and the address. That JSON is a contract, and its shape is settled in phase 1.
- **Unambiguous.** A link names its spec by path, so two specs with the same `operationId` never collide, where Mintlify takes the first match in alphabetical order.

The `operationId` is the address because it's readable, survives a path being renamed, and is what the renderers key their own pages by. Its weakness is that generated ids are mechanical (`createItemItemsPost`) and sometimes missing. Phase 1 measures how often they're missing, and the first open question asks what a link names when there's no id.

### Why these choices

- **A path and a fragment, not a new scheme.** A link to the spec is already a link to a file, relative like any other, completed and checked by the resolver every surface uses. A scheme such as Fern's `api:GET/users` would be a second link syntax, and an agent or a writer would have to learn which one to use.
- **A template, not knowledge of each renderer.** Ascribe doesn't know which renderer a team uses, and renderers change their addresses between releases. A template is one string the team owns. The risk is that a renderer's addresses can't be written as a template, which is what phase 1 tests.
- **No overlays, no multi-file specs.** Both are the pipeline's job, and the research found their tooling still settling. Reading the bundled file gives the same answer and keeps Ascribe's reader small.

## The thread

- **The use case:** a writer links a guide to `getUser`; `ascribe check` fails when the spec is regenerated without it; and in the built site, the link opens `getUser` in the reference a renderer publishes beside the guides.
- **The layers:** the content model (`[apis]`), resolve (reading the spec, a new `Resolution` variant), check (the diagnostics), emit (the address in the site, plain, and JSON outputs), the CLI, the contracts, and the docs. The language server shows the diagnostic without new work, since it runs the same check.
- **The seam it tests:** the renderer's address. Ascribe has never written a link into a URL space it doesn't own. A renderer's address for an operation is made from some of the tag, the `operationId`, the method, and the path, each slugged its own way. The spikes found both: tags and ids slugged by rules a template can't express, and an address that changed within a year (Scalar), so the design takes named profiles, one per renderer, as the `Router` is one per consumer, once open question 2 is settled. Phase 1 still tests each profile against its renderer's built output.
- **Measured on real specs.** The spikes measured the corpus and the renderers before phase 1 (above). Phase 1 runs its own reader over the same sample, and records which specs it can't read.
- **Left out, and where it comes back:** completion, hover, go to definition, and the fix for a moved operation (phase 2); `ascribe link` and the MCP server (phase 2); examples checked against schemas (phase 3); the report of operations no page mentions (phase 4); the embed (phase 5).

## The phases

| Phase | What a user can do after it | Layers |
|---|---|---|
| 1. Checked links | Link a page to an operation, see the build fail when the operation goes, and land on it in the published reference | content model, resolve, check, emit, CLI, contracts, docs |
| 2. In the editor and for agents | Complete an operation after `#`, see its method, path, and summary on hover, jump to it in the spec, and fix a link whose operation was renamed | language server, extension, `ascribe link`, MCP |
| 3. Checked examples | Mark a JSON block as an operation's request or response, and see where it disagrees with the schema | language, resolve, check, language server |
| 4. Coverage | See which operations no page mentions, in the check and in the inventory | check, report, contracts |
| 5. The thin embed | Put an operation's method, path, summary, and deprecation in a page, with a link to its reference page | language, emit, both renderers, elements |

**Phase 1** builds the thread. `examples/quill` gains a small spec and a renderer's reference beside its pages, and a page that links to two operations. Conformance cases hold a link to an operation, a missing one, a link to the spec with no fragment, and an undeclared YAML file. The content-model reference gains `[apis]`, SPEC §5.2 gains the new kind of target and §9.4 the exception for a spec outside the project's folder, which a conformance case holds, and the docs gain a page, "Link to your API reference". *Done when:* `ascribe check` in `examples/quill` reports `link-operation-missing` after `getUser` is removed from the spec; `ascribe build` writes the link as the renderer's address, and following it in the built site opens that operation; and the corpus's results are recorded in the pull request.

**Phase 2** brings the links into the editor and to agents. Completion after `openapi.yaml#` lists the spec's operations, matched on id, method, path, or summary. Hover shows the method, the path, the summary, and whether the operation is deprecated. Go to definition opens the spec at the operation. The spec is re-read when it changes on disk. When an `operationId` disappears and an operation with the same method and path has a new one, which is what a regenerated spec does, the diagnostic offers that id as a safe fix. A link to a deprecated operation is advice, `link-operation-deprecated`. `ascribe link` and `ascribe_link` answer with the operation, so an agent writing a guide can check a link before writing it.

**Phase 3** checks examples. A JSON code block marked as an operation's request or response body (the second open question) is validated against that body's schema, and each disagreement is reported at its place in the block. 3.1's schemas are JSON Schema; 3.0's and 2.0's are translated first, `nullable` among the rest. It needs a JSON Schema validator, a new dependency, and the components a schema references, still within the one file. It's before coverage because it carries the most design risk left: a new syntax, a new dependency, and the translation.

**Phase 4** reports operations no page links to, as advice, `api-operation-unreferenced`, in area `project`, beside `fragment-unused`. The inventory gains a count per API: operations, operations linked, and operations with a checked example. Only links count: a page that mentions `getUser` in prose isn't counted, since that can't be told reliably from a word that happens to match.

**Phase 5** adds the embed: `@operation: ../api/openapi.yaml#getUser` writes a block with the operation's method, path, summary, and a deprecation mark, linked to its reference page. The name has no hyphen, since a hyphenated name is a project's widget (SPEC §6). The block is checked as the link is, and the plain output writes it as a line of text with the link. It renders no schema: a page that wants the parameters links to the reference, or shows an example that phase 3 checks.

## What this leaves out

- **Generated reference pages, and rendering schemas.** The renderers do it, and do it better than a first version would. If teams ask for one look across guides and reference, which is why Stripe, Adyen, and GitHub built their own, generated pages can be proposed then, on top of the links, the addresses, and the reader this builds.
- **Applying overlays and following `$ref`s into other files.** The pipeline bundles; Ascribe reads the result.
- **A spec from another repository.** A `[sources]` git source copies only the files snippets name, and its files aren't assets. A team whose spec lives elsewhere commits a copy, or a later proposal makes a source's file declarable as a spec.
- **A spec per release line.** A versioned build reads the same spec for every line. Lines whose APIs differ need the spec as of each line, which waits for the versions work to say how a line reads another file's history.
- **Availability from the spec.** The brainstorm's `x-ascribe-available` would map onto availability specs (`parse_availability`). Each renderer already reads its own extension for the same fact, and a fifth would make the team mark it again.
- **Drift between the spec and the running API.** Contract testing (Schemathesis, Prism) proves that. Ascribe proves the pages agree with the spec.
- **AsyncAPI and GraphQL.** The `[apis]` section and the link form leave room for them; neither is built here.

## Open questions

1. **What does a link name when an operation has no `operationId`?** Either such an operation can't be linked, and `link-operation-missing` says why, or a second fragment form names it by method and path, such as `#get/users/{id}`. The corpus answers it: .NET's generators write no ids by default, and 10% of published specs have none. Recommended: the second form, mapped to each renderer's address by its profile (question 2); with a template, a link in that form is an error, since no template makes the renderers' addresses for it.
2. **How is a renderer's address made?** A template serves Redoc, Swagger UI, and Elements; starlight-openapi and Docusaurus each need their own slug of the id; Scalar needs a slug of its tag, the method's case, and a profile that follows its releases, since it changed its addresses within the year. Recommended: named profiles for the renderers, `reference = { renderer = "redoc", at = "/api/" }`, each written from the renderer's code and held by a test that builds the renderer at a pinned version and loads every address, as the spike did; and a template, for any other renderer, with the id as written only. Scalar's profile names the versions it was tested with, and the docs say its addresses change with its releases.
3. **How does a JSON block say which operation it belongs to** (phase 3)? An attribute in the info string, as `title` and `tested` are (```` ```json request="openapi.yaml#createUser" ````, with `response="…#createUser 201"`), or a directive around the block. Recommended: the info string, since it keeps the block a plain fenced block that every Markdown viewer still shows; whichever, it's a language change with its own decision before phase 3.
