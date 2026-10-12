# Phase 2: The thread

Part of [Tested examples](README.md). Needs phase 1.

## Goal

An example says what it was tested with, and Ascribe says when that's stopped being true. In a new example project, `examples/tested`, whose docs show a TypeScript file from a folder beside them:

```console
$ cd examples/tested/code && ascribe observe
Running the commands in ascribe.examples.toml:
  test         npm test
  environment  node --version
npm test passed.
Wrote examples.observations/default.toml: 2 files, Node 24.9.0 on Linux.
```

The built page's code block reads "Tested with Node 24.9.0 on Linux". After an edit to the example with no second run, `ascribe check` reports `example-stale` at the `@snippet` line.

This is the thinnest slice through every layer the feature touches. It exists to find out whether the two sides can agree about a file from its bytes alone, on every platform (README, "The thread").

## Context

- README decisions 1 to 14, 16 to 21, and phase 1's five.
- SPEC §4.8 (`@snippet`), §7.3 (sources).
- `crates/ascribe-model/src/lock.rs`: a file Ascribe writes and reads back, with a version, and its hash.
- `crates/ascribe-check/src/tool.rs`: how a program is started today, with its output captured and a time limit.
- `crates/ascribe-resolve/src/snippet/`: where a snippet's file is read.
- `packages/astro/src/code-titles.ts`, and `tests/render/`: a code block's title in both renderers.
- `crates/ascribe-resolve/tests/all/file_reads.rs`: a test that lists every place something is done, and fails on a new one.
- Decision 76 in `project-docs/decisions.md`: what the MCP server, the hook, and the language server never start.

## Design

### The config

`ascribe.examples.toml`, read from a source's root. Unknown keys are errors, as in `ascribe.toml`.

| Key | Type | Means |
|---|---|---|
| `include` | list of patterns | The files that are examples, from this folder. Required, and not empty |
| `ignore` | list of patterns | Files `include` matches that aren't |
| `test` | string | The line of shell that tests them. Required for `ascribe observe`, and not read by anything else |
| `[environment]` | table | Each entry a `label` and a `command` |

Patterns are the content model's (§1.3 of its reference). `examples.observations/` is never an example, whatever `include` says.

A problem in the config is reported where it's read: by `ascribe observe` as an error and exit 2, and by `ascribe check` as a warning on the docs project that names the source, since a config that can't be read means no example can be checked.

### The observation

`examples.observations/default.toml`, as the proposal shows it: `version`, `ascribe`, `command`, `[environment]`, and `[files]`, each file with its hash as phase 1's first decision has it, in path order, one to a line.

- `os` and `arch` are always in `[environment]`, and a config can't declare entries with those names.
- An environment command that fails, or prints nothing, fails the run: an observation that can't say where it ran isn't one.
- A kept line is cut at 200 characters.
- It's written whole to a temporary file and renamed, so an interrupted run leaves the old one.

### The command

`ascribe observe`, run in the folder that holds the config. It doesn't look in parent folders, and takes no path to a config: README decision 18.

1. Read the config. Print the file's path and every command it will run.
2. Run each `[environment]` command, then `test`, through the platform's shell, in the config's folder, with the user's environment and standard input closed. The test's output goes to the terminal as it comes.
3. When `test` exits 0, hash every example and write the observation. When it doesn't, write nothing and exit 1.

- No time limit on `test`. A person started it and can stop it.
- A file that changes between the test starting and the hashing is the author's to avoid. The docs say to run it on a clean tree.
- `--format text|json`. The JSON has the commands, the exit status, the environment, and the files, and its schema is committed with the others.
- Exit 0, 1 when the test fails, 2 when it couldn't run: no config, a bad config, no shell.
- In this phase there's no `--name`. Phase 4 adds it.

The code that starts a shell is in `ascribe-cli` and nowhere else, and a test lists every place the workspace starts a program (README, rules). `ascribe observe` isn't offered as an MCP tool, and no hook or language server request reaches it.

### The docs side

For a source that's a folder in the same repository:

- The config and observations are read from the source's `path`, whatever the source's own `include` allows.
- A snippet's file is an **example** when the config's `include` matches it and `ignore` doesn't (phase 1, decision 5).
- An example's status is `current` when the observation's hash for it equals the file's now, `stale` when it differs, and `unobserved` when the observation doesn't list it or there's no observation.

Two advice checks, as phase 1's decision 4 names them: `example-stale` and `example-unobserved`, at the `@snippet` line, each once for each snippet. The message for a stale one says the file changed, not the example, and names the folder to run `ascribe observe` in.

### The tested line

For a `current` example, the code block's info string carries the tested attribute (phase 1, decision 2): each environment entry's label and shown number, in the config's order, then the operating system.

- **The site output:** the attribute, beside `title` when there is one. `render_site_html` and `@ascribed/astro` both draw a caption, with the title first. The lead-in comes from the element library's stylesheet. The fixtures in `tests/render/` gain a block with a tested line alone, and one with both.
- **The plain output:** a line under the block, "Tested with Node 24.9.0 on Linux."
- **A stale or unobserved example** has no tested line. The page says nothing, and the advice says why to the author.

An operating system is shown by a name a reader knows: Linux, macOS, Windows. Architecture is recorded and not shown.

### The example

`examples/tested`: a docs project, and a `code/` folder beside it with one TypeScript example, its expected output, a test that compares the two with Node's own test runner, `ascribe.examples.toml`, and a committed observation. The page shows the example and its output, as the proposal's "Shown output" has it.

CI runs `ascribe observe` there on Linux, macOS, and Windows and compares each result's `[files]` with the committed one. That comparison is the thread's test of its seam.

## What this phase doesn't handle, said where a user will look

- **The guide,** `docs/content/guides/tested-examples.md`, says a source in another repository shows no tested line yet, and that an example has one observation.
- **The command reference** says `ascribe` has to be installed where the code is, and that today that means Node.

## Tasks

1. The config: its loader, its rules, and its diagnostics.
2. The observation: its format, its writer, and its reader, with the hash phase 1 chose.
3. `ascribe observe`: the shell on three platforms, the output, the exit codes, and the JSON with its schema.
4. The test that lists every place a program is started.
5. A source's config and observations found on the docs side, and each snippet's status.
6. `example-stale` and `example-unobserved`, with conformance cases.
7. The tested line in the site output, both renderers, the stylesheet, and the plain output, with fixtures and a test with a hostile label.
8. `examples/tested`, and the CI comparison across platforms.
9. The guide, the command reference, the diagnostics, the site-render and elements contracts, a contract for the two file formats, an entry in `changes/`, and the decisions recorded.
10. The map of the surface, regenerated.

## Out of scope

- A source in another repository.
- `--name`, a file that's gone, a newer format, the JSON output's `tested`, and `ascribe report`.
- The editor.
- Reading a test framework's results.

## Acceptance criteria

- `ascribe observe` in `examples/tested/code` writes the observation that's committed, on Linux, macOS, and Windows, with the same `[files]` on each.
- A failing test writes nothing and exits 1, and the old observation is byte for byte as it was.
- The example project's page reads "Tested with Node …" in the site output from both renderers and in the plain output.
- Editing the example makes `ascribe check` report `example-stale`, and running `ascribe observe` again clears it.
- A file `include` doesn't match reports nothing and shows nothing.
- A label of `<script>` or of 5,000 characters is shown escaped and cut.
- No project without an `ascribe.examples.toml` changes: `node scripts/compare/outputs.ts --base main` shows it.
- The test that lists program starts fails when a library crate starts one.

## Verify

```sh
cargo test --workspace --locked
(cd examples/tested/code && cargo run -q -p ascribe-cli --manifest-path ../../../Cargo.toml -- observe)
cargo run -p ascribe-cli -- check --config examples/tested
node scripts/compare/outputs.ts --base main --accept 'tested: *'
```

## Commits

1. "Read a source's examples config"
2. "Write down what a test run passed, and where"
3. "Say when an example changed after its last test run"
4. "Show what an example was tested with"
5. "Add an example project with a tested example"
