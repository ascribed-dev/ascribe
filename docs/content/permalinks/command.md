---
type: reference
title: "`ascribe permalinks`"
description: "Keep a project's list of permalinks, check a product's names against it, and retire a permalink with a reason."
---

The commands that keep a project's list of permalinks, check names against it, and retire permalinks. [Permalinks](index.md) is the task they're used in. The options every command accepts are in the [command reference](../command/index.md#options-every-command-accepts), and the JSON each writes with `--format json` is in [their JSON reports' schemas](../contracts/json-reports/sources.md#ascribe-permalinks-sync).

## `ascribe permalinks`
@id {permalink=true}: cli-permalinks
@summary: Keeps the project's list of permalinks, `ascribe.permalinks`, checks names against it, and retires permalinks.

Keeps the project's list of [permalinks](../writing/directives.md#permalinks), `ascribe.permalinks` beside `ascribe.toml`, checks names against it, and retires permalinks. The list names each permalink the content declares and where, and each name [`ascribe.addresses`](../publishing/redirects.md#ascribeaddresses) moves or retires, and is committed with the docs: with it, `ascribe check` reports a permalink that goes away (`permalink-lost`) and one the list doesn't have yet (`permalinks-list-behind`), and a product's CI can check the names it links to with `verify`. A project without the list isn't told about either. Its format is in the [permalinks list contract](../contracts/permalinks-list.md).

The examples below are one project's: a page, `guides/weave.md`, with the permalinks `weaving`, `weave-config`, and `rotate-keys`, and an alias, `config`, for `weave-config`. Then `weave-config`'s section moves to `reference/loom.md`, and `rotate-keys`'s goes.

### `ascribe permalinks sync`
@id {permalink=true}: cli-permalinks-sync
@summary: Brings `ascribe.permalinks` up to date with the project, and makes it when there's none. It never drops a name the project no longer has.

Brings `ascribe.permalinks` up to date with the project, and makes it when there's none: it adds each permalink the list doesn't have, records where each is declared now, so moving a heading to another page and running `sync` changes one line, and records each name `ascribe.addresses` moves or retires as the file has it. It never drops a name the project no longer has: that stays in the list, and `ascribe check` reports it until it's back or retired, so running `sync` can't hide a removal. With `--check`, it writes nothing, and says what it would change, so CI can fail on a list that wasn't committed with the change; `ascribe check` reports the same thing as `permalinks-list-behind`.

@include: ../_generated/cli-permalinks-sync-options.md

The first time, it writes the list:

@snippet {lang=text}: code:crates/ascribe-cli/tests/output/permalinks-sync-new.txt

Once a section has moved and another has gone, `--check` says what it would change:

@snippet {lang=text}: code:crates/ascribe-cli/tests/output/permalinks-sync-check.txt

And without it, it moves the one, keeps the other, and says why:

@snippet {lang=text}: code:crates/ascribe-cli/tests/output/permalinks-sync.txt

With `--format json`, it writes the document its [schema](../contracts/json-reports/sources.md#ascribe-permalinks-sync) describes:

.The JSON for this example
@details:
@snippet {lang=json}: code:crates/ascribe-cli/tests/output/permalinks-sync.json
@end

| Code | Meaning |
|---|---|
| `0` | The list is up to date, whether or not it changed; with `--check`, it was already |
| `1` | With `--check`, the list would change: it's behind the content, or there's none |
| `2` | It couldn't run: the project doesn't load, or the list can't be read or written |

### `ascribe permalinks list`
@id {permalink=true}: cli-permalinks-list
@summary: Shows every permalink the content declares, where it's declared, and each alias and retired permalink, and which ones `ascribe.permalinks` doesn't have yet. It changes nothing.

Shows every permalink the content declares, with the file and line it's declared on, and each name `ascribe.addresses` moves or retires, and says which ones `ascribe.permalinks` doesn't have yet, has somewhere else, or has that the project no longer has. It changes nothing. Once the sections have changed, before the list is synced:

@snippet {lang=text}: code:crates/ascribe-cli/tests/output/permalinks-list.txt

@include: ../_generated/cli-permalinks-list-options.md

With `--format json`, it writes the same in the document its [schema](../contracts/json-reports/sources.md#ascribe-permalinks-list) describes. It exits with `0` whatever it finds, and `2` when the project doesn't load or the list can't be read.

.The JSON for this example
@details:
@snippet {lang=json}: code:crates/ascribe-cli/tests/output/permalinks-list.json
@end

### `ascribe permalinks verify`
@id {permalink=true}: cli-permalinks-verify
@summary: Checks that each name is a permalink or an alias in a list, and names each one that isn't, or was retired. It reads only the list, so a product's CI can run it.

Checks that each name is a permalink or an alias in a list, and names each one that isn't, and each one that was retired, with the reason the docs give. It reads only the list, not the docs, so a product's repository can run it against a copy of the docs' list: [Check the links in the product's CI](index.md#checking-a-products-links) shows the step.

```sh
ascribe permalinks verify cli-check ASC036
ascribe permalinks verify --against docs/ascribe.permalinks --from permalinks.txt
```

@include: ../_generated/cli-permalinks-verify-options.md

What it says goes to standard error. Once `rotate-keys` is retired, `ascribe permalinks verify weave-config config rotate-keys weave-colour --against ascribe.permalinks` passes the permalink and its alias, and names the other two:

@snippet {lang=text}: code:crates/ascribe-cli/tests/output/permalinks-verify.txt

With `--format json`, it writes the document its [schema](../contracts/json-reports/sources.md#ascribe-permalinks-verify) describes:

.The JSON for this example
@details:
@snippet {lang=json}: code:crates/ascribe-cli/tests/output/permalinks-verify.json
@end

| Code | Meaning |
|---|---|
| `0` | The list has every name |
| `1` | It doesn't have one, or has it as retired; each is named on standard error, a retired one with its reason |
| `2` | It couldn't check: no list where it looked, or a list or a `--from` file it can't read |

### `ascribe permalinks retire`
@id {permalink=true}: cli-permalinks-retire
@summary: Retires permalinks on purpose, with a reason, or moves them to `--to`, in `ascribe.addresses`. A retired permalink's address shows the reason; a moved one's leads to its new place.

Retires permalinks on purpose, with a reason, or moves them to `--to`: it writes an entry for each in [`ascribe.addresses`](../publishing/redirects.md#ascribeaddresses), keyed `go:` and the name, `[retired]`'s with the reason and `[moved]`'s with where it leads, making the file when there's none, and, when the project has `ascribe.permalinks`, records there each one the content no longer declares. A retired permalink's address shows the reason, and `verify` tells a product it was retired, with the reason; a moved one's leads to its new place, and `verify` accepts it. `ascribe check` no longer reports either as lost.

```sh
ascribe permalinks retire rotate-keys --reason "Merged into the security guide"
ascribe permalinks retire old-keys --to guides/security.md#keys
ascribe permalinks retire --page guides/legacy.md --reason "The sync agent was removed in 4.0"
ascribe permalinks retire --under guides/legacy/ --reason "The sync agent was removed in 4.0"
```

It retires a name given only when the content no longer declares it and `ascribe.permalinks` has it, and with `--page` or `--under`, each permalink declared in that page or in a file under that folder, or that the list has there. `--page`, `--under`, and `--to` take a path from the current folder, as `ascribe check` prints it, or from the content root; what it writes in `ascribe.addresses` is from the content root. `--to` takes a page, and `#` and a heading's id, or `go:` and another permalink's name, which makes the name an alias of it. It takes `--reason` or `--to`, not both. It refuses a name still declared anywhere else, or one `ascribe.addresses` already moves or retires, and changes nothing then. Retiring what a page holds before the page is deleted leaves each entry doing nothing (`addresses-entry-live`) until it is. The first command above, in the project above:

@snippet {lang=text}: code:crates/ascribe-cli/tests/output/permalinks-retire.txt

@include: ../_generated/cli-permalinks-retire-options.md

With `--format json`, it writes the document its [schema](../contracts/json-reports/sources.md#ascribe-permalinks-retire) describes. It exits with `0` when the names are retired or moved, and `2` when the project doesn't load, the reason is empty, a name can't be retired, or a file can't be written.

.The JSON for this example
@details:
@snippet {lang=json}: code:crates/ascribe-cli/tests/output/permalinks-retire.json
@end
