---
title: Drift
description: Keeping pages from falling behind the code they describe, starting with code examples taken from tested files.
available: next
---

Docs drift when the code changes and the page that describes it doesn't. Ascribe catches what it can when you check, build, or diff, so a reader never sees the old version and a reviewer sees which pages a change reaches.

## Examples from tested code

A code example copied into a page goes stale the day its code changes, and nothing says so. With `@snippet`, a page takes the example from the file itself: the file that's built, tested, and run. There's no copy to fall behind. Ascribe reads the file whenever it checks, builds, or diffs, so changing the code changes every page that shows it.

### 1. Declare a source

A page can't read files outside its project's folder unless `ascribe.toml` names them. A **source** is a folder of code a page may read, with the files in it that are allowed:

```toml
[sources.code]
path = ".."                               # relative to the folder ascribe.toml is in
include = ["service/**", "examples/**"]
ignore = ["**/node_modules/**"]
```

The folder must be in the same git repository as the project. See [`[sources.<name>]`](../reference/content-model.md#18-sourcesname) for every key.

### 2. Mark a region in the code

To show part of a file, mark it with tags in comments. Tag lines are left out of the example, and so are lines you mark to remove:

```python
def connect(host, port):
    # :snippet-start: connect
    client = Client(host, port)
    client.authenticate(load_token())  # :remove:
    client.open()
    # :snippet-end:
    return client
```

The tags are [Bluehawk](https://github.com/mongodb-university/Bluehawk)'s, so code already tagged for it works as it is. A file whose language Ascribe doesn't know the comment syntax of can still be used whole.

### 3. Use it in a page

```markdown
@snippet {title="Connect"}: code:service/client.py#connect
```

The address is the source, the file's path in it, and the region. The page gets a fenced code block with the region's lines, dedented, in the file's language:

```python
client = Client(host, port)
client.open()
```

Leave out `#connect` to show the whole file. The [directive reference](../reference/directives.md#snippet) has every attribute and tag.

### When the code changes

- **Check.** `ascribe check` and the editor report a snippet whose file, source, or region doesn't exist, and a file whose tags don't balance, at the `@snippet` line. Renaming a region or moving a file breaks the build instead of the example.
- **Build.** Every output has the code as it is now. In the JSON output, the code block names its address and the lines of the file it came from.
- **Diff and review.** [`ascribe diff`](../reference/cli.md#ascribe-diff) and review list every page whose snippet's code changed, with its address as what the change comes from: `install.md: 1 changed (through code:service/client.py#connect)`. The base revision's code is read from git, the same way the pages are.
