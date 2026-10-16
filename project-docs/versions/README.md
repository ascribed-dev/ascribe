# Versions

A project says which versions of its product exist, and Ascribe does the rest in steps a project opts into one at a time: checks against the list, a site for each live version from one source, and a frozen copy of a version that's left. The idea, and the research behind it, are in the [proposal](../proposals/versions.md). This is the plan for building it.

## Who it's for

- **A team with one site,** which keeps "not released yet" by hand today: a lifecycle state, a feature that points at it, and a step after each release. Ascribe's own docs are this team.
- **A team that supports more than one version,** which today publishes one site and badges, or filters a whole build for one target and loses every page about anything else.
- **A reader on an old version,** who today has no way to tell, and no way across to the same page in the version they run.
- **Whoever retires a version,** which elsewhere is a runbook of scripts and clean-up by hand.

## The four levels

1. **No versions.** The default. Nothing to configure, and nothing changes.
2. **One site that knows its versions.** `[releases]` in `ascribe.toml`. One site still, with checks, and "planned" computed instead of kept.
3. **A window of live versions.** `versions = "<target>"` on a build. One output for each live line.
4. **An archive.** A line that's left the window, rendered once more by Ascribe alone and frozen.

## What exists today

- **`@available` says where content applies,** with states and histories, checked against the content model (SPEC §4.4). A build badges everything or filters for one target at one version (SPEC §9.3).
- **`[versions]` has `scheme` and `current`.** `current` is one version for every target, added after the proposal was written (decision 67 in `project-docs/decisions.md`), and the `availability-left-behind` check reads it. It's in `main` and in no release. The proposal says `[versions]` "keeps the comparison scheme and nothing else", so the two disagreed; decision 24 removes it, in phase 3.
- **Ascribe's docs declare a lifecycle state, `unreleased`, and a feature, `next`,** that points at it (`docs/ascribe.toml`), and so does the site's own project, held equal by `scripts/docs-site/facts.test.ts`. `RELEASING.md` has the step that rewrites the marks after a release.
- **A build writes its page records,** `_ascribe/pages.json` in the site output: each page's title, description, canonical address, and listing, with the project's `robots.txt` lines. `@ascribed/astro` writes the head (`Head.astro`), one sitemap, and `robots.txt` from them. The [search and answers plan](https://github.com/ascribed-dev/ascribe/blob/5b22286fd2f370e796219e68a7c9d7cdb2ba8acc/project-docs/search-and-answers/README.md) built it and left the place for versions: its decision 19 says a canonical link "becomes another page's address only with versions", and its listing levels leave room for a version between the site and a type.
- **`[listing]` sets how pages are listed** for the site, a folder, or a type, and a narrower rule can only restrict.
- **The outputs for agents exist:** `llms.txt`, each page's Markdown at its address, and `ascribe agents index`. The proposal says Ascribe has no `llms.txt`; it does now.
- **`@ascribed/astro` takes a list of named projects,** each at a base path under Astro's `base` (decisions 81 and 82). Decision 82 named the entries because this plan would want more than one output from one project.
- **Nothing is shared between builds,** and nothing Ascribe writes is above them. The output layout contract says the first (`docs/content/contracts/output-layout.md` §1), and the store's ownership rules work for each build, through that build's manifest (`crates/ascribe-emit/src/store.rs`).
- **A collection is Astro's `glob` loader over one site output** (`packages/astro/src/content.ts`), and the integration mounts one forwarding route for each permalink and refuses two lists that share a route (`packages/astro/src/index.ts`).
- **The binary runs git for `diff`, `drift`, and sources only** (`crates/ascribe-diff/src/git.rs`).
- **One check depends on the day,** `review-overdue`, and `ASCRIBE_TODAY=YYYY-MM-DD` sets the day it runs as.
- **A build's name** is a letter, then letters, digits, `_`, `-`, or `.`. `@` isn't allowed.
- **`examples/quill`** has a versioned target, `self-managed`, a versionless one, `cloud`, and a build filtered for `self-managed 3.3`.

## Decisions

These are settled. Don't reopen them in a phase; if one can't be met, stop and report.

From the proposal:

1. **Four levels, each opted into separately.** A project with no `[releases]` works exactly as it does now.
2. **The list is `[releases.<target>]`:** `live`, `planned`, `ends`, and `archived`. Versions belong to a target, so the list does.
3. **A version in the list is a release line.** `3.4` stands for every `3.4.x`.
4. **The current line is the newest live one.** It isn't written down, so it can't disagree with the list.
5. **"Planned" is computed, and isn't a lifecycle state.** A line newer than current is planned. The availability element, the plain output's availability line, and the JSON output each say so.
6. **A feature key stays the way to mark unreleased work,** since an author often doesn't know its number.
7. **A versioned build is a third thing, beside `badge` and `filter`:** `versions = "<target>"` gives one output for each live line. Content that names the target and isn't in a line is removed from it; everything else is kept and marked as a badge build does. The line is taken whole. The current line keeps planned content; older lines don't show what came after them.
8. **A build is versioned by one target.** Another product's marks stay badges in it.
9. **The current line keeps the build's name, and each other line is `<build>@<line>`:** `site`, and `site@3.4`.
10. **The current line is at the root, and every other live line is under its number, after the base path.** Turning versions on moves no address.
11. **One file above the builds' outputs says what every line knows about the others.** It's the one exception to "nothing is shared between builds", and a contract.
12. **A link to a page that isn't in a line resolves in order:** the page; the page that replaces it, or that it replaces; then the page in the nearest line that has it, as a link out, marked. It's an error only when no live line publishes the target.
13. **Two files never publish to one address, and a page says which page it replaces** with `replaces`, a reserved frontmatter key.
14. **What changed between two lines is read from the marks alone,** with nothing built.
15. **An archive is a last render by Ascribe alone:** plain HTML that depends on nothing, with a way around, search, a statement of what it is, marks that keep crawlers out, and a print stylesheet. It doesn't go in git, and Ascribe doesn't host it.
16. **Archive first, then clean up.** `ascribe versions retire` refuses a line with no address.
17. **Retiring rewrites a mark to the simplest spec that means the same in every live line,** removes a mark only when what's left says nothing, and offers a block in no live line for deletion without deleting it.
18. **A major rewrite isn't a version.** An archived entry can be any address, including a site Ascribe didn't build.
19. **Ascribe sets no limit on the window.**
20. **A release is a command, `ascribe versions release "<target> <line>"`.** It moves the line from `planned` to `live`, rewrites every mark that names a feature pointing at the line to the line's spec, then points the feature at the following planned line, in an order that changes no output. Without a following planned line it needs `--next <line>`, and exits 2 without it.
21. **Redirects aren't written.** `retire` reports each old address and where it should go, in its text and as a list in its JSON shaped like the headers list the search and answers plan writes.
22. **A version between two declared lines is advice, not an error,** with two fixes: declare it, or correct the number. An `archived` entry may omit its address to declare a line that's readable nowhere. A version newer than every declared line is an error.
23. **`numeric` stays the only version scheme.**

From [phase 1](phase-1-decisions.md), where each has its reason, taken by the maintainer on [#494](https://github.com/ascribed-dev/ascribe/issues/494#issuecomment-6195368949), 15 October 2026:

24. **`[versions] current` is removed; the list replaces it.** `[versions]` keeps `scheme` alone. `availability-left-behind` compares each target's versions with that target's current line, and reports nothing for a target with no `[releases]`. A project that sets `current` gets an error that names `[releases]`. It amends decision 67 ([decision #494](../decisions/494-versions-current-removed.md)).
25. **A line is the number as the list writes it, and a version is in the line it starts with.** `3.4` holds `3.4`, `3.4.2`, and `3.4.2.1`, and `live = ["0.2.0"]` is a line too. Every line of one target has the same number of parts; a list that mixes them is an error. A version shorter than the lines starts none, and is in whichever lines follow it by order.
26. **The phrase for a target's line is one a project declares:** `[phrases]` `version = { line = "ascribe" }`. In a line's output it's that line; in a build that isn't versioned it's the target's current line. Nothing is reserved.
27. **"Planned" is written the same in three outputs.** The availability element's target entry has a `planned` attribute beside `states` and `versions`, and its text reads "Self-managed (planned, 3.6)"; the plain output's availability line has the same text; the JSON output has `planned: true` on the target, absent otherwise. The label is `[releases] planned-label`, for the project.
28. **The file the lines share is `<output-dir>/_ascribe/versions.json`,** owned through a manifest of the folder's own, `<output-dir>/_ascribe/manifest.json`, in a build's manifest format. Every `ascribe build` of a project with `[releases]` writes it, for every versioned build; the next build of a project without the table removes it, with the manifest and the folder when nothing else is in it. Its top is `version` and `builds`, keyed by a versioned build's name; a build's entry has `target`, `lines` newest first, and `pages`; a line has `line`, `status` (`current`, `live`, `planned`, or `archived`), `build` (absent for a line with no output), and `base`; a page is keyed by its source path from the content root, with `lines` giving its route in each line that publishes it. Later phases add keys and change none.
29. **A problem in more than one line is said once, with the builds it's in,** as a problem in two builds is said today: a line is the build `site@3.4`, the text says "in every line of `site`" when that's so, and `builds_checked` lists each line. `check`'s JSON gains no `lines` field.
30. **`@ascribed/astro` gives one collection with every line's pages.** An entry's id is its route under the base path (`install`, `3.4/install`), and its data carries `line` and `current`. The integration reads each line's output with a loader of its own, and `@ascribed/astro/Versions.astro` lists a page's lines. A project's entry in `projects` is unchanged, and the check that Astro's `base` holds the project's base path allows for the number.
31. **The two commands are `ascribe versions list` and `ascribe versions changes "<target> <from>" <to>`.** Each takes `--format text|json`, names a line as a mark does, and exits 0, or 2 for a line that isn't declared.
32. **The rule for indexing older lines is `[listing] older-lines`,** a key at the table's top beside `default`, `folders`, and `types`, and a level between the site's `default` and a folder or a type. It restricts and never loosens, and governs search engines and agents together.
33. **When no build is versioned by the target, `release` moves the line before it to `archived`, with no address.** A one-site project keeps one live line, and no mark is rewritten. When a build is versioned by the target, the old line stays live, and `retire` is the way out. `release` also takes a line nobody planned, newer than the current line and older than every planned one.
34. **A line's output leaves out a version at or before the oldest live line; every other build shows it.** The mark keeps its state: `self-managed deprecated 3.2` shows "deprecated". So a mark does nothing only when every build that publishes its content is versioned by its target or filtered for it.

The last two are what Ascribe's own docs meet first: a project with one site has no archive to give `retire`, and every mark it has ever written is older than its one live line.

From [phase 2](phase-2-thread.md), made building it, for the maintainer to confirm in its pull request:

35. **An older line's output says nothing to agents yet.** Its model has `[consumer] agents` off, so its pages carry no pointer to `llms.txt` or a Markdown version, its page records no `markdown`, and its plain output no `llms.txt`; the pages' Markdown is still written. Phase 5 gives each line its own. _Why:_ the integration publishes the current line's alone for now, so an older line pointing at them would link to addresses that don't exist, and every link in its pages has to land. Phase 5 did: decision 57.
36. **The site's list of pages is checked against every line's pages, and the sidebar shows one line's.** `sidebar(nav, entries, { entry })` shows the listed pages that the entry's line has, the current line's without `entry`; a page in any line must be listed, and a listed file must be a page of some line. _Why:_ a page an older line has and the current one doesn't (or the other way round) is still a page of the site, and one list keeps the sidebar the same across lines.
37. **`Versions.astro` renders nothing for a page only its own line has.** _Why:_ a list of one is no way across, and a project with one live line then renders as it did without versions. Phase 5 changes it: decision 55.
38. **The integration builds an older line in a second `ascribe build`, after the first has written `versions.json`.** It says only what that run built: its errors still fail the build, and `ascribe check` reports every line's warnings. _Why:_ the lines aren't known before the current line's build writes the file, and naming a line that has left the list would fail the run.

From [phase 4](phase-4-links.md), made building it, for the maintainer to confirm in its pull request:

39. **A link out of a line is marked with `data-ascribe-release-line`,** an attribute marker on the link holding the line as `[releases.<target>]` writes it; the library's stylesheet shows it after the link as " (3.5)", the plain output writes " (3.5)" after the link, and the JSON output's page target carries `line`. Nothing marks the paragraph. _Why:_ `data-ascribe-line` is already the source-line attribute the parser writes, and a link is the smallest thing to mark.
40. **"Nearest" is the closest newer line first, then the closest older.** _Why:_ a reader is sent forward rather than back when they can be, and the current line never links back when a newer one has the page.
41. **A `replaces` cycle is an error in every build; the page it replaces still published, or another page replacing the same one, is an error in each line where it happens.** `replaces` names a page by its file path, with no `#id`, and a fragment can't have one. _Why:_ a cycle is wrong whatever the build, while what a line publishes is a line's own question, and a problem in more than one line is reported once anyway.
42. **A link that lands on another page keeps its `#id` when that page has the heading there, and otherwise drops it with advice,** `link-fragment-dropped`. _Why:_ a page rewritten under a new name often keeps a section's id, and losing the reader's place isn't worth failing a build for.
43. **`replaces` isn't in the site output's frontmatter, and `versions.json` says it:** a page's `replaces` and `replaced-by`, each only when the other page is in the file; `replaced-by` is the first by path when more than one page replaces it. `Versions.astro` lists the lines of the pages these lead to with the page's own. _Why:_ the site needs the pair, not each page's key, and the file already holds every line's pages.
44. **`ascribe link` says where a link goes in each line, under `lines`; `ascribe render` shows the plain output's " (3.5)" and nothing more.** _Why:_ `render` shows one build's output, and `link` is the question about a link.
From [phase 7](phase-7-release.md), made building it, for the maintainer to confirm in its pull request:

45. **`release` compares the site and plain outputs, not the JSON output.** Rewriting the marks and archiving the old line must change neither; what the release itself changes is listed by build. _Why:_ the JSON output records each mark as written and where it's written, so it changes with every rewrite, however right.
46. **An archived line `release` writes is `{}`, or `{ ended = <date> }` when `ends` gave its last day,** and the line's entry in `ends` goes. _Why:_ `ends` is the only day the project has written down; an address is for whoever publishes the archive to add.
47. **`release` writes only files git can put back:** each file it would write must be tracked and unchanged (`git status`), or it refuses and names them; `--force` writes anyway, and outside a repository it needs `--force`. `--dry-run` needs neither. _Why:_ then the release is one commit to read, and `git checkout` undoes it.
48. **`changes` holds a line to what's marked for a version in it.** The current line doesn't hold what's marked for a later one here, though its output shows it. _Why:_ two lines are compared as they were released, whichever is current, so release notes don't change when a planned feature lands. `changes` also takes `--format markdown`, the draft release notes the phase's design names, beside decision 31's two.
49. **Ascribe releases its own docs with the command, in the release's commit,** not in a follow-up pull request: [RELEASING.md](../../RELEASING.md) step 2. _Why:_ the line moves in that commit already, and a mark a later pull request adds is written `next` and stays so, which the follow-up had to sort out by hand.

From [phase 5](phase-5-readers.md), made building it, for the maintainer to confirm in its pull request:

50. **A page looser than `[listing] older-lines` is its own diagnostic, `listing-loosened-in-line`, at the page level, per build,** not `listing-loosened`, which is at the file level. Its next step is `write`, with no fix offered. _Why:_ whether an older line publishes a page is known per build, and the registry gives each diagnostic one level; the editor checks one build, the current line, so it can't offer a fix it never sees the problem for.
51. **`older-lines` is a level of `listing_from`,** between the default and a folder's or a type's rule, so a record says which rule made a page `unlisted`. _Why:_ a reader of the records asks why a page isn't listed, and the answer is the rule.
52. **The sitemap compares a record's `canonical` with its own address; the records gain no key.** _Why:_ the canonical address already says whether a page is the one to index, and a second key could disagree with it.
53. **`versions.json` gains `label`, `lines[].ends`, and the archived lines** (`status: "archived"`, with `url` and `ended` when the project gives them, and no `build` or `base`), after the live ones. _Why:_ a banner and a list of versions need these, and a site that isn't Astro reads the same file; the archived lines are the ones phase 8 lists, and their addresses are already in `ascribe.toml`.
54. **A page's Markdown opens with "> Documentation for Self-managed 3.4.", the day support ends, and on an older line "The current version is 3.5: <address>."** The address is the page's canonical address when that isn't its own, and the current line's home otherwise. A build with one live line writes none of it, nor the line in `llms.txt`. _Why:_ an agent reads one page with no head to read, and a project with one live line builds what it built before versions.
55. **`Versions.astro` renders when there's more than one line to list,** a live line without the page shown unlinked and an archived line with an address linked to it, each followed by words given as properties. This replaces decision 37. _Why:_ the plan asks for the lines without the page; a page only one line has now lists the others it isn't in, and a project with one live line still renders nothing.
56. **The banner's link is the page's route in the current line from `versions.json`,** or the page that takes its place there, and the current line's home from the integration's routes otherwise. _Why:_ it's the route `Versions.astro` already links, relative to the site, and works without `site`, where a record has no canonical address.
57. **Each line publishes its own outputs for agents.** An older line's model keeps `[consumer] agents`, its `llms.txt` heading is `# <name> (<label> <line>)`, and an older line's opens, after the summary, with "This is an older version. The current version is 3.5: [llms.txt](<address>)." `ascribe agents index` and `ascribe render` without `--build` take a versioned build's current line. _Why:_ the summary blockquote is the page's description, so the pointer is a paragraph of its own; and naming the declared build is naming its current line everywhere else.

From [phase 6](phase-6-surfaces.md), made building it, for the maintainer to confirm in its pull request:

60. **A page the lines share is still resolved once per line.** Measured on a project of 3,000 pages with three live lines, a build took 4.2 s against 1.6 s with one; resolving is about 0.2 s of each line's 0.9 s, and the rest is emitting and writing each line's own files. Resolving once would save at most a sixth of a three-line build, and needs every link routed again per line. _Why:_ the phase said to keep the simpler code when the saving is small, and `check`'s count of a problem once (decision 29) needs no shared resolve.
61. **The editor checks every line of a versioned `[editor] build`,** the editor's line first, and a problem only some lines have ends "(only in build `site@3.4`)". Diagnostics don't follow the line chosen in the lens. _Why:_ the server doesn't know what a client chose, and the Problems panel would change with a picker; every line is what `ascribe check` reports, so the editor agrees with CI.
62. **`ascribe/targets` and `ascribe/preview` list a versioned build's lines under it,** as `lines`, each with `name`, `line`, and `current`, the current line first; a build that isn't versioned has none. A client's picker offers each line as a build, by its name. _Why:_ a line is already a build everywhere a name is taken, so the lens, the preview, and `[editor] build` need nothing new.
63. **A line's page is rendered under the line's folder wherever it's drawn:** `ascribe render`, the preview, and the review report give `/3.4/install/`, with the line's links. _Why:_ an older line's route without its folder is an address no site has.
64. **`ascribe diff`'s JSON keeps a line as a build of its own,** its pages' routes under its folder; its text lists a versioned build's lines together, each changed page once, with "in every line" or the lines it changed in; the report's data gives each build its `line`, and the report lists the page once with a switch between its lines. _Why:_ the JSON says what each output publishes, which a reader of it compares line by line, while a person reads a page once.
65. **`ascribe model` has a `releases` section,** before `builds`: each target's lines, newest first, with `status` and the builds that are each, and a versioned build gains `versions` and `line`. _Why:_ the agent that writes a mark needs the lines it can name, and `ascribe versions list` is about addresses and pages, not the model.
66. **A hover on a version in a mark says the line it's in and its status,** and nothing for a version older than every declared line; **a hover on `replaces` says which page each line publishes,** and Go to Definition opens the replaced page. _Why:_ a version before every line is in none and is shown as the target alone (decision 34), which a hover on every old mark would only repeat.
67. **Review in `astro dev` compares every line, and a page shows its own line's changes.** _Why:_ the site serves every line, so a reviewer who opens `/3.4/install/` expects that line's marks.

## The thread

**The use case:** `examples/quill` declares `live = ["3.4", "3.5"]` for `self-managed` and sets `versions = "self-managed"` on its site build. `ascribe build` writes two outputs. A page marked `self-managed 3.5` is in one and not the other, the pages about the cloud service are in both, and the Astro example serves the older line under `/3.4/`, where a list on each page links to the same page in the other line.

**The layers it crosses:**

| Layer | Where | What the thread needs from it |
|---|---|---|
| The content model | `ascribe-model` | `[releases.<target>] live`, and `versions` on a build |
| Resolve | `ascribe-resolve`, `src/build/` | One build resolved once for each line, with the line taken whole, and each line's routes under its own base path |
| Check | `ascribe-check` | Each line checked, and a problem said once |
| Emit | `ascribe-emit` | An output for each line, and the one file above them, with an owner |
| The command | `ascribe-cli` | `ascribe build` writing every line, and `--build site@3.4` naming one |
| Astro | `@ascribed/astro` | A loader of its own that reads every line's output into one collection, and a list of a page's lines |
| The docs | `docs/content/` | The reference, the output layout contract, and the start of a guide |

**The seam it tests** is one build becoming several outputs that know about each other. Three assumptions meet there. One is that a line's output can be written with its own base path, `/docs/3.4/`, by the code that today takes one base path for a project, so that every link, every asset, and every absolute address in that output is right. Another is that one site generator can serve every line from one entry in its configuration, which Astro's own loader can't do: the lines are separate folders, and a collection reads one. The third is that a file above the outputs can say, for each page, which lines have it, cheaply enough to write on every build. If the first or second fails, the addresses the proposal settled (the current line at the root, the others under their number, in one site) have to give way to a site for each line, and the switcher becomes a link between sites. If the third fails, each line has to read the others' outputs, and builds stop being independent.

**Left out, and where it comes back:**

| Left out | Phase |
|---|---|
| `planned`, `ends`, `archived`, every check the list makes possible, the phrase, and Ascribe's own docs | 3 |
| A link to a page that isn't in a line, and `replaces` | 4 |
| The banner, canonical links, the rule for indexing an older line, the sitemap, and the outputs for agents | 5 |
| Resolving a page once for every line, the editor, review, and the tools | 6 |
| `ascribe versions release`, `list`, and `changes` | 7 |
| The archive | 8 |
| Retiring a line | 9 |
| Search inside an archive | 10 |

After phase 4, a link to a page that isn't in a line lands on the page that replaces it or in another line (decision 12); an older line's pages carry their own address as canonical; and an older line has no `llms.txt` and its pages point to neither it nor their Markdown (decision 35). The reference says each while it's true.

## Phases

| Phase | Result |
|---|---|
| [1: Decisions](phase-1-decisions.md) | The maintainer settles the eleven contracts the phases ship. |
| [2: The thread](phase-2-thread.md) | A project with two live lines builds an output for each, and one site serves both, each page listing the lines it's in. |
| [3: A site that knows its versions](phase-3-list.md) | With `[releases]` alone, a mistyped version is an error, "planned" is computed, a mark that does nothing is found, and Ascribe's own docs stop keeping `unreleased` by hand. |
| [4: Links between lines](phase-4-links.md) | A page added or rewritten inside the window costs no marks on the pages that link to it. |
| [5: What a reader and a crawler are told](phase-5-readers.md) | An older line says what it is and points at the current one, to a person, a search engine, and an agent. |
| [6: Every surface](phase-6-surfaces.md) | The editor, review, and the tools show a page once, with the lines it differs in, and a build does a page's work once. |
| [7: A release, and what changed](phase-7-release.md) | Release day is one command, and what differs between two lines is one report. |
| [8: The archive](phase-8-archive.md) | A line is rendered into a folder of HTML that needs nothing else, and can be found from the live site. |
| [9: Retiring a line](phase-9-retire.md) | A line leaves the window, and every mark it made dead is rewritten or offered for removal. |
| [10: Search in an archive](phase-10-archive-search.md) | An archive opened from a folder can be searched. |

Phase 1 comes first because phase 2 ships two keys, a file's shape, build names, and an API of `@ascribed/astro`. After the thread the order is by risk, then by value. Phase 3 changes three output contracts and is what Ascribe's own docs wait for. It needs only the list, which is phase 2's first commit, so it can be built beside the rest of the thread and needn't wait for it. Phase 4 changes how a link resolves, which is the last thing that could change the shared file. Phases 5 and 6 widen who sees the lines. Phase 7 is the upkeep. Phases 8 to 10 are the fourth level, which nothing before them needs, and phase 8 is a thread of its own: its seam is Ascribe's renderer drawing a whole site, which it has never done.

Each phase's result is something a person can run or open. None builds a layer for a later phase to use.

## Carved out, and waiting

- **A general selector.** A reader choosing "Cloud" or "Self-managed 3.5" is choosing between a target with no versions and a line of another, and that's one case of choosing a platform, an environment, or a language across a whole site. This plan's list of lines is for one target, and is built so a general selector can take it over.
- **Redirects.** Decision 21. They wait for the record of what was published, which the search and answers plan carved out.
- **Navigation.** The content model has none, so the archive writes its own from folders, titles, and headings (phase 8). When the model gains it, the archive uses it.
- **Tested examples.** Saying an example was tested on a version the page doesn't claim waits for [tested examples](../proposals/tested-examples.md).
- **A planned line at an address of its own,** for a release candidate's docs.

## The other plans

- **Search and answers.** Phase 5 here writes into its page records and its `Head.astro`, and its decisions bind what's added: a record's keys are added and none change, and one setting governs search engines and agents together.
- **The surface audit.** Its conventions name the commands, their `--format` words, and their exit codes (`project-docs/checklists.md`, "A new command or option"). Its finding 30 is phase 7's reason.
- **Page templates.** Its phase 5 plans a command, `ascribe init`, that writes a project's first `ascribe.toml`. It isn't built. Whether it offers `[releases]` is that plan's to say, once it exists and this one's phase 3 has shipped.
- **Decisions.** [#381](https://github.com/ascribed-dev/ascribe/issues/381) changes where a decision is recorded. Each phase records its decisions where the repository does at the time.

## Rules for every phase

- Branch before committing; never commit to `main`.
- Read the current code before the phase file's pointers. If the phase file and the code disagree, or a decision above can't be met, stop and report instead of choosing silently.
- **A project with no `[releases]` changes in nothing.** Every phase runs `node scripts/compare/outputs.ts --base main` and shows that the projects without the table are byte for byte the same. A project that gains the table is named with `--accept`, file by file.
- **The language and the contracts change only by a recorded decision.** Go through the matching list in `project-docs/checklists.md` first, and record the decision in the same pull request.
- **A new diagnostic** is added to `tests/conformance/diagnostics.toml` and `crates/ascribe-core/src/diagnostics.rs`, in the same order, with a conformance case. A check about quality is advice, configurable, with its kind of next step.
- **Two renderers must agree.** A change to the site output's markup changes the fixtures in `tests/render/`, and both `render_site_html` and the Astro plugin pass them.
- **Project files are read through `ascribe_resolve::FileSystem`.** Libraries don't print, and don't panic on input.
- **A change about paths runs the full matrix** from its branch. A line's number is a folder's name and part of a build's name, and `site@3.4` has to work on Windows.
- **The map of the surface** is regenerated when a phase adds to it (`node scripts/surface/inventory.ts`), and its diff is read against the conventions.
- **User-visible changes** update the page under `docs/content/` and add an entry in `changes/` in the same phase.
- No phase history in code or docs. Describe what the code does now.
- Before finishing a phase, all of these pass:

  ```sh
  cargo fmt --all --check
  cargo clippy --workspace --all-targets --locked -- -D warnings
  cargo test --workspace --locked
  pnpm format:check && pnpm lint && pnpm typecheck && pnpm test
  ```

## Later, not in this plan

- A copy of the source for each version, in folders or branches.
- Hosting an archive, and a PDF engine.
- A second version scheme.
- Freezing a team's real site. The archive is Ascribe's own render.
- Versioning one build by two products at once.
- Ascribe's own docs publishing more than one line. They stay one site following `main` (decision 19 in `project-docs/decisions.md`).
