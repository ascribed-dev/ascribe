---
title: CLI reference
description: Every command of the lantern CLI.
since: "2.0"
---

Every command takes `--project` to choose a project, and `--output json` for scripts.

## lantern login
@id: login

Signs in and saves a token in `~/.lantern/credentials`. With `--server`, signs in to a self-hosted server instead of {cloud}.

## lantern flags
@id: flags

| Command | What it does |
|---|---|
| `lantern flags create <key>` | Creates a flag. `--type` is `boolean`, `string`, or `number`. |
| `lantern flags get <key>` | Prints a flag's definition. |
| `lantern flags enable <key>` | Turns a flag on, for everyone or for `--user`. |
| `lantern flags disable <key>` | Turns a flag off for everyone. |
| `lantern flags archive <key>` | Archives a flag. Its key can't be reused. |

## lantern rollouts
@id: rollouts

| Command | What it does |
|---|---|
| `lantern rollouts set <key>` | Sets a rollout by `--percent` or `--segment`. |
| `lantern rollouts schedule <key>` | Raises a rollout on a schedule. |

## lantern audit
@id: audit
@available: audit-log

Prints who changed what, and when:

```shell
lantern audit --project checkout --since 7d
```

## Exit codes
@id: exit-codes

| Code | Meaning |
|---|---|
| `0` | Success. |
| `1` | The command failed, such as a flag that doesn't exist. |
| `2` | The command line was wrong. |
