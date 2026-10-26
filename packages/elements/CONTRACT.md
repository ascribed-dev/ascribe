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
| The pointer to `llms.txt`, with `[consumer] agents = true` | [`<ascribe-for-agents>`](#8-ascribe-for-agents) | CSS only |
| A tested example's `@snippet` | [A caption line on its code block](#9-a-tested-code-block), with the class `ascribe-code-tested` | CSS only |
| A link out of a line of a versioned build | [An ordinary link](#10-a-link-out-of-a-line), marked with `data-ascribe-release-line` | Optional CSS |
| `@operation` | [`<ascribe-operation>`](#11-ascribe-operation) | CSS only |
| A code block | [A copy button after its `<pre>`](#12-a-code-blocks-copy-button), which the script adds | JavaScript and CSS |

A change to this contract changes the site emitter (`crates/ascribe-emit`), the element library, and the site-render fixtures (`tests/render/`) together.

## 0. Rules for every element

**Light DOM, CSS, no behavior.** Elements render into the light DOM, so site styles apply and content stays visible to search engines and assistive technology (SPEC §9.7). They're styled with CSS and themed with custom properties whose names start with `--ascribe-`; the package README lists them. The names are part of this contract; their default values aren't, and a release may change them, noting it in the changelog. Only `<ascribe-tabs>`, and the copy button the script adds to code blocks (§12), run JavaScript. Everything else must render meaningfully with the library's CSS and no script, and must stay readable with neither.

**How the emitter writes elements.** Custom elements are raw HTML blocks in CommonMark, so the layout matters:

- An element that **wraps markdown** is written as its opening tag alone on a line, a blank line, the markdown, a blank line, and its closing tag alone on a line (SPEC §9.4). Nested wrapping elements follow the same rule.
- An element that holds **no markdown** (`<ascribe-availability>`, an empty widget) is written as one HTML block: its lines are consecutive, with no blank line inside, and its opening tag is alone on the first line. An element with nothing inside is its opening tag on one line and its closing tag on the next. A custom element's tags on one line aren't an HTML block to CommonMark, which reads them as inline HTML and wraps them in a paragraph; an opening tag alone on its line starts a block (CommonMark's HTML block kind 7).
- Every element is separated from the blocks around it by a blank line, or starts or ends its container.
- Inside a list item or blockquote, every line carries the container's indentation or `>` markers, like any other block there.
- A list whose items hold an element is written loose, whatever it was in the source, because an element needs a blank line before it and CommonMark then reads the list as loose: its items render `<p>`. A list whose items hold no element keeps its tightness.

**Attribute values** are written in double quotes, with `&`, `<`, `>`, and `"` escaped as `&amp;`, `&lt;`, `&gt;`, and `&quot;`. Attributes appear in the order this contract lists them. An optional attribute with no value is left out, never written empty.

**Plain text of inline content.** Where an attribute holds a title or label written as inline markdown, its value is the inline content's text: phrases substituted (they already are, in the resolved page), code spans as their text, emphasis and links as their text, images as their alt text, raw inline HTML dropped, and line breaks as single spaces. So ``.Try the *new* `quill` CLI`` becomes `Try the new quill CLI`.

**Labels** come from the content model: note types' `label` (docs/content/contracts/content-model.md §10), dimensions' and dimension values' `labels` (§6), and lifecycle states' `label` (§8). A value without a declared label is shown as itself.

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

**Choosing `sync`:** among the dimensions every surviving arm names, the first, in the content model's declaration order of `[dimensions]`, then the implicit `release` dimension, on which the arms' values differ; if the arms have the same values on every dimension they share, the first one they share. For the usual group, whose arms each name one dimension, it's that dimension.

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
- **Choosing from a script:** the script exports `choose(dimension, value)`, which does what selecting a tab whose first value is `value`, in a group whose `sync` is `dimension`, does: it records the choice and selects it in every group on the page with that `sync`, and a group with no matching tab keeps its selection. It doesn't check the value. A page with no groups can call it before sending the reader on, and the next page's groups start on the choice, as a permalink's address does with its parameters.

## 4. `<ascribe-availability>` and `<ascribe-availability-target>`

An availability annotation (SPEC §4.4, §9.4), in badge builds and, for content that survives, in filter builds (SPEC §9.3).

### `<ascribe-availability>`

| Attribute | Required | Meaning |
|---|---|---|
| `scope` | yes | What the annotation applies to: `section` (an `@available` at the top of a section), `block` (one bound to a block), `row` (a table row's `available`), or `page`. |

**Children:** one `<ascribe-availability-target>` per target of the effective spec (feature keys resolved), in the spec's order, separated by the text `; `. No markdown. With `scope="row"`, the targets are between `<span class="ascribe-availability-lead-in">(Available: </span>` and `<span class="ascribe-availability-lead-in">)</span>`, so the cell's text is the plain-markdown output's, `(Available: …)` (decision [#430](../../project-docs/decisions/430-row-availability-lead-in.md)).

**Where it goes:**

- `section`: directly after the section's heading, as the next block.
- `block`: directly before the block it annotates, where the `@available` line was.
- `row`: at the end of the row's first cell, after a space (or alone in an empty cell), all on one line: inside a table cell the element has no line breaks.
- `page`: the emitter doesn't write one. Page-level availability reaches the layout as frontmatter (SPEC §9.4, §9.6): `available` is a list with one entry per target, in the spec's order and with feature keys resolved, each holding this element's attributes and its text: `{ target, dimension, states: [...], versions: [...], planned: true, text }`, with `versions` and `planned` left out as the attributes are. A layout that shows it renders this element with `scope="page"`, one `<ascribe-availability-target>` per entry, joining `states` and `versions` with single spaces, and with `planned` when the entry's is `true`.

### `<ascribe-availability-target>`

| Attribute | Required | Meaning |
|---|---|---|
| `target` | yes | The target as the spec names it: a dimension value (`cloud`) or a dimension name (`deployment`, meaning all its values). |
| `dimension` | yes | The dimension the target belongs to, or the target itself when it's a dimension name. |
| `states` | yes | The target's lifecycle states in order, separated by single spaces: `ga` for a bare target, `preview`, or `preview ga deprecated` for a history. |
| `versions` | when the spec gives versions | The version each state starts at, in the same order, separated by single spaces: `3.4`, or `3.3 3.5 4.0`. A bare version (`self-managed 3.3`) is `states="ga" versions="3.3"`. Left out for a versionless target, and for a target given without versions. |
| `planned` | when the last version is planned | Present, with no value, when the last version is in a line the content model's `[releases.<target>]` lists as planned: it hasn't shipped. Left out otherwise. |

**Text content:** what a reader sees, built from labels (§0):

| Spec for the target | Text |
|---|---|
| `cloud` | `Quill Cloud (GA)` |
| `self-managed preview 3.4` | `Self-managed (preview, 3.4+)` |
| `self-managed 3.3` | `Self-managed (GA, 3.3+)` |
| `self-managed (preview 3.3, ga 3.5, deprecated 4.0)` | `Self-managed (preview 3.3, GA 3.5, deprecated 4.0)` |
| `self-managed 3.6`, with 3.6 planned | `Self-managed (planned, 3.6)` |
| `self-managed (preview 3.5, ga 3.6)`, with 3.6 planned | `Self-managed (preview 3.5, planned 3.6)` |

That is: the target's label, then in parentheses either one state's label, followed by `, <version>+` if it has a version, or, for a history, each state's label and version separated by `, `. When the target is `planned`, the last version's state label is the content model's planned label (`[releases] planned-label`, `planned` by default), and a single version has no `+`. The plain-markdown output's availability line uses the same text per target.

```html
<ascribe-availability scope="section">
<ascribe-availability-target target="cloud" dimension="deployment" states="ga">Quill Cloud (GA)</ascribe-availability-target>; <ascribe-availability-target target="self-managed" dimension="deployment" states="preview" versions="3.4">Self-managed (preview, 3.4+)</ascribe-availability-target>
</ascribe-availability>
```

**Rendering, with or without JavaScript:** a line of badges styled by each target's last state (`[states$="deprecated"]` and so on), or as planned (`[planned]`) when it is, with a lead-in such as "Available:" from CSS generated content, which a site can restyle or translate. With `scope="row"`, the badges sit inline in the cell, with no lead-in in sight: the `ascribe-availability-lead-in` spans are visually hidden, and a screen reader reads them. With no CSS, the text reads as a sentence: `Quill Cloud (GA); Self-managed (preview, 3.4+)`, and a row's as `(Available: Quill Cloud (GA))`.

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

A project widget (SPEC §6) becomes a custom element whose tag is the widget's name (SPEC §9.4). Widget names are valid custom-element names: they contain a hyphen, and docs/content/contracts/content-model.md §14 rejects names starting with `ascribe-` and HTML's reserved names. The project supplies the element's implementation; the element library doesn't.

**Attributes**, in this order:

1. `heading`: the plain text of the widget's title line, if it has one, named as on `<ascribe-note>`.
2. `primary`: an identifier primary, if the widget takes one and it's given. A text primary is content instead (below).
3. Each attribute in the widget's declaration, in declared order, with the value given or, if absent, its declared default. A value set's members are joined by single spaces; booleans are `true` or `false`; other values are their text. An attribute with neither a value nor a default is left out. A declaration can't use `heading`, `primary`, or HTML's global or event-handler attribute names (SPEC §7.2; content-model.md, `model-attribute-reserved`), so these never clash.

**Children and placement**, by the widget's form and binding (docs/content/contracts/content-model.md §14):

| Widget | Element |
|---|---|
| Container form | Wraps the container's blocks |
| Line form with a text primary given | Wraps one paragraph of the primary text |
| `binding = "block"`, or `"heading-or-block"` away from a section's top | Wraps the block it binds |
| `binding = "heading"`, or `"heading-or-block"` at a section's top | Empty, directly after the section's heading |
| `binding = "self"` | Empty, where the directive was |

A wrapping widget follows §0's layout for elements that wrap markdown; an empty one follows its layout for elements with nothing inside, the opening tag on one line and the closing tag on the next, so it's an HTML block, never in a paragraph:

```html
<quill-labspace lab="first-sync">
</quill-labspace>
```

**Groups:** the arms of a groupable widget's group become one element per arm, as above, wrapped in `<ascribe-group widget="<name>">`, which follows §0's layout. `<ascribe-group>` is part of the element library: it has no behavior and no styling beyond `display: block`, and exists so a project's CSS or script can find a group's arms together.

## 7. Glossary terms

There's no element for glossary terms. In the site output, an occurrence the glossary links (docs/content/contracts/content-model.md §12), and a link an author writes to a term's heading, is an ordinary markdown link to the term's heading on its glossary page, as a route, with the term's definition as the link title, followed directly by an attribute marker (docs/content/contracts/site-render.md §2.3) that gives the `<a>` a `data-ascribe-term` attribute holding the term's id:

```markdown
[API key](/docs/reference/glossary/#api-key "A secret token that authenticates the Quill agent.")<ascribe-attributes data-ascribe-term="api-key"></ascribe-attributes>
```

```html
<a href="/docs/reference/glossary/#api-key" title="A secret token that authenticates the Quill agent." data-ascribe-term="api-key">API key</a>
```

**Rendering:** a link, which the library's CSS underlines with dots (`a[data-ascribe-term]`). A site restyles it, or shows the definition as a popover, by selecting on the attribute.

## 8. `<ascribe-for-agents>`

With `[consumer] agents = true` (docs/content/contracts/content-model.md §15), each page of the site output opens with a pointer for AI agents: where the index of every page, `llms.txt`, is, and where this page's Markdown version is (SPEC §9.4). It has no attributes, and wraps one paragraph of markdown with two links, both root-relative like every link in the site output:

```markdown
<ascribe-for-agents>

For AI agents: the documentation index is at [llms.txt](/docs/llms.txt), and this page is available as [Markdown](/docs/guides/my-setup.md).

</ascribe-for-agents>
```

It's the first block of the page's body, before any other content, so an agent that reads only the start of a page still finds it. Without `agents = true` the emitter writes no `<ascribe-for-agents>`.

**Rendering:** visually hidden: in the page, and read by an agent or a screen reader, but out of sight, as a site's "skip to content" link is (a clipped 1-pixel box, not `display: none`, which some readers skip). A site that wants readers to see it too restyles it.

## 9. A tested code block

There's no element for a tested example either. A `@snippet` of a file that's a tested example with a current observation (docs/content/contracts/tested-examples.md) becomes a fenced code block whose info string carries the environment it was tested with, after any title: `` ```ts title="connect.ts" tested="Node 24.9.0 and TypeScript 5.9.2 on Linux" `` (docs/content/contracts/site-render.md §5.1). The value is the environment alone, without the words around it (decision [#458b](../../project-docs/decisions/458b-tested-attribute.md)).

The renderer, `@ascribed/astro`'s code highlighting or `render_site_html`, puts the block in a figure with a caption: the title, when there is one, then a line of its own holding the environment, with the class `ascribe-code-tested`:

```html
<figure class="code-title">
  <figcaption>connect.ts<div class="ascribe-code-tested">Node 24.9.0 and TypeScript 5.9.2 on Linux</div></figcaption>
  <pre>…</pre>
</figure>
```

A block with a tested line and no title has the line alone in its caption. A block with a title and no tested line is as it was: its caption is the title.

**Rendering:** the line reads "Tested with Node 24.9.0 and TypeScript 5.9.2 on Linux". The lead-in, "Tested with ", is generated content from the library's stylesheet (`.ascribe-code-tested::before`), as "Available:" is, so a site restyles or translates it once. With no CSS, the caption still reads as a fact about the block. The HTML holds no hidden copy of the lead-in (decision [#459c](../../project-docs/decisions/459c-tested-lead-in.md)). The line is a block of its own, so its text is the plain output's sentence without the lead-in, and a check that each line of the page's HTML appears in its Markdown version finds it there, inside "Tested with … .".

## 10. A link out of a line

There's no element for a link out of a line either. In a line of a versioned build, a link to a page the line doesn't publish can lead to the page in another line (SPEC §5.2). In the site output it's an ordinary markdown link to the page's route in that line, followed directly by an attribute marker (docs/content/contracts/site-render.md §2.3) that gives the `<a>` a `data-ascribe-release-line` attribute holding the line, as `[releases.<target>]` writes it ([decision #502d](../../project-docs/decisions/502d-links-between-lines.md)). Its text is what the author wrote, or the target's title in that line:

```markdown
[Configure tracing](/docs/configure-tracing/)<ascribe-attributes data-ascribe-release-line="3.5"></ascribe-attributes>
```

```html
<a href="/docs/configure-tracing/" data-ascribe-release-line="3.5">Configure tracing</a>
```

**Rendering:** the link, followed by its line in parentheses, " (3.5)", generated content from the library's stylesheet (`a[data-ascribe-release-line]::after`), as "Available:" is, so a site restyles, rewords, or hides it by selecting on the attribute. With no CSS, it's a plain link.

## 11. `<ascribe-operation>`

An operation of a declared API (SPEC §4.13, §7.11), from an `@operation` whose link names an operation the spec has ([decision #923b](../../project-docs/decisions/923b-operation-element.md)). An `@operation` that names none is a problem `ascribe check` reports, and the emitter writes nothing for it.

| Attribute | Required | Meaning |
|---|---|---|
| `api` | yes | The API's name, as `[apis.<name>]` declares it. |
| `method` | yes | The operation's method, in lowercase: `get`. |
| `path` | yes | The operation's path, as the spec writes it: `/users/{id}`. |
| `operation-id` | when it has one | The operation's `operationId`. Left out for an operation with no id. |
| `deprecated` | when it's deprecated | Present, with no value, when the spec marks the operation `deprecated`, as `planned` is on `<ascribe-availability-target>`. Left out otherwise. |

**Children:** no markdown. One line: a link, `<a href="…">`, to the operation's address in the published reference, the address a link to the operation has (SPEC §5.2), whose text is the operation's method in capitals and its path, then `: ` and its summary when it has one, its line breaks as spaces; then, for a deprecated operation, a space and `<span class="ascribe-operation-deprecated">(deprecated)</span>`. So the element's text is the plain output's line. It's one HTML block (§0): its opening tag alone on the first line, the line, and its closing tag.

```html
<ascribe-operation api="public" method="get" path="/users/{id}" operation-id="getUser" deprecated="">
<a href="/api/#operation/getUser">GET /users/{id}: Get a user</a> <span class="ascribe-operation-deprecated">(deprecated)</span>
</ascribe-operation>
```

**Rendering, with or without JavaScript:** a block holding the line, a deprecated operation's link struck through and its "(deprecated)" in the deprecated state's color. A site styles the method or the path by selecting on the attributes (`ascribe-operation[method="delete"]`). With no CSS, it's the link and its words: the deprecation is in the text, never only in the styling.

## 12. A code block's copy button

There's no element for a code block, and the emitter writes nothing for this: the library's script adds a button to each code block in the page, a `<pre>` with a `<code>` child, as the site output's fenced code blocks render, including one in a figure with a title or a tested line (§9) and one in a tab (§3). It copies the code ([decision #1011](../../project-docs/decisions/1011-copy-button.md)).

- **Where it goes:** the button is a `<button type="button" class="ascribe-copy">`, inserted directly after the `<pre>`, as its next sibling: outside the `<pre>` and its `<code>`, so their text, and an HTML-to-text converter's, is the code alone. It has no text of its own; its words are the stylesheet's (below).
- **What it copies:** the `<code>` element's text, without the newline that ends the block, and nothing else: not a figure's caption, not the button.
- **Naming and saying:** its accessible name, `aria-label`, is the stylesheet's word for it when it's made, "Copy". After a click it carries `data-ascribe-copy="copied"`, or `"failed"` when the clipboard refuses, for two seconds, and the page's one live region, a `<div class="ascribe-copy-status" role="status">` the script adds to the end of `<body>`, says the stylesheet's words for that state, "Copied" or "Couldn't copy", then empties.
- **Placement:** the script gives each `<pre>` an `anchor-name` of its own and the button the same `position-anchor`, in their `style`, and the stylesheet puts the button in the block's top corner, at its inline end. A browser without CSS anchor positioning shows it under the block, at its end.
- **Code blocks added later,** by a script, or a page swapped in place, get a button too.

**Without JavaScript** there's no button, and the page reads as it does with no element library: the script adds the button, and the stylesheet doesn't. **Without a clipboard,** on a page that isn't a secure context, the script adds none either.

**Theming:** `--ascribe-copy-display` is the button's `display`, `block` by default; a site sets it to `none` to leave the button out, on `:root` or on any element around the blocks it means. Its colors are `--ascribe-copy-color`, `--ascribe-copy-background`, `--ascribe-copy-hover-background`, and `--ascribe-copy-focus-color`. Its words are generated content, `.ascribe-copy::before`, and `.ascribe-copy[data-ascribe-copy="copied"]::before` and `[data-ascribe-copy="failed"]` for its two states, so a site restyles or translates them once, as it does "Available:", and the button's name and the live region follow. It isn't printed.
