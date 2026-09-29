# tessera-emit

Emitters for Tessera's outputs. An emitter renders phase 12's **resolved tree** (`tessera_resolve::ResolvedPage`) and never works out what a build mode keeps: a selection build that keeps several arms of a group emits them all, and a filter build emits the availability annotations still attached (SPEC §9.2).

| Output | Emitter | Phase |
|---|---|---|
| Plain markdown (`plain/`) | `PlainEmitter` | 18 |
| JSON (`json/`) | `JsonEmitter` | 18 |
| Site markdown plus web components (`site/`) | `SiteEmitter` | 20 |

## Pieces

- `Emitter` (`emitter.rs`): `name`, `page_path`, `render_page`, and the defaults `place_asset` (the mirrored path and a relative reference, asset contract §3), `generated` (files under `_tessera/`), and `warnings`. `emit(&dyn Emitter, &EmitContext, &ResolvedBuild) -> Emission` renders every page and lists each asset the pages use once.
- `assets.rs`: `mirrored_path`, `relative_reference`, `encode_path`, and `markdown_destination`, which write a reference so CommonMark reads it back as written (asset contract §4).
- `OutputDir` (`store.rs`): the [output-layout contract](../../project-docs/contracts/output-layout.md): the lock, staging, manifests, and replacing a previous output without touching a file the manifest didn't list.
- `labels.rs`: text the outputs share (the availability line and each target's text, an arm's label).
- `site/`, `render/`, `zod/`: phase 20 (below).

```rust,ignore
let cx = EmitContext::new(&project, project_root, build);
let emission = emit(&PlainEmitter, &cx, &resolved)?;
let output = OutputDir::lock(&project_root.join(".tessera/build"))?;
output.replace(&build.name, "plain", &emission.files)?;
```

## Plain markdown

Fully resolved CommonMark with no HTML. A page is its title as a level-1 heading (Q111), the page-level `Available:` line if it has one, then its blocks. SPEC §9.4's table, as implemented:

| Source | Output |
|---|---|
| `@note {type=tip}` with a title | `> **Tip: Title**`, a blank quoted line, then the content; without a title, `> **Tip**` |
| `@steps` | The ordered list |
| A group | One section per surviving arm: the arm's bold label, then its content (Q115) |
| `@details` | The title in bold, then the content |
| `@available` | `Available: Quill Cloud (GA); self-managed (preview, 3.4+)` (Q114) |
| A project widget | Its `plain-fallback` (phrases substituted), then its wrapped content unless `plain-content = "drop"` (Q113) |
| Phrases, includes, glossary links | Resolved |
| Links | Absolute URLs with the `[consumer] site` origin; root-relative, with a warning from `ascribe build`, without one |
| Images and linked files | Copied into the output at their mirrored path, and written as relative references |
| Raw HTML in the source | Its text, without the tags; comments, scripts, and styles dropped (Q112) |

Code blocks are always fenced (the fence is longer than any backtick run in the code, and a `phrases=true` info word is dropped), and text is escaped so that an unmodified CommonMark parser reads back the same text.

## Site

Markdown plus web components, for a consumer that renders CommonMark with raw HTML: spec 0.1's is Astro (`AstroProfile`, written against Astro 7.3). `SiteEmitter::new(model)` implements `Emitter`; `place_asset` follows the profile (images mirrored beside their page with a relative reference; other linked files under `_tessera/files/`, referenced by URL and listed with it in the manifest), and `generated` adds `_tessera/schema.ts`. `prepare` refuses a build whose pages share a route (Q143).

| Source | Output |
|---|---|
| Frontmatter | Passed through, phrases substituted; `available` becomes a list of targets (Q142) |
| Heading | An ATX heading ending in `<tessera-attributes id="…"></tessera-attributes>` with its page id, after a space |
| Image | `![alt](./path "title")`, then a marker with its attributes and the model's defaults (Q141) |
| `@note` | `<tessera-note type label heading>` wrapping the content |
| `@steps` | `<tessera-steps>` wrapping the list |
| `@variant` group | `<tessera-tabs sync>` of `<tessera-tab value label>` per surviving arm (a group reduced to one arm is its content) |
| `@details` | `<details>` with `<summary>` holding the title as HTML |
| `@available` | `<tessera-availability scope="section|block">` of `<tessera-availability-target target dimension states versions>` |
| Project widget | An element named after the widget, `<tessera-group widget>` around a group's arms |
| Glossary term | A link to the term's route, with its definition as the title |
| Raw HTML | Unchanged |

Elements and attributes are exactly `packages/elements/CONTRACT.md`'s, in its order, and a wrapping element has a blank line after its opening tag and before its closing tag (SPEC §9.4). `tests/site.rs`, `tests/site_quill.rs` (with the snapshots), and `tests/site_assets.rs` check them.

### `render_site_html`

`render_site_html(markdown) -> String` renders site markdown as HTML: comrak's CommonMark with raw HTML passed through, GFM's tables, strikethrough, bare links, and task lists, and the site-render contract's markers applied (`render/`). It passes every fixture in `tests/render/` (`tests/render_fixtures.rs` compares parsed HTML with `html5ever`). The editor preview (phase 25) uses it; the Astro plugin (phase 21) must pass the same fixtures.

### Zod

`zod::generate(model)` writes the TypeScript module `_tessera/schema.ts`: a `z.strictObject` per content type (imported from `astro/zod`), with the reserved `available` (the list of targets the site output writes) and `variant` keys, and the exports `<type>Schema`, `schemas`, `contentTypes`, and `schema` (Q149). `tests/zod/` is a pnpm workspace package that type-checks the generated files with `tsc` under the workspace's strict settings and validates the Quill pages' frontmatter with them (`pnpm --filter @tessera/zod-check test`). Regenerate its fixtures after a change with `TESSERA_BLESS=1 cargo test -p tessera-emit --test zod`.

## JSON

One document per page, at the page's source path with `.json` for `.md`. `schemaVersion` is `1`. It changes only when a field is removed or changes meaning; consumers ignore fields they don't know. Spans are UTF-8 byte offsets, `[start, end]`, from the start of the file (frontmatter included); lines count from 1. Every `file` is a content path, so a consumer can point back at the source.

### Page

| Field | Type | Meaning |
|---|---|---|
| `schemaVersion` | number | `1` |
| `format` | string | `"tessera-page"` |
| `build` | string | The build's name |
| `path` | string | The page's source path, relative to the content root |
| `route` | string | The page's root-relative URL, as the router made it |
| `site` | string or null | `[consumer] site`, the origin to put before a root-relative URL |
| `title` | string or null | The frontmatter `title`, phrases substituted |
| `frontmatter` | object or null | The frontmatter, with phrases substituted in the fields that ask for it |
| `availability` | object or null | The page's own availability (frontmatter `available`) |
| `headings` | array | Every surviving heading: `level`, `text`, `id` (its page id), `sourceId`, `explicit`, `source` |
| `assets` | array | Every surviving asset reference on the page: `source` (its source path), `path` (the copy's path in this output), `reference` (how this page refers to it), `kind` (`"image"` or `"link"`), `fragment`, `writtenIn` (the file the reference is written in), `span` |
| `blocks` | array | The page's blocks |

### Blocks

Every block has a `type`, and:

| Field | Meaning |
|---|---|
| `source` | `file`, `span`, `lines` (first and last), and `via`: the includes it came through, outermost first (`file`, `span`, `line`), absent for a page's own blocks |
| `availability` | Its effective availability below page level: the innermost spec, with `enclosing` chaining outward to the scope it sits in. Absent when nothing but the page's availability applies |
| `links` | The block's own links and images, resolved: `span`, `kind`, `destination`, `target` |
| `glossary` | Glossary terms linked in its prose: `term`, `text`, `url` |
| `substitutions` | Phrases replaced in its text: `span` (over the source), `key`, `value` |

`links`, `glossary`, and `substitutions` are absent when empty.

| `type` | Fields |
|---|---|
| `heading` | `level`, `id`, `sourceId`, `explicit`, `text`, `inlines` |
| `paragraph` | `inlines` |
| `code` | `fenced`, `info`, `literal` |
| `blockQuote` | `children` |
| `list` | `ordered`, `start`, `tight`, `items` (each `{children}`) |
| `html` | `literal` |
| `thematicBreak` | |
| `table` | `alignments`: each column's `none`, `left`, `center`, or `right`; `rows`: each `{header, cells}`, a cell being `{inlines}` |
| `directive` | A line-form directive that survived: `name`, `attributes`, `title`, `primary`, `binding` (`own`, `heading`, `followingBlock`, `unbound`), `annotation` |
| `container` | `name`, `attributes`, `title`, `children` |
| `group` | `name`, `arms`: each `{label, attributes, title, span, children}` |

An `@available` that survived the build is a `directive` with an `annotation`: `text` (the spec as shown; for a feature key, the spec it stands for), `display` (with the content model's labels), and `feature`. `attributes` is a list of `{key, values}` (a set has several values, a bare key none). A `title` and a text `primary` are inline lists. A `primary` is `{kind: "text", inlines}`, `{kind: "identifier", text}`, or `{kind: "line", text}`.

A group keeps its surviving arms only: a group a selection reduced to one arm isn't a group in the tree, but that arm's blocks, and one with none is gone. An arm's `label` is its title as plain text, or the display labels of the values it names.

### Inlines

Every inline has a `type` and a `span` in the block's file: `text` (`value`, decoded), `code`, `softBreak`, `hardBreak`, `html`, `emphasis` and `strong` (`children`), `link`, `image`, `phrase` (`key`, for a phrase the resolution left, which it doesn't).

A `link` has `destination` (a page link's is its route and id, an external one's is as written), `title`, `children`, and `target`: the block's resolved link at the same span, or `null` for a glossary link (an ordinary link, listed in the block's `glossary`). An `image` has `destination`, `title`, `alt` (plain text), `attributes` (`{key, values}`, such as `width`), and `target`.

A `target` has a `type`:

| `type` | Fields |
|---|---|
| `external` | |
| `page` | `page` (its source path), `id` (a page id, or null), `url`, `textFilled` (the link had no text and took the page's title) |
| `asset` | `source`, `path` (the copy), `reference` (as written on this page), `url`, `fragment` |
| `unresolved` | The build couldn't resolve it; the checks report why |

### Example

```json
{
  "schemaVersion": 1,
  "format": "tessera-page",
  "build": "cloud",
  "path": "keys.md",
  "route": "/keys/",
  "site": "https://docs.quill.dev",
  "title": "API keys",
  "headings": [{ "level": 2, "text": "Create a key", "id": "create-key", "sourceId": "create-key", "explicit": true, "source": { "file": "keys.md", "span": [93, 108], "lines": [5, 5] } }],
  "assets": [],
  "blocks": [{ "type": "heading", "level": 2, "id": "create-key", "…": "…" }]
}
```

The full documents for Quill are the snapshots in `tests/snapshots/`.

## Output ownership

See the [output-layout contract](../../project-docs/contracts/output-layout.md). In short: each build's emitter writes to `<output-dir>/<build>/<emitter>/`, beside `<emitter>.manifest.json`, which lists every file Tessera wrote. A build stages its files, checks that no file it doesn't own is in the way, records ownership, moves the files into place, removes the previous manifest's files it no longer produces, and writes the final manifest. It never deletes a file the manifest never listed.

## Tests

`tests/plain.rs` (each construct), `tests/store.rs` (the output-layout contract), `tests/assets.rs` (the output works with the source removed), and `tests/quill.rs` (`insta` snapshots of every page of `examples/quill` under each build, with both emitters), and, for the site output, `tests/site.rs`, `tests/site_quill.rs`, `tests/site_assets.rs`, `tests/render_fixtures.rs`, and `tests/zod.rs`. Review snapshot changes with `cargo insta review`; never accept them blindly.
