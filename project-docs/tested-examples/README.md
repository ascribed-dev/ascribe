# Tested examples

A code example in an Ascribe site says what it was tested with, and Ascribe knows when that stops being true. The idea, and the research behind it, are in the [proposal](../proposals/tested-examples.md). This is the plan for building it.

## Who it's for

- **A reader copying an example,** who today can't tell whether it ran on the version they have, or ever.
- **A team whose examples are real files with real tests,** which today has no way to say so on the page, and finds out an example went untested when a reader does.
- **An agent using the docs,** which the research found goes wrong on examples mostly by mismatching versions.

## What exists today

- **`@snippet` shows code from a file,** through a source named in `ascribe.toml`, whole or by a tagged region, as a fenced code block (SPEC §4.8). A snippet's title travels in the block's info string, as `title="…"`, and `@ascribed/astro` draws it as a caption (`packages/astro/src/code-titles.ts`).
- **A source is a folder in the same repository, or another repository at a pin.** For the second, `ascribe sources fetch` and `update` copy the files snippets use into `sources/<name>/`, and `ascribe.lock` records each with the hash of its bytes (`crates/ascribe-model/src/lock.rs`, `crates/ascribe-sources`).
- **`ascribe drift`** lists the pages whose examples changed while their words didn't. It says whether the code a page shows changed in this pull request, and nothing about whether it was tested.
- **The JSON output carries a snippet's address, path, and lines** on its block (`crates/ascribe-emit/src/json.rs`).
- **The editor has nothing for a `@snippet` line** beyond its problems, and a snippet is read again when its page changes, not when its code does.
- **Advice checks have a contract:** a name, an area, a kind of next step (`fix`, `choose`, `write`, `review`, or `outside`), and a project can raise one, turn it off, or acknowledge a problem.
- **Ascribe starts three kinds of program:** `git`, with arguments it builds; Vale, as a program the project installs, with a fixed argument list (`crates/ascribe-check/src/tool.rs`); and the programs `ascribe report` runs. It never runs a line of shell.
- **Reading a repository runs nothing the repository supplies** (decision 76 in `project-docs/decisions.md`). The MCP server and the agents' hook never start a program a project names, and the language server asks first.
- **Nothing puts `ascribe` on a machine's path without Node.** Each release publishes an archive for each platform with `SHA256SUMS`, and `@ascribed/cli` is the only install.
- **`examples/docs-repository`** takes its examples from another repository, `ascribed-dev/sources-fixture-code`, which [outside.md](../outside.md) lists.

## Decisions

These are settled. Don't reopen them in a phase; if one can't be met, stop and report.

1. **Ascribe doesn't test code, and doesn't judge tests.** No test frameworks, no toolchains, no sandbox. It says a command passed, where, and nothing more.
2. **Only `ascribe observe` runs anything.** `check`, `build`, `diff`, `drift`, the language server, the MCP server, the hooks, and `ascribe sources` compare files with what was written down, and never start a test or an environment command.
3. **The config is `ascribe.examples.toml`, at a source's root:** `include`, `ignore`, `test`, and `[environment]`. Every path in it and in an observation is from that folder, which is also the folder a snippet's path is from.
4. **An environment entry is a `label` and a `command`.** What's kept is the first line the command prints. What's shown is the first dotted number in it, or the whole line when it has none. The operating system and architecture are noted without being asked for.
5. **An observation is a file Ascribe writes beside the config,** `examples.observations/<name>.toml`, `default` without `--name`: the format's version, the `ascribe` that wrote it, the command, the environment, and the hash of every file `include` matches. No timestamp and no commit.
6. **A person runs `ascribe observe` and commits the result,** as a lockfile is updated. Nothing in CI writes one.
7. **A failed run writes nothing,** and leaves the old observation, which then no longer matches.
8. **The run is the unit.** Every matched file is recorded as passing, whether or not the command touched it, and one failing example blocks every observation. That's the cost of not reading test results, said in the docs.
9. **Only the example files decide whether an observation is stale.** A change to a library an example calls doesn't.
10. **Files, not regions.** An edit anywhere in a file makes its observation stale for every snippet of the file.
11. **A file is tested under each environment whose observation is current.** With none current, it's stale. With none at all, it's unobserved.
12. **An observation in a format this `ascribe` doesn't know** is reported as that, and its files are treated as unobserved.
13. **A page says "tested",** on the code block's caption, carried the way a title is. "Observation" is for the command and the file.
14. **The diagnostics are advice:** stale, unobserved, and an observation of a file that isn't there. A project can raise or turn off each.
15. **Observations travel with a source.** They're read from its root whatever `include` says, no snippet can address one, and for a source in another repository `ascribe sources fetch` and `update` copy them and record them in `ascribe.lock`.
16. **The command is `ascribe observe`,** a verb alone while it's the only command about examples, by the surface audit's conventions. It writes `text` or `json`, and exits 1 when the test fails.
17. **The test command is a line of shell, run as written:** `sh` on Linux and macOS, `cmd` on Windows, as the user, with no sandbox. Ascribe adds nothing to the line and substitutes nothing into it. It says which file and which commands before it runs them.
18. **Only the config in the folder it's run from.** `ascribe observe` never runs a command from a config that arrived as a source's copy, or that a page or another project points at.
19. **A fetched config and its observations are data:** read, never executed, every string escaped where it's shown and limited in length.
20. **Nothing from content reaches a command.**
21. **Shown output needs nothing new.** An expected-output file is an example like any other, shown with a second `@snippet`.
22. **Example repositories show the pattern,** each a repository of its own under the organization: Node with TypeScript, and Python, first.
23. **`ascribe` installs without Node** from a Homebrew tap and an install script, each wrapping what a release already published. Windows gets the archive and instructions.
24. **It's for projects starting clean.** Compatibility with other tools' tags and layouts isn't a goal.

[Phase 1](phase-1-decisions.md) settled five more, each a contract a phase ships, and records each in `project-docs/decisions/` as #458a to #458e:

25. **A file is hashed as its bytes, with CRLF read as LF,** and the hash is named `sha256-lf:`, apart from the lock's `sha256:`. A file hashes the same on a Windows checkout and a Linux one, and a change only to line endings reads as unchanged.
26. **The tested line is an attribute holding the environment alone,** `tested="Node 24.9.0 and TypeScript 5.9.2 on Linux"`, in the code block's info string. "Tested with" comes from the stylesheet, as "Available:" does. A block with a tested line and no title still gets a caption; with both, the title comes first. The plain output writes the whole sentence on a line under the block.
27. **The JSON output's `tested`** is on a block's `snippet`, only when the file is an example: a `status` of `current`, `stale`, or `unobserved`, and `observations`, the current ones, each with its name and its environment as the observation has it. No labels.
28. **The diagnostics are `example-stale`, `example-unobserved`, and `observation-file-missing`:** advice, in a new area, `examples`, with the next step `outside` wherever the code is. The message says what to run: `ascribe observe` in the named folder, or the next `ascribe sources update`. The first two are at the `@snippet` line, the third at the observation.
29. **A snippet of a file that isn't an example reports nothing.** A file is an example only when its source has an `ascribe.examples.toml` whose `include` matches it, so a project without a config sees no change.

## The thread

**The use case:** in an example project whose docs show a TypeScript file from a folder beside them, someone runs `ascribe observe` in that folder. It runs the folder's test command and writes an observation. The built page's code block then reads "Tested with Node 24.9.0 on Linux". They edit the example without running it again, and `ascribe check` says the file changed after its last test run.

**The layers it crosses:**

| Layer | Where | What the thread needs from it |
|---|---|---|
| The config and the observation | `ascribe-model`, beside `ascribe.lock`'s loader | Two file formats, read and validated; one written |
| The command | `ascribe-cli`, and nowhere else | A line of shell run on three platforms, its output captured, and nothing written when it fails |
| Sources | `ascribe-sources`, `ascribe-resolve` | A source's config and observations found at its root, and each snippet's file matched to them |
| Check | `ascribe-check` | Two advice checks |
| Emit | `ascribe-emit` | The tested line on the code block, in the site and plain outputs |
| The renderers | `render_site_html`, `@ascribed/astro`, `@ascribed/elements` | A caption both draw alike |
| The docs | `docs/content/` | A guide, the reference, and two contracts |

**The seam it tests** is two sides agreeing about a file without sharing anything but its bytes. The code side hashes a file where the test ran. The docs side hashes what a snippet reads, perhaps on another machine and another operating system, perhaps from a checkout that rewrote line endings. If those hashes differ for a file nobody edited, every example reads as stale and the feature is noise. The second assumption is that one line of shell can be run and reported the same way on Linux, macOS, and Windows. The third is that a tested line can ride a code block's info string through both renderers, as a title does. If the first fails, an observation has to record something other than a hash of bytes, and phase 1's first decision is reopened before anything ships.

**Left out, and where it comes back:**

| Left out | Phase |
|---|---|
| A source in another repository: fetching, the lock, and what an update says | 3 |
| Several observations, a file that's gone, a newer format, the JSON output, and `ascribe report` | 4 |
| The editor | 5 |
| Installing `ascribe` without Node | 6 |
| The example repositories | 7 |
| Ascribe's own docs | 8 |

After phase 2, a source in another repository reports nothing about testing and shows no tested line, and the reference says so while that's true.

## Phases

| Phase | Result |
|---|---|
| [1: Decisions](phase-1-decisions.md) | The maintainer settles the five contracts the phases ship. |
| [2: The thread](phase-2-thread.md) | `ascribe observe` writes an observation, a page says what its example was tested with, and `ascribe check` says when that's stopped being true. |
| [3: Another repository](phase-3-remote.md) | Examples kept in another repository say what they were tested with, and an update says which went stale. |
| [4: Everything it says](phase-4-status.md) | An example tested under two environments names both, and an agent, a report, and a script each get the same answer. |
| [5: The editor](phase-5-editor.md) | A `@snippet` line shows its file, its status, and its environment, and its problems appear as you type. |
| [6: Installing without Node](phase-6-install.md) | `brew install` or one script puts `ascribe` on the path of a machine with no Node. |
| [7: Repositories to copy](phase-7-repositories.md) | A team starts from a repository that already has examples, tests, observations, and docs that show them. |
| [8: Ascribe's own docs](phase-8-docs.md) | The docs' own examples say what they were tested with. |

Phase 1 comes first because phase 2 ships a file format, a command, an attribute in two contracts, and two diagnostics. After the thread the order is by risk. Phase 3 is the other way a file reaches the docs, by copy and by lock, and the second place the hashes have to agree. Phase 4 changes the JSON output's contract. Phases 5 to 7 widen who can use it, and phase 7 needs phases 3 and 6. Phase 8 is the test on a project that already has hundreds of snippets.

Each phase's result is something a person can run or open. None builds a layer for a later phase to use.

## Carved out, and waiting

- **A result for each example, not for the run.** A reader for JUnit XML would say which test covered which file, and lift decision 8's two limits. It's the first thing to add once this is in use.
- **Observing in CI,** and a GitHub Action that installs `ascribe`.
- **Saying an example was tested on a version its page doesn't claim.** It maps an environment's label to a dimension, and waits for the [versions plan](../versions/README.md), which carves it out too.
- **New tags.** `replace` and `emphasize` designed fresh, and the reservation of the tags Ascribe won't implement dropped. A proposal of its own.
- **A preview that follows the code, and moving between a page and its code.** Both are in the [brainstorm](../brainstorm.md), section 1. Phase 5 adds a hover and no more.

## The other plans

- **Versions.** A line's output keeps or drops a snippet as it does any block, and a tested line needs nothing from it.
- **Page templates.** A tutorial or a how-to made from a template shows examples the same way as any page.
- **The surface audit.** Its conventions name the command, its `--format` words, and its exit codes (`project-docs/checklists.md`, "A new command or option").
- **Decisions.** Each phase records its decisions in `project-docs/decisions/`, numbered by its issue.

## Rules for every phase

- Branch before committing; never commit to `main`.
- Read the current code before the phase file's pointers. If the phase file and the code disagree, or a decision above can't be met, stop and report instead of choosing silently.
- **One place starts a shell, and a test holds it.** The code that runs `test` and the `[environment]` commands is in `ascribe-cli`, behind `ascribe observe`, and no library crate can reach it. A test lists every place the workspace starts a program, as `crates/ascribe-resolve/tests/all/file_reads.rs` lists direct file reads, and fails on a new one.
- **A project with no `ascribe.examples.toml` changes in nothing.** Every phase runs `node scripts/compare/outputs.ts --base main` and shows it. A project that gains one is named with `--accept`.
- **Every string from a config or an observation is untrusted where it's shown:** escaped in HTML, in Markdown, and in a terminal, and cut at a length the phase states. Each phase that shows one has a test with a hostile value.
- **The language and the contracts change only by a recorded decision.** Go through the matching list in `project-docs/checklists.md` first.
- **A new diagnostic** is added to `tests/conformance/diagnostics.toml` and `crates/ascribe-core/src/diagnostics.rs`, in the same order, with a conformance case.
- **Two renderers must agree.** A change to the site output's markup changes the fixtures in `tests/render/`, and both `render_site_html` and the Astro plugin pass them.
- **Project files are read through `ascribe_resolve::FileSystem`.** Libraries don't print, and don't panic on input.
- **A change about paths or line endings runs the full matrix** from its branch. Hashing a file is both.
- **The map of the surface** is regenerated when a phase adds to it (`node scripts/surface/inventory.ts`).
- **User-visible changes** update the page under `docs/content/` and add an entry in `changes/` in the same phase.
- No phase history in code or docs. Describe what the code does now.
- Before finishing a phase, all of these pass:

  ```sh
  cargo fmt --all --check
  cargo clippy --workspace --all-targets --locked -- -D warnings
  cargo test --workspace --locked
  pnpm format:check && pnpm lint && pnpm typecheck && pnpm test
  ```

## Later, not in this plan

- Running or compiling examples from `check`, `build`, or the editor.
- Capturing output at build time; notebooks; playgrounds.
- Judging whether a test is good, or watching the dependencies an example uses.
- A package for a Windows installer.
