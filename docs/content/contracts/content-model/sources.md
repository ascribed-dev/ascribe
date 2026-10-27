---
type: reference
title: "`ascribe.toml` format contract: what pages draw from"
nav-label: "What pages draw from"
description: "The normative contract for the content model's sources, with `ascribe.lock`, and APIs."
---

Part of the [`ascribe.toml` format contract](../content-model.md), whose [contents](../content-model.md#contents) list every section. What these keys are for, with examples, is in the reference's [what pages draw from](../../content-model/sources.md).

## 20. `[sources.<name>]`

A source is a folder outside the content root, or another repository, that a page's snippets MAY read (SPEC §4.8, §7.3, §7.4); a page MUST NOT read a file outside the project's folder any other way. `<name>` is the source's name (§1.2), and a snippet's address starts with it.

@include: ../../_generated/content-model-sources.md

**Rules** (§25.8): a source MUST have exactly one of `path` and `git`, `branch` only with `git`, a URL `git` can be given, and a branch name `git` accepts (`model-source-remote`). `path` MUST be relative (`model-path-absolute`) and name a directory that exists (`model-source-path-missing`), inside the git repository the project is in, when it's in one (`model-source-outside-repository`). With no repository, the language server and `ascribe mcp` MUST read a source only inside what the user opened, a folder of the workspace or the folder `ascribe mcp` was started in, and report one outside it as `model-source-outside-repository` with the variant `opened`; a command reads a source wherever `path` leads. With `git`, `sources/` MUST be outside the content root (`model-source-remote`).

### 20.1 `ascribe.lock` and the copies

A source with `git` is pinned to a commit in `ascribe.lock`, beside `ascribe.toml`, and the files its snippets use are copied into `sources/<name>/` at the paths they have in their repository. So are the repository's `ascribe.examples.toml` and every file in its `examples.observations/`, when it has them, whatever the source's `include` says ([tested examples](../tested-examples.md#1-where-they-are)). Both are written by `ascribe sources fetch` and `ascribe sources update`, and committed. `check` and `build` read only the copies.

```toml
# Written by `ascribe sources fetch` and `ascribe sources update`. Don't edit it by hand.
version = 1

[[source]]
name = "api"
git = "https://github.com/acme/api.git"
commit = "9f2c41d0e0c4a1b2c3d4e5f60718293a4b5c6d7e"
missing = ["examples/quickstart.ts"]

[source.files]
"src/auth.rs" = "sha256:5d41402abc4b2a76b9719d911017c592ae2fd2b1f6b3b0f0d5ce64b4c7d0e1a2"
```

| Key | Type | Description |
|---|---|---|
| `version` | integer | The lock's format: `1`. |
| `source[].name` | string | The source's name. Each source is pinned once. |
| `source[].git` | string | The source's `git` when it was pinned. |
| `source[].commit` | string | The commit, in full: 40 or 64 hexadecimal digits. |
| `source[].files` | table | Each copy's path, relative to the source's folder, and the SHA-256 of its bytes: `sha256:` and 64 lowercase hexadecimal digits. |
| `source[].missing` | array of strings | Optional. The paths, relative to the source's folder, that snippets named and that weren't in the repository at the commit, so weren't copied. No path is in both `files` and `missing`. |

`check` reads the lock and the copies with the source files, and reports what doesn't match: a lock it can't read (`lock-invalid`); a pin of a source `ascribe.toml` doesn't declare with `git`, or declares with another `git` (`lock-source-unknown`); a copy that's missing or doesn't have its hash (`source-copy-changed`); a file in `sources/<name>/` the lock doesn't list (`source-copy-unlocked`); and a copy no snippet uses (`source-copy-unused`, a warning), which a tested examples' config or observation never is. A snippet that names a file with no copy is `snippet-file-missing`: its message says to run `ascribe sources fetch`, or, for a file the lock lists in `missing`, that the file isn't in the repository at the pinned commit, which `fetch` can't change. These are source-file diagnostics; see [the diagnostics reference](../../checks/diagnostics.md).

---

## 21. `[apis.<name>]`

@include: ../../_fragments/content-model/apis.md

**Rules** (§25.13): `spec` MUST be relative (`model-path-absolute`) and, when the project is in a git repository, inside it (`model-api-outside-repository`); with no repository, the language server and `ascribe mcp` MUST read a spec only inside what the user opened, and report one outside it as `model-api-outside-repository` with the variant `opened`, as a source's folder is (§18). `reference` MUST be a table with exactly `renderer` and `at`, `renderer` one of the choices (`model-missing-key`, `model-unknown-key`, `model-invalid-value`), or a string, an address pattern, with `{operationId}` and no placeholder but `{operationId}` and `{tag}` (`model-api-reference`). What needs the spec's file is checked with the project, not by the loader: SPEC §8.2's `api-spec-unreadable`, `api-spec-empty`, `link-operation-missing`, and `link-spec-outside-project`. A compiler MUST percent-encode each value an address pattern's placeholder takes, keeping only RFC 3986's unreserved characters, and MUST join an address that isn't absolute to the site's `[consumer] base-path`, without a line's folder.
