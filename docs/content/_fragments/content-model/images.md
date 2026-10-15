Image attributes ([SPEC §5.3]({repo}/blob/main/SPEC.md#53-images)): which keys may appear in an attribute block after an image, and their types.

@snippet: code:examples/content-models/full.toml#images

| Key | Type | Default | Description |
|---|---|---|---|
| `attributes` | table of attribute types | `{}` | Accepted image attribute keys (`key` rule) and their types (§5). With no entries, an image accepts no attributes, and any attribute block after an image is an error ([SPEC §8.2]({repo}/blob/main/SPEC.md#82-diagnostics), "Unknown key"). Declaration order is canonical order ([SPEC §8.3]({repo}/blob/main/SPEC.md#83-canonical-form)). A declared default reaches the output: in the site output, every image carries the defaults of attributes it doesn't write. |

Alt text and titles aren't attributes; they use CommonMark's syntax ([SPEC §5.3]({repo}/blob/main/SPEC.md#53-images)). Presentation choices aren't image attributes either. Keys HTML already uses on `<img>` (`src`, `alt`, `title`, and its global attributes) are rejected (`model-attribute-reserved`), since the site output writes image attributes onto the `<img>` element ([SPEC §7.2]({repo}/blob/main/SPEC.md#72-declarations)).
