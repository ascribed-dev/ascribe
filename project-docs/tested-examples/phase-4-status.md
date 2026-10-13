# Phase 4: Everything it says

Part of [Tested examples](README.md). Needs phase 2.

## Goal

An example tested under two environments names both, and everything that reads a project gets the same answer about its examples: the page, the JSON output, `ascribe report`, and a script.

```console
$ ascribe observe --name node-22
…
Wrote examples.observations/node-22.toml: 2 files, Node 22.4.0 on Linux.
```

The page then reads "Tested with Node 22.4.0 and 24.9.0 on Linux."

## Context

- README decisions 5, 11, and 12, and phase 1's decisions 2 to 4.
- `crates/ascribe-emit/src/json.rs`, and the JSON output's contract: a block's `snippet`.
- `docs/content/checks/report.md`, and decision 66: `ascribe report`'s sections, and that its JSON is a contract.
- `ascribe model`, which lists a project's sources.

## Design

### Several observations

`ascribe observe --name <name>` writes `examples.observations/<name>.toml`. A name is lowercase words joined by `-`. Without it, `default`.

Each observation is whole and separate: it has its own environment and its own hashes, and running one doesn't touch another. An observation whose name nothing writes any more stays until someone deletes its file, and goes stale like any other.

A file's status across them, as README decision 11 has it:

| Its observations | Status |
|---|---|
| At least one is current | `current`, naming those |
| It's listed, and none is current | `stale` |
| None lists it | `unobserved` |

### The tested line, with several

The current observations' environments are merged into one line:

- An entry with the same label in each is said once, with each distinct shown value in order: "Node 22.4.0 and 24.9.0".
- An entry only some have is said for those.
- Operating systems are merged the same way: "on Linux and macOS".
- Past three values for one entry, the line says how many and names the lowest and highest: "Node, 5 versions from 18.20.0 to 24.9.0". The JSON output has them all.

Order is by version where the values are dotted numbers, and by text otherwise.

### A file that's gone, and a format that's newer

- **`observation-file-missing`:** an observation lists a file that isn't in the folder. Advice, at the observation, for a source in the same repository only (phase 3 says why). It's usually an example that was deleted or renamed without a second run.
- **A newer format:** an observation whose `version` this `ascribe` doesn't know is one warning on the source, naming the `ascribe` that wrote it, and its files are treated as unobserved by that observation. Other observations still count.

### The JSON output

A block's `snippet` gains `tested`, as phase 1's decision 3 has it, on every example and on nothing else. The output's schema and the generated Zod schema follow, and the contract page says the key is new and optional.

### `ascribe report`

The `inventory` section gains the project's examples: how many snippets show an example, and how many of those are current, stale, and unobserved, by source. In the JSON, a new key under `inventory`. A project with no config in any source has no such key and no such lines.

### `ascribe model`

Each source says whether it has a config, how many files its `include` matches among those the docs can see, and the names of its observations with each one's environment.

### `ascribe observe`, finished

- It says what changed since the observation it replaces: files added, removed, and changed, and environment values that differ.
- `--check` runs nothing and exits 1 when the observation named is stale against the folder as it stands, for a team that wants a pre-commit hook. It's the only part of `observe` that could be run unattended, and it starts no program.

## Tasks

1. `--name`, and a file's status across observations.
2. The merged tested line, with its tests: one value, two, more than three, values that aren't versions, and two operating systems.
3. `observation-file-missing`, and the newer-format warning, with conformance cases.
4. `tested` in the JSON output, its schema, and the Zod schema.
5. The examples in `ascribe report` and `ascribe model`.
6. What an `observe` run changed, and `--check`.
7. `examples/tested`: a second observation, committed, and CI comparing both.
8. The guide, the references, the contracts, entries in `changes/`, and the decisions recorded.

## Out of scope

- Mapping an environment to a dimension or a version.
- A report of its own for examples. If one's wanted, it's `ascribe examples`, and `observe` moves into it (README decision 16).
- The editor.

## Acceptance criteria

- With two current observations, the example project's page names both versions in one line, in every output.
- Editing the example makes both observations stale; running `ascribe observe` once leaves the page naming one environment, and the other reported.
- A deleted example is reported at the observation that still lists it.
- An observation with `version = 99` is one warning, and the other observation still gives the page its line.
- `ascribe report --format json` counts the project's examples by status, and matches the pages.
- `ascribe observe --check` starts no program: the list of program starts doesn't grow, and the test for it runs with a `test` command that would fail.

## Verify

```sh
cargo test --workspace --locked
pnpm test
cargo run -p ascribe-cli -- report --config examples/tested
cargo run -p ascribe-cli -- build --config examples/tested
```

## Commits

1. "Observe an example under more than one environment"
2. "Name every environment an example was tested with"
3. "Report an observation of a file that's gone"
4. "Say what an example was tested with in the JSON output"
5. "Count a project's tested examples in its report"
