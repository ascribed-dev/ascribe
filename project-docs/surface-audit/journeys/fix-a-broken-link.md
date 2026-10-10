# Fix a broken link

Part of [The surface audit](../README.md), [phase 1](../phase-1-thread.md). The script first, the same for all four walks; then each walk's notes, written as it went.

## The task

> The project has a broken link. Find it and fix it, so the project checks clean.

That sentence is all the walker is told. Not which page, not which link, not what the fix is.

## Rules for the walker

- **Do what you'd do**, in the surface the walk names, and nothing outside it.
- **Note each step as you take it**: what you typed, clicked, or asked, and what came back, quoted. Not from memory afterwards.
- **Note each moment of doubt**: what you expected, what happened, how you got past it.
- **Don't fix anything else**, don't look at Ascribe's code or its docs' source, and don't read a plan to get unstuck. The published docs, `--help`, and the editor's own help are part of the surface: use them, and note that you did.
- **Stuck for five minutes: stop.** Note where, and what you tried. Getting stuck is the finding.
- **Time it**, from opening the project to deciding you're done.
- **Don't read "Preparing a copy"** at the end before your walk. It says where the link is.

## Starting point

A fresh copy of `examples/quill`, a git repository with one commit, in which one link's target was renamed. Each walk gets its own copy, prepared by someone other than the walker as "Preparing a copy" says. The copy holds the project and nothing else, except what the walk's row below adds.

The binary is the release build of `main` (`cargo build --release -p ascribe-cli`), and its version is noted with each walk.

## The four walks

| Walk | By | With | Without |
|---|---|---|---|
| In the editor | The maintainer | VS Code with the Ascribe extension, the copy opened as the only workspace folder | The integrated terminal, any `ascribe` command, any agent |
| At the command line | The maintainer | A terminal with `ascribe` on the path, and a plain editor (`nano`, `vim`, or TextEdit) | VS Code, the extension, any agent |
| Through the hook | An agent, fresh session | Claude Code in a terminal, with the plugin, in a copy with nothing else | `AGENTS.md`, `.claude/`, `.mcp.json`, the user's own settings and plugins |
| Through MCP | An agent, fresh session | Claude Code in a terminal, with `.mcp.json` registering `ascribe mcp`, and no shell | The plugin, `AGENTS.md`, `.claude/`, Bash |

### In the editor

Open the copy's folder in VS Code and start from there. The extension's settings stay at their defaults, so the project's server starts when you open one of its pages; whether the problem is found before that is part of the walk. **Prompt agent** may be opened to see what it builds, and nothing is sent. Done when you believe the project has no problems left: say how you know.

### At the command line

`cd` into the copy and start from there. `ascribe --help` and the published docs are allowed. Done when `ascribe check` exits 0: say what you ran to get there.

### Through the hook

Claude Code, in a terminal, in the copy, with a clean profile so nothing of the user's reaches the session, and the plugin loaded from this checkout. The task sentence is the whole prompt, and the run is recorded whole:

```shell
export CLAUDE_CONFIG_DIR="$(mktemp -d)"
claude -p "The project has a broken link. Find it and fix it, so the project checks clean." \
  --plugin-dir <checkout>/plugins/ascribe \
  --allowedTools Read,Edit,Write,Glob,Grep,Bash \
  --output-format stream-json --verbose \
  > <checkout>/project-docs/surface-audit/journeys/transcripts/fix-a-broken-link-hook.jsonl
```

The agent has a shell, so it may run `ascribe check` itself; what the hook adds is what it hears after each edit and at the stop. When it stops, the preparer runs `ascribe check` in the copy and notes the exit code and the diff.

### Through MCP

Claude Code, in a terminal, in the copy, with a clean profile, no plugin, and the copy's `.mcp.json` as the agents guide shows it (`"command": "ascribe", "args": ["mcp"]`, with `ascribe` on the path). Bash is left out, so the agent has no shell: the MCP server exists for "an agent with no shell", and in the agents plan's runs an agent with a shell never called a tool. The run is recorded as the hook's is:

```shell
export CLAUDE_CONFIG_DIR="$(mktemp -d)"
claude -p "The project has a broken link. Find it and fix it, so the project checks clean." \
  --allowedTools "Read,Edit,Write,Glob,Grep,mcp__ascribe__*" \
  --disallowedTools Bash \
  --output-format stream-json --verbose \
  > <checkout>/project-docs/surface-audit/journeys/transcripts/fix-a-broken-link-mcp.jsonl
```

When it stops, the preparer runs `ascribe check` in the copy and notes the exit code and the diff.

## Notes

One section per walk. Each has the same parts; an agent's has two more.

### In the editor

*Not yet walked.*

- **When, with what:** date; VS Code and extension versions; the binary the extension ran (`ascribe --version`, from its output channel).
- **Steps:**

  | # | Did | Saw |
  |---|---|---|
  | 1 | | |

- **Doubts:** for each, what was expected, what happened, how it was resolved.
- **Time:** from opening the folder to done.
- **Result:** fixed or not; how the walker knew; what the preparer's `ascribe check` said afterwards.
- **Seen on the way:** one line per thing that looks like a finding, to move to `findings.md`.

### At the command line

*Not yet walked.*

The same parts. "Did" is the command as typed; "Saw" is its output, quoted, with its exit code where it mattered.

### Through the hook

*Not yet walked.*

The same parts, and:

- **Transcript:** `transcripts/fix-a-broken-link-hook.jsonl`.
- **Tried first:** what the agent did before the thing that worked, in order: the commands it guessed, what it read, where it went wrong.
- **What the hook said:** each hook message the agent received, quoted.

### Through MCP

*Not yet walked.*

The same parts as the hook's, with the tools it called in place of what the hook said.

## What the walks had in common

*After all four.* Where the four paths met, and where they didn't; whether the maintainer's two together took under half an hour, and if not, what in the script to change before phase 3.

## Preparing a copy

For the preparer, not the walker.

```shell
cp -R <checkout>/examples/quill <scratch>/fix-a-broken-link-<walk>
cd <scratch>/fix-a-broken-link-<walk>
mv docs/keys.md docs/api-keys.md
git init -q && git add -A && git -c user.name=prep -c user.email=prep@example.com commit -qm "Quill"
```

One commit, with the rename already in it, so `git log` and `git status` say nothing. The rename breaks one link, `[](keys.md#rotate-keys)` in the troubleshooting note of `docs/install-agent.md`, line 93. Nothing else links to `keys.md`.

For the MCP walk, add `.mcp.json` before the commit:

```json
{
  "mcpServers": {
    "ascribe": {
      "command": "ascribe",
      "args": ["mcp"]
    }
  }
}
```

Before handing a copy over, run `ascribe check` in it. With the release build of `main` on 10 October 2026, it exits 1 and reports two things: the error, and advice that the renamed page is now an orphan, since the only link to it is the broken one:

```text
[ASC036] Error: `keys.md` doesn't exist
    ╭─[ docs/install-agent.md:93:90 ]
…
[ASC154] Advice: no other page links to this page or includes it, so readers reach it only through the site's own navigation or search, if those list it
   ╭─[ docs/api-keys.md:2:8 ]
…
checked 4 files: 1 error, 0 warnings, 1 advice
```

The fix that clears both is `[](api-keys.md#rotate-keys)`. A walker who renames the file back also clears both; note which they chose.
