---
type: reference
title: "`ascribe versions`"
description: "List a project's release lines, say what differs between two, release a line in one command, and archive one into a folder of HTML."
available: next
---

The commands for a project's [release lines](index.md): what they are, what differs between two, the release that makes a planned line live, and the archive that keeps a line once it leaves. The options every command accepts are in the [command reference](../command/index.md#options-every-command-accepts).

## Release day

When a line ships, the docs follow it in one command:

```sh
ascribe versions release "self-managed 3.6"
```

Before it, the content written for 3.6 says so through a [feature](../content-model/content.md): `[features.next] available = "self-managed 3.6"`, and marks like `@available: next` on the pages. The release:

1. Rewrites each mark that names a feature whose version is in 3.6 to the version itself, `@available: self-managed 3.6`, so the content stays marked for 3.6 once the feature moves on.
2. Points each such feature at the next planned line, or at `--next`, which it also adds to `planned`, so what's written for the next release can keep using it.
3. Moves 3.6 from `planned` to `live`.
4. When no build is [versioned](index.md#build-a-site-for-each-line) by the target, moves the line that was current to `archived`, with no address: such a project publishes one line, and a reader can't be on the old one any more. Its date in `ends`, if it has one, becomes its `ended`. A project that builds a site for each line keeps the old line live, until you archive it.

It builds the project in memory before and after, and refuses when rewriting the marks or archiving the old line would change what a reader sees: both are meant to change nothing, so a change there is a mark the rewrite got wrong. What the release itself changes, each page whose content was planned and is now live, it lists by build. It writes nothing when it refuses, and `--dry-run` says what it would do and writes nothing.

It writes only files git can put back as they were: `ascribe.toml` and each page it rewrites must be tracked, and unchanged since the last commit. So the release is one commit to read, and `git checkout` undoes it. With `--force` it writes them anyway, as it must outside a repository.

A line that isn't planned can be released too, as long as it falls between the current line and the oldest planned one: a patch release, `3.5.1` when the lines are written as patches, needs no entry first. A feature that names the next planned line keeps naming it.

Then, to start the release notes, see what the new line has that the one before didn't:

```sh
ascribe versions changes "self-managed 3.5" 3.6 --format markdown
```

## `ascribe versions`
@id {permalink=true}: cli-versions

Lists a project's release lines, says what differs between two, releases one, and archives one. Each reads the lines from `ascribe.toml`'s [`[releases]`](../content-model/content.md#71-releasestarget), and a line is named as a mark names it: the target, then the version, `"self-managed 3.4"`.

### `ascribe versions list`
@id {permalink=true}: cli-versions-list

Shows each target's lines, newest first: whether each is current, live, planned, or archived, the output and base path of each live line on each build versioned by the target, with how many pages it publishes, the last day of support from `ends`, and an archived line's address and the day it ended. It reads the project and builds nothing. In `examples/quill`:

@snippet {lang=text}: code:crates/ascribe-cli/tests/output/versions-list.txt

@include: ../_generated/cli-versions-list-options.md

With `--format json`, the document has `schema_version` (`1`), `ascribe_version`, and `targets`, each with `target`, `builds` (the builds versioned by it), and `lines`, newest first, each with `line`, `status` (`current`, `live`, `planned`, or `archived`), `outputs` (for a live line, each with `build`, `base`, and `pages`), and, when the line has them, `ends`, `ended`, and `url`. It exits with `0`, and `2` when the project doesn't load.

### `ascribe versions changes`
@id {permalink=true}: cli-versions-changes

Says what differs between two lines of a target, from the marks alone, with nothing built: the pages, sections, blocks, and table rows that are in the second line and not the first (added), in the first and not the second (removed), and in both with a different state, such as deprecated in one (changed). Each is listed with its file and line, the page, the heading it's under, and its first line. A page whose own mark adds it is one entry, not an entry for each part of it. A page added in place of one removed, which `replaces` links it with either way, is one entry too: it's listed as added, and names the page it's in place of, which isn't listed as removed. `ascribe versions changes "self-managed 3.4" 3.5` in `examples/quill`:

@snippet {lang=text}: code:crates/ascribe-cli/tests/output/versions-changes.txt

The second line is a version of the first's target, and either may come first. Each must be declared in the list, whether live, planned, or archived. A line holds what's marked for a version in it, not what's marked for a later one, so a line is compared as it was released, whichever is current.

@include: ../_generated/cli-versions-changes-options.md

`--format markdown` writes a first draft of release notes: a heading for the line, and one for each page, with what's new, gone, and changed on it.

@snippet {lang=markdown}: code:crates/ascribe-cli/tests/output/versions-notes.md

With `--format json`, the document has `schema_version` (`1`), `ascribe_version`, `target`, `from`, `to`, and `added`, `removed`, and `changed`, each a list with `file`, `line`, `kind` (`page`, `section`, `block`, or `row`), `page` (its title), `heading` (left out at the top of a page), `text` (its first line), for a changed one, `from_state` and `to_state`, its state in each line, such as `deprecated`, and, for a page added in place of another, `in_place_of`, with that page's `file`, `line`, and `page`.

| Code | Meaning |
|---|---|
| `0` | Compared, whether or not anything differs |
| `2` | It couldn't compare: the project doesn't load, or a line isn't named as a mark names one or isn't declared |

### `ascribe versions release`
@id {permalink=true}: cli-versions-release

Releases a line: rewrites the marks that name it through a feature, moves the feature on, makes the line live, and archives the old current line where no build is versioned by the target. [Release day](#release-day) has each step. It edits `ascribe.toml` in place, keeping its comments and order, and each mark where it's written. In a copy of `examples/quill` that plans 3.6, with a section marked `@available: sso` and `[features.sso]` available in `self-managed 3.6`, `ascribe versions release "self-managed 3.6" --next 3.7 --dry-run` says what it would do. Its site is versioned, so 3.5 stays live, with an output of its own, `site@3.5`:

@snippet {lang=text}: code:crates/ascribe-cli/tests/output/versions-release.txt

It refuses a line that's live or archived already, so running it twice changes nothing the second time; a planned line with an older one planned before it; and a line a feature names when no planned line follows it and there's no `--next`. A mark it can't rewrite where it's written, such as a frontmatter `available:` that spans lines, is named with its file and line, to write on one line first.

@include: ../_generated/cli-versions-release-options.md

With `--format json`, the document has `schema_version` (`1`), `ascribe_version`, `written` (false with `--dry-run`), `target`, `line`, `was_planned`, `next` and `next_added` (the line the features moved to, when they moved, and whether it was added to `planned`), `archived` (the line moved to archived, when one was), `features` (each with `key`, `from` and `to`, its spec before and after, and `kept`, the other targets its spec names), `files` (each with `file`, from the project's folder, and its `marks`, each with `line`, `from`, and `to`), and `outputs` (each with `build`, `output`, and the files `added`, `removed`, and `changed`).

| Code | Meaning |
|---|---|
| `0` | Released, or with `--dry-run`, it would be |
| `2` | Nothing was written: the line can't be released, a mark can't be rewritten, a step that should change no output changes some, or git can't put the files back and there's no `--force` |

### `ascribe versions archive`
@id {permalink=true}: cli-versions-archive

Renders a line, live or planned, into a folder of HTML that needs nothing else: every page of the line with a statement of what it is, a sidebar, a contents page, and a print stylesheet, marked so crawlers leave it alone, working from a disk with no server. [Archive a line](index.md#archive-a-line) says what's in it, and the [archive contract](../contracts/archive.md) its layout. In a copy of `examples/quill`, `ascribe versions archive "self-managed 3.4" --to archive/3.4 --current https://docs.quill.dev/` writes the archive and says what to add to `ascribe.toml` once it's hosted:

@snippet {lang=text}: code:crates/ascribe-cli/tests/output/versions-archive.txt

`--to` is from the current folder, and is a folder that's empty or one an archive was written in: writing again replaces the files its manifest lists, and nothing else. It refuses a folder with anything else in it, writing nothing there. It writes nothing into the project, runs the checks for the line's build first, and refuses when they find an error. A planned line's build is checked too, though `ascribe check` checks only the live lines, so the error names the build it found them in. Archiving the current line works, and the text says it's unusual. A link to a page the line doesn't have is written as its text, and the text names each one.

@include: ../_generated/cli-versions-archive-options.md

With `--format json`, the document has `schema_version` (`1`), `ascribe_version`, `target`, `line`, `status` (`current`, `live`, or `planned`), `build` (the versioned build, as declared), `folder` (from the current folder), `contents` (the contents page, from the folder), `pages`, `assets`, `removed` (files of an earlier archive there that this one doesn't have), and `unlinked` (each link written as its text, with `file`, from the project's folder, `text`, `href`, and `line`, the line it led to when it led out of the line).

| Code | Meaning |
|---|---|
| `0` | The archive is written |
| `2` | Nothing was written: the project doesn't load, the line isn't declared or is archived, no build or more than one is versioned by its target and `--build` doesn't say which, the checks find an error, or the folder isn't empty and isn't an archive |
