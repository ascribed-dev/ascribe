---
type: how-to
title: Install
description: "Install the ascribe command: in a project with npm, or on your path with Homebrew, the install script, PowerShell, or a release's archive."
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

To try what's on `main` before it's released, install `@ascribed/cli@next` and `@ascribed/astro@next` instead: a build published every night, with no promise of stability.

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

A machine can have more than one `ascribe`: the one on your path, the one `@ascribed/cli` pins in a project, and the one the editor's extension bundles. A bare `ascribe` runs your path's; `npx ascribe` runs the project's; the editor runs the project's when there is one, and its own otherwise, and puts that one first on the path of its terminals ([Agents](agents/index.md#vs-code) says how). `ascribe --version` says which version you're running.

## See also

- [Getting started](getting-started.md): a first project, from `npm install` to a built page.
- [Command reference](command/index.md): every command, its options, and its exit codes.
