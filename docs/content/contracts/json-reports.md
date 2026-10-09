---
title: JSON report contract
description: The JSON Schemas of what the ascribe commands write with --format json.
---

The commands that take `--format json` each write one JSON document, and this contract gives each document's schema. The schemas are JSON Schema (draft 2020-12), generated from the code that writes the documents, so the two can't disagree. They're in the repository's [`schemas/`]({repo}/tree/main/schemas/) folder. The [command reference](../reference/cli.md) says what each field is for, with an example of each document.

Every document follows these rules:

- `schema_version` changes only when a field is removed or changes meaning. Fields can be added without a new version, so **ignore fields you don't know**. The schemas allow fields they don't list.
- A field the schema doesn't list as required is left out when it has no value. A required field that can be `null` is always written.
- Keys are snake_case.

## `ascribe check` and `ascribe build`

@include: ../_generated/json-check.md

## `ascribe diff`
@available: next

@include: ../_generated/json-diff.md

## `ascribe drift`
@available: next

@include: ../_generated/json-drift.md

## `ascribe sources status`
@available: next

@include: ../_generated/json-sources-status.md

## `ascribe sources update`
@available: next

@include: ../_generated/json-sources-update.md

## `ascribe explain`
@available: next

@include: ../_generated/json-explain.md

With `--list`:

@include: ../_generated/json-explain-list.md

## `ascribe model`
@available: next

@include: ../_generated/json-model.md

## `ascribe outline`
@available: next

@include: ../_generated/json-outline.md

## `ascribe link`
@available: next

@include: ../_generated/json-link.md

## `ascribe refs`
@available: next

@include: ../_generated/json-refs.md

## `ascribe render`
@available: next

@include: ../_generated/json-render.md
