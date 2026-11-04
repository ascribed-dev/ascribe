---
type: reference
title: JSON report contract
description: "The fields of what the `ascribe` commands write with `--format json`, from their JSON Schemas."
permalink: json-reports
---

The commands that take `--format json` each write one JSON document, and this contract gives each document's fields, as tables generated from its schema. The schemas are JSON Schema (draft 2020-12), generated from the code that writes the documents, so the two can't disagree. Each release's schemas are in the [`schemas/`]({repo}/tree/v{version}/schemas/) folder at the release's tag. The [command reference](../command/index.md) says what each command does, with an example of its document.

@available: ascribe planned
The schemas are in `@ascribed/cli` too, as `@ascribed/cli/schemas/<name>.schema.json`, so a project that pins the package has the schemas of the version it runs. A schema's `$id` is its address at the release's tag, such as `https://raw.githubusercontent.com/ascribed-dev/ascribe/v{version}/schemas/check.schema.json`, which never changes.

Every document follows these rules:

- `schema_version` changes only when a field is removed or changes meaning. Fields can be added without a new version, so **ignore fields you don't know**. The schemas allow fields they don't list.
- A field whose values are a fixed set of names, such as a diagnostic's `severity` or `next`, may gain a value without a new version. A reader treats a value it doesn't know as the field says: an unknown `severity` as advice, which never fails a check.
- A field the schema doesn't list as required is left out when it has no value. A required field that can be `null` is always written.
- Keys are snake_case, and so are the names a field's values are. That holds for every JSON document Ascribe writes, the [JSON output](json-output.md) and the files a build records among them; only the language server's and the MCP server's messages are camelCase, as their protocols are.
- `ascribe_version` is the release of Ascribe that wrote the document, such as `0.4.5`, in every command's document, so a tool can compare it. A binary built from a checkout writes its release too; `ascribe --version` says which build it is.

## Reading the tables

Each command's section has tables generated from the schema it links. The first is the document's own fields; then there's one for each type a field names, such as `Range`, in the order they're first named. A field that's an object, or a list of objects, has its fields as rows below it, under its path, such as `diagnostics[].file`; a type named once has its fields there too, and no table of its own.

- **Type** is the field's JSON type: `string`, `integer`, `boolean`, `object`, a type's name, `T[]` for a list, `[T, T]` for a list of fixed length, `{ [name]: T }` for an object of names, `"value"` for a constant, and `A | B` for either. `any` takes any value.
- **Required** says whether the field is always written. A field that isn't is left out when it has no value.
- **Nullable** says whether a required field can be `null`.

A type that's one of several kinds of object, such as a block, has a table of the fields every kind has, then one for each kind, named by the field that tells them apart: `` `type` is `"heading"` ``. A type that's one of several names has a table of the names.

## The pages

The pages are grouped as the [`ascribe.toml` reference](../content-model/index.md#contents) is, by what the commands are for:

- What pages are: [`ascribe init` and `ascribe model`](json-reports/model.md), and [`ascribe templates` and `ascribe new`](json-reports/templates.md)
- How content varies: [`ascribe versions`](json-reports/versions.md)
- Where pages are published: [`ascribe permalinks` and `ascribe addresses`](json-reports/addresses.md)
- What pages draw from: [`ascribe sources` and `ascribe observe`](json-reports/sources.md)
- Checks: `ascribe check`, `ascribe build`, `ascribe fmt`, `ascribe baseline`, and `ascribe vale eject`, on this page, [`ascribe report`](json-reports/report.md), and [`ascribe diff` and `ascribe drift`](json-reports/diff.md)
- Questions: [`ascribe explain`, `outline`, `link`, `refs`, `rename`, `resolve`, and `search`](json-reports/questions.md)
- Agents: [`ascribe agents index`, `sync`, and `prompt`](json-reports/agents.md)

## `ascribe check` and `ascribe build`

@include: ../_generated/json-check.md

## `ascribe fmt`

@include: ../_generated/json-fmt.md

## `ascribe baseline write`

@include: ../_generated/json-baseline-write.md

## `ascribe baseline prune`

@include: ../_generated/json-baseline-prune.md

## `ascribe vale eject`

@include: ../_generated/json-vale-eject.md
