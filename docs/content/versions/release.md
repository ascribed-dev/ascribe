---
type: reference
title: "`ascribe versions`"
description: "List a project's release lines, say what differs between two, release a line in one command, archive one into a folder of HTML, and retire it from the source."
---

The commands for a project's [release lines](lines.md): what they are, what differs between two, the release that makes a planned line live, the archive that keeps a line once it leaves, and the retirement that takes it out of the source. [Publish a site for each version](index.md) and [Archive and retire a line](archive.md) are the tasks they're used in. The options every command accepts are in the [command reference](../command/index.md#options-every-command-accepts), and the JSON each writes with `--format json` is in [JSON reports: `ascribe versions`](../contracts/json-reports/versions.md).

## `ascribe versions`
@id {permalink=true}: cli-versions

Lists a project's release lines, says what differs between two, releases one, archives one, and retires one. Each reads the lines from `ascribe.toml`'s [`[releases]`](../content-model/content.md#71-releasestarget), and a line is named as a mark names it: the target, then the version, `"self-managed 3.4"`.

### `ascribe versions list`
@id {permalink=true}: cli-versions-list

Shows each target's lines, newest first: whether each is current, live, planned, or archived, the output and base path of each live line on each build versioned by the target, with how many pages it publishes, the last day of support from `ends`, and an archived line's address and the day it ended. It reads the project and builds nothing. In `examples/quill`:

@snippet {lang=text}: code:crates/ascribe-cli/tests/output/versions-list.txt

@include: ../_generated/cli-versions-list-options.md

With `--format json`, the same, as its [schema](../contracts/json-reports/versions.md#ascribe-versions-list) has it. It exits with `0`, and `2` when the project doesn't load.

.The JSON for this example
@details:
@snippet {lang=json, phrases=true}: code:crates/ascribe-cli/tests/output/versions-list.json
@end

### `ascribe versions changes`
@id {permalink=true}: cli-versions-changes

Says what differs between two lines of a target, from the marks alone, with nothing built: the pages, sections, blocks, and table rows that are in the second line and not the first (added), in the first and not the second (removed), and in both with a different state, such as deprecated in one (changed). Each is listed with its file and line, the page, the heading it's under, and its first line. A page whose own mark adds it is one entry, not an entry for each part of it. A page added in place of one removed, which `replaces` links it with either way, is one entry too: it's listed as added, and names the page it's in place of, which isn't listed as removed. `ascribe versions changes "self-managed 3.4" 3.5` in `examples/quill`:

@snippet {lang=text}: code:crates/ascribe-cli/tests/output/versions-changes.txt

The second line is a version of the first's target, and either may come first. Each must be declared in the list, whether live, planned, or archived. A line holds what's marked for a version in it, not what's marked for a later one, so a line is compared as it was released, whichever is current.

@include: ../_generated/cli-versions-changes-options.md

`--format markdown` writes a first draft of release notes: a heading for the line, and one for each page, with what's new, gone, and changed on it.

@snippet {lang=markdown}: code:crates/ascribe-cli/tests/output/versions-notes.md

With `--format json`, as its [schema](../contracts/json-reports/versions.md#ascribe-versions-changes) has it:

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

Releases a line: rewrites the marks that name it through a feature, moves the feature on, makes the line live, and archives the old current line where no build is versioned by the target. [When a line ships](index.md#release-day) has each step. It edits `ascribe.toml` in place, keeping its comments and order, and each mark where it's written. In a copy of `examples/quill` that plans 3.6, with a section marked `@available: sso` and `[features.sso]` available in `self-managed 3.6`, `ascribe versions release "self-managed 3.6" --next 3.7 --dry-run` says what it would do. Its site is versioned, so 3.5 stays live, with an output of its own, `site@3.5`:

@snippet {lang=text}: code:crates/ascribe-cli/tests/output/versions-release.txt

It builds the project in memory before and after, and refuses when rewriting the marks or archiving the old line would change what a reader sees: both are meant to change nothing, so a change there is a mark the rewrite got wrong. What the release itself changes, each page whose content was planned and is now live, it lists by build. It writes nothing when it refuses, and `--dry-run` says what it would do and writes nothing.

It writes only files git can put back as they were: `ascribe.toml` and each page it rewrites must be tracked, and unchanged since the last commit. So the release is one commit to read, and `git checkout` undoes it. With `--force` it writes them anyway, as it must outside a repository.

A line that isn't planned can be released too, as long as it falls between the current line and the oldest planned one: a patch release, `3.5.1` when the lines are written as patches, needs no entry first. A feature that names the next planned line keeps naming it.

It refuses a line that's live or archived already, so running it twice changes nothing the second time; a planned line with an older one planned before it; and a line a feature names when no planned line follows it and there's no `--next`. A mark it can't rewrite where it's written, such as a frontmatter `available:` that spans lines, is named with its file and line, to write on one line first.

@include: ../_generated/cli-versions-release-options.md

With `--format json`, as its [schema](../contracts/json-reports/versions.md#ascribe-versions-release) has it:

.The JSON for this example
@details:
@snippet {lang=json, phrases=true}: code:crates/ascribe-cli/tests/output/versions-release.json
@end

| Code | Meaning |
|---|---|
| `0` | Released, or with `--dry-run`, it would be |
| `2` | Nothing was written: the line can't be released, a mark can't be rewritten, a step that should change no output changes some, or git can't put the files back and there's no `--force` |

### `ascribe versions archive`
@id {permalink=true}: cli-versions-archive

Renders a line, live or planned, into a folder of HTML that needs nothing else: every page of the line with a statement of what it is, a sidebar with a search box, a contents page, and a print stylesheet, marked so crawlers leave it alone, working from a disk with no server. [Archive and retire a line](archive.md#archive-a-line) says what's in it, and the [archive contract](../contracts/archive.md) its layout. In a copy of `examples/quill`, `ascribe versions archive "self-managed 3.4" --to archive/3.4 --current https://docs.quill.dev/` writes the archive and says what to add to `ascribe.toml` once it's hosted:

@snippet {lang=text}: code:crates/ascribe-cli/tests/output/versions-archive.txt

`--to` is from the current folder, and is a folder that's empty or one an archive was written in: writing again replaces the files its manifest lists, and nothing else. It refuses a folder with anything else in it, writing nothing there. It writes nothing into the project, runs the checks for the line's build first, and refuses when they find an error. A planned line's build is checked too, though `ascribe check` checks only the live lines, so the error names the build it found them in. Archiving the current line works, and the text says it's unusual. A link to a page the line doesn't have is written as its text, and the text names each one. The text says how large the search index is, and when it's over 8 MB, that a reader's first search waits for it to load; it's written whatever its size.

@include: ../_generated/cli-versions-archive-options.md

With `--format json`, as its [schema](../contracts/json-reports/versions.md#ascribe-versions-archive) has it:

.The JSON for this example
@details:
@snippet {lang=json, phrases=true}: code:crates/ascribe-cli/tests/output/versions-archive.json
@end

| Code | Meaning |
|---|---|
| `0` | The archive is written |
| `2` | Nothing was written: the project doesn't load, the line isn't declared or is archived, no build or more than one is versioned by its target and `--build` doesn't say which, the checks find an error, or the folder isn't empty and isn't an archive |

### `ascribe versions retire`
@id {permalink=true}: cli-versions-retire

Takes the oldest live line out of the window once its archive is hosted: moves it from `live` to `archived` with its address and the day it ended, rewrites each mark for its target to the simplest spec that means the same in every line still live or planned, lists what no line has any more, and says where each of the line's addresses should go. [Archive and retire a line](archive.md#retire-a-line) has each step. It edits `ascribe.toml` in place, keeping its comments and order, and each mark where it's written. In `examples/quill`, `ascribe versions retire "self-managed 3.4" --url https://archive.quill.dev/3.4/ --dry-run --show-unused` says what it would do:

@snippet {lang=text}: code:crates/ascribe-cli/tests/output/versions-retire.txt

The line's `ended` is its date in `ends`, which goes, or today when it had none: UTC, or `ASCRIBE_TODAY` when it's set, as the checks take it. Its address is `--url`, the one its archive is read at.

It builds the project in memory before and after, and refuses when a live line's output changes: a line's output never showed a version from before the window, so the rewrite is meant to change none. A build that isn't a line of the target can change what it shows: a badge build shows those versions, and a filter build for an older version gains the content a mark no longer limits to later ones. So it names each such build's pages, as those it adds, removes, and changes, and the rewrite needs `--force` to go ahead. It writes only files git can put back as they were, as [`release`](#cli-versions-release) does, or anything with `--force`.

It refuses a line that isn't live, which is why running it twice changes nothing the second time; the current line, and a live line that isn't the oldest; a line with no `--url`, since after the clean-up the source no longer holds what the line needs, so [`ascribe versions archive`](#cli-versions-archive) comes first; and a target no build is versioned by, whose list `release` already keeps to one live line.

It deletes nothing unless it's told to. A page, section, block, or table row that no live or planned line has is listed with its file, line, and first line, and so is a mark that says nothing once rewritten: one for every target its scope allows, generally available, with nothing inside it relying on it. Such a mark still draws a badge, so the rewrite keeps it. `--show-unused` lists each, and `--remove-unused` deletes them, along with the `replaces` line of a page that replaced one it deletes. Both work on a line retired already.

The addresses are listed and not written: a host's redirects file is yours to write. Each address the line published goes to the same page in the current line, to the page that replaces it there, or to its page in the archive when the current line has neither. The text names the replaced pages, and the JSON lists every address.

@include: ../_generated/cli-versions-retire-options.md

With `--format json`, as its [schema](../contracts/json-reports/versions.md#ascribe-versions-retire) has it, written here as of 16 October 2026:

.The JSON for this example
@details:
@snippet {lang=json, phrases=true}: code:crates/ascribe-cli/tests/output/versions-retire.json
@end

| Code | Meaning |
|---|---|
| `0` | Retired, or with `--dry-run`, it would be; or, on a line retired already, what it left is listed or deleted |
| `2` | Nothing was written: the line can't be retired, it has no address, a mark can't be rewritten, a live line's output changes, a build that isn't a line changes and there's no `--force`, or git can't put the files back and there's no `--force` |
