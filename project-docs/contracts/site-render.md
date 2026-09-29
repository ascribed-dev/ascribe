# Site-render contract

The site output (SPEC §9.4) is CommonMark with raw HTML: custom elements for Tessera's constructs, and ordinary markdown for everything else. A consumer renders most of it with its own markdown pipeline and needs nothing from Tessera. Two things need more than CommonMark (SPEC §9.5):

- **Explicit heading ids.** A heading's id is its page id (SPEC §5.5), which Tessera computes and validates. The consumer must use that id, not one of its own, and still treat the heading as a heading, so its table of contents and heading processing keep working.
- **Image attributes.** Attributes such as `width` (SPEC §5.3, content-model.md §14) must reach the `<img>`, while the image stays a markdown image, so the consumer's image processing (Astro's optimization) still applies.

This contract defines the one syntax the site output uses for both, the **attribute marker**, and exactly what HTML it must produce. It has two implementations, which must agree:

| Implementation | Phase | Used by |
|---|---|---|
| The Astro markdown plugin in `@ascribed/astro` | 21 | The published site |
| `render_site_html()` in `tessera-emit` | 20 | The editor preview (phase 25) |

The shared fixtures in [`tests/render/`](../../tests/render/) are what keep them equal: both must pass every fixture.

Everything here applies to spec 0.1's only consumer profile, `astro`. It settles content-model.md Q12, which is why `ascribe.toml` has no keys for heading ids or image attributes.

## 1. The attribute marker

A marker is raw inline HTML: an empty `ascribe-attributes` element.

```html
<ascribe-attributes id="streaming-sync"></ascribe-attributes>
<ascribe-attributes width="600" loading="lazy"></ascribe-attributes>
```

Exactly this form, and nothing else, is a marker:

1. The open tag: `<ascribe-attributes`, then zero or more attributes, then `>`. Each attribute is one space, a name, `="`, a value, and `"`. A name is a lowercase ASCII letter followed by lowercase ASCII letters, digits, or hyphens (SPEC Appendix A `key`). A value is any text without `"` or a line ending.
2. Directly after it, with nothing between, the close tag `</ascribe-attributes>`.

CommonMark parses the two tags as two adjacent raw inline HTML nodes. (An implementation may see them as one node or two; the rule is about the source text they cover.) Anything that differs, such as single quotes, an unquoted value, a space before `>`, text between the tags, or a missing close tag, is not a marker.

**Values.** In a value, `&quot;`, `&amp;`, `&lt;`, and `&gt;` stand for `"`, `&`, `<`, and `>`. Any other `&` is a literal `&`. The emitter writes exactly those four escapes, and escapes every `"`, `&`, `<`, and `>` in a value.

### Why a marker

Every alternative is changed by Astro's markdown processing before a plugin can see it. Each of Astro's markdown processors (Sätteri, its default in Astro 7.3, and `unified()`) applies GFM and typographic replacements before a user plugin sees the tree, so an attribute block written as text, such as `## Setup {#setup}` or `![a](b.png){caption="x"}`, reaches the plugin with its quotes curled and its `--` and `...` turned into dashes and ellipses, and an id like `__init__` can become strong emphasis. A raw HTML node is never touched by either: its text is exactly what Tessera wrote. The marker also degrades quietly. A consumer without the plugin renders it as an empty element: invisible, and for a heading, an element carrying the right `id` inside the heading, so links still land.

The other options considered, and why they lost, are in content-model.md §21, Q12.

## 2. Where a marker applies

A marker applies in exactly two positions. Anywhere else it's left alone, as raw HTML, and renders as an empty `ascribe-attributes` element.

### 2.1 At the end of a heading

A marker that is the **last inline content of a heading** (ATX or setext, at any level) applies to the heading. The marker, and any spaces, tabs, and line breaks directly before it, are removed. The heading's content is otherwise unchanged. A marker can be a heading's only content.

```markdown
## Run `tessera check` in *CI* <ascribe-attributes id="run-tessera-check-in-ci"></ascribe-attributes>
```

```html
<h2 id="run-tessera-check-in-ci">Run <code>tessera check</code> in <em>CI</em></h2>
```

"Last inline content" is after CommonMark has removed an ATX heading's closing sequence and trailing whitespace, so `## Title <marker> ##` qualifies. Only the last marker applies; an earlier one in the same heading stays as raw HTML.

### 2.2 Directly after an image

A marker that immediately follows an image, with nothing between the image's closing `)` or `]` and the marker's `<`, applies to the image. This holds for every CommonMark image form: inline `![alt](src)`, full reference `![alt][ref]`, collapsed `![alt][]`, and shortcut `![alt]`, and for an image anywhere inline content can have one, including inside link text. The marker is removed.

```markdown
![The Quill settings page](./settings.png)<ascribe-attributes width="600"></ascribe-attributes>
```

```html
<img src="./settings.png" alt="The Quill settings page" width="600" />
```

A marker after a space, or after anything but an image, doesn't apply.

A marker directly after an image that is also the last inline content of a heading applies to the image, not the heading: this rule names the position exactly, and the heading gets no id from it (Q145). The emitter never writes this, since its heading marker follows a space.

## 3. What applying a marker does

Each of the marker's attributes is set on the element it applies to (`<h1>`–`<h6>`, or `<img>`), with the same name and its decoded value. The emitter never writes a name the element already has: a heading's marker holds only `id`, and a content model can't declare an image attribute named `src`, `alt`, `title`, or any other name HTML gives a meaning (SPEC §7.2; content-model.md, `model-attribute-reserved`). If a marker written by hand does repeat one, the marker's value replaces the element's.

The resulting HTML is the CommonMark rendering of the input with the marker removed, plus those attributes. Nothing else changes: implementations don't renumber, deduplicate, or validate ids, since Tessera has already assigned and checked them.

In Astro, "set on the element" means the attributes must reach the element before Astro's own processing of it: the heading's `id` before Astro's heading-id pass, which keeps an existing id and records it for the table of contents, and an image's attributes before Astro's image processing, which receives the `<img>`'s properties. Phase 21 verifies both against the Astro version it targets, and records the plugin's position in the pipeline.

## 4. What the site emitter guarantees

Phase 20's emitter writes the site output so that the rules above are all a consumer needs:

- **Every heading ends in a marker with its page id**, one space after the heading's text. So the consumer's own slugger never runs on Tessera content, and every heading id on the published page is one `tessera check` validated, including ids numbered for duplicates and ids from `@id`. A heading whose page id is empty (a heading with no text) gets no marker.
- **Headings are ATX headings**, whatever the source used.
- **An image with attributes has a marker directly after it**, holding the image's attributes in canonical order (SPEC §8.3): every attribute the content model declares that the image writes or that has a default, then any it writes that the model doesn't declare (Q141). An image with neither attributes nor defaults has no marker. A value set's members are joined with single spaces (`platform=cloud|on-prem` becomes `platform="cloud on-prem"`); other values are their text (a quoted string without its quotes and escapes).
- **The emitter writes markers nowhere else.** Raw HTML an author writes passes through unchanged, as everywhere in the site output, so an author who writes a `ascribe-attributes` element gets its effect. Element names starting with `tessera-` belong to Tessera (content-model.md §15), so there's no reason to. Since the emitter's own heading marker is always last, an author's marker inside a heading never applies.

The element contract ([`packages/elements/CONTRACT.md`](../../packages/elements/CONTRACT.md)) covers the custom elements, which are ordinary raw HTML to a renderer.

## 5. Everything else

Apart from markers, both implementations render CommonMark with raw HTML passed through unchanged (SPEC §9.5, "HTML passthrough"). In particular, the site output's custom elements are HTML blocks, and the markdown they wrap is parsed as markdown because the emitter puts blank lines around it (SPEC §9.4). An implementation may also enable GFM and typographic replacements, as Astro does by default; the fixtures avoid content those would change, except inside markers, which they must not change.

## 6. Constructs and fixtures

Each construct below has at least one fixture in [`tests/render/`](../../tests/render/), listed in its `fixtures.toml`. [`tests/render/README.md`](../../tests/render/README.md) says how implementations run the fixtures and compare HTML.

| Construct | Rule | What the fixtures check |
|---|---|---|
| `heading` | §2.1 | A marker ending an ATX heading gives it an id; the space before the marker goes |
| `heading-markup` | §2.1 | The marker follows code, emphasis, or a link, which stay as they are |
| `heading-setext` | §2.1 | Setext headings work the same way |
| `heading-closed` | §2.1 | An ATX closing sequence after the marker doesn't matter |
| `heading-empty` | §2.1 | A marker as a heading's only content |
| `heading-duplicates` | §3 | Headings with the same text keep the numbered ids they're given |
| `heading-protected` | §1 | Ids that inline parsing or typography would change (`__init__`, `--`, `...`) come through exactly |
| `image-inline` | §2.2 | `![alt](src)` followed by a marker |
| `image-title` | §2.2, §3 | The image's title stays next to the marker's attributes |
| `image-reference-full` | §2.2 | `![alt][ref]` followed by a marker |
| `image-reference-collapsed` | §2.2 | `![alt][]` followed by a marker |
| `image-reference-shortcut` | §2.2 | `![alt]` followed by a marker |
| `image-in-text` | §2.2 | Several images in one paragraph, with text directly after a marker |
| `image-in-link` | §2.2 | An image inside link text |
| `image-ends-heading` | §2.1, §2.2 | A marker directly after an image that ends a heading applies to the image, not the heading (Q145) |
| `attribute-values` | §1, §3 | Escaped quotes, ampersands, and angle brackets decode; spaces and commas stay |
| `not-a-marker` | §1, §2 | Markers after a space, mid-heading, or in a paragraph, and malformed markers, stay as raw HTML |
| `raw-html` | §5 | Custom elements wrapping markdown, `<details>`, and markers inside them |
