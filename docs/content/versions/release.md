---
type: reference
title: "`ascribe versions`"
description: "List a project's release lines, say what differs between two, release a line in one command, export one into a folder of HTML, and retire it from the source."
---

The commands for a project's [release lines](lines.md): what they are, what differs between two, the release that makes a planned line live, the export that keeps a copy of a line once it leaves, and the retirement that takes it out of the source. [Publish a site for each version](index.md) and [Export and retire a line](archive.md) are the tasks they're used in. The options every command accepts are in the [command reference](../command/index.md#options-every-command-accepts), and the JSON each writes with `--format json` is in [JSON reports: `ascribe versions`](../contracts/json-reports/versions.md).

## `ascribe versions`
@id {permalink=true}: cli-versions
@summary: Lists a project's release lines, says what differs between two, releases one, archives one, and retires one.

Lists a project's release lines, says what differs between two, releases one, archives one, and retires one. Each reads the lines from `ascribe.toml`'s [`[releases]`](../content-model/content.md#71-releasestarget), and a line is named as a mark names it: the target, then the version, `"self-managed 3.4"`.

### `ascribe versions list`
@id {permalink=true}: cli-versions-list
@summary: Shows each target's release lines, newest first: whether each is current, live, planned, or archived, with its output, base path, and last day of support.

Shows each target's lines, newest first: whether each is current, live, planned, or archived, the output and base path of each live line on each build versioned by the target, with how many pages it publishes, the last day of support from `ends`, and an archived line's address and the day it ended. It reads the project and builds nothing. In `examples/quill`:

@snippet {lang=text}: code:crates/ascribe-cli/tests/output/versions-list.txt

@include: ../_generated/cli-versions-list-options.md

With `--format json`, it writes the same in the document its [schema](../contracts/json-reports/versions.md#ascribe-versions-list) describes. It exits with `0`, and `2` when the project doesn't load.

.The JSON for this example
@details:
@snippet {lang=json, phrases=true}: code:crates/ascribe-cli/tests/output/versions-list.json
@end

### `ascribe versions changes`
@id {permalink=true}: cli-versions-changes
@summary: Says what differs between two lines of a target, from the marks alone, with nothing built: the pages, sections, blocks, and table rows added, removed, or changed.

Says what differs between two lines of a target, from the marks alone, with nothing built: the pages, sections, blocks, and table rows that are in the second line and not the first (added), in the first and not the second (removed), and in both with a different state, such as deprecated in one (changed). Each is listed with its file and line, the page, the heading it's under, and its first line. A page whose own mark adds it is one entry, not an entry for each part of it. A page added in place of one removed, which `replaces` links it with either way, is one entry too: it's listed as added, and names the page it's in place of, which isn't listed as removed. `ascribe versions changes "self-managed 3.4" 3.5` in `examples/quill`:

@snippet {lang=text}: code:crates/ascribe-cli/tests/output/versions-changes.txt

The second line is a version of the first's target, and either may come first. Each must be declared in the list, whether live, planned, or archived. A line holds what's marked for a version in it, not what's marked for a later one, so a line is compared as it was released, whichever is current.

@include: ../_generated/cli-versions-changes-options.md

`--format markdown` writes a first draft of release notes: a heading for the line, and one for each page, with what's new, gone, and changed on it.

@snippet {lang=markdown}: code:crates/ascribe-cli/tests/output/versions-notes.md

With `--format json`, it writes the document its [schema](../contracts/json-reports/versions.md#ascribe-versions-changes) describes:

.The JSON for this example
@details:
@snippet {lang=json, phrases=true}: code:crates/ascribe-cli/tests/output/versions-changes.json
@end

| Code | Meaning |
|---|---|
| `0` | Compared, whether or not anything differs |
| `2` | It couldn't compare: the project doesn't load, or a line isn't named as a mark names one or isn't declared |

### `ascribe versions release`
@id {permalink=true}: cli-versions-release
@summary: Releases a line: rewrites each `planned` mark for its target to the line's version, makes the line live, and archives the old current line.

Releases a line: rewrites each mark's [`planned`](../writing/directives.md#availability-specs) for its target to the line's version, makes the line live, and archives the old current line where no build is versioned by the target. [When a line ships](index.md#release-day) has each step. It edits `ascribe.toml` in place, keeping its comments and order, and each mark where it's written. A feature's spec names its versions, so `planned` in one is left as it is. In a copy of `examples/quill` that plans 3.6, with a section marked `@available: self-managed planned`, `ascribe versions release "self-managed 3.6" --next 3.7 --dry-run` says what it would do, `--next` adding 3.7 to `planned` so that `planned` names it from then on. Its site is versioned, so 3.5 stays live, with an output of its own, `site@3.5`:

@snippet {lang=text}: code:crates/ascribe-cli/tests/output/versions-release.txt

When it archives the old current line, the marks that name a version in it say nothing any more: every line left has what they mark. It removes or rewrites each one [`available-same-in-every-line`](../checks/diagnostics/content-checks.md#ASC212) would report, as that check's fix does: a mark with nothing left to say goes, and one that still says something, such as a state or another target, keeps that alone, as `self-managed beta 3.5` becomes `self-managed beta`. A feature's spec is rewritten the same way. The text lists each with its file and line, and the JSON as `cleared` and `cleared_features`, so after it `ascribe check` reports no `available-same-in-every-line` for the archived line. A mark that said nothing before the release, naming an older line, is left for `ascribe check` to report. A badge build showed what these marks said, so the pages it drew a badge on are among the outputs the release changes.

It builds the project in memory before and after, and refuses when rewriting the marks or archiving the old line would change what a reader sees: both are meant to change nothing, so a change there is a mark the rewrite got wrong. What the release itself changes, each page whose content was planned and is now live, it lists by build. It writes nothing when it refuses, and `--dry-run` says what it would do and writes nothing.

It writes only files git can put back as they were: `ascribe.toml` and each page it rewrites must be tracked, and unchanged since the last commit. So the release is one commit to read, and `git checkout` undoes it. With `--force` it writes them anyway, as it must outside a repository.

In a project whose [`[from]`](../content-model/index.md) takes the target's lines from another project's `ascribe.toml`, the release is made where they're written: in that project, its `ascribe.toml`, its pages' marks, and its outputs. The text says so first, and the JSON's `from` names the file, from the current folder; `files` are then from that project's root. The lines move for every project that takes them, from the one release. A mark on a page of a project that takes them isn't rewritten.

A line that isn't planned can be released too, as long as it falls between the current line and the oldest planned one: a patch release, `3.5.1` when the lines are written as patches, needs no entry first. `planned` keeps naming the planned line, so no mark is rewritten.

It refuses a line that's live or archived already, so running it twice changes nothing the second time, and a planned line with an older one planned before it. A mark it can't rewrite where it's written, such as a frontmatter `available:` that spans lines, is named with its file and line, to write on one line first.

@include: ../_generated/cli-versions-release-options.md

With `--format json`, it writes the document its [schema](../contracts/json-reports/versions.md#ascribe-versions-release) describes:

.The JSON for this example
@details:
@snippet {lang=json, phrases=true}: code:crates/ascribe-cli/tests/output/versions-release.json
@end

| Code | Meaning |
|---|---|
| `0` | Released, or with `--dry-run`, it would be |
| `2` | Nothing was written: the line can't be released, a mark can't be rewritten, a step that should change no output changes some, or git can't put the files back and there's no `--force` |

### `ascribe versions export`
@id {permalink=true}: cli-versions-export
@summary: Renders a line into a folder of HTML that needs nothing else, with a sidebar, a search box, and a contents page, working from a disk with no server.

Renders a line, live or planned, into a folder of HTML that needs nothing else: every page of the line with a statement of what it is, a sidebar with a search box, a contents page, and a print stylesheet, marked so crawlers leave it alone, working from a disk with no server. Its archive is a copy of the line to host once the line leaves the window; exporting a line doesn't make it archived, which [`retire`](#cli-versions-retire) does. [Export and retire a line](archive.md#archive-a-line) says what's in it, and the [archive contract](../contracts/archive.md) its layout. In a copy of `examples/quill`, `ascribe versions export "self-managed 3.4" --to archive/3.4 --current https://docs.quill.dev/` writes the archive and says what to add to `ascribe.toml` once it's hosted:

@snippet {lang=text}: code:crates/ascribe-cli/tests/output/versions-export.txt

`--to` is from the current folder, and is a folder that's empty or one an archive was written in: writing again replaces the files its manifest lists, and nothing else. It refuses a folder with anything else in it, writing nothing there. It writes nothing into the project, runs the checks for the line's build first, and refuses when they find an error. A planned line's build is checked too, though `ascribe check` checks only the live lines, so the error names the build it found them in. Exporting the current line works, and the text says it's unusual. A link to a page the line doesn't have is written as its text, and the text names each one. The text says how large the search index is, and when it's over 8 MB, that a reader's first search waits for it to load; it's written whatever its size.

@available: ascribe planned
It was `ascribe versions archive` until this release, which named the copy for the state the line reaches only once it's retired. The old name still runs it, with a warning, for one more release, so a script or CI job that names it keeps working until you change it.

@include: ../_generated/cli-versions-export-options.md

With `--format json`, it writes the document its [schema](../contracts/json-reports/versions.md#ascribe-versions-export) describes:

.The JSON for this example
@details:
@snippet {lang=json, phrases=true}: code:crates/ascribe-cli/tests/output/versions-export.json
@end

| Code | Meaning |
|---|---|
| `0` | The archive is written |
| `2` | Nothing was written: the project doesn't load, the line isn't declared or is archived, no build or more than one is versioned by its target and `--build` doesn't say which, the checks find an error, or the folder isn't empty and isn't an archive |

### `ascribe versions retire`
@id {permalink=true}: cli-versions-retire
@summary: Takes the oldest live line out of the window once its archive is hosted, rewrites the marks for its target, and says where each of its addresses should go.

Takes the oldest live line out of the window once its archive is hosted: moves it from `live` to `archived` with its address and the day it ended, rewrites each mark for its target to the simplest spec that means the same in every line still live or planned, lists what no line has any more, and says where each of the line's addresses should go. [Export and retire a line](archive.md#retire-a-line) has each step. It edits `ascribe.toml` in place, keeping its comments and order, and each mark where it's written. In `examples/quill`, `ascribe versions retire "self-managed 3.4" --url https://archive.quill.dev/3.4/ --dry-run --show-unused` says what it would do:

@snippet {lang=text}: code:crates/ascribe-cli/tests/output/versions-retire.txt

The line's `ended` is its date in `ends`, which goes, or, when it had none, [the day the checks run as](../checks/check.md#checking-as-of-another-day). Its address is `--url`, the one its archive is read at.

It builds the project in memory before and after, and refuses when a live line's output changes: a line's output never showed a version from before the window, so the rewrite is meant to change none. A build that isn't a line of the target can change what it shows: a badge build shows those versions, and a filter build for an older version gains the content a mark no longer limits to later ones. So it names each such build's pages, as those it adds, removes, and changes, and the rewrite needs `--force` to go ahead. A page whose only change is the target's part of its availability lines, such as a cloud build's `self-managed (preview, 3.4+)` becoming `self-managed (preview)`, is the retirement doing its job: it's listed under its own heading, as `expected` in the JSON, and needs no `--force`. Any other change does, another target's part of the same line included. It writes only files git can put back as they were, as [`release`](#cli-versions-release) does, or anything with `--force`.

It refuses a line that isn't live, which is why running it twice changes nothing the second time; the current line, and a live line that isn't the oldest; a line with no `--url`, since after the clean-up the source no longer holds what the line needs, so [`ascribe versions export`](#cli-versions-export) comes first; and a target no build is versioned by, whose list `release` already keeps to one live line. Like `release`, it's made in the project that writes the lines, when `[from]` takes them from there.

It deletes nothing unless it's told to. A page, section, block, or table row that no live or planned line has is listed with its file, line, and first line, and so is a mark that says nothing once rewritten: one for every target its scope allows, generally available, with nothing inside it relying on it. Such a mark still draws a badge, so the rewrite keeps it. `--show-unused` lists each, and `--remove-unused` deletes them, along with the `replaces` line of a page that replaced one it deletes. Both work on a line retired already.

Each address the line published goes to the same page in the current line, to the page that replaces it there, or to its page in the archive when the current line has neither. The text names the replaced pages, and the JSON lists every address. In a project with [`ascribe.addresses`](../publishing/redirects.md), each address that goes to a page in the current line is recorded there as a `[moved]` entry keyed by the address, so the build writes its redirect, and the text says how many; the JSON's `recorded` says whether they were. The archive's addresses aren't recorded, since an entry leads to a page of the project, and without the file nothing is: those are yours to add to the host's redirects.

@include: ../_generated/cli-versions-retire-options.md

With `--format json`, it writes the document its [schema](../contracts/json-reports/versions.md#ascribe-versions-retire) describes, here as of 16 October 2026:

.The JSON for this example
@details:
@snippet {lang=json, phrases=true}: code:crates/ascribe-cli/tests/output/versions-retire.json
@end

| Code | Meaning |
|---|---|
| `0` | Retired, or with `--dry-run`, it would be; or, on a line retired already, what it left is listed or deleted |
| `2` | Nothing was written: the line can't be retired, it has no address, a mark can't be rewritten, a live line's output changes, a build that isn't a line changes and there's no `--force`, or git can't put the files back and there's no `--force` |
