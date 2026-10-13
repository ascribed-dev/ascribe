---
type: reference
title: Tested examples contract
description: The formats of ascribe.examples.toml, which says which of a source's files are examples and how to test them, and of the observation ascribe observe writes beside it.
available: next
---

This contract says what the two files of a source's tested examples hold: `ascribe.examples.toml`, which the code's owners write, and the observation `ascribe observe` writes beside it. Both are TOML. [Show what an example was tested with](../examples/tested.md) walks through them, and [`ascribe observe`](../examples/sources.md#cli-observe) is the command that reads the first and writes the second.

## 1. Where they are

Both are in a source's folder: `ascribe.examples.toml` at its root, and the observations in `examples.observations/` beside it, the one `ascribe observe` writes named `default.toml`. They're read from there whatever the source's own `include` says, and no snippet can address either: a `@snippet` of `ascribe.examples.toml`, or of a file under `examples.observations/`, reports what a file outside the source's `include` does. Every path in either file is relative to that folder, with `/` between segments, which is also how a snippet's path in the source is written.

For a source in another repository, the folder is its copies, `sources/<name>/`. [`ascribe sources fetch` and `update`](../examples/sources.md#cli-sources-fetch) copy the repository's `ascribe.examples.toml` and every file in its `examples.observations/` there, at the pin, beside the files snippets use, and `ascribe.lock` lists each with its hash, as it does every copy ([the lock](content-model/output.md)). A repository with neither copies nothing more.

`check`, `build`, `sources`, and the editor read both files and never run a command in either. Only `ascribe observe` runs the config's commands, and only from the folder it's run in. It refuses a folder that's a source's copies, which it knows from the `ascribe.lock` of a project above it, and exits 2: a config that arrived as a copy is data, and its `test` and commands are shown nowhere.

## 2. `ascribe.examples.toml`

| Key | Type | Means |
|---|---|---|
| `include` | list of patterns | The files that are examples. Required, and not empty |
| `ignore` | list of patterns | Files `include` matches that aren't examples |
| `test` | string | The line of shell that tests them. Required by `ascribe observe`, and read by nothing else |
| `[environment]` | table | What to note about where the test ran: each entry a table with a `command` and an optional `label` |

- **Patterns** are the content model's, as a source's `include` takes them, matched against a file's path from the folder. A file is an example when a pattern in `include` matches it and none in `ignore` does. Nothing in `examples.observations/` is an example, whatever `include` says.
- **An `[environment]` entry's name** is a lowercase letter, then lowercase letters, digits, or hyphens. `os` and `arch` are noted in every observation, so a config can't declare them.
- **`command`** is a line of shell whose first line of output is kept. It's required.
- **`label`** is what a page shows before the entry's value: `Node` in "Node 24.9.0". Without one, it's the entry's name.
- **A key not in this table,** or in an entry, is an error, as in `ascribe.toml`.

A config that can't be read is an error for `ascribe observe`, which exits 2, and the warning [`examples-config-invalid`](../checks/diagnostics/content-checks.md#ASC201) for `ascribe check`, at the source in `ascribe.toml`. Then none of the source's files is an example.

## 3. The observation

```toml
# Written by `ascribe observe`. Don't edit it by hand.
version = 1
ascribe = "0.4.0"
command = "node --test test/greet.test.mjs"

[environment]
os = "linux"
arch = "x64"
node = "v24.9.0"

[files]
"examples/greet.out.txt" = "sha256-lf:2314…"
"examples/greet.ts" = "sha256-lf:224c…"
```

| Key | Type | Means |
|---|---|---|
| `version` | integer | The format's version: `1` |
| `ascribe` | string | The version of Ascribe that wrote it |
| `command` | string | The `test` line that passed |
| `[environment]` | table of strings | `os`, `arch`, then each of the config's entries, by name, in the config's order |
| `[files]` | table of strings | Each example's hash, by path, in path order |

- **`os`** is `linux`, `macos`, or `windows`, or another system's own name. **`arch`** is `x64`, `arm64`, or `ia32`, or another machine's own name.
- **An entry's value** is the first line its command printed, without the white space around it, cut at 200 characters. A command that fails, or prints nothing, fails the run, so every entry has a value.
- **A hash** is `sha256-lf:` and 64 lowercase hexadecimal digits: the SHA-256 of the file's bytes with each CR LF read as LF, so a file hashes the same on a Windows checkout and a Linux one.
- **The text** is the same for the same observation on every platform: the keys in the order above, LF line endings, and nothing that changes from run to run, such as a time.

An observation with a key not in this table, or a `version` other than `1`, can't be read, and reports [`examples-config-invalid`](../checks/diagnostics/content-checks.md#ASC201). Its files are then unobserved.

## 4. An example's status

For a snippet of an example, the observation decides its status:

| Status | When | What the page shows | What `ascribe check` reports |
|---|---|---|---|
| current | `[files]` lists it with the hash it has now | The tested line | Nothing |
| stale | `[files]` lists it with another hash | Nothing | [`example-stale`](../checks/diagnostics/content-checks.md#ASC199), advice |
| unobserved | `[files]` doesn't list it, or there's no observation | Nothing | [`example-unobserved`](../checks/diagnostics/content-checks.md#ASC200), advice |

For a source in another repository, the observation and the file are the copies, so an example's status is what it was at the pin. Its advice says the fix is in that repository, and arrives with the next `ascribe sources update`. The observation usually lists more files than the docs have copies of, which is expected: only the files snippets use are copied.

## 5. The tested line

A current example's code block carries `tested="…"` in its info string, after `title` when it has one ([SPEC §4.8]({repo}/blob/main/SPEC.md)). It's each of the config's `[environment]` entries the observation has a value for, in the config's order, as its label and its shown value, then `on` and the operating system: `Node 24.9.0 and TypeScript 5.9.2 on Linux`. With no entries, it's the operating system alone.

- **The shown value** is the first dotted number in the kept line, `24.9.0` from `v24.9.0`, or the whole line when it has none.
- **The operating system** is `Linux`, `macOS`, or `Windows`, or the recorded name. The architecture isn't shown.
- **A label or a value** has its control characters and runs of white space shown as one space, and is cut at 60 characters, with `…`. Each output escapes what it shows.

The [site-render contract](site-render.md#51-a-tested-code-block) says how the site shows it, and the plain output writes "Tested with" and the line, and a full stop, on a line under the block.
