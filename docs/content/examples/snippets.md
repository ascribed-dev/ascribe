---
type: how-to
title: Show an example from the code
description: Declare a folder of code as a source, mark a region in a file, and show it on a page with @snippet, so the example changes when the code does.
---

With `@snippet`, a page takes a code example from the file itself, and Ascribe reads the file whenever it checks, builds, or diffs, so changing the code changes every page that shows it. [Examples and drift](index.md) says what Ascribe reports when the code changes.

## 1. Declare a source

A page can't read files outside its project's folder unless `ascribe.toml` names them. A **source** is a folder of code a page may read, with the files in it that are allowed:

```toml
[sources.code]
path = ".."                               # relative to the folder ascribe.toml is in
include = ["service/**", "examples/**"]
ignore = ["**/node_modules/**"]
```

The folder must be in the same git repository as the project. When the code is in another repository, see [Keep the docs in another repository](docs-repository.md). See [`[sources.<name>]`](../content-model/output.md#18-sourcesname) for every key.

## 2. Mark a region in the code

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

## 3. Use it in a page

```markdown
@snippet {title="Connect"}: code:service/client.py#connect
```

The address is the source, the file's path in it, and the region. The page gets a fenced code block with the region's lines, dedented, in the file's language:

```python
client = Client(host, port)
client.open()
```

Leave out `#connect` to show the whole file. The [directive reference](../writing/directives.md#snippet) has every attribute and tag.

## See also

- [Report the pages whose examples changed](drift.md): the sentences around an example, which a snippet doesn't keep current.
- [Show what an example was tested with](tested.md): a line under the example naming what it was tested with.
