<!-- Generated from packages/vscode/package.json by packages/vscode/test/unit/docs.test.ts. Edit the manifest, or a command's description in the test, then run `ASCRIBE_BLESS=1 pnpm --filter ascribe-vscode exec vitest run test/unit/docs.test.ts`. -->

| Command | What it does |
|---|---|
| **Ascribe: Restart Language Server** | Stops and starts every project's server that has started, including one that stopped after crashing, and forgets earlier crashes. A server that hasn't started stays off until it's needed; when none has, the command says so. It also picks up `ascribe.toml` files added or deleted. |
| **Ascribe: Show Server Output** | Opens the log of the active file's project. When no file of a project is active and the workspace has several projects, it asks which, showing whether each one's server is running. |
| **Ascribe: Open Page Preview** {available=next} | Opens the page preview of the active page in place of the editor, starting its project's server if it hasn't started. |
| **Ascribe: Open Page Preview to the Side** | Opens the page preview of the active page beside the editor, starting its project's server if it hasn't started. |
| **Ascribe: Open Site Preview** {available=next} | Opens the active page on its project's dev server, in the browser. See [Site preview](../guides/editor.md#site-preview). |
| **Ascribe: Select Preview Build** | Picks the build the preview shows, for the previewed page's project. |
| **Ascribe: Switch Build** {available=next} | Picks the build you're looking at in the active file's project: the one the preview renders and the status bar names. Choosing the editor build follows `[editor] build`. See [The status bar](../guides/editor.md#the-status-bar). |
| **Ascribe: Project Menu** {available=next} | Shows the menu of the active file's project, as clicking its status bar item does: switch the build, show the server's output, restart its server, or open the preview. |
| **Ascribe: Start Review** {available=next} | Marks what changed in the preview, against a base it asks for, for the active page's project. Its server must be running: open one of its pages first. See [Review in the preview](../guides/editor.md#review-in-the-preview). |
| **Ascribe: Stop Review** {available=next} | Turns review off for the active page's project, and frees its base. |
| **Ascribe: Changed Pages** {available=next} | Lists the pages the change touches in the preview's build; choosing one opens it and its preview. |
| **Ascribe: Refresh Comments** {available=next} | Reads the pull request's review threads from GitHub again, for the active page's project. See [Comments in the preview](../guides/editor.md#comments-in-the-preview). |
| **Ascribe: Actions for the Cursor** {available=next} | Opens the [actions bar](../guides/editor.md#the-actions-bar): the fixes for problems at the cursor, then the actions that apply to the cursor or selection. |
| **Ascribe: Wrap in a note** {available=next} | Put the paragraph or the selected blocks in an `@note` callout. One of the [actions](../guides/editor.md#actions). |
| **Ascribe: Change the note's kind** {available=next} | Set the `type` of the `@note` the cursor is in. One of the [actions](../guides/editor.md#actions). |
| **Ascribe: Remove the note, keeping its text** {available=next} | Take the content out of its `@note`. One of the [actions](../guides/editor.md#actions). |
| **Ascribe: Turn the note into collapsible details** {available=next} | Make the `@note` a `@details` block, with a title. One of the [actions](../guides/editor.md#actions). |
| **Ascribe: Wrap in collapsible details** {available=next} | Put the block or the selected blocks in `@details`, with a title. One of the [actions](../guides/editor.md#actions). |
| **Ascribe: Remove the details, keeping the content** {available=next} | Take the content out of its `@details` and its title. One of the [actions](../guides/editor.md#actions). |
| **Ascribe: Make the list steps** {available=next} | Show the numbered list as `@steps`. One of the [actions](../guides/editor.md#actions). |
| **Ascribe: Make the steps a plain list** {available=next} | Remove the list's `@steps`. One of the [actions](../guides/editor.md#actions). |
| **Ascribe: Give the heading a stable id** {available=next} | Add `@id:` under the heading, so links to it survive rewording it. One of the [actions](../guides/editor.md#actions). |
| **Ascribe: Copy a link to this section** {available=next} | Copy the heading's destination from the content root, such as `/guides/install.md#install-cli`, to paste in any page. One of the [actions](../guides/editor.md#actions). |
| **Ascribe: Insert a note** {available=next} | Add an `@note` callout, with its text ready to type. One of the [actions](../guides/editor.md#actions). |
| **Ascribe: Insert steps** {available=next} | Add `@steps` and a numbered list. One of the [actions](../guides/editor.md#actions). |
| **Ascribe: Insert collapsible details** {available=next} | Add a `@details` block with a title. One of the [actions](../guides/editor.md#actions). |
| **Ascribe: Insert content that varies** {available=next} | Add `@variant` arms, one for each value of a dimension you choose. One of the [actions](../guides/editor.md#actions). |
| **Ascribe: Add a variant** {available=next} | Add a `@variant` arm for another value of the group's dimension. One of the [actions](../guides/editor.md#actions). |
| **Ascribe: Remove this variant** {available=next} | Remove the `@variant` arm the cursor is in from its group. One of the [actions](../guides/editor.md#actions). |
| **Ascribe: Mark where it's available** {available=next} | Add `@available:` to the section or block, or `{available=…}` to the table row, with a feature or a spec. One of the [actions](../guides/editor.md#actions). |
| **Ascribe: Make the page one variant** {available=next} | Set a dimension's value under `variant:` in the page's frontmatter. One of the [actions](../guides/editor.md#actions). |
| **Ascribe: Set where the page is available** {available=next} | Set `available:` in the page's frontmatter, with a feature or a spec. One of the [actions](../guides/editor.md#actions). |
| **Ascribe: Link the selected text** {available=next} | Make the selection a link, `[text](…)`, to a page or heading you choose. One of the [actions](../guides/editor.md#actions). |
| **Ascribe: Insert a link** {available=next} | Add a link, `[](…)`, to a page or heading you choose, that shows its title. One of the [actions](../guides/editor.md#actions). |
| **Ascribe: Change where the link goes** {available=next} | Set the link's destination to a page or heading you choose. One of the [actions](../guides/editor.md#actions). |
| **Ascribe: Show the page's title as the link text** {available=next} | Empty the link's text, `[](…)`, so it shows its target's title. One of the [actions](../guides/editor.md#actions). |
| **Ascribe: Insert a phrase** {available=next} | Add a phrase the content model declares, `{key}`, which shows its value. One of the [actions](../guides/editor.md#actions). |
| **Ascribe: Insert an image** {available=next} | Add an image of the project, `![alt](path)`, with its alt text. One of the [actions](../guides/editor.md#actions). |
| **Ascribe: Set the image's width** {available=next} | Set the image's `width` attribute. One of the [actions](../guides/editor.md#actions). |
| **Ascribe: Change the image's description** {available=next} | Set the image's alt text, `![alt]`, which readers who can't see it get. One of the [actions](../guides/editor.md#actions). |
| **Ascribe: Include a fragment** {available=next} | Add `@include:` for a fragment, a page, or one of its sections. One of the [actions](../guides/editor.md#actions). |
| **Ascribe: Insert a code snippet** {available=next} | Add `@snippet:` with code from one of the project's sources. One of the [actions](../guides/editor.md#actions). |
| **Ascribe: Insert a widget** {available=next} | Add one of the project's widgets, `@name`, with the attributes it needs. One of the [actions](../guides/editor.md#actions). |
