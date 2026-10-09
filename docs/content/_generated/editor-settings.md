<!-- Generated from packages/vscode/package.json by packages/vscode/test/unit/docs.test.ts. Edit the manifest, or a command's description in the test, then run `ASCRIBE_BLESS=1 pnpm --filter ascribe-vscode exec vitest run test/unit/docs.test.ts`. -->

| Setting | Default | What it does |
|---|---|---|
| `ascribe.path` | empty | The `ascribe` binary to run, for every project. When empty, each project's own `node_modules/.bin/ascribe`, then the binary included in the extension. Changing it restarts the servers that have started. |
| `ascribe.startServers` | `onDemand` | When to start the language server of each `ascribe.toml` project in the workspace. A project whose server hasn't started shows no problems in the Problems panel. A running server reports pages for the editor's build only (`[editor] build`); `ascribe check` checks every build. `onDemand`: Start a project's language server the first time one of its files is opened, or changes on disk while the window is open. `all`: Start every project's language server when the workspace opens. |
| `ascribe.formatOnSave` | `false` | Format Ascribe constructs into canonical form when saving. |
| `ascribe.preview.scrollPreviewWithEditor` {available=next} | `true` | Scroll the preview to the block at the top of the editor, and to the block the cursor moves to when it's out of view. |
| `ascribe.preview.scrollEditorWithPreview` {available=next} | `true` | Scroll the editor to the line of the block at the top of the preview when the preview is scrolled. |
| `ascribe.review.sourceComments` {available=next} | `auto` | Whether the source editor shows the pull request's review threads on the lines they're on, while review is on. `auto`: Show review threads on source lines unless the GitHub Pull Requests extension is active, which shows them already. `on`: Always show review threads on source lines. `off`: Show review threads in the page preview only. |
| `ascribe.maxCrashes` | `5` | How many crashes of a project's language server, since it was last restarted by hand, make the extension stop restarting it and explain why. |
| `ascribe.trace.server` | `off` | Log the conversation with the language server to its output channel, for reporting a problem. `off`: Log nothing. `messages`: Log each message's name and timing. `verbose`: Log each message with its content. |
