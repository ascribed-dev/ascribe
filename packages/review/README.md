# @ascribed/review

Reviewing [Ascribe](https://github.com/ascribed-dev/ascribe) pages as readers see them: what a change does to each rendered page, and a pull request's review threads beside the blocks they're about.

Each entry point is separate, so a host bundles only what it uses.

| Entry point | Runs in | What it's for |
|---|---|---|
| `@ascribed/review/marks` | the browser | Marking the changed blocks of a rendered page, from `ascribe diff --format json`'s changes |
| `@ascribed/review/overlay` | the browser | The review overlay: changed blocks and review threads on a rendered page |
| `@ascribed/review/github` | Node | Reading and writing a pull request's review threads on GitHub |
| `@ascribed/review/place` | Node | Placing review threads on the blocks of a page |

The rendered page is Ascribe's site output with source anchors (`ascribe build --emit site --anchors`, and always in the editor's page preview), which say where each block is written. The [site-render contract](https://github.com/ascribed-dev/ascribe/blob/main/docs/contracts/site-render.md#7-source-anchors) describes them.

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
| `comment(anchor, body, page)` | Comments on a block. The comment becomes a review thread on the block's file, at its lines at the pull request's head commit (moved back through any local edits), on the right side, in the viewer's pending review, which is created on the first comment. If the block's lines aren't at the head commit, it rejects with `push-first`. If the file isn't in the pull request, or GitHub can't anchor a comment to the line, the comment is held in the pending review's body instead, quoting the block's text, linking its lines, and ending with a hidden marker (below); it reaches the pull request's conversation when the review is submitted. Resolves with the new thread. |
| `reply(threadId, body, when)` | Replies to a review thread: `"withReview"` holds the reply in the pending review; `"now"` sends it at once. If GitHub puts a reply sent with `"now"` into the viewer's pending review instead, the reply is deleted again and the call rejects with `reply-held`. |
| `resolve(threadId, resolved)` | Resolves or reopens a thread, at once. |
| `pending()` | What's in the viewer's pending review: its `id`, new `threads`, `replies` to existing threads, held `conversation` comments, and the `count` of all of them. |
| `submit(event, body?)` | Submits the pending review as `"COMMENT"`, `"APPROVE"`, or `"REQUEST_CHANGES"`, with the held conversation comments ahead of `body`. |
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
| `push-first` | The block's lines aren't at the pull request's head commit. |
| `reply-held` | A reply can't be sent at once while the viewer has a pending review. |
| `not-found` | The thread, or something else named, isn't there. |
| `git` | `git` failed, isn't on the path, or the directory isn't a repository. |

## `@ascribed/review/place`

### Anchors

`Anchor` is `{ source, via }`, a block's `data-ascribe-source` and `data-ascribe-via` (as an array). `parseSource("guides/install.md:12-14")` returns `{ path, first, last }` with the path decoded; `formatSource` and `encodePath` write them; `anchorKey(anchor)` is a string identifying an anchor, `via` included.

### `locateThreads(threads, { root, contentPrefix, headOid })`

Moves threads into the working tree's terms. Threads on files outside the content root are left out. Each other thread gets its content `path` and either `lines` or a `detached` reason:

- A thread on the right side is at its line at the pull request's head commit, moved through the working tree's changes since (`git diff --unified=0`). If the line is gone, it's detached (`"line-gone"`), with the text it was on as `quote`.
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
| `removed` | The `was` of each removed or moved change on the page, from `ascribe diff`. |

A thread goes on the smallest block that holds all its lines (failing that, its last line), and on every block with that source: a fragment included twice gives the thread twice. A thread on the left side goes on a removed block the same way. The result has `blocks` and `removed`, each a list of `{ anchor, threads }` in the page's order, and `detached`: the threads on any of the page's files (its own, and the fragments it shows) that aren't on a block, with reason `"no-block"` when the page has no block for their lines.

### Lines

`lineMap(root, commit, file, "to-worktree" | "to-commit")` maps line numbers of a repository file between a commit and the working tree; `parseHunks` and `shiftLine` do the same for a diff in hand; `linesAt` reads lines of a file at a commit.
