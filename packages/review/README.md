# @ascribed/review

Reviewing [Ascribe](https://github.com/ascribed-dev/ascribe) pages as readers see them: what a change does to each rendered page, and a pull request's review threads beside the blocks they're about.

This README is for building a host. To review a pull request with Ascribe, read [Review](https://ascribed-dev.com/guides/review/).

Each entry point is separate, so a host bundles only what it uses.

| Entry point | Runs in | What it's for |
|---|---|---|
| `@ascribed/review/marks` | the browser | Marking the changed blocks of a rendered page, from `ascribe diff --format json`'s changes |
| `@ascribed/review/overlay` | the browser | The review overlay: a pull request's review threads beside a rendered page's blocks |
| `@ascribed/review/github` | Node | Reading and writing a pull request's review threads on GitHub |
| `@ascribed/review/place` | Node | Placing review threads on the blocks of a page |

The rendered page is Ascribe's site output with source anchors (`ascribe build --emit site --anchors`, and always in the editor's page preview), which say where each block is written. The [site-render contract](https://ascribed-dev.com/contracts/site-render/#7-source-anchors) describes them.

## Marks

```ts
import { markChanges, setShow, showSources } from "@ascribed/review/marks";
import "@ascribed/review/marks.css";

// `page` holds the page rendered with anchors; `was`, optionally, the page as
// it was. `changes` is one page's `changes` from `ascribe diff --format json`.
const marks = markChanges(page, changes, { was });
setShow(page, "changes"); // or "will" (the page as it will be), or "was"
showSources(page); // each block's source file and lines, on hover
```

Each changed block gets a label (Added, Changed, Removed, Moved) and a bar in the margin whose color and style differ by kind. Inside a changed block, the words added are in `<ins>` and the words removed in `<del>`. A removed block is shown where it was, collapsed to one line with **Show**, copied from the page as it was (or from the change's text without it). A moved block links to a stub at its old place, and the stub back to it. A tab's label and a `<details>`' summary say what's changed in what they can hide: "new" for an added tab, else how many changes. `markChanges` returns the marks in page order, for stepping through them with `goTo`. `clearMarks` removes them all.

The blocks are found by their anchors: the element with exactly a change's anchor, or, when there's none (a paragraph in a tight list item has no element of its own), the smallest anchored element with the same file and includes whose lines contain it.

The colors are the `--ascribe-review-*` custom properties at the top of `marks.css`, with light and dark values. Their names don't change; their defaults are Ascribe's design tokens, shared with `@ascribed/elements`, and a release may change them, noting it in the changelog.

## The static report

`src/report/` is the script of `ascribe diff --format html`'s report, which draws the list of changed pages and each page with its marks. It isn't an entry point: `pnpm --filter @ascribed/review embed` bundles it with the marks and `@ascribed/elements` into `crates/ascribe-diff/src/html/`, where the binary embeds it, along with one stylesheet made of the element library's, `marks.css`, and `src/report/report.css`. It also writes the script's SHA-256 beside it, which the report's content security policy names as the one script allowed to run. Run it after changing any of them; `test/embedded.test.ts` fails until you do. A page with a `prompt` in the report's data gets **Copy prompt**, which copies it and opens nothing.

## `@ascribed/review/overlay`

```ts
import { createOverlay } from "@ascribed/review/overlay";

const overlay = createOverlay({ root: article, host });
```

Draws a pull request's threads beside the blocks of `root`, a rendered page with source anchors: in a column beside the page when its container is at least `columnAt` pixels wide (default 600), else as a count on each block that opens its threads. Threads with no block on the page are listed above it, and a bar below it counts unsent comments, with the submit dialog. A reviewer can reply (at once, or with the review), resolve and reopen, comment on any block, and submit or discard the review, all from the keyboard. It draws in shadow roots, so its styles and the page's stay apart; theme it with the `--ascribe-review-*` custom properties `marks.css` declares.

The host (`OverlayHost`) supplies the data and does the work: `load()` for the page's threads (`ReviewSession.threads`), its pending review, and the viewer; `commentTarget`, `comment`, `reply`, `resolve`, `submit`, `discard`, and `allThreads`, as `ReviewSession` has them; and `openSource`, `openThread` (a thread on another page), and optionally `openLink`, `notify`, and `promptAgent`. With `promptAgent`, each thread's card has **Prompt agent** and the list of every thread **Prompt agent: all open**; the overlay passes the host a `PromptRequest` (`{ kind: "thread", threadId }` or `{ kind: "open-threads" }`, and the hosts' own headers ask for `page-changes` and `fragment-reach`) and builds nothing. A rejection's message is shown as why the prompt couldn't be built. `onDidChange` tells the overlay to read the threads again. The overlay never talks to GitHub itself, so a host can keep the token out of the page.

`overlay.refresh()` reads the threads again, `layout()` places them again after the page changed size, `goToThread(id)` goes to one, `showAllComments()` opens the list of every thread, and `dispose()` takes everything away.

Comment bodies are other people's Markdown: `renderMarkdown(document, text)` builds a safe subset (paragraphs, emphasis, code, links that open apart from the page, lists, block quotes) as DOM nodes. Raw HTML stays text, links other than web and email ones are dropped, and an image is a link to it, so nothing loads without a click.

## `@ascribed/review/prompt`

```ts
import { threadPrompt, openThreadsPrompt } from "@ascribed/review/prompt";
```

Builds the agent prompts about review threads, in the agents plan's prompt format: `threadPrompt` asks an agent to address one thread, with its place, the block's source lines, and its comments; `openThreadsPrompt` lists every open thread in two lines each. `openThreadsList` isn't a prompt but the answer of an agent's tool (VS Code's `ascribe_review_threads`): every open thread, at most `MAX_LISTED`, with its place, the pages that show it, and all its comments, each cut at `MAX_COMMENT` characters. A comment is other people's text, so it goes in only inside a fence longer than any backtick run in it, under a sentence saying it's data, with its HTML comments taken out first (`stripComments`). A prompt is at most `LIMIT` (5,000) characters, the binary's limit, which a test holds.

The builders read nothing. On a host's Node side, `buildThreadsPrompt(context, request)` from `@ascribed/review/github` gathers what they need from a `ReviewSession`, the changed pages, and a `readSource` callback (and `buildThreadsList(context)` does for `openThreadsList`), and `promptProject(projectDir)` finds the project's folder, content root, and `AGENTS.md` as the binary does. The prompts about changes come from the binary: `ascribe diff --format prompt`, or the language server's `ascribe/agentPrompt`.

## `@ascribed/review/github`

```ts
import { openReview, ghTransport, tokenTransport } from "@ascribed/review/github";

const review = await openReview({ projectDir: "/path/to/project" });
if (review === undefined) {
  // No open pull request for this branch, or HEAD is detached.
} else {
  const placed = await review.threads(page);
  await review.comment(anchor, "Is this still true?", page);
  console.log(`${(await review.pending()).count} unsent comments`);
  await review.submit("COMMENT");
}
```

### `openReview(options)`

Finds the open pull request whose head is the checkout's current branch and returns a `ReviewSession` for it, or `undefined` when there is none. It looks in the repository of a remote named `upstream`, if there is one, and then in the repository the branch is pushed to, and takes the pull request whose head is in the repository the branch is pushed to. No pull request is a normal answer, not an error.

| Option | Meaning |
|---|---|
| `projectDir` | The directory holding the project's `ascribe.toml`, inside the checkout. |
| `contentPrefix` | The content root's path in the repository, with a trailing `/` (`""` at the root). By default it's worked out from `ascribe.toml`'s `content-root`. |
| `transport` | `(host) => GitHubTransport`. By default `ghTransport({ host })`. The host comes from the branch's remote, so GitHub Enterprise Server works. |
| `mutationInterval` | The least time between two requests that change something, in milliseconds. Default `1000`, which keeps under GitHub's secondary rate limit. |

`session.pullRequest` describes the pull request: `number`, `url`, `repository` (`host`, `owner`, `name`), `baseRefName`, `baseOid`, `headRefName`, `headOid`, and `local`, how the checkout's `HEAD` relates to the head commit: `"same"`, `"behind"` (pull), `"ahead"` (push), `"diverged"`, or `"missing"` (the head commit isn't in the local repository: fetch).

### `ReviewSession`

| Method | What it does |
|---|---|
| `threads(page)` | The page's threads, placed (see [`placeOnPage`](#placeonpagepage-threads)). Reads the pull request once and caches it until `refresh()` or a change made through the session. |
| `comment(anchor, body, page, quote?)` | Comments on a block. The comment becomes a review thread on the block's file, at its lines at the pull request's head commit (moved back through any local edits), on the right side, in the viewer's pending review, which is created on the first comment. If any of the block's lines isn't at the head commit unchanged, it rejects with `push-first`. If the file isn't in the pull request, or GitHub can't anchor a comment to the lines (it can only anchor to lines near the pull request's changes), the comment is held in the pending review's body instead, quoting the block's text (`quote`, the text the page shows, escaped so GitHub shows it as written and cut at 2,000 characters; without it, the block's source), linking its lines, and ending with a hidden marker (below); it reaches the pull request's conversation when the review is submitted. GitHub can't add a body to a pending review created without one, so the session creates its review with a placeholder body, `<!-- ascribe:review -->`, which is never shown or sent; if the pending review was started on GitHub with no body, a held comment rejects with `cant-hold`. Resolves with the new thread. |
| `reply(threadId, body, when)` | Replies to a review thread: `"withReview"` holds the reply in the pending review; `"now"` sends it at once. While the viewer has a pending review, GitHub puts every reply into it, so a reply sent with `"now"` then is deleted again and the call rejects with `reply-held`: offer `"now"` only when `pending().count` is 0. |
| `resolve(threadId, resolved)` | Resolves or reopens a thread, at once. |
| `pending()` | What's in the viewer's pending review: its `id`, new `threads`, `replies` to existing threads, held `conversation` comments, and the `count` of all of them. |
| `submit(event, body?)` | Submits the pending review as `"COMMENT"`, `"APPROVE"`, or `"REQUEST_CHANGES"`, with `body` first and the held conversation comments after it, the two separated by `<!-- ascribe:summary -->` (and without the placeholder). |
| `discard()` | Deletes the pending review and everything in it. |
| `refresh()` | Forgets what's cached. |

Nothing the session does is visible to anyone else until `submit`, except `resolve` and replies sent with `"now"`. Changes go one at a time, spaced by `mutationInterval`.

"Pending" is the GitHub API's word, and these names keep it. Text shown to readers says "unsent".

### Threads

A `Thread` has an `id`, `kind` (`"review"`, or `"conversation"` for a conversation comment with a marker, which GitHub can't resolve or reply to as a thread), `repositoryPath`, `subject` (`"line"` or `"file"`), `side` (`"RIGHT"`, or `"LEFT"` for removed text), the lines GitHub reports (`line`, `startLine`, `originalLine`, `originalStartLine`, and the commits they're counted at), `resolved`, `outdated`, `canResolve`, `canUnresolve`, `canReply`, `comments`, and `quote`, the text the thread is on when it's known. Each comment has an `id`, `author` (`login`, `avatarUrl`, or `null`), `body`, `createdAt`, `url`, and `pending`, whether it's in the viewer's pending review.

Comment bodies are Markdown written by other people, returned as text. Render them safely.

### Hidden markers

A comment held for the conversation ends with a marker naming its block and build:

```html
<!-- ascribe:anchor guides/install.md:12-14 build=site -->
```

When the comment quotes the text the page shows, rather than the block's source, the marker ends ` quote=text`. In a submitted review, the summary comes first and ends with `<!-- ascribe:summary -->`; a summary with no held comments after it has no marker.

Reading a pull request also reads its conversation comments and review bodies, splits them at markers, and places each part like a thread.

### Transports

```ts
interface GitHubTransport {
  graphql<T>(query: string, variables: Record<string, unknown>): Promise<T>;
}
```

- `ghTransport({ host?, command? })` runs `gh api graphql --hostname <host>`, with the query on standard input and each variable as an argument (`-f` for strings, `-F` for numbers, booleans, and `null`). It needs `gh` 2.0.0 or later, signed in to the host (`gh auth login --hostname <host>`).
- `tokenTransport(getToken, { host?, fetch? })` posts to the host's GraphQL endpoint (`https://api.github.com/graphql`, or `https://<host>/api/graphql`) with the token `getToken` returns, called for each request.

Neither stores a token, and neither puts one in an error: anything shaped like a GitHub token is removed from error messages.

### Errors

Failures reject with a `ReviewError`, whose `message` is a sentence to show and whose `code` says what happened:

| Code | Meaning |
|---|---|
| `gh-missing` | `gh` isn't on the path. |
| `gh-too-old` | `gh` is older than 2.0.0. |
| `not-signed-in` | No sign-in for the host, or the host refused it. |
| `rate-limited` | GitHub's secondary rate limit. `retryAfter` is the seconds to wait, when GitHub says. |
| `refused` | GitHub answered with an error. |
| `network` | The request didn't reach GitHub, or the answer wasn't JSON. |
| `push-first` | The block's lines aren't all at the pull request's head commit, unchanged. |
| `reply-held` | A reply can't be sent at once while the viewer has a pending review. |
| `cant-hold` | The viewer's pending review was started on GitHub without a body, and GitHub can't add one to hold a comment. Submit or discard it on GitHub first. |
| `not-found` | The thread, or something else named, isn't there. |
| `git` | `git` failed, isn't on the path, or the directory isn't a repository. |

## `@ascribed/review/place`

### Anchors

`Anchor` is `{ source, via }`, a block's `data-ascribe-source` and `data-ascribe-via` (as an array). `parseSource("guides/install.md:12-14")` returns `{ path, first, last }` with the path decoded; `formatSource` and `encodePath` write them; `anchorKey(anchor)` is a string identifying an anchor, `via` included.

### `locateThreads(threads, { root, contentPrefix, headOid })`

Moves threads into the working tree's terms. Threads on files outside the content root are left out. Each other thread gets its content `path` and either `lines` or a `detached` reason:

- A thread on the right side is at its line at the pull request's head commit, moved through the working tree's changes since (`git diff --unified=0`). A reworded line moves onto the text that replaced it, and the thread is marked `outdated`, with the text it was on as `quote`. If the line was deleted, the thread is detached (`"line-gone"`), with the same `quote`.
- An outdated thread is at its original line at its original commit, moved the same way, and keeps that text as `quote`. It stays marked `outdated`.
- A thread on the left side keeps its line, on the diff's base.
- A thread on a whole file is detached (`"file"`).
- A conversation comment is at the lines its marker names.

### `placeOnPage(page, threads)`

The threads on one page. `page` is a `PageRef`:

| Field | Meaning |
|---|---|
| `build` | The build the page is rendered for. Conversation comments marked for another build are left out. |
| `path` | The page's content path. |
| `anchors` | Every anchored block on the page, in order. |
| `removed` | The `was` of each removed, moved, or changed block on the page, from `ascribe diff`: where threads on the base's text go. |

A thread goes on the smallest block that holds all its lines (failing that, its last line), and on every block with that source: a fragment included twice gives the thread twice. A thread on the left side goes on a removed, moved, or changed block's old lines the same way. The result has `blocks` and `removed`, each a list of `{ anchor, threads }` in the page's order, and `detached`: the threads on any of the page's files (its own, and the fragments it shows) that aren't on a block, with reason `"no-block"` when the page has no block for their lines.

### Lines

`lineMap(root, commit, file, "to-worktree" | "to-commit")` maps line numbers of a repository file between a commit and the working tree; `parseHunks` and `shiftLine` do the same for a diff in hand; `linesAt` reads lines of a file at a commit. A line maps to `{ line, replaced }`: a replaced line maps onto the lines that replaced it, at the same offset (or their last line), with `replaced` set, and a deleted line maps to `undefined`.
