# Day-to-day pain of documenting APIs with OpenAPI: writers' and engineers' accounts

Researched 2026-10-22. Note on dates: the pages in Tom Johnson's *Documenting APIs* course (idratherbewriting.com) report a modified date of 2026-10-21. The course itself dates from about 2017 to 2020 and has been revised since, so treat its material as long-standing, not new. Reddit (r/technicalwriting, r/api) was not reachable from this research environment, so first-hand forum material comes from Hacker News, GitHub issues and discussions, and practitioners' blogs.

## 1. Reference vs narrative: combining generated reference with hand-written guides

### Takeaway
This is the most often described structural pain. OpenAPI tools render only the reference, so teams either run two sites with different looks or push conceptual prose into the spec. Both choices duplicate facts and split the source of truth. Practitioners describe generated reference as "100% correct, but not useful" without the guides around it.

### Cited Findings
- Tom Johnson: the spec and its renderers "typically cover only reference documentation," so a separate user guide is still needed for tutorials, API keys, rate limits and so on. Using OpenAPI creates "two places where you're describing your endpoints and parameters," which must be kept in sync or linked. — [I'd Rather Be Writing, Introduction to the OpenAPI specification](https://idratherbewriting.com/learnapidoc/pubapis_openapi_intro.html)
- Johnson lists six integration strategies, each with a cost:
  - **Put conceptual content into the spec** with expand/collapse sections in `description`. This "only works if you have a relatively small amount of" conceptual information.
  - **Embed Swagger UI in the docs.** "The effect is still kind of clunky," not "a seamlessly branded experience," and schemas "expand beyond their limits" of the container. The workaround was custom CSS and a nav toggle.
  - **Source shared YAML/Liquid data into the spec.** It was "hard to ensure that your OpenAPI spec remains valid as you write content"; he "grew frustrated at not being able to validate my spec immediately" and "eventually gave up."
  - **Use a hosted platform** (Readme.com, Stoplight, Redocly). He notes that "Readme.com requires third-party hosting."
  - **Accept two sites.** "What's so bad about having two different sites?", comparing it to Javadoc and Doxygen.
  - On the cost overall: splitting content into a separate spec "splits your single source of truth into multiple sources," and he asks whether you must "copy and paste the same parameters and other information across both sites?"

  — [I'd Rather Be Writing, Integrating Swagger UI with the rest of your docs](https://idratherbewriting.com/learnapidoc/pubapis_combine_swagger_and_guide.html)
- Johnson's publishing talk describes a gap in tools that combine reference docs with non-reference content. — [I'd Rather Be Writing, Publishing API documentation](https://idratherbewriting.com/publishing-api-docs/)
- Hacker News (2019), first-hand comments on generated docs:
  - ergothus: "The generated documentation was 100% correct, but not useful"; they wanted examples showing "HOW I would actually call it."
  - chipotle_coyote: generated docs, "even when they're very good, only provide one of those -- the reference" (of the tutorial, how-to, explanation and reference quadrants).
  - weaksauce: it is like "describing what a forest looks and operates by listing every tree in the forest."
  - remotedeveloper: "There is zero documentation about what happens when you update an object - only OpenAPI."
  - mfer: generated reference "is good as supplemental rather than primary documentation."

  — [HN thread 21290458](https://news.ycombinator.com/item?id=21290458)
- Hacker News (2019), the counterpoint: dtech says generated docs "will be kept in sync with the code/API" while "a separate specification will become outdated over time," and "a separate openapi spec that is not enforced can quickly become outdated." — [HN thread 21290458](https://news.ycombinator.com/item?id=21290458)
- Adyen (2018) built a custom API Explorer partly so the reference would share the portal's design. The explorer is "powered by OpenAPI specs but has its own design," consistent with the rest of the documentation. Adyen also notes that "automatically generated documentation often provides very limited descriptions." — [Adyen, Improving the developer experience with OpenAPI Specification (Aleksei Akimov, 12 July 2018)](https://www.adyen.com/knowledge-hub/improving-the-developer-experience-with-openapi-specification)
- Stripe (2022): the product docs (Markdoc) and the API reference are separate systems that share React components from Stripe's design library. A planned link between them was to show API reference overlays when readers hover over parameters in guide code samples. — [Stripe, How Stripe builds interactive docs with Markdoc (Ryan Paul, 13 Sept 2022)](https://stripe.dev/blog/markdoc)
- Generated pages lose hand edits. In Docusaurus OpenAPI setups, the documented workflow runs a clean step that deletes the generated `.api.mdx` files (one per endpoint, plus `.tag.mdx` files and `sidebar.ts`) before regenerating, so edits to generated pages are wiped. Custom content has to live in separate pages. — [Agenta update-api-docs workflow](https://playbooks.com/skills/agenta-ai/agenta/update-api-docs); [Bluesky docs repo "clear-and-generate"](https://github.com/bluesky-social/bsky-docs)
- Links into generated reference depend on generated routes. The PaloAltoNetworks Docusaurus plugin changed its default page id from `summary` to `operationId` "to avoid duplicate routes," so every generated URL changed. Issue #1110 reports that a tag with the same name as an operationId produces duplicate routes and loads the wrong page. — [docusaurus-openapi-docs CHANGELOG](https://github.com/PaloAltoNetworks/docusaurus-openapi-docs/blob/main/CHANGELOG.md); [Issue #1110](https://github.com/PaloAltoNetworks/docusaurus-openapi-docs/issues/1110)
- Docusaurus's own broken-link checker misfires on links from MDX to a compiled OpenAPI file, because it treats the link as a doc route. — [docusaurus-plugin-omg issue #4](https://github.com/mcclowes/docusaurus-plugin-omg/issues/4)
- OpenAPI's `externalDocs` object is the only built-in place for linking from reference out to concept pages; Johnson says the user experience remains split. — [I'd Rather Be Writing, externalDocs step](https://idratherbewriting.com/learnapidoc/pubapis_openapi_step8_externaldocs_object.html)

### Inferences
- Writers say much less about broken prose-to-reference links than about duplication and look-and-feel, perhaps because many sites link to anchors by hand and don't check them. The route-instability evidence (the operationId default change, tag/operationId collisions) suggests the breakage is real and silent.
- The common shape of the fix is reference content that can be addressed from prose: link by operationId or by path and method, resolved at build time, with a failure when the target is gone. Respondents ask for "one site, one look, one source of truth," and no tool they mention provides that without paid hosting.
- Whenever a generator owns the output files, hand additions get deleted. Writers then need a separate layer for their content (separate pages, or overlays; see section 4).

### Gaps
- No first-hand Write the Docs Slack threads or Reddit threads were retrieved (Reddit was blocked; the Slack archive isn't indexed).
- No first-hand report was found of a guide going stale *because* the spec changed (as opposed to drifting in general). It is plausible but not documented here.

## 2. Writing in YAML/JSON: long Markdown descriptions inside the spec

### Takeaway
Authoring prose inside YAML strings causes problems with parsers, indentation, quoting and Markdown rendering. It is hard to lint and hard to review in pull requests. The renderers each interpret CommonMark and GFM differently, which compounds the problem.

### Cited Findings
- Redocly (Lorna Mitchell, 14 Feb 2024): "Markdown content containing mistakes buried inside OpenAPI files" is common, and because the Markdown sits inside JSON or YAML, "linting one format inside another is tricky." The workaround is a custom CLI plugin that runs markdownlint on every `description` field. Even a published spec (Apache Airflow) used bold text as headings. — [Redocly blog, Lint Markdown in OpenAPI](https://redocly.com/blog/lint-markdown)
- Redoc issue #972: a Markdown table in a description "swallows" everything after it, and text before the table can stop it rendering. The reporter says the same content worked in Swagger UI. Issue #1113: a table renders "half as a table and half not" despite blank lines around it. — [Redoc #972](https://github.com/Redocly/redoc/issues/972); [Redoc #1113](https://github.com/Redocly/redoc/issues/1113)
- Redoc #1141: a `#` comment inside a fenced code block in a YAML description was apparently treated as a heading, which emptied the block. — [Redoc #1141](https://github.com/Redocly/redoc/issues/1141)
- Redoc #214: headings in a description didn't become sidebar items. The maintainer's answer was "probably you're missing some \n or spaces according to markdown spec," which is a whitespace problem a writer can't see in a YAML block scalar. — [Redoc #214](https://github.com/Redocly/redoc/issues/214)
- Redoc #2876: a long changelog written in the `info.description` introduction crowds the table of contents, and the reporter asks for a heading-depth limit. This shows teams putting long-form narrative into the spec. — [Redoc #2876](https://github.com/Redocly/redoc/issues/2876)
- Mistral's public docs: a PR fixed closing quotes at the wrong column inside multi-line Markdown values. "PyYAML tolerates closing quotes at any indentation level, stricter YAML parsers reject this." — [mistralai/platform-docs-public PR #414](https://github.com/mistralai/platform-docs-public/pull/414)
- A misplaced pair of mapping keys in `openapi.yaml` broke a format check, which then "failed on main for every PR in this repo, including unrelated ones." — [accensa-app PR #695](https://github.com/accensa/accensa-app/pull/695)
- Unquoted description text such as `resume: true` parsed as a mapping and broke the document; the maintainers advised quoting and linting locally. — [mercury-fw issue #250](https://github.com/lucabro81/mercury-fw/issues/250)
- GitHub Community discussion (Aug 2026): "Reviewing large OpenAPI raw JSON/YAML diffs inside standard GitHub Pull Request windows is notoriously error-prone" (AUG235772). The original poster: "A code change is relatively easy to review in a pull request, but API changes can sometimes be harder to spot." The workarounds suggested:
  - build a Redocly or Swagger UI static preview per PR so reviewers can "click an interactive link and visually navigate the modified endpoints";
  - run semantic diffs (oasdiff, Optic) that post a comment on the PR;
  - lint with Spectral.

  — [GitHub community discussion #205222](https://github.com/orgs/community/discussions/205222)
- Tom Johnson gave up sourcing writer-maintained YAML/Liquid into the spec because he could not "validate my spec immediately." — [I'd Rather Be Writing](https://idratherbewriting.com/learnapidoc/pubapis_combine_swagger_and_guide.html)
- Code-first teams write their descriptions in source annotations. A 2018 HN commenter (steve_adams_86) says that with this approach "the code base swells up with annotations" while "the docs are permanently neglected." A 2019 commenter says examples "are a pain to write in Java annotations" (from the search summary of the HN threads). — [HN Ask: best API docs (2018)](https://news.ycombinator.com/item?id=17905919); [HN 21290458](https://news.ycombinator.com/item?id=21290458)
- Vendor framing (marketing, labelled as such): YAML authoring frustrations are "indentation errors, complex schemas, forgotten parameters, and endless scrolling." — [YAML Studio post on DEV](https://dev.to/logesh_r_b27ecd6cf8675f2b/introducing-yaml-studio-your-openapi-yaml-generator-with-superpowers-4e2n)

### Inferences
- The pains have three layers:
  - **YAML syntax.** Quoting, indentation and block scalars produce parse failures that block CI for everyone.
  - **Markdown inside YAML.** There is no linting, no preview, and whitespace that matters but can't be seen.
  - **Renderer divergence.** Tables and headings render differently in Redoc and Swagger UI.

  A compiler that lets descriptions live in Markdown files, referenced from the spec or attached by operationId, removes the first two layers. It can also give one rendering of Markdown in both prose and reference.
- PR review needs a rendered preview of *changed operations*. A raw YAML diff hides the effect of changes to shared `$ref` components.

### Gaps
- No quantitative data on how many teams keep long descriptions in the spec versus in separate Markdown files.
- No first-hand writer post on JSON-specific escaping pain was found; the evidence is YAML.

## 3. Examples: missing, wrong, stale; multi-language samples; where they live

### Takeaway
Consumers rank examples as the most valued part of API docs in every SmartBear survey. Generated reference commonly ships placeholder or missing examples. Multi-language samples kept in the spec (`x-codeSamples`) drift away from the real SDKs.

### Cited Findings
- SmartBear State of API 2020 (3,500+ respondents), as reviewed by Tom Johnson. The most important documentation elements:
  - examples 70%;
  - status and errors 51%;
  - authentication 50%;
  - error messages 49%.

  Johnson admits being "a little confused at how 'examples' differ from 'code samples.'" — [I'd Rather Be Writing, SmartBear 2020 review (10 Sept 2020)](https://idratherbewriting.com/blog/smartBear-2020-state-of-api-docs-review/)
- SmartBear State of Software Quality / API 2023, via Nordic APIs (secondary summary): examples 66%, status and errors 59%, authentication 55%. The figures differ from 2020, but the order is the same. — [Nordic APIs summary](https://nordicapis.com/10-insights-from-the-state-of-software-quality-2023-report/); [SmartBear 2023 PDF](https://assets.smartbear.com/m/5276eb5353c29969/original/State-of-Quality-API-2023-Report.pdf)
- GitHub (2022): before they moved to schema-driven examples, request body examples in GitHub's generated reference were "either placeholders or the parameter's name," which the team calls "a confusing experience." Examples are now "driven entirely from our OpenAPI schema," and GitHub CLI examples were added. — [GitHub Blog, Improved REST API documentation (Rachael Sewell, 24 May 2022)](https://github.blog/2022-05-24-improved-rest-api-documentation/)
- Adyen (2018) "filled the documentation with predefined examples" for common tasks in its explorer. — [Adyen](https://www.adyen.com/knowledge-hub/improving-the-developer-experience-with-openapi-specification)
- Lorna Mitchell (2023): in "almost every organisation I talk to," the OpenAPI description "is lacking words, examples, links or other additions that would make the documentation output better," because a code-generated spec "only contains the information that was in the serverside code, and if you're lucky some comment metadata." — [lornajane.net, Improve existing OpenAPI descriptions (28 Feb 2023)](https://lornajane.net/posts/2023/overlay-improvements-to-openapi)
- `x-codeSamples` puts hand-written multi-language samples in the spec, and they take precedence over generated ones. Speakeasy (vendor) notes that these snippets "may not correspond to a usage snippet from an existing SDK," which "reduces the value of the API documentation." Redocly allows `$ref` to separate sample files so the samples can be validated and reused. — [Speakeasy x-codeSamples guide](https://www.speakeasy.com/guides/openapi/x-codesamples); [Redocly x-codeSamples](https://redocly.com/docs/realm/author/reference/openapi-extensions/x-code-samples)
- Consumer side, HN 2018: Stripe and Twilio were praised for "clear examples (some are even interactive)" (greenyouse) and plot.ly for examples "in multiple languages" (jackgolding). Auth0's examples "neither worked without a lot of hacking/changes" (fanpuns). — [HN 17905919](https://news.ycombinator.com/item?id=17905919)
- Rust's practice of running doc examples as tests is cited as what keeps examples accurate (steveklabnik, HN 2019). — [HN 21290458](https://news.ycombinator.com/item?id=21290458)
- Stripe inserts the reader's own API test keys into code samples, and readers choose their language. — [Stripe Markdoc post](https://stripe.dev/blog/markdoc); [Mintlify blog on Stripe docs (vendor, 10 Nov 2023)](https://www.mintlify.com/blog/stripe-docs)

### Inferences
- Examples suffer the same split as prose. Schema examples live in the spec, tutorial code samples live in guides, and nothing checks that one matches the other or the SDK. Being able to reference one example file from both places, and validate it against the schema, would address a top-ranked need.
- The evidence that examples go stale is mostly indirect: SDK drift, and Auth0 samples that didn't work. No source counts stale examples directly.

### Gaps
- No first-hand account of a CI process for testing `x-codeSamples` against live SDKs was found.

## 4. Collaboration: access, review, accountability, writers brought in late

### Takeaway
The spec is usually owned by engineers, and often generated from code annotations. Writers need access to the source to fix descriptions, or their edits get overwritten on regeneration. Review is described as a "hurried engineer" nodding approval. OpenAPI Overlays exist largely to give writers a layer of their own.

### Cited Findings
- Tom Johnson: "It's common that engineers generate the OpenAPI spec." If the spec is generated, "make sure you get access to the source code to make edits to the annotations," otherwise "your developers will be writing your reference docs." Generating from code means the spec "can no longer serve as the blueprint for creating the API." He usually wrote specs by hand because his developers were unfamiliar with OpenAPI and "not eager" to document. — [I'd Rather Be Writing, OpenAPI intro](https://idratherbewriting.com/learnapidoc/pubapis_openapi_intro.html)
- Johnson on review quality (2020): "Tech writers can pretty much write what they want, get an overworked/hurried engineer to nod approval, and then hit publish." SmartBear 2020 found only 15% of organisations use technical writers, and then only when machine-readable definitions can't generate the docs. — [I'd Rather Be Writing, SmartBear 2020 review](https://idratherbewriting.com/blog/smartBear-2020-state-of-api-docs-review/)
- Phil Sturgeon (8 Apr 2024): when the spec is regenerated from code, writers' edits are lost. "Instead of having your changes overridden, you can put all of your expanded guides and descriptions into an overlay, and have CI patch them together before deploying." — [APIs You Won't Hate, Use OpenAPI Overlays today](https://apisyouwonthate.com/blog/use-openapi-overlays-today/)
- The OpenAPI Initiative's Overlay 1.0 announcement uses as its example "an Overlay maintained by your technical writers" that adds descriptions to every operation. — [Overlay Specification 1.0](https://spec.openapis.org/overlay/v1.0.0); summary via [search of openapis.org](https://www.openapis.org/?p=3563)
- Mitchell (2023) ties poor descriptions to teams that "lack either the knowledge or the enthusiasm to modernise their processes." — [lornajane.net](https://lornajane.net/posts/2023/overlay-improvements-to-openapi)
- Adyen's model: technical writers "work alongside API developers" to make sure the reference has the necessary details and matches the rest of the documentation. Descriptions are written in the spec files. — [Adyen](https://www.adyen.com/knowledge-hub/improving-the-developer-experience-with-openapi-specification)
- Write the Docs community consensus, from the Sept 2021 newsletter recapping Slack: knowing the OpenAPI spec is useful for people defining an API but "not essential" for someone documenting it, since you learn it along the way; understanding YAML and JSON helps. — [Write the Docs newsletter, Sept 2021](https://www.writethedocs.org/blog/newsletter-september-2021/)
- Stripe's culture approach: Dave Nunez (former Documentation Manager) "put one line about documentation on each level of the job ladder" in his first week, and says "Managers need to take this requirement seriously come performance review time." Stripe also runs onboarding writing classes and office hours with technical writers. (Vendor blog that interviews the people involved.) — [Mintlify blog on Stripe docs](https://www.mintlify.com/blog/stripe-docs)
- Stripe moved away from a Ruby/ERB docs app where "content authoring effectively became software development." The validation of Markdoc runs in CI and in an internal VS Code extension, so writers see errors as they type. — [Stripe Markdoc post](https://stripe.dev/blog/markdoc)
- Postman 2025 (5,700+ respondents): 93% of API teams face collaboration blockers, and 55% struggle with inconsistent documentation. Documentation is "scattered across too many places" (chat, wikis, email). — [Postman State of the API 2025](https://www.postman.com/state-of-api/2025/)
- Postman 2024 press release: 39% say inconsistent docs are the biggest roadblock, 44% dig through source code to understand APIs, and 43% rely on colleagues to explain APIs (the last figure is from a secondary summary). — [Business Wire, Postman 2024](https://www.businesswire.com/news/home/20241015081863/en/Postmans-2024-State-of-the-API-Report-Finds-API-First-Approach-Yields-Tangible-Results); [Medium summary](https://medium.com/@naveen.garla/key-insights-from-the-2024-state-of-the-api-report-8f0d26b6a529)

### Inferences
- How writers work depends on whether the spec is generated from code or designed first.
  - **Generated from code:** writers either edit annotations in a codebase they may not be able to reach, or keep an overlay that can silently stop matching the targets it patches.
  - **Designed first:** writers edit YAML directly and run into the problems in section 2.

  Supporting writer-owned descriptions that attach to operations by id, outside the spec file, would help in both workflows.
- Who is accountable for the accuracy of the reference is rarely settled explicitly. The sources describe engineers owning the spec, writers polishing it, and review that rubber-stamps.

### Gaps
- No first-hand source specifically on writers being "brought in late" to API launches was found. It is a common anecdote, but this research found no source to cite for it.
- No data on what fraction of writers have commit access to API source repos.

## 5. Navigation and scale

### Takeaway
Big specs strain both renderers and information architecture. Single-page renderers slow down or freeze; tags, and the vendor extension `x-tagGroups`, are the only grouping tools, with sharp edges; large publishers like GitHub moved from long pages to category pages with every operation in the sidebar.

### Cited Findings
- GitHub (2022): operations had been combined into "very long pages" that "didn't perform or scale well." The REST docs are now split into categories and subcategories. The per-page table of contents was removed, and "all of the pages and operations in the REST API are located directly in the sidebar navigation." — [GitHub Blog](https://github.blog/2022-05-24-improved-rest-api-documentation/)
- GitHub publishes a separate OpenAPI description for each product and each date-based version, in bundled and dereferenced forms, and generates each endpoint's reference from it. — [GitHub Docs, About the OpenAPI description](https://docs.github.com/en/rest/about-the-rest-api/about-the-openapi-description-for-the-rest-api)
- ServiceStack forum: a roughly 1.6 MB spec for about 360 APIs makes Swagger UI hang in the browser, though the page comes back fine with curl. The suggested fix is splitting the document. — [ServiceStack forum](https://forums.servicestack.net/t/swagger-ui-just-hangs-on-load/12692)
- A 1.8 MB example response locked Swagger UI's main thread for over 20 seconds. A later commenter says that years later there is still no flag to turn off pretty-printing. — [Lightrun mirror of swagger-ui issue](https://lightrun.com/answers/swagger-api-swagger-ui-large-response-bodies-cause-hanging)
- Other reports of large specs: a SmartBear community post of a single 7–8 MB operation taking minutes to load in Chrome, and Retool users whose tab froze on a large OpenAPI 3.1 spec. — [SmartBear community](https://community.smartbear.com/tag/large%20spec); [Retool community](https://community.retool.com/t/openapi-spec-freezes-browser-tab/28403)
- Redoc lists "performance optimizations" and "better navigation (menu improvements + search)" as open roadmap items, and offers a `nativeScrollbars` option "for big specs." — [Redoc README (fork mirror)](https://github.com/MasonM/ReDoc)
- With `x-tagGroups`, "any tag that is not in a group will not be included in the output," so partly grouped specs silently drop operations. — [Redocly x-tagGroups docs](https://redocly.com/docs/realm/content/api-docs/openapi-extensions/x-tag-groups.md)
- Adyen added its own extensions (`x-groups`, `x-groupName`, `x-sortIndex`) to control grouping and sort order, because tags were not enough. — [Adyen](https://www.adyen.com/knowledge-hub/improving-the-developer-experience-with-openapi-specification)
- Postman 2025: 34% cannot find existing APIs, and teams duplicate work because they can't discover internal APIs. — [Postman State of the API 2025](https://www.postman.com/state-of-api/2025/)
- HN 2018: consumers complain about poor search and missing headings in big reference docs (PostgreSQL "the absolute worst," Python search "a major weakness"). These are not OpenAPI docs, but they are the same complaints about navigating a large reference. — [HN 17905919](https://news.ycombinator.com/item?id=17905919)

### Inferences
- Both large teams in the sources (GitHub, Adyen) ended up with per-category or per-operation pages and custom grouping metadata. A compiler that emits per-operation pages, lets authors define the navigation outside of tags, and builds a search index would match where large teams end up.
- Tags do two jobs: they organise the API, and they serve as the navigation. Any vendor extension that supplies the navigation instead (`x-tagGroups`, `x-sortIndex`) is not portable between tools.

### Gaps
- No first-hand writer account was found of running a portal that combines many microservice specs (for example, how cross-API search or shared schemas are handled). Box and DigitalOcean posts were not found in this pass.

## 6. Versioning and changelogs (brief)

### Takeaway
The writers' side of versioning gets little first-hand discussion in the sources. Big publishers keep a full spec for each version; changelogs end up stuffed into `info.description`.

### Cited Findings
- GitHub keeps a separate description for each product and each date-based version (it publishes GitHub Enterprise Server docs for every supported release). — [GitHub Docs](https://docs.github.com/en/rest/about-the-rest-api/about-the-openapi-description-for-the-rest-api)
- Adyen keeps specs in versioned folders and says generating the reference "supports versioning." — [Adyen](https://www.adyen.com/knowledge-hub/improving-the-developer-experience-with-openapi-specification)
- A team put a long changelog into the Redoc introduction and asks for a way to keep its headings out of the table of contents. — [Redoc #2876](https://github.com/Redocly/redoc/issues/2876)
- Airflow's spec marks release changes with bold text inside descriptions. — [Redocly blog](https://redocly.com/blog/lint-markdown)

### Inferences
- With no proper home for changelogs and "changed in version X" notes, these facts get written as prose in descriptions. That duplicates information a spec diff could derive.

### Gaps
- Another researcher covers breaking-change tooling; nothing further gathered here.

## 7. Consumer complaints that trace back to how the docs were produced

### Takeaway
The complaints that recur are drift (the reference disagrees with the API), docs that list fields without saying how to use them, missing examples, and undocumented errors and auth. Surveys consistently put time pressure and drift at the root.

### Cited Findings
- SmartBear 2023, the obstacles to keeping API docs current:
  - limited time 67%;
  - "docs out of sync with the API" 52%;
  - shortage of experienced staff 31%;
  - inadequate tools 28%.

  "Limited time is the biggest obstacle preventing API providers from keeping their documentation up-to-date." — [The Content Wrangler (Scott Abel, 6 July 2023)](https://www.thecontentwrangler.com/p/obstacles-to-keeping-api-docs-up)
- SmartBear 2020 found the same pattern: limited time 64%, docs out of sync 47%. 43% rated their own docs "poor and in need of improvement," and 5% "very good." — [I'd Rather Be Writing](https://idratherbewriting.com/blog/smartBear-2020-state-of-api-docs-review/)
- Postman 2023: 52% said lack of documentation was the biggest obstacle to consuming APIs (via search summary; the PDF was not opened). Postman 2025 reframes the root cause as documentation being scattered, not absent, and lists "inconsistent, outdated, or missing documentation creates confusion about API behavior" among common failures. — [Postman 2023 PDF](https://voyager.postman.com/pdf/2023-state-of-the-api-report-postman.pdf); [Postman 2025](https://www.postman.com/state-of-api/2025/)
- HN 2018–2019 first-hand comments:
  - the docs omit a "damned baseURL" (themoat);
  - Twilio's API and SDK references "vary from practically non-existent to completely unusable" (hardly_a_mirage, 2018);
  - Stripe's React Native library "only implements portions of the stripe API" and is poorly documented (jmknoll);
  - "zero documentation about what happens when you update an object - only OpenAPI" (remotedeveloper, 2019).

  — [HN 17905919](https://news.ycombinator.com/item?id=17905919); [HN 21290458](https://news.ycombinator.com/item?id=21290458)
- Postman 2025, on AI agents as consumers: "AI Agents rely on precise, machine-readable signals, not tribal knowledge." Postman recommends OpenAPI specs with examples and error codes, and docs that explain *when and why* to use each endpoint. Only 24% design APIs with agents in mind. — [Postman 2025](https://www.postman.com/state-of-api/2025/)
- One guide suggests a script that calls every endpoint using the documented examples and checks the responses against the documented schemas, as a way to catch reference drift. — [search summary; source page not opened](https://playbooks.com/skills/secondsky/claude-skills/api-reference-documentation)

### Inferences
- "Errors undocumented" and "auth not explained" connect to surveys rating status/errors and authentication as the second and third most valued elements (sections 3 and 7). In generated reference they are often empty, because error responses and security schemes are thin in code-generated specs (Mitchell's point).

### Gaps
- No first-hand consumer thread that specifically complains about undocumented error responses was found. The summary of HN 17905919 notes that no commenter raised it there. The evidence for it is survey rankings.

## 8. What respected teams do, and the cross-theme ranking

### Takeaway
Stripe, GitHub, Adyen and Plaid each built custom tooling to get a reference with the same look as the guides, rich examples, and one source of truth for docs and SDKs. Even the best-resourced teams end up writing their own renderer.

### Cited Findings
- **Stripe:**
  - Built Markdoc (Markdown plus tags, validated in CI and in the editor) after its Ruby/ERB docs became unmaintainable.
  - Shares React components between the docs and the API reference.
  - Personalises samples with the reader's test keys, and offers samples in several languages.
  - Treats docs "like an application rather than a traditional user manual" (Ryan Paul, Head of Docs).

  — [Stripe Markdoc post (2022)](https://stripe.dev/blog/markdoc); [Mintlify blog (vendor, 2023)](https://www.mintlify.com/blog/stripe-docs)
- **GitHub (2022):**
  - Replaced a "fragile" hand-built reference framework that "couldn't scale" with pages generated from the open-source `github/rest-api-description`.
  - Request examples come from the schema, and CLI examples were added.
  - Category pages, with every operation in the sidebar.

  — [GitHub Blog](https://github.blog/2022-05-24-improved-rest-api-documentation/); [GitHub Docs](https://docs.github.com/en/rest/about-the-rest-api/about-the-openapi-description-for-the-rest-api)
- **Adyen (2018):**
  - A custom API Explorer instead of Swagger UI, because of "the somewhat limited possibilities for customizing the Swagger UI" and because "Good UX requires data, which is not available by default through the Swagger UI."
  - Writers and API developers co-author descriptions in the spec.
  - Custom grouping extensions; public specs at [Adyen/adyen-openapi](https://github.com/Adyen/adyen-openapi).

  — [Adyen](https://www.adyen.com/knowledge-hub/improving-the-developer-experience-with-openapi-specification)
- **Plaid (2021):** maintained "three sources of truth" for its API (docs, client libraries, the API itself) and adopted an OpenAPI schema as the shared reference to generate documentation and SDKs (Stephen Jayakar, 15 Sept 2021; summary only, the full post was not retrieved). — [Plushcap summary of Plaid blog](https://plushcap.com/content/plaid/blog/adopting-the-openapi-schema-to-generate-plaids-sdks)
- **Twilio:** publishes OpenAPI specs for all its APIs. No first-hand account of its docs pipeline was found. — [Twilio OpenAPI docs](https://www.twilio.com/docs/openapi)

### Inferences
Cross-theme ranking of how widespread each pain is. This is a judgement from how often and how broadly each appears in the sources above.
1. **Drift, and no time to maintain the docs.** Measured in every survey: SmartBear 2020 and 2023 put limited time at 64–67% and docs out of sync at 47–52%; Postman 2025 found 55% with inconsistent docs. This is the most widespread problem.
2. **Generated reference that is accurate but thin**, without examples, errors or context. It is the most common first-hand complaint (HN, Mitchell's "almost every organisation," Adyen, GitHub's placeholder examples). Examples are the most valued element, at 66–70%.
3. **Reference and guides split** across two sites or two looks, with duplicated facts. This is a long-running theme in the writers' literature (Johnson's six options), and every respected team in the sources built a custom renderer to escape it.
4. **Prose in YAML.** Parse and indentation failures, Markdown that can't be linted, renderer differences, and diffs that are hard to review. Many GitHub issues report it, but it is felt mainly by the people who edit the spec.
5. **Ownership of a generated spec.** Writers' edits are overwritten or need access to the source, and review rubber-stamps. Overlays exist for this. It is widespread wherever the spec is generated from code.
6. **Scale and navigation.** Renderer freezes, tags as the only structure. This affects mainly large APIs, where the pain is severe.
7. **Links from prose into reference breaking.** Evidence is indirect (route ids change between plugin versions, collisions). It is likely under-reported because nothing checks these links.

### Gaps
- Box, DigitalOcean and Twilio engineering posts on their docs pipelines were not found in this pass.
- No API the Docs conference talks were retrieved.
- Sarah Maddox's material on OpenAPI was not retrieved.
