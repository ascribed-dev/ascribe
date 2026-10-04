# Open-source tools and techniques for documentation drift and verification of AI-written docs (state as of 2026-10-03)

Conventions used in these notes:

- Repository metadata (licence, language, stars, last push, latest release) was read from the GitHub API on 2026-10-03; crate metadata from the crates.io API; npm metadata from the npm registry, same day. The linked repo, crate or package page is the source.
- "Vendor claim" marks a number published by the tool's own maintainers or company, not independently reproduced.
- Integration options for a Rust binary are stated as: **link** (a Rust crate exists), **wasm** (embed through a WebAssembly runtime), **shell out** (spawn an external binary), **reimplement**.
- Statements under "Inferences" are my reasoning, not sourced facts.

## 1. Code samples from source (Bluehawk and alternatives); which approaches test that samples compile or run

### Takeaway
Bluehawk is a small Apache-2.0 TypeScript tool (about 150 KB of non-test source) whose last release was October 2024, and its core is eight comment tags, so a Rust reimplementation is tractable and there is no Rust port to link. Extraction tools (Bluehawk, `literalinclude`, AsciiDoc tags, mdBook anchors) only guarantee that the sample is copied from real code; whether it compiles or runs depends on the source file being part of a test suite, whereas mdBook `test`, Rust doctests, Doc Detective and Runme execute the sample itself.

### Cited Findings

**Bluehawk**
- Bluehawk describes itself as "a markup processor for extracting and manipulating arbitrary code": extract code examples for docs, generate formatted examples, and replace "finished" code with "todo" code for tutorial branches — [Bluehawk README](https://github.com/mongodb-university/Bluehawk)
- Licence text in `LICENSE.txt` is Apache License 2.0, copyright 2021 MongoDB, Inc. (GitHub's licence detector reports `NOASSERTION`, so the file has to be read directly) — [Bluehawk LICENSE.txt](https://github.com/mongodb-university/Bluehawk/blob/main/LICENSE.txt)
- Implementation language is TypeScript (315,000 bytes of TypeScript reported by GitHub); 35 stars, 24 open issues, not archived — [Bluehawk repo](https://github.com/mongodb-university/Bluehawk)
- Latest release is 1.6.0, published 2024-10-18 on GitHub and npm; the npm package was last modified 2024-10-18 — [Bluehawk releases](https://github.com/mongodb-university/Bluehawk/releases), [bluehawk on npm](https://www.npmjs.com/package/bluehawk)
- The five most recent commits (all 2025-11-27) are edits to a notification workflow file (`devdocs-notify.yml`), not to the tool itself; last push 2025-11-27 — [Bluehawk commits](https://github.com/mongodb-university/Bluehawk/commits/main)
- npm reported 2,218 downloads for 2026-09-02 to 2026-10-01 — [npm downloads API](https://api.npmjs.org/downloads/point/last-month/bluehawk)
- Size: 78 non-test `.ts` files under `src/` totalling about 152 KB; about 315 KB including tests. Tag implementations live in `src/bluehawk/tags/` (`SnippetTag`, `RemoveTag`, `ReplaceTag`, `StateTag`, `StateRemoveTag`, `UncommentTag`, `EmphasizeTag`), with a parser under `src/bluehawk/parser/` — [Bluehawk source tree](https://github.com/mongodb-university/Bluehawk/tree/main/src/bluehawk)
- Runtime dependencies of 1.6.0: `chevrotain` (parser toolkit), `magic-string`, `ajv` (JSON Schema validation of tag attributes), `yargs`, `memfs`, `source-map`, `ignore`, `isbinaryfile` — [bluehawk on npm](https://www.npmjs.com/package/bluehawk)
- Tag syntax: tags are written inside source comments and wrapped in colons; a line form (`:tag:`) and a block form (`:tag-start:` / `:tag-end:`). Attribute lists are JSON objects on the same line as the opening tag, for example `// :replace-start: { "terms": {"old": "new"} }` — [Bluehawk tags reference](https://mongodb-university.github.io/Bluehawk/reference/tags/)
- The eight tags: `snippet` (block only; needs an identifier unique in the file; `bluehawk snip` writes one file per snippet, for example `Main.snippet.test-block.java`), `state` (block; keeps a range only for named states, selected with `--state`), `state-uncomment`, `state-remove`, `uncomment` (strips one comment level per line; language-dependent), `replace` (substitutes terms inside a range), `emphasize` (line or block; highlights lines in formatted output, needs `--format`), `remove` (line or block; excludes content such as assertions and test setup) — [Bluehawk tags reference](https://mongodb-university.github.io/Bluehawk/reference/tags/)
- CLI commands include `snip`, `copy` and `list tags`; `--format` emits RST or Markdown with emphasis — [Bluehawk tags reference](https://mongodb-university.github.io/Bluehawk/reference/tags/)

**Alternatives that extract (no execution of their own)**
- Sphinx `literalinclude` selects content with `:lines:`, `:start-after:` / `:end-before:`, `:start-at:` / `:end-at:`, `:pyobject:` (a named Python class, function or method), `:diff:`, and highlights with `:emphasize-lines:` — [Sphinx directives](https://www.sphinx-doc.org/en/master/usage/restructuredtext/directives.html)
- AsciiDoc tagged regions use `tag::name[]` / `end::name[]` placed after a line comment in the source file's language; includes can filter with `*`, `**` and `!` (for example `tag=foo;!bar`) — [Asciidoctor tagged regions](https://docs.asciidoctor.org/asciidoc/latest/directives/include-tagged-regions/)
- mdBook `{{#include file.rs:2:10}}` takes line ranges, or named anchors where "the line beginning an anchor must match the regex `ANCHOR:\s*[\w_-]+`" and the end line `ANCHOR_END:\s*[\w_-]+` — [mdBook-specific features](https://rust-lang.github.io/mdBook/format/mdbook.html)
- `embedme` (TypeScript, MIT): latest release v1.22.1 on 2022-09-07, last push 2024-10-06, 238 stars — [embedme repo](https://github.com/zakhenry/embedme)
- `markdown-autodocs` (JavaScript, MIT): latest GitHub release v1.0.7 on 2022-08-29; npm latest 1.0.133 on 2022-09-19; 196 stars — [markdown-autodocs repo](https://github.com/dineshsonachalam/markdown-autodocs)
- `mdsh` (Rust, MIT, "Markdown shell pre-processor"): crate 0.7.0 published 2022-11-03; repo last pushed 2026-07-23; 174 stars; 85 recent crate downloads — [mdsh on crates.io](https://crates.io/crates/mdsh), [mdsh repo](https://github.com/zimbatm/mdsh)
- `cog` (Python, MIT): 406 stars, last push 2026-08-05 — [cog repo](https://github.com/nedbat/cog)

**Approaches that execute or compile the sample**
- mdBook `{{#rustdoc_include}}` includes a whole Rust file but hides the lines outside the selected range, so that readers can expand them and `mdbook test` compiles the complete example; `mdbook test` honours `ignore`, `should_panic`, `no_run` and `compile_fail` — [mdBook-specific features](https://rust-lang.github.io/mdBook/format/mdbook.html)
- mdBook is Rust, MPL-2.0, v0.5.4 (2026-07-06), 22,189 stars — [mdBook repo](https://github.com/rust-lang/mdBook)
- `skeptic` ("Test your Rust markdown documentation via Cargo"): crate 0.13.7 published 2022-02-01, repo last pushed 2024-03-25, but still 1.9 million recent downloads — [skeptic on crates.io](https://crates.io/crates/skeptic), [rust-skeptic repo](https://github.com/budziq/rust-skeptic)
- `trycmd` ("Snapshot testing for a herd of CLI tests"): crate 1.2.1, updated 2026-07-21, MIT OR Apache-2.0, 7.3 million downloads — [trycmd on crates.io](https://crates.io/crates/trycmd)
- `term-transcript` ("Snapshotting and snapshot testing for CLI / REPL applications"): crate 0.5.0, updated 2026-07-18, MIT OR Apache-2.0 — [term-transcript on crates.io](https://crates.io/crates/term-transcript)
- Runme (Go, Apache-2.0, v3.17.5 on 2026-08-28, 2,181 stars) "runs your commands (shell, bash, zsh) or code inside your fenced code blocks"; code blocks are named with fence attributes such as ```` ```sh { name=update-brew } ```` — [Runme README](https://github.com/runmedev/runme)
- Doc Detective (TypeScript, AGPL-3.0, v4.38.1 on 2026-08-13, 134 stars) "ingests test specifications and text files, parses them for testable actions, then executes those actions in a browser"; its docs list shell commands (`runShell`), code (`runCode`), API calls (`httpRequest`), UI interactions and screenshots, across Markdown, AsciiDoc and DITA — [Doc Detective README](https://github.com/doc-detective/doc-detective), [Doc Detective docs](https://docs.doc-detective.com/)
- Doc Detective's install pulls "browsers (Chrome, Firefox), drivers (ChromeDriver, Geckodriver), ffmpeg" and npm packages including webdriverio, appium and sharp; requires Node.js (tested on v20 and v22) — [Doc Detective README](https://github.com/doc-detective/doc-detective)
- `@mdx-js/mdx` is MIT, latest 3.1.1 published 2025-08-29 — [@mdx-js/mdx on npm](https://www.npmjs.com/package/@mdx-js/mdx)

### Inferences
- Bluehawk is in maintenance-only state at best: no release in two years and recent commits touch only CI notification. Depending on it would mean shipping Node alongside a native binary; no Rust or WebAssembly build exists in the repo.
- A Rust reimplementation needs: (a) a per-language comment-token table (line and block comment delimiters, the role of Bluehawk's `LanguageSpecification`); (b) a scanner for `:name:`, `:name-start:` and `:name-end:` inside comments with nesting checks; (c) JSON attribute parsing (`serde_json`); (d) eight tag handlers that are range edits on text (delete, keep, replace, strip one comment prefix, record highlighted lines); (e) dedent of the extracted range; (f) diagnostics for unclosed or duplicated identifiers. About 150 KB of TypeScript includes the CLI, plugin loading and a parser-generator layer that a hand-written Rust scanner would not need, so the essential part is considerably smaller.
- Because Ascribe already has an include directive, the cheapest drift-proof design is "include by named region from a file that the product's own test suite compiles and runs", which is what Bluehawk, AsciiDoc tags and mdBook anchors all converge on. The docs tool then only has to fail the build when a named region disappears; execution stays in the product's test runner.
- Compatibility choice: recognising Bluehawk's `:snippet-start:` markers, AsciiDoc's `tag::`/`end::` markers and mdBook's `ANCHOR:` markers costs little, since all three are "named region delimited by marker comments", and lets users adopt Ascribe without re-tagging source.
- Tools that execute samples in place (Runme, Doc Detective, `mdsh`, `cog`) need a sandbox, a runtime and time budgets. They fit a `check --run` or CI command, not the language server's keystroke path.
- Doc Detective's AGPL-3.0 licence and its browser/Appium runtime make it a shell-out-only candidate; linking or bundling is unattractive on both licence and size grounds.

### Gaps
- I did not fetch the READMEs or docs for `embedme`, `markdown-autodocs`, `cog`, `mdsh`, Docusaurus, Antora or MDX code-import plugins, so their exact marker syntax is not documented here. Only repository metadata was verified for the first four; Docusaurus and Antora include mechanisms and MDX "code import" plugins were not checked at all.
- Whether AsciiDoc and mdBook warn or fail on a missing tag or anchor was not stated on the pages read.
- I found no measurement of how many docs sites using include-by-tag also run the tagged source in CI.
- Rust doctests (`cargo test --doc`, rustdoc) were not looked up in primary docs in this session; the only doctest-related evidence above is mdBook's `mdbook test` and the `skeptic` crate.

## 2. OpenAPI: Rust parsing crates, diff and breaking-change tools, how docs tools render reference

### Takeaway
No single mature Rust crate parses both OpenAPI 3.0 and 3.1: `openapiv3` is 3.0 only and widely used, `oas3` is 3.1 only, and the one crate that claims both (`roas`) is young. There is no maintained Rust spec-diff library; `oasdiff` (Go, Apache-2.0, released 2026-10-01) is the live breaking-change tool and Optic is archived.

### Cited Findings

**Rust crates**
- `openapiv3` 2.2.0 (last updated 2025-06-02; MIT/Apache-2.0; 13.6 million downloads, 3.1 million recent): "data structures that represent the OpenAPI v3.0.x specification"; the README states "this does not cover OpenAPI v3.1 which was an incompatible change". Stated non-goal: round-trip fidelity ("Some defaults show-up when serializing that may not have existed in the input") — [openapiv3 on crates.io](https://crates.io/crates/openapiv3), [openapiv3 README](https://github.com/glademiller/openapiv3)
- `oas3` 0.22.0 (2026-05-06; 2.4 million downloads, 1.15 million recent; repo last pushed 2026-10-01; MSRV 1.87): "Structures and tools to parse, navigate and validate OpenAPI v3.1.x specifications", with the warning "due to v3.1.x being a breaking change from v3.0.x, you may have trouble correctly parsing specs in the older format". API: `oas3::from_yaml`, `from_json`, `Spec`; order-preserving maps — [oas3 on crates.io](https://crates.io/crates/oas3), [oas3-rs README](https://github.com/x52dev/oas3-rs)
- `oas3` licence: the crates.io version record says MIT; the README badge reads "MIT or Apache 2.0 licensed" — [oas3 on crates.io](https://crates.io/crates/oas3); differs from [oas3-rs README](https://github.com/x52dev/oas3-rs)
- `utoipa` 6.0.0 (2026-09-22; MIT OR Apache-2.0; 50.8 million downloads; 4,114 stars): "Compile time generated OpenAPI documentation for Rust"; lists OpenAPI 3.1 as a feature and its example output is `"openapi": "3.1.0"`. It is a code-first generator for Rust web frameworks (actix-web, axum, rocket) — [utoipa on crates.io](https://crates.io/crates/utoipa), [utoipa README](https://github.com/juhaku/utoipa)
- `openapiv3-extended` 6.0.1 (2025-08-08; a fork with the same 3.0.x description; 496 thousand downloads) — [openapiv3-extended on crates.io](https://crates.io/crates/openapiv3-extended)
- `openapiv3_1` 0.1.5 (2026-03-09; "OpenAPI 3.1.x bindings for rust"; 48,590 downloads) — [openapiv3_1 on crates.io](https://crates.io/crates/openapiv3_1)
- `roas` 0.21.0 (2026-09-20; MIT OR Apache-2.0; 11,374 downloads): "parser, validator, and loader for OpenAPI v2.0 / v3.0.x / v3.1.x / v3.2.x" — [roas on crates.io](https://crates.io/crates/roas)
- `schemadoc-diff` 0.1.20 ("OpenApi diff library and breaking changes detector", Apache-2.0): last published 2023-07-13, 1,505 total downloads — [schemadoc-diff on crates.io](https://crates.io/crates/schemadoc-diff)

**Diff and breaking-change tools**
- `oasdiff`: Go, Apache-2.0, v1.33.0 released 2026-10-01, 1,399 stars. "Command-line tool to compare and detect breaking changes in OpenAPI specs." Subcommands `diff`, `breaking`, `changelog`, `checks changelog coverage` ("every possible edit of an OpenAPI document with the checks that cover it"). Documents OpenAPI 3.1 and 3.2 support. Consumed as CLI, Go package ("Embed in a Go program"), GitHub Action, Docker image; output in HTML, JSON, Markdown, text and YAML. Built on `kin-openapi` — [oasdiff README](https://github.com/oasdiff/oasdiff)
- oasdiff.com is a hosted offering "for teams, which adds a per-change PR comment with approve/reject and commit-status checks" (vendor description) — [oasdiff README](https://github.com/oasdiff/oasdiff)
- `openapi-diff` (OpenAPITools): Java, Apache-2.0, 2.1.7 released 2026-01-26, last push 2026-09-04, 1,097 stars — [openapi-diff repo](https://github.com/OpenAPITools/openapi-diff)
- Optic: TypeScript, MIT, repository archived; last release v1.0.9 on 2025-08-10; last push 2026-01-08 — [optic repo](https://github.com/opticdev/optic)
- A secondary write-up dates the archive to 2026-01-12 and says it followed Atlassian's acquisition in April 2024; oasdiff publishes a migration guide from Optic — [DEV Community post](https://dev.to/flarecanary/optic-is-dead-what-now-for-api-drift-detection-2kb8), [oasdiff migration guide](https://www.oasdiff.com/docs/migrate-from-optic)

**How docs tools render reference from a spec**
- Redoc (TypeScript, MIT, v2.5.3 on 2026-05-29, 25,941 stars): supports "OpenAPI 3.1, OpenAPI 3.0, and Swagger 2.0"; "provided as a CLI tool (also distributed as a Docker image), HTML tag, and React component"; `npx @redocly/cli build-docs openapi.yaml` writes a single `redoc-static.html`; alternatively a `<redoc spec-url=...>` element plus `redoc.standalone.js` renders client-side — [Redoc README](https://github.com/Redocly/redoc)
- `@redocly/cli` is MIT, 2.57.0 published 2026-09-30 — [@redocly/cli on npm](https://www.npmjs.com/package/@redocly/cli)
- Scalar (TypeScript, MIT, 16,219 stars, release tagged 2026-10-02): "All you need is a single HTML file. Load the ESM build from our CDN, with no build step required" using `createApiReference` from `@scalar/api-reference`; lists a Rust integration and a package "Scalar OpenAPI to Markdown" — [Scalar README](https://github.com/scalar/scalar)
- Rust wrappers that embed these JavaScript renderers into a server: `utoipa-scalar` 0.4.0, `utoipa-redoc` 7.0.0, `utoipa-swagger-ui` 10.0.1 (all updated 2026-09-22/23) — [utoipa-scalar](https://crates.io/crates/utoipa-scalar), [utoipa-redoc](https://crates.io/crates/utoipa-redoc), [utoipa-swagger-ui](https://crates.io/crates/utoipa-swagger-ui)
- `docusaurus-plugin-openapi-docs` (TypeScript, MIT, v5.2.0 on 2026-08-11, 1,102 stars) "extends the Docusaurus CLI with commands for generating MDX using the OpenAPI specification as the source"; works with Swagger 2.0 and OpenAPI 3.x; requires Docusaurus 3.10.0+ — [docusaurus-openapi-docs README](https://github.com/PaloAltoNetworks/docusaurus-openapi-docs)
- Mintlify's CLI package `mint` is licensed Elastic-2.0 (4.2.984, published 2026-10-03); Fern's CLI package `fern-api` (5.144.3, published 2026-10-02) has no licence field in its npm metadata — [mint on npm](https://www.npmjs.com/package/mint), [fern-api on npm](https://www.npmjs.com/package/fern-api)

### Inferences
- Three rendering patterns are visible: (1) ship a JavaScript bundle that renders the spec in the browser (Redoc, Scalar, Swagger UI); (2) generate one static HTML file at build time (Redocly `build-docs`); (3) generate ordinary docs pages (MDX) from the spec so that they flow through the normal content pipeline (the Docusaurus plugin). Pattern 3 is the one that maps onto a typed content model: operations and schemas become first-class nodes that prose can reference, and a dangling reference becomes a `check` error.
- For parsing, the pragmatic choice is to sniff the `openapi` version string and dispatch to `openapiv3` (3.0) or `oas3` (3.1), or to evaluate `roas` as a single dependency once it has more adoption. All of these are plain serde crates, so they **link** cleanly and are cheap enough for a language server.
- For breaking-change detection the options are: **shell out** to `oasdiff` (a separate Go binary, the only actively released tool with a documented rule catalogue); compile `oasdiff` to WebAssembly (not something the project advertises); or **reimplement** a subset (removed path, removed operation, new required parameter, removed response property, enum narrowing) on top of the parsed model. For the drift use case, a narrower check is enough: "does every operation, parameter and schema that the prose mentions still exist in the spec", which needs no diff engine.

### Gaps
- Whether `utoipa`'s OpenAPI types are suitable for deserialising arbitrary third-party specs was not checked; it is documented as a generator.
- `roas` maturity (test coverage against real-world specs, error quality) was not evaluated; only registry metadata was read.
- How Fern and Mintlify render OpenAPI internally was not verified from primary sources; both are hosted, largely proprietary pipelines.
- The exact count of oasdiff breaking-change checks was not retrieved.
- No independent comparison of false-positive rates between oasdiff and openapi-diff was found.

## 3. Prose linting: Vale and Rust-native alternatives

### Takeaway
Vale (Go, MIT, v3.24.0 released 2026-10-01) remains the reference prose linter, but it is consumed as an external binary; even its official language server is a Rust wrapper that manages the Vale binary. The linkable Rust options are `harper-core` (grammar and spelling, Apache-2.0), `typos` (source-aware spelling), and `rumdl` (Markdown structure rules), which cover different ground from Vale's style-guide rules.

### Cited Findings

**Vale**
- Go, MIT, 6,195 stars, v3.24.0 released 2026-10-01, repo now at `vale-cli/vale` (redirected from `errata-ai/vale`), 14 open issues — [vale repo](https://github.com/vale-cli/vale)
- Speed claim (vendor): GitLab documentation, "2,827 Markdown pages", "82 Rules applied", "Start to finish <20s"; "One Go binary. Parallel checks. No separate runtime."; runs offline on macOS, Windows and Linux — [vale.sh](https://vale.sh/)
- Formats listed: Markdown, AsciiDoc, reStructuredText, MDX, MyST, Quarto, Typst, HTML, XML, DITA, Org, QDoc; code comments in 19 languages; also OpenAPI, Jupyter notebooks and commit messages — [vale.sh](https://vale.sh/)
- Consumption: editors (VS Code, Neovim, Sublime Text, Zed, Emacs, JetBrains, Obsidian and others), GitHub Actions, pre-commit hooks, Language Server Protocol; the site also names Mintlify and Promptless as documentation platforms that integrate it — [vale.sh](https://vale.sh/)
- `vale-ls` (Rust, MIT, v0.6.0 released 2026-10-01, 128 stars) "provides high-level interface for managing Vale and its assets (binary, `StylesPath`, etc.)", with hover, completion of StylesPath assets, document links and code actions — [vale-ls README](https://github.com/vale-cli/vale-ls)
- MDX: "Vale v3.18.0 or later parses MDX natively"; earlier versions needed the external npm tool `mdx2vast`. Vale skips JSX tags, ESM `import`/`export`, JavaScript expressions, fenced blocks and code spans. From v3.19.0, JSX children are linted as Markdown and the element name becomes a class scope (`scope: text.class.Aside`), so a component can be targeted or excluded by name. In-file controls use MDX comments such as `{/* vale off */}` and `{/* vale Style.Redundancy = NO */}` — [Vale MDX docs](https://docs.vale.sh/formats/mdx)
- Scoping: Vale "skips code spans, URLs, and fenced blocks before a rule ever runs" and rules can target headings, lists or table cells; it uses tree-sitter grammars for comments in source code — [vale repo](https://github.com/vale-cli/vale)

**Harper**
- Rust, Apache-2.0, 16,126 stars, GitHub release v2.12.0 on 2026-10-01; maintained under Automattic — [harper repo](https://github.com/Automattic/harper)
- `harper-core` crate: 2.11.0 (2026-09-16), 119,598 downloads, crate size about 1.85 MB, documented at 45% on docs.rs — [harper-core on crates.io](https://crates.io/crates/harper-core), [harper-core docs](https://docs.rs/harper-core/latest/harper_core/)
- Library use is a few lines: `Document::new_curated(text, &parser)`, `FstDictionary::curated()`, `LintGroup::new_curated(dict, Dialect::American)`, `linter.lint(&document)`. A `concurrent` feature swaps `Rc` for `Arc` and is off by default — [harper-core docs](https://docs.rs/harper-core/latest/harper_core/)
- `harper-core` depends on `pulldown-cmark`, `fst`, `levenshtein_automata`, `regex`, `hashbrown`, `serde` and others; no async runtime or network crate in its normal dependencies — [harper-core on crates.io](https://crates.io/crates/harper-core)
- Author's claims (vendor): "it take[s] milliseconds to lint a document, take less than 1/50th of LanguageTool's memory footprint"; LanguageTool "would take several seconds to lint even a moderate-size document" and needs a ~16 GB n-gram dataset. "Harper currently only supports English" — [harper README](https://github.com/Automattic/harper)
- Also ships `harper-ls` and a WebAssembly build (`harper.js`) — [harper repo](https://github.com/Automattic/harper)

**typos**
- Rust, MIT OR Apache-2.0, 4,172 stars, v1.50.3 released 2026-09-25 — [typos repo](https://github.com/crate-ci/typos)
- Two crates: the library `typos` 0.10.44 (2026-08-03; 16 KB; dependencies `bstr`, `itertools`, `serde`, `simdutf8`, `unicode-ident`, `winnow`; MSRV 1.91) and the application `typos-cli` 1.50.3 — [typos on crates.io](https://crates.io/crates/typos), [typos-cli on crates.io](https://crates.io/crates/typos-cli)

**Markdown structure linters**
- `rumdl`: Rust, MIT, v0.2.78 (2026-09-29), 1,548 stars; 88 rules; flavours for GFM, MkDocs, MDX, Quarto, MyST; built-in LSP; crates.io marks the crate as having a library target — [rumdl README](https://github.com/rvben/rumdl), [rumdl on crates.io](https://crates.io/crates/rumdl)
- rumdl benchmark (vendor, February 2026, Rust Book repo, 478 files, includes process start-up): rumdl 217 ms, mado 77 ms, pymarkdown 240 ms, remark-lint 671 ms, markdownlint-cli2 2.2 s, markdownlint-cli 2.7 s. The page concedes "tool capabilities and workloads are not identical" and that exact tool versions were not retained — [rumdl benchmarks](https://rumdl.dev/benchmarks/)
- `mado`: Rust, Apache-2.0, v0.3.2 (2026-09-06), 413 stars; "supports most markdownlint rules"; claims "approx. 49-60x faster than existing linters": 0.129 s against 6.4 to 7.8 s for markdownlint variants on roughly 1,500 GitLab docs files on an M1 Max (vendor) — [mado README](https://github.com/akiomik/mado)
- The crates.io name `mado` belongs to an unrelated crate ("macOS active app and window monitoring"), so akiomik's linter is not available under that crate name — [mado on crates.io](https://crates.io/crates/mado)
- `markdownlint` (JavaScript, MIT, 6,367 stars, pushed 2026-10-04) is the reference rule set these ports follow; `markdownlint-cli2` 0.23.3 published 2026-09-20 — [markdownlint repo](https://github.com/DavidAnson/markdownlint), [markdownlint-cli2 on npm](https://www.npmjs.com/package/markdownlint-cli2)

**LanguageTool and others**
- LanguageTool: Java, LGPL-2.1, 15,102 stars, last push 2026-10-02 — [languagetool repo](https://github.com/languagetool-org/languagetool)
- `languagetool-rust` 3.0.1 (2025-12-31, MIT) is "LanguageTool API bindings in Rust", that is, an HTTP client for a LanguageTool server, not an embedded engine — [languagetool-rust on crates.io](https://crates.io/crates/languagetool-rust)
- `ltex-ls` (LanguageTool language server) is archived; last release 16.0.0 on 2023-03-19 — [ltex-ls repo](https://github.com/valentjn/ltex-ls)
- `nlprule` (pure-Rust engine for LanguageTool rules) was last published 2021-04-24 — [nlprule on crates.io](https://crates.io/crates/nlprule)
- `spellbook` 0.4.2 (2026-06-03, MPL-2.0): "A spellchecking library compatible with Hunspell dictionaries" — [spellbook on crates.io](https://crates.io/crates/spellbook)

### Inferences
- Linkable as Rust crates: `harper-core` (yes, synchronous, no network), `typos` (yes, tiny tokenizer and checker; the dictionary data lives in sibling crates), `rumdl` (has a lib target, though it is published primarily as an application), `spellbook`. Not linkable: Vale (Go), LanguageTool (Java; reachable only over HTTP), `mado` (not published as a library under its name).
- Vale integration has to be **shell out**: spawn `vale --output=JSON` and map alerts to diagnostics, as `vale-ls` and every editor plugin do. That is acceptable on save or in `check`, but a process spawn per keystroke is the wrong shape for the language server; debounce or run on save. The useful part for Ascribe is the reverse direction: Ascribe knows its own directive, include and variant structure, so it can hand Vale flattened prose with a source map, or publish scope information so Vale rules do not fire inside directive arguments.
- Vale's native MDX handling and component-name scoping (v3.18 to v3.19) is the precedent for scoping prose rules by directive name; an Ascribe-native rule engine could expose scopes such as `directive.note` or `phrase` the same way.
- What Vale offers and none of the Rust crates do is a user-authored, YAML-defined style-rule format with a package ecosystem (Microsoft, Google, write-good styles). Reimplementing a subset of Vale's rule types (existence, substitution, occurrence, capitalization) over Ascribe's typed tree is feasible with the `regex` crate, but compatibility with existing Vale packages is the real value, and that argues for shelling out when users already have a `.vale.ini`.
- Harper's millisecond claim and dependency list make it the only grammar checker here that plausibly fits a per-keystroke budget inside the same process. Its cost is binary size (the curated dictionary ships in the crate, 1.85 MB compressed) and English-only coverage.
- All speed numbers above are maintainers' own benchmarks with process start-up included and unequal rule sets; they establish order of magnitude (native tools in tens to hundreds of milliseconds, Node tools in seconds), not rankings.

### Gaps
- No official Vale library API or WebAssembly build was found in the README or site content read; I did not search the Vale issue tracker for a WASM port, so its absence is not confirmed.
- I found no independent, reproducible benchmark of Vale against textlint or proselint. Search snippets from third-party blogs quote figures such as "1.5 seconds versus 50 seconds for proselint", but I did not open or verify those posts.
- `rumdl`'s library API surface could not be read (the docs.rs page failed to load), so how stable or intended that API is remains unknown.
- Harper's false-positive rate on technical documentation (identifiers, product names) was not assessed; no independent evaluation was found. The repository has 966 open issues, which was not analysed.

## 4. Link checking: lychee, htmltest, linkinator

### Takeaway
`lychee` is the only one of the three that is both Rust and published as a library (`lychee-lib`, Apache-2.0 OR MIT), but it brings tokio, reqwest and a GitHub client as dependencies, and its README makes no quantitative speed claim. `htmltest` has not had a release since November 2022; `linkinator` is an actively released Node tool.

### Cited Findings
- lychee: Rust, dual Apache-2.0 / MIT, 3,978 stars, latest release `lychee-v0.24.2` on 2026-05-01, last push 2026-09-28. Checks Markdown, HTML and reStructuredText, with a plain-text fallback. Async with default concurrency of 128 requests, optional on-disk cache (`.lycheecache`), per-host rate limiting, fragment checking — [lychee README](https://github.com/lycheeverse/lychee)
- "lychee is powered by lychee-lib, the Rust library for link checking"; the README's comparison table marks "Use as library" yes for lychee and compares it with awesome_bot, muffet, broken-link-checker, linkinator, linkchecker, markdown-link-check and fink — [lychee README](https://github.com/lycheeverse/lychee)
- `lychee-lib` 0.24.2 (2026-05-01; 211,152 downloads; MSRV 1.88): minimal use is `lychee_lib::check("https://...").await` under `#[tokio::main]`; `ClientBuilder`, `Collector`, `Request` and an `extract` module are the main types — [lychee-lib docs](https://docs.rs/lychee-lib/latest/lychee_lib/)
- `lychee-lib` normal dependencies include `tokio`, `reqwest`, `hyper`, `octocrab` (GitHub API), `html5ever`, `html5gum`, `pulldown-cmark`, `ring`, `governor`, `cookie_store`, `linkify` and about 25 others — [lychee-lib on crates.io](https://crates.io/crates/lychee-lib)
- The lychee README describes itself as "fast, async, stream-based" but gives no benchmark numbers against other checkers — [lychee README](https://github.com/lycheeverse/lychee)
- htmltest: MIT, 378 stars, latest release v0.17.0 on 2022-11-04, last push 2025-01-20, 80 open issues — [htmltest repo](https://github.com/wjdp/htmltest)
- linkinator: TypeScript, MIT, 1,269 stars, v8.1.0 released 2026-08-29 — [linkinator repo](https://github.com/JustinBeckwith/linkinator)
- Other maintained checkers: `muffet` (Go, MIT, v2.11.5 on 2026-06-09, 2,614 stars); `markdown-link-check` (JavaScript, ISC, v3.15.0 on 2026-07-28); `html-proofer` (Ruby, MIT, v5.2.2 on 2026-07-28) — [muffet](https://github.com/raviqqe/muffet), [markdown-link-check](https://github.com/tcort/markdown-link-check), [html-proofer](https://github.com/gjtorikian/html-proofer)

### Inferences
- Link checking splits into two jobs with different budgets. Internal links, anchors, includes and cross-references are resolvable from Ascribe's own content model with no I/O and belong in the language server. External URL checking is network-bound, rate-limited and flaky, and belongs in `check --external` or CI with a cache.
- For the external part, **link** `lychee-lib` behind a Cargo feature if the binary already carries tokio and reqwest; otherwise the dependency weight (TLS, HTTP stack, `octocrab`) is significant for a binary shipped through npm. The alternative is a small in-house checker on an HTTP client already present, reusing lychee's ideas (per-host rate limits, cache file, accepted status codes). Shelling out to a user-installed `lychee` is a third, zero-weight option.
- `htmltest` should be treated as unmaintained for planning purposes.

### Gaps
- No quantitative speed comparison between lychee, htmltest and linkinator was found in primary sources. The brief asked for speed claims; lychee publishes none in its README, and I did not retrieve htmltest's or linkinator's READMEs.
- GitHub reports htmltest's primary language as "HTML" (test fixtures); I did not verify its implementation language from source.
- `lychee-lib` feature flags, and whether the GitHub-client dependency can be disabled, were not checked.

## 5. Drift detection for conceptual content

### Takeaway
The only approaches with published evidence are deterministic reference checks: DOCER extracts code-element references from docs with about 20 regular expressions and flags those that existed when the doc was last edited and no longer exist, and Fiberplane's `drift` binds a doc to files or symbols and fails when a tree-sitter AST fingerprint changes. LLM-based drift agents (Promptless, DeepDocs, Mintlify Workflows, `driftcheck`) have no independent accuracy data that I could find.

### Cited Findings

**DOCER (Tan, Wagner, Treude)**
- Paper: "Wait, wasn't that code here before? Detecting Outdated Software Documentation" presents "a GitHub Actions tool that builds on our previous work's approach" to scan for outdated code references when a pull request is submitted — [arXiv 2307.04291](https://arxiv.org/abs/2307.04291)
- Method: extract code element references from README and wiki pages "using a list of regular expressions"; match them against two revisions, "the repository snapshot when the documentation was last updated and the current revision"; flag a reference when it existed in the snapshot but is no longer found — [arXiv 2307.04291 (ar5iv)](https://ar5iv.labs.arxiv.org/html/2307.04291)
- Prevalence: the abstract says over 25% of the 1,000 most-starred GitHub projects had at least one outdated reference; the paper body gives 28.9% — [arXiv 2307.04291](https://arxiv.org/abs/2307.04291), [ar5iv full text](https://ar5iv.labs.arxiv.org/html/2307.04291)
- Stated limitations: the regular expressions "have not been validated on all possible programming languages"; it cannot detect inaccurate descriptions of functionality or undocumented elements; it "only detects code elements written as text"; change logs that mention deleted elements are a false-positive source. The paper shows two true-positive and two false-positive examples but no precision figure, and reports GitHub issues submitted to 15 projects — [ar5iv full text](https://ar5iv.labs.arxiv.org/html/2307.04291)
- Released implementation: `wesleytanws/DOCER_tool` (MIT, 3 stars, last push 2023-07-10) contains `DOCER.yml` (a workflow triggered on `pull_request` that checks out the repo with full history, the wiki and the tool, runs `analysis.sh`, installs pandas and numpy, runs `report.py`, and comments on the PR), a 455-byte `regex_list.txt`, and a `.DOCER_exclude` file. The research repo `wesleytanws/DOCER` is MIT, 4 stars, last push 2023-05-28 — [DOCER_tool repo](https://github.com/wesleytanws/DOCER_tool), [DOCER repo](https://github.com/wesleytanws/DOCER)
- `regex_list.txt` holds 21 patterns, including backtick-quoted spans, `camelCase`, `PascalCase`, `snake_case`, `UPPER_SNAKE`, call syntax `\w+\([^)]*\)`, `$VAR`, `@annotation`, and template syntax `{{...}}` — [regex_list.txt](https://github.com/wesleytanws/DOCER_tool/blob/main/regex_list.txt)
- Independent-of-original-authors precision data comes from a 2026 reuse of DOCER on agent context files: of 50 randomly sampled flagged references, "32 (64%) were genuine referential rot and the remainder false positives or ambiguous", and the authors attribute false positives to DOCER's broad regular expressions matching non-code tokens. Treude is a co-author of both papers, and annotation was by a single author — [arXiv 2606.09090](https://arxiv.org/html/2606.09090)

**Explicit doc-to-code binding**
- Fiberplane `drift`: Zig, MIT, v0.10.1 released 2026-06-22 (also the last push), 148 stars. "Any markdown file in your repo can declare anchors to code — specific files or AST symbols. When bound code changes, `drift check` flags the doc as stale." — [drift README](https://github.com/fiberplane/drift)
- Mechanism: `drift link docs/auth.md src/auth/provider.ts#AuthConfig` writes a binding to `drift.lock` (TOML: `doc`, `target`, `sig`, optional `origin`) with a signature that is "an XxHash3 hash of the file or symbol's normalized AST". Inline references of the form `@./src/auth/provider.ts#AuthConfig` in the doc body are also stamped — [drift README](https://github.com/fiberplane/drift)
- Staleness: "For supported languages (TypeScript, Python, Rust, Go, Zig, Java), comparison is syntax-aware — drift parses with tree-sitter and hashes a normalized AST fingerprint (node kinds + token text, no whitespace or position data). Reformatting won't trigger false positives." Unsupported languages fall back to raw content comparison. "No VCS history is needed" — [drift README](https://github.com/fiberplane/drift)
- Commands: `check` (exit 1 if stale), `status`, `link`, `unlink`, `refs` (reverse lookup: which docs reference a file); `drift check --changed src/auth` scopes CI to affected docs. It ships a coding-agent skill so that "agents that change code must update the docs they affect" — [drift README](https://github.com/fiberplane/drift)
- Rust building block: the `tree-sitter` crate (0.27.0, MIT, updated 2026-08-30, 42.7 million downloads) — [tree-sitter on crates.io](https://crates.io/crates/tree-sitter)
- Swimm: Auto-sync is described by the vendor as "patented"; it "automatically keeps code snippets up to date with routine code changes via your CI", and when a change is significant a verification check can block the pull request and mark the doc "potentially out of date" — [Swimm Auto-sync docs](https://docs.swimm.io/features/keep-docs-updated-with-auto-sync/)
- Swimm's public GitHub organisation holds a verify action, pre-commit hooks and assets, not the sync engine — [swimm-verify-action](https://github.com/swimmio/swimm-verify-action)
- Danger JS (TypeScript, MIT, release 14.0.6 on 2026-08-27; npm 14.0.7 on 2026-08-28; 5,509 stars) is the general-purpose host for PR rules — [danger-js repo](https://github.com/danger/danger-js), [danger on npm](https://www.npmjs.com/package/danger)

**LLM-based drift detection**
- `deichrenner/driftcheck`: Rust, no licence declared in GitHub metadata, v0.1.7 on 2026-02-04 (also the last push), 6 stars. A pre-push hook that "uses LLM to generate targeted ripgrep queries", finds related docs and checks the diff against them; "Conservative by default — only flags clear, factual errors"; requires `rg` and an OpenAI-compatible endpoint — [driftcheck README](https://github.com/deichrenner/driftcheck)
- Several unrelated projects share the name `driftcheck` (toolchain-version drift in Python, `.env` drift in Go, Terraform drift); none has more than a handful of stars — [GitHub search](https://github.com/search?q=driftcheck&type=repositories)
- Promptless, DeepDocs and Mintlify's Workflows agent are described in vendor and listicle pages as watching PRs and opening doc-update PRs for review; one listicle gives Promptless's entry price as $500 per month and says DeepDocs has a free tier of 10 scans per month (secondary, unverified) — [happysupport.ai roundup](https://www.happysupport.ai/en/blog/best-ai-documentation-tools), [Mintlify Learn](https://learn.mintlify.com/courses/structure-docs/keeping-docs-current), [DeepDocs](https://deepdocs.dev/)

**Traction signals**
- Hacker News "Show HN: Drift – Linter for Documentation Rot" (2026-03-26) got 2 points and no comments; similar Show HN posts for doc-drift tools in February to March 2026 (DocSync, VeriContext) got 3 to 4 points — [HN item 47537155](https://news.ycombinator.com/item?id=47537155), [HN item 47021705](https://news.ycombinator.com/item?id=47021705), [HN item 47145928](https://news.ycombinator.com/item?id=47145928)

### Inferences
- Both evidence-backed techniques are cheap to **reimplement** in Rust, and neither can be linked: DOCER is shell plus Python with pandas, and `drift` is a Zig binary.
  - DOCER-style check: extract code-like tokens from prose and inline code, test whether each exists in the repository now and whether it existed at the doc's last-edit commit. This needs a tokenizer, a repository index (file paths plus an identifier set from a ripgrep-style scan or tree-sitter), and one git lookup per doc. The 64% precision measured in the 2026 study is the argument for restricting extraction to inline code spans and Ascribe's typed references, instead of DOCER's broad prose patterns (`[A-Z]{2,}` matches every acronym), and for exempting changelog-type pages.
  - `drift`-style check: let a page or section declare `tracks: path#Symbol` (front matter or a directive), store a normalised-AST hash in a lockfile, and report staleness in `check`. This needs the `tree-sitter` crate plus one grammar per supported language, which adds binary size per language; a raw-content-hash fallback is trivial.
- These two checks are complementary. DOCER catches references that have disappeared (deleted or renamed symbols) with no author effort. Bindings catch code that still exists but has changed, which is the closest any deterministic tool gets to conceptual drift: it cannot say the prose is wrong, only that the code it describes changed after the prose was last confirmed.
- Ascribe's existing typed model gives a third, zero-false-positive variant: anything already expressed as a typed reference (include targets, snippet regions, OpenAPI operations, glossary terms, availability versions) can be verified exactly. The reverse index (`drift refs`: "which docs mention this file") is cheap to expose and is what a "docs touch" CI rule or an agent needs.
- The LLM agents operate on the PR diff and produce drafts; they move the cost to review. A deterministic staleness list is a natural input to them (and to a coding agent's own loop), which is how Fiberplane positions `drift`.
- Low HN traction for standalone drift linters suggests this is better delivered as a feature of a tool people already run than as a separate product. That is an inference from a very small sample.

### Gaps
- No independent evidence of accuracy or noise for Swimm, Promptless, DeepDocs, Mintlify's agent or Dosu was found; everything located is vendor description or listicle. Nothing substantive was found on Dosu's docs-drift features (its GitHub organisation shows a CLI and unrelated tools).
- Swimm's algorithm and patent details were not retrieved; the docs page read gives no mechanism.
- No primary source was fetched for CODEOWNERS-style doc mappings or "docs touch" CI checks, so they are not documented above beyond Danger's metadata.
- DOCER's original paper reports no precision figure; the 64% figure comes from context files, not READMEs, and from one annotator.
- No user reports on false-positive rates or adoption of Fiberplane `drift` were found.

## 6. Agent context files (AGENTS.md, CLAUDE.md, Cursor rules): staleness checks and the Treude and Baltes findings

### Takeaway
Treude and Baltes ran DOCER unmodified over agent context files and found stale code-element references in 23.0% of 356 sampled repositories, with 64% precision on a hand-checked sample; referential checks are the only category they demonstrate, and four others are listed as open research. Existing linters for these files are tiny, mostly zero-star projects that check whether paths and commands still exist.

### Cited Findings
- Paper: "Context Rot in AI-Assisted Software Development: Repurposing Documentation Consistency for AI Configuration Artifacts", Christoph Treude and Sebastian Baltes, submitted 2026-06-08. Context files "can become stale, a phenomenon we call context rot" — [arXiv 2606.09090](https://arxiv.org/abs/2606.09090)
- Method: DOCER applied without modification; candidates extracted with `git grep -howIP -f regex_list.txt`, then presence checked at the file's first commit and at current HEAD — [arXiv 2606.09090 full text](https://arxiv.org/html/2606.09090)
- Sample: 356 repositories randomly sampled (seed 42) from 4,420 eligible public repositories with AI configuration files, representative at 95% confidence with a 5% margin — [arXiv 2606.09090 full text](https://arxiv.org/html/2606.09090)
- Result: 82 of 356 repositories (23.0%, 95% CI 18.8 to 27.2%) had stale references; 230 stale references across 612 configuration files; median 1 per affected repository, maximum 20 — [arXiv 2606.09090 full text](https://arxiv.org/html/2606.09090)
- By file type (files; verified references; stale; stale share): CLAUDE.md 147; 5,423; 77; 1.42%. AGENTS.md 234; 6,762; 70; 1.04%. Copilot instructions 211; 5,436; 77; 1.42%. GEMINI.md 6; 133; 1. `.cursorrules` 9; 127; 0. Total 612; 18,048; 230; 1.27% — [arXiv 2606.09090 full text](https://arxiv.org/html/2606.09090)
- Precision: manual inspection of 50 sampled stale elements found "32 (64%) were genuine referential rot and the remainder false positives or ambiguous" — [arXiv 2606.09090 full text](https://arxiv.org/html/2606.09090)
- Examples: a library named in `copilot-instructions.md` that is absent from source and build config (microsoft/pr-metrics); a renamed function in a `CLAUDE.md` (dagu-org/dagu); file paths that no longer exist in `AGENTS.md` (rolldown/rolldown, EricLBuehler/mistral.rs) — [arXiv 2606.09090 full text](https://arxiv.org/html/2606.09090)
- Transfer map: referential consistency (DOCER) is "Direct; demonstrated here". Code-comment consistency, API-documentation checks (for example MCP tool descriptions diverging from implementation), architecture traceability, and installation/dependency validity are each marked open. The unifying statement: "a claim made in a text artifact about the state of software can be operationalized as a query against the actual software, then flagged if it diverges" — [arXiv 2606.09090 full text](https://arxiv.org/html/2606.09090)
- Limitations stated: a two-snapshot design that misses references added in later edits (deflating the estimate), broad regular expressions (inflating false positives), a single annotator, public GitHub only. Artefacts: Zenodo doi:10.5281/zenodo.20588740 — [arXiv 2606.09090 full text](https://arxiv.org/html/2606.09090)
- Whether context files help at all is contested. Gloaguen et al. found that "providing context files does not generally improve task success rates, while increasing inference cost by over 20% on average", and concluded they are best kept to non-standard practices — [arXiv 2602.11988](https://arxiv.org/abs/2602.11988)
- Other 2026 studies, as summarised in search results that I did not open: Lulla et al. report 28.6% less runtime and 16.6% fewer output tokens with curated AGENTS.md files; a two-agent ablation found no measurable correctness effect — [arXiv 2601.20404](https://arxiv.org/abs/2601.20404), [arXiv 2607.27250](https://arxiv.org/html/2607.27250v1)
- Existing linters: `openintelligence-labs/agents-md-lint` (Python, MIT, 0 stars, pushed 2026-08-07) checks that backticked paths exist (suggesting where a moved path went) and that executables in shell fences resolve; "No LLM call, no network, no embedding pass, zero runtime dependencies". Its README argues "False positives are the failure mode" and lists what it deliberately ignores: URLs, globs, `~/paths`, `<placeholders>`, `$VARS`, system paths, prose in backticks, non-shell code blocks, and the project's own CLI — [agents-md-lint README](https://github.com/openintelligence-labs/agents-md-lint)
- A GitHub search for AGENTS.md and CLAUDE.md linters returns about a dozen projects, all with 0 or 1 stars (for example `rk-chavali/agents-md-lint`, `yuziri-open/claude-md-lint`, `Penloom-Studio/claude-md-lint`, which counts instructions against an adherence budget) — [GitHub search](https://github.com/search?q=agents.md+lint&type=repositories)
- Fiberplane `drift` covers this case too: any Markdown file, including a skill or context file, can be bound to files and symbols, with an `origin` field so that vendored docs are skipped in other repos — [drift README](https://github.com/fiberplane/drift)

### Inferences
- A context file is a Markdown doc with a very high density of path, command and symbol references, so the same DOCER-style and binding-style checks from section 5 apply unchanged. A docs tool that already parses Markdown and indexes the repo could offer `check` on `AGENTS.md`, `CLAUDE.md`, `.cursor/rules` and Copilot instructions as a low-cost feature; no established open-source tool owns this space.
- The checks with the best expected precision, in order: file and directory paths exist; named scripts and package-manager scripts exist (for example `npm run x` against `package.json`, `cargo` aliases, Makefile targets); executables resolve; identifiers in backticks exist somewhere in the repo. The measured stale rate per reference is low (about 1.3%), so even a modest false-positive rate will dominate the output, which supports the conservative extraction rules that `agents-md-lint` describes.
- The "installation/dependency" row of the transfer map (versions in the file against lockfiles and toolchain files) is deterministic and easy to implement even though the paper leaves it open.

### Gaps
- No tool was found that validates Cursor rules specifically (for example glob patterns in `.cursor/rules` matching no files).
- The paper demonstrates only referential checks; there is no evidence yet on which of the other four check types work on context files.
- No mature or widely adopted context-file linter was found; star counts suggest none has traction.
- The Lulla et al. and two-agent-ablation figures were taken from search summaries, not from the papers.

## 7. Agent-readable output: the Web Documentation Delivery Spec, `afdocs`, llms.txt adoption

### Takeaway
The spec at agentdocsspec.com defines 28 checks in 7 categories about how a deployed docs site serves agents, and `afdocs` (TypeScript, MIT, v0.22.2 released 2026-09-28) tests a live URL against it; most checks concern HTTP behaviour of the hosted site, but several concern build output that a generator controls. llms.txt adoption is in the single digits to low tens of percent, and server-log evidence says general AI crawlers rarely fetch it, with coding agents the notable exception.

### Cited Findings
- The Web Documentation Delivery Spec evaluates docs sites for AI agent consumers through 28 checks across 7 categories; licensed CC BY 4.0; grounded in two articles by Dachary Carey dated 2026-02-18 and 2026-02-19, one "validating 578 coding patterns with Claude" and one analysing Claude Code's web fetch pipeline — [agentdocsspec.com](https://agentdocsspec.com)
- Categories and counts: Content Discoverability (7: llms.txt existence, validity, size, link resolution, links pointing to Markdown, agent-directed pointers), Markdown Availability (2: `.md` URL support and content negotiation via `Accept`), Page Size (6: SPA detection, Markdown and HTML size, content position, single-fetch completeness), Content Structure (5: tabbed-content serialisation, header quality, code-fence validity, link portability), URL Stability (2: soft 404s, redirects), Observability (3: coverage, Markdown/HTML content parity, cache headers), Authentication (3: auth gates, alternative access, bot-protection interference) — [agentdocsspec.com](https://agentdocsspec.com)
- `afdocs`: TypeScript, MIT, 98 stars, v0.22.2 released 2026-09-28; "Status: Early development (0.x). Check IDs, CLI flags, and output formats may change between minor versions"; implements spec v0.6.0 (2026-09-13); requires Node.js 22 or later — [afdocs README](https://github.com/agent-ecosystem/afdocs)
- Consumption: `npx afdocs check https://docs.example.com --format scorecard`; a programmatic TypeScript API; vitest helpers for CI. It "makes HTTP requests to the sites it checks" with a 200 ms default delay. It powers Fern's "Agent Score" — [afdocs README](https://github.com/agent-ecosystem/afdocs)
- Check IDs visible in the docs include `llms-txt-exists`, `llms-txt-valid`, `llms-txt-size`, `llms-txt-links-resolve`, `llms-txt-links-markdown`, `llms-txt-coverage`, `llms-txt-directive-html`, `markdown-url-support`, `content-negotiation`, `page-size-markdown`, `page-size-html`, `rendering-strategy`, `tabbed-content-serialization`, `section-header-quality`, `markdown-code-fence-validity`, `markdown-content-parity`, `single-fetch-completeness`, `markdown-link-portability`, `auth-gate-detection`, `auth-alternative-access`, `bot-protection-interference` — [afdocs checks reference](https://afdocs.dev/checks/), [afdocs README](https://github.com/agent-ecosystem/afdocs)
- Example findings from the README scorecard: a warning that llms.txt is 65,000 characters with advice to split into nested files, and a failure for no agent directive in page HTML, with the fix "Add a visually-hidden element near the top of each page" — [afdocs README](https://github.com/agent-ecosystem/afdocs)
- llms.txt reference tooling: `AnswerDotAI/llms-txt` (Apache-2.0, 2,647 stars, release 0.0.7 on 2026-09-24) — [llms-txt repo](https://github.com/AnswerDotAI/llms-txt)
- llms.txt adoption, as collated by an independent aggregator (original studies not opened): 5.61% of the top 10,000 sites (HTTP Archive analysis, 2026-06-20); 10.13% of about 300,000 domains (SE Ranking, 2025-11-20); 28% of 137,210 domains (Ahrefs, 2026-06-15, a sample skewed to SEO-tool customers) — [agentexperience.tech](https://agentexperience.tech/insights/state-of-llms-txt/)
- Usage, same aggregator citing Ahrefs server logs for May 2026: "97% of llms.txt files received zero requests"; of requests seen, 96% were bots and roughly 77% of those were not AI tools; named AI user agents were GPTBot 4.51% and ClaudeBot 0.80%. It also reports that Claude Code "out-fetched the AI retrieval bots", and that Google Search ignores the file — [agentexperience.tech](https://agentexperience.tech/insights/state-of-llms-txt/)
- A separate page reports 8.7% of the top 1,000 sites as of June 2026 (search snippet, not opened) — [rankability.com](https://www.rankability.com/data/llms-txt-adoption/)

### Inferences
- `afdocs` cannot be linked (Node 22, HTTP-driven, unstable 0.x interface); the realistic integrations are **shell out** in CI against a preview deployment, or **reimplement** the build-time subset natively.
- Checks a static generator controls at build time, and could assert in `build` or `check` with no network: emit `llms.txt` and keep it valid, within the size limit, complete (coverage) and pointing at Markdown; emit a Markdown twin for every page; keep Markdown and HTML content in parity; serialise tabs and variants linearly in the Markdown output; keep code fences balanced; use portable (absolute or resolvable) links; keep each page under the size thresholds; insert the agent directive into page HTML. Checks that depend on hosting (content negotiation, redirects, soft 404s, cache headers, auth, bot protection) can only be verified against a live URL.
- Ascribe's variants and availability features map directly to the spec's tabbed-content and page-size concerns: a typed model can serialise each variant deterministically in the Markdown output, which generic generators find hard.
- The llms.txt data argue for generating it because it is nearly free and coding agents do read it, not for treating it as a search-visibility feature.

### Gaps
- The full list of 28 check IDs and their thresholds (for example the character limits) was not retrieved; 21 IDs are listed above from the pages read.
- Whether `afdocs` can run against a local build directory instead of a URL was not confirmed.
- The adoption and traffic statistics come through an aggregator and SEO-industry studies; I did not open the Ahrefs, SE Ranking or HTTP Archive originals. One widely repeated figure ("408 requests in 500 million bot visits") is attributed to a vendor's internal monitoring without published methodology and should not be relied on.
- The evidence base of the spec itself is one author's experiments centred on Claude; I found no independent replication across agents.

## 8. Verification aids for AI-written changes: checking claims against code or tests, and whether deterministic linting reduces review load

### Takeaway
Tools exist to execute what docs claim (Doc Detective for UI, shell and HTTP steps; Hurl for HTTP assertions; Runme for fenced shell blocks; `trycmd` and `term-transcript` for CLI transcripts in Rust), but I found no measured evidence that a deterministic linter reduces reviewer load for AI-written docs. The 2026 State of Docs survey confirms the problem ("Verification is still really hard") without quantifying any remedy.

### Cited Findings
- State of Docs 2026 (GitBook, published 2026-08-28): 62% use AI for drafting documentation; "Only 25% use AI to write the whole thing"; 78% say AI makes documentation faster and 35% claim time savings of 50% or more, with technical writers reporting smaller gains than other roles; 62% cite hallucinations as a concern. Quoted practitioners: "Verification is still really hard"; one "combines a linter, Vale, with a large language model to enforce our style guidelines" with a human in the loop — [State of Docs 2026](https://www.stateofdocs.com/2026/ai-and-documentation-creation)
- Doc Detective "performs your instructions step-by-step, just like your users would, and reports what works and what doesn't"; results are emitted as JSON (PASS/FAIL plus context). AGPL-3.0, Node, heavy browser runtime — [Doc Detective docs](https://docs.doc-detective.com/), [Doc Detective README](https://github.com/doc-detective/doc-detective)
- Hurl: Rust, Apache-2.0, 8.0.1 released 2026-04-29, 19,235 stars; "powered by libcurl"; building needs libssl, libcurl and libxml2 development files. Crates `hurl` and `hurl_core` (both 8.0.1) are published — [Hurl README](https://github.com/Orange-OpenSource/hurl), [hurl_core on crates.io](https://crates.io/crates/hurl_core)
- Step CI: TypeScript, MPL-2.0, latest release 2.8.2 on 2024-06-10, last push 2024-08-03 — [stepci repo](https://github.com/stepci/stepci)
- Runme executes fenced code blocks with environment retained across blocks, in a CLI and notebook interface (Go, Apache-2.0) — [Runme README](https://github.com/runmedev/runme)
- `trycmd` and `term-transcript` snapshot-test CLI invocations and their output from Rust — [trycmd](https://crates.io/crates/trycmd), [term-transcript](https://crates.io/crates/term-transcript)
- Fiberplane frames deterministic drift checks as an agent guardrail: with `drift check` in CI, "stale docs block merges — so the agent can't silently break documentation" (vendor claim, no measurements) — [drift README](https://github.com/fiberplane/drift)
- The agents-md-lint author's design argument for determinism: "A staleness check that costs an API call will not be run every session, and a check nobody runs is worse than none" (opinion, no measurements) — [agents-md-lint README](https://github.com/openintelligence-labs/agents-md-lint)
- A search snippet attributes to Fern's "Docs Linting Guide" the claim that AI-generated content must pass the same Vale checks as human-written docs and that an agent can read the linter log and commit corrections; the page could not be read, so this is unverified vendor content — [Fern docs linting guide](https://buildwithfern.com/post/docs-linting-guide)

### Inferences
- The claims in a docs change that can be verified mechanically are those that can be turned into a query against the product, in Treude and Baltes's phrasing: "this path/symbol/flag/endpoint exists" (index lookup), "this sample is the code in the repo" (include by region), "this command prints this" (transcript test), "this request returns this" (HTTP assertion), "this is true from version X" (availability metadata against a version source). Everything else (explanations, rationale, ordering of concepts) still needs a human.
- For a reviewer of AI-written docs, the practical value of a deterministic layer is triage: a diff where every reference resolves, every sample is included from tested source and every CLI flag exists in the product's own `--help` or schema leaves only the conceptual claims to read. That is a reasoned expectation; no source measured it.
- Integration shapes: existence and reference checks are in-process and fast enough for the language server. Execution-based checks are `check`-time or CI-only and are best done by **shelling out** (Hurl's native dependencies on libcurl and libxml2 work against a single static binary; Doc Detective is AGPL and Node), or by a small native runner for shell blocks with expected-output comparison in the spirit of `trycmd`.
- A cheap, high-value native check for a CLI-documenting project is "every flag and subcommand mentioned in a code span or shell block exists", driven from a machine-readable command description exported by the product (clap can emit one). This is a suggestion, not something a cited source describes.
- Step CI appears unmaintained (no release since June 2024) and should not be a planned dependency.

### Gaps
- No study, survey or practitioner write-up was found that measures review time or defect escape with and without deterministic doc checks for AI-authored changes. The claim that linters reduce review load is, as far as I could find, asserted (mostly by vendors) and not measured.
- The State of Docs page read did not give a respondent count or a statistic for time spent reviewing AI output; the "reviewing became heavier" framing in search results comes from a blog post that I did not open.
- I did not find an open-source tool that extracts natural-language claims from docs and checks them against code or tests deterministically; the tools found either check references (section 5) or execute explicit steps (this section). LLM-based claim checkers were not surveyed beyond `driftcheck`.
- Write the Docs newsletter and community threads were not searched.
