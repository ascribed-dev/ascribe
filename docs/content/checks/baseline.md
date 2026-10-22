---
type: reference
title: The baseline
description: "Put a project's existing advice off for now with `ascribe.baseline`, so `ascribe check` reports only what's new: the commands that keep it, and its syntax."
---

A project that turns on the [content checks](diagnostics/content-checks.md) can meet thousands of advice at once. The baseline puts them off for now: `ascribe baseline write` lists every advice the project has in `ascribe.baseline`, beside `ascribe.toml`, and from then on `ascribe check` holds those back, counts them, and reports only the advice that's new. The team fixes the backlog at its own pace, and `ascribe baseline prune` takes each fixed one out of the file.

```text
$ npx ascribe baseline write
wrote ascribe.baseline: 3412 problems in 1906 files, 3412 added; commit it with the docs
$ npx ascribe check
checked 2061 files for every build, reported on 2061: 0 errors, 0 warnings, 2 advice, 3412 in the baseline
```

Commit `ascribe.baseline` with the docs: a reviewer sees in its diff what a change puts off, one line a problem.

## What it holds back

- **Advice only.** An error makes a page wrong, and a warning is something the project has decided matters, so neither is ever put off: `ascribe baseline write` leaves them out and says how many. To put off a check's warnings, lower the check to `"advice"` in [`[checks]`](../content-model/checks.md#19-checks), then run `ascribe baseline write` again. That's a change to `ascribe.toml` a reviewer sees.
- **By the problem, not its line.** An entry names the problem's check, its file, the text it's reported at, and its order among problems alike in all three, as [the project report](report.md#whats-new-since-the-last-report) tells one problem from another. A problem stays held back when lines are added above it, and is new, and reported, when the text it's about is edited, which is usually what someone should look at.
- **Until the check is raised.** An entry whose check the project has since raised to a warning or an error doesn't hide it: the problem is reported at its level, and the entry stays, matched.
- **Not what `@intended` does.** An [acknowledgement](../writing/directives.md#intended) is one problem, kept for good, with a reason, beside its block. A baseline is many, kept for now, with none. A problem the team means to keep is acknowledged, and pruned from the baseline.

An entry that matches nothing, because its problem was fixed or its text changed, is reported, as the advice `baseline-entry-unmatched` at its line, so a stale entry can't hide a later problem whose text happens to be the same. `ascribe baseline prune` removes it. It's reported only when `ascribe check` checks the whole project for every build: with paths, `--build`, or `--editor-build`, the check doesn't see every problem the file lists.

`ascribe check --no-baseline` reports everything, as if there were no baseline, and doesn't read the file. `ascribe build`, `ascribe report`, the editor, and the agents' tools report everything too, for now.

## `ascribe baseline`
@id {permalink=true}: cli-baseline

Keeps the project's baseline, `ascribe.baseline` beside `ascribe.toml`. The JSON each writes with `--format json` is in [its schema](../contracts/json-reports.md#ascribe-baseline-write). Both run the whole check first, as `ascribe check` with no options does, without the baseline.

### `ascribe baseline write`
@id {permalink=true}: cli-baseline-write

Adds every advice `ascribe check` reports to `ascribe.baseline`, and makes the file when there's none. The entries it had stay, so a problem is held back until it's fixed and pruned. Warnings and errors are never added: it says how many it skipped, and how to put one off. It writes the whole file in its own form, sorted, so running it twice gives the same file, byte for byte, and two people adding to it rarely change the same line.

@include: ../_generated/cli-baseline-write-options.md

```text
$ npx ascribe baseline write
wrote ascribe.baseline: 3 problems in 2 files, 1 added; commit it with the docs
skipped 1 warning, since a baseline holds advice only; to put one off, lower its check to "advice" in [checks] in ascribe.toml, then run `ascribe baseline write` again
```

| Code | Meaning |
|---|---|
| `0` | It ran, whether or not the file changed |
| `2` | It couldn't run: the project can't be checked, or `ascribe.baseline` can't be read or written |

### `ascribe baseline prune`
@id {permalink=true}: cli-baseline-prune

Removes the entries of `ascribe.baseline` that match nothing the project has any more: the problems that were fixed, and those whose text changed, which `ascribe check` reports as new. It never adds one. Without a baseline, there's nothing to prune, and it writes nothing.

@include: ../_generated/cli-baseline-prune-options.md

```text
$ npx ascribe baseline prune
removed 1 entry that matched nothing from ascribe.baseline: 2 problems in 2 files left; commit it with the docs
```

| Code | Meaning |
|---|---|
| `0` | It ran, whether or not the file changed |
| `2` | It couldn't run: the project can't be checked, or `ascribe.baseline` can't be read or written |

## The file

`ascribe.baseline` is text, one line a problem, after a line that names the format and its version:

```text
# ascribe baseline 1
# Advice put off for now: ascribe baseline write adds, ascribe baseline prune removes.
# check fingerprint order file
description-too-long 3f9a2c1b9e0d4a77 0 guides/install.md
link-text-generic 91c04e2d7b5a6f18 1 guides/install.md
```

- **The first line** is `# ascribe baseline 1`: the format, and its version. A file of a later version is from a newer Ascribe.
- **A line that starts with `#` is a comment**, and a blank line is ignored. A `#` anywhere else is part of the line: in a path it's just text, and a line with spaces before its `#` is read as an entry, so it doesn't parse. `ascribe baseline write` and `prune` write the file again in their own form, with their own comment, so a comment of yours isn't kept.
- **An entry** is four fields, each separated from the next by one space: the check's name; the fingerprint, 16 lowercase hexadecimal digits; the order, a whole number from 0; and the file, last: the rest of the line after the third space, so a path with spaces needs no quoting. The file is from the project's folder, the one `ascribe.toml` is in, with `/`.
- **The entries are sorted** by file, then check, then fingerprint, then order, so a file's entries are together, and a diff reads as a list of problems.

**The fingerprint** is the first 16 hexadecimal digits of the SHA-256 of the text the problem is reported at, or of its line's text when it's reported at no text, with each run of spaces and line ends read as one space and the ends trimmed. **The order** tells apart problems with the same check, file, and fingerprint, as two links to one missing page in one paragraph: 0 for the first in the file, then 1, and so on.

A line that doesn't parse is an error that names it, and `ascribe check` doesn't run with a baseline it can't read: it exits with `2`, and says which line, as `ascribe baseline write` and `prune` do. Fix the line, or run `ascribe check --no-baseline`.
