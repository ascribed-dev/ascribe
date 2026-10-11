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

### With the plugin

- **When, with what:** 10 October 2026, 9:56 PM. Claude Code 2.1.296, model `claude-opus-5-5`, started from zsh on macOS with `target/release` first on the path (`ascribe 0.2.0`). The plugin from the checkout, `ascribe@inline` version 0.2.0. Copy `show-an-example-from-code-plugin`, in a scratch folder, prepared by the agent that ran it. The session's first event listed the plugin's nine tools (`mcp__plugin_ascribe_ascribe__ascribe_check` and the eight others), `LSP`, the skill `ascribe:ascribe`, and the commands `ascribe:check`, `ascribe:new-page`, and `ascribe:review`; the server `plugin:ascribe:ascribe` was `connected`. The user's synced skills and the account's connectors (`claude.ai Claude Docs` connected, three more `needs-auth`) were listed too, and none was used. Standard error was empty.

- **Transcript:** [`transcripts/show-an-example-from-code-plugin.jsonl`](transcripts/show-an-example-from-code-plugin.jsonl).

- **Steps:**

  1. `Skill`, `ascribe:ascribe`. Came back: `Launching skill: ascribe:ascribe`, then the skill's text.
  2. `Grep`, `API key` in `**/*.md`. Came back: `docs/keys.md` and `docs/install-agent.md`.
  3. `Read`, `agent/src/config.js`. The twelve lines.
  4. `Read`, `docs/keys.md`.
  5. `Read`, the plugin's `skills/ascribe/references/directives.md`. Came back, among nine directives: "`@snippet: <source>:<path>#<region>`: a code example from a file outside the pages", and "Which note types, dimensions, and sources a project has: `ascribe model`."
  6. Bash: `ls -a . docs; cat ascribe.toml docs/ascribe.toml AGENTS.md docs/AGENTS.md 2>/dev/null`. Came back: `Exit code 1`, the two listings, and `ascribe.toml`. The other three files don't exist.
  7. Bash: `(which ascribe || ls node_modules/.bin 2>/dev/null); ascribe model 2>&1 | head -80; ascribe explain --help 2>&1 | head -20; ascribe explain 2>&1 | grep -i -E "snippet|source|region"`. Came back: the binary's path; the content model, with page types, dimensions, phrases, and builds, and nothing about sources; `explain`'s help, with "`--list`  List every diagnostic's code and name, one per line, instead". The last command printed nothing: `ascribe explain` with no code is a usage error.
  8. Bash: `for c in $(ascribe explain --list | grep -i -E "snippet|source|region" | awk '{print $1}'); do ascribe explain $c; echo ------; done`. Came back: fifteen diagnostics explained, `ASC042`, `ASC123`, `ASC127` to `ASC135`, and `ASC137` to `ASC140`. In `ASC127`: "a snippet's file is named through a source declared in ascribe.toml, as `<source>:<path>`; declare one with `[sources.<name>]` and the `path` of the folder the code is in". In `ASC131`: "mark one with `:snippet-start: {region}` and `:snippet-end:` comments". In `ASC133`: "Fix `path`, which is relative to the folder ascribe.toml is in".
  9. Bash, one command that read the other page and the folders, then made three edits with a Python script and checked them. It put `// :snippet-start: find-key` and `// :snippet-end:` around four lines of `agent/src/config.js`; added `[sources.agent]` with `path = "agent"` to `ascribe.toml`; and added a section, "How the agent finds its key", to `docs/keys.md`, with one sentence and `@snippet: agent:src/config.js#find-key`. Then `ascribe check --format concise; echo "exit $?"; ascribe render docs/keys.md --build site`. Came back: `checked 4 files: 0 errors, 0 warnings`, `exit 0`, and the page with the four lines in a `js` block.
  10. Said: "Wiring the page to the code is done and the check passes; I'm confirming the final diff." Bash: `git status --short && git diff`. Came back: ` M agent/src/config.js`, ` M ascribe.toml`, ` M docs/keys.md`, and the diff.
  11. Answered: "The API keys page now has a "How the agent finds its key" section that pulls its code straight from `agent/src/config.js`, so it follows the code when that changes. `ascribe check` passes on the whole project with no errors or warnings, and the changes are uncommitted."
  12. The Stop hook ran.

- **Tried first:** the skill, then the code file and the page, then the skill's directive reference, where the one line on `@snippet` is. That line gives the address's form and nothing about where a source or a region comes from. The agent looked for both in this order:
  1. `AGENTS.md` and `docs/AGENTS.md`, which the skill says hold "a project's own rules". Neither exists in a project that has only the plugin.
  2. `ascribe model`, which the directive reference says shows the sources a project has. It had none to show.
  3. `ascribe explain` with no code, searched for "snippet", "source", or "region". Nothing came back.
  4. `ascribe explain --help`, then `ascribe explain --list`, searched the same way, and every match explained.

  The fourth worked. How to declare a source came from the message of `ASC127`, and the tags from the message of `ASC131`: two diagnostics the agent never had. It didn't open the address the directive reference ends on, `https://ascribed-dev.com/go/directives/`, or any other. It called none of the plugin's tools and didn't use `LSP`. It didn't run `ascribe drift`.

- **What the hook or the plugin's tools said:** the hook said nothing: `"hook_name": "Stop"`, `"stdout": ""`, `"stderr": ""`, `"exit_code": 0`, `"outcome": "success"`. No tool of the plugin's was called. Nothing was said after an edit: the three files were written by `python3` in Bash.

- **Doubts:**
  - How much of the file to show. The task said "how the agent finds its key", and the agent marked four of the twelve lines as a region rather than show the file whole. That meant changing the code file, which it said: "These are the only change to the code, and they mark which lines the page shows."
  - Whether its own sentence would fall behind. "I kept the lead-in sentence free of specifics such as which place is checked first, so the prose can't drift from the code." The sentence it wrote is "When the agent loads its configuration, it looks for the key like this:".
  - It saw a problem the task didn't ask about and left it: "The snippet shows that `QUILL_KEY` in the environment wins over `quill.yaml`, but the "Rotate keys" section only tells readers to put the new key in `quill.yaml`." That's about the made-up product's page, not Ascribe.
  - It said the page "follows the code when that changes" without changing the code to see. It had that from what `@snippet` is, not from a run.

- **Time:** 33 seconds by the clock; the session reports 31.7 seconds, twelve turns.

- **Result:** done, and right: the page takes the code from the file, and stays current. The agent knew from `ascribe check` and one render. The preparer ran, in the copy:
  - `ascribe check`: exit 0, `checked 4 files: 0 errors, 0 warnings`.
  - `git status --short`: ` M agent/src/config.js`, ` M ascribe.toml`, ` M docs/keys.md`. `git diff`: two comment lines added to the code file; four lines added to `ascribe.toml`, a comment, `[sources.agent]`, `path = "agent"`, and a blank; seven lines added to the page.
  - `node --check agent/src/config.js`: exit 0. The tags are comments.
  - `ascribe render docs/keys.md --build site`: the new section, with the region's four lines in a `js` block, dedented, and no tag lines.

  And in a second copy, with the agent's work committed and then `process.env.QUILL_KEY ?? config.cloud?.api_key` changed to `process.env.QUILL_API_KEY ?? process.env.QUILL_KEY ?? config.cloud?.api_key` in the code file, nothing else touched:
  - `ascribe render docs/keys.md --build site`: the block has the new line.
  - `ascribe check`: exit 0, `checked 4 files: 0 errors, 0 warnings`.
  - `ascribe drift`: exit 0, with "Examples that changed. The page shows the new code; check the words around it:", then `keys.md` and `agent:src/config.js#find-key (+1 −1)`.
  - `ascribe drift --exit-code`: the same, exit 1.
  - `ascribe diff`: `keys.md: 1 changed (through agent:src/config.js#find-key)` in each of the three builds.

- **Seen on the way:**
  - What taught the agent to declare a source and mark a region was `ascribe explain` on diagnostics it hadn't been given: `ASC127`'s "declare one with `[sources.<name>]` and the `path` of the folder the code is in" and `ASC131`'s "mark one with `:snippet-start: {region}` and `:snippet-end:` comments". The skill's directive reference has one line on `@snippet`, and it doesn't say either. The error messages were the documentation, and the agent found them by searching the list of codes for three words.
  - The skill sends the agent to a file the plugin doesn't give it: "A project's own rules (its page types, phrases, and the directives its pages use) are in the `AGENTS.md` beside its `ascribe.toml`". The agent tried `AGENTS.md` and `docs/AGENTS.md` and got `Exit code 1`. That file is `ascribe agents sync`'s, and a project with the plugin alone has none.
  - The directive reference says "Which note types, dimensions, and sources a project has: `ascribe model`." Checked afterwards by the preparer, in the copy with `[sources.agent]` declared: `ascribe model` prints page types, dimensions, phrases, and builds, and no sources, and `ascribe model --section sources` answers `error: invalid value 'sources' for '--section <NAME>'` with `[possible values: types, dimensions, phrases, features, glossary, widgets, builds]`.
  - `ascribe explain` with no code prints a usage error, and the agent's first search of it found nothing. `--list` was one `--help` away.
  - The agent never ran `ascribe drift`, the command for the task's second half, and the word isn't in the skill. It said the page "follows the code when that changes" on trust. The preparer's change to the code showed it does.
  - `ascribe drift` exits 0 when an example changed and its page's words didn't, and 1 only with `--exit-code`. The agent didn't get that far; it's here for phase 5 beside phase 1's finding on exit codes.
  - The agent declared the source as the whole `agent` folder: `path = "agent"` and no `include`. `ascribe check` took it. Every file under `agent/` is now readable from a page. Checked afterwards by the preparer, in the second copy: `@snippet: agent:package.json` on the page checks clean and renders the whole file.
  - The three edits were made by a script in the shell, so nothing came from the language server.
  - The hook had nothing to say, as in phase 1: the agent had checked already.

## What the session showed

One session, so nothing to compare it with but phase 1's.

- **It got the right answer by the long way round.** `@snippet` was in the skill's directive reference. The two things it needs, a source and a region, weren't, and the agent went through a missing `AGENTS.md`, `ascribe model`, and a usage error before it thought of reading the explanations of errors it didn't have.
- **The plugin was used here, and it was the skill and its directive reference.** The tools, the commands, and the language server weren't touched, and the hook was silent.
- **The agent did the first half of the task and trusted the second.** It showed the code from the file, and said the page would follow the code. It didn't change the code to see, and didn't find `ascribe drift`. Both hold, by the preparer's checks.

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
