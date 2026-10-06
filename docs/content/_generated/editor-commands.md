<!-- Generated from packages/vscode/package.json by packages/vscode/test/unit/docs.test.ts. Edit the manifest, or a command's description in the test, then run `ASCRIBE_BLESS=1 pnpm --filter ascribe-vscode exec vitest run test/unit/docs.test.ts`. -->

| Command | What it does |
|---|---|
| **Ascribe: Restart Language Server** | Stops and starts every project's server that has started, including one that stopped after crashing, and forgets earlier crashes. A server that hasn't started stays off until it's needed; when none has, the command says so. It also picks up `ascribe.toml` files added or deleted. |
| **Ascribe: Show Server Output** | Opens the log of the active file's project. When no file of a project is active and the workspace has several projects, it asks which, showing whether each one's server is running. |
| **Ascribe: Open Page Preview** {available=next} | Opens the page preview of the active page in place of the editor, starting its project's server if it hasn't started. |
| **Ascribe: Open Page Preview to the Side** | Opens the page preview of the active page beside the editor, starting its project's server if it hasn't started. |
| **Ascribe: Open Site Preview** {available=next} | Opens the active page on its project's dev server, in the browser. See [Site preview](../guides/editor.md#site-preview). |
| **Ascribe: Select Preview Build** | Picks the build the preview shows, for the previewed page's project. |
| **Ascribe: Start Review** {available=next} | Marks what changed in the preview, against a base it asks for, for the active page's project. Its server must be running: open one of its pages first. See [Review in the preview](../guides/editor.md#review-in-the-preview). |
| **Ascribe: Stop Review** {available=next} | Turns review off for the active page's project, and frees its base. |
| **Ascribe: Changed Pages** {available=next} | Lists the pages the change touches in the preview's build; choosing one opens it and its preview. |
| **Ascribe: Refresh Comments** {available=next} | Reads the pull request's review threads from GitHub again, for the active page's project. See [Comments in the preview](../guides/editor.md#comments-in-the-preview). |
