---
type: reference
title: "`ascribe sources` and `observe`"
description: "Fetch and update the code a project copies from other repositories, and record what tested a folder's examples."
---

The commands that look after the code a project's pages show: the copies it keeps of its sources in other repositories, and the record of what tested its examples. [Docs in another repository](docs-repository.md) and [Tested examples](tested.md) are the tasks they're used in. The options every command accepts are in the [command reference](../command/index.md#options-every-command-accepts), and the JSON each writes with `--format json` is in [their JSON reports' schemas](../contracts/json-reports/sources.md).

## `ascribe sources`
@id {permalink=true}: cli-sources
@summary: Copies code from sources in other repositories into the project, and moves their pins. They're the only commands that reach another repository, with `git`'s own credentials.

Copies code from [sources in other repositories](../content-model/sources.md#a-source-in-another-repository) into the project, and moves their pins. `check` and `build` read only the copies, so `ascribe sources fetch` and `ascribe sources update` are the only commands that reach another repository. They run `git`, with `git`'s own credentials: whatever lets `git fetch <url>` work in your shell lets them fetch. They never prompt, so a missing credential fails at once.

Fetched repositories are kept in a cache outside the project, one per URL, so a second run fetches only what's new: `ascribe` in your cache folder (`$XDG_CACHE_HOME`, `~/.cache`, `~/Library/Caches`, or `%LOCALAPPDATA%`), or `ASCRIBE_CACHE_DIR`. Nothing needs the cache but these commands; deleting it costs only a slower next run.

What's fetched is someone else's repository, so it's only ever read as text: `git` runs with no hooks, no submodules, and only the `https`, `http`, `ssh`, `git`, and `file` transports, and a file over 1 MB, or one that isn't text, isn't copied.

### `ascribe sources fetch`
@id {permalink=true}: cli-sources-fetch
@summary: Makes the copies match `ascribe.lock`: at each source's pin, it copies the files snippets name and removes the ones no snippet names. It never moves a pin.

Makes the copies match `ascribe.lock`: at each source's pin, it copies the files snippets name that have no copy or a wrong one, and removes the copies no snippet names. It's what you run after writing a new `@snippet` from a source in another repository, or after cloning if the copies were left out. It never moves a pin; a source with no pin yet is pinned to the head of its branch.

Beside the files snippets name, it copies the repository's [tested examples](tested.md#when-the-code-is-in-another-repository) config, `ascribe.examples.toml`, and every file in its `examples.observations/`, whatever the source's `include` says, and the lock lists them as it does every copy. They're read as text, and nothing in them is run. A source whose copies are all in order is left alone, without reaching its repository, so a pin made by an `ascribe` that didn't copy them gets them with the next `update`, or the next time a copy is fetched.

@include: ../_generated/cli-sources-fetch-options.md

It lists each copy it wrote or removed. The first time a source's files are copied, it says so, on standard error: the copies are committed with the docs, so everyone who can read the docs repository can read them. In a new project with one snippet from `api`:

@snippet {lang=text}: code:crates/ascribe-cli/tests/output/sources-fetch.txt

| Code | Meaning |
|---|---|
| `0` | The copies match the lock |
| `1` | A file a snippet names couldn't be copied: it isn't in the repository at the pin, it's over 1 MB, or it isn't text. The reason goes to standard error, and `ascribe check` reports the snippet. |
| `2` | It couldn't run: no `ascribe.toml`, a content model with errors, an `ascribe.lock` it can't read, a name that isn't a source in another repository, or `git` failing (no network, no access), with the source named and `git`'s own message |

### `ascribe sources update`
@id {permalink=true}: cli-sources-update
@summary: Moves each source's pin to the head of its branch, or to `--to`, copies the files snippets use again, rewrites `ascribe.lock`, and says what changed.

Moves each source's pin to the head of its `branch` (or to `--to`), copies the files snippets use again at the new commit, rewrites `ascribe.lock`, and says what changed: the commits between the old pin and the new (how many, and the first lines of the newest 20, or that the pin moved back), the copies that changed, the [tested examples](tested.md#when-the-code-is-in-another-repository) whose status changed, and the pages whose examples changed, grouped as [`ascribe drift`](../review/diff.md#cli-drift) groups them. With nothing to move, it changes no file and says so.

A tested example snippets use is listed when the new pin changed its status: it went stale (the file changed, and its observation didn't follow), no observation lists it any more, or it's tested as it is now. Each is read from the copies, before and after.

@include: ../_generated/cli-sources-update-options.md

@snippet {lang=text}: code:crates/ascribe-cli/tests/output/sources-update.txt

The pages are found as `ascribe drift` finds them, comparing the working tree, with the new copies, against `HEAD`, so run it in a clean checkout. A snippet whose region is gone at the new commit is listed among the examples that no longer resolve, and the update still completes: `ascribe check` then fails on it, which is the signal to fix the page. When the project isn't in a git repository, or has no commit yet, the pages aren't listed, and it says why: commit the docs once before the first update.

`--format markdown` writes Markdown for a pull request's description: each source that moved, with its commits, its copies, and its tested examples whose status changed, then the pages, each linked to its route on `[consumer] site`. For a repository on github.com, the old and new pins link to GitHub's comparison of them. Commit subjects are code spans, so a subject like `Fixes #12 @someone` in the code's repository can't close an issue or notify anyone from the docs' pull request. It writes nothing when nothing moved. `--format json` writes one document, as its [schema](../contracts/json-reports/sources.md#ascribe-sources-update) has it; its `pages` are as the [drift JSON](../review/diff.md#drift-json) has them.

.The JSON for this example
@details:
@snippet {lang=json}: code:crates/ascribe-cli/tests/output/sources-update.json
@end

| Code | Meaning |
|---|---|
| `0` | It ran, whether or not a pin moved. A file a snippet names that couldn't be copied is reported on standard error. |
| `2` | It couldn't: no network, no access, an unknown revision, `--to` with more than one source, or the reasons `fetch` gives. The source is named, with `git`'s own message. |

### `ascribe sources status`
@id {permalink=true}: cli-sources-status
@summary: Shows each source in another repository, with its branch, its pin, and each copy's state. It reads only the project's files: no `git`, no network.

Shows each source in another repository: its repository and branch, its pin, and each copy's state (current, changed here, missing, not in the lock, unused, not copied yet, or not in the repository at the pin). A tested examples' config or observation is current when it's the file the lock pins, though no snippet uses it. It reads only the project's files: no `git`, no network. After the first `fetch`:

@snippet {lang=text}: code:crates/ascribe-cli/tests/output/sources-status.txt

@include: ../_generated/cli-sources-status-options.md

With `--format json`, it writes the same in the document its [schema](../contracts/json-reports/sources.md#ascribe-sources-status) describes:

.The JSON for this example
@details:
@snippet {lang=json}: code:crates/ascribe-cli/tests/output/sources-status.json
@end

| Code | Meaning |
|---|---|
| `0` | It showed the sources, whatever state their copies are in |
| `2` | It couldn't run: a usage error, no `ascribe.toml`, or one with errors |

## `ascribe observe`
@id {permalink=true}: cli-observe
@summary: Runs the test of a folder of tested examples, and when it passes, writes down what it passed with, in `examples.observations/`.

Runs the test of a folder of [tested examples](tested.md), and when it passes, writes down what it passed with: `examples.observations/default.toml`, beside `ascribe.examples.toml`, or `examples.observations/<name>.toml` with `--name`. It's run in the code's folder, the one that holds `ascribe.examples.toml`, not the docs project's: it doesn't look in parent folders for a config, and doesn't read `ascribe.toml` or `--config`. For code in another repository, that's the code's repository: it refuses to run in a source's copies, which it knows from the `ascribe.lock` above them. So `ascribe` has to be installed where the code is: with npm, or on the path where there's no Node ([Install](../install.md)).

@include: ../_generated/cli-observe-options.md

It prints the config's path and every command it will run. When its standard input is a terminal, it then asks **Run them? [y/N]**, on standard error, and runs nothing unless the answer is `y` or `yes`; with `--format json`, the list goes to standard error too, so standard output holds the JSON alone. `--yes` skips the question. Where standard input isn't a terminal, as in CI, a hook, or an agent's shell, nobody is there to answer, so it lists the commands, runs nothing, and exits 2 unless given `--yes`.

Then it runs each `[environment]` command, then `test`, through the platform's shell (`sh`, or `cmd` on Windows), in the folder, with your environment and standard input closed. The test's output goes to the terminal as it comes, and there's no time limit: you started it, and you can stop it. Last, it names the observation it wrote, with how many files and what it recorded of the environment.

When `test` exits 0, it hashes every example and writes the observation whole, to a file beside it that it then renames, so a run that's stopped part way leaves the old one. When it doesn't, it writes nothing. A file that changes while the test runs is hashed as it is at the end, so run it on a clean tree. When the observation replaces one, it says what changed since: the files added, removed, and changed, and the environment values that differ. An observation of another name is never touched.

With `--check`, it runs nothing: not `test`, not the `[environment]` commands. It hashes the examples as they are, and compares them with the observation `--name` names, `default` without it, so a pre-commit hook can run it. In [`examples/tested/code`]({repo}/tree/main/examples/tested/code), after an example changes:

@snippet {lang=text}: code:crates/ascribe-cli/tests/output/observe-check.txt

| Code | Meaning |
|---|---|
| `0` | The test passed, and the observation is written |
| `1` | The test failed. Nothing was written, and the old observation is as it was. |
| `2` | It couldn't run: a `--name` that isn't one; the current folder is a source's copies (`sources/<name>/` of a project whose `ascribe.lock` pins it), whose config is never run; no `ascribe.examples.toml` in the current folder, one with errors or with no `test`, the question answered with anything but yes, no terminal to ask at and no `--yes`, an `[environment]` command that failed or printed nothing, no shell, or an observation it couldn't write |

With `--check`, `0` means the observation is current, `1` that it's stale or missing, and `2` that the config, the examples, or the observation can't be read.

With `--format json`, what the commands print goes to standard error, and standard output has one document, which its [schema](../contracts/json-reports/sources.md#ascribe-observe) describes. When the test fails, `observation`, `recorded`, `files`, `tested`, and `changes` are empty. With `--check`, `check` is `true` and `test` is null:

.The JSON for this example
@details:
@snippet {lang=json}: code:crates/ascribe-cli/tests/output/observe-check.json
@end
