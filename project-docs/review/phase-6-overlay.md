# Phase 6: The overlay, in the page preview

Part of [Review](README.md). Requires phases 4 and 5. TypeScript: `packages/review` (browser part) and `packages/vscode`.

## Goal

In the page preview, the pull request's threads sit beside the blocks they refer to. A reviewer reads them, replies, resolves, comments on any block, and submits the review. The same threads show in the source editor.

## Context

- `packages/review`: the marks (phase 3) and `ReviewSession` (phase 5).
- `packages/vscode/src/preview/` and `packages/vscode/src/webview/preview.ts`: the webview, its message protocol (`protocol.ts`), and its content security policy (`html.ts`).
- Phase 4's review state and commands.
- [`mockup.html`](mockup.html): the reference for the overlay and for everything in this phase. Build from it, at both widths: threads in a column beside the content, each aligned to its block, with both ends highlighted and a connecting line on hover and focus; count markers that open a panel when there's no room for the column; the thread card (location, badges, actions on their own row, comments, the reply box); the composer and its notices; the unsent bar and the submit dialog; the all-comments panel; the detached list at the top; and threads on source lines in the editor. Its **State** menu shows "Not signed in", "Checkout is behind", and "Local commits not pushed".
- VS Code: `vscode.authentication.getSession("github", scopes, { createIfNone })` and which scope posting review comments needs; `vscode.comments.createCommentController` for threads in the source editor; `vscode.extensions.getExtension("GitHub.vscode-pull-request-github")`.

## Design

### The overlay (`@ascribed/review/overlay`)

A framework-free browser component, so it runs in a webview and on any site. It takes a root element and a data source:

```ts
interface OverlayHost {
  load(): Promise<{ changes: Change[]; threads: PlacedThreads; pending: PendingReview; viewer: string }>;
  comment(anchor: Anchor, body: string): Promise<Thread>;
  reply(threadId: string, body: string, when: "now" | "withReview"): Promise<Comment>;
  allThreads(): Promise<ThreadSummary[]>;   // every thread on the pull request's Ascribe pages, with the pages each is on
  resolve(threadId: string, resolved: boolean): Promise<void>;
  submit(event: ReviewEvent, body?: string): Promise<void>;
  openSource(anchor: Anchor): void;
  onDidChange(listener: () => void): void;   // the page re-rendered, or threads refreshed
}
```

- **Threads** show in a column beside the content at wide widths, and as markers that open a panel at narrow widths. Each thread is tied to its block (a line connects them on hover and focus). Resolved threads are collapsed; outdated threads are labeled.
- **Detached threads** (no block on this page) are listed at the top, labelled **Detached**, with the text they were on (GitHub supplies the diff hunk). **Outdated** is a different state: the block is there but its text changed since the comment. An outdated thread stays beside its block, labelled, with a way to see the original text.
- **Replying:** each thread's reply box offers **Reply now** and **Add to review**. While there are unsent comments, **Reply now** is disabled and says why: GitHub adds every reply to the review until it's submitted or discarded (the session rejects a reply sent now with `reply-held`).
- **All comments:** a panel listing every thread on the pull request's pages, with a filter (open, resolved, detached, unsent) and counts. Each entry names its page and block and jumps to it, opening another page if needed. It's the one place a keyboard or screen reader user can reach every thread without walking the page.
- **Hidden content:** opening a thread whose block is inside an unselected tab or a closed `details` reveals it first.
- **Commenting:** hovering or focusing a block shows a comment button; the composer is plain Markdown text with **Add to review** and **Cancel**. A block GitHub can't anchor (its file isn't in the pull request, or its lines are away from the diff's changes) says the comment will go on the pull request's conversation when the review is submitted.
- **Unsent comments** show as a bar: "2 unsent comments · only you can see them" and **Submit review…**. Submitting opens a dialog that lists each unsent comment with a link to its block, the choice (comment, approve, or request changes), and an optional summary. **Discard** is in that dialog only, and confirms with the count and outcome-named buttons ("Discard 2 unsent comments?", **Discard comments**, **Keep review**).
- **Resolve and reopen** act at once; say so where the reviewer chooses them.
- **Comment bodies are untrusted.** Render them as Markdown to a safe subset (text, emphasis, code, links with `rel="noopener noreferrer"`, lists, block quotes; no raw HTML, no images loaded from the network without a click), by building DOM nodes, never by assigning HTML strings.
- **Accessible:** everything works from the keyboard, threads are reachable in reading order, and state changes are announced.
- It isolates its styles from the page's (a shadow root), and takes light or dark from its host.

### In the page preview

- When review is on (phase 4) and the checkout has a pull request, the extension makes a `ReviewSession` with `tokenTransport`, asking VS Code's GitHub sign-in for a session only when the user turns review on (never at startup). If the user declines, the changes still show, without threads, with a **Sign in to see comments** button. If `gh` is signed in and VS Code isn't, offer `ghTransport`.
- **Start Review** now defaults the base to the pull request's base when there is one.
- The webview hosts the overlay; `OverlayHost` is implemented over the webview's message protocol. All GitHub access stays in the extension host: the webview never sees a token and makes no network requests (keep the content security policy as strict as it is).
- If local `HEAD` isn't the pull request's head commit, the header says so (behind: **Pull** to see the latest; ahead: comments can be made only on pushed lines).
- Threads refresh when review starts, on **Refresh** (an editor title action beside phase 4's **Changed Pages**), and after the user's own actions. No polling.

### In the source editor

A comment controller shows the same placed threads on the source lines, with reply and resolve, and a comment action on changed files that adds to the same pending review. One `ReviewSession` backs both views, so a reply in one appears in the other.

If the GitHub Pull Requests extension is installed and active, it already shows these threads in the source editor: don't add a second set. A setting, `ascribe.review.sourceComments` (`auto`, `on`, `off`; default `auto`), decides.

## Tasks

1. The overlay, with unit tests (vitest, jsdom) for placement in the DOM, detached and outdated threads, revealing hidden content, the composer, replying now and with the review, the unsent bar and submit dialog, the all-comments panel and its filters, and the Markdown subset (including that script, raw HTML, and `javascript:` links are inert).
2. The session, sign-in, and webview wiring in the extension, with unit tests against a fake session and an integration test using a fake transport: threads appear on the right blocks, a comment reaches the fake's pending review, and submit sends it.
3. The source editor's comment controller and its setting, with tests for `auto`.
4. A manual pass on a real pull request in a scratch repository: every action, in light and dark, by keyboard only. Say in the pull request what you checked.
5. `docs/editor.md`, `docs/review.md`, the extension's `package.json` (commands, settings), and `CHANGELOG.md`.

## Out of scope

The site preview (phase 7); editing or deleting comments; suggestions; notifications of new comments.

## Acceptance criteria

- A thread made on GitHub shows beside its block in the preview and on its line in the editor; a comment made in the preview shows on GitHub, on the right file and line, after submit and not before; a reply sent with **Reply now** shows at once.
- The webview makes no network requests and never holds a token.
- Review works with no sign-in (changes only), and says how to get comments.

## Verify

```sh
pnpm --filter @ascribed/review test
pnpm --filter ascribe-vscode test
cargo build -p tessera-cli && ASCRIBE_BIN=$PWD/target/debug/ascribe pnpm --filter ascribe-vscode test:integration
pnpm typecheck && pnpm lint && pnpm format:check
```

## Commits

1. "Add the review overlay"
2. "Show and make review comments in the page preview"
3. "Show review threads on source lines"
