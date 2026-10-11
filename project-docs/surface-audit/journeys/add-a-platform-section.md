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

### With the plugin

- **When, with what:** 10 October 2026, 9:56 PM. Claude Code 2.1.296, model `claude-opus-5-5`, started from zsh on macOS with `target/release` first on the path (`ascribe 0.2.0`). The plugin from the checkout, `ascribe@inline` version 0.2.0. Copy `add-a-platform-section-plugin`, in a scratch folder, prepared by the agent that ran it. The session's first event listed the plugin's nine tools (`mcp__plugin_ascribe_ascribe__ascribe_check` and the eight others), `LSP`, the skill `ascribe:ascribe`, and the commands `ascribe:check`, `ascribe:new-page`, and `ascribe:review`; the server `plugin:ascribe:ascribe` was `connected`. The user's synced skills and the account's connectors (`claude.ai Claude Docs` connected, three more `needs-auth`) were listed too, and none was used. Standard error was empty.

- **Transcript:** [`transcripts/add-a-platform-section-plugin.jsonl`](transcripts/add-a-platform-section-plugin.jsonl).

- **Steps:**

  1. `Skill`, `ascribe:ascribe`. Came back: `Launching skill: ascribe:ascribe`, then the skill's text.
  2. `Glob`, `**/*install*`. Came back: `docs/install-agent.md`.
  3. `Glob`, `**/ascribe.toml`. Came back: `ascribe.toml`.
  4. `Read`, `docs/install-agent.md`. The page, with its two groups of `@variant` lines, by `pm` and by `deployment`.
  5. `Read`, `ascribe.toml`. The content model, with `[dimensions.pm]` and `[dimensions.deployment]` and the comments above them.
  6. `Read`, the plugin's `skills/ascribe/references/directives.md`, which the skill links to. Came back, among nine directives: "`@variant {<dimension>=<value>}:` before each arm, and one `@end` after the last: content that differs by a dimension", and "Which note types, dimensions, and sources a project has: `ascribe model`."
  7. `Glob`, `**/*.md`. Four files.
  8. Bash: `cat docs/quickstart.md docs/keys.md docs/_fragments/prerequisites.md; ls -a; which ascribe; ascribe check --format concise; ascribe model --section dimensions`. Came back: the pages; `checked 4 files: 0 errors, 0 warnings`; and "`pm` (Package manager): `npm`, `pnpm`, `yarn`" and "`deployment` (Deployment): `cloud` (versionless), `self-managed`".
  9. Bash, one command that made both edits with a Python script and checked them: it put `[dimensions.os]` into `ascribe.toml` before `[phrases]`, put a section, "Start the agent at startup", into the page before "Streaming sync", and reworded the page's opening sentence to mention it; then `ascribe check --format concise; ascribe render docs/install-agent.md --build site | sed -n '/Start the agent at startup/,/Streaming sync/p' | head -150`. Came back: `checked 4 files: 0 errors, 0 warnings`, then the section as plain Markdown, with `**macOS**`, `**Linux**`, and `**Windows**` each on a line of its own above that system's steps.
  10. Answered: "I added a "Start the agent at startup" section to `docs/install-agent.md`, with the macOS, Linux and Windows steps as three arms of an operating-system switcher, so a reader picks their system and sees only its steps. `ascribe check` passes on the whole project; nothing is committed."
  11. The Stop hook ran.

- **Tried first:** the skill, then the page and the content model, then the skill's directive reference. It saw from `ascribe model --section dimensions` that the project had no dimension for a system, and wrote one in `ascribe.toml` on the pattern of the two above it:

  ```toml
  # `@variant {os=macos}:`: steps that differ by the reader's operating system.
  [dimensions.os]
  label = "Operating system"
  values = ["macos", "linux", "windows"]
  labels = { macos = "macOS", linux = "Linux", windows = "Windows" }
  ```

  No command was guessed wrong, and no `--help` was read. It called none of the plugin's tools and didn't use `LSP`. It rendered one build, `site`.

- **What the hook or the plugin's tools said:** the hook said nothing: `"hook_name": "Stop"`, `"stdout": ""`, `"stderr": ""`, `"exit_code": 0`, `"outcome": "success"`. No tool of the plugin's was called. Nothing was said after an edit: both files were written by `python3` in Bash, so neither edit went through a tool the language server's diagnostics follow.

- **Doubts:**
  - What the agent is told to check is the product, not the page: "**Check before publishing:** the docs never say how the agent is launched, so I assumed `quill start`, run in the foreground from the repository root." That's about Quill, which is made up, and not about Ascribe.
  - It said how the section behaves in builds it hadn't rendered: "The site and self-managed builds show it as a switcher; the cloud build keeps all three arms too." Its one render was `--build site`. The statement is right, by the preparer's checks below; the agent had it from reading `ascribe.toml`'s `[builds.cloud]`, whose `variants = { deployment = "cloud" }` names only the other dimension.
  - It said "a reader picks their system and sees only its steps" after a render that shows all three systems one after another, under bold labels. Nothing in the render says the labels are tabs. The agent had that from `ascribe.toml`'s comment, "The published site: every arm as a switcher".

- **Time:** 60 seconds by the clock; the session reports 58.7 seconds, eleven turns. Most of it was writing the section: more than a hundred lines of launchd, systemd, and PowerShell.

- **Result:** added, and right. The agent knew from `ascribe check` and one render. The preparer ran, in the copy:
  - `ascribe check`: exit 0, `checked 4 files: 0 errors, 0 warnings`.
  - `git status --short`: ` M ascribe.toml`, ` M docs/install-agent.md`. `git diff --stat`: `2 files changed, 119 insertions(+), 1 deletion(-)`. The one deleted line is the page's opening sentence, which now ends "connecting it to {cloud} or a self-managed server, and starting it when your computer starts." The rest of the page is as it was.
  - `ascribe model --section dimensions`: a third line, "`os` (Operating system): `macos`, `linux`, `windows`".
  - `ascribe render docs/install-agent.md --build <build>` for the three builds: the section is in each, between "Connect to Quill" and what follows, with `**macOS**`, `**Linux**`, and `**Windows**` in each. In `cloud`, "Connect to Quill" has lost its two labels, as that build means it to, and the new section keeps its three.
  - In a copy of the copy, `ascribe build`, then `.ascribe/build/<build>/site/install-agent.md`: in all three builds the section is `<ascribe-tabs sync="os">` with three `<ascribe-tab>` elements, `value="macos" label="macOS"`, `value="linux" label="Linux"`, and `value="windows" label="Windows"`, each holding an `<ascribe-steps>`. So on the site a reader does see one system at a time.

- **Seen on the way:**
  - `ascribe render` is how the skill says to see "a page as one build's readers see it", and for a switcher it shows every arm in a row under a bold label: `**macOS**`, steps, `**Linux**`, steps, `**Windows**`, steps. The task's point, one system at a time, can't be seen in it. The agent said it was so without seeing it, and the preparer had to build and read `<ascribe-tabs sync="os">` in the `site` output to know.
  - The agent checked one build's render and described three. Nothing prompted it to render the others, and `ascribe check`'s last line, `checked 4 files: 0 errors, 0 warnings`, doesn't say which builds it covered: phase 1's finding, seen again from the other side. In phase 1 the agent checked each build because the line didn't say; here it didn't.
  - Adding a dimension isn't described anywhere the agent looked. The skill's directive reference says `@variant` is "content that differs by a dimension" and that `ascribe model` lists the dimensions a project has; neither says how a project gets a new one. The agent copied the shape of `[dimensions.deployment]` from the lines above, comment and all, and it worked.
  - Both edits were made by a script in the shell, and the first check afterwards was the agent's own `ascribe check --format concise`. The plugin's description promises "the language server's diagnostics after each edit"; an agent that edits through Bash never gets them, and this one made every edit that way.
  - The skill was loaded first, as in the other sessions of this phase with the plugin, and the directive reference it links to was read from the plugin's folder. The plugin's tools and commands weren't used.
  - The hook had nothing to say, as in phase 1: the agent had checked already.

## What the session showed

One session, so nothing to compare it with but phase 1's.

- **The agent worked out the content model's part by itself.** Told only that the steps differ by system, it found there was no dimension for that, added `[dimensions.os]`, and wrote three `@variant {os=…}:` arms with `@steps` inside each. The check passed at the first try.
- **The plugin was used here, and it was the skill and its directive reference.** The tools, the commands, and the language server weren't touched, and the hook was silent.
- **What the task asked for was never seen by the agent.** "A reader should be able to see just their own system's steps" is true of the built site and invisible in `ascribe render`, which is what the agent looked at. It was right, on the strength of a comment in `ascribe.toml`.

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
