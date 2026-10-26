# @ascribed/elements

The custom elements [Ascribe](https://github.com/ascribed-dev/ascribe)'s site output uses. They implement
[CONTRACT.md](CONTRACT.md) exactly: `<ascribe-note>`, `<ascribe-steps>`,
`<ascribe-tabs>` with `<ascribe-tab>`, `<ascribe-availability>` with
`<ascribe-availability-target>`, `<ascribe-group>`, and `<ascribe-for-agents>`.

- Elements render into the light DOM and are styled by CSS.
- Only `<ascribe-tabs>` uses JavaScript, and the script adds a copy button to
  each code block. Without the script, every tab shows with its label, nothing
  is hidden, and there's no copy button.

An Astro site gets them through `@ascribed/astro`, whose `<Elements />`
component loads the stylesheet and the script.

## Using it

```js
// Registers <ascribe-tabs>, <ascribe-tab>, and <ascribe-group>, and adds
// the copy buttons.
import "@ascribed/elements";
// The styles. Import this alone to style the elements without the script.
import "@ascribed/elements/style.css";
```

The CSS works with no script at all. A site can also link `css/style.css`
directly.

A script can make a reader's choice for a dimension, as selecting a tab does:

```js
import { choose } from "@ascribed/elements";
// Remembered across pages, and selected in this page's `pm` groups.
choose("pm", "pnpm");
```

## Theming

The [element library reference](https://ascribed-dev.com/go/elements/) lists
the CSS custom properties that theme the elements, says what each one sets and
how they follow the page's light or dark scheme, and shows how to style your
own note types and lifecycle states. The properties' names are part of the
[contract](CONTRACT.md) and don't change; their defaults, in `css/style.css`,
may change in a release, as the changelog says.

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
