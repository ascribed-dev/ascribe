---
type: reference
title: "`ascribe.toml`: what pages draw from"
description: "How the content model names what pages draw from outside the content: the source folders and repositories snippets read, and the OpenAPI specs links point into."
---

Part of the [`ascribe.toml` reference](index.md), whose [contents](index.md#contents) list every section. The rules for a loader are in the contract's [what pages draw from](../contracts/content-model/sources.md).

## 20. `[sources.<name>]`

A source is a folder outside the project's content, or another repository, that its pages may take code examples from, with [`@snippet`](../writing/directives.md#snippet) ([SPEC §7.3]({repo}/blob/main/SPEC.md#73-sources)). It's the only way a page can read a file outside the project's folder.

```toml
[sources.code]
path = ".."                               # relative to the folder ascribe.toml is in
include = ["crates/**", "examples/**"]    # what's readable; everything else isn't
ignore = ["**/target/**"]

[sources.api]
git = "https://github.com/acme/api.git"   # another repository
branch = "main"                           # what `ascribe sources update` follows
include = ["src/**", "examples/**"]
```

A page names a file through its source as `<source>:<path>`, with the path relative to the source's folder: `@snippet: code:examples/quill/ascribe.toml#dimensions`. Both kinds of source are addressed the same way, so moving code to another repository, or back, changes `ascribe.toml` and no page.

@include: ../_generated/content-model-sources.md

`<name>` follows the rules for keys (§1.2). A project can declare several sources, of either kind.

**Rules.** `path` is relative (`model-path-absolute`); its folder must exist and be a directory (`model-source-path-missing`) inside the project's repository (`model-source-outside-repository`). With no repository, as in a copy made with `git archive`, `ascribe build` and the other commands read a source wherever `path` leads, but the editor reads one only inside a folder of the workspace it opened, and `ascribe mcp` only inside the folder it was started in: open the folder that contains the source. A source has `path` or `git`, never both, and `branch` only with `git` (`model-source-remote`).

### A source in another repository

`check` and `build` never fetch anything. A source with `git` is read from copies kept in the project:

- `ascribe.lock`, beside `ascribe.toml`, pins each such source to a commit, and records the hash of each file copied from it, and which files snippets name that aren't in the repository at that commit.
- The files the project's snippets use are copied, whole, into `sources/<name>/` beside `ascribe.toml`, at the paths they have in their repository. Only those files are copied, not the repository, and not everything `include` matches. `sources/` can't be inside the content root.

Both are written by [`ascribe sources fetch` and `ascribe sources update`](../examples/sources.md#cli-sources), and committed with the rest of the docs. `check` reports a copy that was changed by hand, a copy the lock doesn't list, a copy no snippet uses any more, and a snippet whose file hasn't been copied yet, or isn't in the repository at the pinned commit because it was moved or deleted there. The copies are published to everyone who can read the docs repository: for a private code repository and public docs, that's a decision to make knowingly.

---

## 21. `[apis.<name>]`

@include: ../_fragments/content-model/apis.md

How to publish a reference and link to it is in [Link to your API reference](../openapi/index.md).

**Rules.** `spec` is a path from the project root, relative (`model-path-absolute`), inside the git repository the project is in, when it's in one (`model-api-outside-repository`). A file that isn't there, that can't be read, or that isn't YAML or JSON is an error at `spec` (`api-spec-unreadable`), and one with no operations is advice (`api-spec-empty`). `reference` is a table with `renderer` and `at`, or an address pattern with `{operationId}` and no placeholder but `{operationId}` and `{tag}` (`model-api-reference`). A link to an operation the spec doesn't have is an error at the link, with the closest ids as its fixes (`link-operation-missing`). An operation no link names, unless the spec marks it `deprecated`, is advice at `spec` (`api-operation-unreferenced`): see [Find the operations no guide links to](../openapi/coverage.md). No other key is allowed (`model-unknown-key`).
