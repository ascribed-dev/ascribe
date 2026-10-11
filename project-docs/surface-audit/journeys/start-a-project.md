# Start a project and publish a first page

Part of [The surface audit](../README.md), [phase 3](../phase-3-journeys.md), journey 1. The script first; then the session's notes.

## The task

> Follow https://ascribed-dev.com/getting-started/ to start a documentation project in this folder, write a first page about making tea, and publish it as a website I can open on this machine.

That sentence is the whole prompt. It gives the getting-started page's address and nothing about what's on it. "A website I can open" is the result a check can't confirm: `ascribe check` passing says nothing about whether there's a site with the page in it.

## Starting point

An empty folder: no files, and not a git repository, since there's nothing in it for a commit to hide. This is the coldest of the journeys. Every place the page and what happens differ is a note.

## The session

One, with no plugin, and with `WebFetch` allowed so the agent can read the page it's given:

```shell
claude -p "Follow https://ascribed-dev.com/getting-started/ to start a documentation project in this folder, write a first page about making tea, and publish it as a website I can open on this machine." \
  --setting-sources project \
  --allowedTools "Read,Edit,Write,Glob,Grep,Bash,WebFetch" \
  --no-session-persistence --include-hook-events \
  --output-format stream-json --verbose \
  > <checkout>/project-docs/surface-audit/journeys/transcripts/start-a-project-nothing.jsonl
```

The flags are [phase 1's](fix-a-broken-link.md#what-changed-in-these-two-commands-and-why), flag for flag, and the reasons for each are there. `<checkout>/target/release` is first on the path, so `ascribe` is the release build of `main` (`ascribe 0.2.0`, with the `permalinks` commands). Standard error goes to a file in the scratch folder, and the notes say what was in it. The session is run once, with ten minutes at most.

The path is a departure from "an empty folder, and nothing else", made because every session and check in this phase uses the one binary: a bare `ascribe` in this session is the release build of `main`, while `npx ascribe`, which the page tells a reader to type, is whatever `npm` installed. The notes say which the agent ran, and what version each was.

## What the preparer checks afterwards

- The files the session left, listed, without `node_modules`.
- `ascribe check` in the folder, with its exit code; and `npx ascribe --version`, for what `npm` installed.
- Whether there's a built site, and whether the page about tea is in it: the built HTML is read for the page's title and text.
- Whether what the agent said to open does open: the file exists, or the address answers.
- Each step of the getting-started page, beside what the agent did for it.

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
mkdir <scratch>/start-a-project-nothing
```

Nothing else. Before the session, see that `https://ascribed-dev.com/getting-started/` answers 200, and note what `npm view @ascribed/cli dist-tags` says, since the page's commands install the `latest` release and the binary on the path is `main`'s.
