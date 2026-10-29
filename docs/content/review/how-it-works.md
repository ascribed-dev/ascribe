---
type: concept
title: How review works
description: What review compares, what its marks mean, how its comments map to the pull request, what it needs from GitHub, and what reaches GitHub.
---

Review is one comparison shown in three places: the **page preview** in VS Code, the **site preview** under `astro dev`, and the **HTML preview** from CI. All three show the same changes with the same marks, and the two previews show the pull request's comments, with the same rules for where each lands. This page is what holds in all three. [Review](index.md) walks through reviewing a pull request; each place's own controls are on [Review in the preview](../editor/review.md), [Review in the site preview](site-preview.md), and [the HTML preview](diff.md#the-html-report).

## What review compares

[Review](../glossary.md#review) compares the working tree with a **base**, as [`ascribe diff`](diff.md#cli-diff) does: the base of the branch's pull request, when it has one, the default branch (the first of `origin/HEAD`, `origin/main`, `origin/master`, `main`, and `master` that exists), or a branch, tag, or commit. It compares from the point where the branch left the base, as a pull request does.

It compares **resolved pages**, not files. A change to a fragment shows on every page that includes it, a change to a [phrase](../glossary.md#phrase) on every page that uses it, and reformatting shows on none. A page that changed only through something it uses says what it changed through, such as `_fragments/requirements.md` or `ascribe.toml`.

The previews follow your edits: the page preview as you type, saved or not, and the site preview each time a save rebuilds the site.

## The marks

Each changed block has a bar in its margin and a label:

- an **added** block has a solid bar and the label **Added**;
- a **changed** block has a dashed bar and the label **Changed**, with the words added highlighted and the words removed struck through;
- a **removed** block is shown where it was, from the base, struck through and collapsed to one line, with **Show** to expand it;
- a **moved** block has a double bar and the label **Moved**, with a link to a stub at its old place, and back.

A tab's label and a `details`' summary say what changed in what they can hide: "new" for an added tab, else how many changes ("2 changes"). Every mark has a label as well as a color, and the colors work in light and dark.

A page's count of changes says where you are, "3 of 10 on this page", with arrows to step through them; past the last one, they offer the next changed page. **Changes / As it will be / As it was** switches between the marks, the page as it will be with none, and the page as it was at the base. In the previews, a mark's label opens its source lines.

## When the project has errors

Review compares pages with errors as they are, since reviewing work in progress is useful, but a page with an error can render oddly, and that oddity isn't the change. So when the project has errors, each place says how many, beside the changes: the page preview's header, with **Show problems**; the site preview's panel; the HTML preview; and `ascribe diff`, on standard error.

The numbers can differ. The page preview counts what VS Code's Problems panel shows, which checks pages for the editor's build only; the others count what `ascribe check` finds for the builds compared.

## Comments and the pull request

Every comment is an ordinary GitHub review comment. Ascribe keeps none of its own, so the pull request on GitHub is the record, and anyone can answer in GitHub's own interface.

- **A comment on a block is a review comment on its source file and lines,** at the pull request's latest commit. A block from a fragment is commented on in the fragment's file.
- **A fragment's thread shows on every page that includes it.** A comment made on one of those pages is on the fragment, so it shows on all of them.
- **GitHub takes review comments only on lines a pull request changes, and the few lines around them.** A comment on any other block, or on a page whose file isn't in the pull request (one that changed through a fragment or a phrase, say), goes in your review's summary instead, quoting the block and linking its lines. The comment box says so before you write. It reaches the pull request's conversation when you submit, and still shows beside its block in both previews, for you and for everyone else who reviews with Ascribe.
- **New comments are unsent until you submit,** and nobody else sees them. GitHub's API calls this your pending review. Submitting sends them as a comment, an approval, or a request for changes, with an optional summary; discarding deletes them.
- **A reply can go at once, unless you have unsent comments.** While you do, GitHub adds every reply to your review, so **Reply now** is unavailable, and says why, until you submit or discard.
- **Resolving and reopening act at once,** on GitHub, and aren't part of your review.
- **A thread whose text changed since it was written** is labeled **Outdated**, beside the text that replaced it, with the text it was on a click away. A thread whose lines are gone from the page, because the text was deleted, the block moved to another page, or the thread is on a whole file, is **detached**: it's listed above the page with the text it was on. A thread on removed text shows on the removed block. A thread carries one of these labels, the most specific that fits: **On removed text**, then **Detached**, then **Outdated**. When the text a thread was on is empty or can't be read, the thread says so in words instead of quoting it.

### Your checkout and the pull request

Comments are placed on the pull request's latest commit, so a comment can go only on lines that are there. When your checkout isn't at that commit, the page preview's header says so: behind, with **Pull**, since some comments may be on lines you don't have yet; ahead, with **Push**, since you can comment only on lines that are on GitHub. In VS Code, a block with unsaved changes can't take a comment until you save its file, since GitHub counts the file's saved lines.

## Signing in to GitHub

| To | You need |
|---|---|
| See what changed, in either preview | `git` on the path, and a checkout of the branch |
| See and write comments in VS Code | A GitHub sign-in in VS Code, or the GitHub CLI (`gh`), signed in |
| See and write comments in the site preview | The GitHub CLI (`gh`) 2.0.0 or later, signed in, where `astro dev` runs |
| Read the HTML preview from CI | Nothing: it opens in any browser |

Review finds the pull request whose head is the branch you're on, in the repository the branch is pushed to, or in a remote named `upstream`, for a fork. So the branch has to track its remote branch, as `gh pr checkout` and `git switch` both set it up to.

In VS Code, review asks to sign in to GitHub when you start it, never sooner, for the `repo` permission that posting review comments needs. VS Code's GitHub sign-in is used for `github.com`, and its GitHub Enterprise sign-in for the server `github-enterprise.uri` names; a remote on any other host gets none, and the preview says why. If you decline, review shows the changes only, and the preview offers **Sign in to see comments**, and **Use GitHub CLI** when `gh` is signed in. Once you choose the GitHub CLI, review uses it in that workspace from then on, until you choose **Sign in to see comments**.

In the site preview, the dev server runs `gh`: sign in with `gh auth login`, with `--hostname` for GitHub Enterprise Server.

Nothing needs a hosted service, and nothing but GitHub stores comments.

## Privacy and security

- **Review is the only time Ascribe contacts GitHub,** and only once you start it. Nothing runs `git` for a comparison, and nothing talks to GitHub, before you choose **Start Review** in VS Code or open the **Ascribe review** app in the site preview. After that, in the same workspace, VS Code asks GitHub whether a branch has a pull request when you open one of its pages, to [offer review](../editor/review.md#the-offer). The `ascribe` binary itself never uses the network: it runs `git` on your machine.
- **Ascribe stores no token.** In VS Code, comments go through VS Code's own GitHub sign-in, or through `gh`. A sign-in is sent only to the host it's for, never to another host a remote names. In the site preview, the dev server runs `gh` on your machine. Neither keeps a token of its own.
- **No token reaches a page.** The previews draw the comments but never talk to GitHub: in VS Code the extension makes every request, and in the site preview the dev server does, over Vite's own connection, which only pages from the dev server can open.
- **Pages and comments can't run code.** The three previews take scripts, event handlers, and `javascript:` links out of a page's HTML, and the HTML preview's content security policy lets only its own script run. Comment bodies are rendered from a safe subset of Markdown: no raw HTML, and an image shows as a link, so nothing loads until you click.
- **What's sent to GitHub** is what you write, and the file and lines it's on. A comment held in the summary also quotes the block's text and carries a hidden marker naming its block, `<!-- ascribe:anchor guides/install.md:12-14 build=site -->`, so that the previews can put it back on the block.

## Related resources

- [](index.md)
- [](ci-report.md)
- [](troubleshooting.md)
- [](../editor/review.md)
- [](site-preview.md)
- [`ascribe diff`](diff.md#cli-diff)
