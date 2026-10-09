# @ascribed/elements

The custom elements [Ascribe](https://github.com/ascribed-dev/ascribe)'s site output uses. They implement
[CONTRACT.md](CONTRACT.md) exactly: `<ascribe-note>`, `<ascribe-steps>`,
`<ascribe-tabs>` with `<ascribe-tab>`, `<ascribe-availability>` with
`<ascribe-availability-target>`, and `<ascribe-group>`.

- Elements render into the light DOM and are styled by CSS.
- Only `<ascribe-tabs>` uses JavaScript. Without the script, every tab shows
  with its label; nothing is hidden.

An Astro site gets them through `@ascribed/astro`, whose `<Elements />`
component loads the stylesheet and the script.

## Using it

```js
// Registers <ascribe-tabs>, <ascribe-tab>, and <ascribe-group>.
import "@ascribed/elements";
// The styles. Import this alone to style the elements without the script.
import "@ascribed/elements/style.css";
```

The CSS works with no script at all. A site can also link `css/style.css`
directly.

## Theming

Set any of these custom properties on `:root`, or on any ancestor of the
elements. Defaults are in `css/style.css`.

The names are part of the [contract](CONTRACT.md) and don't change. The
default values are Ascribe's design tokens, shared with review and the HTML
report, and a release may change them; the changelog says so under **Behavior
change**, with the previous values. A site that needs a fixed look sets the
properties itself.

| Group | Properties |
|---|---|
| Type | `--ascribe-font-family`, `--ascribe-font-size-small`, `--ascribe-font-weight-strong` |
| Spacing and borders | `--ascribe-space`, `--ascribe-space-small`, `--ascribe-border-width`, `--ascribe-border-color`, `--ascribe-radius` |
| Surfaces and text | `--ascribe-text-color`, `--ascribe-muted-color`, `--ascribe-surface-color` |
| Notes | `--ascribe-<type>-color` and `--ascribe-<type>-background`, for `note`, `tip`, `important`, `warning`, and `caution` |
| Steps | `--ascribe-steps-color`, `--ascribe-steps-marker-size`, `--ascribe-steps-marker-text-color` |
| Tabs | `--ascribe-tab-color`, `--ascribe-tab-active-color`, `--ascribe-tab-focus-color`, `--ascribe-tab-hover-background` |
| Availability | `--ascribe-state-color` and `--ascribe-state-background`, and `--ascribe-state-<state>-color` and `--ascribe-state-<state>-background` for `ga`, `preview`, `beta`, `deprecated`, and `removed` |

### Light and dark

The default colors are light or dark as the page is. They're dark where the
page's `color-scheme` is `dark`, or `light dark` while the reader's system is in
dark mode, and light otherwise, including on a page that declares no
`color-scheme`. So a site with a dark mode gets dark elements by declaring it:

```css
:root {
  color-scheme: light dark;
}
```

A page whose colors don't follow its `color-scheme` sets
`data-ascribe-scheme="light"` or `"dark"` on its root element, as the review
marks read it. The defaults use CSS `light-dark()`; a browser without it shows
the light colors. A property a site sets replaces both colors, so set it for
each scheme when you set one:

```css
:root {
  --ascribe-note-color: light-dark(#0550ae, #79c0ff);
}
```

### Glossary terms

A glossary link carries `data-ascribe-term` with the term's id, and its title
is the definition. The library underlines it with dots; restyle it by
selecting on the attribute, such as `a[data-ascribe-term]`.

### A project's own note types and lifecycle states

Notes and availability targets carry their type or states as attributes, so a
project styles its own by selecting on them. The elements take their colors
from `--_color` and `--_background`:

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

`states` lists a target's states in order and the last one styles the badge, so match it as a whole token, as above: `[states="x"], [states$=" x"]`. A bare `[states$="x"]` would also match a state named `pre-x`. Unknown types and states get the default (`note`, or the neutral state) style.

### Generated text

Headings and labels come from the elements' attributes through CSS generated
content, and the availability lead-in is `ascribe-availability::before`
(`"Available: "`). A site can restyle or translate them:

```css
ascribe-availability::before {
  content: "Disponible : ";
}
```

## Tests

`pnpm --filter @ascribed/elements test` compiles the library and drives it
through Playwright. Locally it runs Chromium alone; in CI (`CI` set, as Actions
sets it) it runs Chromium, Firefox, and WebKit, in Playwright's container
image, which has all three. `ASCRIBE_ENGINES=chromium,firefox,webkit` (any
comma-separated subset) chooses the engines either way; install them with
`pnpm --filter @ascribed/elements exec playwright-core install chromium firefox webkit`.
Chromium uses `/opt/pw-browsers/chromium` when present, or the binary in
`ASCRIBE_CHROMIUM`. The compile is skipped when every file in `dist/` is newer
than every file in `src/` and both tsconfigs.
