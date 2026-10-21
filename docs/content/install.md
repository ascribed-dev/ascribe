---
type: how-to
title: Install
description: "Install the ascribe command, in a project with npm, or on your path with Homebrew, the install script, PowerShell, or a release's archive; then start a project with ascribe init."
permalink: install
---

There are two ways to have `ascribe`: in a project, with npm, which pins its version for everyone who works on the project and for CI; or on your path, from a release, where there's no Node, such as a Python or Go repository whose hooks or CI run `ascribe`. The latest release is {version}.

To install the editor, see [Editing](editor/index.md).

## Platforms

Ascribe runs on macOS on Apple silicon, Linux (x64 and arm64, with glibc {glibc} or later (2.39 for 0.1.1 and earlier), which includes the build images of Netlify, Vercel, Cloudflare Pages, and AWS Amplify), and Windows (x64). Intel Macs aren't supported.

## In a project, with npm
@id: npm

You need [Node.js](https://nodejs.org) {node} or later. In your project's directory:

@variant {pm=npm}:
```sh
npm install --save-dev @ascribed/cli
npx ascribe --version
```
@variant {pm=pnpm}:
```sh
pnpm add --save-dev @ascribed/cli
pnpm exec ascribe --version
```
@end

`@ascribed/cli` installs the `ascribe` binary for your platform, and pins its version in `package.json`. Run it as `npx ascribe` (`pnpm exec ascribe` with pnpm), or from a script in `package.json`. If `ascribe` can't find the binary, your package manager left out optional dependencies; reinstall with them enabled.

@note {type=tip}: To try what's on `main` before it's released, install `@ascribed/cli@next` and `@ascribed/astro@next` instead: a build published every night, with no promise of stability.

## On your path, without Node
@id: without-node

Each of these installs exactly what a release published, and checks it against the release's `SHA256SUMS` first. Run it again to update.

### Homebrew

On macOS or Linux, with [Homebrew](https://brew.sh):

```sh
brew install ascribed-dev/tap/ascribe
```

### The install script

On macOS or Linux, the install script puts `ascribe` in `~/.local/bin`, never asks for `sudo`, and says when that folder isn't on your path. Add `-s -- --version <version>` for a release other than the latest, or `-s -- --to <folder>` for another folder:

```sh
curl -fsSL https://ascribed-dev.com/install.sh | sh
```

The script is short enough to read before you run it: [install.sh](https://ascribed-dev.com/install.sh).

### PowerShell

On Windows, in PowerShell, download the latest release's archive and its `SHA256SUMS`, check the one against the other, and put `ascribe.exe` on your path:

```powershell
$base = "https://github.com/ascribed-dev/ascribe/releases"; Invoke-WebRequest "$base/latest/download/SHA256SUMS" -OutFile SHA256SUMS; $v = (Select-String -Path SHA256SUMS -Pattern "github/ascribe-(.+)-win32-x64\.zip").Matches[0].Groups[1].Value; $zip = "ascribe-$v-win32-x64.zip"; Invoke-WebRequest "$base/download/v$v/$zip" -OutFile $zip
if (-not (Select-String -Path SHA256SUMS -SimpleMatch -Pattern "$((Get-FileHash $zip -Algorithm SHA256).Hash.ToLower())  github/$zip")) { throw "$zip doesn't match SHA256SUMS" }
Expand-Archive $zip "$env:LOCALAPPDATA\Programs" -Force; [Environment]::SetEnvironmentVariable("Path", [Environment]::GetEnvironmentVariable("Path", "User") + ";$env:LOCALAPPDATA\Programs\ascribe-$v-win32-x64", "User")
```

Open a new terminal, and `ascribe --version` prints the release.

### A release's archive

Each [release]({repo}/releases) has an archive for each platform, `ascribe-<version>-<platform>.tar.gz` (`.zip` for Windows), where the platform is `darwin-arm64`, `linux-x64`, `linux-arm64`, or `win32-x64`, and `SHA256SUMS`, which lists each archive's SHA-256 as `github/<archive>`.

@steps
1. Download your platform's archive and `SHA256SUMS`.
2. Check the archive's SHA-256 (`sha256sum` on Linux, `shasum -a 256` on macOS, `Get-FileHash` on Windows) against its line in `SHA256SUMS`.
3. Extract it. It holds one folder, `ascribe-<version>-<platform>`, with `ascribe` (`ascribe.exe` on Windows) in it.
4. Put that folder on your path, or move `ascribe` into a folder that is.

## Which `ascribe` runs

A machine can have more than one `ascribe`: the one on your path, the one `@ascribed/cli` pins in a project, and the one the editor's extension bundles. A bare `ascribe` runs your path's; `npx ascribe` runs the project's; the editor runs the project's when there is one, and its own otherwise, and puts that one first on the path of its terminals ([Agents](editor/agents.md#agents-in-vs-code) says how). `ascribe --version` says which version you're running.

## Start a project

With `ascribe` installed, start a project in an empty folder. `--templates` takes the [page templates](templates/index.md) too; leave it out for a project without them.

```sh
mkdir handbook && cd handbook
ascribe init --templates
```

[Getting started](getting-started.md) goes on from there, to a first page and a built site.

### `ascribe init`
@id {permalink=true}: cli-init
@available: ascribe 0.3.1

Starts a project: writes its first `ascribe.toml`, and makes the folder its pages go in, `docs/`. With `--templates`, it takes the page templates too, as [`ascribe templates eject`](templates/command.md#cli-templates-eject) with no name does.

@include: _generated/cli-init-options.md

It writes in the current folder, or in `DIR`, a path from the current folder, which it makes if it isn't there. It asks nothing. Where there's an `ascribe.toml` in the folder or above it, it refuses, names the one it found, and writes nothing: `ascribe templates eject`, run there, adds the templates to that project. To start a second project in a repository that has one, name a folder outside it. It takes no `--config`, since the project is the one it starts.

The file it writes is small, and says what each part is for:

@snippet: code:crates/ascribe-cli/tests/output/init-ascribe.toml

It sets the content root, `docs`, though that's the default, since it's the first thing a project changes. It declares one type, `page`, with a title and a description; since it declares a type, each page names it in its frontmatter, with `type: page`. Its comments name what most projects add next, each with its section of the [`ascribe.toml` reference](content-model/index.md), and it adds none of them: no site address, no builds, and no dimensions.

With `--templates`, the six types are added after `page`, and the templates, their guides, Ascribe's notes, and `templates.lock` are written into `templates/`, as `eject` writes them. When a file is where one would be written, it's named, as `eject` names it, and nothing is written.

It lists each file it wrote, `ascribe.toml` last, from the current folder; the content root, when it made it; with `--templates`, each type it added, as `eject` says it; and last, the [Astro guide](astro/index.md), to publish the pages as a site.

@snippet {lang=text}: code:crates/ascribe-cli/tests/output/init.txt

With `--format json`, as its [schema](contracts/json-reports/sources.md#ascribe-init) has it. When `templates.refused` isn't empty, `written` is empty.

.The JSON for this example
@details:
@snippet {lang=json}: code:crates/ascribe-cli/tests/output/init.json
@end

| Code | Meaning |
|---|---|
| `0` | The project is there: `ascribe.toml` is written, the content root is there, and with `--templates`, every template and its type |
| `2` | It wrote nothing: there's an `ascribe.toml` in the folder or above it, the folder is a file, a template was refused, or it was given `--config`. Or a file couldn't be written; the files it wrote before that stay. |

## See also

- [Getting started](getting-started.md): a first project, from `npm install` to a built page.
- [Command reference](command/index.md): every command, its options, and its exit codes.
