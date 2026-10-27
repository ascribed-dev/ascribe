---
type: how-to
title: Keep the docs in another repository
description: Take code examples from the code's repository when the docs have a repository of their own, pinned to a commit, and keep them up to date with a pull request.
permalink: docs-repository
---

When the docs have a repository of their own, a source can name the code's repository instead of a folder. Pages address it the same way as [a folder beside the docs](snippets.md), so moving the code from one repository to the other changes `ascribe.toml` and no page.

Ascribe doesn't read the other repository whenever it checks. It copies the files your snippets use, whole, into `sources/<name>/` beside `ascribe.toml`, and pins the commit they came from in `ascribe.lock`. You commit both with the docs.

## Take examples from the code's repository

@steps
1. Name the repository in `ascribe.toml`:

   ```toml
   [sources.api]
   git = "https://github.com/acme/api.git"
   branch = "main"                           # the branch `update` follows
   include = ["src/**", "examples/**"]
   ```

2. Write the `@snippet`, then run `ascribe sources fetch`. It pins the source to the head of its branch the first time, and copies the file.
3. Commit `ascribe.lock` and `sources/`.
4. When the code moves on, run `ascribe sources update`, or let [the update pull request](#the-update-pull-request) run it. It moves the pin, copies the files again, and lists the pages whose examples changed, the way [`ascribe drift`](drift.md) does, so the pull request that moves the pin says which pages to read. The pages are found by comparing with the docs' last commit, so in a new repository, commit once first.

Everything else reads the copies: `ascribe check`, `ascribe build`, the editor, and `ascribe diff` and `ascribe drift`, which read the copies at the base from the docs repository's own history. None of them run `git` against the code's repository or use the network, and `ascribe check` fails on a copy edited by hand, since the change belongs in the code. `ascribe sources status` shows each pin and copy.

- **Tested examples.** When the code's repository has an [`ascribe.examples.toml`](tested.md#when-the-code-is-in-another-repository), `fetch` and `update` copy it and its observations too, whatever `include` says, and the pages say what each example was tested with. They're read as text: nothing in them is run here.
- **What's published.** The copies are committed with the docs, so everyone who can read the docs repository can read them. When the code is private and the docs are public, that's a decision to make knowingly; `fetch` and `update` say so the first time they copy a source's files.
- **Access.** `ascribe sources fetch` and `update` run `git`, with whatever credentials let `git fetch` work in that shell. A CI job that only checks or builds needs no access to the code's repository, and neither does the host that builds your site. A job that runs `update` needs read access, for a private repository a token from a GitHub App or a read-only deploy key, given to `git` as any other job would, for example:

  ```sh
  git config --global url."https://x-access-token:${CODE_TOKEN}@github.com/".insteadOf "https://github.com/"
  ```

## The update pull request

Nothing moves a pin unless something runs `ascribe sources update`. This GitHub Actions workflow runs it on a schedule, in the docs repository, and when an example changed, opens a pull request that moves the pins and updates the copies. There's one at a time: while it's open, each run updates it in place, on the branch `ascribe/update-sources`.

@snippet {lang=yaml}: code:examples/docs-repository/.github/workflows/update-sources.yml#workflow

The workflow takes two tokens from a [GitHub App](https://docs.github.com/en/apps/creating-github-apps/registering-a-github-app/registering-a-github-app) that you create for your organization:

@steps
1. Create the App with no webhook, and give it two repository permissions: **Contents** (read and write) and **Pull requests** (read and write).
2. Install it on the docs repository and the code's repositories, and nowhere else.
3. In the docs repository, add the App's client ID as the Actions variable `SOURCES_APP_CLIENT_ID`, and a private key for it as the secret `SOURCES_APP_PRIVATE_KEY`.
4. In the workflow, list the code's repositories in `repositories`.

The first token can write to the docs repository only; the second can only read the code, and `git` uses it only while the pins move. The App's private key can do more than either: with it, anyone who can change a workflow in the docs repository can make a token that writes to the code. When that matters, make a second App with only **Contents** (read), install it on the code's repositories, and give the second token step its client ID and key. An App's token is needed for two reasons: the default token can't read another private repository, and a pull request it opens doesn't run your checks until someone approves them.

Your docs repository needs a workflow that runs `ascribe check` on pull requests, so a pull request with a broken example fails.

The pull request's title names each source and its new pin. Its description is what `ascribe sources update --format markdown` writes: the commits that came in, linked to the comparison on GitHub, the copies that changed, the [tested examples](tested.md#when-the-code-is-in-another-repository) that went stale or were tested again, and the pages to reread, as `ascribe drift` groups them. When an example no longer resolves, because its region was renamed or its file moved, the description says so, with `ascribe check`'s errors, and the pull request fails your docs' checks like any other. So does one that brings an example that went stale: [`example-stale`](../checks/diagnostics/output-checks.md#ASC199) is a warning, which `--deny-warnings` fails on, until the code's repository runs `ascribe observe` again. Because the copies are in its diff, the pull request is reviewed like any change to the docs: your checks, the [HTML preview](../review/index.md), and your site's preview all show the pages with the new code.

- **When it opens one.** Only when a copy changed. A run where the code moved but no example did changes nothing: no branch, no pull request, a green run. A run that can't reach the code's repository fails, with `git`'s message. If the examples go back to what the default branch has, it closes the open pull request.
- **Fixing a page.** Push the fix to the pull request's branch. From then on, the job leaves the branch alone, so your commits aren't overwritten, until the pull request is merged or closed. The next run after that opens a new one.
- **Merging it** brings the docs up to date with the code. Nothing merges it for you.
- **The schedule.** The default is once each weekday. GitHub stops a scheduled workflow in a public repository after 60 days without activity there; you can start it again from the Actions tab, and run it by hand at any time.

This repository runs this workflow on a pair of made-up repositories, built by [`scripts/sources-fixture/setup.ts`]({repo}/blob/main/scripts/sources-fixture/setup.ts) from [`examples/docs-repository`]({repo}/tree/main/examples/docs-repository): the workflow above is taken from that folder.

## See also

- [A source in another repository](../content-model/output.md#a-source-in-another-repository): its keys.
- [`ascribe sources`](sources.md#cli-sources): fetching, updating, and showing a source in another repository.
