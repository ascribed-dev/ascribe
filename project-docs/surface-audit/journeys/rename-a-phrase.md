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

### With the plugin

- **When, with what:** 10 October 2026, 9:56 PM. Claude Code 2.1.296, model `claude-opus-5-5`, started from zsh on macOS with `target/release` first on the path (`ascribe 0.2.0`). The plugin from the checkout, `ascribe@inline` version 0.2.0. Copy `rename-a-phrase-plugin`, in a scratch folder, prepared by the agent that ran it. The session's first event listed the plugin's nine tools (`mcp__plugin_ascribe_ascribe__ascribe_changes`, `…_check`, `…_explain`, `…_format`, `…_link`, `…_model`, `…_outline`, `…_refs`, `…_render`), `LSP`, the skill `ascribe:ascribe`, and the commands `ascribe:check`, `ascribe:new-page`, and `ascribe:review`; the server `plugin:ascribe:ascribe` was `connected`. The user's synced skills (`deep-research`, `design`, `slides`, `design-sync`, and others) and the account's connectors (`claude.ai Claude Docs` connected, three more `needs-auth`) were listed too, and none was used. Standard error was empty.

- **Transcript:** [`transcripts/rename-a-phrase-plugin.jsonl`](transcripts/rename-a-phrase-plugin.jsonl).

- **Steps:**

  1. `Skill`, `ascribe:ascribe`. Came back: `Launching skill: ascribe:ascribe`, then the skill's text, with "`ascribe refs <target>`: where a page, a heading (`guides/install.md#configure`), a phrase (`phrase:product`), or another entry is used."
  2. `Grep`, `cloud`, ignoring case. Came back: 21 lines, the phrase's five uses among them, with `values = ["cloud", "self-managed"]`, `[builds.cloud]`, `available: cloud, self-managed preview 3.3`, `@variant {deployment=cloud}:`, and the YAML key `cloud:`.
  3. `Glob`, `**/*`. The copy's files, and `.git`'s.
  4. `Read`, `ascribe.toml`.
  5. Bash: `ascribe refs phrase:cloud; ls -a; ascribe check --format concise; …; for b in $(ascribe model --section builds --format json | grep -o '"name": *"[^"]*"' | cut -d'"' -f4); do echo "build $b"; done`. Came back: the five places and `5 places use phrase:cloud.`; `checked 4 files: 0 errors, 0 warnings`; `build site`, `build cloud`, `build self-managed-3.3`.
  6. Bash, one command that did the whole job: a shell function that wrote `ascribe render $p --build $b` for the three pages in the three builds into a folder of the session's scratchpad; `snap before`; `sed -i '' 's/{cloud}/{hosted}/g' docs/install-agent.md docs/keys.md`; `sed -i '' 's/^cloud = "Quill Cloud"$/hosted = "Quill Cloud"/' ascribe.toml`; `snap after`; `diff -r "$S/before" "$S/after" && echo RENDERS IDENTICAL`; then `ascribe check --format concise; ascribe refs phrase:cloud; ascribe refs phrase:hosted; git diff --stat; grep -rn '{cloud' docs`. Came back: `RENDERS IDENTICAL`; `checked 4 files: 0 errors, 0 warnings`; `phrase:cloud doesn't exist.`; the five places and `5 places use phrase:hosted.`; `3 files changed, 5 insertions(+), 5 deletions(-)`.
  7. Bash: `wc -c "$S"/after/*; grep -c "Quill Cloud" "$S"/after/site-*`, described as "Confirm the rendered snapshots have real content". Came back: nine files, 7761 bytes in all.
  8. Answered: "The `cloud` phrase is now `hosted`, and readers see no difference: every page renders identically before and after in all three builds (`site`, `cloud`, `self-managed-3.3`), and `ascribe check` reports 0 errors and 0 warnings." And: "I left the other things named `cloud` alone, since they aren't the phrase: the `deployment` dimension value (used in `@variant {deployment=cloud}` and `available: cloud, …`), the `[builds.cloud]` build, and the `cloud:` key in the `quill.yaml` sample."
  9. The Stop hook ran.

- **Tried first:** the skill, before anything else. Then `Grep` for the word, then `ascribe refs phrase:cloud`, in the spelling the skill gives. No command was guessed wrong and no `--help` was read. It never called one of the plugin's tools or `LSP`, and it didn't run `ascribe diff`.

- **What the hook or the plugin's tools said:** the hook said nothing: `"hook_name": "Stop"`, `"stdout": ""`, `"stderr": ""`, `"exit_code": 0`, `"outcome": "success"`. No tool of the plugin's was called. Nothing was said after an edit: both edits were `sed` in Bash, so none went through a tool the language server's diagnostics follow.

- **Doubts:**
  - Which `cloud` is the phrase. The agent's `Grep` put all four kinds in front of it, and `ascribe refs phrase:cloud` answered with five places. Its `sed` patterns were written to take only those: `{cloud}` in the pages, and the line `cloud = "Quill Cloud"` in `ascribe.toml`. From its answer: "I left the other things named `cloud` alone, since they aren't the phrase".
  - Whether its own proof was real. After `RENDERS IDENTICAL` it ran one more command, "Confirm the rendered snapshots have real content", to see that the nine files it had compared weren't empty.

- **Time:** 21 seconds by the clock; the session reports 19.4 seconds, nine turns.

- **Result:** renamed, and right. The agent knew from its own before and after renders, from `ascribe check`, and from `ascribe refs`. The preparer ran, in the copy:
  - `ascribe check`: exit 0, `checked 4 files: 0 errors, 0 warnings`.
  - `git status --short`: ` M ascribe.toml`, ` M docs/install-agent.md`, ` M docs/keys.md`. `git diff`: five changed lines. `cloud = "Quill Cloud"` became `hosted = "Quill Cloud"`, and `{cloud}` became `{hosted}` on three lines of `docs/install-agent.md` (one of them twice) and one of `docs/keys.md`. Nothing else changed.
  - `ascribe refs phrase:cloud`: `phrase:cloud doesn't exist.`, exit 1. `ascribe refs phrase:hosted`: the five places, `5 places use phrase:hosted.`, exit 0.
  - The nine renders, compared with the nine kept before the session: `diff -r` found no difference.
  - `ascribe model --section dimensions`: `deployment` still has `cloud`. `--section builds`: the build is still `cloud`. `--section phrases`: `{hosted}`: Quill Cloud.
  - `ascribe diff`, which the agent didn't run: `compared with main (5a900d7)`, then `site: no changes`, `cloud: no changes`, `self-managed-3.3: no changes`, exit 0.

- **Seen on the way:**
  - The skill was the first thing the agent reached for, and `ascribe refs phrase:cloud` came from it in the skill's own spelling: "a phrase (`phrase:product`)". In phase 1 no agent with a shell loaded the skill or ran `refs`.
  - Nothing of Ascribe's renames a phrase. The rename was `sed -i '' 's/{cloud}/{hosted}/g'` over two pages and a second `sed` for `ascribe.toml`. Ascribe's part was saying where the phrase is and, afterwards, that it's gone.
  - To show "readers shouldn't see any difference" the agent built its own comparison: eighteen `ascribe render` calls into a scratch folder and `diff -r`, then a check that the files weren't empty. `ascribe diff` answers the same question in one command (`site: no changes`, `cloud: no changes`, `self-managed-3.3: no changes`), and the agent never ran it. The skill's list, "Commands that answer questions", has `explain`, `model`, `outline`, `link`, `render`, `refs`, and `check --summary`, and not `diff`.
  - To get the builds' names it ran `ascribe model --section builds --format json` and picked the names out with `grep -o '"name": *"[^"]*"'`: it wanted the builds' names alone, to loop over, and took them from the JSON.
  - `refs` after the rename: `phrase:cloud doesn't exist.`, with exit 1. That's the answer the agent wanted, given as a failure.
  - The hook had nothing to say, as in phase 1: the agent had checked already.
  - `ascribe.toml`'s first line in the copy is `# The content model for examples/quill: the model SPEC Appendix B assumes`. The README was left out and the content model's comment still says what the copy is. The agent read it and made nothing of it.

## What the session showed

One session, so nothing to compare it with but phase 1's. It went the way a careful person would want: find every use with the tool, change only those, prove the pages are the same, check. It took 21 seconds and one edit command.

- **The plugin was used here, and it was the skill.** The first step loaded it, and the command that mattered, `ascribe refs phrase:cloud`, was in it. The tools, the commands, and the language server weren't touched, and the hook was silent.
- **The trap didn't catch it.** `cloud` is a phrase, a dimension's value, a build, and a YAML key in this project. `refs` named the five places that are the phrase, and the agent's patterns took only those.
- **The agent proved the result with `render`, not `diff`.** It wrote a before-and-after harness around `ascribe render` for three pages in three builds. The one command made for the question wasn't in what the skill told it.

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
