# Phase 9: Sources in another repository

Part of [Docs](README.md). Requires phases 7 and 8. Rust only.

## Goal

A project can take its examples from code in other repositories, as many as it needs. Each is pinned to a commit, and the files its pages use are copied into the docs repository, so `check` and `build` still need no network and give the same result every time. One command moves a pin forward and says which pages that changes.

This is what a team whose docs live apart from the code needs, and most docs teams are that team.

## Context

- Phase 7: `[sources.<name>]`, the address (`<source>:<path>#<region>`), extraction, and the keys it reserved for this phase (`git`, `branch`, `commit`).
- Phase 8: `ascribe drift`, which reports examples that changed. A moved pin changes copied files, so it shows there with nothing added.
- `crates/tessera-diff/src/git.rs`: shelling out to `git` with fixed argument lists, and the rule that the binary links no git, HTTP, or async library. `gitfs.rs`: reading files at a revision, which is how `diff` will read the copies as they were at the base.
- `crates/tessera-resolve/src/fs.rs`: the `FileSystem` trait. Copied files are read through it like any other.
- `git`, to check: fetching one commit by its hash with a filter (`git fetch --depth 1 --filter=blob:none <url> <commit>`), which hosts allow it, and how credentials reach it (a credential helper, `GIT_ASKPASS`, an `extraheader`, SSH).
- [Decisions 7, 9, 14, and 15](README.md#decisions).

## Design

### Declaring one

```toml
[sources.api]
git = "https://github.com/acme/api.git"
branch = "main"                  # what `update` follows; a tag pattern is Later
include = ["src/**", "examples/**"]
ignore = ["**/generated/**"]

[sources.cli]                    # as many as the project needs
git = "git@github.com:acme/cli.git"
branch = "main"
include = ["cmd/**"]
```

A source has `path` (a folder in this repository, phase 7) or `git`, never both. Pages address either kind the same way: `api:src/auth.rs#login`. Moving code from this repository to another, or back, changes `ascribe.toml` and no page.

### The lock and the copies

`ascribe.lock`, beside `ascribe.toml`, written by Ascribe and committed:

```toml
version = 1

[[source]]
name = "api"
git = "https://github.com/acme/api.git"
commit = "9f2c41d0…"                 # the full hash
files = { "src/auth.rs" = "sha256:…", "examples/login.sh" = "sha256:…" }
```

The files a page takes a snippet from are copied, whole, into `sources/<name>/` in the project's folder, at the path they have in their repository, and committed. Only those files: not the repository, and not everything `include` matches. `include` and `ignore` say what a page may ask for.

- `check` and `build` read the copies. They never fetch.
- `check` also verifies each copy against the lock's hash. A copy edited by hand is an error that says to change the code where it lives, then update.
- A snippet that names a file with no copy is an error that says to run `ascribe sources fetch`.
- A copy no page uses any more is a warning, and `fetch` removes it.
- `ascribe diff` and `ascribe drift` read the copies at the base from the docs repository's own history. They need no second repository and no network.

### The commands

```
ascribe sources fetch [NAME]...             make the copies match the lock
ascribe sources update [NAME]... [--to <REV>] [--format text|json|summary]
ascribe sources status [--format text|json]  offline: each source, its pin, its copies
```

These are the only commands that use the network (decision 14), and only `fetch` and `update` do.

- **`fetch`** gets, at each source's pinned commit, the files that snippets name and that have no copy or a wrong one. It's what you run after writing a new `@snippet`, or after cloning if the copies were left out. It never moves a pin.
- **`update`** moves a source's pin to the head of its `branch` (or to `--to`), fetches the copies again at the new commit, rewrites the lock, and then reports what changed: the commits between the old pin and the new (count, and the first lines of the newest 20), the copied files that changed, and the pages whose examples changed, in the form of phase 8's report. A snippet whose region no longer exists at the new commit is reported there, and the update still completes: `check` then fails on it, which is the signal to fix the page.
- With nothing to move, `update` changes no file and says so.
- **Exit codes:** `0` when it ran (whether or not anything moved); `2` when it couldn't (no network, no access, an unknown revision), naming the source and repeating `git`'s own message.
- **`--format summary`** writes Markdown for a pull request's description, which phase 10 uses.

### Reaching the repository

Ascribe runs `git` and nothing else. Credentials are `git`'s: whatever lets `git fetch <url>` work in that shell lets Ascribe fetch. The docs say how to give a CI job read access to a private repository, and that the site host needs none, since builds read the copies.

Safety, since this reads a repository someone else may control:

- `git` runs with a fixed argument list, the URL only from `ascribe.toml`, no hooks, no submodules, no LFS, and prompts off, so a missing credential fails at once.
- Fetched content is only ever read as text. A file over a size limit (choose one; a megabyte is generous for an example) is refused, and so is one that isn't text.
- A path is checked against `include` and `ignore` before it's asked for, and can't leave `sources/<name>/`.
- Fetches go to a cache outside the project (the user's cache folder), one bare repository per URL, so a second run is quick. The cache is never needed for `check` or `build`.

### What's copied, and who can see it

Copying a file into the docs repository publishes it to everyone who can read the docs repository. For a private code repository and public docs, that's a decision the team has to make knowingly: `fetch` and `update` say so the first time a source's files are copied, and the guide says it plainly. Copying only the regions used is Later.

## Tasks

1. `git` sources in the content model and `ascribe.lock`: loading, checks, the contract pages, and conformance cases (a lock that names a source the model doesn't, a hash that doesn't match, a copy with no lock entry).
2. `fetch`, against local bare repositories in tests (a `file://` URL is a `git` URL): a first fetch, a second from the cache, a file outside `include`, a file too large, a binary file, a path that tries to leave, and no access.
3. `update`: a moved pin, nothing to move, `--to`, a region that's gone, two sources at once, and its three formats.
4. `status`, `diff`, and `drift` over a pull request that moves a pin, with tests that they touch no network (run them with `git`'s network access disabled).
5. A check that `check` and `build` never run `git` at all on a project with remote sources.
6. The command reference, the content-model reference and contract, `guides/drift.md` (a section for docs kept apart from the code), `CHANGELOG.md`.

## Out of scope

The scheduled pull request (phase 10); following tags or releases instead of a branch; copying only regions; sparse or partial copies of large files; non-`git` sources (an archive, a URL); telling the code repository's pull requests which docs they affect.

## Acceptance criteria

- With the network off, a project with two remote sources checks, builds, diffs, and reports drift.
- `update` on a source whose example changed rewrites the lock and the copy, and names the page.
- The same commit of the docs repository builds the same output on any machine.
- Moving a source from `git` to `path` changes no page.
- Nothing in the binary but `sources fetch` and `sources update` can open a connection, and a test holds that.

## Verify

```sh
cargo test --workspace --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo fmt --all --check
```

## Commits

1. "Pin a source in another repository"
2. "Fetch the files a project's snippets use"
3. "Move a source's pin, and say what it changes"
4. "Read copied sources in diff and drift"
