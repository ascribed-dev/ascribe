<!-- Generated from packages/vscode/src/actions/registry.ts by packages/vscode/test/unit/docs.test.ts. Edit the registry, then run `ASCRIBE_BLESS=1 pnpm --filter ascribe-vscode exec vitest run test/unit/docs.test.ts`. -->

| Action | What it does | Where it applies |
|---|---|---|
| **Wrap in a note** | Put the paragraph or the selected blocks in an `@note` callout. Also in the lightbulb. | A paragraph, or whole blocks selected |
| **Change the note's kind** | Set the `type` of the `@note` the cursor is in. | A note |
| **Remove the note, keeping its text** | Take the content out of its `@note`. Also in the lightbulb. | A note |
| **Turn the note into collapsible details** | Make the `@note` a `@details` block, with a title. Also in the lightbulb. | A note |
| **Wrap in collapsible details** | Put the block or the selected blocks in `@details`, with a title. Also in the lightbulb. | A paragraph, code block, table, include, or snippet, or whole blocks selected |
| **Remove the details, keeping the content** | Take the content out of its `@details` and its title. Also in the lightbulb. | A details block |
| **Make the list steps** | Show the numbered list as `@steps`. Also in the lightbulb. | A numbered list that isn't steps |
| **Make the steps a plain list** | Remove the list's `@steps`. Also in the lightbulb. | Steps |
| **Give the heading a stable id** | Add `@id:` under the heading, so links to it survive rewording it. Also in the lightbulb. | A heading without `@id` |
| **Copy a link to this section** | Copy the heading's destination from the content root, such as `/guides/install.md#install-cli`, to paste in any page. | A heading |
| **Insert a note** | Add an `@note` callout, with its text ready to type. | A blank line between blocks |
| **Insert steps** | Add `@steps` and a numbered list. | A blank line between blocks |
| **Insert collapsible details** | Add a `@details` block with a title. | A blank line between blocks |
| **Insert content that varies** | Add `@variant` arms, one for each value of a dimension you choose. | A blank line between blocks |
| **Add a variant** | Add a `@variant` arm for another value of the group's dimension. | A group of one dimension's `@variant` arms |
| **Remove this variant** | Remove the `@variant` arm the cursor is in from its group. | An arm of a group with others |
| **Mark where it's available** | Add `@available:` to the section or block, or `{available=…}` to the table row, with a feature or a spec. | A heading (its section), a block, or a table's body row |
| **Make the page one variant** | Set a dimension's value under `variant:` in the page's frontmatter. | Anywhere in a page |
| **Set where the page is available** | Set `available:` in the page's frontmatter, with a feature or a spec. | Anywhere in a page |
| **Link the selected text** | Make the selection a link, `[text](…)`, to a page or heading you choose. Also in the lightbulb. | Text selected in one paragraph or heading |
| **Insert a link** | Add a link, `[](…)`, to a page or heading you choose, that shows its title. | The cursor in a paragraph or heading, outside links and phrases |
| **Change where the link goes** | Set the link's destination to a page or heading you choose. | A link |
| **Show the page's title as the link text** | Empty the link's text, `[](…)`, so it shows its target's title. Also in the lightbulb. | A link to a page, with text |
| **Insert a phrase** | Add a phrase the content model declares, `{key}`, which shows its value. | The cursor in a paragraph or heading, outside links and phrases |
| **Insert an image** | Add an image of the project, `![alt](path)`, with its alt text. | A blank line between blocks |
| **Set the image's width** | Set the image's `width` attribute. | An image |
| **Change the image's description** | Set the image's alt text, `![alt]`, which readers who can't see it get. | An image |
| **Include a fragment** | Add `@include:` for a fragment, a page, or one of its sections. | A blank line between blocks |
| **Insert a code snippet** | Add `@snippet:` with code from one of the project's sources. | A blank line between blocks |
| **Insert a widget** | Add one of the project's widgets, `@name`, with the attributes it needs. | A blank line between blocks |
| **Make this a phrase** | Declare the selected text in `[phrases]` and write `{key}` in its place, and in its other occurrences if you choose. Also in the lightbulb. | Text selected in one paragraph or heading |
| **Add to the glossary** | Declare the selected text as a term in `[glossary.terms]`, with its aliases, definition, and link. | Text selected in one paragraph or heading |
| **Change a feature's availability** | Set where a feature in `[features]` is available, such as its state or version. | Anywhere in a page |
| **Rename this phrase everywhere** | Change a phrase's key in `[phrases]` and every `{key}` that uses it: the one at the cursor, or one you choose. | Anywhere in a page; a phrase at the cursor is the one renamed |
| **Rename a dimension value everywhere** | Change a value in `[dimensions]` and everywhere it's used: `@variant` attributes, `variant:`, availability, and builds. | Anywhere in a page; a `@variant` attribute's value at the cursor is the one renamed |
