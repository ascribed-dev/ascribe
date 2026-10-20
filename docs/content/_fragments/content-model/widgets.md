Project widgets (SPEC §6): directives a documentation set defines. Each declaration is a directive schema with the same parts a built-in directive's schema has (SPEC §3, §4).

@snippet: code:examples/content-models/full.toml#widgets

`<name>` is the widget's name (SPEC A `widget-name`: lowercase, with at least one hyphen). Names starting with `ascribe-` are reserved for Ascribe's element library, and the names HTML reserves for itself (`annotation-xml`, `color-profile`, `font-face`, `font-face-src`, `font-face-uri`, `font-face-format`, `font-face-name`, `missing-glyph`) aren't allowed, because the site output emits a widget as a custom element with the widget's name ([SPEC §9.4]({repo}/blob/main/SPEC.md#94-outputs)).

@include: ../../_generated/content-model-widgets.md

**Binding values:**

| Value | [SPEC §3.8]({repo}/blob/main/SPEC.md#38-binding) binding | Behaves like |
|---|---|---|
| `"self"` | Self: its own primary, or nothing | `@include` |
| `"heading"` | Preceding heading: at the top of its section | `@id` |
| `"block"` | Following block. If the primary is a text primary and it's given, the primary is the content and nothing else is bound | `@steps`; with `primary = "text?"`, `@note` |
| `"heading-or-block"` | At the top of a section, the section; anywhere else, the following block | `@available` |
