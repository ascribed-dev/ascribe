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
- **Your agent is part of your editor.** Use it as you would, and note what you asked, what it answered, and what it could and couldn't do. Don't ask it about Ascribe's code or plans, and don't correct it: what it gets wrong is a finding too.
- **Stuck for five minutes: stop.** Note where, and what you tried. Getting stuck is the finding.
- **Time it**, from opening the project to deciding you're done.
- **Don't read "Preparing a copy"** at the end before your walk. It says where the link is.

## Starting point

A fresh copy of `examples/quill`, a git repository with one commit, in which one link's target was renamed. Each walk gets its own copy, prepared by someone other than the walker as "Preparing a copy" says. The copy holds the project and nothing else, except what the walk's row below adds.

The binary is the release build of `main` (`cargo build --release -p ascribe-cli`), and its version is noted with each walk. The build of 10 October 2026 is from the audit's branch, which differs from `main` in `project-docs/` only: the same binary.

## The four walks

| Walk | By | With | Without |
|---|---|---|---|
| In the editor | The maintainer | VS Code with the Ascribe extension, the copy opened as the only workspace folder, and the agent the maintainer has there | The integrated terminal, any `ascribe` command |
| At the command line | The maintainer | A terminal with `ascribe` on the path, and a plain editor (`nano`, `vim`, or TextEdit) | VS Code, the extension, any agent |
| Through the hook | An agent, fresh session | Claude Code in a terminal, with the plugin, in a copy with nothing else | `AGENTS.md`, `.claude/`, `.mcp.json`, the user's own settings and plugins |
| Through MCP | An agent, fresh session | Claude Code in a terminal, with `.mcp.json` registering `ascribe mcp`, and no shell | The plugin, `AGENTS.md`, `.claude/`, Bash |

### In the editor

Open the copy's folder in VS Code and start from there. VS Code opens a folder it hasn't seen in Restricted Mode, where the extension reports nothing: trust the folder (**Manage** in the banner, then **Trust**) and note what you saw before you did. The extension's settings stay at their defaults, so the project's server starts when you open one of its pages; whether the problem is found before that is part of the walk. Your agent is allowed, as it is in your real editor; the script first said nothing is sent to one, and the maintainer's walk went straight past that, which settled it. Your own editor settings stay as they are: the walk is your real editor, and if saving does more than save, note it and stop. Done when you believe the project has no problems left: say how you know, and which builds that covers.

### At the command line

`cd` into the copy and start from there. `ascribe --help` and the published docs are allowed. Done when `ascribe check` exits 0: say what you ran to get there.

### Through the hook

Claude Code, in a terminal, in the copy, with the plugin loaded from this checkout and only the project's own settings read. The task sentence is the whole prompt, and the run is recorded whole, hook events included:

```shell
claude -p "The project has a broken link. Find it and fix it, so the project checks clean." \
  --setting-sources project \
  --plugin-dir <checkout>/plugins/ascribe \
  --allowedTools "Read,Edit,Write,Glob,Grep,Bash,mcp__plugin_ascribe_ascribe__*" \
  --no-session-persistence --include-hook-events \
  --output-format stream-json --verbose \
  > <checkout>/project-docs/surface-audit/journeys/transcripts/fix-a-broken-link-hook.jsonl
```

The agent has a shell, so it may run `ascribe check` itself; what the plugin adds is its skill, its MCP server, the language server's diagnostics after an edit, and the check at the stop. When it stops, the preparer runs `ascribe check` in the copy and notes the exit code and the diff.

### Through MCP

Claude Code, in a terminal, in the copy, with no plugin, and the copy's `.mcp.json` as the agents guide shows it (`"command": "ascribe", "args": ["mcp"]`, with `ascribe` on the path). Bash is left out, so the agent has no shell: the MCP server exists for "an agent with no shell", and in the agents plan's runs an agent with a shell never called a tool. The run is recorded as the hook's is:

```shell
claude -p "The project has a broken link. Find it and fix it, so the project checks clean." \
  --setting-sources project \
  --strict-mcp-config --mcp-config .mcp.json \
  --allowedTools "Read,Edit,Write,Glob,Grep,mcp__ascribe__*" \
  --disallowedTools Bash \
  --no-session-persistence \
  --output-format stream-json --verbose \
  > <checkout>/project-docs/surface-audit/journeys/transcripts/fix-a-broken-link-mcp.jsonl
```

When it stops, the preparer runs `ascribe check` in the copy and notes the exit code and the diff.

### What changed in these two commands, and why

The script first gave both walks a clean profile, `export CLAUDE_CONFIG_DIR="$(mktemp -d)"`. Tried on 10 October 2026 with a one-word prompt, it answered `Not logged in · Please run /login`: a new profile has no sign-in. So the walks use the normal profile, narrowed:

- **`--setting-sources project`** reads the copy's settings and not the user's, so no user settings, hooks, or plugins reach the session. No memory did either: the session's memory folder is named for the copy's path, and was new.
- **What still reaches it,** from the account and not from settings: the user's synced skills (`deep-research`, `design`, `slides`, `design-sync` were listed beside Claude Code's own), and, without `--strict-mcp-config`, the account's connectors (`claude.ai Claude Docs` connected, three more waiting for a sign-in). Neither session used any of them.
- **The hook walk can't use `--strict-mcp-config`.** With it, the plugin's own MCP server isn't loaded (`mcp_servers: []`), and the plugin isn't what a user installs. Without it, the plugin's server is `plugin:ascribe:ascribe`, and the account's connectors come too.
- **The hook walk allows the plugin's tools** (`mcp__plugin_ascribe_ascribe__*`). The script first left them out, and `claude -p` refuses a tool that isn't allowed, which a person with the plugin would have been asked about and could approve.
- **The MCP walk needs `--mcp-config .mcp.json`.** With `--strict-mcp-config` alone the copy's `.mcp.json` isn't read (`mcp_servers: []`); a project's servers wait for an approval nobody is there to give.
- **`mcp__ascribe__*` is accepted as written.** A trial session called `mcp__ascribe__ascribe_model` with no refusal.
- **`--include-hook-events`** puts each hook's start and answer in the transcript. Without it the first hook walk's transcript couldn't show whether the hook ran.
- **`--no-session-persistence`** keeps the session out of the user's history.

## Notes

One section per walk. Each has the same parts; an agent's has two more:

- **When, with what:** the date and time, the editor or shell, the binary and its version, and which copy.
- **Steps:** a numbered list, written as you go. One line per step: what you did (the command as typed, or what you clicked), then what came back, quoted. Not a table: the maintainer's first walk showed a table is too fiddly to fill while working, so the agents' dry runs are the only ones in that form.
- **Doubts:** for each, what was expected, what happened, how it was resolved.
- **Time:** from opening the project to deciding you're done.
- **Result:** fixed or not; how the walker knew; what the preparer's `ascribe check` said afterwards.
- **Seen on the way:** one line per thing that looks like a finding, to move to `findings.md`.

The first two sections are a dry run of the maintainer's walks, done by an agent on 10 October 2026 to try the script before the maintainer spends the half hour. They don't stand in for the maintainer's walks. The agent prepared the copies itself, so it knew where the link was, and went through the surface anyway.

### Dry run by an agent, in the editor

- **When, with what:** 10 October 2026. VS Code 1.141.0 on macOS (arm64), driven through computer use, which can click and scroll in VS Code and can't type, press a key, right-click, or open the menu bar. Extension `ascribe.ascribe-vscode@0.2.0`. The binary, from the **Ascribe** output channel: `[info] Using bundled binary /Users/kylerollins/.vscode/extensions/ascribe.ascribe-vscode-0.2.0/bin/darwin-arm64/ascribe`. The channel doesn't give a version; `--version` on that path says `ascribe 0.2.0`, and its SHA-256 is the same as `target/release/ascribe`'s (`c43672e1…`), built from this branch, which differs from `main` in `project-docs/` only. The walker's own VS Code settings applied: `editor.formatOnSave` is on and Prettier is the formatter for Markdown.

  The walk was done twice, each in its own copy. The first ended with the link fixed and the page broken by the save (step 9). The second repeated it and saved without formatting.

- **Steps, first copy:**

  | # | Did | Saw |
  |---|---|---|
  | 1 | `code -n <copy>` | A banner: "Restricted Mode is intended for safe code browsing. Trust this folder to enable all features." with **Manage** and **Learn More**. The status bar: **Restricted Mode**, 0 errors, 0 warnings. No Ascribe icon in the activity bar. |
  | 2 | Clicked the status bar's problem count; opened `docs/api-keys.md`, `docs/install-agent.md`, and `docs/quickstart.md` from the Explorer | The Problems panel: "No problems have been detected in the workspace." with each page open. The pages were colored (`@id:` and `{cloud}` stood out), and the language was "Markdown". Nothing from Ascribe said it wasn't checking. |
  | 3 | Clicked **Manage**, then **Trust** | "You trust this folder". The status bar went to 1 error, 0 warnings, 1 info, with "Review: off" and "fix-a-broken-link-editor · site". The Ascribe icon appeared. In the Explorer, `install-agent.md` turned red with a 1. |
  | 4 | Looked at the Problems panel | Under `install-agent.md`: "`` `keys.md` doesn't exist `` ascribe(ASC036) [Ln 93, Col 90]". Under `api-keys.md`: "no other page links to this page or includes it, so readers reach it only through the…  ascribe(ASC154) [Ln 2, Col 8]". |
  | 5 | Clicked the ASC036 row | `install-agent.md` at line 93, with `(keys.md#rotate-keys)` selected ("19 selected"). No lightbulb in the editor's margin. A lightbulb on the row in the Problems panel. |
  | 6 | Clicked that lightbulb | A menu with one entry: "Prompt agent: fix this problem". |
  | 7 | Clicked it | The status bar: "Ascribe: Prompt copied. Paste it into your agent." A notification: "Ascribe: Prompt agent can also fill the prompt in for you, in VS Code's chat or Claude Code, without sending it." with **VS Code's chat** and **Claude Code**. Dismissed it; pasted and sent nothing. |
  | 8 | The title bar's search box, **Show and Run Commands**, scrolled to "Ascribe: Actions for the Cursor" and clicked it; then **Change where the link goes**; then "Rotate keys  api-keys.md#rotate-keys" | The actions bar, "What would you like to do here?", in groups **Structure**, **Link**, **Content model**, starting with "Wrap in a note". Under **Link**: "Link the selected text" and "Change where the link goes  Set the link's destination to a page or heading you choose". The picker, "Link to which page or heading?", listed three pages and eight headings. After the choice, line 93 read `See [](api-keys.md#rotate-keys).`, with "Rotate keys" shown dimmed between the brackets. The ASC036 row went at once. The ASC154 row stayed, and the file was unsaved. |
  | 9 | **Show and Run Commands**, scrolled to "File: Save", clicked it | The page was rewritten as it saved: line 93 became line 106. The Problems panel showed three new things: "the included file `` `\_fragments/prerequisites.md` `` doesn't exist  ascribe(ASC021) [Ln 17, Col 11]", "a blank line separates `` `@steps` `` from the block it applies to; remove it so the dire…  ascribe(ASC016) [Ln 23, Col 1]", and under `prerequisites.md`, "no file includes this fragment, so nothing publishes it  ascribe(ASC155) [Ln 1, Col 1]". ASC154 had gone. |
  | 10 | Stopped | The rule is not to fix anything else. |

- **Steps, second copy:**

  | # | Did | Saw |
  |---|---|---|
  | 1 | `code -n <copy>`, **Manage**, **Trust**, with no page open | The Ascribe icon appeared. The status bar stayed at 0 errors, 0 warnings, and had no "Review: off" and no build. |
  | 2 | Opened `docs/quickstart.md`, a page with no problem | The status bar went to 1 error, 0 warnings, 1 info; `install-agent.md` turned red in the Explorer; the Problems panel had the two rows of the first copy's step 4. |
  | 3 | Clicked the ASC036 row; **Show and Run Commands**, scrolled to "Ascribe: Change where the link goes" and clicked it; then "Rotate keys  api-keys.md#rotate-keys" | The same picker, straight from the palette. Line 93 changed as before; ASC036 went, ASC154 stayed. |
  | 4 | **Show and Run Commands**, scrolled to "File: Save without Formatting", clicked it | The Problems panel: "No problems have been detected in the workspace." The status bar: 0 errors, 0 warnings. |
  | 5 | "Ascribe: Show Server Output" | Two lines: the `[info]` line quoted above, and `[error] ascribe-lsp: using the project at /var/folders/…/fix-a-broken-link-editor-2/ascribe.toml`. |

- **Where it needed typing:** nowhere. The fix was four clicks after the cursor was on the link: the command, the action, the heading, the save. What clicks couldn't reach was the editor's own lightbulb and context menu (a key or a right-click) and the palette's filter, so each command was found by scrolling the whole list. A person types three letters.

- **Doubts:**
  - Expected Ascribe's problems as soon as the folder opened. Got "No problems have been detected in the workspace." in Restricted Mode with three pages open, and again after trusting it with none open. Resolved by trusting the folder, then by opening a page; any page did.
  - Expected the lightbulb to offer the page the link meant. It offered "Prompt agent: fix this problem" and nothing else. Resolved by looking through the palette for a command about links.
  - In the picker, a page ("API keys  api-keys.md") and a heading ("Rotate keys  api-keys.md#rotate-keys") both looked right. Chose the heading because the broken link had `#rotate-keys`. Nothing in the picker pointed at either.
  - After the change, ASC154 was still listed and the file was unsaved. Wasn't sure whether the advice was a second thing to fix or would go with the save. It went with the save.
  - After the first copy's save, three problems that weren't there before. Nothing said the save had rewritten the page; the line number changing was the sign.
  - "No problems" is the Problems panel's word for the build the status bar names, `site`. The Content Model view lists three builds. Didn't find a command that checks the others.

- **Time:** first copy, about 2 minutes 15 seconds from opening the folder to the save. Second copy, about 1 minute 20 seconds to "No problems". Much of both was scrolling the palette. An agent's times, and no measure of a person's.

- **Result:** first copy: the link fixed and the page not clean. The preparer's `ascribe check` exited 1 with `checked 4 files: 1 error, 1 warning, 1 advice` (ASC021, ASC016, ASC155); `git diff --stat` said `1 file changed, 16 insertions(+), 3 deletions(-)`, one line of it the link. Second copy: fixed. The walker knew from the Problems panel's "No problems have been detected in the workspace." The preparer's `ascribe check` exited 0 with `checked 4 files: 0 errors, 0 warnings`, and `git diff` was the one line:

  ```diff
  -A common cause is an expired API key. Generate a new key, then restart the agent. See [](keys.md#rotate-keys).
  +A common cause is an expired API key. Generate a new key, then restart the agent. See [](api-keys.md#rotate-keys).
  ```

- **Seen on the way:**
  - In Restricted Mode the extension is silent and the panel says the project is clean: "No problems have been detected in the workspace." with the broken page open, and the only notice is VS Code's own banner.
  - Trusted, with no page open, the status bar says 0 errors; opening `quickstart.md` brought `install-agent.md`'s error. The project's problems wait for a page to be opened, and nothing says so.
  - Saving with VS Code's format on save and Prettier as the Markdown formatter broke a page that checked clean apart from the link, and half of what it changed was reported by nothing. The detail is under "What the save changed", below.
  - The lightbulb's only entry for ASC036 was "Prompt agent: fix this problem", while **Change where the link goes**, two commands away, listed "Rotate keys  api-keys.md#rotate-keys". More evidence for "`choose` with nothing to choose from" in the inventory's list, not a new one.
  - The actions bar, opened on the broken link, started with "Wrap in a note" and had no "Prompt agent: fix this problem". The inventory's row for it says "the fixes for problems at the cursor first, then the actions". Checked afterwards by the preparer, outside the walk: the bar lists only the fixes that edit the text, and that entry only runs a command, so it's as designed. What's loose is the row's wording.
  - The error went as the link changed; the advice ASC154, which the same change answers, stayed until the file was saved.
  - Command titles are in two styles in one list: "Ascribe: Actions for the Cursor", "Ascribe: Changed Pages", "Ascribe: Dim What the Build Leaves Out", "Ascribe: Prompt Agent to Fix This File"; and "Ascribe: Change where the link goes", "Ascribe: Add a variant", "Ascribe: Give the heading a stable id". "Ascribe: Focus on Content model View" names the view the side bar calls "Content Model".
  - The output channel logs `ascribe-lsp: using the project at …/ascribe.toml` at `[error]`, when nothing is wrong, and doesn't give the binary's version.
  - "Prompt agent: fix this problem" replaced what was on the clipboard, and said so only in the status bar.
  - Nothing in the editor checks every build: the inventory's item, seen again. Here it didn't matter, since `ascribe check` agreed.

- **What the save changed:** the first copy's `git diff` of `docs/install-agent.md` after **File: Save**: 16 lines added and 3 removed, of which one changed line is the link. The rest is Prettier's, formatting the page as Markdown:

  | Prettier's change | Where | What reported it |
  |---|---|---|
  | `@include: _fragments/prerequisites.md` became `@include: \_fragments/prerequisites.md` | line 17 | `` [ASC021] Error: the included file `\_fragments/prerequisites.md` doesn't exist ``; and, since the fragment is no longer included, `[ASC155] Advice: no file includes this fragment, so nothing publishes it` at `docs/_fragments/prerequisites.md:1:1` |
  | A blank line after `@steps` | line 23 | `` [ASC016] Warning: a blank line separates `@steps` from the block it applies to; remove it so the directive touches its block `` |
  | A blank line between a heading and its `@id:` line | five headings: Prerequisites, Install the agent, Connect to {product}, Streaming sync, Troubleshooting | Nothing |
  | A blank line after `@variant {pm=npm}:`, `@variant {pm=pnpm}:`, and `@variant {pm=yarn}:`, and between each one's code block and the next `@variant` or `@end` | in the first step, and before the `@end` that closes the deployment variants | Nothing |
  | `version: {version}` became `version: { version }` | inside a `` ```yaml phrases=true `` block | Nothing |

  It isn't particular to this task or this copy. The pages are `.md` files and VS Code's language for them is "Markdown", so whatever formats Markdown formats them: the walker's settings were `"editor.formatOnSave": true` and `"editor.defaultFormatter": "esbenp.prettier-vscode"` under `"[markdown]"`, a common pair. Any page with an underscore in a directive's path, an `@steps`, an `@id:` under a heading, an `@variant`, or a phrase in a code block would change the same way. This repository's own Prettier run leaves Markdown out, so it doesn't happen here; a user's project has nothing that does that for it.

  Checked afterwards by the preparer, outside the walk, by rendering the page for the `site` build before and after Prettier (`ascribe render docs/install-agent.md --build site`), with the repository's Prettier at its defaults: the blank lines after `@id:` and around `@variant` change nothing in the output; `{ version }` does, and the built page says `version: { version }` where it said `3.4.1`, with nothing reported. With that Prettier the `_fragments` path wasn't escaped, so that change depends on the user's Prettier settings. What the extension does about another formatter wasn't looked at: the finding is written from the walk, and what to do about it is phase 5's.

### Dry run by an agent, at the command line

- **When, with what:** 10 October 2026. The agent's own shell (zsh, macOS), with `target/release` first on the path; `ascribe --version` says `ascribe 0.2.0`. The edit was made with `sed`, standing in for a plain editor.

- **Steps:**

  | # | Did | Saw |
  |---|---|---|
  | 1 | `cd <copy>`, then `ascribe --help` | Exit 0. Among the examples: `ascribe check`  "Check the project in this folder or above"; `ascribe explain ASC036`  "What a diagnostic means, and how to fix it"; `ascribe link keys.md --from guides/install.md`  "Whether a link works, and what to write". Last line: `Documentation: https://ascribed-dev.com/go/cli-reference/`. |
  | 2 | `ascribe check` | Exit 1. `` [ASC036] Error: `keys.md` doesn't exist `` at `docs/install-agent.md:93:90`, with `link-target-missing` under `[](keys.md#rotate-keys)`; `[ASC154] Advice: no other page links to this page or includes it, so readers reach it only through the site's own navigation or search, if those list it` at `docs/api-keys.md:2:8`; `checked 4 files: 1 error, 0 warnings, 1 advice`. |
  | 3 | `ascribe explain ASC036` | Exit 0. `Fix: Fix the path, or create the file. Names must match exactly, including case, and the file must be inside the project.` Its example: wrong, `[API keys](api-key.md)`; right, `[API keys](keys.md)`. `Docs: https://ascribed-dev.com/go/ASC036/`. |
  | 4 | `ascribe link 'keys.md#rotate-keys' --from docs/install-agent.md` | Exit 1. `` keys.md#rotate-keys doesn't work from install-agent.md: `keys.md` doesn't exist. `` Nothing else. |
  | 5 | `ls docs` | `_fragments`, `api-keys.md`, `install-agent.md`, `playground.png`, `quickstart.md`. |
  | 6 | `ascribe link 'api-keys.md#rotate-keys' --from docs/install-agent.md` | Exit 0. `api-keys.md#rotate-keys exists: a heading on api-keys.md.` then `Write: [Rotate keys](api-keys.md#rotate-keys)`. |
  | 7 | `ascribe refs keys.md`; `ascribe refs api-keys.md` | `keys.md doesn't exist.`, exit 1; `0 places use api-keys.md.`, exit 0. |
  | 8 | `sed -i '' '93s/](keys\.md#rotate-keys)/](api-keys.md#rotate-keys)/' docs/install-agent.md` | Exit 0, no output. |
  | 9 | `ascribe check` | Exit 0. `checked 4 files: 0 errors, 0 warnings`. |
  | 10 | Opened the two addresses the commands printed, in a browser | `https://ascribed-dev.com/go/ASC036/` and `https://ascribed-dev.com/go/cli-reference/` both gave "Page not found": "There's no page at this address. It may have moved: try the navigation, search, or the start page." Done after step 9, so it cost the walk nothing. |

- **Doubts:**
  - Expected `check` or `link` to name the page the link meant. Neither did. Resolved with `ls docs`, and a guess that `api-keys.md` was `keys.md` renamed, confirmed by `ascribe link` on the guess.
  - `link` said `Write: [Rotate keys](api-keys.md#rotate-keys)`, and the page had `[](keys.md#rotate-keys)`, with no text. Wasn't sure whether to put the text in. Kept the empty brackets, changing only what was broken.
  - Whether the advice was a second thing to fix. `check` exits 1 for the error alone, so left it; it went with the fix.

- **Time:** 17 seconds from `cd` to the clean check. An agent's time; a person reads the help.

- **Result:** fixed. Ran `ascribe --help`, `ascribe check`, `ascribe explain ASC036`, `ascribe link` twice, `ls`, `ascribe refs` twice, `sed`, `ascribe check`. The last exited 0, and `git diff` was the same one line as the editor's second copy. `git status --short` said ` M docs/install-agent.md`.

- **Seen on the way:**
  - Both addresses Ascribe printed are "Page not found" on the published site: `Docs: https://ascribed-dev.com/go/ASC036/` from `explain`, the same address behind the code in the Problems panel, and `Documentation: https://ascribed-dev.com/go/cli-reference/` from `--help`.
  - One page, two names in two commands: `check` says `docs/install-agent.md:93:90`, and `link`, given `--from docs/install-agent.md`, answers "doesn't work from install-agent.md".
  - `link` tells the walker to write `[Rotate keys](api-keys.md#rotate-keys)` where the page uses `[]`, which shows the title by itself. Following it changes more than the fix needs.
  - `check`'s last line counts advice only when there is some: `checked 4 files: 1 error, 0 warnings, 1 advice`, then `checked 4 files: 0 errors, 0 warnings`.
  - `link` listed no `Closest:` for `keys.md` with `api-keys.md` beside it, and `check`'s text had no `Help:` line: the inventory's items, seen again.
  - `explain ASC036`'s example calls `keys.md` right and `api-key.md` wrong, close to this copy's names with the sense turned round.

### What the dry run changed in the script

The setup worked: both copies gave the one error and the one advice "Preparing a copy" shows, at exit 1. Neither walk ran long. Changed in the script already:

- **In the editor** now says the folder opens in Restricted Mode and has to be trusted. Without that step the walk ends at "No problems have been detected in the workspace."
- **The notes' "When, with what"** asked for `ascribe --version` from the output channel, which doesn't print it. It now asks for the binary's path from there, and the version from the preparer.
- **Preparing a copy** now says the extension runs the binary it bundles, and to compare that with the release build; and what a clean `ascribe check` prints.
- **The notes' steps are a numbered list, not a table.** After the maintainer's editor walk: "there's no way I'm going to work in a markdown table as I go. It's too fiddly." Phase 3's scripts take the list.

Decided by the maintainer on 10 October 2026, and written into the script:

1. **Format on save stays as the maintainer has it.** The walk is the maintainer's real editor. If the save does more than save, the walker notes it in a line and stops, as the rules say; the dry run has the detail. The walk ends on the preparer's `ascribe check`, not on a clean panel.
2. **The binary** built on 10 October 2026 is from the audit's branch, which differs from `main` in `project-docs/` only. The script says so.
3. **The help's examples give the command-line walk its commands.** `ascribe --help` shows `ascribe explain ASC036` and `ascribe link keys.md --from guides/install.md`, this task's code and file name, because the walk's project is Ascribe's own example. The rename stands, and the notes say when the help handed the commands over.
4. **The printed addresses were 404** because the published site is built from the nightly canary, and the canary of 10 October 2026 predates the permalinks. "Preparing a copy" says to check one address before a walk.
5. **The editor walk's end** now asks which builds "no problems" covers.

### In the editor

- **When, with what:** 10 October 2026, 7:26 to 7:34 PM. The maintainer's VS Code, with the extension built from this branch and installed from a `.vsix` the same day; its bundled binary is byte for byte the release build (`ascribe 0.2.0`, SHA-256 `c43672e1…`). The copy `~/ascribe-walks/fix-a-broken-link-editor`, prepared by the agent.

- **Steps, as the walker noted them:**

   1. start at 7:26 PM
   2. see a `docs/` directory, `ascribe.toml`, and `README.md`. VS Code indicates error in `docs/` directory with color and `1` next to it.
   3. open `docs/install-agent.md` to look for error. Doesn't show directly in the file at first glance.
   4. open VS Code problems panel to look for details. See ``keys.md` doesn't exist` and click on it. Realize it was shown in the editor, I just didn't notice it because the link's underline sits above the squiggles.
   5. use quick fix: prompt agent with a Sonnet 5.5 agent in Claude Code VS Code extension. It's reponse:
   ```
   I fixed the broken link, but I couldn't run `ascribe check`, so the fix is unverified.

   On line 93 of [docs/install-agent.md](docs/install-agent.md#L93), the link pointed at `keys.md`, which doesn't exist. The file is `docs/api-keys.md`, and it has the `rotate-keys` anchor (`## Rotate keys`, `@id: rotate-keys`). The link now reads `[](api-keys.md#rotate-keys)`.

   The `ascribe` command isn't installed here. It's not on the PATH, and there's no local copy in the project. Please run `ascribe check docs/install-agent.md` on your machine to confirm the ASC036 error is gone.
   ```
   6. project seems clean. Nothing in the problems panel.
   7. end at 7:34 PM

- **Doubts:** the walker's notes have one, in step 3 and 4: the error wasn't seen in the page at first, because the link's underline sits over the squiggle, and the Problems panel found it.

- **Time:** 8 minutes.

- **Result:** fixed. The walker knew from "Nothing in the problems panel", which is the `site` build's; the agent that made the edit said the fix was unverified. The preparer's `ascribe check` afterwards exited 0, `checked 4 files: 0 errors, 0 warnings`, and `git diff` was the one line, `keys.md` to `api-keys.md` on line 93. No agent files were left in the copy.

- **Seen on the way:**
  - The error was reported where the link is, and the walker didn't see it: "the link's underline sits above the squiggles". The Explorer's red `1` on `docs/` and the Problems panel are what led to it.
  - The first thing the walker did on the problem was the lightbulb's **Prompt agent**, and sent the prompt to Claude Code's VS Code extension, though the script said nothing is sent and the walk had "any agent" under *Without*. The script's constraint didn't match how the maintainer works in the editor. The fix was the agent's, so the walk didn't try **Change where the link goes**, the click path the dry run found.
  - The agent in the same window had no `ascribe`: "It's not on the PATH, and there's no local copy in the project." The extension bundles a binary the editor runs, and an agent beside it can't reach it or any check, so it fixed the link blind and asked the person to verify. The extension's MCP server is offered to VS Code's own agents, not to Claude Code's extension.
  - "Project seems clean. Nothing in the problems panel." The panel covers the editor's build, `site`; nothing in the editor checked the other two. Here `ascribe check` agreed. The inventory's item, seen a third time.

### At the command line

- **When, with what:** 10 October 2026, 7:41 to 7:44 PM. Terminal.app, with the release build first on the path, `ascribe 0.2.0`. The copy `~/ascribe-walks/fix-a-broken-link-cli`, prepared by the agent.

- **Steps, as the walker noted them:**

   1. time: 7:41 PM
   2. initial output:
   ```
   cd ~/ascribe-walks/fix-a-broken-link-cli
   ascribe --version

   ascribe 0.2.0
   ```
   3. use `ascribe --help`
   4. use `ascribe check`
   5. see the errors found, realize I hate text editing in the terminal and start wondering if this isn't something an agent is better suited to
   6. 7:44PM end because family needs me

- **Doubts:** one, in step 5, and it ended the walk: whether a person at a terminal is who does this at all. "realize I hate text editing in the terminal and start wondering if this isn't something an agent is better suited to".

- **Time:** 3 minutes, to where it stopped.

- **Result:** not finished. The walker found the problem with the second command, `ascribe check`, and stopped before editing. The preparer's `ascribe check` afterwards exited 1 with the copy as prepared, `1 error, 0 warnings, 1 advice`, and no file changed. Not walked again: the agent's dry run above is the command-line path finished, and phase 1's two agent walks are the agent's version of it.

- **Seen on the way:**
  - Two commands found the problem: `ascribe --help`, then `ascribe check`. The help was enough to pick the command.
  - The maintainer, given a terminal and a plain editor, didn't want to edit the page there and reached for an agent, as in the editor walk. Both of the maintainer's walks ended with the same move. For phase 3, that's the answer to whether a person at the command line is a journey worth walking by hand.

### Through the hook

Run twice, each in its own copy. The first recording was made without `--include-hook-events`, so it couldn't show what the hook said; the second is the same command with that flag, in a fresh copy. It wasn't run again to see if it would go better, and it went the same way.

- **When, with what:** 10 October 2026, 9:17 PM and 9:19 PM. Claude Code 2.1.296, model `claude-opus-5-5`, started from zsh on macOS with `target/release` first on the path (`ascribe 0.2.0`). The plugin from this checkout, `ascribe@inline` version 0.2.0. Copies `fix-a-broken-link-hook` and `fix-a-broken-link-hook-2`, in a scratch folder, prepared by the agent that ran them. `https://ascribed-dev.com/go/ASC036/` answered 404 just before; neither session opened an address.

- **Transcript:** [`transcripts/fix-a-broken-link-hook.jsonl`](transcripts/fix-a-broken-link-hook.jsonl) and [`transcripts/fix-a-broken-link-hook-2.jsonl`](transcripts/fix-a-broken-link-hook-2.jsonl).

- **What the session had:** the tools listed `mcp__plugin_ascribe_ascribe__ascribe_check` and the plugin's eight others, and `LSP`; the skills listed `ascribe:ascribe`; the commands listed `ascribe:check`, `ascribe:new-page`, `ascribe:review`.

- **Steps, first run:**

  1. Bash: `ascribe check 2>&1 | head -50; ls -la; git ls-files | head -100`. Came back: `` [ASC036] Error: `keys.md` doesn't exist `` at `docs/install-agent.md:93:90`, the `ASC154` advice at `docs/api-keys.md:2:8`, `checked 4 files: 1 error, 0 warnings, 1 advice`, and the file list.
  2. Bash: `cat docs/api-keys.md; grep -rn "keys" docs/*.md README.md ascribe.toml | grep -v "^docs/api-keys.md"`. Came back: the page, with `## Rotate keys` and `@id: rotate-keys`; the broken link; and `README.md:10:  keys.md                          has the `rotate-keys` id the page links to`.
  3. Bash: `cat README.md; sed -i '' 's/\[\](keys.md#rotate-keys)/[](api-keys.md#rotate-keys)/' docs/install-agent.md && ascribe check; git diff --stat`. Came back: the README, which says `docs/install-agent.md` "is byte for byte the page in the specification"; then `checked 4 files: 0 errors, 0 warnings`.
  4. Bash: `git checkout docs/install-agent.md && git mv docs/api-keys.md docs/keys.md && ascribe check; git status --short`. Came back: `Updated 1 path from the index`, `checked 4 files: 0 errors, 0 warnings`, `R  docs/api-keys.md -> docs/keys.md`.
  5. Answered: "I fixed it by renaming `docs/api-keys.md` to `docs/keys.md` rather than editing the link, because the README lists the file as `keys.md` and says `install-agent.md` is byte for byte the page in the spec."

- **Steps, second run:**

  1. Bash: `ascribe check 2>&1 | head -50; git ls-files | head -100`. The same two reports.
  2. Bash: `grep -n -i "rotat\|^#\|{#" docs/api-keys.md; grep -rn "keys" docs/*.md README.md | grep -v "^docs/api-keys.md"`. The heading, its `@id`, the link, and the README's line.
  3. Bash: `cat README.md ascribe.toml; git log --stat | head -30`. Then said: "The README says `docs/install-agent.md` must stay byte-for-byte the spec page and lists the target as `keys.md`, so the misnamed file is the thing to fix, not the link."
  4. Bash: `git mv docs/api-keys.md docs/keys.md && ascribe check 2>&1 | tail -5; for b in site cloud self-managed-3.3; do ascribe check --build $b 2>&1 | tail -1; done; git status --short`. Came back: `checked 4 files: 0 errors, 0 warnings`, four times.
  5. The Stop hook ran: `"hook_name": "Stop"`, `"stdout": ""`, `"stderr": ""`, `"exit_code": 0`, `"outcome": "success"`.

- **Tried first:** `ascribe check`, in the first command of both runs, with no `--help` before it. Nothing was guessed wrong. Neither run called `ascribe explain`, `ascribe link`, or `ascribe refs`, loaded the skill, called one of the plugin's tools, or used `LSP`. Both found the page the link meant with `grep` and `cat`.

- **What the hook said:** nothing, in both. The plugin's `hooks.json` has one hook, at the stop, and there the project was already clean. Nothing was said after an edit either: the first run edited with `sed` in Bash and the second edited no page, so no edit went through a tool the language server's diagnostics follow.

- **Doubts:**
  - The first run fixed the link, saw the check pass, then read the README and undid it for the rename. The second read the README first. The agent's doubt was which of the two was wrong, the link or the file's name, and the copy's own README answered it.
  - The second run checked each build by name after `ascribe check` had passed. `checked 4 files: 0 errors, 0 warnings` doesn't say which builds that covers.

- **Time:** 16 seconds and 18 seconds by the clock; the sessions report 14.3 and 13.7 seconds, five turns each.

- **Result:** fixed, both times, by renaming the file back, which "Preparing a copy" allows for. The agent knew from `ascribe check`. The preparer's `ascribe check` exited 0 with `checked 4 files: 0 errors, 0 warnings`; `git status --short` said `R  docs/api-keys.md -> docs/keys.md`, staged, and `git diff` was empty.

- **Seen on the way:**
  - With a shell, the agent used nothing the plugin gave it. Nine tools, a skill, three commands, and the language server were there, and both runs were `ascribe check`, `grep`, `cat`, and `git mv`. The agents plan's runs found the same.
  - The hook had nothing to say, because the agent had already checked. Its worth is for the agent that doesn't, and this task doesn't show that.
  - Checked afterwards by the preparer, outside the walk: `ascribe agents hook claude-code --event stop`, run in a copy with the broken link still committed and nothing changed, printed nothing and exited 0. The agents guide says it "checks every build of each project whose pages or `ascribe.toml` the working tree changes". An agent that did nothing in this copy would have been let go with the link broken.
  - `ascribe check`'s last line doesn't name the builds it covered, and the agent checked `site`, `cloud`, and `self-managed-3.3` one at a time to be sure. The JSON has `builds_checked`; the text has no word for it.
  - The copy's README told the agent which way to fix it: "`keys.md`  has the `rotate-keys` id the page links to". That's the script's, not Ascribe's. See "Preparing a copy".
  - Nothing offered `api-keys.md` here either. `grep` found it.

### Through MCP

- **When, with what:** 10 October 2026, 9:18 PM. Claude Code 2.1.296, model `claude-opus-5-5`, with one MCP server, `ascribe`, from the copy's `.mcp.json` (`ascribe 0.2.0` on the path), no plugin, and no Bash. Copy `fix-a-broken-link-mcp`, prepared by the agent that ran it.

- **Transcript:** [`transcripts/fix-a-broken-link-mcp.jsonl`](transcripts/fix-a-broken-link-mcp.jsonl).

- **Steps:**

  1. `ToolSearch`, `select:mcp__ascribe__ascribe_check,mcp__ascribe__ascribe_link,mcp__ascribe__ascribe_refs`. Claude Code's own step: it loads a tool's definition before the first call.
  2. `ascribe_check` with `{"response_format": "detailed"}`. Came back: `"builds_checked":["site","cloud","self-managed-3.3"]`, and two diagnostics. For `ASC036`: `"file":"docs/install-agent.md"`, `"fixes":[]`, `"help":"Fix the path, or create the file. Names must match exactly, including case, and the file must be inside the project."`, `` "message":"`keys.md` doesn't exist" ``, `"next":"choose"`. For `ASC154`: `"file":"docs/api-keys.md"`, `"next":"review"`.
  3. `Read`, `docs/install-agent.md`, lines 88 to 97. The note, with `See [](keys.md#rotate-keys).` on line 93.
  4. `ascribe_link` with `{"target": "keys.md#rotate-a-key", "from": "docs/install-agent.md"}`. Came back: `"closest":[]`, `"exists":false`, `"from":"install-agent.md"`, `"kind":"missing"`, `` "problem":"`keys.md` doesn't exist" ``.
  5. `Grep`, `^#|^title` in `docs/api-keys.md`. Came back: `title: API keys`, `## Create a key`, `## Rotate keys`.
  6. `Edit`, `[](keys.md#rotate-keys)` to `[](api-keys.md#rotate-keys)`.
  7. `ascribe_check` with `{}`. Came back: `{"concise":"checked 4 files: 0 errors, 0 warnings\n"}`.
  8. Answered: "The broken link is fixed and the project now checks clean: 4 files, 0 errors, 0 warnings." and "That fix also cleared the one advice-level hint, that no page linked to `api-keys.md`."

- **Tried first:** `ascribe_check`, asking for the detailed form at once. Then `ascribe_link`, with an id that wasn't the link's (`rotate-a-key` for `rotate-keys`), which answered with nothing to go on. It never called `ascribe_explain` or `ascribe_refs`, though it had loaded `ascribe_refs`.

- **The tools it called:** `ascribe_check` twice and `ascribe_link` once, of nine. `Read`, `Grep`, and `Edit` did the rest.

- **Doubts:**
  - Which page the link meant. `ascribe_link` gave `"closest":[]`. The agent went to `docs/api-keys.md` next, and the only place that name had appeared was the second diagnostic, the advice about an orphan page. It didn't read the README.
  - It wrote `api-keys.md#rotate-keys` from the heading's text, `## Rotate keys`, and didn't ask `ascribe_link` whether that target exists before or after the edit. The second `ascribe_check` was its proof.

- **Time:** 15 seconds by the clock; the session reports 13.2 seconds, eight turns.

- **Result:** fixed, by changing the link. The agent knew from `ascribe_check`. The preparer's `ascribe check` exited 0 with `checked 4 files: 0 errors, 0 warnings`; `git status --short` said ` M docs/install-agent.md`, and `git diff` was the one line the editor's and the command line's walks made.

- **Seen on the way:**
  - `ascribe_link` on the missing page gave `"closest":[]`, with `api-keys.md` beside it. The inventory's "`choose` with nothing to choose from", seen a fourth time.
  - What led the agent to the right page was a different diagnostic: `ASC154`'s `"file":"docs/api-keys.md"`. The orphan advice did the work `closest` is for.
  - One page, two names in one session: `ascribe_check` says `"file":"docs/install-agent.md"`, and `ascribe_link`, given `"from": "docs/install-agent.md"`, answers `"from":"install-agent.md"`. The command-line dry run saw the same in the text.
  - The second `ascribe_check` answered `{"concise":"checked 4 files: 0 errors, 0 warnings\n"}`: a line of text for a terminal, inside JSON, where the first call's detailed form had `builds_checked` and counts as fields. The agent asked for `detailed` first without being told to.
  - Claude Code wrote 94 lines to standard error while loading the tools, of the form `unknown format "uint32" ignored in schema at path "#/$defs/Pos/properties/line"` and `unknown format "uint" ignored in schema at path "#/$defs/CodeCount/properties/count"`. The tools' output schemas name number formats the client's validator doesn't know. Nothing failed, and a person running the client would see it.
  - Without a shell, the agent used the tools at once, and for the same two steps a shell gives: check, then check again.

## What the walks had in common

**The maintainer's two,** 10 October 2026. Together they took 11 minutes, well under the half hour, and both ended with the same move: on seeing the problem, the maintainer reached for an agent. In the editor, that was the lightbulb's **Prompt agent**, sent to Claude Code's extension though the script said nothing is sent; at the terminal, it was stopping rather than editing by hand. The maintainer's verdict: the editor walk was a reasonable ask; a person at a terminal with a plain editor isn't how they work, so phase 3's maintainer walks happen in the editor with an agent allowed, and the command line is the agents' surface. "That's just how people work these days." Written into this script's rules and into [phase 3](../phase-3-journeys.md).

Where the paths met: every one of the four found the problem with its first look at the project's problems (the Explorer's red `1` and the Problems panel; `ascribe check`), and every finished one made the same one-line change. Where they didn't: nothing on any surface offered `api-keys.md` as the page the link meant, so each walker worked that out from the folder listing, and the editor's one click path to it (**Change where the link goes**) was found by the dry run and not by the maintainer, who went to the agent first.

**The agents' two,** 10 October 2026. Three sessions, each about 15 seconds. Every one ran the check first and got it right at the first try: `ascribe check` in a shell with no `--help` before it, and `ascribe_check` where there was no shell. None was stuck.

What they tried first, and whether the plugin and the MCP server changed what the agent did:

- **With a shell, nothing of the plugin's was used.** Its tools, its skill, its commands, and the language server were all offered, and the agent ran `ascribe check`, `grep`, `cat`, and `git mv`. The stop hook ran and said nothing, since the project was already clean.
- **Without a shell, the tools were used at once,** for the two steps a shell gives: check, then check again. One call to `ascribe_link` between them returned nothing to act on.
- **No surface named the page the link meant,** on any of the seven walks and runs. A person used the folder listing, the agents used `grep`, the README, and the orphan advice's file name. The editor's **Change where the link goes** lists it, and only the dry run found that.
- **The two agents fixed it two ways.** With the copy's README in reach, the one with a shell renamed the file back, twice. The one without changed the link. Both check clean. Only the second is what the task meant, and nothing in Ascribe could have said so: the README was the script's.

The command line turned out to be the surface the walks agree on: every walker's first move was the check, and every one who finished ended on it.

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

The fix that clears both is `[](api-keys.md#rotate-keys)`. A walker who renames the file back also clears both; note which they chose. `examples/quill`'s `README.md` comes with the copy, and it lists the file as `keys.md` and says `install-agent.md` is byte for byte the specification's page; both of the hook's runs read it and renamed the file back. For a later journey, leave an example's README out of the copy: it describes the example to someone reading the repository, and it isn't part of the project. The walk's project is Ascribe's own example, so `ascribe --help`'s examples name this task's code and file (`ascribe explain ASC036`, `ascribe link keys.md --from guides/install.md`); the rename stands, and the notes say when the help handed the commands over. Fixed, `ascribe check` exits 0 and prints one line, `checked 4 files: 0 errors, 0 warnings`.

The addresses the binary prints, `https://ascribed-dev.com/go/…`, were "Page not found" for every walk here: the site builds from the nightly canary, and the canary of 10 October 2026 predated permalinks. They've answered since the canary `0.2.1-next.13` was published and the site rebuilt, checked on 10 October 2026 after the walks. Before a later session, open one (`https://ascribed-dev.com/go/ASC036/`) all the same: a binary built from `main` can be ahead of the site by a day.

For the editor walk, the extension runs the binary it bundles (`~/.vscode/extensions/ascribe.ascribe-vscode-<version>/bin/<platform>/ascribe`) unless the project installs its own or `ascribe.path` is set, not the one on the path. Before the walk, compare it with the release build (`shasum -a 256` on both).
