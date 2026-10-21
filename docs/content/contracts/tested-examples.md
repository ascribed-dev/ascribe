---
type: reference
title: Tested examples contract
description: The formats of ascribe.examples.toml, which says which of a source's files are examples and how to test them, and of the observation ascribe observe writes beside it.
---

This contract says what the two files of a source's tested examples hold: `ascribe.examples.toml`, which the code's owners write, and the observation `ascribe observe` writes beside it. Both are TOML. [Show what an example was tested with](../examples/tested.md) walks through them, and [`ascribe observe`](../examples/sources.md#cli-observe) is the command that reads the first and writes the second.

## 1. Where they are

Both are in the folder a source's `path` names: `ascribe.examples.toml` at its root, and the observations in `examples.observations/` beside it, each named `<name>.toml`: `default.toml` for the one `ascribe observe` writes, and another for each `--name`. A name is lowercase words of letters and digits joined by `-`, such as `node-22`; a file in the folder named otherwise isn't an observation, and is left alone. They're read from there whatever the source's own `include` says, and no snippet can address either: a `@snippet` of `ascribe.examples.toml`, or of a file under `examples.observations/`, reports what a file outside the source's `include` does. Every path in either file is relative to that folder, with `/` between segments, which is also how a snippet's path in the source is written.

For a source in another repository, the folder is its copies, `sources/<name>/`. [`ascribe sources fetch` and `update`](../examples/sources.md#cli-sources-fetch) copy the repository's `ascribe.examples.toml` and every file in its `examples.observations/` there, at the pin, beside the files snippets use, and `ascribe.lock` lists each with its hash, as it does every copy ([the lock](content-model/output.md#181-ascribelock-and-the-copies)). A repository with neither copies nothing more.

`check`, `build`, `sources`, and the editor read both files and never run a command in either. Only `ascribe observe` runs the config's commands, and only from the folder it's run in; when its input is a terminal, as in the editor's Observe Examples task, it lists them and asks before running them. It refuses a folder that's a source's copies, which it knows from the `ascribe.lock` of a project above it, and exits 2: a config that arrived as a copy is data, and its `test` and commands are shown nowhere.

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

An observation with a key not in this table can't be read, and reports [`examples-config-invalid`](../checks/diagnostics/content-checks.md#ASC201), once for each problem. One whose `version` isn't `1` is in a format this version of Ascribe doesn't know: it's one `examples-config-invalid`, naming the `ascribe` that wrote it, and nothing else in it is read. Either way it counts for none of its files, and the other observations still count.

Each observation is whole and separate, with its own environment and hashes. One whose name nothing writes any more stays until someone deletes it, and goes stale like any other. A file one lists that isn't in the folder reports [`observation-file-missing`](../checks/diagnostics/content-checks.md#ASC202), advice, at its line in the observation.

## 4. An example's status

For a snippet of an example, the observations that can be read decide its status:

| Status | When | What the page shows | What `ascribe check` reports |
|---|---|---|---|
| current | At least one lists it with the hash it has now | The tested line, from those | Nothing |
| stale | One or more list it, none with the hash it has now | Nothing | [`example-stale`](../checks/diagnostics/content-checks.md#ASC199), advice |
| unobserved | None lists it, or there's no observation | Nothing | [`example-unobserved`](../checks/diagnostics/content-checks.md#ASC200), advice |

For a source in another repository, the observation and the file are the copies, so an example's status is what it was at the pin. Its advice says the fix is in that repository, and arrives with the next `ascribe sources update`. The observation usually lists more files than the docs have copies of, which is expected: only the files snippets use are copied.

## 5. The tested line

A current example's code block carries `tested="…"` in its info string, after `title` when it has one ([SPEC §4.8]({repo}/blob/main/SPEC.md)). It's made from the observations current for the file: each of the config's `[environment]` entries one of them has a value for, in the config's order, as its label and its shown values, then `on` and the operating systems: `Node 24.9.0 and TypeScript 5.9.2 on Linux`. With no entries, it's the operating systems alone.

- **An entry is said once,** with each distinct shown value: `Node 22.4.0 and 24.9.0`. An entry only some of the observations have is said for those. Two entries, one of which lists values, are parted by a comma too: `Node 22.4.0 and 24.9.0, and TypeScript 5.9.2`.
- **The values are in order:** by version when every one is a dotted number, comparing each number in turn, and by text otherwise, ignoring case.
- **Past three values,** an entry is said as how many there are and the lowest and highest: `Node, 5 versions from 18.20.0 to 24.9.0`, or `values` when they aren't versions.
- **The operating systems** are merged the same way, always listed: `on Linux and macOS`.

- **The shown value** is the first dotted number in the kept line, `24.9.0` from `v24.9.0`, or the whole line when it has none.
- **The operating system** is `Linux`, `macOS`, or `Windows`, or the recorded name. The architecture isn't shown.
- **A label or a value** has its control characters and runs of white space shown as one space, and is cut at 60 characters, with `…`. Each output escapes what it shows.

The [site-render contract](site-render.md#51-a-tested-code-block) says how the site shows it, and the plain output writes "Tested with" and the line, and a full stop, on a line under the block.

## 6. In the JSON output

A code block's `snippet` in the JSON output has `tested` when its file is an example, and on no other block ([decision #458c]({repo}/blob/main/project-docs/decisions/458c-tested-json.md)). The key is new, and optional: a reader that doesn't know it can skip it.

```json
"tested": {
  "status": "current",
  "observations": [
    { "name": "default", "environment": { "os": "linux", "arch": "x64", "node": "v24.9.0" } },
    { "name": "node-22", "environment": { "os": "linux", "arch": "x64", "node": "v22.4.0" } }
  ]
}
```

| Key | Means |
|---|---|
| `status` | `current`, `stale`, or `unobserved`, as §4 has them |
| `observations` | The observations current for the file, in name order, each with its `name` and its `environment` as it has it: `os`, `arch`, then each entry's kept line, not its shown value, so a reader parses versions its own way. Empty unless `status` is `current` |

Labels aren't in it: they're in the config. `ascribe report --format json` counts the same statuses for each source, in `inventory.examples`.
