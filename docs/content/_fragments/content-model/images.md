Image attributes ([SPEC §5.3]({repo}/blob/main/SPEC.md#53-images)): which keys may appear in an attribute block after an image, and their types.

@snippet: code:examples/content-models/full.toml#images

@include: ../../_generated/content-model-images.md

Alt text and titles aren't attributes; they use CommonMark's syntax ([SPEC §5.3]({repo}/blob/main/SPEC.md#53-images)). Presentation choices aren't image attributes either. Keys HTML already uses on `<img>` (`src`, `alt`, `title`, and its global attributes) are rejected (`model-attribute-reserved`), since the site output writes image attributes onto the `<img>` element ([SPEC §7.2]({repo}/blob/main/SPEC.md#72-declarations)).
