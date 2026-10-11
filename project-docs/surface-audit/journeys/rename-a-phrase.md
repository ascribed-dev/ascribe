# Rename a phrase everywhere

Part of [The surface audit](../README.md), [phase 3](../phase-3-journeys.md), journey 4. The script first; then the session's notes.

## The task

> We're renaming the `cloud` phrase to `hosted`. Change it everywhere it's used; readers shouldn't see any difference.

That sentence is the whole prompt. "Phrase" is the project's own word, as a user who has one would say it. What a check can't confirm is the last clause. In this project `cloud` is four things: the phrase (`cloud = "Quill Cloud"` under `[phrases]`, written `{cloud}` in pages), a value of the `deployment` dimension, the name of a build, and a key in a YAML example. A rename that takes too much checks clean if it takes it consistently, and changes the builds.

## Starting point

A fresh copy of `examples/quill`, without its `README.md`, as a git repository with one commit. `ascribe check` exits 0 in it.

## The session

One, with the plugin:

```shell
claude -p "We're renaming the `cloud` phrase to `hosted`. Change it everywhere it's used; readers shouldn't see any difference." \
  --setting-sources project \
  --plugin-dir <checkout>/plugins/ascribe \
  --allowedTools "Read,Edit,Write,Glob,Grep,Bash,mcp__plugin_ascribe_ascribe__*" \
  --no-session-persistence --include-hook-events \
  --output-format stream-json --verbose \
  > <checkout>/project-docs/surface-audit/journeys/transcripts/rename-a-phrase-plugin.jsonl
```

The flags are [phase 1's](fix-a-broken-link.md#what-changed-in-these-two-commands-and-why), flag for flag, and the reasons for each are there. `<checkout>/target/release` is first on the path, so `ascribe` is the release build of `main` (`ascribe 0.2.0`, with the `permalinks` commands). Standard error goes to a file in the scratch folder, and the notes say what was in it. The session is run once, with ten minutes at most.

## What the preparer checks afterwards

- `ascribe check`, with its exit code; `git status --short` and `git diff`.
- `ascribe refs phrase:cloud` and `ascribe refs phrase:hosted`: the first should say the phrase doesn't exist, and the second should list the five places the first listed before.
- `ascribe render docs/<page>.md --build <build>` for the three pages in the three builds, compared with the same nine renders made before the session: no difference is what "readers shouldn't see any difference" means.
- `ascribe model --section dimensions` and `--section builds`: the `deployment` value `cloud` and the build `cloud` are still called that.

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
cp -R <checkout>/examples/quill <scratch>/rename-a-phrase-plugin
cd <scratch>/rename-a-phrase-plugin
mv README.md ../removed-README.md
git init -q -b main && git add -A && git -c user.name=prep -c user.email=prep@example.com commit -qm "Quill"
ascribe check    # exits 0: checked 4 files: 0 errors, 0 warnings
```

Before the session, keep the nine renders somewhere outside the copy, and note what `ascribe refs phrase:cloud` says. With the release build of `main` on 10 October 2026:

```text
docs/install-agent.md:7:181: phrase
docs/install-agent.md:61:12: phrase
docs/install-agent.md:83:1: phrase
docs/install-agent.md:83:56: phrase
docs/keys.md:9:12: phrase

5 places use phrase:cloud.
```

The other things called `cloud`, which a right answer leaves alone: in `ascribe.toml`, `values = ["cloud", "self-managed"]`, `versionless = ["cloud"]`, `labels = { cloud = "Quill Cloud" }`, `[builds.cloud]`, `variants = { deployment = "cloud" }`, and `availability = { filter = "cloud" }`; in `docs/install-agent.md`, `available: cloud, self-managed preview 3.3`, `@variant {deployment=cloud}:`, the YAML key `cloud:`, and `@available: cloud, self-managed preview 3.4`. Don't run `ascribe build` in the copy: it leaves `.ascribe/` behind.
