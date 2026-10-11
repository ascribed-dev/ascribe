# Add a section that differs by platform

Part of [The surface audit](../README.md), [phase 3](../phase-3-journeys.md), journey 2. The script first; then the session's notes.

## The task

> Add a section to the install page on starting the agent when the computer starts. It's done differently on macOS, Linux, and Windows, and a reader should be able to see just their own system's steps.

That sentence is the whole prompt. It doesn't name a directive, a dimension, or `ascribe.toml`. The project varies by package manager (`pm`) and by deployment (`deployment`) and has nothing for an operating system, so the agent has to work out that the content model needs a new dimension, or do without one. What a check can't confirm is how the section reads: one system's steps at a time, in each of the project's three builds.

## Starting point

A fresh copy of `examples/quill`, without its `README.md`, as a git repository with one commit. `ascribe check` exits 0 in it.

## The session

One, with the plugin:

```shell
claude -p "Add a section to the install page on starting the agent when the computer starts. It's done differently on macOS, Linux, and Windows, and a reader should be able to see just their own system's steps." \
  --setting-sources project \
  --plugin-dir <checkout>/plugins/ascribe \
  --allowedTools "Read,Edit,Write,Glob,Grep,Bash,mcp__plugin_ascribe_ascribe__*" \
  --no-session-persistence --include-hook-events \
  --output-format stream-json --verbose \
  > <checkout>/project-docs/surface-audit/journeys/transcripts/add-a-platform-section-plugin.jsonl
```

The flags are [phase 1's](fix-a-broken-link.md#what-changed-in-these-two-commands-and-why), flag for flag, and the reasons for each are there. `<checkout>/target/release` is first on the path, so `ascribe` is the release build of `main` (`ascribe 0.2.0`, with the `permalinks` commands). Standard error goes to a file in the scratch folder, and the notes say what was in it. The session is run once, with ten minutes at most.

## What the preparer checks afterwards

- `ascribe check`, with its exit code; `git status --short` and `git diff`.
- `ascribe model --section dimensions`: whether the systems are a dimension of the content model, and what it's called.
- `ascribe render docs/install-agent.md --build <build>` for `site`, `cloud`, and `self-managed-3.3`: whether the new section is there in each, and how the three systems' steps are set apart in the plain Markdown.
- `ascribe build`, then the `site` output of the page: whether the three systems are tabs of one switcher (`<ascribe-tabs>`), which is what lets a reader see one at a time.
- Whether the rest of the page is unchanged.

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
cp -R <checkout>/examples/quill <scratch>/add-a-platform-section-plugin
cd <scratch>/add-a-platform-section-plugin
mv README.md ../removed-README.md
git init -q -b main && git add -A && git -c user.name=prep -c user.email=prep@example.com commit -qm "Quill"
ascribe check    # exits 0: checked 4 files: 0 errors, 0 warnings
```

"The install page" is `docs/install-agent.md`. The copy has no `.gitignore`, so an agent that runs `ascribe build` leaves `.ascribe/` untracked; that shows in `git status` and isn't the agent's mistake.
