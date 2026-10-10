# Proposal: tested examples

A proposal, not a plan. It says what the feature is and what it would look like, so the idea can be judged before anything is designed in detail. It follows the research in [Code examples tied to tested code](reports/Code%20examples%20tied%20to%20tested%20code.md).

## The goal

A code example in an Ascribe site says what it was tested with, and Ascribe knows when that stops being true.

`@snippet` already shows code from a real file, and `ascribe drift` already reports a page whose example changed. What's missing is any knowledge of the test: whether the file was tested, in what environment, and whether it has changed since.

The two answer different questions, and neither replaces the other. `drift` says whether the code a page shows changed in this pull request. An observation says whether the code a page shows was tested.

It's for projects starting clean. Compatibility with other tools' tags and layouts isn't a goal.

## What Ascribe does, and doesn't

- **It doesn't test code, and doesn't judge tests.** No test frameworks, no language toolchains, no sandbox. Whether a test is any good is the authors' responsibility. Ascribe says a command passed, and nothing more; arbitrating beyond that would be a testing product, which this isn't.
- **It wraps the project's own test command,** when asked, and writes down what passed and where.
- **It reads what was written down everywhere else.** `check`, `build`, and the editor never run anything. They compare files with the observations, which is exact and fast.

## Three pieces

### 1. A config file at the source's root

`ascribe.examples.toml`, in the folder a docs project names as a source: the `path` of a `[sources.<name>]` in the same repository, or the root of another repository.

```toml
# What counts as an example, relative to this file.
include = ["examples/**"]
ignore = ["**/node_modules/**"]

# The command that tests them. It passes or it fails.
test = "npm test"

# What to note about where it ran: a label to show, and a command that prints it.
[environment]
node = { label = "Node", command = "node --version" }
typescript = { label = "TypeScript", command = "npx tsc --version" }
```

- **One folder for everything.** A snippet's path is relative to its source's folder, and so is every path in the config and in an observation. That's what lets the docs side match a file a page shows with a file that was observed. Two sources that point at one folder share its config.
- **The `[environment]` table is how Ascribe stays out of every language's ecosystem.** The repository says how to ask for its versions, and Ascribe writes down the answers. The operating system and architecture are noted without being asked for.
- **What's kept from a command's output** is its first line. What's shown is the first dotted number in that line (`24.9.0` from `v24.9.0`, `5.9.2` from `Version 5.9.2`, `1.22.3` from `go version go1.22.3 darwin/arm64`), or the whole line when it has none. A `label` is what readers see beside it; without one, it's the key.

### 2. An observation, written by the CLI

```sh
ascribe observe                 # runs `test` from the config
ascribe observe --name node-22  # a second observation, made under another environment
```

Ascribe watches the project's command run and writes down what it saw: an **observation**. The word is chosen for the size of its claim. An observation says what happened and under what conditions, not that the code is correct. If the command passes, Ascribe writes one beside the config, as TOML, like `ascribe.toml`:

```toml
# examples.observations/default.toml. Written by `ascribe observe`. Don't edit it.
version = 1
ascribe = "0.3.0"
command = "npm test"

[environment]
os = "linux"
arch = "x64"
node = "v24.9.0"
typescript = "Version 5.9.2"

[files]
"examples/connect.out.txt" = "sha256:07fe…"
"examples/connect.ts" = "sha256:9b1c…"
```

- **A person runs it, and commits the result.** `ascribe observe` is run by hand, at the source's root, before committing, as a lockfile is updated. The observation describes that person's machine, and says so. Running it in CI is left for later.
- **One file per environment.** Without `--name` it's `default.toml`. An example observed under Node 22 and under Node 24 has two.
- **No timestamp and no commit,** so an observation changes only when the code or the environment does. It's written before the commit that will hold it, so it couldn't name that commit; the hashes are the statement.
- **Every example file is hashed,** with the hash `ascribe.lock` already uses, including expected-output files, which are examples like any other. Files are listed in order, one to a line, so two changes to different examples merge without conflict.
- **A failed run writes nothing** and leaves the old observation in place, which then no longer matches.
- **Only the example files decide whether it's stale.** An observation says these files passed this command. It goes stale when one of those files changes. A change to a library an example calls doesn't make it stale; catching that is the test's job the next time it runs, not Ascribe's.
- **It says which `ascribe` wrote it, and in what format,** so an older `ascribe` reading a newer one's file can say so instead of misreading it.

What that costs, said plainly:

- **The run is the unit.** Every file `include` matches is recorded as passing, whether or not the command touched it. Making sure the command does test them is the authors' job.
- **One failing example blocks every observation.** Nothing is written until the whole command passes, so one broken example leaves every example in the folder stale. A result for each example would lift both limits, and is the first thing listed under "Later".
- **It follows the code like a lockfile.** Any edit to an example means running `ascribe observe` again, and so does merging two branches that each edited one.

### 3. What the docs side does with it

A docs project reaches the examples through `[sources.<name>]`, as today. Observations travel with the source, by rules of their own:

- **They're read from the source's root whatever `include` says.** A source that makes only `examples/**` readable still has its observations read. No snippet can address one.
- **For a source in another repository, `ascribe sources fetch` and `update` copy them** into `sources/<name>/` with the files snippets use, and record them in `ascribe.lock`, so `check` treats them as any other copy.
- **`ascribe sources update` says what changed,** in the summary it already writes for the update pull request: the examples that went stale or lost their observation with the new pin, beside the pages whose examples changed.

Then, with nothing run:

| The observation says | The file now | Ascribe reports |
|---|---|---|
| The hash matches | Unchanged | Tested as shown, with the environment |
| The hash differs | Edited since | A stale observation: the file changed after its last test run |
| Nothing about this file | Any | Unobserved: not tested, said plainly |
| A file that isn't there | Gone | An observation of a file that no longer exists |

Three details:

- **Files, not regions.** An observation covers a whole file, since that's what was tested. An edit to a comment, or to another region, makes it stale for every snippet of the file, including one whose own text didn't change. The message says the file changed, not the example.
- **Several observations.** A file is tested as shown under each environment whose observation is current. The page names those. When none is current, the example is stale.
- **Newer than this `ascribe`.** An observation in a format this version doesn't know is reported as that, and the example is treated as unobserved.

Where it shows up:

- **On the code block.** A line in the block's caption: "Tested with Node 24.9.0 and TypeScript 5.9.2 on Linux." With several current observations: "Tested with Node 22.4.0 and 24.9.0." The page says "tested", the everyday word; "observation" is for the command and the file.
- **Carried the way a title is.** A `@snippet` becomes a fenced code block, and a title reaches both renderers and the plain output through the block's info string. The tested line travels the same way, as one more attribute there. A block with a tested line and no title still gets a caption. This is a change to the site-render and elements contracts.
- **In the JSON output,** beside the snippet's address, path, and lines, which it already carries: the status, and each current observation's environment. An agent reading the Markdown version gets the caption's line. The research found that matching versions is where agents go wrong with examples.
- **In the editor.** The language server has nothing for a `@snippet` line today. A hover there would show the file, the status, and the environment, and the stale and unobserved diagnostics appear on that line.
- **As diagnostics,** under the [content checks](content-checks/README.md) contract: stale, unobserved, and an observation of a missing file. All advisory, all able to be raised or turned off. Their next step depends on where the code is. For a source in the same repository, it's a command the author runs: `ascribe observe`. For a source in another repository, it's `outside`: the fix is made there, and arrives with the next `ascribe sources update`.
- **In `ascribe report`:** how many of a project's examples have a current observation.

One check nothing else could make: a page says a feature is available from version 3.4, and its example was tested on 3.2. Availability is already in the content model; a project would map an environment label to a dimension to turn this on.

## Shown output

The pattern that held up at scale keeps three things apart: the example as a plain function, a file of its expected output, and a test that compares the two. Ascribe needs nothing new for it:

```markdown
@snippet: code:examples/connect.ts#connect
@snippet {lang=text, title="Output"}: code:examples/connect.out.txt
```

The output file is one the test compares against, and the observation hashes it. So the output a reader sees is a tested artifact, with no execution by Ascribe.

## Example repositories

Being opinionated about testing without building a test product means showing the pattern, in a repository a team can copy. Each has:

- examples as plain functions, tagged;
- an expected-output file for each;
- tests that compare the two, in that language's usual framework;
- `ascribe.examples.toml`;
- committed observations, made with `ascribe observe`;
- a docs project that shows the examples.

They hold no custom code, since the CLI writes the observations. Start with two, Node with TypeScript and Python.

Each is a repository of its own under the organization, as the code behind `examples/docs-repository` already is, so a team can copy one whole. Each repository's own tests run in CI on a schedule, so the pattern it shows keeps working: the research's clearest warning was about sample infrastructure left to rot. That's CI running the tests, not writing observations. Each repository and its scheduled job is one more thing outside this repository to look after, recorded in [outside.md](outside.md).

## Getting `ascribe` where the examples are

Today nothing puts `ascribe` on a machine's path. The extension carries a binary for its own use, and `@ascribed/cli` puts one in a project's `node_modules`, reached with `npx`. That suits a docs project, which is a Node project anyway. It doesn't suit this feature: `ascribe observe` runs where the code is, which may be a Python or Go repository with no `node_modules`.

Each release already publishes a binary archive per platform on GitHub, with `SHA256SUMS`. Standalone installs would wrap them, with no Node:

- A Homebrew formula, in a tap the release workflow updates, for macOS and Linux: `brew install ascribed-dev/tap/ascribe`.
- An install script for Linux and macOS that downloads the archive for the platform, checks it against `SHA256SUMS`, and puts the binary in a folder on the path. It takes a version, and installs the latest release without one.
- Windows gets the archive and instructions at first. A package for a Windows installer waits for someone to need it.

These install exactly what a release published, so they add no build. They do add things outside the repository to look after (a tap, and a script at an address people will pipe into a shell), which go in [outside.md](outside.md).

One caution carries over from the editor: a project can already have two versions of `ascribe`, the extension's and the one npm pins. A third on the path can disagree with both. That's why an observation names the version that wrote it.

## Security

`ascribe observe` runs a command. Ascribe runs `git` today and nothing else, so this is new, and it's held to these rules:

- **Only when asked.** A person types `ascribe observe`. Nothing else starts it: not `check`, `build`, `diff`, or `drift`, not the language server or the editor, and not `ascribe sources fetch` or `update`.
- **Only the config in front of you.** It runs the `test` and `[environment]` commands of the `ascribe.examples.toml` in the folder it's run from, and says which file and which commands before it runs them. It never runs a command from a config that arrived as a pinned source's copy, or that a page or another project points at.
- **Through the platform's shell, as written.** `git` is started directly, with arguments Ascribe builds. A test command is a line of shell (`npm test`, `pytest -q && go test ./...`), so it's handed to the shell as the config has it, the way a package manager runs a project's scripts: `sh` on Linux and macOS, `cmd` on Windows. Ascribe adds nothing to the line and substitutes nothing into it.
- **A fetched config is data.** On the docs side, a source's config and observations are read and never executed. Every string in them (an environment's value, a label, a file name) is treated as untrusted text: escaped wherever it's shown, and limited in length.
- **No more privilege than the person has, and no less.** The command runs as the user, with no sandbox. Ascribe doesn't pretend otherwise: the reference says so, in the same words as for any tool that runs a project's scripts.
- **Nothing from content reaches a command.** No page, phrase, or attribute is ever part of what's run.

## What this leaves out

- Running or compiling examples from `check`, `build`, or the editor.
- Capturing output at build time; notebooks; playgrounds.
- Judging whether a test is good, or watching the dependencies an example uses.
- Anything in CI.
- New tags. `replace` and `emphasize` are worth designing fresh, and the reservation of the tags Ascribe won't implement can go, but that's a separate proposal.

## Later

- **A result for each example, not for the run.** Most test frameworks can write JUnit XML, and one reader for it would say which test covered which file. That lifts the two limits of observing a whole run: a file is recorded only when a test names it, and one failing example no longer blocks the rest. It's the first thing to add once the basic form is in use.
- **Observing in CI.** A job that runs `ascribe observe` and commits the result, or one that checks committed observations are still true, with a GitHub Action to install `ascribe`. It would record CI's environment instead of a person's, at the cost of a workflow that pushes.
- **Versions.** Mapping an environment's label to a version in the content model, so Ascribe can say an example was tested on a version its page doesn't claim. See the [versions proposal](versions.md).
