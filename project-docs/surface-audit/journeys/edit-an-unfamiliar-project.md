# Arrive in an unfamiliar project and make a correct edit

Part of [The surface audit](../README.md), [phase 3](../phase-3-journeys.md), journey 7. The script first, the same for all three sessions; then each session's notes.

## The task

> Add a policy on laptops to the security handbook: disks are encrypted, screens lock after five minutes, and a lost laptop is reported within an hour.

That sentence is the whole prompt: a one-line request, as someone would give it to an agent in a repository neither has read. It doesn't say where the security handbook is, that it's a project of its own, or what a policy is there.

What a check can't confirm is whether the page is a policy as this project means one. The repository has three Ascribe projects and no `ascribe.toml` at its root. The security handbook is the innermost, `handbook/pages/security/`, nested in the engineering handbook's content root. Its content model gives a file under `policies/` the type `policy`, whose frontmatter needs `owner`, `review` (`quarterly` or `yearly`), and `effective` (a date). It has a note type, `policy`, that its two policies state their rule in, and phrases for the team's channel and address. A page written anywhere else checks clean with a `title` alone.

## Starting point

A fresh copy of `examples/monorepo`, without the example's `README.md`, as a git repository with one commit. `docs/README.md` stays: it's the Lantern docs' own, part of the made-up repository. Each of the three projects checks clean.

## The three sessions

| Session | With | Without |
|---|---|---|
| With the plugin | The plugin, loaded from the checkout | `AGENTS.md`, `.agents/`, `.claude/` |
| With what `ascribe agents sync` wrote | The files `ascribe agents sync` writes, in the one commit | The plugin |
| With nothing | The copy alone | The plugin, and anything `ascribe agents sync` writes |

The third is the baseline: what an agent does with no help shows what the help is worth, and what it guesses Ascribe's commands are called.

With the plugin:

```shell
claude -p "Add a policy on laptops to the security handbook: disks are encrypted, screens lock after five minutes, and a lost laptop is reported within an hour." \
  --setting-sources project \
  --plugin-dir <checkout>/plugins/ascribe \
  --allowedTools "Read,Edit,Write,Glob,Grep,Bash,mcp__plugin_ascribe_ascribe__*" \
  --no-session-persistence --include-hook-events \
  --output-format stream-json --verbose \
  > <checkout>/project-docs/surface-audit/journeys/transcripts/edit-an-unfamiliar-project-plugin.jsonl
```

The other two, each in its own copy, with `sync` or `nothing` in the transcript's name:

```shell
claude -p "Add a policy on laptops to the security handbook: disks are encrypted, screens lock after five minutes, and a lost laptop is reported within an hour." \
  --setting-sources project \
  --allowedTools "Read,Edit,Write,Glob,Grep,Bash" \
  --no-session-persistence --include-hook-events \
  --output-format stream-json --verbose \
  > <checkout>/project-docs/surface-audit/journeys/transcripts/edit-an-unfamiliar-project-<setup>.jsonl
```

The flags are [phase 1's](fix-a-broken-link.md#what-changed-in-these-two-commands-and-why), flag for flag, and the reasons for each are there. `<checkout>/target/release` is first on the path, so `ascribe` is the release build of `main` (`ascribe 0.2.0`, with the `permalinks` commands). Standard error goes to a file in the scratch folder, and the notes say what was in it. The session is run once, with ten minutes at most.

The second session's files are what `ascribe agents sync` writes with no option, run once in each project: an `AGENTS.md` beside each `ascribe.toml`, a block for each project in the `AGENTS.md` at the repository's root, and the skill in `.agents/skills/ascribe/`. No `--target claude` is given, so there's no `CLAUDE.md` and nothing under `.claude/`. Whether Claude Code reads what the plain command writes is part of what the session shows.

## What the preparer checks afterwards

For each session:

- `ascribe check` in each of the three projects (`--config docs`, `--config handbook`, `--config handbook/pages/security`), with each exit code; `git status --short` and `git diff`.
- Where the page is, and its type: `ascribe outline <page> --config handbook/pages/security`.
- Its frontmatter, beside the two policies already there (`secrets.md`, `access-reviews.md`).
- Whether a reader can reach it: whether the security handbook's `index.md` links to it, and whether `ascribe check` gives the orphan advice, `ASC154`.
- Whether it states its rule in a `policy` note, as the other two do, and writes the project's phrases (`{team}`, `{channel}`, `{report}`) where it names what they stand for.
- `ascribe report inventory --config handbook/pages/security`, which counts policies by owner.
- `ascribe render <page> --build site --config handbook/pages/security`: the page as a reader sees it.

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

For the preparer. For the first and third sessions:

```shell
cp -R <checkout>/examples/monorepo <scratch>/edit-an-unfamiliar-project-<setup>
cd <scratch>/edit-an-unfamiliar-project-<setup>
mv README.md ../removed-README.md
git init -q -b main && git add -A && git -c user.name=prep -c user.email=prep@example.com commit -qm "Lantern"
```

For the second, `ascribe agents sync` runs before the commit, and after `git init`, since the command writes to the repository's root and finds it from git:

```shell
cp -R <checkout>/examples/monorepo <scratch>/edit-an-unfamiliar-project-sync
cd <scratch>/edit-an-unfamiliar-project-sync
mv README.md ../removed-README.md
git init -q -b main
(cd docs && ascribe agents sync)
(cd handbook && ascribe agents sync)
(cd handbook/pages/security && ascribe agents sync)
git add -A && git -c user.name=prep -c user.email=prep@example.com commit -qm "Lantern"
```

It's run three times because it's run in a project: at the repository's root it answers `error: no ascribe.toml found in … or any parent directory; run ascribe from a project, or pass --config` and exits 2. The first run writes `AGENTS.md`, `../AGENTS.md`, `../.agents/skills/ascribe/SKILL.md`, and `../.agents/skills/ascribe/references/directives.md`; the other two write their own `AGENTS.md`, add their block to the root's, and say `unchanged` for the skill.

In every copy, each project checks clean before the session: `ascribe check` says `checked 10 files: 0 errors, 0 warnings` in `docs`, `checked 7 files` in `handbook`, and `checked 5 files` in `handbook/pages/security`.
