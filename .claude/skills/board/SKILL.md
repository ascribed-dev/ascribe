---
name: board
description: "Read Ascribe's project board, move a card on it, or approve a pull request, from a session that can't reach GitHub Projects. Use whenever work involves the board (https://github.com/orgs/ascribed-dev/projects/1): triaging, picking up, handing off, or reviewing a card, or when `gh project` or `gh pr` fails with \"GraphQL is not available\"."
---

# The project board

Ascribe's work is tracked on [the project board](https://github.com/orgs/ascribed-dev/projects/1). Each card is an issue in `ascribed-dev/ascribe`, and its **Status** is its column: Backlog, Ready, In progress, In review, Done. The project's instructions say who moves a card out of each column. The skills `triage`, `implement`, and `review` say how each role does its part.

A Claude cloud session can't reach the board itself. Its GitHub access covers this repository's REST API and nothing else: no GraphQL, so no `gh project`, and no `gh pr`, `gh issue`, or `gh repo` commands, which use GraphQL too. Two workflows reach the board for you, and their headers have the details: [`board.yml`](../../../.github/workflows/board.yml) and [`approve.yml`](../../../.github/workflows/approve.yml).

## Read the board

Run Board, wait for it, then read `board.json` from the branch `board`:

```sh
since=$(date -u +%Y-%m-%dT%H:%M:%SZ)
gh workflow run board.yml --ref main
sleep 5
run=$(gh run list --workflow board.yml --event workflow_dispatch --limit 5 --json databaseId,createdAt \
  --jq "[.[] | select(.createdAt >= \"$since\")] | last | .databaseId")
gh run watch "$run" --exit-status > /dev/null
gh api 'repos/ascribed-dev/ascribe/contents/board.json?ref=board' --jq .content | base64 -d > board.json
```

`--ref main` matters: without it `gh workflow run` asks GraphQL for the default branch, and fails. `board.json` has the project, its fields with each single-select field's options, and every item. An item's `status` is its column, and `content` names its issue or pull request:

```sh
jq -r '.items[] | select(.status == "Ready") | "#\(.content.number) \(.content.title)"' board.json
```

A run writes `board.json` only when the board changed, so the file can be older than the run. That's expected; it's still current.

## Move a card

The same workflow, with the issue and the Status by its name as the board shows it:

```sh
gh workflow run board.yml --ref main -f issue=123 -f status="In review"
```

Then wait for it as above. A failed run says why in its error, for instance a Status the board doesn't have. An issue or pull request that isn't on the board yet is added. Move only cards your role owns, and only to the next column.

## Approve a pull request

Agents post as the maintainer, who also authors their pull requests, and GitHub doesn't let a pull request's author approve it. Approve posts the approval as `github-actions[bot]`, for the commit you reviewed:

```sh
gh workflow run approve.yml --ref main -f pr=123 -f sha="$(gh api repos/ascribed-dev/ascribe/pulls/123 --jq .head.sha)"
```

Take the commit from what you reviewed, not from a fresh fetch, when the two could differ: Approve refuses when the pull request's head has moved past the commit you give it, so new commits always get a review first. Wait for the run and check that it passed before you merge.

## Everything else on GitHub

Use the GitHub MCP tools, or `gh api` with a REST path, for issues and pull requests: reading them, commenting, reviewing, and opening a pull request. Merging is a REST call too, and this repository merges with a merge commit:

```sh
gh api -X PUT repos/ascribed-dev/ascribe/pulls/123/merge -f merge_method=merge -f sha=<the head you approved>
```
