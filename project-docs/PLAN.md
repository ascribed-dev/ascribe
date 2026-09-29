# Tessera implementation plan

This plan covers building Tessera v1: the compiler, its command-line interface and language server, the VS Code extension, the Astro integration, and the web component library. [SPEC.md](../SPEC.md) defines what they must do; this document defines how. The order of work, and which parts can proceed in parallel, is in [phases/](phases/README.md).

## Scope

**In v1**

- A Rust compiler that parses, validates, resolves, and emits Tessera documentation sets.
- The `tessera` binary: `check`, `build`, `fmt`, and `lsp` subcommands.
- A VS Code extension that hosts the language server, with a live preview.
- `@tessera/astro`, the Astro integration, and `@tessera/elements`, the web component library.
- Distribution through npm, GitHub releases, and the VS Code Marketplace.

**Not in v1**

- The AI layer (a chat participant and authoring skills, as in warp-writer). The language server's project index is designed to be the context it will use later.
- Multi-locale content. The spec doesn't define it yet.
- Consumers other than Astro. Consumer profiles are designed in, but only Astro's is implemented. The plain-markdown output doesn't depend on any consumer.

## Key decisions

| Area | Decision | Why |
|---|---|---|
| Language | Rust | One fast native binary for the CLI, CI, and the editor |
| Parser base | A fork of [comrak](https://github.com/kivikakk/comrak) | A port of the CommonMark reference parser with source positions, which already adds extensions as block types |
| Content model | `ascribe.toml`; Tessera generates the Zod schema Astro needs | Rust reads it natively, so the language server doesn't need Node; TOML never guesses types |
| Editor integration | The same binary runs as a language server (`tessera lsp`) | Keeps the project index in memory, receives edits as they happen, and gets editor features from the LSP client library |
| Distribution | The binary through npm, with per-platform packages | Astro projects install it like any dev dependency, and CI and the editor can use the same version |
| Extension's binary | Bundled in platform-specific extension packages; the project's own `node_modules` copy is preferred when present | Works immediately, and matches CI when the project pins a version |
| Web components | Plain custom elements, no framework | Small, no dependencies, and they work in any site |

## Repository layout

One repository holds the Rust workspace and the JavaScript packages:

```
crates/
  comrak-tessera/     the forked CommonMark parser
  tessera-core/       shared types: spans, line index, directive schemas, attribute and availability-spec parsers
  tessera-syntax/     Tessera parsing: Tessera lines, structure pass, syntax tree
  tessera-model/      loads and validates ascribe.toml
  tessera-check/      file- and page-level validation, diagnostics
  tessera-resolve/    project graph and resolution passes
  tessera-emit/       site markdown, plain markdown, JSON; Zod schema generation
  tessera-fmt/        formatter
  tessera-lsp/        language server
  tessera-cli/        the `tessera` binary
packages/
  cli/                @tessera/cli and its per-platform binary packages
  astro/              @tessera/astro
  elements/           @tessera/elements
  vscode/             the VS Code extension
tests/
  conformance/        examples extracted from SPEC.md, with expected results
  commonmark/         the CommonMark spec test suite
examples/             sample documentation sets, including the Quill page
```

`tessera-cli` and `tessera-lsp` depend on the same crates. That's what makes the spec's promise, that the command line and the editor report the same diagnostics, true by construction.

## Compiler

### Parser (`comrak-tessera`, `tessera-syntax`)

Tessera changes CommonMark's block structure, so its grammar has to live inside the block parser. Parsing the output of an unmodified parser can't produce the right structure. For example, an unmodified parser folds an unindented `@note` into the list item above it, where Tessera ends the list.

The changes to comrak stay small:

1. **One new leaf block, the Tessera line.** It covers directive lines and end lines. It's recognized only for known keywords, which come from the built-in set and the content model's project widgets. It can interrupt a paragraph and is never a lazy continuation line. When it has a text primary, it continues onto following lines the way a paragraph does.
2. **Inline extensions** for `{key}` phrases and for attribute blocks directly after images. Phrase candidates are parsed syntactically and resolved against the registry later, since they don't affect block structure.

Everything else happens in a **structure pass** over the parsed tree, in `tessera-syntax`:

- **Containers and groups.** A stack walk over each parent's children. This works because containers can't straddle list items or blockquotes, and each directive's form is visible on its own line. The walk also implements the rule that a new `@variant` arm reports containers still open inside the previous arm.
- **Title lines** need no lookahead in the block parser. A title line followed by a directive line becomes a one-line paragraph that the directive line interrupts. The structure pass turns a one-line paragraph that starts with `.` and sits directly above a directive that accepts a title into that directive's title. A `.` line that continues a longer paragraph stays text, as the spec requires.
- **Bindings.** Heading-bound directives at the top of a section, and following-block directives to the block they touch.

**Positions.** Every node keeps byte offsets, and each Tessera line keeps sub-spans for its name, attributes, colon, and primary. The formatter and the language server need these. Line and column, including LSP's UTF-16 positions, are computed on demand from a per-file line index.

**Fork maintenance.** The fork lives in the workspace with its license, and a documented procedure for merging upstream releases. Tessera's changes stay confined to a few clearly marked places.

### Content model (`tessera-model`)

`tessera-model` loads `ascribe.toml` into typed structures and enforces the content model's own rules, such as rejecting a name used in more than one role. Specifying the file's format is part of the first milestone. A sketch:

```toml
spec = "0.1"

[types.guide.frontmatter]
title = "string"
description = "string?"

[fragments]
patterns = ["includes/**"]

[dimensions.deployment]
values = ["cloud", "self-managed"]
versionless = ["cloud"]
labels = { cloud = "Quill Cloud", self-managed = "Self-managed" }

[dimensions.pm]
values = ["npm", "pnpm", "yarn"]
labels = { yarn = "Yarn" }

[phrases]
product = "Quill"
cloud = "Quill Cloud"

[features.streaming-sync]
name = "Streaming sync"
available = "cloud, self-managed preview 3.4"

[widgets.quill-labspace]
forms = ["line"]
binding = "self"
attributes = { lab = "string" }

[consumer]
profile = "astro"

[builds.site]
variants = "switch"
availability = "badge"
```

### Validation (`tessera-check`)

- Every row of the spec's diagnostics table (§8.2) gets a stable code (for example `ASC012`), a severity, a source span, and, where one exists, a quick fix. The CLI and the language server share these structures.
- File-level checks run per file. Page-level checks run per page and per build, after resolution (§8.1).
- The CLI prints diagnostics for people (via `miette` or `ariadne`) or as JSON for tools.

### Project graph and resolution (`tessera-resolve`)

- The graph indexes files, fragments, pages, heading ids, titles, include edges, and links. Pages that include a fragment are recorded, so a change to the fragment re-checks them.
- The resolution passes run in the spec's order (§9.2): includes, availability, build modes, phrases, heading ids, links, glossary. Included content keeps its source file, so relative paths resolve correctly.
- Slugging follows the consumer profile. The first implementation is a port of `github-slugger`, which Astro uses.
- The graph updates per file so the language server can keep it current as the author types. If profiling shows a need, it can move to an incremental-computation framework such as `salsa` later.

### Output (`tessera-emit`)

- **Plain markdown and JSON** come first. They're the easiest to snapshot-test, and they prove the resolution passes.
- **Site markdown** follows: markdown plus `tessera-*` custom elements, shaped by the consumer profile.
- **Zod schema generation** for Astro's content collections, from the content model's frontmatter schemas.

### Formatter (`tessera-fmt`)

The formatter rewrites Tessera constructs into canonical form (§8.3) and returns minimal text edits. It never re-renders the author's markdown, which makes it safe to run on save. Its conformance check: formatting any conformance example is idempotent and doesn't change the parsed tree.

## Astro integration and elements

- **`@tessera/astro`** runs `tessera build` before Astro loads content. It writes the generated Zod schema and wires the compiled pages into a content collection. It also loads the element library, and adds the markdown plugin that applies explicit heading ids, because Astro's own heading ids come from its slugger.
- **`@tessera/elements`** holds the custom elements in the light DOM, themed through CSS custom properties. Only `<ascribe-tabs>` needs JavaScript, and without it every arm shows with its label (§9.7). Tab selections sync by dimension across the page, and are remembered.

## VS Code extension

The extension is a small TypeScript client. The language intelligence lives in `tessera lsp`.

**Client responsibilities**

- Find the binary: the project's `node_modules/.bin/tessera` first, then the one bundled in the extension.
- Start `tessera lsp` through `vscode-languageclient`, and restart it when `ascribe.toml` changes in ways the server can't reload in place.
- Provide a TextMate grammar injected into markdown for immediate highlighting, as warp-writer does. The server adds semantic tokens for anything that depends on the content model, such as known keywords, declared phrases, and title lines.
- Provide the preview (below).

**Language server features**, following spec §10:

| Feature | Details |
|---|---|
| Diagnostics | Every §8.2 diagnostic, pushed as the author types |
| Completion | Directive names; attribute keys and values; dimension values and lifecycle states; phrase and feature keys; include paths; link targets searched by page and heading title and inserted as file paths |
| Hover | A link's full path and a preview of the target; a phrase's value; a feature's availability |
| Navigation | Go to definition for links, includes, ids, and phrases; a CodeLens naming and opening the target file |
| Inline hints | The resolved text of empty-text links |
| Quick fixes | "Did you mean" for misspelled directives, adding a missing trailing colon, removing a blank line before a bound block, converting route-style links to file paths |
| Refactoring | Renaming or moving a file updates links and includes; changing an id updates links; renaming a phrase key updates its uses |
| Formatting | Canonical form through `tessera-fmt`, on save if enabled |
| Title lines | Displayed distinctly, so accidental titles are easy to spot |

**Preview.** A webview renders the current page through the site emitter and `@tessera/elements`, using the same code path as the published site, so the preview and the site match. It updates as the author types, including unsaved changes, and offers a picker for the build whose modes to preview (for example, `switch` or a selected dimension).

**Packaging.** Platform-specific extension packages each include the matching binary. The extension checks the project's binary version and warns when it's older than the extension expects.

## Testing

- **Conformance.** Examples extracted from SPEC.md, plus additional cases for every rule and every diagnostic. Each case pairs an input with its expected syntax tree, diagnostics, and outputs, and is snapshot-tested with `insta`.
- **CommonMark.** The official spec suite runs against the fork. The few cases where Tessera deliberately differs, which involve lines that are valid directives, are listed and justified.
- **Real-world corpora.** The Astro, Elastic, and Docker documentation sets from the reaction test are converted in part to Tessera, to test performance and catch false positives such as prose `@` and `{…}`.
- **Performance targets.** A full `tessera check` of a 3,000-page project in a few seconds; language server responses on a keystroke within about 50 ms.
- **Extension.** Integration tests with `@vscode/test-electron` against the example projects.

## Milestones

Each milestone ends with an exit criterion. The language server comes early, right after validation, because authoring-time feedback is the core of the product and needs only parsing and checks. [phases/](phases/README.md) breaks these milestones into smaller phases for implementation, and puts page-level checks after resolution, since the spec runs them on resolved pages (§8.1).

| # | Milestone | Includes | Exit criterion |
|---|---|---|---|
| 0 | Foundations | Repository and workspace; CI; conformance harness that extracts SPEC.md examples; CommonMark suite running against unmodified comrak; the `ascribe.toml` format specified | Both test suites run in CI |
| 1 | Parser spike | The Tessera-line block in the comrak fork: interrupting paragraphs, no lazy continuation, text primaries that continue across lines | Targeted tests pass; decide whether to continue with the fork |
| 2 | Parser | The structure pass (forms, containers, groups, titles, bindings, lists and blockquotes); phrase and image-attribute inlines; spans and line index | All §3–§6 conformance examples produce their expected trees; the CommonMark suite passes apart from documented differences |
| 3 | Content model and file-level checks | `tessera-model`; file-level validation; diagnostic codes; `tessera check` with readable and JSON output | Every file-level row of §8.2 has a passing test |
| 4 | Project graph and page-level checks | Fragments; includes with source-path ownership; slugs; links; ids per build | Every page-level row of §8.2 has a passing test; the Quill example checks clean |
| 5 | Language server and extension v0 | `tessera lsp` with document sync, diagnostics, completion, hover, navigation, CodeLens, inline hints; the extension client, binary lookup, and highlighting | The extension is usable day to day on the example project |
| 6 | Resolution, plain markdown, JSON | The §9.2 passes; `tessera build`; plain-markdown and JSON emitters; build modes | The Quill example builds under every build mode, with snapshot tests |
| 7 | Site output and Astro | Site-markdown emitter; Astro consumer profile; Zod generation; `@tessera/astro`; `@tessera/elements`; npm packages for the binary | A sample Astro site renders the Quill page with working tabs, notes, and badges, built in CI |
| 8 | Formatter, quick fixes, refactoring | `tessera fmt`; formatting on save; the quick fixes and refactorings above | Formatting conformance examples is idempotent and preserves the tree; refactor tests pass |
| 9 | Preview | The webview preview through the site emitter, with a build picker | The preview matches the Astro sample's output for the Quill page |
| 10 | Release | Marketplace and npm publishing; documentation; checks that the Tessera name is available on GitHub, npm, and the Marketplace | v1 published |

## Risks

| Risk | Mitigation |
|---|---|
| The comrak fork is harder to extend or maintain than expected | The milestone 1 spike decides early; markdown-rs is the fallback |
| Tessera's changes to block structure interact badly with CommonMark edge cases | The CommonMark suite and real-world corpora in CI; documented, justified differences only |
| General markdown formatters merge title and directive lines | `tessera fmt`, guidance to exclude Tessera sources from other formatters, and a diagnostic when a directive line is merged into a paragraph |
| VS Code can't hide text within a line | Rely on hover, CodeLens, and inlay hints, as the spec already does |
| Large projects slow the language server | Per-file incremental updates, profiling against the 3,000-page Elastic corpus, and `salsa` if needed |
| The editor and CI disagree | One set of crates for both, and a version check between the extension and the project's binary |
