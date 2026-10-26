---
type: reference
title: The permalinks list
description: The format of ascribe.permalinks, the list of a project's permalinks that something outside the docs can check its links against.
---

A project can commit a list of its [permalinks](../writing/directives.md#permalinks), `ascribe.permalinks`, beside `ascribe.toml` (SPEC §7.5), with the names `ascribe.addresses` moves or retires (§7.6). With it, `ascribe check` warns when a permalink goes away, and something outside the docs, such as a product's CI, can check the names it links to against a copy of the file, without the docs. This contract says what the file holds, how Ascribe reads and writes it, and what a reader of a copy can rely on.

| Who | Uses this contract to |
|---|---|
| `ascribe permalinks sync` | Write the list |
| `ascribe check` and the editor | Compare it with what the content declares |
| `ascribe permalinks verify`, or a tool of your own | Check names against a copy of it |

## 1. The file

```toml
# Written by Ascribe. Don't edit it: run `ascribe permalinks sync`.
version = 1

[permalinks]
ASC036 = "_generated/diagnostics-source-files.md#ASC036"
cli-check = "reference/cli.md#cli-check"
cli-reference = "reference/cli.md"

[aliases]
check = "cli-check"

[moved]
rotate-keys = "guides/security.md#keys"

[retired]
legacy-sync = "The sync agent was removed in 4.0"
```

It's TOML, with these keys:

- **`version`** is `1`. A later format gets a new number, and a version of Ascribe that doesn't read it says so (`permalinks-list-invalid`) rather than guess.
- **`[permalinks]`** has an entry for each permalink, keyed by its name. A name is a valid id: letters, digits, hyphens, underscores, and periods. A name with a period is quoted, as TOML requires.

An entry's value is where the name is **declared**: the path of the file it's written in, from the content root, and, for a heading, `#` and the heading's id. A page's own permalink, from its frontmatter, is the page's path alone. A heading in a fragment is declared in the fragment, whichever pages include it.

- **`[aliases]`** has an entry for each name [`ascribe.addresses`](../publishing/redirects.md#ascribeaddresses) moves to another permalink, `"go:check" = "go:cli-check"`, keyed by its name, whose value is the permalink's name. Absent when there are none.
- **`[moved]`** has an entry for each name `ascribe.addresses` moves to a page, keyed by its name, whose value is where, as the file writes it: a page's path from the content root, and `#` and a heading's id. Absent when there are none.
- **`[retired]`** has an entry for each name `ascribe.addresses` retires, keyed by its name, whose value is why it was retired. Absent when there are none.

A name is in one of the four tables, never two. The three tables were added to version 1: a list without them is a list with none, and a version of Ascribe that predates one reports a list with it as invalid rather than misread it.

A place is where the name is written in the source, not the address a reader goes to: that's the site's address, the permalinks' path, and the name, the same in every build ([Permalinks](../permalinks/index.md)).

## 2. What it lists

The list is the project's, not one build's. Each permalink the content declares, by a page's `permalink` or an `@id {permalink=true}`, in any source file, has an entry, whether or not a build publishes it. A name declared in more than one place, such as on two pages no build publishes together, has one entry, at any of those places. Each name `ascribe.addresses` moves or retires has an entry too, unless the content declares the name, whose entry in `ascribe.addresses` does nothing then (`addresses-entry-live`).

## 3. How Ascribe checks it

`ascribe check` compares the list with the content whenever it checks the project, and the editor does the same. Without the file, there's nothing to compare, and nothing is reported.

| Diagnostic | When |
|---|---|
| [`permalinks-list-behind`](../checks/diagnostics/pages.md#ASC176) | A declared name has no entry, or its entry is at a place that doesn't declare it any more. It's reported where the name is declared. A name `ascribe.addresses` moves or retires without its entry, or with an entry that says otherwise, is reported on it in `ascribe.addresses`. |
| [`permalink-lost`](../checks/diagnostics/pages.md#ASC177) | An entry's name isn't declared anywhere, and `ascribe.addresses` neither moves nor retires it. It's reported on the entry, in `ascribe.permalinks`. The editor's quick fix retires it. |
| [`permalinks-list-invalid`](../checks/diagnostics/pages.md#ASC175) | The file isn't a list this version reads. The list isn't used until it's fixed. |

Moving a permalink to another heading or page makes the list behind, and doesn't lose it: the address still works. Retiring or moving a lost permalink, with [`ascribe permalinks retire`](../permalinks/command.md#cli-permalinks-retire), ends the warning: the name moves to `[retired]`, `[moved]`, or `[aliases]`.

## 4. How Ascribe writes it

[`ascribe permalinks sync`](../permalinks/command.md#cli-permalinks-sync) writes the list, and the editor's quick fix on `permalinks-list-behind` writes the same text. Writing the list:

- adds an entry for each declared name the list doesn't have;
- moves an entry whose place no longer declares its name to a place that does;
- keeps an entry whose name isn't declared any more, so writing the list never hides a removal;
- writes each name `ascribe.addresses` moves or retires to `[aliases]`, `[moved]`, or `[retired]`, moving a name there from `[permalinks]` when it was declared before.

The text is the same for the same list on every platform: the comment, `version`, a blank line, `[permalinks]`, then one line for each name, sorted, then, when there are any, a blank line, `[aliases]`, and its lines, and the same for `[moved]` and `[retired]`, with `\n` line endings. So a change to the list is a change of a line or two in review.

## 5. Reading a copy

A copy of the file is all a tool needs to check a link: a name is a permalink of the docs when the copy's `[permalinks]`, `[aliases]`, or `[moved]` has it. A name in `[retired]` was a permalink, and isn't now: its address shows the reason, and a link to it should change. That says the name is declared, not that it has an address: a build gives it one only when it publishes the page or heading that declares it, so a marked heading in a fragment no page includes, or on a page no build publishes, is listed and leads nowhere (`fragment-unused` and `page-orphan` point at most of these). [`ascribe permalinks verify`](../permalinks/command.md#cli-permalinks-verify) does this, and [Checking a product's links](../permalinks/index.md#checking-a-products-links) shows it in CI. A tool of your own can read the file with any TOML parser, and:

- should fail on a `version` it doesn't know, rather than read it as version 1;
- should fail on a name in `[retired]`, saying why it was retired;
- shouldn't rely on the places, which change when the docs are reorganized; only the names are kept.
