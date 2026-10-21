---
section: The language
---

**A field reads code spans only when it sets `inline = "code"`, on a content type's top-level `string` fields, and reads nothing else as markup.** Every output carries the plain text and the formatted form (SPEC §7.2). *Considered:* reading code spans in every title, and links or emphasis in titles. _Why:_ reading code spans in every title would change the meaning of every existing title with a backtick and make `title` the one string field that isn't plain text; and a link or emphasis goes wrong where a title becomes the text of another link. _From:_ the content model contract's decisions, item 22 ([§26 of its validation rules](https://github.com/ascribed-dev/ascribe/blob/eb7e077/docs/content/contracts/content-model/validation.md#26-decisions)), with release 0.2.0, 6 October 2026; a file of its own since [issue #689](https://github.com/ascribed-dev/ascribe/issues/689).
