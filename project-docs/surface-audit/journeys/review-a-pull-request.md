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
