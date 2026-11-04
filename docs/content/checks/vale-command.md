---
type: reference
title: "`ascribe vale`"
description: "Write Vale's preset out as the project's own configuration: options, output, and exit codes."
---

The command that sets up Vale for a project. [Check prose with Vale](vale.md) is the task it's used in. The options every command accepts are in the [command reference](../command/index.md#options-every-command-accepts).

## `ascribe vale`
@id {permalink=true}: cli-vale

@summary
Sets up Vale for a project. Vale itself is run by `ascribe check --vale` and the editor.

### `ascribe vale eject`
@id {permalink=true}: cli-vale-eject
@summary: Writes the preset `[checks.vale]` names out as the project's own Vale configuration, `.vale.ini` and a `.vale/` styles folder, and points `[checks.vale]` at it.

Writes the preset `[checks.vale]` names out as the project's own Vale configuration: `.vale.ini` and a `.vale/` styles folder at the project root, with the rules `off` names turned off in `.vale.ini`. Then it changes `[checks.vale]` in `ascribe.toml` to `config = ".vale.ini"`, in place of `preset` and `off`, keeping its other keys and comments.

It lists each file it wrote, `ascribe.toml` last. It changes `ascribe.toml` only when git could put it back, as every [command that writes](../command/index.md#commands-that-write) does: with changes not committed, it's refused, and named, unless you pass `--force`. With `--dry-run`, it says what it would write, and writes nothing.

@include: ../_generated/cli-vale-eject-options.md

With `--format json`, it writes the preset and the files in the document its [schema](../contracts/json-reports.md#ascribe-vale-eject) describes.

| Code | Meaning |
|---|---|
| `0` | The preset is the project's own now |
| `2` | It wrote nothing: no `ascribe.toml`, a content model with errors, a `[checks.vale]` that names no preset, a `.vale.ini` or `.vale` that's there already, or, without `--force`, an `ascribe.toml` git couldn't put back. Or a file couldn't be written; the files it wrote before that stay. |

## See also

- [Check prose with Vale](vale.md): the preset, your own configuration, and where Vale runs.
- [`[checks.vale]`](../content-model/checks.md#checksvale): every key.
