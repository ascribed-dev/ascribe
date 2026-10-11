---
title: Install Lantern
---

## Install the CLI

@variant {platform=linux}:
```sh
curl -fsSL https://lantern.example.com/install.sh | sh
```
@variant {platform=macos}:
```sh
brew install lantern
```
@variant {platform=windows}:
```powershell
winget install Lantern.CLI
```
@end

## Start it at login
@available: linux, macos

`lantern service install` starts Lantern when you log in, and keeps it running in the background.

## Sign in

Run `lantern login` and follow the link it prints.
