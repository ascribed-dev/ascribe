# Arrive in an unfamiliar project and make a correct edit

Part of [The surface audit](../README.md), [phase 3](../phase-3-journeys.md), journey 7. The script first, the same for all three sessions; then each session's notes.

## The task

> Add a policy on laptops to the security handbook: disks are encrypted, screens lock after five minutes, and a lost laptop is reported within an hour.

That sentence is the whole prompt: a one-line request, as someone would give it to an agent in a repository neither has read. It doesn't say where the security handbook is, that it's a project of its own, or what a policy is there.

What a check can't confirm is whether the page is a policy as this project means one. The repository has three Ascribe projects and no `ascribe.toml` at its root. The security handbook is the innermost, `handbook/pages/security/`, nested in the engineering handbook's content root. Its content model gives a file under `policies/` the type `policy`, whose frontmatter needs `owner`, `review` (`quarterly` or `yearly`), and `effective` (a date). It has a note type, `policy`, that its two policies state their rule in, and phrases for the team's channel and address. A page written anywhere else checks clean with a `title` alone.

## Starting point

A fresh copy of `examples/monorepo`, without the example's `README.md`, as a git repository with one commit. `docs/README.md` stays: it's the Lantern docs' own, part of the made-up repository. Each of the three projects checks clean.

## The three sessions

| Session | With | Without |
|---|---|---|
| With the plugin | The plugin, loaded from the checkout | `AGENTS.md`, `.agents/`, `.claude/` |
| With what `ascribe agents sync` wrote | The files `ascribe agents sync` writes, in the one commit | The plugin |
| With nothing | The copy alone | The plugin, and anything `ascribe agents sync` writes |

The third is the baseline: what an agent does with no help shows what the help is worth, and what it guesses Ascribe's commands are called.

With the plugin:

```shell
claude -p "Add a policy on laptops to the security handbook: disks are encrypted, screens lock after five minutes, and a lost laptop is reported within an hour." \
  --setting-sources project \
  --plugin-dir <checkout>/plugins/ascribe \
  --allowedTools "Read,Edit,Write,Glob,Grep,Bash,mcp__plugin_ascribe_ascribe__*" \
  --no-session-persistence --include-hook-events \
  --output-format stream-json --verbose \
  > <checkout>/project-docs/surface-audit/journeys/transcripts/edit-an-unfamiliar-project-plugin.jsonl
```

The other two, each in its own copy, with `sync` or `nothing` in the transcript's name:

```shell
claude -p "Add a policy on laptops to the security handbook: disks are encrypted, screens lock after five minutes, and a lost laptop is reported within an hour." \
  --setting-sources project \
  --allowedTools "Read,Edit,Write,Glob,Grep,Bash" \
  --no-session-persistence --include-hook-events \
  --output-format stream-json --verbose \
  > <checkout>/project-docs/surface-audit/journeys/transcripts/edit-an-unfamiliar-project-<setup>.jsonl
```

The flags are [phase 1's](fix-a-broken-link.md#what-changed-in-these-two-commands-and-why), flag for flag, and the reasons for each are there. `<checkout>/target/release` is first on the path, so `ascribe` is the release build of `main` (`ascribe 0.2.0`, with the `permalinks` commands). Standard error goes to a file in the scratch folder, and the notes say what was in it. The session is run once, with ten minutes at most.

The second session's files are what `ascribe agents sync` writes with no option, run once in each project: an `AGENTS.md` beside each `ascribe.toml`, a block for each project in the `AGENTS.md` at the repository's root, and the skill in `.agents/skills/ascribe/`. No `--target claude` is given, so there's no `CLAUDE.md` and nothing under `.claude/`. Whether Claude Code reads what the plain command writes is part of what the session shows.

## What the preparer checks afterwards

For each session:

- `ascribe check` in each of the three projects (`--config docs`, `--config handbook`, `--config handbook/pages/security`), with each exit code; `git status --short` and `git diff`.
- Where the page is, and its type: `ascribe outline <page> --config handbook/pages/security`.
- Its frontmatter, beside the two policies already there (`secrets.md`, `access-reviews.md`).
- Whether a reader can reach it: whether the security handbook's `index.md` links to it, and whether `ascribe check` gives the orphan advice, `ASC154`.
- Whether it states its rule in a `policy` note, as the other two do, and writes the project's phrases (`{team}`, `{channel}`, `{report}`) where it names what they stand for.
- `ascribe report inventory --config handbook/pages/security`, which counts policies by owner.
- `ascribe render <page> --build site --config handbook/pages/security`: the page as a reader sees it.

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

All three sessions: 10 October 2026, 9:57 PM, at the same time, each in its own copy. Claude Code 2.1.296, model `claude-opus-5-5`, started from zsh on macOS in the copy's root with `target/release` first on the path (`ascribe 0.2.0`). In each, the user's synced skills and the account's connectors (`claude.ai Claude Docs` connected, three more `needs-auth`) were listed, and none was used. Standard error was empty in each.

### With the plugin

- **When, with what:** as above, in the copy `edit-an-unfamiliar-project-plugin`, with the plugin from the checkout, `ascribe@inline` version 0.2.0. The session's first event listed the plugin's nine tools (`mcp__plugin_ascribe_ascribe__ascribe_check` and the eight others), `LSP`, the skill `ascribe:ascribe`, and the commands `ascribe:check`, `ascribe:new-page`, and `ascribe:review`; the server `plugin:ascribe:ascribe` was `connected`.

- **Transcript:** [`transcripts/edit-an-unfamiliar-project-plugin.jsonl`](transcripts/edit-an-unfamiliar-project-plugin.jsonl).

- **Steps:**

  1. `Skill`, `ascribe:new-page`, with the request as its argument: "policy on laptops in the security handbook: disks are encrypted, screens lock after five minutes, a lost laptop is reported within an hour". Came back: `Launching skill: ascribe:new-page`, then the command's text: "Run `ascribe agents prompt new-page --arg type=<type> --arg title=<title> [--arg path=<path>]`, and do what the prompt it prints asks."
  2. Bash: `git ls-files | head -200; echo ---; cat ascribe.toml`. Came back: `Exit code 1`, the thirty files, and `cat: ascribe.toml: No such file or directory`.
  3. Bash: `cd handbook/pages/security && cat ascribe.toml && echo --- && ascribe model 2>&1 | head -150 && echo --- && cat content/policies/secrets.md && echo --- && cat content/index.md`. Came back: the content model's file; the model as `ascribe model` prints it, with "`policy` (files `policies/**`): frontmatter `title` (string, required), `owner` (string, required), `review` (enum(quarterly, yearly), required), `effective` (date, required), `review-by` (date)"; a policy; and the index.
  4. Bash: `ascribe agents prompt new-page --arg type=policy --arg title="Laptops" --arg path=. 2>&1; echo ---; cat content/policies/access-reviews.md; echo ---; cat content/_fragments/report.md; echo ---; cat content/incident-response.md`. Came back, first, the prompt:

     ````text
     Write a new page of type `policy` titled "Laptops".

     Where: in the content root, `handbook/pages/security/content/`, at a path matching `policies/**` (relative to it).

     Project: handbook/pages/security/

     Start the file with this frontmatter, and fill in each `…`:

     ```yaml
     ---
     title: "Laptops"
     owner: …
     review: …
     effective: …
     ---
     ```
     ````

     then the type's five fields, "A file or folder whose name starts with `_` is a fragment, not a page: don't name the page that way.", and "When you're done, run `ascribe check handbook/pages/security/content` and fix what it reports." Then the three files.
  5. `Write`, `handbook/pages/security/content/policies/laptops.md`: frontmatter `title: Laptops`, `owner: security`, `review: yearly`, `effective: 2026-10-10`, `review-by: 2027-10-10`; a paragraph; `@note {type=policy}` with the three rules; three sections, each with an `@id:`; and `@include: /_fragments/report.md` at the end.
  6. `Edit`, `content/index.md`: `- [](policies/laptops.md)` added to the list under **Policies**, between the two there.
  7. Bash: `ascribe check content 2>&1; echo "exit $?"; ascribe render content/policies/laptops.md 2>&1 | head -60`. Came back: `checked 6 files, reported on 6: 0 errors, 0 warnings`, `exit 0`, and the page, with `> **Policy**` and, at the end, the fragment's text with its phrases filled in: "Email security@lantern.example or post in #security."
  8. Bash: `ascribe refs content/policies/laptops.md 2>&1 | head -20; git status --short`, described as "Confirm the index links to the new page". Came back: `content/index.md:17:3: link`, `1 place uses content/policies/laptops.md.`
  9. Answered: "The laptop policy is added to the security handbook at `handbook/pages/security/content/policies/laptops.md`, and it's linked from the handbook's Policies list in `content/index.md`. `ascribe check` passes with no errors or warnings; nothing is committed."
  10. The Stop hook ran.

- **Tried first:** the plugin's command, `ascribe:new-page`, before it had looked at a file. Then `cat ascribe.toml` at the root, which isn't there; the file list showed three. It went to `handbook/pages/security`, read the content model twice over (the file, then `ascribe model`), and ran the command the plugin's command names, with the type and title it had now worked out. It didn't load the skill `ascribe:ascribe`, call one of the plugin's tools, or use `LSP`. No `--help` was read.

- **What the hook or the plugin's tools said:** the hook said nothing: `"hook_name": "Stop"`, `"stdout": ""`, `"stderr": ""`, `"exit_code": 0`, `"outcome": "success"`. No tool of the plugin's was called. The page was written with `Write` and the index changed with `Edit`, the tools the language server's diagnostics follow, and nothing came back with either but "File created successfully" and "has been updated successfully". Both were clean, so there may have been nothing to say; the transcript can't show whether the language server looked.

- **Doubts:**
  - The prompt's last line told it to run `ascribe check handbook/pages/security/content`. It was standing in `handbook/pages/security`, and ran `ascribe check content` instead. It didn't say why.
  - What to put in the fields the request didn't give, listed in its answer: "**Owner:** `security`, matching the other two policies.", "**Review cycle:** `yearly`, with `review-by: 2027-10-10`.", "**Effective date:** today, 2026-10-10."
  - Whether a reader can get to the page: it checked with `ascribe refs`, "Confirm the index links to the new page".

- **Time:** 25 seconds by the clock; the session reports 24.1 seconds, ten turns.

- **Result:** added, and a policy as the project means one. The preparer's checks are with the other two sessions', below.

- **Seen on the way:**
  - The command `ascribe:new-page` was the agent's first move, and it's a wrapper round a command line: "Run `ascribe agents prompt new-page --arg type=<type> --arg title=<title> [--arg path=<path>]`". The agent couldn't run it until it knew the type, and to know the type it had to find the project and read its model, which is most of the job. The command's `argument-hint` is `<type> <title> [path]`, and the agent gave it the request's sentence.
  - The prompt the command prints names its check from the repository's root, to an agent standing in the project: "When you're done, run `ascribe check handbook/pages/security/content`". Checked afterwards by the preparer, from `handbook/pages/security`, where the agent ran the prompt: `error: handbook/pages/security/content doesn't exist`, exit 2. From the root it works. The agent ran `ascribe check content`. The same shape as the review prompt's `Project: docs/` in [journey 5](review-a-pull-request.md).
  - The prompt's frontmatter leaves out `review-by`, which the type lists as optional and both policies there have. The agent added it from the neighbors.
  - `ascribe check content` ends `checked 6 files, reported on 6: 0 errors, 0 warnings`, and `ascribe check` ends `checked 6 files: 0 errors, 0 warnings`. Two forms of the one line.
  - `cat ascribe.toml` at the root: `No such file or directory`. A repository with three projects has nothing at its root that says so, and the plugin says nothing about which project a request is for. The agent found them in `git ls-files`.
  - The hook ran at the root of a repository with three projects, after a change to one, and said nothing. Checked afterwards by the preparer: `ascribe agents hook claude-code --event stop` there exits 0 with no output, and `ascribe check` there exits 2, `error: no ascribe.toml found in … or any parent directory`.

### With what `ascribe agents sync` wrote

- **When, with what:** as above, in the copy `edit-an-unfamiliar-project-sync`, with no plugin. The copy's one commit holds what `ascribe agents sync` wrote: `AGENTS.md` at the root with a block for each project, an `AGENTS.md` beside each `ascribe.toml`, and `.agents/skills/ascribe/`. The session's first event listed no tool, skill, or command of Ascribe's: the skill in `.agents/skills/` isn't among its skills. It listed a plugin of Claude Code's own, `cc-plugin-agents-md@builtin`, as every session here did.

- **Transcript:** [`transcripts/edit-an-unfamiliar-project-sync.jsonl`](transcripts/edit-an-unfamiliar-project-sync.jsonl).

- **Steps:**

  1. `Read`, `handbook/pages/security/AGENTS.md`. Came back: the project's rules, with "`policy` (files `policies/**`): frontmatter `title` (string, required), `owner` (string, required), `review` (enum(quarterly, yearly), required), `effective` (date, required), `review-by` (date)"; the three phrases; "`@note {type=<type>}: <text>` … Here: types `note`, `tip`, `important`, `warning`, `caution`, `policy`"; and "Programmatic check: after editing a page, run `ascribe check <file> --format concise` and fix every error. Before you finish, run `ascribe check --config handbook/pages/security`."
  2. `Read`, `handbook/pages/security/ascribe.toml`.
  3. Bash: `cd …/handbook/pages/security && find . -type f | grep -v '^./.git/' | head -100`. The project's seven files.
  4. Bash: a loop that printed `policies/secrets.md`, `policies/access-reviews.md`, `index.md`, `_fragments/report.md`, and `incident-response.md`.
  5. `Write`, `handbook/pages/security/content/policies/laptops.md`: the same frontmatter as the first session's, to the day; a paragraph; `@note {type=policy}` with the request's sentence, "Disks are encrypted, screens lock after five minutes, and a lost laptop is reported within an hour."; three sections with `@id:` lines; and `@include: ../_fragments/report.md`.
  6. `Edit`, `content/index.md`: the same line in the same place.
  7. Bash, from the root: `ascribe check handbook/pages/security/content/policies/laptops.md --format concise; ascribe check handbook/pages/security/content/index.md --format concise; ascribe check --config handbook/pages/security`. Came back: `checked 6 files, reported on 1: 0 errors, 0 warnings`, twice, and `checked 6 files: 0 errors, 0 warnings`.
  8. Answered: "The laptop policy is added at `handbook/pages/security/content/policies/laptops.md` and linked from the Policies list on the security handbook's index page. `ascribe check --config handbook/pages/security` passes with 0 errors and 0 warnings."

- **Tried first:** the right project's `AGENTS.md`, as its first step, with no search before it. The transcript has no step that reads the root's `AGENTS.md`, whose third block says "## Ascribe documentation in `handbook/pages/security/`" and "This project's rules are in `handbook/pages/security/AGENTS.md`: read them before editing a page there." So the root's file reached the session before its first step, and the agent did what it says. It ran no `ascribe` command until the page was written, and then the three the rules give, as given. It didn't read `.agents/skills/ascribe/`, run `ascribe model`, or read a `--help`.

- **What the hook or the plugin's tools said:** nothing; the session had neither.

- **Doubts:** one, in its answer, on a date it chose: "The other two policies have review dates set to 2030, so change this if you want it to match." It had `review-by: 2027-10-10`, a year on, for a policy reviewed `yearly`.

- **Time:** 21 seconds by the clock; the session reports 19.8 seconds, eight turns.

- **Result:** added, and a policy as the project means one. The preparer's checks are below.

- **Seen on the way:**
  - Of what `ascribe agents sync` wrote, `AGENTS.md` was used and the skill wasn't. The session's first step is a `Read` of the one right file of three, which only the root's block could have told it; the skill in `.agents/skills/ascribe/` isn't in the session's list of skills and was never opened. `ascribe agents sync --help` says of `--target claude`: "the skill in .claude/skills/ascribe/". The plain command doesn't write that, and Claude Code didn't find the other.
  - The root's block did the routing the other two sessions did by hand. With the plugin, the agent ran `cat ascribe.toml` at the root and failed; with nothing, it searched. Here the first file it opened was the innermost project's rules.
  - The rules were enough to write the page without one command: the type's fields, the note type `policy`, the phrases, and the two checks were all in `AGENTS.md`, and the agent ran `ascribe` only to check.
  - It ran the rules' two checks to the letter, from the root, and they worked from there: `ascribe check <file> --format concise` for each file, then `ascribe check --config handbook/pages/security`. The rules' paths are written for the root, and the agent was at the root.

### With nothing

- **When, with what:** as above, in the copy `edit-an-unfamiliar-project-nothing`, with no plugin and nothing of `ascribe agents sync`'s. The session's first event listed no tool, skill, or command of Ascribe's.

- **Transcript:** [`transcripts/edit-an-unfamiliar-project-nothing.jsonl`](transcripts/edit-an-unfamiliar-project-nothing.jsonl).

- **Steps:**

  1. `Glob`, `**/*`. The copy's files, with `handbook/pages/security/ascribe.toml` and the two policies among them.
  2. `Grep`, `handbook|laptop|security`, ignoring case. Nine files.
  3. Bash: `cd handbook && for f in ascribe.toml pages/security/ascribe.toml pages/security/content/index.md pages/security/content/policies/*.md pages/security/content/_fragments/report.md pages/security/content/incident-response.md pages/index.md pages/_fragments/accounts.md; do echo "=== $f"; cat "$f"; done; echo; grep -rn -i "laptop\|security" pages/onboarding pages/on-call.md ../docs/README.md ../docs/ascribe.toml`. Came back: both content models and the pages. The handbook's begins "One of three Ascribe projects in examples/monorepo; see its README. The security team's handbook is a project of its own inside this one's content root (pages/security/)".
  4. Bash: `which ascribe; ls ~/Documents/GitHub/ascribe 2>/dev/null | head -30; cat ../docs/README.md | head -40`, described as "Look for the ascribe CLI and its docs". Came back: the binary's path; the top level of Ascribe's checkout; and the docs' README, with "Check every page, under both builds:" and `npx ascribe check`.
  5. `Write`, `handbook/pages/security/content/policies/laptops.md`: the same frontmatter again; a paragraph; `@note {type=policy}` with the three rules; three sections with `@id:` lines; and `@include: ../_fragments/report.md`.
  6. `Edit`, `content/index.md`: the same line in the same place.
  7. Bash: `cd pages/security && ascribe check 2>&1 | tail -30; git status --short`. Came back: `checked 6 files: 0 errors, 0 warnings`.
  8. Answered: "The laptop policy is now in the security handbook at `handbook/pages/security/content/policies/laptops.md`, and it is linked from the Policies list on the handbook's index page. `ascribe check` passes on the security project with 0 errors and 0 warnings; nothing is committed."

- **Tried first:** the files. It listed them, searched for three words, and read both handbooks' content models and every page of the security one. Everything it wrote, it had from those: the type's fields from `[types.policy.frontmatter]`, and the note, the `@id:` lines, the include, and the index's list from the two policies beside it. Its one Ascribe command was `ascribe check`, guessed right at the first try, from the project's folder, with no `--help`. The name could have come from the docs' README, which says `npx ascribe check`, or from nowhere.

  It listed Ascribe's own checkout (`ls ~/Documents/GitHub/ascribe`), which `which ascribe` had pointed at, and read nothing in it.

- **What the hook or the plugin's tools said:** nothing; the session had neither.

- **Doubts:** none about Ascribe. Its answer lists what it made up: "**Added rules:** "Don't turn encryption off" and "lock it yourself whenever you step away" are my additions."

- **Time:** 22 seconds by the clock; the session reports 21.5 seconds, eight turns.

- **Result:** added, and a policy as the project means one. The preparer's checks are below.

- **Seen on the way:**
  - With no help, the agent wrote a page that the other two sessions' pages match nearly line for line. What told it how was the project: a content model whose comments say what each part is for ("Policies record who owns them, how often they're reviewed, and since when they apply."), and two pages of the same type to copy.
  - The baseline guessed one command, `ascribe check`, and guessed it right. It guessed nothing wrongly because it asked Ascribe nothing else: no `model`, no `outline`, no `render`, no `--help`.
  - It ran the check from the project's folder, after `cd pages/security`. At the root it would have had `error: no ascribe.toml found`; it never tried.
  - The copy's content models name the example: "One of three Ascribe projects in examples/monorepo; see its README." The README was left out of the copy, and the comment that points at it wasn't. For a later run, that line is the setup's to take out.
  - `which ascribe` led to a listing of Ascribe's checkout here too, as in [journey 6](set-up-ci-checks.md), and no further.

## What the preparer's checks said

The same for all three sessions, run in each copy:

- `ascribe check --config docs`: exit 0, `checked 10 files: 0 errors, 0 warnings`. `--config handbook`: exit 0, `checked 7 files: 0 errors, 0 warnings`. `--config handbook/pages/security`: exit 0, `checked 6 files: 0 errors, 0 warnings`, one file more than before. No `ASC154`.
- `git status --short -uall`: ` M handbook/pages/security/content/index.md` and `?? handbook/pages/security/content/policies/laptops.md`, and nothing else. `git diff`: one line, `- [](policies/laptops.md)`, between the other two policies in the index.
- `ascribe outline policies/laptops.md --config handbook/pages/security`: `Laptops`, then `content/policies/laptops.md, type policy`, and three headings with ids.
- The frontmatter, the same in all three: `title: Laptops`, `owner: security`, `review: yearly`, `effective: 2026-10-10`, `review-by: 2027-10-10`. The two policies already there have `owner: security`, `review` of `yearly` and `quarterly`, and a `review-by` in 2030.
- `ascribe refs policies/laptops.md --config handbook/pages/security`: `content/index.md:17:3: link`, `1 place uses policies/laptops.md.`
- Each page states its rule in `@note {type=policy}`, as the other two do. None writes a phrase itself, and none needed to: each gets `{report}`, `{channel}`, and `{team}` by including `_fragments/report.md`, and none names the team, its channel, or its address in its own words.
- `ascribe report inventory --config handbook/pages/security`: `5 pages.`, `By type: policy 3, page 2.`, `By owner: security 3, no owner 2.`, `Pages nothing links to: none.`
- `ascribe render policies/laptops.md --build site --config handbook/pages/security`: the page, ending with the fragment's text, "Email security@lantern.example or post in #security."

Where they differ: the include is written `/_fragments/report.md` with the plugin and `../_fragments/report.md` in the other two, and both resolve; the last heading is "If a laptop is lost" (`#if-a-laptop-is-lost`) with the plugin and with nothing, and "A lost or stolen laptop" (`#lost-or-stolen`) with `sync`'s files; the sentences differ.

## What the sessions had in common

- **The same page, three times.** Same folder, same file name, same five lines of frontmatter, the rule in a `policy` note, the report fragment included, the same line added to the index. By every check the preparer ran, the three results are equally right.
- **The help changed the route, not the result or the time.** 25 seconds with the plugin, 21 with `sync`'s files, 22 with nothing. The session with the most help took the most steps: the plugin's command sent it to a prompt it needed the content model to ask for.
- **Each found the project a different way.** With the plugin: `cat ascribe.toml` at the root, a failure, then the file list. With `sync`'s files: the root's `AGENTS.md`, which names the project's own, opened first. With nothing: `Glob` and `Grep`.
- **Each learned the type a different way.** With the plugin: `ascribe model`, then the `new-page` prompt, which repeated it. With `sync`'s files: `AGENTS.md`. With nothing: `ascribe.toml` itself. All three then read the two policies there and wrote one like them.
- **All three checked, each in its own form.** `ascribe check content` from the project; `ascribe check <file> --format concise` twice and `ascribe check --config handbook/pages/security` from the root; `ascribe check` from the project. No one checked the other two projects, and none needed to.
- **What was used of each kind of help:** of the plugin, the command `ascribe:new-page` and through it `ascribe agents prompt`; not the skill, the tools, or the language server, and the hook was silent. Of `sync`'s files, `AGENTS.md`; not the skill. The baseline used `ascribe check` and nothing else of Ascribe's.
- **What the task didn't test.** The project was small and well commented, with two pages of the type to copy. The baseline shows that this much can be read off the files; it doesn't show what happens where there's no neighbor to copy.

## Preparing a copy

For the preparer. For the first and third sessions:

```shell
cp -R <checkout>/examples/monorepo <scratch>/edit-an-unfamiliar-project-<setup>
cd <scratch>/edit-an-unfamiliar-project-<setup>
mv README.md ../removed-README.md
git init -q -b main && git add -A && git -c user.name=prep -c user.email=prep@example.com commit -qm "Lantern"
```

For the second, `ascribe agents sync` runs before the commit, and after `git init`, since the command writes to the repository's root and finds it from git:

```shell
cp -R <checkout>/examples/monorepo <scratch>/edit-an-unfamiliar-project-sync
cd <scratch>/edit-an-unfamiliar-project-sync
mv README.md ../removed-README.md
git init -q -b main
(cd docs && ascribe agents sync)
(cd handbook && ascribe agents sync)
(cd handbook/pages/security && ascribe agents sync)
git add -A && git -c user.name=prep -c user.email=prep@example.com commit -qm "Lantern"
```

It's run three times because it's run in a project: at the repository's root it answers `error: no ascribe.toml found in … or any parent directory; run ascribe from a project, or pass --config` and exits 2. The first run writes `AGENTS.md`, `../AGENTS.md`, `../.agents/skills/ascribe/SKILL.md`, and `../.agents/skills/ascribe/references/directives.md`; the other two write their own `AGENTS.md`, add their block to the root's, and say `unchanged` for the skill.

In every copy, each project checks clean before the session: `ascribe check` says `checked 10 files: 0 errors, 0 warnings` in `docs`, `checked 7 files` in `handbook`, and `checked 5 files` in `handbook/pages/security`.
