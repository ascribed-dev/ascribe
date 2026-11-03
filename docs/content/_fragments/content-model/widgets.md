Project widgets (SPEC §6): directives a documentation set defines. Each declaration sets the same parts a built-in directive has: its forms, its primary (the text after the colon), its attributes, what it applies to, its title, and whether it groups (SPEC §3, §4).

@snippet: code:examples/content-models/full.toml#widgets

`<name>` is the widget's name: lowercase words of letters and digits joined by single hyphens, with at least one hyphen, starting with a letter (a [widget name](../../content-model/index.md#12-names)). Names starting with `ascribe-` are reserved for Ascribe's element library, and the names HTML reserves for itself (`annotation-xml`, `color-profile`, `font-face`, `font-face-src`, `font-face-uri`, `font-face-format`, `font-face-name`, `missing-glyph`) aren't allowed, because the site output writes a widget as a custom element with the widget's name ([SPEC §9.4]({repo}/blob/main/SPEC.md#94-outputs)).

@include: ../../_generated/content-model-widgets.md

**Binding values:**

| Value | What it applies to ([SPEC §3.8]({repo}/blob/main/SPEC.md#38-binding)) | Behaves like |
|---|---|---|
| `"self"` | Self: its own primary, or nothing | `@include` |
| `"heading"` | Preceding heading: at the top of its section | `@id` |
| `"block"` | Following block. When the widget takes text after the colon and the text is given, the text is the content and it applies to nothing else | `@steps`; with `primary = "text?"`, `@note` |
| `"heading-or-block"` | At the top of a section, the section; anywhere else, the following block | `@available` |
