---
type: troubleshooting
title: Troubleshooting review
description: What to do when review shows no pull request, no comments, or no marks, says the project has errors, or won't start.
available: ascribe 0.2.0
---

The problems you can meet reviewing a pull request with Ascribe, in the page preview, the site preview, or the report from CI, by what you see. [Review](index.md) says how review works when nothing's wrong.

## No pull request found

Review shows the changes against the base you picked, and no comments.

**Cause.** Review found no open pull request whose head is the branch you're on ([where it looks](how-it-works.md#signing-in-to-github)): the branch isn't pushed, doesn't track its remote branch, or has no open pull request.

**Solution.**

@steps
1. Check that the branch is pushed and tracks its remote branch: `git status` names it.
2. Check that the pull request is open.
3. Choose **Ascribe: Refresh Comments**, or **Refresh** in the site preview.

## Showing changes only: comments need GitHub

The page preview says "Showing changes only. Comments need GitHub.", or the site preview says to run `gh auth login`.

**Cause.** Review isn't signed in to GitHub, or the remote is on a host it has no sign-in for ([Signing in to GitHub](how-it-works.md#signing-in-to-github)).

**Solution.** How to sign in depends on where you're reviewing:

- **In the page preview,** choose **Sign in to see comments**, or, when `gh` is signed in, **Use GitHub CLI**.
- **In the site preview,** run `gh auth login` (with `--hostname` for GitHub Enterprise Server), then choose **Refresh**.

## Comments are off in the site preview

The panel shows the changes only, and names a setting.

**Cause.** `astro dev` listens on the network, or your Vite config lets other pages talk to the dev server, so anyone who could reach it could comment as you ([Comments](site-preview.md#comments) lists the settings).

**Solution.** Remove the setting the panel names, and start `astro dev` again.

## Nothing is marked in the site preview

The panel says the page's blocks carry no source anchors, or that most of its changed blocks carry none, and lists the page's changes and threads instead, each with **Open source**; or it says the route isn't an Ascribe page.

**Cause.** Your layout or a component rebuilds the content without each block's `data-ascribe-source`, which places the marks, or the route isn't an Ascribe page ([Pages with nothing to mark](site-preview.md#pages-with-nothing-to-mark)).

**Solution.** Render the page's content as the integration gives it, keeping each block's `data-ascribe-source`.

## The project has errors

The page preview says how many errors the project has, with **Show problems**; the site preview, the HTML report, and `ascribe diff` (on standard error) say so too.

**Cause.** Review compares pages with errors as they are, and a page with an error can render oddly ([When the project has errors](how-it-works.md#when-the-project-has-errors), which also says why the numbers can differ).

**Solution.** Run `ascribe check` to list the errors, or choose **Show problems**, and fix them.

## A thread is detached

A thread is listed above the page, with the text it was on.

**Cause.** Its lines are no longer on the page ([Comments and the pull request](how-it-works.md#comments-and-the-pull-request)).

**Solution.** Reply to it or resolve it where it's listed. **View on GitHub** opens it in the pull request.

## Can't comment on a block

The page preview offers **Pull** or **Push**, or a block says to push or save first.

**Cause.** Comments are placed on the pull request's latest commit, and your checkout isn't at it, or the block has unsaved changes ([Your checkout and the pull request](how-it-works.md#your-checkout-and-the-pull-request)).

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

- [Review](index.md): reviewing a pull request.
- [How review works](how-it-works.md): what review compares, and how comments map to the pull request.
- [Review in the site preview](site-preview.md): the site's side of review.
- [Comments in the preview](../editor/review.md#comments-in-the-preview): the editor's side.
- [`ascribe diff`](diff.md#cli-diff): the comparison every part of review makes.
