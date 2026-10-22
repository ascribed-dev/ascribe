---
type: reference
title: JSON report contract
description: The JSON Schemas of what the ascribe commands write with --format json.
---

The commands that take `--format json` each write one JSON document, and this contract gives each document's schema. The schemas are JSON Schema (draft 2020-12), generated from the code that writes the documents, so the two can't disagree. They're in the repository's [`schemas/`]({repo}/tree/main/schemas/) folder. The [command reference](../command/index.md) says what each command does, with an example of its document.

Every document follows these rules:

- `schema_version` changes only when a field is removed or changes meaning. Fields can be added without a new version, so **ignore fields you don't know**. The schemas allow fields they don't list.
- A field whose values are a fixed set of names, such as a diagnostic's `severity` or `next`, may gain a value without a new version. A reader treats a value it doesn't know as the field says: an unknown `severity` as advice, which never fails a check.
- A field the schema doesn't list as required is left out when it has no value. A required field that can be `null` is always written.
- Keys are snake_case.

The schemas of `ascribe check`, `ascribe build`, `ascribe fmt`, and `ascribe baseline` are on this page, and the rest on six more:

- [`ascribe diff` and `ascribe drift`](json-reports/diff.md)
- [`ascribe report`](json-reports/report.md)
- [`ascribe init`, `ascribe sources`, `ascribe observe`, and `ascribe permalinks`](json-reports/sources.md)
- [`ascribe templates`](json-reports/templates.md)
- [`ascribe versions`](json-reports/versions.md)
- [`ascribe explain`, `model`, `outline`, `link`, `refs`, `resolve`, `search`, and `agents index`](json-reports/questions.md)

## `ascribe check` and `ascribe build`

@include: ../_generated/json-check.md

## `ascribe fmt`

@include: ../_generated/json-fmt.md

## `ascribe baseline write`

@include: ../_generated/json-baseline-write.md

## `ascribe baseline prune`

@include: ../_generated/json-baseline-prune.md
