---
type: troubleshooting
title: Troubleshooting review
description: What to do when review shows no pull request, no comments, or no marks, says the project has errors, or won't start.
available: ascribe 0.2.0
---

The problems you can meet reviewing a pull request with Ascribe, in the page preview, the site preview, or the report from CI, by what you see. [Review](../review.md) says how review works when nothing's wrong.

## No pull request found

Review shows the changes against the base you picked, and no comments.

**Cause.** Review finds the pull request whose head is the branch you're on, in the repository the branch is pushed to (or in a remote named `upstream`, for a fork). It found none: the branch isn't pushed, doesn't track its remote branch, or has no open pull request.

**Solution.**

@steps
1. Check that the branch is pushed and tracks its remote branch: `git status` names it.
2. Check that the pull request is open.
3. Choose **Ascribe: Refresh Comments**, or **Refresh** in the site preview.

## Showing changes only: comments need GitHub

The page preview says "Showing changes only. Comments need GitHub.", or the site preview says to run `gh auth login`.

**Cause.** Review isn't signed in to GitHub, or the remote is on a host it has no sign-in for.

**Solution.** How to sign in depends on where you're reviewing:

- **In the page preview,** choose **Sign in to see comments**, or, when `gh` is signed in, **Use GitHub CLI**. When the remote isn't on `github.com` or the server `github-enterprise.uri` names, the preview says so instead of offering a sign-in.
- **In the site preview,** run `gh auth login` (with `--hostname` for GitHub Enterprise Server), then choose **Refresh**.

## Comments are off in the site preview

The panel shows the changes only, and names a setting.

**Cause.** `astro dev` listens on the network (`--host`), or your Vite config lets other pages talk to the dev server (`server.cors: true`, `server.allowedHosts: true`, or `legacy.skipWebSocketTokenCheck`). Anyone who could reach the dev server could otherwise comment as you, so review turns comments off.

**Solution.** Remove the setting the panel names, and start `astro dev` again.

## Nothing is marked in the site preview

The panel says the blocks carry no source anchors, and lists the page's changes and threads instead, each with **Open source**; or it says the route isn't an Ascribe page.

**Cause.** The marks are placed by the `data-ascribe-source` attribute on each block. Your layout or a component rebuilds the content without it. A route that isn't an Ascribe page has no blocks to mark, and the panel lists the changed pages instead.

**Solution.** Render the page's content as the integration gives it, keeping each block's `data-ascribe-source`. [Review in the site preview](../../reference/astro.md#review-in-the-site-preview) has what the site's side needs.

## The project has errors

The page preview says how many errors the project has, with **Show problems**; the site preview, the HTML report, and `ascribe diff` (on standard error) say so too.

**Cause.** Review still compares the pages, but a page with an error can render oddly, and that oddity isn't the change. The numbers can differ: the page preview counts what VS Code's Problems panel shows, which checks pages for the editor's build only, while the others count what `ascribe check` finds for the builds compared.

**Solution.** Run `ascribe check` to list the errors, or choose **Show problems**, and fix them.

## A thread is detached

A thread is listed above the page, with the text it was on.

**Cause.** Its lines are no longer on the page: the text was deleted, the block moved to another page, or the thread is on a whole file.

**Solution.** Reply to it or resolve it where it's listed. **View on GitHub** opens it in the pull request.

## Can't comment on a block

The page preview offers **Pull** or **Push**, or a block says to push or save first.

**Cause.** Comments are placed on the pull request's latest commit, and your checkout isn't at it. Behind, some comments may be on lines you don't have yet. Ahead, with commits you haven't pushed, you can comment only on lines that are on GitHub. In VS Code, a block with unsaved changes can't be commented on either.

**Solution.** Bring the checkout to the pull request's latest commit:

- **Behind:** choose **Pull**.
- **Ahead:** push your commits, or choose **Push**.
- **Unsaved changes:** save the file.

## Review won't start

Review stops with a message.

**Cause.** Review needs `git` on the path, and the project in a git repository. A base that doesn't exist, or a shallow clone without the merge base, stops it, with a message that says which. In VS Code, the project's language server must be running.

**Solution.** Do what the message says, which is one of these:

- Install `git`, or put it on the path.
- Pick a base that exists, or fetch the history the message names.
- In VS Code, open one of the project's pages first, so its language server starts.

## For more information

- [Review](../review.md): reviewing a pull request, and how comments map to it.
- [Review in the site preview](../../reference/astro.md#review-in-the-site-preview): the site's side of review.
- [Comments in the preview](../editor.md#comments-in-the-preview): the editor's side.
- [`ascribe diff`](../../reference/cli/diff.md#cli-diff): the comparison every part of review makes.
