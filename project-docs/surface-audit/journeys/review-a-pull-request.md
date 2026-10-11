# Review a pull request's docs

Part of [The surface audit](../README.md), [phase 3](../phase-3-journeys.md), journey 5. The script first; then the session's notes.

## The task

The whole prompt is what `ascribe diff --format prompt` writes in the fixture's project, and nothing is added to it. With the release build of `main` on 10 October 2026:

```text
Review what this change does to the 4 pages it changes.

Project: docs/

Base: `origin/main`, from where this branch left it (8254eaf)
To read a page: `ascribe render <page> --build <build>`
In build `site`:
- content/getting-started.md: 3 changed, 1 added (also through content/_fragments/prerequisites.md, ascribe.toml)
- content/guides/create-flags.md: 1 changed, 1 added (through content/_fragments/prerequisites.md)
- content/guides/rollouts.md: 1 changed, 2 added, 1 removed (also through ascribe.toml)
- content/guides/self-hosting.md: 1 changed (through ascribe.toml)
In build `self-hosted`:
- content/getting-started.md: 3 changed, 1 added (also through content/_fragments/prerequisites.md, ascribe.toml)
- content/guides/create-flags.md: 1 changed, 1 added (through content/_fragments/prerequisites.md)
- content/guides/rollouts.md: 1 changed, 2 added, 1 removed (also through ascribe.toml)
- content/guides/self-hosting.md: 1 changed (through ascribe.toml)

Report what reads wrongly; don't edit.
```

The commit in the `Base:` line is the fixture's, and differs with each run of its script. A check can't confirm anything about this task: the branch checks clean before and after, and the result is a report.

## Starting point

The review fixture's checkout, as `scripts/review-fixture/setup.ts --local` makes it: Lantern's docs from `examples/monorepo/docs` in `docs/`, with an Astro site's files around them; a base commit on `main`; a branch, `scheduled-rollouts`, with two commits; and a bare repository beside the checkout as `origin`. `--local` makes no pull request on GitHub, so there are no review threads, only the changes.

It isn't one of the other journeys' copies in two ways, both the fixture's own. It has three commits and a remote, since the change under review is the task. And it has the README the script writes at its root, which says what the repository is: "A made-up docs project (Lantern, from Ascribe's `examples/monorepo`) with one open pull request, for trying Ascribe's review by hand. Don't edit it". The fixture is used as it's made.

The session starts in the checkout's root, not in `docs/`. The prompt is the same from either: `ascribe diff --format prompt --config docs` at the root writes the same text, byte for byte, and its `Project: docs/` line is written for a reader at the root.

## The session

One, with no plugin. The phase's table gives this journey the prompt and nothing else:

```shell
claude -p "$(cat <scratch>/review-prompt.txt)" \
  --setting-sources project \
  --allowedTools "Read,Edit,Write,Glob,Grep,Bash" \
  --no-session-persistence --include-hook-events \
  --output-format stream-json --verbose \
  > <checkout>/project-docs/surface-audit/journeys/transcripts/review-a-pull-request-prompt.jsonl
```

The flags are [phase 1's](fix-a-broken-link.md#what-changed-in-these-two-commands-and-why), flag for flag, and the reasons for each are there. `<checkout>/target/release` is first on the path, so `ascribe` is the release build of `main` (`ascribe 0.2.0`, with the `permalinks` commands). Standard error goes to a file in the scratch folder, and the notes say what was in it. The session is run once, with ten minutes at most.

`Edit` and `Write` stay allowed, as in every session, so "don't edit" is the prompt's to hold and not the harness's.

## What the preparer checks afterwards

- `git status --short` and `git diff`: the prompt says "don't edit", so both should be empty. `ascribe check --config docs`, with its exit code.
- The agent's report, beside what the branch changed. The fixture's script lists it under "What review should show", and `git diff origin/main` has the lines:
  - `guides/rollouts.md`: an opening line added ("Rollouts are how most teams ship risky changes."), "instead of all of them" reworded to "before it reaches all of them", a line added after the percentages ("Start at 1% or lower for a change that touches payments or sign-in."), and the note under **Segments** removed.
  - `_fragments/prerequisites.md`, which `getting-started.md` and `guides/create-flags.md` include: "Node.js 20" became "Node.js 22", and a third item was added, "The **Release manager** role."
  - `ascribe.toml`: `version = "2.5.0"` became `"2.6.0"`, which changes every page that writes `{version}`.
  - `getting-started.md`: the Linux install command's address.
- Which of those the report mentions, and whether it says anything reads wrongly that the branch didn't change.
- Whether the agent could run the command the prompt gives, as given.

## Notes

One section per session, with phase 1's parts, in this order:

- **When, with what:** the date and time, Claude Code's version and model, the binary, the copy, and what the session's first event listed of the plugin's (its tools, its skill, its commands, the language server), with whether a synced skill or an account's connector was used.
- **Transcript:** its path.
- **Steps:** a numbered list, one line per step: what the agent ran or asked, then what came back, quoted from the transcript.
- **Tried first:** the commands it guessed, what it read, and where it went wrong.
- **What the hook or the plugin's tools said:** quoted.
- **Doubts:** each quoting the transcript.
- **Time:** by the clock, and as the session's last event reports it.
- **Result:** what the agent said it had done, then what the preparer's checks said.
- **Seen on the way:** one line per thing that looks like a finding, each quoting its evidence. Phase 5 writes the findings.

### From the prompt

- **When, with what:** 10 October 2026, 9:57 PM. Claude Code 2.1.296, model `claude-opus-5-5`, started from zsh on macOS with `target/release` first on the path (`ascribe 0.2.0`), in the fixture's checkout, `review-a-pull-request/fixture`, on the branch `scheduled-rollouts` at `d7db026`. No plugin: the session's first event listed no tool, skill, or command of Ascribe's, and no `LSP`. The user's synced skills and the account's connectors (`claude.ai Claude Docs` connected, three more `needs-auth`) were listed, and none was used. Standard error was empty.

- **Transcript:** [`transcripts/review-a-pull-request-prompt.jsonl`](transcripts/review-a-pull-request-prompt.jsonl).

- **Steps:**

  1. Bash: `git diff origin/main...HEAD; git diff 8254eaf HEAD --stat; ls -R . | head -80; cat ascribe.toml; which ascribe`. Came back: the branch's diff of four files, the listing, and the binary's path. There's no `ascribe.toml` at the root.
  2. Bash: `cd docs; cat ascribe.toml README.md; for b in site self-hosted; do for p in content/getting-started.md content/guides/create-flags.md content/guides/rollouts.md content/guides/self-hosting.md; do echo "=== $b $p"; ascribe render $p --build $b; done; done; grep -rn "2\.5\|2\.6\|Node\|Release manager\|role\|get.lantern\|lantern.enabled\|attribute" content src README.md ../README.md`. Came back: the content model, the eight renders, each starting with its page's title, and the search's lines.
  3. Bash: `cd docs; ascribe --help 2>&1 | head -40`. Came back: `(eval):cd:1: no such file or directory: docs`, since the shell was in `docs/` already, then the help.
  4. Bash: `ascribe check --format concise 2>&1 | tail -20; cat content/reference/*.md content/index.md content/_fragments/support.md; git log --format='%h %s%n%b' origin/main..HEAD; ascribe refs phrase:version 2>&1; ascribe refs content/_fragments/prerequisites.md 2>&1 | head -20`. Came back: `checked 10 files: 0 errors, 0 warnings`; the pages; the two commits, `d7db026 Add an opening line, and widen the advice` and `a5054a9 Document rollouts for 2.6`; `2 places use phrase:version.`, in `content/getting-started.md` and `content/guides/self-hosting.md`; and `2 places use content/_fragments/prerequisites.md.`
  5. Bash: `ascribe diff 8254eaf content/guides/rollouts.md --build site 2>&1 | head -60`, described as "Show the reader-visible diff of the rollouts page". Came back: `error: unexpected argument 'content/guides/rollouts.md' found`.
  6. Bash: `ascribe diff --help | head -30; ascribe diff content/guides/rollouts.md --since 8254eaf --build site 2>&1 | head -60`. Came back: the help, then `error: unexpected argument '--since' found`.
  7. Said: "Rendering the reader-visible diff of the rollouts page to confirm a couple of findings before I write them up." Bash: `ascribe diff content/guides/rollouts.md --base 8254eaf 2>&1 | head -80`. Came back: `error: a PAGE is named only with --format prompt`.
  8. Bash: `ascribe diff --base 8254eaf 2>&1 | head -150`, described as "Show the page-by-page reader-visible diff". Came back: `compared with 8254eaf (8254eaf)`, then the four pages of each build with their counts, the lines the prompt already had.
  9. Answered with eight numbered points under the four pages, starting "I read all four pages as rendered in both builds. `ascribe check` passes with 0 errors, but eight things read wrongly; the first three matter most." and ending "I made no edits."

- **Tried first:** `git diff origin/main...HEAD`, before anything of Ascribe's. Then the prompt's own command, `ascribe render <page> --build <build>`, for the four pages in the two builds, eight times in one loop. It ran it from `docs/`, having taken `Project: docs/` as where to stand, and every render worked.

  Then it tried three times to see what had changed on one page as a reader would, and couldn't:
  1. `ascribe diff 8254eaf content/guides/rollouts.md --build site`: the base as an argument, as `git diff` takes it.
  2. `ascribe diff content/guides/rollouts.md --since 8254eaf --build site`: a guessed option.
  3. `ascribe diff content/guides/rollouts.md --base 8254eaf`: the right option, and the answer was `error: a PAGE is named only with --format prompt`.

  The fourth, with no page, gave it the counts it had started with. It wrote its report from the renders of the branch, the raw `git diff`, and the commit messages. It never read a base revision's page as a reader saw it.

- **What the hook or the plugin's tools said:** nothing; the session had neither.

- **Doubts:**
  - It couldn't tell what the branch meant to change from what it changed by accident, and said so three times, each from the commit messages: "Nothing in the branch's commit messages mentions it", "This is also unexplained by the commits, and I can't check it from here; confirm the URL exists", and "The Segments note was removed, and neither commit message accounts for it."
  - Whether `2.6.0` belongs in the `self-hosted` build: "In `ascribe.toml`, `[builds.self-hosted]` filters on `self-hosted 2.5` and its comment says "what 2.5 doesn't have". Nothing renders differently today, but the 2.5-filtered docs now tell readers to expect 2.6.0."
  - Whether a new prerequisite belongs on the pages it reached: "It was added to the shared `_fragments/prerequisites.md`, so it appears on Get started and Create feature flags, but not on Roll out a flag, which doesn't include the fragment."

- **Time:** 43 seconds by the clock; the session reports 42.2 seconds, nine turns.

- **Result:** a review that covers every change on the branch and edits nothing. The preparer ran, in the checkout:
  - `git status --short`: nothing. `git diff`: nothing. The branch is still at `d7db026`.
  - `ascribe check --config docs`: exit 0, `checked 10 files: 0 errors, 0 warnings`.

  And read the report beside what the branch changed:
  - `guides/rollouts.md`, the opening line added: the report's point 6, "The new opening line takes the glossary link away from the definition." Checked by the preparer with `ascribe render content/guides/rollouts.md --build site`: the first line is `[Rollouts](https://docs.lantern.example/reference/glossary/#rollout) are how most teams ship risky changes.`, and "A rollout turns a flag on…" under it has no link. That's something only the built page shows; `git diff` doesn't.
  - "instead of all of them" reworded: point 5, "The reworded definition reverses its own point."
  - The line added after the percentages: point 7, "The new advice contradicts the example above it."
  - The note removed under **Segments**: point 8.
  - The fragment, "Node.js 20" to "Node.js 22": point 2. The third item, "The **Release manager** role.": point 1, on both pages that include it.
  - `version`, `2.5.0` to `2.6.0`: point 4 on `getting-started.md`, and the last line on `guides/self-hosting.md`, "The image tag is now `ghcr.io/lantern/server:2.6.0`."
  - The Linux install command's address: point 3.

  Every change is there, and nothing in the report is about text the branch didn't touch. Whether each "reads wrongly" is a reviewer's judgment; the fixture's changes were written to exercise review, not to be wrong, and the report treats several as faults.

  The agent could run the prompt's command as given, from `docs/`.

- **Seen on the way:**
  - The agent wanted one page's changes as a reader sees them, in the terminal, and no form of `ascribe diff` gave it. Three tries: `error: unexpected argument 'content/guides/rollouts.md' found`, `error: unexpected argument '--since' found`, and `error: a PAGE is named only with --format prompt`. The text format lists pages and counts (`guides/rollouts.md: 1 changed, 2 added, 1 removed`), and the prompt gives a command to read a page and none to see what changed on it. The agent's first look at the changes was `git diff`.
  - `ascribe diff` takes a page, and only to write a prompt about it: "a PAGE is named only with --format prompt". The agent's guess was that a page narrows the report, as a path narrows `git diff`.
  - `ascribe diff --base 8254eaf` starts `compared with 8254eaf (8254eaf)`: the revision as given, then the commit, the same thing twice when a commit is given.
  - The prompt names the pages from the project, `content/guides/rollouts.md`, and the project from the repository, `Project: docs/`. The agent read the second as "go there" and the first worked. Checked beforehand by the preparer: from the root, `ascribe render content/guides/rollouts.md --build site --config docs` answers `` error: `content/guides/rollouts.md` isn't a page or fragment of the project ``. The session didn't meet it.
  - The prompt says `content/guides/rollouts.md` changed "also through ascribe.toml", and the branch's one change to `ascribe.toml` is the `version` phrase, which `ascribe refs phrase:version` puts on two other pages: `2 places use phrase:version.`, `content/getting-started.md` and `content/guides/self-hosting.md`. The page doesn't write `{version}`. What moved on it with nothing in its file saying so is the glossary link, and the glossary is in `ascribe.toml`. Not looked into further; the agent didn't remark on it.
  - The agent ran `ascribe check --format concise` unasked, and opened its report with it: "`ascribe check` passes with 0 errors, but eight things read wrongly". The prompt doesn't mention the check.
  - The best catch in the report came from `ascribe render`, not from the diff: the glossary link moving to the new first line. The prompt's "as readers will see them" did that work.
  - The prompt says "don't edit", `Edit` and `Write` were allowed, and the agent edited nothing.
  - The fixture tells the agent what it is, twice: the README the script writes, and `docs/ascribe.toml`'s first comment, "One of three Ascribe projects in examples/monorepo; see its README." The agent read both and made nothing of either.

## What the session showed

One session, so nothing to compare it with but phase 1's.

- **The prompt was enough to start, and its one command worked.** The agent rendered all four pages in both builds in its second step, from the folder the prompt names.
- **It reviewed the branch's head, with `git diff` for what changed.** It asked `ascribe diff` three ways for one page's changes as a reader sees them and was refused each time, so the "as readers will see them" half of the review was the new pages only, never old beside new.
- **It guessed two things about `ascribe diff` wrongly:** that the base is an argument, and that a page narrows the report. `--since` was a third.
- **Nothing of the plugin's was there to use,** and nothing was missed that the notes can show: the session used `render`, `check`, `refs`, and `diff`, all from the prompt, the help, or a guess.

## Preparing a copy

For the preparer. From the checkout, with Node.js 24 (the script is TypeScript, run by `node` itself):

```shell
ASCRIBE_BIN=<checkout>/target/release/ascribe \
  node scripts/review-fixture/setup.ts --local --dir <scratch>/review-a-pull-request
cd <scratch>/review-a-pull-request/fixture/docs
ascribe diff --format prompt > <scratch>/review-prompt.txt
```

`ASCRIBE_BIN` is for the script's own check that the base and the branch's head pass `ascribe check --deny-warnings`; without it the script looks for `target/debug/ascribe` and skips the check when there isn't one. `--dir` must not exist with a `fixture` folder in it. The script links `examples/astro-site/node_modules` into `docs/` when that folder exists; run it before `pnpm install` has made one, so the session's copy holds the project and no packages.

The session runs in `<scratch>/review-a-pull-request/fixture`. `ascribe diff` there, from `docs/`, says `site: 4 pages changed` and `self-hosted: 4 pages changed`, and `git diff origin/main --stat` lists four files: `docs/ascribe.toml`, `docs/content/_fragments/prerequisites.md`, `docs/content/getting-started.md`, and `docs/content/guides/rollouts.md`.

The prompt's command, `ascribe render <page> --build <build>`, takes the pages as the prompt lists them (`content/guides/rollouts.md`) only from inside `docs/`. At the root, `ascribe render content/guides/rollouts.md --build site --config docs` answers `` error: `content/guides/rollouts.md` isn't a page or fragment of the project ``. That's left as it is: what the agent does about it is part of the session.
