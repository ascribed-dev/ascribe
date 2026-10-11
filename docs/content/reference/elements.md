---
title: Element library reference
description: "`@ascribed/elements`: the custom elements the site output uses, the CSS custom properties that theme them, and styling a project's own note types and lifecycle states."
permalink: elements
---

The site output writes Ascribe's constructs as custom elements, such as `<ascribe-note>` for a note. `@ascribed/elements` is the library that draws them: a stylesheet for every element, and a small script for `<ascribe-tabs>`, the only one with behavior. This page lists the elements and the CSS custom properties a site sets to theme them, and shows how to style what the library doesn't know, such as a project's own note types.

The [element contract]({repo}/blob/main/packages/elements/CONTRACT.md) defines each element exactly: its attributes, its children, and how it renders with and without the script. It's in the package, as `CONTRACT.md`.

## Using it

A site built with [`@ascribed/astro`](../guides/astro.md) has the library already: its `<Elements />` component loads the stylesheet and the script. Any other site imports them:

```js
// Registers <ascribe-tabs>, <ascribe-tab>, and <ascribe-group>.
import "@ascribed/elements";
// The styles. Import this alone to style the elements without the script.
import "@ascribed/elements/style.css";
```

The elements render into the page, with no shadow DOM, so the site's own styles apply to them too. The stylesheet works with no script at all: without it, every tab of a group shows, one after another, each with its label, and nothing is hidden.

## The elements

| Element | What the site output writes it for | Contract |
|---|---|---|
| `<ascribe-note>` | A `@note`. Its `type` attribute is the note type, such as `tip`. | [§1]({repo}/blob/main/packages/elements/CONTRACT.md#1-ascribe-note) |
| `<ascribe-steps>` | A `@steps` procedure, around its ordered list. | [§2]({repo}/blob/main/packages/elements/CONTRACT.md#2-ascribe-steps) |
| `<ascribe-tabs>` | A `@variant` group that keeps more than one arm. With the script, its tabs switch, and a group with `sync` remembers the reader's choice across pages. | [§3]({repo}/blob/main/packages/elements/CONTRACT.md#3-ascribe-tabs-and-ascribe-tab) |
| `<ascribe-tab>` | One arm of `<ascribe-tabs>`, with its `label`. | [§3]({repo}/blob/main/packages/elements/CONTRACT.md#3-ascribe-tabs-and-ascribe-tab) |
| `<ascribe-availability>` | An `@available` badge, holding one target for each thing it's available on. | [§4]({repo}/blob/main/packages/elements/CONTRACT.md#4-ascribe-availability-and-ascribe-availability-target) |
| `<ascribe-availability-target>` | One target of `<ascribe-availability>`. Its `states` attribute lists its lifecycle states, and the last one styles the badge. | [§4]({repo}/blob/main/packages/elements/CONTRACT.md#4-ascribe-availability-and-ascribe-availability-target) |
| `<ascribe-group>` | A group of a project widget's arms, so the project's styles or script can find them together. The library gives it `display: block` and nothing else. | [§6]({repo}/blob/main/packages/elements/CONTRACT.md#6-project-widgets) |
| `<ascribe-for-agents>` | The pointer to `llms.txt` at the top of each page, with `[consumer] agents = true`. The library hides it from sight, not from the page's text. | [§8]({repo}/blob/main/packages/elements/CONTRACT.md#8-ascribe-for-agents) |

A `@details` block is HTML's own `<details>`, and a glossary term an ordinary link marked with `data-ascribe-term`, so neither has an element. The library underlines a term's link with dots; restyle it by selecting `a[data-ascribe-term]`. A project's own widget is an element named after it, which the project implements.

## Custom properties

Set any of these on `:root`, or on any element around the elements you mean. Their names are part of the element contract and don't change. Their default values are Ascribe's design tokens, which a release may change, as its changelog says under **Behavior change** with the previous values; a site that needs a fixed look sets the properties itself. The defaults are in the package's `css/style.css`.

@include: ../_generated/elements-properties.md

### Light and dark

The default colors are light or dark as the page is: dark where the page's `color-scheme` is `dark`, or `light dark` while the reader's system is in dark mode, and light otherwise, including on a page that declares no `color-scheme`. So a site with a dark mode gets dark elements by declaring it:

```css
:root {
  color-scheme: light dark;
}
```

A page whose colors don't follow its `color-scheme` sets `data-ascribe-scheme="light"` or `"dark"` on its root element. The defaults use CSS `light-dark()`, and a browser without it shows the light colors.

A property a site sets replaces both colors, so set both when you set one:

```css
:root {
  --ascribe-note-color: light-dark(#0550ae, #79c0ff);
}
```

## A project's own note types and lifecycle states

A note carries its type, and an availability target its states, as attributes, so a project styles its own by selecting on them. A note or a badge takes its colors from `--_color` and `--_background`:

```css
ascribe-note[type="security"] {
  --_color: #b62324;
  --_background: #fff0f0;
}

ascribe-availability-target[states="sunset"],
ascribe-availability-target[states$=" sunset"] {
  --_color: #6e7781;
  --_background: #eaeef2;
}
```

`states` lists a target's states in order, and the last one styles the badge, so match it as a whole word, as above: `[states="x"], [states$=" x"]`. A bare `[states$="x"]` would also match a state named `pre-x`. A type or a state the library doesn't style gets the default: the `note` type's colors, or the neutral badge's, `--ascribe-state-color` and `--ascribe-state-background`.

The colors can come from the library's own properties, so a project's state looks like one of Ascribe's in both schemes: this site gives its `unreleased` state the colors of `preview`, with `--_color: var(--ascribe-state-preview-color)` and `--_background: var(--ascribe-state-preview-background)`.

## Generated text

A note's heading line and a tab's label, when the script isn't running, come from the elements' attributes through CSS generated content. So does the lead-in of a line of availability badges, `ascribe-availability::before`, which is `Available: `. A site can restyle or translate it:

```css
ascribe-availability::before {
  content: "Disponible : ";
}
```

## Choosing a tab from a script

`choose(dimension, value)` makes a reader's choice for a dimension, as selecting a tab does: it selects the value in every group on the page that syncs on the dimension, and remembers it, so the next page's groups start on it.

```js
import { choose } from "@ascribed/elements";
// Remembered across pages, and selected in this page's `pm` groups.
choose("pm", "pnpm");
```

A permalink's page uses it to carry a choice in its address; see [Linking from a product](../guides/permalinks.md#linking-from-a-product).
