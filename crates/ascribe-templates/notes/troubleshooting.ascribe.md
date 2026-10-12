# Writing troubleshooting in Ascribe

Ascribe's notes on troubleshooting, to read after its guide: only what's particular to writing it in an Ascribe project. They're yours to change, as the template is.

- **A symptom is an address the product can link to.** Under the symptom's heading, `@id {permalink=true}: sync-refused` gives it an address that keeps working when the page moves or the heading is reworded. The product's error message can link straight to it, and `ascribe permalinks verify` checks those links in the product's CI.
- **A cause and its solution are bold paragraphs.** The template's `###` headings give each symptom the same "Cause" and "Solution" headings, and `heading-duplicate-without-id` warns about every one after the first, since its id is numbered and changes when a symptom moves. Start each with `**Cause.**` and `**Solution.**` instead, or keep the headings and give each one an `@id` of its own.
- **Quote the message as the product prints it,** in a code block, so a reader's search finds the page. When your tests record that output, take it with `@snippet` rather than pasting it.
- **A cause on one platform or release is marked.** `@available: quill 3.4` directly above a cause badges it, and a build filtered to another release leaves it out. When only the fix differs by platform, write a variant group for it.
- **A solution is a procedure.** Write `@steps` directly above its numbered list.
- **A step that risks data has a warning before it:** `@note {type=warning}: Don't delete the workspace: its changes aren't on the remote yet.`

The directives are in the [directives reference](https://ascribed-dev.com/docs/reference/directives/), and permalinks in [their guide](https://ascribed-dev.com/docs/guides/permalinks/).
