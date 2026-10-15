# Phase 1: Decisions

Part of [Versions](README.md). Needs nothing. No code: a pull request that records what the maintainer decides.

## Goal

Eleven choices are made before anything ships, because each goes into a contract: keys in `ascribe.toml`, three outputs, a file's shape, a report's JSON, an API of `@ascribed/astro`, and commands' names.

This phase can't be folded into phase 2. A thread may narrow what it handles. It may not ship a shape the plan expects to change.

## Context

- The [proposal](../proposals/versions.md), whole, and its "Settled" section.
- `project-docs/decisions.md`: decision 67 (`[versions] current`), 81 and 82 (the Astro integration's projects), 84 and 87 (the site's own project).
- `docs/content/content-model/index.md`: §1.2 (names), §7 (`[versions]`), §11 (`[phrases]`), §16 (`[builds.<name>]`), §22 (`[listing]`).
- `packages/elements/CONTRACT.md`: the availability element's attributes and its text.
- `docs/content/contracts/output-layout.md` §1, and `json-reports.md`: how a report names the builds a problem is in.
- `project-docs/checklists.md`, "A new command or option".
- The [search and answers plan](https://github.com/ascribed-dev/ascribe/blob/5b22286fd2f370e796219e68a7c9d7cdb2ba8acc/project-docs/search-and-answers/README.md): decisions 5, 15, 17, and 19.

## The decisions

Each has a recommendation. The maintainer may take it, change it, or ask for more. The maintainer took every recommendation, on [#494](https://github.com/ascribed-dev/ascribe/issues/494#issuecomment-6195368949), 15 October 2026, and each is recorded under its heading and in the [README's decisions](README.md#decisions), numbered 24 to 34.

### 1. What becomes of `[versions] current`

`current` was added after the proposal, as one version for every target. The proposal's list makes it a second home for the same fact, and one that can't say two products are at different versions.

| Option | Means |
|---|---|
| **Remove it; the list replaces it** (recommended) | `[versions]` keeps `scheme` alone, as the proposal says. `availability-left-behind` compares each target's versions with that target's current line, and reports nothing for a target with no `[releases]`. A project that sets `current` gets an error that names `[releases]` |
| Keep both, and report when they disagree | Nothing breaks. Two homes for one fact, and a check whose only job is to compare them |
| Keep `current` as the short form of a list with one line | One key for a one-site project. Two ways to write the same thing, and "which target" is still unanswered |

`current` is in `main` and in no release, so removing it breaks nobody. It amends decision 67, whose own reason ends "a table per target can come later".

**Decided:** remove it; the list replaces it. `[versions]` keeps `scheme` alone, `availability-left-behind` compares each target's versions with that target's current line, and a project that sets `current` gets an error naming `[releases]`. It amends decision 67, as [decision #494](../decisions/494-versions-current-removed.md) records. _Why:_ one home for the current version, which the list already gives for each target; `current` is in no release, so removing it breaks nobody.

### 2. What a line is

The proposal says `3.4` stands for every `3.4.x`. Ascribe's own marks are `ascribe 0.1.1` and its phrase is `0.2.0`, so "how long is a line's number" decides whether Ascribe's docs can use the list at all.

| Option | Means |
|---|---|
| **A line is the number as the list writes it, and a version is in the line it starts with** (recommended) | `3.4` holds `3.4`, `3.4.2`, and `3.4.2.1`. A project chooses how fine its lines are by how it writes them: `live = ["3.4", "3.5"]` for a window of minor releases, `live = ["0.2.0"]` for one site that follows its latest release. Every line of one target has the same number of parts, and a list that mixes them is an error |
| A line is always the first two parts | `3.4`, always. Ascribe's docs couldn't say `0.2.0` is current while `0.2.1` is planned |
| A line is a range the list spells out | `"3.4" = { from = "3.4.0", before = "3.5.0" }`. Says everything, and is a version range, which the language deliberately doesn't have |

A version shorter than its target's lines (`3` against `3.4` and `3.5`) starts no line and is in whichever lines follow it by order, as it is today.

**Decided:** a line is the number as the list writes it, and a version is in the line it starts with. Every line of one target has the same number of parts, and a list that mixes them is an error. _Why:_ a project chooses how fine its lines are by writing them, so Ascribe's docs can declare `0.2.0` while a product with minor lines declares `3.4`, with no range syntax the language doesn't have.

### 3. The phrase for a target's line

The proposal wants one home for "the current version", resolving to the line of the build it's in. A built-in name would collide: `docs/ascribe.toml` already has a phrase named `version`.

| Option | Looks like | For and against |
|---|---|---|
| **A phrase a project declares, whose value is a target's line** (recommended) | `[phrases]` `version = { line = "ascribe" }` | The project picks the key, so nothing is reserved and nothing collides. In a line's output it's that line; in a build that isn't versioned it's the current line. A second shape for a phrase's value |
| A reserved key for each target | `{self-managed-version}` | Nothing to declare. A key the project didn't write, and a new kind of name to keep out of `[phrases]` |
| No phrase; a project keeps one by hand, held by a test | As today | Nothing new. The fact keeps two homes in every project |

With decision 2 as recommended, Ascribe's docs write `live = ["0.2.0"]` and `version = { line = "ascribe" }`, and the hand-kept phrase and its test go, as the proposal's settled answer on releases expects. A project whose lines are `3.4` gets `3.4`, which is what an install command for a line names.

**Decided:** a phrase a project declares, whose value is a target's line: `version = { line = "ascribe" }`. In a line's output it's that line; in a build that isn't versioned it's the target's current line. _Why:_ the project picks the key, so nothing is reserved and nothing collides with a phrase a project already has, such as `docs/ascribe.toml`'s `version`.

### 4. How "planned" is written in three outputs

| Output | Recommended | Instead of |
|---|---|---|
| The availability element | A `planned` attribute on the target's entry, beside `states` and `versions`, present when the entry's last version is in a planned line. The text reads "Self-managed (planned, 3.6)" | A state named `planned` in `states`: it would be a lifecycle state after all, and collide with a project's own |
| The plain output's availability line | The same text for each target, since the element contract already says the two share it | |
| The JSON output | `planned: true` on the target, absent otherwise | A `status` word: there's one status to say, and a later one can be a key of its own |

The label, "planned", is set in `[releases]` as `planned-label`, for the project and not for each target. A state's label is changed the same way.

**Recommended as a whole.** The one thing to weigh is the element's text: "(planned, 3.6)" puts the word before the version, where a state's label goes, so a reader sees it as one.

**Decided:** as recommended, in all three. The element's target entry gets a `planned` attribute beside `states` and `versions`, and its text reads "Self-managed (planned, 3.6)"; the plain output's availability line says the same; the JSON output has `planned: true` on the target, absent otherwise. The label is `[releases] planned-label`, for the project. _Why:_ planned is something the list says about a version, not a lifecycle state, so it gets no place in `states`; the word goes before the version, where a state's label goes, so a reader sees it as one.

### 5. The file the lines share

| Question | Recommended | Why |
|---|---|---|
| Where | `<output-dir>/_ascribe/versions.json` | Above every build's folder, which is what "shared" means. `_ascribe` can't be a build's name, since a name starts with a letter |
| Who owns it | Ascribe, by a manifest of the folder's own, `<output-dir>/_ascribe/manifest.json`, in the format a build's manifest has | The store's rules work for each build, through its manifest, and nothing owns a file above them today. With a manifest there, the same rules apply one level up: Ascribe replaces and removes only what the manifest lists, and refuses a folder it didn't write |
| When it's removed | By the next `ascribe build` of a project that no longer has `[releases]`, with the manifest and the folder when nothing else is in it | Otherwise a site would go on reading a list the project deleted |
| When it's written | By every `ascribe build` of a project that has `[releases]`, for every versioned build, whichever build was asked for | A build of one line still has to know the others |
| Its top | `version`, and `builds`, keyed by a versioned build's name | One file for a project with two versioned builds |
| A build's entry | `target`; `lines`, in order, newest first; and `pages` | |
| A line | `line`, `status` (`current`, `live`, `planned`, or `archived`), `build` (the output's name, absent for a line with no output), `base` (the path its pages are under), and, from phase 3, `ends` and `url` | A site that isn't Astro can write a switcher from this alone |
| A page | Keyed by the source file's path from the content root. `lines`: for each line that publishes it, its route | A route differs by line and a source path doesn't. Phase 4 adds `replaces` and `replaced-by`, and changes nothing here |

Each key is contract from the phase that writes it. Later phases add keys and change none.

**The other shape considered:** a file in each line's output that lists the others. Every output would then change whenever any line did, and "nothing is shared" would be broken in every folder and not in one.

**Decided:** as recommended, every row. The file is `<output-dir>/_ascribe/versions.json`, owned through `<output-dir>/_ascribe/manifest.json`, written by every build of a project with `[releases]` and removed by the first build without it, in the shape the table gives. _Why:_ one file above the builds is the one exception to "nothing is shared", and a manifest of the folder's own lets the store's existing ownership rules hold one level up with no new ones.

### 6. How a problem in more than one line is said

A page that's the same in every line would otherwise report each of its problems once for each line.

| Option | Means |
|---|---|
| **Once, with the builds it's in, as a problem in two builds is said today** (recommended) | Nothing new in `check`'s JSON: a line is a build, named `site@3.4`. The text output says "in every line of `site`" when that's so, and names the lines otherwise. `builds_checked` lists each line |
| A new `lines` field beside the builds | Says "line" in the JSON. A second way to say where a problem is, for every reader of the report to learn |

**Decided:** once, with the builds it's in, as a problem in two builds is said today. A line is a build named `site@3.4`; the text says "in every line of `site`" when that's so; `builds_checked` lists each line; `check`'s JSON gains no `lines` field. _Why:_ no second way to say where a problem is for every reader of the report to learn.

### 7. What `@ascribed/astro` gives a site for the lines

Decision 82 expected a collection for each line. A site would then define collections it can't name ahead of time, and write a route for each.

| Option | Means |
|---|---|
| **One collection, with the line in each entry** (recommended) | A project's entry in `projects` is unchanged. Its collection holds every line's pages: an entry's id is its route under the base path (`install`, and `3.4/install`), and its data carries `line` and `current`. The one `[...slug].astro` a site already has serves every line, and an address falls out of the id. A component, `@ascribed/astro/Versions.astro`, lists a page's lines. It costs a loader: `ascribeCollection` is Astro's `glob` over one folder today, and the lines are separate folders, so the integration reads each line's output itself and prefixes the id |
| A collection for each line, named `<name>@<line>` | Matches the builds' names. A site's `content.config.ts` has to list collections that change with `ascribe.toml` |
| An entry in `projects` for each line | No change to the integration. The site's configuration repeats the list, and has to be edited on release day |

The integration's check that Astro's `base` holds the project's base path allows for the number.

**Decided:** one collection, with the line in each entry. Ids are routes under the base path (`install`, `3.4/install`), an entry's data carries `line` and `current`, a loader of the integration's own reads each line's output, and `@ascribed/astro/Versions.astro` lists a page's lines. _Why:_ a site's configuration doesn't change with `ascribe.toml`, and the one `[...slug].astro` it already has serves every line. If phase 2 finds a loader can't match `glob`, this decision is reopened before anything ships.

### 8. The two commands not yet named

`release`, `archive`, and `retire` are settled. Two more are needed.

| What it does | Recommended | Why |
|---|---|---|
| Says which lines exist, each one's status and address, and how many pages each has | `ascribe versions list` | `list` is what `permalinks` uses for the same job |
| Says what differs between two lines, from the marks alone | `ascribe versions changes "<target> <from>" <to>` | Not `diff`, which compares two revisions of the source, and not under `report`, whose subcommands are about one state of the project |

Each takes `--format text|json`, names a line as a mark does (`"self-managed 3.4"`), and exits 0, or 2 for a line that isn't declared.

**Decided:** `ascribe versions list` and `ascribe versions changes "<target> <from>" <to>`, each with `--format text|json`, naming a line as a mark does, and exiting 0, or 2 for a line that isn't declared. _Why:_ `list` is the word `permalinks` uses for the same job, and `changes` isn't `diff`, which compares revisions of the source, nor under `report`, whose subcommands are about one state of the project.

### 9. Where the rule for an older line's indexing lives

By default an older live line can be indexed, with its canonical links pointing at the current one. A project whose old pages outrank its new ones wants them out of the index.

| Option | Means |
|---|---|
| **A level in `[listing]`** (recommended) | `[listing] older-lines = "unlisted"`, between the site's `default` and a folder or a type, where the search and answers plan left the place. It restricts and never loosens, like every level, and a page's record says the rule came from it |
| A key in `[releases.<target>]` | Beside the lines it's about. A second place that sets how a page is listed, outside the rule that says which place wins |
| A key on the build | `versions = { target = "self-managed", index = "current" }`. The build's key becomes a table for one option |

It governs search engines and agents together, by that plan's decision 15. `older-lines` is a key at the table's top, beside `default`, `folders`, and `types`, and not a level under `folders`.

**Decided:** a level in `[listing]`: `older-lines = "unlisted"`, a key at the table's top beside `default`, `folders`, and `types`, between the site's `default` and a folder or a type. _Why:_ it's the place the search and answers plan left, so one rule still says which place wins, and it governs search engines and agents together by that plan's decision 15.

### 10. What a release does to the line before it, where nothing is versioned

A project with one site has no build that's versioned, so no line of it was ever published alone and there's nothing to archive. But `retire` refuses a line with no address (README decision 16), so without a rule here its `live` list grows by a line at every release and never shrinks. Ascribe's own docs, with `live = ["0.2.0"]`, meet this at their next release.

| Option | Means |
|---|---|
| **`release` moves the line before it to `archived`, with no address, when no build is versioned by the target** (recommended) | One live line, always, for a one-site project, and nothing to remember. The old line is declared, so a mark that names it still checks clean (README decision 22). No mark is rewritten: in a badge build each still shows its version. When a build is versioned by the target, `release` leaves the old line live, as the proposal has it, and `retire` is the way out |
| `retire` needs no address when no build is versioned by the target | Keeps `release` doing one thing. A one-site project has to run two commands at every release, and `retire`'s clean-up would rewrite marks that a badge build still shows |
| Leave it; a one-site project edits the list by hand | Nothing to build. The first project to use the list is the one that finds out |

`release` also takes a line nobody planned: one newer than the current line and older than every planned one, such as a patch release. It's added and made live, no feature names it, and `--next` isn't asked for.

**Decided:** `release` moves the line before it to `archived`, with no address, when no build is versioned by the target, and leaves it live otherwise. `release` also takes a line nobody planned between the current line and the oldest planned one. _Why:_ a one-site project keeps one live line with nothing to remember, the old line stays declared so its marks check clean, and no mark is rewritten.

### 11. What a line's output shows of a version from before the window

Retiring rewrites `self-managed 3.3` to `self-managed` and calls it a safe fix: "no line's output changes" (README decision 17). That's only true if no line's output was showing "3.3+" to begin with. And phase 3's check for a mark that does nothing is only right where the mark really shows nothing.

| Option | Means |
|---|---|
| **A line's output leaves out a version at or before the oldest live line; every other build shows it** (recommended) | In a versioned build, every reader of every live line has the content, so "3.3+" tells them nothing, and the mark keeps its state: `self-managed deprecated 3.2` shows "deprecated". `retire`'s rewrite then changes no line's output, and the proof is exact. A badge build that isn't versioned shows the version as it does today, so a one-site project's "since 0.1.1" stays, and nothing about it is reported |
| Every build shows every version | Nothing changes in the outputs. `retire` can't promise what decision 17 says, and has to list each badge it changed |
| No build shows a version before the window | One rule. A one-site project loses "since 0.1.1" from every page the day it adopts the list |

It follows that a mark "does nothing" only when every build that publishes its content is versioned by its target or filtered for it. Phase 3's two checks report in that case alone, so a project with one badge build is never asked to rewrite its history or declare every line it ever shipped.

**Decided:** a line's output leaves out a version at or before the oldest live line, keeping the mark's state; every other build shows it. A mark does nothing only when every build that publishes its content is versioned by its target or filtered for it. _Why:_ every reader of a live line has that content, so the version tells them nothing, and `retire`'s rewrite then changes no line's output exactly as README decision 17 says; a one-site project keeps "since 0.1.1" and is never asked to rewrite its history.

## Tasks

1. The maintainer reads the eleven and answers each.
2. Record each answer in this file, under its heading, as **Decided:** with the reason, and in the README's decisions, numbered from 24.
3. Record decision 1 where the repository records decisions, as an amendment to decision 67.
4. Add to the proposal what the answers change in it: `[versions]`, and the phrase.

## Out of scope

- Names of diagnostics. Each phase names its own by the conventions, and records them.
- Anything about the archive's format. Phase 8 opens with its own.

## Acceptance criteria

- Each of the eleven has an answer and a reason.
- The README's decisions list them, and nothing in a later phase file contradicts one.
- Phase 2 can be started without asking the maintainer anything.

## Commits

1. "Record the decisions the Versions plan ships"
