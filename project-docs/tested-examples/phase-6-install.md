# Phase 6: Installing without Node

Part of [Tested examples](README.md). Needs nothing in this plan, and is wanted by phase 7.

## Goal

`ascribe` gets onto the path of a machine that has no Node, because `ascribe observe` runs where the code is, and that may be a Python or a Go repository.

```sh
brew install ascribed-dev/tap/ascribe
```

```sh
curl -fsSL https://ascribed-dev.com/install.sh | sh
```

Each installs exactly what a release published. Nothing new is built.

It needs no other phase, so it can be done at any time, and is useful by itself: the agents' hooks already say to prefer the binary over the npm launcher.

## Context

- README decision 23.
- `RELEASING.md`: what a release publishes (an archive for each of four platforms, and `SHA256SUMS`), who approves each step, and the trusted publishing that means the workflow holds no token.
- `.github/workflows/release.yml` and `release-build.yml`.
- [outside.md](../outside.md): every account and repository Ascribe relies on outside this one, and who can change each.
- The docs' getting started, and the agents guide's section on hooks, which both say how `ascribe` is installed today.

## Decisions first

Three choices open this phase, each recorded before anything is published, since each is an address people will type or pipe into a shell:

1. **Where the install script lives.** Recommended: in this repository, published with the docs site at `https://ascribed-dev.com/install.sh`, so it's reviewed like any change and served from the one domain Ascribe already has. The other way is a raw file on GitHub, whose address names a branch and is longer.
2. **How the tap is updated.** Recommended: the release workflow opens a pull request on `ascribed-dev/homebrew-tap` with the new version and hashes, and the maintainer merges it. A workflow that pushes to another repository needs a credential this repository doesn't hold today, and a pull request keeps a person between a release and what `brew` installs. The other way is pushing straight to the tap.
3. **What the script does by default.** Recommended: install to `~/.local/bin`, never ask for `sudo`, and say when that folder isn't on the path. A script that writes to `/usr/local/bin` needs a password on most machines.

## Design

### The script

`install.sh`, POSIX `sh`, for Linux and macOS:

1. Work out the platform: `linux-x64`, `linux-arm64`, or `darwin-arm64`. Anything else stops with the list of what's published and a link to the instructions.
2. Take a version from `--version`, or ask GitHub which release is latest.
3. Download that release's archive and `SHA256SUMS`, over HTTPS, from the release's own address.
4. Check the archive's hash against the file, and stop when it doesn't match or no tool to check it is found. It never installs something it couldn't check.
5. Put the binary in the folder, and print the version it now runs.

It reads no variable it doesn't document, runs nothing it downloaded but the binary's own `--version`, and is short enough to read before piping. A test runs it in a container for each Linux platform and on macOS in CI, against the last release.

Linux on x64 with an old glibc is refused with the same sentence the docs use for the minimum.

### The tap

A formula in `ascribed-dev/homebrew-tap` that names each platform's archive and its hash. `brew install ascribed-dev/tap/ascribe` works on macOS and Linux.

The release workflow gains a last step, after the GitHub release is published and not before: write the formula from the release's `SHA256SUMS` and open the pull request. A dry run writes the formula as an artifact and opens nothing.

### Windows

No installer. The getting-started page says to download the archive, check it against `SHA256SUMS`, and put `ascribe.exe` on the path, with the three commands that do it in PowerShell.

### Three versions on one machine

A machine can now have the extension's binary, the one npm pinned in a project, and one on the path. The docs say which wins where: the editor runs its own or the project's, `npx ascribe` runs the project's, and a bare `ascribe` runs the path's. `ascribe --version` says which it is, an observation names the one that wrote it, and the extension already tells an agent in its window which binary to run.

### What it adds outside the repository

`outside.md` gains the tap (a repository, and who can merge to it) and the script's address (a file the docs site serves, changed only through this repository). `RELEASING.md` gains the step that merges the tap's pull request, and a check after a release: `brew install` and the script each give the new version.

## Tasks

1. Record the three decisions.
2. `install.sh`, with its tests on each platform, and the site serving it.
3. The tap repository, with the maintainer, and its first formula.
4. The release workflow's step, and its dry run.
5. `RELEASING.md` and `outside.md`.
6. Getting started, the agents guide's line on hooks, the command reference, and an entry in `changes/`.

## Out of scope

- A package for Windows: winget, Scoop, or an MSI.
- A GitHub Action that installs `ascribe`. It belongs with observing in CI.
- Linux packages: `.deb`, `.rpm`, or a distribution's repository.
- Updating in place. `brew upgrade`, or run the script again.

## Acceptance criteria

- On a machine with no Node, the script installs the last release and `ascribe --version` prints it.
- With the archive's bytes changed, the script stops before installing.
- `brew install ascribed-dev/tap/ascribe` gives the same version on macOS and on Linux.
- A release's dry run writes the formula and opens no pull request.
- `outside.md` lists the tap and the script, with who can change each.

## Verify

```sh
sh site/public/install.sh --version 0.3.0 --to "$(mktemp -d)"
pnpm test
```

## Commits

1. "Record how ascribe installs without Node"
2. "Add an install script that checks what it downloads"
3. "Update the Homebrew tap from a release"
4. "Say how to install ascribe without Node"
