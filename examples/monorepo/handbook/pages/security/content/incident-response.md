---
title: Incident response
description: What happens when something goes wrong, from the first report to the review.
---

An incident is anything that may expose customer data or take the service down. When in doubt, call it an incident: closing one costs nothing.

## Severity levels
@id: severity-levels

| Level | Meaning | Response |
|---|---|---|
| SEV1 | Customer data exposed, or the service down for everyone. | Page the {team} lead at once, day or night. |
| SEV2 | The service degraded for some customers, no data exposed. | Respond within an hour, in working hours. |
| SEV3 | A problem with no customer impact yet. | Fix it in the normal course of work. |

## During an incident
@id: during

@steps
1. Name one incident lead. They decide; everyone else helps.
2. Open a channel for the incident, and post updates there every 30 minutes.
3. Contain first: rotate leaked secrets, block the attacker, turn off the feature.
4. Only then investigate and fix the cause.

@note {type=caution}
Don't delete logs or data during an incident, even to contain it. We need them to find out what happened.

## After an incident
@id: after

Every SEV1 and SEV2 gets a written review within five working days. Reviews are blameless: they ask what made the mistake easy, not who made it.

.The review template
@details:
- What happened, with a timeline.
- What the impact was, and on whom.
- What we did, and what worked.
- What we'll change, with an owner and a date for each change.
@end
