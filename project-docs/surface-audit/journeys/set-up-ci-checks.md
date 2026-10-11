# Set up checks in CI

Part of [The surface audit](../README.md), [phase 3](../phase-3-journeys.md), journey 6. The script first; then the session's notes.

## The task

> Set up GitHub Actions for this repository, so a pull request that breaks the docs fails its checks.

That sentence is the whole prompt. It doesn't say which commands the workflow should run, or how the runner gets Ascribe. What a check can't confirm is the workflow: `ascribe check` says nothing about a file under `.github/`, and the result that matters is a workflow that fails when it should.

## Starting point

A fresh copy of `examples/quill`, without its `README.md`, as a git repository with one commit. It has no `.github/` folder and no `package.json`, so nothing in it says how Ascribe is installed: the getting-started page's workflow runs `npm ci` and `npx ascribe check`, which needs a `package.json` and a lockfile this project doesn't have.

## The session

One, with the plugin:

```shell
claude -p "Set up GitHub Actions for this repository, so a pull request that breaks the docs fails its checks." \
  --setting-sources project \
  --plugin-dir <checkout>/plugins/ascribe \
  --allowedTools "Read,Edit,Write,Glob,Grep,Bash,mcp__plugin_ascribe_ascribe__*" \
  --no-session-persistence --include-hook-events \
  --output-format stream-json --verbose \
  > <checkout>/project-docs/surface-audit/journeys/transcripts/set-up-ci-checks-plugin.jsonl
```

The flags are [phase 1's](fix-a-broken-link.md#what-changed-in-these-two-commands-and-why), flag for flag, and the reasons for each are there. `<checkout>/target/release` is first on the path, so `ascribe` is the release build of `main` (`ascribe 0.2.0`, with the `permalinks` commands). Standard error goes to a file in the scratch folder, and the notes say what was in it. The session is run once, with ten minutes at most.

## What the preparer checks afterwards

- `ascribe check`, with its exit code; `git status --short` and `git diff`.
- The workflow, read whole and quoted in the notes: what starts it, how it installs Ascribe and at which version, which commands it runs, and whether its YAML parses.
- Whether it would run as written: an action or a package it names that doesn't exist, an `npm ci` with no lockfile, a version that isn't published (`npm view @ascribed/cli versions`).
- Its commands, run by hand in a copy of the session's copy, twice: as the session left it, where they should pass, and with a link broken (`keys.md` renamed, as in phase 1), where they should fail. GitHub isn't involved: the workflow itself isn't run.
- Beside it, the workflow the getting-started page gives, and where the agent's differs.

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

For the preparer.

```shell
cp -R <checkout>/examples/quill <scratch>/set-up-ci-checks-plugin
cd <scratch>/set-up-ci-checks-plugin
mv README.md ../removed-README.md
git init -q -b main && git add -A && git -c user.name=prep -c user.email=prep@example.com commit -qm "Quill"
ascribe check    # exits 0: checked 4 files: 0 errors, 0 warnings
```

The copy has no remote, so nothing can be pushed and no workflow runs. Before the session, note what `npm view @ascribed/cli dist-tags` says: a workflow that installs from `npm` gets a release, and the binary on the path is `main`'s.
