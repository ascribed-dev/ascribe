---
title: Site-render contract
description: The markers the site output writes and how a consumer renders them.
---

The site output (SPEC §9.4) is CommonMark with raw HTML: custom elements for Ascribe's constructs, and ordinary markdown for everything else. A consumer renders most of it with its own markdown pipeline and needs nothing from Ascribe. Two things need more than CommonMark (SPEC §9.5):

- **Explicit heading ids.** A heading's id is its page id (SPEC §5.5), which Ascribe computes and validates. The consumer must use that id, not one of its own, and still treat the heading as a heading, so its table of contents and heading processing keep working.
- **Image attributes.** Attributes such as `width` (SPEC §5.3, content-model.md §14) must reach the `<img>`, while the image stays a markdown image, so the consumer's image processing (Astro's optimization) still applies.

This contract defines the one syntax the site output uses for both, the **attribute marker**, and exactly what HTML it must produce. It also defines **source anchors** (§7): in review mode, each block of the output says which source lines it came from. It has two implementations, which must agree:

| Implementation | Used by |
|---|---|
| The Astro markdown plugin in `@ascribed/astro` | The published site |
| `render_site_html()` in `tessera-emit` | The editor preview |

The shared fixtures in [`tests/render/`]({repo}/tree/main/tests/render/) are what keep them equal: both must pass every fixture.

Everything here applies to spec 0.1's only consumer profile, `astro`. It settles content-model.md §21, item 12, which is why `ascribe.toml` has no keys for heading ids or image attributes.

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

Every alternative is changed by Astro's markdown processing before a plugin can see it. Each of Astro's markdown processors (Sätteri, its default in Astro 7.3, and `unified()`) applies GFM and typographic replacements before a user plugin sees the tree, so an attribute block written as text, such as `## Setup {#setup}` or `![a](b.png){caption="x"}`, reaches the plugin with its quotes curled and its `--` and `...` turned into dashes and ellipses, and an id like `__init__` can become strong emphasis. A raw HTML node is never touched by either: its text is exactly what Ascribe wrote. The marker also degrades quietly. A consumer without the plugin renders it as an empty element: invisible, and for a heading, an element carrying the right `id` inside the heading, so links still land.

The other options considered, and why they lost, are in content-model.md §21, item 12.

## 2. Where a marker applies

A marker applies in exactly two positions. Anywhere else it's left alone, as raw HTML, and renders as an empty `ascribe-attributes` element.

### 2.1 At the end of a heading

A marker that is the **last inline content of a heading** (ATX or setext, at any level) applies to the heading. The marker, and any spaces, tabs, and line breaks directly before it, are removed. The heading's content is otherwise unchanged. A marker can be a heading's only content.

```markdown
## Run `ascribe check` in *CI* <ascribe-attributes id="run-ascribe-check-in-ci"></ascribe-attributes>
```

```html
<h2 id="run-ascribe-check-in-ci">Run <code>ascribe check</code> in <em>CI</em></h2>
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

A marker directly after an image that is also the last inline content of a heading applies to the image, not the heading: this rule names the position exactly, and the heading gets no id from it. The emitter never writes this, since its heading marker follows a space.

## 3. What applying a marker does

Each of the marker's attributes is set on the element it applies to (`<h1>`–`<h6>`, or `<img>`), with the same name and its decoded value. The emitter never writes a name the element already has: a heading's marker holds only `id`, and a content model can't declare an image attribute named `src`, `alt`, `title`, or any other name HTML gives a meaning (SPEC §7.2; content-model.md, `model-attribute-reserved`). If a marker written by hand does repeat one, the marker's value replaces the element's.

The resulting HTML is the CommonMark rendering of the input with the marker removed, plus those attributes. Nothing else changes: implementations don't renumber, deduplicate, or validate ids, since Ascribe has already assigned and checked them.

In Astro, "set on the element" means the attributes must reach the element before Astro's own processing of it: the heading's `id` before Astro's heading-id pass, which keeps an existing id and records it for the table of contents, and an image's attributes before Astro's image processing, which receives the `<img>`'s properties. The Astro plugin is tested against the Astro version it targets for both.

## 4. What the site emitter guarantees

The site emitter writes the site output so that the rules above are all a consumer needs:

- **Every heading ends in a marker with its page id**, one space after the heading's text. So the consumer's own slugger never runs on Ascribe content, and every heading id on the published page is one `ascribe check` validated, including ids numbered for duplicates and ids from `@id`. A heading whose page id is empty (a heading with no text) gets no marker.
- **Headings are ATX headings**, whatever the source used.
- **An image with attributes has a marker directly after it**, holding the image's attributes in canonical order (SPEC §8.3): every attribute the content model declares that the image writes or that has a default, then any it writes that the model doesn't declare. An image with neither attributes nor defaults has no marker. A value set's members are joined with single spaces (`platform=cloud|on-prem` becomes `platform="cloud on-prem"`); other values are their text (a quoted string without its quotes and escapes).
- **The emitter writes markers nowhere else.** Raw HTML an author writes passes through unchanged, as everywhere in the site output, so an author who writes an `ascribe-attributes` element gets its effect. Element names starting with `ascribe-` belong to Ascribe (content-model.md §15), so there's no reason to. Since the emitter's own heading marker is always last, an author's marker inside a heading never applies.

The element contract ([`packages/elements/CONTRACT.md`]({repo}/blob/main/packages/elements/CONTRACT.md)) covers the custom elements, which are ordinary raw HTML to a renderer.

## 5. Everything else

Apart from markers, both implementations render CommonMark with raw HTML passed through unchanged (SPEC §9.5, "HTML passthrough"). In particular, the site output's custom elements are HTML blocks, and the markdown they wrap is parsed as markdown because the emitter puts blank lines around it (SPEC §9.4). An implementation may also enable GFM and typographic replacements, as Astro does by default; the fixtures avoid content those would change, except inside markers, which they must not change.

## 6. Constructs and fixtures

Each construct below has at least one fixture in [`tests/render/`]({repo}/tree/main/tests/render/), listed in its `fixtures.toml`. [`tests/render/README.md`]({repo}/blob/main/tests/render/README.md) says how implementations run the fixtures and compare HTML.

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
| `image-ends-heading` | §2.1, §2.2 | A marker directly after an image that ends a heading applies to the image, not the heading |
| `attribute-values` | §1, §3 | Escaped quotes, ampersands, and angle brackets decode; spaces and commas stay |
| `not-a-marker` | §1, §2 | Markers after a space, mid-heading, or in a paragraph, and malformed markers, stay as raw HTML |
| `raw-html` | §5 | Custom elements wrapping markdown, `<details>`, and markers inside them |

## 7. Source anchors
@available: next

With anchors on (`ascribe build --emit site --anchors`, and always in the editor preview), every block of a rendered page carries the source file and lines it came from, so a reviewer's tools can point from the page back to the source. With anchors off, which is the default, the site output is byte for byte what it is without this section, and nothing here applies.

### 7.1 What an anchor says

A block's anchor is two HTML attributes on the block's own element:

```html
<p data-ascribe-source="guides/install.md:12-14">…</p>
<pre data-ascribe-source="_fragments/prereqs.md:3-9" data-ascribe-via="guides/install.md:20">…</pre>
```

- **A block's source** is `<path>:<first>-<last>`. `<path>` is the content path of the file the block's text is written in, with `/` between segments on every platform, and each segment percent-encoded except for ASCII letters, digits, `-`, `.`, `_`, and `~`. `<first>` and `<last>` are line numbers counted from 1, in decimal, with `<last>` never less than `<first>`. A one-line block still writes both: `guides/install.md:12-12`.
- **An include** is `<path>:<line>`: the file holding the `@include` and its line.
- `data-ascribe-source` holds the block's source. When the block came through includes, `data-ascribe-via` holds them, outermost first, separated by single spaces. A block written in the page itself has no `data-ascribe-via`.

To parse one, split at the last `:`; the path can't contain an unencoded `:`. These are the same facts as the JSON output's `source` (`crates/tessera-emit/README.md`), without byte spans.

### 7.2 Which blocks have one

Every block a reviewer could point at: headings, paragraphs, code blocks, lists and each list item, tables, block quotes, a paragraph holding only an image, thematic breaks, raw HTML blocks that start with an element, and each element the emitter writes (`ascribe-note`, `ascribe-steps`, `ascribe-tabs` and each `ascribe-tab`, `details`, `ascribe-availability`, project widgets, and `ascribe-group` and each element in it). Blocks inside containers have their own.

- An element that wraps the block after it (a line-form `@note`, `@steps`, `@details`, or widget) spans the directive and that block. A line-form directive whose text is its content (`@note: Text.`) gives that text's paragraph its own line too.
- A paragraph in a tight list has no element (CommonMark renders its text straight into the `<li>`), so it has no anchor: its item's anchor says where it is.
- A raw HTML block that doesn't start with an open tag (one that starts with a comment, a closing tag, or text) has no anchor.

### 7.3 How an anchor gets through the markdown pipeline

The site output is markdown the consumer renders, and §1's reasoning holds: only raw HTML reaches a plugin unchanged. So the emitter writes each anchor in one of two ways.

**On an element it writes itself**, as attributes of the element's opening tag, after its other attributes:

```html
<ascribe-note type="tip" label="Tip" data-ascribe-source="guides/install.md:9-11">
```

Raw HTML passes through every renderer unchanged, so nothing else is needed.

**Before a block the markdown pipeline renders**, as an **anchor comment**: an HTML comment on a line of its own directly before the block, at the block's indentation (inside the list item or block quote that holds it):

```markdown
<!--ascribe-anchor tag="p" source="guides/install.md:12-14"-->
Run the installer, then
restart your shell.

<!--ascribe-anchor tag="ul" source="guides/install.md:16-18" items="16-16 17-18"-->
- Linux
- macOS, with
  Homebrew
```

Exactly this form is an anchor comment: `<!--ascribe-anchor`, then one or more attributes written as a marker's are (§1: one space, a name, `="`, a value, `"`), then directly `-->`. Values are escaped as a marker's are. The attributes:

| Attribute | Meaning |
|---|---|
| `tag` | The element the block renders as: `h1`–`h6`, `p`, `pre`, `ul`, `ol`, `table`, `blockquote`, `hr`, or, for a raw HTML block, the name of the element it starts with, lowercased. Required. |
| `source` | The block's source (§7.1). Required. |
| `via` | The includes it came through (§7.1). Absent when there are none. |
| `items` | For a list, each item's lines, `<first>-<last>`, in order, separated by single spaces. An item's file and includes are its list's. |

Why a comment: CommonMark makes an HTML comment an HTML block of its own (type 2) that ends on the line holding `-->`, and lets it interrupt a paragraph. So the comment never joins the block before or after it, the block after it starts where it would have, and a list whose blocks it sits between doesn't become loose, since there's no blank line. Both of Astro's markdown processors hand a plugin the comment as a `raw` node that is a sibling of the block's element. A consumer without the plugin renders an HTML comment: invisible.

### 7.4 Applying an anchor comment

A renderer applies each anchor comment, and removes it, along with the whitespace directly after it (a line ending, or the whitespace-only text node that holds it):

1. The **target** is what follows the comment after only whitespace: its next sibling, skipping whitespace-only text. When that is an element named `tag`, or raw HTML whose text starts with an open tag named `tag` (ASCII case-insensitive), it gets `data-ascribe-source` with the `source` value, and `data-ascribe-via` with the `via` value when there is one. In raw HTML, the attributes are written into its first tag, directly after the tag's name.
2. When the target is a `ul` or `ol` and the comment has `items`, and the list has as many `li` children as `items` has ranges, each `li` gets `data-ascribe-source` with the list's path and its range, and the list's `data-ascribe-via`. Otherwise its items get none.
3. Anything else, such as text, an element of another name, or no next sibling, is no target: the comment is removed and applies to nothing.

An attribute the target already has is replaced. Implementations apply anchors after the consumer's syntax highlighting (which keeps a code block's `<pre>`) and before anything that reads an element's properties.

### 7.5 The same page

With anchors on, each renderer's HTML equals its HTML with anchors off, once the `data-ascribe-source` and `data-ascribe-via` attributes are removed and compared as `tests/render/README.md` says: the same elements, nesting, and text, no list turned loose, and no paragraph added or lost. The emitter writes everything else exactly as it does with anchors off.

### 7.6 Fixtures

The anchor fixtures in [`tests/render/`]({repo}/tree/main/tests/render/) are written by the site emitter from a source in each fixture's `source/`, with anchors (`input.md`) and without (`unanchored.md`). Both implementations render `input.md` to `expected.html`, and check §7.5 on each fixture and on the pages of `examples/quill` in `tests/render/corpus/`.

| Construct | Rule | What the fixtures check |
|---|---|---|
| `anchor-markdown-blocks` | §7.3, §7.4 | Headings, paragraphs, code, a table, a quote, an image's paragraph, and a break get the anchor their comment names |
| `anchor-list-items` | §7.4 | A list's comment gives each item its lines |
| `anchor-tight-list` | §7.2 | A tight list's text has no paragraph and no anchor; a code block or list under it has one, and the list stays tight |
| `anchor-raw-html` | §7.4 | A raw HTML block gets its anchor in its first tag |
| `anchor-elements` | §7.3 | Elements the emitter writes carry their anchor on their own tag |
| `anchor-nesting` | §7.2 | Blocks in containers in containers each have their own anchor |
| `anchor-via` | §7.1 | A block from a fragment, and from a fragment in a fragment, lists its includes outermost first |
