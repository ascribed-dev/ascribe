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

The tags are [Bluehawk](https://github.com/mongodb-university/Bluehawk)'s, so code already tagged for it works as it is:

- `:snippet-start: <name>` opens a region, and `:snippet-end:` closes the innermost open one. `:snippet-end: <name>` closes the region with that name, so regions can overlap as well as nest. A region name is letters, digits, `-`, `_`, and `.`, and is used once in a file.
- `:remove-start:` and `:remove-end:` leave out the lines between them, and a line that ends with `:remove:` in a comment is left out itself.
- Tag lines and removed lines never appear in a snippet, nor do blank lines at the start and end of what's left, so a formatter's blank line before an end tag doesn't end up in the page. What's left is dedented by its common indentation.
- A tag counts only in a comment on a line of its own: `//` (C, Go, Java, JavaScript, Rust, TypeScript, and others), `#` (Python, Ruby, shell, TOML, YAML, GraphQL, and others), `--` (Lua, SQL, Haskell, Elm), `;` (INI, Lisp), `<!-- -->` (HTML, XML, Markdown), or `/* */` (CSS). Astro, Svelte, and Vue components take either `//` or `<!-- -->`, so a region can be marked in the frontmatter or script and in the markup, and SCSS and Less take either `//` or `/* */`.
- A file with no extension of its own is matched by its whole name: `Dockerfile`, `Containerfile`, `Makefile`, `GNUmakefile`, `Justfile`, `.env` and `.env.*`, and `.gitignore` take `#` comments, and a snippet of one gets a language from its name, such as `dockerfile` for a `Dockerfile`. A file whose name and extension aren't in the [tables in the spec]({repo}/blob/main/SPEC.md#48-snippet) can be used whole, but not by region.
- Bluehawk's other tags (`state`, `replace`, `uncomment`, and `emphasize`) are reserved. A file that uses one can't be used by a snippet yet, so its code is never shown with a tag left in.

## 3. Use it in a page

```markdown
@snippet {title="Connect"}: code:service/client.py#connect
```

The address is the source, the file's path in it, and the region. The page gets a fenced code block with the region's lines, dedented, in the file's language:

```python
client = Client(host, port)
client.open()
```

Leave out `#connect` to show the whole file. The [directive reference](../writing/directives.md#snippet) has every attribute.

## See also

- [Report the pages whose examples changed](drift.md): the sentences around an example, which a snippet doesn't keep current.
- [Show what an example was tested with](tested.md): a line under the example naming what it was tested with.
