# Proposal: This page as a webview

A proposal, not a plan. It says what the change is and what it would look like, so it can be judged before anything is designed in detail. It answers the question [decision #544a](../decisions/544a-sidebar-views.md) left for after the sidebar's views were in use: whether any of them should be a webview, This page first. The [brainstorm](../brainstorm.md#21-the-editors-views-whats-left) kept the question.

## The goal

This page says everything it knows about a page in full, at any width the sidebar has, and lets the author open what it names without losing their place in the page.

A tree can't. VS Code's tree views draw each row on one line and cut it off with "…", and no setting wraps them. What's cut is often what matters. A tested example's row is "Example" and then `code:crates/ascribe-cli/tests/output/check.json, changed since it was tested`, and at a sidebar's usual width the reader sees the start of the address and not the state. The tooltip has it, for someone who already knows to hover.

## What exists

- **This page is a tree view,** `ascribe.thisPage`, made in `packages/vscode/src/ui/sidebarViews.ts` with `createTreeView`. It's drawn from one request, `ascribe/page`, asked on each cursor move, and asks nothing while it's hidden or collapsed.
- **Its rows are built by pure functions** in `packages/vscode/src/ui/sidebar.ts`: `thisPageRoots`, `thisPageChildren`, and `thisPageLook` turn the server's answer into sections and rows, each with a label, a value, a tooltip, an icon, whether it warns, and what choosing it opens. `ThisPageApi.items()` reads them, and the unit and integration tests go through it.
- **Its sections:** About, Review while review is on, Health, Connections, and Memos. Every one starts open, which makes the view long. Choosing an include opens it in place of the page.
- **It uses little of the tree API.** No item has a menu, the view has no Collapse All, and no item is a file VS Code decorates. Pages and Content model use all three, and stay trees.
- **The extension has one webview already,** the preview: its scripts and stylesheets are built by `packages/vscode/esbuild.mjs` into `dist/webview/` from `src/webview/`, with their own `tsconfig.webview.json`; its HTML and content security policy are made in `src/preview/html.ts`; and it keeps its state with `getState` and `setState`.
- **The extension already copes with a webview taking focus.** VS Code reports no active text editor while a webview has focus, so Insert at the Cursor writes into the project's last Markdown editor, which `LastEditors` tracks. It's in `src/ui/insertAt.ts`, and `sidebarViews.ts` imports it. The sidebar's project follows the same way: `followedProject` in `sidebar.ts` keeps the project it shows when the active file belongs to none.
- **The preview's tests already wait for a webview to draw.** The extension host can't read a webview's document, so the preview's webview posts `rendered` (`src/preview/protocol.ts`) once the page is in its document, with a report of what it drew, and the tests wait for that message.
- **Icons are codicons** ([decision 50](../decisions.md)), drawn by the tree as `ThemeIcon`s.

A mockup of the view, kept locally with the other mockups (`.gitignore` leaves that folder out), shows Ascribe's own `checks/check.md` in Dark Modern and Light Modern, beside today's tree at the same widths.

## What it looks like

This page becomes a webview view: `"type": "webview"` in the manifest, and a `WebviewViewProvider` in the extension. Only This page changes. Review, Pages, and Content model stay native trees in the same container; each view in a container is a tree or a webview on its own.

- **A head:** the page's title, with its code spans drawn as code, its type, and its path.
- **The sections,** in today's order and with today's rows. Only About starts open: Review, Health, Connections, and Memos start closed. A closed section's header still says whether to open it: Health's count of things to do, Connections' count of links, Memos' "none".
- **Rows wrap,** and lead with what they say. A tested example's row is its state first, as a colored label (tested, or changed since it was tested), then its address, then what it was tested with and the heading it's under. Every row whose state can warn does the same.
- **Includes open beside the page:** an include, a snippet, or a tested example opens in the editor group beside the page's, with `preserveFocus`. The focus stays where the click left it, in the view, and This page stays on the page, since it follows the page's editor, not the focus (see "Which page it shows").
- **The view's badge** shows Health's count of things to do, so it's seen with This page collapsed, and with Health closed, as it is until someone opens it.

The rows are a straight port: the same facts, from the same request. No new action, button, or meter comes with this change; each is a decision of its own, later.

## What the tree gave, and what the webview keeps

A tree gets these from VS Code. A webview gets none of them, so each was weighed:

1. **Keyboard:** Tab between section headers and rows, and Enter or Space to toggle a section or open a row, which the HTML gives; and Up and Down between rows, with a roving focus. Left, Right, Home, and End are left out. The view doesn't trap the focus: VS Code's own commands, such as **View: Focus Active Editor Group** (`workbench.action.focusActiveEditorGroup`), leave it as they leave any view, and Escape anywhere in the view puts the focus back in the page's editor.
2. **Screen readers:** through the HTML's own meaning: a `<details>` and `<summary>` for each section, a button for each row, and a description list for About's rows. Not ARIA's tree roles, which promise the whole of a tree's keyboard.
3. **Type to find:** left out. The view is one page's facts, short enough to scan.
4. **Which sections are open** is kept with `setState`: one state for each section, the same for every page, so someone who opens Health once has it open on every page after, and closing it closes it everywhere. Until then the badge carries its count.
5. **Hover text** is a `title` on each row's label. Where an explanation matters, it moves into the row, which can now hold it.
6. **Icons** are the codicon font, `@vscode/codicons`, a new dependency, loaded from the extension's files. A section with nothing in it is dimmed by a class. It's an icon font, which decision 50 already has the extension ship, read from disk and never downloaded, so decision 48's rule against web fonts, which is about a reader downloading a font for text, doesn't reach it; the decision file says so.
7. **Colors** are only `--vscode-*` variables, which VS Code sets in every webview for the current theme, high contrast included, with its borders and focus outlines. The color test in `scripts/design/tokens.test.ts` reads the stylesheets `design/tokens.toml`'s `[emit]` names, so this one is added there, and a color written in it fails. That includes a fallback: the test reads `var(--x, #fff)` as a color. The preview's stylesheet has fallbacks, generated from the tokens, for the moments before VS Code's variables arrive; this one has none, since a webview view is always given them.
8. **The message above the items** (no page open, the server's state) is a paragraph.
9. **Choosing a row opens it,** as today, and includes beside the page. No row stays highlighted: there's nothing to select.
10. **Showing it again should be instant enough:** the extension keeps the last answer and sends it when the webview has loaded, instead of keeping a hidden webview alive (`retainContextWhenHidden`), which costs memory for every window. The thread measures it once: the time from expanding the view to its rows drawn, on Ascribe's docs, against the tree's. If the delay is one a person sees, `retainContextWhenHidden` is the fallback, and the trade is that memory.
11. **Section headers stick** to the top while their section scrolls (`position: sticky`). No indent guides: the rows are one level deep.
12. **The tests keep their API.** `thisPageRoots` and `thisPageLook` stay the model, and the webview only draws what they build, so `ThisPageApi.items()` and the unit tests read the model as today. An integration test checks that the webview loads, receives the answer, and draws it, in the preview's way: the webview posts a message once the rows are in its document, with what it drew, and the test waits for it.

What VS Code still gives a webview view for nothing: its title, description, and badge; **Focus on This Page View** in the Command Palette; and its place, size, and collapsing in the container.

## What a webview changes

- **A product icon theme isn't followed.** The tree's icons follow one; a bundled codicon font doesn't. Few people install one, and the icons are still VS Code's own.
- **Every state is the webview's to draw:** no page open, a fragment, the server starting, the server failed, and Restricted Mode. Each gets a test, since they're what's forgotten.
- **Code spans in a title are drawn as code,** and the webview parses nothing to do it. The server sends the title as written, backticks and all, and the tree showed them. `ascribe/page` gains the title's formatted form too, as the JSON output writes a title: a list of text and code pieces. The webview makes an element of each, so text people wrote is never read as markup in the view.
- **There are no `<a>` elements.** VS Code stops a webview from navigating: a click on an `http` or `https` link opens it in the browser. Every row opens something in the editor, which only a message to the extension can do, so each is a button that sends one, and the extension opens it, after checking the message names an action it knows and an address the server's answer gave.
- **Text from pages is set as text.** Titles, memos' first lines, and review threads' excerpts are written by people and agents, so none is put into the page as HTML. The content security policy has a nonce, allows the extension's own scripts, stylesheets, and font, and nothing inline.
- **There's no find:** a webview view has no find widget. Its text can be selected and copied, which a tree's descriptions can't.

## Building it, and what to watch

- **The webview is made late and thrown away often.** VS Code calls the provider when the view is first shown, and disposes of the webview when the view is collapsed or the sidebar shows another container. A message sent before its script has run is lost. So the webview says it's ready, and only then does the extension send it an answer: the newest it has when the ready message arrives, not the one it had when the webview was made, since the cursor may have moved in between. Every ready gets an answer, so a webview that reloads is drawn again. Each answer carries a number that rises, and the webview ignores one older than what it has drawn. One test covers the answer changing between the webview's creation and its ready message. Open sections and the scroll position come back from `getState`.
- **Clicking the view takes the focus from the editor.** While the webview has focus, VS Code reports no active text editor, and a view that follows the active editor would go empty the moment it's clicked. Opening beside is relative to the page's editor group, not to whatever is active.
- **An update keeps the reader's place.** The answer changes as the cursor moves, and the problems change as the server checks. Drawing the whole view again would jump its scroll and drop the keyboard's focus, so rows have stable keys and only what changed is drawn again.
- **It appears a moment later than a tree:** a webview is a page in a frame. Its script is small, with no framework, and it draws the section headers at once so the view never flashes empty.
- **Cursor moves stay one request each,** as today, and a hidden view still asks nothing.

### Which page it shows

This page shows the page of the last Markdown editor that was active, with a page of a project in it, as `LastEditors` tracks for Insert at the Cursor. Only a Markdown editor of a page becoming active moves it.

- **Focus leaving the editor doesn't move it:** to the view itself, another view, a panel, or an editor that isn't a page. It stays on its page, as `followedProject` stays on its project when the active file belongs to none.
- **A page of another project** moves it when that page's editor becomes active, and not when it's only visible in another group.
- **When the page's editor is closed,** it shows the page of the editor VS Code makes active, if that's a page, and otherwise "Open a page to see what's known about it.", the message the tree shows today.

The thread's first test asserts this rule: click in the view, open an include beside, and the view still shows the page, with the focus in the view.

## Alongside

**Pages starts with every section closed.** It's a tree, and this is one line in `pagesLook` (`expanded`), so it ships on its own, before this work. Opening a page still reveals it in Pages, which opens the sections above it, so the open page's path is the one thing shown. It changes what a new user sees first in Pages, so it comes with its line in `changes/editor/` and a sentence in the docs' sidebar page.

## What changes besides the code

- **Decision #544a** says the views are native trees and that the webview question waits. A decision file, added in the pull request that makes the change, says This page is a webview and why, and that the others stay trees.
- **The docs:** "This page, Pages, and Content model" in `docs/content/editor/sidebar.md`: which sections start open, that includes open beside, and that the rows wrap. The walkthrough's sidebar step and the extension's README, where they describe This page. An entry in `changes/editor/`.
- **The dependency:** `@vscode/codicons`, in the extension's package and its notices.

## What this leaves out

- **The Review view,** whose thread excerpts are cut off too. This page goes first; once it's been used, Review can follow if it should.
- **Pages and Content model as webviews.** They use what a tree does well: dragging, revealing, file decorations, and menus.
- **New actions in This page,** such as running `ascribe observe` from a changed example, adding a memo, or copying an address. Each is a decision of its own.
- **Type to find,** a product icon theme, and Left, Right, Home, and End.

## The thread

For a later plan, by the repository's [steel threads](../../.claude/skills/steel-threads/SKILL.md) skill. This isn't the plan.

- **The layers:** the extension's provider and its messages (`packages/vscode/src/ui/`), the webview's script and stylesheet (`packages/vscode/src/webview/`, built by `esbuild.mjs`), the model (`sidebar.ts`, unchanged in what it builds), the manifest, the color test, and the docs. The language server's `ascribe/page` (`crates/ascribe-lsp/src/page.rs`) gains the title's formatted form, through `shapes.rs` to the extension's types, when the head is drawn.
- **The riskiest seams** are focus and the handshake: a view that follows the page's editor, drawn by a webview that takes the focus when it's clicked, opening a file beside an editor that isn't active any more; and an answer that has to reach a webview made late, thrown away, and made again. If This page can't stay on the page while the author uses it, or shows an old answer after it's opened again, the design changes before any section is drawn.
- **The thinnest use case:** in `examples/quill`, with `docs/install-agent.md` open, This page is the webview, with About open and its rows wrapped. Click in the view: it stays on the page ("Which page it shows"). Collapse it, move the cursor into another page, and open it again: it comes back with About open, showing the other page. The thread also measures how long the view takes to draw on Ascribe's docs, against the tree (point 10).
- **Left for the widening, in order of risk:** opening beside, and every row's action; updating in place as the cursor moves, with the keyboard's focus kept; the other sections, and Review's; the head's formatted title; the states (no page, a fragment, the server starting or failed, Restricted Mode); the keyboard, Escape, and screen readers; the badge; the docs and the decision.

## Settled

Answered by the maintainer on 20 October 2026, after using the views and looking at the mockup: This page becomes a webview, and the others stay trees; only About starts open; includes open beside the page; and each of the twelve things the tree gave is kept or left out as "What the tree gave" says, with what "What a webview changes" lists accepted. _Why:_ a tree can't wrap, and what it cuts off is what an author needs to see; This page uses almost nothing else a tree gives, and what it does use is cheap to build again.
