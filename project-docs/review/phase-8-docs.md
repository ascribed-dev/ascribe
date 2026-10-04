# Phase 8: Docs and a full pass

Part of [Review](README.md). Requires phases 1 to 7. Docs, and fixes for what the pass finds.

## Goal

- **A review guide** that a reviewer new to Ascribe can follow start to finish.
- **The whole flow tried by hand** on a real pull request, with what was found fixed or filed.

## Context

- `docs/review.md`, grown piece by piece through phases 3 to 7.
- `docs/README.md` (the docs' index), `docs/editor.md`, `docs/astro.md`, `docs/cli.md`, `packages/vscode/README.md` (the Marketplace page), `packages/astro/README.md`, `packages/review/README.md`.
- `CHANGELOG.md`'s unreleased section.
- `scripts/review-fixture/setup.ts`: builds the repository the pass runs on ([The fixture](#the-fixture)).
- [`mockup.html`](mockup.html): the reference the built UI is checked against in the pass, and the source of the guide's wording (Page preview, Site preview, unsent, detached, outdated).

## Design

### The guide

`docs/review.md`, in the order a reviewer meets things:

1. **What you get,** in a paragraph, with one screenshot of the site preview under review.
2. **Review a pull request:** check out the branch, start review, read the changed pages, comment, submit. The page preview first (it needs nothing else), then the site preview.
3. **What you need for each part:** a table. The diff: `git`. Comments in VS Code: a GitHub sign-in. Comments on the site preview: `gh`, signed in. The report in CI: nothing for the reader.
4. **How comments map to the pull request:** a comment on a block is a review comment on its file and line; a fragment's thread shows on every page that includes it; GitHub anchors comments only on changed lines and the few around them, so a comment on any other block goes in your review's summary and still shows on the block; new comments stay unsent, visible only to you, until you submit; a reply can go at once unless you have unsent comments; resolving acts at once.
5. **The report in CI.**
6. **When something's missing:** no pull request found, not signed in, anchors not arriving on the site, a thread that's detached, local commits not pushed.
7. **Privacy and security:** what runs where, that no token is stored or reaches a page, and that review mode is the only time Ascribe contacts GitHub.

Link it from `docs/README.md`, `docs/editor.md`, `docs/astro.md`, and both package READMEs, and remove what those repeat.

### The fixture

The pass runs on a repository made for it, not on this one: `ascribed-dev/review-fixture`, private. A script builds it from nothing, so every pass starts from the same pull request and the comments an earlier pass left are gone:

```sh
pnpm --filter @ascribed/review build
node scripts/review-fixture/setup.ts ascribed-dev/review-fixture
```

It needs `git`, and `gh` signed in with write access to that repository. It closes the fixture's open pull request, force-pushes both branches, opens a new pull request, seeds its threads, and leaves a checkout at the pull request's head, with the commands to open each view. It refuses a repository it didn't build. `--local` makes the checkout alone, with no GitHub: enough for the changes, without threads.

The repository is the Lantern docs from `examples/monorepo`, in a `docs/` subfolder, with `examples/astro-site`'s site around them. The pull request has two commits, and its threads were made between them:

| What review has to handle | Where the fixture has it |
|---|---|
| A page's own file changed: a block reworded, added, and removed | `guides/rollouts.md` |
| A fragment two pages include | `_fragments/prerequisites.md`, on `getting-started.md` and `guides/create-flags.md` |
| A page changed only through a fragment | `guides/create-flags.md` |
| A phrase changed (`{version}`), so pages change through `ascribe.toml` | `getting-started.md`, `guides/self-hosting.md` |
| A variant arm changed | The Linux install command in `getting-started.md` |
| Two threads on one block | The opening paragraph of `guides/rollouts.md` |
| An outdated thread: its line was reworded by the second commit | "Start at 1% or lower…" in `guides/rollouts.md` |
| Threads whose lines the second commit moved | Every thread in `guides/rollouts.md` |
| A thread on removed text | The removed note in `guides/rollouts.md` |
| A multi-line thread, on a fragment | `_fragments/prerequisites.md`, lines 3 to 5 |
| A resolved thread | `_fragments/prerequisites.md`, line 3 |
| A comment in the review summary, on a line away from the changes | The last paragraph of `guides/rollouts.md` |
| A comment in the review summary, on a file the pull request doesn't touch | `guides/self-hosting.md` |

Not in it, so made by hand during the pass: a detached thread (delete a commented block locally), unpushed commits, a checkout behind the pull request, and a comment from someone else (every seeded comment is from whoever ran the script).

If `examples/monorepo` changes so the script's edits no longer apply, the script stops and says which; update it in the same pull request as the example.

### The pass

On the fixture:

1. Build it with the script, and open the checkout it leaves.
2. With the mockup open beside it, review it three ways: the CI report, the page preview, the site preview. Go through the mockup's **Try** buttons and **State** menu and reproduce each one in the real thing. List every difference in the pull request, and for each say whether the build or the mockup should change. In each, check the changed pages list, the marks, each thread's placement, and switching to the other views.
3. Comment from the page preview and from the site preview, including on a page whose file isn't in the pull request; submit; confirm on GitHub.
4. Repeat the editor steps on Windows if a machine is available, and say if not.
5. Time each step that makes the user wait (starting review, the diff rerun after an edit, loading threads) and record the numbers.

Fix what's small. File an issue for what isn't, and list the issues in the pull request.

## Tasks

1. The guide and the links.
2. The pass, with its findings in the pull request.
3. The CHANGELOG's unreleased section read as one release's notes.

## Out of scope

Releasing (a separate step, following `RELEASING.md`).

## Acceptance criteria

- A reviewer can follow `docs/review.md` from a fresh checkout to a submitted review without reading anything else.
- The pass's findings are fixed or filed, and its timings are recorded.

## Verify

```sh
pnpm lint && pnpm format:check
cargo test --workspace --locked
pnpm test
```

## Commits

1. "Write the review guide"
2. One per fix from the pass
