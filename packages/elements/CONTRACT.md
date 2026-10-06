# Element contract

The site output (SPEC §9.4) is markdown plus custom elements. This contract is the interface between the **site emitter**, which writes the elements, and the **element library**, `@ascribed/elements`, which implements them. For each element it gives the tag name, the attributes and what they mean, the expected children, and how it renders with and without JavaScript. The emitter writes exactly this markup; the library implements every element and attribute here, and none beyond them.

| Ascribe source | Site output | Library |
|---|---|---|
| `@note` | [`<ascribe-note>`](#1-ascribe-note) | CSS only |
| `@steps` | [`<ascribe-steps>`](#2-ascribe-steps) | CSS only |
| `@variant` group, several arms | [`<ascribe-tabs>`](#3-ascribe-tabs-and-ascribe-tab) with `<ascribe-tab>` | JavaScript (the only element that needs it, SPEC §9.7) |
| `@available`, badge or filter | [`<ascribe-availability>`](#4-ascribe-availability-and-ascribe-availability-target) with `<ascribe-availability-target>` | CSS only |
| `@details` | [`<details>`](#5-details) with `<summary>` | Optional CSS |
| Project widget | [An element named after the widget](#6-project-widgets), and `<ascribe-group>` for groups | The project's own; `<ascribe-group>` is the library's |
| Glossary term | [An ordinary link](#7-glossary-terms), marked with `data-ascribe-term` | Optional CSS |

A change to this contract changes the site emitter (`crates/tessera-emit`), the element library, and the site-render fixtures (`tests/render/`) together.

## 0. Rules for every element

**Light DOM, CSS, no behavior.** Elements render into the light DOM, so site styles apply and content stays visible to search engines and assistive technology (SPEC §9.7). They're styled with CSS and themed with custom properties whose names start with `--ascribe-`; the package README lists them. Only `<ascribe-tabs>` runs JavaScript. Everything else must render meaningfully with the library's CSS and no script, and must stay readable with neither.

**How the emitter writes elements.** Custom elements are raw HTML blocks in CommonMark, so the layout matters:

- An element that **wraps markdown** is written as its opening tag alone on a line, a blank line, the markdown, a blank line, and its closing tag alone on a line (SPEC §9.4). Nested wrapping elements follow the same rule.
- An element that holds **no markdown** (`<ascribe-availability>`, an empty widget) is written as one HTML block: its lines are consecutive, with no blank line inside.
- Every element is separated from the blocks around it by a blank line, or starts or ends its container.
- Inside a list item or blockquote, every line carries the container's indentation or `>` markers, like any other block there.
- A list whose items hold an element is written loose, whatever it was in the source, because an element needs a blank line before it and CommonMark then reads the list as loose: its items render `<p>`. A list whose items hold no element keeps its tightness.

**Attribute values** are written in double quotes, with `&`, `<`, `>`, and `"` escaped as `&amp;`, `&lt;`, `&gt;`, and `&quot;`. Attributes appear in the order this contract lists them. An optional attribute with no value is left out, never written empty.

**Plain text of inline content.** Where an attribute holds a title or label written as inline markdown, its value is the inline content's text: phrases substituted (they already are, in the resolved page), code spans as their text, emphasis and links as their text, images as their alt text, raw inline HTML dropped, and line breaks as single spaces. So ``.Try the *new* `quill` CLI`` becomes `Try the new quill CLI`.

**Labels** come from the content model: note types' `label` (docs/content/contracts/content-model.md §11), dimensions' and dimension values' `labels` (§7), and lifecycle states' `label` (§9). A value without a declared label is shown as itself.

## 1. `<ascribe-note>`

A callout (SPEC §4.5), from `@note` in all three forms.

| Attribute | Required | Meaning |
|---|---|---|
| `type` | yes | The note type, such as `tip`, or a type the project declares. Always written, including the default `note`. |
| `label` | yes | The note type's display label, such as `Tip`. |
| `heading` | no | The plain text of the note's title line (§0). It isn't called `title` because HTML's `title` attribute shows as a tooltip over the whole element. |

**Children:** the note's content, as markdown blocks. With a text primary (`@note: text`), a paragraph of that text; with no colon, the one block it binds; as a container, every block up to `@end`.

```html
<ascribe-note type="tip" label="Tip" heading="Try it without installing">

You can run Quill in the browser at play.quill.dev with no local setup.

</ascribe-note>
```

**Rendering, with or without JavaScript:** a block styled by `type`, with a heading line showing `label` followed by `: ` and `heading`, or `label` alone when there's no heading (so the type never depends on color), generated from the attributes by CSS (`::before` with `attr()`). Unknown types (a project's own) get the default note style, and are themed by selecting on `[type="…"]`. With no CSS, the content shows as ordinary blocks.

## 2. `<ascribe-steps>`

A procedure (SPEC §4.6).

**Attributes:** none.

**Children:** exactly one ordered list, the list `@steps` binds, with its items as the author wrote them (including a `start` number).

```html
<ascribe-steps>

1. Install the agent package.
2. Verify the install.

</ascribe-steps>
```

**Rendering, with or without JavaScript:** the list styled as numbered steps with CSS counters or list styling. With no CSS, an ordinary ordered list.

## 3. `<ascribe-tabs>` and `<ascribe-tab>`

A `@variant` group that keeps more than one arm in the build (SPEC §9.3, §9.4): every group under `switch`, and a group whose selection leaves several arms. A group reduced to one arm is written as that arm's content, with no element; a group with no arm left is removed.

### `<ascribe-tabs>`

| Attribute | Required | Meaning |
|---|---|---|
| `sync` | dimensional groups only | The dimension this group switches on. Groups with the same `sync` on a page switch together, and the reader's choice is remembered across pages. Labeled groups have no `sync` and never sync (SPEC §4.3: they're local to their page). |

**Choosing `sync`:** among the dimensions every surviving arm names, the first, in the content model's declaration order of `[dimensions]`, on which the arms' values differ; if the arms have the same values on every dimension they share, the first one they share. For the usual group, whose arms each name one dimension, it's that dimension.

**Children:** one `<ascribe-tab>` per surviving arm, in source order, and nothing else.

### `<ascribe-tab>`

| Attribute | Required | Meaning |
|---|---|---|
| `value` | dimensional arms only | The arm's values for the group's `sync` dimension, separated by single spaces: `npm`, or `npm pnpm` for an arm written `{pm=npm\|pnpm}`. Dimension values are names without spaces, so the list is unambiguous. |
| `label` | yes | What the tab is called. For a labeled arm, the plain text of its title (§0). For a dimensional arm, each of its attributes, in canonical order, as the labels of its values joined by ` / `, with attributes joined by `, `: `npm`, `Quill Cloud, npm`, `npm / pnpm`. |

**Children:** the arm's content, as markdown blocks.

````html
<ascribe-tabs sync="pm">

<ascribe-tab value="npm" label="npm">

```shell
npm install -g @quill/agent
```

</ascribe-tab>

<ascribe-tab value="pnpm" label="pnpm">

```shell
pnpm add -g @quill/agent
```

</ascribe-tab>

</ascribe-tabs>
````

### Rendering

**Without JavaScript** every tab shows, one after another, each introduced by its `label` (generated by CSS), so no content is hidden (SPEC §9.7).

**With JavaScript** the element follows the WAI-ARIA Authoring Practices tabs pattern:

- It inserts a tab list as its first child: an element with `role="tablist"` holding one `<button role="tab">` per tab, whose text is the tab's `label`. Each `<ascribe-tab>` gets `role="tabpanel"`, an `id` if it has none, and `aria-labelledby` pointing at its button; the buttons get `aria-controls`, `aria-selected`, and a roving `tabindex`.
- One tab shows at a time; the others get the `hidden` attribute.
- Left and Right arrows move between tabs and select them, wrapping at the ends; Home and End go to the first and last tab.
- **Initial selection:** for a group with `sync`, the first tab whose `value` contains the remembered value for that dimension; otherwise, and for labeled groups, the first tab.
- **Syncing:** selecting a tab in a group with `sync` records the first value in the tab's `value` as the choice for that dimension, then selects, in every `<ascribe-tabs>` on the page with the same `sync`, the first tab whose `value` contains it. A group with no matching tab keeps its selection.
- **Remembering:** the choice is stored in `localStorage` under the key `ascribe-tabs:<sync>`, such as `ascribe-tabs:pm`, and read when a group starts. Storage can be unavailable (private windows, blocked storage): every read and write is guarded, and without storage the choice lasts for the page only.

## 4. `<ascribe-availability>` and `<ascribe-availability-target>`

An availability annotation (SPEC §4.4, §9.4), in badge builds and, for content that survives, in filter builds (SPEC §9.3).

### `<ascribe-availability>`

| Attribute | Required | Meaning |
|---|---|---|
| `scope` | yes | What the annotation applies to: `section` (an `@available` at the top of a section), `block` (one bound to a block), `row` (a table row's `available`), or `page`. |

**Children:** one `<ascribe-availability-target>` per target of the effective spec (feature keys resolved), in the spec's order, separated by the text `; `. No markdown.

**Where it goes:**

- `section`: directly after the section's heading, as the next block.
- `block`: directly before the block it annotates, where the `@available` line was.
- `row`: at the end of the row's first cell, after a space (or alone in an empty cell), all on one line: inside a table cell the element has no line breaks.
- `page`: the emitter doesn't write one. Page-level availability reaches the layout as frontmatter (SPEC §9.4, §9.6): `available` is a list with one entry per target, in the spec's order and with feature keys resolved, each holding this element's attributes and its text: `{ target, dimension, states: [...], versions: [...], text }`, with `versions` left out as the attribute is. A layout that shows it renders this element with `scope="page"`, one `<ascribe-availability-target>` per entry, joining `states` and `versions` with single spaces.

### `<ascribe-availability-target>`

| Attribute | Required | Meaning |
|---|---|---|
| `target` | yes | The target as the spec names it: a dimension value (`cloud`) or a dimension name (`deployment`, meaning all its values). |
| `dimension` | yes | The dimension the target belongs to, or the target itself when it's a dimension name. |
| `states` | yes | The target's lifecycle states in order, separated by single spaces: `ga` for a bare target, `preview`, or `preview ga deprecated` for a history. |
| `versions` | when the spec gives versions | The version each state starts at, in the same order, separated by single spaces: `3.4`, or `3.3 3.5 4.0`. A bare version (`self-managed 3.3`) is `states="ga" versions="3.3"`. Left out for a versionless target, and for a target given without versions. |

**Text content:** what a reader sees, built from labels (§0):

| Spec for the target | Text |
|---|---|
| `cloud` | `Quill Cloud (GA)` |
| `self-managed preview 3.4` | `Self-managed (preview, 3.4+)` |
| `self-managed 3.3` | `Self-managed (GA, 3.3+)` |
| `self-managed (preview 3.3, ga 3.5, deprecated 4.0)` | `Self-managed (preview 3.3, GA 3.5, deprecated 4.0)` |

That is: the target's label, then in parentheses either one state's label, followed by `, <version>+` if it has a version, or, for a history, each state's label and version separated by `, `. The plain-markdown output's availability line uses the same text per target.

```html
<ascribe-availability scope="section">
<ascribe-availability-target target="cloud" dimension="deployment" states="ga">Quill Cloud (GA)</ascribe-availability-target>; <ascribe-availability-target target="self-managed" dimension="deployment" states="preview" versions="3.4">Self-managed (preview, 3.4+)</ascribe-availability-target>
</ascribe-availability>
```

**Rendering, with or without JavaScript:** a line of badges styled by each target's last state (`[states$="deprecated"]` and so on), with a lead-in such as "Available:" from CSS generated content, which a site can restyle or translate. With `scope="row"`, the badges sit inline in the cell, with no lead-in. With no CSS, the text reads as a sentence: `Quill Cloud (GA); Self-managed (preview, 3.4+)`.

## 5. `<details>`

`@details` compiles to HTML's own `<details>` element (SPEC §9.4), which works without JavaScript everywhere.

- `<details>` has no attributes: it starts closed.
- Its first child is `<summary>`, on the line after `<details>`, holding the title line's inline content **rendered as HTML** (not plain text): `<summary>Show the <code>quill.yaml</code> reference</summary>`. Because `<details>` starts an HTML block, the emitter renders the title's markdown itself. An image in the title is written as its alt text, since a raw `<img>` there would get no image processing and its relative path wouldn't resolve.
- Then a blank line, the content as markdown blocks, a blank line, and `</details>`.

```html
<details>
<summary>Show the full configuration reference</summary>

The content.

</details>
```

The element library may style `details` and `summary`, and implements no element for them.

## 6. Project widgets

A project widget (SPEC §6) becomes a custom element whose tag is the widget's name (SPEC §9.4). Widget names are valid custom-element names: they contain a hyphen, and docs/content/contracts/content-model.md §15 rejects names starting with `ascribe-` and HTML's reserved names. The project supplies the element's implementation; the element library doesn't.

**Attributes**, in this order:

1. `heading`: the plain text of the widget's title line, if it has one, named as on `<ascribe-note>`.
2. `primary`: an identifier primary, if the widget takes one and it's given. A text primary is content instead (below).
3. Each attribute in the widget's declaration, in declared order, with the value given or, if absent, its declared default. A value set's members are joined by single spaces; booleans are `true` or `false`; other values are their text. An attribute with neither a value nor a default is left out. A declaration can't use `heading`, `primary`, or HTML's global or event-handler attribute names (SPEC §7.2; content-model.md, `model-attribute-reserved`), so these never clash.

**Children and placement**, by the widget's form and binding (docs/content/contracts/content-model.md §15):

| Widget | Element |
|---|---|
| Container form | Wraps the container's blocks |
| Line form with a text primary given | Wraps one paragraph of the primary text |
| `binding = "block"`, or `"heading-or-block"` away from a section's top | Wraps the block it binds |
| `binding = "heading"`, or `"heading-or-block"` at a section's top | Empty, directly after the section's heading |
| `binding = "self"` | Empty, where the directive was |

A wrapping widget follows §0's layout for elements that wrap markdown; an empty one is a single line: `<quill-labspace lab="first-sync"></quill-labspace>`.

**Groups:** the arms of a groupable widget's group become one element per arm, as above, wrapped in `<ascribe-group widget="<name>">`, which follows §0's layout. `<ascribe-group>` is part of the element library: it has no behavior and no styling beyond `display: block`, and exists so a project's CSS or script can find a group's arms together.

## 7. Glossary terms

There's no element for glossary terms. In the site output, an occurrence the glossary links (docs/content/contracts/content-model.md §13) is an ordinary markdown link to the term's `link` target, as a route, with the term's definition as the link title, followed directly by an attribute marker (docs/content/contracts/site-render.md §2.3) that gives the `<a>` a `data-ascribe-term` attribute holding the term's id:

```markdown
[API key](/docs/reference/glossary/#api-key "A secret token that authenticates the Quill agent.")<ascribe-attributes data-ascribe-term="api-key"></ascribe-attributes>
```

```html
<a href="/docs/reference/glossary/#api-key" title="A secret token that authenticates the Quill agent." data-ascribe-term="api-key">API key</a>
```

Terms without a `link` stay plain text (content-model.md Q7).

**Rendering:** a link, which the library's CSS underlines with dots (`a[data-ascribe-term]`). A site restyles it, or shows the definition as a popover, by selecting on the attribute.
