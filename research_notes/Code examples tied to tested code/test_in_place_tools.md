# Tools that test or execute code examples written inside documentation ("test the docs in place"), as of October 2026

Research date: 2026-10-09. Registry and repository figures (versions, dates, stars, downloads) were read from the PyPI, crates.io, npm, pypistats and GitHub APIs on that date; the citation is the human-readable page for the same record. "Not confirmed" means I did not see it at a primary source in this session.

## Catalog: what each approach does, how an example is marked, and what "passing" means

### Takeaway
The family splits into four mechanisms: (1) compile-and-run with assertions written in the example (Rust, Deno, mktestdocs, pytest-markdown-docs), (2) transcript comparison, where a prompt line is followed by expected output (Python doctest, Elixir, Haskell, Julia REPL doctests, cram, OCaml mdx, byexample), (3) evaluate-and-render, where the tool writes the real output into the page so there is nothing to compare (Scala mdoc, markdown-exec, knitr, Quarto, MyST-NB, Org-babel), and (4) type-check only (twoslash, typescript-docs-verifier, `deno check --doc`, Go examples without `// Output:`, mdoc `compile-only`). Markers are almost always fence info-string words or HTML comments placed before the fence; hidden boilerplate is a line prefix (`# ` in Rust and mdBook), a cut marker (twoslash), or an invisible block (Sphinx, Knit, mdoc).

### Cited Findings

#### Language-native doctests

**Rust doctests (rustdoc)**
- Passing: "regular doctests are considered to 'pass' if they compile and run without panicking"; results are checked with `assert!` macros, not by output comparison — [rustdoc book, Documentation tests](https://doc.rust-lang.org/rustdoc/write-documentation/documentation-tests.html)
- Fence attributes: `ignore` (skip; the docs call it almost never the right choice and suggest `text` or hidden lines instead), `should_panic` (must compile and panic), `no_run` (compile only, for network code or code that could cause undefined behavior), `compile_fail` (compilation must fail, else the test fails), `edition2015` through `edition2024`, `standalone_crate` (do not merge with other doctests), `ignore-<target>` (per-target skip; from 1.88.0 `ignore-x86_64` overrides a plain `ignore`), `test_harness` — [rustdoc book](https://doc.rust-lang.org/rustdoc/write-documentation/documentation-tests.html)
- Hidden boilerplate: lines starting with `# ` are compiled but not displayed; `##` escapes a literal `#` — [rustdoc book](https://doc.rust-lang.org/rustdoc/write-documentation/documentation-tests.html)
- Implicit boilerplate: rustdoc inserts `allow` attributes for unused variables, dead code, and similar; inserts `extern crate <mycrate>;` unless `#![doc(test(no_crate_inject))]`; and wraps the text in `fn main() { ... }` if there is no `fn main`. To use `?`, the example needs a hidden `main` returning `Result`, or (since 1.34.0) a hidden trailing `# Ok::<(), io::Error>(())` — [rustdoc book](https://doc.rust-lang.org/rustdoc/write-documentation/documentation-tests.html)
- Markdown files outside doc comments: a README is tested by `#[doc = include_str!("../README.md")] #[cfg(doctest)] pub struct ReadmeDoctests;` — [rustdoc book](https://doc.rust-lang.org/rustdoc/write-documentation/documentation-tests.html)
- Speed: before the 2024 edition each doctest was compiled as its own executable; from the 2024 edition "compatible doctests are merged as one before being run", still each run in its own process — [rustdoc book](https://doc.rust-lang.org/rustdoc/write-documentation/documentation-tests.html); [Edition Guide, Rustdoc combined tests](https://doc.rust-lang.org/edition-guide/rust-2024/rustdoc-doctests.html)

**Python `doctest`**
- Marking: `>>> ` starts an example and `... ` continues it; expected output follows immediately and runs to the next `>>> ` or blank line; a blank output line is written `<BLANKLINE>`; stdout is captured, stderr is not — [Python docs, doctest](https://docs.python.org/3/library/doctest.html)
- Passing: actual output text must match expected output text; flags loosen matching: `ELLIPSIS` (`...` matches any substring), `NORMALIZE_WHITESPACE`, `IGNORE_EXCEPTION_DETAIL`, `SKIP` ("do not run the example at all"), `DONT_ACCEPT_TRUE_FOR_1`; per-example directives are written as `# doctest: +ELLIPSIS` — [Python docs, doctest](https://docs.python.org/3/library/doctest.html)
- Text files: `doctest.testfile("example.txt")` treats a file "as if it were a single giant docstring" — [Python docs, doctest](https://docs.python.org/3/library/doctest.html)
- Setup: `DocTestSuite` and `DocFileSuite` take `setUp`, `tearDown`, and `globs`; these live in the test loader, not in the document — [Python docs, doctest](https://docs.python.org/3/library/doctest.html)
- Sphinx's `sphinx.ext.doctest` adds `testsetup`, `testcode`, and `testoutput` directives with a `:hide:` option (run but not rendered), `doctest_global_setup` in `conf.py`, and a `skipif` option — [Sphinx docs, sphinx.ext.doctest](https://www.sphinx-doc.org/en/master/usage/extensions/doctest.html)

**Go testable examples**
- Marking: functions named `Example`, `ExampleF`, `ExampleT`, `ExampleT_M` (plus `_suffix` for more than one) in `_test.go` files; godoc attaches them to the identifier by name — [Go blog, Testable Examples in Go (7 May 2015)](https://go.dev/blog/examples); [pkg.go.dev/testing](https://pkg.go.dev/testing)
- Passing: with a trailing `// Output:` comment, stdout is captured and compared (leading and trailing whitespace ignored); `// Unordered output:` accepts the lines in any order; with no output comment the example is compiled but not run — [pkg.go.dev/testing](https://pkg.go.dev/testing)
- The compile-only form is recommended for "code that cannot run as unit tests", such as code that uses the network — [Go blog](https://go.dev/blog/examples)
- Extra context: a whole-file example (one example function, at least one other declaration, no tests or benchmarks) shows the entire file — [Go blog](https://go.dev/blog/examples)
- Note: Go examples live in test files and are pulled into the docs by name, so they sit on the border with the extraction family.

**Elixir `ExUnit.DocTest`**
- Marking: `iex>` prompt, `...>` continuation, expected value on the next line; an empty line separates tests; exceptions are written as `** (Name) message`, and since Elixir 1.19.0 a trailing `...` is allowed in the message — [ExUnit.DocTest](https://ex-unit.hexdocs.pm/ExUnit.DocTest.html)
- The result may be omitted to skip the assertion; values that print as `#Name<...>` are compared as strings — [ExUnit.DocTest](https://ex-unit.hexdocs.pm/ExUnit.DocTest.html)
- Markdown files: `doctest_file "README.md"`, added in 1.15.0 — [ExUnit.DocTest](https://ex-unit.hexdocs.pm/ExUnit.DocTest.html)
- Options `:only`, `:except`, `:tags`, `:import` are given in the test module, not the doc — [ExUnit.DocTest](https://ex-unit.hexdocs.pm/ExUnit.DocTest.html)

**Julia Documenter.jl**
- Marking: a `jldoctest` fence; a script doctest has a line with exactly `# output` and the expected output after it; a block with `julia>` prompts is a REPL doctest — [Documenter.jl, Doctests](https://documenter.juliadocs.org/stable/man/doctests/)
- Shared state: blocks with the same label in the same file are "evaluated in the same module, and hence share scope" — [Documenter.jl](https://documenter.juliadocs.org/stable/man/doctests/)
- Setup: `DocTestSetup` and `DocTestTeardown` in `@meta` blocks, or `jldoctest; setup = :(...)`; they are re-evaluated for each doctest block — [Documenter.jl](https://documenter.juliadocs.org/stable/man/doctests/)
- Output matching: `DocTestFilters` are regex and substitution pairs applied to both expected and actual output; exceptions match by prefix, with `[...]` marking where checking stops — [Documenter.jl](https://documenter.juliadocs.org/stable/man/doctests/)
- Status: v1.19.0 released 2026-09-02, MIT, 919 stars — [JuliaDocs/Documenter.jl](https://github.com/JuliaDocs/Documenter.jl)

**Haskell `doctest`**
- Marking: `>>>` in Haddock comments, expected output on following lines; `:{` and `:}` for multi-line; `<BLANKLINE>`; a line of `...` matches any lines; `prop>` checks a QuickCheck property; a `$setup` chunk "is run before each example group" — [sol/doctest README](https://github.com/sol/doctest)
- It runs through GHCi and compares what the REPL prints; a failure skips the rest of the group; `--fast` skips the `:reload` between groups — [sol/doctest README](https://github.com/sol/doctest)
- Limits: works only where GHC's interactive mode is supported; `cabal doctest` is marked experimental and needs cabal-install 3.12 or later — [sol/doctest README](https://github.com/sol/doctest)
- Status: 400 stars, last push 2026-09-26, MIT — [sol/doctest](https://github.com/sol/doctest)

**Deno `deno test --doc`**
- It "can evaluate the code snippets written in your JSDoc comments and markdown files and run them as tests"; each block becomes a `Deno.test` case in a standalone module placed beside the documented file — [Deno docs, doc tests](https://docs.deno.com/runtime/test/doc_tests.md)
- Fences recognized: `js`, `javascript`, `mjs`, `cjs`, `jsx`, `ts`, `typescript`, `mts`, `cts`, `tsx`; `ignore` after the language skips a block — [Deno docs, documentation tests](https://docs.deno.com/runtime/reference/documentation/)
- Hidden boilerplate is replaced by auto-import: "any items exported from the module are automatically included in the generated test code using the same name" — [Deno docs, doc tests](https://docs.deno.com/runtime/test/doc_tests.md)
- Type-check only: `deno check --doc` (JSDoc) or `--doc-only` (Markdown) — [Deno docs, doc tests](https://docs.deno.com/runtime/test/doc_tests.md)
- Permissions: a snippet's hashbang can carry permission flags, and a snippet never gets broader permissions than `deno test` was granted — [Deno docs, doc tests](https://docs.deno.com/runtime/test/doc_tests.md)

**Java `{@snippet}` (JEP 413, JDK 18)**
- Inline, external (`class=` or `file=` with `region=`), and hybrid snippets; markup comments `@highlight`, `@replace`, `@link`, `@start`/`@end` — [Oracle, Programmer's Guide to Snippets (JDK 21)](https://docs.oracle.com/en/java/javase/21/javadoc/snippets.html)
- "The Standard Doclet does not compile or otherwise test snippets; instead, it supports the ability of external tools and library code to test them." The one check: for a hybrid snippet it verifies that the inline form and the external form are the same — [Oracle snippets guide](https://docs.oracle.com/en/java/javase/21/javadoc/snippets.html)
- The guide recommends external snippets for testing because they compile and run with ordinary tools; inline snippets must be found with the Compiler Tree API and wrapped in a compilation unit — [Oracle snippets guide](https://docs.oracle.com/en/java/javase/21/javadoc/snippets.html)

**Swift snippets (SE-0356, DocC)**
- Status "Implemented (Swift 5.7)"; snippets are `.swift` files in a top-level `Snippets` directory, each built as an executable target; `swift build --build-snippets` builds them and `swift run Snippet1` runs one — [SE-0356](https://github.com/swiftlang/swift-evolution/blob/main/proposals/0356-swift-snippets.md)
- `// snippet.hide` and `// snippet.show` hide setup lines; `// snippet.IDENTIFIER` and `// snippet.end` name slices — [SE-0356](https://github.com/swiftlang/swift-evolution/blob/main/proposals/0356-swift-snippets.md)
- This is an extraction design, not in-place: the proposal rejected snippets inside doc comments and rejected a literate (code in Markdown) design as "not a small undertaking" — [SE-0356](https://github.com/swiftlang/swift-evolution/blob/main/proposals/0356-swift-snippets.md)
- Status: swift-docc-plugin 1.5.0, 2026-04-27, Apache-2.0 — [swiftlang/swift-docc-plugin](https://github.com/swiftlang/swift-docc-plugin)

#### Markdown testing tools

**mdBook `mdbook test`**
- Runs Rust blocks (and blocks with no language) through rustdoc; other languages are skipped; `--library-path` points rustdoc at a Cargo `deps` directory so examples can use a crate; `--chapter` tests one chapter — [mdBook, test command](https://rust-lang.github.io/mdBook/cli/test.html)
- Hidden lines use the rustdoc `# ` prefix; other languages can set a prefix with `[output.html.code.hidelines]`, but that affects display only — [mdBook, mdBook-specific features](https://rust-lang.github.io/mdBook/format/mdbook.html)
- `{{#rustdoc_include file.rs:anchor}}` inserts a whole file with the unselected lines prefixed by `#`, so the reader sees a fragment and `mdbook test` compiles the full program — [mdBook, mdBook-specific features](https://rust-lang.github.io/mdBook/format/mdbook.html)
- Status: v0.5.4, 2026-07-06, MPL-2.0, about 22,200 stars — [rust-lang/mdBook](https://github.com/rust-lang/mdBook)

**Sybil (Python)**
- Checks examples "by parsing them from their source and evaluating the parsed examples as part of your normal test run"; parsers for ReST, Markdown, and MyST covering doctest, code blocks (Python and other languages), capture, skip, and clear-namespace; integrates with pytest and unittest — [Sybil docs](https://sybil.readthedocs.io/en/latest/)
- Status: 10.1.0 released 2026-06-13, MIT; about 318,000 downloads in the last month — [PyPI sybil](https://pypi.org/project/sybil/); [pypistats sybil](https://pypistats.org/packages/sybil)

**pytest-markdown-docs (Modal)**
- Tests fences starting `python`, `python3`, or `py` in `.md`, `.mdx`, `.svx`, and Python docstrings; info-string words: `notest` (skip), `continuation` (keep state from the previous block), `fixture:<name>` (inject a pytest fixture as a global), `retry:N`; in MDX the same options go in a `{/* pmd-metadata: ... */}` comment; `pytest_markdown_docs_globals` in `conftest.py` supplies names to every snippet — [modal-labs/pytest-markdown-docs](https://github.com/modal-labs/pytest-markdown-docs)
- Passing is "runs without raising"; there is no output comparison in the README — [modal-labs/pytest-markdown-docs](https://github.com/modal-labs/pytest-markdown-docs)
- Status: 0.9.2 released 2026-03-23, MIT, 85 stars; about 139,000 downloads in the last month — [PyPI pytest-markdown-docs](https://pypi.org/project/pytest-markdown-docs/); [pypistats](https://pypistats.org/packages/pytest-markdown-docs)

**pytest-codeblocks**
- Picks up `python` and `sh`/`bash`/`zsh` fences; markers are HTML comments before the fence: `<!--pytest.mark.skip-->`, `<!--pytest.mark.skipif(...)-->`, `<!--pytest.mark.xfail-->`, `<!--pytest-codeblocks:cont-->` (merge with previous block), `<!--pytest-codeblocks:expected-output-->` (the next block is the expected output), `<!--pytest-codeblocks:importorskip(pkg)-->`, `<!--pytest-codeblocks:skipfile-->` — [nschloe/pytest-codeblocks](https://github.com/nschloe/pytest-codeblocks)
- Status: 0.18.0 released 2026-06-15, MIT, 122 stars; about 115,000 downloads in the last month — [PyPI pytest-codeblocks](https://pypi.org/project/pytest-codeblocks/); [pypistats](https://pypistats.org/packages/pytest-codeblocks)

**mktestdocs**
- `check_md_file(fpath)` runs each Python block and fails on any error; blocks are independent unless `memory=True`; `check_docstring` does the same for docstrings; `lang="bash"` runs shell blocks; `register_executor` adds another language — [koaning/mktestdocs](https://github.com/koaning/mktestdocs)
- Status: 0.2.5 released 2025-07-25, Apache-2.0, 161 stars; about 35,500 downloads in the last month — [PyPI mktestdocs](https://pypi.org/project/mktestdocs/); [pypistats](https://pypistats.org/packages/mktestdocs)

**pytest-examples (Pydantic)** (added)
- Tests Python examples in docstrings and Markdown; can "lint code examples using `ruff` and `black`", run them, and check `print` output written as `#> 3` comments; `--update-examples` rewrites the formatting and the `#>` comments in place — [pydantic/pytest-examples](https://github.com/pydantic/pytest-examples)
- Status: v0.0.18 released 2025-05-06, MIT, 180 stars, repository pushed 2026-10-08; about 516,000 downloads in the last month — [PyPI pytest-examples](https://pypi.org/project/pytest-examples/); [pypistats](https://pypistats.org/packages/pytest-examples)

**xdoctest** (added): 1.3.2 released 2026-03-27, Apache-2.0, about 1.04 million downloads in the last month — [PyPI xdoctest](https://pypi.org/project/xdoctest/); [pypistats](https://pypistats.org/packages/xdoctest)

**Hugging Face doc-builder runnable blocks** (added, 2026)
- Fence `py runnable:<name>`; `py runnable:<name>:2` continues an earlier block and shares its context; `# doc-builder: hide` keeps a line executable but not displayed; `# pytest-decorator: <path>` attaches a decorator such as a slow marker and is stripped from the rendered page; pytest discovers the blocks in Markdown and reports ordinary tracebacks — [Hugging Face, From doctest to runnable Markdown (4 Apr 2026)](https://huggingface.co/blog/huggingface/runnable-examples)

**phmdoctest**: generates a pytest file from Python blocks in Markdown. Last release 1.4.0 on 2022-03-19; last push 2022-06-11; 22 stars — [PyPI phmdoctest](https://pypi.org/project/phmdoctest/); [tmarktaylor/phmdoctest](https://github.com/tmarktaylor/phmdoctest)

**markdown-doctest (npm)**: last npm release 1.1.0 on 2020-10-07; last push 2020-10-07; 170 stars; about 10,000 downloads in the last month — [npm markdown-doctest](https://www.npmjs.com/package/markdown-doctest); [Widdershin/markdown-doctest](https://github.com/Widdershin/markdown-doctest)

**Rust `skeptic`**
- `build.rs` calls `skeptic::generate_doc_tests(&["README.md"])` and a test file includes the generated code; fences must be tagged `rust`; `ignore`, `no_run`, `should_panic`; a block tagged `skeptic-template` is a format-string template wrapped around every untagged example, the tool's form of hidden boilerplate — [budziq/rust-skeptic](https://github.com/budziq/rust-skeptic)
- Its README points to `doc-comment` for simple cases — [budziq/rust-skeptic](https://github.com/budziq/rust-skeptic)
- Status: crate 0.13.7, last published 2022-02-01; repository last pushed 2024-03-25; about 1.9 million downloads in the last 90 days (mostly existing dependents) — [crates.io skeptic](https://crates.io/crates/skeptic)

**Rust `doc-comment`**: 0.3.4, updated 2025-10-24; about 17 million downloads in the last 90 days — [crates.io doc-comment](https://crates.io/crates/doc-comment)

**OCaml `mdx`**
- Shell blocks: "Lines beginning with a dollar sign and a space are commands and will be run in the shell"; other lines are expected output; `[N]` lines assert an exit code. OCaml toplevel blocks: phrases start with `#` and end with `;;` — [realworldocaml/mdx](https://github.com/realworldocaml/mdx)
- Labels in `<!-- $MDX ... -->` comments: `skip`, `version=`, `non-deterministic[=output|command]`, `dir=`, `file=` and `part=` (sync a block with a region of a real source file), `env=` (separate environments), `set-VAR=`, `unset-VAR` — [realworldocaml/mdx](https://github.com/realworldocaml/mdx)
- Output upkeep: on a mismatch `<file.md>.corrected` is written and `dune promote` accepts it — [realworldocaml/mdx](https://github.com/realworldocaml/mdx)
- Status: 2.7.0 released 2026-10-02, ISC, 291 stars — [realworldocaml/mdx](https://github.com/realworldocaml/mdx)

**Scala `mdoc` (successor to `tut`)**
- A `scala mdoc` fence is compiled and run, and results are rendered as `// res0: Int = 3` comments. Modifiers: `silent`, `fail` (must not compile; "A `fail` code fence with no compile error fails the build"), `warn`, `crash` (must throw at run time), `invisible` (runs, renders nothing), `passthrough` (stdout embedded as Markdown), `compile-only`, `reset`, `reset-object`, `nest`, `to-string`, `width=`, `height=` — [mdoc, Modifiers](https://scalameta.org/mdoc/docs/modifiers.html)
- Compared with tut: tut "interprets statements as if they were typed in a REPL session"; mdoc compiles fences as ordinary Scala programs, so examples can be pasted into real code; errors are reported at the Markdown file's own line and column (for example `why.md:2:22`); `@VERSION@` variables are injected and an undefined one is an error — [mdoc, Why mdoc](https://scalameta.org/mdoc/docs/why.html)
- Speed claim: a medium document goes from "~5 seconds with a cold compiler down to 500ms with a hot compiler" — [mdoc, Why mdoc](https://scalameta.org/mdoc/docs/why.html)
- Status: mdoc v2.9.2 released 2026-09-03, Apache-2.0, 402 stars — [scalameta/mdoc](https://github.com/scalameta/mdoc). tut was archived on 2021-04-13; its README says "This project is archived and unmaintained" and "Please switch to mdoc" — [tpolecat/tut](https://github.com/tpolecat/tut)

**Kotlin `kotlinx-knit`**
- A Gradle plugin that reads Markdown and writes example files and test files that are committed; `knitCheck` "checks that all the files are up-to-date and fail the build if not". Directives are HTML comments: `KNIT` (emit an example file), `INCLUDE` (hidden code such as imports), `PREFIX`, `SUFFIX`, `TEST_NAME`, `TEST` (follows a `text` block holding expected output; can carry a predicate such as `lines.single().toInt() in 1..100`; `LINES_START` checks only the start of the output) — [Kotlin/kotlinx-knit](https://github.com/Kotlin/kotlinx-knit)
- In kotlinx.coroutines docs, `//sampleStart` and `//sampleEnd` bound the visible part of a full runnable program, with `{kotlin-runnable="true"}` after the fence — [kotlinx.coroutines, coroutines-basics.md](https://github.com/Kotlin/kotlinx.coroutines/blob/master/docs/topics/coroutines-basics.md)
- Status: 0.5.1 released 2026-01-23, 321 stars — [Kotlin/kotlinx-knit](https://github.com/Kotlin/kotlinx-knit)

**TypeScript `twoslash`**
- "A markup format for TypeScript code ... which let the TypeScript compiler do the extra leg-work"; used on the TypeScript website — [twoslash guide](https://twoslash.netlify.app/guide/)
- Notations: `// ^?` (show the type), `// ---cut---` / `---cut-before---`, `---cut-after---`, `---cut-start---`/`---cut-end---` (hide code that is still compiled), `// @filename: x.ts` (several files in one block), `// @noErrors`, `// @errors: 2322` (expected error codes), compiler options as `// @strict: false`, `// @showEmit` — [twoslash notations](https://twoslash.netlify.app/refs/notations)
- Status: `twoslash` 0.3.9 released 2026-06-22, MIT, about 3.9 million downloads in the last month; the older `@typescript/twoslash` 3.2.13 about 70,000; `shiki-twoslash` last released 2023-04-28 — [npm twoslash](https://www.npmjs.com/package/twoslash); [npm @typescript/twoslash](https://www.npmjs.com/package/@typescript/twoslash); [npm shiki-twoslash](https://www.npmjs.com/package/shiki-twoslash)

**typescript-docs-verifier (BBC)**
- "Each code snippet is compiled (but not run) and any compilation errors are reported"; imports of the project's own package name are replaced with its `main` or `exports`; "Code snippets must compile independently from any other code snippets in the file"; `<!-- ts-docs-verifier:ignore -->` skips a block — [bbc/typescript-docs-verifier](https://github.com/bbc/typescript-docs-verifier)
- Status: 3.0.2 released 2026-03-02, Apache-2.0, about 32,000 downloads in the last month — [npm typescript-docs-verifier](https://www.npmjs.com/package/typescript-docs-verifier)

**TSDoc / TypeDoc `@example`**
- TypeDoc renders `@example` and does not check it; with no fenced block it treats the whole tag as code — [TypeDoc, @example](https://typedoc.org/documents/Tags._example.html)
- `docs-ts` type-checks and runs examples with ts-node — [npm @docs-ts/docs-ts](https://npmjs.com/package/@docs-ts/docs-ts) (seen in a search summary, not opened)
- `tsdoc-testify` last released 2019-12-06 with 99 downloads in the last month; `vite-plugin-doctest` 3.0.0 released 2026-07-17 with about 3,600 — [npm tsdoc-testify](https://www.npmjs.com/package/tsdoc-testify); [npm vite-plugin-doctest](https://www.npmjs.com/package/vite-plugin-doctest)

**Shell sessions**
- `cram`: `.t` files; lines starting with two spaces, `$`, and a space are run; two spaces and `>` continue; other indented lines are expected output; suffixes `(re)`, `(glob)`, `(no-eol)`, `(esc)`; `-i` prompts to merge actual output back into the test — [cram](https://bitheap.org/cram/). PyPI release 0.7 is dated 2016-02-24; the repository was pushed 2026-08-01 — [PyPI cram](https://pypi.org/project/cram/); [aiiie/cram](https://github.com/aiiie/cram). The fork `prysk` last released 0.20.0 on 2024-05-07 — [PyPI prysk](https://pypi.org/project/prysk/)
- `mdsh` (Rust, zimbatm): a Markdown pre-processor that rewrites output blocks in place; `` `$ command` `` runs a command, `<` includes a file, `>` emits raw Markdown; `--frozen` will "Fail if the output is different from the input. Useful for CI"; it finds blocks with regular expressions and cannot handle output that contains triple backticks — [zimbatm/mdsh](https://github.com/zimbatm/mdsh). 174 stars, pushed 2026-07-23. (The unrelated npm package `mdsh` dates from 2014 — [npm mdsh](https://www.npmjs.com/package/mdsh).)
- `markdown-exec` (MkDocs/Python-Markdown): `exec="true"` on a fence runs it at build time and inserts the result; languages include bash, console, md, py, pycon, pyodide, sh, tree; `returncode="2"` declares an expected non-zero exit — [markdown-exec docs](https://pawamoy.github.io/markdown-exec/). 1.12.4 released 2026-10-06, ISC, about 979,000 downloads in the last month — [PyPI markdown-exec](https://pypi.org/project/markdown-exec/); [pypistats](https://pypistats.org/packages/markdown-exec)
- `runme`: makes Markdown fences runnable as notebook cells or from a CLI; per-cell attributes in the info string: `name`, `interactive`, `background`, `cwd`, `excludeFromRunAll`, `interpreter`, `ignore`, `promptEnv` (prompt for exported environment variables), `skipPrompts` — [Runme, cell-level configuration](https://docs.runme.dev/configuration/cell-level). The page describes no output comparison; a cell fails on its exit status. v3.17.6 released 2026-10-05, Apache-2.0, about 2,200 stars — [runmedev/runme](https://github.com/runmedev/runme)
- `rundoc`: last PyPI release 0.4.5 on 2020-10-19; last push 2020-08-21 — [PyPI rundoc](https://pypi.org/project/rundoc/); [eclecticiq/rundoc](https://github.com/eclecticiq/rundoc)
- `byexample`: one tool for transcript-style examples in Python, Ruby, Shell, GDB, JavaScript, C/C++, Java, PowerShell, iasm, Go, and Rust; it "will compare the output of the examples with the expected ones and it will show any difference"; options are written as `# byexample: +skip`, `+norm-ws` — [byexample](https://byexamples.github.io/byexample/). 11.0.0 released 2026-03-23, GPLv3, 68 stars, about 2,400 downloads in the last month — [PyPI byexample](https://pypi.org/project/byexample/); [pypistats](https://pypistats.org/packages/byexample)

#### Executed documents and literate programming

- **MyST-NB**: `nb_execution_mode` is `off`, `auto` (default; runs only notebooks "with missing outputs"), `force`, `cache` (jupyter-cache; skips notebooks whose code is unchanged), or `inline`; per-cell timeout defaults to 30 s; errors are warnings unless `sphinx-build -W` or `nb_execution_raise_on_error=True`; `raises-exception` cell tag or `nb_execution_allow_errors` for expected errors — [MyST-NB, Execute and cache](https://myst-nb.readthedocs.io/en/latest/computation/execute.html). v1.4.0 released 2026-03-02; about 587,000 downloads in the last month — [PyPI myst-nb](https://pypi.org/project/myst-nb/); [pypistats](https://pypistats.org/packages/myst-nb)
- **nbsphinx**: 0.9.8 released 2025-11-28, MIT, about 849,000 downloads in the last month — [PyPI nbsphinx](https://pypi.org/project/nbsphinx/); [pypistats](https://pypistats.org/packages/nbsphinx)
- **Jupyter Book**: 2.1.7 released 2026-09-23 — [PyPI jupyter-book](https://pypi.org/project/jupyter-book/)
- **nbval**: `--nbval` re-runs a notebook and "the outputs generated will be compared against those in the .ipynb file"; `--nbval-lax` only fails on errors; cell markers `# NBVAL_IGNORE_OUTPUT`, `# NBVAL_CHECK_OUTPUT`, `# NBVAL_RAISES_EXCEPTION`, `# NBVAL_SKIP`; a sanitize file of regex replacements masks dates and similar — [nbval docs](https://nbval.readthedocs.io/en/latest/). Last release 0.11.0 on 2024-03-04; about 404,000 downloads in the last month — [PyPI nbval](https://pypi.org/project/nbval/); [pypistats](https://pypistats.org/packages/nbval). `nbmake` (run-only): 1.5.5 on 2024-12-23, about 366,000 — [PyPI nbmake](https://pypi.org/project/nbmake/)
- **Quarto**: per-block options `eval`, `echo`, `output`, `warning`, `error` (when true, "errors executing code will not halt processing of the document"), `include` — [Quarto, Execution options](https://quarto.org/docs/computations/execution-options.html). `freeze: true` means documents are never re-rendered in a project render and `freeze: auto` re-renders "only when source changes"; results are stored in `_freeze`, which Quarto says to check into version control — [Quarto, Managing execution](https://quarto.org/docs/projects/code-execution.html). v1.10.19 released 2026-10-06 — [quarto-dev/quarto-cli](https://github.com/quarto-dev/quarto-cli)
- **knitr / R Markdown**: chunk options `eval`, `echo`, `include` (if `FALSE` "nothing will be written into the output document, but the code is still evaluated"), `error` (knitr default `TRUE`: "the code evaluation will not stop even in case of errors!"; "R Markdown has changed this default value to `FALSE`"), `cache`, `engine` (python, sql, julia, bash, and others), `purl`, `ref.label` — [knitr options](https://yihui.org/knitr/options/). v1.52 released 2026-09-06 — [yihui/knitr](https://github.com/yihui/knitr)
- **Org-babel**: results are inserted into the buffer under the control of the `:results` header argument (`value` or `output`; `replace`, `append`, `silent`); a session runs code in a persistent interpreter — [Org manual, Results of Evaluation](https://orgmode.org/manual/Results-of-Evaluation.html)
- **Pluto.jl**: "a reactive, educational notebook for Julia" with a built-in package manager and HTML export; MIT — [Pluto docs](https://plutojl.org/en/docs/). v1.0.4 released 2026-10-04 — [JuliaPluto/Pluto.jl](https://github.com/JuliaPluto/Pluto.jl)
- **nbdev**: notebooks are the source; `#| export` cells become the module; test cells live in the same notebook, and `nbdev-test` "will execute this cell (and all other test cells) and fail if they raise any exceptions"; docs are built with Quarto — [nbdev tutorial](https://nbdev.fast.ai/tutorials/tutorial.html). 3.3.25 released 2026-10-05, Apache-2.0, about 5,300 stars — [AnswerDotAI/nbdev](https://github.com/AnswerDotAI/nbdev)
- **Entangled**: fences carry attributes such as `{.cpp #hello-world}` and `{.cpp file=hello_world.cc}`; code is tangled to source files and edits to those files flow back into the Markdown ("a two-way synchronisation mechanism"); `<<name>>` noweb references compose blocks; a daemon (`entangled watch`) keeps them in sync — [Entangled](https://entangled.github.io/). v2.4.3 released 2026-06-09, Apache-2.0, 105 stars — [entangled/entangled.py](https://github.com/entangled/entangled.py)

#### Products and agents

- **Doc Detective**: "a documentation testing framework that ensures your docs are always right"; reads Markdown, AsciiDoc, DITA; it "scans commands, code blocks, and examples users are expected to run" and runs each "in a real environment" — [Doc Detective docs](https://docs.doc-detective.com/). Tests are either written inline as comments in the source or detected from markup patterns; actions include `runShell` (assert on exit code and output), `runCode`, `httpRequest` (assert on headers and body), `checkLink`, `find`, `click`, `screenshot`, `record`, `loadVariables` (reads a `.env` file); agent-tool setup guides for Claude Code, Copilot CLI, Gemini CLI, plus an MCP server and a "Self-healing documentation" page — [Doc Detective docs index](https://docs.doc-detective.com/llms.txt). v4.38.1 released 2026-08-13, AGPL-3.0, 136 stars, about 3,800 npm downloads in the last month — [doc-detective/doc-detective](https://github.com/doc-detective/doc-detective); [npm doc-detective](https://www.npmjs.com/package/doc-detective)
- **Mintlify**: CI checks on Pro and Enterprise plans cover broken internal links and Vale prose linting; no snippet execution is documented — [Mintlify, CI checks](https://www.mintlify.com/docs/deploy/ci)
- **Microsoft Agent Framework `verify-samples` tool**: runs sample projects and verifies output with "deterministic checks and AI-powered verification"; skips samples whose environment variables are missing — [verify-samples-tool listing](https://mcpservers.org/de/agent-skills/microsoft/verify-samples-tool) (secondary listing; the repository itself was not opened)
- **quickstarted**: an agent follows a quickstart; the verdict is the exit code of a script the author writes, and the agent's own success claim is recorded but not trusted; marked early stage — [PyPI quickstarted](https://pypi.org/project/quickstarted/0.3.1/)

### Inferences
- "Passing" is weakest where adoption is highest: twoslash, Go examples without `// Output:`, `no_run`, and pytest-markdown-docs prove only that the example compiles or does not raise. Output-checked styles (doctest transcripts, mdx, cram, `// Output:`) prove more and break more.
- Evaluate-and-render tools (mdoc, markdown-exec, knitr, Quarto, MyST-NB) sidestep "is the shown output current" entirely, at the cost of running code in every docs build.
- Marker placement falls into three camps: info-string words (Rust, pytest-markdown-docs, mdoc, Deno, runme), HTML comments before the fence (pytest-codeblocks, mdx, Knit, typescript-docs-verifier), and in-code comments (twoslash, doctest directives, nbval, doc-builder). HTML comments survive any Markdown renderer; info-string words can confuse syntax highlighters that expect only a language name.

### Gaps
- Swift DocC's own article on snippets returned only a heading when fetched; behavior is cited from SE-0356. Whether the `--enable-experimental-snippet-support` flag is still required in swift-docc-plugin 1.5.0 was not confirmed.
- Sybil's `invisible-code` and fixture details, phmdoctest's directives, markdown-doctest's configuration file, rundoc's tag syntax, and `doc-comment`'s README (which, as I recall, recommends `#[doc = include_str!]` on Rust 1.54 and later) were not opened; only their registry status is confirmed.
- Whether twoslash throws on an unexpected compiler error (my understanding is that it does, unless `@noErrors` or `@errors` is set) is not stated on the two pages fetched.
- Knit's `ARBITRARY_TIME` and `FLEXIBLE_THREAD` test modes were not on the README as fetched.
- noweb and `lit` were not researched. Jupyter Book 2's execution model (built on the MyST document engine) was not read.
- Stripe: no primary source describes how Stripe checks code samples in Markdoc; a secondhand article says examples are generated programmatically — [KnowledgeOwl article](https://www.knowledgeowl.com/help/code-examples-shine-like-stripe). ReadMe: no documented snippet testing found.
- Haskell doctest download counts (Hackage) and Hex/Julia registry counts were not collected.

## What are the documented advantages of keeping the example in the docs and testing it there, compared with keeping it in a code base and extracting it?

### Takeaway
The stated advantages are that the example the reader sees is the source of truth, the author writes prose and code in one place with no marker and include machinery, and the test comes for free with the normal test run. The strongest written case in 2026 is Hugging Face's, which kept examples in Markdown but changed from transcripts to ordinary code.

### Cited Findings
- Hugging Face: "the example itself should remain a primary source of truth"; Markdown acts as a thin test container so the same snippet is not rewritten for docs and tests — [Hugging Face, From doctest to runnable Markdown](https://huggingface.co/blog/huggingface/runnable-examples)
- Python's doctest documentation recommends text files with prose interleaved with examples because they give a coherent narrative rather than a collection of isolated functions, and says examples "should add genuine value to the documentation" — [Python docs, doctest "Soapbox"](https://docs.python.org/3/library/doctest.html)
- Sybil's framing: examples are evaluated "as part of your normal test run" — [Sybil docs](https://sybil.readthedocs.io/en/latest/)
- Rust: a README becomes tests with two lines (`#[doc = include_str!]` on a `#[cfg(doctest)]` item), with no extraction step — [rustdoc book](https://doc.rust-lang.org/rustdoc/write-documentation/documentation-tests.html)
- Deno auto-imports the documented module's exports, so an in-place example needs no visible or hidden import — [Deno docs, doc tests](https://docs.deno.com/runtime/test/doc_tests.md)
- mdoc reports errors at the Markdown file's own line and column and compiles fences as ordinary programs, so examples "can be copy-pasted into normal Scala programs" — [mdoc, Why mdoc](https://scalameta.org/mdoc/docs/why.html)
- Java's own guide notes the cost of the alternative: a hybrid snippet (inline text plus an external tested file) is a maintenance burden because the two must be kept the same, which is why javadoc checks that they agree — [Oracle snippets guide](https://docs.oracle.com/en/java/javase/21/javadoc/snippets.html)
- LangChain describes the extraction route's setup cost as something that "can feel so daunting that the project never happens" (six manual steps: extract, add setup and teardown, add markers, run the extractor, include the snippet, run CI) — [LangChain, How We Made Our Docs Test Themselves (15 Apr 2026)](https://www.langchain.com/blog/our-docs-test-themselves)

### Inferences
- In-place testing has the lowest cost per example and the lowest cost to start; extraction has a fixed cost (markers, a directory of sample projects, an extractor) that pays off as examples get longer.
- In-place keeps review simple: a pull request that changes an example shows the change in the page the reader will see.

### Gaps
- No controlled or quantitative comparison (defect rates, author time) of in-place against extraction was found.

## What are the documented pain points?

### Takeaway
The recurring complaints are brittle output matching, hidden setup that either clutters the example or makes the shown code not what runs, slow compilation per example, weak editor and formatter support inside Markdown or comments, and poor debuggability. Several are structural enough that Swift, Java, and the Rust Book chose external files for anything beyond a short example.

### Cited Findings

**Hidden setup and misleading examples**
- Rust doctests insert `extern crate`, `allow` attributes, and a `fn main` wrapper the reader never sees, and need a hidden `# Ok::<(), io::Error>(())` line to use `?` — [rustdoc book](https://doc.rust-lang.org/rustdoc/write-documentation/documentation-tests.html)
- python-dev, 8 June 2013: Serhiy Storchaka objected that an enum doctest depended on `test.test_enum`, so "The reader should look into the external test module", and asked whether doctest could have "invisible" code that is run but not displayed — [python-dev thread](https://mail.python.org/archives/list/python-dev@python.org/message/NB4HIZS5BHSM2O3XJGHQRHQQEE764PJ3/)
- Sphinx answers with `:hide:` blocks and `doctest_global_setup` in `conf.py`, which moves context out of the page — [Sphinx docs, sphinx.ext.doctest](https://www.sphinx-doc.org/en/master/usage/extensions/doctest.html)
- Hugging Face lists "Setup and teardown clutter examples" among doctest's problems — [Hugging Face](https://huggingface.co/blog/huggingface/runnable-examples)
- Elixir: "doctests are not recommended when your code examples contain side effects" and "doctests do not run in any kind of sandbox" — [ExUnit.DocTest](https://ex-unit.hexdocs.pm/ExUnit.DocTest.html)

**Brittle output**
- Python's docs warn against printing sets (order is not guaranteed), object addresses, and floating-point values that vary by platform, and warn that `ELLIPSIS` can match too much — [Python docs, doctest](https://docs.python.org/3/library/doctest.html)
- Ned Batchelder (2008): you can't run a subset of tests, a failure partway stops the rest of the docstring, code runs in a special way, and explaining and testing are different jobs — [Things I don't like about doctest](https://nedbatchelder.com/blog/200811/things_i_dont_like_about_doctest)
- Ian Bicking (2012): `<BLANKLINE>` is ugly, volatile values are awkward, and `...` doubling as the continuation prompt limits ignoring output — [Why doctest.js is better than Python's doctest](https://ianbicking.org/2012/10/02/why-doctest-js-is-better-than-pythons-doctest)
- Zope developers (2010) called doctests hard to port to Python 3 because they "rely on details of output" — [zope-dev thread](https://mail.zope.org/pipermail/zope-dev/2010-April/040205.html) (seen in a search summary)
- Hugging Face: "Output matching is brittle"; debugging means comparing strings instead of inspecting state — [Hugging Face](https://huggingface.co/blog/huggingface/runnable-examples)

**Slow CI**
- Rust: "the slowest part of doctests is to compile them"; for `core`, 98% of doctest time was compilation — [rustdoc book](https://doc.rust-lang.org/rustdoc/write-documentation/documentation-tests.html); [Guillaume Gomez, Doctests - How were they improved? (17 Aug 2024)](https://blog.guillaume-gomez.fr/articles/2024-08-17+Doctests+-+How+were+they+improved%3F)
- Merged doctests, measured by the rustdoc maintainer: sysinfo 4.6 s to 1.11 s; geos 3.95 s to 0.45 s; `core` 54.08 s to 13.5 s; `std` 12 s to 3.56 s; jiff 4 min 39 s to 7.2 s — [Guillaume Gomez](https://blog.guillaume-gomez.fr/articles/2024-08-17+Doctests+-+How+were+they+improved%3F)
- Merging is only on for 2024-edition crates, and excludes `compile_fail`, crate-level attributes, `test_harness`, and `--nocapture`; if the merged build fails, rustdoc falls back to compiling each doctest separately; a bug in that fallback was fixed in Rust 1.85.1 — [Guillaume Gomez (2024)](https://blog.guillaume-gomez.fr/articles/2024-08-17+Doctests+-+How+were+they+improved%3F); [Guillaume Gomez (20 Mar 2025)](https://blog.guillaume-gomez.fr/articles/2025-03-20+Rustdoc+merged+doctests+%28solved%29+issue+on+stable)
- mdoc: about 5 s per medium document with a cold compiler — [mdoc, Why mdoc](https://scalameta.org/mdoc/docs/why.html)
- Executed documents: Quarto's `freeze` exists because of projects with many computational documents and dependencies that are fragile over time; MyST-NB has a cache mode and a 30 s default cell timeout — [Quarto, Managing execution](https://quarto.org/docs/projects/code-execution.html); [MyST-NB](https://myst-nb.readthedocs.io/en/latest/computation/execute.html)

**Editor, linter, and formatter support**
- SE-0356's motivation: code listings in documentation get no editor support and so tend to contain errors and read like pseudocode; "The code isn't built regularly" — [SE-0356](https://github.com/swiftlang/swift-evolution/blob/main/proposals/0356-swift-snippets.md)
- Docploy chose separate files so examples can use "codebase-specific linters, syntax highlighting, and testing" — [Docploy (25 Aug 2022)](https://dev.to/docploy/why-you-should-test-your-documentation-code-examples-537i)
- rust-analyzer highlights doctest code by injection, largely syntactically, because full analysis of a snippet "usually produces unresolved references" — [rust-analyzer source, inject.rs](https://rust-lang.github.io/rust-analyzer/src/ide/syntax_highlighting/inject.rs.html) (seen in a search summary)
- Java inline snippets cannot contain `*/`, must have balanced braces, and cannot distinguish a character from its Unicode escape — [Oracle snippets guide](https://docs.oracle.com/en/java/javase/21/javadoc/snippets.html)
- Counter-example: pytest-examples lints in-Markdown examples with ruff and black and rewrites them in place; `@eslint/markdown` (about 3.8 million downloads in the last month) lints JavaScript in fences — [pydantic/pytest-examples](https://github.com/pydantic/pytest-examples); [npm @eslint/markdown](https://www.npmjs.com/package/@eslint/markdown)

**Multi-file examples and long tutorials**
- Continuation is opt-in and tool-specific: `continuation` (pytest-markdown-docs), `cont` (pytest-codeblocks), `memory=True` (mktestdocs), `runnable:<name>:2` (doc-builder), shared labels (Documenter), `env=` (mdx), `reset`/`nest` (mdoc). typescript-docs-verifier forbids it: snippets "must compile independently" — sources in the catalog above
- Multi-file in one block exists only in twoslash (`// @filename:`) among the tools read — [twoslash notations](https://twoslash.netlify.app/refs/notations)
- The Rust Book's rule: any listing beyond the most trivial is extracted into a file, each a full Cargo project under `listings/`, pulled in with `{{#rustdoc_include}}` — [rust-lang/book ADMIN_TASKS.md](https://github.com/rust-lang/book/blob/main/ADMIN_TASKS.md)
- A 2026 practitioner guide says doctest-style tools do not cover multi-step flows that start clients, call tools, and assert on structured output — [agentpatterns.ai, Runnable documentation](https://agentpatterns.ai/verification/runnable-documentation/) (secondary source)

**Other limits**
- Rust doctests reach only a library's public API and have not run for binary targets — [cargo issue #5477 via users.rust-lang.org](https://users.rust-lang.org/t/testing-and-maintaining-example-codes-in-documentation/60985) (forum source). Cross-compiled doctests were skipped until a Cargo change merged in May 2025 — [cargo PR #15462](https://github.com/rust-lang/cargo/pull/15462/)
- mdsh locates blocks by regular expression, not a Markdown parser — [zimbatm/mdsh](https://github.com/zimbatm/mdsh)
- Documenter's `fix = true` "can occasionally replace the wrong snippet", so the docs say to commit first — [Documenter.jl](https://documenter.juliadocs.org/stable/man/doctests/)

### Inferences
- The pain points grow with example size. Every source that moved to external files (Rust Book, Swift, OpenImageIO, LangChain, Docploy) did so for long, multi-step, or service-dependent examples; none argues against in-place testing of short API examples.
- Hidden lines trade one failure for another: without them examples carry noise; with them, a reader who copies the visible code may get something that does not compile. mdBook's expandable hidden lines (and Kotlin's `sampleStart`/`sampleEnd` inside a runnable program) are the only designs read here that let the reader see the hidden part.

### Gaps
- No issue-tracker survey was done; pain points come from official docs, maintainers' posts, and blog posts. Write the Docs talks were not found or read.
- No measured CI cost for the Markdown test tools (Sybil, pytest-markdown-docs) was found.

## How do these tools deal with examples that need a real service (a database, an API key)?

### Takeaway
Mostly by not running them: compile-only or skip markers are the standard answer, with test-framework fixtures and environment files as the route for teams that do want a live run. No tool read here has a first-class notion of secrets.

### Cited Findings
- Compile but do not run: Rust `no_run` ("useful for network examples"), Go examples with no `// Output:` comment (for code that "cannot run as unit tests", such as network access), mdoc `compile-only`, `deno check --doc` — [rustdoc book](https://doc.rust-lang.org/rustdoc/write-documentation/documentation-tests.html); [Go blog](https://go.dev/blog/examples); [mdoc modifiers](https://scalameta.org/mdoc/docs/modifiers.html); [Deno docs](https://docs.deno.com/runtime/test/doc_tests.md)
- Skip: doctest `+SKIP` (for "unavailable resources"), `notest`, `<!--pytest.mark.skip-->`, `skipif` with a condition, `importorskip`, mdx `skip`, nbval `# NBVAL_SKIP`, Deno `ignore`, Quarto and knitr `eval: false` — sources in the catalog above
- Run only on request: mdx `non-deterministic=command` blocks are not run unless enabled, and `non-deterministic=output` blocks run without their output being checked — [realworldocaml/mdx](https://github.com/realworldocaml/mdx)
- Fixtures: pytest-markdown-docs injects a named pytest fixture with `fixture:<name>` and has `retry:N` for flaky blocks; Sybil integrates with pytest and unittest; doctest's `DocFileSuite(setUp=, tearDown=)`; Documenter's `DocTestSetup`/`DocTestTeardown` — [modal-labs/pytest-markdown-docs](https://github.com/modal-labs/pytest-markdown-docs); [Sybil docs](https://sybil.readthedocs.io/en/latest/); [Python docs](https://docs.python.org/3/library/doctest.html); [Documenter.jl](https://documenter.juliadocs.org/stable/man/doctests/)
- Markers that route to slow or gated CI jobs: doc-builder's `# pytest-decorator:` line attaches, for example, a slow-test decorator — [Hugging Face](https://huggingface.co/blog/huggingface/runnable-examples)
- Environment and secrets: Doc Detective's `loadVariables` reads a `.env` file; runme prompts for exported variables (`promptEnv`) and has `skipPrompts` for unattended runs; mdx has `set-VAR=` and `unset-VAR` labels — [Doc Detective index](https://docs.doc-detective.com/llms.txt); [Runme](https://docs.runme.dev/configuration/cell-level); [realworldocaml/mdx](https://github.com/realworldocaml/mdx)
- Sandboxing: Deno forwards permission flags from a snippet's hashbang and never grants more than `deno test` has — [Deno docs](https://docs.deno.com/runtime/test/doc_tests.md)
- Skip when the secret is absent: Microsoft's verify-samples tool "automatically checks and skips samples with missing env vars" — [verify-samples-tool listing](https://mcpservers.org/de/agent-skills/microsoft/verify-samples-tool) (secondary)
- Stored outputs as a substitute for a live run: Quarto `freeze` with `_freeze` committed, MyST-NB `auto` mode (only notebooks with missing outputs are run) — [Quarto](https://quarto.org/docs/projects/code-execution.html); [MyST-NB](https://myst-nb.readthedocs.io/en/latest/computation/execute.html)

### Inferences
- Compile-only is the common fallback and is weak for API clients: it catches renamed functions and changed types, and misses changed server behavior.
- A design that wants live runs needs three things the in-place tools mostly leave to the test framework: a way to name a fixture from the fence, a way to gate a block on an environment variable, and a way to keep the secret out of the rendered page.

### Gaps
- No tool documentation read here describes masking secrets in captured output that is then rendered (relevant to evaluate-and-render tools).
- How Doc Detective handles secrets beyond `.env` loading was not confirmed; its inline-test syntax page returned 404.

## How do they express "this output is expected" and keep shown output current?

### Takeaway
There are three models: the author writes expected output and the tool compares; the tool compares and can rewrite the document on request (a snapshot workflow); or the tool renders real output at build time so no expected text exists. The second model is the one newer tools converge on.

### Cited Findings

**Author-written, compared**
- Python: text after the `>>>` line, loosened by `ELLIPSIS` and `NORMALIZE_WHITESPACE` — [Python docs](https://docs.python.org/3/library/doctest.html)
- Go: `// Output:` and `// Unordered output:` — [pkg.go.dev/testing](https://pkg.go.dev/testing)
- Julia: text after `# output`, with regex filters applied to both sides — [Documenter.jl](https://documenter.juliadocs.org/stable/man/doctests/)
- pytest-codeblocks: a following block marked `expected-output` — [nschloe/pytest-codeblocks](https://github.com/nschloe/pytest-codeblocks)
- Knit: a `text` block followed by a `TEST` directive, optionally with a predicate — [Kotlin/kotlinx-knit](https://github.com/Kotlin/kotlinx-knit)
- cram: indented lines with `(re)` and `(glob)` suffixes; mdx: lines without `$`, and `[N]` for exit codes — [cram](https://bitheap.org/cram/); [realworldocaml/mdx](https://github.com/realworldocaml/mdx)
- Rust has no output comparison; results are asserted in code — [rustdoc book](https://doc.rust-lang.org/rustdoc/write-documentation/documentation-tests.html)

**Compared, with rewrite on request**
- Documenter: `doctest(MyPackage, fix = true)` "will run the doctests, and overwrite the old results with the new output" — [Documenter.jl](https://documenter.juliadocs.org/stable/man/doctests/)
- mdx: writes `<file.md>.corrected`; `dune promote` accepts it — [realworldocaml/mdx](https://github.com/realworldocaml/mdx)
- pytest-examples: `--update-examples` rewrites `#>` print comments — [pydantic/pytest-examples](https://github.com/pydantic/pytest-examples)
- cram: `-i` merges actual output into the test file — [cram](https://bitheap.org/cram/)
- mdsh: rewrites output blocks in place; `--frozen` fails in CI if a rewrite would change anything — [zimbatm/mdsh](https://github.com/zimbatm/mdsh)
- Knit: `knit` regenerates and `knitCheck` fails if files are stale — [Kotlin/kotlinx-knit](https://github.com/Kotlin/kotlinx-knit)
- nbval: compares re-executed output with the output stored in the `.ipynb` — [nbval docs](https://nbval.readthedocs.io/en/latest/)
- The Rust Book keeps command output in `output.txt` files regenerated by `tools/update-rustc.sh`, with a `manual-regeneration` comment for output a script cannot produce — [rust-lang/book ADMIN_TASKS.md](https://github.com/rust-lang/book/blob/main/ADMIN_TASKS.md)

**Rendered at build time**
- mdoc renders results as comments under the code; `fail`, `warn`, and `crash` render the real error — [mdoc modifiers](https://scalameta.org/mdoc/docs/modifiers.html)
- twoslash `// ^?` renders the compiler's own type at that position; `@errors` renders the real message — [twoslash notations](https://twoslash.netlify.app/refs/notations); [twoslash guide](https://twoslash.netlify.app/guide/)
- markdown-exec, knitr, Quarto, MyST-NB, Org-babel insert results into the output — sources in the catalog above

**Non-deterministic output**
- Filters or sanitizers: Documenter `DocTestFilters`, nbval's regex sanitize file, doctest `ELLIPSIS`, cram `(re)`/`(glob)`, Knit's `LINES_START` and predicates — sources in the catalog above
- Marked non-deterministic: mdx `non-deterministic=output` — [realworldocaml/mdx](https://github.com/realworldocaml/mdx)

### Inferences
- A check mode and an update mode over the same data (mdx, mdsh `--frozen`, Knit, pytest-examples, Documenter) is the pattern that removes the brittleness complaint: the author stops hand-editing expected output and reviews a diff instead.
- Expected failures are a separate axis and are well served in place: `compile_fail`, `should_panic`, mdoc `fail`/`warn`/`crash`, twoslash `@errors`, `xfail`, `raises-exception`, markdown-exec `returncode=`. These examples are awkward to express as extracted regions of a code base that must itself build.

### Gaps
- Whether any Markdown-level tool pins expected compiler error text (as opposed to error codes or "must fail") in a stable way across compiler versions was not found; Rust's `compile_fail` accepts an error code in the fence (as I recall), not confirmed here.

## Which are actively maintained and widely adopted, and which are abandoned?

### Takeaway
The language-native mechanisms and the evaluate-and-render tools are healthy; in the Markdown-testing tier, Sybil, pytest-markdown-docs, pytest-codeblocks, pytest-examples, mdx, mdoc, Knit, and twoslash are active, while tut, skeptic, phmdoctest, markdown-doctest, rundoc, and tsdoc-testify are archived or dormant.

### Cited Findings

| Tool | Latest release (date) | Signal | Status | Source |
|---|---|---|---|---|
| twoslash | 0.3.9 (2026-06-22) | 3.9M npm/month | Active | [npm](https://www.npmjs.com/package/twoslash) |
| markdown-exec | 1.12.4 (2026-10-06) | 979k PyPI/month | Active | [PyPI](https://pypi.org/project/markdown-exec/) |
| xdoctest | 1.3.2 (2026-03-27) | 1.04M PyPI/month | Active | [PyPI](https://pypi.org/project/xdoctest/) |
| nbsphinx | 0.9.8 (2025-11-28) | 849k PyPI/month | Active | [PyPI](https://pypi.org/project/nbsphinx/) |
| MyST-NB | 1.4.0 (2026-03-02) | 587k PyPI/month | Active | [PyPI](https://pypi.org/project/myst-nb/) |
| pytest-examples | 0.0.18 (2025-05-06) | 516k PyPI/month | Active (repo pushed 2026-10-08) | [GitHub](https://github.com/pydantic/pytest-examples) |
| nbval | 0.11.0 (2024-03-04) | 404k PyPI/month | Slow (repo pushed 2025-09-17) | [GitHub](https://github.com/computationalmodelling/nbval) |
| Sybil | 10.1.0 (2026-06-13) | 318k PyPI/month | Active | [PyPI](https://pypi.org/project/sybil/) |
| pytest-markdown-docs | 0.9.2 (2026-03-23) | 139k PyPI/month | Active | [PyPI](https://pypi.org/project/pytest-markdown-docs/) |
| pytest-codeblocks | 0.18.0 (2026-06-15) | 115k PyPI/month | Active | [PyPI](https://pypi.org/project/pytest-codeblocks/) |
| mktestdocs | 0.2.5 (2025-07-25) | 36k PyPI/month | Maintained, slow | [PyPI](https://pypi.org/project/mktestdocs/) |
| typescript-docs-verifier | 3.0.2 (2026-03-02) | 32k npm/month | Maintained | [npm](https://www.npmjs.com/package/typescript-docs-verifier) |
| mdBook | 0.5.4 (2026-07-06) | 22k stars | Active | [GitHub](https://github.com/rust-lang/mdBook) |
| OCaml mdx | 2.7.0 (2026-10-02) | 291 stars | Active | [GitHub](https://github.com/realworldocaml/mdx) |
| Scala mdoc | 2.9.2 (2026-09-03) | 402 stars | Active | [GitHub](https://github.com/scalameta/mdoc) |
| kotlinx-knit | 0.5.1 (2026-01-23) | 321 stars | Maintained | [GitHub](https://github.com/Kotlin/kotlinx-knit) |
| Documenter.jl | 1.19.0 (2026-09-02) | 919 stars | Active | [GitHub](https://github.com/JuliaDocs/Documenter.jl) |
| Haskell doctest | (pushed 2026-09-26) | 400 stars | Active | [GitHub](https://github.com/sol/doctest) |
| runme | 3.17.6 (2026-10-05) | 2.2k stars; 2.2k npm/month | Active | [GitHub](https://github.com/runmedev/runme) |
| Doc Detective | 4.38.1 (2026-08-13) | 136 stars; 3.8k npm/month; AGPL-3.0 | Active | [GitHub](https://github.com/doc-detective/doc-detective) |
| byexample | 11.0.0 (2026-03-23) | 2.4k PyPI/month; GPLv3 | Maintained, small | [PyPI](https://pypi.org/project/byexample/) |
| mdsh (Rust) | (pushed 2026-07-23) | 174 stars | Maintained | [GitHub](https://github.com/zimbatm/mdsh) |
| Entangled | 2.4.3 (2026-06-09) | 105 stars | Maintained | [GitHub](https://github.com/entangled/entangled.py) |
| nbdev | 3.3.25 (2026-10-05) | 5.3k stars; 64k PyPI/month | Active | [GitHub](https://github.com/AnswerDotAI/nbdev) |
| Quarto | 1.10.19 (2026-10-06) | 6.1k stars | Active | [GitHub](https://github.com/quarto-dev/quarto-cli) |
| knitr | 1.52 (2026-09-06) | 2.5k stars | Active | [GitHub](https://github.com/yihui/knitr) |
| Pluto.jl | 1.0.4 (2026-10-04) | 5.4k stars | Active | [GitHub](https://github.com/JuliaPluto/Pluto.jl) |
| doc-comment (Rust) | 0.3.4 (2025-10-24) | 17M crates.io/90 days | Maintained; largely superseded by `include_str!` (not confirmed) | [crates.io](https://crates.io/crates/doc-comment) |
| skeptic (Rust) | 0.13.7 (2022-02-01) | 1.9M crates.io/90 days; repo pushed 2024-03-25 | Dormant | [crates.io](https://crates.io/crates/skeptic) |
| cram | 0.7 (2016-02-24) | repo pushed 2026-08-01 | No release in 10 years | [PyPI](https://pypi.org/project/cram/) |
| phmdoctest | 1.4.0 (2022-03-19) | 11.5k PyPI/month | Dormant | [PyPI](https://pypi.org/project/phmdoctest/) |
| markdown-doctest | 1.1.0 (2020-10-07) | 10k npm/month | Dormant | [npm](https://www.npmjs.com/package/markdown-doctest) |
| rundoc | 0.4.5 (2020-10-19) | 56 stars | Dormant | [PyPI](https://pypi.org/project/rundoc/) |
| tsdoc-testify | 0.0.3 (2019-12-06) | 99 npm/month | Abandoned | [npm](https://www.npmjs.com/package/tsdoc-testify) |
| tut (Scala) | archived 2021-04-13 | 572 stars | Archived; "Please switch to mdoc" | [GitHub](https://github.com/tpolecat/tut) |

- Licenses seen: MIT (Sybil, pytest-markdown-docs, pytest-codeblocks, pytest-examples, twoslash, Documenter, Haskell doctest, mdsh, Pluto), Apache-2.0 (mktestdocs, mdoc, runme, nbdev, Entangled, typescript-docs-verifier, xdoctest, swift-docc-plugin), ISC (mdx, markdown-exec), MPL-2.0 (mdBook), GPLv3 (byexample), GPL-2.0-or-later (cram, prysk), AGPL-3.0 (Doc Detective) — registry and repository pages linked in the table

### Inferences
- Download counts overstate use for packages that are build dependencies of popular projects (twoslash through Shiki integrations; skeptic and doc-comment through old dependents) and say nothing about how many examples are tested.
- Every abandoned tool in the list was a single-purpose bridge that a platform feature later replaced (tut by mdoc; skeptic and doc-comment by `#[doc = include_str!]`; phmdoctest by pytest plugins that collect Markdown directly). The surviving third-party tools plug into an existing test runner rather than generating test files.
- The copyleft licenses (AGPL for Doc Detective, GPL for byexample and cram) matter for anyone thinking of embedding rather than invoking these tools.

### Gaps
- Download figures for Hackage, Hex, Julia's registry, Maven (mdoc, Knit), and opam (mdx) were not collected, so adoption for those is shown by stars only.
- "Dormant" is judged from release and push dates alone; none of those repositories carries an archive flag except tut.

## Has any project published a comparison of, or a migration between, in-place testing and extraction from tested code, and what did it conclude?

### Takeaway
Yes, in both directions, and no source declares one approach the winner. Published moves toward extraction (OpenImageIO, LangChain, Docploy, the Rust Book's listings, Swift's design) cite full-program context, linting, and editor support; the one prominent 2026 write-up that stayed in place (Hugging Face) kept Markdown as the source of truth and dropped transcript matching instead.

### Cited Findings
- **Hugging Face, 4 April 2026 (stayed in place, changed the mechanism)**: moved from doctest to runnable Markdown blocks collected by pytest. Reasons against doctest: "Good documentation and good tests are not the same thing", brittle output matching, setup clutter, string-comparison debugging; the history given is that projects moved real testing to pytest and left docs unexecuted, and "The result was a familiar problem: documentation drift." The new blocks "execute as normal Python code, not interpreter transcripts", and "the example itself should remain a primary source of truth." The post lists no drawbacks of the new approach — [Hugging Face, From doctest to runnable Markdown](https://huggingface.co/blog/huggingface/runnable-examples)
- **LangChain, 15 April 2026 (moved to extraction)**: inline samples were moved to standalone files under `src/code-samples/{product}`, given setup and teardown, linted, tagged with `:snippet-start:`/`:snippet-end:` (and `:remove-start:`/`:remove-end:`), extracted with Bluehawk, included as Mintlify reusable snippets, and run by a scheduled GitHub Action that opens tickets on failure. The migration itself is done by an agent skill. The post gives no sample counts or CI times and does not compare against in-place testing — [LangChain, How We Made Our Docs Test Themselves](https://www.langchain.com/blog/our-docs-test-themselves)
- **OpenImageIO (moved to extraction)**: because some doc examples "have bugs or are incorrect in various ways", the project is converting "every significant code example in the docs into a test" so "the documentation will merely include the test code by reference, so they can never get out of sync again"; mechanism is `BEGIN-`/`END-` marker comments and Sphinx `literalinclude` with `:start-after:` and `:end-before:`; page last edited 21 September 2023 — [OpenImageIO wiki](https://github.com/OpenImageIO/oiio/wiki/Converting-documentation-examples-to-tests)
- **Docploy, 25 August 2022 (chose extraction)**: "We made the decision to import code from a separate file rather than embed the code directly into a Markdown file", so the file can use the code base's linters, highlighting, and tests; the included file is a Jest test and only the text between `// [start]` and `// [end]` is shown — [Docploy](https://dev.to/docploy/why-you-should-test-your-documentation-code-examples-537i)
- **The Rust Book (hybrid)**: listings beyond the trivial live as full Cargo projects under `listings/`; `{{#rustdoc_include}}` pulls in the whole file with unshown lines hidden, so `mdbook test` still tests the page in place — [rust-lang/book ADMIN_TASKS.md](https://github.com/rust-lang/book/blob/main/ADMIN_TASKS.md); [mdBook](https://rust-lang.github.io/mdBook/format/mdbook.html)
- **Swift SE-0356 (design-time comparison)**: rejected snippets in doc comments (it "limits their utility") and a literate design ("not a small undertaking"), and also rejected taking snippets from tests: "Snippets are not meant to be tests or come directly from tests" — [SE-0356](https://github.com/swiftlang/swift-evolution/blob/main/proposals/0356-swift-snippets.md)
- **Java (design-time comparison)**: supports both and recommends external snippets when the code is to be tested, since javadoc itself tests nothing — [Oracle snippets guide](https://docs.oracle.com/en/java/javase/21/javadoc/snippets.html)
- **Scala tut to mdoc (in-place to in-place)**: the move was from REPL semantics to program semantics, for copy-paste fidelity, error positions, and speed — [mdoc, Why mdoc](https://scalameta.org/mdoc/docs/why.html)
- **OCaml mdx (bridge)**: one tool does both; `file=` and `part=` labels keep a fenced block in sync with a region of a real source file, alongside in-place toplevel and shell blocks — [realworldocaml/mdx](https://github.com/realworldocaml/mdx)
- **MongoDB**: docs guidance says to link generated code files with `literalinclude` or `io-code-block` instead of hard-coding blocks, and a `grove-migrate` skill converts untested inline examples — [MongoDB meta docs, code example testing](https://www.mongodb.com/es/docs/meta/grove/code-example-testing) (seen in a search summary, not opened)
- **Spocktest** author: example code should be injected into documentation rather than documentation serving as the place tests are kept, citing poor debuggability of sphinx-doctest tests — [PyPI spocktest](https://pypi.org/project/spocktest/) (seen in a search summary)

### Inferences
- The published pattern is a split by example size: short API examples are tested in place (Rust, Python, Elixir, Julia, Go, Deno all keep this as the default), and tutorials or service-backed samples are full projects included by reference.
- Hybrids are the convergent design: mdBook `rustdoc_include`, mdx `file=`/`part=`, Java hybrid snippets, and Knit (Markdown is the source, generated files are committed and checked) each keep one source of truth and verify the other side mechanically.
- Agents changed the economics in 2026: both the LangChain migration and Doc Detective's agent tooling treat the one-time conversion cost, formerly the main argument for in-place testing, as something an agent absorbs. That favors extraction for large doc sets, but the evidence is two vendor posts.

### Gaps
- No neutral, side-by-side evaluation was found; every source is a project explaining its own choice, and two of the 2026 posts promote the author's product.
- No source reports a migration from extraction back to in-place testing.
- No Write the Docs talk on this comparison was located in this session.
