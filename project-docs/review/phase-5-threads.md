# Phase 5: Review threads

Part of [Review](README.md). Requires phase 2 (for the diff's JSON), and can run at the same time as phase 3. TypeScript only: `packages/review`. No UI.

## Goal

The Node part of `@ascribed/review`: find the pull request for the checkout, read its review threads, work out which block each belongs to, and post comments, replies, and resolutions through a pending review. Phases 6 and 7 put a UI on it.

## Context

- `packages/review`: this phase fills its Node entry points, `./github` and `./place`. If the package doesn't exist yet, do the README's [package setup](README.md#the-package-setup) first, as its own commit. Leave `src/marks/` to phase 3.
- `scripts/release/` (`version.ts`, `manifests.ts`, `pack.ts`, `publish.ts`) and `RELEASING.md`: every published package is versioned in lock step. Adding a package means adding it there.
- GitHub's GraphQL API: `pullRequest.reviewThreads` (each with `path`, `line`, `startLine`, `originalLine`, `diffSide`, `isResolved`, `isOutdated`, `subjectType`, and its comments), `addPullRequestReviewThread`, `addPullRequestReviewThreadReply`, `resolveReviewThread`, `unresolveReviewThread`, `submitPullRequestReview`, and the viewer's pending review. Check all of them against the current schema.
- [`mockup.html`](mockup.html): this phase has no UI, but the mockup shows every case the session has to support. Use its **Try** buttons as a checklist for placement and posting: a thread on a fragment shown on two pages; a thread inside a tab that isn't showing, and inside closed details; a thread on removed text; a detached thread with the text it was on; an outdated thread with its original text; a comment GitHub can't anchor, sent to the conversation; a block that can't be commented on until its lines are pushed; **Reply now** and **Add to review**; the unsent count; and resolve acting at once.
- GitHub's limits: content-creating requests are limited per minute and per hour (the secondary rate limit); read the current numbers.
- `gh`: `gh pr view --json`, `gh api graphql`, `gh auth status`, and how it reports "no pull request for this branch" and "not signed in".

## Design

### Transports

One interface, two implementations:

```ts
interface GitHubTransport {
  graphql<T>(query: string, variables: Record<string, unknown>): Promise<T>;
}
```

- **`ghTransport`** runs `gh api graphql` with a fixed argument list, passing variables as arguments and the query on standard input. It finds `gh` on the path and reports clearly when it's missing, too old, or not signed in.
- **`tokenTransport(getToken)`** uses `fetch` with a token its host supplies. The VS Code extension passes one from VS Code's GitHub sign-in (phase 6). The package never stores a token or writes one to a log.

GitHub Enterprise Server hosts: take the host from the repository's remote, and pass it through (`gh api --hostname`, the API URL for `fetch`).

### Finding the pull request

From the repository's remote and current branch: the open pull request whose head is this branch (`gh pr view`'s rule). Return its number, URL, base and head commits, and whether the local `HEAD` is the pull request's head commit, behind it, or ahead of it. No pull request is a normal answer, not an error.

### Placing threads

A thread sits on a path and a line, on one side of the pull request's diff, at the pull request's head commit. A rendered block has an anchor: a file and a line range in the working tree. Placing is finding the block for the thread:

1. **Path:** the thread's path is repository-relative; anchors are content paths. Convert with the project's prefix (phase 2's `repository.projectPrefix`). A thread on a file outside the project is not placed and not shown.
2. **Line:** if the working tree's file equals the head commit's, the line is the line. Otherwise shift it through the line changes between the two (`git diff --unified=0 <head> -- <path>`, parsed). A line that no longer exists gives a **detached** thread.
3. **Side:** a thread on the right side belongs to the block whose anchor contains the line. A thread on the left side (on removed text) belongs to the removed block from the diff whose `was` contains it.
4. **Includes:** a thread on a fragment's line belongs to that block on every page that includes the fragment. Placement is by anchor, so this needs nothing extra; test it.
5. **Outdated** threads (GitHub's `isOutdated`) are placed by `originalLine` at their original commit if that can be shifted to the working tree, and otherwise detached. They're marked outdated either way.

The result for a page: threads by anchor, plus the page's detached threads, plus threads marked by hidden marker (below).

### Posting

- **A comment on a block** becomes a review thread on the block's file, at the block's last line (and `startLine` at its first, for a multi-line block), on the right side. Lines are shifted back from the working tree to the head commit; if the block's lines don't exist at the head commit (local edits not pushed), posting is refused with a message saying to push first.
- **When GitHub can't anchor it:** the file isn't in the pull request (a page that changed only through a phrase or the model). The comment goes on the pull request's conversation instead, quoting the block's text, linking the page, and ending with a hidden marker:

  ```html
  <!-- ascribe:anchor guides/install.md:12-14 build=site -->
  ```

  Reading threads also reads conversation comments with this marker and places them like threads. They can't be resolved on GitHub; show them as plain comments.
- **Pending review:** new threads and replies go into the viewer's pending review (created on the first one). `pending()` returns what's in it; `submit(event, body)` submits it as a comment, an approval, or a request for changes; `discard()` deletes it.
- **Replies** to an existing thread go either way, as the caller asks: sent at once, or held in the pending review. Check how the API treats a reply while a pending review exists (it may attach every reply to it); if sending at once isn't possible then, return an error the UI can explain.
- **"Pending" is the API's word.** Keep it in this package's names. User-facing text says "unsent" (README decision 7).
- **Resolve and unresolve** act at once.
- **Limits:** one mutation at a time, with a delay that keeps under the secondary rate limit, and a clear error when GitHub refuses.

### Shape for hosts

One object per pull request, which both hosts use and which phase 6's overlay talks to through its host:

```ts
interface ReviewSession {
  pullRequest: PullRequestInfo;
  threads(page: PageRef): Promise<PlacedThreads>;   // cached; refresh() refetches
  comment(anchor: Anchor, body: string, page: PageRef): Promise<Thread>;
  reply(threadId: string, body: string, when: "now" | "withReview"): Promise<Comment>;
  resolve(threadId: string, resolved: boolean): Promise<void>;
  pending(): Promise<PendingReview>;
  submit(event: "COMMENT" | "APPROVE" | "REQUEST_CHANGES", body?: string): Promise<void>;
  discard(): Promise<void>;
  refresh(): Promise<void>;
}
```

Comment bodies are Markdown from other people. This package returns them as text; rendering them safely is the overlay's job (phase 6).

## Tasks

1. The package setup, if phase 3 hasn't done it.
2. Both transports, tested against a fake `gh` executable (a script on the path that replays recorded answers) and a fake `fetch`.
3. Finding the pull request, placing, and posting, tested against recorded GraphQL responses and a temporary git repository. Cover: a right-side and a left-side thread, a multi-line thread, a thread on a fragment, an outdated thread, a line shifted by local edits, a removed line (detached), a file outside the project, the hidden-marker path, pagination past 100 threads, and each failure (`gh` missing, not signed in, no pull request, rate limited).
4. A manual check against a real pull request in a scratch repository: read threads, post two comments into a pending review, submit, resolve one. Say in the pull request what you ran and saw. Do not post to this repository's pull requests.
5. `packages/review/README.md` (the API of `./github` and `./place`) and `CHANGELOG.md`.

## Out of scope

Any UI; editing or deleting comments; reactions; suggestions; GitLab.

## Acceptance criteria

- Threads are placed on the right blocks in every case listed, and detached threads are returned, not dropped.
- Nothing is visible to others until `submit`, except resolve and unresolve, and replies sent with `"now"`.
- No test touches the network, and no token appears in logs or errors.

## Verify

```sh
pnpm --filter @ascribed/review test
pnpm typecheck && pnpm lint && pnpm format:check
pnpm test
```

## Commits

1. "Add @ascribed/review to the workspace and the release" (only if phase 3 hasn't)
2. "Read a pull request's review threads"
3. "Place review threads on rendered blocks"
4. "Post comments through a pending review"
