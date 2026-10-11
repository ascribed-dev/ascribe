# Findings

Part of [The surface audit](README.md). These are [phase 1](phase-1-thread.md)'s: what one task, fixing a broken link, turned up, from the [inventory's slice](inventory-slice.md) and the [walks' notes](journeys/fix-a-broken-link.md). [Phase 5](phase-5-findings.md) adds the rest, reorders the whole list, and writes the opening section.

Each finding has a number, its kind, where it is, the evidence quoted, what it costs a user, one recommendation, and a size. They're in order of cost to a user. A recommendation is one of rename, merge, remove, add, document, or leave. Nothing here is fixed by this plan.

"The walks" are seven: an agent's dry run in the editor (two copies) and at the command line, the maintainer's in the editor and at the command line, an agent with the plugin (two runs), and an agent with the MCP server and no shell.

## The seven questions, on this slice

1. **One word for one thing.** The pattern holds for the parts of a project: a build, a page, a phrase are the same word everywhere looked at. It breaks on what Ascribe reports (finding 9) and on what's counted as one (finding 10).
2. **The same thing, the same way.** The commands agree on `--build`, on `--color`, on `text` being the default format, and on 2 for a command that couldn't run. They disagree on the format words (finding 11), on how the project is named (finding 12), and on how a page is named back (finding 13). Exit code 1 looks like seven things and is one (finding 20).
3. **What can be done where.** Three gaps on this task: checking every build from the editor (finding 5), the page a link meant, which only one editor action offers (finding 1), and running Ascribe from an agent in the editor (finding 4). The capability table is phase 2's.
4. **Whether each thing can be found.** `ascribe check` was: every walker's first move, with or without `--help`. `ascribe link`, `ascribe explain`, and `ascribe refs` were used by the dry run, which had read `--help`, and by no agent with a shell. **Change where the link goes** was found only by scrolling the palette. With a shell, nothing in the plugin was reached for (finding 17).
5. **Where a choice is made.** One on this task: the editor's build is the project's choice (`[editor] build`), and what the Problems panel covers follows from it with nothing in the editor saying so (finding 5).
6. **What a problem says to do next.** The same broken link gives five different next steps: nothing in `check`'s text, the fix's sentence in its JSON and in `explain`, **Prompt agent** alone in the lightbulb, an empty `closest` from `link`, and the page itself in the one action that lists it (findings 1 and 14).
7. **What's left over.** Nothing found on this task.

## The findings

### 1. `choose`, with nothing to choose from

- **Kind:** a gap.
- **Where:** `ASC036` in `ascribe check`, the lightbulb, `ascribe link`, `ascribe_link`.
- **Evidence:** The Diagnostics reference defines the next step `choose` as "You pick among things Ascribe can list: a link's target". For the broken link, `check`'s JSON has `"next":"choose"` and `"fixes":[]`. The lightbulb's menu had "one entry: "Prompt agent: fix this problem"". `ascribe link 'keys.md#rotate-keys' --from docs/install-agent.md` printed one line and no `Closest:`; `ascribe_link` answered `"closest":[]`. `api-keys.md` was in the same folder, with a heading whose id is `rotate-keys`. Checked afterwards: `link` does list a near spelling (`ascribe link api-key.md` gives `Closest:` `[API keys](api-keys.md)`), and `check` gives `"fixes":[]` for that link too. One place had it: **Change where the link goes** listed "Rotate keys  api-keys.md#rotate-keys".
- **Cost:** On all seven walks the walker found the page some other way: `ls docs`, `grep`, the copy's README, or the file name in a different diagnostic. A person in the editor is offered an agent before they're offered the page.
- **Recommendation:** add. `check` carries `link`'s closest targets as the diagnostic's fixes, marked `unsafe`, so the lightbulb, the JSON, and the tools all offer them; and `closest` looks for the link's heading id on other pages, which finds a renamed page however far its new name is from the old.
- **Size:** medium.

### 2. A formatter on save changes what a page builds to, and nothing says so

- **Kind:** a gap.
- **Where:** VS Code, a project's pages, any Markdown formatter.
- **Evidence:** From the editor dry run, with `"editor.formatOnSave": true` and Prettier as the formatter for Markdown: "The page was rewritten as it saved: line 93 became line 106." Of Prettier's five kinds of change, two were reported (`ASC021`, `ASC016`) and three weren't. One of the three changes the output: "`version: {version}` became `version: { version }`", and "the built page says `version: { version }` where it said `3.4.1`, with nothing reported."
- **Cost:** A person with a common pair of settings breaks pages by saving them, and for part of the damage no check tells them.
- **Recommendation:** add. The extension is the formatter for a project's pages, so another one doesn't run on them, and getting started says so. What to do about a phrase written `{ version }` is a separate question for the language, not answered here.
- **Size:** medium.

### 3. In Restricted Mode the extension is silent, and the panel says the project is clean

- **Kind:** a gap.
- **Where:** VS Code, a folder opened for the first time.
- **Evidence:** "The Problems panel: "No problems have been detected in the workspace." with each page open." "Nothing from Ascribe said it wasn't checking." The only notice was VS Code's own banner.
- **Cost:** Someone who opens a project for the first time and looks for problems is told there are none.
- **Recommendation:** add. The extension says in the status bar that it isn't checking until the folder is trusted.
- **Size:** small.

### 4. An agent in the editor can't run Ascribe

- **Kind:** a gap.
- **Where:** **Prompt agent**, the extension's bundled binary, an agent in the same window.
- **Evidence:** The maintainer's editor walk, the agent's answer: "I fixed the broken link, but I couldn't run `ascribe check`, so the fix is unverified." "The `ascribe` command isn't installed here. It's not on the PATH, and there's no local copy in the project." `check --format prompt`, which **Prompt agent** hands over, ends "When you're done, run `ascribe check` and fix what it reports."
- **Cost:** The path the maintainer took first ends with an unverified edit and a request to check by hand, in an editor that has the binary.
- **Recommendation:** add. The extension puts the binary it runs on the path of the window's terminals, and the prompt names the command as that window can run it.
- **Size:** small.

### 5. Nothing in the editor checks every build

- **Kind:** a gap.
- **Where:** the Problems panel, the palette.
- **Evidence:** The editor guide: "before calling a project clean, run `ascribe check`". The Problems panel shows "the editor's build only". From the dry run: "The Content Model view lists three builds. Didn't find a command that checks the others." From the maintainer's walk: "Project seems clean. Nothing in the problems panel."
- **Cost:** "No problems" means one build of three, and the docs' answer is a terminal. It didn't matter on this task. It will on a page that differs by build.
- **Recommendation:** add. A command that checks every build and puts the result in the Problems panel, and the status bar's build saying when the panel covers it alone.
- **Size:** medium.

### 6. The project's problems wait for a page to be opened

- **Kind:** friction.
- **Where:** VS Code, a trusted folder with no page open.
- **Evidence:** "The status bar stayed at 0 errors, 0 warnings, and had no "Review: off" and no build." Then, on opening "`docs/quickstart.md`, a page with no problem": "The status bar went to 1 error, 0 warnings, 1 info".
- **Cost:** A project with errors shows none until any page is opened, and nothing says it's waiting.
- **Recommendation:** add. The status bar says the project isn't being checked yet, as in finding 3.
- **Size:** small.

### 7. Every address the binary prints was "Page not found"

- **Kind:** a contradiction.
- **Where:** `ascribe --help`, `ascribe explain`, the code's link in the Problems panel, `docs` in `check`'s JSON; the published site.
- **Evidence:** "`https://ascribed-dev.com/go/ASC036/` and `https://ascribed-dev.com/go/cli-reference/` both gave "Page not found"". Still 404 at 9:17 PM on 10 October 2026, before the agents' walks.
- **Cost:** For a binary built from `main`, the help's last line and every diagnostic's link lead nowhere until the site is rebuilt from a canary that has permalinks. The released 0.2.0 prints the older addresses, which work.
- **Recommendation:** leave, if the addresses answer after the next nightly canary. If they don't, it's a bug in the permalinks work, and an issue.
- **Size:** small.

### 8. The error under a link is hard to see

- **Kind:** friction.
- **Where:** VS Code's editor, a diagnostic on a link.
- **Evidence:** The maintainer: "Doesn't show directly in the file at first glance." and "I just didn't notice it because the link's underline sits above the squiggles."
- **Cost:** A person looking at the broken line doesn't see it marked. The Explorer's red `1` and the Problems panel got them there in under a minute.
- **Recommendation:** leave. VS Code draws both lines, and the other two signs worked. Look again in phase 5 if the editor's inventory shows the extension chooses the link's underline.
- **Size:** small.

### 9. Five words for what Ascribe reports

- **Kind:** a naming problem.
- **Where:** every surface that reports one.
- **Evidence:** From the inventory: "The registry, `explain`, the JSON, and the plugin's server say *diagnostic*; the docs for `concise`, the Problems panel, the prompts, the MCP tool, and the skill say *problem*; `report` says *finding*; Vale's are *alerts*; the summary line and the hooks count *errors*, *warnings*, *advice*." One tool has both: `ascribe_explain` is titled "Explain a diagnostic", and `ascribe_check` reports "every *problem*".
- **Cost:** Someone who reads "problem" in the editor and "diagnostic" in `--help` has to work out they're one thing. Nobody on the walks was stopped by it.
- **Recommendation:** document, then rename what departs. One rule: a *problem* is what Ascribe found in your project, and a *diagnostic* is the kind of problem, with a code, that the reference and `explain` describe. `report`'s "findings" become problems.
- **Size:** small.

### 10. What counts as a problem depends on who's told

- **Kind:** a contradiction.
- **Where:** `check --format prompt`, `check`'s summary line, the hooks.
- **Evidence:** For one project: `Fix the 2 problems `ascribe check` reports in this project.`, and `checked 4 files: 1 error, 0 warnings, 1 advice`. The hook after an edit reports "*errors*, at most 10, errors only". The skill says "fix every *error*, and the *warnings* your edit caused".
- **Cost:** An agent handed the prompt is told to fix advice that the hook and the skill tell it to leave alone.
- **Recommendation:** rename. The prompt counts as the summary does: "Fix the 1 error", with advice listed after it as something to look at.
- **Size:** small.

### 11. Five sets of words for `--format`, and `summary` means two things

- **Kind:** a contradiction.
- **Where:** `--format` on fourteen commands; `check --summary`.
- **Evidence:** From the inventory: "`text|json` (nine commands); `text|concise|json|prompt` (`check`); `text|json|html|prompt` (`diff`); `text|json|summary` (`drift`, `sources update`); `text|summary|json|prompt` (`report`). `summary` means Markdown for CI in three commands, and `--summary` on `check` means counts."
- **Cost:** A format learned on one command can't be assumed on the next, and one word names a file format in one place and a shorter report in another.
- **Recommendation:** rename. The format that writes Markdown for a CI job is `markdown`; `--summary` stays an option that shortens a report. The words and their meanings are now in the [checklist](../checklists.md#a-new-command-or-option). That a command has only the formats it needs is right, and stays.
- **Size:** small.

### 12. Three ways to say which project

- **Kind:** a duplicate.
- **Where:** `--config` on every command; `[PATH]` on `model` and `agents rules`; `--project` on `refs`.
- **Evidence:** From the inventory: "`--config <PATH>` on every command; `[PATH]` positional on `model` and `agents rules`; `--project <PATH>` on `refs`." `--config` takes "the content model, or a directory holding one"; the other two take "a file or folder in the project, to find its `ascribe.toml` from".
- **Cost:** Three spellings for one question, and the one every command has is the only one that can't be given a page.
- **Recommendation:** merge. `--config` accepts any path in the project and finds the nearest `ascribe.toml` at or above it; `[PATH]` and `--project` go.
- **Size:** small.

### 13. One page, two names in one answer

- **Kind:** a contradiction.
- **Where:** `ascribe check` and `ascribe link`; `ascribe_check` and `ascribe_link`.
- **Evidence:** "`check` says `docs/install-agent.md:93:90`, and `link`, given `--from docs/install-agent.md`, answers "doesn't work from install-agent.md"." In the MCP session: `"file":"docs/install-agent.md"` from one tool, and `"from":"install-agent.md"` from the next, for `"from": "docs/install-agent.md"` given.
- **Cost:** An agent that takes a path from one answer and gives it to the next command is using two bases without being told. Both work as input today, so nothing broke.
- **Recommendation:** rename, so a command names a page back as it was given, from the project's folder. **This changes a value in `link`'s JSON, so it needs the maintainer's decision before it's an issue.**
- **Size:** small.

### 14. `check`'s text gives a next step only when there's a fix to apply

- **Kind:** a gap.
- **Where:** `ascribe check`, text.
- **Evidence:** For `ASC036` the text has the message and the source, and "No `Help:` line (an `ASC041` in the reference's sample has one, from its fix's title)". The JSON for the same diagnostic has `"help":"Fix the path, or create the file. Names must match exactly, including case, and the file must be inside the project."`
- **Cost:** A person at a terminal gets less than a tool reading JSON, and has to know to run `ascribe explain`.
- **Recommendation:** add. The text prints the diagnostic's help when it has no fix to name.
- **Size:** small.

### 15. `check`'s last line doesn't say which builds it covered

- **Kind:** a gap.
- **Where:** `ascribe check`, text and `concise`.
- **Evidence:** After `checked 4 files: 0 errors, 0 warnings`, the agent with the plugin ran `for b in site cloud self-managed-3.3; do ascribe check --build $b 2>&1 | tail -1; done`, and reported "0 errors, 0 warnings, on the default check and on each of the `site`, `cloud` and `self-managed-3.3` builds." The JSON has `"builds_checked":["site","cloud","self-managed-3.3"]`.
- **Cost:** A careful agent checks three more times to learn what one word would have told it.
- **Recommendation:** add. The last line names the builds, or says "every build".
- **Size:** small.

### 16. `ascribe_check` answers with a line of terminal text inside JSON

- **Kind:** friction.
- **Where:** `ascribe_check`, the default `response_format`.
- **Evidence:** The agent's first call asked for `{"response_format": "detailed"}` unprompted. Its second, with `{}`, got `{"concise":"checked 4 files: 0 errors, 0 warnings\n"}`.
- **Cost:** Small here. The default is the cheaper answer, and the agent chose the other one first.
- **Recommendation:** leave. The concise form is the default the tools were given, and decision 56 fixes its shape. Phase 3's sessions will show whether agents keep asking for `detailed`.
- **Size:** small.

### 17. With a shell, an agent uses nothing the plugin adds

- **Kind:** a duplicate.
- **Where:** the plugin: nine tools, a skill, three commands, the language server, the stop hook.
- **Evidence:** Both runs with the plugin were `ascribe check`, `grep`, `cat`, and `git mv` in Bash. Neither "loaded the skill, called one of the plugin's tools, or used `LSP`." The hook ran once: `"stdout": ""`, `"exit_code": 0`.
- **Cost:** None on this task. It's a question about what the plugin is for when the commands are in reach.
- **Recommendation:** leave. One easy task is no ground for removing anything, and the hook's worth is the agent that doesn't check. Phase 3's harder tasks decide.
- **Size:** none yet.

### 18. The stop hook lets an agent go from a project that was already broken

- **Kind:** a gap that's deliberate.
- **Where:** `ascribe agents hook claude-code --event stop`.
- **Evidence:** Run by hand in a copy with the broken link committed and nothing changed: no output, exit 0. The agents guide: it "checks every build of each project whose pages or `ascribe.toml` the working tree changes".
- **Cost:** An agent asked to fix a project and doing nothing isn't held.
- **Recommendation:** leave. A hook that held every agent for errors it didn't cause would be turned off, and the guide says what it covers.
- **Size:** none.

### 19. `link` tells the walker to write more than the fix needs

- **Kind:** friction.
- **Where:** `ascribe link`, `Write:`.
- **Evidence:** "`link` said `Write: [Rotate keys](api-keys.md#rotate-keys)`, and the page had `[](keys.md#rotate-keys)`, with no text. Wasn't sure whether to put the text in."
- **Cost:** A moment's doubt, and a diff larger than the fix if it's followed.
- **Recommendation:** add. `Write:` gives the empty form too, since Ascribe shows the title for it.
- **Size:** small.

### 20. Exit code 1 looks like seven things

- **Kind:** not a problem, once it's said.
- **Where:** every command's exit codes.
- **Evidence:** From the inventory: "Errors (`check`, `build`); a file out of date (`agents sync --check`, `fmt --check`); a target that doesn't exist (`link`, `refs`); a build that doesn't publish the page (`outline`, `render`); something changed (`diff --exit-code`); a finding at a level (`report --exit-code`); a file that couldn't be copied (`sources fetch`)." And five commands' codes are "not stated separately".
- **Cost:** None in use. Each is "the command ran, and the answer is the one you'd act on", and 2 is always "couldn't run". A name of Ascribe's own that doesn't exist (a diagnostic's code, a build) is 2, and a thing of the project's that doesn't exist (a page, a heading) is 1.
- **Recommendation:** document. The rule is now in the [checklist](../checklists.md#a-new-command-or-option); the reference states the codes for `agents rules`, `agents skill`, `agents prompt`, `agents hook`, and `sources status`.
- **Size:** small.

### 21. The MCP tools' schemas make the client print 94 warnings

- **Kind:** friction.
- **Where:** `ascribe mcp`'s tools, their output schemas; Claude Code 2.1.296.
- **Evidence:** On standard error while the tools loaded: `unknown format "uint32" ignored in schema at path "#/$defs/Pos/properties/line"`, `unknown format "uint" ignored in schema at path "#/$defs/CodeCount/properties/count"`, and 92 more.
- **Cost:** Nothing fails. A person who starts the client in a terminal sees a screen of warnings about Ascribe.
- **Recommendation:** remove the number formats from the generated schemas, which say nothing a client uses. **The schemas are a contract, so this needs the maintainer's decision before it's an issue.**
- **Size:** small.

### 22. Command titles in the palette are in two styles

- **Kind:** a naming problem.
- **Where:** the extension's commands.
- **Evidence:** "`Ascribe: Actions for the Cursor`, `Ascribe: Changed Pages`, `Ascribe: Dim What the Build Leaves Out`, `Ascribe: Prompt Agent to Fix This File`; and `Ascribe: Change where the link goes`, `Ascribe: Add a variant`, `Ascribe: Give the heading a stable id`. `Ascribe: Focus on Content model View` names the view the side bar calls "Content Model"."
- **Cost:** The list reads as two products. Nobody was slowed by it.
- **Recommendation:** rename to one style. Which one is phase 5's, with the whole manifest in front of it.
- **Size:** small.

### 23. Smaller things in the editor

- **Kind:** friction, four times.
- **Where:** the output channel, the clipboard, the Problems panel, `check`'s summary.
- **Evidence:**
  - "The output channel logs `ascribe-lsp: using the project at …/ascribe.toml` at `[error]`, when nothing is wrong, and doesn't give the binary's version."
  - "`Prompt agent: fix this problem` replaced what was on the clipboard, and said so only in the status bar."
  - "The error went as the link changed; the advice ASC154, which the same change answers, stayed until the file was saved."
  - "`check`'s last line counts advice only when there is some: `checked 4 files: 1 error, 0 warnings, 1 advice`, then `checked 4 files: 0 errors, 0 warnings`."
- **Cost:** Each is a moment's doubt.
- **Recommendation:** rename the log's level and add the version; leave the clipboard, which the notification that follows explains; document that checks across pages run on save; and print all three counts every time.
- **Size:** small, each.

## Not findings

- **`explain ASC036`'s example calls `keys.md` right and `api-key.md` wrong.** That's this script's rename meeting Ascribe's own example, not something a user's project would meet.
- **The agent with the plugin renamed the file back.** The copy's README told it to. The script's, and changed for phase 3.
