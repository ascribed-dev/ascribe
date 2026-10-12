# Page templates

Someone starting a page of a known kind (a how-to, a concept, a troubleshooting page) starts from the right shape, with guidance on what belongs in it. So does an agent asked to write one. The shape and the guidance come from [The Good Docs Project](https://www.thegooddocsproject.dev/template), and stay current with it. The idea is in the [proposal](../proposals/page-templates.md). This is the plan for building it.

## Who it's for

- **A writer starting a page,** who today copies a neighbor.
- **An agent asked for a page,** which today gets a type's frontmatter and nothing about what follows it.
- **A team starting a project,** which today begins from one line, `spec = "0.1"`, and a block copied out of getting started.

## What exists today

- **A page type is fields, not a body.** `[types.<name>]` in `ascribe.toml` has `files` and a frontmatter schema (`docs/content/reference/content-model.md` §4).
- **`ascribe agents prompt new-page`** writes a new page's frontmatter for a type and where its file goes. The plugin's `ascribe:new-page` command wraps it. The surface audit's finding 59 is about that command needing the type before it can ask.
- **One thing ships inside the binary and is written out once as the project's own:** the Vale preset, by `ascribe vale eject` (`crates/ascribe-check/src/prose/eject.rs`). It refuses when its files are there already, and exits 0 or 2.
- **One command edits `ascribe.toml` in place,** keeping its comments and order: `ascribe permalinks retire`.
- **`\{` is a literal brace** (SPEC §3, the escapes table), so `\{Task name}` is ordinary text that shows as `{Task name}`.
- **Checks about quality are advice** a project can raise, turn off, or acknowledge, each with a kind of next step.
- **Generated files have a pattern:** a test fails when one is stale, and `ASCRIBE_BLESS=1` rewrites it (`ARCHITECTURE.md`). A scheduled job that follows another repository and opens a pull request exists too, in `examples/docs-repository/.github/workflows/update-sources.yml`.
- **No construct holds text that's never published.** The [memos proposal](../proposals/memos.md) designs one, `@memo`, and it isn't built.
- **Nothing starts a project or a page for a person** (the surface audit's finding 53), and every Astro site rewrites the same four files (finding 29).

**In The Good Docs Project,** release v1.6.0, tagged 11 June 2026: 25 templates in three packs, a guide with each, an `index.json` naming each template's files, and the MIT No Attribution license. Their repository is on GitLab, `gitlab.com/tgdp/templates`. The copy under their GitHub organisation is archived, with tags only to v0.2.0, and isn't the source. GitLab's tags can be read without signing in, and its releases can't. Checked unchanged, six Core templates gave 5 errors and 18 warnings, from two causes: placeholders in curly brackets, and each template's relative link to its guide.

## Decisions

These are settled. Don't reopen them in a phase; if one can't be met, stop and report.

1. **Convert, don't rewrite.** A script writes Ascribe's copies from a pinned release of theirs. Nobody edits a converted file. A sentence of theirs that's wrong is fixed in their repository.
2. **Three layers, kept apart:** their template and guide, converted; a few structural rules of ours, as data the script reads; and Ascribe's notes on a type, written by hand in a file of their own.
3. **A structural rule that no longer matches fails loudly.** The script stops, naming the template and the place.
4. **Placeholders are their curly brackets, escaped.** Their wording isn't touched.
5. **A leftover is found by comparing a page with its type's template,** with nothing added to the language. It only knows the template as it is now.
6. **The starter set ships inside the binary,** and gets into a project by being copied, once. From then on the files are the project's, and nothing in Ascribe overwrites them.
7. **A type names its template and its guide,** as files in the project, outside its content. A type with neither works as it does today.
8. **One construct for text that's never published,** shared with notes. Templates don't get their own.
9. **A template is a page with things to fill in.** No conditions, loops, or variables beyond the title.
10. **What ships first is the Core pack, less the README:** concept, how-to, tutorial, reference, troubleshooting, and release notes.
11. **Credit is given, though the license asks for none.** Each converted file says where it came from and at which release.
12. **Ascribe's own docs use the types.**
13. **The command is `ascribe templates eject`,** and `ascribe templates status` says what's changed upstream since the copy, by the surface audit's conventions.
14. **A type names its files with two keys: `template`, one path, and `guide`, a list of paths,** both from the project's folder and outside the content root. The guide and Ascribe's notes are two entries of the list, in the order to read them, so `templates status` can show the guide's upstream changes alone. `guide` is always a list, as `files` is, since no key in the content model takes two shapes. There's no `notes` key: it would give "note" a fourth meaning.
15. **The copied files go in `templates/`, beside `ascribe.toml`,** or where `--to <DIR>` says, a path from the current folder. Not under `.ascribe/`, which the docs say can be deleted: these are files the team edits and commits.
16. **`ascribe templates eject [NAME]...` checks everything first and writes nothing when anything is refused.** A type declared already or a file in the way is named, nothing is written, and it exits 2, as `ascribe vale eject` does. With no name it takes the whole set. It edits `ascribe.toml` in place, keeping comments and order, giving each type `files = ["<name>/**"]`, `template`, `guide`, and its frontmatter, and says so. `--format text|json`. `ascribe templates list` reads only the binary.
17. **A placeholder is a run of literal text that an escaped brace opens and its closing brace ends,** found by reading the template as any page is read. The check, `template-placeholder-left`, is advice, configurable, next step `write`, comparing canonical form and ignoring how a line was wrapped. A placeholder changed by a word isn't found: the check finds what was never touched.
18. **A template's instructions stay as quoted paragraphs until `@memo` exists,** and the same check reports one left whole in a page, with a message that says it's an instruction. Nothing waits on the memos plan, and a forgotten instruction is reported before it's published.
19. **Their release is pinned in one file the script and the test both read; their source is fetched into an ignored folder, never committed; Ascribe's copies are committed in the crate that carries them,** each with a first line saying it's generated, from what, and at which release; the structural rules are a data file beside the script; and Ascribe's notes are hand-written files the script never opens.

The options each of 14 to 19 was chosen from are in [phase 1](phase-1-decisions.md).

Six more, from a review that read their v1.6.0 files against the plan, before phase 2 started. Each says what a decision above left unsaid, and two amend one.

20. **A placeholder can run across blocks, and is one placeholder.** Their how-to's "See also" opens a brace in a paragraph and closes it three list items later. Inline text doesn't cross a block, so decision 17's "run of literal text" is read block by block: in the block where the brace opens, the placeholder runs to its closing brace or to the block's end; it takes every block after that whole; and in the block that holds the closing brace, from the block's start to the brace. The conversion escapes the one opening brace and changes nothing else. The check compares each piece and reports the placeholder once, where its first piece is found.
21. **A template's title is its frontmatter's, and a field left as the template has it is a placeholder left.** Every one of their templates opens with a level-1 heading, `# Title` in the how-to and `# Release notes - {Product name} {Release number - 0.0.0}` in the release notes. Ascribe takes a page's title from its frontmatter, and a site's layout draws the heading. So the conversion moves that heading's text into `title`, as a quoted string with its braces as they are, and takes the heading out of the body. Decision 17's definition is for a page's body. In frontmatter the rule is plainer: a field whose value is the same as its type's template has, braces or not, was never filled in.
22. **What a template says about itself goes.** Each opens with a block quote, the link to its guide and then a paragraph about the template and its `{curly brackets}`, and closes with another. Both are about the template and not about the page a writer is making, so the conversion removes them whole. Decision 18's instructions are the ones inside a section, about that section.
23. **A converted template carries no line saying where it came from,** which amends decisions 11 and 19: the credit is in each guide, in `templates.lock`, and on the docs' page, and not in a template. A page copied from a template carries every line the template has, and nothing unpublished exists to hold one. The guide says it, in words, since a guide isn't a page. And `ascribe templates eject` writes one small file under `templates/`, `templates.lock`, listing each file it wrote with their release, Ascribe's version, a hash of what was written, and their index's summary of the type. It's Ascribe's file, as `ascribe.lock` is: written by the command, committed with the templates, and not edited. `ascribe templates status` reads it.
24. **Starting a project is `ascribe init`, and `ascribe templates eject` always needs a project to add to.** A command named for writing templates out that also wrote a project's first `ascribe.toml` would have two meanings, which the surface audit's conventions exist to catch, and deciding it after `eject` is documented would mean a rename. `ascribe init` writes the first `ascribe.toml`, and with `--templates` takes the starter set too, by calling what `eject` calls. Phase 5 builds it. Until then `eject` says there's no project and how to make one by hand.
25. **Their repository is read by its tags,** at `gitlab.com/tgdp/templates`. The pin is a tag and its commit. What changed between two releases is the diff between their tags: the index's `last_updated` isn't a guide to it, since the how-to's is 15 December 2022, four releases before v1.6.0.

One more, from phase 2's model task ([#400](https://github.com/ascribed-dev/ascribe/issues/400)), which left where a type's files are checked and how a template is read to the phase.

26. **A type's files are checked where the project's files are read, and a template takes no read of its own.** The loader checks what's written: `template` is a string, `guide` a list of strings, and neither is absolute. `ascribe_check::type_files` checks where each leads, through `FileSystem`: `probe` for the file and its exact name, `real_path` for where a link leads, and `read_file` for the template, the reads a snippet's code file and `ascribe.lock` already take, so `file_reads.rs` gains nothing. A file is in the project's boundary when it's in the folder `ascribe.toml` is in, links followed, and outside the content root and the output directory. A problem is the model's, at the path in `ascribe.toml`, and stops the load as the model's errors do: `model-type-file` for where a file is, and `model-type-template` for a template that can't be read, whose frontmatter isn't YAML, or whose syntax has an error by the registry's severity. A warning in a template isn't a problem. The language server checks the files when `ascribe.toml` changes, not when a template does.

Five more, from phase 2's command ([#401](https://github.com/ascribed-dev/ascribe/issues/401)), which left the lock's shape, the type's frontmatter, and what eject does at the edges to the phase.

27. **`eject` lives in `ascribe-templates`, beside the set it writes,** and takes the content model, `ascribe.toml`'s text, the folder, and the names, returning what it wrote, what it added, and what it refused. A refusal isn't an error: it's listed, in the text and the JSON, and the command exits 2 having written nothing. A name the set doesn't have is a refusal too, so one run names everything wrong with it. What stops the whole run is an error with a code: a folder no type can name a file in (`templates_folder_refused`), a `templates.lock` it can't add to (`templates_lock_invalid`), types it can't add (`model_edit_refused`), and a file it can't write (`templates_unwritable`).
28. **A type is added at the end of `ascribe.toml`, as text,** and nothing above it is touched: no comment, order, or spacing changes, which is "in place" at its plainest. The result must load as a content model with the types in it, or nothing is written: a file that declares its types as one inline table can't take a `[types.<name>]` after it, and is told to add it by hand. The type's frontmatter is the template's: each field the template's frontmatter has, as a string a page must have, so the fields have one home, the converted template. A file that declared no types, with no `types` key or an empty `[types]`, had every page of the implicit `page` type, and adding one type takes that away (validation contract, decision 18), so eject writes `[types.page]` out first as it was, and every page outside the new types keeps checking as it did.
29. **The folder eject writes into is held to decision 26's rule before anything is written:** inside the folder `ascribe.toml` is in, and outside the content root and the output directory, links followed. `ascribe_check::type_files::folder_problem` says why a folder fails it, so the rule has one home. `--to` is a path from the current folder; outside the project's folder, the command refuses it before calling eject.
30. **`templates.lock` is TOML, as `ascribe.lock` is,** with `version = 1` and one `[[template]]` for each template ejected: its `name`, their `summary`, `repository`, `tag`, `commit`, and `license`, the version of Ascribe that wrote it (`ascribe`, the binary's whole version), and `[template.files]`, each file's path from the folder `ascribe.toml` is in with `sha256:` and its hash, as `ascribe.lock` writes one. Paths are from the project's folder, as `ascribe.toml`'s are, so `templates status` can match an entry to a type's keys. A second eject adds its entries, keeping what's there; an entry of the same name is replaced, which happens only when its type was removed and its files deleted.
31. **The JSON's paths are from the folder `ascribe.toml` is in, and the text's from the current folder,** as the checklist says printed paths are. The JSON is read by tools, which compare it with `ascribe.toml`. `ascribe templates list` takes no `--format`: a line for each template is all it says, and nothing reads it yet.

Three more, from phase 2's check ([#402](https://github.com/ascribed-dev/ascribe/issues/402)), which left what "has a placeholder" means at its edges to the phase.

32. **A placeholder is found when a page has every piece of it, in order, and is reported at the first.** A block, for the comparison, is a paragraph, a heading, or a table's cell, read as the text it shows: emphasis and links are their text, a code span keeps its backticks, and each run of white space is one space. A piece of one block is found anywhere in a block of the page; each piece after the first is found in a later block than the one before it. A placeholder the writer has begun filling in, keeping the "See also" paragraph and writing their own list, isn't found, which is decision 17's "the check finds what was never touched" for one that runs across blocks. The same placeholder twice in a template, as the how-to's two steps are, is one placeholder, reported at each place a page has it.
33. **An instruction is a paragraph inside a block quote of the template, and is found when a paragraph of the page, quoted or not, is the same text.** A placeholder inside an instruction is part of the instruction, and isn't reported on its own as well. A field is left when its value, white space made single spaces, is the template's, and an empty field of the template is nothing to fill in. One diagnostic, `template-placeholder-left`, with a message for each of the three: a placeholder, a field, an instruction.
34. **A template is read once for each project the checks are given, when a page of its type is first checked, through that project's `FileSystem`,** with `read_file`, as decision 26 reads it. A template that can't be read is the content model's error there, and here it isn't looked at. A project made from another with one page's text changed keeps the templates already read; the language server makes a project for each check, so it reads the template again each time.

One more, from phase 3's notes ([#404](https://github.com/ascribed-dev/ascribe/issues/404)), which left where the notes live and how the lock tells them apart to the phase.

35. **Ascribe's notes on a type are `notes/<name>.ascribe.md` in `ascribe-templates`, beside `set/` and not in it,** since the conversion owns `set/` and removes what it didn't write. The binary carries them as `NOTES`, and eject writes each under the same name in the templates' folder, second in the type's `guide`, and refuses it as it refuses the template when a file is in the way. `templates.lock` records them under `[template.ascribe-files]`, each path with its hash, so `[template.files]` stays their files alone and `templates status` compares a release of theirs with what came from it. Every template of the set has notes, and a test holds it. The notes on the concept, the tutorial, and troubleshooting were found by writing a page of each in a sample project first, as the phase says.

Three more, from phase 3's conversion of the other five ([#403](https://github.com/ascribed-dev/ascribe/issues/403)). Every one of the five converted by rule; none needed a step by hand.

36. **What a template says about itself goes wherever it is, and a rule names it when it isn't in a block quote.** Their reference opens with two block quotes about itself, not one, so the conversion takes every block quote a template opens with. Their release notes say it in a block quote and then in two paragraphs written as placeholders, one linking to a process document that isn't copied. A rule of a new kind, `about`, names each such paragraph by its first line, as theirs has it, and the conversion removes it whole, as decision 22 removes the block quotes. Like a directive, it must find its paragraph exactly once or the script stops.
37. **"Removing a rule makes the script stop" means the check stops.** Each directive and each `about` rule finds its line exactly once or throws. Taken away, a rule changes what the script writes, so `node scripts/templates/convert.ts --check`, which CI runs, stops on the committed copies; a test takes each rule away in turn and shows the output changes. A template's table without its `description` stops `readRules` itself.
38. **The set's order is the how-to first, then decision 10's:** concept, tutorial, reference, troubleshooting, and release notes. It's the order of `rules.toml`, so `set.toml`, `ascribe templates list`, and `eject` with no name follow it, and the how-to the set began with stays first. The tutorial's procedure takes `@steps` by the same rule the how-to's does: both are numbered steps under a "Task name" heading.

Three more, from phase 3's command for the six ([#405](https://github.com/ascribed-dev/ascribe/issues/405)), which left what eject says and what the tests and the example hold to the phase.

39. **Eject's text says once that a type's folder is only a start, and `files` is the line to change.** It follows the lines naming each type and its folder, one sentence whatever the number of types. The JSON doesn't change: `added` already carries each type's `files` and `folder`. Each type's frontmatter is still the template's own, and every one of the six has a title and a description and nothing else, so decision 28 needs nothing added for them.
40. **"Checks clean" means nothing at all to report, advice included, and "reports its placeholders" is held line by line.** A project eject gave the whole set checks with no diagnostic. A page copied whole from each template is reported on every line where the template opens a placeholder with `\{`, and on its title and description, and nowhere else, with nothing but `template-placeholder-left`; a count would hold the check's own rules for repeats, which decision 32 settles and its conformance cases hold. Both are tests of `ascribe-templates`, through the library, as its other eject tests are.
41. **The example is made again by `eject` with no name, and its filled pages are the "reports nothing" half.** `examples/templates` holds every file eject writes, Ascribe's notes with them, and one page of each type, every placeholder written over, about the same made-up tool as before. The CLI's test of it compares each file with the set and asks `ascribe check --format json` for an empty list, since `--deny-warnings` lets advice through.

Three more, from phase 6's job ([#411](https://github.com/ascribed-dev/ascribe/issues/411)).

42. **The update pull request is opened with the job's own `GITHUB_TOKEN`,** with contents and pull-requests write, so its checks don't start by themselves, and its description says to close and reopen it to start them. A GitHub App's token would start them, and is the maintainer's to make. _From:_ the issue, as its default.
43. **The update makes `examples/templates` again too,** by `ascribe templates eject` with no name in an empty project, with the binary built from the new copies, so decision 41's test holds on the pull request without a person's step. **A rule that no longer finds its place moves the pin and nothing else,** so the pull request is where the rule is fixed, and `update.ts --pin <tag>` then writes the copies.
44. **The job is tried after the merge by setting the pin back on its branch, in a commit of its own,** with the workflow's `from` input: the branch's first commit sets the pin back and writes the copies at that tag, and the second is the update. The pull request's diff from `main` is then empty, its second commit is what the job would open, and it is closed rather than merged. It goes on `templates/trial`, so it never replaces an open update pull request.

One more, from phase 4's rules for agents ([#407](https://github.com/ascribed-dev/ascribe/issues/407)), which left the key's shape and the cut to the phase.

45. **A type says what it's for with `purpose`, one string, and the rules' type lines are the first thing cut.** It's optional, and the loader checks only that it's text with something in it (`model-wrong-type`, `model-empty-text`). `ascribe model` shows it, in its text and as `purpose` in its JSON, absent when a type has none. In the rules `ascribe agents sync` writes, each type's line ends with its `purpose`, or, for a type whose `template` a `templates.lock` beside it lists, their index's `summary`; a lock that's missing or isn't one eject wrote gives nothing, and isn't reported there. The phrases are cut to 2,000 characters as before; then, when the whole is over 4,000, the type lines are cut before the directives, and the cut says `ascribe model --section types` lists the rest. The six types of the set, with their summaries, fit whole.

Two more, from phase 4's start of a page for an agent ([#406](https://github.com/ascribed-dev/ascribe/issues/406)), which left the command at the terminal, and the edges of where a page goes, to the phase.

46. **A page starts from `ascribe_templates::start`, and the prompt is its only caller for now.** Given a type, a title, and optionally a path from the content root, it returns the page's path and text, and the type's `template` and `guide`. The text is the template, read through the project's `FileSystem`, with its frontmatter's `title` set as a double-quoted YAML string and nothing else changed; a template with no `title` gets one, first. A type with no template gives `title` and each other field it requires, empty, since a value written in would hide a field never filled in. With no path, the page goes in the first folder one of the type's `files` covers, the segments of the pattern before its first glob, where a page named for the title is of the type, named by the title slugged as a route's segment is, with `.md`. A path given ending in `.md` is the page's, and any other is a folder, where the page is named for the title; either way it must be a page of the type: inside the content root, not a fragment's, and matched by the type alone. The prompt's `path`, which named only the project, is the page's when it's in the content root, from the current folder or, when nothing is there, from the content root, as `retire --page` takes a page; the plugin's command already passed it a file or folder for the page, and the trial session did. A file there, of any case, is refused (`page_exists`), and nothing is ever written. `new-page` names that file as taken and gives the same text, and the agent picks another path of the type's. The guide's paths come before the template, and the whole prompt keeps its limit of 5,000 characters: the type's fields are cut first, then the template, and the cut names the template's path.
47. **No command at the terminal starts a page.** The checklist names a top-level command by a verb for what it does to the project, and none says "make a page from a template" without misleading: `start` reads as starting a server, `new` isn't a verb, and `write` says nothing of a template. It isn't under `templates`, whose commands are about the set, nor `agents`. The surface audit found two users, the writer in the editor and the agent, and phase 4 covers both: the editor's action, still to come, and the prompt. For a person at a terminal, copying the template is one line, and the templates page says so. `ascribe init`, which starts a project, is no precedent: it's the one command for its thing, as a page's would be, but a project is started once and a page often, from a type the project chose.

Six more, from phase 6's `ascribe templates status` ([#412](https://github.com/ascribed-dev/ascribe/issues/412)), which left where it looks, what it says, and its exit codes to the phase.

48. **`status` lives in `ascribe-templates`, and takes the set as an argument.** `status` takes the content model and an `Upstream`: the set's list and the text of each of its files and of Ascribe's notes, by name. `Upstream::carried()` is the binary's set, and a test makes a newer one by changing a text and the tag. It reads the project through its `FileSystem` and nothing else: no network, no git. It returns what it found and never fails, since a lock it can't read is something to report. `diff` gives `--diff`'s unified diffs from the same answer. The lock is read by `parse_lock`, which eject's own read now calls too, so the two can't disagree about what a lock is.
49. **A lock is looked for in `templates/`, and beside each type's `template`;** an entry is the first of its name in those locks. **A template is taken** when a lock has an entry for it, or when the model has a type of a set template's name with a `template` key. That covers `--to`, as decision 45's `summaries` does, without an option of its own. A type of a set template's name with no entry is `no_entry` when a lock is beside its template, `invalid_lock` when that lock isn't one eject wrote, and `no_lock` when there's none, and its files, the type's `template` and `guide`, are compared with the set alone: `matches_set`, with `upstream` and `project` both `unknown`. A file of the set's that the project doesn't have isn't listed; a template the set has and the project hasn't taken is in `not_taken`.
50. **The lock's hash tells the two sides apart, file by file.** `upstream` is `changed` when the set's text doesn't hash to what the lock recorded, `removed` when the set no longer has the template, and `unchanged` otherwise; `project` is `changed` when the file doesn't, and `deleted` when it's gone. A file's line endings are made LF before it's hashed, as eject wrote it, so a checkout that made them CRLF isn't a change. `sections` is given only when the set changed a file and the project didn't, since only then is the file what the set had, and it compares the two by heading: each heading of the set's the file hasn't (`added`), each the file has and the set hasn't (`removed`), and each both have with other text under it (`changed`), with null for what comes before the first heading. A heading written twice is two sections.
51. **`status` reads the model without the check of the types' files,** so a template that's gone is reported as `deleted` rather than stopping it. Every other rule of the loader still applies (`load_str_bounded`, as `type_files::load_model` calls it before its check), and a model that doesn't load for any other reason exits `2`.
52. **`--exit-code` exits `1` when any taken template's `upstream` is `changed` or `removed`:** a template the set no longer has is a change upstream a CI job would want to know of. `upstream_changed` in the JSON says the same. `unknown` doesn't count, since without an entry nothing says the set changed anything.
53. **`--diff NAME` prints the change to one template and nothing else,** as a unified diff from the project's file to the set's, for each file whose `upstream` is `changed`, or, without an entry, each that isn't the set's. When the project has changed the file, or nothing is remembered of it, it says so on standard error, since the diff then undoes the project's changes too. It exits `0`, or `1` with `--exit-code` when that template changed upstream. A name of the set's the project hasn't taken is a thing of the project's that isn't there, `1`, with the command to take it; a name the set doesn't have is Ascribe's own, `2`, with the code `unknown_template`. `--diff` and `--format` don't go together.

## The thread

**The use case:** in a project, `ascribe templates eject how-to` writes the how-to template and its guide and declares the type; a page copied from the template and half filled in makes `ascribe check` say which of the template's placeholders are still there.

**The layers it crosses:**

| Layer | Where | What the thread needs from it |
|---|---|---|
| Their repository | A pinned release, read by a script | One template and its guide |
| The conversion | `scripts/`, and the committed copies | The how-to, as a valid Ascribe page, with a test that it matches the release |
| The binary | `ascribe-cli`, and the crate that holds the set | The copies, carried inside it |
| The content model | `ascribe-model`, `ascribe.toml` | `template` and `guide` on a type |
| The command | `ascribe-cli` | Writing the files, and the type into `ascribe.toml` in place |
| Check | `ascribe-check`, the registry | One advice check, which reads a file outside the content |

**The seam it tests** is text Ascribe didn't write becoming a page Ascribe accepts, mechanically, and staying comparable afterwards. Two assumptions meet there. One is that a script with a short table of rules can turn their Markdown into a page that passes `ascribe check` and `ascribe fmt` unchanged, without a person touching it. The other is that "the same text as a placeholder in the template" is an exact comparison once both have been through the formatter and a writer's editor. If the first fails, the set has to be maintained by hand, and following upstream becomes a merge. If the second fails, a leftover needs a construct of its own, which the proposal chose not to add.

**Left out, and where it comes back:**

| Left out | Phase |
|---|---|
| The other five templates, their rules, and Ascribe's notes on each | 3 |
| Starting a page: the prompt, the editor's action, the rules for agents | 4 |
| A project with no `ascribe.toml` | 5 |
| Following a new release of theirs; `ascribe templates status` | 6 |
| Ascribe's own docs taking the types | 7 |

After phase 2, a page is started from a template by copying the file. The command reference says so while that's true.

## Phases

| Phase | Result |
|---|---|
| [1: Decisions](phase-1-decisions.md) | The maintainer settles the keys on a type, where the files go, what counts as a placeholder, and what carries a template's instructions until `@memo` exists. |
| [2: The thread](phase-2-thread.md) | `ascribe templates eject how-to` gives a project a how-to template and guide, and `ascribe check` reports a placeholder left in a page. |
| [3: The six](phase-3-six.md) | The whole starter set: six templates, six guides, Ascribe's notes on each, and a content model that declares the six types. |
| [4: Starting a page](phase-4-start.md) | A person starts a page from a type in the editor, and an agent gets the template and is sent to the guide. |
| [5: A project from nothing](phase-5-project.md) | In an empty folder, `ascribe init` gives a first `ascribe.toml`, and the starter set with it when asked, and getting started points at it. |
| [6: Following their releases](phase-6-upstream.md) | A new release of theirs arrives as a pull request to read, and a project can see what's changed since its copy. |
| [7: Ascribe's own docs](phase-7-docs.md) | The docs use the types, and the templates have met pages that already exist. |

Phase 1 comes first because phase 2 ships two keys in `ascribe.toml`, a command, and a diagnostic. After the thread the order is by risk: phase 3 puts the conversion through five more inputs, which is where a rule could prove not to be mechanical. Phases 4 and 5 widen who can use it, phase 6 is the upkeep, and phase 7 is the test on real pages.

Each phase's result is something a person can run or open. None builds a layer for a later phase to use.

## Carved out, and waiting

- **A template's instructions as text that's never published.** They wait for `@memo`, which the [memos proposal](../proposals/memos.md) designs and nothing has built. Until it exists they're carried as phase 1 decides. When it does, the conversion gains one rule, the set is regenerated, and a project's existing copies are its own to change or not. That's a small change to this plan's output and not a phase of it.
- **A site's starter files** (the surface audit's finding 29): the collection, the route, a layout that loads the elements, and the sidebar's list. They'd be written by `@ascribed/astro`, not the binary, into an Astro project, which is a different package, a different seam, and a design that depends on navigation, which has no proposal. It has an issue of its own, [#344](https://github.com/ascribed-dev/ascribe/issues/344), and this plan's command is named so a second thing to eject fits beside it.
- **Checking a page's shape against its type** ("a how-to has a procedure"). It needs a way to describe a page's outline in the content model. Later, as the proposal says.

## The other plans

- **The surface audit.** Its conventions name the command and its verb, say what `--format` words it takes and what it exits with, and say a path given to a command is from the current folder. Its finding 53 is phase 5, and its finding 59, the plugin's `new-page` command, is in phase 4's way and should be read first.
- **Search and answers.** A template's page has a title and a description from its first line of frontmatter, so pages made from one start with what that plan's checks ask for.
- **Memos.** This plan is the first user of `@memo`, and doesn't wait for it.

## Rules for every phase

- Branch before committing; never commit to `main`.
- Read the current code before the phase file's pointers. If the phase file and the code disagree, or a decision above can't be met, stop and report instead of choosing silently.
- **Nobody edits a converted file.** A change to one is a change to the script, to a rule, or to their repository. The test that compares the copies with the pinned release fails otherwise.
- **The language and the contracts change only by a recorded decision.** Go through the matching list in `project-docs/checklists.md` first, and add the decision as a file in `project-docs/decisions/` in the same pull request.
- **A new diagnostic** is added to `tests/conformance/diagnostics.toml` and `crates/ascribe-core/src/diagnostics.rs`, in the same order, with a conformance case. A check about quality is advice, configurable, with its kind of next step.
- **Project files are read through `ascribe_resolve::FileSystem`.** A template is a project file outside the content root, and how it's reached is phase 2's to get right once. Libraries don't print, and don't panic on input.
- **Every converted page passes `ascribe check` and is unchanged by `ascribe fmt`.** A test runs both on the whole set.
- **The map of the surface** is regenerated when a phase adds to it (`node scripts/surface/inventory.ts`), and its diff is read against the conventions.
- **User-visible changes** update the page under `docs/content/` and the unreleased section of `CHANGELOG.md` in the same phase.
- No phase history in code or docs. Describe what the code does now.
- Before finishing a phase, all of these pass:

  ```sh
  cargo fmt --all --check
  cargo clippy --workspace --all-targets --locked -- -D warnings
  cargo test --workspace --locked
  pnpm format:check && pnpm lint && pnpm typecheck && pnpm test
  ```

## Later, not in this plan

- The rest of their templates: quickstart and the installation guide first, then the API pair.
- Templates for a fragment or a section.
- Converting a project's existing pages to a type.
- Checking that Ascribe's glossary, its Vale preset, and its changelog agree with their guidance on each.
