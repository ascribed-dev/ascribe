# Show an example from the code, and keep it current

Part of [The surface audit](../README.md), [phase 3](../phase-3-journeys.md), journey 3. The script first; then the session's notes.

## The task

> The API keys page should show how the agent finds its key, using the code in agent/src/config.js itself, so the page stays right when that code changes.

That sentence is the whole prompt. It names the page and the code file, and not `@snippet`, a source, or `ascribe drift`. What a check can't confirm is the second half: that the page changes when the code does. A code block pasted into the page checks clean and is exactly what the task asks not to do.

## Starting point

A fresh copy of `examples/quill`, without its `README.md`, with two files added for the example to come from, as a git repository with one commit:

- `agent/src/config.js`, twelve lines: `loadConfig`, which reads `quill.yaml` and takes the key from `QUILL_KEY` or from `cloud.api_key`.
- `agent/package.json`, so the folder looks like the agent's package.

The project's `ascribe.toml` is at the copy's root with `content-root = "docs"`, and it declares no source. A page can't read `agent/` until `ascribe.toml` names it.

## The session

One, with the plugin:

```shell
claude -p "The API keys page should show how the agent finds its key, using the code in agent/src/config.js itself, so the page stays right when that code changes." \
  --setting-sources project \
  --plugin-dir <checkout>/plugins/ascribe \
  --allowedTools "Read,Edit,Write,Glob,Grep,Bash,mcp__plugin_ascribe_ascribe__*" \
  --no-session-persistence --include-hook-events \
  --output-format stream-json --verbose \
  > <checkout>/project-docs/surface-audit/journeys/transcripts/show-an-example-from-code-plugin.jsonl
```

The flags are [phase 1's](fix-a-broken-link.md#what-changed-in-these-two-commands-and-why), flag for flag, and the reasons for each are there. `<checkout>/target/release` is first on the path, so `ascribe` is the release build of `main` (`ascribe 0.2.0`, with the `permalinks` commands). Standard error goes to a file in the scratch folder, and the notes say what was in it. The session is run once, with ten minutes at most.

## What the preparer checks afterwards

- `ascribe check`, with its exit code; `git status --short` and `git diff`.
- How the code reached the page: a `@snippet` from a source in `ascribe.toml`, or a copy in a fenced block.
- `ascribe render docs/keys.md --build site`: the page as built, with the code in it.
- In a copy of the session's copy, with the agent's work committed: change a line of `agent/src/config.js`, then `ascribe render docs/keys.md --build site` again, `ascribe check`, `ascribe drift`, and `ascribe drift --exit-code`. The example is kept current if the render has the new line and `drift` names the page.
- Whether `agent/src/config.js` itself was changed (tags for a region are a change to it), and whether it still parses (`node --check`).

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
cp -R <checkout>/examples/quill <scratch>/show-an-example-from-code-plugin
cd <scratch>/show-an-example-from-code-plugin
mv README.md ../removed-README.md
mkdir -p agent/src
```

`agent/src/config.js`:

```js
// Reads the agent's configuration: quill.yaml, and the API key.
import { readFileSync } from "node:fs";
import { parse } from "yaml";

export function loadConfig(path = "quill.yaml") {
  const config = parse(readFileSync(path, "utf8"));
  const key = process.env.QUILL_KEY ?? config.cloud?.api_key;
  if (!key) {
    throw new Error("No API key: set QUILL_KEY, or cloud.api_key in quill.yaml.");
  }
  return { ...config, key };
}
```

`agent/package.json`:

```json
{
  "name": "@quill/agent",
  "version": "3.4.1",
  "type": "module",
  "main": "src/config.js",
  "dependencies": {
    "yaml": "^2.5.0"
  }
}
```

Then:

```shell
git init -q -b main && git add -A && git -c user.name=prep -c user.email=prep@example.com commit -qm "Quill"
ascribe check    # exits 0: checked 4 files: 0 errors, 0 warnings
```

"The API keys page" is `docs/keys.md`. The copy has one commit on `main`, so `ascribe drift`, which compares with the default branch, compares the working tree with that commit: before the preparer's own check, the agent's work is committed in a second copy, so `drift` sees only the change to the code.
