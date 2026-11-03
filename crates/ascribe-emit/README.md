# ascribe-emit

Emitters for Ascribe's outputs. An emitter renders the **resolved tree** (`ascribe_resolve::ResolvedPage`) and never works out what a build mode keeps: a selection build that keeps several arms of a group emits them all, and a filter build emits the availability annotations still attached (SPEC §9.2).

| Output | Emitter | Phase |
|---|---|---|
| Plain markdown (`plain/`) | `PlainEmitter` | 18 |
| JSON (`json/`) | `JsonEmitter` | 18 |
| Site markdown plus web components (`site/`) | `SiteEmitter` | 20 |

## Pieces

- `Emitter` (`emitter.rs`): `name`, `page_path`, `render_page`, and the defaults `place_asset` (the mirrored path and a relative reference, asset contract §3), `generated` (files under `_ascribe/`), and `warnings`. `emit(&dyn Emitter, &EmitContext, &ResolvedBuild) -> Emission` renders every page and lists each asset the pages use once.
- `assets.rs`: `mirrored_path`, `relative_reference`, `encode_path`, and `markdown_destination`, which write a reference so CommonMark reads it back as written (asset contract §4).
- `write_outputs` (`write.rs`): what `ascribe build` does once its checks pass: resolves each build and writes each output through `OutputDir`, reporting each as it's written.
- `OutputDir` (`store.rs`): the [output-layout contract](https://ascribed-dev.com/go/output-layout-contract/): the lock, staging, manifests, and replacing a previous output without touching a file the manifest didn't list.
- `labels.rs`: text the outputs share (the availability line and each target's text, an arm's label).
- `site/`, `render/`, `zod/`: the site output (below).

```rust,ignore
let cx = EmitContext::new(&project, project_root, build);
let emission = emit(&PlainEmitter::new(project.model()), &cx, &resolved)?;
let output = OutputDir::lock(&project_root.join(".ascribe/build"))?;
output.replace(&build.name, "plain", &emission.files)?;
```

## Plain markdown

Fully resolved CommonMark with no HTML. A page is its title as a level-1 heading (with its code spans, when `title` sets `inline = "code"`), the page-level `Available:` line if it has one, then its blocks. SPEC §9.4's table, as implemented:

| Source | Output |
|---|---|
| `@note {type=tip}` with a title | `> **Tip: Title**`, a blank quoted line, then the content; without a title, `> **Tip**` |
| `@steps` | The ordered list |
| A `@variant` group with more than one arm left | A line, `A reader sees one of the next 3 sections at a time, chosen by package manager.` (the dimension the site's tabs sync on; nothing after `at a time` for labeled arms), then one section per arm: a heading one level below the section it's in (at most 6) naming each dimension and value, `### Package manager: pnpm`, or the arm's title, then its content, each of its headings ending with the arm's label in parentheses (`## Install (pnpm)`) |
| A group of a project widget | One section per arm: the arm's bold label, then its content, its headings ending with the label in parentheses |
| `@details` | The title in bold, then the content |
| `@available` | `Available: Quill Cloud (GA); self-managed (preview, 3.4+)` |
| A project widget | Its `plain-fallback` (phrases substituted), then its wrapped content unless `plain-content = "drop"` |
| Phrases, includes, glossary links | Resolved |
| Links | Absolute URLs with the `[consumer] site` origin; root-relative, with a warning from `ascribe build`, without one |
| Images and linked files | Copied into the output at their mirrored path, and written as relative references |
| Raw HTML in the source | Its text, without the tags; comments, scripts, and styles dropped |

Code blocks are always fenced (the fence is longer than any backtick run in the code, and a `phrases=true` info word is dropped), and text is escaped so that an unmodified CommonMark parser reads back the same text.

`PlainEmitter::new(model)` reads `[consumer] agents`; `PlainEmitter::default()` is the output without it. With `agents = true` the output is what a site publishes for agents, laid out by URL (output-layout contract §1.1): each page at its Astro entry id with `.md`, opening with a blockquote linking to `llms.txt`; every asset under `_ascribe/files/`, written as an absolute URL and listed with its `url` in the manifest; and `generated` adds `llms.txt`, and a file per section when it's split, from `ascribe_resolve::llms`. `prepare` then refuses a build whose pages share a route, as the site emitter's does.

## Site

Markdown plus web components, for a consumer that renders CommonMark with raw HTML: spec 0.1's is Astro (`AstroProfile`, written against Astro 7.3). `SiteEmitter::new(model)` implements `Emitter`; with `agents = true`, each page opens with `<ascribe-for-agents>` (element contract §8); `place_asset` follows the profile (images mirrored beside their page with a relative reference; other linked files under `_ascribe/files/`, referenced by URL and listed with it in the manifest), and `generated` adds `_ascribe/schema.ts`, `_ascribe/permalinks.json`, `_ascribe/pages.json`, the build's page records from `ascribe_resolve::pages` (output-layout contract §1.3), and `_ascribe/headers.json`, the response headers each page's Markdown version needs from the host, from the same records (§1.4), `_ascribe/nav.json`, the build's share of the navigation (§1.6), and `_ascribe/help.json`, each permalink's section rendered by the plain renderer for a product to show (§1.7). A page's `listing` is in its record and left out of its frontmatter. `prepare` refuses a build whose pages share a route.

| Source | Output |
|---|---|
| Frontmatter | Passed through, phrases substituted; `available` becomes a list of targets; a field that sets `inline = "code"` is its plain text, with its HTML under `formatted` |
| Heading | An ATX heading ending in `<ascribe-attributes id="…"></ascribe-attributes>` with its page id, after a space |
| Image | `![alt](./path "title")`, then a marker with its attributes and the model's defaults |
| `@note` | `<ascribe-note type label heading>` wrapping the content |
| `@steps` | `<ascribe-steps>` wrapping the list |
| `@variant` group | `<ascribe-tabs sync>` of `<ascribe-tab value label>` per surviving arm, each starting with its label as `<p class="ascribe-tab-label">` (a group reduced to one arm is its content) |
| `@details` | `<details>` with `<summary>` holding the title as HTML |
| `@available` | `<ascribe-availability scope="section|block">` of `<ascribe-availability-target target dimension states versions>` |
| Project widget | An element named after the widget, `<ascribe-group widget>` around a group's arms |
| Glossary term | A link to the term's route, with its definition as the title |
| Raw HTML | Unchanged |

Elements and attributes are exactly `packages/elements/CONTRACT.md`'s, in its order, and a wrapping element has a blank line after its opening tag and before its closing tag (SPEC §9.4). `tests/all/site.rs`, `tests/all/site_quill.rs` (with the snapshots), and `tests/all/site_assets.rs` check them.

### `render_site_html`

`render_site_html(markdown) -> String` renders site markdown as HTML: comrak's CommonMark (the fork in `crates/comrak-ascribe`, with Ascribe's option off) with raw HTML passed through, GFM's tables, strikethrough, bare links, and task lists, and the site-render contract's markers applied (`render/`). It passes every fixture in `tests/render/` (`tests/all/render_fixtures.rs` compares parsed HTML with `html5ever`). The editor preview uses it; the Astro plugin must pass the same fixtures.

### Zod

`zod::generate(model)` writes the TypeScript module `_ascribe/schema.ts`: a `z.strictObject` per content type (imported from `astro/zod`), with the reserved `available` (the list of targets the site output writes) and `variant` keys (and `formatted`, for a type whose fields set `inline = "code"`), and the exports `<type>Schema`, `schemas`, and `schema`. A type the project declares has the reserved `type` too, as the literal of its name, and with several types `schema` is a `z.discriminatedUnion` on it. `tests/zod/` is a pnpm workspace package that type-checks the generated files with `tsc` under the workspace's strict settings and validates the Quill pages' frontmatter with them (`pnpm --filter @ascribed/zod-check test`). Regenerate its fixtures after a change with `ASCRIBE_BLESS=1 cargo test -p ascribe-emit --test all zod`.

## JSON

See the [JSON output contract](../../docs/content/contracts/json-output.md), which says what each page's document holds, and its schema, [`schemas/json-page.schema.json`](../../schemas/json-page.schema.json), generated from the types in `src/json.rs` by `crates/ascribe-cli/src/shapes.rs` (`ASCRIBE_BLESS=1 cargo test -p ascribe-cli shapes`). To change what a field says, change the type and its doc comment. `schema_version` is `JSON_SCHEMA_VERSION`, which changes only when a field is removed or changes meaning. Keys, and the names a field's values are, are snake_case. Beside the pages, `generated` adds `_ascribe/nav.json`, the site output's navigation with its formatted labels as pieces.

`tests/all/json_schema.rs` holds every page of every example and conformance case to the schema, and `tests/all/quill.rs` snapshots the full documents for Quill, in `tests/all/snapshots/`.

## Output ownership

See the [output-layout contract](https://ascribed-dev.com/go/output-layout-contract/). In short: each build's emitter writes to `<output-dir>/<build>/<emitter>/`, beside `<emitter>.manifest.json`, which lists every file Ascribe wrote. A build stages its files, checks that no file it doesn't own is in the way, records ownership, moves the files into place, removes the previous manifest's files it no longer produces, and writes the final manifest. It never deletes a file the manifest never listed.

## Tests

`tests/all/plain.rs` (each construct), `tests/all/reading.rs` (what `ascribe search` reads agrees with the plain output, for every example and conformance case), `tests/all/agents.rs` (the outputs for agents), `tests/all/store.rs` (the output-layout contract), `tests/all/write.rs` (`write_outputs`), `tests/all/assets.rs` (the output works with the source removed), `tests/all/formatted.rs` (fields read with `inline = "code"`), `tests/all/json_schema.rs` (the JSON output against its schema), and `tests/all/quill.rs` (`insta` snapshots of every page of `examples/quill` under each build, with both emitters), and, for the site output, `tests/all/site.rs`, `tests/all/site_quill.rs`, `tests/all/site_assets.rs`, `tests/all/site_anchors.rs` (source anchors), `tests/all/render_fixtures.rs`, and `tests/all/zod.rs`. Review snapshot changes with `cargo insta review`; never accept them blindly.
