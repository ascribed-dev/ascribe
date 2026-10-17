# Writing a concept in Ascribe

Ascribe's notes on the concept, to read after its guide: only what's particular to writing one in an Ascribe project. They're yours to change, as the template is.

- **The definition is the glossary's too.** A term the page defines can have a section on the glossary page, a heading marked `@term` with a sentence or two and a link to this page. Then every page that uses the term links it to the glossary, in each build, without anyone marking it. Keep the glossary's sentence and the page's definition saying the same thing.
- **A diagram has alt text that says what it shows.** Ascribe warns about an image without it. The file goes beside the page, and its name must match exactly, including case.
- **A comparison that differs by release** marks its rows in their first cell: `` | Streaming sync {available="quill preview 3.4"} | Yes | No | ``.
- **The related resources link files, with empty text.** `[](../how-to/create-a-workspace.md)` shows that page's title as it is now, and `ascribe check` reports the link when the page moves. Group them under the template's headings, or under one list when there are only a few.
- **A concept has no procedure.** When the page starts to need steps, they're a how-to of their own: link to it.

The glossary is in the [content model reference](https://ascribed-dev.com/docs/content-model/content/#12-glossary).
