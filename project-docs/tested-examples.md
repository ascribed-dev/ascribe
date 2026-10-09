# Proposal: tested examples

A proposal, not a plan. It says what the feature is and what it would look like, so the idea can be judged before anything is designed in detail. It follows the research in [Code examples tied to tested code](../reports/Code%20examples%20tied%20to%20tested%20code.md).

## The goal

A code example in an Ascribe site says what it was tested with, and Ascribe knows when that stops being true.

`@snippet` already shows code from a real file, and `ascribe drift` already reports a page whose example changed. What's missing is any knowledge of the test: whether the file was tested, in what environment, and whether it has changed since.

It's for projects starting clean. Compatibility with other tools' tags and layouts isn't a goal.

## What Ascribe does, and doesn't

- **It doesn't test code.** No test frameworks, no language toolchains, no sandbox.
- **It wraps the project's own test command,** when asked, and writes down what passed and where.
- **It reads what was written down everywhere else.** `check`, `build`, and the editor never run anything. They compare files with the observations, which is exact and fast.

## Three pieces

### 1. A config file where the examples live

`ascribe.examples.toml`, in the folder the examples are in: the docs repository, or another one.

```toml
# What counts as an example, relative to this file.
include = ["examples/**"]
ignore = ["**/node_modules/**"]

# The command that tests them. It passes or it fails.
test = "npm test"

# What to note about where it ran: a label, and a command that prints it.
[environment]
node = "node --version"
typescript = "npx tsc --version"
```

The `[environment]` table is how Ascribe stays out of every language's ecosystem. The repository says how to ask for its versions, and Ascribe writes down the answers. The operating system and architecture are noted without being asked for.

### 2. An observation, written by the CLI

```sh
ascribe observe                 # runs `test` from the config
ascribe observe --name node-24  # one observation per environment, for a matrix
ascribe observe --check         # for CI: are the committed observations still true?
```

Ascribe watches the project's command run and writes down what it saw: an **observation**. The word is chosen for the size of its claim. An observation says what happened and under what conditions, not that the code is correct. If the command passes, Ascribe writes one beside the config:

```toml
# examples.observations/node-24.toml. Written by `ascribe observe`. Don't edit it.
passed = true
commit = "4f2a9c1"
command = "npm test"

[environment]
os = "linux"
arch = "x64"
node = "v24.9.0"
typescript = "Version 5.9.2"

[files]
"examples/connect.ts" = "sha256:9b1c…"
"examples/connect.out.txt" = "sha256:07fe…"
```

- **One file per environment,** so jobs in a CI matrix never write the same file.
- **No timestamp,** so an observation changes only when the code or the environment does.
- **Every example file is hashed,** including expected-output files, which are examples like any other.
- **A failed run writes nothing** and leaves the old observation in place, which then no longer matches.

### 3. What the docs side does with it

A docs project reaches the examples through `[sources.<name>]`, as today. `ascribe sources update` brings a remote source's config and observations in with the pinned files. Then, with nothing run:

| The observation says | The file now | Ascribe reports |
|---|---|---|
| Passed, and the hash matches | Unchanged | Tested as shown, with the environment |
| Passed, and the hash differs | Edited since | A stale observation: the example changed after its last test run |
| Nothing about this file | Any | Unobserved: not tested, said plainly |

Where that shows up:

- **On the code block.** A line with the example: "Tested with Node 24.9 and TypeScript 5.9 on Linux." With several observations: "Tested on Node 22 and 24." The page says "tested", the everyday word; "observation" is for the command and the file.
- **In the plain and JSON outputs,** so an agent reading the Markdown version gets the same line. The research found that matching versions is where agents go wrong with examples.
- **As diagnostics,** under the [content checks](content-checks/README.md) contract: an example whose observation is stale, and an example with none. Both advisory, both able to be raised or turned off.
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
- a CI workflow that runs `ascribe observe` across a small matrix;
- a docs project that shows the examples.

They hold no custom code, since the CLI writes the observations. Start with two, Node with TypeScript and Python. Each runs in CI on a schedule: the research's clearest warning was about sample infrastructure left to rot.

## Getting `ascribe` where the examples are

Today nothing puts `ascribe` on a machine's path. The extension carries a binary for its own use, and `@ascribed/cli` puts one in a project's `node_modules`, reached with `npx`. That suits a docs project, which is a Node project anyway. It doesn't suit this feature: `ascribe observe` runs where the code is, which may be a Python or Go repository with no `node_modules`, and mostly runs in CI.

Each release already publishes a binary archive per platform on GitHub, with `SHA256SUMS`. Two things would wrap them:

- **Standalone installs, with no Node.**
  - A Homebrew formula, in a tap the release workflow updates, for macOS and Linux: `brew install ascribed-dev/tap/ascribe`.
  - An install script for Linux and macOS that downloads the archive for the platform, checks it against `SHA256SUMS`, and puts the binary in a folder on the path. It takes a version, and installs the latest release without one.
  - Windows gets the archive and instructions at first. A package for a Windows installer waits for someone to need it.
- **A GitHub Action** that does the same in a workflow:

  ```yaml
  - uses: ascribed-dev/ascribe/setup@v0.3.0
  - run: ascribe observe
  ```

  It can live in a folder of this repository, so its version is the release's and there's no second repository to keep in step. It installs the version of the tag it's used at, checks the checksum, and adds the binary to the job's path.

Both install exactly what a release published, so neither adds a build. Both add something outside the repository to look after (a tap, and a script at an address people will pipe into a shell), which goes in [outside.md](outside.md).

One caution carries over from the editor: a project can already have two versions of `ascribe`, the extension's and the one npm pins. A third on the path can disagree with both. An observation should say which version of `ascribe` wrote it, and the docs side should say so when it reads one written by a newer version than its own.

## What this leaves out

- Running or compiling examples from `check`, `build`, or the editor.
- Capturing output at build time; notebooks; playgrounds.
- A result for each example, not for the run. A later version could read JUnit XML, which most test frameworks can write, to say which test covered which file.
- New tags. `replace` and `emphasize` are worth designing fresh, and the reservation of the tags Ascribe won't implement can go, but that's a separate proposal.

## Open questions

- **Who writes the observations?** They have to be committed, so a pinned source carries them. Either a person runs `ascribe observe` before committing and CI runs `--check`, as with a lockfile, or a CI job commits it. The first observes the developer's machine, not CI's. The second needs a workflow that pushes. The example repositories have to pick one and show it.
- **What makes an observation stale?** Hashing the example files catches an edited example. It doesn't catch a change to the library the example calls. The config could name more files to hash (a lockfile, the package's source), at the cost of observations that change more often.
- **How honest is "tested"?** An observation says a command passed. It can't say the test was any good, and it can be written by hand. The wording on the page should claim only what's known: "tested with", not "verified".
- **Wrapping a command is new.** Ascribe runs `git` today and nothing else. `ascribe observe` would run whatever the config names, so it needs the same care as any tool that does: only on request, never from `check`, `build`, or the editor, and never from a config in a repository the user didn't choose to run it in.
- **The format.** TOML above, to match `ascribe.toml`. `ascribe.lock`'s format should decide it.
- **Display.** The code block's caption is part of the elements' contract, so the tested line is a change to it, and needs a form for the plain output too.
