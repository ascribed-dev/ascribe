# Phase 3: Another repository

Part of [Tested examples](README.md). Needs phase 2.

## Goal

Examples kept in another repository say what they were tested with. `ascribe sources fetch` brings a source's config and observations with the files snippets use, `ascribe.lock` records them, and `ascribe sources update` says which examples went stale with the new pin.

It comes straight after the thread because it's the second place the two sides have to agree about a file: through a copy and a lock, this time.

## Context

- README decisions 15, 18, and 19.
- SPEC §7.4, and `crates/ascribe-sources/src/remote.rs`: what a fetch copies, and how the lock records it.
- `ascribe sources update`'s summary, in the command reference and in `examples/docs-repository/.github/workflows/update-sources.yml`, which opens a pull request with it.
- `ascribed-dev/sources-fixture-code`, in [outside.md](../outside.md): the repository `examples/docs-repository` and the tests fetch from, and `scripts/sources-fixture/`, which describes its commits.
- Decision 76: every `git` Ascribe starts, and what a fetched repository can't make it run.

## Design

### What's fetched

For a source with `git`, a fetch copies, beside the files snippets use:

- `ascribe.examples.toml`, from the repository's root at the pin;
- every file in `examples.observations/` there.

They go in `sources/<name>/`, at the same paths, and `ascribe.lock` lists each with its hash, as it does every copy. A source whose repository has no config copies nothing more, and is what it was.

They're copied whatever the source's `include` says, and no snippet can address one: `code:ascribe.examples.toml` and anything under `examples.observations/` are refused with the diagnostic a path outside `include` gets.

### What's compared

The docs side does what phase 2 does, reading the copies. A snippet's file is hashed from its copy in `sources/<name>/`, with phase 1's hash, and compared with the copied observation.

Only the files snippets use are copied, so an observation usually lists more files than the docs have. That's expected: `observation-file-missing` is about the code's side, and is never reported for a source in another repository.

### A fetched config is data

`test` and every `command` in a fetched config are read as strings and shown nowhere but `ascribe model`. Nothing on the docs side runs one, and `ascribe observe` run inside `sources/<name>/` refuses: the folder is a copy Ascribe wrote, which it knows from the lock.

A config or an observation that doesn't parse is a warning on the source, and its examples are unobserved.

### What an update says

`ascribe sources update` already says which pages' examples changed with a new pin. Its summary, in text and JSON, gains the examples that:

- went stale: the file changed and the observation didn't follow;
- lost their observation: the new pin doesn't list them;
- gained one, or became current again.

The JSON's new keys are added to `sources.schema.json`, and none changes.

### The next step

For these sources the advice's message says the fix is in the other repository and arrives with the next `ascribe sources update` (phase 1, decision 4).

### The fixture

`sources-fixture-code` gains an `ascribe.examples.toml`, a test, and an observation, in commits that give the tests each case: current, stale after a later commit, and an example the observation doesn't list. It's a repository outside this one, so the maintainer pushes those commits, and `scripts/sources-fixture/changes.ts` and `outside.md` record them.

## Tasks

1. The config and the observations copied by `fetch` and `update`, and recorded in the lock.
2. The two paths a snippet can't address.
3. The comparison from copies, and `observation-file-missing` left out for these sources.
4. `ascribe observe` refusing a folder that's a source's copy.
5. The update's summary, in text and JSON, with the schema.
6. The fixture repository's commits, with the maintainer, and the tests against them.
7. `examples/docs-repository`: its page shows a tested line, and its update workflow's pull request says what went stale.
8. The drift guide, the tested examples guide, the command reference, the lock's contract, an entry in `changes/`, and the decisions recorded.

## Out of scope

- Fetching more of a repository than snippets use.
- Running anything in the other repository.
- A source at a path outside the repository that isn't `git`.

## Acceptance criteria

- After `ascribe sources fetch` in `examples/docs-repository`, its page reads "Tested with …", and `ascribe.lock` lists the config and the observation.
- Deleting a copied observation makes `ascribe check` report the copy as missing, as it does for any locked file.
- At the fixture's later pin, `ascribe sources update` names the example that went stale, and `ascribe check` reports it with the message for another repository.
- `ascribe observe` in `sources/<name>/` exits 2 and says why.
- A fetched config whose `test` is `rm -rf ~` is fetched, listed, and never run: a test builds, checks, and updates with one, and the list of program starts doesn't grow.
- A source with no config in its repository fetches exactly what it did.

## Verify

```sh
cargo test --workspace --locked
cargo run -p ascribe-cli -- sources fetch --config examples/docs-repository
cargo run -p ascribe-cli -- check --config examples/docs-repository
```

## Commits

1. "Fetch a source's examples config and observations"
2. "Compare a fetched example with its observation"
3. "Say which examples an update left stale"
