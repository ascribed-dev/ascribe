# Findings

Part of [The surface audit](README.md). What the audit found, each finding with its evidence quoted, and what it recommends. Nothing here is fixed by this plan: [phase 5](phase-5-findings.md) ends with the maintainer reading this and saying which findings become issues.

Each finding has a number, its kind, where it is, the evidence quoted, what it costs a user, one recommendation, and a size. A recommendation is one of rename, merge, remove, add, document, or leave. A finding keeps the number it was given when it was written, since other files point at it by number, so the numbers aren't the order. [The order](#in-order-of-what-they-cost-a-user) is below.

Where they came from:

| Findings | From |
|---|---|
| 1 to 23 | [Phase 1](phase-1-thread.md): one task, fixing a broken link, walked seven times. "The walks" are an agent's dry run in the editor (two copies) and at the command line, the maintainer's in the editor and at the command line, an agent with the plugin (two runs), and an agent with the MCP server and no shell |
| 24 to 27 | The `ascribe permalinks` commands, against the conventions |
| 28 to 44 | [Phase 4](phase-4-docs-site.md): [the docs site](docs-site.md) |
| 45 to 62 | [Phase 3](phase-3-journeys.md): nine agents' sessions; and [the capability table](capabilities.md) |

## What the audit found

**The surface holds together better than its size suggests.** One command, `ascribe check`, was the first move of every walker and every agent, with or without help, and every one that finished ended on it. The words for a project's parts (a build, a page, a phrase) are the same everywhere looked at. The commands agree on `--build`, on `text` as the default, and on 2 for "couldn't run". The MCP tools are the commands, under nearly the same names. Agents given a task and the skill got eight of nine right, most in under a minute.

**Where it doesn't hold, the cause is usually one of four.**

- **A next step that isn't there.** A problem's kind says "choose" and nothing lists the choices (1). A fix is worked out and can't be applied outside VS Code (52). `check`'s text gives no help where its JSON does (14). The skill points at two places that don't exist (46).
- **Two bases for a path.** A page is named from the project's folder by one command and from the content root by the next, in what they print (13), in what they accept (26), and in the prompts they write (49). It's the one inconsistency that reached every kind of user.
- **The editor and the command line are two products.** The writing help is all in VS Code, and renaming too (51). Checking every build, `report`, and `sources` are only at the command line (5). An agent in the editor can't run the binary the editor runs (4). And the editor says "no problems" in three states where it hasn't looked (3, 5, 6).
- **Things the first site had to build.** Navigation (28), a starter (29), "unreleased" (30), and four smaller ones. Three are already proposals; navigation, the largest, is planned nowhere.

**The agents' help is mostly unused.** Across eleven sessions with a shell, no agent called an MCP tool or the language server, the stop hook never had anything to say, and the same page came out in the same time with the plugin, with `sync`'s files, and with nothing (17). What agents did use was the skill, `AGENTS.md`, and `ascribe explain`. That's where a small amount of work would reach furthest (46, 47).

**Nothing found is left over.** Question 7 turned up one unused field in the docs' own model (34) and twelve stub files (43), and nothing in the product.

### In order of what they cost a user

First, the ones a user meets and is misled or stopped by:

1. **2.** A formatter on save changes what a page builds to, and nothing says so.
2. **3, 6, 5.** The editor says the project is clean when it hasn't checked: in Restricted Mode, before a page is opened, and for every build but one.
3. **1.** `choose`, with nothing to choose from.
4. **13, 26, 49.** One page, two bases for its path.
5. **4.** An agent in the editor can't run Ascribe.
6. **45.** A binary built from `main` says it's the release.
7. **28.** The sidebar is kept by hand, and nothing plans otherwise.

Then the gaps someone works round:

8. **51, 52.** Nothing at the command line renames, or applies a fix.
9. **48.** No form of `ascribe diff` shows one page's changes in a terminal.
10. **50.** A repository with several projects has nothing at its root.
11. **46, 47.** The skill points at two things that aren't there, and is silent on five an agent needed.
12. **53.** Nothing starts a project, or a page, for a person.
13. **29, 30, 31, 32.** What the next site will build again.
14. **36.** Permalinks that differ only by case.
15. **38, 54, 40, 37.** The docs: no guide to content that varies, getting started stops short, the home page lists a third of the commands, and every diagnostic sends its reader to the specification.

Then naming and agreement, which cost attention more than time:

16. **11, 24, 25.** `--format`'s words; `sync` without `--check`; `--list` meaning two things.
17. **9, 10.** Five words for a problem, and two counts of them.
18. **12.** Three ways to say which project.
19. **14, 15, 56, 19.** What `check`, `render`, and `link` leave out of their text.
20. **59, 58, 60.** The `new-page` command; what `sync` writes for Claude Code; two CI recipes.
21. **21, 23, 62, 39, 41, 42, 43, 44.** Smaller things.

And the ones recommended as "leave" or as only saying what's already so, each with its reason: **7, 8, 16, 17, 18, 20, 22, 27, 33, 35, 55, 57, 61.**

**Two pairs of recommendations pull against each other,** and which way to go is the maintainer's:

- **12 and the checklist's rule for naming a project.** Finding 12 merges `[PATH]` and `--project` into `--config`. The checklist's rule already says a command finds its project from its path arguments, which is what `[PATH]` on `model` is. Keeping the positional path and dropping only `--project` on `refs` is the smaller change, and fits the rule as written.
- **51, 52, and D1.** A command that renames and one that applies fixes would each want a tool, and the tools change nothing by decision 56. The commands can exist without tools, which keeps the decision and leaves an agent with no shell where it is.

## The seven questions

1. **One word for one thing.** Holds for the parts of a project. Breaks on what Ascribe reports (9, 10), on `sync` (24), on `--list` (25) and `--from` (27), and on two tools' names (61).
2. **The same thing, the same way.** The commands agree on `--build`, `--color`, `text` as the default, and 2 for "couldn't run". They disagree on the format words (11), on how the project is named (12), and on how a page is named, out (13), in (26), and in prompts (49). Exit code 1 looks like seven things and is one (20). The extension's titles are in two styles (22).
3. **What can be done where.** [The capability table](capabilities.md). Its gaps are findings 1, 4, 5, 48, 51, 52, and 53; its "no" cells each have a reason.
4. **Whether each thing can be found.** `ascribe check` was, by everyone. `explain`, `link`, and `refs` were found through `--help` or the skill, never guessed. `diff` and `drift` weren't found by the agents whose tasks they were for (47). **Change where the link goes** was found only by scrolling the palette. On the site, the home page names 6 of 18 commands (40) and no guide covers variants (38).
5. **Where a choice is made.** The editor's build is the project's choice, and the Problems panel follows it without saying so (5). The site's address is chosen twice, in two files that must agree (32). `ascribe agents sync`'s default writes for some agents and not others (58).
6. **What a problem says to do next.** The same broken link gives five different next steps (1, 14). Error messages turned out to be the best documentation an agent found (47).
7. **What's left over.** One field in the docs' own model (34), and twelve stub files (43). Nothing in the product.

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

### 7. Every address the binary prints was "Page not found", for a day

- **Kind:** a contradiction, since closed.
- **Where:** `ascribe --help`, `ascribe explain`, the code's link in the Problems panel, `docs` in `check`'s JSON; the published site.
- **Evidence:** "`https://ascribed-dev.com/go/ASC036/` and `https://ascribed-dev.com/go/cli-reference/` both gave "Page not found"". Still 404 at 9:17 PM on 10 October 2026, before the agents' walks. After the canary `0.2.1-next.13` was published and the site rebuilt, the same day: `go/cli-reference/`, `go/cli-check/`, `go/agents-guide/`, and `go/getting-started/` answer 200 and forward. `go/ASC036/` answers `301` with `location: /go/asc036/`, which answers 200 and forwards to `/reference/diagnostics/#ASC036`.
- **Cost:** A binary built from `main` can print addresses the site doesn't have yet, for up to a day. Nobody on a release meets it: the site is rebuilt before a release's binary exists.
- **Recommendation:** leave. It was the site waiting for a canary, not a fault in permalinks. One thing to look at in phase 4: the host answers a diagnostic's address, which the binary prints in capitals, with a redirect to lower case first. It works, and it's a second hop on every diagnostic's link.
- **Size:** none.

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
- **Seen since:** the two names are deliberate in one place. `outline`'s JSON has both, and the reference says "`page` is the path from the content root, as links write it, and `file` is from the project root, as `ascribe check` reports it." So `link`'s `from` is a `page` under a name that doesn't say so, answering an input given as a `file`. `permalinks retire --to`, `--page`, and `--under` take the `page` form only (finding 26).
- **Recommendation:** rename, so a command names a page back as it was given, from the project's folder. It changes a value in `link`'s JSON; the maintainer agreed to that on 10 October 2026, and the change records it in `project-docs/decisions.md`.
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
- **Seen again in phase 3:** an agent "checked one build's render and described three", with nothing in the line to prompt it ([journey 2](journeys/add-a-platform-section.md)). And the line has two forms: "`ascribe check content` ends `checked 6 files, reported on 6: 0 errors, 0 warnings`, and `ascribe check` ends `checked 6 files: 0 errors, 0 warnings`" ([journey 7](journeys/edit-an-unfamiliar-project.md)).
- **Recommendation:** add. The last line names the builds, or says "every build".
- **Size:** small.

### 16. `ascribe_check` answers with a line of terminal text inside JSON

- **Kind:** friction.
- **Where:** `ascribe_check`, the default `response_format`.
- **Evidence:** The agent's first call asked for `{"response_format": "detailed"}` unprompted. Its second, with `{}`, got `{"concise":"checked 4 files: 0 errors, 0 warnings\n"}`.
- **Cost:** Small here. The default is the cheaper answer, and the agent chose the other one first.
- **Recommendation:** leave. The concise form is the default the tools were given, and decision 56 fixes its shape. Phase 3's sessions will show whether agents keep asking for `detailed`.
- **Size:** small.

### 17. With a shell, an agent uses the plugin's skill, and nothing else of it

- **Kind:** a duplicate.
- **Where:** the plugin: nine tools, a skill, three commands, the language server, the stop hook.
- **Evidence:** Phase 1, two runs: `ascribe check`, `grep`, `cat`, and `git mv` in Bash, and neither "loaded the skill, called one of the plugin's tools, or used `LSP`." Phase 3, five sessions with the plugin, on harder tasks: four loaded the skill first and worked from it, and one ran the command `ascribe:new-page`. None called one of the plugin's tools or `LSP`: the notes say so of each ("The plugin's tools and commands weren't used", "never called one of the plugin's tools or `LSP`"). The stop hook ran in all seven and said nothing in each: `"stdout": ""`, `"exit_code": 0`. No diagnostic reached an agent after an edit: "Both edits were made by a script in the shell", and the one session that used `Write` and `Edit` got nothing back. In [journey 7](journeys/edit-an-unfamiliar-project.md), the same page was written with the plugin (25 seconds), with `ascribe agents sync`'s files (21), and with nothing (22): "The help changed the route, not the result or the time."
- **Cost:** None to these agents. To the project, a plugin whose description promises "the language server's diagnostics after each edit" to an agent that edits through its shell and never gets them, and nine tools that an agent with a shell passes over for the commands they wrap.
- **Recommendation:** document. The plugin's description says what each part is for: the skill for every agent, the tools for one with no shell, the hook for one that doesn't check. Nothing is removed: the one session without a shell used the tools at once, and a hook that's silent when the agent has checked is working. The skill is what was used, so it's where the work goes (findings 46 and 47).
- **Size:** small.

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
- **Recommendation:** remove the number formats from the generated schemas, which say nothing a client uses. They're there because the generator writes each Rust integer type's name as a `format` (`uint`, `uint32`, `uint8`, `int32`, 121 times in `schemas/`), not because anyone chose them; `"type": "integer"` and `"minimum": 0` stay, and the generated TypeScript types them as `number` either way. What's lost is a width hint for someone generating typed code from the schemas, which nobody does. The schemas are a contract; the maintainer agreed to the removal on 10 October 2026, and the change records it in `project-docs/decisions.md`.
- **Size:** small.

### 22. Command titles in the palette are in two styles

- **Kind:** a naming problem.
- **Where:** the extension's commands.
- **Evidence:** "`Ascribe: Actions for the Cursor`, `Ascribe: Changed Pages`, `Ascribe: Dim What the Build Leaves Out`, `Ascribe: Prompt Agent to Fix This File`; and `Ascribe: Change where the link goes`, `Ascribe: Add a variant`, `Ascribe: Give the heading a stable id`. `Ascribe: Focus on Content model View` names the view the side bar calls "Content Model"."
- **Cost:** The list reads as two products. Nobody was slowed by it.
- **Recommendation:** leave the two styles, and say why. With the whole manifest read ([the map](../surface.md#the-vs-code-extension)): the 28 in title case are commands that work the editor, and the 35 in sentence form are actions, whose label is also what the actions bar and the lightbulb show. One label in two cases would be two strings for one thing. The rule is in the [checklist](../checklists.md#a-new-editor-command-action-or-setting) now. Rename one thing: the view is "Content model" in the manifest and "Content Model" in the side bar's command.
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

## `ascribe permalinks`, against the conventions

The group merged the day the conventions were written, so it was built without them (README decision 8). Its name and its subcommands fit: a plural noun, four verbs, `--format text|json`, and `verify` exiting 1 for a name that isn't there and 2 when it couldn't check. Four things depart. They come after the walks' findings because no walk met them; phase 5 puts them in order with the rest.

### 24. Two commands named `sync`, and only one can say whether it's current

- **Kind:** a contradiction.
- **Where:** `ascribe agents sync`, `ascribe permalinks sync`.
- **Evidence:** `agents sync` has `--check`: "0 the files are written, or with `--check` up to date; 1 with `--check`, a file is out of date". `permalinks sync`'s options are `--config`, `--format`, and `--color`, and it "exits with `0` when the list is up to date, whether or not it changed". Whether the list is behind is `ascribe check`'s to say (`permalinks-list-behind`). Tried: `ascribe permalinks sync --check` answers `error: unexpected argument '--check' found`, exit 2.
- **Cost:** Someone who has learned `agents sync --check` for CI tries it on the other `sync` and gets a usage error. The answer exists, somewhere else.
- **Recommendation:** add. `permalinks sync --check`, exiting 1 when the list would change, as the checklist's convention for `sync` has it. `ascribe check` goes on reporting it too.
- **Size:** small.

### 25. `--list` is a path on one command and a switch on two

- **Kind:** a naming problem.
- **Where:** `ascribe permalinks verify --list <PATH>`; `ascribe explain --list`; `ascribe agents prompt --list`.
- **Evidence:** `--list <PATH>`: "The list to check against." `--list`: "List every diagnostic's code and name, one per line, instead." and "List the prompts, with their arguments, instead."
- **Cost:** One option word, two meanings, and one of them takes a value. Small, and it's in the group that also has a `list` subcommand.
- **Recommendation:** rename `verify`'s to say what it names, such as `--against <PATH>`. Nothing has released with it.
- **Size:** small.

### 26. `permalinks retire` names a page from the content root, where other commands take a path from the current folder

- **Kind:** a contradiction.
- **Where:** `ascribe permalinks retire --to`, `--page`, `--under`; `PAGE` on `outline` and `render`; `--from` on `link`.
- **Evidence:** `--page <PAGE>`: "Retire every permalink this page declares … by its path from the content root". Tried on 10 October 2026 in a copy of `examples/quill`, whose content root is `docs`, with a permalink on a heading of `docs/keys.md`, from the project's folder. `ascribe render docs/keys.md --build site` and `ascribe render keys.md --build site` both wrote the page. `ascribe permalinks retire --page keys.md --reason test` answered `retired rotate-keys`. `ascribe permalinks retire --page docs/keys.md --reason test` answered `` error: can't retire `docs/keys.md`: it declares no permalinks, and the list has none there ``, exit 2.
- **Cost:** The path that every other command takes, and that `check` prints, is refused here with a message that says the page has no permalinks when it has one. Someone would go and look for a missing `permalink=true` that isn't missing.
- **Recommendation:** rename, with finding 13: a page given to a command is a path from the current folder everywhere, as the checklist has it, and what's written into `ascribe.toml` stays from the content root.
- **Size:** small.

### 27. `--from` means the page a link is on, and a file of names

- **Kind:** a naming problem.
- **Where:** `ascribe link --from <PAGE>`; `ascribe permalinks verify --from <FILE>`.
- **Evidence:** `link`: "`--from <PAGE>` (required): the page the link is on". `verify`: "Also check the names in this file, one on each line."
- **Cost:** Slight. The two are far apart, and each reads naturally where it is.
- **Recommendation:** leave. An option is named for its part in its own command, which is what the convention says. It's listed so phase 5 sees it beside the others.
- **Size:** none.

## The docs site

[Phase 4](phase-4-docs-site.md)'s, from [the docs site's notes](docs-site.md): the site read as the first one built with Ascribe (findings 28 to 34), and as the way people find the surface (35 to 44). They come after the others because no walk met them; phase 5 puts them in order with the rest.

### 28. The sidebar is a list kept by hand, and the site orders its pages three ways

- **Kind:** a gap.
- **Where:** `site/src/nav.ts`, `site/test/nav.test.ts`; `llms.txt`; `docs/content/index.md`.
- **Evidence:** `nav.ts`: "Ascribe's content model has no page order or grouping, so the site lists its pages here, by their source files under docs/content/." The test: "The navigation lists every published page, and only those." The published sidebar's groups are "Start", "Guides", "Reference", "Contracts", with the guides in the order `nav.ts` gives. `https://ascribed-dev.com/llms.txt`, which Ascribe writes, has "## Pages", "## Contracts", "## Guides", "## Reference", each in order of file name. The home page is a third list, written in `index.md`, and it has fallen behind the other two (finding 40). The Pages view in the editor says it outright: "Ascribe doesn't know your site's navigation, so an orphan may still be reachable from a menu: it's a hint, not an error."
- **Cost:** 126 lines and a test in the first site, and the same again in each site after it. An agent reading `llms.txt` and a person reading the sidebar are given the same pages in two orders, and nothing can check a third list written in a page.
- **Recommendation:** add, as `site/README.md` lists it under "Navigation": an order and a group for each page, `ascribe check` reporting a published page in no group, the groups in the outputs, and a label for the navigation. No proposal in `project-docs/proposals/` plans it; the brainstorm's "Navigation and site structure" records the gap, and the versions proposal already writes its own way around it.
- **Size:** large.

### 29. Six more things the next site will write again

- **Kind:** a gap, in part deliberate.
- **Where:** `site/src/pages/[...slug].astro`, `site/src/content.config.ts`, `site/src/components/Toc.astro`, `site/src/components/Search.astro`, `site/src/pages/404.astro`, `site/test/links.test.ts`, `site/test/redirects.test.ts`, `site/netlify/edge-functions/markdown.ts`.
- **Evidence:** Getting started ends its Astro steps with "Define the content collection, a route, and a layout that loads the elements. [Astro] walks through each file." `examples/astro-site/src` has the same `content.config.ts`, `pages/[...slug].astro`, `pages/404.astro`, and `layouts/Docs.astro` the site has. Of the rest: the search proposal says "Search for readers, and anything a site serves. That's Pagefind's job, or the site's choice."; the search and answers proposal says "What the host owns: status codes, headers, content negotiation, bot protection, logs.", and plans redirects while carving out "everything that needs to know what was published before". `site/public/_redirects` has no rule yet, and a 41-line test for the rules it will have. `links.test.ts` checks "Every link and anchor inside the built site", which no command of Ascribe's does for a site's own built pages.
- **Cost:** A site builder copies four files from a guide before a page shows, then writes search, redirects, and a check of the built links themselves. Each is small; together they're 361 lines of this site.
- **Recommendation:** leave search, the table of contents, the not-found page, and the request for Markdown: each is decided, or already as small as it gets. Add a starter with the four files, so the guide's steps 4 to 6 are a command. Whether Ascribe checks a built site's own links, and notices a page whose address moved, is for the search and answers proposal's carved-out part, and should be said there.
- **Size:** small for the starter.

### 30. "Unreleased" is kept by hand, in five places

- **Kind:** a gap, already planned.
- **Where:** `docs/ascribe.toml`: `[dimensions.release]`, `[versions]`, `[lifecycle.unreleased]`, `[features.next]`, `[phrases] version`; `site/src/styles/site.css`; every `next` mark in `docs/content/`.
- **Evidence:** The model: "A feature in `main` that no release has yet is `@available: next`. When a release ships it, each `next` mark becomes `ascribe <version>`". `RELEASING.md`: "set `version` in `docs/ascribe.toml`'s `[phrases]` and `[versions] current` to the release, and change each `next` mark under `docs/content/` to `ascribe <version>`". There are 106 such marks today, 86 written by hand. The stylesheet: "Unreleased features are this site's one lifecycle state." The versions proposal: "Pointing `next` at the following line, and rewriting the marks that meant this one, is a second, and it's the one that was done by hand after 0.2.0."
- **Cost:** A release ends with a pull request that edits one number in two places and about a hundred marks, by hand, and a dimension exists that no content varies along.
- **Recommendation:** add, as the versions proposal's level 2 has it ("One site that knows its versions"). Nothing new to plan: this is the first customer having asked for what's proposed.
- **Size:** medium, the proposal's.

### 31. A layout can't ask a page where its source is

- **Kind:** a gap.
- **Where:** `site/src/site.ts`, `site/src/layouts/Docs.astro`; what `@ascribed/astro`'s collection gives a layout.
- **Evidence:** For "Edit this page on GitHub", `site.ts` takes the source path out of the path the collection loaded the page from: `/\.ascribe\/build\/[^/]+\/site\/(.+)$/`, with the comment "the site output mirrors the content root's paths", and throws "not a page of the site output" otherwise. A built page's frontmatter has its `title` and no path. `_ascribe/permalinks.json` in the same output has one for each permalink: `"source": "a.md"`.
- **Cost:** An edit link, which most docs sites have, depends on where Ascribe's output folder is and how it's laid out, read back out of a file path.
- **Recommendation:** add. The page's path from the content root is in what the collection gives a layout, beside `title` and `available`.
- **Size:** small. It adds a field to the site output, which is a contract.

### 32. The site's address is written in two files that must agree

- **Kind:** a duplicate.
- **Where:** `[consumer] site`, `base-path`, `trailing-slash` in `ascribe.toml`; `site`, `base`, `trailingSlash` in `astro.config.mjs`.
- **Evidence:** `site/astro.config.mjs`: "`site`, `base`, and `trailingSlash` repeat ../docs/ascribe.toml's [consumer]; the integration fails the build if they disagree." So a pull request's preview, which has its own address, rewrites both files before it builds, in `.github/workflows/site.yml`: `sed -i "s|\"$site\"|\"$address\"|" docs/ascribe.toml site/astro.config.mjs`, then two `grep -q` lines to check the rewrite took.
- **Cost:** Every site states three settings twice, and any site with previews at their own addresses writes this step. Question 5 asks whether a choice is made in two places; this one is, and the product's answer is to fail when they differ.
- **Recommendation:** merge. The integration gives Astro the three settings from `[consumer]` when `astro.config.mjs` doesn't set them, and goes on failing when both are set and differ. A preview then changes one line.
- **Size:** small.

### 33. Facts the docs repeat from other files are held by tests, not by Ascribe

- **Kind:** friction.
- **Where:** `docs/ascribe.toml` `[phrases]` and `[sources.code]`; `tests/conformance/tests/docs.rs`; `scripts/docs-site/facts.test.ts`.
- **Evidence:** Four of the docs' five phrases copy a value from another file: `version` ("the workspace version in Cargo.toml"), `node` ("`engines.node` in package.json"), `glibc` ("`GLIBC` in .github/workflows/release-build.yml"), and `vscode` ("`engines.vscode` in packages/vscode/package.json"). `the_docs_phrases_match_their_sources` fails with "the phrase `{key}` is … in docs/ascribe.toml, but … in {file}". The folders in `[sources.code] include` are "listed in three other places, which need the same change when `include` changes", and `facts.test.ts` "fails until they agree".
- **Cost:** About 160 lines of test in two languages to keep seven values equal. A project without Ascribe's test suite would have the phrases and nothing holding them.
- **Recommendation:** leave. The tests work, the versions proposal gives `version` a home, and three values don't make the case for a phrase read from a file. It's listed so phase 5 sees that the first customer asked, and that no proposal answers.
- **Size:** none.

### 34. The docs' own model declares a field no page uses

- **Kind:** left over.
- **Where:** `docs/ascribe.toml`, `[types.reference.frontmatter]`.
- **Evidence:** `since = "string?"`. No file under `docs/content/` has `since:` in its frontmatter; the six pages that say when they arrived use `available:`. `ascribe report inventory --config docs` answers "Entries nothing uses: none.", because what it counts is "the fragments, phrases, features, glossary terms, and images nothing uses". With `since` gone the two page types, `guide` and `reference`, declare the same two fields.
- **Cost:** None to a reader. It shows that a frontmatter field can go unused without anything saying so.
- **Recommendation:** remove the field from the docs' model. Whether the inventory should count a field no page sets is a question for phase 5, with the rest of what `report` counts.
- **Size:** small.

### 35. A diagnostic's address takes two requests on this host

- **Kind:** friction. It's what finding 7 left for this phase.
- **Where:** `https://ascribed-dev.com/go/ASC036/`, which `ascribe explain`, `check`'s JSON, and the Problems panel print; Netlify.
- **Evidence:** `curl -sI https://ascribed-dev.com/go/ASC036/` answers `HTTP/2 301`, `location: /go/asc036/`, `server: Netlify`; `/go/asc036/` answers 200 with a page whose `<meta http-equiv="refresh">` goes to `/reference/diagnostics/#ASC036`. It's the host, not the integration or Astro's build: an address that has nothing to do with permalinks does the same (`/Reference/cli/` answers `301`, `location: /reference/cli/`; `/go/CLI-CHECK/` answers `301`, `location: /go/cli-check/`), and a capitalised address with no page behind it gets no redirect (`/go/NO-SUCH/` answers 404). The integration makes one route, `` `/${at.slice(root.length)}[id]` `` in `packages/astro/src/permalinks.ts`, with each permalink's name as it's written, so the build has the page at `go/ASC036/` (read in the code; the site wasn't built for this). By these answers, Netlify serves a path in lower case and redirects any other spelling of it there. The redirect keeps a query (`/go/ASC036/?pm=pnpm` goes to `/go/asc036/?pm=pnpm`), and the fragment is in the page's own forward, so nothing is lost.
- **Cost:** One more request on every diagnostic's link, 181 of the docs' 219 permalinks: measured once, 0.27 s against 0.21 s. A reader doesn't see it. A permalink whose name has a capital behaves this way on any host that folds case.
- **Recommendation:** leave. Printing the code in lower case would fix this host and break one that keeps case, where the page is at `go/ASC036/` and nowhere else; and `ASC036` is how the code is written everywhere a person reads it. If it's ever worth a hop, the place to change is what the integration names the page, not what the binary prints.
- **Size:** none.

### 36. Permalinks that differ only by case are accepted, and are one address here

- **Kind:** a gap.
- **Where:** `@id {permalink=true}`, `permalink:` in frontmatter; `ascribe check`; a host that folds case.
- **Evidence:** Tried on 10 October 2026 in a scratch project: two headings on one page with `@id {permalink=true}: Setup` and `@id {permalink=true}: setup`, and another page with `permalink: SETUP`. `ascribe check --format concise` answered `checked 2 files: 0 errors, 0 warnings`, and `ascribe build` wrote all three to `_ascribe/permalinks.json`: `"SETUP"` to `/b/`, `"Setup"` and `"setup"` to `/a/`. On the published site every spelling of an address is sent to the lower-case one (finding 35), so there the three would be one address. Build names already have the rule: "Build names are unique ignoring case (content-model.md, `model-build-name-case`)", and the output-layout contract refuses "paths that differ only in case (they'd be one file on macOS and Windows)". What the host does with two such pages wasn't tried; that both can't be reached is inferred from finding 35.
- **Cost:** Two names a product links to, one page answering for both, and nothing reported before or after the deploy. Unlikely, and silent when it happens.
- **Recommendation:** add. `permalink-duplicate` compares names without case, as the rule for build names does.
- **Size:** small.

### 37. Every diagnostic's entry sends its reader on to the specification

- **Kind:** a gap.
- **Where:** the Diagnostics reference's entries, generated from `tests/conformance/diagnostics.toml`.
- **Evidence:** The entry the binary's address leads to, for the broken link: "#### ASC036 `link-target-missing`", then "Error · file level · next step: choose · [SPEC §5.2]({repo}/blob/main/SPEC.md#52-links)", then **When**, **Message**, and **Fix**. All 180 entries have a link to `SPEC.md` on GitHub in that place. Links to a page of the docs are rare beside it: in the three generated files, two lines link to the directive reference, four to the `ascribe.toml` reference (all among the 22 content checks, none among the 72 entries about `ascribe.toml` itself), and three to a guide. The directive reference has the reverse link for each of its sections ("each section links to its part of the spec"), with 29 of them.
- **Cost:** The one page a person is sent to from an error has, as its only way onward, the normative specification on another site. The page written for them, with the example of a link that works, is two clicks back through the sidebar. Question 6 asks what a problem says to do next; in the docs the answer is "read the spec".
- **Recommendation:** add. Each entry links to the section of the directive reference or the `ascribe.toml` reference that covers it, beside the spec's. The registry already names the spec's section, and the two references already name theirs, so the link can be generated.
- **Size:** medium.

### 38. No guide covers content that varies

- **Kind:** a gap.
- **Where:** the sidebar's "Guides"; `@variant`, `@available`, `[dimensions]`, `[builds]`.
- **Evidence:** The seven guides are "Astro", "Editing", "Review", "Drift", "Agents", "Prose, through Vale", and "The project report". Counting `@variant`, `[builds.`, and `[dimensions.` in the guides: getting started has 5, the Astro guide 2, the editor guide 2, and the other five none. What getting started says of the feature is one bullet: "`@variant` marks alternatives by package manager. The site shows them as tabs, and a build can keep just one." The rest is `reference/directives.md` ("### `@variant`", "#### Whole-page variants", "### `@available`", "#### Availability specs") and `reference/content-model.md` ("## 6. `[dimensions.<name>]`", "## 16. `[builds.<name>]`"). The docs' own project has one build, so it has no page that shows a second one either.
- **Cost:** The proposal's second task, "Add a section that differs by platform", has no guide: the reader lands on two references and assembles a dimension, a variant group, and a build from them. It's what the language is for, by getting started's first sentence: "alternatives by platform or product, availability".
- **Recommendation:** add a guide: one dimension, one variant group, one availability mark, and two builds that publish them differently. Phase 3's journey for that task will show how far an agent gets without it.
- **Size:** medium.

### 39. The guide to permalinks is a section of the Astro guide

- **Kind:** friction.
- **Where:** `docs/content/guides/astro.md`, "## Permalinks"; `ascribe permalinks`.
- **Evidence:** The section is 100 of the page's 287 lines, with "### Linking from a product", "### Checking a product's links", "### Moving, aliasing, and retiring", and "### A permalink that isn't here". It's the only guide that names `ascribe permalinks`, seven times. Three of its four parts aren't about Astro: a product's code, a workflow "in the product's repository", and `ascribe permalinks retire`. It has a paragraph for everyone else: "**With another generator,** read `_ascribe/permalinks.json`". The home page mentions permalinks once, under "For implementers", as the format of a file.
- **Cost:** Someone who reads `ascribe permalinks --help`, follows it to the reference, and wants the how finds it under a site generator's name, or doesn't. A project on another generator has no reason to open the page.
- **Recommendation:** document. Permalinks get a guide of their own, and the Astro guide keeps what the integration does.
- **Size:** small.

### 40. The home page and getting started describe fewer pages and commands than there are

- **Kind:** a contradiction.
- **Where:** `docs/content/index.md`; `docs/content/getting-started.md`, "Next".
- **Evidence:** The home page: "[Command reference](reference/cli.md): `ascribe check`, `build`, `diff`, `drift`, `fmt`, and `lsp`, their options, outputs, and exit codes." Getting started: "[Command reference](reference/cli.md): `check`, `build`, `fmt`, and `lsp`." `ascribe --help` lists eighteen commands, and the reference has a section for each. The home page's list has sixteen of the docs' eighteen other pages; `guides/vale.md` ("Prose, through Vale") and `guides/report.md` ("The project report") aren't on it. Checked on the published page: its article links to neither `/guides/vale/` nor `/guides/report/`, and the sidebar beside it lists both.
- **Cost:** A reader who starts at the home page isn't told about two guides or twelve commands. The sidebar and `llms.txt` are right, so it's the page that's behind.
- **Recommendation:** document: add the two guides, and describe the command reference without listing commands. It will fall behind again while it's a list kept by hand (finding 28).
- **Size:** small.

### 41. `ascribe.toml` is described twice, in the same words

- **Kind:** a duplicate.
- **Where:** `docs/content/reference/content-model.md` (812 lines); `docs/content/contracts/content-model.md` (1,013 lines).
- **Evidence:** Of the reference's 301 lines longer than 40 characters, 123 are in the contract word for word. Both open with "### 1.1 The file", "### 1.2 Names", "### 1.3 Paths and patterns", and "### 1.4 How this reference describes keys"; the contract kept the last heading, and says its example files are "each valid under this reference". One piece is shared properly: both have `@include: ../_fragments/field-type-grammar.md`, the docs' only fragment written by hand. The sections are numbered one apart (`[builds.<name>]` is §16 in one and §17 in the other). Each page calls the other its counterpart: the contract says "For a guide to writing one, see the [`ascribe.toml` reference]".
- **Cost:** A change to a table of `ascribe.toml` is made in two pages, by hand, and `AGENTS.md`'s rule is "One home per fact. Don't copy a list, a name, or a version into a second file." A reader who searches for a key gets two pages that say the same thing and doesn't know which to trust.
- **Recommendation:** merge what's the same into fragments both pages include, as the field types' grammar already is. The contract keeps what only it has: the map to the spec, the validation rules, the decisions. The contract's text doesn't change, so no contract moves.
- **Size:** medium.

### 42. The element library and the review package have no page on the site

- **Kind:** a gap.
- **Where:** `@ascribed/elements`, `@ascribed/review`; `docs/content/guides/astro.md`, "Styling".
- **Evidence:** "Styling" is one paragraph: the elements are "themed with CSS custom properties such as `--ascribe-tip-color` and `--ascribe-tab-active-color`. The [element library's README]({repo}/blob/main/packages/elements/README.md#theming) lists them, and shows how to style your own note types and lifecycle states." The site that says so needed it: `site/src/styles/site.css` sets `--_color: var(--ascribe-state-preview-color)` for its own lifecycle state. `@ascribed/review` is linked the same way, to `packages/review/README.md`. Neither is a page, so neither is in the sidebar, in `llms.txt`, or in what search indexes.
- **Cost:** A site builder theming the elements, the third kind of user the audit names, leaves the docs for a README on GitHub at `main`, which may not be the version they installed. The names are a contract (`packages/elements/CONTRACT.md`) and the docs don't list them.
- **Recommendation:** document. A reference page for the elements and their properties, taking the list from the package so it has one home. Leave `@ascribed/review`: its README says "This README is for building a host", which nobody outside the repository does yet.
- **Size:** small.

### 43. READMEs link to pages by address, and the last move left twelve stubs

- **Kind:** a contradiction, and something left over.
- **Where:** `README.md`, `packages/*/README.md`, `crates/*/README.md`, `plugins/ascribe/README.md`; `docs/*.md` outside `content/`.
- **Evidence:** `site/README.md`: "What links in from outside the docs, the binary's help and diagnostics and the editor's links, uses a permalink instead, such as `/go/ASC036/`". The READMEs published to npm and the Marketplace are outside the docs too, and none of them does: the READMEs have 27 links to pages by address and none to `/go/`. Nineteen of the 27 are in the root's, the packages', and the plugin's (`https://ascribed-dev.com/guides/review/` four times, `/guides/editor/` four times, `/guides/agents/` three times); the other eight are in the crates'. It has already cost once. `docs/cli.md` and eleven files like it say: "This page is now [content/reference/cli.md](content/reference/cli.md). Package READMEs published before the move link here, so this file stays until a release republishes them." They were added on 5 October 2026. `v0.2.0` was tagged on 6 October, and its `packages/cli/README.md` links to `ascribed-dev.com/reference/cli/`, so the release that republishes them has happened.
- **Cost:** A page that moves breaks the READMEs of every version already published, and the only cover is a line in `site/public/_redirects` someone remembers to write. The twelve stubs are files a reader of the repository opens and learns nothing from.
- **Recommendation:** rename and remove. The READMEs link to permalinks, as the manifest and the plugin do (`/go/editor-guide/`, `/go/cli-reference/`, `/go/getting-started/`, `/go/agents-guide/`, and `/go/diagnostics/` exist; the Astro and review guides, the `ascribe.toml` reference, and two contracts need one each). The twelve stubs go: nothing has to be kept for 0.1.1's READMEs (README decision 3).
- **Size:** small.

### 44. What Ascribe writes into a project is said on seven pages, and listed on none

- **Kind:** a gap.
- **Where:** `.ascribe/`, `ascribe.lock`, `sources/`, `ascribe.permalinks`, the block in `AGENTS.md`, the skill's folder, the hooks' files.
- **Evidence:** Getting started: "Add the output directory to `.gitignore`". The Vale guide: "Add `.ascribe/` to your `.gitignore`, if it isn't there. Ascribe writes the preset there". The drift guide: "Commit `ascribe.lock` and `sources/`." The Astro guide: "It writes where it's running (`url` and `build`) to `.ascribe/dev.json`", and "The docs commit their list of permalinks, `ascribe.permalinks`". The agents guide has a table of three more (`.claude/settings.json` and `.mcp.json`, `.codex/hooks.json`, `.github/hooks/ascribe.json`), and `AGENTS.md` and the skill above it. Two contracts cover the formats of two of them. The proposal's table has this as a surface of its own, "Files Ascribe puts in a project", and no page has it as a subject.
- **Cost:** Someone who finds a file they didn't write, or is deciding what to commit, has no page that says which files are Ascribe's, which to commit, and which to ignore.
- **Recommendation:** document: one table, in the `ascribe.toml` reference or a page of its own, of each file, what writes it, and whether it's committed. Phase 2's inventory has the same list to draw on.
- **Size:** small.

## The agents' sessions

From [phase 3](phase-3-journeys.md): nine sessions on seven tasks. Each finding names the journey its evidence is in. Two of the nine weigh less than the rest: the session that set up CI left its copy and read Ascribe's own checkout, and the session that started a project used the published release throughout, as getting started told it to.

### 45. A binary built from `main` says it's the release

- **Kind:** a contradiction.
- **Where:** `ascribe --version`; `@ascribed/cli` on npm.
- **Evidence:** [Journey 6](journeys/set-up-ci-checks.md): "A binary built from `main` calls itself `ascribe 0.2.0`, the same as the release on `npm`, while the nightly package built from the same branch says `ascribe 0.2.1-next.13 (42f00096ceb0)`." The agent read "the version you have locally and the current npm `latest`" as one version and pinned it. That release rejects the project: ``[ASC061] Error: unknown key `role` ``, exit 2: "The workflow fails for a broken link, and it fails without one".
- **Cost:** Anyone working from a build of `main` is told they have the release, and what they write against it can fail on the release. Here it made a workflow that fails on every pull request.
- **Recommendation:** rename. A binary that isn't a release's or a canary's says so in its version, with its commit, as the canary's does.
- **Size:** small.

### 46. The skill sends an agent to two places that aren't there

- **Kind:** a contradiction.
- **Where:** the skill (`SKILL.md` and its directive reference); `ascribe model`; a project with the plugin and no `AGENTS.md`.
- **Evidence:** [Journey 3](journeys/show-an-example-from-code.md). The skill: "A project's own rules … are in the `AGENTS.md` beside its `ascribe.toml`". "The agent tried `AGENTS.md` and `docs/AGENTS.md` and got `Exit code 1`. That file is `ascribe agents sync`'s, and a project with the plugin alone has none." The directive reference: "Which note types, dimensions, and sources a project has: `ascribe model`." With a source declared, "`ascribe model` prints page types, dimensions, phrases, and builds, and no sources, and `ascribe model --section sources` answers `error: invalid value 'sources' for '--section <NAME>'`".
- **Cost:** The agent's first two moves on a task about sources were dead ends the skill pointed it at.
- **Recommendation:** add sources to what `ascribe model` shows, and have the skill say where a project's rules are when there's no `AGENTS.md`: `ascribe agents rules` prints them.
- **Size:** small.

### 47. The skill doesn't say how to do five things the sessions needed

- **Kind:** a gap.
- **Where:** the skill, its directive reference, and its list "Commands that answer questions".
- **Evidence:**
  - **Declare a source and mark a region.** "What taught the agent … was `ascribe explain` on diagnostics it hadn't been given: `ASC127`'s "declare one with `[sources.<name>]` and the `path` of the folder the code is in" and `ASC131`'s "mark one with `:snippet-start: {region}` and `:snippet-end:` comments". The skill's directive reference has one line on `@snippet`, and it doesn't say either." ([Journey 3](journeys/show-an-example-from-code.md).)
  - **Add a dimension.** "Adding a dimension isn't described anywhere the agent looked … The agent copied the shape of `[dimensions.deployment]` from the lines above". ([Journey 2](journeys/add-a-platform-section.md).)
  - **Show that a change changed nothing for readers.** "The agent built its own comparison: eighteen `ascribe render` calls into a scratch folder and `diff -r` … `ascribe diff` answers the same question in one command … The skill's list … has `explain`, `model`, `outline`, `link`, `render`, `refs`, and `check --summary`, and not `diff`." ([Journey 4](journeys/rename-a-phrase.md).)
  - **Know when an example changed.** "The agent never ran `ascribe drift`, the command for the task's second half, and the word isn't in the skill." (Journey 3.)
  - **Check in CI.** "Nothing the agent had says how to check a project in CI. Not the copy, not `ascribe --help`, not the skill: the word isn't in the skill." ([Journey 6](journeys/set-up-ci-checks.md).)
- **Cost:** Each agent got there, by copying, by reading error messages, or by building its own tool. The skill is the one thing of the plugin's that agents used (finding 17), and it's silent on five of the seven tasks.
- **Recommendation:** add each to the skill: a line for `diff` and one for `drift` in the commands, how a source and a dimension are declared in the reference, and the two commands a CI job runs.
- **Size:** small.

### 48. No form of `ascribe diff` shows what changed on one page, in a terminal

- **Kind:** a gap.
- **Where:** `ascribe diff`.
- **Evidence:** [Journey 5](journeys/review-a-pull-request.md): "The agent wanted one page's changes as a reader sees them, in the terminal, and no form of `ascribe diff` gave it. Three tries: `error: unexpected argument 'content/guides/rollouts.md' found`, `error: unexpected argument '--since' found`, and `error: a PAGE is named only with --format prompt`. The text format lists pages and counts (`guides/rollouts.md: 1 changed, 2 added, 1 removed`)". "The agent's first look at the changes was `git diff`."
- **Cost:** A reviewer at a terminal, person or agent, is told how many blocks changed on a page and has to go to `git diff` or the HTML report to see them. `diff`'s `PAGE` argument reads as "narrow to this page", as a path does for `git diff`, and means something else.
- **Recommendation:** add. `ascribe diff PAGE` shows that page's changed blocks as text, and narrows the JSON, so `PAGE` means the page the command is about, as it does on `outline` and `render`.
- **Size:** medium.

### 49. A prompt names its paths from the repository's root, to an agent standing in the project

- **Kind:** a contradiction.
- **Where:** `ascribe agents prompt new-page`; `ascribe diff --format prompt`.
- **Evidence:** [Journey 7](journeys/edit-an-unfamiliar-project.md): the prompt ends "When you're done, run `ascribe check handbook/pages/security/content`". "From `handbook/pages/security`, where the agent ran the prompt: `error: handbook/pages/security/content doesn't exist`, exit 2. From the root it works." [Journey 5](journeys/review-a-pull-request.md): "The prompt names the pages from the project, `content/guides/rollouts.md`, and the project from the repository, `Project: docs/`", and from the root "`ascribe render content/guides/rollouts.md --build site --config docs` answers `` error: `content/guides/rollouts.md` isn't a page or fragment of the project ``."
- **Cost:** A command copied out of a prompt works from one folder and fails from another, and the prompt doesn't say which. Both agents happened to be standing where it worked, or worked round it.
- **Recommendation:** rename, with findings 13 and 26. A prompt says once which folder its commands are for, and every path in it is from there.
- **Size:** small.

### 50. A repository with several projects has nothing at its root

- **Kind:** a gap.
- **Where:** `ascribe check`, `ascribe agents sync`, the stop hook, the plugin; a repository such as `examples/monorepo`.
- **Evidence:** [Journey 7](journeys/edit-an-unfamiliar-project.md): "`cat ascribe.toml` at the root: `No such file or directory`. A repository with three projects has nothing at its root that says so, and the plugin says nothing about which project a request is for." "`ascribe check` there exits 2, `error: no ascribe.toml found in … or any parent directory`", and `ascribe agents sync` "exits 2 with `no ascribe.toml found`" there too, so the preparer ran it in each project. What worked was what `sync` leaves behind: "The root's block did the routing the other two sessions did by hand."
- **Cost:** Where an agent starts, at the root, every command's answer is that there's no project. With `sync`'s files an agent opens the right project first; without them it searches.
- **Recommendation:** add. A command run above several projects and in none names them in its error, and `ascribe agents sync` at such a root writes each project's files.
- **Size:** medium.

### 51. Nothing at the command line renames across a project

- **Kind:** a gap.
- **Where:** the command line and the MCP tools, against **Rename this phrase everywhere**, **Rename a dimension value everywhere**, and `F2` in VS Code.
- **Evidence:** [The capability table](capabilities.md#finding-and-changing-across-the-project): three rows where VS Code renames and the command line and the tools don't. [Journey 4](journeys/rename-a-phrase.md): "Nothing of Ascribe's renames a phrase. The rename was `sed -i '' 's/{cloud}/{hosted}/g'` over two pages and a second `sed` for `ascribe.toml`. Ascribe's part was saying where the phrase is and, afterwards, that it's gone." The editor's rename "reaches every use: prose, link text and destinations, frontmatter fields with `phrases = true`, code blocks with `phrases=true`, and the files a `@snippet {phrases=true}` shows."
- **Cost:** The session's `sed` was right for a project of three pages. In one where a phrase is used in a snippet's file or a frontmatter field, `sed` over the pages misses what the editor's rename knows about, and the agent is the kind of user that has only `sed`.
- **Recommendation:** add. One command that does what `F2` does, for a phrase, a dimension's value, a heading's id, and a page that moves.
- **Size:** medium.

### 52. A fix can be listed anywhere and applied only in VS Code

- **Kind:** a gap.
- **Where:** `fixes` in `ascribe check --format json` and `ascribe_check`; the lightbulb.
- **Evidence:** [The capability table](capabilities.md#checking-and-fixing): "**gap**: `check`'s JSON lists each fix, and no command applies one", beside "The lightbulb's quick fixes". Each fix in the JSON is marked `safe` or `unsafe`, and decision 51 says a fix is `safe` "only when applying it can't change what the page says and leaves nothing to decide".
- **Cost:** An agent reads a fix as data and makes the edit itself, character by character, for a change Ascribe has already worked out and called safe.
- **Recommendation:** add. `ascribe check --fix` applies the safe ones, as `ascribe fmt` rewrites what it can.
- **Size:** medium.

### 53. Nothing starts a project, or a page, for a person

- **Kind:** a gap.
- **Where:** the command line, VS Code.
- **Evidence:** [The capability table](capabilities.md#starting): for a project, "`ascribe.toml` is written by hand, from getting started", and the walkthrough's "Open a project" opens one that exists. For a page, an agent has `ascribe agents prompt new-page` and a person has nothing. In [journey 7](journeys/edit-an-unfamiliar-project.md) the prompt "leaves out `review-by`, which the type lists as optional and both policies there have."
- **Cost:** Every new project begins by copying a block out of the docs, and every new page by copying a neighbor.
- **Recommendation:** add, with the [page templates proposal](../proposals/page-templates.md), which gives a type its page to start from. A project's first `ascribe.toml` belongs with it.
- **Size:** medium.

### 54. Getting started is titled for a published site and stops one step short of one

- **Kind:** friction.
- **Where:** `docs/content/getting-started.md`, as published.
- **Evidence:** [Journey 1](journeys/start-a-project.md): "Its last instruction for publishing is "Define the content collection, a route, and a layout that loads the elements. [Astro] walks through each file.", and nothing on the page builds, serves, or opens the site". "The page's two routes to a site, a new Astro project with Ascribe added or an Ascribe project with Astro added, aren't told apart". "`ascribe.toml`'s `[consumer]` `site` and Astro's `site` have to be the same, and the getting-started page shows each … without saying so." "`npx ascribe build`, the page's own step, wasn't needed: `[@ascribed/astro] running ascribe build for "site"`."
- **Cost:** A newcomer needs two pages to do what the first one promises. The agent got there in 97 seconds; it had to choose the last command itself.
- **Recommendation:** document. The page ends with the site open in a browser, or says plainly that the Astro guide is the second half. With finding 40.
- **Size:** small.

### 55. Getting started teaches the check with a link broken on purpose, and a first page has none

- **Kind:** friction.
- **Where:** `docs/content/getting-started.md`; `examples/getting-started`.
- **Evidence:** [Journey 1](journeys/start-a-project.md): "An agent writing its own first page doesn't make that link, so it never sees a diagnostic, a code, or `explain`: the first check is `checked 1 file: 0 errors, 0 warnings`."
- **Cost:** Someone who writes their own first page, as the page invites, finishes without having seen what Ascribe says when something's wrong.
- **Recommendation:** leave. The page's example is right for a reader following it, and a clean first check is the better first experience. It's here because the walkthrough and the page could both say where to see a diagnostic on purpose.
- **Size:** none.

### 56. `ascribe render` can't show that readers see one arm of a switcher at a time

- **Kind:** a gap.
- **Where:** `ascribe render`, for a build with `variants = "switch"`.
- **Evidence:** [Journey 2](journeys/add-a-platform-section.md): "for a switcher it shows every arm in a row under a bold label: `**macOS**`, steps, `**Linux**`, steps, `**Windows**`, steps. The task's point, one system at a time, can't be seen in it. The agent said it was so without seeing it, and the preparer had to build and read `<ascribe-tabs sync="os">` in the `site` output to know."
- **Cost:** The command the skill gives for "a page as one build's readers see it" shows a switcher the same as three sections in a row. An agent reports the difference on trust.
- **Recommendation:** add. The rendered text says, above a switcher's arms, that a reader sees one of them and which dimension chooses.
- **Size:** small.

### 57. A source with no `include` makes every file in its folder readable, and nothing remarks on it

- **Kind:** friction.
- **Where:** `[sources.<name>]`; `@snippet`.
- **Evidence:** [Journey 3](journeys/show-an-example-from-code.md): "The agent declared the source as the whole `agent` folder: `path = "agent"` and no `include`. `ascribe check` took it. Every file under `agent/` is now readable from a page … `@snippet: agent:package.json` on the page checks clean and renders the whole file." Ascribe's own docs declare theirs with `include`, commented "what's readable; everything else isn't".
- **Cost:** The shortest source that works is the widest. Nothing leaks until a page asks for a file, and then nothing says the file wasn't meant to be shown.
- **Recommendation:** leave, and tell the security plan, whose finding about sources outside a project is the same question from the other side. A check here would speak on every project that declares a source the simple way.
- **Size:** none.

### 58. What `ascribe agents sync` writes by default, Claude Code reads half of

- **Kind:** friction.
- **Where:** `ascribe agents sync`, with no `--target`.
- **Evidence:** [Journey 7](journeys/edit-an-unfamiliar-project.md): "Of what `ascribe agents sync` wrote, `AGENTS.md` was used and the skill wasn't … the skill in `.agents/skills/ascribe/` isn't in the session's list of skills and was never opened. `ascribe agents sync --help` says of `--target claude`: "the skill in .claude/skills/ascribe/". The plain command doesn't write that, and Claude Code didn't find the other." And: "The rules were enough to write the page without one command".
- **Cost:** Slight here, since `AGENTS.md` was enough. Someone who runs the plain command for Claude Code has a skill on disk that their agent doesn't see, and nothing says so.
- **Recommendation:** document. `sync`'s output says which agents read each file it wrote, and names the target that adds the rest.
- **Size:** small.

### 59. The plugin's `new-page` command needs the answer before it can ask

- **Kind:** friction.
- **Where:** the plugin's command `ascribe:new-page`; `ascribe agents prompt new-page`.
- **Evidence:** [Journey 7](journeys/edit-an-unfamiliar-project.md): "it's a wrapper round a command line: "Run `ascribe agents prompt new-page --arg type=<type> --arg title=<title> [--arg path=<path>]`". The agent couldn't run it until it knew the type, and to know the type it had to find the project and read its model, which is most of the job. The command's `argument-hint` is `<type> <title> [path]`, and the agent gave it the request's sentence." Of the three sessions: "The session with the most help took the most steps".
- **Cost:** The one plugin command an agent reached for added steps to the task it was for.
- **Recommendation:** rename its arguments to what a user has: the request. The command tells the agent to read `ascribe model` for the type, then gives the prompt.
- **Size:** small.

### 60. Two recipes for checking in CI, and neither is the one a project gets

- **Kind:** a contradiction.
- **Where:** `docs/content/getting-started.md`, "Check it in CI"; `examples/docs-repository/.github/workflows/check.yml`.
- **Evidence:** [Journey 6](journeys/set-up-ci-checks.md): "The getting-started page: `actions/checkout@v4`, `npm ci`, `npx ascribe check --deny-warnings`, `npx ascribe fmt --check`. `examples/docs-repository/.github/workflows/check.yml`: actions pinned by commit, `npm install --global @ascribed/cli@next`, `ascribe check --deny-warnings`, and no `fmt`." "The agent took half of each." It also "looked for three things Ascribe doesn't have, by name: an action to install it (`setup-ascribe`, `uses: ascribed-dev`), an output format for GitHub's annotations (`--format github`), and `cargo install`."
- **Cost:** Two of Ascribe's own files disagree about the job most projects set up once and copy. This session weighs less than the others, since it read them from Ascribe's checkout; the two recipes are there whoever reads them.
- **Recommendation:** merge into one recipe, a file the page shows with `@snippet` and the example uses. What the agent looked for and didn't find is listed for the maintainer, not recommended: nothing here shows a project needs them.
- **Size:** small.

### 61. Two MCP tools aren't named for the command they answer as

- **Kind:** a naming problem.
- **Where:** `ascribe_format` (`ascribe fmt`), `ascribe_changes` (`ascribe diff`); the other seven tools.
- **Evidence:** [The map](../surface.md#the-mcp-server): `ascribe_check` is `ascribe check`, `ascribe_explain` is `ascribe explain`, and so for `model`, `outline`, `link`, `refs`, and `render`. Then `ascribe_format` answers as `ascribe fmt`, and `ascribe_changes` as `ascribe diff`.
- **Cost:** Slight. An agent that knows the command can't spell the tool from it twice in nine.
- **Recommendation:** leave. Each name says what the agent gets better than the command's does, and the names are a contract (decision 56). The convention for a new tool is the command's own word.
- **Size:** none.

### 62. Smaller things at the command line

- **Kind:** friction, four times.
- **Where:** `ascribe diff`, `ascribe explain`, `ascribe model`, `ascribe refs`.
- **Evidence:**
  - "`ascribe diff --base 8254eaf` starts `compared with 8254eaf (8254eaf)`: the revision as given, then the commit, the same thing twice when a commit is given." (Journey 5.)
  - "`ascribe explain` with no code prints a usage error, and the agent's first search of it found nothing. `--list` was one `--help` away." (Journey 3.)
  - "To get the builds' names it ran `ascribe model --section builds --format json` and picked the names out with `grep -o '"name": *"[^"]*"'`". (Journey 4.)
  - "`refs` after the rename: `phrase:cloud doesn't exist.`, with exit 1. That's the answer the agent wanted, given as a failure." (Journey 4.)
- **Cost:** Each is a moment.
- **Recommendation:** print the commit once; have `explain` with no code say `--list` exists; leave the other two. A list of names is what the JSON is for, and exit 1 for a thing of the project's that isn't there is the convention (finding 20).
- **Size:** small, each.

## The proposals, against the findings

What each pending proposal or plan would add to the surface, laid against the conventions in [the checklists](../checklists.md#a-new-command-or-option) and the findings above. Where a name or a shape would repeat something the audit found, the proposal is changed, and says so.

| Proposal | What it adds | Against the audit |
|---|---|---|
| [Permalinks](../proposals/permalinks.md) (built) | `ascribe permalinks sync`, `list`, `verify`, `retire`; `@id {permalink=true}`; `[permalinks]` | Fits, with four departures already written up: findings 24, 25, 26, and 27. Finding 36 is its too |
| [Tested examples](../proposals/tested-examples.md) | `ascribe observe`, with `--name` | The name fits: a verb alone, while it's the only command about examples. If a second arrives (listing observations, saying which are stale), both belong to a group, `ascribe examples`, and the proposal now says so. It names no formats or exit codes; the conventions give them: `text` and `json`, and 1 when an example's test fails |
| [Versions](../proposals/versions.md) | `ascribe versions retire`, `ascribe versions release`, `ascribe archive --version "…"` | Two fit. The third doesn't, twice: it's a second command about versions outside their group, and `--version` already prints Ascribe's. It's `ascribe versions archive "self-managed 3.3"` now, in the proposal |
| [Page templates](../proposals/page-templates.md) | A command that writes the starter set into a project, left unnamed for the audit | `ascribe templates eject`: a group named for the thing, and the verb `ascribe vale eject` uses for the same job, writing Ascribe's defaults out once as the project's own. A later "what changed upstream" is `ascribe templates status`. In the proposal now. It also answers half of finding 53 |
| [Search](../proposals/search.md) | `ascribe search`, the tool `ascribe_search`, the request `ascribe/search` | The working name stands: a verb, a tool named for its command (finding 61's convention), `--limit` as `refs` and `report` have it, and exit codes that follow the rule. Its `path` argument should be read with findings 13 and 26: a path from the current folder |
| [The cache](../proposals/cache.md) | `--recheck` and `--no-history` on `ascribe report links`; files under `.ascribe/` | Nothing to rename. The files are two more for finding 44's table of what Ascribe writes into a project |
| [Notes](../proposals/notes.md) | `@comment`; note files | Its own open question is finding 9's: "note" already means a callout in `@note`, `[notes.<type>]`, and `note:` targets. One word for one thing says the new thing needs another word, and the proposal's question stands for the maintainer |
| [Search and answers](../proposals/search-and-answers.md) | A setting for how crawlers treat pages; files a build writes | Its setting belongs in `ascribe.toml`, by the rule for where a choice lives: it's about the content, not one person's editor or one run. Finding 32 applies: it must not be a second place the site's address is written |
| [The website](../website/README.md) (plan) | `projects` in `@ascribed/astro`'s options | Findings 29 and 32 are its neighbors: a starter, and the address written twice. Its phase 2 moves every page link in the READMEs, which is finding 43; if the READMEs link to permalinks first, that step is nothing |

## Not findings

- **`explain ASC036`'s example calls `keys.md` right and `api-key.md` wrong.** That's this script's rename meeting Ascribe's own example, not something a user's project would meet.
- **The agent with the plugin renamed the file back.** The copy's README told it to. The script's, and changed for phase 3.
- **Claude Code's `WebFetch` summarized the getting-started page and took every command out.** "That's the client's tool, not the site." The page's own pointer to its Markdown is the answer, and the agent found it.
- **The published docs describe a command the release doesn't have.** `npx ascribe agents --help` on 0.2.0 answers `error: unrecognized subcommand 'agents'`. The page marks that section "in main, not yet released", three times, and the agent read the mark rightly and skipped it. Finding 30 is about how that mark is kept.
- **The CI session's workflow fails on every pull request.** Partly the setup's: the copy's project uses a key the release doesn't know. What's Ascribe's in it is finding 45.
