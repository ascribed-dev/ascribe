---
title: On call
description: What being on call means, and what to do when you're paged.
owner: sre
---

Each team has an on-call rotation of one week. While you're on call, you respond to pages for your team's services within 15 minutes, day or night.

## When you're paged
@id: when-paged

@steps
1. Acknowledge the page, so the rotation doesn't escalate it.
2. Check the service's dashboard and its recent deploys.
3. If customers are affected, open an incident in {help} with `/incident start`.
4. Fix it or roll it back, then write down what happened while it's fresh.

@note {type=warning}
If you think customer data may be exposed, stop and contact the security team before you do anything else. Their handbook has the incident process.

## Handing over
@id: handing-over

At the end of your week, write a short handover for the next person: open issues, noisy alerts, and anything you'd like fixed.

@team-contact {channel=team-sre}: sre
