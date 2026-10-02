---
title: Roll out a flag
description: Turn a flag on for a percentage of users, a segment, or on a schedule.
---

A rollout turns a flag on for more users over time, so a problem reaches a few users instead of all of them.

## Percentage rollouts
@id: percentage-rollouts

A percentage rollout turns the flag on for a share of users. Each user's share is stable: someone who sees the new checkout at 10% still sees it at 20%.

```shell
lantern rollouts set new-checkout --percent 10
```

Raise the percentage in steps, and watch your error rates between them. A common plan is 1%, 10%, 50%, then 100%.

## Segments
@id: segments

A segment is a saved group of users, matched by attributes your app sends with each check:

```shell
lantern segments create beta-testers --match "plan == 'enterprise' && beta == true"
lantern rollouts set new-checkout --segment beta-testers
```

@note
Segments match on the attributes you pass to `lantern.enabled()`. An attribute your app doesn't send never matches.

## Scheduled rollouts
@id: scheduled-rollouts
@available: scheduled-rollouts

A scheduled rollout raises the percentage for you, on a timetable:

```shell
lantern rollouts schedule new-checkout --steps "1%@09:00,10%@13:00,50%@tomorrow 09:00"
```

{product} pauses the schedule if the flag's error rate rises above the threshold you set with `--max-error-rate`.

## Roll back
@id: roll-back

To turn a flag off for everyone at once:

```shell
lantern flags disable new-checkout --project checkout
```

The change reaches every SDK within a few seconds. Rolling back doesn't delete the rollout, so you can resume it once you've fixed the problem.
