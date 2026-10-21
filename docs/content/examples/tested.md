---
type: how-to
title: Show what an example was tested with
description: Run a folder's tests with ascribe observe, commit what it records, and have each page say what its examples were tested with, and warn when one changed after its last test run.
---

A snippet keeps an example on the page the same as the file, but the page can't say whether the file still works. With a test for the examples and `ascribe observe`, a code block says what it was tested with, "Tested with Node 24.9.0 on Linux", and `ascribe check` says when an example changed after its last test run.

Ascribe doesn't test the code. It runs the command you give it, and when that passes, writes down where it ran and what the examples were. Which examples the command really exercises is up to your test.

[`examples/tested`]({repo}/tree/main/examples/tested) in this repository is a small project set up this way: a TypeScript example, the output it should print, and a test with Node's own test runner that compares the two.

These docs are set up this way too. The command output they show is a file a test compares with what the command prints, and [`ascribe.examples.toml`]({repo}/blob/main/ascribe.examples.toml) at the repository's root names those files and the tests, so each such block says which Rust it was tested with. The source and configuration the docs quote aren't in it: nothing tests them, so they show no tested line. The docs raise `example-stale` to an error, so they can't publish a tested line that's no longer true.

## 1. Say which files are examples

In the source's folder, the one `[sources.<name>]` in `ascribe.toml` names with `path`, add `ascribe.examples.toml`:

@snippet {lang=toml}: code:examples/tested/code/ascribe.examples.toml

- **`include`** lists the files that are examples, by the same patterns as a source's `include`, from this folder. **`ignore`** takes back files `include` matches.
- **`test`** is a line of shell that tests them. It runs in this folder, as written, with `sh`, or `cmd` on Windows.
- **`[environment]`** names what a reader wants to know about where the test ran. Each entry has a `command` whose first line is kept, and a `label`, the key when it has none. The page shows the label and the first dotted number in the line: `node --version` prints `v24.9.0`, shown as "Node 24.9.0". The operating system and architecture are always recorded, and only the system is shown.

An expected output is an example like any other: show it with a second `@snippet`, and have the test compare the program's output with it.

## 2. Run the test and commit what it records

In the same folder, run [`ascribe observe`](sources.md#cli-observe). It names the commands it will run, runs them, and when the test passes, writes `examples.observations/default.toml`: where the test ran, and a hash of every example.

```console
$ ascribe observe
Running the commands in ascribe.examples.toml:
  test         node --test test/greet.test.mjs
  environment  node --version
node --test test/greet.test.mjs passed.
Wrote examples.observations/default.toml: 2 files, Node 24.9.0 on Linux.
```

Commit the observation with the code, as you would a lockfile. When the test fails, nothing is written, and the old observation stays. When it replaces one, it says what changed since: the files added, removed, and changed, and the environment values that differ.

`ascribe observe` runs where the code is, so `ascribe` has to be installed there: with npm, or [without Node](../install.md#without-node) from Homebrew or the install script. Run it on a clean tree: the files are hashed after the test, as they are then.

## 3. Show the example

Nothing changes in the page. A `@snippet` of an example the observation lists, as the file is now, gets a caption on the site, and a line under it in the plain output:

```markdown
@snippet: code:examples/greet.ts#greet
```

On the site, "Tested with" comes from the element library's stylesheet, so a site can translate it.

## Test under more than one environment

Run `ascribe observe --name` once in each environment, and commit each observation. `--name node-22` writes `examples.observations/node-22.toml`; a name is lowercase words of letters and digits joined by `-`. Each observation is whole and separate, with its own environment and its own hashes, and running one doesn't touch another.

```console
$ ascribe observe --name node-22
…
Wrote examples.observations/node-22.toml: 2 files, Node 22.4.0 on Linux.
```

A page names every environment whose observation lists the file as it is: "Tested with Node 22.4.0 and 24.9.0 on Linux". Each entry is said once, with each of its values in version order, and the operating systems are merged the same way, "on Linux and macOS". Past three values, the line says how many there are and the lowest and highest: "Node, 5 versions from 18.20.0 to 24.9.0". The [JSON output](../contracts/tested-examples.md#6-in-the-json-output) has every one.

`examples/tested` has two observations, `default` with the Node in the repository's `.nvmrc` and `node-22` with Node 22, and CI writes both again on every platform to compare their files with the committed ones.

An observation nothing writes any more stays until you delete its file. It goes stale like any other, and a page doesn't name it.

## When an example changes

An edit to an example, anywhere in the file, makes its observations out of date. A change to a file an example only calls, such as a library, doesn't: only the example files are compared. The page then drops the tested line, and `ascribe check` reports [`example-stale`](../checks/diagnostics/content-checks.md#ASC199) at each `@snippet` of the file, naming the folder to run `ascribe observe` in. An example no observation lists reports `example-unobserved`. Both are advice: the pages still build. Run `ascribe observe` again and commit, and both clear. With two observations, running one again brings the page back with that one's environment, and the other is still out of date until it's run again too: `ascribe model --section sources` says how many of each observation's files are as it found them.

An example that's deleted or renamed without a second run leaves its observation listing a file that isn't there. `ascribe check` reports [`observation-file-missing`](../checks/diagnostics/content-checks.md#ASC202) at that line of the observation, naming the command that writes it again.

In VS Code, the advice is on the `@snippet` line as you write, and a hover on the address says what the file was tested with, or why it says nothing, and which observations say so. The quick fix **Copy the command** copies the line that runs `ascribe observe` in the example's folder; the editor never runs it. When an observation is written, the advice follows without the page being edited. See [the editor](../editor/writing.md#hover).

@available: ascribe 0.4.0
In the example itself, a lens over each region a page shows says how many pages show it, "Shown in 1 page", and lists them, so you know which pages a change will change before you make it. See [Code files a page shows](../editor/writing.md#code-files-a-page-shows).

To check before a commit that an observation still matches the folder, run `ascribe observe --check`, with `--name` for one other than `default`. It runs nothing, not the test and not the environment commands, and exits 1 when the observation is stale or missing, so a pre-commit hook can run it. It's stricter than the pages: a file the config's `include` no longer matches, or one that's gone, counts as a change, though the files still in the observation stay current until the next run.

## When the code is in another repository

For a [source in another repository](docs-repository.md), the config and the observation stay in the code's repository, beside the examples, and `ascribe observe` runs there. [`ascribe sources fetch`](sources.md#cli-sources-fetch) copies both into `sources/<name>/` with the files your snippets use, whatever the source's `include` says, and `ascribe.lock` lists them as it does every copy. The pages then say what each example was tested with, as they do for a folder in the same repository.

- **An update says what changed.** When [`ascribe sources update`](sources.md#cli-sources-update) moves a pin, its summary names the examples that went stale (the file changed and its observation didn't follow), the ones the new observation doesn't list, and the ones tested again. The [update pull request](docs-repository.md#the-update-pull-request) carries the same lines.
- **The fix is in the code's repository.** `example-stale` and `example-unobserved` say so: run `ascribe observe` there and commit, and the next update brings the observation here.
- **Nothing here runs the config.** Its `test` and commands are read as text and never run, and `ascribe observe` refuses to run in `sources/<name>/`.

## What it doesn't do yet

- **The run is the unit.** Every file `include` matches is recorded as passing when the test passes, whether or not the test touched it, and one failing example blocks the whole run.

## Start from a repository

Two repositories have all of this set up, to copy whole: examples as plain programs, each with the output it prints beside it, a test for each, the config, an observation for each of two versions of the runtime, and docs that show every example and its output with a tested line naming both. Their CI runs the tests weekly and on every pull request, builds the docs, and fails a pull request that changes an example without observing it again, with `ascribe observe --check`.

- [**tested-examples-node**](https://github.com/ascribed-dev/tested-examples-node): Node with TypeScript, tested with Node's own test runner, with `ascribe` from npm.
- [**tested-examples-python**](https://github.com/ascribed-dev/tested-examples-python): Python, tested with pytest, with `ascribe` from the install script and no Node.

Copy one, replace its examples with yours, and run `ascribe observe --name` once under each version you test. Its README says how to add an example, and what changes when the docs live in another repository.

## See also

- [`ascribe observe`](sources.md#cli-observe): its options, exit codes, and JSON.
- [Tested examples](../contracts/tested-examples.md): the two files' formats.
- [Sources](../content-model/output.md#18-sourcesname): declaring the folder the examples are in.
- [The report's inventory](../checks/report.md#inventory): how many of a project's snippets show a tested example, and how many of those are current, stale, and unobserved.
