# @tessera/elements

The custom elements Tessera's site output uses (SPEC §9.7). They implement
[CONTRACT.md](CONTRACT.md) exactly: `<tessera-note>`, `<tessera-steps>`,
`<tessera-tabs>` with `<tessera-tab>`, `<tessera-availability>` with
`<tessera-availability-target>`, and `<tessera-group>`.

- Elements render into the light DOM and are styled by CSS.
- Only `<tessera-tabs>` uses JavaScript. Without the script, every tab shows
  with its label; nothing is hidden.
- The package is `private` until phase 27.

## Using it

```js
// Registers <tessera-tabs>, <tessera-tab>, and <tessera-group>.
import "@tessera/elements";
// The styles. Import this alone to style the elements without the script.
import "@tessera/elements/style.css";
```

The CSS works with no script at all. A site can also link `css/style.css`
directly.

## Theming

Set any of these custom properties on `:root`, or on any ancestor of the
elements. Defaults are in `css/style.css`.

| Group | Properties |
|---|---|
| Type | `--tessera-font-family`, `--tessera-font-size-small`, `--tessera-font-weight-strong` |
| Spacing and borders | `--tessera-space`, `--tessera-space-small`, `--tessera-border-width`, `--tessera-border-color`, `--tessera-radius` |
| Surfaces and text | `--tessera-text-color`, `--tessera-muted-color`, `--tessera-surface-color` |
| Notes | `--tessera-<type>-color` and `--tessera-<type>-background`, for `note`, `tip`, `important`, `warning`, and `caution` |
| Steps | `--tessera-steps-color`, `--tessera-steps-marker-size`, `--tessera-steps-marker-text-color` |
| Tabs | `--tessera-tab-color`, `--tessera-tab-active-color`, `--tessera-tab-focus-color`, `--tessera-tab-hover-background` |
| Availability | `--tessera-state-color` and `--tessera-state-background`, and `--tessera-state-<state>-color` and `--tessera-state-<state>-background` for `ga`, `preview`, `beta`, `deprecated`, and `removed` |

### A project's own note types and lifecycle states

Notes and availability targets carry their type or states as attributes, so a
project styles its own by selecting on them. The elements take their colors
from `--_color` and `--_background`:

```css
tessera-note[type="security"] {
  --_color: #b62324;
  --_background: #fff0f0;
}

tessera-availability-target[states="sunset"],
tessera-availability-target[states$=" sunset"] {
  --_color: #6e7781;
  --_background: #eaeef2;
}
```

`states` lists a target's states in order and the last one styles the badge, so match it as a whole token, as above: `[states="x"], [states$=" x"]`. A bare `[states$="x"]` would also match a state named `pre-x`. Unknown types and states get the default (`note`, or the neutral state) style.

### Generated text

Headings and labels come from the elements' attributes through CSS generated
content, and the availability lead-in is `tessera-availability::before`
(`"Available: "`). A site can restyle or translate them:

```css
tessera-availability::before {
  content: "Disponible : ";
}
```

## Tests

`pnpm --filter @tessera/elements test` compiles the library and drives Chromium
through Playwright in Chromium, Firefox, and WebKit. Install them with
`pnpm --filter @tessera/elements exec playwright-core install chromium firefox webkit`.
Chromium uses `/opt/pw-browsers/chromium` when present, or the binary in
`TESSERA_CHROMIUM`. `TESSERA_ENGINES=chromium` (a comma-separated subset) runs
fewer engines on a machine that can't install all three.
