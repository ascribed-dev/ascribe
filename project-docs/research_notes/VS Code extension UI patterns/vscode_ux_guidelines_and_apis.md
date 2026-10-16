# VS Code extension UX guidelines and UI APIs (as of October 2026)

Method note: the UX guideline pages, extension guides, and contribution-point reference were read from their Markdown source in `microsoft/vscode-docs` (main branch). Each UX guideline page carries `DateApproved: 10/14/2026`, so they are current. Stable versus proposed status was checked against `src/vscode-dts/vscode.d.ts` (stable) and the `vscode.proposed.*.d.ts` files in `microsoft/vscode` main. Release notes v1.80 (July 2023) to v1.143 (dated 2026-10-21) were scanned. VS Code moved from monthly to weekly releases in March 2026 (1.110 on 2026-03-04, 1.111 on 2026-03-09, then one release a week) — [v1.110](https://code.visualstudio.com/updates/v1_110), [v1.111](https://code.visualstudio.com/updates/v1_111), [v1.142](https://code.visualstudio.com/updates/v1_142).

## What the official UX guidelines say (view containers, views, tree views, welcome views, webviews, panel, secondary side bar, status bar, notifications, quick picks, command palette, context menus, walkthroughs, settings, custom editors)

### Takeaway
The guidelines push hard toward native UI and restraint: one view container per extension, 3 to 5 views at most, tree views for data, webviews only "when absolutely necessary", few status bar items, few notifications, and no custom settings pages. Ascribe's four stacked tree views fall inside the stated "3-5 comfortable max", but the guidelines also warn that other extensions share the same containers and that sidebar content which could be a command should be a command.

### Cited Findings

**Overall architecture**
- The workbench is split into containers (Activity Bar, Primary Side Bar, Secondary Side Bar, Editor, Panel, Status Bar) and items (views, view toolbar, sidebar toolbar, editor toolbar, panel toolbar, status bar items). Common UI elements: Command Palette, Quick Pick, Notifications, Webviews, Context Menus, Walkthroughs, Settings — [UX Guidelines overview](https://code.visualstudio.com/api/ux-guidelines/overview)
- Left status bar items are scoped to the whole workspace; right items are scoped to the active file — [overview](https://code.visualstudio.com/api/ux-guidelines/overview)

**Activity Bar**
- Do: use an icon that matches the default Activity Bar icon style; give the view container a clear, obvious name. Don't: duplicate an existing icon; use an Activity Bar item to open a webview panel — [Activity Bar](https://code.visualstudio.com/api/ux-guidelines/activity-bar)
- Icon spec for view containers: 24x24, centered, single color, SVG recommended; states inherit opacity (default 60%, hover/active 100%) — [contributes.viewsContainers](https://code.visualstudio.com/api/references/contribution-points#contributes.viewsContainers)

**Sidebars (Primary and Secondary)**
- Do: group related views together; use clear, descriptive names for containers and views — [Sidebars](https://code.visualstudio.com/api/ux-guidelines/sidebars)
- Don't: "Use an excessive number of View Containers. A single View Container (such as a Sidebar with Views unique to that extension) is generally enough for most extensions." Don't: "Use an excessive number of Views (3-5 is a comfortable max for most screen sizes)". Don't: "Add content to the Sidebar that could be a simple Command." Don't: repeat existing functionality — [Sidebars](https://code.visualstudio.com/api/ux-guidelines/sidebars)
- Primary Sidebar: "Use good judgement when adding content here—too much contributed UI can lead to a cluttered experience" — [Sidebars](https://code.visualstudio.com/api/ux-guidelines/sidebars)
- Sidebar toolbar behavior: a container with more than one view shows a single `...` button to show/hide each view; a container with only one view consolidates that view's actions into the sidebar toolbar. Be careful not to add too many actions; prefer existing product icons with descriptive command names — [Sidebars](https://code.visualstudio.com/api/ux-guidelines/sidebars)
- Secondary Sidebar (UX page wording): "While extensions cannot contribute Views directly to it by default, users can drag Views from the Primary Sidebar or the Panel" — [Sidebars](https://code.visualstudio.com/api/ux-guidelines/sidebars); **contradicted by** release notes 1.106: "Extension authors can now register view containers in the Secondary Side Bar by using the new `secondarySidebar` contribution point" (it was proposed as `contribSecondarySideBar` in 1.104) — [v1.104](https://code.visualstudio.com/updates/v1_104), [v1.106](https://code.visualstudio.com/updates/v1_106). The contribution-point reference also still says "At present, you can contribute them to the Activity Bar (`activitybar`) and Panel (`panel`)" — [contributes.viewsContainers](https://code.visualstudio.com/api/references/contribution-points#contributes.viewsContainers). The release notes are the newer, more specific source; the reference pages appear not updated.

**Views**
- Do: use existing icons; use file icons for language files; use a Tree View for displaying data; add an icon to every view (in case it is moved to the Activity Bar or Secondary Sidebar, which represent views by icon); keep the number of views to a minimum; keep names short; limit custom Webview Views — [Views](https://code.visualstudio.com/api/ux-guidelines/views)
- Don't: repeat existing functionality; use tree items as single action items (firing a command on click); use custom webview views if not necessary; use an Activity Bar item (view container) to open a webview in the editor — [Views](https://code.visualstudio.com/api/ux-guidelines/views)
- "Limit the number of Views created as other extensions can contribute in the same View Container." Views can be rearranged by the user or moved to another container — [Views](https://code.visualstudio.com/api/ux-guidelines/views)
- View locations: existing containers (`explorer`, `scm`, `debug`, `test`), custom containers in the Activity Bar, any container in the Panel; users can drag to the Secondary Sidebar — [Views](https://code.visualstudio.com/api/ux-guidelines/views), [contributes.views](https://code.visualstudio.com/api/references/contribution-points#contributes.views)
- A view's `icon` is used when its title cannot be shown (e.g. dragged to the Activity Bar); `contextualTitle` is used when the view is moved out of its default container. Opening a view fires `onView:${viewId}` activation; `when` controls visibility — [contributes.views](https://code.visualstudio.com/api/references/contribution-points#contributes.views)
- Views can show progress by referencing the view's ID (`ProgressLocation`) — [Views](https://code.visualstudio.com/api/ux-guidelines/views)
- View actions: "Be careful not to add too many actions to avoid noise and confusion"; built-in product icons help fit in; custom SVG allowed — [Views](https://code.visualstudio.com/api/ux-guidelines/views)

**Tree views**
- Do: use descriptive labels to give context; use product icons to distinguish item types. Don't: use tree view items as buttons to fire commands; nest deeply ("A few levels of folders/items is a good balance"); add more than three actions to an item — [Views § Tree Views](https://code.visualstudio.com/api/ux-guidelines/views#tree-views)

**Welcome views**
- Do: use only when necessary; prefer links over buttons; buttons only for primary actions; clear link text; limit length, number of welcome views, and number of buttons. Don't: use buttons unnecessarily; use for promotions; use generic "read more" link text — [Views § Welcome Views](https://code.visualstudio.com/api/ux-guidelines/views#welcome-views)
- Mechanics: welcome content applies only to empty tree views (no children and no `TreeView.message`); a command link alone on a line renders as a button; `when` controls visibility; content from core comes first, then built-in extensions, then others — [contributes.viewsWelcome](https://code.visualstudio.com/api/references/contribution-points#contributes.viewsWelcome)
- Codicons are supported in welcome views since 1.95 — [v1.95](https://code.visualstudio.com/updates/v1_95)

**Webviews and webview views**
- "webviews should only be used if you absolutely need them." Do: only when necessary; activate only when contextually appropriate; open only for the active window; make all elements themeable (theme color tokens); follow accessibility guidance (contrast, ARIA labels, keyboard navigation); use command actions in the toolbar and in the view — [Webviews](https://code.visualstudio.com/api/ux-guidelines/webviews)
- Don't: use for promotions; use for wizards; open on every window; open on extension updates (use a notification); add functionality unrelated to the editor/workspace; repeat existing functionality (Welcome page, Settings, configuration) — [Webviews](https://code.visualstudio.com/api/ux-guidelines/webviews)
- Webview views (webviews in a sidebar or panel container) follow the same guidance; example given is the PR extension's "create pull request" form with dropdowns, inputs, buttons — [Webviews](https://code.visualstudio.com/api/ux-guidelines/webviews)
- Endorsed pattern: the GitHub Pull Requests extension shows PRs "in a custom tree view and then uses a webview for a detail view" — [Webviews](https://code.visualstudio.com/api/ux-guidelines/webviews)

**Panel**
- Do: render views that benefit from horizontal space; use for supporting functionality. Don't: use for views meant to be always visible ("users often minimize the Panel"); render webview content that fails to resize/reflow when dragged to the sidebars — [Panel](https://code.visualstudio.com/api/ux-guidelines/panel)
- Panel toolbar: don't add an excessive number of icon buttons (use a context menu); don't duplicate default panel icons — [Panel](https://code.visualstudio.com/api/ux-guidelines/panel)

**Editor actions**
- Do: show only when contextually appropriate; use library icons; use the overflow menu for secondary actions. Don't: add more than one icon; add custom colors; use emojis — [Editor Actions](https://code.visualstudio.com/api/ux-guidelines/editor-actions)
- In `editor/title`, groups `navigation` and `1_run` appear in the primary area; others go under `...` — [contributes.menus](https://code.visualstudio.com/api/references/contribution-points#contributes.menus)

**Status bar**
- Left = workspace-wide items; right = contextual (language, spacing, feedback). "Limit the number of items added." Do: short text labels; icons only when necessary and for clear metaphors. Don't: add custom colors; add more than one icon or more than one item unless necessary — [Status Bar](https://code.visualstudio.com/api/ux-guidelines/status-bar)
- Background progress: status bar item with a (spinning) loading icon; escalate to a progress notification only when attention is needed. Error/warning background colors are "a last resort" — [Status Bar](https://code.visualstudio.com/api/ux-guidelines/status-bar)

**Notifications**
- Decision tree: multi-step input → multi-step quick pick; single immediate input → modal; low-priority progress → status bar; user-triggered → find the right moment; multiple → combine; not needed → show nothing — [Notifications](https://code.visualstudio.com/api/ux-guidelines/notifications)
- Do: notify only when absolutely necessary; add "Do not show again" to every notification; one at a time. Don't: repeat notifications; promote; ask for feedback on first install; show actions if there aren't any — [Notifications](https://code.visualstudio.com/api/ux-guidelines/notifications)
- Progress notifications are "a last resort as progress is best kept within context (within a view or editor)"; provide details link (logs), cancel, timeouts; never leave one running — [Notifications](https://code.visualstudio.com/api/ux-guidelines/notifications)
- Modal dialogs: only for immediate required input; offer Always/Never; don't use to confirm multiple steps, for messages that need no action, or for actions the user didn't initiate — [Notifications](https://code.visualstudio.com/api/ux-guidelines/notifications)

**Quick picks**
- Do: icons for clear metaphors; `description` for current items; `detail` for brief context; multi-step pattern for a series of basic inputs (with "1/3" in the title); offer "create new" in pick lists; use titles for multi-step, no-text-input, text-input, and global-button picks; separators for obvious groups. Don't: repeat existing functionality; title when the placeholder suffices; inputs without a placeholder. Avoid quick picks for long, wizard-like flows — [Quick Picks](https://code.visualstudio.com/api/ux-guidelines/quick-picks)

**Command Palette**
- Do: add keyboard shortcuts where appropriate; clear names; group commands under one `category`. Don't: overwrite existing keybindings; use emojis in command names — [Command Palette](https://code.visualstudio.com/api/ux-guidelines/command-palette)

**Context menus**
- Do: show actions when contextually appropriate; group similar actions; put large groups into a submenu. Don't: show actions for every file without context. File-related actions belong in the Explorer context menu, scoped to the relevant file types — [Context Menus](https://code.visualstudio.com/api/ux-guidelines/context-menus)

**Walkthroughs**
- Do: helpful images that work across themes (SVG with theme colors); give each step an action, using verbs. Don't: too many steps; multiple walkthroughs unless absolutely necessary — [Walkthroughs](https://code.visualstudio.com/api/ux-guidelines/walkthroughs)
- Walkthroughs open automatically on install; steps support `when`; completion events: `onCommand`, `onSettingChanged`, `onContext`, `extensionInstalled`, `onView`, `onLink` — [contributes.walkthroughs](https://code.visualstudio.com/api/references/contribution-points#contributes.walkthroughs)

**Settings**
- Do: default values for each setting; clear descriptions; link to docs, related settings, and setting IDs. Don't: "Create your own settings page/webview"; long descriptions — [Settings](https://code.visualstudio.com/api/ux-guidelines/settings)

**Custom editors**
- There is no dedicated UX guidelines page for custom editors (the file `custom-editors.md` does not exist under `api/ux-guidelines`; the Editor Actions page links to the custom editor guide instead) — [Editor Actions](https://code.visualstudio.com/api/ux-guidelines/editor-actions)
- Extension guide: use `CustomTextEditorProvider` for text formats (VS Code's `TextDocument` is the model, so undo/save/hot exit come for free) and `CustomEditorProvider` for binary formats (extension owns the model) — [Custom Editor API](https://code.visualstudio.com/api/extension-guides/custom-editors)
- `priority: "default"` takes over matching files; `"option"` makes it available via "Reopen With" without being the default — [contributes.customEditors](https://code.visualstudio.com/api/references/contribution-points#contributes.customEditors)

### Inferences
- Ascribe's four views (Projects, Used By, Pages, Content Model) are within the 3-5 limit, but "Used By" behaves like a contextual detail of the active page, and "Projects" may be something a quick pick or status bar item could do; the guideline "content that could be a simple Command" applies.
- Every Ascribe view should have its own `icon` and `contextualTitle`, since users may move views to the Secondary Side Bar or Panel.
- A tree for navigation plus a webview for detail (the PR extension pattern) is the officially sanctioned way to add richer UI.

### Gaps
- No official UX guidelines page exists for custom editors, the Secondary Side Bar contribution point, chat/AI UI, or language status items.
- The UX pages do not give numeric limits beyond "3-5 views", "more than three actions to an item", and "one icon" for editor actions.

## What APIs exist now (stable vs proposed) for tree views, decorations, webview views, editors, language features, diffs, timeline, walkthroughs, settings/profiles, outline/breadcrumbs

### Takeaway
Nearly everything a docs tool needs is stable: tree badges, descriptions, checkboxes, drag and drop, file decorations, `reveal`, webview views, custom (text) editors, notebooks, comments, testing with coverage, language status items, inlay hints, CodeLens, document drop/paste providers, document symbols (outline/breadcrumbs), and `vscode.changes` (multi-diff). Still proposed in October 2026: Markdown labels in tree items, tree "active item" (follow-cursor), Markdown tree messages, `TreeView.reveal` extensions, the Timeline API, standalone quick-diff providers, custom-editor diffs, and the chat output renderer.

### Cited Findings

**Tree views (stable)**
- `TreeView` and `WebviewView` both have `description?: string` and `badge?: ViewBadge` (a number plus tooltip on the view's container icon) — [vscode.d.ts](https://github.com/microsoft/vscode/blob/main/src/vscode-dts/vscode.d.ts)
- `TreeView.message?: string` (plain string in stable); a non-empty message suppresses welcome content — [vscode.d.ts](https://github.com/microsoft/vscode/blob/main/src/vscode-dts/vscode.d.ts), [contributes.viewsWelcome](https://code.visualstudio.com/api/references/contribution-points#contributes.viewsWelcome). A Markdown message is proposed (`treeViewMarkdownMessage`) — [proposed API list](https://github.com/microsoft/vscode/tree/main/src/vscode-dts)
- `TreeView.reveal(element, { select?, focus?, expand?: boolean | number })` requires `getParent` on the provider — [vscode.d.ts](https://github.com/microsoft/vscode/blob/main/src/vscode-dts/vscode.d.ts)
- `TreeViewOptions`: `showCollapseAll`, `canSelectMany`, `dragAndDropController` (`TreeDragAndDropController` with `dragMimeTypes`/`dropMimeTypes`), `manageCheckboxStateManually`; `TreeItem.checkboxState` and `onDidChangeCheckboxState`; `TreeItemLabel.highlights`; `TreeDataProvider.resolveTreeItem` for lazy tooltips/commands; `accessibilityInformation` — [vscode.d.ts](https://github.com/microsoft/vscode/blob/main/src/vscode-dts/vscode.d.ts)
- Tree checkbox API finalized in 1.80; VS Code manages parent/child checked state unless `manageCheckboxStateManually` is set — [v1.80](https://code.visualstudio.com/updates/v1_80)
- View actions: `view/title` (group `navigation` = inline, others in `...`), `view/item/context` (group `inline` = inline icons on the item) — [Tree View API guide](https://code.visualstudio.com/api/extension-guides/tree-view), [contributes.menus](https://code.visualstudio.com/api/references/contribution-points#contributes.menus)
- Filtering: there is no extension filter API for tree views; trees get the built-in Find control, whose default keybinding was changed in 1.89 to `list.find` to reduce accidental opening — [v1.89](https://code.visualstudio.com/updates/v1_89). Extensions typically implement filters via `view/title` commands plus context keys (inferred; see Inferences).
- `<viewId>.open` command (since 1.100) opens a tree view or webview view without focusing it: `{ preserveFocus: boolean }` — [v1.100](https://code.visualstudio.com/updates/v1_100)
- Proposed: `treeItemMarkdownLabel` (`TreeItemLabel2.label: string | MarkdownString`, supporting only codicons and whole-string bold/italic/strikethrough). Announced in 1.106 release notes as "Extension authors can now use MarkdownString in tree view item labels" but the API is still a `vscode.proposed.treeItemMarkdownLabel.d.ts` file on main — [v1.106](https://code.visualstudio.com/updates/v1_106), [proposed d.ts](https://github.com/microsoft/vscode/blob/main/src/vscode-dts/vscode.proposed.treeItemMarkdownLabel.d.ts)
- Proposed: `treeViewActiveItem` (follow-the-cursor style "active item" highlighting) and `treeViewReveal` — [proposed API list](https://github.com/microsoft/vscode/tree/main/src/vscode-dts). Stable `TreeView` has no `activeItem` (the only `activeItems` in vscode.d.ts belongs to `QuickPick`) — [vscode.d.ts](https://github.com/microsoft/vscode/blob/main/src/vscode-dts/vscode.d.ts)

**Decorations, status, language features (stable)**
- `window.registerFileDecorationProvider` (badge letter, color, tooltip on resources in Explorer, tabs, and trees whose items set `resourceUri`) — [vscode.d.ts](https://github.com/microsoft/vscode/blob/main/src/vscode-dts/vscode.d.ts)
- `languages.createLanguageStatusItem` (status shown in the `{}` language status hover next to the language mode, scoped by document selector) — [vscode.d.ts](https://github.com/microsoft/vscode/blob/main/src/vscode-dts/vscode.d.ts); `languageStatusText` is a proposal — [proposed API list](https://github.com/microsoft/vscode/tree/main/src/vscode-dts)
- `languages.registerInlayHintsProvider`, `registerCodeLensProvider`, `registerHoverProvider`, `registerDocumentLinkProvider`, `registerFoldingRangeProvider`, `registerDocumentSymbolProvider` are stable — [vscode.d.ts](https://github.com/microsoft/vscode/blob/main/src/vscode-dts/vscode.d.ts); document symbols are the "Show all Symbol Definitions Within a Document" feature (LSP `textDocument/documentSymbol`) — [Programmatic Language Features](https://code.visualstudio.com/api/language-extensions/programmatic-language-features)
- `languages.registerDocumentDropEditProvider` and `registerDocumentPasteEditProvider` are stable; the paste API was finalized in 1.97 and lets an extension write clipboard metadata on copy, produce custom edits (including creating files) on paste, and offer multiple paste options chosen via the paste control or `editor.pasteAs.preferences`; VS Code's Markdown extension uses it to paste images and URLs as links — [v1.97](https://code.visualstudio.com/updates/v1_97), [vscode.d.ts](https://github.com/microsoft/vscode/blob/main/src/vscode-dts/vscode.d.ts)

**Editors, notebooks, comments, testing**
- `window.registerCustomEditorProvider` (custom and custom text editors), `window.registerWebviewViewProvider`, `notebooks.createNotebookController`, `comments.createCommentController`, `tests.createTestController` are stable — [vscode.d.ts](https://github.com/microsoft/vscode/blob/main/src/vscode-dts/vscode.d.ts)
- Since 1.110, webview panels and custom editors can use a `ThemeIcon` as their tab icon — [v1.110](https://code.visualstudio.com/updates/v1_110)
- Proposed (1.120–1.132): `customEditorDiffs` (`resolveCustomEditorInlineDiff`, `resolveCustomEditorSideBySideDiff`), separate custom-editor priorities for diffs/merges, and priorities by editor mode; this powers the new Markdown preview in diff views — [v1.120](https://code.visualstudio.com/updates/v1_120), [v1.129](https://code.visualstudio.com/updates/v1_129), [v1.132](https://code.visualstudio.com/updates/v1_132)
- File-level comments API arrived in 1.97; comment thread reveal in 1.91; comment thread `collapsibleState` in 1.95 — [v1.97](https://code.visualstudio.com/updates/v1_97), [v1.91](https://code.visualstudio.com/updates/v1_91), [v1.95](https://code.visualstudio.com/updates/v1_95)
- Testing: run profiles of kind Run, Debug, or Coverage; coverage via `run.addCoverage()` with `FileCoverage`; test coverage API finalized during 1.85–1.88, attributable (per-test) coverage in 1.90–1.96 — [Testing API](https://code.visualstudio.com/api/extension-guides/testing), [v1.87](https://code.visualstudio.com/updates/v1_87), [v1.96](https://code.visualstudio.com/updates/v1_96)

**Diffs, timeline, quick diff**
- `vscode.diff` (two resources) and `vscode.changes` (title plus resource list, opens the multi-file "changes" editor) are built-in commands — [Built-in Commands](https://code.visualstudio.com/api/references/commands)
- `QuickDiffProvider` exists in stable only as part of a `SourceControl`; a standalone `quickDiffProvider` proposal, `tabInputMultiDiff`, `contribMultiDiffEditorMenus`, and `timeline` are proposed — [vscode.d.ts](https://github.com/microsoft/vscode/blob/main/src/vscode-dts/vscode.d.ts), [proposed API list](https://github.com/microsoft/vscode/tree/main/src/vscode-dts)

**Quick input**
- Finalized 1.109: quick input button location (buttons to the right of the input) and toggle buttons; 1.106–1.108 proposals: prompt support, file icons for quick pick items, resource URIs; icons in `QuickPickItem` finalized 1.81 — [v1.109](https://code.visualstudio.com/updates/v1_109), [v1.106](https://code.visualstudio.com/updates/v1_106), [v1.108](https://code.visualstudio.com/updates/v1_108), [v1.81](https://code.visualstudio.com/updates/v1_81)

**Walkthroughs, settings, profiles**
- Walkthroughs: `contributes.walkthroughs` with steps (Markdown or image media), `when`, and completion events — [contributes.walkthroughs](https://code.visualstudio.com/api/references/contribution-points#contributes.walkthroughs)
- Settings: `IOpenSettingsActionOptions` gained `revealSetting` and `focusSearch` in 1.81 for opening the Settings editor at a specific setting — [v1.81](https://code.visualstudio.com/updates/v1_81)
- 1.109: the extension editor now shows an extension's configuration defaults — [v1.109](https://code.visualstudio.com/updates/v1_109)

**Markdown-adjacent built-ins added in 2026 (relevant to a docs tool's overlap)**
- 1.119: "Switch to Preview View" / "Switch to Editor View" button and commands; 1.120: Markdown preview for diffs (Preview), via "Reopen Editor With" or `workbench.diffEditorAssociations`; 1.121: Mermaid diagrams in the Markdown preview and YAML frontmatter rendered as a table by default (`markdown.preview.frontMatter`) — [v1.119](https://code.visualstudio.com/updates/v1_119), [v1.120](https://code.visualstudio.com/updates/v1_120), [v1.121](https://code.visualstudio.com/updates/v1_121)

### Inferences
- The Outline view and breadcrumbs consume `DocumentSymbolProvider` output, so an Ascribe document symbol provider (directives, headings, frontmatter keys) gives native outline and breadcrumbs with no custom view. (The docs page cited describes the symbol provider; the Outline/breadcrumb linkage is common knowledge, not quoted from the fetched page.)
- "Used By" could be expressed without a dedicated view: CodeLens ("used by 3 pages"), references (`registerReferenceProvider`), or call-hierarchy-style peek, plus `vscode.changes` for reviewing affected pages.
- Follow-the-active-editor highlighting in a Pages tree must still be built with stable `reveal(...)` on `window.onDidChangeActiveTextEditor`; the proposed `treeViewActiveItem` cannot be used in a Marketplace extension.
- Tree filters must be built as `view/title` toggle commands backed by `setContext` and `TreeView.description` to show the active filter.

### Gaps
- No extension API for Profiles was found in vscode.d.ts (only terminal and test run "profiles"); I found no evidence an extension can read or contribute to profiles.
- I did not confirm whether Timeline is still planned for finalization; it has been proposed since 2020.
- The exact shape of `treeViewActiveItem` and `treeViewReveal` proposals was not read.

## AI integration points for an extension (2025–2026): chat participants, language model tools, MCP registration, prompt files/instructions/skills/agents, agent mode, chat output renderers

### Takeaway
An extension now has five stable ways into chat: language model tools (`vscode.lm.registerTool`, invoked by agent mode or `#name`), MCP server definition providers, a chat participant (`@name`), contributed instructions/prompt files/skills/custom agents (declared in `package.json`), and direct `vscode.lm` model calls. Chat output renderers (webviews inside chat) and chat context providers are still proposed. Microsoft's guidance is to prefer tools over participants and at most one participant per extension.

### Cited Findings
- Four options and when to pick each: Language Model Tool (agent mode auto-invocation, needs VS Code APIs, Marketplace distribution); MCP tool (no VS Code APIs, reusable across clients, local or remote); Chat Participant (own the whole interaction, "extend ask mode"); Language Model API (AI inside your own UI, e.g. code actions, hovers, custom views) — [AI extensibility overview](https://code.visualstudio.com/api/extension-guides/ai/ai-extensibility-overview)
- MCP tools "do not have access to the VS Code extension APIs" — [AI extensibility overview](https://code.visualstudio.com/api/extension-guides/ai/ai-extensibility-overview)
- Tools: declare in `contributes.languageModelTools` (`name` as `{verb}_{noun}` prefixed by extension, `displayName`, `modelDescription`, `userDescription`, `canBeReferencedInPrompt`, `toolReferenceName` for `#name`, `icon`, `inputSchema`, `when`, `tags`), implement with `vscode.lm.registerTool` — [contributes.languageModelTools](https://code.visualstudio.com/api/references/contribution-points#contributes.languageModelTools)
- Tool guidelines: detailed descriptions of what the tool does and when not to use it; a generic confirmation is always shown for extension tools, but tools can customize the message; errors should be messages that make sense to the LLM with next steps — [Language Model Tool API](https://code.visualstudio.com/api/extension-guides/ai/tools)
- Proposed (1.114): fine-grained tool approval scoped to an argument combination (e.g. "Allow reading 'foo.txt'") — [v1.114](https://code.visualstudio.com/updates/v1_114); `toolInvocationApproveCombination`, `contribLanguageModelToolSets`, `toolProgress` are in the proposed list — [proposed API list](https://github.com/microsoft/vscode/tree/main/src/vscode-dts)
- 1.127: "Agent tools are generally available" — [v1.127](https://code.visualstudio.com/updates/v1_127)
- MCP from an extension: contribute `contributes.mcpServerDefinitionProviders` (`id`, `label`) and call `vscode.lm.registerMcpServerDefinitionProvider` with `onDidChangeMcpServerDefinitions`, `provideMcpServerDefinitions`, `resolveMcpServerDefinition` (for auth/user interaction); definitions are `McpStdioServerDefinition` or `McpHttpServerDefinition` — [MCP developer guide](https://code.visualstudio.com/api/extension-guides/ai/mcp). Introduced in 1.100, extension APIs in 1.101 — [v1.100](https://code.visualstudio.com/updates/v1_100), [v1.101](https://code.visualstudio.com/updates/v1_101)
- VS Code supports MCP stdio, Streamable HTTP, legacy SSE; tools, prompts (as slash commands), resources (attachable as chat context), elicitation, sampling, OAuth, server instructions, roots, and MCP Apps — [MCP developer guide](https://code.visualstudio.com/api/extension-guides/ai/mcp)
- MCP Apps: tools return `_meta.ui.resourceUri` pointing to a `ui://` resource of MIME `text/html;profile=mcp-app` rendered in a sandboxed iframe inline in chat (VS Code supports `inline` display only; "send message" fills the input but doesn't auto-send); announced January 2026 — [MCP developer guide](https://code.visualstudio.com/api/extension-guides/ai/mcp), [MCP Apps blog post](https://code.visualstudio.com/blogs/2026/01/26/mcp-apps-support)
- MCP naming: tools `{verb}_{noun}` snake case; resources title case; prompts camelCase — [MCP developer guide](https://code.visualstudio.com/api/extension-guides/ai/mcp)
- Chat participant: `@name` + `/command`, responds via `ChatResponseStream` (Markdown, command buttons, file trees, progress, references, inline references), follow-ups, participant detection — [Chat Participant API](https://code.visualstudio.com/api/extension-guides/ai/chat)
- Participant guidance: "It doesn't make sense for every extension to contribute a chat participant… we discourage extensions from contributing multiple chat participants. Up to one chat participant per extension"; participants "should not be purely question-answering bots"; language extensions should rather contribute tools (e.g. `#cpp`) and smart actions; ask consent before costly or irreversible operations; don't call it "participant" in UI — [Chat Participant API](https://code.visualstudio.com/api/extension-guides/ai/chat)
- If the extension does other things, don't add an extension dependency on GitHub Copilot — [Chat Participant API](https://code.visualstudio.com/api/extension-guides/ai/chat)
- Static customizations by contribution point (each takes `path` plus optional `when`): `chatInstructions` (`*.instructions.md`, applied automatically when relevant), `chatPromptFiles` (`*.prompt.md`, invoked as slash commands), `chatSkills` (`SKILL.md` per the Agent Skills spec; folder name must match `name`), `chatAgents` (`*.agent.md` custom agents with tools/model in frontmatter; optional `sessionTypes`) — [contribution points](https://code.visualstudio.com/api/references/contribution-points#contributes.chatInstructions). Prompt/instructions contributions shipped in 1.105 — [v1.105](https://code.visualstudio.com/updates/v1_105)
- `contributes.languageModelChatProviders` + `vscode.lm.registerLanguageModelChatProvider` lets an extension supply models (1.104; configuration schema with `secret` fields in 1.109) — [contribution points](https://code.visualstudio.com/api/references/contribution-points#contributes.languageModelChatProviders), [v1.109](https://code.visualstudio.com/updates/v1_109)
- Chat output renderer (proposed, `chatOutputRenderer`): a tool returns data with a MIME type; `vscode.chat.registerChatOutputRenderer(viewType, renderer)` renders it in a webview inside the chat response (sample renders Mermaid). Introduced 1.103, updated 1.109; still proposed on main — [v1.103](https://code.visualstudio.com/updates/v1_103), [v1.109](https://code.visualstudio.com/updates/v1_109), [proposed d.ts](https://github.com/microsoft/vscode/blob/main/src/vscode-dts/vscode.proposed.chatOutputRenderer.d.ts)
- Chat context provider (proposed, 1.107): extensions contribute workspace, implicit (active webview), and explicit ("Add Context") context; the GitHub PR extension is the example — [v1.107](https://code.visualstudio.com/updates/v1_107)
- Other chat proposals on main: `chatSessionsProvider`, `chatSessionCustomizationProvider`, `chatHooks`, `chatStatusItem`, `chatParticipantAdditions`, `defaultChatParticipant`, `mcpServerDefinitions`, `mcpToolDefinitions`, `remoteCodingAgents`, `agentSessionsWorkspace` — [proposed API list](https://github.com/microsoft/vscode/tree/main/src/vscode-dts)
- Agent plugins: experimental in 1.110 (bundles of skills, commands, agents, MCP servers, hooks; installable from the Extensions view via `@agentPlugins`); 1.133 adopted the open Agent Plugins 1.0 standard (`plugin.json`, `skills/`, `mcp.json`), launched August 6, 2026 — [v1.110](https://code.visualstudio.com/updates/v1_110), [v1.133](https://code.visualstudio.com/updates/v1_133)
- Chat Customizations editor (1.113, Preview) centralizes instructions, prompt files, agents, skills, MCP servers, plugins — [v1.113](https://code.visualstudio.com/updates/v1_113)
- Agents window / "Visual Studio Code Agents" companion app (1.115 Preview onward) runs parallel agent sessions; customizations (instructions, prompt files, agents, MCP servers, hooks, plugins) carry over — [v1.115](https://code.visualstudio.com/updates/v1_115)

### Inferences
- For Ascribe, the lowest-cost, highest-reach combination is: (1) an MCP server definition provider that launches the existing `ascribe` binary's MCP mode, so the same tools work in Copilot, Claude, and other clients; (2) a few `languageModelTools` only where VS Code APIs matter (active editor, diagnostics, open preview); (3) `chatSkills`/`chatInstructions` that ship the existing Ascribe skill content with `when` clauses keyed to `ascribe.toml` workspaces. A chat participant is optional and, per guidance, at most one.
- MCP Apps or the (proposed) chat output renderer could show rendered page previews or content-model diagrams inline in chat; only MCP Apps is usable in a Marketplace extension today.

### Gaps
- I found no authoritative write-up of how specific docs-as-code extensions (e.g. Microsoft Learn Authoring Pack, Mintlify, Docusaurus, Astro) use chat participants or tools; search results were generic. This needs a separate survey of Marketplace extensions.
- The exact finalization release for `vscode.lm.registerTool` and MCP provider APIs was not pinned (they appear in stable vscode.d.ts; tools were proposed during 1.91–1.94).

## Guidance and consensus on view sprawl, moving views between sidebars, user customization, and performance limits

### Takeaway
Official guidance treats view count as a shared, scarce budget (one container, 3-5 views, every view needs an icon because users will move it). Users have full control: drag views or whole containers between Primary Side Bar, Secondary Side Bar, and Panel, hide views and toolbar actions, and reset locations. Since 2025 the Secondary Side Bar defaults to showing Chat, which competes for the same space; extensions can now contribute containers there directly. Webviews carry memory cost; trees are virtualized.

### Cited Findings
- Guidance on sprawl: one view container is "generally enough"; 3-5 views "is a comfortable max"; "Limit the number of Views created as other extensions can contribute in the same View Container"; "Keep the number of Views to a minimum" — [Sidebars](https://code.visualstudio.com/api/ux-guidelines/sidebars), [Views](https://code.visualstudio.com/api/ux-guidelines/views)
- Users can drag and drop views and panels between Primary and Secondary Side Bars and Panel; VS Code remembers the layout; "Reset Location" per view and **View: Reset View Locations**; keyboard via **View: Move View** / **Move Focused View**; views can be grouped by dropping on another container's Activity Bar item; toolbar actions can be hidden via right-click **Hide** — [Custom Layout](https://code.visualstudio.com/docs/configure/custom-layout)
- Activity Bar position can be Default, Top, Bottom, or Hidden — [Custom Layout](https://code.visualstudio.com/docs/configure/custom-layout)
- Secondary Side Bar: hidden by default until 1.100 added `workbench.secondarySideBar.defaultVisibility`; by 1.104 the Secondary Side Bar with the Chat view is visible by default when a workspace is first opened; current docs: shown by default for folders/workspaces, hidden in empty windows — [v1.100](https://code.visualstudio.com/updates/v1_100), [v1.104](https://code.visualstudio.com/updates/v1_104), [Custom Layout](https://code.visualstudio.com/docs/configure/custom-layout)
- Extensions can contribute view containers to `secondarySidebar` (proposed 1.104, released 1.106) "alongside built-in views like Chat" — [v1.106](https://code.visualstudio.com/updates/v1_106)
- 1.124 "Widen the editor with the side bar chevron"; 1.129 Modern UI preview (`workbench.experimental.modernUI`); 1.136 and 1.141 a Compact layout density (`window.density.layout`) — [v1.124](https://code.visualstudio.com/updates/v1_124), [v1.129](https://code.visualstudio.com/updates/v1_129), [v1.136](https://code.visualstudio.com/updates/v1_136), [v1.141](https://code.visualstudio.com/updates/v1_141)
- Webview performance: `getState`/`setState` "have much lower performance overhead than `retainContextWhenHidden`", which "has high memory overhead and should only be used when other persistence techniques will not work"; use `WebviewPanelSerializer` to restore across restarts — [Webview API](https://code.visualstudio.com/api/extension-guides/webview)
- 1.118 streams large local resources to webviews in chunks (faster, lower memory), benefiting webviews and custom editors — [v1.118](https://code.visualstudio.com/updates/v1_118)
- Panel webviews must reflow when moved to the sidebars — [Panel](https://code.visualstudio.com/api/ux-guidelines/panel)
- VS Code's list/tree widget is a virtual renderer that keeps only visible rows in the DOM — [Lists And Trees wiki](https://github.com/microsoft/vscode/wiki/Lists-And-Trees)

### Inferences
- Rendering large trees is cheap; the cost is in the extension's `getChildren` work and in refreshes (`onDidChangeTreeData` firing for the root). Lazy children, firing change events for specific elements, and `resolveTreeItem` for tooltips are the levers.
- A redesign should assume some users will put Ascribe in the Secondary Side Bar next to Chat (or move it there), so views must work narrow and with icon-only headers; contributing directly to `secondarySidebar` is now possible but places Ascribe in competition with Chat by default.
- Consolidating to fewer views (e.g. one tree with sections, or one tree plus contextual surfaces such as CodeLens, the Outline, and the language status item) matches the guidance better than adding more.

### Gaps
- No official numeric performance limit for tree views (item counts, refresh rates) was found; the search returned no Microsoft guidance on `getChildren` paging.
- I found no recent community survey or GitHub discussion quantifying "view sprawl" complaints; evidence is the guidelines themselves.

## Recent release notes (2024–2026) that changed extension UI options

### Takeaway
The main UI-relevant API changes since 2024: Secondary Side Bar containers (1.106), Markdown tree labels (proposed), webview/custom editor ThemeIcon tabs (1.110), document paste API (1.97), quick input button location and toggles (1.109), `<view>.open` without focus (1.100), and a wave of chat/AI contribution points (tools, MCP providers, prompt/instruction/skill/agent files, model providers). The workbench itself moved toward agents (Chat in Secondary Side Bar by default, Agents window, plugins) and a modern/compact UI.

### Cited Findings
- 1.80 (Jul 2023): tree checkbox API final — [v1.80](https://code.visualstudio.com/updates/v1_80)
- 1.81: icons in `QuickPickItem` final; `revealSetting`/`focusSearch` for opening settings — [v1.81](https://code.visualstudio.com/updates/v1_81)
- 1.85–1.88: test coverage API; chat agents proposals; multi-document highlight API — [v1.85](https://code.visualstudio.com/updates/v1_85), [v1.87](https://code.visualstudio.com/updates/v1_87)
- 1.90–1.95: chat participant and language model APIs finalized (1.90), tools for language models (1.91–1.95), participant detection, codicons in welcome views (1.95) — [v1.90](https://code.visualstudio.com/updates/v1_90), [v1.95](https://code.visualstudio.com/updates/v1_95)
- 1.97 (Feb 2025): document paste API final; file-level comments — [v1.97](https://code.visualstudio.com/updates/v1_97)
- 1.100 (May 2025): MCP servers contributed by extensions; Secondary Side Bar default visibility setting; `<viewId>.open` without focus — [v1.100](https://code.visualstudio.com/updates/v1_100)
- 1.103: chat output renderer (proposed); 1.104: Chat view visible by default, language model chat provider API, secondary side bar containers (proposed) — [v1.103](https://code.visualstudio.com/updates/v1_103), [v1.104](https://code.visualstudio.com/updates/v1_104)
- 1.105: prompt and instructions file contributions — [v1.105](https://code.visualstudio.com/updates/v1_105)
- 1.106 (Nov 2025): `secondarySidebar` view containers; quick pick toggle/prompt/file-icon proposals; GitHub-style alerts in `MarkdownString`; MarkdownString tree labels (proposed `treeItemMarkdownLabel`) — [v1.106](https://code.visualstudio.com/updates/v1_106)
- 1.107: chat context provider proposal — [v1.107](https://code.visualstudio.com/updates/v1_107)
- 1.109 (Feb 2026): quick input button location and toggle APIs final; chat prompt files API, chat item controller, chat output renderer updates (proposed) — [v1.109](https://code.visualstudio.com/updates/v1_109)
- 1.110 (Mar 2026): ThemeIcon tab icons for webviews/custom editors; agent plugins (experimental); weekly releases begin — [v1.110](https://code.visualstudio.com/updates/v1_110)
- 1.114: fine-grained tool approval (proposed) — [v1.114](https://code.visualstudio.com/updates/v1_114)
- 1.118: chunked webview resource loading — [v1.118](https://code.visualstudio.com/updates/v1_118)
- 1.120–1.132: custom editor diffs and per-mode custom editor priorities (proposed); Markdown preview for diffs — [v1.120](https://code.visualstudio.com/updates/v1_120), [v1.132](https://code.visualstudio.com/updates/v1_132)
- 1.129: Modern UI preview (experimental); 1.136/1.141: compact layout density — [v1.129](https://code.visualstudio.com/updates/v1_129), [v1.141](https://code.visualstudio.com/updates/v1_141)
- 1.133 (Aug 2026): Agent Plugins 1.0 standard — [v1.133](https://code.visualstudio.com/updates/v1_133)

### Inferences
- The direction of travel is that VS Code's own chrome is getting denser and agent-centric; an extension that adds many always-visible views fights both the guidance and the default layout.
- Because Markdown tree labels and active-item tracking remain proposed, a redesign targeting the Marketplace should plan on plain labels plus `description`, `FileDecoration`, and codicons.

### Gaps
- Weekly release notes after 1.110 use inconsistent section headings, so a few extension API changes may have been missed by heading-based scanning (1.125's "Extension Authoring" section content and 1.137's "Contributions to extensions" were not read in full).
